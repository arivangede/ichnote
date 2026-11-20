//! Integration tests for workspaces, notes, folders, and document updates.
#![allow(clippy::unwrap_used)]

use notetice_core::{NoteFilter, NotePatch, Store};
use uuid::Uuid;
use yrs::types::xml::XmlFragment;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, ReadTxn, Transact, Update, XmlTextPrelim};

const FIXTURE: &[u8] = include_bytes!("fixtures/blocknote-note.bin");

/// Simulates the editor: a Yjs client that loads the stored state and makes an edit.
fn editor_edit(state: &[u8], text: &str) -> Vec<u8> {
    let doc = Doc::new();
    let frag = doc.get_or_insert_xml_fragment("document-store");
    let mut txn = doc.transact_mut();
    txn.apply_update(Update::decode_v1(state).unwrap()).unwrap();
    let before = txn.state_vector();
    let len = frag.len(&txn);
    frag.insert(&mut txn, len, XmlTextPrelim::new(text));
    drop(txn);
    doc.transact().encode_state_as_update_v1(&before)
}

fn doc_text(state: &[u8]) -> String {
    let doc = Doc::new();
    let frag = doc.get_or_insert_xml_fragment("document-store");
    doc.transact_mut()
        .apply_update(Update::decode_v1(state).unwrap())
        .unwrap();
    let txn = doc.transact();
    frag.get_string(&txn)
}

fn fts_body(store_path: &std::path::Path, note: Uuid) -> String {
    let conn = rusqlite::Connection::open(store_path).unwrap();
    conn.query_row(
        "SELECT body FROM notes_fts WHERE note_id = ?1",
        [note.to_string()],
        |r| r.get(0),
    )
    .unwrap()
}

fn ids(notes: &[notetice_core::NoteMeta]) -> Vec<Uuid> {
    notes.iter().map(|n| n.id).collect()
}

#[test]
fn default_workspace_is_created_once() {
    let mut store = Store::open_in_memory().unwrap();
    let a = store.ensure_default_workspace("Personal").unwrap();
    let b = store.ensure_default_workspace("Other").unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(a.id.get_version_num(), 7);
    assert_eq!(store.list_workspaces().unwrap().len(), 1);
}

#[test]
fn note_lifecycle_and_filters() {
    let mut store = Store::open_in_memory().unwrap();
    let ws = store.create_workspace("Personal").unwrap().id;
    let folder = store.create_folder(ws, "Work", None).unwrap();

    let a = store.create_note(ws, None, true).unwrap();
    let b = store.create_note(ws, Some(folder.id), false).unwrap();
    assert!(a.in_inbox && a.title.is_empty());

    let a = store
        .update_note_meta(
            a.id,
            &NotePatch {
                title: Some("Hello".into()),
                pinned: Some(true),
                ..NotePatch::default()
            },
        )
        .unwrap();
    assert_eq!(a.title, "Hello");
    assert!(a.pinned);

    assert_eq!(store.list_notes(ws, NoteFilter::All).unwrap().len(), 2);
    assert_eq!(
        ids(&store.list_notes(ws, NoteFilter::Inbox).unwrap()),
        vec![a.id]
    );
    assert_eq!(
        ids(&store.list_notes(ws, NoteFilter::Pinned).unwrap()),
        vec![a.id]
    );
    assert_eq!(
        ids(&store.list_notes(ws, NoteFilter::Folder(folder.id)).unwrap()),
        vec![b.id]
    );

    // Move out of the folder.
    store
        .update_note_meta(
            b.id,
            &NotePatch {
                folder_id: Some(None),
                ..NotePatch::default()
            },
        )
        .unwrap();
    assert!(
        store
            .list_notes(ws, NoteFilter::Folder(folder.id))
            .unwrap()
            .is_empty()
    );

    // Trash and restore.
    let deleted = store.delete_note(a.id).unwrap();
    assert!(deleted.deleted_at.is_some());
    assert_eq!(
        ids(&store.list_notes(ws, NoteFilter::All).unwrap()),
        vec![b.id]
    );
    assert_eq!(
        ids(&store.list_notes(ws, NoteFilter::Trash).unwrap()),
        vec![a.id]
    );
    assert!(store.list_notes(ws, NoteFilter::Pinned).unwrap().is_empty());
    let restored = store.restore_note(a.id).unwrap();
    assert_eq!(restored.deleted_at, None);
    assert!(store.list_notes(ws, NoteFilter::Trash).unwrap().is_empty());
}

