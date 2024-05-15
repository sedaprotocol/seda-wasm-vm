use wasmer::{Function, FunctionEnv, FunctionEnvMut, Store, WasmPtr};

use crate::{context::VmContext, errors::Result};

pub fn execution_result_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn execution_result(env: FunctionEnvMut<'_, VmContext>, result_ptr: WasmPtr<u8>, result_length: i32) -> Result<()> {
        let ctx = env.data();
        let memory = ctx.memory_view(&env);

        let result = result_ptr.slice(&memory, result_length as u32)?.read_to_vec()?;
        let mut vm_result = ctx.result.lock();
        *vm_result = result;

        Ok(())
    }

    Function::new_typed_with_env(store, vm_context, execution_result)
}
