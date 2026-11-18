//! Core of Ichnote: data model, local SQLite storage, and CRDT documents.
//!
//! This crate must not depend on Tauri so it can be tested on its own.

/// Errors produced by `ichnote-core`.
#[derive(Debug, thiserror::Error)]
pub enum Error {}

/// Convenience result type for `ichnote-core`.
pub type Result<T> = std::result::Result<T, Error>;

/// Version of this crate, as declared in `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
