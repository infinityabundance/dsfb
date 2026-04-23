//! Continuous Rigor audit report for the 4-stage verification pipeline.
//!
//! ## Purpose
//!
//! Each example in this crate runs the same 4-stage Continuous Rigor pipeline:
//!
//! ```text
//! Stage I   — Physics-Only Baseline       : synthetic, zero impairment, verifies math
//! Stage II  — Impairment Injection        : same signal + hardware/channel impairments
//! Stage III — SigMF-Annotated Playback    : structurally representative public dataset
//! Stage IV  — Audit Report                : predicted vs. ground-truth comparison
//! ```
//!
//! This module provides the data types that consolidate Stage IV output:
//! per-stage detection statistics, latency accounting, and one-line SBIR
//! pitch metrics (false-alarm rate, lead-time advantage, 10⁻⁵ FA threshold).
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - All fields are value types: no pointers, no heap references
//! - Stage results carry `label: &'static str` for provenance traceability
//! - `AuditReport` is `Copy` so it can be placed on the stack and returned
//!   through FFI boundaries without heap allocation

// ── Per-Stage Detection Statistics ────────────────────────────────────────

/// Detection statistics for one stage of the Continuous Rigor pipeline.
///
/// Populated by the per-stage loops in each example.
#[derive(Debug, Clone, Copy)]
pub struct StageResult {
    /// Human-readable stage label (e.g. `"Stage I: Physics Baseline"`).
    pub label: &'static str,
    /// Total observations processed in this stage (including calibration).
    pub n_obs: u32,
    /// Observations in the "calm" pre-event segment (for FA counting).
    pub n_calm_obs: u32,
    /// Policy::Review or Policy::Escalate events in the calm segment
    /// (false alarms).
    pub n_false_alarms: u32,
    /// Policy::Review or Policy::Escalate events in the event / post-onset
    /// segment (detections).
    pub n_detections: u32,
    /// Sample index k of the first detection (None if no detection).
    pub first_detection_k: Option<u32>,
    /// Lyapunov exponent λ at the first detection (None if no detection).
    pub lambda_at_detection: Option<f32>,
    /// Maximum |λ| observed in the event segment.
    pub lambda_event_peak: f32,
    /// Ground-truth onset sample index (from SigMF annotation or scenario model).
    pub ground_truth_onset_k: u32,
}

impl StageResult {
    /// Construct a zeroed-out result with a label and ground-truth onset.
    pub const fn new(label: &'static str, ground_truth_onset_k: u32) -> Self {
        Self {
            label,
            n_obs: 0,
            n_calm_obs: 0,
            n_false_alarms: 0,
            n_detections: 0,
            first_detection_k: None,
            lambda_at_detection: None,
            lambda_event_peak: 0.0,
            ground_truth_onset_k,
        }
    }

    /// False-alarm rate: n_false_alarms / n_calm_obs.
    ///
    /// Returns 0.0 if no calm observations.
    pub fn false_alarm_rate(&self) -> f32 {
        if self.n_calm_obs == 0 { return 0.0; }
        self.n_false_alarms as f32 / self.n_calm_obs as f32
    }

    /// Lead-time advantage in samples: ground_truth − first_detection.
    ///
    /// Positive → DSFB detected BEFORE the ground-truth reference time.
    /// Negative → DSFB detected AFTER (degraded performance or late onset).
    /// Returns `None` if no detection was made.
    pub fn lead_time_samples(&self) -> Option<i32> {
        self.first_detection_k.map(|k| {
            self.ground_truth_onset_k as i32 - k as i32
        })
    }

    /// Lead-time expressed as milliseconds, given the sample rate in Hz.
    pub fn lead_time_ms(&self, sample_rate_hz: f32) -> Option<f32> {
        self.lead_time_samples().map(|lt| {
            lt as f32 / sample_rate_hz * 1000.0
        })
    }

    /// Whether this stage meets the SBIR 10⁻⁵ false-alarm threshold.
    ///
    /// At a calm segment of N observations, FA rate < 10⁻⁵ requires < N/10⁵
    /// false alarms.  This method uses the observed rate as a proxy.
    /// Formal FA validation requires large-N Monte Carlo beyond what a
    /// single pipeline run provides — see paper §L5 for caveat.
    pub fn meets_1e5_fa_threshold(&self) -> bool {
        self.false_alarm_rate() < 1e-5
    }
}

// ── Ground-Truth Annotation (SigMF-inspired) ──────────────────────────────

/// A single ground-truth event annotation, modelled after SigMF core/annotation.
///
/// In a production pipeline this would parse a `.sigmf-meta` JSON file.
/// Here it is hand-specified from the dataset documentation.
#[derive(Debug, Clone, Copy)]
pub struct SigMfAnnotation {
    /// Descriptive label (e.g. `"spoofer_onset"`, `"regime_transition"`).
    pub label: &'static str,
    /// Sample index at which the event begins (relative to file start).
    pub onset_sample: u32,
    /// Sample index at which the event ends (0 = unknown).
    pub end_sample: u32,
    /// Annotation confidence from dataset provider (1.0 = high).
    pub confidence: f32,
}

impl SigMfAnnotation {
    /// Construct a precise, high-confidence annotation.
    pub const fn precise(label: &'static str, onset: u32, end: u32) -> Self {
        Self {
            label,
            onset_sample: onset,
            end_sample: end,
            confidence: 1.0,
        }
    }
}

// ── 4-Stage Audit Report ──────────────────────────────────────────────────

