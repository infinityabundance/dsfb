//! Evaluation: lead-time estimation, detection comparison, and
//! summary statistics.

use crate::baselines;
use crate::types::{SummaryMetrics, WindowMetrics};
use std::collections::HashMap;

/// Evaluate DSFB and baseline detection results.
///
/// Returns summary metrics including lead times and first detections.
pub fn evaluate(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    sustained: usize,
    trust_threshold: f64,
    baseline_k: f64,
) -> SummaryMetrics {
    let total = metrics.len();

    // DSFB detection: trust score exceeds threshold for `sustained` consecutive windows.
    let dsfb_flags: Vec<bool> = metrics.iter().map(|m| m.trust_score >= trust_threshold).collect();
    let dsfb_first = baselines::first_sustained_detection(&dsfb_flags, sustained);
    let dsfb_lead = dsfb_first.map(|d| failure_window as i64 - d as i64);

    // Baseline detections.
    let mut baseline_first = HashMap::new();
    let mut baseline_leads = HashMap::new();

    // For each baseline metric, compute nominal-window statistics and detect.
    let nominal_metrics: Vec<&WindowMetrics> = metrics.iter().filter(|m| m.index < nominal_end).collect();

    let methods: Vec<(&str, Box<dyn Fn(&WindowMetrics) -> f64>)> = vec![
        ("RMS", Box::new(|m: &WindowMetrics| m.baseline_rms)),
        ("Kurtosis", Box::new(|m: &WindowMetrics| m.kurtosis)),
        ("Crest Factor", Box::new(|m: &WindowMetrics| m.crest_factor)),
        ("Rolling Variance", Box::new(|m: &WindowMetrics| m.baseline_rolling_var)),
        ("Lag-1 Autocorrelation", Box::new(|m: &WindowMetrics| m.residual_autocorr)),
        ("Spectral Band Energy", Box::new(|m: &WindowMetrics| m.spectral_band_energy)),
    ];

    for (name, extractor) in &methods {
        let nominal_vals: Vec<f64> = nominal_metrics.iter().map(|m| extractor(m)).collect();
        let mean = crate::baseline::mean(&nominal_vals);
        let std = crate::baseline::std_dev(&nominal_vals);

        let flags: Vec<bool> = metrics
            .iter()
            .map(|m| baselines::exceeds_threshold(extractor(m), mean, std, baseline_k))
            .collect();
        let first = baselines::first_sustained_detection(&flags, sustained);
        let lead = first.map(|d| failure_window as i64 - d as i64);
        baseline_first.insert(name.to_string(), first);
        baseline_leads.insert(name.to_string(), lead);
    }

    SummaryMetrics {
        dsfb_lead_time_windows: dsfb_lead,
        baseline_first_detections: baseline_first,
        baseline_lead_times: baseline_leads,
        total_windows: total,
        failure_window,
        nominal_end_window: nominal_end,
    }
}
