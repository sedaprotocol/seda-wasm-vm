use std::{fs, path::Path, sync::Arc};

use seda_runtime_sdk::{VmCallData, WasmId};
use wasmer::{sys::EngineBuilder, CompilerConfig, Module, Singlepass, Store};
use wasmer_middlewares::Metering;

use crate::{
    errors::Result,
    metering::get_wasm_operation_gas_cost,
    wasm_cache::{wasm_cache_id, wasm_cache_load, wasm_cache_store},
};

pub struct RuntimeContext {
    pub wasm_store:  Store,
    pub wasm_module: Module,
    pub wasm_hash:   String,
}

impl RuntimeContext {
    pub fn new(sedad_home: &Path, call_data: &VmCallData) -> Result<Self> {
        let mut engine = Singlepass::default();

        if let Some(gas_limit) = call_data.gas_limit {
            let metering = Arc::new(Metering::new(gas_limit, get_wasm_operation_gas_cost));
            engine.push_middleware(metering);
        }

        let store = Store::new(EngineBuilder::new(engine));

        let (wasm_module, wasm_hash) = match &call_data.wasm_id {
            WasmId::Bytes(wasm_bytes) => {
                let wasm_id = wasm_cache_id(wasm_bytes);
                let wasm_module = Module::new(&store, wasm_bytes)?;

                (wasm_module, wasm_id)
            }
            WasmId::Id(wasm_id) => {
                let wasm_module = wasm_cache_load(sedad_home, &store, wasm_id)?;

                (wasm_module, wasm_id.to_string())
            }
            WasmId::Path(wasm_path) => {
                let wasm_bytes = fs::read(wasm_path)?;
                let wasm_id = wasm_cache_id(&wasm_bytes);

                let wasm_module = match wasm_cache_load(sedad_home, &store, &wasm_id) {
                    Ok(module) => module,
                    // The binary didn't exist in cache when we loaded it, so we cache it now
                    // to speed up future executions
                    Err(_) => wasm_cache_store(sedad_home, &store, &wasm_id, &wasm_bytes)?,
                };

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
