import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Download, List, PanelLeft } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { referencedAttachments } from "../lib/attachments";
import {
  createFolder,
  createNote,
  deleteFolder,
  deleteNote,
  type Folder,
  getAttachments,
  getNote,
  getSettings,
  listFolders,
  listNotes,
  listWorkspaces,
  loadDoc,
  type NoteFilter,
  type NoteMeta,
  type NotePatch,
  onNotesChanged,
  renameFolder,
  restoreNote,
  type SearchHit,
  search,
  setSetting,
  type Uuid,
  updateNoteMeta,
  type Workspace,
  writeExportFiles,
} from "../lib/ipc";
import {
  linkAttachments,
  markdownFileName,
  noteToMarkdown,
  splitPath,
  workspaceExportFiles,
} from "../lib/markdown";
import { flushAll } from "../lib/pendingWrites";
import { NoteEditor } from "./NoteEditor";
import { NoteList } from "./NoteList";
import { Sidebar } from "./Sidebar";

const SEARCH_DELAY_MS = 120;
const SIDEBAR_SETTING = "ui.sidebar_collapsed";
const LIST_SETTING = "ui.list_collapsed";
const IS_MAC = navigator.platform.toLowerCase().includes("mac");
const MOD = IS_MAC ? "⌘" : "Ctrl+";
const TOAST_MS = 5000;

function errorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) return String(err.message);
  return String(err);
}

function viewHeading(view: NoteFilter, folders: Folder[]): string {
  switch (view.kind) {
    case "inbox":
      return "Inbox";
    case "all":
      return "All notes";
    case "pinned":
      return "Pinned";
    case "trash":
      return "Trash";
    case "folder":
      return folders.find((f) => f.id === view.folderId)?.name ?? "Folder";
  }
}

function emptyText(view: NoteFilter): string {
  switch (view.kind) {
    case "inbox":
      return "Your Inbox is clear. Quick notes land here until you sort them.";
    case "pinned":
      return "No pinned notes. Pin a note to keep it at hand.";
    case "trash":
      return "Trash is empty.";
    case "folder":
      return "This folder is empty.";
    case "all":
      return "No notes yet. Start one with New note.";
  }
}

