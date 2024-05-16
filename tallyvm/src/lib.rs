use std::collections::HashMap;

use seda_runtime_sdk::{VmType, WasmId};
use seda_wasm_vm::{start_runtime, RuntimeContext, VmCallData, VmResult};

use crate::errors::Result;

mod errors;

pub fn execute_tally_vm(wasm_bytes: Vec<u8>, args: Vec<String>, envs: HashMap<String, String>) -> Result<VmResult> {
    let wasm_id = WasmId::Bytes(wasm_bytes);
    let runtime_context = RuntimeContext::new(&wasm_id)?;

    let result = start_runtime(
        VmCallData {
            call_id: None,
            wasm_id,
            args,
            envs,
            program_name: runtime_context.wasm_hash.to_string(),
            start_func: None,
            vm_type: VmType::Tally,
        },
        runtime_context,
    );

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::execute_tally_vm;

    #[test]
    fn test_execute_tally_vm() {
        let wasm_bytes = include_bytes!("../../debug.wasm");
        let result = execute_tally_vm(
            wasm_bytes.to_vec(),
            vec!["testHttpSuccess".to_string()],
            HashMap::default(),
        )
        .unwrap();

        result.stdout.iter().for_each(|line| print!("{}", line));

        dbg!(result);
    }
}
