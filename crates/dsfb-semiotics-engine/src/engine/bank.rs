use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::engine::types::HeuristicBankEntry;
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;

/// Metadata describing the built-in heuristic bank.
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
    pub bank_version: String,
    pub entry_count: usize,
    pub duplicate_ids: Vec<String>,
    pub missing_compatibility_links: Vec<String>,
    pub missing_incompatibility_links: Vec<String>,
    pub unknown_link_targets: Vec<String>,
    pub provenance_gaps: Vec<String>,
    pub scope_sanity_notes: Vec<String>,
    pub valid: bool,
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
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                bank_version: "heuristic-bank/v2".to_string(),
                note: "Built-in conservative typed heuristic bank for deterministic structural semiotics retrieval.".to_string(),
            },
            entries: crate::engine::semantics_layer::builtin_heuristic_bank_entries(),
        }
    }

    /// Validates the registry for duplicate IDs, graph consistency, and basic entry completeness.
    pub fn validate(&self) -> Result<HeuristicBankValidationReport> {
        let report = self.validation_report();
        if report.valid {
            Ok(report)
        } else {
            Err(anyhow!(
                "built-in heuristic bank registry failed validation"
            ))
        }
    }

    /// Returns the full validation report without converting invalid state into an error.
    #[must_use]
    pub fn validation_report(&self) -> HeuristicBankValidationReport {
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

        let mut unknown_link_targets = Vec::new();
        let mut missing_compatibility_links = Vec::new();
        let mut missing_incompatibility_links = Vec::new();
        let mut provenance_gaps = Vec::new();
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

            for target in &entry.compatible_with {
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
                    if !target_entry.incompatible_with.contains(&entry.heuristic_id) {
                        missing_incompatibility_links.push(format!(
                            "{} lists {} as incompatible, but the reverse link is missing.",
                            entry.heuristic_id, target
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
        }

        let valid = duplicates.is_empty()
            && unknown_link_targets.is_empty()
            && provenance_gaps.is_empty()
            && scope_sanity_notes.is_empty();
        HeuristicBankValidationReport {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            bank_version: self.metadata.bank_version.clone(),
            entry_count: self.entries.len(),
            duplicate_ids: duplicates,
            missing_compatibility_links,
            missing_incompatibility_links,
            unknown_link_targets,
            provenance_gaps,
            scope_sanity_notes,
            valid,
        }
    }
}
