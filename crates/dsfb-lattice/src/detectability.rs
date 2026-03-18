use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct EnvelopeParameters {
    pub baseline_runs: usize,
    pub sigma_multiplier: f64,
    pub additive_floor: f64,
    pub smoothing: String,
    pub interpolation: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct EnvelopeProvenance {
    pub regime_label: String,
    pub envelope_mode: String,
    pub envelope_basis: String,
    pub envelope_universal: bool,
    pub baseline_reference_residual_peak: f64,
    pub baseline_ensemble_peak: f64,
    pub baseline_reference_signal_peak: f64,
    pub baseline_reference_signal_energy: f64,
    pub parameters: EnvelopeParameters,
}

#[derive(Clone, Debug, Serialize)]
pub struct Envelope {
    pub mean: Vec<f64>,
    pub std: Vec<f64>,
    pub max_baseline: Vec<f64>,
    pub upper: Vec<f64>,
    pub provenance: EnvelopeProvenance,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CrossingRegimeLabel {
    Clean,
    Stressed,
    Combined,
}

impl CrossingRegimeLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Stressed => "stressed",
            Self::Combined => "combined",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DetectabilityInterpretationClass {
    StructuralDetected,
    StressDetected,
    EarlyLowMarginCrossing,
    NotDetected,
}

impl DetectabilityInterpretationClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StructuralDetected => "structural_detected",
            Self::StressDetected => "stress_detected",
            Self::EarlyLowMarginCrossing => "early_low_margin_crossing",
            Self::NotDetected => "not_detected",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectabilityInterpretationSettings {
    pub persistence_window_steps: usize,
    pub early_crossing_fraction_threshold: f64,
    pub low_margin_threshold: f64,
    pub structural_margin_threshold: f64,
    pub structural_post_crossing_fraction_threshold: f64,
}

impl Default for DetectabilityInterpretationSettings {
    fn default() -> Self {
        Self {
            persistence_window_steps: 12,
            early_crossing_fraction_threshold: 0.12,
            low_margin_threshold: 0.08,
            structural_margin_threshold: 0.15,
            structural_post_crossing_fraction_threshold: 0.60,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DetectabilitySummary {
    pub global_signal_peak: f64,
    pub global_envelope_peak: f64,
    pub global_signal_peak_time: f64,
    pub global_envelope_peak_time: f64,
    pub crossing_regime_label: CrossingRegimeLabel,
    pub first_crossing_time: Option<f64>,
    pub first_crossing_step: Option<usize>,
    pub signal_at_first_crossing: Option<f64>,
    pub envelope_at_first_crossing: Option<f64>,
    pub crossing_margin: Option<f64>,
    pub normalized_crossing_margin: Option<f64>,
    pub consecutive_crossing_time: Option<f64>,
    pub consecutive_crossing_step: Option<usize>,
    pub post_crossing_persistence_duration: Option<f64>,
    pub post_crossing_fraction: Option<f64>,
    pub peak_margin_after_crossing: Option<f64>,
    pub interpretation_class: DetectabilityInterpretationClass,
    pub interpretation_note: String,
}

pub fn build_envelope(
    baseline_norms: &[Vec<f64>],
    sigma_multiplier: f64,
    floor: f64,
    regime_label: &str,
    baseline_reference_residual_peak: f64,
    baseline_reference_signal_peak: f64,
    baseline_reference_signal_energy: f64,
) -> Envelope {
    if baseline_norms.is_empty() {
        return Envelope {
            mean: Vec::new(),
            std: Vec::new(),
            max_baseline: Vec::new(),
            upper: Vec::new(),
            provenance: build_provenance(
                regime_label,
                baseline_norms.len(),
                sigma_multiplier,
                floor,
                baseline_reference_residual_peak,
                0.0,
                baseline_reference_signal_peak,
                baseline_reference_signal_energy,
            ),
        };
    }

    let steps = baseline_norms[0].len();
    let mut mean = vec![0.0; steps];
    let mut std = vec![0.0; steps];
    let mut max_baseline = vec![0.0; steps];
    let mut upper = vec![0.0; steps];

    for step in 0..steps {
        let values: Vec<f64> = baseline_norms.iter().map(|run| run[step]).collect();
        let average = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values
            .iter()
            .map(|value| (value - average).powi(2))
            .sum::<f64>()
            / values.len() as f64;
        let deviation = variance.sqrt();
        let max_value = values.iter().copied().fold(0.0_f64, f64::max);
        mean[step] = average;
        std[step] = deviation;
        max_baseline[step] = max_value;
        upper[step] = (average + sigma_multiplier * deviation).max(max_value) + floor;
    }

    let baseline_ensemble_peak = max_baseline.iter().copied().fold(0.0_f64, f64::max);

    Envelope {
        mean,
        std,
        max_baseline,
        upper,
        provenance: build_provenance(
            regime_label,
            baseline_norms.len(),
            sigma_multiplier,
            floor,
            baseline_reference_residual_peak,
            baseline_ensemble_peak,
            baseline_reference_signal_peak,
            baseline_reference_signal_energy,
        ),
    }
}

pub fn evaluate_signal(
    signal: &[f64],
    envelope: &Envelope,
    consecutive: usize,
    dt: f64,
    normalization_epsilon: f64,
    regime_label: CrossingRegimeLabel,
    interpretation_settings: &DetectabilityInterpretationSettings,
) -> DetectabilitySummary {
    let (global_signal_peak, global_signal_peak_time) = peak_with_time(signal, dt);
    let (global_envelope_peak, global_envelope_peak_time) = peak_with_time(&envelope.upper, dt);

    let mut first_crossing_step = None;
    let mut consecutive_crossing_step = None;
    let mut streak = 0usize;

    for (step, value) in signal.iter().enumerate() {
        let threshold = envelope.upper.get(step).copied().unwrap_or(0.0);
        if *value > threshold {
            if first_crossing_step.is_none() {
                first_crossing_step = Some(step);
            }
            streak += 1;
            if streak >= consecutive && consecutive_crossing_step.is_none() {
                consecutive_crossing_step = Some(step + 1 - consecutive);
            }
        } else {
            streak = 0;
        }
    }

    let signal_at_first_crossing =
        first_crossing_step.and_then(|step| signal.get(step).copied());
    let envelope_at_first_crossing = first_crossing_step
        .and_then(|step| envelope.upper.get(step).copied());
    let crossing_margin = signal_at_first_crossing
        .zip(envelope_at_first_crossing)
        .map(|(signal_value, envelope_value)| signal_value - envelope_value);
    let normalized_crossing_margin = crossing_margin
        .zip(envelope_at_first_crossing)
        .map(|(margin, envelope_value)| margin / (envelope_value + normalization_epsilon));
    let (
        post_crossing_persistence_duration,
        post_crossing_fraction,
        peak_margin_after_crossing,
    ) = post_crossing_metrics(signal, envelope, first_crossing_step, dt, interpretation_settings);
    let (interpretation_class, interpretation_note) = classify_detectability(
        signal.len(),
        dt,
        regime_label,
        first_crossing_step,
        first_crossing_step.map(|step| step as f64 * dt),
        consecutive_crossing_step,
        normalized_crossing_margin,
        post_crossing_fraction,
        interpretation_settings,
    );

    DetectabilitySummary {
        global_signal_peak,
        global_envelope_peak,
        global_signal_peak_time,
        global_envelope_peak_time,
        crossing_regime_label: regime_label,
        first_crossing_time: first_crossing_step.map(|step| step as f64 * dt),
        first_crossing_step,
        signal_at_first_crossing,
        envelope_at_first_crossing,
        crossing_margin,
        normalized_crossing_margin,
        consecutive_crossing_time: consecutive_crossing_step.map(|step| step as f64 * dt),
        consecutive_crossing_step,
        post_crossing_persistence_duration,
        post_crossing_fraction,
        peak_margin_after_crossing,
        interpretation_class,
        interpretation_note,
    }
}

pub fn crossing_regime_label(
    additive_noise_std: f64,
    predictor_spring_scale: f64,
) -> CrossingRegimeLabel {
    let has_noise = additive_noise_std > 0.0;
    let has_mismatch = (predictor_spring_scale - 1.0).abs() > 1.0e-12;
    match (has_noise, has_mismatch) {
        (false, false) => CrossingRegimeLabel::Clean,
        (true, true) => CrossingRegimeLabel::Combined,
        _ => CrossingRegimeLabel::Stressed,
    }
}

fn build_provenance(
    regime_label: &str,
    baseline_runs: usize,
    sigma_multiplier: f64,
    floor: f64,
    baseline_reference_residual_peak: f64,
    baseline_ensemble_peak: f64,
    baseline_reference_signal_peak: f64,
    baseline_reference_signal_energy: f64,
) -> EnvelopeProvenance {
    EnvelopeProvenance {
        regime_label: regime_label.to_string(),
        envelope_mode: "baseline_derived".to_string(),
        envelope_basis: "upper[t] = max(max_baseline[t], mean[t] + sigma_multiplier * std[t]) + additive_floor over baseline residual-norm runs under the same configuration.".to_string(),
        envelope_universal: false,
        baseline_reference_residual_peak,
        baseline_ensemble_peak,
        baseline_reference_signal_peak,
        baseline_reference_signal_energy,
        parameters: EnvelopeParameters {
            baseline_runs,
            sigma_multiplier,
            additive_floor: floor,
            smoothing: "none".to_string(),
            interpolation: "none".to_string(),
        },
    }
}

fn post_crossing_metrics(
    signal: &[f64],
    envelope: &Envelope,
    first_crossing_step: Option<usize>,
    dt: f64,
    settings: &DetectabilityInterpretationSettings,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let Some(start_step) = first_crossing_step else {
        return (None, None, None);
    };
    if signal.is_empty() {
        return (None, None, None);
    }

    let end_step = (start_step + settings.persistence_window_steps.max(1)).min(signal.len());
    let mut persistence_steps = 0usize;
    let mut above_count = 0usize;
    let mut peak_margin = f64::NEG_INFINITY;
    let mut still_persistent = true;

    for step in start_step..end_step {
        let threshold = envelope.upper.get(step).copied().unwrap_or(0.0);
        let margin = signal[step] - threshold;
        peak_margin = peak_margin.max(margin);

        if margin > 0.0 {
            above_count += 1;
            if still_persistent {
                persistence_steps += 1;
            }
        } else {
            still_persistent = false;
        }
    }

    let window_len = end_step.saturating_sub(start_step).max(1);
    (
        Some(persistence_steps as f64 * dt),
        Some(above_count as f64 / window_len as f64),
        Some(peak_margin.max(0.0)),
    )
}

fn classify_detectability(
    signal_len: usize,
    dt: f64,
    regime_label: CrossingRegimeLabel,
    first_crossing_step: Option<usize>,
    first_crossing_time: Option<f64>,
    consecutive_crossing_step: Option<usize>,
    normalized_crossing_margin: Option<f64>,
    post_crossing_fraction: Option<f64>,
    settings: &DetectabilityInterpretationSettings,
) -> (DetectabilityInterpretationClass, String) {
    let Some(first_step) = first_crossing_step else {
        return (
            DetectabilityInterpretationClass::NotDetected,
            "No pointwise crossing against the baseline-derived envelope was observed in this run."
                .to_string(),
        );
    };

    let total_duration = signal_len.saturating_sub(1) as f64 * dt;
    let first_time = first_crossing_time.unwrap_or(first_step as f64 * dt);
    let normalized_margin = normalized_crossing_margin.unwrap_or(0.0);
    let post_fraction = post_crossing_fraction.unwrap_or(0.0);
    let early_crossing =
        first_time <= total_duration.max(dt) * settings.early_crossing_fraction_threshold;
    let low_margin = normalized_margin <= settings.low_margin_threshold;
    let sustained = consecutive_crossing_step.is_some()
        || post_fraction >= settings.structural_post_crossing_fraction_threshold;
    let strong_margin = normalized_margin >= settings.structural_margin_threshold;

    if regime_label != CrossingRegimeLabel::Clean
        && early_crossing
        && low_margin
        && post_fraction < settings.structural_post_crossing_fraction_threshold
    {
        return (
            DetectabilityInterpretationClass::EarlyLowMarginCrossing,
            "A stressed-regime crossing appeared very early with a small normalized margin and limited immediate persistence, so it is treated as a potentially stress-confounded early low-margin crossing rather than clean structural separation."
                .to_string(),
        );
    }

    if (strong_margin && sustained)
        || (regime_label == CrossingRegimeLabel::Clean && (strong_margin || sustained))
    {
        return (
            DetectabilityInterpretationClass::StructuralDetected,
            "The pointwise crossing is accompanied by enough margin and immediate post-crossing persistence to be treated as structural detectability within this controlled synthetic setup."
                .to_string(),
        );
    }

    (
        DetectabilityInterpretationClass::StressDetected,
        "A pointwise crossing occurred, but under the current stressed regime the margin and persistence are not strong enough to label it as clean structural separation. It is therefore retained as a stress-detected event."
            .to_string(),
    )
}

fn peak_with_time(values: &[f64], dt: f64) -> (f64, f64) {
    let mut peak_value = 0.0;
    let mut peak_step = 0usize;

    for (step, value) in values.iter().copied().enumerate() {
        if step == 0 || value > peak_value {
            peak_value = value;
            peak_step = step;
        }
    }

    (peak_value, peak_step as f64 * dt)
}
