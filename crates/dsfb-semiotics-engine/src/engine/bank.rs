use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::engine::semantics::{
    build_bank_validation_report, ensure_supported_bank_schema, load_builtin_registry,
    load_external_registry_json,
};
use crate::engine::types::HeuristicBankEntry;
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::math::metrics::hash_serializable_hex;

/// Source category for the heuristic bank used by one deterministic run.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BankSourceKind {
    Builtin,
    External,
}

impl BankSourceKind {
    /// Returns the machine-readable label exported in CSV and report artifacts.
    #[must_use]
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::External => "external",
        }
    }
}

/// Resolved metadata describing the loaded heuristic bank for one run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadedBankDescriptor {
    pub schema_version: String,
    pub bank_schema_version: String,
    pub bank_version: String,
    pub source_kind: BankSourceKind,
    pub source_path: Option<String>,
    pub content_hash: String,
    pub strict_validation: bool,
    pub validation_mode: String,
    pub note: String,
}

/// Metadata describing a typed heuristic bank artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicBankMetadata {
    pub schema_version: String,
    pub bank_version: String,
    pub note: String,
}

/// Validation summary for the typed heuristic bank registry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicBankValidationReport {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_schema_version: String,
    pub bank_version: String,
    pub bank_source_kind: BankSourceKind,
    pub bank_source_path: Option<String>,
    pub bank_content_hash: String,
    pub strict_validation: bool,
    pub validation_mode: String,
    pub entry_count: usize,
    pub duplicate_ids: Vec<String>,
    pub self_link_notes: Vec<String>,
    pub compatibility_conflicts: Vec<String>,
    pub missing_compatibility_links: Vec<String>,
    pub missing_incompatibility_links: Vec<String>,
    pub strict_validation_errors: Vec<String>,
    pub unknown_link_targets: Vec<String>,
    pub provenance_gaps: Vec<String>,
    pub regime_tag_notes: Vec<String>,
    pub retrieval_priority_notes: Vec<String>,
    pub scope_sanity_notes: Vec<String>,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
    pub valid: bool,
    pub note: String,
}

/// Governed registry for typed heuristic bank entries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicBankRegistry {
    pub metadata: HeuristicBankMetadata,
    pub entries: Vec<HeuristicBankEntry>,
}

impl HeuristicBankRegistry {
    /// Builds the deterministic built-in heuristic bank registry.
    #[must_use]
    pub fn builtin() -> Self {
        load_builtin_registry()
    }

    /// Loads the deterministic built-in heuristic bank and validates it under the selected
    /// strictness policy.
    pub fn load_builtin(
        strict_validation: bool,
    ) -> Result<(Self, LoadedBankDescriptor, HeuristicBankValidationReport)> {
        let registry = Self::builtin();
        ensure_supported_bank_schema(&registry.metadata.schema_version)?;
        let descriptor =
            registry.loaded_descriptor(BankSourceKind::Builtin, None, strict_validation);
        let report = registry.validate_with_descriptor(&descriptor)?;
        Ok((registry, descriptor, report))
    }

    /// Loads a typed external heuristic-bank JSON artifact and validates it under the selected
    /// strictness policy.
    #[cfg(feature = "external-bank")]
    pub fn load_external_json(
        path: &Path,
        strict_validation: bool,
    ) -> Result<(Self, LoadedBankDescriptor, HeuristicBankValidationReport)> {
        let registry = load_external_registry_json(path)?;
        ensure_supported_bank_schema(&registry.metadata.schema_version)?;
        let descriptor =
            registry.loaded_descriptor(BankSourceKind::External, Some(path), strict_validation);
        let report = registry.validate_with_descriptor(&descriptor)?;
        Ok((registry, descriptor, report))
    }

    /// Returns a clear error when external-bank loading is requested but the crate was compiled
    /// without the supporting feature.
    #[cfg(not(feature = "external-bank"))]
    pub fn load_external_json(
        path: &Path,
        _strict_validation: bool,
    ) -> Result<(Self, LoadedBankDescriptor, HeuristicBankValidationReport)> {
        Err(anyhow!(
            "external heuristic-bank loading is unavailable because the `external-bank` feature is disabled; requested {}",
            path.display()
        ))
    }

    /// Validates the registry with default builtin-bank metadata under strict governance.
    pub fn validate(&self) -> Result<HeuristicBankValidationReport> {
        let descriptor = self.loaded_descriptor(BankSourceKind::Builtin, None, true);
        self.validate_with_descriptor(&descriptor)
    }

    /// Returns the full validation report with default builtin-bank metadata under strict
    /// governance.
    #[must_use]
    pub fn validation_report(&self) -> HeuristicBankValidationReport {
        let descriptor = self.loaded_descriptor(BankSourceKind::Builtin, None, true);
        self.validation_report_with_descriptor(&descriptor)
    }

    /// Validates the registry under the provided loaded-bank descriptor.
    pub fn validate_with_descriptor(
        &self,
        descriptor: &LoadedBankDescriptor,
    ) -> Result<HeuristicBankValidationReport> {
        let report = self.validation_report_with_descriptor(descriptor);
        if report.valid {
            Ok(report)
        } else {
            Err(anyhow!(
                "heuristic bank registry failed validation for source `{}` (version `{}`)",
                descriptor.source_kind.as_label(),
                descriptor.bank_version
            ))
        }
    }

    /// Returns the full validation report under the provided loaded-bank descriptor without
    /// converting invalid state into an error.
    #[must_use]
    pub fn validation_report_with_descriptor(
        &self,
        descriptor: &LoadedBankDescriptor,
    ) -> HeuristicBankValidationReport {
        build_bank_validation_report(self, descriptor)
    }

    fn loaded_descriptor(
        &self,
        source_kind: BankSourceKind,
        source_path: Option<&Path>,
        strict_validation: bool,
    ) -> LoadedBankDescriptor {
        let content_hash = hash_serializable_hex("heuristic_bank", self)
            .map(|digest| digest.fnv1a_64_hex)
            .unwrap_or_else(|_| "hash-unavailable".to_string());
        LoadedBankDescriptor {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            bank_schema_version: self.metadata.schema_version.clone(),
            bank_version: self.metadata.bank_version.clone(),
            source_kind: source_kind.clone(),
            source_path: source_path.map(|path| path.display().to_string()),
            content_hash,
            strict_validation,
            validation_mode: if strict_validation {
                "strict".to_string()
            } else {
                "permissive".to_string()
            },
            note: match source_kind {
                BankSourceKind::Builtin => "Compiled builtin heuristic bank used for deterministic offline reference runs.".to_string(),
                BankSourceKind::External => "External heuristic bank artifact loaded and validated at startup before deterministic retrieval.".to_string(),
            },
        }
    }
}
