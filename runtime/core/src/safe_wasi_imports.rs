use lazy_static::lazy_static;

lazy_static! {
    pub static ref SAFE_WASI_IMPORTS: Vec<String> = {
        [
            "args_get",
            "args_sizes_get",
            "proc_exit",
            "fd_write",
            "environ_get",
            "environ_sizes_get",
        ]
        .iter()
        .map(|import| import.to_string())
        .collect()
    };
}
