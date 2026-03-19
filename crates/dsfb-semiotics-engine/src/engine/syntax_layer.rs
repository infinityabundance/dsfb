use crate::engine::types::{GrammarState, GrammarStatus, SignTrajectory, SyntaxCharacterization};
use crate::math::metrics::{count_where, mean};

pub fn characterize_syntax(
    sign: &SignTrajectory,
    grammar: &[GrammarStatus],
) -> SyntaxCharacterization {
    let drift_values = sign
        .samples
        .iter()
        .map(|sample| sample.drift.first().copied().unwrap_or_default())
        .collect::<Vec<_>>();
    let slew_norms = sign
        .samples
        .iter()
        .map(|sample| sample.slew_norm)
        .collect::<Vec<_>>();
    let positive = count_where(&drift_values, |value| value > 0.0);
    let negative = count_where(&drift_values, |value| value < 0.0);
    let slew_mean = mean(&slew_norms);
    let slew_threshold = (slew_mean * 2.2).max(0.08);
    let repeated_grazing_count = grammar
        .windows(2)
        .filter(|window| {
            matches!(window[1].state, GrammarState::Boundary)
                && !matches!(window[0].state, GrammarState::Violation)
        })
        .count();

    let monotone_drift_fraction = if drift_values.is_empty() {
        0.0
    } else {
        positive as f64 / drift_values.len() as f64
    };
    let outward_drift_fraction = monotone_drift_fraction;
    let inward_drift_fraction = if drift_values.is_empty() {
        0.0
    } else {
        negative as f64 / drift_values.len() as f64
    };
    let curvature_energy = if slew_norms.is_empty() {
        0.0
    } else {
        slew_norms.iter().map(|value| value * value).sum::<f64>() / slew_norms.len() as f64
    };
    let max_slew_norm = slew_norms.iter().copied().fold(0.0, f64::max);
    let slew_spike_count = count_where(&slew_norms, |value| value > slew_threshold);

    let trajectory_label = if monotone_drift_fraction > 0.8 && max_slew_norm < 0.09 {
        "monotone-drift-dominated".to_string()
    } else if max_slew_norm > 0.25 && slew_spike_count > 0 {
        "curvature-or-event-dominated".to_string()
    } else if repeated_grazing_count >= 3 {
        "near-boundary-recurrent".to_string()
    } else {
        "mixed-structured".to_string()
    };

    SyntaxCharacterization {
        scenario_id: sign.scenario_id.clone(),
        outward_drift_fraction,
        inward_drift_fraction,
        monotone_drift_fraction,
        curvature_energy,
        max_slew_norm,
        slew_spike_count,
        repeated_grazing_count,
        trajectory_label,
    }
}
