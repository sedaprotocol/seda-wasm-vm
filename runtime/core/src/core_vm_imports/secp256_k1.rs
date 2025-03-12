use k256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use sha3::{Digest, Keccak256};
use wasmer::{Function, FunctionEnv, FunctionEnvMut, Store, WasmPtr};

use crate::{context::VmContext, errors::Result, metering::apply_gas_cost, RuntimeError};

/// Verifies a `Secp256k1` ECDSA signature.
///
/// Inputs:
///     - message (any payload in bytes)
///     - signature (r and s as two 32-byte values or a 64-byte concatenated value)
///     - public_key (bytes as a compressed point on the Secp256k1 curve)
///
/// Output:
///     - u8 (boolean, 1 for true)
pub fn secp256k1_verify_import_obj(store: &mut Store, vm_context: &FunctionEnv<VmContext>) -> Function {
    fn secp256k1_verify(
        mut env: FunctionEnvMut<'_, VmContext>,
        message: WasmPtr<u8>,
        message_length: i64,
        signature: WasmPtr<u8>,
        signature_length: i32,
        public_key: WasmPtr<u8>,
        public_key_length: i32,
    ) -> Result<u8> {
        // Return error if any length is negative
        if message_length < 0 || signature_length < 0 || public_key_length < 0 {
            return Err(RuntimeError::Unknown("Negative length provided".to_string()));
        }

        // Convert lengths to u64 and check for overflow before adding
        let message_len = u64::try_from(message_length).unwrap_or(0);
        let sig_len = u64::try_from(signature_length).unwrap_or(0);
        let pubkey_len = u64::try_from(public_key_length).unwrap_or(0);

        let total_len = message_len
            .checked_add(sig_len)
            .and_then(|sum| sum.checked_add(pubkey_len))
            .ok_or_else(|| RuntimeError::Unknown("Length overflow in secp256k1_verify".to_string()))?;

        apply_gas_cost(crate::metering::ExternalCallType::Secp256k1Verify(total_len), &mut env)?;

        let ctx = env.data();
        let memory = ctx.memory_view(&env);

        // Fetch function arguments as Vec<u8>
        let message: Vec<u8> = message.slice(&memory, message_length as u32)?.read_to_vec()?;
        let signature = signature.slice(&memory, signature_length as u32)?.read_to_vec()?;
        let public_key = public_key.slice(&memory, public_key_length as u32)?.read_to_vec()?;

        // `Secp256k1` verification (using Keccak256 hashing)
        let public_key_obj = VerifyingKey::from_sec1_bytes(&public_key)?;
        let signature_obj = Signature::from_slice(&signature)?;
        let hashed_message = Keccak256::digest(&message);

        // Verify the signature using prehashed message
        Ok(public_key_obj
            .verify_prehash(&hashed_message, &signature_obj)
            .is_ok()
            .into())
    }

    Function::new_typed_with_env(store, vm_context, secp256k1_verify)
}
