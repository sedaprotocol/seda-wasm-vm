use core::hash::Hash;
use std::collections::HashMap;

use super::raw;
use crate::{FromBytes, Result, ToBytes};

fn increment_bytes(b256: &mut [u8], mut amount: u64) -> u64 {
    let mut i = b256.len() - 1;

    while amount > 0 {
        amount += b256[i] as u64;
        b256[i] = amount as u8;
        amount /= 256;

        if i == 0 {
            break;
        }
        i -= 1;
    }

    amount
}

/**
 * Gets all the key => value pairs that start with the given prefix
 */
pub fn shared_memory_range_by_prefix<F: ToBytes + Clone, K, V>(prefix: F) -> Result<HashMap<K, V>>
where
    K: FromBytes + Eq + Hash,
    V: FromBytes,
{
    let from = prefix.clone();
    let mut to = prefix.to_bytes().eject();
    increment_bytes(&mut to, 1);

    shared_memory_range(from, to)
}

/**
 * Gets all the key => value pairs given a range using `from` (inclusive) and `to` (exclusive)
 */
pub fn shared_memory_range<F: ToBytes, T: ToBytes, K, V>(from: F, to: T) -> Result<HashMap<K, V>>
where
    K: FromBytes + Eq + Hash,
    V: FromBytes,
{
    let from = from.to_bytes();
    let from_len = from.len() as u32;

    let to = to.to_bytes();
    let to_len = to.len() as u32;

    let result_len = unsafe { raw::shared_memory_range(from.as_ptr(), from_len, to.as_ptr(), to_len) };
    let mut result_data_ptr = vec![0; result_len as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_len);
    }

    let raw_result: HashMap<String, Vec<u8>> = serde_json::from_slice(&result_data_ptr)?;
    let mut result: HashMap<K, V> = HashMap::new();

    for (key, value) in raw_result.iter() {
        result.insert(K::from_bytes(key.as_bytes())?, V::from_bytes(value)?);
    }

    Ok(result)
}

pub fn shared_memory_get<K: ToBytes, V: FromBytes>(key: K) -> Result<V> {
    let key = key.to_bytes();
    let key_len = key.len() as i64;
    let value_len = unsafe { raw::shared_memory_read_length(key.as_ptr(), key_len) };
    let result_data_ptr = vec![0; value_len as usize];

    unsafe {
        raw::shared_memory_read(key.as_ptr(), key_len, result_data_ptr.as_ptr(), value_len);
    }

    V::from_bytes_vec(result_data_ptr)
}

pub fn shared_memory_set<K: ToBytes, V: ToBytes>(key: K, value: V) {
    let key = key.to_bytes();
    let key_len = key.len() as i64;
    let value = value.to_bytes();
    let value_len = value.len() as i64;

    unsafe {
        raw::shared_memory_write(key.as_ptr(), key_len, value.as_ptr(), value_len);
    }
}

pub fn shared_memory_remove<K: ToBytes>(key: K) {
    let key = key.to_bytes();
    let key_len = key.len() as i64;

    unsafe {
        raw::shared_memory_remove(key.as_ptr(), key_len);
    }
}

pub fn shared_memory_contains_key<K: ToBytes>(key: K) -> bool {
    let key = key.to_bytes();
    let key_len = key.len() as i64;

    let result = unsafe { raw::shared_memory_contains_key(key.as_ptr(), key_len) };

    match result {
        0 => false,
        1 => true,
        _ => unreachable!("Shared memory returned invalid bool in u8: {}", result),
    }
}
