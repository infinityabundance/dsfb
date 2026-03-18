use nalgebra::{DMatrix, SymmetricEigen};
use serde::Serialize;

use crate::detectability::{
    CrossingRegimeLabel, DetectabilityInterpretationClass, DetectabilitySummary,
    EnvelopeProvenance,
};
use crate::residuals::TimeSeriesBundle;
use crate::spectra::SpectralComparison;
use crate::utils::{covariance_trace, offdiag_energy};

#[derive(Clone, Debug, Serialize)]
pub struct CanonicalMetricGuide {
    pub description: String,
    pub comparison_backbone: String,
    pub note: String,
    pub metric_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpectralCanonicalMetrics {
    pub delta_norm_2: f64,
    pub max_abs_eigenvalue_shift: f64,
    pub mean_abs_eigenvalue_shift: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResidualCanonicalMetrics {
    pub max_raw_residual_norm: f64,
    pub max_normalized_residual_norm: f64,
    pub residual_energy_ratio: f64,
    pub time_to_peak_residual: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct TemporalCanonicalMetrics {
    pub max_drift_norm: f64,
    pub max_slew_norm: f64,
    pub time_to_peak_drift: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DetectabilityCanonicalMetrics {
    pub detected: bool,
    pub crossing_regime_label: CrossingRegimeLabel,
    pub interpretation_class: DetectabilityInterpretationClass,
    pub first_crossing_time: Option<f64>,
    pub first_crossing_step: Option<usize>,
    pub signal_at_first_crossing: Option<f64>,
    pub envelope_at_first_crossing: Option<f64>,
    pub crossing_margin: Option<f64>,
    pub normalized_crossing_margin: Option<f64>,
    pub post_crossing_persistence_duration: Option<f64>,
    pub post_crossing_fraction: Option<f64>,
    pub peak_margin_after_crossing: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CorrelationCanonicalMetrics {
    pub covariance_trace: f64,
    pub covariance_offdiag_energy: f64,
    pub covariance_rank_estimate: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct EnvelopeCanonicalMetrics {
    pub envelope_mode: String,
    pub envelope_basis: String,
    pub envelope_sigma_multiplier: f64,
    pub envelope_additive_floor: f64,
    pub envelope_baseline_runs: usize,
    pub envelope_universal: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CanonicalCaseMetrics {
    pub subject: String,
    pub case: String,
    pub perturbation_class: String,
    pub spectral: SpectralCanonicalMetrics,
    pub residual: ResidualCanonicalMetrics,
    pub temporal: TemporalCanonicalMetrics,
    pub detectability: DetectabilityCanonicalMetrics,
    pub correlation: CorrelationCanonicalMetrics,
    pub envelope: EnvelopeCanonicalMetrics,
}

#[derive(Clone, Debug, Serialize)]
pub struct CanonicalMetricRow {
    pub subject: String,
    pub case: String,
    pub perturbation_class: String,
    pub delta_norm_2: f64,
    pub max_abs_eigenvalue_shift: f64,
    pub mean_abs_eigenvalue_shift: f64,
    pub max_raw_residual_norm: f64,
    pub max_normalized_residual_norm: f64,
    pub residual_energy_ratio: f64,
    pub time_to_peak_residual: f64,
    pub max_drift_norm: f64,
    pub max_slew_norm: f64,
    pub time_to_peak_drift: f64,
    pub detected: bool,
    pub crossing_regime_label: CrossingRegimeLabel,
    pub interpretation_class: DetectabilityInterpretationClass,
    pub first_crossing_time: Option<f64>,
    pub first_crossing_step: Option<usize>,
    pub signal_at_first_crossing: Option<f64>,
    pub envelope_at_first_crossing: Option<f64>,
    pub crossing_margin: Option<f64>,
    pub normalized_crossing_margin: Option<f64>,
    pub post_crossing_persistence_duration: Option<f64>,
    pub post_crossing_fraction: Option<f64>,
    pub peak_margin_after_crossing: Option<f64>,
    pub covariance_trace: f64,
    pub covariance_offdiag_energy: f64,
    pub covariance_rank_estimate: usize,
    pub envelope_mode: String,
    pub envelope_basis: String,
    pub envelope_sigma_multiplier: f64,
    pub envelope_additive_floor: f64,
    pub envelope_baseline_runs: usize,
    pub envelope_universal: bool,
}

pub fn canonical_metric_guide() -> CanonicalMetricGuide {
    CanonicalMetricGuide {
        description: "These canonical evaluation quantities are the comparison backbone for dsfb-lattice synthetic benchmark runs. They are intended to stabilize run-to-run comparability without claiming a complete evaluation theory.".to_string(),
        comparison_backbone: "Future crate revisions may add auxiliary metrics, but the canonical quantities should be preserved where possible so controlled synthetic runs remain comparable.".to_string(),
        note: "The canonical layer covers spectral, residual, temporal, detectability, correlation, and envelope-provenance quantities. They remain bounded to this crate's harmonic toy setting and baseline-derived thresholds.".to_string(),
        metric_names: vec![
            "delta_norm_2".to_string(),
            "max_abs_eigenvalue_shift".to_string(),
            "mean_abs_eigenvalue_shift".to_string(),
            "max_raw_residual_norm".to_string(),
            "max_normalized_residual_norm".to_string(),
            "residual_energy_ratio".to_string(),
            "time_to_peak_residual".to_string(),
            "max_drift_norm".to_string(),
            "max_slew_norm".to_string(),
            "time_to_peak_drift".to_string(),
            "detected".to_string(),
            "crossing_regime_label".to_string(),
            "interpretation_class".to_string(),
            "first_crossing_time".to_string(),
            "first_crossing_step".to_string(),
            "signal_at_first_crossing".to_string(),
            "envelope_at_first_crossing".to_string(),
            "crossing_margin".to_string(),
            "normalized_crossing_margin".to_string(),
            "post_crossing_persistence_duration".to_string(),
            "post_crossing_fraction".to_string(),
            "peak_margin_after_crossing".to_string(),
            "covariance_trace".to_string(),
            "covariance_offdiag_energy".to_string(),
            "covariance_rank_estimate".to_string(),
            "envelope_mode".to_string(),
            "envelope_basis".to_string(),
            "envelope_parameters".to_string(),
            "envelope_universal".to_string(),
        ],
    }
}

pub fn build_canonical_case_metrics(
    subject: &str,
    case: &str,
    perturbation_class: &str,
    comparison: &SpectralComparison,
    bundle: &TimeSeriesBundle,
    detectability: Option<&DetectabilitySummary>,
    covariance: &DMatrix<f64>,
    envelope: &EnvelopeProvenance,
    dt: f64,
) -> CanonicalCaseMetrics {
    let mean_abs_eigenvalue_shift = if comparison.per_mode_abs_shift.is_empty() {
        0.0
    } else {
        comparison.per_mode_abs_shift.iter().sum::<f64>() / comparison.per_mode_abs_shift.len() as f64
    };
    let (max_raw_residual_norm, time_to_peak_residual) = peak_with_time(&bundle.residual_norms, dt);
    let max_normalized_residual_norm = bundle
        .normalized_residual_norms
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let (max_drift_norm, time_to_peak_drift) = peak_with_time(&bundle.drift_norms, dt);
    let max_slew_norm = bundle.slew_norms.iter().copied().fold(0.0_f64, f64::max);

    CanonicalCaseMetrics {
        subject: subject.to_string(),
        case: case.to_string(),
        perturbation_class: perturbation_class.to_string(),
        spectral: SpectralCanonicalMetrics {
            delta_norm_2: comparison.delta_norm_2,
            max_abs_eigenvalue_shift: comparison.max_abs_shift,
            mean_abs_eigenvalue_shift,
        },
        residual: ResidualCanonicalMetrics {
            max_raw_residual_norm,
            max_normalized_residual_norm,
            residual_energy_ratio: bundle.residual_energy_ratio,
            time_to_peak_residual,
        },
        temporal: TemporalCanonicalMetrics {
            max_drift_norm,
            max_slew_norm,
            time_to_peak_drift,
        },
        detectability: DetectabilityCanonicalMetrics {
            detected: detectability
                .and_then(|summary| summary.first_crossing_step)
                .is_some(),
            crossing_regime_label: detectability
                .map(|summary| summary.crossing_regime_label)
                .unwrap_or(CrossingRegimeLabel::Clean),
            interpretation_class: detectability
                .map(|summary| summary.interpretation_class)
                .unwrap_or(DetectabilityInterpretationClass::NotDetected),
            first_crossing_time: detectability.and_then(|summary| summary.first_crossing_time),
            first_crossing_step: detectability.and_then(|summary| summary.first_crossing_step),
            signal_at_first_crossing: detectability.and_then(|summary| summary.signal_at_first_crossing),
            envelope_at_first_crossing: detectability.and_then(|summary| summary.envelope_at_first_crossing),
            crossing_margin: detectability.and_then(|summary| summary.crossing_margin),
            normalized_crossing_margin: detectability
                .and_then(|summary| summary.normalized_crossing_margin),
            post_crossing_persistence_duration: detectability
                .and_then(|summary| summary.post_crossing_persistence_duration),
            post_crossing_fraction: detectability
                .and_then(|summary| summary.post_crossing_fraction),
            peak_margin_after_crossing: detectability
                .and_then(|summary| summary.peak_margin_after_crossing),
        },
        correlation: CorrelationCanonicalMetrics {
            covariance_trace: covariance_trace(covariance),
            covariance_offdiag_energy: offdiag_energy(covariance),
            covariance_rank_estimate: covariance_rank_estimate(covariance),
        },
        envelope: EnvelopeCanonicalMetrics {
            envelope_mode: envelope.envelope_mode.clone(),
            envelope_basis: envelope.envelope_basis.clone(),
            envelope_sigma_multiplier: envelope.parameters.sigma_multiplier,
            envelope_additive_floor: envelope.parameters.additive_floor,
            envelope_baseline_runs: envelope.parameters.baseline_runs,
            envelope_universal: envelope.envelope_universal,
        },
    }
}

pub fn flatten_canonical_metrics(metrics: &[CanonicalCaseMetrics]) -> Vec<CanonicalMetricRow> {
    metrics
        .iter()
        .map(|metric| CanonicalMetricRow {
            subject: metric.subject.clone(),
            case: metric.case.clone(),
            perturbation_class: metric.perturbation_class.clone(),
            delta_norm_2: metric.spectral.delta_norm_2,
            max_abs_eigenvalue_shift: metric.spectral.max_abs_eigenvalue_shift,
            mean_abs_eigenvalue_shift: metric.spectral.mean_abs_eigenvalue_shift,
            max_raw_residual_norm: metric.residual.max_raw_residual_norm,
            max_normalized_residual_norm: metric.residual.max_normalized_residual_norm,
            residual_energy_ratio: metric.residual.residual_energy_ratio,
            time_to_peak_residual: metric.residual.time_to_peak_residual,
            max_drift_norm: metric.temporal.max_drift_norm,
            max_slew_norm: metric.temporal.max_slew_norm,
            time_to_peak_drift: metric.temporal.time_to_peak_drift,
            detected: metric.detectability.detected,
            crossing_regime_label: metric.detectability.crossing_regime_label,
            interpretation_class: metric.detectability.interpretation_class,
            first_crossing_time: metric.detectability.first_crossing_time,
            first_crossing_step: metric.detectability.first_crossing_step,
            signal_at_first_crossing: metric.detectability.signal_at_first_crossing,
            envelope_at_first_crossing: metric.detectability.envelope_at_first_crossing,
            crossing_margin: metric.detectability.crossing_margin,
            normalized_crossing_margin: metric.detectability.normalized_crossing_margin,
            post_crossing_persistence_duration: metric
                .detectability
                .post_crossing_persistence_duration,
            post_crossing_fraction: metric.detectability.post_crossing_fraction,
            peak_margin_after_crossing: metric.detectability.peak_margin_after_crossing,
            covariance_trace: metric.correlation.covariance_trace,
            covariance_offdiag_energy: metric.correlation.covariance_offdiag_energy,
            covariance_rank_estimate: metric.correlation.covariance_rank_estimate,
            envelope_mode: metric.envelope.envelope_mode.clone(),
            envelope_basis: metric.envelope.envelope_basis.clone(),
            envelope_sigma_multiplier: metric.envelope.envelope_sigma_multiplier,
            envelope_additive_floor: metric.envelope.envelope_additive_floor,
            envelope_baseline_runs: metric.envelope.envelope_baseline_runs,
            envelope_universal: metric.envelope.envelope_universal,
        })
        .collect()
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

fn covariance_rank_estimate(covariance: &DMatrix<f64>) -> usize {
    if covariance.nrows() == 0 || covariance.ncols() == 0 {
        return 0;
    }

    let eigen = SymmetricEigen::new(covariance.clone());
    let largest = eigen
        .eigenvalues
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);
    if largest <= 1.0e-12 {
        return 0;
    }

    let threshold = (0.05 * largest).max(1.0e-10);
    eigen
        .eigenvalues
        .iter()
        .filter(|value| **value >= threshold)
        .count()
}
