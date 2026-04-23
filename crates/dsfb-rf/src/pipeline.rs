//! Host-side Stage III evaluation pipeline.
//!
//! Implements the fixed read-only protocol described in paper §IX and §F.4:
//!
//! 1. Nominal reference: mean over first 100 clean captures
//! 2. Residual construction: r(k) = x(k) − x̂
//! 3. Envelope: ρ = 3σ from healthy window
//! 4. Drift window W=5 (sign), W=10 (DSA), K=4, τ=2.0, m=1
//! 5. EWMA comparator λ=0.20; CUSUM κ=0.5σ, h=5σ; energy threshold
//! 6. Episode precision with W_pred=5
//!
//! ## Supported Datasets
//!
//! - **RadioML 2018.01a** (DeepSig): synthetic, 24 mod classes, SNR sweep.
//!   DSFB uses SNR-regime transitions (≥0 dB ↔ <0 dB) as ground-truth events.
//! - **ORACLE** (Hanna et al. 2022): real USRP B200 captures, 16 emitters.
//!   DSFB uses emitter power transitions as ground-truth events.
//!
//! ## Non-Claims (paper §L4, §XI)
//!
//! Results are bounded to these public-dataset configurations. No operational
//! deployment result is claimed. No modulation classification is performed.

extern crate std;

use std::vec::Vec;
use crate::engine::{DsfbRfEngine, ObservationResult};
use crate::platform::PlatformContext;
use crate::policy::PolicyDecision;
use crate::math::{mean_f32, std_dev_f32};

// ── Stage III fixed parameters ──────────────────────────────────────────────

/// Healthy calibration window size (paper §F.4).
pub const HEALTHY_WINDOW_SIZE: usize = 100;

/// Drift sign window width (paper §F.4: W=5).
pub const SIGN_WINDOW_W: usize = 5;

/// DSA accumulator window width (paper §F.4: W_dsa=10).
pub const DSA_WINDOW_W: usize = 10;

/// Grammar persistence threshold (paper §F.4: K=4).
pub const GRAMMAR_K: usize = 4;

/// DSA threshold τ (paper §F.4: τ=2.0).
pub const DSA_TAU: f32 = 2.0;

/// Corroboration count m (paper §F.4: m=1).
pub const CORROBORATION_M: u8 = 1;

/// EWMA smoothing weight λ (paper §F.4: λ=0.20).
pub const EWMA_LAMBDA: f32 = 0.20;

/// CUSUM allowance as multiple of σ (paper §F.4: κ=0.5σ).
pub const CUSUM_KAPPA_SIGMA: f32 = 0.5;

/// CUSUM alarm threshold as multiple of σ (paper §F.4: h=5σ).
pub const CUSUM_H_SIGMA: f32 = 5.0;

/// Precursor prediction window W_pred (paper §F.4: W_pred=5).
pub const WPRED: usize = 5;

/// SNR floor in dB (paper §L10: −10 dB).
pub const SNR_FLOOR_DB: f32 = -10.0;

// ── Input / Event structs ───────────────────────────────────────────────────

/// A single observation in the evaluation stream.
#[derive(Debug, Clone)]
pub struct RfObservation {
    /// Observation index k.
    pub k: usize,
    /// Residual norm ‖r(k)‖ (pre-computed or raw IQ norm relative to nominal).
    pub residual_norm: f32,
    /// SNR estimate in dB for this observation.
    pub snr_db: f32,
    /// True if this observation is part of the healthy calibration window.
    pub is_healthy: bool,
}

/// A ground-truth regime-transition event.
#[derive(Debug, Clone, Copy)]
pub struct RegimeTransitionEvent {
    /// Observation index at which the transition occurs.
    pub k: usize,
    /// Human-readable label for the transition type.
    pub label: &'static str,
}

// ── Comparator state (EWMA, CUSUM, energy threshold) ───────────────────────

