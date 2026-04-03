use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current time in milliseconds since UNIX epoch.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX epoch")
        .as_millis() as i64
}
