use crate::{PromiseStatus, VmCallData};

pub fn vm_call(call_data: VmCallData) -> PromiseStatus {
    let action = serde_json::to_vec(&call_data).expect("Invalid vm_call action");
    let result_length = unsafe { super::raw::vm_call(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize vm_call result")
}
