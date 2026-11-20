//! Location of the app data directory (database, backups).

use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::Context;

/// Environment variable that overrides the data directory (used to run two instances side by
/// side when testing folder sync on one machine).
pub const DATA_DIR_ENV: &str = "NOTETICE_DATA_DIR";

/// Directory name under the OS data directory
/// (`~/Library/Application Support/Notetice` on macOS, `%APPDATA%\Notetice` on Windows).
const APP_DIR_NAME: &str = "Notetice";

/// Resolves the data directory from the environment and the OS default.
pub fn resolve() -> anyhow::Result<PathBuf> {
    resolve_from(std::env::var_os(DATA_DIR_ENV), dirs::data_dir())
}

fn resolve_from(env: Option<OsString>, os_data_dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(dir) = env.filter(|v| !v.is_empty()) {
        let dir = PathBuf::from(dir);
        anyhow::ensure!(
            dir.is_absolute(),
            "{DATA_DIR_ENV} must be an absolute path, got {}",
            dir.display()
        );
        return Ok(dir);
    }
    Ok(os_data_dir
        .context("the OS does not report a data directory")?
        .join(APP_DIR_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_override_wins() {
        let dir = resolve_from(Some("/tmp/a".into()), Some("/data".into())).unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/a"));
    }

    #[test]
    fn empty_env_falls_back_to_os_dir() {
        let dir = resolve_from(Some("".into()), Some("/data".into())).unwrap();
        assert_eq!(dir, PathBuf::from("/data/Notetice"));
        let dir = resolve_from(None, Some("/data".into())).unwrap();
        assert_eq!(dir, PathBuf::from("/data/Notetice"));
    }

    #[test]
    fn relative_override_is_rejected() {
        assert!(resolve_from(Some("rel".into()), None).is_err());
        assert!(resolve_from(None, None).is_err());
    }
}
