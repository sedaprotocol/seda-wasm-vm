use secp256_k1::secp256k1_verify_import_obj;
use wasmer::{imports, FunctionEnv, Imports, Store};

use crate::context::VmContext;

mod call_result;
mod execution_result;
mod secp256_k1;

pub fn create_custom_core_imports(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Imports {
    let core_imports = imports! {
        "seda_v1" => {
            "call_result_write" => call_result::call_result_value_write_import_obj(store, vm_context),
            "call_result_length" => call_result::call_result_value_length_import_obj(store, vm_context),
            "execution_result" => execution_result::execution_result_import_obj(store, vm_context),
            "secp256k1_verify" => secp256k1_verify_import_obj(store, vm_context),
        }
    };

    core_imports
}
