//! Writing exported files (Markdown is produced by the frontend with BlockNote).

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};

/// One file to write, relative to the export directory.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ExportFile {
    /// Relative path using `/` as separator, e.g. `Work/Meeting notes.md`.
    pub path: String,
    /// File contents (UTF-8).
    pub contents: String,
}

const MAX_COMPONENT_CHARS: usize = 120;
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Makes one path component safe on macOS, Windows, and Linux.
#[must_use]
pub fn sanitize_component(name: &str) -> String {
    let replaced: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let trimmed = replaced.trim().trim_matches('.').trim();
    let mut out: String = trimmed.chars().take(MAX_COMPONENT_CHARS).collect();
    out = out.trim_end().to_owned();
    if out.is_empty() {
        return "Untitled".to_owned();
    }
    let stem = out
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if WINDOWS_RESERVED.contains(&stem.as_str()) {
        out.insert(0, '_');
    }
    out
}

/// Sanitizes a `/`-separated relative path. `..` and empty components cannot escape `dir`
/// because every component is sanitized and dot-only names become `Untitled`.
fn safe_relative_path(path: &str) -> PathBuf {
    path.split('/')
        .filter(|c| !c.trim().is_empty())
        .map(sanitize_component)
        .collect()
}

/// Appends ` (2)`, ` (3)`, … before the extension until the path is unused.
fn dedupe(path: PathBuf, used: &HashSet<PathBuf>) -> PathBuf {
    if !used.contains(&path) {
        return path;
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (2..)
        .map(|n| path.with_file_name(format!("{stem} ({n}){ext}")))
        .find(|p| !used.contains(p))
        .unwrap_or(path)
}

fn write_atomic(target: &Path, contents: &[u8]) -> Result<()> {
    let mut tmp_name = target.as_os_str().to_owned();
    tmp_name.push(".tmp");
    let tmp = PathBuf::from(tmp_name);
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(contents)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, target)?;
    Ok(())
}

/// Writes `files` under `dir` (created if missing), each atomically (temp file, fsync, rename).
/// Paths are sanitized and made unique within the batch. Returns the written paths.
///
/// # Errors
/// Fails if `dir` is not absolute or on an I/O error.
pub fn write_files(dir: &Path, files: &[ExportFile]) -> Result<Vec<PathBuf>> {
    if !dir.is_absolute() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "export directory must be absolute",
        )));
    }
    let mut used = HashSet::new();
    let mut written = Vec::with_capacity(files.len());
    for file in files {
        let rel = dedupe(safe_relative_path(&file.path), &used);
        used.insert(rel.clone());
        let target = dir.join(&rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        write_atomic(&target, file.contents.as_bytes())?;
        written.push(target);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_are_sanitized() {
        assert_eq!(sanitize_component("a/b:c*?"), "a-b-c--");
        assert_eq!(sanitize_component("  .. "), "Untitled");
        assert_eq!(sanitize_component(""), "Untitled");
        assert_eq!(sanitize_component("con.md"), "_con.md");
        assert_eq!(sanitize_component("line\nbreak"), "line break");
        assert_eq!(
            sanitize_component(&"x".repeat(300)).len(),
            MAX_COMPONENT_CHARS
        );
    }

    #[test]
    fn writes_nested_unique_files_inside_dir() {
        let dir = tempfile::tempdir().unwrap();
        let files = [
            ExportFile {
                path: "Work/Plan.md".into(),
                contents: "# A".into(),
            },
            ExportFile {
                path: "Work/Plan.md".into(),
                contents: "# B".into(),
            },
            ExportFile {
                path: "../../escape.md".into(),
                contents: "x".into(),
            },
        ];
        let written = write_files(dir.path(), &files).unwrap();
        assert_eq!(written[0], dir.path().join("Work/Plan.md"));
        assert_eq!(written[1], dir.path().join("Work/Plan (2).md"));
        assert!(written[2].starts_with(dir.path()));
        assert_eq!(fs::read_to_string(&written[1]).unwrap(), "# B");
        assert!(written.iter().all(|p| p.exists()));
        assert!(!dir.path().join("Work/Plan.md.tmp").exists());
    }

    #[test]
    fn relative_dir_is_rejected() {
        assert!(write_files(Path::new("relative"), &[]).is_err());
    }
}
