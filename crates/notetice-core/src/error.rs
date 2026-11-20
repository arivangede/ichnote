//! Error type for `notetice-core`.

/// Errors produced by `notetice-core`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An SQLite operation failed.
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// The database was created by a newer version of note-tice!.
    #[error("database schema version {found} is newer than the supported version {supported}")]
    SchemaTooNew {
        /// Version found in the database.
        found: usize,
        /// Highest version this build knows.
        supported: usize,
    },

    /// A stored value could not be interpreted.
    #[error("corrupt data: {0}")]
    Corrupt(String),

    /// A CRDT update could not be decoded or applied.
    #[error("invalid CRDT update: {0}")]
    InvalidUpdate(String),

    /// The requested item does not exist.
    #[error("{kind} not found: {id}")]
    NotFound {
        /// What kind of item was looked up.
        kind: &'static str,
        /// The ID that was looked up.
        id: String,
    },

    /// A filesystem operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience result type for `notetice-core`.
pub type Result<T> = std::result::Result<T, Error>;
