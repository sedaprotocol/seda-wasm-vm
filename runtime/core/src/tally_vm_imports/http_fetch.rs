use std::collections::HashMap;

use seda_runtime_sdk::{HttpFetchResponse, PromiseStatus};
use wasmer::{Function, FunctionEnv, FunctionEnvMut, Store, WasmPtr};

use crate::{errors::Result, VmContext};

/// Mostly a polyfill, otherwise the tally and dr binary cannot be one and the same
/// It simply errors but allows the WASM binary to continue.
pub fn http_fetch_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn http_fetch(env: FunctionEnvMut<'_, VmContext>, _result_ptr: WasmPtr<u8>, _result_length: i32) -> Result<u32> {
        let ctx = env.data();

        let message = "http_fetch is not allowed in tally".as_bytes().to_vec();
        let http_response: HttpFetchResponse = HttpFetchResponse {
            url:            "".to_string(),
            status:         0,
            headers:        HashMap::default(),
            content_length: message.len(),
            bytes:          message,
        };

        let result: PromiseStatus = PromiseStatus::Rejected(serde_json::to_vec(&http_response)?);

        let mut call_value = ctx.call_result_value.write();
        *call_value = serde_json::to_vec(&result)?;

        Ok(call_value.len() as u32)
    }

    Function::new_typed_with_env(store, vm_context, http_fetch)
}
