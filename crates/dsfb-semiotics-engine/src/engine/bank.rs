use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::engine::types::HeuristicBankEntry;
use crate::io::schema::{ARTIFACT_SCHEMA_VERSION, HEURISTIC_BANK_SCHEMA_VERSION};
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
        Self {
            metadata: HeuristicBankMetadata {
                schema_version: HEURISTIC_BANK_SCHEMA_VERSION.to_string(),
                bank_version: "heuristic-bank/v3".to_string(),
                note: "Built-in conservative typed heuristic bank for deterministic structural semiotics retrieval.".to_string(),
            },
            entries: crate::engine::semantics_layer::builtin_heuristic_bank_entries(),
        }
    }

    /// Loads the deterministic built-in heuristic bank and validates it under the selected
    /// strictness policy.
    pub fn load_builtin(
        strict_validation: bool,
    ) -> Result<(Self, LoadedBankDescriptor, HeuristicBankValidationReport)> {
        let registry = Self::builtin();
        registry.ensure_supported_schema()?;
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
        let source = std::fs::read_to_string(path).with_context(|| {
            format!("failed to read external heuristic bank {}", path.display())
        })?;
        let registry: Self = serde_json::from_str(&source)
            .with_context(|| format!("failed to parse heuristic bank JSON {}", path.display()))?;
        let registry = registry.normalized();
        registry.ensure_supported_schema()?;
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

    /// Validates the registry with default builtin-bank metadata and non-strict graph symmetry.
    pub fn validate(&self) -> Result<HeuristicBankValidationReport> {
        let descriptor = self.loaded_descriptor(BankSourceKind::Builtin, None, false);
        self.validate_with_descriptor(&descriptor)
    }

    /// Returns the full validation report with default builtin-bank metadata and non-strict graph
    /// symmetry.
    #[must_use]
    pub fn validation_report(&self) -> HeuristicBankValidationReport {
        let descriptor = self.loaded_descriptor(BankSourceKind::Builtin, None, false);
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
        let mut seen = BTreeSet::new();
        let mut duplicates = Vec::new();
        for entry in &self.entries {
            if !seen.insert(entry.heuristic_id.clone()) {
                duplicates.push(entry.heuristic_id.clone());
            }
        }

        let ids = self
            .entries
            .iter()
            .map(|entry| entry.heuristic_id.clone())
            .collect::<BTreeSet<_>>();
        let entry_map = self
            .entries
            .iter()
            .map(|entry| (entry.heuristic_id.as_str(), entry))
            .collect::<BTreeMap<_, _>>();

        let mut self_link_notes = Vec::new();
        let mut compatibility_conflicts = Vec::new();
        let mut unknown_link_targets = Vec::new();
        let mut missing_compatibility_links = Vec::new();
        let mut missing_incompatibility_links = Vec::new();
        let mut provenance_gaps = Vec::new();
        let mut regime_tag_notes = Vec::new();
        let mut retrieval_priority_notes = Vec::new();
        let mut scope_sanity_notes = Vec::new();

        for entry in &self.entries {
            if entry.provenance.source.trim().is_empty() || entry.provenance.note.trim().is_empty()
            {
                provenance_gaps.push(format!(
                    "{} is missing complete provenance text.",
                    entry.heuristic_id
                ));
            }
            if entry.applicability_note.trim().is_empty() {
                provenance_gaps.push(format!(
                    "{} is missing an applicability note.",
                    entry.heuristic_id
                ));
            }
            if entry.retrieval_priority == 0 {
                retrieval_priority_notes.push(format!(
                    "{} uses retrieval_priority=0; priorities should be positive and explicit.",
                    entry.heuristic_id
                ));
            }
            let mut seen_regime_tags = BTreeSet::new();
            for tag in &entry.regime_tags {
                if tag.trim().is_empty() {
                    regime_tag_notes.push(format!(
                        "{} contains an empty regime tag.",
                        entry.heuristic_id
                    ));
                } else if !seen_regime_tags.insert(tag.clone()) {
                    regime_tag_notes.push(format!(
                        "{} repeats regime tag `{}`.",
                        entry.heuristic_id, tag
                    ));
                }
            }

            if entry.compatible_with.contains(&entry.heuristic_id) {
                self_link_notes.push(format!(
                    "{} lists itself as compatible.",
                    entry.heuristic_id
                ));
            }
            if entry.incompatible_with.contains(&entry.heuristic_id) {
                self_link_notes.push(format!(
                    "{} lists itself as incompatible.",
                    entry.heuristic_id
                ));
            }

            for target in &entry.compatible_with {
                if entry.incompatible_with.contains(target) {
                    compatibility_conflicts.push(format!(
                        "{} marks {} as both compatible and incompatible.",
                        entry.heuristic_id, target
                    ));
                }
                if !ids.contains(target) {
                    unknown_link_targets.push(format!(
                        "{} marks unknown compatible target {}.",
                        entry.heuristic_id, target
                    ));
                    continue;
                }
                if let Some(target_entry) = entry_map.get(target.as_str()) {
                    if !target_entry.compatible_with.contains(&entry.heuristic_id) {
                        missing_compatibility_links.push(format!(
                            "{} lists {} as compatible, but the reverse link is missing.",
                            entry.heuristic_id, target
                        ));
                    }
                    if target_entry.incompatible_with.contains(&entry.heuristic_id) {
                        compatibility_conflicts.push(format!(
                            "{} lists {} as compatible, but {} lists {} as incompatible.",
                            entry.heuristic_id, target, target, entry.heuristic_id
                        ));
                    }
                }
            }
            for target in &entry.incompatible_with {
                if !ids.contains(target) {
                    unknown_link_targets.push(format!(
                        "{} marks unknown incompatible target {}.",
                        entry.heuristic_id, target
                    ));
                    continue;
                }
                if let Some(target_entry) = entry_map.get(target.as_str()) {
                    let reverse_missing =
                        !target_entry.incompatible_with.contains(&entry.heuristic_id);
                    let directional_exception = entry
                        .directional_incompatibility_exceptions
                        .contains(target)
                        || target_entry
                            .directional_incompatibility_exceptions
                            .contains(&entry.heuristic_id);
                    if reverse_missing && !directional_exception {
                        missing_incompatibility_links.push(format!(
                            "{} lists {} as incompatible, but the reverse link is missing.",
                            entry.heuristic_id, target
                        ));
                    }
                    if target_entry.compatible_with.contains(&entry.heuristic_id) {
                        compatibility_conflicts.push(format!(
                            "{} lists {} as incompatible, but {} lists {} as compatible.",
                            entry.heuristic_id, target, target, entry.heuristic_id
                        ));
                    }
                }
            }

            if let (Some(minimum), Some(maximum)) = (
                entry.scope_conditions.min_outward_drift_fraction,
                entry.scope_conditions.max_outward_drift_fraction,
            ) {
                if minimum > maximum {
                    scope_sanity_notes.push(format!(
                        "{} has min_outward_drift_fraction > max_outward_drift_fraction.",
                        entry.heuristic_id
                    ));
                }
            }
            if let (Some(minimum), Some(maximum)) = (
                entry.scope_conditions.min_inward_drift_fraction,
                entry.scope_conditions.max_inward_drift_fraction,
            ) {
                if minimum > maximum {
                    scope_sanity_notes.push(format!(
                        "{} has min_inward_drift_fraction > max_inward_drift_fraction.",
                        entry.heuristic_id
                    ));
                }
            }
            if let (Some(minimum), Some(maximum)) = (
                entry.scope_conditions.min_aggregate_monotonicity,
                entry.scope_conditions.max_aggregate_monotonicity,
            ) {
                if minimum > maximum {
                    scope_sanity_notes.push(format!(
                        "{} has min_aggregate_monotonicity > max_aggregate_monotonicity.",
                        entry.heuristic_id
                    ));
                }
            }
            if let (Some(minimum), Some(maximum)) = (
                entry.scope_conditions.min_slew_spike_strength,
                entry.scope_conditions.max_slew_spike_strength,
            ) {
                if minimum > maximum {
                    scope_sanity_notes.push(format!(
                        "{} has min_slew_spike_strength > max_slew_spike_strength.",
                        entry.heuristic_id
                    ));
                }
            }
            if let (Some(minimum), Some(maximum)) = (
                entry.scope_conditions.min_coordinated_group_breach_fraction,
                entry.scope_conditions.max_coordinated_group_breach_fraction,
            ) {
                if minimum > maximum {
                    scope_sanity_notes.push(format!(
                        "{} has min_coordinated_group_breach_fraction > max_coordinated_group_breach_fraction.",
                        entry.heuristic_id
                    ));
                }
            }
        }

        let strict_validation_errors = if descriptor.strict_validation {
            missing_compatibility_links
                .iter()
                .chain(&missing_incompatibility_links)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let mut violations = duplicates.clone();
        violations.extend(self_link_notes.clone());
        violations.extend(compatibility_conflicts.clone());
        violations.extend(unknown_link_targets.clone());
        violations.extend(provenance_gaps.clone());
        violations.extend(regime_tag_notes.clone());
        violations.extend(retrieval_priority_notes.clone());
        violations.extend(scope_sanity_notes.clone());
        violations.extend(strict_validation_errors.clone());

        let warnings = if descriptor.strict_validation {
            Vec::new()
        } else {
            missing_compatibility_links
                .iter()
                .chain(&missing_incompatibility_links)
                .cloned()
                .collect::<Vec<_>>()
        };

        let valid = violations.is_empty();

        let note = if valid {
            format!(
                "Heuristic bank `{}` from `{}` passed deterministic registry validation.",
                descriptor.bank_version,
                descriptor.source_kind.as_label()
            )
        } else if descriptor.strict_validation {
            "Heuristic bank validation failed under strict graph-symmetry checks or mandatory registry integrity rules.".to_string()
        } else {
            "Heuristic bank validation failed under mandatory registry integrity rules. Missing reverse graph links are reported separately and become fatal only in strict mode.".to_string()
        };

        HeuristicBankValidationReport {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            bank_schema_version: self.metadata.schema_version.clone(),
            bank_version: self.metadata.bank_version.clone(),
            bank_source_kind: descriptor.source_kind.clone(),
            bank_source_path: descriptor.source_path.clone(),
            bank_content_hash: descriptor.content_hash.clone(),
            strict_validation: descriptor.strict_validation,
            validation_mode: descriptor.validation_mode.clone(),
            entry_count: self.entries.len(),
            duplicate_ids: duplicates,
            self_link_notes,
            compatibility_conflicts,
            missing_compatibility_links,
            missing_incompatibility_links,
            strict_validation_errors,
            unknown_link_targets,
            provenance_gaps,
            regime_tag_notes,
            retrieval_priority_notes,
            scope_sanity_notes,
            violations,
            warnings,
            valid,
            note,
        }
    }

    fn normalized(mut self) -> Self {
        self.entries.sort_by(|left, right| {
            left.heuristic_id
                .cmp(&right.heuristic_id)
                .then_with(|| left.retrieval_priority.cmp(&right.retrieval_priority))
        });
        for entry in &mut self.entries {
            entry.compatible_with.sort();
            entry.incompatible_with.sort();
            entry.directional_incompatibility_exceptions.sort();
            entry.regime_tags.sort();
        }
        self
    }

    fn ensure_supported_schema(&self) -> Result<()> {
        if self.metadata.schema_version != HEURISTIC_BANK_SCHEMA_VERSION {
            return Err(anyhow!(
                "heuristic bank schema version mismatch: expected `{}`, got `{}`",
                HEURISTIC_BANK_SCHEMA_VERSION,
                self.metadata.schema_version
            ));
        }
        Ok(())
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
