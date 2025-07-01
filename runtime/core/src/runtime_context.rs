use std::{fs, path::Path, sync::Arc};

use seda_runtime_sdk::{VmCallData, WasmId};
use wasmer::{sys::BaseTunables, CompilerConfig, Engine, Module, NativeEngineExt, Pages, Singlepass, Store, Target};
use wasmer_middlewares::Metering;

use crate::{
    errors::Result,
    memory::LimitingTunables,
    metering::get_wasm_operation_gas_cost,
    wasm_cache::{get_full_wasm_path_from_id, valid_wasm_cache_id, wasm_cache_id, wasm_cache_load, wasm_cache_store},
};

pub fn make_runtime_engine(max_memory_pages: u32) -> Engine {
    let mut engine = Engine::headless();

    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_memory_pages));
    engine.set_tunables(tunables);
    engine
}

pub fn make_compiling_engine(max_memory_pages: u32) -> Store {
    let mut compiler = Singlepass::new();

    let metering = Arc::new(Metering::new(0, get_wasm_operation_gas_cost));
    compiler.push_middleware(metering);
    let mut engine = Engine::from(compiler);

    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_memory_pages));
    engine.set_tunables(tunables);

    Store::new(engine)
}

pub struct RuntimeContext {
    pub wasm_store:  Store,
    pub wasm_module: Module,
    pub wasm_hash:   String,
}

impl RuntimeContext {
    pub fn new(sedad_home: &Path, call_data: &VmCallData) -> Result<Self> {
        let engine = make_runtime_engine(call_data.max_memory_pages);
        let store = Store::new(engine);

        let (wasm_module, wasm_hash) = match &call_data.wasm_id {
            WasmId::Bytes(wasm_bytes) => {
                let wasm_id = wasm_cache_id(wasm_bytes);
                let wasm_path = get_full_wasm_path_from_id(sedad_home, &wasm_id);

                let mut compiled = wasm_path.is_some();
                if compiled && !valid_wasm_cache_id(wasm_path.as_ref().unwrap()) {
                    fs::remove_file(wasm_path.as_ref().unwrap())?;
                    compiled = false;
                }

                if compiled {
                    let wasm_module = wasm_cache_load(&store, wasm_path.as_ref().unwrap())?;
                    return Ok(Self {
                        wasm_module,
                        wasm_store: store,
                        wasm_hash: wasm_id,
                    });
                }

                // If not, compile and cache it
                let wasm_module = wasm_cache_store(
                    sedad_home,
                    &make_compiling_engine(call_data.max_memory_pages),
                    &store,
                    &wasm_id,
                    wasm_bytes,
                )?;

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
