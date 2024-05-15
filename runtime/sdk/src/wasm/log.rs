use super::raw;
use crate::Level;

pub fn _log(level: Level, msg: &str, line_info: &str) {
    let level_str = serde_json::to_string(&level).unwrap();

    unsafe {
        raw::_log(
            level_str.as_ptr(),
            level_str.len() as i32,
            msg.as_ptr(),
            msg.len() as i64,
            line_info.as_ptr(),
            line_info.len() as i64,
        );
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {{
				let _msg = format!($($arg)*);
				let _line_info = format!("{}:{}", file!(), line!());
				$crate::wasm::_log($level, &_msg, &_line_info)
    }};
}

pub use log;
