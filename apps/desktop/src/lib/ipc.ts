// Typed wrappers around Tauri commands. Every frontend call into Rust goes through here.
// Types mirror the serde types in crates/notetice-core and apps/desktop/src-tauri/src/commands.rs.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Uuid = string;

export interface AppInfo {
  version: string;
  dataDir: string;
  deviceId: Uuid;
}

export interface Workspace {
  id: Uuid;
  name: string;
  syncPath: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface NoteMeta {
  id: Uuid;
  workspaceId: Uuid;
  title: string;
  folderId: Uuid | null;
  pinned: boolean;
  inInbox: boolean;
  createdAt: number;
  updatedAt: number;
  deletedAt: number | null;
}

export interface Folder {
  id: Uuid;
  name: string;
  parentId: Uuid | null;
  createdAt: number;
  deletedAt: number | null;
}

export type NoteFilter =
  | { kind: "all" }
  | { kind: "inbox" }
  | { kind: "pinned" }
  | { kind: "folder"; folderId: Uuid }
  | { kind: "trash" };

/** Absent fields are left unchanged; `folderId: null` moves the note out of its folder. */
export interface NotePatch {
  title?: string;
  folderId?: Uuid | null;
  pinned?: boolean;
  inInbox?: boolean;
}

export interface SearchHit {
  note: NoteMeta;
  /** Excerpt with matches wrapped in HIGHLIGHT_START / HIGHLIGHT_END. */
  snippet: string;
}

export const HIGHLIGHT_START = "\u0002";
export const HIGHLIGHT_END = "\u0003";

export interface Attachment {
  id: Uuid;
  workspaceId: Uuid;
  hash: string;
  name: string;
  mime: string;
  size: number;
  createdAt: number;
}

export interface ExportAttachment {
  id: Uuid;
  /** Relative target path with `/` separators. */
  path: string;
}

export interface ExportFile {
  /** Relative path with `/` separators. */
  path: string;
  contents: string;
}

/** Header carrying the document ID of a binary `apply_local_update` request. */
export const DOC_ID_HEADER = "notetice-doc-id";

export function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}

export function listWorkspaces(): Promise<Workspace[]> {
  return invoke<Workspace[]>("list_workspaces");
}

export function listNotes(workspaceId: Uuid, filter: NoteFilter): Promise<NoteMeta[]> {
  return invoke<NoteMeta[]>("list_notes", { workspaceId, filter });
}

export function getNote(noteId: Uuid): Promise<NoteMeta> {
  return invoke<NoteMeta>("get_note", { noteId });
}

export function createNote(
  workspaceId: Uuid,
  options: { folderId?: Uuid | null; inInbox?: boolean } = {},
): Promise<NoteMeta> {
  return invoke<NoteMeta>("create_note", {
    workspaceId,
    folderId: options.folderId ?? null,
    inInbox: options.inInbox ?? false,
  });
}

/** Full document state as a Yjs v1 update, transferred as raw bytes. */
export async function loadDoc(docId: Uuid): Promise<Uint8Array> {
  const buf = await invoke<ArrayBuffer | number[]>("load_doc", { docId });
  return buf instanceof ArrayBuffer ? new Uint8Array(buf) : Uint8Array.from(buf);
}

/** Sends a Yjs v1 update from the editor as a raw binary body. */
export function applyLocalUpdate(docId: Uuid, update: Uint8Array): Promise<NoteMeta> {
  return invoke<NoteMeta>("apply_local_update", update, {
    headers: { [DOC_ID_HEADER]: docId },
  });
}

export function updateNoteMeta(noteId: Uuid, patch: NotePatch): Promise<NoteMeta> {
  return invoke<NoteMeta>("update_note_meta", { noteId, patch });
}

export function deleteNote(noteId: Uuid): Promise<NoteMeta> {
  return invoke<NoteMeta>("delete_note", { noteId });
}

export function restoreNote(noteId: Uuid): Promise<NoteMeta> {
  return invoke<NoteMeta>("restore_note", { noteId });
}

export function listFolders(workspaceId: Uuid): Promise<Folder[]> {
  return invoke<Folder[]>("list_folders", { workspaceId });
}

export function createFolder(workspaceId: Uuid, name: string): Promise<Folder> {
  return invoke<Folder>("create_folder", { workspaceId, name });
}

export function renameFolder(workspaceId: Uuid, folderId: Uuid, name: string): Promise<void> {
  return invoke<void>("rename_folder", { workspaceId, folderId, name });
}

export function deleteFolder(workspaceId: Uuid, folderId: Uuid): Promise<void> {
  return invoke<void>("delete_folder", { workspaceId, folderId });
}

export function search(workspaceId: Uuid, query: string, limit?: number): Promise<SearchHit[]> {
  return invoke<SearchHit[]>("search", { workspaceId, query, limit: limit ?? null });
}

export function getSettings(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>("get_settings");
}

export function setSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_setting", { key, value });
}

/** Writes files (and copies of attachments) under an absolute directory; returns how many
 * note files were written. */
export function writeExportFiles(
  dir: string,
  files: ExportFile[],
  attachments: ExportAttachment[] = [],
): Promise<number> {
  return invoke<number>("write_export_files", { dir, files, attachments });
}

/** Stores a file inserted into a note; the contents travel as a raw binary body. */
export async function saveAttachment(workspaceId: Uuid, file: File): Promise<Attachment> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  return invoke<Attachment>("save_attachment", bytes, {
    headers: {
      "notetice-workspace-id": workspaceId,
      "notetice-file-name": encodeURIComponent(file.name),
      "notetice-mime": file.type || "application/octet-stream",
    },
  });
}

export function getAttachments(ids: Uuid[]): Promise<Attachment[]> {
  return invoke<Attachment[]>("get_attachments", { ids });
}

export function onNotesChanged(
  handler: (payload: { workspaceId: Uuid }) => void,
): Promise<UnlistenFn> {
  return listen<{ workspaceId: Uuid }>("notes-changed", (event) => handler(event.payload));
}

/** Splits a search snippet into plain and highlighted parts. */
export function splitSnippet(snippet: string): { text: string; match: boolean }[] {
  const parts: { text: string; match: boolean }[] = [];
  let rest = snippet;
  while (rest.length > 0) {
    const start = rest.indexOf(HIGHLIGHT_START);
    if (start === -1) {
      parts.push({ text: rest, match: false });
      break;
    }
    if (start > 0) parts.push({ text: rest.slice(0, start), match: false });
    const end = rest.indexOf(HIGHLIGHT_END, start + 1);
    const stop = end === -1 ? rest.length : end;
    parts.push({ text: rest.slice(start + 1, stop), match: true });
    rest = end === -1 ? "" : rest.slice(end + 1);
  }
  return parts.filter((p) => p.text.length > 0);
}
