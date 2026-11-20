//! Sync engine for Ichnote: outbox, cursors, and pluggable transports.
//!
//! This crate must not depend on Tauri so it can be tested on its own.

/// Errors produced by `ichnote-sync`.
#[derive(Debug, thiserror::Error)]
pub enum Error {}

/// Convenience result type for `ichnote-sync`.
pub type Result<T> = std::result::Result<T, Error>;

/// Version of the core crate this sync engine was built against.
pub const CORE_VERSION: &str = ichnote_core::VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_against_core() {
        assert_eq!(CORE_VERSION, env!("CARGO_PKG_VERSION"));
    }
}
