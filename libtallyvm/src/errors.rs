use seda_wasm_vm::RuntimeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TallyVmError {
    #[error(transparent)]
    RuntimeError(#[from] RuntimeError),

    #[error(transparent)]
    MemoryAccessError(#[from] wasmer::MemoryAccessError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

pub type Result<T, E = TallyVmError> = core::result::Result<T, E>;
