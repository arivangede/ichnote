//! Core of Ichnote: data model, local SQLite storage, and CRDT documents.
//!
//! This crate must not depend on Tauri so it can be tested on its own.

pub mod db;
pub mod docs;
pub mod error;
pub mod ids;
mod index;
pub mod meta;
pub mod notes;

pub use db::Store;
pub use docs::DocKind;
pub use error::{Error, Result};
pub use meta::{Folder, NoteMeta, NotePatch, Workspace};
pub use notes::NoteFilter;

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
