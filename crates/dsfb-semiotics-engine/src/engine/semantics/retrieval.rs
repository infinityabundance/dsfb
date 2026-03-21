use std::collections::{BTreeMap, BTreeSet};

use super::compatibility::compatibility_assessment;
use super::explanations::{
    admissibility_explanation, metric_highlights, observation_support_is_limited, rationale,
    regime_explanation, scope_explanation,
};
use super::scope_eval::scope_satisfied;
use super::types::{available_regimes, coordinated_group_breach_ratio, GrammarEvidence};
use crate::engine::bank::HeuristicBankRegistry;
use crate::engine::settings::{RetrievalIndexSettings, SemanticRetrievalSettings};
use crate::engine::types::{
    AdmissibilityRequirement, CoordinatedResidualStructure, GrammarState, GrammarStatus,
    HeuristicBankEntry, HeuristicCandidate, SemanticDisposition, SemanticMatchResult,
    SemanticRetrievalAudit, SyntaxCharacterization,
};
use crate::math::metrics::{format_metric, hash_serializable_hex};

/// Deterministic semantic-bank retrieval index used to narrow candidate sets before exact typed
/// validation.
#[derive(Clone, Debug)]
pub(crate) struct SemanticRetrievalIndex {
    bank_content_hash: String,
    minimum_bank_size: usize,
    any_entries: Vec<usize>,
    boundary_entries: Vec<usize>,
    violation_entries: Vec<usize>,
    no_violation_entries: Vec<usize>,
    entries_without_regime_tags: Vec<usize>,
    entries_by_regime_tag: BTreeMap<String, Vec<usize>>,
    entries_requiring_group_breach: Vec<usize>,
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

#[derive(Clone, Copy)]
pub(crate) struct SemanticRetrievalContext<'a> {
    pub scenario_id: &'a str,
    pub syntax: &'a SyntaxCharacterization,
    pub grammar: &'a [GrammarStatus],
    pub coordinated: Option<&'a CoordinatedResidualStructure>,
    pub registry: &'a HeuristicBankRegistry,
    pub settings: &'a SemanticRetrievalSettings,
    pub index_settings: &'a RetrievalIndexSettings,
    pub index: Option<&'a SemanticRetrievalIndex>,
}

pub fn retrieve_semantics(
    scenario_id: &str,
    syntax: &SyntaxCharacterization,
    grammar: &[GrammarStatus],
    coordinated: Option<&CoordinatedResidualStructure>,
) -> SemanticMatchResult {
    retrieve_semantics_with_context(SemanticRetrievalContext {
        scenario_id,
        syntax,
        grammar,
        coordinated,
        registry: &HeuristicBankRegistry::builtin(),
        settings: &SemanticRetrievalSettings::default(),
        index_settings: &RetrievalIndexSettings::default(),
        index: None,
    })
}

pub fn retrieve_semantics_with_registry(
    scenario_id: &str,
    syntax: &SyntaxCharacterization,
    grammar: &[GrammarStatus],
    coordinated: Option<&CoordinatedResidualStructure>,
    registry: &HeuristicBankRegistry,
    settings: &SemanticRetrievalSettings,
) -> SemanticMatchResult {
    retrieve_semantics_with_context(SemanticRetrievalContext {
        scenario_id,
        syntax,
        grammar,
        coordinated,
        registry,
        settings,
        index_settings: &RetrievalIndexSettings::default(),
        index: None,
    })
}