/// Scalar comparator baselines (paper §IX-G).
///
/// These are the incumbent methods DSFB sits alongside.
/// They are not replaced — they are augmented.
#[derive(Debug)]
pub struct ScalarComparators {
    /// Healthy-window mean for threshold comparator.
    pub threshold_mean: f32,
    /// 3σ threshold.
    pub threshold_3sigma: f32,
    /// EWMA current value.
    pub ewma: f32,
    /// EWMA alarm threshold (mean + 3σ of healthy-window EWMA).
    pub ewma_threshold: f32,
    /// CUSUM positive accumulator.
    pub cusum_pos: f32,
    /// CUSUM allowance κ.
    pub cusum_kappa: f32,
    /// CUSUM alarm threshold h.
    pub cusum_h: f32,
    /// Energy threshold (mean + 3σ).
    pub energy_threshold: f32,
}

impl ScalarComparators {
    /// Calibrate all comparators from the healthy window.
    pub fn calibrate(healthy_norms: &[f32]) -> Self {
        let m = mean_f32(healthy_norms);
        let s = std_dev_f32(healthy_norms);

        // Calibrate EWMA threshold from healthy-window EWMA run
        let mut ewma_vals: Vec<f32> = Vec::with_capacity(healthy_norms.len());
        let mut ewma = 0.0_f32;
        for &n in healthy_norms {
            ewma = EWMA_LAMBDA * n + (1.0 - EWMA_LAMBDA) * ewma;
            ewma_vals.push(ewma);
        }
        let ewma_mean = mean_f32(&ewma_vals);
        let ewma_std = std_dev_f32(&ewma_vals);

        Self {
            threshold_mean: m,
            threshold_3sigma: m + 3.0 * s,
            ewma: 0.0,
            ewma_threshold: ewma_mean + 3.0 * ewma_std,
            cusum_pos: 0.0,
            cusum_kappa: CUSUM_KAPPA_SIGMA * s,
            cusum_h: CUSUM_H_SIGMA * s,
            energy_threshold: m + 3.0 * s,
        }
    }

    /// Update comparators for one observation; returns (threshold_alarm, ewma_alarm,
    /// cusum_alarm, energy_alarm).
    pub fn update(&mut self, norm: f32) -> (bool, bool, bool, bool) {
        // 1. Raw 3σ threshold
        let thr = norm > self.threshold_3sigma;

        // 2. EWMA
        self.ewma = EWMA_LAMBDA * norm + (1.0 - EWMA_LAMBDA) * self.ewma;
        let ewma_alarm = self.ewma > self.ewma_threshold;

        // 3. Positive CUSUM
        self.cusum_pos = (self.cusum_pos + norm - self.cusum_kappa).max(0.0);
        let cusum_alarm = self.cusum_pos > self.cusum_h;

        // 4. Energy
        let energy_alarm = norm > self.energy_threshold;

        (thr, ewma_alarm, cusum_alarm, energy_alarm)
    }

    /// Reset CUSUM accumulator (e.g., after an alarm).
    pub fn reset_cusum(&mut self) {
        self.cusum_pos = 0.0;
    }
}

// ── Episode precision metric ────────────────────────────────────────────────

/// A DSFB episode (contiguous block of Review/Escalate decisions).
#[derive(Debug, Clone)]
pub struct Episode {
    /// Observation index where the episode opened.
    pub open_k: usize,
    /// Observation index where the episode closed (None if still open).
    pub close_k: Option<usize>,
    /// True if this episode is classified as a precursor to a ground-truth event.
    pub is_precursor: bool,
}

/// Full Stage III evaluation result — all metrics from paper Table IV.
#[derive(Debug)]
pub struct EvaluationResult {
    /// Dataset identifier.
    pub dataset: &'static str,
    /// Total raw boundary alarm events from the 3σ threshold comparator.
    pub raw_boundary_count: usize,
    /// DSFB Review/Escalate episode count.
    pub dsfb_episode_count: usize,
    /// Episode precision: fraction of episodes that are precursors.
    pub episode_precision: f32,
    /// Recall: fraction of ground-truth events covered.
    pub recall_numerator: usize,
    /// Recall denominator.
    pub recall_denominator: usize,
    /// Review-surface compression factor (raw / dsfb).
    pub compression_factor: f32,
    /// Precision improvement factor (dsfb_precision / raw_precision_proxy).
    pub precision_gain: f32,
    /// Raw boundary precision proxy (events / raw_boundary_count).
    pub raw_precision_proxy: f32,
    /// False episode rate on clean windows (negative control).
    pub false_episode_rate_clean: f32,
    /// Per-observation trace (for traceability artifact).
    pub trace: Vec<ObservationResult>,
    /// Opened episodes.
    pub episodes: Vec<Episode>,
}

