//! Local SQLite storage: opening, migrations, settings, and device identity.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;
use yrs::ClientID;

use crate::error::{Error, Result};
use crate::ids::new_id;

/// Schema migrations, applied in order. Index `i` brings the schema to `user_version = i + 1`.
///
/// Never edit a migration that has shipped; append a new one instead.
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/0001_init.sql"),
    include_str!("migrations/0002_attachments.sql"),
];

/// Settings key holding this device's UUIDv7.
pub const DEVICE_ID_KEY: &str = "device_id";

/// Handle to the local note-tice! database.
///
/// All reads and writes of notes, metadata, and the derived indexes go through a `Store`.
#[derive(Debug)]
pub struct Store {
    pub(crate) conn: Connection,
    device_id: Uuid,
    /// yrs client ID for edits made by Rust in this session (fresh per session, never reused).
    client_id: ClientID,
    /// Where attachment files live (`<db dir>/attachments`). `None` for in-memory stores.
    pub(crate) attachments_dir: Option<PathBuf>,
}

impl Store {
    /// Opens (or creates) the database at `path` and applies pending migrations.
    ///
    /// # Errors
    /// Fails if the file cannot be opened, a migration fails, or the schema is newer than this build.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut store = Self::init(Connection::open(path)?)?;
        store.attachments_dir = Some(
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join("attachments"),
        );
        Ok(store)
    }

    /// Opens a fresh in-memory database. Intended for tests.
    ///
    /// # Errors
    /// Fails if a migration fails.
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self> {
        // WAL keeps readers non-blocking; FULL makes every commit durable across power loss.
        conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        migrate(&mut conn)?;
        let device_id = ensure_device_id(&conn)?;
        let client_id = yrs::Doc::new().client_id();
        Ok(Self {
            conn,
            device_id,
            client_id,
            attachments_dir: None,
        })
    }

    /// The UUIDv7 identifying this device, created on first open.
    #[must_use]
    pub fn device_id(&self) -> Uuid {
        self.device_id
    }

    pub(crate) fn client_id(&self) -> ClientID {
        self.client_id
    }

    /// Schema version currently applied to the database.
    ///
    /// # Errors
    /// Fails if the pragma cannot be read.
    pub fn schema_version(&self) -> Result<usize> {
        user_version(&self.conn)
    }

    /// Reads one setting.
    ///
    /// # Errors
    /// Fails on a database error.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
                r.get(0)
            })
            .optional()?)
    }

    /// Writes one setting, replacing any previous value.
    ///
    /// # Errors
    /// Fails on a database error.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Returns all settings.
    ///
    /// # Errors
    /// Fails on a database error.
    pub fn settings(&self) -> Result<BTreeMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

fn user_version(conn: &Connection) -> Result<usize> {
    let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    usize::try_from(v).map_err(|_| Error::Corrupt(format!("negative user_version {v}")))
}

fn migrate(conn: &mut Connection) -> Result<()> {
    let current = user_version(conn)?;
    if current > MIGRATIONS.len() {
        return Err(Error::SchemaTooNew {
            found: current,
            supported: MIGRATIONS.len(),
        });
    }
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(
            None,
            "user_version",
            i64::try_from(i + 1).unwrap_or(i64::MAX),
        )?;
        tx.commit()?;
        tracing::info!(version = i + 1, "applied database migration");
    }
    Ok(())
}

fn ensure_device_id(conn: &Connection) -> Result<Uuid> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [DEVICE_ID_KEY],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(s) = existing {
        return Uuid::parse_str(&s).map_err(|e| Error::Corrupt(format!("device_id: {e}")));
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)",
        params![DEVICE_ID_KEY, id.to_string()],
    )?;
    tracing::info!(device_id = %id, "created device id");
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_database_is_fully_migrated() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), MIGRATIONS.len());
        // FTS5 is available in the bundled SQLite.
        store
            .conn
            .execute(
                "INSERT INTO notes_fts (note_id, title, body) VALUES ('x', 'Café', 'body')",
                [],
            )
            .unwrap();
    }

    #[test]
    fn reopen_keeps_device_id_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notetice.db");
        let first = Store::open(&path).unwrap();
        let id = first.device_id();
        assert_eq!(id.get_version_num(), 7);
        drop(first);
        let second = Store::open(&path).unwrap();
        assert_eq!(second.device_id(), id);
        assert_eq!(second.schema_version().unwrap(), MIGRATIONS.len());
    }

    #[test]
    fn newer_schema_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notetice.db");
        drop(Store::open(&path).unwrap());
        let conn = Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        drop(conn);
        assert!(matches!(
            Store::open(&path),
            Err(Error::SchemaTooNew { found: 99, .. })
        ));
    }

    #[test]
    fn settings_round_trip() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.get_setting("theme").unwrap(), None);
        store.set_setting("theme", "dark").unwrap();
        store.set_setting("theme", "light").unwrap();
        assert_eq!(
            store.get_setting("theme").unwrap().as_deref(),
            Some("light")
        );
        assert!(store.settings().unwrap().contains_key(DEVICE_ID_KEY));
    }
}
