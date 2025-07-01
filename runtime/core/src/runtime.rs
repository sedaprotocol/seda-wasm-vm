use core::str;
use std::io::Read;

use tokio::task;
use wasmer::Instance;
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};
use wasmer_wasix::{Pipe, WasiEnv, WasiRuntimeError};

use crate::{
    context::VmContext,
    metering::vm_gas_startup_cost,
    runtime_context::RuntimeContext,
    vm::*,
    vm_imports::create_wasm_imports,
};

/// Maximum size in bytes for VM execution results.
/// Prevents Wasmer runtime errors when values become too large during execution.
const MAX_VM_RESULT_SIZE_BYTES: usize = 96000;

fn internal_run_vm(
    call_data: VmCallData,
    context: RuntimeContext,
    stdout: &mut Vec<String>,
    stderr: &mut Vec<String>,
    stdout_limit: usize,
    stderr_limit: usize,
) -> ExecutionResult<(Vec<u8>, i32, u64)> {
    let mut local_stdout = std::mem::take(stdout);
    let mut local_stderr = std::mem::take(stderr);

    let (res, local_stdout, local_stderr) = task::block_in_place(move || {
        let res = _internal_run_vm(
            call_data,
            context,
            &mut local_stdout,
            &mut local_stderr,
            stdout_limit,
            stderr_limit,
        );

        (res, local_stdout, local_stderr)
    });
    *stdout = local_stdout;
    *stderr = local_stderr;

    res
}

fn _internal_run_vm(
    call_data: VmCallData,
    mut context: RuntimeContext,
    stdout: &mut Vec<String>,
    stderr: &mut Vec<String>,
    stdout_limit: usize,
    stderr_limit: usize,
) -> ExecutionResult<(Vec<u8>, i32, u64)> {
    // If the gas limit is set, we need to calculate the startup cost
    let gas_cost = if let Some(gas_limit) = call_data.gas_limit {
        let Ok(Some(gas_cost)): Result<Option<u64>, _> = vm_gas_startup_cost(&call_data.args) else {
            return Err(VmResultStatus::GasStartupCostTooHigh(gas_limit));
        };
        if gas_cost > gas_limit {
            return Err(VmResultStatus::GasStartupCostTooHigh(gas_limit));
        }
        gas_cost
    } else {
        0
    };

    // _start is the default WASI entrypoint
    let function_name = call_data.clone().start_func.unwrap_or_else(|| "_start".to_string());

    let (stdout_tx, mut stdout_rx) = Pipe::channel();
    let (stderr_tx, mut stderr_rx) = Pipe::channel();

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
        .map_err(|e| VmResultStatus::FailedToCreateWasmerInstance(e.to_string(), gas_cost))?;

    vm_context.as_mut(&mut context.wasm_store).instance = Some(wasmer_instance.clone());

    let env_mut = vm_context.as_mut(&mut context.wasm_store);
    env_mut.memory = Some(
        wasmer_instance
            .exports
            .get_memory("memory")
            .map_err(|_| VmResultStatus::FailedToGetWASMMemory(gas_cost))?
            .clone(),
    );

    wasi_env
        .initialize(&mut context.wasm_store, wasmer_instance.clone())
        .map_err(|_| VmResultStatus::FailedToGetWASMFn(gas_cost))?;

    // spawn the actual VM run on a separate thread
    #[allow(clippy::type_complexity)]
    let ((exec_bytes, exit_code, gas_used), local_stderr) =
        std::thread::spawn(move || -> Result<((Vec<u8>, i32, u64), Vec<String>), VmResultStatus> {
            tracing::debug!("Calling WASM entrypoint");
            let main_func = wasmer_instance
                .exports
                .get_function(&function_name)
                .map_err(|_| VmResultStatus::FailedToGetWASMFn(gas_cost))?;

            // Apply startup cost before calling the main function
            if let Some(gas_limit) = call_data.gas_limit {
                set_remaining_points(&mut context.wasm_store, &wasmer_instance, gas_limit - gas_cost);
            }

            let runtime_result = main_func.call(&mut context.wasm_store, &[]);
            wasi_env.on_exit(&mut context.wasm_store, None);

            let mut exit_code: i32 = 0;
            let mut thread_stderr = Vec::new();

            if let Err(err) = runtime_result {
                tracing::error!("Error running WASM: {err:?}");
                if err.is::<crate::errors::RuntimeError>() {
                    let runtime_error = err.downcast::<crate::errors::RuntimeError>().unwrap();
                    thread_stderr.push(format!("Runtime error: {runtime_error}"));
                    exit_code = 252;
                } else {
                    let wasix_error = WasiRuntimeError::from(err);
                    if let Some(wasi_exit_code) = wasix_error.as_exit_code() {
                        exit_code = wasi_exit_code.raw();
                    } else {
                        exit_code = 252;
                    }
                }
            }

            let gas_used = if let Some(gas_limit) = call_data.gas_limit {
                match get_remaining_points(&mut context.wasm_store, &wasmer_instance) {
                    MeteringPoints::Exhausted => {
                        thread_stderr.push("Ran out of gas".to_string());
                        exit_code = 250;
                        gas_limit
                    }
                    MeteringPoints::Remaining(remaining) => gas_limit - remaining,
                }
            } else {
                0
            };

            tracing::debug!("VM completed or out of gas");

            let mut execution_result = vm_context.as_ref(&context.wasm_store).result.lock();
            if execution_result.len() > MAX_VM_RESULT_SIZE_BYTES {
                thread_stderr.push(format!(
                    "Result size ({} bytes) exceeds maximum allowed size ({} bytes)",
                    execution_result.len(),
                    MAX_VM_RESULT_SIZE_BYTES
                ));
                return Err(VmResultStatus::ResultSizeExceeded(gas_used));
            }
            let exec_bytes = std::mem::take(&mut *execution_result);

            Ok(((exec_bytes, exit_code, gas_used), thread_stderr))
        })
        .join()
        .expect("ah")?;

    // merge any runtime-error messages into outer stderr
    for msg in local_stderr {
        stderr.push(msg);
    }

    // collect WASM stdout
    let mut stdout_buffer = Vec::new();
    let bytes_read = stdout_rx
        .read_to_end(&mut stdout_buffer)
        .map_err(|_| VmResultStatus::FailedToGetWASMStdout(gas_used))?;

    if bytes_read > 0 {
        stdout.push(
            str::from_utf8(&stdout_buffer[..stdout_limit.min(stdout_buffer.len())])
                .map_err(|_| VmResultStatus::FailedToConvertVMPipeToString("stdout".to_string(), gas_used))?
                .to_string(),
        );
    }

    // collect WASM stderr
    let mut stderr_buffer = Vec::new();
    let bytes_read = stderr_rx
        .read_to_end(&mut stderr_buffer)
        .map_err(|_| VmResultStatus::FailedToGetWASMStderr(gas_used))?;

    if bytes_read > 0 {
        stderr.push(
            str::from_utf8(&stderr_buffer[..stderr_limit.min(stderr_buffer.len())])
                .map_err(|_| VmResultStatus::FailedToConvertVMPipeToString("stderr".to_string(), gas_used))?
                .to_string(),
        );
    }

    Ok((exec_bytes, exit_code, gas_used))
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
        Err(error) => {
            let info: ExitInfoWithGasUsed = error.into();
            VmResult {
                stdout,
                stderr,
                result: None,
                gas_used: info.1,
                exit_info: info.0,
            }
        }
    }
}
