# note-tice!

*take note.*

note-tice! is a lightweight, local-first, open-source note-taking app with a block editor,
an always-on-top quick window, and serverless sync through a folder you already own
(iCloud Drive, Dropbox, Google Drive for desktop, OneDrive, Syncthing).

> **Status: pre-alpha.** Phase 0 (foundation) is in place; nothing is usable yet.
> See the roadmap in [`docs/DESIGN.md`](docs/DESIGN.md) (section 9).

## Highlights (planned for v1)

- **Local-first** — all data lives on your device in SQLite; works fully offline.
- **Lightweight** — built with Rust and Tauri v2 (OS WebView, not Electron).
- **Quick window** — global shortcut opens a small window with pinned notes and quick capture.
- **Sync without a server** — each device writes only to its own folder inside a shared sync folder.
- **Conflict-free** — notes and metadata are CRDTs (Yjs / `yrs`), so offline edits merge automatically.
- **Open format** — the sync file format is documented so you can always take your data elsewhere.

## Repository layout

```
crates/notetice-core   data model, SQLite storage, CRDT documents (no Tauri)
crates/notetice-sync   sync engine and transports (no Tauri)
apps/desktop          Tauri v2 app: src-tauri (Rust) + src (React + TypeScript)
docs/                 design document, ADRs, sync format spec
```

## Building from source

Prerequisites (macOS): [Rust](https://rustup.rs) (stable), Node.js 22+, pnpm,
Xcode Command Line Tools. See the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
pnpm install
pnpm tauri dev          # run the desktop app in development
pnpm tauri build        # build a release .app / .dmg
```

Checks run in CI:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm lint && pnpm typecheck && pnpm test
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Contributions are accepted under the
[Developer Certificate of Origin](https://developercertificate.org/) (sign off your commits).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
