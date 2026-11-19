import "@blocknote/mantine/style.css";
import type { BlockNoteEditor } from "@blocknote/core";
import { withCollaboration } from "@blocknote/core/yjs";
import { BlockNoteView, type Theme } from "@blocknote/mantine";
import { useCreateBlockNote } from "@blocknote/react";
import { Inbox, Pin, RotateCcw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { type Folder, type NoteMeta, type NotePatch, updateNoteMeta } from "../lib/ipc";
import { NoteSync, type SaveState } from "../lib/noteSync";
import { registerFlush } from "../lib/pendingWrites";
import { RegMark } from "./RegMark";

const TITLE_SAVE_DELAY_MS = 400;

interface NoteEditorProps {
  note: NoteMeta;
  folders: Folder[];
  autoFocusTitle: boolean;
  onPatch: (patch: NotePatch) => void;
  onDelete: () => void;
  onRestore: () => void;
  onError: (message: string) => void;
  toolbarExtra?: React.ReactNode;
}

function errorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) return String(err.message);
  return String(err);
}

export function NoteEditor(props: NoteEditorProps) {
  const { note, onError } = props;
  const [sync, setSync] = useState<NoteSync | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [contentState, setContentState] = useState<SaveState>("saved");
  const titleState = useTitle(note, onError);
  const editorRef = useRef<BlockNoteEditor | null>(null);
  const trashed = note.deletedAt !== null;

  useEffect(() => {
    let alive = true;
    let opened: NoteSync | null = null;
    let unregister = () => {};
    let unsubscribe = () => {};
    NoteSync.open(note.id)
      .then((s) => {
        if (!alive) {
          void s.close();
          return;
        }
        opened = s;
        unregister = registerFlush(() => s.flush());
        unsubscribe = s.subscribe(setContentState);
        setSync(s);
      })
      .catch((err: unknown) => setLoadError(errorMessage(err)));
    return () => {
      alive = false;
      unsubscribe();
      unregister();
      opened
        ?.close()
        .catch((err: unknown) => onError(`Could not save the note: ${errorMessage(err)}`));
    };
  }, [note.id, onError]);

  const saveState: SaveState =
    contentState === "error" || titleState.state === "error"
      ? "error"
      : contentState === "pending" || titleState.state === "pending"
        ? "pending"
        : "saved";

  return (
    <article className="editor-pane" aria-label={note.title || "Untitled note"}>
      <div className="editor-toolbar">
        <div className="toolbar-actions">
          {trashed ? (
            <button type="button" className="button" onClick={props.onRestore}>
              <RotateCcw size={16} aria-hidden="true" />
              Restore
            </button>
          ) : (
            <>
              <label className="folder-select">
                <span className="visually-hidden">Folder</span>
                <select
                  value={note.folderId ?? ""}
                  onChange={(e) => props.onPatch({ folderId: e.target.value || null })}
                >
                  <option value="">No folder</option>
                  {props.folders.map((f) => (
                    <option key={f.id} value={f.id}>
                      {f.name}
                    </option>
                  ))}
                </select>
              </label>
              <button
                type="button"
                className="icon-button"
                aria-pressed={note.inInbox}
                aria-label={note.inInbox ? "Remove from Inbox" : "Move to Inbox"}
                title={note.inInbox ? "Remove from Inbox" : "Move to Inbox"}
                onClick={() => props.onPatch({ inInbox: !note.inInbox })}
              >
                <Inbox size={16} aria-hidden="true" />
              </button>
              <button
                type="button"
                className="icon-button"
                aria-pressed={note.pinned}
                aria-label={note.pinned ? "Unpin" : "Pin"}
                title={note.pinned ? "Unpin" : "Pin"}
                onClick={() => props.onPatch({ pinned: !note.pinned })}
              >
                <Pin size={16} aria-hidden="true" />
              </button>
              {props.toolbarExtra}
              <button
                type="button"
                className="icon-button danger"
                aria-label="Move to Trash"
                title="Move to Trash"
                onClick={props.onDelete}
              >
                <Trash2 size={16} aria-hidden="true" />
              </button>
            </>
          )}
        </div>
      </div>

      {trashed && (
        <p className="trash-banner" role="note">
          This note is in the Trash. Restore it to edit.
        </p>
      )}

      <div className="editor-scroll">
        <div className="editor-sheet">
          <div className="title-row">
            {!trashed && <RegMark state={saveState} />}
            <textarea
              className="title-input"
              rows={1}
              value={titleState.title}
              placeholder="Untitled"
              aria-label="Title"
              readOnly={trashed}
              // biome-ignore lint/a11y/noAutofocus: a new note starts with its title
              autoFocus={props.autoFocusTitle}
              onChange={(e) => titleState.setTitle(e.target.value.replace(/\n/g, " "))}
              onBlur={() => void titleState.flush()}
              onKeyDown={(e) => {
                if (e.key === "Enter" || (e.key === "ArrowDown" && !e.shiftKey)) {
                  e.preventDefault();
                  editorRef.current?.focus();
                }
              }}
            />
          </div>
          {loadError ? (
            <p className="editor-error" role="alert">
              This note could not be opened: {loadError}
            </p>
          ) : sync ? (
            <BlockEditor sync={sync} editable={!trashed} editorRef={editorRef} />
          ) : null}
        </div>
      </div>
    </article>
  );
}

