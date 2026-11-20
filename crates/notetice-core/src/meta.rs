//! The workspace meta document (DESIGN.md 5.1): notes list, titles, folders, pins, tombstones.
//!
//! The meta document is owned by Rust. Each note and folder is a nested `Y.Map` so concurrent
//! edits to different fields of the same note merge cleanly. Deleting sets `deleted_at`
//! (a tombstone); entries are never removed.
//!
//! ```text
//! Y.Doc (id = workspace_id)
//! ├── Y.Map "workspace" → { name }
//! ├── Y.Map "notes"     → note_id → Y.Map { title, folder_id, pinned, in_inbox,
//! │                                         created_at, updated_at, deleted_at }
//! └── Y.Map "folders"   → folder_id → Y.Map { name, parent_id, created_at, deleted_at }
//! ```

use std::sync::Arc;

use rusqlite::Connection;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;
use yrs::ClientID;
use yrs::any::Number;
use yrs::{Any, Doc, Map, MapPrelim, MapRef, Out, ReadTxn, Transact, TransactionMut};

use crate::docs::{self, DocKind};
use crate::error::{Error, Result};
use crate::ids::{new_id, now_ms};
use crate::index;

const WORKSPACE: &str = "workspace";
const NOTES: &str = "notes";
const FOLDERS: &str = "folders";

/// A note's metadata as stored in the meta document (and mirrored in `notes_index`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteMeta {
    /// Note ID (= its content document ID).
    pub id: Uuid,
    /// Workspace the note belongs to.
    pub workspace_id: Uuid,
    /// Title, edited separately from the content.
    pub title: String,
    /// Folder, if any. A folder that is missing or deleted is treated as no folder.
    pub folder_id: Option<Uuid>,
    /// Pinned notes are shown first and in the quick window.
    pub pinned: bool,
    /// Captured but not yet sorted.
    pub in_inbox: bool,
    /// Unix ms.
    pub created_at: i64,
    /// Unix ms; for display only.
    pub updated_at: i64,
    /// Tombstone (unix ms). `Some` means the note is in the trash.
    pub deleted_at: Option<i64>,
}

/// A folder from the meta document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    /// Folder ID.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Parent folder (nesting is modelled but not yet exposed in the UI).
    pub parent_id: Option<Uuid>,
    /// Unix ms.
    pub created_at: i64,
    /// Tombstone (unix ms).
    pub deleted_at: Option<i64>,
}

/// A workspace row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    /// Workspace ID (= its meta document ID).
    pub id: Uuid,
    /// Display name (cached from the meta document).
    pub name: String,
    /// Sync folder, if sync is enabled.
    pub sync_path: Option<String>,
    /// Unix ms.
    pub created_at: i64,
    /// Unix ms.
    pub updated_at: i64,
}

/// A partial update of a note's metadata. Absent fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotePatch {
    /// New title.
    pub title: Option<String>,
    /// `Some(None)` moves the note out of any folder.
    #[serde(default, deserialize_with = "double_option")]
    pub folder_id: Option<Option<Uuid>>,
    /// Pin or unpin.
    pub pinned: Option<bool>,
    /// Put into or take out of the Inbox.
    pub in_inbox: Option<bool>,
}

fn double_option<'de, D, T>(d: D) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}

/// Root maps of an opened meta document.
pub(crate) struct MetaDoc {
    pub doc: Doc,
    pub workspace: MapRef,
    pub notes: MapRef,
    pub folders: MapRef,
}

impl MetaDoc {
    pub fn open(state: &[u8], client_id: ClientID) -> Result<Self> {
        let doc = docs::doc_from_state(state, client_id)?;
        let workspace = doc.get_or_insert_map(WORKSPACE);
        let notes = doc.get_or_insert_map(NOTES);
        let folders = doc.get_or_insert_map(FOLDERS);
        Ok(Self {
            doc,
            workspace,
            notes,
            folders,
        })
    }
}

fn int(v: i64) -> Any {
    Any::Number(Number::Int(v))
}

