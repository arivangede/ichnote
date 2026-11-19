//! Core of Ichnote: data model, local SQLite storage, and CRDT documents.
//!
//! This crate must not depend on Tauri so it can be tested on its own.

pub mod db;
pub mod error;
pub mod ids;

pub use db::Store;
pub use error::{Error, Result};

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
