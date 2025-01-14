use std::{
    collections::BTreeMap,
    ffi::{c_char, CStr, CString},
    mem,
    path::{Path, PathBuf},
    ptr,
    sync::OnceLock,
};

use seda_runtime_sdk::{ExitInfo, VmType, WasmId};
use seda_wasm_vm::{
    init_logger,
    start_runtime,
    wasm_cache::wasm_cache_id,
    RuntimeContext,
    RuntimeError,
    VmCallData,
    VmResult,
};

use crate::errors::Result;

mod errors;

#[derive(Debug)]
#[repr(C)]
pub struct FfiExitInfo {
    exit_message: *const c_char,
    exit_code:    i32,
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_ffi_exit_info(exit_info: *mut FfiExitInfo) {
    if !(*exit_info).exit_message.is_null() {
        let _ = CString::from_raw((*exit_info).exit_message as *mut c_char);
        (*exit_info).exit_message = std::ptr::null();
    }
}

impl From<ExitInfo> for FfiExitInfo {
    fn from(exit_info: ExitInfo) -> Self {
        FfiExitInfo {
            exit_message: CString::new(exit_info.exit_message).unwrap().into_raw(),
            exit_code:    exit_info.exit_code,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FfiVmResult {
    stdout_ptr: *const *const c_char,
    stdout_len: usize,
    stderr_ptr: *const *const c_char,
    stderr_len: usize,
    result_ptr: *const u8,
    result_len: usize,
    exit_info:  FfiExitInfo,
    gas_used:   u64,
}

impl FfiVmResult {
    fn from_result(vm_result: VmResult, max_result_bytes: usize) -> Self {
        let stdout: Vec<CString> = vm_result
            .stdout
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();
        let stdout_storage: Vec<*const c_char> = stdout.into_iter().map(|s| s.into_raw() as *const _).collect();
        let boxed_slice: Box<[*const c_char]> = stdout_storage.into_boxed_slice();
        let stdout_ptr = boxed_slice.as_ptr();
        let stdout_len = boxed_slice.len();
        mem::forget(boxed_slice);

        let stderr: Vec<CString> = vm_result
            .stderr
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();
        let stderr_storage: Vec<*const c_char> = stderr.into_iter().map(|s| s.into_raw() as *const _).collect();
        let boxed_slice: Box<[*const c_char]> = stderr_storage.into_boxed_slice();
        let stderr_ptr = boxed_slice.as_ptr();
        let stderr_len = boxed_slice.len();
        mem::forget(boxed_slice);

        let result = vm_result.result.unwrap_or_default().into_boxed_slice();
        let result_ptr = result.as_ptr();
        let result_len = result.len();
        mem::forget(result);

        if result_len > max_result_bytes {
            FfiVmResult {
                exit_info: FfiExitInfo {
                    exit_message: CString::new(format!("Result larger than {max_result_bytes}bytes."))
                        .unwrap()
                        .into_raw(),
                    exit_code:    255,
                },
                result_ptr: ptr::null(),
                result_len,
                gas_used: vm_result.gas_used,
                stdout_ptr,
                stdout_len,
                stderr_ptr,
                stderr_len,
            }
        } else {
            FfiVmResult {
                stdout_ptr,
                stdout_len,
                stderr_ptr,
                stderr_len,
                result_ptr,
                result_len,
                exit_info: vm_result.exit_info.into(),
                gas_used: vm_result.gas_used,
            }
        }
    }
}
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_ffi_vm_result(vm_result: *mut FfiVmResult) {
    if !(*vm_result).stdout_ptr.is_null() {
        let stdout = Vec::from_raw_parts(
            (*vm_result).stdout_ptr as *mut _,
            (*vm_result).stdout_len,
            (*vm_result).stdout_len,
        );

        for elem in stdout {
            let s = CString::from_raw(elem);
            mem::drop(s);
        }
    }

    if !(*vm_result).stderr_ptr.is_null() {
        let stderr = Vec::from_raw_parts(
            (*vm_result).stderr_ptr as *mut _,
            (*vm_result).stderr_len,
            (*vm_result).stderr_len,
        );

        for elem in stderr {
            let s = CString::from_raw(elem);
            mem::drop(s);
        }
    }

    if !(*vm_result).result_ptr.is_null() {
        let result = Vec::from_raw_parts(
            (*vm_result).result_ptr as *mut u8,
            (*vm_result).result_len,
            (*vm_result).result_len,
        );
        mem::drop(result);
    }

    free_ffi_exit_info(&mut (*vm_result).exit_info);
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn execute_tally_vm(
    sedad_home: *const c_char,
    wasm_bytes: *const u8,
    wasm_bytes_len: usize,
    args_ptr: *const *const c_char,
    args_count: usize,
    env_keys_ptr: *const *const c_char,
    env_values_ptr: *const *const c_char,
    env_count: usize,
    max_result_bytes: usize,
) -> FfiVmResult {
    static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();
    let sedad_home = CStr::from_ptr(sedad_home).to_string_lossy().into_owned();
    let sedad_home = PathBuf::from(sedad_home);
    let _guard = LOG_GUARD.get_or_init(|| init_logger(&sedad_home));

    let wasm_bytes = std::slice::from_raw_parts(wasm_bytes, wasm_bytes_len).to_vec();

    let args: Vec<String> = (0..args_count)
        .map(|i| {
            let ptr = *args_ptr.add(i);
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        })
        .collect();

    let mut envs = BTreeMap::new();
    for i in 0..env_count {
        let key_ptr = *env_keys_ptr.add(i);
        let value_ptr = *env_values_ptr.add(i);

        let key = CStr::from_ptr(key_ptr).to_string_lossy().into_owned();
        let value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();

        envs.insert(key, value);
    }

    match _execute_tally_vm(&sedad_home, wasm_bytes, args, envs) {
        Ok(vm_result) => FfiVmResult::from_result(vm_result, max_result_bytes),
        // TODO: maybe we should consider exiting the process since its a vm error, not a user error?
        // Not sure how that would work with the ffi though
        Err(e) => FfiVmResult {
            stdout_ptr: ptr::null(),
            stdout_len: 0,
            stderr_ptr: ptr::null(),
            stderr_len: 0,
            result_ptr: ptr::null(),
            result_len: 0,
            exit_info:  FfiExitInfo {
                exit_message: CString::new(format!("VM Error: {e}")).unwrap().into_raw(),
                exit_code:    e.exit_code(),
            },
            gas_used:   0,
        },
    }
}

const DEFAULT_GAS_LIMIT_ENV_VAR: &str = "DR_TALLY_GAS_LIMIT";

fn _execute_tally_vm(
    sedad_home: &Path,
    wasm_bytes: Vec<u8>,
    args: Vec<String>,
    envs: BTreeMap<String, String>,
) -> Result<VmResult> {
    tracing::info!("Executing Tally VM");
    let wasm_hash = wasm_cache_id(&wasm_bytes);
    let env_vars = envs.clone();
    let gas_limit = env_vars
        .get(DEFAULT_GAS_LIMIT_ENV_VAR)
        .ok_or(RuntimeError::NodeError(format!(
            "{DEFAULT_GAS_LIMIT_ENV_VAR} is required to be set as an env variable"
        )))?;

    let call_data = VmCallData {
        call_id: None,
        wasm_id: WasmId::Bytes(wasm_bytes),
        args,
        envs,
        program_name: wasm_hash,
        start_func: None,
        vm_type: VmType::Tally,
        gas_limit: Some(gas_limit.parse::<u64>()?),
    };

    let runtime_context = RuntimeContext::new(sedad_home, &call_data)?;
    let result = start_runtime(call_data, runtime_context);

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::{
        collections::BTreeMap,
        ffi::{c_char, CString},
    };

    use crate::{_execute_tally_vm, DEFAULT_GAS_LIMIT_ENV_VAR};

    #[test]
    fn execute_tally_vm() {
        let wasm_bytes = include_bytes!("../../integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        // VM_MODE dr to force the http_fetch path
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let tempdir = std::env::temp_dir();
        let result = _execute_tally_vm(
            &tempdir,
            wasm_bytes.to_vec(),
            vec![hex::encode("testHttpSuccess")],
            envs,
        )
        .unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            result.exit_info.exit_message,
            "http_fetch is not allowed in tally".to_string()
        );
        assert_eq!(result.gas_used, 5001589256245);
    }

    #[test]
    fn execute_c_tally_vm() {
        let wasm_bytes = include_bytes!("../../integration-test.wasm");

        let args = [hex::encode("testHttpSuccess")];
        let arg_cstrings: Vec<CString> = args
            .iter()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let arg_ptrs: Vec<*const c_char> = arg_cstrings.iter().map(|s| s.as_ptr()).collect();

        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        // VM_MODE dr to force the http_fetch path
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());
        let env_key_cstrings: Vec<CString> = envs
            .keys()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let env_key_ptrs: Vec<*const c_char> = env_key_cstrings.iter().map(|s| s.as_ptr()).collect();
        let env_value_cstrings: Vec<CString> = envs
            .values()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let env_value_ptrs: Vec<*const c_char> = env_value_cstrings.iter().map(|s| s.as_ptr()).collect();

        let tempdir = std::env::temp_dir().display().to_string();
        let mut result = unsafe {
            super::execute_tally_vm(
                CString::new(tempdir).unwrap().into_raw(),
                wasm_bytes.as_ptr(),
                wasm_bytes.len(),
                arg_ptrs.as_ptr(),
                args.len(),
                env_key_ptrs.as_ptr(),
                env_value_ptrs.as_ptr(),
                envs.len(),
                1024,
            )
        };

        unsafe {
            assert_eq!(
                std::ffi::CStr::from_ptr(result.exit_info.exit_message)
                    .to_string_lossy()
                    .into_owned(),
                "http_fetch is not allowed in tally".to_string()
            );
        }
        assert_eq!(result.gas_used, 5001589256245);

        unsafe {
            super::free_ffi_vm_result(&mut result);
        }
    }

    #[test]
    fn execute_c_tally_vm_exceeds_byte_limit() {
        let wasm_bytes = include_bytes!("../../tally.wasm");

        let args = [hex::encode("tally")];
        let arg_cstrings: Vec<CString> = args
            .iter()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let arg_ptrs: Vec<*const c_char> = arg_cstrings.iter().map(|s| s.as_ptr()).collect();

        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        // VM_MODE dr to force the http_fetch path
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());
        let env_key_cstrings: Vec<CString> = envs
            .keys()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let env_key_ptrs: Vec<*const c_char> = env_key_cstrings.iter().map(|s| s.as_ptr()).collect();
        let env_value_cstrings: Vec<CString> = envs
            .values()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let env_value_ptrs: Vec<*const c_char> = env_value_cstrings.iter().map(|s| s.as_ptr()).collect();

        let tempdir = std::env::temp_dir().display().to_string();
        let mut result = unsafe {
            super::execute_tally_vm(
                CString::new(tempdir).unwrap().into_raw(),
                wasm_bytes.as_ptr(),
                wasm_bytes.len(),
                arg_ptrs.as_ptr(),
                args.len(),
                env_key_ptrs.as_ptr(),
                env_value_ptrs.as_ptr(),
                envs.len(),
                1,
            )
        };

        unsafe {
            assert_eq!(
                std::ffi::CStr::from_ptr(result.exit_info.exit_message)
                    .to_string_lossy()
                    .into_owned(),
                "Result larger than 1bytes.".to_string()
            );
        }
        assert_eq!(result.exit_info.exit_code, 255);
        assert_eq!(result.gas_used, 5000125072945);

        unsafe {
            super::free_ffi_vm_result(&mut result);
        }
    }

    #[test]
    fn execute_tally_vm_proxy_http_fetch() {
        let wasm_bytes = include_bytes!("../../integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let tempdir = std::env::temp_dir();
        let result = _execute_tally_vm(
            &tempdir,
            wasm_bytes.to_vec(),
            vec![hex::encode("testProxyHttpFetch")],
            envs,
        )
        .unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            result.exit_info.exit_message,
            "proxy_http_fetch is not allowed in tally".to_string()
        );
        assert_eq!(result.gas_used, 5005670298145);
    }

    #[test]
    fn execute_tally_vm_no_args() {
        let wasm_bytes = include_bytes!("../../tally.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let tempdir = std::env::temp_dir();
        let result = _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![], envs).unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));
        assert_eq!(result.gas_used, 5000004190715);
    }

    #[test]
    fn execute_tally_vm_with_low_gas_limit() {
        let wasm_bytes = include_bytes!("../../integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "1000".to_string());

        let tempdir = std::env::temp_dir();
        let result = _execute_tally_vm(
            &tempdir,
            wasm_bytes.to_vec(),
            vec![hex::encode("testHttpSuccess")],
            envs,
        )
        .unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(result.exit_info.exit_code, 250);
        assert_eq!(result.gas_used, 1000);
    }

    #[test]
    fn execute_tally_keccak256() {
        let wasm_bytes = include_bytes!("../../integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let tempdir = std::env::temp_dir();
        let result =
            _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![hex::encode("testKeccak256")], envs).unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            String::from_utf8(result.result.unwrap()).unwrap(),
            // "testKeccak256" hashed
            "fe8baa653979909c621153b53c973bab3832768b5e77896a5b5944d20d48c7a6"
        );
        assert_eq!(result.gas_used, 5000710408730);
    }
}