#[test]
fn deleting_a_folder_keeps_its_notes() {
    let mut store = Store::open_in_memory().unwrap();
    let ws = store.create_workspace("Personal").unwrap().id;
    let folder = store.create_folder(ws, "Ideas", None).unwrap();
    store.rename_folder(ws, folder.id, "Big ideas").unwrap();
    assert_eq!(store.list_folders(ws).unwrap()[0].name, "Big ideas");
    let note = store.create_note(ws, Some(folder.id), false).unwrap();

    store.delete_folder(ws, folder.id).unwrap();
    assert!(store.list_folders(ws).unwrap().is_empty());
    let note = store.get_note(note.id).unwrap();
    assert_eq!(note.folder_id, None);
    assert_eq!(note.deleted_at, None);
}

#[test]
fn editor_updates_are_stored_and_indexed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notetice.db");
    let mut store = Store::open(&path).unwrap();
    let ws = store.create_workspace("Personal").unwrap().id;
    let note = store.create_note(ws, None, false).unwrap();

    // First the full BlockNote document, as the editor would send it after typing.
    store.apply_local_update(note.id, FIXTURE).unwrap();
    // Then an incremental edit made on top of the stored state.
    let state = store.load_doc(note.id).unwrap();
    store
        .apply_local_update(note.id, &editor_edit(&state, "Second line"))
        .unwrap();

    let text = doc_text(&store.load_doc(note.id).unwrap());
    assert!(text.contains("Grocery list"), "{text}");
    assert!(text.ends_with("Second line"), "{text}");
    let body = fts_body(&path, note.id);
    assert!(body.starts_with("Grocery list\n"), "{body}");
    assert!(body.ends_with("\nSecond line"), "{body}");
}

#[test]
fn invalid_updates_are_rejected_without_changes() {
    let mut store = Store::open_in_memory().unwrap();
    let ws = store.create_workspace("Personal").unwrap().id;
    let note = store.create_note(ws, None, false).unwrap();
    let before = store.load_doc(note.id).unwrap();

    assert!(store.apply_local_update(note.id, &[0xff, 0x01]).is_err());
    assert_eq!(store.load_doc(note.id).unwrap(), before);
    // The meta document is owned by Rust; the editor may not write to it.
    assert!(store.apply_local_update(ws, FIXTURE).is_err());
    assert!(store.apply_local_update(Uuid::now_v7(), FIXTURE).is_err());
}

#[test]
fn data_survives_reopen_without_clean_shutdown() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notetice.db");
    let (ws, note) = {
        let mut store = Store::open(&path).unwrap();
        let ws = store.create_workspace("Personal").unwrap().id;
        let note = store.create_note(ws, None, true).unwrap().id;
        store
            .update_note_meta(
                note,
                &NotePatch {
                    title: Some("Kept".into()),
                    ..NotePatch::default()
                },
            )
            .unwrap();
        store.apply_local_update(note, FIXTURE).unwrap();
        // Simulate a crash: the store is leaked, so nothing runs on drop.
        std::mem::forget(store);
        (ws, note)
    };
    let store = Store::open(&path).unwrap();
    let meta = store.get_note(note).unwrap();
    assert_eq!(meta.title, "Kept");
    assert_eq!(meta.workspace_id, ws);
    assert!(doc_text(&store.load_doc(note).unwrap()).contains("café beans"));
}

