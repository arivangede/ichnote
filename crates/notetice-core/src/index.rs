//! Derived caches: `notes_index` and `notes_fts`, plus the sync outbox.
//!
//! Everything here can be rebuilt from the `docs` table with [`crate::Store::rebuild_index`].

use rusqlite::{Connection, params};
use uuid::Uuid;
use yrs::ClientID;

use crate::docs::{self, DocKind};
use crate::error::Result;
use crate::meta::{self, NoteMeta};

/// Writes a note's metadata into `notes_index` and its title into `notes_fts`.
pub(crate) fn upsert_note(conn: &Connection, note: &NoteMeta) -> Result<()> {
    conn.execute(
        "INSERT INTO notes_index
           (id, workspace_id, title, folder_id, pinned, in_inbox, created_at, updated_at, deleted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
           workspace_id = excluded.workspace_id, title = excluded.title,
           folder_id = excluded.folder_id, pinned = excluded.pinned,
           in_inbox = excluded.in_inbox, created_at = excluded.created_at,
           updated_at = excluded.updated_at, deleted_at = excluded.deleted_at",
        params![
            note.id.to_string(),
            note.workspace_id.to_string(),
            note.title,
            note.folder_id.map(|f| f.to_string()),
            note.pinned,
            note.in_inbox,
            note.created_at,
            note.updated_at,
            note.deleted_at,
        ],
    )?;
    let id = note.id.to_string();
    let changed = conn.execute(
        "UPDATE notes_fts SET title = ?2 WHERE note_id = ?1",
        params![id, note.title],
    )?;
    if changed == 0 {
        conn.execute(
            "INSERT INTO notes_fts (note_id, title, body) VALUES (?1, ?2, '')",
            params![id, note.title],
        )?;
    }
    Ok(())
}

/// Stores the plain-text body of a note in `notes_fts`.
pub(crate) fn set_body(conn: &Connection, note_id: Uuid, body: &str) -> Result<()> {
    let id = note_id.to_string();
    let changed = conn.execute(
        "UPDATE notes_fts SET body = ?2 WHERE note_id = ?1",
        params![id, body],
    )?;
    if changed == 0 {
        conn.execute(
            "INSERT INTO notes_fts (note_id, title, body) VALUES (?1, '', ?2)",
            params![id, body],
        )?;
    }
    Ok(())
}

/// Queues an update for the sync transport, but only if the workspace is synced.
///
/// Unsynced workspaces skip the outbox entirely; enabling sync later writes a full snapshot.
pub(crate) fn enqueue_outbox(
    conn: &Connection,
    workspace_id: Uuid,
    doc_id: Uuid,
    kind: DocKind,
    payload: &[u8],
    now: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO outbox (workspace_id, doc_id, doc_kind, payload, created_at)
         SELECT ?1, ?2, ?3, ?4, ?5 FROM workspaces WHERE id = ?1 AND sync_path IS NOT NULL",
        params![
            workspace_id.to_string(),
            doc_id.to_string(),
            kind.as_i64(),
            payload,
            now
        ],
    )?;
    Ok(())
}

/// Rebuilds `notes_index` and `notes_fts` from the stored CRDT documents.
pub(crate) fn rebuild(conn: &Connection, client_id: ClientID) -> Result<()> {
    conn.execute("DELETE FROM notes_index", [])?;
    conn.execute("DELETE FROM notes_fts", [])?;
    let workspaces: Vec<String> = {
        let mut stmt = conn.prepare("SELECT id FROM docs WHERE kind = ?1")?;
        stmt.query_map([DocKind::Meta.as_i64()], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    for ws in workspaces {
        let workspace_id = docs::parse_uuid(&ws)?;
        let meta = meta::load(conn, client_id, workspace_id)?;
        for note in meta::all_notes(&meta, workspace_id) {
            upsert_note(conn, &note)?;
            if let Some(row) = docs::read_doc(conn, note.id)? {
                let doc = docs::doc_from_state(&row.state, client_id)?;
                set_body(conn, note.id, &docs::note_plain_text(&doc))?;
            }
        }
    }
    Ok(())
}
