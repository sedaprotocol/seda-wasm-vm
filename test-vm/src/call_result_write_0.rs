#[link(wasm_import_module = "seda_v1")]
extern "C" {
    pub fn call_result_write(result: *const u8, result_length: u32);
}

pub fn call_result_write_0() {
    let result = [];
    unsafe {
        call_result_write(result.as_ptr(), 1);
    }
}
