mod context;
mod core_vm_imports;
mod errors;

mod memory;
pub mod metering;
mod resources_dir;
mod runtime;
mod runtime_context;
mod safe_wasi_imports;
mod tally_vm_imports;
pub mod vm;
mod vm_imports;
mod wasi_vm_imports;
pub mod wasm_cache;

use std::path::Path;

pub const WASMER_VERSION: &str = env!("WASMER_VERSION");
pub const WASMER_TYPES_VERSION: &str = env!("WASMER_TYPES_VERSION");
pub const WASMER_MIDDLEWARES_VERSION: &str = env!("WASMER_MIDDLEWARES_VERSION");
pub const WASMER_WASIX_VERSION: &str = env!("WASMER_WASIX_VERSION");
const VERSION_FILE_NAME: &str = concat!(
    env!("WASMER_VERSION"),
    "-",
    env!("WASMER_TYPES_VERSION"),
    "-",
    env!("WASMER_MIDDLEWARES_VERSION"),
    "-",
    env!("WASMER_WASIX_VERSION")
);

// non-test build: pure const fn with no allocation
#[cfg(not(feature = "test-utils"))]
pub const fn get_version_file_name() -> &'static str {
    VERSION_FILE_NAME
}

#[cfg(feature = "test-utils")]
mod test_override {
    use std::sync::Mutex;

    use super::VERSION_FILE_NAME;

    // allows overwriting on each call
    static TEST_OVERRIDE: Mutex<Option<&'static str>> = Mutex::new(None);

    pub fn get_version_file_name() -> &'static str {
        TEST_OVERRIDE.lock().unwrap().unwrap_or(VERSION_FILE_NAME)
    }

    pub fn set_test_version_file_name(val: &'static str) {
        *TEST_OVERRIDE.lock().unwrap() = Some(val);
    }
}

pub use context::VmContext;
pub use core_vm_imports::create_custom_core_imports;
pub use errors::RuntimeError;
pub use runtime::start_runtime;
pub use runtime_context::RuntimeContext;
pub use safe_wasi_imports::*;
#[cfg(feature = "test-utils")]
pub use test_override::*;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

pub fn init_logger(sedad_home: &Path) -> (impl tracing::Subscriber + Send + Sync, WorkerGuard) {
    let level_filter = EnvFilter::builder()
        .with_default_directive(
            if cfg!(debug_assertions) {
                LevelFilter::TRACE
            } else {
                LevelFilter::INFO
            }
            .into(),
        )
        .from_env_lossy();
    let file_appender = tracing_appender::rolling::daily(sedad_home.join("sedavm_logs"), "log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let layer = fmt::Layer::new().with_writer(non_blocking).with_ansi(false);
    let subscriber = tracing_subscriber::registry().with(level_filter).with(layer);
    (subscriber, guard)
}
