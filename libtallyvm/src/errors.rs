use std::num::ParseIntError;

use seda_wasm_vm::RuntimeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TallyVmError {
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] RuntimeError),

    #[error("ParseInt error: {0}")]
    ParseInt(#[from] ParseIntError),
}

impl TallyVmError {
    pub fn exit_code(&self) -> i32 {
        match self {
            TallyVmError::RuntimeError(_) => 251,
            TallyVmError::ParseInt(_) => 252,
        }
    }
}

pub type Result<T, E = TallyVmError> = core::result::Result<T, E>;
