# CLAUDE.md — Ichnote

Ichnote is a lightweight, local-first, open-source note-taking app (Notion-like) built with Rust + Tauri v2.
The full design is in `docs/DESIGN.md` (written in Indonesian). **Read it before starting any task.**
The current phase and progress are tracked in the Roadmap section (section 9) of that document.

## Workflow

- Work **one phase at a time**, and within a phase, in small steps that compile and pass tests.
- Before writing code for a new phase, propose a short plan and list any ambiguities or open decisions.
- After each step: run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and frontend lint/typecheck.
- Make small, focused git commits with clear messages (Conventional Commits: `feat:`, `fix:`, `chore:`...).
- Tick roadmap checkboxes in `docs/DESIGN.md` when a task is truly done.
- When the design doc is wrong, outdated, or incomplete, say so and propose a change to the doc rather than silently diverging.

## Verify APIs, don't trust snippets

Code snippets in `docs/DESIGN.md` are illustrative. Tauri v2, its plugins, `yrs`, and BlockNote change between versions.
Always check the actual API of the installed version (docs.rs, official docs, or the crate source) before using it.
Use the latest stable versions when adding dependencies and record them in lockfiles.

## Architecture rules (do not break)

- `crates/ichnote-core` and `crates/ichnote-sync` must **not** depend on Tauri. They must be testable on their own.
- All IDs are **UUIDv7**. Never use auto-increment IDs for synced data (local-only tables like `outbox` are the exception).
- Never hard-delete synced data. Use tombstones (`deleted_at`) in the workspace meta CRDT document.
- All changes to notes and metadata go through CRDT documents (Yjs in the frontend, `yrs` in Rust). SQLite tables like `notes_index` and `notes_fts` are derived caches that can be rebuilt.
- In folder sync, a device writes **only** inside its own `devices/<device_id>/` folder. Never write to or delete from another device's folder.
- File writes for sync are atomic: write `*.tmp` → fsync → rename.
- The sync file format (`.ichn`) is a public contract. Any change requires bumping the format version and updating `docs/sync-format.md`.

## Development conveniences

- Support an environment variable `ICHNOTE_DATA_DIR` to override the app data directory. In dev builds, allow running two instances side by side with different data dirs (single-instance check disabled in dev) so folder sync can be tested on one machine.
- Prefer integration tests that simulate multiple devices with separate databases and a shared temp folder.

## Conventions

- Code, identifiers, comments, commit messages, and `README.md`: **English** (international open-source audience).
- Design docs may stay in Indonesian.
- Rust: `thiserror` in library crates, `anyhow` in the app layer, `tracing` for logs. No `unwrap()` outside tests.
- Frontend: React + TypeScript (strict), Vite.

## Commands (update as the project evolves)

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
pnpm install
pnpm tauri dev          # run the desktop app
```

## Prerequisites (macOS)

Rust (rustup, stable), Node.js LTS + pnpm, Xcode Command Line Tools. See the Tauri v2 prerequisites guide.
