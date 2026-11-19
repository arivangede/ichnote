import { Inbox, PanelLeftClose, Pin, Plus } from "lucide-react";
import { type KeyboardEvent, useRef } from "react";
import { displayTitle, formatNoteDate } from "../lib/format";
import { type NoteMeta, type SearchHit, splitSnippet, type Uuid } from "../lib/ipc";

interface NoteListProps {
  heading: string;
  notes: NoteMeta[];
  hits: SearchHit[] | null;
  selectedId: Uuid | null;
  onSelect: (id: Uuid) => void;
  onCreate: (() => void) | null;
  emptyText: string;
  loading: boolean;
  onCollapse: () => void;
}

const MOD = navigator.platform.toLowerCase().includes("mac") ? "⌘" : "Ctrl+";

export function NoteList(props: NoteListProps) {
  const { notes, hits, selectedId, onSelect } = props;
  const listRef = useRef<HTMLUListElement>(null);
  const rows: { note: NoteMeta; snippet?: string }[] = hits
    ? hits.map((h) => ({ note: h.note, snippet: h.snippet }))
    : notes.map((note) => ({ note }));

  const onKeyDown = (e: KeyboardEvent<HTMLUListElement>) => {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const index = rows.findIndex((r) => r.note.id === selectedId);
    const next = rows[e.key === "ArrowDown" ? index + 1 : Math.max(index - 1, 0)];
    if (!next) return;
    onSelect(next.note.id);
    listRef.current?.querySelector<HTMLButtonElement>(`[data-note-id="${next.note.id}"]`)?.focus();
  };

  const count = rows.length;
  return (
    <section className="list-pane" aria-labelledby="list-heading">
      <div className="titlebar-strip list-strip" data-tauri-drag-region />
      <header className="list-header" data-tauri-drag-region>
        <div>
          <h1 id="list-heading" className="list-title">
            {props.heading}
          </h1>
          <p className="list-count">{count === 1 ? "1 note" : `${count} notes`}</p>
        </div>
        <div className="list-actions">
          {props.onCreate && (
            <button
              type="button"
              className="button"
              onClick={props.onCreate}
              aria-keyshortcuts="Meta+N"
              title={`New note (${MOD}N)`}
            >
              <Plus size={16} aria-hidden="true" />
              New note
            </button>
          )}
          <button
            type="button"
            className="icon-button"
            aria-label="Hide note list"
            title={`Hide note list (${MOD}⇧\\)`}
            onClick={props.onCollapse}
          >
            <PanelLeftClose size={16} aria-hidden="true" />
          </button>
        </div>
      </header>

      {count === 0 && !props.loading ? (
        <p className="list-empty">{props.emptyText}</p>
      ) : (
        <ul className="note-list" ref={listRef} onKeyDown={onKeyDown} aria-label={props.heading}>
          {rows.map(({ note, snippet }) => (
            <li key={note.id}>
              <button
                type="button"
                className="note-row"
                data-note-id={note.id}
                aria-current={note.id === selectedId ? "true" : undefined}
                onClick={() => onSelect(note.id)}
              >
                <span className="note-row-top">
                  <span
                    className={note.title.trim() ? "note-row-title" : "note-row-title untitled"}
                  >
                    {displayTitle(note.title)}
                  </span>
                  <time
                    className="note-row-date"
                    dateTime={new Date(note.deletedAt ?? note.updatedAt).toISOString()}
                  >
                    {formatNoteDate(note.deletedAt ?? note.updatedAt)}
                  </time>
                </span>
                {snippet && (
                  <span className="note-row-snippet">
                    {splitSnippet(snippet).map((part, i) =>
                      part.match ? (
                        // biome-ignore lint/suspicious/noArrayIndexKey: parts are positional
                        <mark key={i}>{part.text}</mark>
                      ) : (
                        // biome-ignore lint/suspicious/noArrayIndexKey: parts are positional
                        <span key={i}>{part.text}</span>
                      ),
                    )}
                  </span>
                )}
                {(note.pinned || note.inInbox) && (
                  <span className="note-row-flags">
                    {note.pinned && (
                      <span className="flag">
                        <Pin size={12} aria-hidden="true" /> Pinned
                      </span>
                    )}
                    {note.inInbox && (
                      <span className="flag">
                        <Inbox size={12} aria-hidden="true" /> Inbox
                      </span>
                    )}
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
