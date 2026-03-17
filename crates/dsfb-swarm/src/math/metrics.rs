use serde::Serialize;

use crate::config::ScenarioKind;

#[derive(Debug, Clone)]
pub struct MetricsInput<'a> {
    pub scenario: ScenarioKind,
    pub scenario_name: &'a str,
    pub agents: usize,
    pub steps: usize,
    pub dt: f64,
    pub noise_level: f64,
    pub onset_step: usize,
    pub lambda2: &'a [f64],
    pub scalar_flags: &'a [bool],
    pub multimode_flags: &'a [bool],
    pub baseline_state_flags: &'a [bool],
    pub baseline_disagreement_flags: &'a [bool],
    pub baseline_lambda2_flags: &'a [bool],
    pub affected_trust: &'a [f64],
    pub scalar_residuals: &'a [f64],
    pub scalar_envelopes: &'a [f64],
    pub combined_scores: &'a [f64],
    pub laplacian_delta_norms: &'a [f64],
    pub runtime_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioSummary {
    pub scenario: String,
    pub scenario_kind: String,
    pub agents: usize,
    pub steps: usize,
    pub noise_level: f64,
    pub onset_step: usize,
    pub visible_failure_step: Option<usize>,
    pub scalar_detection_step: Option<usize>,
    pub multimode_detection_step: Option<usize>,
    pub baseline_state_detection_step: Option<usize>,
    pub baseline_disagreement_detection_step: Option<usize>,
    pub baseline_lambda2_detection_step: Option<usize>,
    pub scalar_detection_lead_time: Option<f64>,
    pub multimode_detection_lead_time: Option<f64>,
    pub trust_suppression_delay: Option<f64>,
    pub scalar_false_positive_rate: f64,
    pub scalar_true_positive_rate: f64,
    pub multimode_false_positive_rate: f64,
    pub multimode_true_positive_rate: f64,
    pub max_abs_residual: f64,
    pub max_scalar_envelope: f64,
    pub max_combined_score: f64,
    pub lambda2_min: f64,
    pub lambda2_mean: f64,
    pub lambda2_final: f64,
    pub residual_topology_correlation: f64,
    pub residual_bound_ratio: f64,
    pub runtime_ms: f64,
}

pub fn summarize(input: MetricsInput<'_>) -> ScenarioSummary {
    let visible_failure_step = visible_failure_step(input.lambda2, input.onset_step);
    let scalar_detection_step = first_true_at_or_after(input.scalar_flags, input.onset_step);
    let multimode_detection_step = first_true_at_or_after(input.multimode_flags, input.onset_step);
    let baseline_state_detection_step = first_true_at_or_after(input.baseline_state_flags, input.onset_step);
    let baseline_disagreement_detection_step =
        first_true_at_or_after(input.baseline_disagreement_flags, input.onset_step);
    let baseline_lambda2_detection_step =
        first_true_at_or_after(input.baseline_lambda2_flags, input.onset_step);

    let scalar_detection_lead_time =
        lead_time_seconds(scalar_detection_step, visible_failure_step, input.dt);
    let multimode_detection_lead_time =
        lead_time_seconds(multimode_detection_step, visible_failure_step, input.dt);
    let trust_suppression_delay = first_below_threshold_at_or_after(input.affected_trust, input.onset_step, 0.55)
        .map(|step| ((step - input.onset_step) as f64) * input.dt);
    let max_abs_residual = input
        .scalar_residuals
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    let max_scalar_envelope = input
        .scalar_envelopes
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let max_combined_score = input
        .combined_scores
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let lambda2_min = input.lambda2.iter().copied().fold(f64::INFINITY, f64::min);
    let lambda2_mean = input.lambda2.iter().sum::<f64>() / input.lambda2.len().max(1) as f64;
    let lambda2_final = input.lambda2.last().copied().unwrap_or(0.0);
    let residual_topology_correlation = pearson_correlation_abs(input.scalar_residuals, input.laplacian_delta_norms);
    let residual_bound_ratio = if max_scalar_envelope > 0.0 {
        max_abs_residual / max_scalar_envelope
    } else {
        0.0
    };

    ScenarioSummary {
        scenario: input.scenario_name.to_string(),
        scenario_kind: input.scenario.as_str().to_string(),
        agents: input.agents,
        steps: input.steps,
        noise_level: input.noise_level,
        onset_step: input.onset_step,
        visible_failure_step,
        scalar_detection_step,
        multimode_detection_step,
        baseline_state_detection_step,
        baseline_disagreement_detection_step,
        baseline_lambda2_detection_step,
        scalar_detection_lead_time,
        multimode_detection_lead_time,
        trust_suppression_delay,
        scalar_false_positive_rate: rate_before_onset(input.scalar_flags, input.onset_step),
        scalar_true_positive_rate: rate_after_onset(input.scalar_flags, input.onset_step),
        multimode_false_positive_rate: rate_before_onset(input.multimode_flags, input.onset_step),
        multimode_true_positive_rate: rate_after_onset(input.multimode_flags, input.onset_step),
        max_abs_residual,
        max_scalar_envelope,
        max_combined_score,
        lambda2_min,
        lambda2_mean,
        lambda2_final,
        residual_topology_correlation,
        residual_bound_ratio,
        runtime_ms: input.runtime_ms,
    }
}

