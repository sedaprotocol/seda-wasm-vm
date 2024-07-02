mod context;
mod core_vm_imports;
mod errors;
mod metering;
mod resources_dir;
mod runtime;
mod runtime_context;
mod safe_wasi_imports;
mod tally_vm_imports;
mod vm_imports;
pub mod wasm_cache;

pub use context::VmContext;
pub use core_vm_imports::create_custom_core_imports;
pub use errors::RuntimeError;
pub use runtime::start_runtime;
pub use runtime_context::RuntimeContext;
pub use safe_wasi_imports::*;
pub use seda_runtime_sdk::{VmCallData, VmResult};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

pub fn init_logger() -> WorkerGuard {
    let level_filter = EnvFilter::try_from_default_env().unwrap_or_default();
    let file_appender = tracing_appender::rolling::daily("tally_vm_logs", "log");
    let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

    let mut file_logger = fmt::Layer::new().with_writer(non_blocking);
    file_logger.set_ansi(false);

    let subscriber = tracing_subscriber::registry().with(level_filter).with(file_logger);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set logger.");

    file_guard
}
