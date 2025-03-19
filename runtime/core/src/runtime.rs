use core::str;
use std::io::Read;

use seda_runtime_sdk::{ExecutionResult, ExitInfo, VmCallData, VmResult, VmResultStatus};
use wasmer::Instance;
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};
use wasmer_wasix::{Pipe, WasiEnv, WasiRuntimeError};

use crate::{
    context::VmContext,
    metering::{GAS_PER_BYTE, GAS_STARTUP},
    runtime_context::RuntimeContext,
    vm_imports::create_wasm_imports,
};

/// Maximum size in bytes for VM execution results.
/// Prevents Wasmer runtime errors when values become too large during execution.
const MAX_VM_RESULT_SIZE_BYTES: usize = 96000;

fn internal_run_vm(
    call_data: VmCallData,
    mut context: RuntimeContext,
    stdout: &mut Vec<String>,
    stderr: &mut Vec<String>,
    stdout_limit: usize,
    stderr_limit: usize,
) -> ExecutionResult<(Vec<u8>, i32, u64)> {
    // _start is the default WASI entrypoint
    let function_name = call_data.clone().start_func.unwrap_or_else(|| "_start".to_string());

    let (stdout_tx, mut stdout_rx) = Pipe::channel();
    let (stderr_tx, mut stderr_rx) = Pipe::channel();

    // leftovers from upgrading to wasmer 4.2.4...
    // there has to be a cleaner way to do this
    // maybe actix to spawn a future that times out???
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = runtime.enter();

    let mut wasi_env = WasiEnv::builder(function_name.clone())
        .envs(call_data.envs.clone())
        .args(call_data.args.clone())
        .stdout(Box::new(stdout_tx))
        .stderr(Box::new(stderr_tx))
        .finalize(&mut context.wasm_store)
        .map_err(|_| VmResultStatus::WasiEnvInitializeFailure)?;

    let vm_context = VmContext::create_vm_context(&mut context.wasm_store, wasi_env.env.clone(), call_data.clone());

    let imports = create_wasm_imports(
        &mut context.wasm_store,
        &vm_context,
        &wasi_env,
        &context.wasm_module,
        &call_data,
    )
    .map_err(|_| VmResultStatus::FailedToCreateVMImports)?;

    let wasmer_instance = Instance::new(&mut context.wasm_store, &context.wasm_module, &imports)
        .map_err(|e| VmResultStatus::FailedToCreateWasmerInstance(e.to_string()))?;

    vm_context.as_mut(&mut context.wasm_store).instance = Some(wasmer_instance.clone());

    let env_mut = vm_context.as_mut(&mut context.wasm_store);
    env_mut.memory = Some(
        wasmer_instance
            .exports
            .get_memory("memory")
            .map_err(|_| VmResultStatus::FailedToGetWASMMemory)?
            .clone(),
    );

    wasi_env
        .initialize(&mut context.wasm_store, wasmer_instance.clone())
        .map_err(|_| VmResultStatus::FailedToGetWASMFn)?;

    tracing::debug!("Calling WASM entrypoint");
    let main_func = wasmer_instance
        .exports
        .get_function(&function_name)
        .map_err(|_| VmResultStatus::FailedToGetWASMFn)?;

    // Apply arguments gas cost
    if let Some(gas_limit) = call_data.gas_limit {
        let args_bytes_total = call_data.args.iter().fold(0, |acc, v| acc + v.len());
        // Gas startup costs (for spinning up the VM)
        let gas_cost = (GAS_PER_BYTE * args_bytes_total as u64) + GAS_STARTUP;

        if gas_cost < gas_limit {
            set_remaining_points(&mut context.wasm_store, &wasmer_instance, gas_limit - gas_cost);
        } else {
            set_remaining_points(&mut context.wasm_store, &wasmer_instance, 0);
        }
    }

    let runtime_result = main_func.call(&mut context.wasm_store, &[]);

    wasi_env.on_exit(&mut context.wasm_store, None);
    drop(_guard);
    drop(runtime);

    let mut exit_code: i32 = 0;

    if let Err(err) = runtime_result {
        tracing::error!("Error running WASM: {err:?}");

        // TODO this makes me think that wasm host functions should
        // return a different kind of error rather than a RuntimeError.
        if err.is::<crate::errors::RuntimeError>() {
            let runtime_error = err.downcast::<crate::errors::RuntimeError>().unwrap();
            stderr.push(format!("Runtime error: {runtime_error}"));
            exit_code = 252;
        } else {
            // we convert the error to a wasix error
            let wasix_error = WasiRuntimeError::from(err);

            if let Some(wasi_exit_code) = wasix_error.as_exit_code() {
                exit_code = wasi_exit_code.raw();
            }
        }
    }

    let gas_used: u64 = if let Some(gas_limit) = call_data.gas_limit {
        match get_remaining_points(&mut context.wasm_store, &wasmer_instance) {
            MeteringPoints::Exhausted => {
                stderr.push("Ran out of gas".to_string());
                exit_code = 250;

                gas_limit
            }
            MeteringPoints::Remaining(remaining_gas) => gas_limit - remaining_gas,
        }
    } else {
        0
    };
    tracing::debug!("VM completed or out of gas");

    let mut execution_result = vm_context.as_ref(&context.wasm_store).result.lock();

    // Add size check for execution result
    if execution_result.len() > MAX_VM_RESULT_SIZE_BYTES {
        stderr.push(format!(
            "Result size ({} bytes) exceeds maximum allowed size ({} bytes)",
            execution_result.len(),
            MAX_VM_RESULT_SIZE_BYTES
        ));
        return Err(VmResultStatus::ResultSizeExceeded);
    }

    // Under the hood read_to_string called str::from_utf8
    // though it did it in chunks, but I think this is fine.
    let mut stdout_buffer = vec![0; stdout_limit];
    let bytes_read = stdout_rx
        .read(&mut stdout_buffer)
        .map_err(|_| VmResultStatus::FailedToConvertVMPipeToString)?;

    if bytes_read > 0 {
        // push the buffer but cap at stdout_limit in bytes
        stdout.push(
            str::from_utf8(&stdout_buffer)
                .map_err(|_| VmResultStatus::FailedToConvertVMPipeToString)?
                .to_string(),
        );
    }

    let mut stderr_buffer = vec![0; stderr_limit];
    let bytes_read = stderr_rx
        .read(&mut stderr_buffer)
        .map_err(|_| VmResultStatus::FailedToGetWASMStderr)?;

    if bytes_read > 0 {
        stderr.push(
            str::from_utf8(&stderr_buffer)
                .map_err(|_| VmResultStatus::FailedToConvertVMPipeToString)?
                .to_string(),
        );
    }

    Ok((std::mem::take(&mut execution_result), exit_code, gas_used))
}

pub fn start_runtime(
    call_data: VmCallData,
    context: RuntimeContext,
    stdout_limit: usize,
    stderr_limit: usize,
) -> VmResult {
    tracing::debug!("Starting runtime");
    let mut stdout: Vec<String> = vec![];
    let mut stderr: Vec<String> = vec![];

    let vm_execution_result = internal_run_vm(call_data, context, &mut stdout, &mut stderr, stdout_limit, stderr_limit);

    tracing::info!("TALLY VM execution completed");
    match vm_execution_result {
        Ok((result, exit_code, gas_used)) => {
            tracing::info!("TALLY VM gas used: {gas_used}");
            VmResult {
                stdout,
                stderr,
                gas_used,
                exit_info: ExitInfo {
                    exit_code,
                    exit_message: match exit_code {
                        0 => "Ok".to_string(),
                        _ => "Not ok".to_string(),
                    },
                },
                result: Some(result),
            }
        }
        Err(error) => VmResult {
            stdout,
            stderr,
            result: None,
            exit_info: error.into(),
            gas_used: 0,
        },
    }
}
