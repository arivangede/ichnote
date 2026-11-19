---
version: 1
slug: "src-main-mainwindow-tsx"
primary_target: "src/main/MainWindow.tsx"
related_targets: []
---

# Main window — surface brief

Scope: main window (`src/main/`), visitor mode: Operate. Task: write and find notes; sidebar (Inbox, All notes, Pinned, folders, Trash), search, note list, editor (title + BlockNote). Frequent, all-day use; keyboard-first; light/dark follows the OS; WCAG AA.

## Direction contract

FORM: Letterpress proof — seed:f9cdadb56c0a5167 (assigned).

THESIS: A note is ink pressed onto stock: the editor is a clean proof sheet, the shell is the ink. Refuses the grey-on-grey sidebar-plus-white-page default.

OWN-WORLD: Deep blue-black ink sidebar shell; off-white proof stock for list and editor (dark mode: ink stock, paper-white type); one indigo ink accent for selection and focus; proof red only for Trash/destructive. One sturdy grotesk (system UI stack) on a 4px grid, tabular numerals for dates.

STORY: The visitor sees where notes live (sidebar), which notes match (list), and the note itself (editor), and always knows whether their words are safely on disk.

FIRST VIEWPORT: Ink sidebar left (workspace wordmark, search, Inbox / All / Pinned / folders / Trash), note list middle (title, excerpt-free rows with date, pin), editor right with large title input and BlockNote content.

SIGNATURE: Registration mark beside the note title: a crop-mark ⊕ drawn dashed while an update is pending, inked solid once persisted, struck through on error. Raise from emission-line rail: state carried in line form (dash/solid/strike), not hue. Raise from Crouwel grid: every spacing on one 4px grid.

RISK: Ink shell may feel heavy in light mode; keep it to the sidebar only.
