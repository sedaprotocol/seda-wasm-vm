use std::{
    collections::BTreeMap,
    ffi::{c_char, CStr, CString},
    mem,
    path::{Path, PathBuf},
    ptr,
    sync::OnceLock,
};

use seda_wasm_vm::{
    init_logger,
    start_runtime,
    vm::{ExitInfo, VmCallData, VmResult, VmType, WasmId},
    RuntimeContext,
    RuntimeError,
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

        if is_tally && result_len > max_result_bytes {
            drop(result);
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
            mem::forget(result);
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
    stdout_limit: usize,
    stderr_limit: usize,
) -> FfiVmResult {
    let result = std::panic::catch_unwind(|| {
        #[cfg(test)]
        {
            let should_panic = std::env::var("_GIBBERISH_CHECK_TO_PANIC").unwrap_or_default();
            if should_panic == "true" {
                panic!("Panic for testing");
            }
        }

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

        let is_tally = envs.get("VM_MODE").is_some_and(|mode| mode == "tally");
        (
            _execute_tally_vm(&sedad_home, wasm_bytes, args, envs, stdout_limit, stderr_limit),
            is_tally,
        )
    });

    match result {
        Ok((Ok(vm_result), is_tally)) => FfiVmResult::from_result(vm_result, max_result_bytes, is_tally),
        Ok((Err(e), _)) => FfiVmResult {
            stdout_ptr: std::ptr::null(),
            stdout_len: 0,
            stderr_ptr: std::ptr::null(),
            stderr_len: 0,
            result_ptr: std::ptr::null(),
            result_len: 0,
            exit_info:  FfiExitInfo {
                exit_message: CString::new(format!("VM Error: {e}")).unwrap().into_raw(),
                exit_code:    e.exit_code(),
            },
            gas_used:   0,
        },

        Err(e) => FfiVmResult {
            stdout_ptr: std::ptr::null(),
            stdout_len: 0,
            stderr_ptr: std::ptr::null(),
            stderr_len: 0,
            result_ptr: std::ptr::null(),
            result_len: 0,
            exit_info:  FfiExitInfo {
                exit_message: CString::new(format!(
                    "The tally VM panicked.\n\
                     Please report this issue at: \
                     https://github.com/sedaprotocol/seda-wasm-vm/issues.\n\
                     Panic Error:\n{e:?}"
                ))
                .unwrap()
                .into_raw(),

                exit_code: 42,
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
    stdout_limit: usize,
    stderr_limit: usize,
) -> Result<VmResult> {
    tracing::info!("Executing Tally VM");
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
        // program_name is not used in the SEDA SDK (It refers in CLI to the first argument)
        // Better to hardcode it to something fast than the binary id.
        program_name: "data-request".to_string(),
        start_func: None,
        vm_type: VmType::Tally,
        gas_limit: Some(gas_limit.parse::<u64>()?),
        ..Default::default()
    };

    let runtime_context = RuntimeContext::new(sedad_home, &call_data)?;
    let result = start_runtime(call_data, runtime_context, stdout_limit, stderr_limit);

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::{
        collections::BTreeMap,
        ffi::{c_char, CStr, CString},
        mem,
    };

    use seda_sdk_rs::bytes::ToBytes;
    use tempdir::TempDir;

    use crate::{_execute_tally_vm, DEFAULT_GAS_LIMIT_ENV_VAR};

    #[test]
    fn can_get_runtime_versions() {
        assert_eq!(seda_wasm_vm::WASMER_VERSION, "5.0.4");
        assert_eq!(seda_wasm_vm::WASMER_TYPES_VERSION, "5.0.4");
        assert_eq!(seda_wasm_vm::WASMER_MIDDLEWARES_VERSION, "2.5.0");
        assert_eq!(seda_wasm_vm::WASMER_WASIX_VERSION, "0.34.0");
    }

    #[test]
    fn cache_works() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("CONSENSUS".to_string(), "true".to_string());
        envs.insert("VM_MODE".to_string(), "tally".to_string());

        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());
        envs.insert("DR_REPLICATION_FACTOR".to_string(), "1".to_string());

        let method = "infinite_loop_wasi".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("cache_works").unwrap();
        let tempdir = temp_dir.path();

        let now = std::time::Instant::now();
        let _result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![method_hex.clone()],
            envs.clone(),
            1024,
            1024,
        )
        .unwrap();
        let first_run = now.elapsed();
        println!("First run took: {:?}", first_run);

        let now = std::time::Instant::now();
        let _result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();
        let second_run = now.elapsed();
        println!("Second run took: {:?}", second_run);

        // second run should be faster than first run
        assert!(second_run < first_run);
    }

    #[test]
    fn timing_cache_invalidates_on_new_version() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("CONSENSUS".to_string(), "true".to_string());
        envs.insert("VM_MODE".to_string(), "tally".to_string());

        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());
        envs.insert("DR_REPLICATION_FACTOR".to_string(), "1".to_string());

        let method = "infinite_loop_wasi".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("cache_invalidates_on_new_version").unwrap();
        let tempdir = temp_dir.path();

        seda_wasm_vm::set_test_version_file_name("1.0.0");
        let now = std::time::Instant::now();
        let _result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![method_hex.clone()],
            envs.clone(),
            1024,
            1024,
        )
        .unwrap();
        let first_run = now.elapsed();
        println!("First run took: {:?}", first_run);

        seda_wasm_vm::set_test_version_file_name("1.0.1");
        let now = std::time::Instant::now();
        let _result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();
        let second_run = now.elapsed();
        println!("Second run took: {:?}", second_run);

        // second run should be about the same as the first run
        let diff = if first_run > second_run {
            first_run - second_run
        } else {
            second_run - first_run
        };
        println!("Diff: {}ms", diff.as_millis());
        // Allow a 50% difference, as the first run might be slower due to cache
        // warmup, but the second run should be about the same.
        // Use relative difference for robust comparison.
        let max_run = std::cmp::max(first_run, second_run);
        assert!(
            diff.as_secs_f64() / max_run.as_secs_f64() < 0.5,
            "Difference is more than 50%"
        );
    }

    #[test]
    fn execute_tally_vm() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        // VM_MODE dr to force the http_fetch path
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());

        let temp_dir = TempDir::new("execute_tally_vm").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![hex::encode("testHttpSuccess")],
            envs,
            1024,
            1024,
        )
        .unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            String::from_utf8_lossy(&result.result.unwrap()),
            "http_fetch is not allowed in tally".to_string()
        );
        assert_eq!(result.gas_used, 19287742795000);
    }

    #[test]
    fn execute_c_tally_vm() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/integration-test.wasm");

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

        let temp_dir = TempDir::new("execute_c_tally_vm").unwrap();
        let tempdir = temp_dir.path().display().to_string();
        let tempdir_craw = CString::new(tempdir).unwrap().into_raw();
        let mut result = unsafe {
            super::execute_tally_vm(
                tempdir_craw,
                wasm_bytes.as_ptr(),
                wasm_bytes.len(),
                arg_ptrs.as_ptr(),
                args.len(),
                env_key_ptrs.as_ptr(),
                env_value_ptrs.as_ptr(),
                envs.len(),
                1024,
                1024,
                1024,
            )
        };

        let result_msg = unsafe {
            CStr::from_ptr(result.result_ptr as *const c_char)
                .to_string_lossy()
                .into_owned()
        };

        assert_eq!(result_msg, "http_fetch is not allowed in tally".to_string());
        assert_eq!(result.gas_used, 19287742795000);

        unsafe {
            super::free_ffi_vm_result(&mut result);
            let tempdir_c = CString::from_raw(tempdir_craw);
            mem::drop(tempdir_c);
        }
    }

    #[test]
    fn execute_c_tally_vm_exceeds_byte_limit() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/tally.wasm");

        let args = [hex::encode("tally"), hex::encode("[{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"), hex::encode("[0,0,0]")];
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

        let temp_dir = TempDir::new("execute_c_tally_vm_exceeds_byte_limit").unwrap();
        let tempdir = temp_dir.path().display().to_string();
        let tempdir_craw = CString::new(tempdir).unwrap().into_raw();
        let mut result = unsafe {
            super::execute_tally_vm(
                tempdir_craw,
                wasm_bytes.as_ptr(),
                wasm_bytes.len(),
                arg_ptrs.as_ptr(),
                args.len(),
                env_key_ptrs.as_ptr(),
                env_value_ptrs.as_ptr(),
                envs.len(),
                1,
                1024,
                1024,
            )
        };

        let exit_msg = unsafe {
            CStr::from_ptr(result.exit_info.exit_message)
                .to_string_lossy()
                .into_owned()
        };

        assert_eq!(exit_msg, "Result larger than 1bytes.".to_string());
        assert_eq!(result.exit_info.exit_code, 255);
        assert_eq!(result.gas_used, 29703554900000);

        unsafe {
            super::free_ffi_vm_result(&mut result);
            let tempdir_c = CString::from_raw(tempdir_craw);
            mem::drop(tempdir_c);
        }
    }

    #[test]
    fn execute_c_tally_vm_exceeds_byte_limit_does_not_matter_for_dr_mode() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/tally.wasm");

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

        let temp_dir = TempDir::new("execute_c_tally_vm_exceeds_byte_limit_does_not_matter_for_dr_mode").unwrap();
        let tempdir = temp_dir.path().display().to_string();
        let tempdir_craw = CString::new(tempdir).unwrap().into_raw();
        let mut result = unsafe {
            super::execute_tally_vm(
                tempdir_craw,
                wasm_bytes.as_ptr(),
                wasm_bytes.len(),
                arg_ptrs.as_ptr(),
                args.len(),
                env_key_ptrs.as_ptr(),
                env_value_ptrs.as_ptr(),
                envs.len(),
                1,
                1024,
                1024,
            )
        };

        let exit_msg = unsafe {
            CStr::from_ptr(result.exit_info.exit_message)
                .to_string_lossy()
                .into_owned()
        };
        assert_eq!(exit_msg, "Ok".to_string());
        assert_eq!(result.exit_info.exit_code, 0);
        assert_eq!(result.gas_used, 9156653346250);

        unsafe {
            super::free_ffi_vm_result(&mut result);
            let tempdir_c = CString::from_raw(tempdir_craw);
            mem::drop(tempdir_c);
        }
    }

    #[test]
    fn execute_tally_vm_proxy_http_fetch() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());

        let temp_dir = TempDir::new("execute_tally_vm_proxy_http_fetch").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![hex::encode("testProxyHttpFetch")],
            envs,
            1024,
            1024,
        )
        .unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            String::from_utf8_lossy(&result.result.unwrap()),
            "proxy_http_fetch is not allowed in tally".to_string()
        );
        assert_eq!(result.gas_used, 21736902545000);
    }

    #[test]
    fn execute_tally_vm_no_args() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/tally.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());

        let temp_dir = TempDir::new("execute_tally_vm_no_args").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![], envs, 1024, 1024).unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));
        assert_eq!(result.gas_used, 10124565078750);
    }

    #[test]
    fn execute_tally_vm_with_low_gas_limit() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        // enough to cover startup cost + some
        let method_hex = hex::encode("testHttpSuccess");
        let startup_gas = (method_hex.len() as u64 * 10_000) + (1_000_000_000_000 * 5);
        let total_gas = startup_gas + 1_000;
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), total_gas.to_string());

        let temp_dir = TempDir::new("execute_tally_vm_with_low_gas_limit").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();

        assert_eq!(result.exit_info.exit_code, 250);
        assert_eq!(result.gas_used, total_gas);
    }

    #[test]
    fn vm_does_not_run_if_startup_cost_is_higher_than_gas_limit() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        // enough to cover startup cost + some
        let method_hex = hex::encode("testHttpSuccess");
        let startup_gas = (method_hex.len() as u64 * 10_000) + (1_000_000_000_000 * 5);
        let total_gas = startup_gas - 1_000;
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), total_gas.to_string());

        let temp_dir = TempDir::new("vm_does_not_run_if_startup_cost_is_higher_than_gas_limit").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();

        assert_eq!(result.exit_info.exit_code, 14);
        assert!(result.gas_used > 0);
    }

    #[test]
    fn execute_tally_keccak256() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/integration-test.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "150000000000000".to_string());

        let temp_dir = TempDir::new("execute_tally_keccak256").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![hex::encode("testKeccak256")],
            envs,
            1024,
            1024,
        )
        .unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            String::from_utf8(result.result.unwrap()).unwrap(),
            // "testKeccak256" hashed
            "fe8baa653979909c621153b53c973bab3832768b5e77896a5b5944d20d48c7a6"
        );
        assert_eq!(result.gas_used, 11250594475000);
    }

    #[test]
    fn simple_price_feed() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/simplePriceFeed.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let temp_dir = TempDir::new("simple_price_feed").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![hex::encode("btc-usdc")],
            envs,
            1024,
            1024,
        )
        .unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(
            String::from_utf8_lossy(&result.result.unwrap()),
            "Error while fetching price feed".to_string()
        );
        assert!(result.gas_used > 0);
    }

    #[test]
    fn polyfill_does_not_crash_vm() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/randomNumber.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "dr".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let temp_dir = TempDir::new("polyfill_does_not_crash_vm").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![], envs, 1024, 1024).unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(result.exit_info.exit_code, 252);
        assert_eq!(result.exit_info.exit_message, "Not ok".to_string());
        assert!(result.gas_used > 0);
    }

    #[test]
    fn userland_non_zero_exit_code() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/null_byte_string.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert("DR_REPLICATION_FACTOR".to_string(), "1".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let temp_dir = TempDir::new("userland_non_zero_exit_code").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![
                "0xd66196506df89851d1200962310cc4bd5ee7b4d19c852a4afd0ccf07e636606f".to_string(),
                "[{\"reveal\":[123,34,98,108,111,99,107,72,97,115,104,34,58,34 ,48,120,57,50,55,55,98,53,53,55,48,48,100,97,57,48,53,48,98,53,53,97,97,54,55,52,48,55,49,57,101,50,53,98,48,48,102,51,57,97,99,99,49,53,102,49,49,98,54,52,48,99,98,56,50,101,52,48,100,97,56,102,56,54,48,100,34,44,34,98,108,111,99,107,78,117,109,98,101,114,34,58,34,48,120,49,52,50,98,98,55,56,34,44,34,102,114,111,109, 34,58,34,48,120,99,48,100,98,98,53,49,101,54,48,55,102,52,57,53,54,57,99,52,50,99,53,99,101,101,50,101,98,51,51,100,99,53,98,97,99,50,56,100,53,34,125],\"salt\":[211,175,124,217,173,184,107,223,93,111,189,56,113,215,248,115,214,157,229,183,30,213,237,186,209,254,246,247,222,155,241,183,157,123,93,180,213,253,57,211,19 0,56,125,189,120,247,93,116],\"id\":\"f495c06137a92787312086267884196ec4476f6faf4bd074eafb289b65de272f\",\"exit_code\":0,\"gas_used\":42369302985625,\"proxy_public_keys\":[]}]".to_string(),
                "[0]".to_string()
            ],
            envs,
            1024,
            1024
        ).unwrap();

        assert_eq!(result.exit_info.exit_code, 1);
        assert_eq!(result.exit_info.exit_message, "Not ok".to_string());
        assert!(result.gas_used > 0);
    }

    #[test]
    fn assign_too_much_memory() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/assign_too_much_memory.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert("DR_REPLICATION_FACTOR".to_string(), "1".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "300000000000000".to_string());

        let temp_dir = TempDir::new("assign_too_much_memory").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![
                "0xd66196506df89851d1200962310cc4bd5ee7b4d19c852a4afd0ccf07e636606f".to_string(),
                "[{\"reveal\":[123,34,98,108,111,99,107,72,97,115,104,34,58,34 ,48,120,57,50,55,55,98,53,53,55,48,48,100,97,57,48,53,48,98,53,53,97,97,54,55,52,48,55,49,57,101,50,53,98,48,48,102,51,57,97,99,99,49,53,102,49,49,98,54,52,48,99,98,56,50,101,52,48,100,97,56,102,56,54,48,100,34,44,34,98,108,111,99,107,78,117,109,98,101,114,34,58,34,48,120,49,52,50,98,98,55,56,34,44,34,102,114,111,109, 34,58,34,48,120,99,48,100,98,98,53,49,101,54,48,55,102,52,57,53,54,57,99,52,50,99,53,99,101,101,50,101,98,51,51,100,99,53,98,97,99,50,56,100,53,34,125],\"dr_block_height\":1,\"id\":\"f495c06137a92787312086267884196ec4476f6faf4bd074eafb289b65de272f\",\"exit_code\":0,\"gas_used\":42369302985625,\"proxy_public_keys\":[]}]".to_string(),
                "[0]".to_string()
            ],
            envs,
            1024,
            1024,
        ).unwrap();

        assert_eq!(result.exit_info.exit_code, 4);
        assert_eq!(result.exit_info.exit_message, "Error: Failed to create WASMER instance: Insufficient resources: Failed to create memory: A user-defined error occurred: Minimum exceeds the allowed memory limit".to_string());
    }

    #[test]
    fn import_length_overflow() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string()); // 50 tGas

        let temp_dir = TempDir::new("import_length_overflow").unwrap();
        let tempdir = temp_dir.path();

        let method = "import_length_overflow".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();

        assert_eq!(result.stderr[0], "Runtime error: Out of gas");
        assert!(result.gas_used > 0);
    }

    #[test]
    fn price_feed_tally() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let temp_dir = TempDir::new("price_feed_tally").unwrap();
        let tempdir = temp_dir.path();

        let method = "price_feed_tally".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let reveals = "[{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":200,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,50,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":198,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,52,53,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":201,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,50,56,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":199,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,55,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":202,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,48,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":197,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,52,49,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":200,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,53,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":203,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,57,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":196,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,51,125]},{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":201,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,54,125]}]".to_string();
        let consensus = "[0,0,0,0,0,0,0,0,0,0]".to_string();

        let result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![method_hex, reveals, consensus],
            envs,
            1024,
            1024,
        )
        .unwrap();
        assert_eq!(result.gas_used, 14103058802500);
    }

    #[test]
    fn call_result_write_len_0() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let method = "call_result_write_0".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("call_result_write_len_0").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();

        assert_eq!(result.exit_info.exit_code, 252);
        assert_eq!(result.exit_info.exit_message, "Not ok".to_string());
        assert_eq!(result.stderr.len(), 1);
        assert_eq!(result.stderr[0], "Runtime error: Invalid Memory Access: call_result_write: result_data_ptr length does not match call_value length");
        assert!(result.gas_used > 0);
    }

    #[test]
    fn execute_c_tally_vm_panic() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/integration-test.wasm");

        let args: [String; 0] = [];
        let arg_cstrings: Vec<CString> = args
            .iter()
            .cloned()
            .map(|s| CString::new(s).expect("CString::new failed"))
            .collect();
        let arg_ptrs: Vec<*const c_char> = arg_cstrings.iter().map(|s| s.as_ptr()).collect();

        let envs: BTreeMap<String, String> = BTreeMap::new();
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

        let temp_dir = TempDir::new("execute_c_tally_vm_panic").unwrap();
        let tempdir = temp_dir.path().display().to_string();
        let tempdir_craw = CString::new(tempdir).unwrap().into_raw();
        std::env::set_var("_GIBBERISH_CHECK_TO_PANIC", "true");
        let mut result = unsafe {
            super::execute_tally_vm(
                tempdir_craw,
                wasm_bytes.as_ptr(),
                wasm_bytes.len(),
                arg_ptrs.as_ptr(),
                args.len(),
                env_key_ptrs.as_ptr(),
                env_value_ptrs.as_ptr(),
                envs.len(),
                1024,
                1024,
                1024,
            )
        };
        std::env::remove_var("_GIBBERISH_CHECK_TO_PANIC");

        let exit_msg = unsafe {
            CStr::from_ptr(result.exit_info.exit_message)
                .to_string_lossy()
                .into_owned()
        };

        assert!(exit_msg.contains("The tally VM panicked."));
        assert_eq!(result.gas_used, 0);
        assert_eq!(result.exit_info.exit_code, 42);

        unsafe {
            super::free_ffi_vm_result(&mut result);
            let tempdir_c = CString::from_raw(tempdir_craw);
            mem::drop(tempdir_c);
        }
    }

    #[test]
    fn test_stdout_and_stderr_limit() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let method = "hello_world".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("test_stdout_and_stderr_limit").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 2, 2).unwrap();

        assert_eq!(result.exit_info.exit_code, 0);
        assert_eq!(result.stdout.len(), 1);
        // the full 4 bytes would be "Foo\n"
        assert_eq!(result.stdout[0], "Fo");
        assert_eq!(result.stderr.len(), 1);
        // the full 4 bytes would be "Bar\n"
        assert_eq!(result.stderr[0], "Ba");
    }

    #[test]
    fn test_long_stdout_and_stderr() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let method = "long_stdout_stderr".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("test_long_stdout_and_stderr").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();

        assert_eq!(result.exit_info.exit_code, 0);
        assert_eq!(result.stdout.len(), 1);
        assert_eq!(result.stdout[0].len(), 1024);
        assert_eq!(result.stdout[0], "Hello, World!\n".repeat(100)[..1024]);
        assert_eq!(result.stderr.len(), 1);
        assert_eq!(result.stderr[0].len(), 1024);
        assert_eq!(result.stderr[0], "I AM ERROR\n".repeat(100)[..1024]);
        assert!(result.gas_used > 0);
    }

    #[test]
    fn test_stdout_and_stderr_fail_when_given_non_utf8() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let temp_dir = TempDir::new("test_stdout_and_stderr_fail_when_given_non_utf8").unwrap();
        let tempdir = temp_dir.path();

        let method = "stderr_non_utf8".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let result =
            _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs.clone(), 1024, 1024).unwrap();
        assert_eq!(result.exit_info.exit_code, 8);
        assert_eq!(result.stderr.len(), 0);
        assert_eq!(
            &result.exit_info.exit_message,
            "Error: Failed to convert VM pipe `stderr` output to String"
        );
        assert!(result.gas_used > 0);

        let method = "stdout_non_utf8".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();
        assert_eq!(result.exit_info.exit_code, 8);
        assert_eq!(result.stdout.len(), 0);
        assert_eq!(
            &result.exit_info.exit_message,
            "Error: Failed to convert VM pipe `stdout` output to String"
        );
        assert!(result.gas_used > 0);
    }

    #[test]
    fn cannot_spam_call_result_write() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let temp_dir = TempDir::new("cannot_spam_call_result_write").unwrap();
        let tempdir = temp_dir.path();

        let method = "cannot_spam_call_result_write".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let result =
            _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs.clone(), 1024, 1024).unwrap();
        assert_eq!(result.exit_info.exit_code, 252);
        assert_eq!(result.stderr.len(), 1);
        assert_eq!(result.stderr[0], "Runtime error: Invalid Memory Access: call_result_write: result_data_ptr length does not match call_value length");
        assert_eq!(&result.exit_info.exit_message, "Not ok");
        assert!(result.gas_used > 0);
    }

    #[test]
    fn timing_call_infinite_loop() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let method = "infinite_loop_wasi".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("timing_call_infinite_loop").unwrap();
        let tempdir = temp_dir.path();
        let start = std::time::Instant::now();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();
        let elapsed = start.elapsed();

        assert_eq!(result.exit_info.exit_code, 252);
        assert_eq!(result.exit_info.exit_message, "Not ok".to_string());
        assert_eq!(result.stderr.len(), 1);
        assert_eq!(result.stderr[0], "Runtime error: Out of gas");
        assert!(elapsed.as_secs() < 2);
        assert!(result.gas_used > 0);
    }

    #[test]
    fn dr_playground_multiple_price_feed() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/price-feed-playground.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let temp_dir = TempDir::new("dr_playground_multiple_price_feed").unwrap();
        let tempdir = temp_dir.path();

        let method = "test".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let reveals = "[{\"dr_block_height\":1,\"exit_code\":0,\"gas_used\":502047984,\"reveal\":[123,34,112,114,105,99,101,34,58,49,48,48,48,48,48,48,125]}]".to_string();
        let consensus = "[0]".to_string();

        std::fs::create_dir_all(tempdir).unwrap();
        let result = _execute_tally_vm(
            tempdir,
            wasm_bytes.to_vec(),
            vec![method_hex, reveals, consensus],
            envs,
            1024,
            1024,
        )
        .unwrap();
        result.stdout.iter().for_each(|line| print!("{}", line));

        assert_eq!(result.gas_used, 11986115812500);
    }

    #[test]
    fn timing_spam_fd_write() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/spam-fd-write.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let temp_dir = TempDir::new("timing_spam_fd_write").unwrap();
        let tempdir = temp_dir.path();

        let start = std::time::Instant::now();
        let _result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![], envs, 1024, 1024).unwrap();
        let duration = start.elapsed();

        assert!(
            duration < std::time::Duration::from_millis(50),
            "Execution took too long: {:?} (should be < 50ms)",
            duration
        );
    }

    #[test]
    fn memory_fill_prealloc() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let method = "memory_fill_prealloc".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("memory_fill_prealloc").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();

        assert_eq!(result.exit_info.exit_code, 252);
        assert_eq!(result.stderr[0], "memory allocation of 44832551 bytes failed\n");
    }

    #[test]
    fn memory_fill_dynamic() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("VM_MODE".to_string(), "tally".to_string());
        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());

        let method = "memory_fill_dynamic".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("memory_fill_dynamic").unwrap();
        let tempdir = temp_dir.path();
        let result = _execute_tally_vm(tempdir, wasm_bytes.to_vec(), vec![method_hex], envs, 1024, 1024).unwrap();

        assert_eq!(result.exit_info.exit_code, 252);
        assert_eq!(result.stderr[0], "memory allocation of 8192000 bytes failed\n");
    }

    #[test]
    fn execute_binary_100_times() {
        let wasm_bytes = include_bytes!("../../test-wasm-files/test-vm.wasm");
        let mut envs: BTreeMap<String, String> = BTreeMap::new();
        envs.insert("CONSENSUS".to_string(), "true".to_string());
        envs.insert("VM_MODE".to_string(), "tally".to_string());

        envs.insert(DEFAULT_GAS_LIMIT_ENV_VAR.to_string(), "50000000000000".to_string());
        envs.insert("DR_REPLICATION_FACTOR".to_string(), "1".to_string());

        let method = "infinite_loop_wasi".to_string();
        let method_hex = hex::encode(method.to_bytes().eject());

        let temp_dir = TempDir::new("execute_binary_100_times").unwrap();
        let tempdir = temp_dir.path();
        let start_time = std::time::Instant::now();

        for _ in 0..100 {
            let _result = _execute_tally_vm(
                tempdir,
                wasm_bytes.to_vec(),
                vec![method_hex.clone()],
                envs.clone(),
                1024,
                1024,
            )
            .unwrap();
        }

        let total_duration = start_time.elapsed();
        println!("Total execution time for 100 runs: {:?}", total_duration);
        let average_duration = total_duration / 100;
        println!("Average execution time for 100 runs: {:?}", average_duration);

        assert!(average_duration < std::time::Duration::from_secs(10));
    }
}
