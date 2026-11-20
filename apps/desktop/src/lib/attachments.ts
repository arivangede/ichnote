// Attachments in notes. Note content stores a stable, platform-independent URL
// (`notetice-attachment:<id>`); for display it is mapped to the `ntc` URI scheme served by Rust.
import { convertFileSrc } from "@tauri-apps/api/core";
import type { Attachment, Uuid } from "./ipc";

export const ATTACHMENT_URL_PREFIX = "notetice-attachment:";
/** Must match `SCHEME` in src-tauri/src/attachments.rs. */
export const ATTACHMENT_PROTOCOL = "ntc";

const ID_PATTERN = "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";

export function attachmentUrl(id: Uuid): string {
  return `${ATTACHMENT_URL_PREFIX}${id}`;
}

export function attachmentId(url: string): Uuid | null {
  const m = new RegExp(`^${ATTACHMENT_URL_PREFIX}(${ID_PATTERN})$`).exec(url);
  return m?.[1] ?? null;
}

/** Maps a stored URL to one the webview can load; other URLs pass through. */
export function resolveFileUrl(
  url: string,
  convert: (path: string, protocol: string) => string = convertFileSrc,
): Promise<string> {
  const id = attachmentId(url);
  return Promise.resolve(id ? convert(id, ATTACHMENT_PROTOCOL) : url);
}

/** Every attachment ID referenced in a Markdown (or any) text, in order, without duplicates. */
export function referencedAttachments(text: string): Uuid[] {
  const re = new RegExp(`${ATTACHMENT_URL_PREFIX}(${ID_PATTERN})`, "g");
  return [...new Set([...text.matchAll(re)].map((m) => m[1] as string))];
}

/** Export file name of an attachment: unique by ID, readable by name. */
export function attachmentExportName(a: Pick<Attachment, "id" | "name">): string {
  const safe = a.name.replace(/[/\\:*?"<>|]/g, "-").trim() || "file";
  return `${a.id.slice(-8)}-${safe}`;
}

/** Replaces attachment URLs in Markdown with relative links to exported copies. */
export function rewriteAttachmentLinks(
  markdown: string,
  names: Map<Uuid, string>,
  prefix: string,
): string {
  const re = new RegExp(`${ATTACHMENT_URL_PREFIX}(${ID_PATTERN})`, "g");
  return markdown.replace(re, (whole, id: string) => {
    const name = names.get(id);
    return name ? encodeURI(`${prefix}${name}`) : whole;
  });
}
