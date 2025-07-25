use core::fmt;
use std::collections::BTreeMap;

use seda_sdk_rs::bytes::{Bytes, ToBytes};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum ConsensusType {
    Executor,
    Relayer,
}

impl fmt::Display for ConsensusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsensusType::Executor => write!(f, "Executor"),
            ConsensusType::Relayer => write!(f, "Relayer"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum WasmId {
    // Doesn't exist for Singlepass compiler
    // /// The ID of the WASM file, loads from cache
    // Id(String),
    // Unlikely to be used for tallyvm?
    // /// The path on disk of the WASM file
    // Path(String),
    /// The bytes of the binary
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum VmType {
    Tally,
    DataRequest,
    Core,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum WasmEngine {
    Cranelift,
    Singlepass,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct VmCallData {
    /// Identifier for differentiating between processes
    /// If assigned, the runtime will queue the call if there is already a process running with that id
    /// If left empty it auto assigns an id and will run immidiatly
    pub call_id: Option<String>,

    /// Identifier for which WASM file to pick
    pub wasm_id: WasmId,

    /// Arguments to pass to the WASM binary
    pub args: Vec<String>,

    /// Environment variables you want to pass to the WASM binary
    pub envs: BTreeMap<String, String>,

    /// Name of the binary, ex. "consensus", "fisherman", etc
    pub program_name: String,

    /// The function we need to execute, defaults to the WASI default ("_start")
    pub start_func: Option<String>,

    /// Amount of gas units the VM is allowed to use, None means infinite
    pub gas_limit: Option<u64>,

    /// Which VM context you want to run in
    pub vm_type: VmType,

    pub max_memory_pages: u32,
}

impl VmCallData {
    pub fn args_bytes_len(&self) -> usize {
        // Sum up the length of each argument string plus 1 byte for null terminator
        self.args.iter().fold(0, |acc, arg| acc + arg.len() + 1)
    }

    pub fn env_bytes_len(&self) -> usize {
        // Sum up the length of each environment variable key and value plus 2 bytes for null terminators
        self.envs
            .iter()
            .fold(0, |acc, (key, val)| acc + key.len() + val.len() + 2)
    }
}

impl Default for VmCallData {
    fn default() -> Self {
        Self {
            vm_type:          VmType::Tally,
            args:             vec![],
            call_id:          None,
            envs:             Default::default(),
            program_name:     "default".to_string(),
            start_func:       None,
            wasm_id:          WasmId::Bytes(vec![]),
            gas_limit:        None,
            // 160 pages * 64KB per page = 10MB maximum memory
            max_memory_pages: 160,
        }
    }
}

impl fmt::Display for VmCallData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VmCallData {{ call_id: {:?}, args: {:?}, envs: {:?}, program_name: {:?}, start_func: {:?}, vm_type: {:?} }}",
            self.call_id, self.args, self.envs, self.program_name, self.start_func, self.vm_type
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct ExitInfo {
    pub exit_message: String,
    pub exit_code:    i32,
}

impl ExitInfo {
    pub fn is_ok(&self) -> bool {
        self.exit_code == 0
    }
}

impl From<(String, i32)> for ExitInfo {
    fn from((exit_message, exit_code): (String, i32)) -> Self {
        Self {
            exit_message,
            exit_code,
        }
    }
}

/// Represents the result of a Vm instance
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct VmResult {
    pub stdout:    Vec<String>,
    pub stderr:    Vec<String>,
    pub result:    Option<Vec<u8>>,
    pub exit_info: ExitInfo,
    pub gas_used:  u64,
}

impl VmResult {
    pub fn create_err<M: ToString>(message: M, exit_code: i32) -> VmResult {
        VmResult {
            stdout:    vec![],
            stderr:    vec![message.to_string()],
            result:    None,
            exit_info: ExitInfo {
                exit_message: message.to_string(),
                exit_code,
            },
            gas_used:  0,
        }
    }
}

impl ToBytes for VmResult {
    fn to_bytes(self) -> Bytes {
        // TODO: Handle this unwrap (First we need a try_to_bytes())
        serde_json::to_vec(&self).unwrap().to_bytes()
    }
}

// TODO create a readme of all these once its better established
/// The possible statuses of a [VmResult]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub enum VmResultStatus {
    /// When the Vm runs and exits successfully
    Ok(String),
    /// When the WASI environment variables could not be initialized
    WasiEnvInitializeFailure,
    /// When the host functions could not be exported to the VM
    FailedToCreateVMImports,
    /// When the WASMER instance could not be created
    FailedToCreateWasmerInstance(String, u64),
    /// When a function from the WASM VM does not exist
    FailedToGetWASMFn(u64),
    /// When we fail to fetch the WASM VM stdout
    FailedToGetWASMStdout(u64),
    /// When we fail to fetch the WASM VM stderr
    FailedToGetWASMStderr(u64),
    /// When we fail to fetch the WASM VM stderr
    FailedToConvertVMPipeToString(String, u64),
    /// An execution error from the WASM Runtime
    ExecutionError(String, u64),
    /// When we fail to get the memory export
    FailedToGetWASMMemory(u64),
    /// When the execution result size exceeds the maximum allowed size
    ResultSizeExceeded(u64),
    GasStartupCostTooHigh(u64),
}

impl From<VmResultStatus> for ExitInfoWithGasUsed {
    fn from(value: VmResultStatus) -> Self {
        match value {
            VmResultStatus::Ok(msg) => Self((format!("Success: {msg}"), 0).into(), 0),
            VmResultStatus::WasiEnvInitializeFailure => {
                Self(("Error: Failed to initialize Wasi Env".into(), 2).into(), 0)
            }
            VmResultStatus::FailedToCreateVMImports => {
                Self(("Error: Failed to create host imports for VM".into(), 3).into(), 0)
            }
            VmResultStatus::FailedToCreateWasmerInstance(msg, startup_cost) => Self(
                (format!("Error: Failed to create WASMER instance: {msg}"), 4).into(),
                startup_cost,
            ),
            VmResultStatus::FailedToGetWASMFn(startup_cost) => Self(
                ("Error: Failed to find specified function in WASM binary".into(), 5).into(),
                startup_cost,
            ),
            VmResultStatus::FailedToGetWASMStdout(gas_used) => {
                Self(("Error: Failed to get STDOUT of VM".into(), 6).into(), gas_used)
            }
            VmResultStatus::FailedToGetWASMStderr(gas_used) => {
                Self(("Error: Failed to get STDERR of VM".into(), 7).into(), gas_used)
            }
            VmResultStatus::FailedToConvertVMPipeToString(kind, gas_used) => Self(
                (format!("Error: Failed to convert VM pipe `{kind}` output to String"), 8).into(),
                gas_used,
            ),
            VmResultStatus::ExecutionError(err, gas_used) => {
                Self((format!("Execution Error: {err}"), 9).into(), gas_used)
            }
            VmResultStatus::FailedToGetWASMMemory(gas_used) => Self(
                ("Error: Failed to get memory export from WASM".into(), 10).into(),
                gas_used,
            ),
            VmResultStatus::ResultSizeExceeded(gas_used) => Self(
                ("Error: Execution result size exceeds maximum allowed size".into(), 13).into(),
                gas_used,
            ),
            VmResultStatus::GasStartupCostTooHigh(gas_limit) => Self(
                (
                    "Error: Gas startup cost is too expensive. Args might be too large.".into(),
                    14,
                )
                    .into(),
                gas_limit,
            ),
        }
    }
}

impl From<VmResultStatus> for ExecutionResult {
    fn from(value: VmResultStatus) -> Self {
        Ok(value)
    }
}

pub type ExecutionResult<T = VmResultStatus, E = VmResultStatus> = core::result::Result<T, E>;

pub struct ExitInfoWithGasUsed(pub ExitInfo, pub u64);

impl From<ExecutionResult> for ExitInfoWithGasUsed {
    fn from(value: ExecutionResult) -> Self {
        match value {
            Ok(ok) => ok.into(),
            Err(err) => err.into(),
        }
    }
}
