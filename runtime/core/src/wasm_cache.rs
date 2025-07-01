use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use wasmer::{Module, Store};

use crate::{
    errors::{Result, VmHostError},
    get_version_file_name,
    resources_dir::resources_home_dir,
};

pub const WASM_CACHE_FOLDER_NAME: &str = "wasm_cache";

fn create_cache_path(sedad_home: &Path, id: &str) -> Result<PathBuf> {
    let wasm_cache_path = resources_home_dir(sedad_home)
        .join(WASM_CACHE_FOLDER_NAME)
        .join(get_version_file_name());

    if !wasm_cache_path.exists() {
        std::fs::create_dir_all(&wasm_cache_path)?;
    }

    if wasm_cache_path.exists() && !wasm_cache_path.is_dir() {
        Err(VmHostError::InvalidCachePath(wasm_cache_path.display().to_string()))?;
    }

    Ok(wasm_cache_path.join(id))
}

pub fn wasm_cache_id<T: AsRef<[u8]>>(wasm_binary: T) -> String {
    seahash::hash(wasm_binary.as_ref()).to_string()
}

pub fn get_full_wasm_path_from_id(sedad_home: &Path, id: &str) -> PathBuf {
    resources_home_dir(sedad_home)
        .join(WASM_CACHE_FOLDER_NAME)
        .join(get_version_file_name())
        .join(id)
}

pub fn valid_wasm_cache_id(wasm_cache_path: &Path) -> bool {
    let version_dir = wasm_cache_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str());

    if version_dir.is_none() || version_dir.unwrap() != get_version_file_name() {
        return false;
    }

    true
}

pub fn wasm_cache_store<T: AsRef<[u8]>>(
    sedad_home: &Path,
    compile_store: &Store,
    store: &Store,
    id: &str,
    wasm_binary: T,
) -> Result<Module> {
    let wasm_cache_path = create_cache_path(sedad_home, id)?;
    let module = Module::new(&compile_store, &wasm_binary)?;

    let mut file = File::create(&wasm_cache_path)?;
    let buffer = module.serialize()?;
    file.write_all(&buffer)?;
    drop(module);

    let wasm_module = unsafe { Module::deserialize_from_file(&store, &wasm_cache_path)? };
    Ok(wasm_module)
}

pub fn wasm_cache_load(store: &Store, wasm_cache_path: &Path) -> Result<Module> {
    unsafe {
        let ret = Module::deserialize_from_file(&store, wasm_cache_path);

        if ret.is_err() {
            // If an error occurs while deserializing then we can not trust it anymore
            // so delete the cache file
            let _ = std::fs::remove_file(wasm_cache_path);
        }

        Ok(ret?)
    }
}
