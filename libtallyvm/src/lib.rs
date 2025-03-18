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
    fn from_result(vm_result: VmResult, max_result_bytes: usize, is_tally: bool) -> Self {
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

        if is_tally && result_len > max_result_bytes {
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
    let is_tally = if let Some(mode) = envs.get("VM_MODE") {
        mode == "tally"
    } else {
        // TODO should we default or return an error?
        false
    };

    match _execute_tally_vm(&sedad_home, wasm_bytes, args, envs) {
        Ok(vm_result) => FfiVmResult::from_result(vm_result, max_result_bytes, is_tally),
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
        ..Default::default()
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

    use seda_runtime_sdk::ToBytes;

    use crate::{_execute_tally_vm, DEFAULT_GAS_LIMIT_ENV_VAR};

    #[test]
    fn execute_tally_vm() {
        let wasm_bytes = include_bytes!("../../integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        // VM_MODE dr to force the http_fetch path
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());

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
            String::from_utf8_lossy(&result.result.unwrap()),
            "http_fetch is not allowed in tally".to_string()
        );
        assert_eq!(result.gas_used, 20566535451250);
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
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());
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
                std::ffi::CStr::from_ptr(result.result_ptr as *const c_char)
                    .to_string_lossy()
                    .into_owned(),
                "http_fetch is not allowed in tally".to_string()
            );
        }
        assert_eq!(result.gas_used, 20566535451250);

        unsafe {
            super::free_ffi_vm_result(&mut result);
        }
    }

    #[test]
    fn execute_c_tally_vm_exceeds_byte_limit() {
        let wasm_bytes = include_bytes!("../../tally.wasm");

        let args = [hex::encode("tally"), hex::encode("[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"), hex::encode("[0,0,0]")];
        let arg_cstrings: Vec<CString> = args
            .iter()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let arg_ptrs: Vec<*const c_char> = arg_cstrings.iter().map(|s| s.as_ptr()).collect();

        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());
        envs.insert("CONSENSUS".to_string(), true.to_string());
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
        assert_eq!(result.gas_used, 29473111092500);

        unsafe {
            super::free_ffi_vm_result(&mut result);
        }
    }

    #[test]
    fn execute_c_tally_vm_exceeds_byte_limit_does_not_matter_for_dr_mode() {
        let wasm_bytes = include_bytes!("../../tally.wasm");

        let args = [hex::encode("tally")];
        let arg_cstrings: Vec<CString> = args
            .iter()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let arg_ptrs: Vec<*const c_char> = arg_cstrings.iter().map(|s| s.as_ptr()).collect();

        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());
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
                "Ok".to_string()
            );
        }
        assert_eq!(result.exit_info.exit_code, 0);
        assert_eq!(result.gas_used, 9114698646250);

        unsafe {
            super::free_ffi_vm_result(&mut result);
        }
    }

    #[test]
    fn execute_tally_vm_proxy_http_fetch() {
        let wasm_bytes = include_bytes!("../../integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());

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
            String::from_utf8_lossy(&result.result.unwrap()),
            "proxy_http_fetch is not allowed in tally".to_string()
        );
        assert_eq!(result.gas_used, 23111707163750);
    }

    #[test]
    fn execute_tally_vm_no_args() {
        let wasm_bytes = include_bytes!("../../tally.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());

        let tempdir = std::env::temp_dir();
        let result = _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![], envs).unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));
        assert_eq!(result.gas_used, 10096086678750);
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
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());

        let tempdir = std::env::temp_dir();
        let result =
            _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![hex::encode("testKeccak256")], envs).unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            String::from_utf8(result.result.unwrap()).unwrap(),
            // "testKeccak256" hashed
            "fe8baa653979909c621153b53c973bab3832768b5e77896a5b5944d20d48c7a6"
        );
        assert_eq!(result.gas_used, 11564472550000);
    }

    #[test]
    fn simple_price_feed() {
        let wasm_bytes = include_bytes!("../../simplePriceFeed.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let tempdir = std::env::temp_dir();
        let result = _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![hex::encode("btc-usdc")], envs).unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            String::from_utf8_lossy(&result.result.unwrap()),
            "Error while fetching price feed".to_string()
        );
    }

    #[test]
    fn polyfill_does_not_crash_vm() {
        let wasm_bytes = include_bytes!("../../randomNumber.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let tempdir = std::env::temp_dir().join("foo");
        std::fs::create_dir_all(&tempdir).unwrap();
        let result = _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![], envs).unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(result.exit_info.exit_code, 252);
        assert_eq!(result.exit_info.exit_message, "Not ok".to_string());
    }

    #[test]
    fn userland_non_zero_exit_code() {
        let wasm_bytes = include_bytes!("../../null_byte_string.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert("DR_REPLICATION_FACTOR".to_string(), "1".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let tempdir = std::env::temp_dir().join("foo");
        std::fs::create_dir_all(&tempdir).unwrap();
        let result = _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![
					"0xd66196506df89851d1200962310cc4bd5ee7b4d19c852a4afd0ccf07e636606f".to_string(),
					"[{\"reveal\":[123,34,98,108,111,99,107,72,97,115,104,34,58,34 ,48,120,57,50,55,55,98,53,53,55,48,48,100,97,57,48,53,48,98,53,53,97,97,54,55,52,48,55,49,57,101,50,53,98,48,48,102,51,57,97,99,99,49,53,102,49,49,98,54,52,48,99,98,56,50,101,52,48,100,97,56,102,56,54,48,100,34,44,34,98,108,111,99,107,78,117,109,98,101,114,34,58,34,48,120,49,52,50,98,98,55,56,34,44,34,102,114,111,109, 34,58,34,48,120,99,48,100,98,98,53,49,101,54,48,55,102,52,57,53,54,57,99,52,50,99,53,99,101,101,50,101,98,51,51,100,99,53,98,97,99,50,56,100,53,34,125],\"salt\":[211,175,124,217,173,184,107,223,93,111,189,56,113,215,248,115,214,157,229,183,30,213,237,186,209,254,246,247,222,155,241,183,157,123,93,180,213,253,57,211,19 0,56,125,189,120,247,93,116],\"id\":\"f495c06137a92787312086267884196ec4476f6faf4bd074eafb289b65de272f\",\"exit_code\":0,\"gas_used\":42369302985625,\"proxy_public_keys\":[]}]".to_string(),
					"[0]".to_string()
					], envs).unwrap();

        assert_eq!(result.exit_info.exit_code, 1);
        assert_eq!(result.exit_info.exit_message, "Not ok".to_string());
    }

    #[test]
    fn assign_too_much_memory() {
        let wasm_bytes = include_bytes!("../../assign_too_much_memory.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert("DR_REPLICATION_FACTOR".to_string(), "1".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let tempdir = std::env::temp_dir().join("foo");
        std::fs::create_dir_all(&tempdir).unwrap();
        let result = _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![
					"0xd66196506df89851d1200962310cc4bd5ee7b4d19c852a4afd0ccf07e636606f".to_string(),
					"[{\"reveal\":[123,34,98,108,111,99,107,72,97,115,104,34,58,34 ,48,120,57,50,55,55,98,53,53,55,48,48,100,97,57,48,53,48,98,53,53,97,97,54,55,52,48,55,49,57,101,50,53,98,48,48,102,51,57,97,99,99,49,53,102,49,49,98,54,52,48,99,98,56,50,101,52,48,100,97,56,102,56,54,48,100,34,44,34,98,108,111,99,107,78,117,109,98,101,114,34,58,34,48,120,49,52,50,98,98,55,56,34,44,34,102,114,111,109, 34,58,34,48,120,99,48,100,98,98,53,49,101,54,48,55,102,52,57,53,54,57,99,52,50,99,53,99,101,101,50,101,98,51,51,100,99,53,98,97,99,50,56,100,53,34,125],\"salt\":[211,175,124,217,173,184,107,223,93,111,189,56,113,215,248,115,214,157,229,183,30,213,237,186,209,254,246,247,222,155,241,183,157,123,93,180,213,253,57,211,19 0,56,125,189,120,247,93,116],\"id\":\"f495c06137a92787312086267884196ec4476f6faf4bd074eafb289b65de272f\",\"exit_code\":0,\"gas_used\":42369302985625,\"proxy_public_keys\":[]}]".to_string(),
					"[0]".to_string()
					], envs).unwrap();

        assert_eq!(result.exit_info.exit_code, 4);
        assert_eq!(result.exit_info.exit_message, "Error: Failed to create WASMER instance: Insufficient resources: Failed to create memory: A user-defined error occurred: Minimum exceeds the allowed memory limit".to_string());
    }

    #[test]
    fn import_length_overflow() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasip1/debug/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string()); // 50 tGas

        let tempdir = std::env::temp_dir().join("foo");
        std::fs::create_dir_all(&tempdir).unwrap();

        let method = "import_length_overflow".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let result = _execute_tally_vm(&tempdir, wasm_bytes.to_vec(), vec![method_hex], envs).unwrap();

        assert_eq!(result.stderr[0], "Runtime error: Out of gas");
    }
    #[test]
    fn price_feed_tally() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasip1/debug/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let tempdir = std::env::temp_dir().join("foo");
        std::fs::create_dir_all(&tempdir).unwrap();

        let method = "price_feed_tally".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let reveals = "[{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":200,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,50,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":198,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,52,53,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":201,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,50,56,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":199,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,55,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":202,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,48,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":197,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,52,49,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":200,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,53,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":203,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,57,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":196,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,51,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":201,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,54,125]}]".to_string();
        let consensus = "[0,0,0,0,0,0,0,0,0,0]".to_string();

        let result = _execute_tally_vm(
            &tempdir,
            wasm_bytes.to_vec(),
            vec![method_hex, reveals, consensus],
            envs,
        )
        .unwrap();
        assert_eq!(result.gas_used, 14549853896250);
    }
}
