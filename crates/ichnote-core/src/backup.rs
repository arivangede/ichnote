//! Daily backups of the local database (DESIGN.md section 11).

use std::fs;
use std::path::{Path, PathBuf};

use crate::db::Store;
use crate::error::Result;
use crate::ids::now_ms;

const PREFIX: &str = "ichnote-";
const SUFFIX: &str = ".db";
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// Parses the unix-ms timestamp out of a backup file name (`ichnote-<unix_ms>.db`).
fn backup_time(path: &Path) -> Option<i64> {
    path.file_name()?
        .to_str()?
        .strip_prefix(PREFIX)?
        .strip_suffix(SUFFIX)?
        .parse()
        .ok()
}

fn list_backups(dir: &Path) -> Result<Vec<(i64, PathBuf)>> {
    let mut backups: Vec<(i64, PathBuf)> = fs::read_dir(dir)?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter_map(|p| backup_time(&p).map(|t| (t, p)))
        .collect();
    backups.sort();
    Ok(backups)
}

impl Store {
    /// Writes a backup into `dir` if the newest one is older than a day, then keeps only the
    /// `keep` newest backups. Returns the path of the new backup, if one was written.
    ///
    /// # Errors
    /// Fails if the directory cannot be read or written, or on a database error.
    pub fn backup_if_due(&self, dir: &Path, keep: usize) -> Result<Option<PathBuf>> {
        self.backup_if_due_at(dir, keep, now_ms())
    }

    pub(crate) fn backup_if_due_at(
        &self,
        dir: &Path,
        keep: usize,
        now: i64,
    ) -> Result<Option<PathBuf>> {
        fs::create_dir_all(dir)?;
        let backups = list_backups(dir)?;
        if backups.last().is_some_and(|(t, _)| now - t < DAY_MS) {
            return Ok(None);
        }
        let target = dir.join(format!("{PREFIX}{now}{SUFFIX}"));
        // VACUUM INTO refuses to overwrite, and a crash must not leave a half-written backup
        // under a valid name: write to a temporary name, then rename.
        let tmp = dir.join(format!("{PREFIX}{now}{SUFFIX}.tmp"));
        if tmp.exists() {
            fs::remove_file(&tmp)?;
        }
        self.conn
            .execute("VACUUM INTO ?1", [tmp.to_string_lossy().as_ref()])?;
        fs::rename(&tmp, &target)?;
        tracing::info!(path = %target.display(), "wrote database backup");

        let backups = list_backups(dir)?;
        let excess = backups.len().saturating_sub(keep);
        for (_, old) in backups.into_iter().take(excess) {
            fs::remove_file(&old)?;
        }
        Ok(Some(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backups_are_daily_restorable_and_rotated() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path().join("ichnote.db")).unwrap();
        store.create_workspace("Personal").unwrap();
        let backups = dir.path().join("backups");

        // Old backups from previous days.
        fs::create_dir_all(&backups).unwrap();
        let now = 100 * DAY_MS;
        for day in 1..=8 {
            fs::write(
                backups.join(format!("ichnote-{}.db", now - day * DAY_MS)),
                b"",
            )
            .unwrap();
        }
        fs::write(backups.join("unrelated.txt"), b"").unwrap();

        let written = store.backup_if_due_at(&backups, 7, now).unwrap().unwrap();
        assert_eq!(list_backups(&backups).unwrap().len(), 7);
        assert!(backups.join("unrelated.txt").exists());
        // The oldest ones were removed; the newest is the one just written.
        assert_eq!(list_backups(&backups).unwrap().last().unwrap().1, written);

        // Not due again within a day.
        assert!(
            store
                .backup_if_due_at(&backups, 7, now + DAY_MS - 1)
                .unwrap()
                .is_none()
        );

        // The backup is a complete database.
        let restored = Store::open(&written).unwrap();
        assert_eq!(restored.list_workspaces().unwrap().len(), 1);
        assert_eq!(restored.device_id(), store.device_id());
    }
}
