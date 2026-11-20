import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), convertFileSrc: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

import {
  attachmentExportName,
  attachmentId,
  attachmentUrl,
  referencedAttachments,
  resolveFileUrl,
  rewriteAttachmentLinks,
} from "./attachments";

const A = "01a1f183-155a-742b-8142-8bf1bb30173e";
const B = "01a1f183-200c-7400-894d-e6ce540b0e91";

describe("attachments", () => {
  it("round-trips stored URLs", () => {
    expect(attachmentId(attachmentUrl(A))).toBe(A);
    expect(attachmentId("https://example.com/a.png")).toBeNull();
    expect(attachmentId(`notetice-attachment:${A}x`)).toBeNull();
  });

  it("resolves only attachment URLs to the ntc scheme", async () => {
    const convert = vi.fn((p: string, proto: string) => `${proto}://localhost/${p}`);
    await expect(resolveFileUrl(attachmentUrl(A), convert)).resolves.toBe(`ntc://localhost/${A}`);
    await expect(resolveFileUrl("https://x/y.png", convert)).resolves.toBe("https://x/y.png");
  });

  it("finds and rewrites references for export", () => {
    const md = `![cat](notetice-attachment:${A})\n[doc](notetice-attachment:${B}) ![again](notetice-attachment:${A})`;
    expect(referencedAttachments(md)).toEqual([A, B]);
    const names = new Map([[A, attachmentExportName({ id: A, name: "my cat.png" })]]);
    expect(rewriteAttachmentLinks(md, names, "../attachments/")).toBe(
      `![cat](../attachments/bb30173e-my%20cat.png)\n[doc](notetice-attachment:${B}) ![again](../attachments/bb30173e-my%20cat.png)`,
    );
    expect(attachmentExportName({ id: A, name: "a/b:c.pdf" })).toBe("bb30173e-a-b-c.pdf");
  });
});