/// Consolidated 4-stage Continuous Rigor audit report for one example run.
///
/// Produced at the end of every benchmark example.  Contains Stage I–III
/// statistics plus Stage IV comparison metrics.
///
/// ## SBIR Pitch Keys
///
/// The fields most relevant to SBIR Phase II reviewers are:
///
/// - `stage_i.false_alarm_rate()` — should be 0.0 in clean synthetic
/// - `stage_ii.false_alarm_rate()` — should be < 10⁻³ under full impairment
/// - `stage_iii.lead_time_samples()` — positive = DSFB detects before nav/link failure
/// - `observer_contract_holds` — true if no upstream mutations occurred
/// - `unsafe_count` — always 0, enforced by `#![forbid(unsafe_code)]`
#[derive(Debug, Clone, Copy)]
pub struct AuditReport {
    /// Example / dataset label.
    pub dataset_label: &'static str,
    /// Stage I: Physics-Only Baseline result.
    pub stage_i: StageResult,
    /// Stage II: Impairment-Injected result.
    pub stage_ii: StageResult,
    /// Stage III: SigMF Playback result.
    pub stage_iii: StageResult,
    /// Nominal sample rate of the dataset [Hz].
    pub sample_rate_hz: f32,
    /// Whether the observer contract (read-only, no upstream mutation) held.
    /// Always `true` in this crate; recorded for provenance.
    pub observer_contract_holds: bool,
    /// Number of `unsafe` blocks in the crate: always 0.
    pub unsafe_count: u32,
    /// Non-claim statement: what this report does NOT prove.
    pub non_claim: &'static str,
}

impl AuditReport {
    /// Print a formatted audit report to the host console via `println!`.
    ///
    /// This method is `std`-only (gated at call-site in examples).
    #[cfg(feature = "std")]
    pub fn print(&self) {
        extern crate std;
        use std::println;

        println!();
        println!("┌─────────────────────────────────────────────────────");
        println!("│  CONTINUOUS RIGOR AUDIT — {}", self.dataset_label);
        let obs_status = if self.observer_contract_holds { "HOLDS" } else { "VIOLATED" };
        println!("│  Sample rate: {:.0} Hz   Observer contract: {}   unsafe: {}",
            self.sample_rate_hz, obs_status, self.unsafe_count);
        println!("├─────────────────────────────────────────────────────");

        for (idx, stage) in [&self.stage_i, &self.stage_ii, &self.stage_iii]
            .iter().enumerate()
        {
            let stage_num = idx + 1;
            let fa_rate = stage.false_alarm_rate();
            let fa_flag = if fa_rate < 1e-5 { "✓ < 10⁻⁵" }
                          else if fa_rate < 1e-3 { "⚠ < 10⁻³" }
                          else { "✗ ≥ 10⁻³" };

            println!("│");
            println!("│  Stage {}  {}", stage_num, stage.label);
            println!("│    Observations  : {} ({} calm)",
                stage.n_obs, stage.n_calm_obs);
            println!("│    False alarms  : {}  FA rate: {:.2e}  [{}]",
                stage.n_false_alarms, fa_rate, fa_flag);
            match stage.first_detection_k {
                Some(k) => println!("│    Detections    : {}  First at k={}",
                    stage.n_detections, k),
                None => println!("│    Detections    : {}  First at k=NONE",
                    stage.n_detections),
            }
            match stage.lead_time_samples() {
                Some(lt) => println!("│    Lead time     : {:+} samples ({:+.1} ms)",
                    lt, lt as f32 / self.sample_rate_hz * 1000.0),
                None => println!("│    Lead time     : N/A"),
            }
            if let Some(lam) = stage.lambda_at_detection {
                println!("│    λ at detect   : {:+.4}", lam);
            }
            println!("│    λ_event_peak  : {:+.4}", stage.lambda_event_peak);
            println!("│    GT onset      : k={}", stage.ground_truth_onset_k);
        }

        println!("├─────────────────────────────────────────────────────");
        println!("│  NON-CLAIM: {}", self.non_claim);
        println!("└─────────────────────────────────────────────────────");
        println!();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_result_fa_rate_zero_calm_obs() {
        let r = StageResult::new("test", 100);
        assert_eq!(r.false_alarm_rate(), 0.0);
    }

    #[test]
    fn stage_result_lead_time_none_when_no_detection() {
        let r = StageResult::new("test", 500);
        assert_eq!(r.lead_time_samples(), None);
    }

    #[test]
    fn stage_result_lead_time_positive_early_detection() {
        let mut r = StageResult::new("test", 500);
        r.first_detection_k = Some(480);
        assert_eq!(r.lead_time_samples(), Some(20));
    }

    #[test]
    fn stage_result_lead_time_negative_late_detection() {
        let mut r = StageResult::new("test", 500);
        r.first_detection_k = Some(520);
        assert_eq!(r.lead_time_samples(), Some(-20));
    }

    #[test]
    fn stage_result_fa_rate_threshold() {
        let mut r = StageResult::new("test", 100);
        r.n_calm_obs = 10_000;
        r.n_false_alarms = 0;
        assert!(r.meets_1e5_fa_threshold());
        r.n_false_alarms = 1; // 1/10000 = 1e-4 > 1e-5
        assert!(!r.meets_1e5_fa_threshold());
    }

    #[test]
    fn sigmf_annotation_precise() {
        let ann = SigMfAnnotation::precise("onset", 1000, 2000);
        assert_eq!(ann.onset_sample, 1000);
        assert!((ann.confidence - 1.0).abs() < 1e-6);
    }
}
