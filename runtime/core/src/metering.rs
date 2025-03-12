use wasmer::{wasmparser::Operator, FunctionEnvMut, WASM_PAGE_SIZE};
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};

use crate::{
    errors::{Result, VmHostError},
    RuntimeError,
    VmContext,
};

pub fn is_accounting(operator: &Operator) -> bool {
    matches!(
        operator,
        Operator::Loop { .. }
            | Operator::End
            | Operator::If { .. }
            | Operator::Else
            | Operator::Br { .. }
            | Operator::BrTable { .. }
            | Operator::BrIf { .. }
            | Operator::Call { .. }
            | Operator::CallIndirect { .. }
            | Operator::Return
            | Operator::Throw { .. }
            | Operator::ThrowRef
            | Operator::Rethrow { .. }
            | Operator::Delegate { .. }
            | Operator::Catch { .. }
            | Operator::ReturnCall { .. }
            | Operator::ReturnCallIndirect { .. }
            | Operator::BrOnCast { .. }
            | Operator::BrOnCastFail { .. }
            | Operator::CallRef { .. }
            | Operator::ReturnCallRef { .. }
            | Operator::BrOnNull { .. }
            | Operator::BrOnNonNull { .. }
    )
}

const GAS_MULTIPLIER: u64 = 150;
const GAS_PER_OPERATION: u64 = 125 * GAS_MULTIPLIER;
const GAS_ACCOUNTING_MULTIPLIER: u64 = 3_000;
const GAS_MEMORY_GROW_BASE: u64 = 1_000_000;

// Gas for reading and writing bytes
pub const GAS_PER_BYTE: u64 = 10_000;
const GAS_PER_BYTE_EXECUTION_RESULT: u64 = 10_000_000;

pub const TERA_GAS: u64 = 1_000_000_000_000;
// Makes it so you can do roughly 30 http requests with the current gas calculations.
const GAS_HTTP_FETCH_BASE: u64 = TERA_GAS * 5;

const GAS_BN254_VERIFY_BASE: u64 = TERA_GAS;
// Makes it so you can do roughly 25 proxy http requests with the current gas calculations.
const GAS_PROXY_HTTP_FETCH_BASE: u64 = TERA_GAS * 7;
const GAS_SECP256K1_BASE: u64 = TERA_GAS;
const GAS_KECCAK256_BASE: u64 = TERA_GAS;
pub const GAS_STARTUP: u64 = TERA_GAS * 5;

// WASI Gas
const GAS_ARGS_GET_BASE: u64 = TERA_GAS;
const GAS_ARGS_SIZES_GET_BASE: u64 = TERA_GAS;
const GAS_ENVIRON_GET_BASE: u64 = TERA_GAS;
const GAS_ENVIRON_SIZES_GET_BASE: u64 = TERA_GAS;
const GAS_FD_WRITE_BASE: u64 = TERA_GAS;

/// Gas cost for each operator
pub fn get_wasm_operation_gas_cost(operator: &Operator) -> u64 {
    if is_accounting(operator) {
        return GAS_PER_OPERATION * GAS_ACCOUNTING_MULTIPLIER;
    }

    match operator {
        Operator::MemoryGrow { mem, mem_byte: _ } => {
            GAS_MEMORY_GROW_BASE + ((WASM_PAGE_SIZE as u64 * *mem as u64) * GAS_PER_BYTE)
        }
        _ => GAS_PER_OPERATION,
    }
}

#[derive(Debug)]
pub enum ExternalCallType {
    /// Takes as argument the bytes length
    ExecutionResult(u64),
    /// Takes as argument the bytes length
    HttpFetchRequest(u64),
    /// Takes as argument the bytes length
    HttpFetchResponse(u64),
    /// Takes as argument the length of the message
    Bn254Verify(u64),
    /// Takes as argument the bytes length
    ProxyHttpFetchRequest(u64),
    /// Takes as argument the length of the message
    Secp256k1Verify(u64),
    /// Takes as argument the length of the message
    Keccak256(u64),

    /// WASI Imports
    ArgsGet(u64),
    ArgsSizesGet(u64),
    EnvironGet(u64),
    EnvironSizesGet(u64),
    /// Takes as argument the number of I/O vectors
    FdWrite(u64),
}

pub fn check_enough_gas(gas_cost: u64, remaining_gas: u64, gas_limit: u64) -> Result<u64> {
    let gas_used = gas_limit - remaining_gas;

    if (gas_cost + gas_used) > gas_limit {
        return Err(RuntimeError::OutOfGas);
    }

    Ok(remaining_gas - gas_cost)
}

pub fn apply_gas_cost(external_call_type: ExternalCallType, env: &mut FunctionEnvMut<'_, VmContext>) -> Result<()> {
    let context: &VmContext = env.data();
    let instance = match &context.instance {
        None => Err(VmHostError::InstanceNotSet),
        Some(v) => Ok(v.clone()),
    }?;

    if let Some(gas_limit) = context.call_data.gas_limit {
        let remaining_gas = match get_remaining_points(env, &instance) {
            MeteringPoints::Exhausted => 0,
            MeteringPoints::Remaining(remaining_gas) => remaining_gas,
        };

        let gas_cost = match external_call_type {
            ExternalCallType::ExecutionResult(bytes_length) => GAS_PER_BYTE_EXECUTION_RESULT * bytes_length,
            ExternalCallType::HttpFetchRequest(bytes_length) => GAS_HTTP_FETCH_BASE + (GAS_PER_BYTE * bytes_length),
            ExternalCallType::HttpFetchResponse(bytes_length) => GAS_PER_BYTE * bytes_length,
            ExternalCallType::Bn254Verify(bytes_length) => GAS_BN254_VERIFY_BASE + (GAS_PER_BYTE * bytes_length),
            ExternalCallType::ProxyHttpFetchRequest(bytes_length) => {
                GAS_PROXY_HTTP_FETCH_BASE + (GAS_PER_BYTE * bytes_length)
            }
            ExternalCallType::Secp256k1Verify(bytes_length) => {
                GAS_SECP256K1_BASE + GAS_KECCAK256_BASE + (GAS_PER_BYTE * bytes_length)
            }
            ExternalCallType::Keccak256(bytes_length) => GAS_KECCAK256_BASE + (GAS_PER_BYTE * bytes_length),
            ExternalCallType::ArgsGet(bytes_length) => GAS_ARGS_GET_BASE + (GAS_PER_BYTE * bytes_length),
            ExternalCallType::ArgsSizesGet(bytes_length) => GAS_ARGS_SIZES_GET_BASE + (GAS_PER_BYTE * bytes_length),
            ExternalCallType::EnvironGet(bytes_length) => GAS_ENVIRON_GET_BASE + (GAS_PER_BYTE * bytes_length),
            ExternalCallType::EnvironSizesGet(bytes_length) => {
                GAS_ENVIRON_SIZES_GET_BASE + (GAS_PER_BYTE * bytes_length)
            }
            ExternalCallType::FdWrite(iovs_len) => GAS_FD_WRITE_BASE + (GAS_PER_BYTE * iovs_len),
        };

        let gas_left = check_enough_gas(gas_cost, remaining_gas, gas_limit)?;
        set_remaining_points(env, &instance, gas_left);
    }

    Ok(())
}
