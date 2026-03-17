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
    pub best_baseline_name: String,
    pub best_baseline_lead_time: Option<f64>,
    pub best_baseline_true_positive_rate: Option<f64>,
    pub best_baseline_false_positive_rate: Option<f64>,
    pub lead_time_gain_vs_best_baseline: Option<f64>,
    pub tpr_gain_vs_best_baseline: Option<f64>,
    pub fpr_delta_vs_best_baseline: Option<f64>,
    pub fpr_reduction_vs_best_baseline: Option<f64>,
    pub dsfb_advantage_margin: Option<f64>,
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
    let visible_failure_step =
        visible_failure_step(input.scenario, input.lambda2, input.onset_step);
    let scalar_detection_step = first_true_at_or_after(input.scalar_flags, input.onset_step);
    let multimode_detection_step = first_true_at_or_after(input.multimode_flags, input.onset_step);
    let baseline_state_detection_step =
        first_true_at_or_after(input.baseline_state_flags, input.onset_step);
    let baseline_disagreement_detection_step =
        first_true_at_or_after(input.baseline_disagreement_flags, input.onset_step);
    let baseline_lambda2_detection_step =
        first_true_at_or_after(input.baseline_lambda2_flags, input.onset_step);
    let trust_drop_step = trust_drop_step(input.affected_trust, input.onset_step);

    let scalar_detection_lead_time =
        lead_time_seconds(scalar_detection_step, visible_failure_step, input.dt);
    let multimode_detection_lead_time =
        lead_time_seconds(multimode_detection_step, visible_failure_step, input.dt);
    let baseline_state_lead_time = lead_time_seconds(
        baseline_state_detection_step,
        visible_failure_step,
        input.dt,
    );
    let baseline_disagreement_lead_time = lead_time_seconds(
        baseline_disagreement_detection_step,
        visible_failure_step,
        input.dt,
    );
    let baseline_lambda2_lead_time = lead_time_seconds(
        baseline_lambda2_detection_step,
        visible_failure_step,
        input.dt,
    );
    let baseline_state_tpr = rate_after_onset(input.baseline_state_flags, input.onset_step);
    let baseline_state_fpr = rate_before_onset(input.baseline_state_flags, input.onset_step);
    let baseline_disagreement_tpr =
        rate_after_onset(input.baseline_disagreement_flags, input.onset_step);
    let baseline_disagreement_fpr =
        rate_before_onset(input.baseline_disagreement_flags, input.onset_step);
    let baseline_lambda2_tpr = rate_after_onset(input.baseline_lambda2_flags, input.onset_step);
    let baseline_lambda2_fpr = rate_before_onset(input.baseline_lambda2_flags, input.onset_step);
    let best_baseline = select_best_baseline([
        (
            "state_norm",
            baseline_state_lead_time,
            baseline_state_tpr,
            baseline_state_fpr,
        ),
        (
            "disagreement_energy",
            baseline_disagreement_lead_time,
            baseline_disagreement_tpr,
            baseline_disagreement_fpr,
        ),
        (
            "raw_lambda2",
            baseline_lambda2_lead_time,
            baseline_lambda2_tpr,
            baseline_lambda2_fpr,
        ),
    ]);
    let best_detector_lead =
        best_available(scalar_detection_lead_time, multimode_detection_lead_time);
    let scalar_false_positive_rate = rate_before_onset(input.scalar_flags, input.onset_step);
    let scalar_true_positive_rate = rate_after_onset(input.scalar_flags, input.onset_step);
    let multimode_false_positive_rate = rate_before_onset(input.multimode_flags, input.onset_step);
    let multimode_true_positive_rate = rate_after_onset(input.multimode_flags, input.onset_step);
    let best_detector_tpr = scalar_true_positive_rate.max(multimode_true_positive_rate);
    let best_detector_fpr = scalar_false_positive_rate.min(multimode_false_positive_rate);
    let lead_time_gain_vs_best_baseline = best_baseline
        .as_ref()
        .and_then(|(_, lead, _, _)| best_detector_lead.map(|detector| detector - *lead));
    let tpr_gain_vs_best_baseline = best_baseline
        .as_ref()
        .map(|(_, _, baseline_tpr, _)| best_detector_tpr - *baseline_tpr);
    let fpr_delta_vs_best_baseline = best_baseline
        .as_ref()
        .map(|(_, _, _, baseline_fpr)| baseline_fpr - best_detector_fpr);
    let fpr_reduction_vs_best_baseline = best_baseline
        .as_ref()
        .map(|(_, _, _, baseline_fpr)| baseline_fpr - best_detector_fpr);
    let dsfb_advantage_margin =
        best_baseline
            .as_ref()
            .and_then(|(_, lead, baseline_tpr, baseline_fpr)| {
                best_detector_lead.map(|detector| {
                    let lead_gain = detector - *lead;
                    let tpr_bonus = 0.35 * (best_detector_tpr - *baseline_tpr).max(0.0);
                    let fpr_penalty = 1.25 * (best_detector_fpr - *baseline_fpr).max(0.0);
                    lead_gain + tpr_bonus - fpr_penalty
                })
            });
    let multimode_minus_scalar_seconds = match (scalar_detection_step, multimode_detection_step) {
        (Some(scalar), Some(multimode)) => Some((scalar as f64 - multimode as f64) * input.dt),
        _ => None,
    };
    let trust_suppression_delay =
        trust_drop_step.map(|step| ((step - input.onset_step) as f64) * input.dt);
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
    let peak_mode_shape_norm = input
        .mode_shape_norms
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let peak_stack_score = input.stack_scores.iter().copied().fold(0.0_f64, f64::max);
    let lambda2_min = input.lambda2.iter().copied().fold(f64::INFINITY, f64::min);
    let lambda2_mean = input.lambda2.iter().sum::<f64>() / input.lambda2.len().max(1) as f64;
    let lambda2_final = input.lambda2.last().copied().unwrap_or(0.0);
    let residual_topology_correlation =
        pearson_correlation_abs(input.combined_scores, input.laplacian_delta_norms);
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
        best_baseline_name: best_baseline
            .as_ref()
            .map(|(name, _, _, _)| (*name).to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        best_baseline_lead_time: best_baseline.as_ref().map(|(_, lead, _, _)| *lead),
        best_baseline_true_positive_rate: best_baseline.as_ref().map(|(_, _, tpr, _)| *tpr),
        best_baseline_false_positive_rate: best_baseline.as_ref().map(|(_, _, _, fpr)| *fpr),
        lead_time_gain_vs_best_baseline,
        tpr_gain_vs_best_baseline,
        fpr_delta_vs_best_baseline,
        fpr_reduction_vs_best_baseline,
        dsfb_advantage_margin,
        multimode_minus_scalar_seconds,
        trust_drop_step,
        trust_suppression_delay,
        scalar_false_positive_rate,
        scalar_true_positive_rate,
        multimode_false_positive_rate,
        multimode_true_positive_rate,
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
    match kind {
        ScenarioKind::Nominal => None,
        ScenarioKind::GradualEdgeDegradation => {
            let (peak_step, peak_value) = post_onset_peak(lambda2, onset)?;
            let threshold = (0.30 * peak_value).max(0.02);
            first_sustained_below(lambda2, peak_step.saturating_add(1), threshold, 6)
        }
        ScenarioKind::AdversarialAgent => None,
        ScenarioKind::CommunicationLoss | ScenarioKind::All => {
            let baseline = pre_onset_baseline(lambda2, onset);
            let warmup_mean = baseline.iter().sum::<f64>() / baseline.len().max(1) as f64;
            let threshold = (0.50 * warmup_mean).max(0.015);
            first_sustained_below(lambda2, onset, threshold, 3)
        }
    }
}

