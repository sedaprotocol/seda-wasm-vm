use lazy_static::lazy_static;

lazy_static! {
    pub static ref SAFE_WASI_IMPORTS: Vec<String> = {
        [
            // "execution_result",
            // "http_fetch",
            // "call_result_write",
            "args_get",
            "args_sizes_get",
            "proc_exit",
            // "random_get",
            "fd_write",
            "environ_get",
            "environ_sizes_get",
        ]
        .iter()
        .map(|import| import.to_string())
        .collect()
    };
}
