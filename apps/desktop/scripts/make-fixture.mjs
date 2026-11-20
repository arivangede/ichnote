// Generates a BlockNote-produced Yjs note document used by the Rust tests in notetice-core.
// Run from apps/desktop: `node scripts/make-fixture.mjs`.
import { writeFileSync } from "node:fs";
import { BlockNoteEditor } from "@blocknote/core";
import { blocksToYDoc } from "@blocknote/core/yjs";
import * as Y from "yjs";

const editor = BlockNoteEditor.create();
const doc = blocksToYDoc(
  editor,
  [
    { type: "heading", props: { level: 1 }, content: "Grocery list" },
    {
      type: "paragraph",
      content: [
        { type: "text", text: "Buy ", styles: {} },
        { type: "text", text: "fresh", styles: { bold: true } },
        { type: "text", text: " café beans", styles: {} },
      ],
    },
    { type: "checkListItem", props: { checked: true }, content: "Milk" },
    {
      type: "bulletListItem",
      content: "Parent",
      children: [{ type: "bulletListItem", content: "Nested child" }],
    },
    { type: "codeBlock", props: { language: "rust" }, content: "fn main() {}" },
  ],
  "document-store",
);
const out = new URL(
  "../../../crates/notetice-core/tests/fixtures/blocknote-note.bin",
  import.meta.url,
);
writeFileSync(out, Y.encodeStateAsUpdate(doc));
console.log(`wrote ${out.pathname}`);
console.log(doc.getXmlFragment("document-store").toString());
