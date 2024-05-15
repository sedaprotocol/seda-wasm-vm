use crate::{PromiseStatus, ToBytes};

pub fn wasm_exists<ID: ToBytes>(id: ID) -> bool {
    let id = id.to_bytes().to_vec();
    let result = unsafe { super::raw::wasm_exists(id.as_ptr(), id.len() as u32) };

    match result {
        0 => false,
        1 => true,
        _ => unreachable!("wasm_exists returned invalid bool in u8: {}", result),
    }
}

// TODO what happens if these raw functions errors?
// The import definition does return a Result<()> type
pub fn wasm_store(bytes: &[u8]) -> PromiseStatus {
    let result_length = unsafe { super::raw::wasm_store(bytes.as_ptr(), bytes.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize wasm_store")
}
