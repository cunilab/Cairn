//! Assemble the daemon-owned frame around server-authorized continuity data.
//!
//! The alpha.9 edge is intentionally thin: repository working state is local,
//! while prior handoffs, warnings, pins, and durable knowledge are authorized
//! by the server. This module therefore accepts continuity only from the
//! authenticated retrieval response (or its account-bound outage cache) and
//! never reconstructs it from historical SQLite tables.

use crate::state::{repo_state, Daemon, Resolved};
use cairn_core::context::{assemble, Caps, ContextInputs, Level0};
use cairn_core::domain::{Handoff, RepositoryState};
use cairn_core::wire::{ContextPayload, ContextWarning, PinnedConstraint, WireError};
use serde::Deserialize;
use serde_json::Value;

#[derive(Default, Deserialize)]
struct Continuity {
    #[serde(default)]
    previous_handoff: Option<Handoff>,
    #[serde(default)]
    warnings: Vec<ContextWarning>,
    #[serde(default)]
    pins: Vec<PinnedConstraint>,
}

/// Build the complete non-durable frame. The caller merges the server-selected
/// durable sections afterward; reading or selecting them here would spend twice
/// for content that is then replaced.
pub async fn build(
    daemon: &Daemon,
    resolved: &Resolved,
    continuity: Option<&Value>,
    server_answered: bool,
    budget: usize,
) -> Result<ContextPayload, WireError> {
    let git = crate::state::git_status(resolved.repo.worktree_path.clone()).await?;
    let repository = repo_state(&git);
    let (continuity, degraded) = match continuity.cloned() {
        Some(value) => match serde_json::from_value(value) {
            Ok(continuity) => (continuity, false),
            Err(_) => (Continuity::default(), true),
        },
        None => (Continuity::default(), server_answered),
    };
    let config = daemon.config.read().await.clone();

    let caps = Caps {
        warnings_in_context_max: config.warnings_in_context_max,
        pins_in_context_max: config.pins_in_context_max,
        reserve_fraction: config.min_safe_context_fraction,
        global_share_max: cairn_core::context::GLOBAL_SHARE_MAX,
    };
    Ok(assemble_level0(
        &resolved.project,
        repository,
        continuity.previous_handoff.as_ref(),
        &continuity.warnings,
        &continuity.pins,
        caps,
        budget,
        true,
        degraded,
    ))
}

#[allow(clippy::too_many_arguments)]
fn assemble_level0(
    project: &cairn_core::domain::Project,
    repository: RepositoryState,
    previous_handoff: Option<&Handoff>,
    warnings: &[ContextWarning],
    pins: &[PinnedConstraint],
    caps: Caps,
    budget: usize,
    has_history: bool,
    degraded: bool,
) -> ContextPayload {
    let decisions = previous_handoff
        .map(|handoff| handoff.decisions.as_slice())
        .unwrap_or_default();
    let known_failures = previous_handoff
        .map(|handoff| handoff.failures.as_slice())
        .unwrap_or_default();
    assemble(
        &ContextInputs {
            project,
            repository,
            previous_handoff,
            decisions,
            known_failures,
            session_memory: &[],
            branch_memory: &[],
            project_memory: &[],
            patterns: &[],
            has_history,
            degraded,
            level0: Level0 {
                warnings,
                pins,
                previous_next_action: None,
                explain: false,
                caps,
            },
            personal_notes: &[],
            team_guidance: &[],
        },
        budget,
    )
}
