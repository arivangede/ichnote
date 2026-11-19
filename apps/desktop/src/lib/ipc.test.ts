import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

import {
  applyLocalUpdate,
  createNote,
  DOC_ID_HEADER,
  getAppInfo,
  loadDoc,
  splitSnippet,
  updateNoteMeta,
} from "./ipc";

describe("ipc", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("getAppInfo calls app_info", async () => {
    invoke.mockResolvedValue({ version: "0.1.0", dataDir: "/d", deviceId: "x" });
    await expect(getAppInfo()).resolves.toMatchObject({ version: "0.1.0" });
    expect(invoke).toHaveBeenCalledWith("app_info");
  });

  it("createNote sends explicit nulls and defaults", async () => {
    invoke.mockResolvedValue({});
    await createNote("ws");
    expect(invoke).toHaveBeenCalledWith("create_note", {
      workspaceId: "ws",
      folderId: null,
      inInbox: false,
    });
  });

  it("loadDoc returns bytes from an ArrayBuffer or a number array", async () => {
    invoke.mockResolvedValueOnce(new Uint8Array([1, 2]).buffer);
    expect(Array.from(await loadDoc("d"))).toEqual([1, 2]);
    invoke.mockResolvedValueOnce([3]);
    expect(Array.from(await loadDoc("d"))).toEqual([3]);
    expect(invoke).toHaveBeenCalledWith("load_doc", { docId: "d" });
  });

  it("applyLocalUpdate sends a raw body with the doc id header", async () => {
    invoke.mockResolvedValue({});
    const update = new Uint8Array([9, 9]);
    await applyLocalUpdate("doc", update);
    expect(invoke).toHaveBeenCalledWith("apply_local_update", update, {
      headers: { [DOC_ID_HEADER]: "doc" },
    });
  });

  it("updateNoteMeta keeps null folderId (move out of folder)", async () => {
    invoke.mockResolvedValue({});
    await updateNoteMeta("n", { folderId: null });
    expect(invoke).toHaveBeenCalledWith("update_note_meta", {
      noteId: "n",
      patch: { folderId: null },
    });
  });

  it("splitSnippet separates highlighted terms", () => {
    expect(splitSnippet("Buy \u0002café\u0003 beans")).toEqual([
      { text: "Buy ", match: false },
      { text: "café", match: true },
      { text: " beans", match: false },
    ]);
    expect(splitSnippet("plain")).toEqual([{ text: "plain", match: false }]);
    expect(splitSnippet("\u0002open")).toEqual([{ text: "open", match: true }]);
  });
});
