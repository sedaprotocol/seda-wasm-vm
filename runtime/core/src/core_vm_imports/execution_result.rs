use wasmer::{Function, FunctionEnv, FunctionEnvMut, Store, WasmPtr};

use crate::{context::VmContext, errors::Result, metering::apply_gas_cost};

pub fn execution_result_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn execution_result(
        mut env: FunctionEnvMut<'_, VmContext>,
        result_ptr: WasmPtr<u8>,
        result_length: i32,
    ) -> Result<()> {
        // Return error if length is negative
        if result_length < 0 {
            return Err(crate::RuntimeError::Unknown("Negative length provided".to_string()));
        }

        apply_gas_cost(
            crate::metering::ExternalCallType::ExecutionResult(result_length as u64),
            &mut env,
        )?;

        let ctx = env.data();
        let memory = ctx.memory_view(&env);

        let result = result_ptr.slice(&memory, result_length as u32)?.read_to_vec()?;
        let mut vm_result = ctx.result.lock();
        *vm_result = result;
        vm_result.shrink_to_fit();

        Ok(())
    }

    Function::new_typed_with_env(store, vm_context, execution_result)
}
