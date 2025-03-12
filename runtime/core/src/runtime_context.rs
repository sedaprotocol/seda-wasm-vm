use std::{path::Path, sync::Arc};

use seda_runtime_sdk::{VmCallData, WasmId};
use wasmer::{
    sys::{BaseTunables, EngineBuilder},
    CompilerConfig,
    Module,
    Pages,
    Singlepass,
    Store,
    Target,
};
use wasmer_middlewares::Metering;

use crate::{
    errors::Result,
    memory::LimitingTunables,
    metering::get_wasm_operation_gas_cost,
    wasm_cache::wasm_cache_id,
};

pub struct RuntimeContext {
    pub wasm_store:  Store,
    pub wasm_module: Module,
    pub wasm_hash:   String,
}

impl RuntimeContext {
    pub fn new(_sedad_home: &Path, call_data: &VmCallData) -> Result<Self> {
        let base = BaseTunables::for_target(&Target::default());
        let tunables = LimitingTunables::new(base, Pages(call_data.max_memory_pages));
        let mut single_pass_config = Singlepass::new();

        if let Some(gas_limit) = call_data.gas_limit {
            let metering = Arc::new(Metering::new(gas_limit, get_wasm_operation_gas_cost));
            single_pass_config.push_middleware(metering);
        }

        let builder = EngineBuilder::new(single_pass_config);
        let mut engine = builder.engine();
        engine.set_tunables(tunables);

        let store = Store::new(engine);

        let (wasm_module, wasm_hash) = match &call_data.wasm_id {
            WasmId::Bytes(wasm_bytes) => {
                let wasm_id = wasm_cache_id(wasm_bytes);
                let wasm_module = Module::new(&store, wasm_bytes)?;

                (wasm_module, wasm_id)
            }
        };

        Ok(Self {
            wasm_module,
            wasm_store: store,
            wasm_hash,
        })
    }
}
