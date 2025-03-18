use anyhow::Result;

#[link(wasm_import_module = "seda_v1")]
extern "C" {
    pub fn secp256k1_verify(
        message: *const u8,
        message_length: i64,
        signature: *const u8,
        signature_length: i32,
        public_key: *const u8,
        public_key_length: i32,
    ) -> u8;
}

pub fn import_length_overflow() -> Result<()> {
    let result = [1, 2, 3];
    unsafe {
        secp256k1_verify(result.as_ptr(), u32::MAX as i64, result.as_ptr(), 0, result.as_ptr(), 0);
    }

    Ok(())
}
