#[link(wasm_import_module = "seda_v1")]
extern "C" {
    pub fn call_result_write(result: *const u8, result_length: u32);
    pub fn secp256k1_verify(
        message: *const u8,
        message_length: i64,
        signature: *const u8,
        signature_length: i32,
        public_key: *const u8,
        public_key_length: i32,
    ) -> u8;
}

pub fn cannot_spam_call_result_write() {
    let message = b"Hello, SEDA!";
    let signature = hex::decode("58376cc76f4d4959b0adf8070ecf0079db889915a75370f6e39a8451ba5be0c35f091fa4d2fda3ced5b6e6acd1dbb4a45f2c6a1e643622ee4cf8b802b373d38f").unwrap();
    let public_key = hex::decode("02a2bebd272aa28e410cc74cef28e5ce74a9ffc94caf817ed9bd23b01ce2068c7b").unwrap();

    let message_len = message.len() as i64;
    let signature_bytes = signature.to_vec();
    let signature_length = signature_bytes.len() as i32;
    let public_key_bytes = public_key.to_vec();
    let public_key_length = public_key_bytes.len() as i32;

    let result_length: u8 = unsafe {
        secp256k1_verify(
            message.as_ptr(),
            message_len,
            signature_bytes.as_ptr(),
            signature_length,
            public_key_bytes.as_ptr(),
            public_key_length,
        )
    };

    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        // this is fine
        call_result_write(result_data_ptr.as_mut_ptr(), result_length.into());
        // this second one should error
        call_result_write(result_data_ptr.as_mut_ptr(), result_length.into());
    }
}
