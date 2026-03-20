//! Governed heuristic-bank validation helpers.

use std::collections::{BTreeMap, BTreeSet};

use crate::engine::bank::{
    HeuristicBankRegistry, HeuristicBankValidationReport, LoadedBankDescriptor,
};
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;

pub(crate) fn build_bank_validation_report(
    registry: &HeuristicBankRegistry,
    descriptor: &LoadedBankDescriptor,
) -> HeuristicBankValidationReport {
    let mut seen = BTreeSet::new();
    let mut duplicates = Vec::new();
    for entry in &registry.entries {
        if !seen.insert(entry.heuristic_id.clone()) {
            duplicates.push(entry.heuristic_id.clone());
        }
    }

    let ids = registry
        .entries
        .iter()
        .map(|entry| entry.heuristic_id.clone())
        .collect::<BTreeSet<_>>();
    let entry_map = registry
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

    for entry in &registry.entries {
        if entry.provenance.source.trim().is_empty() || entry.provenance.note.trim().is_empty() {
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
                let reverse_missing = !target_entry.incompatible_with.contains(&entry.heuristic_id);
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

    let note = if valid && warnings.is_empty() {
        format!(
            "Heuristic bank `{}` from `{}` passed deterministic registry validation.",
            descriptor.bank_version,
            descriptor.source_kind.as_label()
        )
    } else if valid {
        format!(
            "Heuristic bank `{}` from `{}` passed mandatory registry checks under explicit permissive governance. Reverse-link findings remain exported as warnings and this run is not governance-clean.",
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
        bank_schema_version: registry.metadata.schema_version.clone(),
        bank_version: registry.metadata.bank_version.clone(),
        bank_source_kind: descriptor.source_kind.clone(),
        bank_source_path: descriptor.source_path.clone(),
        bank_content_hash: descriptor.content_hash.clone(),
        strict_validation: descriptor.strict_validation,
        validation_mode: descriptor.validation_mode.clone(),
        entry_count: registry.entries.len(),
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
