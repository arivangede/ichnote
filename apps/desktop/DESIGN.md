---
name: note-tice!
description: Local-first block notes. Leave a trace.
colors:
  ink: "oklch(0.235 0.035 268)"
  ink-hover: "oklch(0.3 0.04 268)"
  ink-current: "oklch(0.34 0.03 268)"
  on-ink: "oklch(0.96 0.01 95)"
  on-ink-muted: "oklch(0.78 0.025 268)"
  ink-rule: "oklch(0.34 0.03 268)"
  stock: "oklch(0.985 0.006 95)"
  stock-raised: "oklch(1 0 0)"
  stock-list: "oklch(0.965 0.008 95)"
  stock-sunk: "oklch(0.93 0.01 95)"
  row-hover: "oklch(0.935 0.01 95)"
  rule: "oklch(0.87 0.012 95)"
  text: "oklch(0.25 0.025 268)"
  text-muted: "oklch(0.47 0.02 268)"
  accent: "oklch(0.5 0.12 65)"
  accent-soft: "oklch(0.93 0.07 85)"
  on-accent: "oklch(0.985 0.006 95)"
  danger: "oklch(0.5 0.18 27)"
typography:
  headline:
    fontFamily: "system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "32px"
    fontWeight: 650
    lineHeight: "40px"
    letterSpacing: "-0.02em"
  title:
    fontFamily: "system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "17px"
    fontWeight: 600
    lineHeight: "24px"
    letterSpacing: "-0.01em"
  wordmark:
    fontFamily: "system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "17px"
    fontWeight: 500
    lineHeight: "24px"
    letterSpacing: "0.22em"
  body:
    fontFamily: "system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: "20px"
  body-small:
    fontFamily: "system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: "20px"
  label:
    fontFamily: "system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: "16px"
    fontFeature: "tnum"
rounded:
  hairline: "2px"
  key: "4px"
  base: "6px"
spacing:
  "1": "4px"
  "2": "8px"
  "3": "12px"
  "4": "16px"
  "5": "20px"
  "6": "24px"
  "8": "32px"
  "12": "48px"
components:
  nav-item:
    backgroundColor: "transparent"
    textColor: "{colors.on-ink-muted}"
    rounded: "{rounded.base}"
    padding: "4px 8px"
    height: "32px"
  nav-item-hover:
    backgroundColor: "{colors.ink-hover}"
    textColor: "{colors.on-ink}"
  nav-item-current:
    backgroundColor: "{colors.ink-current}"
    textColor: "{colors.on-ink}"
  search-field:
    backgroundColor: "{colors.ink-hover}"
    textColor: "{colors.on-ink}"
    rounded: "{rounded.base}"
    padding: "0 8px"
    height: "32px"
  button:
    backgroundColor: "{colors.stock-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.base}"
    padding: "0 12px"
    height: "32px"
  icon-button:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.base}"
    size: "28px"
  icon-button-hover:
    backgroundColor: "{colors.stock-sunk}"
    textColor: "{colors.text}"
  icon-button-pressed:
    backgroundColor: "{colors.accent-soft}"
    textColor: "{colors.accent}"
  note-row:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.base}"
    padding: "12px"
  note-row-hover:
    backgroundColor: "{colors.row-hover}"
  note-row-current:
    backgroundColor: "{colors.accent-soft}"
    textColor: "{colors.accent}"
  select:
    backgroundColor: "{colors.stock-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.base}"
    padding: "0 8px"
    height: "28px"
  toast:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.on-ink}"
    rounded: "{rounded.base}"
    padding: "8px 16px"
  title-input:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    typography: "{typography.headline}"
    padding: "0 54px"
---

# Design System: note-tice!

## Overview

**Creative North Star: "Letterpress Proof"**

A note is ink pressed onto stock. The editor is a clean proof sheet; the shell is the ink. The sidebar is a single deep blue-black ink slab carrying the wordmark, search, and navigation, while the note list and editor sit on warm off-white proof stock. In dark mode the relationship inverts in value but not in role: the stock becomes ink-toned and the type becomes paper-white, and the sidebar drops to a near-black ink so it still reads as the darker, denser layer.

The window is dense and quiet, built for all-day, keyboard-first use. Colour never decorates; it does one of three jobs: shell (ink), selection and focus (one amber accent), or danger (proof red). Everything else is a value step within ink or stock. One sturdy system grotesk carries every role, and every spacing value sits on a 4px grid.

The signature is the registration mark beside the note title: a printer's crop mark that reports whether the note is on disk. Save state is carried in line form (dashed, solid, struck through), not in hue. The world refuses the grey-on-grey sidebar-plus-white-page default.

