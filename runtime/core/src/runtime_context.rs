use std::fs;

use seda_runtime_sdk::WasmId;
use wasmer::{Module, Store};

use crate::{
    errors::Result,
    wasm_cache::{wasm_cache_id, wasm_cache_load, wasm_cache_store},
};

pub struct RuntimeContext {
    pub wasm_store:  Store,
    pub wasm_module: Module,
    pub wasm_hash:   String,
}

impl RuntimeContext {
    pub fn new(wasm_id: &WasmId) -> Result<Self> {
        let store = Store::default();

        let (wasm_module, wasm_hash) = match wasm_id {
            WasmId::Bytes(wasm_bytes) => {
                let wasm_id = wasm_cache_id(wasm_bytes);

                let wasm_module = match wasm_cache_load(&store, &wasm_id) {
                    Ok(module) => module,
                    // The binary didn't exist in cache when we loaded it, so we cache it now
                    // to speed up future executions
                    Err(_) => wasm_cache_store(&store, &wasm_id, wasm_bytes)?,
                };

                (wasm_module, wasm_id)
            }
            WasmId::Id(wasm_id) => {
                let wasm_module = wasm_cache_load(&store, wasm_id)?;

                (wasm_module, wasm_id.to_string())
            }
            WasmId::Path(wasm_path) => {
                let wasm_bytes = fs::read(wasm_path)?;
                let wasm_id = wasm_cache_id(&wasm_bytes);

                let wasm_module = match wasm_cache_load(&store, &wasm_id) {
                    Ok(module) => module,
                    // The binary didn't exist in cache when we loaded it, so we cache it now
                    // to speed up future executions
                    Err(_) => wasm_cache_store(&store, &wasm_id, &wasm_bytes)?,
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
