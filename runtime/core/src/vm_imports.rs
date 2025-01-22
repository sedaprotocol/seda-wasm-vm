use seda_runtime_sdk::{VmCallData, VmType};
use wasmer::{Exports, FunctionEnv, Imports, Module, Store, WasmPtr};
use wasmer_wasix::{get_wasi_version, WasiFunctionEnv};

use crate::{
    errors::Result,
    tally_vm_imports::{create_custom_tally_imports, SAFE_TALLY_IMPORTS},
    VmContext,
};

pub fn create_wasm_imports(
    store: &mut Store,
    vm_context: &FunctionEnv<VmContext>,
    wasi_env: &WasiFunctionEnv,
    wasm_module: &Module,
    call_data: &VmCallData,
) -> Result<Imports> {
    let wasi_import_obj = wasi_env.import_object(store, wasm_module)?;
    let wasi_version = get_wasi_version(wasm_module, false);
    let mut final_imports = Imports::new();

    let (allowed_imports, custom_imports) = match call_data.vm_type {
        VmType::Tally => {
            let tally_imports = create_custom_tally_imports(store, vm_context);

            (SAFE_TALLY_IMPORTS.to_vec(), tally_imports)
        }
        VmType::DataRequest => todo!(),
        VmType::Core => todo!(),
    };

    // Only allow imports that the user defined
    let mut allowed_host_exports = Exports::new();
    let mut allowed_wasi_exports = Exports::new();

    for allowed_import in allowed_imports.iter() {
        // "env" is all our custom host imports
        if let Some(found_export) = custom_imports.get_export("seda_v1", allowed_import) {
            allowed_host_exports.insert(allowed_import.to_string(), found_export);
        } else if let Some(wasi_version) = wasi_version {
            // When we couldn't find a match in our custom import we try WASI imports
            // WASI has different versions of compatibility so it depends how the WASM was
            // build, that's why we use wasi_verison to determine the correct export
            if let Some(found_export) = wasi_import_obj.get_export(wasi_version.get_namespace_str(), allowed_import) {
                allowed_wasi_exports.insert(allowed_import.to_string(), found_export);
            }
        }
    }

    final_imports.register_namespace("seda_v1", allowed_host_exports);

    if let Some(wasi_version) = wasi_version {
        // additionally polyfill the "random_get" wasi import
        allowed_wasi_exports.insert(
            "random_get".to_string(),
            // https://wasix.org/docs/api-reference/wasi/random_get
            // https://docs.rs/wasix/latest/wasix/lib_generated64/fn.random_get.html
            crate::generic_polyfill_import_obj!(
                "random_get", i32,
                _buf: WasmPtr<u8>,
                _buf_len: i32
            )(store, vm_context),
        );
        final_imports.register_namespace(wasi_version.get_namespace_str(), allowed_wasi_exports);
    }

    Ok(final_imports)
}
