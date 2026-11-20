// Keeps one note's Y.Doc in step with Rust: loads the stored state, batches local edits,
// and persists them through `apply_local_update`. Remote updates (Phase 3) are applied with
// origin "remote" so they are never echoed back.
import * as Y from "yjs";
import { applyLocalUpdate, loadDoc, type NoteMeta, type Uuid } from "./ipc";

/** Name of the XmlFragment BlockNote binds to; must match NOTE_FRAGMENT in notetice-core. */
export const NOTE_FRAGMENT = "document-store";
export const REMOTE_ORIGIN = "remote";
const LOAD_ORIGIN = "load";
/** Edits are batched for this long before being persisted. */
export const FLUSH_DELAY_MS = 300;
const RETRY_DELAY_MS = 2000;

/** `saved`: everything is on disk. `pending`: edits are waiting or being written. */
export type SaveState = "saved" | "pending" | "error";

export interface NoteSyncDeps {
  apply: (docId: Uuid, update: Uint8Array) => Promise<NoteMeta>;
  load: (docId: Uuid) => Promise<Uint8Array>;
}

const defaultDeps: NoteSyncDeps = { apply: applyLocalUpdate, load: loadDoc };

export class NoteSync {
  readonly doc = new Y.Doc();
  private queue: Uint8Array[] = [];
  private timer: ReturnType<typeof setTimeout> | null = null;
  private inFlight: Promise<void> | null = null;
  private state: SaveState = "saved";
  private listeners = new Set<(state: SaveState) => void>();
  private destroyed = false;

  private constructor(
    readonly docId: Uuid,
    private readonly deps: NoteSyncDeps,
  ) {
    this.doc.on("update", this.onUpdate);
  }

  /** Loads the stored document. */
  static async open(docId: Uuid, deps: NoteSyncDeps = defaultDeps): Promise<NoteSync> {
    const sync = new NoteSync(docId, deps);
    const state = await deps.load(docId);
    if (state.length > 0) Y.applyUpdate(sync.doc, state, LOAD_ORIGIN);
    return sync;
  }

  get fragment(): Y.XmlFragment {
    return this.doc.getXmlFragment(NOTE_FRAGMENT);
  }

  get saveState(): SaveState {
    return this.state;
  }

  subscribe(listener: (state: SaveState) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /** Applies an update that came from another device. */
  applyRemote(update: Uint8Array): void {
    Y.applyUpdate(this.doc, update, REMOTE_ORIGIN);
  }

  private setState(state: SaveState): void {
    if (state === this.state) return;
    this.state = state;
    for (const l of this.listeners) l(state);
  }

  private onUpdate = (update: Uint8Array, origin: unknown): void => {
    if (origin === REMOTE_ORIGIN || origin === LOAD_ORIGIN) return;
    this.queue.push(update);
    this.setState("pending");
    this.schedule(FLUSH_DELAY_MS);
  };

  private schedule(delay: number): void {
    if (this.timer !== null) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.timer = null;
      void this.flush().catch(() => undefined);
    }, delay);
  }

  /**
   * Persists every queued edit. Resolves once they are on disk; rejects if writing failed
   * (the edits stay queued and are retried).
   */
  async flush(): Promise<void> {
    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    // Writes are strictly sequential so the store sees updates in order.
    while (this.inFlight) await this.inFlight.catch(() => undefined);
    if (this.queue.length === 0) return;
    const batch = this.queue;
    this.queue = [];
    const merged = batch.length === 1 ? (batch[0] as Uint8Array) : Y.mergeUpdates(batch);
    const write = this.deps.apply(this.docId, merged).then(
      () => {
        if (this.queue.length === 0) this.setState("saved");
      },
      (err: unknown) => {
        // Keep the edits; they are retried with whatever comes next.
        this.queue.unshift(merged);
        this.setState("error");
        if (!this.destroyed) this.schedule(RETRY_DELAY_MS);
        throw err;
      },
    );
    this.inFlight = write;
    try {
      await write;
    } finally {
      this.inFlight = null;
    }
    // Edits made while writing.
    if (this.queue.length > 0) await this.flush();
  }

  /** Flushes and releases the document. */
  async close(): Promise<void> {
    try {
      await this.flush();
    } finally {
      this.destroyed = true;
      if (this.timer !== null) clearTimeout(this.timer);
      this.doc.off("update", this.onUpdate);
      this.doc.destroy();
      this.listeners.clear();
    }
  }
}
