use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
    ptr,
};

use seda_runtime_sdk::{ExitInfo, VmType, WasmId};
use seda_wasm_vm::{start_runtime, RuntimeContext, VmCallData, VmResult};

use crate::errors::Result;

mod errors;

#[derive(Debug)]
#[repr(C)]
pub struct FfiExitInfo {
    exit_message: *const c_char,
    exit_code:    i32,
}

impl FfiExitInfo {
    fn from_exit_info(exit_info: ExitInfo) -> Self {
        FfiExitInfo {
            exit_message: CString::new(exit_info.exit_message).unwrap().into_raw(),
            exit_code:    exit_info.exit_code,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FfiVmResult {
    stdout:     *const *const c_char,
    stdout_len: usize,
    stderr:     *const *const c_char,
    stderr_len: usize,
    result:     *const u8,
    result_len: usize,
    exit_info:  FfiExitInfo,
}

impl FfiVmResult {
    fn from_vm_result(vm_result: VmResult) -> Self {
        let stdout: Vec<CString> = vm_result
            .stdout
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();
        let stderr: Vec<CString> = vm_result
            .stderr
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();

        let stdout_ptrs: Vec<*const c_char> = stdout.iter().map(|s| s.as_ptr()).collect();
        let stderr_ptrs: Vec<*const c_char> = stderr.iter().map(|s| s.as_ptr()).collect();

        FfiVmResult {
            stdout:     stdout_ptrs.as_ptr(),
            stdout_len: stdout_ptrs.len(),
            stderr:     stderr_ptrs.as_ptr(),
            stderr_len: stderr_ptrs.len(),
            result:     vm_result.result.as_deref().unwrap_or(&[]).as_ptr(),
            result_len: vm_result.result.as_deref().unwrap_or(&[]).len(),
            exit_info:  FfiExitInfo::from_exit_info(vm_result.exit_info),
        }
    }
}

/// # Safety
///
/// TODO something more meaningful here
#[no_mangle]
pub unsafe extern "C" fn execute_tally_vm(
    wasm_bytes: *const u8,
    wasm_bytes_len: usize,
    args_ptr: *const *const c_char,
    args_count: usize,
    env_keys_ptr: *const *const c_char,
    env_values_ptr: *const *const c_char,
    env_count: usize,
) {
    let wasm_bytes = std::slice::from_raw_parts(wasm_bytes, wasm_bytes_len).to_vec();

    let args: Vec<String> = (0..args_count)
        .map(|i| {
            let ptr = unsafe { *args_ptr.add(i) };
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        })
        .collect();

    let mut envs = HashMap::new();
    for i in 0..env_count {
        let key_ptr = unsafe { *env_keys_ptr.add(i) };
        let value_ptr = unsafe { *env_values_ptr.add(i) };

        let key = CStr::from_ptr(key_ptr).to_string_lossy().into_owned();
        let value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();

        envs.insert(key, value);
    }

    match _execute_tally_vm(wasm_bytes, args, envs) {
        Ok(vm_result) => FfiVmResult::from_vm_result(vm_result),
        // TODO better handle this lol
        Err(_) => FfiVmResult {
            stdout:     ptr::null(),
            stdout_len: 0,
            stderr:     ptr::null(),
            stderr_len: 0,
            result:     ptr::null(),
            result_len: 0,
            exit_info:  FfiExitInfo {
                exit_message: ptr::null(),
                exit_code:    -1,
            },
        },
    };
}

fn _execute_tally_vm(wasm_bytes: Vec<u8>, args: Vec<String>, envs: HashMap<String, String>) -> Result<VmResult> {
    dbg!(&args, &envs);
    let wasm_id = WasmId::Bytes(wasm_bytes);
    let runtime_context = RuntimeContext::new(&wasm_id)?;

    let result = start_runtime(
        VmCallData {
            call_id: None,
            wasm_id,
            args,
            envs,
            program_name: runtime_context.wasm_hash.to_string(),
            start_func: None,
            vm_type: VmType::Tally,
        },
        runtime_context,
    );

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::_execute_tally_vm;

    #[test]
    fn test_execute_tally_vm() {
        let wasm_bytes = include_bytes!("../../debug.wasm");
        let result = _execute_tally_vm(
            wasm_bytes.to_vec(),
            vec!["testHttpSuccess".to_string()],
            HashMap::default(),
        )
        .unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));

        dbg!(result);
    }
}