// TRACE:ALGORITHM:ALG-SEMANTIC-RETRIEVAL:Typed semantic retrieval:Applies admissibility, regime, scope, and compatibility filtering to conservative semantic interpretation.
pub(crate) fn retrieve_semantics_with_context(
    context: SemanticRetrievalContext<'_>,
) -> SemanticMatchResult {
    let SemanticRetrievalContext {
        scenario_id,
        syntax,
        grammar,
        coordinated,
        registry,
        settings,
        index_settings,
        index,
    } = context;
    let evidence = grammar_evidence(grammar);
    let (prefilter_indices, retrieval_path, index_buckets_considered) =
        indexed_prefilter(registry, index_settings, index, &evidence, coordinated);
    let prefilter_entries = prefilter_indices
        .iter()
        .map(|index| &registry.entries[*index])
        .collect::<Vec<_>>();
    let prefilter_candidate_ids = prefilter_entries
        .iter()
        .map(|entry| entry.heuristic_id.clone())
        .collect::<Vec<_>>();
    let mut admissible_entries = Vec::new();
    let mut rejected_by_admissibility_ids = Vec::new();
    for entry in &prefilter_entries {
        if admissibility_satisfied(entry, &evidence) {
            admissible_entries.push(*entry);
        } else {
            rejected_by_admissibility_ids.push(entry.heuristic_id.clone());
        }
    }

    let mut regime_entries = Vec::new();
    let mut rejected_by_regime_ids = Vec::new();
    let candidate_ids_post_admissibility = admissible_entries
        .iter()
        .map(|entry| entry.heuristic_id.clone())
        .collect::<Vec<_>>();
    for entry in &admissible_entries {
        if regime_satisfied(entry, &evidence, coordinated) {
            regime_entries.push(*entry);
        } else {
            rejected_by_regime_ids.push(entry.heuristic_id.clone());
        }
    }

    let mut scope_entries = Vec::new();
    let mut rejected_by_scope_ids = Vec::new();
    let candidate_ids_post_regime = regime_entries
        .iter()
        .map(|entry| entry.heuristic_id.clone())
        .collect::<Vec<_>>();
    for entry in regime_entries {
        if scope_satisfied(entry, syntax, coordinated, settings.comparison_epsilon) {
            scope_entries.push(entry);
        } else {
            rejected_by_scope_ids.push(entry.heuristic_id.clone());
        }
    }

    let candidate_ids_post_scope = scope_entries
        .iter()
        .map(|entry| entry.heuristic_id.clone())
        .collect::<Vec<_>>();

    let mut candidates = scope_entries
        .iter()
        .map(|entry| build_candidate(entry, syntax, &evidence, coordinated))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .entry
            .retrieval_priority
            .cmp(&left.entry.retrieval_priority)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| left.entry.heuristic_id.cmp(&right.entry.heuristic_id))
    });

    let selected_labels = candidates
        .iter()
        .map(|candidate| candidate.entry.motif_label.clone())
        .collect::<Vec<_>>();
    let selected_heuristic_ids = candidates
        .iter()
        .map(|candidate| candidate.entry.heuristic_id.clone())
        .collect::<Vec<_>>();
    let retrieval_audit = SemanticRetrievalAudit {
        heuristic_bank_entry_count: registry.entries.len(),
        heuristic_candidates_post_admissibility: candidate_ids_post_admissibility.len(),
        heuristic_candidates_post_regime: candidate_ids_post_regime.len(),
        heuristic_candidates_pre_scope: candidate_ids_post_regime.len(),
        heuristic_candidates_post_scope: candidate_ids_post_scope.len(),
        heuristics_rejected_by_admissibility: rejected_by_admissibility_ids.len(),
        heuristics_rejected_by_regime: rejected_by_regime_ids.len(),
        heuristics_rejected_by_scope: rejected_by_scope_ids.len(),
        heuristics_selected_final: selected_heuristic_ids.len(),
        retrieval_path,
        prefilter_candidate_count: prefilter_candidate_ids.len(),
        prefilter_candidate_ids,
        index_buckets_considered,
        candidate_ids_post_admissibility,
        candidate_ids_post_regime,
        candidate_ids_post_scope,
        rejected_by_admissibility_ids,
        rejected_by_regime_ids,
        rejected_by_scope_ids,
        note: "Counts reflect typed bank entries after admissibility, regime, and scope filtering in that order. `heuristic_candidates_pre_scope` is an outward-facing alias for the post-regime count.".to_string(),
    };
    let compatibility = compatibility_assessment(&candidates);
    let conflict_notes = compatibility
        .conflicts
        .iter()
        .chain(&compatibility.unresolved)
        .cloned()
        .collect::<Vec<_>>();
    let observation_limited = observation_support_is_limited(syntax, &evidence, settings);
    let (
        disposition,
        resolution_basis,
        unknown_reason_class,
        unknown_reason_detail,
        compatibility_note,
        compatibility_reasons,
        note,
    ) = if candidates.is_empty() {
        if observation_limited {
            (
                SemanticDisposition::Unknown,
                "Unknown returned because the sampled trajectory provided only limited structural evidence for conservative retrieval.".to_string(),
                Some("low-evidence".to_string()),
                Some(format!(
                    "Low-evidence Unknown was retained because outward={}, inward={}, residual_norm_path_monotonicity={}, mean_squared_slew_norm={}, late_slew_growth_score={}, boundary_episodes={}, violations={}. These exported values stayed below the current bank's conservative evidence thresholds.",
                    format_metric(syntax.outward_drift_fraction),
                    format_metric(syntax.inward_drift_fraction),
                    format_metric(syntax.residual_norm_path_monotonicity),
                    format_metric(syntax.mean_squared_slew_norm),
                    format_metric(syntax.late_slew_growth_score),
                    syntax.boundary_grazing_episode_count,
                    evidence.violation_count
                )),
                "No heuristic bank entry matched, and the sampled trajectory provided only limited structural evidence for conservative semantic retrieval.".to_string(),
                Vec::new(),
                "Unknown is returned here because the observation shows weak admissibility interaction and limited radial or curvature structure. The bank is not forced to label low-evidence cases.".to_string(),
            )
        } else {
            let regime_summary = if evidence.regimes.is_empty() {
                "none".to_string()
            } else {
                evidence.regimes.join("|")
            };
            let metric_summary = format!(
                "outward={}, inward={}, residual_norm_path_monotonicity={}, mean_squared_slew_norm={}, late_slew_growth_score={}, slew_spikes={}, spike_strength={}, boundary_episodes={}, coordinated_group_breach_fraction={}",
                format_metric(syntax.outward_drift_fraction),
                format_metric(syntax.inward_drift_fraction),
                format_metric(syntax.residual_norm_path_monotonicity),
                format_metric(syntax.mean_squared_slew_norm),
                format_metric(syntax.late_slew_growth_score),
                syntax.slew_spike_count,
                format_metric(syntax.slew_spike_strength),
                syntax.boundary_grazing_episode_count,
                format_metric(syntax.coordinated_group_breach_fraction),
            );
            (
                SemanticDisposition::Unknown,
                "Unknown returned because no typed heuristic bank entry covered the observed admissibility-qualified syntax under the available regime and grouped-evidence checks.".to_string(),
                Some("bank-noncoverage".to_string()),
                Some(format!(
                    "Bank-noncoverage Unknown was retained because syntax label `{}` with regimes `{}` and motif summary `{}` did not satisfy any current typed bank entry after admissibility, scope, and regime filtering.",
                    syntax.trajectory_label,
                    regime_summary,
                    metric_summary
                )),
                "No heuristic bank entry satisfied the constrained admissibility, scope, and regime checks.".to_string(),
                Vec::new(),
                "Unknown is returned conservatively because the current typed bank does not cover the observed admissibility-qualified syntax under the configured evidence and regime constraints.".to_string(),
            )
        }
    } else if candidates.len() == 1 {
        (
            SemanticDisposition::Match,
            "Single qualified heuristic remained after admissibility, regime, and scope filtering.".to_string(),
            None,
            None,
            format!(
                "Single heuristic bank entry (`{}`) satisfied the constrained retrieval rules.",
                selected_heuristic_ids[0]
            ),
            Vec::new(),
            "The returned motif remains an illustrative compatibility statement only. It is not a unique-cause diagnosis.".to_string(),
        )
    } else if compatibility.conflicts.is_empty() && compatibility.unresolved.is_empty() {
        (
            SemanticDisposition::CompatibleSet,
            "Multiple heuristics remained, and every matched pair is explicitly marked compatible in the typed bank.".to_string(),
            None,
            None,
            format!(
                "CompatibleSet returned because `{}` matched and every pair is explicitly marked compatible in the typed bank.",
                selected_heuristic_ids.join("`, `")
            ),
            compatibility.compatible_pairs.clone(),
            "The engine reports an explicitly compatible motif set only when every matched pair is marked compatible. The result remains non-exclusive and causally conservative.".to_string(),
        )
    } else {
        (
            SemanticDisposition::Ambiguous,
            "Multiple heuristics remained, but the bank recorded either explicit conflicts or unresolved compatibility pairings, so the engine did not collapse them into one label.".to_string(),
            None,
            None,
            format!(
                "Ambiguous returned because {} matched entries produced {} explicit conflicts and {} unresolved compatibility pairings.",
                candidates.len(),
                compatibility.conflicts.len(),
                compatibility.unresolved.len()
            ),
            Vec::new(),
            "Ambiguity is explicit rather than silently resolved. The engine does not force a unique semantic label when matched heuristics conflict or when compatibility is not explicitly established.".to_string(),
        )
    };

    SemanticMatchResult {
        scenario_id: scenario_id.to_string(),
        disposition,
        motif_summary: format!(
            "syntax={}, outward={}, inward={}, residual_norm_path_monotonicity={}, mean_squared_slew_norm={}, late_slew_growth_score={}, slew_spikes={}, spike_strength={}, coordinated_group_breach_fraction={}, boundary_episodes={}, boundary_recoveries={}, violations={}, regimes={}",
            syntax.trajectory_label,
            format_metric(syntax.outward_drift_fraction),
            format_metric(syntax.inward_drift_fraction),
            format_metric(syntax.residual_norm_path_monotonicity),
            format_metric(syntax.mean_squared_slew_norm),
            format_metric(syntax.late_slew_growth_score),
            syntax.slew_spike_count,
            format_metric(syntax.slew_spike_strength),
            format_metric(syntax.coordinated_group_breach_fraction),
            syntax.boundary_grazing_episode_count,
            syntax.boundary_recovery_count,
            evidence.violation_count,
            if evidence.regimes.is_empty() {
                "none".to_string()
            } else {
                evidence.regimes.join("|")
            }
        ),
        retrieval_audit,
        candidates,
        selected_labels,
        selected_heuristic_ids,
        resolution_basis,
        unknown_reason_class,
        unknown_reason_detail,
        compatibility_note,
        compatibility_reasons,
        conflict_notes,
        note,
    }
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
                linear_candidates_considered: linear_result
                    .retrieval_audit
                    .prefilter_candidate_count,
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

