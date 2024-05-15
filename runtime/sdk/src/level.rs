use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Level {
    Debug,
    Error,
    Info,
    Trace,
    Warn,
}

// TODO only log line_info with a config option.
impl Level {
    pub fn log(self, message: &str, line_info: &str) {
        let message = format!("{message}\n    at {line_info}");
        match self {
            Level::Debug => tracing::debug!(message),
            Level::Error => tracing::error!(message),
            Level::Info => tracing::info!(message),
            Level::Trace => tracing::trace!(message),
            Level::Warn => tracing::warn!(message),
        }
    }
}
