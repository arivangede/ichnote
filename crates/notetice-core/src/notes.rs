//! Public operations on workspaces, notes, folders, and documents.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yrs::Transact;

use crate::db::Store;
use crate::docs::{self, DocKind};
use crate::error::{Error, Result};
use crate::ids::{new_id, now_ms};
use crate::index;
use crate::meta::{self, Folder, NoteMeta, NotePatch, Workspace};

const NOTE_COLUMNS: &str = "SELECT id, workspace_id, title, folder_id, pinned, in_inbox, \
     created_at, updated_at, deleted_at FROM notes_index";

/// Minimum time between `updated_at` bumps caused by content edits.
const TOUCH_INTERVAL_MS: i64 = 60_000;

/// Which notes to list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "folderId", rename_all = "camelCase")]
pub enum NoteFilter {
    /// Every live note.
    All,
    /// Live notes in the Inbox.
    Inbox,
    /// Live pinned notes.
    Pinned,
    /// Live notes in a folder.
    Folder(Uuid),
    /// Deleted notes (the trash).
    Trash,
}

impl Store {
    /// Lists all workspaces, oldest first.
    ///
    /// # Errors
    /// Fails on a database error.
    pub fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, sync_path, created_at, updated_at FROM workspaces ORDER BY created_at, id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
            ))
        })?;
        rows.map(|row| {
            let (id, name, sync_path, created_at, updated_at) = row?;
            Ok(Workspace {
                id: docs::parse_uuid(&id)?,
                name,
                sync_path,
                created_at,
                updated_at,
            })
        })
        .collect()
    }

    /// Creates a workspace and its meta document.
    ///
    /// # Errors
    /// Fails on a database error.
    pub fn create_workspace(&mut self, name: &str) -> Result<Workspace> {
        let client_id = self.client_id();
        let tx = self.conn.transaction()?;
        let ws = meta::create_workspace(&tx, client_id, name)?;
        tx.commit()?;
        Ok(ws)
    }

    /// Returns the first workspace, creating one called `name` if there is none.
    ///
    /// # Errors
    /// Fails on a database error.
    pub fn ensure_default_workspace(&mut self, name: &str) -> Result<Workspace> {
        match self.list_workspaces()?.into_iter().next() {
            Some(ws) => Ok(ws),
            None => self.create_workspace(name),
        }
    }

    /// Creates an empty note. Returns its metadata; its ID is also its content document ID.
    ///
    /// # Errors
    /// Fails if the workspace does not exist or on a database error.
    pub fn create_note(
        &mut self,
        workspace_id: Uuid,
        folder_id: Option<Uuid>,
        in_inbox: bool,
    ) -> Result<NoteMeta> {
        let client_id = self.client_id();
        let note_id = new_id();
        let now = now_ms();
        let tx = self.conn.transaction()?;
        meta::mutate(&tx, client_id, workspace_id, |txn, m| {
            meta::create_note(txn, m, note_id, folder_id, in_inbox, now);
            Ok(((), vec![note_id]))
        })?;
        let empty = docs::encode_state(&docs::new_doc(client_id));
        docs::write_doc(&tx, note_id, workspace_id, DocKind::Note, &empty, now)?;
        index::set_body(&tx, note_id, "")?;
        tx.commit()?;
        self.get_note(note_id)
    }

    /// Reads a note's metadata from the index.
    ///
    /// # Errors
    /// Fails if the note does not exist.
    pub fn get_note(&self, note_id: Uuid) -> Result<NoteMeta> {
        let id = note_id.to_string();
        self.query_notes(&format!("{NOTE_COLUMNS} WHERE id = ?1"), &[&id])?
            .pop()
            .ok_or(Error::NotFound { kind: "note", id })
    }

    fn query_notes(&self, sql: &str, args: &[&dyn rusqlite::ToSql]) -> Result<Vec<NoteMeta>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt
            .query_map(args, |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, bool>(4)?,
                    r.get::<_, bool>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                    r.get::<_, Option<i64>>(8)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(
                |(
                    id,
                    ws,
                    title,
                    folder_id,
                    pinned,
                    in_inbox,
                    created_at,
                    updated_at,
                    deleted_at,
                )| {
                    Ok(NoteMeta {
                        id: docs::parse_uuid(&id)?,
                        workspace_id: docs::parse_uuid(&ws)?,
                        title,
                        folder_id: folder_id.as_deref().map(docs::parse_uuid).transpose()?,
                        pinned,
                        in_inbox,
                        created_at,
                        updated_at,
                        deleted_at,
                    })
                },
            )
            .collect()
    }

    fn note_workspace(&self, note_id: Uuid) -> Result<Uuid> {
        let row = docs::require_doc(&self.conn, note_id)?;
        if row.kind != DocKind::Note {
            return Err(Error::NotFound {
                kind: "note",
                id: note_id.to_string(),
            });
        }
        Ok(row.workspace_id)
    }

    /// Lists notes, most recently updated first (trash: most recently deleted first).
    ///
    /// # Errors
    /// Fails on a database error.
    pub fn list_notes(&self, workspace_id: Uuid, filter: NoteFilter) -> Result<Vec<NoteMeta>> {
        let (condition, order) = match filter {
            NoteFilter::All => ("deleted_at IS NULL", "updated_at DESC"),
            NoteFilter::Inbox => ("deleted_at IS NULL AND in_inbox = 1", "updated_at DESC"),
            NoteFilter::Pinned => ("deleted_at IS NULL AND pinned = 1", "updated_at DESC"),
            NoteFilter::Folder(_) => ("deleted_at IS NULL AND folder_id = ?2", "updated_at DESC"),
            NoteFilter::Trash => ("deleted_at IS NOT NULL", "deleted_at DESC"),
        };
        let sql = format!(
            "{NOTE_COLUMNS} WHERE workspace_id = ?1 AND {condition} ORDER BY {order}, id DESC"
        );
        let ws = workspace_id.to_string();
        match filter {
            NoteFilter::Folder(folder) => {
                let folder = folder.to_string();
                self.query_notes(&sql, &[&ws, &folder])
            }
            _ => self.query_notes(&sql, &[&ws]),
        }
    }

    fn mutate_note<R>(
        &mut self,
        note_id: Uuid,
        f: impl FnOnce(&mut yrs::TransactionMut, &meta::MetaDoc, Uuid) -> Result<R>,
    ) -> Result<NoteMeta> {
        let workspace_id = self.note_workspace(note_id)?;
        let client_id = self.client_id();
        let tx = self.conn.transaction()?;
        meta::mutate(&tx, client_id, workspace_id, |txn, m| {
            f(txn, m, workspace_id)?;
            Ok(((), vec![note_id]))
        })?;
        tx.commit()?;
        self.get_note(note_id)
    }

    /// Changes a note's title, folder, pin, or Inbox state.
    ///
    /// # Errors
    /// Fails if the note does not exist or on a database error.
    pub fn update_note_meta(&mut self, note_id: Uuid, patch: &NotePatch) -> Result<NoteMeta> {
        let now = now_ms();
        self.mutate_note(note_id, |txn, m, _| {
            meta::patch_note(txn, m, note_id, patch, now)
        })
    }

    /// Moves a note to the trash (sets its tombstone).
    ///
    /// # Errors
    /// Fails if the note does not exist or on a database error.
    pub fn delete_note(&mut self, note_id: Uuid) -> Result<NoteMeta> {
        let now = now_ms();
        self.mutate_note(note_id, |txn, m, _| {
            meta::set_note_deleted(txn, m, note_id, Some(now))
        })
    }

    /// Restores a note from the trash.
    ///
    /// # Errors
    /// Fails if the note does not exist or on a database error.
    pub fn restore_note(&mut self, note_id: Uuid) -> Result<NoteMeta> {
        self.mutate_note(note_id, |txn, m, _| {
            meta::set_note_deleted(txn, m, note_id, None)
        })
    }

    /// Lists live folders of a workspace, sorted by name.
    ///
    /// # Errors
    /// Fails if the workspace does not exist.
    pub fn list_folders(&self, workspace_id: Uuid) -> Result<Vec<Folder>> {
        let m = meta::load(&self.conn, self.client_id(), workspace_id)?;
        Ok(meta::live_folders(&m))
    }

    /// Creates a folder.
    ///
    /// # Errors
    /// Fails if the workspace does not exist or on a database error.
    pub fn create_folder(
        &mut self,
        workspace_id: Uuid,
        name: &str,
        parent_id: Option<Uuid>,
    ) -> Result<Folder> {
        let client_id = self.client_id();
        let folder_id = new_id();
        let now = now_ms();
        let tx = self.conn.transaction()?;
        meta::mutate(&tx, client_id, workspace_id, |txn, m| {
            meta::create_folder(txn, m, folder_id, name, parent_id, now);
            Ok(((), vec![]))
        })?;
        tx.commit()?;
        Ok(Folder {
            id: folder_id,
            name: name.to_owned(),
            parent_id,
            created_at: now,
            deleted_at: None,
        })
    }

    /// Renames a folder.
    ///
    /// # Errors
    /// Fails if the folder does not exist or on a database error.
    pub fn rename_folder(&mut self, workspace_id: Uuid, folder_id: Uuid, name: &str) -> Result<()> {
        let client_id = self.client_id();
        let tx = self.conn.transaction()?;
        meta::mutate(&tx, client_id, workspace_id, |txn, m| {
            meta::rename_folder(txn, m, folder_id, name)?;
            Ok(((), vec![]))
        })?;
        tx.commit()?;
        Ok(())
    }

    /// Deletes a folder (tombstone). Its notes are kept and moved out of the folder.
    ///
    /// # Errors
    /// Fails if the folder does not exist or on a database error.
    pub fn delete_folder(&mut self, workspace_id: Uuid, folder_id: Uuid) -> Result<()> {
        let client_id = self.client_id();
        let now = now_ms();
        let tx = self.conn.transaction()?;
        meta::mutate(&tx, client_id, workspace_id, |txn, m| {
            let moved = meta::delete_folder(txn, m, workspace_id, folder_id, now)?;
            Ok(((), moved))
        })?;
        tx.commit()?;
        Ok(())
    }

    /// Returns the full state of a document, encoded as a Yjs v1 update.
    ///
    /// # Errors
    /// Fails if the document does not exist.
    pub fn load_doc(&self, doc_id: Uuid) -> Result<Vec<u8>> {
        Ok(docs::require_doc(&self.conn, doc_id)?.state)
    }

    /// Applies a Yjs v1 update produced by the editor to a note document.
    ///
    /// In one SQLite transaction: merges the update, stores the new state, refreshes the
    /// full-text index, bumps the note's `updated_at` (throttled), and queues the update in the
    /// outbox when the workspace is synced. Returns the note's metadata.
    ///
    /// # Errors
    /// Fails if the document is not a note, the update is invalid, or on a database error.
    pub fn apply_local_update(&mut self, doc_id: Uuid, update: &[u8]) -> Result<NoteMeta> {
        let client_id = self.client_id();
        let now = now_ms();
        let tx = self.conn.transaction()?;
        let row = docs::require_doc(&tx, doc_id)?;
        if row.kind != DocKind::Note {
            return Err(Error::InvalidUpdate(
                "only note documents accept updates from the editor".to_owned(),
            ));
        }
        let doc = docs::doc_from_state(&row.state, client_id)?;
        doc.transact_mut()
            .apply_update(docs::decode_update(update)?)
            .map_err(|e| Error::InvalidUpdate(e.to_string()))?;
        docs::write_doc(
            &tx,
            doc_id,
            row.workspace_id,
            DocKind::Note,
            &docs::encode_state(&doc),
            now,
        )?;
        index::set_body(&tx, doc_id, &docs::note_plain_text(&doc))?;
        index::enqueue_outbox(&tx, row.workspace_id, doc_id, DocKind::Note, update, now)?;
        meta::mutate(&tx, client_id, row.workspace_id, |txn, m| {
            let touched = meta::touch_note(txn, m, doc_id, now, TOUCH_INTERVAL_MS)?;
            Ok(((), if touched { vec![doc_id] } else { vec![] }))
        })?;
        tx.commit()?;
        self.get_note(doc_id)
    }

    /// Rebuilds `notes_index` and `notes_fts` from the stored documents.
    ///
    /// # Errors
    /// Fails if a document cannot be decoded or on a database error.
    pub fn rebuild_index(&mut self) -> Result<()> {
        let client_id = self.client_id();
        let tx = self.conn.transaction()?;
        index::rebuild(&tx, client_id)?;
        tx.commit()?;
        Ok(())
    }
}
