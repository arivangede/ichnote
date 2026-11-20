# Contributing to note-tice!

Thanks for your interest! note-tice! is at an early stage, so please open an issue to discuss
larger changes before starting work.

## Before you start

Read [`docs/DESIGN.md`](docs/DESIGN.md) (written in Indonesian) — it describes the architecture,
data model, sync format, and roadmap. Architecture decisions are recorded in [`docs/adr/`](docs/adr/).

## Development setup

Prerequisites (macOS): Rust stable via rustup (the toolchain is pinned in `rust-toolchain.toml`),
Node.js 22+, pnpm (version pinned in `package.json`), Xcode Command Line Tools.

```bash
pnpm install
pnpm tauri dev
```

## Checks

Every change must pass:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm lint          # Biome; use `pnpm format` to auto-fix
pnpm typecheck
pnpm test
```

## Architecture rules

- `crates/notetice-core` and `crates/notetice-sync` must not depend on Tauri (CI enforces this).
- All IDs are UUIDv7. Synced data is never hard-deleted — use tombstones (`deleted_at`).
- All note and metadata changes go through CRDT documents; SQLite index tables are rebuildable caches.
- In folder sync, a device writes only inside its own `devices/<device_id>/` folder.
- Sync file writes are atomic: write `*.tmp` → fsync → rename.
- The `.ntc` sync format is a public contract: changes require a format version bump and an
  update to `docs/sync-format.md`.

## Code conventions

- Code, identifiers, comments, commit messages: English.
- Rust: `thiserror` in library crates, `anyhow` in the app, `tracing` for logs.
  No `unwrap()` outside tests (enforced by clippy).
- Frontend: React + TypeScript (strict). All calls into Rust go through `apps/desktop/src/lib/ipc.ts`.

## Commits and pull requests

- Small, focused commits using [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `chore:`, `docs:`, `ci:`, `refactor:`, `test:`).
- **Sign off every commit** (Developer Certificate of Origin):

  ```bash
  git commit -s -m "fix: handle missing sync folder"
  ```

  This adds a `Signed-off-by: Your Name <you@example.com>` line, certifying that you wrote the
  change or otherwise have the right to submit it under the project's license
  (see <https://developercertificate.org/>).

## License

By contributing, you agree that your contributions are dual licensed under MIT OR Apache-2.0,
as described in the [README](README.md#license).
