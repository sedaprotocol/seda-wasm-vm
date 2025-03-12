use wasmer::{Extern, Function, FunctionEnv, FunctionEnvMut, Store, Value, WasmPtr};
use wasmer_wasix::types::wasi::Errno;

use crate::{errors::Result, metering::apply_gas_cost, RuntimeError, VmContext};

pub fn fd_write_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn fd_write(
        mut env: FunctionEnvMut<'_, VmContext>,
        fd: u32,
        iovs: WasmPtr<u8>,
        iovs_len: u32,
        nwritten: WasmPtr<u32>,
    ) -> Result<Errno> {
        // Apply gas cost based on the number of iovs (I/O vectors)
        // Each iov represents a buffer to write, so we charge based on the number of buffers
        apply_gas_cost(crate::metering::ExternalCallType::FdWrite(iovs_len as u64), &mut env)?;

        let ctx = env.data().clone();
        let wasi_import_obj = ctx
            .wasi_imports
            .clone()
            .ok_or_else(|| RuntimeError::Unknown("Failed to get wasi_import_obj".to_string()))?;

        let wasi_version = ctx
            .wasi_version
            .ok_or_else(|| RuntimeError::Unknown("Failed to get wasi_version".to_string()))?;

        let export = wasi_import_obj
            .get_export(wasi_version.get_namespace_str(), "fd_write")
            .ok_or_else(|| RuntimeError::Unknown("Failed to get export".to_string()))?;

        if let Extern::Function(func) = export {
            let result = func.call(
                &mut env,
                &[
                    Value::from(fd),
                    Value::from(iovs.offset()),
                    Value::from(iovs_len),
                    Value::from(nwritten.offset()),
                ],
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
            "Could not call external fd_write function".to_string(),
        ))
    }

    Function::new_typed_with_env(store, vm_context, fd_write)
}
