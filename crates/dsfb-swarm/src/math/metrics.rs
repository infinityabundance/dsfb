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
    pub mode_shape_norms: &'a [f64],
    pub stack_scores: &'a [f64],
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
    pub baseline_state_lead_time: Option<f64>,
    pub baseline_disagreement_lead_time: Option<f64>,
    pub baseline_lambda2_lead_time: Option<f64>,
    pub multimode_minus_scalar_seconds: Option<f64>,
    pub trust_drop_step: Option<usize>,
    pub trust_suppression_delay: Option<f64>,
    pub scalar_false_positive_rate: f64,
    pub scalar_true_positive_rate: f64,
    pub multimode_false_positive_rate: f64,
    pub multimode_true_positive_rate: f64,
    pub max_abs_residual: f64,
    pub max_scalar_envelope: f64,
    pub max_combined_score: f64,
    pub peak_mode_shape_norm: f64,
    pub peak_stack_score: f64,
    pub lambda2_min: f64,
    pub lambda2_mean: f64,
    pub lambda2_final: f64,
    pub residual_topology_correlation: f64,
    pub residual_bound_ratio: f64,
    pub runtime_ms: f64,
}

pub fn summarize(input: MetricsInput<'_>) -> ScenarioSummary {
    let visible_failure_step = visible_failure_step(input.scenario, input.lambda2, input.onset_step);
    let scalar_detection_step = first_true_at_or_after(input.scalar_flags, input.onset_step);
    let multimode_detection_step = first_true_at_or_after(input.multimode_flags, input.onset_step);
    let baseline_state_detection_step = first_true_at_or_after(input.baseline_state_flags, input.onset_step);
    let baseline_disagreement_detection_step =
        first_true_at_or_after(input.baseline_disagreement_flags, input.onset_step);
    let baseline_lambda2_detection_step =
        first_true_at_or_after(input.baseline_lambda2_flags, input.onset_step);
    let trust_drop_step = trust_drop_step(input.affected_trust, input.onset_step);

    let scalar_detection_lead_time =
        lead_time_seconds(scalar_detection_step, visible_failure_step, input.dt);
    let multimode_detection_lead_time =
        lead_time_seconds(multimode_detection_step, visible_failure_step, input.dt);
    let baseline_state_lead_time =
        lead_time_seconds(baseline_state_detection_step, visible_failure_step, input.dt);
    let baseline_disagreement_lead_time =
        lead_time_seconds(baseline_disagreement_detection_step, visible_failure_step, input.dt);
    let baseline_lambda2_lead_time =
        lead_time_seconds(baseline_lambda2_detection_step, visible_failure_step, input.dt);
    let multimode_minus_scalar_seconds = match (scalar_detection_step, multimode_detection_step) {
        (Some(scalar), Some(multimode)) => Some((scalar as f64 - multimode as f64) * input.dt),
        _ => None,
    };
    let trust_suppression_delay = trust_drop_step.map(|step| ((step - input.onset_step) as f64) * input.dt);
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
    let peak_mode_shape_norm = input.mode_shape_norms.iter().copied().fold(0.0_f64, f64::max);
    let peak_stack_score = input.stack_scores.iter().copied().fold(0.0_f64, f64::max);
    let lambda2_min = input.lambda2.iter().copied().fold(f64::INFINITY, f64::min);
    let lambda2_mean = input.lambda2.iter().sum::<f64>() / input.lambda2.len().max(1) as f64;
    let lambda2_final = input.lambda2.last().copied().unwrap_or(0.0);
    let residual_topology_correlation = pearson_correlation_abs(input.combined_scores, input.laplacian_delta_norms);
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
        baseline_state_lead_time,
        baseline_disagreement_lead_time,
        baseline_lambda2_lead_time,
        multimode_minus_scalar_seconds,
        trust_drop_step,
        trust_suppression_delay,
        scalar_false_positive_rate: rate_before_onset(input.scalar_flags, input.onset_step),
        scalar_true_positive_rate: rate_after_onset(input.scalar_flags, input.onset_step),
        multimode_false_positive_rate: rate_before_onset(input.multimode_flags, input.onset_step),
        multimode_true_positive_rate: rate_after_onset(input.multimode_flags, input.onset_step),
        max_abs_residual,
        max_scalar_envelope,
        max_combined_score,
        peak_mode_shape_norm,
        peak_stack_score,
        lambda2_min,
        lambda2_mean,
        lambda2_final,
        residual_topology_correlation,
        residual_bound_ratio,
        runtime_ms: input.runtime_ms,
    }
}

fn visible_failure_step(kind: ScenarioKind, lambda2: &[f64], onset: usize) -> Option<usize> {
    if matches!(kind, ScenarioKind::Nominal) {
        return None;
    }
    let baseline = pre_onset_baseline(lambda2, onset);
    let warmup_mean = baseline.iter().sum::<f64>() / baseline.len().max(1) as f64;
    let threshold = match kind {
        ScenarioKind::GradualEdgeDegradation => (0.70 * warmup_mean).max(0.02),
        ScenarioKind::AdversarialAgent => (0.68 * warmup_mean).max(0.02),
        ScenarioKind::CommunicationLoss => (0.45 * warmup_mean).max(0.015),
        ScenarioKind::Nominal | ScenarioKind::All => (0.45 * warmup_mean).max(0.015),
    };
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

fn trust_drop_step(values: &[f64], onset: usize) -> Option<usize> {
    if onset == 0 || values.is_empty() {
        return None;
    }
    let baseline = pre_onset_baseline(values, onset);
    let baseline_mean = baseline.iter().sum::<f64>() / baseline.len() as f64;
    let baseline_std = (baseline
        .iter()
        .map(|value| {
            let delta = value - baseline_mean;
            delta * delta
        })
        .sum::<f64>()
        / baseline.len() as f64)
        .sqrt();
    let threshold = (baseline_mean - 2.5 * baseline_std).min(0.80 * baseline_mean);
    let mut consecutive = 0usize;
    for (index, value) in values.iter().enumerate().skip(onset) {
        if *value < threshold {
            consecutive += 1;
            if consecutive >= 3 {
                return Some(index);
            }
        } else {
            consecutive = 0;
        }
    }
    None
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

fn pre_onset_baseline<'a>(values: &'a [f64], onset: usize) -> &'a [f64] {
    let end = onset.min(values.len());
    if end == 0 {
        return &values[..values.len().min(1)];
    }
    let window = end.clamp(12, 24);
    let start = end.saturating_sub(window);
    &values[start..end]
}
