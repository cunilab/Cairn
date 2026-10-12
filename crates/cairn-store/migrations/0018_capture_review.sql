-- Local, short-lived task text and already-validated capture command snapshots.
-- Neither table is part of any delivery spool or remote synchronization path.
CREATE TABLE local_task_records (
  account_id TEXT NOT NULL,
  server_key TEXT NOT NULL,
  session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  turn_id TEXT NOT NULL,
  task_text TEXT NOT NULL,
  task_digest TEXT NOT NULL,
  truncated INTEGER NOT NULL CHECK (truncated IN (0,1)),
  redacted INTEGER NOT NULL CHECK (redacted IN (0,1)),
  coverage_status TEXT NOT NULL DEFAULT 'unknown'
    CHECK (coverage_status IN ('unknown','covered','no_durable_requirement','missing')),
  coverage_task_digest TEXT,
  coverage_findings_digest TEXT,
  coverage_revision TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  PRIMARY KEY (account_id, server_key, session_id, turn_id)
);
CREATE INDEX local_task_records_expiry ON local_task_records(expires_at);
CREATE INDEX local_task_records_cap ON local_task_records(account_id, server_key, updated_at DESC);

CREATE TABLE local_capture_findings (
  account_id TEXT NOT NULL,
  server_key TEXT NOT NULL,
  session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  turn_id TEXT NOT NULL,
  command_id TEXT NOT NULL,
  payload TEXT NOT NULL,
  payload_digest TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  PRIMARY KEY (account_id, server_key, session_id, turn_id, command_id)
);
CREATE INDEX local_capture_findings_expiry ON local_capture_findings(expires_at);
CREATE INDEX local_capture_findings_cap ON local_capture_findings(account_id, server_key, created_at DESC);