/** Title edits are saved after a short pause, on blur, and before the window closes. */
function useTitle(note: NoteMeta, onError: (message: string) => void) {
  const [title, setTitleValue] = useState(note.title);
  const [state, setState] = useState<SaveState>("saved");
  const pending = useRef<string | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flush = useCallback(async () => {
    if (timer.current) clearTimeout(timer.current);
    timer.current = null;
    const value = pending.current;
    if (value === null) return;
    pending.current = null;
    try {
      await updateNoteMeta(note.id, { title: value });
      if (pending.current === null) setState("saved");
    } catch (err) {
      pending.current ??= value;
      setState("error");
      onError(`Could not save the title: ${errorMessage(err)}`);
    }
  }, [note.id, onError]);

  useEffect(() => registerFlush(flush), [flush]);
  useEffect(() => () => void flush(), [flush]);

  const setTitle = (value: string) => {
    setTitleValue(value);
    pending.current = value;
    setState("pending");
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => void flush(), TITLE_SAVE_DELAY_MS);
  };

  return { title, setTitle, state, flush };
}

const LIGHT_THEME: Theme = {
  colors: {
    editor: { text: "var(--text)", background: "var(--stock)" },
    menu: { text: "var(--text)", background: "var(--stock-raised)" },
    tooltip: { text: "var(--stock)", background: "var(--text)" },
    hovered: { text: "var(--text)", background: "var(--stock-sunk)" },
    selected: { text: "var(--on-accent)", background: "var(--accent)" },
    disabled: { text: "var(--text-muted)", background: "var(--stock-sunk)" },
    shadow: "var(--shadow-color)",
    border: "var(--rule)",
    sideMenu: "var(--text-muted)",
  },
  borderRadius: 6,
  fontFamily: "var(--font-text)",
};

function BlockEditor(props: {
  sync: NoteSync;
  editable: boolean;
  editorRef: React.RefObject<BlockNoteEditor | null>;
}) {
  const editor = useCreateBlockNote(
    withCollaboration({
      collaboration: {
        fragment: props.sync.fragment,
        user: { name: "Me", color: "#4b55c4" },
      },
    }),
    [props.sync],
  );
  props.editorRef.current = editor;
  return (
    <BlockNoteView
      editor={editor}
      editable={props.editable}
      theme={{ light: LIGHT_THEME, dark: LIGHT_THEME }}
      className="block-editor"
    />
  );
}
