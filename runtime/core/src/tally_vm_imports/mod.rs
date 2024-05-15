use wasmer::{Exports, FunctionEnv, Imports, Store};

use crate::{create_custom_core_imports, VmContext, SAFE_WASI_IMPORTS};

mod http_fetch;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SAFE_TALLY_IMPORTS: Vec<String> = {
        ["execution_result", "http_fetch", "call_result_write"]
            .iter()
            .map(|import| import.to_string())
            .chain(SAFE_WASI_IMPORTS.to_vec())
            .collect()
    };
}

pub fn create_custom_tally_imports(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Imports {
    let core_imports = create_custom_core_imports(store, vm_context);
    let mut tally_exports = Exports::new();
    let mut final_imports = Imports::new();

    tally_exports.insert("http_fetch", http_fetch::http_fetch_import_obj(store, vm_context));

    if let Some(core_exports) = core_imports.get_namespace_exports("seda_v1") {
        for (export_name, export) in core_exports.iter() {
            tally_exports.insert(export_name, export.clone());
        }
    }

    final_imports.register_namespace("seda_v1", tally_exports);

    final_imports
}
