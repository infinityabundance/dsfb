//! Deterministic retrieval indexing and scaling helpers.

use std::collections::{BTreeMap, BTreeSet};

use crate::engine::bank::HeuristicBankRegistry;
use crate::engine::settings::{RetrievalIndexSettings, SemanticRetrievalSettings};
use crate::engine::types::{
    AdmissibilityRequirement, CoordinatedResidualStructure, GrammarStatus, SyntaxCharacterization,
};
use crate::math::metrics::hash_serializable_hex;

use super::super::types::{available_regimes, GrammarEvidence};
use super::{grammar_evidence, retrieve_semantics_with_context, SemanticRetrievalContext};

/// Deterministic semantic-bank retrieval index used to narrow candidate sets before exact typed
/// validation.
#[derive(Clone, Debug)]
pub(crate) struct SemanticRetrievalIndex {
    pub(super) bank_content_hash: String,
    pub(super) minimum_bank_size: usize,
    pub(super) any_entries: Vec<usize>,
    pub(super) boundary_entries: Vec<usize>,
    pub(super) violation_entries: Vec<usize>,
    pub(super) no_violation_entries: Vec<usize>,
    pub(super) entries_without_regime_tags: Vec<usize>,
    pub(super) entries_by_regime_tag: BTreeMap<String, Vec<usize>>,
    pub(super) entries_requiring_group_breach: Vec<usize>,
}

/// Deterministic candidate-count report for indexed retrieval scaling.
#[derive(Clone, Debug)]
pub(crate) struct RetrievalScalingObservation {
    pub bank_size: usize,
    pub retrieval_path: String,
    pub linear_candidates_considered: usize,
    pub indexed_prefilter_candidate_count: usize,
    pub indexed_post_scope_candidate_count: usize,
    pub index_buckets_considered: usize,
    pub note: String,
}

// TRACE:ALGORITHM:ALG-SEMANTIC-INDEX:Deterministic semantic prefilter index:Builds reproducible candidate buckets for larger heuristic banks without replacing exact validation.
pub(crate) fn build_retrieval_index(
    registry: &HeuristicBankRegistry,
    settings: &RetrievalIndexSettings,
) -> SemanticRetrievalIndex {
    let mut index = SemanticRetrievalIndex {
        bank_content_hash: hash_serializable_hex("semantic_retrieval_index", registry)
            .map(|digest| digest.fnv1a_64_hex)
            .unwrap_or_else(|_| "hash-unavailable".to_string()),
        minimum_bank_size: settings.minimum_bank_size,
        any_entries: Vec::new(),
        boundary_entries: Vec::new(),
        violation_entries: Vec::new(),
        no_violation_entries: Vec::new(),
        entries_without_regime_tags: Vec::new(),
        entries_by_regime_tag: BTreeMap::new(),
        entries_requiring_group_breach: Vec::new(),
    };
    for (entry_index, entry) in registry.entries.iter().enumerate() {
        match entry.admissibility_requirements {
            AdmissibilityRequirement::Any => index.any_entries.push(entry_index),
            AdmissibilityRequirement::BoundaryInteraction => {
                index.boundary_entries.push(entry_index)
            }
            AdmissibilityRequirement::ViolationRequired => {
                index.violation_entries.push(entry_index)
            }
            AdmissibilityRequirement::NoViolation => index.no_violation_entries.push(entry_index),
        }
        if entry.regime_tags.is_empty() {
            index.entries_without_regime_tags.push(entry_index);
        } else {
            for tag in &entry.regime_tags {
                index
                    .entries_by_regime_tag
                    .entry(tag.clone())
                    .or_default()
                    .push(entry_index);
            }
        }
        if entry.scope_conditions.require_group_breach {
            index.entries_requiring_group_breach.push(entry_index);
        }
    }
    index
}

