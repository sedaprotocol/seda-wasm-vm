use crate::{DatabaseGetAction, DatabaseSetAction, PromiseStatus};

pub fn db_set(key: &str, value: &str) -> PromiseStatus {
    let database_set_action = DatabaseSetAction {
        key:   key.to_string(),
        value: value.to_string().into_bytes(),
    };

    let action = serde_json::to_string(&database_set_action).unwrap();
    let result_length = unsafe { super::raw::db_set(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize db_set")
}

pub fn db_get(key: &str) -> PromiseStatus {
    let database_get_action = DatabaseGetAction { key: key.to_string() };

    let action = serde_json::to_string(&database_get_action).unwrap();
    let result_length = unsafe { super::raw::db_get(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize db_get")
}
