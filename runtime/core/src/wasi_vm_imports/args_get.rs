use wasmer::{Extern, Function, FunctionEnv, FunctionEnvMut, Store, Value, WasmPtr};
use wasmer_wasix::types::wasi::Errno;

use crate::{errors::Result, metering::apply_gas_cost, RuntimeError, VmContext};

pub fn args_get_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn args_get(
        mut env: FunctionEnvMut<'_, VmContext>,
        argv: WasmPtr<WasmPtr<u8>>,
        argv_buf: WasmPtr<u8>,
    ) -> Result<Errno> {
        let ctx = env.data().clone();
        apply_gas_cost(
            crate::metering::ExternalCallType::ArgsGet(ctx.call_data.args_bytes_len() as u64),
            &mut env,
        )?;

        let wasi_import_obj = ctx
            .wasi_imports
            .clone()
            .ok_or_else(|| RuntimeError::Unknown("Failed to get wasi_import_obj".to_string()))?;

        let wasi_version = ctx
            .wasi_version
            .ok_or_else(|| RuntimeError::Unknown("Failed to get wasi_version".to_string()))?;

        let export = wasi_import_obj
            .get_export(wasi_version.get_namespace_str(), "args_get")
            .ok_or_else(|| RuntimeError::Unknown("Failed to get export".to_string()))?;

        if let Extern::Function(func) = export {
            let result = func.call(&mut env, &[Value::from(argv.offset()), Value::from(argv_buf.offset())])?;

            let result_code = result[0]
                .i32()
                .ok_or_else(|| RuntimeError::Unknown("Failed to get result code".to_string()))?;

            if result_code == 0 {
                return Ok(Errno::Success);
            }

            return Ok(Errno::Inval);
        }

        Err(RuntimeError::Unknown(
            "Could not call external args_get function".to_string(),
        ))
    }

    Function::new_typed_with_env(store, vm_context, args_get)
}

pub fn args_sizes_get_import_obj(store: &mut Store, wasi_env: &FunctionEnv<VmContext>) -> Function {
    fn args_sizes_get(
        mut env: FunctionEnvMut<'_, VmContext>,
        argc: WasmPtr<u8>,
        argv_buf_size: WasmPtr<u8>,
    ) -> Result<Errno> {
        let ctx = env.data().clone();
        apply_gas_cost(
            crate::metering::ExternalCallType::ArgsSizesGet(ctx.call_data.args_bytes_len() as u64),
            &mut env,
        )?;

        let wasi_import_obj = ctx
            .wasi_imports
            .clone()
            .ok_or_else(|| RuntimeError::Unknown("Failed to get wasi_import_obj".to_string()))?;

        let wasi_version = ctx
            .wasi_version
            .ok_or_else(|| RuntimeError::Unknown("Failed to get wasi_version".to_string()))?;

        let export = wasi_import_obj
            .get_export(wasi_version.get_namespace_str(), "args_sizes_get")
            .ok_or_else(|| RuntimeError::Unknown("Failed to get export".to_string()))?;

        if let Extern::Function(func) = export {
            let result = func.call(
                &mut env,
                &[Value::from(argc.offset()), Value::from(argv_buf_size.offset())],
            )?;

            let result_code = result[0]
                .i32()
                .ok_or_else(|| RuntimeError::Unknown("Failed to get result code".to_string()))?;
            if result_code == 0 {
                return Ok(Errno::Success);
            }

            return Ok(Errno::Inval);
        }

        Err(RuntimeError::Unknown(
            "Could not call external args_sizes_get function".to_string(),
        ))
    }

    Function::new_typed_with_env(store, wasi_env, args_sizes_get)
}
