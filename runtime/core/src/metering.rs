use wasmer::wasmparser::Operator;

/// Gas cost for each operator
/// TODO: For now we give everything an equal gas cost, we should expand this
pub fn get_wasm_operation_cost(_operator: &Operator) -> u64 {
    1
}
