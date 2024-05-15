pub use bn254::{PrivateKey as Bn254PrivateKey, PublicKey as Bn254PublicKey, Signature as Bn254Signature};

use super::raw;

pub fn bn254_verify(message: &[u8], signature: &Bn254Signature, public_key: &Bn254PublicKey) -> bool {
    let message_len = message.len() as i64;
    let signature_bytes = signature.to_uncompressed().expect("Signature should be valid");
    let signature_length = signature_bytes.len() as i64;
    let public_key_bytes = public_key.to_uncompressed().expect("Public Key should be valid");
    let public_key_length = public_key_bytes.len() as i64;

    let result = unsafe {
        raw::bn254_verify(
            message.as_ptr(),
            message_len,
            signature_bytes.as_ptr(),
            signature_length,
            public_key_bytes.as_ptr(),
            public_key_length,
        )
    };

    match result {
        0 => false,
        1 => true,
        _ => panic!("Bn254 verify returned invalid bool in u8: {}", result),
    }
}
