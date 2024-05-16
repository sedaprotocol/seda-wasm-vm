use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
    mem,
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

impl From<VmResult> for FfiVmResult {
    fn from(vm_result: VmResult) -> Self {
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

        FfiVmResult {
            stdout_ptr,
            stdout_len,
            stderr_ptr,
            stderr_len,
            result_ptr,
            result_len,
            exit_info: vm_result.exit_info.into(),
        }
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn execute_tally_vm(
    wasm_bytes: *const u8,
    wasm_bytes_len: usize,
    args_ptr: *const *const c_char,
    args_count: usize,
    env_keys_ptr: *const *const c_char,
    env_values_ptr: *const *const c_char,
    env_count: usize,
) -> FfiVmResult {
    let wasm_bytes = std::slice::from_raw_parts(wasm_bytes, wasm_bytes_len).to_vec();

    let args: Vec<String> = (0..args_count)
        .map(|i| {
            let ptr = *args_ptr.add(i);
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        })
        .collect();

    let mut envs = HashMap::new();
    for i in 0..env_count {
        let key_ptr = *env_keys_ptr.add(i);
        let value_ptr = *env_values_ptr.add(i);

        let key = CStr::from_ptr(key_ptr).to_string_lossy().into_owned();
        let value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();

        envs.insert(key, value);
    }

    match _execute_tally_vm(wasm_bytes, args, envs) {
        Ok(vm_result) => vm_result.into(),
        Err(_) => FfiVmResult {
            stdout_ptr: ptr::null(),
            stdout_len: 0,
            stderr_ptr: ptr::null(),
            stderr_len: 0,
            result_ptr: ptr::null(),
            result_len: 0,
            exit_info:  FfiExitInfo {
                exit_message: CString::new("Error executing VM").unwrap().into_raw(),
                exit_code:    -1,
            },
        },
    }
}

fn _execute_tally_vm(wasm_bytes: Vec<u8>, args: Vec<String>, envs: HashMap<String, String>) -> Result<VmResult> {
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
