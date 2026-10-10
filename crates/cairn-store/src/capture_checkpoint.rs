//! Account/server/session/turn-bound native finalization bookkeeping.

use crate::{tx, Result, Store};
use sqlx::SqliteConnection;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct CheckpointKey {
    pub account_id: Uuid,
    /// Digest of the configured server URL, not a credential.
    pub server_key: String,
    pub session_id: Uuid,
    pub turn_id: Uuid,
}

/// Claim at most one intervention without overwriting a completed disposition.
pub async fn intervene(store: &Store, key: &CheckpointKey) -> Result<bool> {
    let mut transaction = tx::begin(store, "capture_checkpoint").await?;
    prune(&mut transaction).await?;
    let result = sqlx::query(
        "INSERT INTO capture_checkpoints
           (account_id,server_key,session_id,turn_id,intervened,updated_at)
         VALUES (?1,?2,?3,?4,1,?5)
         ON CONFLICT (account_id,server_key,session_id,turn_id)
         DO UPDATE SET intervened=1,updated_at=excluded.updated_at
           WHERE capture_checkpoints.intervened=0
             AND capture_checkpoints.disposition IS NULL",
    )
    .bind(key.account_id.to_string())
    .bind(&key.server_key)
    .bind(key.session_id.to_string())
    .bind(key.turn_id.to_string())
    .bind(chrono::Utc::now().timestamp())
    .execute(&mut *transaction)
    .await?;
    tx::commit(transaction, "capture_checkpoint").await?;
    Ok(result.rows_affected() == 1)
}

pub async fn no_durable_finding(store: &Store, key: &CheckpointKey) -> Result<()> {
    let mut transaction = tx::begin(store, "capture_checkpoint").await?;
    complete_in(&mut transaction, key, "no_durable_finding").await?;
    tx::commit(transaction, "capture_checkpoint").await
}

/// Called inside the command admission transaction so a crash cannot grant
/// credit without retaining the command, or retain it without its credit.
pub(crate) async fn capture_admitted_in(
    connection: &mut SqliteConnection,
    key: &CheckpointKey,
) -> Result<()> {
    complete_in(connection, key, "capture_admitted").await
}

async fn complete_in(
    connection: &mut SqliteConnection,
    key: &CheckpointKey,
    disposition: &str,
) -> Result<()> {
    prune(connection).await?;
    sqlx::query(
        "INSERT INTO capture_checkpoints
           (account_id,server_key,session_id,turn_id,disposition,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6)
         ON CONFLICT (account_id,server_key,session_id,turn_id)
         DO UPDATE SET disposition=excluded.disposition,updated_at=excluded.updated_at
           WHERE capture_checkpoints.disposition IS NULL
              OR excluded.disposition='capture_admitted'",
    )
    .bind(key.account_id.to_string())
    .bind(&key.server_key)
    .bind(key.session_id.to_string())
    .bind(key.turn_id.to_string())
    .bind(disposition)
    .bind(chrono::Utc::now().timestamp())
    .execute(connection)
    .await?;
    Ok(())
}