#[test]
fn rebuilt_index_matches_incremental_index() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notetice.db");
    let mut store = Store::open(&path).unwrap();
    let ws = store.create_workspace("Personal").unwrap().id;
    let folder = store.create_folder(ws, "F", None).unwrap();
    let a = store.create_note(ws, Some(folder.id), true).unwrap().id;
    let b = store.create_note(ws, None, false).unwrap().id;
    store.apply_local_update(a, FIXTURE).unwrap();
    store
        .update_note_meta(
            b,
            &NotePatch {
                title: Some("B".into()),
                pinned: Some(true),
                ..NotePatch::default()
            },
        )
        .unwrap();
    store.delete_note(b).unwrap();

    let snapshot = |p: &std::path::Path| {
        let conn = rusqlite::Connection::open(p).unwrap();
        let mut idx: Vec<String> = conn
            .prepare("SELECT id || title || ifnull(folder_id,'') || pinned || in_inbox || created_at || updated_at || ifnull(deleted_at,'') FROM notes_index ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let fts: Vec<String> = conn
            .prepare("SELECT note_id || title || body FROM notes_fts ORDER BY note_id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        idx.extend(fts);
        idx
    };
    let incremental = snapshot(&path);
    store.rebuild_index().unwrap();
    assert_eq!(snapshot(&path), incremental);
    assert_eq!(incremental.len(), 4);
}

#[test]
fn content_edits_bump_updated_at_at_most_once_a_minute() {
    let mut store = Store::open_in_memory().unwrap();
    let ws = store.create_workspace("Personal").unwrap().id;
    let note = store.create_note(ws, None, false).unwrap();
    let meta_before = store.load_doc(ws).unwrap();
    let after = store.apply_local_update(note.id, FIXTURE).unwrap();
    // Created moments ago, so the bump is throttled and the meta doc is untouched.
    assert_eq!(after.updated_at, note.updated_at);
    assert_eq!(store.load_doc(ws).unwrap(), meta_before);
}

#[test]
fn unsynced_workspaces_do_not_fill_the_outbox() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notetice.db");
    let mut store = Store::open(&path).unwrap();
    let ws = store.create_workspace("Personal").unwrap().id;
    let note = store.create_note(ws, None, false).unwrap();
    store.apply_local_update(note.id, FIXTURE).unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    let n: i64 = conn
        .query_row("SELECT count(*) FROM outbox", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
}

#[test]
fn search_matches_titles_and_bodies_with_prefixes_and_diacritics() {
    let mut store = Store::open_in_memory().unwrap();
    let ws = store.create_workspace("Personal").unwrap().id;
    let other_ws = store.create_workspace("Other").unwrap().id;
    let grocery = store.create_note(ws, None, false).unwrap().id;
    store.apply_local_update(grocery, FIXTURE).unwrap();
    let titled = store.create_note(ws, None, false).unwrap().id;
    store
        .update_note_meta(
            titled,
            &NotePatch {
                title: Some("Cafe opening hours".into()),
                ..NotePatch::default()
            },
        )
        .unwrap();
    let elsewhere = store.create_note(other_ws, None, false).unwrap().id;
    store.apply_local_update(elsewhere, FIXTURE).unwrap();

    // "cafe" matches "café" (diacritics removed), title hits rank first.
    let hits = store.search(ws, "cafe", 20).unwrap();
    assert_eq!(
        ids(&hits.iter().map(|h| h.note.clone()).collect::<Vec<_>>()),
        vec![titled, grocery]
    );
    assert!(
        hits[1].snippet.contains("\u{2}café\u{3}"),
        "{}",
        hits[1].snippet
    );

    // Prefix match, all words required.
    assert_eq!(store.search(ws, "groc mil", 20).unwrap().len(), 1);
    assert!(store.search(ws, "grocery nothing", 20).unwrap().is_empty());
    // FTS syntax in user input is harmless.
    assert!(
        store
            .search(ws, "\"unbalanced NEAR( *", 20)
            .unwrap()
            .is_empty()
    );
    assert!(store.search(ws, "", 20).unwrap().is_empty());

    // Trashed notes are not found.
    store.delete_note(grocery).unwrap();
    assert!(store.search(ws, "milk", 20).unwrap().is_empty());
}
