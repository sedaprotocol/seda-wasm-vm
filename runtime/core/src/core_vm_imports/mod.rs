use wasmer::{imports, FunctionEnv, Imports, Store, WasmPtr};

use crate::context::VmContext;

mod call_result;
mod execution_result;
mod keccak256;
mod secp256_k1;

#[macro_export]
macro_rules! generic_polyfill_import_obj {
    ($name:expr, $ret:ty $(, $arg_name:ident: $arg_type:ty)*) => {
        |store: &mut ::wasmer::Store,
         vm_context: &::wasmer::FunctionEnv<$crate::context::VmContext>|
         -> ::wasmer::Function {
            fn generic_polyfill(_: ::wasmer::FunctionEnvMut<'_, $crate::context::VmContext>, $($arg_name: $arg_type,)*) -> $crate::errors::Result<$ret> {
                Err($crate::errors::RuntimeError::Polyfilled(stringify!($name)))
            }

            ::wasmer::Function::new_typed_with_env(store, vm_context, generic_polyfill)
        }
    };
}

pub fn create_custom_core_imports(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Imports {
    let core_imports = imports! {
        "seda_v1" => {
            "bn254_verify" => generic_polyfill_import_obj!(
                "bn254_verify", u8,
                _message: WasmPtr<u8>,
                _message_length: i64,
                _signature: WasmPtr<u8>,
                _signature_length: i64,
                _public_key: WasmPtr<u8>,
                _public_key_length: i64
            )(store, vm_context),
            "call_result_length" => call_result::call_result_value_length_import_obj(store, vm_context),
            "call_result_write" => call_result::call_result_value_write_import_obj(store, vm_context),
            "execution_result" => execution_result::execution_result_import_obj(store, vm_context),
            "keccak256" => keccak256::keccak256_import_obj(store, vm_context),
            "secp256k1_verify" => secp256_k1::secp256k1_verify_import_obj(store, vm_context)
        },
    };

    core_imports
}
