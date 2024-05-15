use crate::{promises::HttpFetchOptions, HttpFetchAction, PromiseStatus};

pub fn http_fetch<URL: ToString>(url: URL, options: Option<HttpFetchOptions>) -> PromiseStatus {
    let http_action = HttpFetchAction {
        url:     url.to_string(),
        options: options.unwrap_or_default(),
    };

    let action = serde_json::to_string(&http_action).unwrap();
    let result_length = unsafe { super::raw::http_fetch(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize http_fetch")
}
