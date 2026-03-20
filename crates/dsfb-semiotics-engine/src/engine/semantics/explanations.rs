//! Semantic explanation assembly helpers.

use crate::engine::settings::SemanticRetrievalSettings;
use crate::engine::types::{
    AdmissibilityRequirement, CoordinatedResidualStructure, HeuristicBankEntry,
    SyntaxCharacterization,
};
use crate::math::metrics::format_metric;

use super::types::{available_regimes, coordinated_group_breach_ratio, GrammarEvidence};

pub(crate) fn rationale(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> String {
    format!(
        "{} {} {} {}",
        admissibility_explanation(entry, evidence),
        regime_explanation(entry, evidence, coordinated),
        scope_explanation(entry, syntax, coordinated),
        entry.applicability_note,
    )
}

pub(crate) fn metric_highlights(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> Vec<String> {
    let coordinated_breach = syntax
        .coordinated_group_breach_fraction
        .max(coordinated_group_breach_ratio(coordinated));
    match entry.heuristic_id.as_str() {
        "H-PERSISTENT-OUTWARD-DRIFT" | "H-PERSISTENT-ADMISSIBILITY-DEPARTURE" => vec![
            format!(
                "outward_drift_fraction={}",
                format_metric(syntax.outward_drift_fraction)
            ),
            format!(
                "residual_norm_path_monotonicity={}",
                format_metric(syntax.residual_norm_path_monotonicity)
            ),
            format!(
                "radial_sign_persistence={}",
                format_metric(syntax.radial_sign_persistence)
            ),
        ],
        "H-DISCRETE-EVENT" | "H-DISCRETE-EVENT-CURVATURE-HYBRID" => vec![
            format!("slew_spike_count={}", syntax.slew_spike_count),
            format!(
                "slew_spike_strength={}",
                format_metric(syntax.slew_spike_strength)
            ),
            format!("max_slew_norm={}", format_metric(syntax.max_slew_norm)),
        ],
        "H-CURVATURE-RICH-TRANSITION" | "H-CURVATURE-LED-DEPARTURE" => vec![
            format!(
                "mean_squared_slew_norm={}",
                format_metric(syntax.mean_squared_slew_norm)
            ),
            format!(
                "late_slew_growth_score={}",
                format_metric(syntax.late_slew_growth_score)
            ),
            format!("max_slew_norm={}", format_metric(syntax.max_slew_norm)),
        ],
        "H-BOUNDED-OSCILLATORY" | "H-NEAR-BOUNDARY-OSCILLATORY" => vec![
            format!(
                "residual_norm_path_monotonicity={}",
                format_metric(syntax.residual_norm_path_monotonicity)
            ),
            format!(
                "inward_drift_fraction={}",
                format_metric(syntax.inward_drift_fraction)
            ),
            format!(
                "boundary_grazing_episode_count={}",
                syntax.boundary_grazing_episode_count
            ),
        ],
        "H-STRUCTURED-NOISY-TRAJECTORY" => vec![
            format!(
                "mean_squared_slew_norm={}",
                format_metric(syntax.mean_squared_slew_norm)
            ),
            format!("slew_spike_count={}", syntax.slew_spike_count),
            format!(
                "drift_channel_sign_alignment={}",
                format_metric(syntax.drift_channel_sign_alignment)
            ),
        ],
        "H-COORDINATED-RISE" | "H-COORDINATED-DEPARTURE" => vec![
            format!(
                "coordinated_group_breach_fraction={}",
                format_metric(coordinated_breach)
            ),
            format!(
                "outward_drift_fraction={}",
                format_metric(syntax.outward_drift_fraction)
            ),
            format!(
                "drift_channel_sign_alignment={}",
                format_metric(syntax.drift_channel_sign_alignment)
            ),
        ],
        "H-INWARD-CONTAINMENT" | "H-INWARD-RECOVERY" => vec![
            format!(
                "inward_drift_fraction={}",
                format_metric(syntax.inward_drift_fraction)
            ),
            format!("min_margin={}", format_metric(syntax.min_margin)),
            format!("boundary_recovery_count={}", syntax.boundary_recovery_count),
        ],
        _ => vec![
            format!(
                "outward_drift_fraction={}",
                format_metric(syntax.outward_drift_fraction)
            ),
            format!(
                "mean_squared_slew_norm={}",
                format_metric(syntax.mean_squared_slew_norm)
            ),
            format!(
                "coordinated_group_breach_fraction={}",
                format_metric(coordinated_breach)
            ),
        ],
    }
}

pub(crate) fn admissibility_explanation(
    entry: &HeuristicBankEntry,
    evidence: &GrammarEvidence,
) -> String {
    match entry.admissibility_requirements {
        AdmissibilityRequirement::Any => {
            "Admissibility check passed because this bank entry accepts any grammar state mix."
                .to_string()
        }
        AdmissibilityRequirement::BoundaryInteraction => format!(
            "Admissibility check passed because boundary interactions were observed {} time(s).",
            evidence.boundary_count
        ),
        AdmissibilityRequirement::ViolationRequired => format!(
            "Admissibility check passed because violation states were observed {} time(s).",
            evidence.violation_count
        ),
        AdmissibilityRequirement::NoViolation => {
            "Admissibility check passed because no violation states were observed.".to_string()
        }
    }
}

pub(crate) fn regime_explanation(
    entry: &HeuristicBankEntry,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> String {
    let available = available_regimes(evidence, coordinated);
    if entry.regime_tags.is_empty() {
        "Regime check passed because this bank entry does not require specific regime tags."
            .to_string()
    } else {
        let matched = available
            .iter()
            .filter(|regime| entry.regime_tags.contains(*regime))
            .cloned()
            .collect::<Vec<_>>();
        format!(
            "Regime check passed because available regimes `{}` satisfied required tags `{}` via `{}`.",
            if available.is_empty() {
                "none".to_string()
            } else {
                available.join("|")
            },
            entry.regime_tags.join("|"),
            if matched.is_empty() {
                "none".to_string()
            } else {
                matched.join("|")
            }
        )
    }
}

pub(crate) fn scope_explanation(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> String {
    let scope = &entry.scope_conditions;
    let mut notes = Vec::new();
    if let Some(minimum) = scope.min_outward_drift_fraction {
        notes.push(format!(
            "outward_drift_fraction={} >= {}",
            format_metric(syntax.outward_drift_fraction),
            format_metric(minimum)
        ));
    }
    if let Some(maximum) = scope.max_outward_drift_fraction {
        notes.push(format!(
            "outward_drift_fraction={} <= {}",
            format_metric(syntax.outward_drift_fraction),
            format_metric(maximum)
        ));
    }
    if let Some(minimum) = scope.min_inward_drift_fraction {
        notes.push(format!(
            "inward_drift_fraction={} >= {}",
            format_metric(syntax.inward_drift_fraction),
            format_metric(minimum)
        ));
    }
    if let Some(maximum) = scope.max_inward_drift_fraction {
        notes.push(format!(
            "inward_drift_fraction={} <= {}",
            format_metric(syntax.inward_drift_fraction),
            format_metric(maximum)
        ));
    }
    if let Some(maximum) = scope.max_curvature_energy {
        notes.push(format!(
            "mean_squared_slew_norm={} <= {}",
            format_metric(syntax.mean_squared_slew_norm),
            format_metric(maximum)
        ));
    }
    if let Some(minimum) = scope.min_curvature_energy {
        notes.push(format!(
            "mean_squared_slew_norm={} >= {}",
            format_metric(syntax.mean_squared_slew_norm),
            format_metric(minimum)
        ));
    }
    if let Some(maximum) = scope.max_curvature_onset_score {
        notes.push(format!(
            "late_slew_growth_score={} <= {}",
            format_metric(syntax.late_slew_growth_score),
            format_metric(maximum)
        ));
    }
    if let Some(minimum) = scope.min_curvature_onset_score {
        notes.push(format!(
            "late_slew_growth_score={} >= {}",
            format_metric(syntax.late_slew_growth_score),
            format_metric(minimum)
        ));
    }
    if let Some(minimum) = scope.min_directional_persistence {
        notes.push(format!(
            "radial_sign_persistence={} >= {}",
            format_metric(syntax.radial_sign_persistence),
            format_metric(minimum)
        ));
    }
    if let Some(minimum) = scope.min_sign_consistency {
        notes.push(format!(
            "radial_sign_dominance={} >= {}",
            format_metric(syntax.radial_sign_dominance),
            format_metric(minimum)
        ));
    }
    if let Some(minimum) = scope.min_channel_coherence {
        notes.push(format!(
            "drift_channel_sign_alignment={} >= {}",
            format_metric(syntax.drift_channel_sign_alignment),
            format_metric(minimum)
        ));
    }
    if let Some(minimum) = scope.min_aggregate_monotonicity {
        notes.push(format!(
            "residual_norm_path_monotonicity={} >= {}",
            format_metric(syntax.residual_norm_path_monotonicity),
            format_metric(minimum)
        ));
    }
    if let Some(maximum) = scope.max_aggregate_monotonicity {
        notes.push(format!(
            "residual_norm_path_monotonicity={} <= {}",
            format_metric(syntax.residual_norm_path_monotonicity),
            format_metric(maximum)
        ));
    }
    if let Some(minimum) = scope.min_slew_spike_count {
        notes.push(format!(
            "slew_spike_count={} >= {}",
            syntax.slew_spike_count, minimum
        ));
    }
    if let Some(maximum) = scope.max_slew_spike_count {
        notes.push(format!(
            "slew_spike_count={} <= {}",
            syntax.slew_spike_count, maximum
        ));
    }
    if let Some(minimum) = scope.min_slew_spike_strength {
        notes.push(format!(
            "slew_spike_strength={} >= {}",
            format_metric(syntax.slew_spike_strength),
            format_metric(minimum)
        ));
    }
    if let Some(maximum) = scope.max_slew_spike_strength {
        notes.push(format!(
            "slew_spike_strength={} <= {}",
            format_metric(syntax.slew_spike_strength),
            format_metric(maximum)
        ));
    }
    if let Some(minimum) = scope.min_boundary_grazing_episodes {
        notes.push(format!(
            "boundary_grazing_episode_count={} >= {}",
            syntax.boundary_grazing_episode_count, minimum
        ));
    }
    if let Some(maximum) = scope.max_boundary_grazing_episodes {
        notes.push(format!(
            "boundary_grazing_episode_count={} <= {}",
            syntax.boundary_grazing_episode_count, maximum
        ));
    }
    if let Some(minimum) = scope.min_boundary_recovery_count {
        notes.push(format!(
            "boundary_recovery_count={} >= {}",
            syntax.boundary_recovery_count, minimum
        ));
    }
    if let Some(minimum) = scope.min_coordinated_group_breach_fraction {
        notes.push(format!(
            "coordinated_group_breach_fraction={} >= {}",
            format_metric(
                syntax
                    .coordinated_group_breach_fraction
                    .max(coordinated_group_breach_ratio(coordinated))
            ),
            format_metric(minimum)
        ));
    }
    if let Some(maximum) = scope.max_coordinated_group_breach_fraction {
        notes.push(format!(
            "coordinated_group_breach_fraction={} <= {}",
            format_metric(
                syntax
                    .coordinated_group_breach_fraction
                    .max(coordinated_group_breach_ratio(coordinated))
            ),
            format_metric(maximum)
        ));
    }
    if scope.require_group_breach {
        notes.push(format!(
            "coordinated_group_breach_fraction={} > 0",
            format_metric(
                syntax
                    .coordinated_group_breach_fraction
                    .max(coordinated_group_breach_ratio(coordinated))
            )
        ));
    }
    if notes.is_empty() {
        format!(
            "Scope check passed for syntax label `{}` because this bank entry does not impose additional numeric constraints.",
            syntax.trajectory_label
        )
    } else {
        format!(
            "Scope check passed for syntax label `{}` because {}.",
            syntax.trajectory_label,
            notes.join(", ")
        )
    }
}

pub(crate) fn observation_support_is_limited(
    syntax: &SyntaxCharacterization,
    evidence: &GrammarEvidence,
    settings: &SemanticRetrievalSettings,
) -> bool {
    syntax
        .outward_drift_fraction
        .max(syntax.inward_drift_fraction)
        < settings.observation_limited_max_directional_fraction
        && syntax.radial_sign_persistence < settings.observation_limited_max_radial_persistence
        && syntax.radial_sign_dominance < settings.observation_limited_max_radial_dominance
        && syntax.late_slew_growth_score < settings.observation_limited_max_late_slew_growth
        && syntax.slew_spike_count == 0
        && syntax.boundary_grazing_episode_count == 0
        && evidence.boundary_count == 0
        && evidence.violation_count == 0
}
