use std::{fs, io::Write, path::Path, sync::Arc};

use seda_runtime_sdk::{VmCallData, WasmId};
use wasmer::{sys::BaseTunables, CompilerConfig, Engine, Module, NativeEngineExt, Pages, Singlepass, Store, Target};
use wasmer_middlewares::Metering;

use crate::{
    errors::Result,
    memory::LimitingTunables,
    metering::get_wasm_operation_gas_cost,
    wasm_cache::wasm_cache_id,
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
    pub fn new(_sedad_home: &Path, call_data: &VmCallData) -> Result<Self> {
        let engine = make_runtime_engine(call_data.max_memory_pages);
        let store = Store::new(engine);

        let (wasm_module, wasm_hash) = match &call_data.wasm_id {
            WasmId::Bytes(wasm_bytes) => {
                let wasm_id = wasm_cache_id(wasm_bytes);

                // Check if the module is already cached
                fs::create_dir_all("./wasm_cache")?;
                let compiled = Path::new("./wasm_cache").join(&wasm_id);

                if compiled.exists() {
                    let wasm_module = unsafe { Module::deserialize_from_file(&store, compiled)? };
                    return Ok(Self {
                        wasm_module,
                        wasm_store: store,
                        wasm_hash: wasm_id,
                    });
                }

                // If not, compile and cache it
                let wasm_module = Module::new(&make_compiling_engine(call_data.max_memory_pages), wasm_bytes)?;

                let mut file = fs::File::create(&compiled)?;
                let buffer = wasm_module.serialize()?;
                file.write_all(&buffer)?;

                drop(wasm_module);
                let wasm_module = unsafe { Module::deserialize_from_file(&store, compiled)? };

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
