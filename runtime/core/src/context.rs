use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use wasmer::{AsStoreRef, FunctionEnv, Imports, Instance, Memory, MemoryView, Store};
use wasmer_wasix::{WasiEnv, WasiVersion};

use crate::vm::VmCallData;

#[derive(Clone)]
pub struct VmContext {
    pub call_data:    VmCallData,
    pub result:       Arc<Mutex<Vec<u8>>>,
    pub memory:       Option<Memory>,
    pub wasi_env:     FunctionEnv<WasiEnv>,
    pub wasi_imports: Option<Imports>,
    pub wasi_version: Option<WasiVersion>,

    /// Used for internal use only
    /// This is used to temp store a result of an action
    /// For ex doing a http fetch is 3 calls (action, get_length, write_result)
    /// Between actions we need this result value, so instead of doing the
    /// action multiple times We temp store the value for later use.
    /// NOTE: It's pretty unsafe if it's not being used correctly. Since our SDK
    /// use these 3 calls in sequental we are fine, but it could crash if the
    /// order changes.
    pub call_result_value: Arc<RwLock<Vec<u8>>>,
    pub instance:          Option<Instance>,
}

impl VmContext {
    #[allow(clippy::too_many_arguments)]
    pub fn create_vm_context(
        store: &mut Store,
        wasi_env: FunctionEnv<WasiEnv>,
        call_data: VmCallData,
    ) -> FunctionEnv<VmContext> {
        FunctionEnv::new(
            store,
            VmContext {
                result: Arc::new(Mutex::new(Vec::new())),
                memory: None,
                wasi_env,
                call_result_value: Arc::new(RwLock::new(Vec::new())),
                instance: None,
                wasi_imports: None,
                call_data,
                wasi_version: None,
            },
        )
    }

    /// Provides safe access to the memory
    /// (it must be initialized before it can be used)
    pub fn memory_view<'a>(&'a self, store: &'a impl AsStoreRef) -> MemoryView<'a> {
        self.memory().view(store)
    }

    /// Get memory, that needs to have been set fist
    pub fn memory(&self) -> &Memory {
        self.memory.as_ref().unwrap()
    }
}