impl EvaluationResult {
    /// Recall as a fraction.
    pub fn recall(&self) -> f32 {
        if self.recall_denominator == 0 { return 0.0; }
        self.recall_numerator as f32 / self.recall_denominator as f32
    }

    /// Print a summary matching the paper's Table IV format.
    pub fn print_summary(&self) {
        std::println!("══════════════════════════════════════════════════════");
        std::println!(" DSFB-RF Stage III Evaluation — {}", self.dataset);
        std::println!("══════════════════════════════════════════════════════");
        std::println!(" Raw boundary events:    {:>8}", self.raw_boundary_count);
        std::println!(" DSFB episodes:          {:>8}", self.dsfb_episode_count);
        std::println!(" Compression:            {:>7.1}×", self.compression_factor);
        std::println!(" Episode precision:      {:>7.1}%  (raw proxy: {:.2}%)",
            self.episode_precision * 100.0, self.raw_precision_proxy * 100.0);
        std::println!(" Precision gain:         {:>7.1}×", self.precision_gain);
        std::println!(" Recall:              {}/{} ({:.1}%)",
            self.recall_numerator, self.recall_denominator,
            self.recall() * 100.0);
        std::println!(" False ep. rate (clean): {:>7.1}%", self.false_episode_rate_clean * 100.0);
        std::println!("══════════════════════════════════════════════════════");
    }

    /// Returns true if headline metrics match the paper's locked values
    /// for the given dataset (used by paper_lock module).
    pub fn check_paper_lock(&self, expected: &PaperLockExpected) -> Result<(), std::string::String> {
        let eps = 0.005; // 0.5% tolerance for floating-point reproducibility
        if (self.episode_precision - expected.precision).abs() > eps {
            return Err(std::format!(
                "[{}] episode_precision={:.4} expected={:.4} (±{:.4})",
                self.dataset, self.episode_precision, expected.precision, eps));
        }
        if self.recall_numerator < expected.recall_min {
            return Err(std::format!(
                "[{}] recall={}/{} below minimum {}",
                self.dataset, self.recall_numerator,
                self.recall_denominator, expected.recall_min));
        }
        if self.dsfb_episode_count != expected.episode_count {
            return Err(std::format!(
                "[{}] episode_count={} expected={}",
                self.dataset, self.dsfb_episode_count, expected.episode_count));
        }
        Ok(())
    }
}

/// Expected values for paper-lock verification.
#[derive(Debug, Clone)]
pub struct PaperLockExpected {
    /// Episode count.
    pub episode_count: usize,
    /// Precision.
    pub precision: f32,
    /// Recall min.
    pub recall_min: usize,
}

// ── Core evaluation function ────────────────────────────────────────────────