fn first_true_at_or_after(flags: &[bool], start: usize) -> Option<usize> {
    flags
        .iter()
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
        (Some(detection), Some(failure)) => Some((failure as f64 - detection as f64) * dt),
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

fn first_sustained_below(
    values: &[f64],
    start: usize,
    threshold: f64,
    persistence: usize,
) -> Option<usize> {
    let mut count = 0usize;
    for (step, value) in values.iter().enumerate().skip(start) {
        if *value < threshold {
            count += 1;
            if count >= persistence {
                return Some(step + 1 - persistence);
            }
        } else {
            count = 0;
        }
    }
    None
}

fn post_onset_peak(values: &[f64], onset: usize) -> Option<(usize, f64)> {
    values
        .iter()
        .enumerate()
        .skip(onset)
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map(|(index, value)| (index, *value))
}

fn best_available(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn select_best_baseline<const N: usize>(
    candidates: [(&'static str, Option<f64>, f64, f64); N],
) -> Option<(&'static str, f64, f64, f64)> {
    candidates
        .into_iter()
        .filter_map(|(name, lead_time, tpr, fpr)| lead_time.map(|lead| (name, lead, tpr, fpr)))
        .max_by(|left, right| baseline_rank(left).total_cmp(&baseline_rank(right)))
}

fn baseline_rank(candidate: &(&'static str, f64, f64, f64)) -> f64 {
    let (_, lead_time, tpr, fpr) = *candidate;
    lead_time + 0.30 * tpr - 0.60 * fpr
}
