use super::compatibility::compatibility_assessment;
use super::explanations::observation_support_is_limited;
use super::scope_eval::scope_satisfied;
use crate::engine::bank::HeuristicBankRegistry;
use crate::engine::settings::{RetrievalIndexSettings, SemanticRetrievalSettings};
use crate::engine::types::{
    CoordinatedResidualStructure, GrammarStatus, SemanticDisposition, SemanticMatchResult,
    SemanticRetrievalAudit, SyntaxCharacterization,
};
use crate::math::metrics::format_metric;

mod index;
mod scoring;

use self::index::indexed_prefilter;
pub(crate) use self::index::{
    benchmark_retrieval_scaling, build_retrieval_index, SemanticRetrievalIndex,
};
use self::scoring::{
    admissibility_satisfied, build_candidate, candidate_preview, grammar_evidence, regime_satisfied,
};

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
    let mut ranked_candidates_post_regime = regime_entries
        .iter()
        .map(|entry| build_candidate(entry, syntax, &evidence, coordinated))
        .collect::<Vec<_>>();
    ranked_candidates_post_regime.sort_by(|left, right| {
        right
            .entry
            .retrieval_priority
            .cmp(&left.entry.retrieval_priority)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| left.entry.heuristic_id.cmp(&right.entry.heuristic_id))
    });
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
        ranked_candidates_post_regime: ranked_candidates_post_regime
            .iter()
            .map(|candidate| candidate_preview("post_regime", candidate))
            .collect(),
        ranked_candidates_post_scope: candidates
            .iter()
            .map(|candidate| candidate_preview("post_scope", candidate))
            .collect(),
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
