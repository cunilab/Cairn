//! Explicit authority for reusing project memory.
//!
//! Capture attestation records who accepted responsibility for a bounded
//! support statement. It is not verification and cannot create a
//! [`crate::domain::VerificationAuthority`].

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

pub const SUPPORT_SUMMARY_MAX_BYTES: usize = 512;
pub const SOURCE_REFERENCE_MAX_BYTES: usize = 256;
pub const SOURCE_REVISION_MAX_BYTES: usize = 128;

/// Why a caller is reading memory.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReusePurpose {
    Reuse,
    /// The compatibility default for archival REST and older clients.
    #[default]
    Inspect,
}

impl ReusePurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reuse => "reuse",
            Self::Inspect => "inspect",
        }
    }
}

impl FromStr for ReusePurpose {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "reuse" => Ok(Self::Reuse),
            "inspect" => Ok(Self::Inspect),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaptureBasis {
    UserReport,
    InspectedSource,
}

impl CaptureBasis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UserReport => "user_report",
            Self::InspectedSource => "inspected_source",
        }
    }
}

/// Client-authored support for a project-memory capture.
///
/// Actor, time, eligibility, and verification authority are deliberately
/// absent. The server derives the actor from authentication and computes the
/// other fields from stored state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureAttestation {
    pub basis: CaptureBasis,
    pub support_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    /// Optional eligible project-memory dependency with no dependency of its
    /// own (one hop maximum). The server records its revision; a caller cannot
    /// claim what revision it inspected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_memory_id: Option<Uuid>,
}

impl CaptureAttestation {
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_text(
            &self.support_summary,
            SUPPORT_SUMMARY_MAX_BYTES,
            "support_summary is required",
            "support_summary is too long",
        )?;
        validate_optional(
            self.source_reference.as_deref(),
            SOURCE_REFERENCE_MAX_BYTES,
            "source_reference is empty",
            "source_reference is too long",
        )?;
        validate_optional(
            self.source_revision.as_deref(),
            SOURCE_REVISION_MAX_BYTES,
            "source_revision is empty",
            "source_revision is too long",
        )?;
        if self.basis == CaptureBasis::InspectedSource
            && (self.source_reference.is_none() || self.source_revision.is_none())
        {
            return Err("inspected_source needs source_reference and source_revision");
        }
        for value in [
            Some(self.support_summary.as_str()),
            self.source_reference.as_deref(),
            self.source_revision.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if crate::redact::contains_secret(value) {
                return Err("capture attestation contains secret-shaped text");
            }
        }
        Ok(())
    }
}

fn validate_optional(
    value: Option<&str>,
    max: usize,
    empty: &'static str,
    oversized: &'static str,
) -> Result<(), &'static str> {
    match value {
        Some(value) => validate_text(value, max, empty, oversized),
        None => Ok(()),
    }
}

fn validate_text(
    value: &str,
    max: usize,
    empty: &'static str,
    oversized: &'static str,
) -> Result<(), &'static str> {
    if value.trim().is_empty() {
        return Err(empty);
    }
    if value.len() > max {
        return Err(oversized);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspected_source_requires_a_named_revision() {
        let attestation = CaptureAttestation {
            basis: CaptureBasis::InspectedSource,
            support_summary: "Read the checked-in configuration.".into(),
            source_reference: Some("config/runtime.toml".into()),
            source_revision: None,
            dependency_memory_id: None,
        };
        assert_eq!(
            attestation.validate(),
            Err("inspected_source needs source_reference and source_revision")
        );
    }

    #[test]
    fn actor_and_authority_are_not_deserializable_attestation_fields() {
        for forbidden in ["actor_user_id", "verification_authority", "eligible"] {
            let mut value = serde_json::json!({
                "basis": "user_report",
                "support_summary": "The user chose this approach."
            });
            value[forbidden] = serde_json::json!("attested");
            assert!(serde_json::from_value::<CaptureAttestation>(value).is_err());
        }
    }
}