// TRACE:CLAIM:CLM-RETRIEVAL-SCALING-REPORT:Retrieval scaling evidence:Exports deterministic candidate-count scaling observations for indexed versus fallback retrieval paths.
pub(crate) fn benchmark_retrieval_scaling(
    syntax: &SyntaxCharacterization,
    grammar: &[GrammarStatus],
    coordinated: Option<&CoordinatedResidualStructure>,
    registry: &HeuristicBankRegistry,
    settings: &SemanticRetrievalSettings,
    index_settings: &RetrievalIndexSettings,
) -> Vec<RetrievalScalingObservation> {
    if !index_settings.export_latency_report {
        return Vec::new();
    }
    let evidence = grammar_evidence(grammar);
    let mut scaling_points = index_settings.benchmark_scaling_points.clone();
    if scaling_points.is_empty() {
        scaling_points.push(registry.entries.len());
    }
    scaling_points.sort_unstable();
    scaling_points.dedup();

    scaling_points
        .into_iter()
        .map(|bank_size| {
            let scaled_registry = scaled_registry(registry, bank_size);
            let scaled_index = build_retrieval_index(&scaled_registry, index_settings);
            let linear_result = retrieve_semantics_with_context(SemanticRetrievalContext {
                scenario_id: "retrieval_scaling_linear",
                syntax,
                grammar,
                coordinated,
                registry: &scaled_registry,
                settings,
                index_settings: &RetrievalIndexSettings {
                    enabled: false,
                    ..index_settings.clone()
                },
                index: None,
            });
            let indexed_result = retrieve_semantics_with_context(SemanticRetrievalContext {
                scenario_id: "retrieval_scaling_indexed",
                syntax,
                grammar,
                coordinated,
                registry: &scaled_registry,
                settings,
                index_settings,
                index: Some(&scaled_index),
            });
            let (_, _, index_buckets_considered) = indexed_prefilter(
                &scaled_registry,
                index_settings,
                Some(&scaled_index),
                &evidence,
                coordinated,
            );
            RetrievalScalingObservation {
                bank_size: scaled_registry.entries.len(),
                retrieval_path: indexed_result.retrieval_audit.retrieval_path.clone(),
                linear_candidates_considered: linear_result.retrieval_audit.prefilter_candidate_count,
                indexed_prefilter_candidate_count: indexed_result
                    .retrieval_audit
                    .prefilter_candidate_count,
                indexed_post_scope_candidate_count: indexed_result
                    .retrieval_audit
                    .heuristic_candidates_post_scope,
                index_buckets_considered,
                note: "Retrieval scaling uses deterministic candidate-count proxies over enlarged typed banks rather than wall-clock timing, so the report stays reproducible across reruns.".to_string(),
            }
        })
        .collect()
}

pub(super) fn indexed_prefilter(
    registry: &HeuristicBankRegistry,
    settings: &RetrievalIndexSettings,
    index: Option<&SemanticRetrievalIndex>,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> (Vec<usize>, String, usize) {
    let Some(index) = index.filter(|index| index_matches_registry(index, registry)) else {
        return linear_prefilter(registry, settings);
    };
    if !settings.enabled || registry.entries.len() < index.minimum_bank_size {
        return linear_prefilter(registry, settings);
    }

    let admissibility_indices = relevant_admissibility_indices(index, evidence);
    let mut regime_indices = index.entries_without_regime_tags.clone();
    let mut buckets_considered = 1usize;
    for regime in available_regimes(evidence, coordinated) {
        if let Some(indices) = index.entries_by_regime_tag.get(&regime) {
            regime_indices.extend(indices.iter().copied());
            buckets_considered += 1;
        }
    }
    regime_indices.sort_unstable();
    regime_indices.dedup();

    let mut prefilter = intersect_sorted(&admissibility_indices, &regime_indices);
    if coordinated.is_none() && !index.entries_requiring_group_breach.is_empty() {
        let required = index
            .entries_requiring_group_breach
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        prefilter.retain(|entry_index| !required.contains(entry_index));
        buckets_considered += 1;
    }
    if prefilter.is_empty() {
        return linear_prefilter(registry, settings);
    }
    (prefilter, "indexed".to_string(), buckets_considered)
}

fn linear_prefilter(
    registry: &HeuristicBankRegistry,
    settings: &RetrievalIndexSettings,
) -> (Vec<usize>, String, usize) {
    (
        (0..registry.entries.len()).collect(),
        if settings.enabled {
            "linear-fallback".to_string()
        } else {
            "linear".to_string()
        },
        0,
    )
}

fn relevant_admissibility_indices(
    index: &SemanticRetrievalIndex,
    evidence: &GrammarEvidence,
) -> Vec<usize> {
    let mut indices = index.any_entries.clone();
    if evidence.boundary_count > 0 {
        indices.extend(index.boundary_entries.iter().copied());
    }
    if evidence.violation_count > 0 {
        indices.extend(index.violation_entries.iter().copied());
    } else {
        indices.extend(index.no_violation_entries.iter().copied());
    }
    indices.sort_unstable();
    indices.dedup();
    indices
}

fn intersect_sorted(left: &[usize], right: &[usize]) -> Vec<usize> {
    let right_set = right.iter().copied().collect::<BTreeSet<_>>();
    left.iter()
        .copied()
        .filter(|item| right_set.contains(item))
        .collect()
}

fn index_matches_registry(
    index: &SemanticRetrievalIndex,
    registry: &HeuristicBankRegistry,
) -> bool {
    hash_serializable_hex("semantic_retrieval_index", registry)
        .map(|digest| digest.fnv1a_64_hex == index.bank_content_hash)
        .unwrap_or(false)
}

fn scaled_registry(registry: &HeuristicBankRegistry, bank_size: usize) -> HeuristicBankRegistry {
    if bank_size <= registry.entries.len() {
        let mut clone = registry.clone();
        clone.entries.truncate(bank_size);
        return clone;
    }

    let entries = (0..bank_size)
        .map(|index| {
            let template = &registry.entries[index % registry.entries.len()];
            let mut entry = template.clone();
            entry.heuristic_id = format!("{}__{:04}", template.heuristic_id, index);
            entry.compatible_with.clear();
            entry.incompatible_with.clear();
            entry.directional_incompatibility_exceptions.clear();
            entry
        })
        .collect::<Vec<_>>();
    HeuristicBankRegistry {
        metadata: registry.metadata.clone(),
        entries,
    }
}