fn opt_int(v: Option<i64>) -> Any {
    v.map_or(Any::Null, int)
}

fn opt_id(v: Option<Uuid>) -> Any {
    v.map_or(Any::Null, |id| Any::String(Arc::from(id.to_string())))
}

fn get_any<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> Option<Any> {
    match map.get(txn, key) {
        Some(Out::Any(a)) => Some(a),
        _ => None,
    }
}

fn get_str<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> Option<String> {
    match get_any(map, txn, key) {
        Some(Any::String(s)) => Some(s.to_string()),
        _ => None,
    }
}

fn get_int<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> Option<i64> {
    match get_any(map, txn, key) {
        Some(Any::Number(Number::Int(i))) => Some(i),
        // Yjs in JS writes every number as a float.
        #[allow(clippy::cast_possible_truncation)]
        Some(Any::Number(Number::Float(f))) => Some(f as i64),
        _ => None,
    }
}

fn get_bool<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> bool {
    matches!(get_any(map, txn, key), Some(Any::Bool(true)))
}

fn get_id<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> Option<Uuid> {
    get_str(map, txn, key).and_then(|s| Uuid::parse_str(&s).ok())
}

fn child_map<T: ReadTxn>(parent: &MapRef, txn: &T, key: &str) -> Option<MapRef> {
    match parent.get(txn, key) {
        Some(Out::YMap(m)) => Some(m),
        _ => None,
    }
}

fn read_note<T: ReadTxn>(
    notes: &MapRef,
    txn: &T,
    workspace_id: Uuid,
    note_id: Uuid,
) -> Option<NoteMeta> {
    let m = child_map(notes, txn, &note_id.to_string())?;
    Some(NoteMeta {
        id: note_id,
        workspace_id,
        title: get_str(&m, txn, "title").unwrap_or_default(),
        folder_id: get_id(&m, txn, "folder_id"),
        pinned: get_bool(&m, txn, "pinned"),
        in_inbox: get_bool(&m, txn, "in_inbox"),
        created_at: get_int(&m, txn, "created_at").unwrap_or(0),
        updated_at: get_int(&m, txn, "updated_at").unwrap_or(0),
        deleted_at: get_int(&m, txn, "deleted_at"),
    })
}

/// Reads every note in a meta document.
pub(crate) fn all_notes(meta: &MetaDoc, workspace_id: Uuid) -> Vec<NoteMeta> {
    let txn = meta.doc.transact();
    let keys: Vec<String> = meta.notes.keys(&txn).map(str::to_owned).collect();
    keys.iter()
        .filter_map(|k| Uuid::parse_str(k).ok())
        .filter_map(|id| read_note(&meta.notes, &txn, workspace_id, id))
        .collect()
}

fn read_folders(meta: &MetaDoc) -> Vec<Folder> {
    let txn = meta.doc.transact();
    let keys: Vec<String> = meta.folders.keys(&txn).map(str::to_owned).collect();
    keys.iter()
        .filter_map(|k| {
            let id = Uuid::parse_str(k).ok()?;
            let m = child_map(&meta.folders, &txn, k)?;
            Some(Folder {
                id,
                name: get_str(&m, &txn, "name").unwrap_or_default(),
                parent_id: get_id(&m, &txn, "parent_id"),
                created_at: get_int(&m, &txn, "created_at").unwrap_or(0),
                deleted_at: get_int(&m, &txn, "deleted_at"),
            })
        })
        .collect()
}

fn note_map(notes: &MapRef, txn: &TransactionMut, note_id: Uuid) -> Result<MapRef> {
    child_map(notes, txn, &note_id.to_string()).ok_or_else(|| Error::NotFound {
        kind: "note",
        id: note_id.to_string(),
    })
}

fn folder_map(folders: &MapRef, txn: &TransactionMut, folder_id: Uuid) -> Result<MapRef> {
    child_map(folders, txn, &folder_id.to_string()).ok_or_else(|| Error::NotFound {
        kind: "folder",
        id: folder_id.to_string(),
    })
}

