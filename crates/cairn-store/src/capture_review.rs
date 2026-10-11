//! Short-lived local inputs for checking whether a native turn is complete.
//!
//! These records are deliberately separate from delivery spools: task text is
//! never synchronized remotely, and a finding snapshot survives command
//! delivery only long enough to bind a review to the exact local input it
//! inspected. An explicitly configured comparator is a separate caller.

use crate::{tx, Result, Store, StoreError};
use serde_json::Value;
use sqlx::{Row, SqliteConnection};
use uuid::Uuid;

use crate::capture_checkpoint::CheckpointKey;

const TTL_SECONDS: i64 = 24 * 60 * 60;
const MAX_RECORDS: i64 = 128;
const TASK_MAX_BYTES: usize = 16 * 1024;
const REVIEW_FINDINGS_MAX: usize = 8;
const FINDING_CONTENT_MAX_BYTES: usize = 2048;
const FINDING_PAYLOAD_MAX_BYTES: usize = 8192;

#[derive(Debug, Clone, PartialEq)]
pub struct ReviewFinding {
    pub command_id: Uuid,
    pub payload: Value,
    /// Digest of the exact current task that admitted this authored source block.
    pub source_task_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReviewInput {
    pub task: String,
    pub truncated: bool,
    pub redacted: bool,
    pub task_digest: String,
    pub findings_digest: String,
    pub findings: Vec<ReviewFinding>,
}

fn refused(message: impl Into<String>) -> StoreError {
    StoreError::Refused {
        code: "invalid_request",
        message: message.into(),
    }
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn key_binds(key: &CheckpointKey) -> (&str, String, String, String) {
    (
        &key.server_key,
        key.account_id.to_string(),
        key.session_id.to_string(),
        key.turn_id.to_string(),
    )
}

async fn prune(connection: &mut SqliteConnection) -> Result<()> {
    let cutoff = now();
    sqlx::query("DELETE FROM local_task_records WHERE expires_at < ?1")
        .bind(cutoff)
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM local_capture_findings WHERE expires_at < ?1")
        .bind(cutoff)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

/// Remove expired local-only material at daemon startup and during maintenance.
pub async fn prune_expired(store: &Store) -> Result<()> {
    let mut transaction = tx::begin(store, "capture_review").await?;
    prune(&mut transaction).await?;
    tx::commit(transaction, "capture_review").await?;
    store.checkpoint().await
}

async fn prune_key(connection: &mut SqliteConnection, key: &CheckpointKey) -> Result<()> {
    prune(connection).await?;
    let (server, account, _, _) = key_binds(key);
    // Keep snapshots for no more than the most recent 128 turns on this lane.
    sqlx::query(
        "DELETE FROM local_capture_findings
         WHERE (account_id, server_key, session_id, turn_id) IN (
           SELECT account_id, server_key, session_id, turn_id FROM (
             SELECT account_id, server_key, session_id, turn_id, MAX(created_at) AS newest
             FROM local_capture_findings WHERE account_id=?1 AND server_key=?2
             GROUP BY account_id, server_key, session_id, turn_id
             ORDER BY newest DESC, session_id DESC, turn_id DESC
             LIMIT -1 OFFSET ?3
           )
         )",
    )
    .bind(&account)
    .bind(server)
    .bind(MAX_RECORDS)
    .execute(&mut *connection)
    .await?;
    sqlx::query(
        "DELETE FROM local_task_records
         WHERE rowid IN (
           SELECT rowid FROM local_task_records
           WHERE account_id=?1 AND server_key=?2
           ORDER BY updated_at DESC, rowid DESC
           LIMIT -1 OFFSET ?3
         )",
    )
    .bind(&account)
    .bind(server)
    .bind(MAX_RECORDS)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

/// Store bounded, already-redacted task text locally for this exact native turn.
pub async fn record_task(
    store: &Store,
    key: &CheckpointKey,
    text: &str,
    truncated: bool,
    redacted: bool,
) -> Result<()> {
    if text.len() > TASK_MAX_BYTES {
        return Err(refused("task text exceeds 16384 UTF-8 bytes"));
    }
    let digest = cairn_core::digest(text);
    let mut transaction = tx::begin(store, "capture_review").await?;
    prune_key(&mut transaction, key).await?;
    let (server, account, session, turn) = key_binds(key);
    let time = now();
    sqlx::query(
        "INSERT INTO local_task_records
           (account_id,server_key,session_id,turn_id,task_text,task_digest,truncated,redacted,
            coverage_status,coverage_task_digest,coverage_findings_digest,coverage_revision,
            created_at,updated_at,expires_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'unknown',NULL,NULL,NULL,?9,?9,?10)
         ON CONFLICT (account_id,server_key,session_id,turn_id) DO UPDATE SET
           task_text=excluded.task_text, task_digest=excluded.task_digest,
           truncated=excluded.truncated, redacted=excluded.redacted,
           coverage_status=CASE WHEN local_task_records.task_digest != excluded.task_digest
                OR local_task_records.truncated != excluded.truncated
                OR local_task_records.redacted != excluded.redacted
             THEN 'unknown' ELSE local_task_records.coverage_status END,
           coverage_task_digest=CASE WHEN local_task_records.task_digest != excluded.task_digest
                OR local_task_records.truncated != excluded.truncated
                OR local_task_records.redacted != excluded.redacted THEN NULL ELSE local_task_records.coverage_task_digest END,
           coverage_findings_digest=CASE WHEN local_task_records.task_digest != excluded.task_digest
                OR local_task_records.truncated != excluded.truncated
                OR local_task_records.redacted != excluded.redacted THEN NULL ELSE local_task_records.coverage_findings_digest END,
           coverage_revision=CASE WHEN local_task_records.task_digest != excluded.task_digest
                OR local_task_records.truncated != excluded.truncated
                OR local_task_records.redacted != excluded.redacted THEN NULL ELSE local_task_records.coverage_revision END,
           updated_at=excluded.updated_at",
    )
    .bind(&account)
    .bind(server)
    .bind(&session)
    .bind(&turn)
    .bind(text)
    .bind(digest)
    .bind(truncated as i64)
    .bind(redacted as i64)
    .bind(time)
    .bind(time + TTL_SECONDS)
    .execute(&mut *transaction)
    .await?;
    // Enforce the per-lane cap after the new record is visible as well.
    prune_key(&mut transaction, key).await?;
    tx::commit(transaction, "capture_review").await
}

/// Remove all short-lived local review material for one credential lane.
/// Credential replacement must not let a later account or server inspect it.
pub async fn purge_lane(store: &Store, account_id: Uuid, server_key: &str) -> Result<()> {
    let mut transaction = tx::begin(store, "capture_review").await?;
    sqlx::query("DELETE FROM local_task_records WHERE account_id=?1 AND server_key=?2")
        .bind(account_id.to_string())
        .bind(server_key)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM local_capture_findings WHERE account_id=?1 AND server_key=?2")
        .bind(account_id.to_string())
        .bind(server_key)
        .execute(&mut *transaction)
        .await?;
    tx::commit(transaction, "capture_review").await?;
    store.checkpoint().await
}

async fn findings_in(
    connection: &mut SqliteConnection,
    key: &CheckpointKey,
) -> Result<Vec<ReviewFinding>> {
    let (server, account, session, turn) = key_binds(key);
    let rows = sqlx::query(
        "SELECT command_id,payload,source_task_digest FROM local_capture_findings
         WHERE account_id=?1 AND server_key=?2 AND session_id=?3 AND turn_id=?4
         ORDER BY command_id",
    )
    .bind(&account)
    .bind(server)
    .bind(&session)
    .bind(&turn)
    .fetch_all(&mut *connection)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(ReviewFinding {
                command_id: Uuid::parse_str(row.get::<&str, _>("command_id"))
                    .map_err(|_| StoreError::Corrupt("local capture command id".into()))?,
                payload: serde_json::from_str(row.get::<&str, _>("payload"))
                    .map_err(|_| StoreError::Corrupt("local capture payload".into()))?,
                source_task_digest: row.get("source_task_digest"),
            })
        })
        .collect()
}

fn findings_digest(findings: &[ReviewFinding]) -> String {
    let mut source = String::new();
    for finding in findings {
        source.push_str(&finding.command_id.to_string());
        source.push(':');
        source.push_str(&cairn_core::digest(&finding.payload.to_string()));
        source.push('\n');
        source.push(':');
        source.push_str(finding.source_task_digest.as_deref().unwrap_or(""));
    }
    cairn_core::digest(&source)
}

/// Load the exact bounded snapshot a reviewer must judge. Expired input reads absent.
pub async fn load_review(store: &Store, key: &CheckpointKey) -> Result<Option<ReviewInput>> {
    let mut transaction = tx::begin(store, "capture_review").await?;
    prune_key(&mut transaction, key).await?;
    let (server, account, session, turn) = key_binds(key);
    let row = sqlx::query(
        "SELECT task_text,task_digest,truncated,redacted FROM local_task_records
         WHERE account_id=?1 AND server_key=?2 AND session_id=?3 AND turn_id=?4",
    )
    .bind(&account)
    .bind(server)
    .bind(&session)
    .bind(&turn)
    .fetch_optional(&mut *transaction)
    .await?;
    let result = if let Some(row) = row {
        let findings = findings_in(&mut transaction, key).await?;
        Some(ReviewInput {
            task: row.get("task_text"),
            task_digest: row.get("task_digest"),
            truncated: row.get::<i64, _>("truncated") != 0,
            redacted: row.get::<i64, _>("redacted") != 0,
            findings_digest: findings_digest(&findings),
            findings,
        })
    } else {
        None
    };
    tx::commit(transaction, "capture_review").await?;
    Ok(result)
}

fn valid_status(status: &str) -> bool {
    matches!(
        status,
        "covered" | "no_durable_requirement" | "missing" | "unknown"
    )
}

/// Save only a review whose task and finding hashes still describe current input.
pub async fn save_review(
    store: &Store,
    key: &CheckpointKey,
    input: &ReviewInput,
    status: &str,
    revision: &str,
) -> Result<bool> {
    if !valid_status(status) {
        return Err(refused("invalid capture review status"));
    }
    if revision.trim().is_empty() {
        return Err(refused("capture review revision is required"));
    }
    if matches!(status, "covered" | "no_durable_requirement") && (input.truncated || input.redacted)
    {
        return Err(refused(
            "truncated or redacted task input cannot receive positive coverage",
        ));
    }
    if matches!(status, "covered" | "no_durable_requirement")
        && input.findings.len() > REVIEW_FINDINGS_MAX
    {
        return Err(refused(
            "more than eight findings cannot receive positive coverage",
        ));
    }
    let mut transaction = tx::begin(store, "capture_review").await?;
    prune_key(&mut transaction, key).await?;
    let (server, account, session, turn) = key_binds(key);
    let current = sqlx::query(
        "SELECT task_digest,truncated,redacted FROM local_task_records
         WHERE account_id=?1 AND server_key=?2 AND session_id=?3 AND turn_id=?4",
    )
    .bind(&account)
    .bind(server)
    .bind(&session)
    .bind(&turn)
    .fetch_optional(&mut *transaction)
    .await?;
    let task_matches = current.is_some_and(|row| {
        row.get::<String, _>("task_digest") == input.task_digest
            && (row.get::<i64, _>("truncated") != 0) == input.truncated
            && (row.get::<i64, _>("redacted") != 0) == input.redacted
    });
    let findings_match =
        findings_digest(&findings_in(&mut transaction, key).await?) == input.findings_digest;
    let matches = task_matches && findings_match;
    if !matches {
        tx::commit(transaction, "capture_review").await?;
        return Ok(false);
    }
    let result = sqlx::query(
        "UPDATE local_task_records SET coverage_status=?1,coverage_task_digest=?2,
          coverage_findings_digest=?3,coverage_revision=?4,updated_at=?5
         WHERE account_id=?6 AND server_key=?7 AND session_id=?8 AND turn_id=?9
           AND task_digest=?2",
    )
    .bind(status)
    .bind(&input.task_digest)
    .bind(&input.findings_digest)
    .bind(revision)
    .bind(now())
    .bind(&account)
    .bind(server)
    .bind(&session)
    .bind(&turn)
    .execute(&mut *transaction)
    .await?;
    tx::commit(transaction, "capture_review").await?;
    Ok(result.rows_affected() == 1)
}

/// Called only from the attested command admission transaction.
pub(crate) async fn snapshot_finding_in(
    connection: &mut SqliteConnection,
    key: &CheckpointKey,
    command_id: Uuid,
    payload: &Value,
) -> Result<()> {
    if !payload
        .get("capture_attestation")
        .is_some_and(Value::is_object)
    {
        return Err(refused("capture review requires attested finding payload"));
    }
    prune_key(connection, key).await?;
    let encoded = serde_json::to_string(payload)
        .map_err(|error| StoreError::Corrupt(format!("capture review payload: {error}")))?;
    if encoded.len() > FINDING_PAYLOAD_MAX_BYTES
        || payload
            .get("content")
            .and_then(Value::as_str)
            .is_none_or(|content| {
                content.trim().is_empty() || content.len() > FINDING_CONTENT_MAX_BYTES
            })
    {
        return Err(refused(
            "native finding exceeds bounded review payload limits",
        ));
    }
    let (server, account, session, turn) = key_binds(key);
    let source_task_digest: Option<String> = match payload
        .get("capture_attestation")
        .and_then(Value::as_object)
        .and_then(|attestation| attestation.get("basis"))
        .and_then(Value::as_str)
    {
        Some("user_report") => {
            let content = payload
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let task = sqlx::query(
                "SELECT task_text,task_digest,truncated,redacted FROM local_task_records
                 WHERE account_id=?1 AND server_key=?2 AND session_id=?3 AND turn_id=?4
                   AND expires_at>=?5",
            )
            .bind(&account)
            .bind(server)
            .bind(&session)
            .bind(&turn)
            .bind(now())
            .fetch_optional(&mut *connection)
            .await?
            .ok_or_else(|| refused("authored finding requires current task input"))?;
            if task.get::<i64, _>("truncated") != 0 || task.get::<i64, _>("redacted") != 0 {
                return Err(refused(
                    "authored finding requires safe complete task input",
                ));
            }
            let task_text: String = task.get("task_text");
            cairn_core::capture_source::unique_source_passage(&task_text, content).ok_or_else(
                || refused("authored finding must quote one unique complete task block"),
            )?;
            Some(task.get("task_digest"))
        }
        _ => None,
    };
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM local_capture_findings
         WHERE account_id=?1 AND server_key=?2 AND session_id=?3 AND turn_id=?4 AND command_id<>?5",
    )
    .bind(&account)
    .bind(server)
    .bind(&session)
    .bind(&turn)
    .bind(command_id.to_string())
    .fetch_one(&mut *connection)
    .await?;
    if count >= REVIEW_FINDINGS_MAX as i64 {
        return Err(refused("native turn admits at most eight review findings"));
    }
    let time = now();
    sqlx::query(
        "INSERT INTO local_capture_findings
          (account_id,server_key,session_id,turn_id,command_id,payload,payload_digest,source_task_digest,created_at,expires_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT (account_id,server_key,session_id,turn_id,command_id) DO NOTHING",
    )
    .bind(&account)
    .bind(server)
    .bind(&session)
    .bind(&turn)
    .bind(command_id.to_string())
    .bind(&encoded)
    .bind(cairn_core::digest(&encoded))
    .bind(source_task_digest)
    .bind(time)
    .bind(time + TTL_SECONDS)
    .execute(&mut *connection)
    .await?;
    // The insert can create a new turn after the preflight cap check.
    prune_key(connection, key).await?;
    // A new finding invalidates any earlier completeness assessment.
    sqlx::query(
        "UPDATE local_task_records SET coverage_status='unknown',coverage_task_digest=NULL,
         coverage_findings_digest=NULL,coverage_revision=NULL,updated_at=?1
         WHERE account_id=?2 AND server_key=?3 AND session_id=?4 AND turn_id=?5",
    )
    .bind(time)
    .bind(account)
    .bind(server)
    .bind(session)
    .bind(turn)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub(crate) async fn has_current_positive_review_in(
    connection: &mut SqliteConnection,
    key: &CheckpointKey,
    revision: &str,
) -> Result<bool> {
    prune_key(connection, key).await?;
    let (server, account, session, turn) = key_binds(key);
    let row = sqlx::query(
        "SELECT task_digest,truncated,redacted,coverage_status,coverage_task_digest,
                coverage_findings_digest,coverage_revision
           FROM local_task_records
          WHERE account_id=?1 AND server_key=?2 AND session_id=?3 AND turn_id=?4",
    )
    .bind(account)
    .bind(server)
    .bind(session)
    .bind(turn)
    .fetch_optional(&mut *connection)
    .await?;
    let Some(row) = row else { return Ok(false) };
    if row.get::<i64, _>("truncated") != 0
        || row.get::<i64, _>("redacted") != 0
        || !matches!(
            row.get::<String, _>("coverage_status").as_str(),
            "covered" | "no_durable_requirement"
        )
        || revision.is_empty()
        || row.get::<Option<String>, _>("coverage_revision").as_deref() != Some(revision)
        || row
            .get::<Option<String>, _>("coverage_task_digest")
            .as_deref()
            != Some(row.get::<String, _>("task_digest").as_str())
    {
        return Ok(false);
    }
    let findings = findings_in(connection, key).await?;
    if findings.len() > REVIEW_FINDINGS_MAX {
        return Ok(false);
    }
    Ok(row
        .get::<Option<String>, _>("coverage_findings_digest")
        .as_deref()
        == Some(findings_digest(&findings).as_str()))
}

/// Current review state for this exact local input. Incomplete task text never
/// reports positive coverage, even if an older record was once reviewed.
pub async fn review_status(store: &Store, key: &CheckpointKey, revision: &str) -> Result<String> {
    let mut transaction = tx::begin(store, "capture_review").await?;
    let positive = has_current_positive_review_in(&mut transaction, key, revision).await?;
    let (server, account, session, turn) = key_binds(key);
    let status = sqlx::query_scalar::<_, String>(
        "SELECT coverage_status FROM local_task_records
         WHERE account_id=?1 AND server_key=?2 AND session_id=?3 AND turn_id=?4",
    )
    .bind(account)
    .bind(server)
    .bind(session)
    .bind(turn)
    .fetch_optional(&mut *transaction)
    .await?;
    tx::commit(transaction, "capture_review").await?;
    Ok(if positive {
        status.unwrap_or_else(|| "unknown".into())
    } else {
        "unknown".into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fixture() -> (Store, CheckpointKey) {
        let store = Store::open_memory().await.unwrap();
        fixture_with_store(store).await
    }

    async fn fixture_with_store(store: Store) -> (Store, CheckpointKey) {
        let project = Uuid::now_v7();
        let session = Uuid::now_v7();
        sqlx::query("INSERT INTO projects (id,name,git_common_dir,created_at,updated_at) VALUES (?1,'review','/review','now','now')")
            .bind(project.to_string()).execute(store.pool()).await.unwrap();
        sqlx::query("INSERT INTO sessions (id,project_id,user_id,agent,branch,worktree_path,agent_session_key,status,started_at,last_event_at,daemon_run_id) VALUES (?1,?2,'user','codex','main','/review','caller','active','now','now','run')")
            .bind(session.to_string()).bind(project.to_string()).execute(store.pool()).await.unwrap();
        (
            store,
            CheckpointKey {
                account_id: Uuid::now_v7(),
                server_key: "server".into(),
                session_id: session,
                turn_id: Uuid::now_v7(),
            },
        )
    }

    #[tokio::test]
    async fn expiry_and_lane_purge_erase_task_bytes_from_database_and_wal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("edge.sqlite3");
        let (store, key) = fixture_with_store(Store::open(&path).await.unwrap()).await;
        let marker = "private-task-marker-7c63c2ee-erase-after-retention";
        let wal = dir.path().join("edge.sqlite3-wal");
        for expired in [true, false] {
            record_task(&store, &key, marker, false, false)
                .await
                .unwrap();
            let bytes = std::fs::read(&wal).unwrap();
            assert!(bytes.windows(marker.len()).any(|w| w == marker.as_bytes()));
            if expired {
                sqlx::query("UPDATE local_task_records SET expires_at=0")
                    .execute(store.pool())
                    .await
                    .unwrap();
                prune_expired(&store).await.unwrap();
            } else {
                purge_lane(&store, key.account_id, &key.server_key)
                    .await
                    .unwrap();
            }
            for file in [&path, &wal] {
                let bytes = std::fs::read(file).unwrap_or_default();
                assert!(
                    !bytes.windows(marker.len()).any(|w| w == marker.as_bytes()),
                    "deleted task remains in {}",
                    file.display()
                );
            }
        }
    }

    #[tokio::test]
    async fn blocked_checkpoint_reports_failure_and_cleanup_can_retry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("edge.sqlite3");
        let store = Store::open_with_busy_timeout(&path, std::time::Duration::from_millis(20))
            .await
            .unwrap();
        let (store, key) = fixture_with_store(store).await;
        record_task(&store, &key, "private-busy-checkpoint-marker", false, false)
            .await
            .unwrap();
        let mut reader = store.pool().begin().await.unwrap();
        sqlx::query("SELECT task_text FROM local_task_records")
            .fetch_all(&mut *reader)
            .await
            .unwrap();
        let error = purge_lane(&store, key.account_id, &key.server_key)
            .await
            .unwrap_err();
        assert!(matches!(error, StoreError::Io(ref error)
            if error.kind() == std::io::ErrorKind::WouldBlock));
        reader.commit().await.unwrap();
        prune_expired(&store).await.unwrap();
        let marker = b"private-busy-checkpoint-marker";
        for file in [path, dir.path().join("edge.sqlite3-wal")] {
            let bytes = std::fs::read(file).unwrap_or_default();
            assert!(!bytes.windows(marker.len()).any(|w| w == marker));
        }
    }

    async fn add_finding(store: &Store, key: &CheckpointKey) {
        let mut transaction = tx::begin(store, "test").await.unwrap();
        snapshot_finding_in(
            &mut transaction,
            key,
            Uuid::now_v7(),
            &serde_json::json!({"content":"complete finding","capture_attestation":{"basis":"inspected_source","support_summary":"inspected support","source_reference":"local://test","source_revision":"test"}}),
        )
        .await
        .unwrap();
        tx::commit(transaction, "test").await.unwrap();
    }

    async fn admit_payload(
        store: &Store,
        key: &CheckpointKey,
        payload: &Value,
    ) -> Result<crate::spool::CommandAdmission> {
        crate::spool::spool_command_with_checkpoint(
            store,
            crate::spool::NewCommand {
                scope: crate::spool::CommandScope::Session(key.session_id),
                project_id: None,
                account_id: key.account_id,
                server_instance_id: None,
                kind: crate::spool::CommandKind::RememberAttested,
                payload,
            },
            crate::spool::SpoolCapacity::default(),
            Some(key),
        )
        .await
    }

    fn authored(content: &str) -> Value {
        serde_json::json!({"content":content, "capture_attestation":{
            "basis":"user_report", "support_summary":"The user authored this durable policy."}})
    }

    async fn assert_no_admission(store: &Store) {
        for table in [
            "command_spool",
            "command_seq",
            "capture_checkpoints",
            "local_capture_findings",
        ] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(store.pool())
                .await
                .unwrap();
            assert_eq!(count, 0, "{table} changed after source refusal");
        }
    }

    #[tokio::test]
    async fn authored_source_refusal_rolls_back_command_and_checkpoint_state() {
        let complete = "Use DuckDB for disposable benchmarks.\nOnly on synthetic data; production uses PostgreSQL.";
        let duplicate = "Keep backups encrypted.\n\nKeep backups encrypted.";
        for (task, content, truncated, redacted, expired) in [
            (None, complete, false, false, false),
            (
                Some(complete),
                "Use DuckDB for disposable benchmarks.",
                false,
                false,
                false,
            ),
            (Some(complete), complete, true, false, false),
            (Some(complete), complete, false, true, false),
            (Some(complete), complete, false, false, true),
            (
                Some(duplicate),
                "Keep backups encrypted.",
                false,
                false,
                false,
            ),
        ] {
            let (store, key) = fixture().await;
            if let Some(task) = task {
                record_task(&store, &key, task, truncated, redacted)
                    .await
                    .unwrap();
            }
            if expired {
                sqlx::query("UPDATE local_task_records SET expires_at=0")
                    .execute(store.pool())
                    .await
                    .unwrap();
            }
            let error = admit_payload(&store, &key, &authored(content))
                .await
                .unwrap_err();
            assert!(matches!(error, StoreError::Refused { code, .. } if code == "invalid_request"));
            assert_no_admission(&store).await;
        }
    }

    #[tokio::test]
    async fn authored_proof_preserves_conditions_and_rejects_stale_task_credit() {
        let (store, key) = fixture().await;
        let block = "Never remove an audit log.\nAn exception requires both reviewer approvals.";
        record_task(&store, &key, block, false, false)
            .await
            .unwrap();
        let mut payload = authored(block);
        payload["source_task_digest"] = serde_json::json!("caller-forged-proof");
        admit_payload(&store, &key, &payload).await.unwrap();
        let input = load_review(&store, &key).await.unwrap().unwrap();
        assert_eq!(
            input.findings[0].source_task_digest.as_deref(),
            Some(input.task_digest.as_str())
        );
        let stored: String = sqlx::query_scalar("SELECT payload FROM command_spool")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&stored).unwrap()["content"],
            block
        );
        assert!(save_review(&store, &key, &input, "covered", "source-v2")
            .await
            .unwrap());
        let changed = format!("{block}\n\nTeam policy: retain both approval signatures.");
        record_task(&store, &key, &changed, false, false)
            .await
            .unwrap();
        let current = load_review(&store, &key).await.unwrap().unwrap();
        assert_ne!(
            current.findings[0].source_task_digest.as_deref(),
            Some(current.task_digest.as_str())
        );
        assert!(!save_review(&store, &key, &input, "covered", "source-v2")
            .await
            .unwrap());
        assert_eq!(
            review_status(&store, &key, "source-v2").await.unwrap(),
            "unknown"
        );
    }

