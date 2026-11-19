# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

(Tauri v2 desktop app rendering in the OS WebView. macOS first, Windows later; the UI has its own cross-platform look rather than imitating either OS. Only keyboard shortcuts differ per OS.)

## Users

General note-takers moving off tools like Notion or Apple Notes: people who write lists, meeting notes, ideas, and checklists through the day and want them to stay on their own device. They are not assumed to be technical. Secondary: people sharing a notebook with a small team through a folder they already sync (Google Drive, Dropbox, iCloud, Syncthing).

## Product Purpose

A lightweight, local-first block note app. Notes live in a local database, work fully offline, and sync between devices and teammates through a shared folder, with no server or account. Success: someone uses it every day on one device without ever losing a note after a restart or crash, then adds devices without conflicts.

## Positioning

Notion-like block editing with local-first ownership and serverless folder sync built on CRDTs: every device writes only its own files, so edits from many devices merge without "conflicted copy" files. Open format, open source (MIT OR Apache-2.0).

## Operating Context

- Main window: sidebar (Inbox, All notes, Pinned, folders, Trash, search), note list, editor with a separate title above block content (headings, paragraphs, checklists, lists, code, quotes).
- Quick window (Phase 2): always-on-top window from a global shortcut for pinned notes and quick capture into the Inbox.
- Markdown export per note and per workspace.

## Capabilities and Constraints

- One workspace in the UI for now (default "Personal"); folders are one level deep in the UI.
- Delete moves notes to Trash (restorable); there is no permanent delete (tombstones are synced).
- Sync status UI arrives in Phase 3; do not show sync controls before then.

## Brand Commitments

- Name: Ichnote, tagline "leave a trace." Lowercase wordmark "ichnote".
- Logo: `assets/brand/ichnote-logo.png`, an ink-black serif "I" split by a flowing trace line, on a warm off-white paper ground. UI color derives from this ink-on-paper pairing.

## Evidence on Hand

Logo and app icon in `assets/brand/`. No users, testimonials, or metrics yet; do not invent any.

## Product Principles

1. The user's data is theirs: open formats, always exportable, never lost.
2. Writing comes first: the editor is the product; chrome stays quiet.
3. Fast and light: instant open, instant capture, small footprint.
4. Honest state: what is saved, deleted, or synced is always clear.

## Accessibility & Inclusion

- Theme follows the OS light/dark setting.
- Every action is reachable from the keyboard, with visible focus.
- Text and controls meet WCAG 2.2 AA contrast in both themes.
