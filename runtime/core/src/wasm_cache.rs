use std::{fs::File, io::Write, path::PathBuf};

use sha3::{Digest, Keccak256};
use wasmer::{Module, Store};

use crate::{errors::Result, resources_dir::resources_home_dir};

pub const WASM_CACHE_FOLDER_NAME: &str = "wasm_cache";

fn create_cache_path<ID: ToString>(id: ID) -> Result<PathBuf> {
    let mut wasm_cache_path = resources_home_dir();
    wasm_cache_path.push(WASM_CACHE_FOLDER_NAME);

    std::fs::create_dir_all(&wasm_cache_path)?;
    wasm_cache_path.push(id.to_string());

    Ok(wasm_cache_path)
}

pub fn wasm_cache_id<T: AsRef<[u8]>>(wasm_binary: T) -> String {
    let mut hash = Keccak256::new();
    hash.update(&wasm_binary);
    hex::encode(hash.finalize())
}

pub fn wasm_cache_store<ID: ToString, T: AsRef<[u8]>>(store: &Store, id: ID, wasm_binary: T) -> Result<Module> {
    let wasm_cache_path = create_cache_path(id)?;
    let module = Module::new(&store, &wasm_binary)?;

    let mut file = File::create(wasm_cache_path)?;
    let buffer = module.serialize()?;
    file.write_all(&buffer)?;

    Ok(module)
}

pub fn wasm_cache_load<ID: ToString>(store: &Store, id: ID) -> Result<Module> {
    let wasm_cache_path = create_cache_path(id)?;

    unsafe {
        let ret = Module::deserialize_from_file(&store, wasm_cache_path.clone());

        if ret.is_err() {
            // If an error occurs while deserializing then we can not trust it anymore
            // so delete the cache file
            let _ = std::fs::remove_file(wasm_cache_path);
        }

        Ok(ret?)
    }
}