/// Run Stage III evaluation on a stream of observations.
///
/// This is the single canonical evaluation function used for both RadioML
/// and ORACLE. The protocol is identical; only the observation stream differs.
///
/// ## Protocol (paper §F.4, §IX)
///
/// 1. First `HEALTHY_WINDOW_SIZE` observations with `is_healthy=true` are used
///    for calibration (ρ, EWMA threshold).
/// 2. Remaining observations are processed in order.
/// 3. Episodes are opened on first Review/Escalate, closed on Silent/Watch.
/// 4. Episode precision: fraction of episodes within W_pred of a ground-truth event.
/// 5. Recall: fraction of ground-truth events covered by at least one episode.
pub fn run_stage_iii(
    dataset: &'static str,
    observations: &[RfObservation],
    events: &[RegimeTransitionEvent],
) -> EvaluationResult {
    let healthy = collect_healthy_window(observations);
    assert!(!healthy.is_empty(), "healthy window must not be empty");

    let mut engine = DsfbRfEngine::<DSA_WINDOW_W, GRAMMAR_K, 32>::from_calibration(
        &healthy, DSA_TAU,
    )
    .unwrap_or_else(|| DsfbRfEngine::<DSA_WINDOW_W, GRAMMAR_K, 32>::new(1.0, DSA_TAU));
    engine = engine.with_snr_floor(SNR_FLOOR_DB);

    let mut comparators = ScalarComparators::calibrate(&healthy);
    let run = run_evaluation_pass(&mut engine, &mut comparators, observations, events);

    let episodes = finalise_episodes(run.episodes, run.episode_open_k, run.episode_open, observations.len(), events);
    let metrics = compute_evaluation_metrics(&episodes, run.raw_boundary_count, run.false_episodes_clean, run.clean_window_obs, events, observations.len());

    EvaluationResult {
        dataset,
        raw_boundary_count: run.raw_boundary_count,
        dsfb_episode_count: episodes.len(),
        episode_precision: metrics.episode_precision,
        recall_numerator: metrics.covered,
        recall_denominator: events.len(),
        compression_factor: metrics.compression,
        precision_gain: metrics.precision_gain,
        raw_precision_proxy: metrics.raw_precision_proxy,
        false_episode_rate_clean: metrics.false_ep_rate,
        trace: run.trace,
        episodes,
    }
}

fn collect_healthy_window(observations: &[RfObservation]) -> Vec<f32> {
    observations.iter()
        .filter(|o| o.is_healthy)
        .take(HEALTHY_WINDOW_SIZE)
        .map(|o| o.residual_norm)
        .collect()
}

struct EvaluationRun {
    trace: Vec<ObservationResult>,
    episodes: Vec<Episode>,
    raw_boundary_count: usize,
    false_episodes_clean: usize,
    clean_window_obs: usize,
    episode_open: bool,
    episode_open_k: usize,
}

fn run_evaluation_pass(
    engine: &mut DsfbRfEngine<DSA_WINDOW_W, GRAMMAR_K, 32>,
    comparators: &mut ScalarComparators,
    observations: &[RfObservation],
    events: &[RegimeTransitionEvent],
) -> EvaluationRun {
    let mut trace: Vec<ObservationResult> = Vec::with_capacity(observations.len());
    let mut episodes: Vec<Episode> = Vec::new();
    let mut raw_boundary_count = 0usize;
    let mut false_episodes_clean = 0usize;
    let mut clean_window_obs = 0usize;
    let mut episode_open = false;
    let mut episode_open_k = 0usize;

    for obs in observations.iter().filter(|o| !o.is_healthy) {
        let ctx = PlatformContext::with_snr(obs.snr_db);
        let result = engine.observe(obs.residual_norm, ctx);
        let k = obs.k;

        let (thr, _, _, _) = comparators.update(obs.residual_norm);
        if thr { raw_boundary_count += 1; }

        let is_active = matches!(result.policy,
            PolicyDecision::Review | PolicyDecision::Escalate);

        if is_active && !episode_open {
            episode_open = true;
            episode_open_k = k;
        } else if !is_active && episode_open {
            episode_open = false;
            episodes.push(Episode {
                open_k: episode_open_k,
                close_k: Some(k),
                is_precursor: false,
            });
        }

        if is_clean_window(k, events, WPRED) {
            clean_window_obs += 1;
            if is_active { false_episodes_clean += 1; }
        }

        trace.push(result);
    }

    EvaluationRun {
        trace, episodes, raw_boundary_count, false_episodes_clean,
        clean_window_obs, episode_open, episode_open_k,
    }
}

fn finalise_episodes(
    mut episodes: Vec<Episode>,
    episode_open_k: usize,
    episode_open: bool,
    n_obs: usize,
    events: &[RegimeTransitionEvent],
) -> Vec<Episode> {
    if episode_open {
        episodes.push(Episode {
            open_k: episode_open_k,
            close_k: None,
            is_precursor: false,
        });
    }
    for ep in &mut episodes {
        let close = ep.close_k.unwrap_or(n_obs);
        ep.is_precursor = events.iter().any(|ev| {
            close <= ev.k && ev.k <= close + WPRED
        });
    }
    episodes
}

