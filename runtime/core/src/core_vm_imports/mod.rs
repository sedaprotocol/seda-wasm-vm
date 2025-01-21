use secp256_k1::secp256k1_verify_import_obj;
use wasmer::{imports, FunctionEnv, Imports, Store};

use crate::context::VmContext;

mod call_result;
mod execution_result;
mod keccak256;
mod secp256_k1;

// TODO: need a way to add additional arguments to the polyfill
macro_rules! generic_polyfill_import_obj {
    ($name:expr, $ret:ty) => {
        |store: &mut ::wasmer::Store,
         vm_context: &::wasmer::FunctionEnv<crate::context::VmContext>|
         -> ::wasmer::Function {
            fn generic_polyfill(_: ::wasmer::FunctionEnvMut<'_, VmContext>) -> crate::errors::Result<$ret> {
                tracing::info!("Polyfill for function {} is not implemented in tally mode", $name);

                Ok(<$ret>::default())
            }

            ::wasmer::Function::new_typed_with_env(store, vm_context, generic_polyfill)
        }
    };
}

pub fn create_custom_core_imports(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Imports {
    let core_imports = imports! {
        "seda_v1" => {
            // TODO: when merged to run the dr vm these should not be polyfills
            "shared_memory_contains_key" => generic_polyfill_import_obj!("shared_memory_contains_key", u8)(store, vm_context),
            "shared_memory_read" => generic_polyfill_import_obj!("shared_memory_read", ())(store, vm_context),
            "shared_memory_read_length" => generic_polyfill_import_obj!("shared_memory_read_length", i64)(store, vm_context),
            "shared_memory_write" => generic_polyfill_import_obj!("shared_memory_write", ())(store, vm_context),
            "shared_memory_remove" => generic_polyfill_import_obj!("shared_memory_remove", ())(store, vm_context),
            "shared_memory_range" => generic_polyfill_import_obj!("shared_memory_range", u32)(store, vm_context),
            "_log" => generic_polyfill_import_obj!("_log", ())(store, vm_context),
            "bn254_verify" => generic_polyfill_import_obj!("bn254_verify", u8)(store, vm_context),
            "proxy_http_fetch" => generic_polyfill_import_obj!("proxy_http_fetch", u32)(store, vm_context),
            "http_fetch" => generic_polyfill_import_obj!("http_fetch", u32)(store, vm_context),
            "chain_view" => generic_polyfill_import_obj!("chain_view", u32)(store, vm_context),
            "chain_send_tx" => generic_polyfill_import_obj!("chain_send_tx", u32)(store, vm_context),
            "chain_tx_status" => generic_polyfill_import_obj!("chain_tx_status", u32)(store, vm_context),
            "main_chain_call" => generic_polyfill_import_obj!("main_chain_call", u32)(store, vm_context),
            "main_chain_call_tx_status" => generic_polyfill_import_obj!("main_chain_call_tx_status", u32)(store, vm_context),
            "main_chain_view" => generic_polyfill_import_obj!("main_chain_view", u32)(store, vm_context),
            "main_chain_query" => generic_polyfill_import_obj!("main_chain_query", u32)(store, vm_context),
            "vm_call" => generic_polyfill_import_obj!("vm_call", u32)(store, vm_context),
            "db_set" => generic_polyfill_import_obj!("db_set", u32)(store, vm_context),
            "db_get" => generic_polyfill_import_obj!("db_get", u32)(store, vm_context),
            "trigger_event" => generic_polyfill_import_obj!("trigger_event", ())(store, vm_context),
            "wasm_exists" => generic_polyfill_import_obj!("wasm_exists", u8)(store, vm_context),
            "wasm_store" => generic_polyfill_import_obj!("wasm_store", u32)(store, vm_context),
            "identity_sign" => generic_polyfill_import_obj!("identity_sign", u32)(store, vm_context),
            "use_gas" => generic_polyfill_import_obj!("use_gas", ())(store, vm_context),
            "abort_app" => generic_polyfill_import_obj!("abort_app", ())(store, vm_context),

            "call_result_write" => call_result::call_result_value_write_import_obj(store, vm_context),
            "call_result_length" => call_result::call_result_value_length_import_obj(store, vm_context),
            "execution_result" => execution_result::execution_result_import_obj(store, vm_context),
            "secp256k1_verify" => secp256k1_verify_import_obj(store, vm_context),
            "keccak256" => keccak256::keccak256_import_obj(store, vm_context)
        }
    };

    core_imports
}
