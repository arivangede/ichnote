//! Tauri commands (DESIGN.md section 8). Every frontend call into Rust goes through here.

use std::collections::BTreeMap;
use std::path::PathBuf;

use ichnote_core::export::ExportFile;
use ichnote_core::{Attachment, Folder, NoteFilter, NoteMeta, NotePatch, SearchHit, Workspace};
use serde::{Deserialize, Serialize};
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::state::AppState;

/// Header carrying the document ID for binary `apply_local_update` requests.
pub const DOC_ID_HEADER: &str = "ichnote-doc-id";
/// Settings that the frontend may not change.
const READ_ONLY_SETTINGS: &[&str] = &[ichnote_core::db::DEVICE_ID_KEY];
const DEFAULT_SEARCH_LIMIT: u32 = 50;
/// Headers of a binary `save_attachment` request.
pub const WORKSPACE_ID_HEADER: &str = "ichnote-workspace-id";
pub const FILE_NAME_HEADER: &str = "ichnote-file-name";
pub const MIME_HEADER: &str = "ichnote-mime";
/// Largest attachment accepted (bytes).
const MAX_ATTACHMENT_BYTES: usize = 256 * 1024 * 1024;

/// Error returned to the frontend as `{ message }`.
#[derive(Debug, Serialize)]
pub struct CommandError {
    message: String,
}

impl<E: std::fmt::Display> From<E> for CommandError {
    fn from(err: E) -> Self {
        let message = err.to_string();
        tracing::warn!(error = %message, "command failed");
        Self { message }
    }
}

type CmdResult<T> = Result<T, CommandError>;

/// Payload of the `notes-changed` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotesChanged {
    workspace_id: Uuid,
}

fn notes_changed(app: &AppHandle, workspace_id: Uuid) {
    if let Err(err) = app.emit("notes-changed", NotesChanged { workspace_id }) {
        tracing::warn!(error = %err, "failed to emit notes-changed");
    }
}

/// Basic facts about the running app.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    data_dir: PathBuf,
    device_id: Uuid,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: ichnote_core::VERSION,
        data_dir: state.data_dir().to_owned(),
        device_id: state.store().device_id(),
    }
}

#[tauri::command]
pub async fn list_workspaces(state: State<'_, AppState>) -> CmdResult<Vec<Workspace>> {
    Ok(state.store().list_workspaces()?)
}

#[tauri::command]
pub async fn create_workspace(state: State<'_, AppState>, name: String) -> CmdResult<Workspace> {
    Ok(state.store().create_workspace(name.trim())?)
}

#[tauri::command]
pub async fn list_notes(
    state: State<'_, AppState>,
    workspace_id: Uuid,
    filter: NoteFilter,
) -> CmdResult<Vec<NoteMeta>> {
    Ok(state.store().list_notes(workspace_id, filter)?)
}

#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, note_id: Uuid) -> CmdResult<NoteMeta> {
    Ok(state.store().get_note(note_id)?)
}

#[tauri::command]
pub async fn create_note(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: Uuid,
    folder_id: Option<Uuid>,
    in_inbox: Option<bool>,
) -> CmdResult<NoteMeta> {
    let note = state
        .store()
        .create_note(workspace_id, folder_id, in_inbox.unwrap_or(false))?;
    notes_changed(&app, workspace_id);
    Ok(note)
}

/// Returns the document state as raw bytes (an `ArrayBuffer` in JS), not a JSON array.
#[tauri::command]
pub async fn load_doc(state: State<'_, AppState>, doc_id: Uuid) -> CmdResult<Response> {
    Ok(Response::new(state.store().load_doc(doc_id)?))
}

/// Applies an editor update. The body is the raw Yjs update; the doc ID is in a header.
#[tauri::command]
pub async fn apply_local_update(
    app: AppHandle,
    state: State<'_, AppState>,
    request: Request<'_>,
) -> CmdResult<NoteMeta> {
    let InvokeBody::Raw(update) = request.body() else {
        return Err(CommandError::from("expected a binary request body"));
    };
    let doc_id = request
        .headers()
        .get(DOC_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| CommandError::from(format!("missing {DOC_ID_HEADER} header")))?;
    let doc_id = Uuid::parse_str(doc_id)?;
    let before = state.store().get_note(doc_id)?;
    let note = state.store().apply_local_update(doc_id, update)?;
    if note.updated_at != before.updated_at {
        notes_changed(&app, note.workspace_id);
    }
    Ok(note)
}

#[tauri::command]
pub async fn update_note_meta(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: Uuid,
    patch: NotePatch,
) -> CmdResult<NoteMeta> {
    let note = state.store().update_note_meta(note_id, &patch)?;
    notes_changed(&app, note.workspace_id);
    Ok(note)
}

