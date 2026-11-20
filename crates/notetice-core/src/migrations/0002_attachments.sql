-- Attachments (images and other files) inserted into notes.
-- File contents live in <data dir>/attachments/<hash[0..2]>/<hash>, named by their blake3 hash,
-- so identical files are stored once. Notes reference attachments by `id`.
CREATE TABLE attachments (
  id            TEXT PRIMARY KEY,        -- UUIDv7
  workspace_id  TEXT NOT NULL REFERENCES workspaces(id),
  hash          TEXT NOT NULL,           -- blake3 of the contents, hex
  name          TEXT NOT NULL,           -- original file name
  mime          TEXT NOT NULL,
  size          INTEGER NOT NULL,
  created_at    INTEGER NOT NULL
);
CREATE INDEX idx_attachments_hash ON attachments(hash);
