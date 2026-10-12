-- Local native-turn bookkeeping contains no authored material.
CREATE TABLE capture_checkpoints (
  account_id TEXT NOT NULL,
  server_key TEXT NOT NULL,
  session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  turn_id TEXT NOT NULL,
  intervened INTEGER NOT NULL DEFAULT 0 CHECK (intervened IN (0,1)),
  disposition TEXT CHECK (disposition IN ('capture_admitted','no_durable_finding')),
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (account_id, server_key, session_id, turn_id)
);
CREATE INDEX capture_checkpoint_retention ON capture_checkpoints(updated_at);
