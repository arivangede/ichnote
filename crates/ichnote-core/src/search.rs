//! Full-text search over note titles and bodies (FTS5).

use serde::Serialize;
use uuid::Uuid;

use crate::db::Store;
use crate::error::Result;
use crate::meta::NoteMeta;

/// Marks the start of a matched term in [`SearchHit::snippet`].
pub const HIGHLIGHT_START: char = '\u{2}';
/// Marks the end of a matched term in [`SearchHit::snippet`].
pub const HIGHLIGHT_END: char = '\u{3}';

/// One search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    /// The matching note.
    pub note: NoteMeta,
    /// A short excerpt with matches wrapped in [`HIGHLIGHT_START`] / [`HIGHLIGHT_END`].
    pub snippet: String,
}

/// Turns free text into a safe FTS5 query: every word must match, as a prefix.
///
/// User input never reaches FTS5 syntax directly, so operators such as `AND`, `NEAR`, `*`, `:`
/// or unbalanced quotes are searched for literally instead of causing errors.
#[must_use]
pub fn build_fts_query(input: &str) -> Option<String> {
    let terms: Vec<String> = input
        .split_whitespace()
        .map(|w| w.replace('"', ""))
        .filter(|w| w.chars().any(char::is_alphanumeric))
        .map(|w| format!("\"{w}\"*"))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

impl Store {
    /// Searches live notes of a workspace, best matches first. Titles weigh more than bodies.
    ///
    /// # Errors
    /// Fails on a database error.
    pub fn search(&self, workspace_id: Uuid, query: &str, limit: u32) -> Result<Vec<SearchHit>> {
        let Some(fts_query) = build_fts_query(query) else {
            return Ok(Vec::new());
        };
        let mut stmt = self.conn.prepare(
            "SELECT n.id, snippet(notes_fts, -1, ?4, ?5, '…', 12)
             FROM notes_fts JOIN notes_index n ON n.id = notes_fts.note_id
             WHERE notes_fts MATCH ?1 AND n.workspace_id = ?2 AND n.deleted_at IS NULL
             ORDER BY bm25(notes_fts, 0.0, 10.0, 1.0), n.updated_at DESC
             LIMIT ?3",
        )?;
        let rows = stmt
            .query_map(
                rusqlite::params![
                    fts_query,
                    workspace_id.to_string(),
                    limit,
                    HIGHLIGHT_START.to_string(),
                    HIGHLIGHT_END.to_string()
                ],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|(id, snippet)| {
                Ok(SearchHit {
                    note: self.get_note(crate::docs::parse_uuid(&id)?)?,
                    snippet,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_building_escapes_syntax() {
        assert_eq!(build_fts_query("  "), None);
        assert_eq!(build_fts_query("* : ()"), None);
        assert_eq!(
            build_fts_query("caf AND \"x"),
            Some("\"caf\"* \"AND\"* \"x\"*".to_owned())
        );
    }
}
