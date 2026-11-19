import type { SaveState } from "../lib/noteSync";

const LABEL: Record<SaveState, string> = {
  saved: "Saved",
  pending: "Saving…",
  error: "Not saved — retrying",
};

/**
 * A printer's registration mark that shows whether the note is on disk: dashed while edits are
 * waiting, inked solid once written, struck through when writing failed.
 */
export function RegMark({ state }: { state: SaveState }) {
  return (
    <span className="regmark" data-state={state} title={LABEL[state]}>
      <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true" focusable="false">
        <circle className="regmark-ring" cx="12" cy="12" r="6.5" pathLength="100" />
        <path className="regmark-cross" d="M12 1.5v21M1.5 12h21" />
        <circle className="regmark-dot" cx="12" cy="12" r="2.25" />
        <path className="regmark-strike" d="M4 20 20 4" />
      </svg>
      <span className="visually-hidden" role="status" aria-live="polite">
        {LABEL[state]}
      </span>
    </span>
  );
}