// TRACE:DEFINITION:DEF-GRAMMAR-EVIDENCE:Grammar evidence summary:Reduces grammar trajectory state into counts and regime tags used by semantic retrieval.
fn grammar_evidence(grammar: &[GrammarStatus]) -> GrammarEvidence {
    let boundary_count = grammar
        .iter()
        .filter(|status| matches!(status.state, GrammarState::Boundary))
        .count();
    let violation_count = grammar
        .iter()
        .filter(|status| matches!(status.state, GrammarState::Violation))
        .count();
    let regimes = grammar
        .iter()
        .map(|status| status.regime.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    GrammarEvidence {
        boundary_count,
        violation_count,
        regimes,
    }
}

fn build_candidate(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> HeuristicCandidate {
    let available_regimes = available_regimes(evidence, coordinated);
    let matched_regimes = if entry.regime_tags.is_empty() {
        available_regimes.clone()
    } else {
        available_regimes
            .iter()
            .filter(|regime| entry.regime_tags.contains(*regime))
            .cloned()
            .collect::<Vec<_>>()
    };

    HeuristicCandidate {
        entry: entry.clone(),
        score: score_candidate(entry, syntax, evidence, coordinated),
        metric_highlights: metric_highlights(entry, syntax, coordinated),
        admissibility_explanation: admissibility_explanation(entry, evidence),
        regime_explanation: regime_explanation(entry, evidence, coordinated),
        scope_explanation: scope_explanation(entry, syntax, coordinated),
        rationale: rationale(entry, syntax, evidence, coordinated),
        matched_regimes,
    }
}

fn admissibility_satisfied(entry: &HeuristicBankEntry, evidence: &GrammarEvidence) -> bool {
    match entry.admissibility_requirements {
        AdmissibilityRequirement::Any => true,
        AdmissibilityRequirement::BoundaryInteraction => evidence.boundary_count > 0,
        AdmissibilityRequirement::ViolationRequired => evidence.violation_count > 0,
        AdmissibilityRequirement::NoViolation => evidence.violation_count == 0,
    }
}

fn regime_satisfied(
    entry: &HeuristicBankEntry,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> bool {
    let available = available_regimes(evidence, coordinated);
    entry.regime_tags.is_empty() || entry.regime_tags.iter().any(|tag| available.contains(tag))
}

fn score_candidate(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    _evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> f64 {
    let group_breach_ratio = coordinated_group_breach_ratio(coordinated);
    let score = match entry.heuristic_id.as_str() {
        "H-PERSISTENT-OUTWARD-DRIFT" => {
            0.28 * syntax.outward_drift_fraction
                + 0.24 * syntax.radial_sign_persistence
                + 0.24 * syntax.residual_norm_path_monotonicity
                + 0.12 * syntax.radial_sign_dominance
                + 0.06 * (1.0 / (1.0 + 20.0 * syntax.mean_squared_slew_norm))
                + 0.06 * (1.0 - syntax.late_slew_growth_score)
        }
        "H-PERSISTENT-ADMISSIBILITY-DEPARTURE" => {
            let breach_severity =
                (-syntax.min_margin).max(0.0) / (((-syntax.min_margin).max(0.0)) + 0.1);
            0.28 * syntax.outward_drift_fraction
                + 0.24 * syntax.radial_sign_persistence
                + 0.22 * syntax.residual_norm_path_monotonicity
                + 0.12 * syntax.radial_sign_dominance
                + 0.08 * breach_severity
                + 0.06 * (1.0 / (1.0 + 20.0 * syntax.mean_squared_slew_norm))
        }
        "H-DISCRETE-EVENT" => {
            0.28 * (syntax.max_slew_norm / (syntax.max_slew_norm + 0.15))
                + 0.22 * (syntax.slew_spike_count.min(3) as f64 / 3.0)
                + 0.22 * (syntax.mean_squared_slew_norm / (syntax.mean_squared_slew_norm + 0.03))
                + 0.18 * (syntax.late_slew_growth_score / (syntax.late_slew_growth_score + 0.2))
                + 0.10 * (syntax.slew_spike_strength / (syntax.slew_spike_strength + 0.2))
        }
        "H-CURVATURE-RICH-TRANSITION" => {
            0.30 * (syntax.mean_squared_slew_norm / (syntax.mean_squared_slew_norm + 0.03))
                + 0.25 * syntax.late_slew_growth_score
                + 0.15 * (syntax.slew_spike_count.min(3) as f64 / 3.0)
                + 0.10 * (syntax.slew_spike_strength / (syntax.slew_spike_strength + 0.2))
                + 0.10 * syntax.drift_channel_sign_alignment
                + 0.10 * (1.0 - syntax.residual_norm_path_monotonicity)
        }
        "H-CURVATURE-LED-DEPARTURE" => {
            let breach_severity =
                (-syntax.min_margin).max(0.0) / (((-syntax.min_margin).max(0.0)) + 0.1);
            0.28 * (syntax.mean_squared_slew_norm / (syntax.mean_squared_slew_norm + 0.01))
                + 0.26 * syntax.late_slew_growth_score
                + 0.16 * syntax.outward_drift_fraction
                + 0.12 * syntax.drift_channel_sign_alignment
                + 0.10 * syntax.radial_sign_persistence
                + 0.08 * breach_severity
        }
        "H-MIXED-REGIME-TRANSITION" => {
            let regime_evidence = 1.0;
            0.24 * syntax.late_slew_growth_score
                + 0.20 * (syntax.mean_squared_slew_norm / (syntax.mean_squared_slew_norm + 0.01))
                + 0.16 * syntax.outward_drift_fraction
                + 0.14 * syntax.radial_sign_persistence
                + 0.10 * syntax.radial_sign_dominance
                + 0.08 * syntax.drift_channel_sign_alignment
                + 0.08 * (syntax.slew_spike_strength / (syntax.slew_spike_strength + 0.2))
                + 0.08 * regime_evidence
        }
        "H-BOUNDARY-GRAZING" => {
            0.35 * (syntax.boundary_grazing_episode_count.min(4) as f64 / 4.0)
                + 0.20 * (syntax.boundary_recovery_count.min(4) as f64 / 4.0)
                + 0.20 * (1.0 / (1.0 + syntax.min_margin.abs() * 15.0))
                + 0.15 * (1.0 - syntax.outward_drift_fraction.clamp(0.0, 1.0))
                + 0.10 * (1.0 / (1.0 + 20.0 * syntax.mean_squared_slew_norm))
        }
        "H-RECURRENT-BOUNDARY-RECURRENCE" => {
            0.32 * (syntax.boundary_grazing_episode_count.min(5) as f64 / 5.0)
                + 0.24 * (syntax.boundary_recovery_count.min(5) as f64 / 5.0)
                + 0.14 * (1.0 / (1.0 + syntax.min_margin.abs() * 12.0))
                + 0.12 * (1.0 - syntax.late_slew_growth_score)
                + 0.10 * (1.0 / (1.0 + 15.0 * syntax.mean_squared_slew_norm))
                + 0.08 * (1.0 - syntax.outward_drift_fraction.clamp(0.0, 1.0))
        }
        "H-COORDINATED-RISE" => {
            0.38 * syntax
                .coordinated_group_breach_fraction
                .max(group_breach_ratio)
                + 0.22 * syntax.outward_drift_fraction
                + 0.18 * syntax.drift_channel_sign_alignment
                + 0.22 * syntax.radial_sign_persistence
        }
        "H-COORDINATED-DEPARTURE" => {
            let breach_ratio = syntax
                .coordinated_group_breach_fraction
                .max(group_breach_ratio);
            0.34 * breach_ratio
                + 0.22 * syntax.outward_drift_fraction
                + 0.16 * syntax.radial_sign_persistence
                + 0.14 * syntax.radial_sign_dominance
                + 0.14 * syntax.drift_channel_sign_alignment
        }
        "H-INWARD-CONTAINMENT" => {
            0.35 * syntax.inward_drift_fraction
                + 0.20 * syntax.radial_sign_persistence
                + 0.20 * syntax.radial_sign_dominance
                + 0.15 * (syntax.min_margin / (syntax.min_margin + 0.1)).clamp(0.0, 1.0)
                + 0.05 * (1.0 - syntax.outward_drift_fraction.clamp(0.0, 1.0))
                + 0.05 * (1.0 - syntax.late_slew_growth_score)
        }
        "H-INWARD-RECOVERY" => {
            0.30 * syntax.inward_drift_fraction
                + 0.22 * syntax.radial_sign_persistence
                + 0.18 * syntax.radial_sign_dominance
                + 0.14 * (syntax.boundary_recovery_count.min(4) as f64 / 4.0)
                + 0.10 * (syntax.min_margin / (syntax.min_margin + 0.1)).clamp(0.0, 1.0)
                + 0.06 * (1.0 - syntax.late_slew_growth_score)
        }
        "H-BOUNDED-OSCILLATORY" => {
            let balance =
                1.0 - (syntax.outward_drift_fraction - syntax.inward_drift_fraction).abs();
            0.24 * (1.0 - syntax.residual_norm_path_monotonicity)
                + 0.22 * balance.clamp(0.0, 1.0)
                + 0.18 * syntax.radial_sign_persistence
                + 0.14 * (syntax.mean_squared_slew_norm / (syntax.mean_squared_slew_norm + 0.005))
                + 0.12 * (syntax.slew_spike_count.min(6) as f64 / 6.0)
                + 0.10 * syntax.drift_channel_sign_alignment
        }
        "H-STRUCTURED-NOISY-TRAJECTORY" => {
            let balance =
                1.0 - (syntax.outward_drift_fraction - syntax.inward_drift_fraction).abs();
            0.22 * (syntax.mean_squared_slew_norm / (syntax.mean_squared_slew_norm + 0.01))
                + 0.18 * (syntax.slew_spike_count.min(20) as f64 / 20.0)
                + 0.16 * balance.clamp(0.0, 1.0)
                + 0.14 * syntax.radial_sign_persistence
                + 0.12 * syntax.radial_sign_dominance
                + 0.10 * syntax.drift_channel_sign_alignment
                + 0.08 * syntax.late_slew_growth_score
        }
        "H-BASELINE-COMPATIBLE" => {
            let balance =
                1.0 - (syntax.outward_drift_fraction - syntax.inward_drift_fraction).abs();
            0.28 * balance.clamp(0.0, 1.0)
                + 0.24 * (1.0 - syntax.residual_norm_path_monotonicity)
                + 0.18 * (1.0 / (1.0 + 50.0 * syntax.mean_squared_slew_norm))
                + 0.12 * (1.0 - syntax.late_slew_growth_score)
                + 0.10 * (syntax.min_margin / (syntax.min_margin + 0.1)).clamp(0.0, 1.0)
                + 0.08 * (1.0 / (1.0 + 20.0 * syntax.slew_spike_strength))
        }
        _ => 0.0,
    };
    score.clamp(0.0, 1.0)
}

fn indexed_prefilter(
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
