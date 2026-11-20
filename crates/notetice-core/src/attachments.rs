//! Attachments: images and other files inserted into notes.
//!
//! Contents are stored once per distinct file under `<data dir>/attachments/<hh>/<hash>`, named
//! by their blake3 hash and never modified. Each insertion gets its own UUIDv7 row so a note can
//! keep its own file name and type.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use rusqlite::{OptionalExtension, params};
use serde::Serialize;
use uuid::Uuid;

use crate::db::Store;
use crate::docs::parse_uuid;
use crate::error::{Error, Result};
use crate::ids::{new_id, now_ms};

/// Prefix of the URL stored in notes for an attachment: `notetice-attachment:<id>`.
pub const URL_PREFIX: &str = "notetice-attachment:";

/// Metadata of a stored attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    /// Attachment ID (UUIDv7).
    pub id: Uuid,
    /// Workspace it was added to.
    pub workspace_id: Uuid,
    /// blake3 hash of the contents (hex).
    pub hash: String,
    /// Original file name.
    pub name: String,
    /// MIME type as reported when it was added.
    pub mime: String,
    /// Size in bytes.
    pub size: u64,
    /// Unix ms.
    pub created_at: i64,
}

impl Attachment {
    /// The URL stored in note content.
    #[must_use]
    pub fn url(&self) -> String {
        format!("{URL_PREFIX}{}", self.id)
    }
}

fn clean_name(name: &str) -> String {
    let name: String = name.chars().filter(|c| !c.is_control()).collect();
    let name = name.trim();
    if name.is_empty() {
        "file".to_owned()
    } else {
        name.to_owned()
    }
}

fn clean_mime(mime: &str) -> String {
    let mime = mime.trim();
    let valid = mime.split_once('/').is_some_and(|(a, b)| {
        !a.is_empty()
            && !b.is_empty()
            && mime
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "/.+-_".contains(c))
    });
    if valid {
        mime.to_ascii_lowercase()
    } else {
        "application/octet-stream".to_owned()
    }
}

impl Store {
    fn attachments_dir(&self) -> Result<&PathBuf> {
        self.attachments_dir.as_ref().ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "attachments need a file-backed store",
            ))
        })
    }

    fn blob_path(&self, hash: &str) -> Result<PathBuf> {
        let prefix = hash.get(..2).unwrap_or("00");
        Ok(self.attachments_dir()?.join(prefix).join(hash))
    }

    /// Stores a file and returns its new attachment record.
    ///
    /// Identical contents are written only once; the write is atomic (temp file, fsync, rename).
    ///
    /// # Errors
    /// Fails if the workspace does not exist, the store is in memory, or on an I/O error.
    pub fn save_attachment(
        &mut self,
        workspace_id: Uuid,
        name: &str,
        mime: &str,
        bytes: &[u8],
    ) -> Result<Attachment> {
        let hash = blake3::hash(bytes).to_hex().to_string();
        let path = self.blob_path(&hash)?;
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let tmp = path.with_extension("tmp");
            {
                let mut f = fs::File::create(&tmp)?;
                f.write_all(bytes)?;
                f.sync_all()?;
            }
            fs::rename(&tmp, &path)?;
        }
        let attachment = Attachment {
            id: new_id(),
            workspace_id,
            hash,
            name: clean_name(name),
            mime: clean_mime(mime),
            size: bytes.len() as u64,
            created_at: now_ms(),
        };
        self.conn.execute(
            "INSERT INTO attachments (id, workspace_id, hash, name, mime, size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                attachment.id.to_string(),
                workspace_id.to_string(),
                attachment.hash,
                attachment.name,
                attachment.mime,
                i64::try_from(attachment.size).unwrap_or(i64::MAX),
                attachment.created_at
            ],
        )?;
        Ok(attachment)
    }

    /// Looks up an attachment and the path of its contents.
    ///
    /// # Errors
    /// Fails if it does not exist or on a database error.
    pub fn attachment(&self, id: Uuid) -> Result<(Attachment, PathBuf)> {
        let row = self
            .conn
            .query_row(
                "SELECT workspace_id, hash, name, mime, size, created_at FROM attachments WHERE id = ?1",
                [id.to_string()],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, i64>(4)?,
                        r.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| Error::NotFound {
                kind: "attachment",
                id: id.to_string(),
            })?;
        let (ws, hash, name, mime, size, created_at) = row;
        let path = self.blob_path(&hash)?;
        Ok((
            Attachment {
                id,
                workspace_id: parse_uuid(&ws)?,
                hash,
                name,
                mime,
                size: u64::try_from(size).unwrap_or(0),
                created_at,
            },
            path,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_dedupes_and_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path().join("notetice.db")).unwrap();
        let ws = store.create_workspace("Personal").unwrap().id;

        let a = store
            .save_attachment(ws, "cat.png", "image/png", b"png-bytes")
            .unwrap();
        let b = store
            .save_attachment(ws, "copy.png", "IMAGE/PNG", b"png-bytes")
            .unwrap();
        assert_ne!(a.id, b.id);
        assert_eq!(a.hash, b.hash);
        assert_eq!(b.mime, "image/png");
        assert_eq!(a.url(), format!("notetice-attachment:{}", a.id));

        let (meta, path) = store.attachment(a.id).unwrap();
        assert_eq!(meta, a);
        assert_eq!(fs::read(&path).unwrap(), b"png-bytes");
        assert!(path.starts_with(dir.path().join("attachments")));
        // Only one blob for both attachments, and no temp files left behind.
        let blobs: Vec<_> = fs::read_dir(path.parent().unwrap()).unwrap().collect();
        assert_eq!(blobs.len(), 1);
    }

    #[test]
    fn names_and_types_are_cleaned() {
        assert_eq!(clean_name("  \n "), "file");
        assert_eq!(clean_name("a\u{0}b.txt"), "ab.txt");
        assert_eq!(
            clean_mime("text/html; charset=utf-8"),
            "application/octet-stream"
        );
        assert_eq!(clean_mime(""), "application/octet-stream");
        assert_eq!(
            clean_mime("application/vnd.ms-excel"),
            "application/vnd.ms-excel"
        );
    }

    #[test]
    fn unknown_ids_and_in_memory_stores_fail() {
        let mut store = Store::open_in_memory().unwrap();
        let ws = store.create_workspace("Personal").unwrap().id;
        assert!(store.save_attachment(ws, "a", "text/plain", b"x").is_err());
        assert!(store.attachment(Uuid::now_v7()).is_err());
    }

    #[test]
    fn unknown_workspace_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path().join("notetice.db")).unwrap();
        assert!(
            store
                .save_attachment(Uuid::now_v7(), "a", "text/plain", b"x")
                .is_err()
        );
    }
}
