use sha3::{Digest, Keccak256};
use wasmer::{Function, FunctionEnv, FunctionEnvMut, Store, WasmPtr};

use crate::{errors::Result, VmContext};

pub fn keccak256_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn keccak256(env: FunctionEnvMut<'_, VmContext>, message_ptr: WasmPtr<u8>, message_length: u32) -> Result<u32> {
        let ctx = env.data();
        let memory = ctx.memory_view(&env);

        let message = message_ptr.slice(&memory, message_length)?.read_to_vec()?;
        let hash = Keccak256::digest(message);

        let mut call_value = ctx.call_result_value.write();
        *call_value = hash.to_vec();

        Ok(call_value.len() as u32)
    }

    Function::new_typed_with_env(store, vm_context, keccak256)
}