#[tauri::command]
pub async fn delete_note(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: Uuid,
) -> CmdResult<NoteMeta> {
    let note = state.store().delete_note(note_id)?;
    notes_changed(&app, note.workspace_id);
    Ok(note)
}

#[tauri::command]
pub async fn restore_note(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: Uuid,
) -> CmdResult<NoteMeta> {
    let note = state.store().restore_note(note_id)?;
    notes_changed(&app, note.workspace_id);
    Ok(note)
}

#[tauri::command]
pub async fn list_folders(
    state: State<'_, AppState>,
    workspace_id: Uuid,
) -> CmdResult<Vec<Folder>> {
    Ok(state.store().list_folders(workspace_id)?)
}

#[tauri::command]
pub async fn create_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: Uuid,
    name: String,
) -> CmdResult<Folder> {
    let folder = state
        .store()
        .create_folder(workspace_id, name.trim(), None)?;
    notes_changed(&app, workspace_id);
    Ok(folder)
}

#[tauri::command]
pub async fn rename_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: Uuid,
    folder_id: Uuid,
    name: String,
) -> CmdResult<()> {
    state
        .store()
        .rename_folder(workspace_id, folder_id, name.trim())?;
    notes_changed(&app, workspace_id);
    Ok(())
}

#[tauri::command]
pub async fn delete_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: Uuid,
    folder_id: Uuid,
) -> CmdResult<()> {
    state.store().delete_folder(workspace_id, folder_id)?;
    notes_changed(&app, workspace_id);
    Ok(())
}

#[tauri::command]
pub async fn search(
    state: State<'_, AppState>,
    workspace_id: Uuid,
    query: String,
    limit: Option<u32>,
) -> CmdResult<Vec<SearchHit>> {
    Ok(state
        .store()
        .search(workspace_id, &query, limit.unwrap_or(DEFAULT_SEARCH_LIMIT))?)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> CmdResult<BTreeMap<String, String>> {
    Ok(state.store().settings()?)
}

#[tauri::command]
pub async fn set_setting(state: State<'_, AppState>, key: String, value: String) -> CmdResult<()> {
    if READ_ONLY_SETTINGS.contains(&key.as_str()) {
        return Err(CommandError::from(format!("setting {key} is read-only")));
    }
    Ok(state.store().set_setting(&key, &value)?)
}

fn header<'a>(request: &'a Request<'_>, name: &str) -> CmdResult<&'a str> {
    request
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| CommandError::from(format!("missing {name} header")))
}

/// Stores a file inserted into a note. The body is the raw file; metadata is in headers
/// (the file name is `encodeURIComponent`-encoded).
#[tauri::command]
pub async fn save_attachment(
    state: State<'_, AppState>,
    request: Request<'_>,
) -> CmdResult<Attachment> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err(CommandError::from("expected a binary request body"));
    };
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Err(CommandError::from(format!(
            "file is too large (limit {} MB)",
            MAX_ATTACHMENT_BYTES / 1024 / 1024
        )));
    }
    let workspace_id = Uuid::parse_str(header(&request, WORKSPACE_ID_HEADER)?)?;
    let name = crate::attachments::percent_decode(header(&request, FILE_NAME_HEADER)?)
        .ok_or_else(|| CommandError::from("invalid file name encoding"))?;
    let mime = header(&request, MIME_HEADER).unwrap_or("application/octet-stream");
    Ok(state
        .store()
        .save_attachment(workspace_id, &name, mime, bytes)?)
}

/// Metadata of attachments (for export).
#[tauri::command]
pub async fn get_attachments(
    state: State<'_, AppState>,
    ids: Vec<Uuid>,
) -> CmdResult<Vec<Attachment>> {
    let store = state.store();
    ids.into_iter()
        .map(|id| Ok(store.attachment(id)?.0))
        .collect()
}

/// An attachment to copy into an export, at a relative path.
#[derive(Debug, Deserialize)]
pub struct ExportAttachment {
    id: Uuid,
    path: String,
}

/// Writes exported files under `dir`, plus copies of the referenced attachments.
/// Returns the number of note files written.
#[tauri::command]
pub async fn write_export_files(
    state: State<'_, AppState>,
    dir: PathBuf,
    files: Vec<ExportFile>,
    attachments: Option<Vec<ExportAttachment>>,
) -> CmdResult<usize> {
    let copies = {
        let store = state.store();
        attachments
            .unwrap_or_default()
            .into_iter()
            .map(|a| Ok((a.path, store.attachment(a.id)?.1)))
            .collect::<CmdResult<Vec<_>>>()?
    };
    let written = tauri::async_runtime::spawn_blocking(move || {
        ichnote_core::export::copy_files(&dir, &copies)?;
        ichnote_core::export::write_files(&dir, &files)
    })
    .await??;
    Ok(written.len())
}
