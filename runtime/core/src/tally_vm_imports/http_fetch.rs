use std::collections::HashMap;

use seda_runtime_sdk::{HttpFetchResponse, PromiseStatus};
use wasmer::{Function, FunctionEnv, FunctionEnvMut, Store, WasmPtr};

use crate::{
    errors::{Result, VmHostError},
    metering::apply_gas_cost,
    VmContext,
};

/// Mostly a polyfill, otherwise the tally and dr binary cannot be one and the same
/// It simply errors but allows the WASM binary to continue.
pub fn http_fetch_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn http_fetch(mut env: FunctionEnvMut<'_, VmContext>, _result_ptr: WasmPtr<u8>, result_length: i32) -> Result<u32> {
        apply_gas_cost(
            crate::metering::ExternalCallType::HttpFetchRequest(result_length as u64),
            &mut env,
        )?;

        let len = {
            let ctx = env.data();

            let message = "http_fetch is not allowed in tally".as_bytes().to_vec();
            let http_response: HttpFetchResponse = HttpFetchResponse {
                url:            "".to_string(),
                status:         0,
                headers:        HashMap::default(),
                content_length: message.len(),
                bytes:          message,
            };

            let result: PromiseStatus =
                PromiseStatus::Rejected(serde_json::to_vec(&http_response).map_err(VmHostError::from)?);

            let mut call_value = ctx.call_result_value.write();
            *call_value = serde_json::to_vec(&result).map_err(VmHostError::from)?;

            call_value.len()
        };

        apply_gas_cost(
            crate::metering::ExternalCallType::HttpFetchResponse(len as u64),
            &mut env,
        )?;

        Ok(len as u32)
    }

    Function::new_typed_with_env(store, vm_context, http_fetch)
}
