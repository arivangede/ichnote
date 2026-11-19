//! Identifiers and timestamps.

use std::time::{SystemTime, UNIX_EPOCH};

pub use uuid::Uuid;

/// Creates a new time-ordered UUIDv7. All synced data uses these IDs.
#[must_use]
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

/// Current wall-clock time in unix milliseconds.
///
/// Only used for display and ordering; CRDT merges never depend on it.
#[must_use]
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_v7_and_ordered() {
        let a = new_id();
        let b = new_id();
        assert_eq!(a.get_version_num(), 7);
        assert!(a < b);
    }

    #[test]
    fn now_is_after_2025() {
        assert!(now_ms() > 1_735_689_600_000);
    }
}
