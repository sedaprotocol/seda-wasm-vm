use wasmer::{Function, FunctionEnv, FunctionEnvMut, Store, WasmPtr};

use crate::{context::VmContext, errors::Result, RuntimeError};

pub fn call_result_value_length_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn call_result_value_length(env: FunctionEnvMut<'_, VmContext>) -> Result<u32> {
        let ctx = env.data();
        let call_value = ctx.call_result_value.read();

        Ok(call_value.len() as u32)
    }

    Function::new_typed_with_env(store, vm_context, call_result_value_length)
}

pub fn call_result_value_write_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn call_result_value(
        env: FunctionEnvMut<'_, VmContext>,
        result_data_ptr: WasmPtr<u8>,
        result_data_length: u32,
    ) -> Result<()> {
        let ctx = env.data();
        let memory = ctx.memory_view(&env);

        let target = result_data_ptr.slice(&memory, result_data_length)?;
        if target.is_empty() {
            return Err(RuntimeError::InvalidMemoryAccess(
                "call_result_write: result_data_ptr is empty cannot write to it",
            ));
        }

        let mut call_result_value = ctx.call_result_value.write();
        let call_value = std::mem::replace(&mut *call_result_value, Vec::with_capacity(0));
        drop(call_result_value);

        if call_value.is_empty() || call_value.len() as u32 != result_data_length {
            return Err(RuntimeError::InvalidMemoryAccess(
                "call_result_write: result_data_ptr length does not match call_value length",
            ));
        }

        for index in 0..result_data_length as u64 {
            if target.read(index).is_err() {
                return Err(RuntimeError::InvalidMemoryAccess(
                    "call_result_write: result_data_ptr length does not match result_data_length",
                ));
            }

            target.index(index).write(call_value[index as usize])?;
        }

        Ok(())
    }

    Function::new_typed_with_env(store, vm_context, call_result_value)
}