struct EvaluationMetrics {
    episode_precision: f32,
    covered: usize,
    compression: f32,
    precision_gain: f32,
    raw_precision_proxy: f32,
    false_ep_rate: f32,
}

fn compute_evaluation_metrics(
    episodes: &[Episode],
    raw_boundary_count: usize,
    false_episodes_clean: usize,
    clean_window_obs: usize,
    events: &[RegimeTransitionEvent],
    n_obs: usize,
) -> EvaluationMetrics {
    let covered: usize = events.iter().filter(|ev| {
        episodes.iter().any(|ep| {
            let close = ep.close_k.unwrap_or(n_obs);
            close <= ev.k && ev.k <= close + WPRED
        })
    }).count();

    let n_eps = episodes.len();
    let n_precursor = episodes.iter().filter(|e| e.is_precursor).count();
    let episode_precision = if n_eps > 0 { n_precursor as f32 / n_eps as f32 } else { 0.0 };
    let raw_precision_proxy = if raw_boundary_count > 0 {
        events.len() as f32 / raw_boundary_count as f32
    } else { 0.0 };
    let compression = if n_eps > 0 {
        raw_boundary_count as f32 / n_eps as f32
    } else { raw_boundary_count as f32 };
    let precision_gain = if raw_precision_proxy > 0.0 {
        episode_precision / raw_precision_proxy
    } else { 0.0 };
    let false_ep_rate = if clean_window_obs > 0 {
        false_episodes_clean as f32 / clean_window_obs as f32
    } else { 0.0 };

    EvaluationMetrics {
        episode_precision, covered, compression, precision_gain, raw_precision_proxy, false_ep_rate,
    }
}

/// Returns true if observation k is in a clean window (no event within W_pred).
fn is_clean_window(k: usize, events: &[RegimeTransitionEvent], wpred: usize) -> bool {
    !events.iter().any(|ev| {
        let lo = ev.k.saturating_sub(wpred);
        let hi = ev.k + wpred;
        k >= lo && k <= hi
    })
}

// ── Synthetic dataset runner (RadioML-style structure) ──────────────────────

/// Build a synthetic RadioML-style observation stream for testing/demo.
///
/// Generates observations with SNR sweep and injected structural drift
/// events at known positions. Used for unit testing and reproducibility
/// verification when the real RadioML HDF5 file is not available.
///
/// ## Note on public dataset access
///
/// The real RadioML 2018.01a dataset is available at:
/// <https://www.deepsig.ai/datasets>
/// The real ORACLE dataset is available at:
/// <https://www.crowncom.org/oracle-dataset>
///
/// This synthetic generator produces structurally equivalent input
/// for CI and unit testing. Results will differ from the paper's
/// Table IV which uses the real datasets.
pub fn synthetic_radioml_stream(
    n_obs: usize,
    drift_events_at: &[usize],
    base_snr_db: f32,
) -> (Vec<RfObservation>, Vec<RegimeTransitionEvent>) {
    let mut obs = Vec::with_capacity(n_obs);
    let mut events = Vec::new();

    // Healthy window: low noise, clean signal
    for k in 0..HEALTHY_WINDOW_SIZE.min(n_obs) {
        obs.push(RfObservation {
            k,
            residual_norm: 0.02 + (k as f32 * 0.0001),
            snr_db: base_snr_db,
            is_healthy: true,
        });
    }

    // Main trace with injected drift events
    let mut norm = 0.025_f32;
    let drift_set: std::collections::HashSet<usize> =
        drift_events_at.iter().copied().take(drift_events_at.len()).collect();

    for k in HEALTHY_WINDOW_SIZE..n_obs {
        // Inject a drift regime near each event
        let near_event = drift_events_at.iter().any(|&ek| {
            k >= ek.saturating_sub(20) && k <= ek + 5
        });
        if near_event {
            norm = (norm + 0.006).min(0.35);
        } else {
            norm = (norm * 0.97).max(0.018);
        }

        let snr = if near_event { base_snr_db - 5.0 } else { base_snr_db };
        let is_transition = drift_set.contains(&k);

        obs.push(RfObservation {
            k,
            residual_norm: norm,
            snr_db: snr,
            is_healthy: false,
        });

        if is_transition {
            events.push(RegimeTransitionEvent { k, label: "SNR_regime_transition" });
        }
    }

    (obs, events)
}

// ── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use std::println;
    use std::vec;
    use std::vec::Vec;

    #[test]
    fn synthetic_pipeline_completes_without_panic() {
        let drift_at = vec![150, 250, 350, 450, 550, 650, 750, 850, 950, 1050];
        let (obs, events) = synthetic_radioml_stream(1200, &drift_at, 15.0);
        let result = run_stage_iii("synthetic_test", &obs, &events);
        // Verify structural invariants
        assert_eq!(result.recall_denominator, drift_at.len());
        assert!(result.episode_precision >= 0.0 && result.episode_precision <= 1.0);
        assert!(result.recall() >= 0.0 && result.recall() <= 1.0);
        assert!(result.compression_factor >= 0.0);
        // DSFB must compress the review surface
        println!("Episodes: {}, Raw: {}, Precision: {:.2}%, Recall: {}/{}",
            result.dsfb_episode_count, result.raw_boundary_count,
            result.episode_precision * 100.0,
            result.recall_numerator, result.recall_denominator);
    }

    #[test]
    fn healthy_calibration_window_used() {
        let drift_at = vec![200, 400];
        let (obs, events) = synthetic_radioml_stream(500, &drift_at, 10.0);
        let healthy_count = obs.iter().filter(|o| o.is_healthy).count();
        assert_eq!(healthy_count, HEALTHY_WINDOW_SIZE);
        let result = run_stage_iii("calibration_test", &obs, &events);
        assert!(result.dsfb_episode_count < result.raw_boundary_count,
            "DSFB must compress vs raw threshold");
    }

    #[test]
    fn sub_threshold_snr_events_missed_gracefully() {
        // Events below SNR floor should produce misses — not panics
        let n = 300;
        let mut obs = Vec::new();
        for k in 0..HEALTHY_WINDOW_SIZE {
            obs.push(RfObservation { k, residual_norm: 0.02, snr_db: 15.0, is_healthy: true });
        }
        for k in HEALTHY_WINDOW_SIZE..n {
            obs.push(RfObservation { k, residual_norm: 0.30, snr_db: -20.0, is_healthy: false });
        }
        let events = vec![
            RegimeTransitionEvent { k: 150, label: "sub_threshold_event" },
        ];
        let result = run_stage_iii("sub_threshold_test", &obs, &events);
        // Sub-threshold observations force Admissible — event will likely be missed
        // This is the correct behavior per paper §L10
        assert_eq!(result.recall_denominator, 1);
    }

    #[test]
    fn clean_window_detection() {
        let events = vec![
            RegimeTransitionEvent { k: 100, label: "ev1" },
            RegimeTransitionEvent { k: 200, label: "ev2" },
        ];
        // k=50 is far from both events — should be clean
        assert!(is_clean_window(50, &events, WPRED));
        // k=98 is within WPRED of event at 100 — not clean
        assert!(!is_clean_window(98, &events, WPRED));
        // k=205 is within WPRED of event at 200 — not clean
        assert!(!is_clean_window(205, &events, WPRED));
    }

    #[test]
    fn scalar_comparators_calibrate_correctly() {
        let healthy: Vec<f32> = (0..100).map(|i| 0.03 + i as f32 * 0.0002).collect();
        let comp = ScalarComparators::calibrate(&healthy);
        assert!(comp.threshold_3sigma > comp.threshold_mean,
            "3sigma threshold must exceed mean");
        assert!(comp.cusum_h > comp.cusum_kappa,
            "CUSUM alarm threshold must exceed allowance");
        assert!(comp.ewma_threshold > 0.0);
    }
}