async fn prune(connection: &mut SqliteConnection) -> Result<()> {
    sqlx::query("DELETE FROM capture_checkpoints WHERE updated_at < ?1")
        .bind(chrono::Utc::now().timestamp() - 90 * 24 * 60 * 60)
        .execute(connection)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fixture() -> (Store, CheckpointKey) {
        let store = Store::open_memory().await.unwrap();
        let session = Uuid::now_v7();
        let project = Uuid::now_v7();
        sqlx::query("INSERT INTO projects (id,name,git_common_dir,created_at,updated_at) VALUES (?1,'probe','/probe','now','now')")
            .bind(project.to_string()).execute(store.pool()).await.unwrap();
        sqlx::query("INSERT INTO sessions
          (id,project_id,user_id,agent,branch,worktree_path,agent_session_key,status,started_at,last_event_at,daemon_run_id)
          VALUES (?1,?2,'user','codex','main','/probe','caller','active','now','now','run')")
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
    async fn intervenes_once_and_cannot_erase_completed_disposition() {
        let (store, key) = fixture().await;
        let (a, b) = tokio::join!(intervene(&store, &key), intervene(&store, &key));
        assert_eq!(usize::from(a.unwrap()) + usize::from(b.unwrap()), 1);
        no_durable_finding(&store, &key).await.unwrap();
        assert!(!intervene(&store, &key).await.unwrap());
        let disposition: String = sqlx::query_scalar("SELECT disposition FROM capture_checkpoints")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(disposition, "no_durable_finding");
    }

    #[tokio::test]
    async fn previous_turn_account_or_server_cannot_supply_credit() {
        let (store, key) = fixture().await;
        no_durable_finding(&store, &key).await.unwrap();
        let other_session = Uuid::now_v7();
        sqlx::query("INSERT INTO sessions
          (id,project_id,user_id,agent,branch,worktree_path,agent_session_key,status,started_at,last_event_at,daemon_run_id)
          SELECT ?1,project_id,user_id,agent,branch,'/other','other','active',started_at,last_event_at,daemon_run_id
          FROM sessions WHERE id=?2")
            .bind(other_session.to_string()).bind(key.session_id.to_string())
            .execute(store.pool()).await.unwrap();
        for changed in [
            CheckpointKey {
                session_id: other_session,
                ..key.clone()
            },
            CheckpointKey {
                turn_id: Uuid::now_v7(),
                ..key.clone()
            },
            CheckpointKey {
                account_id: Uuid::now_v7(),
                ..key.clone()
            },
            CheckpointKey {
                server_key: "other".into(),
                ..key.clone()
            },
        ] {
            assert!(intervene(&store, &changed).await.unwrap());
        }
    }

    #[tokio::test]
    async fn expired_bookkeeping_is_pruned_and_session_deletion_is_not_blocked() {
        let (store, key) = fixture().await;
        no_durable_finding(&store, &key).await.unwrap();
        sqlx::query("UPDATE capture_checkpoints SET updated_at=0")
            .execute(store.pool())
            .await
            .unwrap();
        assert!(intervene(&store, &key).await.unwrap());
        sqlx::query("DELETE FROM sessions WHERE id=?1")
            .bind(key.session_id.to_string())
            .execute(store.pool())
            .await
            .unwrap();
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM capture_checkpoints")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(rows, 0);
    }

    async fn admit(
        store: &Store,
        key: &CheckpointKey,
        capacity: crate::spool::SpoolCapacity,
    ) -> Result<crate::spool::CommandAdmission> {
        crate::spool::spool_command_with_checkpoint(
            store,
            crate::spool::NewCommand {
                scope: crate::spool::CommandScope::Session(key.session_id),
                project_id: None,
                account_id: key.account_id,
                server_instance_id: None,
                kind: crate::spool::CommandKind::RememberAttested,
                payload: &serde_json::json!({"content":"bounded supported finding"}),
            },
            capacity,
            Some(key),
        )
        .await
    }

    #[tokio::test]
    async fn admitted_command_and_credit_commit_together_without_downgrade() {
        let (store, key) = fixture().await;
        assert!(matches!(
            admit(&store, &key, crate::spool::SpoolCapacity::default())
                .await
                .unwrap(),
            crate::spool::CommandAdmission::Spooled(_)
        ));
        assert!(!intervene(&store, &key).await.unwrap());
        no_durable_finding(&store, &key).await.unwrap();
        let disposition: String = sqlx::query_scalar("SELECT disposition FROM capture_checkpoints")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(disposition, "capture_admitted");
    }

    #[tokio::test]
    async fn refused_admission_grants_no_credit() {
        let (store, key) = fixture().await;
        let capacity = crate::spool::SpoolCapacity {
            max_events: 0,
            ..Default::default()
        };
        assert!(matches!(
            admit(&store, &key, capacity).await.unwrap(),
            crate::spool::CommandAdmission::Saturated { .. }
        ));
        assert!(intervene(&store, &key).await.unwrap());
    }

    #[tokio::test]
    async fn credit_failure_rolls_back_command_and_ordinal() {
        let (store, key) = fixture().await;
        sqlx::query(
            "CREATE TRIGGER deny_credit BEFORE INSERT ON capture_checkpoints
          BEGIN SELECT RAISE(ABORT,'checkpoint unavailable'); END",
        )
        .execute(store.pool())
        .await
        .unwrap();
        assert!(admit(&store, &key, crate::spool::SpoolCapacity::default())
            .await
            .is_err());
        let commands: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM command_spool")
            .fetch_one(store.pool())
            .await
            .unwrap();
        let ordinals: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM command_seq")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!((commands, ordinals), (0, 0));
    }
}
