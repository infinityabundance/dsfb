//! Core types shared across the crate.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level experiment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Which bearing set to analyse (1, 2, or 3).
    pub bearing_set: u32,
    /// Which channel index within that set to use as the primary channel (0-based).
    pub primary_channel: usize,
    /// Number of samples per analysis window.
    pub window_size: usize,
    /// Fraction of total run duration used as the nominal baseline.
    pub nominal_fraction: f64,
    /// Rolling quantile level for the admissibility envelope (e.g. 0.99).
    pub envelope_quantile: f64,
    /// Minimum consecutive windows for sustained detection.
    pub sustained_count: usize,
    /// Threshold on the trust score for precursor detection (0..1).
    pub trust_threshold: f64,
    /// Output directory root.
    pub output_dir: PathBuf,
    /// Fixed random seed for reproducibility.
    pub seed: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bearing_set: 1,
            primary_channel: 0,
            window_size: 20480,
            nominal_fraction: 0.15,
            envelope_quantile: 0.99,
            sustained_count: 5,
            trust_threshold: 0.5,
            output_dir: PathBuf::from("output-dsfb-endoduction"),
            seed: 42,
        }
    }
}

/// Per-window metrics collected during analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowMetrics {
    /// Window index (0-based, chronological order).
    pub index: usize,
    /// File name of the source snapshot.
    pub file_name: String,
    /// RMS of raw signal in this window.
    pub rms: f64,
    /// Kurtosis of raw signal.
    pub kurtosis: f64,
    /// Crest factor of raw signal.
    pub crest_factor: f64,
    /// Rolling variance of the residual.
    pub residual_variance: f64,
    /// Lag-1 autocorrelation of the residual.
    pub residual_autocorr: f64,
    /// Fraction of residual samples outside the admissibility envelope.
    pub envelope_breach_fraction: f64,
    /// Spectral centroid shift relative to nominal baseline.
    pub spectral_centroid_shift: f64,
    /// Persistence score (fraction of sign-consistent residual runs).
    pub persistence: f64,
    /// Drift magnitude (slope of residual over the window).
    pub drift: f64,
    /// Variance growth rate relative to nominal.
    pub variance_growth: f64,
    /// Composite DSFB trust / precursor score (0..1).
    pub trust_score: f64,
    /// RMS of raw signal (baseline metric).
    pub baseline_rms: f64,
    /// Rolling variance of raw signal (baseline metric).
    pub baseline_rolling_var: f64,
    /// Spectral band energy (baseline metric).
    pub spectral_band_energy: f64,
}

/// Outcome of the full pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    /// Crate version.
    pub crate_version: String,
    /// Git revision if available.
    pub git_revision: Option<String>,
    /// Timestamp of the run (ISO-8601).
    pub timestamp: String,
    /// Configuration used.
    pub config: Config,
    /// Dataset source description.
    pub dataset_source: String,
    /// Number of snapshot files processed.
    pub snapshots_processed: usize,
    /// Number of windows in the nominal baseline.
    pub nominal_windows: usize,
    /// First sustained DSFB detection window index, if any.
    pub dsfb_first_detection: Option<usize>,
    /// Summary metrics.
    pub summary: SummaryMetrics,
    /// Files produced in this run.
    pub files_produced: Vec<String>,
    /// Gate pass/fail results.
    pub gates: GateResults,
}

/// Aggregate summary metrics for the run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMetrics {
    /// Empirical lead time: windows between first sustained DSFB detection and failure.
    pub dsfb_lead_time_windows: Option<i64>,
    /// First sustained detection window for each baseline method.
    pub baseline_first_detections: std::collections::HashMap<String, Option<usize>>,
    /// Lead time for each baseline method.
    pub baseline_lead_times: std::collections::HashMap<String, Option<i64>>,
    /// Total number of windows.
    pub total_windows: usize,
    /// Index of the failure reference window.
    pub failure_window: usize,
    /// Index past which the nominal regime is defined.
    pub nominal_end_window: usize,
}

/// Gate pass/fail results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResults {
    pub crate_builds: bool,
    pub real_data_used: bool,
    pub timestamped_output: bool,
    pub twelve_figures: bool,
    pub csv_produced: bool,
    pub json_produced: bool,
    pub pdf_produced: bool,
    pub zip_produced: bool,
    pub baseline_comparisons: bool,
    pub manifest_produced: bool,
}

impl GateResults {
    /// Returns true only if every gate passes.
    pub fn all_passed(&self) -> bool {
        self.crate_builds
            && self.real_data_used
            && self.timestamped_output
            && self.twelve_figures
            && self.csv_produced
            && self.json_produced
            && self.pdf_produced
            && self.zip_produced
            && self.baseline_comparisons
            && self.manifest_produced
    }
}