    #[tokio::test]
    async fn authored_source_cannot_be_borrowed_from_another_turn_or_credential_lane() {
        let (store, key) = fixture().await;
        let task = "All production exports require redaction.";
        record_task(&store, &key, task, false, false).await.unwrap();
        for changed in [
            CheckpointKey {
                turn_id: Uuid::now_v7(),
                ..key.clone()
            },
            CheckpointKey {
                account_id: Uuid::now_v7(),
                ..key.clone()
            },
            CheckpointKey {
                server_key: "other-server".into(),
                ..key.clone()
            },
        ] {
            let error = admit_payload(&store, &changed, &authored(task))
                .await
                .unwrap_err();
            assert!(matches!(error, StoreError::Refused { code, .. } if code == "invalid_request"));
            assert_no_admission(&store).await;
        }
    }

    #[tokio::test]
    async fn inspected_and_unbound_commands_cannot_forge_authored_source_proof() {
        let (store, key) = fixture().await;
        let task = "Retain signed release manifests for five years.";
        record_task(&store, &key, task, false, false).await.unwrap();
        let inspected = serde_json::json!({"content":task,
            "source_task_digest":cairn_core::digest(task), "capture_attestation":{
                "basis":"inspected_source", "support_summary":"Synthetic inspected observation.",
                "source_reference":"local://fixture", "source_revision":"fixture"}});
        admit_payload(&store, &key, &inspected).await.unwrap();
        let input = load_review(&store, &key).await.unwrap().unwrap();
        assert_eq!(input.findings[0].source_task_digest, None);
        let payload = authored(task);
        let admission = crate::spool::spool_command_with_checkpoint(
            &store,
            crate::spool::NewCommand {
                scope: crate::spool::CommandScope::Session(key.session_id),
                project_id: None,
                account_id: key.account_id,
                server_instance_id: None,
                kind: crate::spool::CommandKind::RememberAttested,
                payload: &payload,
            },
            crate::spool::SpoolCapacity::default(),
            None,
        )
        .await
        .unwrap();
        let crate::spool::CommandAdmission::Spooled(command) = admission else {
            panic!("unbound command refused")
        };
        let snapshots: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM local_capture_findings WHERE command_id=?1")
                .bind(command.command_id.to_string())
                .fetch_one(store.pool())
                .await
                .unwrap();
        assert_eq!(snapshots, 0);
    }

