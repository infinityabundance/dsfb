use crate::engine::types::{GrammarState, GrammarStatus, SignTrajectory, SyntaxCharacterization};
use crate::math::metrics::{
    channel_sign_coherence, dominant_sign_fraction, mean, monotone_alignment_fraction,
    monotonicity_score, persistence_fraction, radial_drift, scalar_derivative, sign_with_deadband,
    standard_deviation, episode_count,
};

pub fn characterize_syntax(
    sign: &SignTrajectory,
    grammar: &[GrammarStatus],
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
        .map(|sample| channel_sign_coherence(&sample.drift, 1.0e-6))
        .collect::<Vec<_>>();
    let slew_norms = sign
        .samples
        .iter()
        .map(|sample| sample.slew_norm)
        .collect::<Vec<_>>();
    let margins = grammar.iter().map(|status| status.margin).collect::<Vec<_>>();
    let margin_rates = scalar_derivative(&margins, &times);
    let min_margin = margins.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let mean_margin_delta = mean(&margin_rates);
    let outward_count = margin_rates
        .iter()
        .zip(&radial_drifts)
        .filter(|(margin_rate, radial)| **margin_rate < -1.0e-6 || (margin_rate.abs() <= 1.0e-6 && **radial > 1.0e-6))
        .count();
    let inward_count = margin_rates
        .iter()
        .zip(&radial_drifts)
        .filter(|(margin_rate, radial)| **margin_rate > 1.0e-6 || (margin_rate.abs() <= 1.0e-6 && **radial < -1.0e-6))
        .count();
    let slew_mean = mean(&slew_norms);
    let slew_threshold = (slew_mean + 1.5 * standard_deviation(&slew_norms)).max(1.0e-4);
    let boundary_grazing_episode_count = episode_count(
        &grammar
            .iter()
            .map(|status| matches!(status.state, GrammarState::Boundary))
            .collect::<Vec<_>>(),
    );
    let repeated_grazing_count = boundary_grazing_episode_count.saturating_sub(1);
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
    let sign_consistency = dominant_sign_fraction(&radial_signs);
    let directional_persistence = persistence_fraction(&radial_signs);
    let channel_coherence = mean(&channel_coherences);
    let aggregate_monotonicity = monotonicity_score(&residual_norms);
    let monotone_drift_fraction = monotone_alignment_fraction(&residual_norms, 1.0e-6);
    let curvature_energy = if slew_norms.is_empty() {
        0.0
    } else {
        slew_norms.iter().map(|value| value * value).sum::<f64>() / slew_norms.len() as f64
    };
    let max_slew_norm = slew_norms.iter().copied().fold(0.0, f64::max);
    let slew_spike_count = slew_norms
        .iter()
        .filter(|value| **value > slew_threshold)
        .count();
    let mean_radial_drift = mean(&radial_drifts);

    let trajectory_label = if outward_drift_fraction > 0.68
        && aggregate_monotonicity > 0.74
        && directional_persistence > 0.7
        && curvature_energy < 0.02
    {
        "persistent-outward-drift".to_string()
    } else if inward_drift_fraction > 0.6 && min_margin > 0.0 && mean_radial_drift <= 0.0 {
        "inward-compatible-containment".to_string()
    } else if (curvature_energy > 0.025 || max_slew_norm > 0.2)
        && slew_spike_count > 0
        && aggregate_monotonicity < 0.85
    {
        "curvature-rich-or-event-like".to_string()
    } else if boundary_grazing_episode_count >= 3 && violation_count == 0 {
        "near-boundary-recurrent".to_string()
    } else {
        "mixed-structured".to_string()
    };

    SyntaxCharacterization {
        scenario_id: sign.scenario_id.clone(),
        outward_drift_fraction,
        inward_drift_fraction,
        sign_consistency,
        directional_persistence,
        channel_coherence,
        aggregate_monotonicity,
        monotone_drift_fraction,
        curvature_energy,
        mean_radial_drift,
        min_margin,
        mean_margin_delta,
        max_slew_norm,
        slew_spike_count,
        boundary_grazing_episode_count,
        repeated_grazing_count,
        trajectory_label,
    }
}
