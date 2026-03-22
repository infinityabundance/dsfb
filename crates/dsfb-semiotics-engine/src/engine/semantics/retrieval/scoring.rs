//! Deterministic candidate construction and scoring helpers for semantic retrieval.

use std::collections::BTreeSet;

use super::super::compatibility::compatibility_assessment;
use super::super::explanations::{
    admissibility_explanation, metric_highlights, observation_support_is_limited, rationale,
    regime_explanation, scope_explanation,
};
use super::super::types::{available_regimes, coordinated_group_breach_ratio, GrammarEvidence};
use crate::engine::types::{
    AdmissibilityRequirement, CoordinatedResidualStructure, GrammarState, GrammarStatus,
    HeuristicBankEntry, HeuristicCandidate, RetrievalAuditCandidatePreview, SyntaxCharacterization,
};
use crate::math::metrics::format_metric;

// TRACE:DEFINITION:DEF-GRAMMAR-EVIDENCE:Grammar evidence summary:Reduces grammar trajectory state into counts and regime tags used by semantic retrieval.
pub(super) fn grammar_evidence(grammar: &[GrammarStatus]) -> GrammarEvidence {
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

pub(super) fn build_candidate(
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

pub(super) fn candidate_preview(
    stage: &str,
    candidate: &HeuristicCandidate,
) -> RetrievalAuditCandidatePreview {
    RetrievalAuditCandidatePreview {
        stage: stage.to_string(),
        heuristic_id: candidate.entry.heuristic_id.clone(),
        short_label: candidate.entry.short_label.clone(),
        motif_label: candidate.entry.motif_label.clone(),
        score: candidate.score,
    }
}

pub(super) fn admissibility_satisfied(
    entry: &HeuristicBankEntry,
    evidence: &GrammarEvidence,
) -> bool {
    match entry.admissibility_requirements {
        AdmissibilityRequirement::Any => true,
        AdmissibilityRequirement::BoundaryInteraction => evidence.boundary_count > 0,
        AdmissibilityRequirement::ViolationRequired => evidence.violation_count > 0,
        AdmissibilityRequirement::NoViolation => evidence.violation_count == 0,
    }
}

pub(super) fn regime_satisfied(
    entry: &HeuristicBankEntry,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> bool {
    let available = available_regimes(evidence, coordinated);
    entry.regime_tags.is_empty() || entry.regime_tags.iter().any(|tag| available.contains(tag))
}

pub(super) fn score_candidate(
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

#[allow(dead_code)]
pub(super) fn retrieval_process_note(
    syntax: &SyntaxCharacterization,
    evidence: &GrammarEvidence,
    observation_limited: bool,
) -> String {
    if observation_limited {
        format!(
            "Low-evidence retrieval state with outward={}, inward={}, monotonicity={}, mean_squared_slew_norm={}, late_slew_growth_score={}",
            format_metric(syntax.outward_drift_fraction),
            format_metric(syntax.inward_drift_fraction),
            format_metric(syntax.residual_norm_path_monotonicity),
            format_metric(syntax.mean_squared_slew_norm),
            format_metric(syntax.late_slew_growth_score)
        )
    } else {
        let compatible_regime_summary = if evidence.regimes.is_empty() {
            "none".to_string()
        } else {
            evidence.regimes.join("|")
        };
        format!(
            "Coverage-limited retrieval state with regimes={} and syntax label `{}`",
            compatible_regime_summary, syntax.trajectory_label
        )
    }
}

#[allow(dead_code)]
pub(super) fn retrieval_compatibility_summary(
    candidates: &[HeuristicCandidate],
    syntax: &SyntaxCharacterization,
    evidence: &GrammarEvidence,
    settings: &super::SemanticRetrievalSettings,
) -> String {
    let compatibility = compatibility_assessment(candidates);
    let observation_limited = observation_support_is_limited(syntax, evidence, settings);
    if observation_limited {
        retrieval_process_note(syntax, evidence, observation_limited)
    } else if compatibility.conflicts.is_empty() && compatibility.unresolved.is_empty() {
        "Compatible-set retrieval state under explicit bank compatibility.".to_string()
    } else {
        format!(
            "Ambiguity retained with {} conflicts and {} unresolved compatibility relations.",
            compatibility.conflicts.len(),
            compatibility.unresolved.len()
        )
    }
}
