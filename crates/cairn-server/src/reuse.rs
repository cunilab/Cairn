//! One project-memory reuse predicate for every server read path.

use crate::error::{ApiError, ApiResult};

pub const REQUIRED_SCHEMA: i64 = 9;
pub const POLICY_ID: &str = "project_attestation_v1";
pub const ACCOUNTABILITY_DISCLOSURE: &str =
    "Capture attestation identifies the authenticated author and their stated support; it is not objective verification.";

pub fn require_schema(schema_version: i64) -> ApiResult<()> {
    if schema_version < REQUIRED_SCHEMA {
        return Err(ApiError::new(
            axum::http::StatusCode::CONFLICT,
            "reuse_policy_unavailable",
            "project-memory reuse needs server schema 9",
        ));
    }
    Ok(())
}

/// SQL condition for current project-memory reuse eligibility.
///
/// `alias` is supplied only by server source code, never by a request. Keeping
/// the condition here makes search, graph, context, pins, and warnings use the
/// same invalidation rules.
pub fn eligible(alias: &'static str) -> String {
    format!(
        "{alias}.deleted_at IS NULL
         AND {alias}.state = 'active'
         AND {alias}.superseded_by_id IS NULL
         AND {alias}.verification IS DISTINCT FROM 'conflicted'
         AND {alias}.verification IS DISTINCT FROM 'drifted'
         AND {alias}.verification IS DISTINCT FROM 'needs_recheck'
         AND EXISTS (
             SELECT 1 FROM project_memory_attestations reuse_att
              LEFT JOIN memories reuse_dep ON reuse_dep.id = reuse_att.dependency_memory_id
             WHERE reuse_att.memory_id = {alias}.id
               AND reuse_att.invalidated_at IS NULL
               AND (
                   reuse_att.dependency_memory_id IS NULL
                   OR (
                       reuse_dep.updated_at = reuse_att.dependency_updated_at
                       AND reuse_dep.project_id = {alias}.project_id
                       AND reuse_dep.deleted_at IS NULL
                       AND reuse_dep.state = 'active'
                       AND reuse_dep.superseded_by_id IS NULL
                       AND reuse_dep.verification IS DISTINCT FROM 'conflicted'
                       AND reuse_dep.verification IS DISTINCT FROM 'drifted'
                       AND reuse_dep.verification IS DISTINCT FROM 'needs_recheck'
                       AND EXISTS (
                           SELECT 1 FROM project_memory_attestations reuse_dep_att
                            WHERE reuse_dep_att.memory_id = reuse_dep.id
                              AND reuse_dep_att.invalidated_at IS NULL
                              AND reuse_dep_att.dependency_memory_id IS NULL
                       )
                       AND NOT EXISTS (
                           SELECT 1 FROM memory_relations reuse_dep_conflict
                            WHERE reuse_dep_conflict.deleted_at IS NULL
                              AND reuse_dep_conflict.kind = 'conflicts_with'
                              AND (reuse_dep.id = reuse_dep_conflict.from_memory_id
                                   OR reuse_dep.id = reuse_dep_conflict.to_memory_id)
                       )
                   )
               )
         )
         AND NOT EXISTS (
             SELECT 1 FROM memory_relations reuse_conflict
              WHERE reuse_conflict.deleted_at IS NULL
                AND reuse_conflict.kind = 'conflicts_with'
                AND ({alias}.id = reuse_conflict.from_memory_id
                     OR {alias}.id = reuse_conflict.to_memory_id)
         )"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_predicate_covers_lifecycle_conflict_and_dependency_change() {
        let sql = eligible("m");
        for required in [
            "m.state = 'active'",
            "m.superseded_by_id IS NULL",
            "m.verification IS DISTINCT FROM 'conflicted'",
            "reuse_conflict.kind = 'conflicts_with'",
            "reuse_dep.updated_at = reuse_att.dependency_updated_at",
            "reuse_att.invalidated_at IS NULL",
        ] {
            assert!(sql.contains(required), "missing {required}: {sql}");
        }
    }
}
