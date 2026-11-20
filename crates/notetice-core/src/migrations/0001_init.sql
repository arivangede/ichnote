-- Initial schema (DESIGN.md section 5.2).

-- Settings and device identity ('device_id' is created on first open).
CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE workspaces (
  id          TEXT PRIMARY KEY,          -- UUIDv7
  name        TEXT NOT NULL,
  sync_path   TEXT,                      -- NULL = not synced
  created_at  INTEGER NOT NULL,          -- unix ms
  updated_at  INTEGER NOT NULL
);

-- All CRDT documents (workspace meta and note content).
CREATE TABLE docs (
  id            TEXT PRIMARY KEY,        -- for meta docs: = workspace_id
  workspace_id  TEXT NOT NULL REFERENCES workspaces(id),
  kind          INTEGER NOT NULL,        -- 0 = meta, 1 = note
  state         BLOB NOT NULL,           -- encoded yrs state (v1)
  updated_at    INTEGER NOT NULL
);

-- Derived from the meta documents. Can be rebuilt at any time.
CREATE TABLE notes_index (
  id            TEXT PRIMARY KEY,
  workspace_id  TEXT NOT NULL,
  title         TEXT NOT NULL DEFAULT '',
  folder_id     TEXT,
  pinned        INTEGER NOT NULL DEFAULT 0,
  in_inbox      INTEGER NOT NULL DEFAULT 0,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  deleted_at    INTEGER
);
CREATE INDEX idx_notes_ws      ON notes_index(workspace_id, deleted_at, updated_at DESC);
CREATE INDEX idx_notes_pinned  ON notes_index(workspace_id, pinned) WHERE deleted_at IS NULL;

-- Full-text search over the plain text of note content. Derived, rebuildable.
CREATE VIRTUAL TABLE notes_fts USING fts5(
  note_id UNINDEXED, title, body, tokenize = 'unicode61 remove_diacritics 2'
);

-- Local updates not yet written to a transport (local only, never synced).
CREATE TABLE outbox (
  seq           INTEGER PRIMARY KEY AUTOINCREMENT,
  workspace_id  TEXT NOT NULL,
  doc_id        TEXT NOT NULL,
  doc_kind      INTEGER NOT NULL,
  payload       BLOB NOT NULL,           -- yrs update v1
  created_at    INTEGER NOT NULL
);

-- Next file sequence number this device writes, per workspace.
CREATE TABLE sync_out_state (
  workspace_id  TEXT PRIMARY KEY,
  next_seq      INTEGER NOT NULL DEFAULT 1,
  last_compact  INTEGER
);

-- How far this device has read files from other devices.
CREATE TABLE sync_cursor (
  workspace_id    TEXT NOT NULL,
  source_device   TEXT NOT NULL,
  last_seq        INTEGER NOT NULL,
  PRIMARY KEY (workspace_id, source_device)
);

-- Devices known in a workspace (from device.json).
CREATE TABLE devices (
  workspace_id  TEXT NOT NULL,
  device_id     TEXT NOT NULL,
  name          TEXT NOT NULL,
  platform      TEXT NOT NULL,
  last_seen_at  INTEGER,
  PRIMARY KEY (workspace_id, device_id)
);
