use crate::engine::types::{
    CoordinatedResidualStructure, GrammarState, GrammarStatus, SignTrajectory,
    SyntaxCharacterization,
};
use crate::math::metrics::{
    adjacent_sign_agreement_fraction, dominant_nonzero_sign_fraction, episode_count,
    late_slew_growth_score, mean, positive_excess_strength, radial_drift, recovery_count,
    residual_norm_path_monotonicity, scalar_derivative, sign_with_deadband, standard_deviation,
    trend_aligned_increment_fraction, within_sample_sign_alignment,
};

pub fn characterize_syntax(
    sign: &SignTrajectory,
    grammar: &[GrammarStatus],
) -> SyntaxCharacterization {
    characterize_syntax_with_coordination(sign, grammar, None)
}

pub fn characterize_syntax_with_coordination(
    sign: &SignTrajectory,
    grammar: &[GrammarStatus],
    coordinated: Option<&CoordinatedResidualStructure>,
) -> SyntaxCharacterization {
    let times = sign
        .samples
        .iter()
        .map(|sample| sample.time)
        .collect::<Vec<_>>();
    let residual_norms = sign
        .samples
        .iter()
        .map(|sample| sample.residual_norm)
        .collect::<Vec<_>>();
    let residual_norm_rates = scalar_derivative(&residual_norms, &times);
    let radial_drifts = sign
        .samples
        .iter()
        .map(|sample| radial_drift(&sample.residual, &sample.drift))
        .collect::<Vec<_>>();
    let radial_signs = radial_drifts
        .iter()
        .map(|value| sign_with_deadband(*value, 1.0e-6))
        .collect::<Vec<_>>();
    let channel_coherences = sign
        .samples
        .iter()
        .map(|sample| within_sample_sign_alignment(&sample.drift, 1.0e-6))
        .collect::<Vec<_>>();
    let slew_norms = sign
        .samples
        .iter()
        .map(|sample| sample.slew_norm)
        .collect::<Vec<_>>();
    let margins = grammar
        .iter()
        .map(|status| status.margin)
        .collect::<Vec<_>>();
    let margin_rates = scalar_derivative(&margins, &times);
    let min_margin = margins.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let mean_margin_delta = mean(&margin_rates);
    let outward_count = margin_rates
        .iter()
        .zip(&residual_norm_rates)
        .zip(&radial_drifts)
        .filter(|((margin_rate, residual_rate), radial)| {
            **margin_rate < -1.0e-6
                || (**residual_rate > 1.0e-6 && **radial > 1.0e-6)
                || (margin_rate.abs() <= 1.0e-6 && **radial > 1.0e-6)
        })
        .count();
    let inward_count = margin_rates
        .iter()
        .zip(&residual_norm_rates)
        .zip(&radial_drifts)
        .filter(|((margin_rate, residual_rate), radial)| {
            **margin_rate > 1.0e-6
                || (**residual_rate < -1.0e-6 && **radial < -1.0e-6)
                || (margin_rate.abs() <= 1.0e-6 && **radial < -1.0e-6)
        })
        .count();
    let slew_mean = mean(&slew_norms);
    let slew_threshold = (slew_mean + 1.5 * standard_deviation(&slew_norms)).max(1.0e-4);
    let boundary_flags = grammar
        .iter()
        .map(|status| matches!(status.state, GrammarState::Boundary))
        .collect::<Vec<_>>();
    let non_admissible_flags = grammar
        .iter()
        .map(|status| !matches!(status.state, GrammarState::Admissible))
        .collect::<Vec<_>>();
    let boundary_grazing_episode_count = episode_count(&boundary_flags);
    let boundary_recovery_count = recovery_count(&non_admissible_flags);
    let repeated_grazing_count = boundary_grazing_episode_count.saturating_sub(1);
    let coordinated_group_breach_fraction = coordinated_group_breach_fraction(coordinated);
    let violation_count = grammar
        .iter()
        .filter(|status| matches!(status.state, GrammarState::Violation))
        .count();

    let sample_count = sign.samples.len();
    let outward_drift_fraction = if sample_count == 0 {
        0.0
    } else {
        outward_count as f64 / sample_count as f64
    };
    let inward_drift_fraction = if sample_count == 0 {
        0.0
    } else {
        inward_count as f64 / sample_count as f64
    };
    let radial_sign_dominance = dominant_nonzero_sign_fraction(&radial_signs);
    let radial_sign_persistence = adjacent_sign_agreement_fraction(&radial_signs);
    let drift_channel_sign_alignment = mean(&channel_coherences);
    let residual_norm_path_monotonicity = residual_norm_path_monotonicity(&residual_norms);
    let residual_norm_trend_alignment = trend_aligned_increment_fraction(&residual_norms, 1.0e-6);
    let mean_squared_slew_norm = if slew_norms.is_empty() {
        0.0
    } else {
        slew_norms.iter().map(|value| value * value).sum::<f64>() / slew_norms.len() as f64
    };
    let late_slew_growth_score = late_slew_growth_score(&slew_norms);
    let max_slew_norm = slew_norms.iter().copied().fold(0.0, f64::max);
    let slew_spike_count = slew_norms
        .iter()
        .filter(|value| **value > slew_threshold)
        .count();
    let slew_spike_strength = if slew_norms.is_empty() {
        0.0
    } else {
        positive_excess_strength(&slew_norms, slew_threshold) / slew_norms.len() as f64
    };
    let mean_radial_drift = mean(&radial_drifts);
    let outward_inward_imbalance = (outward_drift_fraction - inward_drift_fraction).abs();

    // This low-structure branch is intentionally conservative. It surfaces trajectories whose
    // sampled residual evolution remains admissible, nearly balanced between outward and inward
    // motion, and only weakly structured under the current deterministic summaries. It is not a
    // health classifier.
    let baseline_like_structure = coordinated_group_breach_fraction == 0.0
        && violation_count == 0
        && boundary_grazing_episode_count == 0
        && outward_inward_imbalance < 0.08
        && residual_norm_path_monotonicity < 0.08
        && mean_squared_slew_norm < 1.0e-5
        && max_slew_norm < 0.002
        && late_slew_growth_score < 0.20
        && slew_spike_strength < 1.0e-3;

    let trajectory_label = if coordinated_group_breach_fraction > 0.08
        && outward_drift_fraction > 0.45
        && drift_channel_sign_alignment > 0.55
        && radial_sign_persistence > 0.45
    {
        "coordinated-outward-rise".to_string()
    } else if outward_drift_fraction > 0.68
        && residual_norm_path_monotonicity > 0.74
        && radial_sign_persistence > 0.7
        && mean_squared_slew_norm < 0.02
        && late_slew_growth_score < 0.35
    {
        "persistent-outward-drift".to_string()
    } else if inward_drift_fraction > 0.6 && min_margin > 0.0 && mean_radial_drift <= 0.0 {
        "inward-compatible-containment".to_string()
    } else if slew_spike_count > 0
        && (slew_spike_strength > 0.02 || max_slew_norm > 0.01)
        && late_slew_growth_score > 0.4
    {
        "discrete-event-like".to_string()
    } else if late_slew_growth_score > 0.45
        || (mean_squared_slew_norm > 0.02 && max_slew_norm > 0.01)
        || (slew_spike_count > 0 && slew_spike_strength > 0.015 && max_slew_norm > 0.005)
    {
        "curvature-rich-transition".to_string()
    } else if boundary_grazing_episode_count >= 3 && violation_count == 0 {
        "near-boundary-recurrent".to_string()
    } else if baseline_like_structure {
        "weakly-structured-baseline-like".to_string()
    } else {
        "mixed-structured".to_string()
    };

    SyntaxCharacterization {
        scenario_id: sign.scenario_id.clone(),
        outward_drift_fraction,
        inward_drift_fraction,
        sign_consistency: radial_sign_dominance,
        directional_persistence: radial_sign_persistence,
        channel_coherence: drift_channel_sign_alignment,
        aggregate_monotonicity: residual_norm_path_monotonicity,
        monotone_drift_fraction: residual_norm_trend_alignment,
        curvature_energy: mean_squared_slew_norm,
        curvature_onset_score: late_slew_growth_score,
        radial_sign_dominance,
        radial_sign_persistence,
        drift_channel_sign_alignment,
        residual_norm_path_monotonicity,
        residual_norm_trend_alignment,
        mean_squared_slew_norm,
        late_slew_growth_score,
        mean_radial_drift,
        min_margin,
        mean_margin_delta,
        max_slew_norm,
        slew_spike_count,
        slew_spike_strength,
        boundary_grazing_episode_count,
        boundary_recovery_count,
        repeated_grazing_count,
        coordinated_group_breach_fraction,
        trajectory_label,
    }
}

fn coordinated_group_breach_fraction(coordinated: Option<&CoordinatedResidualStructure>) -> f64 {
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