**Key Characteristics:**
- Three-column grid: ink sidebar (232px), proof-stock list (260 to 320px), editor sheet (fluid, content capped at 760px).
- Ink appears only on the sidebar and the transient toast; stock carries all content.
- One accent hue (amber, hue 65–78) for selection, pressed toggles, focus, caret, and text selection.
- Proof red (hue 27) only for destructive hover and save errors.
- Flat surfaces separated by 1px rules and value steps; one shadow, on the toast.
- Everything on a 4px grid; tabular numerals for dates and counts.

## Colors

Ink on paper: a cool blue-black ink (hue 268) against warm cream stock (hue 95), with one amber ink and one proof red that only appear when they mean something. Dark-mode values are defined in the `prefers-color-scheme: dark` block of `src/styles.css` and are recorded in the sidecar.

### Primary
- **Press Amber** (accent): the current note's title in the list, pressed toggle icons (Pin, Inbox), the 2px focus ring, the text caret, and the BlockNote selected state. Lightens to a bright amber in dark mode so it stays AA on ink stock.
- **Amber Wash** (accent-soft): background of the current note row, pressed icon buttons, text selection, and search-hit highlights in snippets.
- **On Accent** (on-accent): text on solid accent fills (BlockNote selected menu items).

### Secondary
- **Proof Red** (danger): destructive icon hover (Move to Trash), the save-error strike on the registration mark, and inline editor errors. Nothing else.

### Neutral
- **Shell Ink** (ink): the sidebar ground and the toast. The densest surface in both themes.
- **Ink Hover / Ink Current** (ink-hover, ink-current): sidebar hover and the current navigation item; the current item is a lighter ink step, not a second accent.
- **On Ink / On Ink Muted** (on-ink, on-ink-muted): sidebar text; muted for resting items, placeholders, and section labels, full for hover and current.
- **Ink Rule** (ink-rule): borders and dividers inside the sidebar.
- **Proof Stock** (stock): the editor sheet and app ground.
- **List Stock** (stock-list): the note-list pane, one step darker than the sheet in light mode and one step deeper in dark mode, so the sheet is always the lightest working surface.
- **Raised Stock** (stock-raised): secondary buttons, selects, and BlockNote menus.
- **Sunk Stock** (stock-sunk): icon-button hover and the Trash banner.
- **Row Hover** (row-hover): note-row hover.
- **Rule** (rule): 1px dividers between panes, rows, and the toolbar; also the stock-side scrollbar thumb.
- **Type Ink / Muted Type** (text, text-muted): body text and secondary text (dates, counts, snippets, untitled placeholders).

### Named Rules
**The Three Jobs Rule.** Colour only signals shell, selection, or danger. A new hue needs a new job, and there are no new jobs.

**The Ink Stays In The Shell Rule.** The ink surface is reserved for the sidebar and the transient toast. Content panes are always stock.

## Typography

**Display Font:** none; the title is set in the UI face.
**Body Font:** system-ui (with -apple-system, Segoe UI, Roboto, sans-serif)

**Character:** One sturdy grotesk, the platform's own, set with tight negative tracking on large sizes and generous tracking only on the lowercase wordmark. Hierarchy comes from size and weight steps, never from a second family.

### Hierarchy
- **Headline** (650, 32px, 40px): the note title input at the top of the proof sheet.
- **Title** (600, 17px, 24px, -0.01em): the note-list pane heading (All notes, Trash, folder name).
- **Wordmark** (500, 17px, 24px, 0.22em): the lowercase "note-tice!" wordmark atop the sidebar only.
- **Body** (400, 14px, 20px): base UI text: navigation, rows (titles at 500), buttons (500), search.
- **Body Small** (400, 13px): note-row snippets (two-line clamp) and empty-folder text.
- **Label** (12px, 16px, tabular numerals): note counts, row dates, row flags, and the "Folders" section heading (600). Sentence case; never uppercase.

### Named Rules
**The Tabular Time Rule.** Dates and counts always use tabular numerals so columns of times do not jitter.

**The One Face Rule.** Every role uses the system UI stack; differentiate by size and weight only.

## Layout

A fixed three-column grid fills the window: sidebar 232px, note list `minmax(260px, 320px)`, editor `minmax(0, 1fr)`. Below 900px the first two columns narrow to 200px and 240px. Each column scrolls independently; the body never scrolls.

All spacing comes from one 4px grid (4, 8, 12, 16, 20, 24, 32, 48). The sidebar pads 20px top and 12px sides; the list header pads 20px / 16px; rows pad 12px. The editor toolbar is a 48px bar; the proof sheet is centred at a maximum of 760px with 48px top padding and 30vh of trailing space so the last line can be written mid-screen. The title is inset 54px to align with BlockNote's block text inside its side-menu gutter. Control heights step 24 / 28 / 32px.

