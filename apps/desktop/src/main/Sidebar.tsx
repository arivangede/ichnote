import {
  Check,
  FileDown,
  FolderClosed,
  FolderPlus,
  Inbox,
  Notebook,
  Pin,
  Search,
  Trash2,
  X,
} from "lucide-react";
import { type FormEvent, type KeyboardEvent, type RefObject, useState } from "react";
import type { Folder, NoteFilter, Uuid } from "../lib/ipc";
import { LogoMark } from "./LogoMark";

interface SidebarProps {
  view: NoteFilter;
  onView: (view: NoteFilter) => void;
  folders: Folder[];
  query: string;
  onQuery: (query: string) => void;
  searchRef: RefObject<HTMLInputElement | null>;
  onCreateFolder: (name: string) => void;
  onRenameFolder: (id: Uuid, name: string) => void;
  onDeleteFolder: (id: Uuid) => void;
  onExportAll: () => void;
  workspaceName: string;
}

const MOD = navigator.platform.toLowerCase().includes("mac") ? "⌘" : "Ctrl+";

function sameView(a: NoteFilter, b: NoteFilter): boolean {
  if (a.kind === "folder" && b.kind === "folder") return a.folderId === b.folderId;
  return a.kind === b.kind;
}

export function Sidebar(props: SidebarProps) {
  const { view, onView, folders, query, onQuery, searchRef } = props;
  const [adding, setAdding] = useState(false);
  const [renaming, setRenaming] = useState<Uuid | null>(null);
  const searching = query.trim() !== "";

  const navItem = (target: NoteFilter, label: string, icon: React.ReactNode) => {
    const current = !searching && sameView(view, target);
    return (
      <button
        type="button"
        className="nav-item"
        aria-current={current ? "page" : undefined}
        onClick={() => {
          onQuery("");
          onView(target);
        }}
      >
        {icon}
        <span className="nav-label">{label}</span>
      </button>
    );
  };

  return (
    <aside className="sidebar">
      <div className="titlebar-strip" data-tauri-drag-region />
      <div className="brand">
        <span className="brand-mark">
          <LogoMark size={18} />
        </span>
        <span className="brand-name">{props.workspaceName}</span>
      </div>

      <search className="search">
        <Search size={16} aria-hidden="true" />
        <input
          ref={searchRef}
          type="search"
          value={query}
          placeholder="Search notes"
          aria-label="Search notes"
          aria-keyshortcuts="Meta+F"
          onChange={(e) => onQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              onQuery("");
              e.currentTarget.blur();
            }
          }}
        />
        <span className="search-hint" aria-hidden="true">
          {MOD}F
        </span>
      </search>

      <nav className="nav" aria-label="Notes">
        {navItem({ kind: "inbox" }, "Inbox", <Inbox size={16} aria-hidden="true" />)}
        {navItem({ kind: "all" }, "All notes", <Notebook size={16} aria-hidden="true" />)}
        {navItem({ kind: "pinned" }, "Pinned", <Pin size={16} aria-hidden="true" />)}

        <div className="nav-section">
          <h2 className="nav-heading" id="folders-heading">
            Folders
          </h2>
          <button
            type="button"
            className="icon-button on-ink"
            aria-label="New folder"
            title="New folder"
            onClick={() => setAdding(true)}
          >
            <FolderPlus size={16} aria-hidden="true" />
          </button>
        </div>
        <ul className="folder-list" aria-labelledby="folders-heading">
          {folders.map((f) =>
            renaming === f.id ? (
              <li key={f.id}>
                <FolderNameForm
                  initial={f.name}
                  label={`Rename folder ${f.name}`}
                  onDone={(name) => {
                    setRenaming(null);
                    if (name && name !== f.name) props.onRenameFolder(f.id, name);
                  }}
                />
              </li>
            ) : (
              <li key={f.id} className="folder-row">
                <button
                  type="button"
                  className="nav-item"
                  aria-current={
                    !searching && sameView(view, { kind: "folder", folderId: f.id })
                      ? "page"
                      : undefined
                  }
                  onClick={() => {
                    onQuery("");
                    onView({ kind: "folder", folderId: f.id });
                  }}
                  onDoubleClick={() => setRenaming(f.id)}
                  onKeyDown={(e) => {
                    if (e.key === "F2" || (e.key === "Enter" && e.altKey)) setRenaming(f.id);
                  }}
                  title="Double-click or F2 to rename"
                >
                  <FolderClosed size={16} aria-hidden="true" />
                  <span className="nav-label">{f.name}</span>
                </button>
                <button
                  type="button"
                  className="icon-button on-ink folder-delete"
                  aria-label={`Delete folder ${f.name} (its notes are kept)`}
                  title="Delete folder (its notes are kept)"
                  onClick={() => props.onDeleteFolder(f.id)}
                >
                  <X size={14} aria-hidden="true" />
                </button>
              </li>
            ),
          )}
          {adding && (
            <li>
              <FolderNameForm
                initial=""
                label="New folder name"
                onDone={(name) => {
                  setAdding(false);
                  if (name) props.onCreateFolder(name);
                }}
              />
            </li>
          )}
          {folders.length === 0 && !adding && <li className="nav-empty">No folders yet</li>}
        </ul>
      </nav>

      <div className="nav nav-bottom">
        {navItem({ kind: "trash" }, "Trash", <Trash2 size={16} aria-hidden="true" />)}
        <button type="button" className="nav-item" onClick={props.onExportAll}>
          <FileDown size={16} aria-hidden="true" />
          <span className="nav-label">Export all as Markdown</span>
        </button>
      </div>
    </aside>
  );
}

function FolderNameForm(props: {
  initial: string;
  label: string;
  onDone: (name: string | null) => void;
}) {
  const [name, setName] = useState(props.initial);
  const submit = (e?: FormEvent) => {
    e?.preventDefault();
    props.onDone(name.trim() || null);
  };
  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") props.onDone(null);
  };
  return (
    <form className="folder-form" onSubmit={submit}>
      <input
        // biome-ignore lint/a11y/noAutofocus: the field appears because the user asked for it
        autoFocus
        aria-label={props.label}
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={onKeyDown}
        onBlur={() => submit()}
      />
      <button type="submit" className="icon-button on-ink" aria-label="Save folder name">
        <Check size={14} aria-hidden="true" />
      </button>
    </form>
  );
}
