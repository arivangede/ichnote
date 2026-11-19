import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import * as Y from "yjs";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

import type { NoteMeta } from "./ipc";
import { FLUSH_DELAY_MS, NOTE_FRAGMENT, NoteSync, type SaveState } from "./noteSync";

function storedDoc(text: string): Uint8Array {
  const doc = new Y.Doc();
  doc.getXmlFragment(NOTE_FRAGMENT).insert(0, [new Y.XmlText(text)]);
  return Y.encodeStateAsUpdate(doc);
}

function textOf(updates: Uint8Array[], base: Uint8Array): string {
  const doc = new Y.Doc();
  Y.applyUpdate(doc, base);
  for (const u of updates) Y.applyUpdate(doc, u);
  return doc.getXmlFragment(NOTE_FRAGMENT).toString();
}

function type(sync: NoteSync, text: string): void {
  const frag = sync.fragment;
  frag.insert(frag.length, [new Y.XmlText(text)]);
}

describe("NoteSync", () => {
  const base = storedDoc("Hello");
  let sent: Uint8Array[];
  let apply: ReturnType<typeof vi.fn<(id: string, u: Uint8Array) => Promise<NoteMeta>>>;

  beforeEach(() => {
    vi.useFakeTimers();
    sent = [];
    apply = vi.fn<(id: string, u: Uint8Array) => Promise<NoteMeta>>(async (_id, u) => {
      sent.push(u);
      return {} as NoteMeta;
    });
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  const open = () => NoteSync.open("doc", { apply, load: async () => base });

  it("loads stored state without sending it back", async () => {
    const sync = await open();
    expect(sync.fragment.toString()).toBe("Hello");
    await vi.advanceTimersByTimeAsync(FLUSH_DELAY_MS * 2);
    expect(apply).not.toHaveBeenCalled();
    expect(sync.saveState).toBe("saved");
  });

  it("batches edits into one merged update after the delay", async () => {
    const sync = await open();
    const states: SaveState[] = [];
    sync.subscribe((s) => states.push(s));
    type(sync, " a");
    type(sync, " b");
    expect(sync.saveState).toBe("pending");
    await vi.advanceTimersByTimeAsync(FLUSH_DELAY_MS);
    expect(apply).toHaveBeenCalledTimes(1);
    expect(textOf(sent, base)).toBe("Hello a b");
    expect(states).toEqual(["pending", "saved"]);
  });

  it("does not echo remote updates", async () => {
    const sync = await open();
    const other = new Y.Doc();
    Y.applyUpdate(other, base);
    const before = Y.encodeStateVector(other);
    other.getXmlFragment(NOTE_FRAGMENT).insert(1, [new Y.XmlText("!")]);
    sync.applyRemote(Y.encodeStateAsUpdate(other, before));
    await vi.advanceTimersByTimeAsync(FLUSH_DELAY_MS * 2);
    expect(apply).not.toHaveBeenCalled();
  });

  it("close flushes pending edits immediately", async () => {
    const sync = await open();
    type(sync, " bye");
    await sync.close();
    expect(textOf(sent, base)).toBe("Hello bye");
  });

  it("keeps failed edits and retries them", async () => {
    const sync = await open();
    apply.mockRejectedValueOnce(new Error("disk full"));
    type(sync, " x");
    await vi.advanceTimersByTimeAsync(FLUSH_DELAY_MS);
    expect(sync.saveState).toBe("error");
    expect(sent).toHaveLength(0);
    await vi.advanceTimersByTimeAsync(2000);
    expect(sync.saveState).toBe("saved");
    expect(textOf(sent, base)).toBe("Hello x");
  });

  it("sends edits made during a write in a following write, in order", async () => {
    let release: () => void = () => undefined;
    apply.mockImplementationOnce(
      (_id: string, u: Uint8Array) =>
        new Promise<NoteMeta>((resolve) => {
          sent.push(u);
          release = () => resolve({} as NoteMeta);
        }),
    );
    const sync = await open();
    type(sync, " 1");
    const first = sync.flush();
    type(sync, " 2");
    const second = sync.flush();
    release();
    await Promise.all([first, second]);
    expect(apply).toHaveBeenCalledTimes(2);
    expect(textOf(sent, base)).toBe("Hello 1 2");
    expect(sync.saveState).toBe("saved");
  });
});
