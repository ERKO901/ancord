use std::time::{SystemTime, UNIX_EPOCH};

/// Sekundy od epochy Unixu.
pub fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}
