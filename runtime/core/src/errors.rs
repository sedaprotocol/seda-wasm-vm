use std::{num::ParseIntError, string::FromUtf8Error};

use thiserror::Error;
use wasmer::{CompileError, ExportError};
use wasmer_wasix::{FsError, WasiError, WasiStateCreationError};

#[derive(Debug, Error)]
pub enum VmHostError {
    #[error("Instance on VmContext was not set")]
    InstanceNotSet,
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("Invalid WASM cache path, it exists but is not a directory: {0}")]
    InvalidCachePath(String),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    WasmCompileError(#[from] CompileError),

    #[error(transparent)]
    WasiError(#[from] WasiError),
    #[error(transparent)]
    WasiStateCreationError(#[from] WasiStateCreationError),

    #[error(transparent)]
    FunctionNotFound(#[from] ExportError),

    #[error("Error while running: {0}")]
    ExecutionError(#[from] wasmer::RuntimeError),

    #[error("VM Host Error: {0}")]
    VmHostError(#[from] VmHostError),

    #[error("{0}")]
    WasiFsError(#[from] FsError),

    #[error("{0}")]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    // TODO this is scuffed and not true for test_host.
    #[error("Node Error: {0}")]
    NodeError(String),

    #[error("SDK Error: {0}")]
    SDKError(#[from] seda_sdk_rs::errors::SDKError),

    #[error(transparent)]
    MemoryAccessError(#[from] wasmer::MemoryAccessError),

    #[error(transparent)]
    WasmSerializeError(#[from] wasmer::SerializeError),

    #[error(transparent)]
    WasmDeserializeError(#[from] wasmer::DeserializeError),

    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),

    #[error(transparent)]
    Ecdsa(#[from] k256::ecdsa::Error),

    #[error("Out of gas")]
    OutOfGas,

    #[error("Polyfill for function {0} is not implemented in tally mode")]
    Polyfilled(&'static str),

    #[error("{0}")]
    Unknown(String),

    #[error("Invalid Memory Access: {0}")]
    InvalidMemoryAccess(&'static str),
}

pub type Result<T, E = RuntimeError> = core::result::Result<T, E>;