export function MainWindow() {
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [view, setView] = useState<NoteFilter>({ kind: "all" });
  const [notes, setNotes] = useState<NoteMeta[]>([]);
  const [folders, setFolders] = useState<Folder[]>([]);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[] | null>(null);
  const [selected, setSelected] = useState<NoteMeta | null>(null);
  const [freshNoteId, setFreshNoteId] = useState<Uuid | null>(null);
  const [loading, setLoading] = useState(true);
  const [toast, setToast] = useState<string | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [listCollapsed, setListCollapsed] = useState(false);

  const report = useCallback((message: string) => setToast(message), []);
  const fail = useCallback((err: unknown) => report(errorMessage(err)), [report]);

  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), TOAST_MS);
    return () => clearTimeout(t);
  }, [toast]);

  useEffect(() => {
    getSettings()
      .then((s) => {
        setSidebarCollapsed(s[SIDEBAR_SETTING] === "true");
        setListCollapsed(s[LIST_SETTING] === "true");
      })
      .catch(fail);
  }, [fail]);

  const toggleSidebar = useCallback(() => {
    setSidebarCollapsed((v) => {
      setSetting(SIDEBAR_SETTING, String(!v)).catch(fail);
      return !v;
    });
  }, [fail]);

  const toggleList = useCallback(() => {
    setListCollapsed((v) => {
      setSetting(LIST_SETTING, String(!v)).catch(fail);
      return !v;
    });
  }, [fail]);

  useEffect(() => {
    listWorkspaces()
      .then((ws) => setWorkspace(ws[0] ?? null))
      .catch(fail);
  }, [fail]);

  const workspaceId = workspace?.id ?? null;

  const reload = useCallback(async () => {
    if (!workspaceId) return;
    const [list, folderList] = await Promise.all([
      listNotes(workspaceId, view),
      listFolders(workspaceId),
    ]);
    setNotes(list);
    setFolders(folderList);
    setLoading(false);
    // A folder that disappeared (deleted) falls back to All notes.
    if (view.kind === "folder" && !folderList.some((f) => f.id === view.folderId)) {
      setView({ kind: "all" });
    }
  }, [workspaceId, view]);

  useEffect(() => {
    reload().catch(fail);
  }, [reload, fail]);

  useEffect(() => {
    if (!workspaceId) return;
    const unlisten = onNotesChanged((p) => {
      if (p.workspaceId === workspaceId) reload().catch(fail);
    });
    return () => {
      void unlisten.then((f) => f());
    };
  }, [workspaceId, reload, fail]);

  // Keep the open note's metadata current after any change.
  const selectedId = selected?.id ?? null;
  // biome-ignore lint/correctness/useExhaustiveDependencies: refresh when the list changes
  useEffect(() => {
    if (!selectedId) return;
    getNote(selectedId)
      .then(setSelected)
      .catch(() => setSelected(null));
  }, [selectedId, notes]);

  // Search as you type.
  useEffect(() => {
    if (!workspaceId || query.trim() === "") {
      setHits(null);
      return;
    }
    const t = setTimeout(() => {
      search(workspaceId, query).then(setHits).catch(fail);
    }, SEARCH_DELAY_MS);
    return () => clearTimeout(t);
  }, [workspaceId, query, fail]);

  // Save everything before the window closes.
  useEffect(() => {
    const unlisten = getCurrentWindow().onCloseRequested(async () => {
      await flushAll();
    });
    return () => {
      void unlisten.then((f) => f());
    };
  }, []);

  const select = useCallback(
    (id: Uuid) => {
      getNote(id).then(setSelected).catch(fail);
    },
    [fail],
  );

  const create = useCallback(async () => {
    if (!workspaceId) return;
    try {
      const note = await createNote(workspaceId, {
        folderId: view.kind === "folder" ? view.folderId : null,
        inInbox: view.kind === "inbox",
      });
      if (view.kind === "trash" || view.kind === "pinned") setView({ kind: "all" });
      setQuery("");
      setFreshNoteId(note.id);
      setSelected(note);
      await reload();
    } catch (err) {
      fail(err);
    }
  }, [workspaceId, view, reload, fail]);

  // Global shortcuts: Cmd/Ctrl+N new note, Cmd/Ctrl+F search, Cmd/Ctrl+\ sidebar,
  // Cmd/Ctrl+Shift+\ note list.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey) || e.altKey) return;
      if (e.code === "Backslash") {
        e.preventDefault();
        if (e.shiftKey) toggleList();
        else toggleSidebar();
        return;
      }
      if (e.shiftKey) return;
      const key = e.key.toLowerCase();
      if (key === "n") {
        e.preventDefault();
        void create();
      } else if (key === "f") {
        e.preventDefault();
        searchRef.current?.focus();
        searchRef.current?.select();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [create, toggleList, toggleSidebar]);

  const patch = useCallback(
    (p: NotePatch) => {
      if (!selected) return;
      updateNoteMeta(selected.id, p).then(setSelected).catch(fail);
    },
    [selected, fail],
  );

  const removeSelected = useCallback(() => {
    if (!selected) return;
    const id = selected.id;
    deleteNote(id)
      .then(() => {
        setSelected(null);
        report("Moved to Trash.");
      })
      .catch(fail);
  }, [selected, fail, report]);

  const restoreSelected = useCallback(() => {
    if (!selected) return;
    restoreNote(selected.id).then(setSelected).catch(fail);
  }, [selected, fail]);

  const exportSelected = useCallback(async () => {
    if (!selected) return;
    try {
      await flushAll();
      const path = await save({
        title: "Export note as Markdown",
        defaultPath: markdownFileName(selected.title),
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });
      if (!path) return;
      const contents = noteToMarkdown(selected.title, await loadDoc(selected.id));
      const { dir, name } = splitPath(path);
      const ids = referencedAttachments(contents);
      const linked = linkAttachments(
        [{ path: name, contents }],
        ids.length > 0 ? await getAttachments(ids) : [],
        "",
      );
      await writeExportFiles(dir, linked.files, linked.copies);
      report(`Exported to ${name}.`);
    } catch (err) {
      fail(err);
    }
  }, [selected, report, fail]);

  const exportAll = useCallback(async () => {
    if (!workspaceId) return;
    try {
      const dir = await open({ title: "Choose where to export all notes", directory: true });
      if (typeof dir !== "string") return;
      await flushAll();
      const all = await listNotes(workspaceId, { kind: "all" });
      const states = new Map<string, Uint8Array>();
      for (const n of all) states.set(n.id, await loadDoc(n.id));
      const day = new Date().toISOString().slice(0, 10);
      const root = `Ichnote export ${day}`;
      const files = workspaceExportFiles(all, folders, states, root);
      const ids = referencedAttachments(files.map((f) => f.contents).join("\n"));
      const linked = linkAttachments(files, ids.length > 0 ? await getAttachments(ids) : [], root);
      const count = await writeExportFiles(dir, linked.files, linked.copies);
      report(`Exported ${count === 1 ? "1 note" : `${count} notes`} to “${root}”.`);
    } catch (err) {
      fail(err);
    }
  }, [workspaceId, folders, report, fail]);

  const heading = query.trim() ? `Results for “${query.trim()}”` : viewHeading(view, folders);
  const listEmpty = query.trim() ? `No notes match “${query.trim()}”.` : emptyText(view);
  const canCreate = view.kind !== "trash" && query.trim() === "";

  const folderActions = useMemo(
    () => ({
      create: (name: string) => {
        if (!workspaceId) return;
        createFolder(workspaceId, name)
          .then((f) => setView({ kind: "folder", folderId: f.id }))
          .catch(fail);
      },
      rename: (id: Uuid, name: string) => {
        if (workspaceId) renameFolder(workspaceId, id, name).catch(fail);
      },
      remove: (id: Uuid) => {
        if (!workspaceId) return;
        deleteFolder(workspaceId, id)
          .then(() => report("Folder deleted. Its notes are in All notes."))
          .catch(fail);
      },
    }),
    [workspaceId, fail, report],
  );

  const expandButtons = listCollapsed && (
    <button
      type="button"
      className="icon-button"
      aria-label="Show note list"
      title={`Show note list (${MOD}⇧\\)`}
      onClick={toggleList}
    >
      <List size={16} aria-hidden="true" />
    </button>
  );

  return (
    <div
      className="app"
      data-platform={IS_MAC ? "mac" : "other"}
      data-sidebar={sidebarCollapsed ? "collapsed" : "open"}
      data-list={listCollapsed ? "collapsed" : "open"}
    >
      <div className="pane-slot" inert={sidebarCollapsed}>
        <Sidebar
          view={view}
          onView={setView}
          folders={folders}
          query={query}
          onQuery={setQuery}
          searchRef={searchRef}
          onCreateFolder={folderActions.create}
          onRenameFolder={folderActions.rename}
          onDeleteFolder={folderActions.remove}
          onExportAll={() => void exportAll()}
          workspaceName={workspace?.name ?? ""}
        />
      </div>
      <div className="pane-slot" inert={listCollapsed}>
        <NoteList
          heading={heading}
          notes={notes}
          hits={hits}
          selectedId={selected?.id ?? null}
          onSelect={select}
          onCreate={canCreate ? () => void create() : null}
          emptyText={listEmpty}
          loading={loading}
          onCollapse={toggleList}
        />
      </div>
      {selected ? (
        <NoteEditor
          key={selected.id}
          note={selected}
          folders={folders}
          autoFocusTitle={selected.id === freshNoteId}
          onPatch={patch}
          onDelete={removeSelected}
          onRestore={restoreSelected}
          onError={report}
          toolbarStart={expandButtons}
          toolbarExtra={
            <button
              type="button"
              className="icon-button"
              aria-label="Export as Markdown"
              title="Export as Markdown"
              onClick={() => void exportSelected()}
            >
              <Download size={16} aria-hidden="true" />
            </button>
          }
        />
      ) : (
        <div className="editor-pane">
          <div className="editor-toolbar" data-tauri-drag-region>
            <div className="toolbar-actions">{expandButtons}</div>
          </div>
          <div className="editor-empty">
            <p>Select a note, or start a new one.</p>
            <p>
              <kbd>{MOD}N</kbd> New note · <kbd>{MOD}F</kbd> Search
            </p>
          </div>
        </div>
      )}
      {/* Window controls: right of the macOS traffic lights; stays put when the sidebar hides. */}
      <div className="titlebar-controls">
        <button
          type="button"
          className="icon-button titlebar-button"
          aria-label={sidebarCollapsed ? "Show sidebar" : "Hide sidebar"}
          aria-expanded={!sidebarCollapsed}
          title={`${sidebarCollapsed ? "Show" : "Hide"} sidebar (${MOD}\\)`}
          onClick={toggleSidebar}
        >
          <PanelLeft size={16} aria-hidden="true" />
        </button>
      </div>
      <div className="toast-region" aria-live="polite">
        {toast && (
          <div className="toast" role="status">
            {toast}
          </div>
        )}
      </div>
    </div>
  );
}
