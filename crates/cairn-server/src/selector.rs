use serde::Serialize;

pub const REQUIRED_SCHEMA: i64 = 11;
pub const MAX_RECORDS: usize = 72;

/// The complete admitted record is delivered, so one digest identifies its bytes.
#[derive(Debug, Clone, Serialize)]
pub struct LexicalProvenance {
    pub kind: &'static str,
    pub content_sha256: String,
}

pub fn require_schema(schema_version: i64) -> crate::error::ApiResult<()> {
    if schema_version < REQUIRED_SCHEMA {
        return Err(crate::error::ApiError::new(
            axum::http::StatusCode::CONFLICT,
            "lexical_retrieval_policy_unavailable",
            "task-scoped project recall needs server schema 11",
        ));
    }
    Ok(())
}

pub fn provenance(content: &str) -> LexicalProvenance {
    LexicalProvenance {
        kind: "lexical_full_record",
        content_sha256: cairn_core::digest(content),
    }
}
