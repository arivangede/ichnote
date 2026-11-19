// Markdown export. Conversion uses BlockNote's own (lossy) Markdown serializer so the schema
// knowledge stays in BlockNote; Rust only writes the files.
import { BlockNoteEditor } from "@blocknote/core";
import { yXmlFragmentToBlocks } from "@blocknote/core/yjs";
import * as Y from "yjs";
import type { ExportFile, Folder, NoteMeta } from "./ipc";
import { NOTE_FRAGMENT } from "./noteSync";

let headless: BlockNoteEditor | null = null;

function editor(): BlockNoteEditor {
  headless ??= BlockNoteEditor.create();
  return headless;
}

/** Converts a stored note document (Yjs v1 state) to Markdown, with the title as `# heading`. */
export function noteToMarkdown(title: string, state: Uint8Array): string {
  const doc = new Y.Doc();
  try {
    if (state.length > 0) Y.applyUpdate(doc, state);
    const ed = editor();
    const blocks = yXmlFragmentToBlocks(ed, doc.getXmlFragment(NOTE_FRAGMENT));
    const body = ed.blocksToMarkdownLossy(blocks).trim();
    const heading = title.trim() ? `# ${title.trim()}\n\n` : "";
    return `${heading}${body}\n`;
  } finally {
    doc.destroy();
  }
}

/** File name for a note (sanitized again in Rust). */
export function markdownFileName(title: string): string {
  const base = title.trim() || "Untitled";
  return `${base.replace(/[/\\]/g, "-")}.md`;
}

/** Splits an absolute path chosen in a save dialog into its directory and file name. */
export function splitPath(path: string): { dir: string; name: string } {
  const i = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return { dir: path.slice(0, i) || "/", name: path.slice(i + 1) };
}

/** Builds the files of a workspace export: one `.md` per live note, inside its folder. */
export function workspaceExportFiles(
  notes: NoteMeta[],
  folders: Folder[],
  states: Map<string, Uint8Array>,
  rootName: string,
): ExportFile[] {
  const folderNames = new Map(folders.map((f) => [f.id, f.name.replace(/[/\\]/g, "-")]));
  return notes
    .filter((n) => n.deletedAt === null)
    .map((n) => {
      const folder = n.folderId ? folderNames.get(n.folderId) : undefined;
      const name = markdownFileName(n.title);
      return {
        path: [rootName, folder, name].filter(Boolean).join("/"),
        contents: noteToMarkdown(n.title, states.get(n.id) ?? new Uint8Array()),
      };
    });
}