/// Applies `f` to the meta document of `workspace_id` inside the caller's SQLite transaction,
/// then persists the new state, reindexes the notes `f` reports as touched, and queues the
/// update in the outbox when the workspace is synced.
pub(crate) fn mutate<R>(
    conn: &Connection,
    client_id: ClientID,
    workspace_id: Uuid,
    f: impl FnOnce(&mut TransactionMut, &MetaDoc) -> Result<(R, Vec<Uuid>)>,
) -> Result<R> {
    let row = docs::require_doc(conn, workspace_id)?;
    if row.kind != DocKind::Meta {
        return Err(Error::NotFound {
            kind: "workspace",
            id: workspace_id.to_string(),
        });
    }
    let meta = MetaDoc::open(&row.state, client_id)?;
    let (result, touched, update) = {
        let mut txn = meta.doc.transact_mut();
        let (result, touched) = f(&mut txn, &meta)?;
        (result, touched, txn.encode_update_v1())
    };
    // An empty Yjs update (no structs, empty delete set) means `f` changed nothing.
    if update.as_slice() == [0, 0] {
        return Ok(result);
    }
    let now = now_ms();
    docs::write_doc(
        conn,
        workspace_id,
        workspace_id,
        DocKind::Meta,
        &docs::encode_state(&meta.doc),
        now,
    )?;
    {
        let txn = meta.doc.transact();
        for id in touched {
            if let Some(note) = read_note(&meta.notes, &txn, workspace_id, id) {
                index::upsert_note(conn, &note)?;
            }
        }
    }
    index::enqueue_outbox(
        conn,
        workspace_id,
        workspace_id,
        DocKind::Meta,
        &update,
        now,
    )?;
    Ok(result)
}

/// Reads and opens the meta document of a workspace.
pub(crate) fn load(conn: &Connection, client_id: ClientID, workspace_id: Uuid) -> Result<MetaDoc> {
    let row = docs::require_doc(conn, workspace_id)?;
    MetaDoc::open(&row.state, client_id)
}

pub(crate) fn create_workspace(
    conn: &Connection,
    client_id: ClientID,
    name: &str,
) -> Result<Workspace> {
    let id = new_id();
    let now = now_ms();
    conn.execute(
        "INSERT INTO workspaces (id, name, sync_path, created_at, updated_at)
         VALUES (?1, ?2, NULL, ?3, ?3)",
        rusqlite::params![id.to_string(), name, now],
    )?;
    let meta = MetaDoc::open(&[], client_id)?;
    {
        let mut txn = meta.doc.transact_mut();
        meta.workspace.insert(&mut txn, "name", name);
    }
    docs::write_doc(
        conn,
        id,
        id,
        DocKind::Meta,
        &docs::encode_state(&meta.doc),
        now,
    )?;
    Ok(Workspace {
        id,
        name: name.to_owned(),
        sync_path: None,
        created_at: now,
        updated_at: now,
    })
}

pub(crate) fn create_note(
    txn: &mut TransactionMut,
    meta: &MetaDoc,
    note_id: Uuid,
    folder_id: Option<Uuid>,
    in_inbox: bool,
    now: i64,
) {
    let entry = MapPrelim::from([
        ("title", Any::from("")),
        ("folder_id", opt_id(folder_id)),
        ("pinned", Any::Bool(false)),
        ("in_inbox", Any::Bool(in_inbox)),
        ("created_at", int(now)),
        ("updated_at", int(now)),
        ("deleted_at", Any::Null),
    ]);
    meta.notes.insert(txn, note_id.to_string(), entry);
}

pub(crate) fn patch_note(
    txn: &mut TransactionMut,
    meta: &MetaDoc,
    note_id: Uuid,
    patch: &NotePatch,
    now: i64,
) -> Result<()> {
    let m = note_map(&meta.notes, txn, note_id)?;
    if let Some(title) = &patch.title {
        m.insert(txn, "title", title.as_str());
    }
    if let Some(folder_id) = patch.folder_id {
        m.insert(txn, "folder_id", opt_id(folder_id));
    }
    if let Some(pinned) = patch.pinned {
        m.insert(txn, "pinned", pinned);
    }
    if let Some(in_inbox) = patch.in_inbox {
        m.insert(txn, "in_inbox", in_inbox);
    }
    m.insert(txn, "updated_at", int(now));
    Ok(())
}