fn visible_failure_step(lambda2: &[f64], onset: usize) -> Option<usize> {
    let warmup_mean = lambda2.iter().take(onset.max(1)).sum::<f64>() / onset.max(1) as f64;
    let threshold = (0.45 * warmup_mean).max(0.015);
    lambda2
        .iter()
        .enumerate()
        .skip(onset)
        .find_map(|(step, value)| (*value < threshold).then_some(step))
}

fn first_true_at_or_after(flags: &[bool], start: usize) -> Option<usize> {
    flags.iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, flag)| (*flag).then_some(index))
}

fn first_below_threshold_at_or_after(values: &[f64], start: usize, threshold: f64) -> Option<usize> {
    values
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, value)| (*value < threshold).then_some(index))
}

fn lead_time_seconds(detection: Option<usize>, failure: Option<usize>, dt: f64) -> Option<f64> {
    match (detection, failure) {
        (Some(detection), Some(failure)) if detection <= failure => Some((failure - detection) as f64 * dt),
        _ => None,
    }
}

fn rate_before_onset(flags: &[bool], onset: usize) -> f64 {
    if onset == 0 {
        return 0.0;
    }
    flags.iter().take(onset).filter(|flag| **flag).count() as f64 / onset as f64
}

fn rate_after_onset(flags: &[bool], onset: usize) -> f64 {
    let post = flags.len().saturating_sub(onset);
    if post == 0 {
        return 0.0;
    }
    flags.iter().skip(onset).filter(|flag| **flag).count() as f64 / post as f64
}

fn pearson_correlation_abs(left: &[f64], right: &[f64]) -> f64 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let left = left.iter().map(|value| value.abs()).collect::<Vec<_>>();
    let right = right.to_vec();
    let mean_left = left.iter().sum::<f64>() / left.len() as f64;
    let mean_right = right.iter().sum::<f64>() / right.len() as f64;
    let mut numerator = 0.0;
    let mut left_denom = 0.0;
    let mut right_denom = 0.0;
    for (left_value, right_value) in left.iter().zip(right.iter()) {
        let dl = left_value - mean_left;
        let dr = right_value - mean_right;
        numerator += dl * dr;
        left_denom += dl * dl;
        right_denom += dr * dr;
    }
    if left_denom <= 1.0e-12 || right_denom <= 1.0e-12 {
        0.0
    } else {
        numerator / (left_denom.sqrt() * right_denom.sqrt())
    }
}