**The Crouwel Grid Rule.** Every margin, gap, and padding is a multiple of 4px. The 54px title inset is the one exception, and it exists to align with the editor library's gutter.

## Elevation & Depth

Flat. Depth is carried by value steps between ink, list stock, and sheet stock, and by 1px rules between panes and rows. The only shadow in the app is under the toast, which floats over content for a moment; BlockNote menus receive the same shadow colour through the editor theme.

### Shadow Vocabulary
- **Toast lift** (`box-shadow: 0 4px 16px var(--shadow-color)`): the transient status toast only.

**The Flat Stock Rule.** Panes, rows, and buttons never cast shadows. If something needs to stand apart, step its stock value or rule it off.

## Shapes

One gentle radius (6px) on every interactive shape: nav items, search, buttons, icon buttons, selects, hovered and current note rows, the toast, and BlockNote menus. Keyboard hints use 4px; search-hit marks use 2px. Note rows are square-edged list entries divided by 1px rules at rest and only round off when hovered or selected, where the rule beneath them disappears. Icons are Lucide line icons at 16px (12px in row flags, 14px for small actions), matching the line weight of the rules.

## Components

### Buttons
Quiet and functional; nothing shouts.
- **Shape:** gently rounded (6px), 32px tall, 12px horizontal padding, 8px icon gap, 500 weight.
- **Default:** raised stock fill with a 1px rule border (New note, Restore).
- **Hover:** the border darkens to muted type; no fill change, no lift.
- **Icon button:** 28px square, transparent, muted icon; hover fills with sunk stock. As a toggle (`aria-pressed`) it takes the Amber Wash with an accent icon. The destructive variant turns its icon Proof Red on hover. On ink it shrinks to 24px and hovers to ink-hover.
- **Focus:** every control shows a 2px accent outline at 2px offset; inside the sidebar the outline switches to on-ink.

### Inputs / Fields
- **Search (on ink):** 32px, ink-hover fill, 1px ink-rule border, 6px radius, leading search icon and a trailing shortcut hint in label size. Focus brightens the border to on-ink-muted.
- **Folder name (on ink):** same geometry with an on-ink-muted border.
- **Folder select (on stock):** 28px, raised stock, 1px rule border, 6px radius.
- **Title:** borderless, transparent, Headline type, grows with its content; placeholder in muted type.

### Navigation
- **Sidebar items:** 32px minimum height, 16px icon plus label, 12px gap, muted on-ink at rest. Hover fills ink-hover; the current item fills ink-current, turns full on-ink, and goes to 500 weight.
- **Sections:** "Folders" heading in Label at 600 with an inline add-folder icon; Trash and Export sit below a 1px ink-rule divider at the bottom.

### Note List Rows
- **Structure:** title (500, ellipsised) and tabular date on one baseline; optional two-line snippet; optional flags (Pinned, Inbox) as 12px icon plus label.
- **States:** hover fills row-hover and rounds; the current row fills Amber Wash, rounds, and its title turns Press Amber. Untitled notes show a muted, regular-weight placeholder.

### Toast
Ink chip at the bottom centre, 8px / 16px padding, 6px radius, the toast-lift shadow, rising 8px into place over 260ms on the out-expo curve.

### Registration Mark (signature)
A 20px crop mark (ring, crosshair, centre dot) set in the title's left gutter, with a visually hidden live status. **Saved:** solid ring and dot in full type colour. **Pending:** ring and crosshair dashed, dot scaled to zero, muted colour. **Error:** Proof Red with a diagonal strike. Transitions run 420ms on `cubic-bezier(0.16, 1, 0.3, 1)`; reduced motion zeroes all durations.

**The Line-Not-Hue Rule.** Persistence state is expressed by the line's form (dashed, solid, struck). Colour only joins on error, and the strike still carries the meaning without it.

## Do's and Don'ts

### Do:
- **Do** take every surface colour from the ink or stock families and every meaning from accent or danger, in both themes.
- **Do** keep every spacing value on the 4px grid (4 through 48).
- **Do** use the 6px radius for every interactive shape.
- **Do** show focus with the 2px accent outline at 2px offset (on-ink inside the sidebar).
- **Do** set dates and counts in tabular numerals.
- **Do** report save state through the registration mark's line form.
- **Do** route BlockNote's theme through the same custom properties so the editor follows light and dark with the shell.

### Don't:
- **Don't** use ink as a ground for content panes; it belongs to the sidebar and the toast.
- **Don't** use Proof Red for anything except destructive actions and failures.
- **Don't** add shadows to panes, rows, or buttons; the toast is the only lifted element.
- **Don't** introduce a second typeface or uppercase labels.
- **Don't** signal saved or pending state with colour alone.
