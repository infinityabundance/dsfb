//! Scope-condition evaluation for admissibility-qualified semantic retrieval.

use crate::engine::types::{
    CoordinatedResidualStructure, HeuristicBankEntry, SyntaxCharacterization,
};

pub(crate) fn scope_satisfied(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    coordinated: Option<&CoordinatedResidualStructure>,
    epsilon: f64,
) -> bool {
    let scope = &entry.scope_conditions;
    if !min_ok(
        syntax.outward_drift_fraction,
        scope.min_outward_drift_fraction,
        epsilon,
    ) {
        return false;
    }
    if !max_ok(
        syntax.outward_drift_fraction,
        scope.max_outward_drift_fraction,
        epsilon,
    ) {
        return false;
    }
    if !min_ok(
        syntax.inward_drift_fraction,
        scope.min_inward_drift_fraction,
        epsilon,
    ) {
        return false;
    }
    if !max_ok(
        syntax.inward_drift_fraction,
        scope.max_inward_drift_fraction,
        epsilon,
    ) {
        return false;
    }
    if !max_ok(
        syntax.mean_squared_slew_norm,
        scope.max_curvature_energy,
        epsilon,
    ) {
        return false;
    }
    if !min_ok(
        syntax.mean_squared_slew_norm,
        scope.min_curvature_energy,
        epsilon,
    ) {
        return false;
    }
    if !max_ok(
        syntax.late_slew_growth_score,
        scope.max_curvature_onset_score,
        epsilon,
    ) {
        return false;
    }
    if !min_ok(
        syntax.late_slew_growth_score,
        scope.min_curvature_onset_score,
        epsilon,
    ) {
        return false;
    }
    if !min_ok(
        syntax.radial_sign_persistence,
        scope.min_directional_persistence,
        epsilon,
    ) {
        return false;
    }
    if !min_ok(
        syntax.radial_sign_dominance,
        scope.min_sign_consistency,
        epsilon,
    ) {
        return false;
    }
    if !min_ok(
        syntax.drift_channel_sign_alignment,
        scope.min_channel_coherence,
        epsilon,
    ) {
        return false;
    }
    if !min_ok(
        syntax.residual_norm_path_monotonicity,
        scope.min_aggregate_monotonicity,
        epsilon,
    ) {
        return false;
    }
    if !max_ok(
        syntax.residual_norm_path_monotonicity,
        scope.max_aggregate_monotonicity,
        epsilon,
    ) {
        return false;
    }
    if !min_usize_ok(syntax.slew_spike_count, scope.min_slew_spike_count) {
        return false;
    }
    if !max_usize_ok(syntax.slew_spike_count, scope.max_slew_spike_count) {
        return false;
    }
    if !min_ok(
        syntax.slew_spike_strength,
        scope.min_slew_spike_strength,
        epsilon,
    ) {
        return false;
    }
    if !max_ok(
        syntax.slew_spike_strength,
        scope.max_slew_spike_strength,
        epsilon,
    ) {
        return false;
    }
    if !min_usize_ok(
        syntax.boundary_grazing_episode_count,
        scope.min_boundary_grazing_episodes,
    ) {
        return false;
    }
    if !max_usize_ok(
        syntax.boundary_grazing_episode_count,
        scope.max_boundary_grazing_episodes,
    ) {
        return false;
    }
    if !min_usize_ok(
        syntax.boundary_recovery_count,
        scope.min_boundary_recovery_count,
    ) {
        return false;
    }
    if !min_ok(
        syntax
            .coordinated_group_breach_fraction
            .max(coordinated_group_breach_ratio(coordinated)),
        scope.min_coordinated_group_breach_fraction,
        epsilon,
    ) {
        return false;
    }
    if !max_ok(
        syntax
            .coordinated_group_breach_fraction
            .max(coordinated_group_breach_ratio(coordinated)),
        scope.max_coordinated_group_breach_fraction,
        epsilon,
    ) {
        return false;
    }
    if scope.require_group_breach && !has_group_breach(coordinated) {
        return false;
    }
    true
}

fn has_group_breach(coordinated: Option<&CoordinatedResidualStructure>) -> bool {
    coordinated
        .map(|structure| {
            structure
                .points
                .iter()
                .any(|point| point.aggregate_margin < 0.0)
        })
        .unwrap_or(false)
}

fn coordinated_group_breach_ratio(coordinated: Option<&CoordinatedResidualStructure>) -> f64 {
    match coordinated {
        Some(structure) if !structure.points.is_empty() => {
            structure
                .points
                .iter()
                .filter(|point| point.aggregate_margin < 0.0)
                .count() as f64
                / structure.points.len() as f64
        }
        _ => 0.0,
    }
}

fn min_ok(value: f64, minimum: Option<f64>, epsilon: f64) -> bool {
    minimum
        .map(|minimum| value + epsilon >= minimum)
        .unwrap_or(true)
}

fn max_ok(value: f64, maximum: Option<f64>, epsilon: f64) -> bool {
    maximum
        .map(|maximum| value <= maximum + epsilon)
        .unwrap_or(true)
}

fn min_usize_ok(value: usize, minimum: Option<usize>) -> bool {
    minimum.map(|minimum| value >= minimum).unwrap_or(true)
}

fn max_usize_ok(value: usize, maximum: Option<usize>) -> bool {
    maximum.map(|maximum| value <= maximum).unwrap_or(true)
}
