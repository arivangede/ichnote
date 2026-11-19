//! Shared application state.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use anyhow::Context;
use ichnote_core::Store;

/// File name of the database inside the data directory.
const DB_FILE: &str = "ichnote.db";
/// Backups kept in `<data dir>/backups`.
const BACKUPS_KEPT: usize = 7;
/// Name of the workspace created on first run.
const DEFAULT_WORKSPACE_NAME: &str = "Personal";

/// State managed by Tauri and shared by all commands.
pub struct AppState {
    store: Mutex<Store>,
    data_dir: PathBuf,
}

impl AppState {
    /// Opens the database in `data_dir`, takes the daily backup, and makes sure a workspace exists.
    pub fn open(data_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)
            .with_context(|| format!("cannot create data directory {}", data_dir.display()))?;
        let db_path = data_dir.join(DB_FILE);
        let mut store = Store::open(&db_path)
            .with_context(|| format!("cannot open database {}", db_path.display()))?;
        if let Err(err) = store.backup_if_due(&data_dir.join("backups"), BACKUPS_KEPT) {
            // A failed backup must not keep the user out of their notes.
            tracing::warn!(error = %err, "daily backup failed");
        }
        store
            .ensure_default_workspace(DEFAULT_WORKSPACE_NAME)
            .context("cannot create the default workspace")?;
        tracing::info!(data_dir = %data_dir.display(), device_id = %store.device_id(), "store ready");
        Ok(Self {
            store: Mutex::new(store),
            data_dir: data_dir.to_owned(),
        })
    }

    /// Locks the store. A poisoned lock is recovered: every store operation is transactional, so
    /// a panic in another command cannot have left the database half-written.
    pub fn store(&self) -> MutexGuard<'_, Store> {
        self.store
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The data directory in use.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_creates_db_backup_and_default_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("nested/data");
        let state = AppState::open(&data).unwrap();
        assert!(data.join(DB_FILE).exists());
        assert_eq!(std::fs::read_dir(data.join("backups")).unwrap().count(), 1);
        let workspaces = state.store().list_workspaces().unwrap();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].name, DEFAULT_WORKSPACE_NAME);
        drop(state);
        // Reopening does not create another workspace.
        let state = AppState::open(&data).unwrap();
        assert_eq!(state.store().list_workspaces().unwrap().len(), 1);
    }
}
