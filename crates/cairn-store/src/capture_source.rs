//! Short-lived, bounded local source proof for a native turn.
//!
//! Task text is redacted before this boundary and is never synchronized. It
//! exists only to validate an explicit current-turn user-report quotation.

use crate::{tx, Result, Store, StoreError};
use serde_json::Value;
use sqlx::{Row, SqliteConnection};
use uuid::Uuid;

use crate::capture_checkpoint::CheckpointKey;

const TTL_SECONDS: i64 = 24 * 60 * 60;
const MAX_RECORDS: i64 = 128;
const TASK_MAX_BYTES: usize = 16 * 1024;
const FINDING_CONTENT_MAX_BYTES: usize = 2048;
const FINDING_PAYLOAD_MAX_BYTES: usize = 8192;

fn refused(message: impl Into<String>) -> StoreError {
    StoreError::Refused { code: "invalid_request", message: message.into() }
}
fn now() -> i64 { chrono::Utc::now().timestamp() }
fn key_binds(key: &CheckpointKey) -> (&str, String, String, String) {
    (&key.server_key, key.account_id.to_string(), key.session_id.to_string(), key.turn_id.to_string())
}
async fn prune(connection: &mut SqliteConnection) -> Result<()> {
    sqlx::query("DELETE FROM local_task_records WHERE expires_at < ?1").bind(now()).execute(&mut *connection).await?;
    Ok(())
}
/// Remove expired local source proof at daemon startup and during maintenance.
pub async fn prune_expired(store: &Store) -> Result<()> {
    let mut transaction = tx::begin(store, "capture_source").await?;
    prune(&mut transaction).await?;
    tx::commit(transaction, "capture_source").await?;
    store.checkpoint().await
}
async fn prune_key(connection: &mut SqliteConnection, key: &CheckpointKey) -> Result<()> {
    prune(connection).await?;
    let (server, account, _, _) = key_binds(key);
    sqlx::query("DELETE FROM local_task_records WHERE rowid IN (SELECT rowid FROM local_task_records WHERE account_id=?1 AND server_key=?2 ORDER BY updated_at DESC, rowid DESC LIMIT -1 OFFSET ?3)")
        .bind(account).bind(server).bind(MAX_RECORDS).execute(&mut *connection).await?;
    Ok(())
}
/// Store bounded, already-redacted task text locally for this exact native turn.
pub async fn record_task(store: &Store, key: &CheckpointKey, text: &str, truncated: bool, redacted: bool) -> Result<()> {
    if text.len() > TASK_MAX_BYTES { return Err(refused("task text exceeds 16384 UTF-8 bytes")); }
    let mut transaction = tx::begin(store, "capture_source").await?;
    prune_key(&mut transaction, key).await?;
    let (server, account, session, turn) = key_binds(key);
    let time = now();
    sqlx::query("INSERT INTO local_task_records (account_id,server_key,session_id,turn_id,task_text,task_digest,truncated,redacted,created_at,updated_at,expires_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?9,?10) ON CONFLICT (account_id,server_key,session_id,turn_id) DO UPDATE SET task_text=excluded.task_text,task_digest=excluded.task_digest,truncated=excluded.truncated,redacted=excluded.redacted,updated_at=excluded.updated_at")
        .bind(account).bind(server).bind(session).bind(turn).bind(text).bind(cairn_core::digest(text)).bind(truncated).bind(redacted).bind(time).bind(time + TTL_SECONDS).execute(&mut *transaction).await?;
    prune_key(&mut transaction, key).await?;
    tx::commit(transaction, "capture_source").await
}
/// Validate a bounded user-report payload against one unique complete local task block.
/// Called from the spool admission transaction; no finding payload is retained locally.
pub(crate) async fn validate_admission_in(connection: &mut SqliteConnection, key: &CheckpointKey, payload: &Value) -> Result<()> {
    let encoded = serde_json::to_string(payload).map_err(|error| StoreError::Corrupt(format!("capture source payload: {error}")))?;
    let content = payload.get("content").and_then(Value::as_str).ok_or_else(|| refused("native finding requires content"))?;
    if encoded.len() > FINDING_PAYLOAD_MAX_BYTES || content.trim().is_empty() || content.len() > FINDING_CONTENT_MAX_BYTES { return Err(refused("native finding exceeds bounded source limits")); }
    if payload.get("capture_attestation").and_then(Value::as_object).and_then(|a| a.get("basis")).and_then(Value::as_str) != Some("user_report") { return Ok(()); }
    let (server, account, session, turn) = key_binds(key);
    let task = sqlx::query("SELECT task_text,truncated,redacted FROM local_task_records WHERE account_id=?1 AND server_key=?2 AND session_id=?3 AND turn_id=?4 AND expires_at>=?5")
        .bind(account).bind(server).bind(session).bind(turn).bind(now()).fetch_optional(&mut *connection).await?.ok_or_else(|| refused("authored finding requires current task input"))?;
    if task.get::<i64,_>("truncated") != 0 || task.get::<i64,_>("redacted") != 0 { return Err(refused("authored finding requires safe complete task input")); }
    let task_text: String = task.get("task_text");
    cairn_core::capture_source::unique_source_passage(&task_text, content).ok_or_else(|| refused("authored finding must quote one unique complete task block"))?;
    Ok(())
}
/// Remove local source proof when credentials change.
pub async fn purge_lane(store: &Store, account_id: Uuid, server_key: &str) -> Result<()> {
    let mut transaction = tx::begin(store, "capture_source").await?;
    sqlx::query("DELETE FROM local_task_records WHERE account_id=?1 AND server_key=?2").bind(account_id.to_string()).bind(server_key).execute(&mut *transaction).await?;
    tx::commit(transaction, "capture_source").await?;
    store.checkpoint().await
}
