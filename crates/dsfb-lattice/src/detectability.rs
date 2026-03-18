use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Envelope {
    pub mean: Vec<f64>,
    pub std: Vec<f64>,
    pub max_baseline: Vec<f64>,
    pub upper: Vec<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DetectabilitySummary {
    pub first_crossing_step: Option<usize>,
    pub consecutive_crossing_step: Option<usize>,
    pub first_crossing_time: Option<f64>,
    pub consecutive_crossing_time: Option<f64>,
    pub envelope_peak: f64,
    pub signal_peak: f64,
}

pub fn build_envelope(baseline_norms: &[Vec<f64>], sigma_multiplier: f64, floor: f64) -> Envelope {
    if baseline_norms.is_empty() {
        return Envelope {
            mean: Vec::new(),
            std: Vec::new(),
            max_baseline: Vec::new(),
            upper: Vec::new(),
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

    Envelope {
        mean,
        std,
        max_baseline,
        upper,
    }
}

pub fn evaluate_signal(
    signal: &[f64],
    envelope: &Envelope,
    consecutive: usize,
    dt: f64,
) -> DetectabilitySummary {
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

    DetectabilitySummary {
        first_crossing_step,
        consecutive_crossing_step,
        first_crossing_time: first_crossing_step.map(|step| step as f64 * dt),
        consecutive_crossing_time: consecutive_crossing_step.map(|step| step as f64 * dt),
        envelope_peak: envelope.upper.iter().copied().fold(0.0_f64, f64::max),
        signal_peak: signal.iter().copied().fold(0.0_f64, f64::max),
    }
}
