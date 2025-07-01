pub const WASM_CACHE_FOLDER_NAME: &str = "wasm_cache";

// not used with singlepass compiler
// fn create_cache_path<ID: ToString>(sedad_home: &Path, id: ID) -> Result<PathBuf> {
//     let mut wasm_cache_path = resources_home_dir(sedad_home);
//     wasm_cache_path.push(WASM_CACHE_FOLDER_NAME);

//     std::fs::create_dir_all(&wasm_cache_path)?;
//     wasm_cache_path.push(id.to_string());

//     Ok(wasm_cache_path)
// }
pub fn wasm_cache_id<T: AsRef<[u8]>>(wasm_binary: T) -> String {
    seahash::hash(wasm_binary.as_ref()).to_string()
}

// not used with singlepass compiler
// pub fn wasm_cache_store<ID: ToString, T: AsRef<[u8]>>(
//     sedad_home: &Path,
//     store: &Store,
//     id: ID,
//     wasm_binary: T,
// ) -> Result<Module> {
//     let wasm_cache_path = create_cache_path(sedad_home, id)?;
//     let module = Module::new(&store, &wasm_binary)?;

//     let mut file = File::create(wasm_cache_path)?;
//     let buffer = module.serialize()?;
//     file.write_all(&buffer)?;

//     Ok(module)
// }

// not used with singlepass compiler
// pub fn wasm_cache_load<ID: ToString>(sedad_home: &Path, store: &Store, id: ID) -> Result<Module> {
//     let wasm_cache_path = create_cache_path(sedad_home, id)?;

//     unsafe {
//         let ret = Module::deserialize_from_file(&store, wasm_cache_path.clone());

//         if ret.is_err() {
//             // If an error occurs while deserializing then we can not trust it anymore
//             // so delete the cache file
//             let _ = std::fs::remove_file(wasm_cache_path);
//         }

//         Ok(ret?)
//     }
// }
