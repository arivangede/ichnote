//! CRDT documents (yrs, binary compatible with Yjs) and their storage in the `docs` table.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yrs::types::xml::{XmlFragment, XmlOut};
use yrs::updates::decoder::Decode;
use yrs::{ClientID, Doc, GetString, Options, ReadTxn, StateVector, TextRef, Transact, Update};

use crate::error::{Error, Result};

/// Name of the `Y.XmlFragment` that holds a note's BlockNote content.
///
/// The frontend binds BlockNote to `ydoc.getXmlFragment(NOTE_FRAGMENT)`.
pub const NOTE_FRAGMENT: &str = "document-store";

/// The two kinds of CRDT documents (DESIGN.md 5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocKind {
    /// One per workspace: notes list, titles, folders, pins, tombstones.
    Meta,
    /// One per note: the editor blocks.
    Note,
}

impl DocKind {
    /// Value stored in SQLite and in sync files.
    #[must_use]
    pub fn as_i64(self) -> i64 {
        match self {
            Self::Meta => 0,
            Self::Note => 1,
        }
    }

    /// Parses the stored value.
    ///
    /// # Errors
    /// Fails for unknown values.
    pub fn from_i64(v: i64) -> Result<Self> {
        match v {
            0 => Ok(Self::Meta),
            1 => Ok(Self::Note),
            other => Err(Error::Corrupt(format!("unknown doc kind {other}"))),
        }
    }
}

/// A stored document row.
#[derive(Debug, Clone)]
pub(crate) struct DocRow {
    pub workspace_id: Uuid,
    pub kind: DocKind,
    pub state: Vec<u8>,
}

pub(crate) fn parse_uuid(s: &str) -> Result<Uuid> {
    Uuid::parse_str(s).map_err(|e| Error::Corrupt(format!("invalid uuid {s:?}: {e}")))
}

pub(crate) fn read_doc(conn: &Connection, doc_id: Uuid) -> Result<Option<DocRow>> {
    let row = conn
        .query_row(
            "SELECT workspace_id, kind, state FROM docs WHERE id = ?1",
            [doc_id.to_string()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Vec<u8>>(2)?,
                ))
            },
        )
        .optional()?;
    row.map(|(ws, kind, state)| {
        Ok(DocRow {
            workspace_id: parse_uuid(&ws)?,
            kind: DocKind::from_i64(kind)?,
            state,
        })
    })
    .transpose()
}

pub(crate) fn require_doc(conn: &Connection, doc_id: Uuid) -> Result<DocRow> {
    read_doc(conn, doc_id)?.ok_or_else(|| Error::NotFound {
        kind: "document",
        id: doc_id.to_string(),
    })
}

pub(crate) fn write_doc(
    conn: &Connection,
    doc_id: Uuid,
    workspace_id: Uuid,
    kind: DocKind,
    state: &[u8],
    now: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO docs (id, workspace_id, kind, state, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET state = excluded.state, updated_at = excluded.updated_at",
        params![
            doc_id.to_string(),
            workspace_id.to_string(),
            kind.as_i64(),
            state,
            now
        ],
    )?;
    Ok(())
}

/// Decodes a v1 update.
pub(crate) fn decode_update(bytes: &[u8]) -> Result<Update> {
    Update::decode_v1(bytes).map_err(|e| Error::InvalidUpdate(e.to_string()))
}

/// Creates an empty document that uses `client_id` for local edits.
pub(crate) fn new_doc(client_id: ClientID) -> Doc {
    Doc::with_options(Options::with_client_id(client_id))
}

/// Builds a document from stored state, using `client_id` for any new local edits.
pub(crate) fn doc_from_state(state: &[u8], client_id: ClientID) -> Result<Doc> {
    let doc = new_doc(client_id);
    if !state.is_empty() {
        let update = decode_update(state)?;
        doc.transact_mut()
            .apply_update(update)
            .map_err(|e| Error::InvalidUpdate(e.to_string()))?;
    }
    Ok(doc)
}

/// Encodes the full document state (including pending updates) as a v1 update.
pub(crate) fn encode_state(doc: &Doc) -> Vec<u8> {
    doc.transact()
        .encode_state_as_update_v1(&StateVector::default())
}

/// Extracts the plain text of a note document for full-text search.
///
/// Walks every `XmlText` in the BlockNote fragment in document order and puts each on its own
/// line. Formatting marks are attributes on the text, so they never leak into the result.
pub fn note_plain_text(doc: &Doc) -> String {
    let fragment = doc.get_or_insert_xml_fragment(NOTE_FRAGMENT);
    let txn = doc.transact();
    let mut lines = Vec::new();
    for node in fragment.successors(&txn) {
        if let XmlOut::Text(text) = node {
            let text_ref: &TextRef = text.as_ref();
            let s = text_ref.get_string(&txn);
            if !s.trim().is_empty() {
                lines.push(s);
            }
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/blocknote-note.bin");

    #[test]
    fn extracts_text_from_blocknote_fixture() {
        let doc = doc_from_state(FIXTURE, ClientID::new(1)).unwrap();
        assert_eq!(
            note_plain_text(&doc),
            "Grocery list\nBuy fresh café beans\nMilk\nParent\nNested child\nfn main() {}"
        );
    }

    #[test]
    fn state_round_trip_is_stable() {
        let doc = doc_from_state(FIXTURE, ClientID::new(1)).unwrap();
        let state = encode_state(&doc);
        let again = doc_from_state(&state, ClientID::new(2)).unwrap();
        assert_eq!(note_plain_text(&doc), note_plain_text(&again));
        assert_eq!(
            doc.transact().state_vector(),
            again.transact().state_vector()
        );
    }

    #[test]
    fn garbage_update_is_rejected() {
        assert!(matches!(
            doc_from_state(&[0xff, 0x00, 0x13], ClientID::new(1)),
            Err(Error::InvalidUpdate(_))
        ));
    }

    #[test]
    fn doc_kind_round_trip() {
        for k in [DocKind::Meta, DocKind::Note] {
            assert_eq!(DocKind::from_i64(k.as_i64()).unwrap(), k);
        }
        assert!(DocKind::from_i64(7).is_err());
    }
}
