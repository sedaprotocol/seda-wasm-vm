use std::num::ParseIntError;

use seda_wasm_vm::RuntimeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TallyVmError {
    #[error(transparent)]
    RuntimeError(#[from] RuntimeError),

    #[error(transparent)]
    ParseInt(#[from] ParseIntError)
}

pub type Result<T, E = TallyVmError> = core::result::Result<T, E>;