pub(crate) fn set_note_deleted(
    txn: &mut TransactionMut,
    meta: &MetaDoc,
    note_id: Uuid,
    deleted_at: Option<i64>,
) -> Result<()> {
    let m = note_map(&meta.notes, txn, note_id)?;
    m.insert(txn, "deleted_at", opt_int(deleted_at));
    Ok(())
}

/// Bumps `updated_at` unless it was bumped less than `min_interval_ms` ago.
///
/// Throttled because every write to the meta document is synced and kept in its history.
pub(crate) fn touch_note(
    txn: &mut TransactionMut,
    meta: &MetaDoc,
    note_id: Uuid,
    now: i64,
    min_interval_ms: i64,
) -> Result<bool> {
    let m = note_map(&meta.notes, txn, note_id)?;
    let last = get_int(&m, txn, "updated_at").unwrap_or(0);
    if now - last < min_interval_ms {
        return Ok(false);
    }
    m.insert(txn, "updated_at", int(now));
    Ok(true)
}

pub(crate) fn create_folder(
    txn: &mut TransactionMut,
    meta: &MetaDoc,
    folder_id: Uuid,
    name: &str,
    parent_id: Option<Uuid>,
    now: i64,
) {
    let entry = MapPrelim::from([
        ("name", Any::from(name)),
        ("parent_id", opt_id(parent_id)),
        ("created_at", int(now)),
        ("deleted_at", Any::Null),
    ]);
    meta.folders.insert(txn, folder_id.to_string(), entry);
}

pub(crate) fn rename_folder(
    txn: &mut TransactionMut,
    meta: &MetaDoc,
    folder_id: Uuid,
    name: &str,
) -> Result<()> {
    folder_map(&meta.folders, txn, folder_id)?.insert(txn, "name", name);
    Ok(())
}

/// Tombstones a folder and moves its live notes out of it. Returns the moved notes.
pub(crate) fn delete_folder(
    txn: &mut TransactionMut,
    meta: &MetaDoc,
    workspace_id: Uuid,
    folder_id: Uuid,
    now: i64,
) -> Result<Vec<Uuid>> {
    folder_map(&meta.folders, txn, folder_id)?.insert(txn, "deleted_at", int(now));
    let keys: Vec<String> = meta.notes.keys(txn).map(str::to_owned).collect();
    let mut moved = Vec::new();
    for key in keys {
        let Ok(id) = Uuid::parse_str(&key) else {
            continue;
        };
        let Some(note) = read_note(&meta.notes, txn, workspace_id, id) else {
            continue;
        };
        if note.folder_id == Some(folder_id) && note.deleted_at.is_none() {
            note_map(&meta.notes, txn, id)?.insert(txn, "folder_id", Any::Null);
            moved.push(id);
        }
    }
    Ok(moved)
}

/// Live folders of a workspace, sorted by name.
pub(crate) fn live_folders(meta: &MetaDoc) -> Vec<Folder> {
    let mut folders: Vec<Folder> = read_folders(meta)
        .into_iter()
        .filter(|f| f.deleted_at.is_none())
        .collect();
    folders.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.id.cmp(&b.id))
    });
    folders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_deserializes_absent_null_and_value() {
        let p: NotePatch = serde_json_like(r#"{"title":"x"}"#);
        assert_eq!(p.folder_id, None);
        let p: NotePatch = serde_json_like(r#"{"folderId":null}"#);
        assert_eq!(p.folder_id, Some(None));
        let id = new_id();
        let p: NotePatch = serde_json_like(&format!(r#"{{"folderId":"{id}"}}"#));
        assert_eq!(p.folder_id, Some(Some(id)));
    }

    fn serde_json_like<T: for<'de> Deserialize<'de>>(s: &str) -> T {
        serde_json::from_str(s).unwrap()
    }
}