    #[tokio::test]
    async fn changing_trusted_proof_invalidates_cached_positive_coverage() {
        let (store, key) = fixture().await;
        let task = "Never publish an unsigned artifact.";
        record_task(&store, &key, task, false, false).await.unwrap();
        admit_payload(&store, &key, &authored(task)).await.unwrap();
        let input = load_review(&store, &key).await.unwrap().unwrap();
        assert!(save_review(&store, &key, &input, "covered", "source-v2")
            .await
            .unwrap());
        sqlx::query("UPDATE local_capture_findings SET source_task_digest=NULL")
            .execute(store.pool())
            .await
            .unwrap();
        assert_eq!(
            review_status(&store, &key, "source-v2").await.unwrap(),
            "unknown"
        );
        assert!(!save_review(&store, &key, &input, "covered", "source-v2")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn oversized_and_ninth_native_finding_roll_back_admission() {
        let (store, key) = fixture().await;
        for payload in [
            serde_json::json!({"content":"x".repeat(2049),"capture_attestation":{"basis":"inspected_source","support_summary":"inspected support","source_reference":"local://test","source_revision":"test"}}),
            serde_json::json!({"content":"bounded finding","capture_attestation":{"basis":"inspected_source","support_summary":"inspected support","source_reference":"local://test","source_revision":"test"},"extra":"x".repeat(8192)}),
        ] {
            assert!(admit_payload(&store, &key, &payload).await.is_err());
        }
        for table in [
            "command_spool",
            "command_seq",
            "capture_checkpoints",
            "local_capture_findings",
        ] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(store.pool())
                .await
                .unwrap();
            assert_eq!(count, 0, "{table}");
        }
        let payload = serde_json::json!({"content":"bounded supported finding","capture_attestation":{"basis":"inspected_source","support_summary":"inspected support","source_reference":"local://test","source_revision":"test"}});
        for _ in 0..8 {
            admit_payload(&store, &key, &payload).await.unwrap();
        }
        assert!(admit_payload(&store, &key, &payload).await.is_err());
        let commands: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM command_spool")
            .fetch_one(store.pool())
            .await
            .unwrap();
        let findings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM local_capture_findings")
            .fetch_one(store.pool())
            .await
            .unwrap();
        let next: i64 = sqlx::query_scalar("SELECT next_seq FROM command_seq")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!((commands, findings, next), (8, 8, 9));
    }

    #[tokio::test]
    async fn revision_and_new_finding_invalidate_positive_coverage() {
        let (store, key) = fixture().await;
        record_task(
            &store,
            &key,
            "Preserve the required qualifier.",
            false,
            false,
        )
        .await
        .unwrap();
        add_finding(&store, &key).await;
        let input = load_review(&store, &key).await.unwrap().unwrap();
        assert!(save_review(&store, &key, &input, "covered", "judge-v1")
            .await
            .unwrap());
        assert_eq!(
            review_status(&store, &key, "judge-v1").await.unwrap(),
            "covered"
        );
        assert!(
            !crate::capture_checkpoint::intervene_for_coverage(&store, &key, "judge-v1")
                .await
                .unwrap()
        );
        assert_eq!(
            review_status(&store, &key, "judge-v2").await.unwrap(),
            "unknown"
        );
        add_finding(&store, &key).await;
        assert!(!save_review(&store, &key, &input, "covered", "judge-v1")
            .await
            .unwrap());
        assert_eq!(
            review_status(&store, &key, "judge-v1").await.unwrap(),
            "unknown"
        );
        assert!(
            crate::capture_checkpoint::intervene_for_coverage(&store, &key, "judge-v1")
                .await
                .unwrap()
        );
        assert!(
            !crate::capture_checkpoint::intervene_for_coverage(&store, &key, "judge-v1")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn truncated_task_cannot_receive_positive_coverage() {
        let (store, key) = fixture().await;
        record_task(&store, &key, "Truncated source", true, false)
            .await
            .unwrap();
        let input = load_review(&store, &key).await.unwrap().unwrap();
        assert!(save_review(&store, &key, &input, "covered", "judge-v1")
            .await
            .is_err());
        assert_eq!(
            review_status(&store, &key, "judge-v1").await.unwrap(),
            "unknown"
        );
    }

    #[tokio::test]
    async fn expiry_and_session_deletion_remove_local_task_text() {
        let (store, key) = fixture().await;
        record_task(&store, &key, "Private local task", false, false)
            .await
            .unwrap();
        sqlx::query("UPDATE local_task_records SET expires_at=0")
            .execute(store.pool())
            .await
            .unwrap();
        assert!(load_review(&store, &key).await.unwrap().is_none());
        record_task(&store, &key, "Private local task", false, false)
            .await
            .unwrap();
        sqlx::query("DELETE FROM sessions WHERE id=?1")
            .bind(key.session_id.to_string())
            .execute(store.pool())
            .await
            .unwrap();
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM local_task_records")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(rows, 0);
    }

    #[tokio::test]
    async fn task_records_are_capped_and_never_enter_the_command_spool() {
        let (store, key) = fixture().await;
        for _ in 0..129 {
            let turn = CheckpointKey {
                turn_id: Uuid::now_v7(),
                ..key.clone()
            };
            record_task(&store, &turn, "Local task only", false, false)
                .await
                .unwrap();
        }
        let records: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM local_task_records")
            .fetch_one(store.pool())
            .await
            .unwrap();
        let commands: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM command_spool")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(records, MAX_RECORDS);
        assert_eq!(commands, 0);
    }

    #[tokio::test]
    async fn source_proof_upgrade_preserves_records_but_revokes_legacy_coverage() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::migrate::run_to(&pool, 18).await.unwrap();
        let project = Uuid::now_v7().to_string();
        let session = Uuid::now_v7().to_string();
        sqlx::query("INSERT INTO projects (id,name,git_common_dir,created_at,updated_at) VALUES (?1,'upgrade','/upgrade','now','now')").bind(&project).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO sessions (id,project_id,user_id,agent,branch,worktree_path,agent_session_key,status,started_at,last_event_at,daemon_run_id) VALUES (?1,?2,'user','codex','main','/upgrade','upgrade','active','now','now','upgrade')").bind(&session).bind(&project).execute(&pool).await.unwrap();
        let text = "Never publish without approval.";
        let payload =
            serde_json::json!({"content":text,"capture_attestation":{"basis":"user_report"}})
                .to_string();
        let digest = cairn_core::digest(text);
        sqlx::query("INSERT INTO local_task_records (account_id,server_key,session_id,turn_id,task_text,task_digest,truncated,redacted,coverage_status,coverage_task_digest,coverage_findings_digest,coverage_revision,created_at,updated_at,expires_at) VALUES ('account','server',?1,'turn',?2,?3,0,0,'covered',?3,'legacy-findings','capture-completeness-v1',1,1,9999999999)")
            .bind(&session).bind(text).bind(&digest).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO local_capture_findings (account_id,server_key,session_id,turn_id,command_id,payload,payload_digest,created_at,expires_at) VALUES ('account','server',?1,'turn','command',?2,?3,1,9999999999)")
            .bind(&session).bind(&payload).bind(cairn_core::digest(&payload)).execute(&pool).await.unwrap();
        crate::migrate::run_to(&pool, 19).await.unwrap();
        let (retained, status, task_credit, finding_credit, revision): (String,String,Option<String>,Option<String>,Option<String>) = sqlx::query_as("SELECT task_text,coverage_status,coverage_task_digest,coverage_findings_digest,coverage_revision FROM local_task_records").fetch_one(&pool).await.unwrap();
        assert_eq!(retained, text);
        assert_eq!(status, "unknown");
        assert_eq!((task_credit, finding_credit, revision), (None, None, None));
        let (retained, proof): (String, Option<String>) =
            sqlx::query_as("SELECT payload,source_task_digest FROM local_capture_findings")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(retained, payload);
        assert_eq!(proof, None);
    }

    #[tokio::test]
    async fn credential_lane_purge_removes_task_and_finding_snapshots() {
        let (store, key) = fixture().await;
        record_task(&store, &key, "Local credential-bound task", false, false)
            .await
            .unwrap();
        add_finding(&store, &key).await;
        purge_lane(&store, key.account_id, &key.server_key)
            .await
            .unwrap();
        assert!(load_review(&store, &key).await.unwrap().is_none());
        let findings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM local_capture_findings")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(findings, 0);
    }
}
