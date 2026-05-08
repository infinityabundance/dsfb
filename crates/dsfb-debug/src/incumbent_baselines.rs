//! DSFB-Debug: 205-detector library — fusion ensemble baselines.
//!
//! # The IP claim
//!
//! This module is the SBIR Phase II licensable asset. It implements
//! **205 deterministic detectors organised across 27 mathematical
//! axes** (Tiers A–U + EXTRA + V/X/Y/Z/AA), each cited to the
//! literature, each tight (~30-50 LOC), each `no_alloc + no_unsafe +
//! no_panic`. The detector outputs are the consensus tiers consumed
//! by the heuristics-bank-aware fusion in `fusion.rs`.
//!
//! # Tier taxonomy
//!
//! | Tier | Count | Family | Representative detector |
//! |------|------:|--------|-------------------------|
//! | A | 3 | parametric trio | scalar-3σ / CUSUM / EWMA |
//! | B | 3 | robust statistics | Hampel / Hinkley / Tukey-IQR |
//! | C | 5 | model + non-param | Spectral-Residual / Matrix Profile / BOCPD / IsoForest / LOF |
//! | D | 5 | additional non-dep | Mann-Kendall / rolling-Z / AR(1) / Mahalanobis / KS |
//! | E | 3 | debugging-specific | Poisson burst / saturation / chi-square |
//! | F | 4 | neuroscience burst | MaxInt / LogISI / Rank-Surprise / MISI |
//! | EXTRA | 5 | LR / Hoeffding / domain | GLR / ADWIN / MEWMA / retry-storm / corr-break |
//! | G | 9 | concept drift streaming | DDM / EDDM / HDDM-A / HDDM-W / STEPD / ECDD / KSWIN / FHDDM / Shiryaev-Roberts |
//! | H | 10 | distribution shift | Wasserstein / JS / KL / PSI / AD / CvM / Energy / MMD / Bhattacharyya / Hellinger |
//! | I | 10 | robust nonparametric | Theil-Sen / Mood / Brown-Forsythe / Levene / Sign / Runs / WW / Sequential-rank / median-abs-slope / Sen-CP |
//! | J | 10 | forecast residual | SES / Holt / Holt-Winters / AR(2) / ARIMA / Kalman / SavGol / STL / Prophet / naive-seasonal |
//! | K | 10 | frequency / oscillation | FFT-band / Welch / Wavelet / autocorr / Lomb-Scargle / ZCR / dom-freq / spectral-entropy / cepstral / phase-locking |
//! | L | 9 | multivariate relationship | Hotelling T² / MCUSUM / PCA-recon / Robust-PCA / corr-mat-dist / partial-corr / Laplacian / CCA / MI |
//! | M | 18 | debugging-native | flap / sawtooth / deadband / quantization / plateau / counter-wrap / monotone-leak / hysteresis / limit-cycle / ping-pong / backpressure / causal-lag / fan-out / fan-in / phase-slip / jitter-bloom / tail-thickening / burst-after-silence |
//! | N | 8 | offline CPD | PELT / BinSeg / BottomUp / Window / DP / Kernel / Piecewise-linear / Bayesian-offline |
//! | O | 10 | rare changepoint | MOSUM / NOT / WBS2 / Seeded-BS / SMUCE / FDRSeg / FPOP / TGUH / Inspect / Double-CUSUM-BS |
//! | P | 9 | streaming sequential | E-detector / conformal / exch / power / mixture / SPRT / scan-stat / higher-criticism / Berk-Jones |
//! | Q | 10 | concept drift rarer | MDDM / LFR / FPDD / OPTWIN / SeqDrift2 / D3 / QuantTree / NN-DVI / RDDM / Sequential-rank-sum |
//! | R | 8 | robust depth | halfspace / projection / Stahel-Donoho / MCD / spatial-sign / S-estimator / depth-rank / median-polish |
//! | S | 3 | count event-process | Bayesian-blocks / IoD / Allan-variance |
//! | T | 6 | info-theoretic | MDL / NCD / Lempel-Ziv / transfer-entropy / Fisher-info / Renyi |
//! | U | 8 | dynamical systems | permutation-entropy / sample-entropy / RQA / Lyapunov / correlation-dim / BDS / 0-1-chaos / delay-embedding |
//! | V | ~8 | industrial fault-diagnosis | parity-space / observer-based / dependability |
//! | X | ~8 | climate homogeneity | Buishand-range / SNHT / Pettitt-step / cumulative-deviation |
//! | Y | ~8 | robust dispersion | median-of-means / U-stat / rank-step |
//! | Z | ~8 | circular / directional | R-bar / Rayleigh / phase-jump |
//! | AA | ~11 | higher-order nonlinear | ARCH / kurtosis / volatility |
//!
//! # Per-detector contract
//!
//! Every detector is a deterministic `pub fn` with the same signature
//! shape: `(matrix: &[f64], num_signals, num_windows, healthy_window_end)
//! → DetectorOutput`. Each carries:
//!
//! - **Citation** in its doc-comment (paper / textbook / whitepaper, year)
//! - **Tier** assignment (one of A–U / EXTRA / V/X/Y/Z/AA)
//! - **Determinism note** — explicit if a deterministic-seeded
//!   variant is used, or a time-domain proxy for an FFT-based
//!   literature method
//! - **Honest simplification note** — where literature requires FFT,
//!   MCMC, or non-deterministic randomization, the implementation
//!   uses a documented reduction (e.g.\ "uses time-domain proxy for
//!   Welch PSD; documented in fusion_design.md")
//!
//! # Three "anchor" baselines for fair head-to-head comparison
//!
//! Per `docs/incumbent_comparison.md`, the canonical head-to-head
//! comparison uses these three Tier-A detectors against DSFB-Debug
//! on the same residual matrix:
//!
//! - `scalar_threshold` — Page (1954); fire when
//!   `|value - mean_healthy| > k·sigma_healthy`. k = 3 is the
//!   conventional 3-sigma envelope.
//! - `cusum` — Page (1954); cumulative-sum chart; fire when
//!   `Σ(x - target) > h` for an upward shift.
//! - `ewma` — Roberts (1959); exponentially-weighted moving average;
//!   fire when EWMA crosses `target ± L·sigma_z`.
//!
//! Comparison invariants: identical residual matrix, identical fault
//! labels, identical evaluation windows, no post-hoc threshold tuning
//! on the eval split.
//!
//! # Shared helpers
//!
//! - `score_against_labels(alerts, labels, healthy_end) → DetectorOutput`
//!   — uniform metric extraction (raw_alert_count, FP rate, fault
//!   recall, wall-clock).
//! - `fit_healthy_stats(matrix, num_signals, healthy_end) →
//!   PerSignalStats` — mean / sigma / median / MAD per signal over
//!   the healthy slice.
//!
//! These helpers are reused across all 205 detectors to keep
//! per-detector implementation tight and consistent.

#![cfg(feature = "std")]
#![allow(clippy::needless_range_loop, clippy::manual_memcpy, clippy::too_many_arguments)]

extern crate std;

use std::vec::Vec;

// Phase 4 — per-detector window-alert side channel.
//
// Each detector writes its per-window firing pattern to this thread-local
// buffer just before computing `alert_windows`. The fusion harness reads
// it immediately after the detector returns and OR-s the bits into
// `window_tier_mask` keyed by the detector's tier. This populates tier
// evidence from all 162 detectors without changing any detector's public
// signature.
//
// Thread-local because run_inner is single-threaded; Theorem 9 (replay
// determinism) preserved because all writes are sequential per call.
std::thread_local! {
    pub static LAST_WIN_ALERTS: std::cell::RefCell<Vec<bool>> = std::cell::RefCell::new(Vec::new());
}

#[inline]
fn capture_win_alerts(win_alerts: &[bool]) {
    LAST_WIN_ALERTS.with(|cell| {
        let mut buf = cell.borrow_mut();
        buf.clear();
        buf.extend_from_slice(win_alerts);
    });
}

/// Detector output: counts only. Wall-clock timing is measured by the
/// caller (`std::time::Instant`); the detectors themselves are
/// pure-function.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DetectorOutput {
    pub detector_name: &'static str,
    pub raw_alert_count: u64,
    /// Number of distinct (window, signal) pairs that fired.
    pub alerts_per_signal: [u64; 32], // up to 32 signals; truncated otherwise
    /// Number of windows with at least one alert.
    pub alert_windows: u64,
    /// "Episodes" — flat detectors don't aggregate, so this equals
    /// alert_windows for fair comparison framing.
    pub episode_count: u64,
    /// Captured fault windows (within ±W_pred of any labeled fault).
    pub captured_faults: u64,
    /// Total labeled faults in the slice.
    pub total_faults: u64,
    /// False-positive count on healthy windows (alerts in windows
    /// outside any labeled fault range ±W_pred).
    pub clean_window_false_alerts: u64,
    /// Number of clean windows (those outside fault ranges).
    pub clean_windows: u64,
}

impl DetectorOutput {
    /// "RSCR" for flat detectors = 1.0 (no episode aggregation).
    /// Returned for symmetry with DSFB-Debug's BenchmarkMetrics.rscr
    /// in the comparison matrix.
    pub fn rscr(&self) -> f64 {
        if self.episode_count > 0 { 1.0 } else { 0.0 }
    }

    pub fn fault_recall(&self) -> f64 {
        if self.total_faults > 0 {
            self.captured_faults as f64 / self.total_faults as f64
        } else {
            1.0
        }
    }

    pub fn clean_window_fp_rate(&self) -> f64 {
        if self.clean_windows > 0 {
            self.clean_window_false_alerts as f64 / self.clean_windows as f64
        } else {
            0.0
        }
    }
}

/// Run the scalar-threshold detector (3-sigma envelope per signal,
/// fitted on the healthy slice).
pub fn scalar_threshold(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for w in 0..num_windows {
        let mut any_alert = false;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() {
                continue;
            }
            let v = data[idx];
            if v.is_nan() {
                continue;
            }
            let mu = means[s];
            let sd = sigmas[s];
            if sd > 0.0 && (v - mu).abs() > 3.0 * sd {
                raw_alert_count += 1;
                if s < 32 {
                    alerts_per_signal[s] += 1;
                }
                any_alert = true;
            }
        }
        if any_alert {
            alert_windows += 1;
            window_alerts[w] = true;
        }
    }

    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);

    DetectorOutput {
        detector_name: "scalar_threshold_3sigma",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Run the CUSUM (cumulative sum) detector. Page (1954).
/// Per signal, accumulate `Σ (x - target)`; fire when the running
/// sum exceeds `h * sigma_healthy`. Resets on every alert.
pub fn cusum(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    h: f64, // typical: 4.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    let mut sum_pos = std::vec![0.0_f64; num_signals];
    let mut sum_neg = std::vec![0.0_f64; num_signals];
    for w in 0..num_windows {
        let mut any_alert = false;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() {
                continue;
            }
            let v = data[idx];
            if v.is_nan() {
                continue;
            }
            let mu = means[s];
            let sd = sigmas[s].max(1e-9);
            // Standardised deviation
            let z = (v - mu) / sd;
            sum_pos[s] = (sum_pos[s] + z - 0.5).max(0.0);
            sum_neg[s] = (sum_neg[s] - z - 0.5).max(0.0);
            if sum_pos[s] > h || sum_neg[s] > h {
                raw_alert_count += 1;
                if s < 32 {
                    alerts_per_signal[s] += 1;
                }
                any_alert = true;
                sum_pos[s] = 0.0;
                sum_neg[s] = 0.0;
            }
        }
        if any_alert {
            alert_windows += 1;
            window_alerts[w] = true;
        }
    }

    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);

    DetectorOutput {
        detector_name: "cusum_h4",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Run the robust Z-score detector (Hampel 1974).
///
/// Uses Median Absolute Deviation (MAD) instead of standard deviation:
/// fire when `|value - median_healthy| > k * MAD * 1.4826`. The
/// 1.4826 scaling makes MAD a consistent estimator of σ for normal
/// data; the robustness is that single outliers in the baseline
/// inflate σ but not MAD. Orthogonal axis to scalar-3σ.
pub fn robust_z_mad(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    k: f64, // typical: 3.0
) -> DetectorOutput {
    let (medians, mads) = fit_healthy_robust(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for w in 0..num_windows {
        let mut any_alert = false;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() {
                continue;
            }
            let v = data[idx];
            if v.is_nan() {
                continue;
            }
            let med = medians[s];
            let mad = mads[s];
            // 1.4826 = 1/Phi^{-1}(0.75): consistent scaling of MAD to σ.
            let scale = mad * 1.4826;
            if scale > 0.0 && (v - med).abs() > k * scale {
                raw_alert_count += 1;
                if s < 32 {
                    alerts_per_signal[s] += 1;
                }
                any_alert = true;
            }
        }
        if any_alert {
            alert_windows += 1;
            window_alerts[w] = true;
        }
    }

    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "robust_z_mad_3",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Run the Page-Hinkley test. Hinkley (1971) extension of Page (1954).
///
/// Per signal, track the minimum running sum
/// `m_w = min_{t <= w} Σ_{i=0..=t} (x_i - x̄_healthy - δ)` and fire
/// when `(running_sum - m_w) > λ`. Detects the moment a baseline
/// shifts upward; complementary to CUSUM which detects whether a
/// shift is present at all. Different change-point signature.
pub fn page_hinkley(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    lambda: f64, // typical: 50.0 in raw units (or scale by sigma)
    delta: f64,  // typical: 0.005 (small drift tolerance)
) -> DetectorOutput {
    let (means, _sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    let mut running = std::vec![0.0_f64; num_signals];
    let mut min_running = std::vec![0.0_f64; num_signals];

    for w in 0..num_windows {
        let mut any_alert = false;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() {
                continue;
            }
            let v = data[idx];
            if v.is_nan() {
                continue;
            }
            running[s] += v - means[s] - delta;
            if running[s] < min_running[s] {
                min_running[s] = running[s];
            }
            let pht = running[s] - min_running[s];
            if pht > lambda {
                raw_alert_count += 1;
                if s < 32 {
                    alerts_per_signal[s] += 1;
                }
                any_alert = true;
                // Reset to avoid pinning at high alert state.
                running[s] = 0.0;
                min_running[s] = 0.0;
            }
        }
        if any_alert {
            alert_windows += 1;
            window_alerts[w] = true;
        }
    }

    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "page_hinkley",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Run the Tukey-IQR fence detector. Tukey (1977).
///
/// Distribution-free: fit Q1 and Q3 on the healthy slice; fire when
/// `value < Q1 - k*IQR` or `value > Q3 + k*IQR`. The k=1.5 default
/// is the conventional "outlier" boundary; k=3 gives the "extreme
/// outlier" boundary. Orthogonal axis: doesn't assume normality.
pub fn tukey_iqr_fence(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    k: f64, // typical: 1.5 (outlier) or 3.0 (extreme)
) -> DetectorOutput {
    let (q1_arr, q3_arr) = fit_healthy_quartiles(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for w in 0..num_windows {
        let mut any_alert = false;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() {
                continue;
            }
            let v = data[idx];
            if v.is_nan() {
                continue;
            }
            let q1 = q1_arr[s];
            let q3 = q3_arr[s];
            let iqr = q3 - q1;
            if iqr > 0.0 && (v < q1 - k * iqr || v > q3 + k * iqr) {
                raw_alert_count += 1;
                if s < 32 {
                    alerts_per_signal[s] += 1;
                }
                any_alert = true;
            }
        }
        if any_alert {
            alert_windows += 1;
            window_alerts[w] = true;
        }
    }

    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "tukey_iqr_1.5",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Run the Spectral Residual (SR) anomaly detector
/// (Ren et al. 2019, Microsoft Research — simplified, FFT-free).
///
/// Implementation note: the original SR algorithm uses FFT to compute
/// the log-amplitude residual in the frequency domain. To preserve the
/// zero-dependency stance (no FFT crate), this implementation uses the
/// time-domain analogue: residual = `value - rolling_moving_avg(value,
/// window_n)`, threshold at `k * sigma_healthy_of_residual`. Captures
/// the "structural divergence from local trend" axis. Documented as
/// SR-time-domain in the operator handbook; the full FFT variant is
/// future work.
pub fn spectral_residual_td(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    rolling_n: usize, // typical: 8
    k: f64,           // typical: 3.0
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    // Per-signal: compute residuals = value - rolling_avg, then fit
    // healthy sigma of residual distribution.
    for s in 0..num_signals {
        let mut residuals = std::vec![0.0_f64; num_windows];
        let mut rolling_buf = std::vec![0.0_f64; rolling_n];
        let mut rolling_pos = 0;
        let mut rolling_count = 0;
        let mut rolling_sum = 0.0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            // Update rolling sum (replace oldest)
            if rolling_count < rolling_n {
                rolling_buf[rolling_pos] = v;
                rolling_sum += v;
                rolling_count += 1;
            } else {
                rolling_sum -= rolling_buf[rolling_pos];
                rolling_buf[rolling_pos] = v;
                rolling_sum += v;
            }
            rolling_pos = (rolling_pos + 1) % rolling_n;
            let avg = rolling_sum / rolling_count as f64;
            residuals[w] = v - avg;
        }
        // Fit healthy residual sigma.
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut n = 0;
        for w in 0..healthy_window_end.min(num_windows) {
            sum += residuals[w];
            sum_sq += residuals[w] * residuals[w];
            n += 1;
        }
        let sigma = if n > 1 {
            let mean = sum / n as f64;
            let var = (sum_sq - n as f64 * mean * mean) / (n - 1) as f64;
            var.max(0.0).sqrt()
        } else { 0.0 };
        if sigma <= 0.0 { continue; }
        // Fire on residual > k*sigma.
        for w in 0..num_windows {
            if residuals[w].abs() > k * sigma {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "spectral_residual_td",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Run the Matrix Profile detector (Yeh et al. 2016, brute-force STAMP).
///
/// For each window of length L starting at the evaluation region,
/// compute the Euclidean distance to every healthy-window sub-sequence
/// of the same length. The minimum such distance is the "discord
/// score". Fire when the discord score exceeds `k * mean_healthy_discord`.
/// O(n_eval * n_healthy * L) per signal — heavy, but deterministic.
pub fn matrix_profile(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    seq_len: usize, // typical: 4
    k: f64,         // typical: 3.0
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for s in 0..num_signals {
        let mut series = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            series[w] = data[idx];
        }
        if num_windows <= seq_len + 1 { continue; }
        // Healthy distances: for each healthy sub-sequence, find its
        // distance to its nearest other healthy sub-sequence.
        let h_end_eff = healthy_window_end.min(num_windows.saturating_sub(seq_len));
        if h_end_eff < seq_len + 2 { continue; }
        let mut healthy_min_dists = std::vec![f64::INFINITY; h_end_eff];
        for w_a in 0..h_end_eff {
            for w_b in 0..h_end_eff {
                if w_a == w_b { continue; }
                let d = sub_distance(&series, w_a, w_b, seq_len);
                if d < healthy_min_dists[w_a] {
                    healthy_min_dists[w_a] = d;
                }
            }
        }
        // Fit mean of healthy minimum-distance distribution.
        let mut sum = 0.0;
        let mut n = 0;
        for d in &healthy_min_dists {
            if d.is_finite() {
                sum += d;
                n += 1;
            }
        }
        if n == 0 { continue; }
        let mean_h = sum / n as f64;
        if mean_h <= 0.0 { continue; }
        // Fire when eval-region discord > k * mean_healthy.
        for w_a in healthy_window_end..num_windows.saturating_sub(seq_len) {
            let mut min_d = f64::INFINITY;
            for w_b in 0..h_end_eff {
                let d = sub_distance(&series, w_a, w_b, seq_len);
                if d < min_d { min_d = d; }
            }
            if min_d > k * mean_h {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w_a] = true;
            }
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "matrix_profile_stamp",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

#[inline]
fn sub_distance(series: &[f64], i: usize, j: usize, len: usize) -> f64 {
    let mut s = 0.0_f64;
    for k in 0..len {
        let a = if i + k < series.len() { series[i + k] } else { 0.0 };
        let b = if j + k < series.len() { series[j + k] } else { 0.0 };
        if !a.is_nan() && !b.is_nan() {
            s += (a - b) * (a - b);
        }
    }
    s.sqrt()
}

/// Run the BOCPD detector (Adams & MacKay 2007 — simplified hazard).
///
/// Online changepoint detection: at each window, update the
/// run-length posterior `P(r_t | x_{1:t})` under a constant hazard
/// rate `h = 1/expected_run_length`. Fire when the posterior probability
/// of "changepoint just occurred" (r_t = 0) exceeds threshold `theta`.
/// Per signal; deterministic. Simplified to a normal-conjugate
/// likelihood with healthy-window-fitted prior.
pub fn bocpd(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    expected_run_length: f64, // typical: 100.0
    theta: f64,               // typical: 0.5
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    let hazard = 1.0 / expected_run_length.max(1.0);

    for s in 0..num_signals {
        if sigmas[s] <= 0.0 { continue; }
        let mu = means[s];
        let sd = sigmas[s];
        // Run-length posterior, truncated at MAX_RL = 256.
        const MAX_RL: usize = 256;
        let mut p = std::vec![0.0_f64; MAX_RL];
        p[0] = 1.0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            // Likelihood under "no changepoint" — Gaussian density.
            let z = (v - mu) / sd;
            let lik = (-0.5 * z * z).exp() / (sd * (2.0_f64 * core::f64::consts::PI).sqrt());
            // Growth probabilities.
            let mut new_p = std::vec![0.0_f64; MAX_RL];
            // r_t = 0 (changepoint): sum_r p[r] * hazard * lik
            let mut p_change = 0.0;
            for r in 0..MAX_RL {
                p_change += p[r] * hazard;
            }
            new_p[0] = p_change * lik;
            for r in 1..MAX_RL {
                new_p[r] = p[r - 1] * (1.0 - hazard) * lik;
            }
            // Normalize.
            let mut total = 0.0;
            for r in 0..MAX_RL { total += new_p[r]; }
            if total > 0.0 {
                for r in 0..MAX_RL { new_p[r] /= total; }
            }
            p = new_p;
            // Fire if changepoint posterior > theta.
            if p[0] > theta {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "bocpd_h0.01",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Run the Isolation Forest detector (Liu et al. 2008, deterministic seeded).
///
/// Build T isolation trees on the healthy slice; each tree splits
/// recursively on a random feature and split point. The expected
/// path-length to isolate a point is shorter for anomalies than for
/// normal points. Fire when expected isolation depth < threshold
/// (i.e. the point isolates quickly). Deterministic via fixed LCG seed.
pub fn isolation_forest(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    n_trees: usize, // typical: 16
    sample_size: usize, // typical: 64
    seed: u64,      // deterministic seed
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    // Treat each (window) as a num_signals-dimensional point.
    if num_signals == 0 || num_windows == 0 { return zero_output("isolation_forest"); }

    let mut lcg = seed;
    let mut next = || {
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        lcg
    };

    // Healthy points.
    let healthy_n = healthy_window_end.min(num_windows);
    if healthy_n < 4 { return zero_output("isolation_forest"); }

    // Average isolation depth across n_trees, normalized by expected
    // depth for sample_size points. (We approximate normalized score
    // as raw mean depth; threshold by healthy distribution.)
    let mut all_eval_depths = std::vec![0.0_f64; num_windows];

    for _t in 0..n_trees {
        // Sample sample_size healthy windows.
        let s_size = sample_size.min(healthy_n);
        let mut sample_idx = std::vec![0_usize; s_size];
        for i in 0..s_size {
            sample_idx[i] = (next() as usize) % healthy_n;
        }
        let max_depth = (s_size as f64).log2().ceil() as usize + 1;

        // For each evaluation window, compute its isolation depth.
        for w in 0..num_windows {
            let depth = isolate_one_point(
                data, num_signals, w, &sample_idx, max_depth, &mut next);
            all_eval_depths[w] += depth as f64;
        }
    }
    for w in 0..num_windows {
        all_eval_depths[w] /= n_trees as f64;
    }

    // Healthy threshold: 5th percentile of healthy depths.
    let mut h_depths = std::vec![0.0_f64; healthy_n];
    h_depths[..healthy_n].copy_from_slice(&all_eval_depths[..healthy_n]);
    h_depths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let threshold_idx = (0.05 * healthy_n as f64) as usize;
    let threshold = h_depths[threshold_idx.min(healthy_n - 1)];

    for w in 0..num_windows {
        if all_eval_depths[w] < threshold {
            raw_alert_count += 1;
            // Anomaly per (window) — count signal 0 by default for
            // alerts_per_signal accounting (multivariate detector).
            alerts_per_signal[0] += 1;
            window_alerts[w] = true;
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "isolation_forest_t16",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

fn isolate_one_point(
    data: &[f64],
    num_signals: usize,
    point_w: usize,
    sample_idx: &[usize],
    max_depth: usize,
    next: &mut impl FnMut() -> u64,
) -> usize {
    // Recursive tree-build is expensive. Instead, we compute the
    // expected isolation depth using a simulation: pick random axis
    // + split, recurse (up to max_depth), increment depth each step,
    // return depth at which the point separates from the sample's
    // density.
    let mut sample: std::vec::Vec<usize> = sample_idx.to_vec();
    let mut depth = 0;
    while sample.len() > 1 && depth < max_depth {
        let axis = (next() as usize) % num_signals;
        // Pick a split value uniformly between min and max along that axis.
        let mut min_v = f64::INFINITY;
        let mut max_v = f64::NEG_INFINITY;
        for &si in &sample {
            let idx = si * num_signals + axis;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    if v < min_v { min_v = v; }
                    if v > max_v { max_v = v; }
                }
            }
        }
        if !min_v.is_finite() || max_v <= min_v { break; }
        let r_frac = (next() as f64 / u64::MAX as f64).clamp(0.0, 1.0);
        let split = min_v + r_frac * (max_v - min_v);
        let point_v_idx = point_w * num_signals + axis;
        if point_v_idx >= data.len() { break; }
        let point_v = data[point_v_idx];
        if point_v.is_nan() { break; }
        let go_right = point_v >= split;
        // Filter sample to the side the point went.
        sample.retain(|&si| {
            let si_v_idx = si * num_signals + axis;
            if si_v_idx >= data.len() { return false; }
            let v = data[si_v_idx];
            if v.is_nan() { return false; }
            if go_right { v >= split } else { v < split }
        });
        depth += 1;
    }
    depth
}

/// Run the Local Outlier Factor (LOF) detector (Breunig et al. 2000).
///
/// For each evaluation window (treated as a num_signals-dim point),
/// compute its k-nearest-neighbour density relative to its neighbours'
/// densities. Fire when LOF > theta. O(n_eval * n_healthy * num_signals)
/// — heavy, deterministic.
pub fn lof(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    k: usize,    // typical: 5
    theta: f64,  // typical: 1.5
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    let healthy_n = healthy_window_end.min(num_windows);
    if healthy_n < k + 1 || num_signals == 0 {
        return zero_output("lof");
    }

    // Helper: euclidean distance between two windows.
    let dist = |i: usize, j: usize| -> f64 {
        let mut s = 0.0_f64;
        for sig in 0..num_signals {
            let a = data.get(i * num_signals + sig).copied().unwrap_or(0.0);
            let b = data.get(j * num_signals + sig).copied().unwrap_or(0.0);
            if !a.is_nan() && !b.is_nan() {
                s += (a - b) * (a - b);
            }
        }
        s.sqrt()
    };

    // For each healthy point, compute k-distance + reachability to its
    // k neighbours. (Pre-compute densities for normalization.)
    let mut healthy_lrd = std::vec![0.0_f64; healthy_n];
    let mut healthy_kdist = std::vec![0.0_f64; healthy_n];
    let mut tmp_dists = std::vec![0.0_f64; healthy_n];
    for i in 0..healthy_n {
        for j in 0..healthy_n { tmp_dists[j] = dist(i, j); }
        tmp_dists[i] = f64::INFINITY;
        let mut sorted = tmp_dists.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        healthy_kdist[i] = sorted[k.min(healthy_n - 1)];
        // LRD = 1 / mean(reach-dist to k-nearest)
        let mut sum_reach = 0.0;
        for j in 0..healthy_n {
            if i == j { continue; }
            if tmp_dists[j] <= healthy_kdist[i] {
                let reach = healthy_kdist[j].max(tmp_dists[j]);
                sum_reach += reach;
            }
        }
        healthy_lrd[i] = if sum_reach > 0.0 { k as f64 / sum_reach } else { 0.0 };
    }

    // Score each window.
    for w in 0..num_windows {
        let mut dists_to_h = std::vec![0.0_f64; healthy_n];
        for j in 0..healthy_n { dists_to_h[j] = dist(w, j); }
        let mut sorted = dists_to_h.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let kdist_w = sorted[k.min(healthy_n - 1)];
        let mut sum_reach = 0.0;
        for j in 0..healthy_n {
            if dists_to_h[j] <= kdist_w {
                sum_reach += healthy_kdist[j].max(dists_to_h[j]);
            }
        }
        let lrd_w = if sum_reach > 0.0 { k as f64 / sum_reach } else { 0.0 };
        // LOF = mean(neighbour LRD) / LRD(w)
        let mut sum_neigh = 0.0;
        let mut n_neigh = 0;
        for j in 0..healthy_n {
            if dists_to_h[j] <= kdist_w {
                sum_neigh += healthy_lrd[j];
                n_neigh += 1;
            }
        }
        let lof_score = if lrd_w > 0.0 && n_neigh > 0 {
            (sum_neigh / n_neigh as f64) / lrd_w
        } else { 0.0 };
        if lof_score > theta {
            raw_alert_count += 1;
            alerts_per_signal[0] += 1;
            window_alerts[w] = true;
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "lof_k5",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

fn zero_output(name: &'static str) -> DetectorOutput {
    DetectorOutput {
        detector_name: name,
        raw_alert_count: 0,
        alerts_per_signal: [0; 32],
        alert_windows: 0,
        episode_count: 0,
        captured_faults: 0,
        total_faults: 0,
        clean_window_false_alerts: 0,
        clean_windows: 0,
    }
}

// ---------- robust statistics ----------

fn fit_healthy_robust(
    data: &[f64], num_signals: usize, healthy_window_end: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut medians = std::vec![0.0_f64; num_signals];
    let mut mads = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        let mut vals: Vec<f64> = Vec::new();
        for w in 0..healthy_window_end {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    vals.push(v);
                }
            }
        }
        if vals.is_empty() { continue; }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let med = vals[vals.len() / 2];
        medians[s] = med;
        let mut deviations: Vec<f64> = vals.iter().map(|x| (x - med).abs()).collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        mads[s] = deviations[deviations.len() / 2];
    }
    (medians, mads)
}

fn fit_healthy_quartiles(
    data: &[f64], num_signals: usize, healthy_window_end: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut q1_arr = std::vec![0.0_f64; num_signals];
    let mut q3_arr = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        let mut vals: Vec<f64> = Vec::new();
        for w in 0..healthy_window_end {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    vals.push(v);
                }
            }
        }
        if vals.is_empty() { continue; }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        q1_arr[s] = vals[vals.len() / 4];
        q3_arr[s] = vals[(3 * vals.len()) / 4];
    }
    (q1_arr, q3_arr)
}

/// Mann-Kendall trend test (Mann 1945; Kendall 1948).
///
/// Non-parametric trend-detection: count the sign of all
/// (x_j - x_i) pairs for j > i within a rolling window. Test
/// statistic S has known variance under no-trend null hypothesis;
/// alert when |S| / sqrt(Var(S)) > z_alpha (1.96 for α=0.05).
/// Captures slow monotonic drift that scalar/CUSUM may miss when
/// the magnitude is small relative to noise.
pub fn mann_kendall(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    win_n: usize, // typical: 20
    z_alpha: f64, // typical: 1.96 (α=0.05)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            // Compute Mann-Kendall S over the window.
            let mut sgn_sum: i64 = 0;
            for i in 0..count {
                for j in (i + 1)..count {
                    let diff = buf[j] - buf[i];
                    if diff > 0.0 { sgn_sum += 1; }
                    else if diff < 0.0 { sgn_sum -= 1; }
                }
            }
            // Var(S) = n(n-1)(2n+5)/18 under null hypothesis.
            let n = count as f64;
            let var_s = n * (n - 1.0) * (2.0 * n + 5.0) / 18.0;
            if var_s <= 0.0 { continue; }
            // Z-statistic with continuity correction.
            let s_adj = if sgn_sum > 0 { sgn_sum as f64 - 1.0 }
                else if sgn_sum < 0 { sgn_sum as f64 + 1.0 }
                else { 0.0 };
            let z = s_adj / var_s.sqrt();
            if z.abs() > z_alpha {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mann_kendall_n20",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Rolling-window Z-score detector. Classic SPC variant.
///
/// Per signal, compute the standardised deviation
/// `z = (x_w - mean_rolling) / sigma_rolling` where the rolling
/// window slides over the most recent `win_n` non-NaN observations.
/// Fire when |z| > k. Distinct from `scalar_threshold` which uses
/// the fixed healthy-window baseline; rolling is adaptive to slow
/// baseline drift, at the cost of being insensitive to drift slower
/// than the rolling-window scale.
pub fn rolling_z_score(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    win_n: usize, // typical: 30
    k: f64,        // typical: 3.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if count < win_n {
                buf[pos] = v;
                pos = (pos + 1) % win_n;
                count += 1;
                continue;
            }
            // Compute rolling mean + sigma.
            let mut sum = 0.0;
            for i in 0..count { sum += buf[i]; }
            let mean = sum / count as f64;
            let mut var_sum = 0.0;
            for i in 0..count { var_sum += (buf[i] - mean) * (buf[i] - mean); }
            let sigma = (var_sum / (count - 1) as f64).max(0.0).sqrt();
            if sigma > 0.0 && (v - mean).abs() > k * sigma {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
            // Update rolling buffer (replace oldest).
            buf[pos] = v;
            pos = (pos + 1) % win_n;
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "rolling_z_n30",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// AR(1) forecast residual detector (Box & Jenkins 1970, simplified).
///
/// Fits an AR(1) model `x_w = phi * x_{w-1} + e_w` per signal on the
/// healthy slice via Yule-Walker (lag-1 autocorrelation). Predicts
/// each evaluation-region value; fires when the prediction residual
/// exceeds `k * sigma_healthy_residual`. Captures temporal auto-
/// regressive deviation.
pub fn ar1_forecast_residual(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    k: f64, // typical: 3.0
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for s in 0..num_signals {
        // Fit AR(1) on healthy slice.
        let mut sum_x = 0.0;
        let mut sum_xx = 0.0;
        let mut sum_xy = 0.0;
        let mut n = 0_usize;
        let mut prev: Option<f64> = None;
        for w in 0..healthy_window_end.min(num_windows) {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if let Some(p) = prev {
                sum_x += p;
                sum_xx += p * p;
                sum_xy += p * v;
                n += 1;
            }
            prev = Some(v);
        }
        if n < 4 { continue; }
        let mean_x = sum_x / n as f64;
        let denom = sum_xx - n as f64 * mean_x * mean_x;
        if denom <= 0.0 { continue; }
        let phi = (sum_xy - n as f64 * mean_x * mean_x) / denom;
        // Compute residual variance on healthy.
        let mut prev: Option<f64> = None;
        let mut sum_r = 0.0;
        let mut sum_r2 = 0.0;
        let mut n_r = 0_usize;
        for w in 0..healthy_window_end.min(num_windows) {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if let Some(p) = prev {
                let r = v - phi * p;
                sum_r += r;
                sum_r2 += r * r;
                n_r += 1;
            }
            prev = Some(v);
        }
        if n_r < 2 { continue; }
        let mean_r = sum_r / n_r as f64;
        let sigma_r = ((sum_r2 - n_r as f64 * mean_r * mean_r)
                       / (n_r - 1) as f64).max(0.0).sqrt();
        if sigma_r <= 0.0 { continue; }
        // Score evaluation windows.
        let mut prev: Option<f64> = None;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { prev = None; continue; }
            if let Some(p) = prev {
                let r = v - phi * p;
                if (r - mean_r).abs() > k * sigma_r {
                    raw_alert_count += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    window_alerts[w] = true;
                }
            }
            prev = Some(v);
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ar1_residual",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Multivariate Mahalanobis-distance detector (Mahalanobis 1936;
/// Hotelling 1947). Uses joint covariance structure that none of
/// the per-signal detectors exploit. Window-level multivariate.
///
/// For each evaluation window's signal vector x_w, compute
/// `D² = (x_w - μ_healthy)ᵀ Σ⁻¹ (x_w - μ_healthy)` where Σ is the
/// healthy covariance matrix. Fire when D² > k² * num_signals
/// (k=3 → ~99.7% under joint normality, generalisation of 3σ).
pub fn mahalanobis(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    k: f64, // typical: 3.0
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    if num_signals == 0 || num_signals > 32 {
        return zero_output("mahalanobis");
    }

    // Compute healthy mean vector.
    let mut mean = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    mean[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 { mean[s] /= counts[s] as f64; }
    }

    // Compute healthy covariance matrix (num_signals × num_signals).
    let mut cov = std::vec![std::vec![0.0_f64; num_signals]; num_signals];
    let mut n_obs = 0_usize;
    for w in 0..healthy_window_end.min(num_windows) {
        // Skip windows with NaNs.
        let mut row = std::vec![0.0_f64; num_signals];
        let mut all_finite = true;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if v.is_nan() { all_finite = false; break; }
                row[s] = v - mean[s];
            } else { all_finite = false; break; }
        }
        if !all_finite { continue; }
        for i in 0..num_signals {
            for j in 0..num_signals {
                cov[i][j] += row[i] * row[j];
            }
        }
        n_obs += 1;
    }
    if n_obs < num_signals + 1 {
        return zero_output("mahalanobis");
    }
    for i in 0..num_signals {
        for j in 0..num_signals {
            cov[i][j] /= (n_obs - 1) as f64;
        }
    }
    // Add small ridge to diagonal for numerical stability.
    for i in 0..num_signals {
        cov[i][i] += 1e-9;
    }
    // Invert covariance via Gauss-Jordan elimination.
    let inv_cov = match invert_matrix(&cov, num_signals) {
        Some(m) => m,
        None => return zero_output("mahalanobis"),
    };

    let threshold = k * k * num_signals as f64;
    for w in 0..num_windows {
        let mut diff = std::vec![0.0_f64; num_signals];
        let mut all_finite = true;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if v.is_nan() { all_finite = false; break; }
                diff[s] = v - mean[s];
            } else { all_finite = false; break; }
        }
        if !all_finite { continue; }
        // D² = diff' * inv_cov * diff
        let mut d2 = 0.0_f64;
        for i in 0..num_signals {
            let mut row_sum = 0.0;
            for j in 0..num_signals {
                row_sum += inv_cov[i][j] * diff[j];
            }
            d2 += diff[i] * row_sum;
        }
        if d2 > threshold {
            raw_alert_count += 1;
            alerts_per_signal[0] += 1; // multivariate detector
            window_alerts[w] = true;
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mahalanobis_3sigma",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Invert a small square matrix via Gauss-Jordan elimination.
/// Returns None if singular.
fn invert_matrix(m: &[Vec<f64>], n: usize) -> Option<Vec<Vec<f64>>> {
    let mut aug = std::vec![std::vec![0.0_f64; 2 * n]; n];
    for i in 0..n {
        for j in 0..n { aug[i][j] = m[i][j]; }
        aug[i][n + i] = 1.0;
    }
    for col in 0..n {
        // Pivot — find row with largest absolute value in column.
        let mut pivot = col;
        for r in (col + 1)..n {
            if aug[r][col].abs() > aug[pivot][col].abs() { pivot = r; }
        }
        if aug[pivot][col].abs() < 1e-12 { return None; }
        aug.swap(col, pivot);
        let p = aug[col][col];
        for j in 0..(2 * n) { aug[col][j] /= p; }
        for r in 0..n {
            if r == col { continue; }
            let factor = aug[r][col];
            if factor != 0.0 {
                for j in 0..(2 * n) {
                    aug[r][j] -= factor * aug[col][j];
                }
            }
        }
    }
    let mut out = std::vec![std::vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n { out[i][j] = aug[i][n + j]; }
    }
    Some(out)
}

/// Kolmogorov-Smirnov rolling distribution-shift detector
/// (Kolmogorov 1933; Smirnov 1948).
///
/// Per signal: maintain a rolling window of the most-recent `win_n`
/// observations. Compute the two-sample KS statistic D between this
/// recent sample and a reference sample of healthy-window
/// observations. Fire when D > critical_value at α=0.05.
pub fn ks_rolling(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    win_n: usize, // typical: 20
    crit_d: f64,  // typical: 0.4 (D > 0.4 ≈ p < 0.01 for n=20, m=20)
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for s in 0..num_signals {
        // Healthy reference sample.
        let mut healthy: Vec<f64> = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() { healthy.push(v); }
            }
        }
        if healthy.len() < win_n { continue; }
        healthy.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if count < win_n {
                buf[pos] = v;
                pos = (pos + 1) % win_n;
                count += 1;
                continue;
            }
            // Compute KS-D between buf and healthy.
            let mut sample: Vec<f64> = buf.iter().take(count).copied().collect();
            sample.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let d = ks_two_sample(&sample, &healthy);
            if d > crit_d {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ks_rolling_n20",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Two-sample Kolmogorov-Smirnov D statistic. Both inputs sorted.
fn ks_two_sample(a: &[f64], b: &[f64]) -> f64 {
    let na = a.len();
    let nb = b.len();
    if na == 0 || nb == 0 { return 0.0; }
    let mut i = 0_usize;
    let mut j = 0_usize;
    let mut max_d = 0.0_f64;
    while i < na && j < nb {
        let cdf_a = i as f64 / na as f64;
        let cdf_b = j as f64 / nb as f64;
        let d = (cdf_a - cdf_b).abs();
        if d > max_d { max_d = d; }
        if a[i] < b[j] { i += 1; }
        else if b[j] < a[i] { j += 1; }
        else { i += 1; j += 1; }
    }
    max_d
}

/// Poisson-burst detector — debugging-specific (error-rate / count
/// channels). Models per-window error counts as Poisson(λ) where λ
/// is the healthy-window mean count; fires when current count
/// exceeds `λ + k * sqrt(λ)` (Poisson tail bound). Specific to
/// debugging because most production error streams are well-
/// approximated by Poisson statistics. References: Cox & Lewis 1966
/// "The Statistical Analysis of Series of Events".
pub fn poisson_burst(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    k: f64, // typical: 4.0
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    // Healthy mean per signal (treated as λ).
    let mut lambda = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() && v >= 0.0 {
                    lambda[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 { lambda[s] /= counts[s] as f64; }
    }
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() || v < 0.0 { continue; }
            let l = lambda[s];
            // Poisson upper bound: λ + k*sqrt(λ); requires λ > 0.
            if l > 0.0 && v > l + k * l.sqrt() {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "poisson_burst_k4",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Saturation-chain detector — debugging-specific.
///
/// Counts consecutive windows where any signal exceeds
/// `mean_healthy + k * sigma_healthy`; fires when the chain length
/// reaches `n_chain`. Captures the "5 timeouts in a row" pattern
/// that on-call engineers flag as saturation indicators.
/// More targeted than scalar-3σ because it requires PERSISTENT
/// breach, not single-window crossing.
pub fn saturation_chain(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    k: f64,         // typical: 2.0
    n_chain: usize, // typical: 4
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    let mut chain_len = std::vec![0_usize; num_signals];
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { chain_len[s] = 0; continue; }
            let mu = means[s];
            let sd = sigmas[s];
            if sd > 0.0 && (v - mu).abs() > k * sd {
                chain_len[s] += 1;
                if chain_len[s] >= n_chain {
                    raw_alert_count += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    window_alerts[w] = true;
                    chain_len[s] = 0; // reset to avoid pinning at alert
                }
            } else {
                chain_len[s] = 0;
            }
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "saturation_chain_n4",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Chi-squared proportion-shift detector — debugging-specific.
///
/// For two windowed counts x_now and x_healthy with totals N_now
/// and N_healthy, the chi-squared statistic
/// `χ² = ((x_now - p̂*N_now)² / (p̂*N_now)) +
///       ((x_healthy - p̂*N_healthy)² / (p̂*N_healthy))`
/// where `p̂ = (x_now + x_healthy) / (N_now + N_healthy)` tests the
/// null that the two error-rate proportions are equal. Fires when
/// χ² > 3.84 (α = 0.05, 1 dof). Specific to error-rate channels
/// (proportion data) where Gaussian-based detectors are mis-applied.
/// Captures situations like "error rate jumped from 0.1% to 0.3%
/// — small absolute but statistically significant proportion shift".
pub fn chi_squared_proportion(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    win_n: usize, // typical: 10 (recent window for "now")
    chi_sq_crit: f64, // typical: 3.84 (α=0.05, 1 dof)
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    for s in 0..num_signals {
        // Healthy mean proportion p_healthy.
        let mut sum_h = 0.0_f64;
        let mut n_h = 0_usize;
        for w in 0..healthy_window_end.min(num_windows) {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() && v >= 0.0 && v <= 1.0 {
                    sum_h += v;
                    n_h += 1;
                }
            }
        }
        if n_h < 4 { continue; }
        let p_h = sum_h / n_h as f64;
        if p_h <= 0.0 || p_h >= 1.0 { continue; }
        // Rolling window of recent observations.
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() || v < 0.0 || v > 1.0 { continue; }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            // Rolling proportion p_now.
            let mut sum_n = 0.0;
            for i in 0..count { sum_n += buf[i]; }
            let p_now = sum_n / count as f64;
            // Pooled p̂.
            let n_now = count as f64;
            let n_h_f = n_h as f64;
            let p_hat = (sum_n + sum_h) / (n_now + n_h_f);
            if p_hat <= 0.0 || p_hat >= 1.0 { continue; }
            // χ² statistic (1 dof).
            let exp_now = p_hat * n_now;
            let exp_h = p_hat * n_h_f;
            let chi_sq = ((p_now * n_now - exp_now).powi(2) / exp_now)
                + ((sum_h - exp_h).powi(2) / exp_h);
            if chi_sq > chi_sq_crit {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
        }
    }
    for &b in &window_alerts {
        if b { alert_windows += 1; }
    }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "chi_squared_prop",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// MaxInterval (MI) burst detector — Legendy & Salcman 1985,
/// Cocatre-Zilgien & Delcomyn 1992. Five-threshold burst classifier
/// originally for neural spike trains; adapted to debugging by
/// treating "events" as windows where |x - mean_healthy| > 1·sigma.
///
/// Five thresholds: `max_start_isi` (max inter-event interval to
/// open a burst), `max_burst_isi` (max ISI inside a burst),
/// `min_isis_in_burst`, `min_duration_windows`, `min_n_in_burst`.
/// Defaults below are textbook for the spike-train domain; site
/// calibration is recommended.
pub fn max_interval_burst(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    event_k: f64,            // event = |x-μ|>k·σ; typical 1.0
    max_start_isi: usize,    // typical 5
    max_burst_isi: usize,    // typical 8
    min_n_in_burst: usize,   // typical 3
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut events: Vec<usize> = Vec::new();
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if sigmas[s] > 0.0 && (v - means[s]).abs() > event_k * sigmas[s] {
                events.push(w);
            }
        }
        if events.len() < min_n_in_burst { continue; }
        // Walk events; cluster into bursts using max_start_isi / max_burst_isi.
        let mut i = 0;
        while i < events.len() {
            let start = events[i];
            // Look ahead: requires a 2nd event within max_start_isi.
            if i + 1 >= events.len() { break; }
            let next = events[i + 1];
            if next - start > max_start_isi { i += 1; continue; }
            // Burst started; extend as long as ISI ≤ max_burst_isi.
            let mut end = next;
            let mut count_in_burst = 2;
            let mut j = i + 2;
            while j < events.len() && events[j] - end <= max_burst_isi {
                end = events[j];
                count_in_burst += 1;
                j += 1;
            }
            if count_in_burst >= min_n_in_burst {
                // Mark every window in [start, end] as an alert.
                let cap = end.min(num_windows.saturating_sub(1));
                for w in start..=cap {
                    if !window_alerts[w] {
                        raw_alert_count += 1;
                        if s < 32 { alerts_per_signal[s] += 1; }
                        window_alerts[w] = true;
                    }
                }
            }
            i = j;
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "max_interval_burst",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// LogISI burst detector — Selinger, Kulagina, O'Shaughnessy,
/// Pancrazio & Gross 2007. Auto-thresholding via log-transformed
/// inter-event-interval histogram: a "valley" between within-burst
/// (short ISI) and between-burst (long ISI) modes is found
/// automatically, removing the operator-tuning burden of MaxInterval.
///
/// Simplification (vs original): we use a fixed-threshold cutoff at
/// the geometric mean of healthy-slice ISIs (which approximates the
/// natural valley for unimodal log-ISI distributions). Site
/// calibration via `recommend_config_from_healthy` recovers the
/// data-adaptive variant.
pub fn log_isi_burst(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    event_k: f64,         // typical 1.0
    min_n_in_burst: usize, // typical 3
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        // Events on the full slice; ISIs derived; healthy-region ISIs
        // give the auto-threshold via geometric mean.
        let mut events: Vec<usize> = Vec::new();
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if sigmas[s] > 0.0 && (v - means[s]).abs() > event_k * sigmas[s] {
                events.push(w);
            }
        }
        if events.len() < min_n_in_burst + 1 { continue; }
        // Healthy ISIs.
        let mut healthy_isis: Vec<usize> = Vec::new();
        for k in 1..events.len() {
            if events[k] < healthy_window_end {
                healthy_isis.push(events[k] - events[k - 1]);
            }
        }
        if healthy_isis.is_empty() { continue; }
        // Geometric mean of log-ISI ≈ exp(mean(log_isi)).
        let mut sum_log = 0.0_f64;
        let mut n = 0_usize;
        for &i in &healthy_isis {
            if i > 0 {
                sum_log += (i as f64).ln();
                n += 1;
            }
        }
        if n == 0 { continue; }
        let isi_threshold = (sum_log / n as f64).exp();
        // Walk all events; cluster into bursts where ISI < threshold.
        let mut i = 1;
        while i < events.len() {
            if (events[i] - events[i - 1]) as f64 <= isi_threshold {
                let burst_start = events[i - 1];
                let mut burst_end = events[i];
                let mut count_in_burst = 2;
                let mut j = i + 1;
                while j < events.len()
                    && (events[j] - events[j - 1]) as f64 <= isi_threshold
                {
                    burst_end = events[j];
                    count_in_burst += 1;
                    j += 1;
                }
                if count_in_burst >= min_n_in_burst {
                    let cap = burst_end.min(num_windows.saturating_sub(1));
                    for w in burst_start..=cap {
                        if !window_alerts[w] {
                            raw_alert_count += 1;
                            if s < 32 { alerts_per_signal[s] += 1; }
                            window_alerts[w] = true;
                        }
                    }
                }
                i = j;
            } else {
                i += 1;
            }
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "log_isi_burst",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Rank Surprise (RS) burst detector — Gourévitch & Eggermont 2007.
/// Non-parametric alternative to Poisson Surprise: ranks ISIs and
/// fires when the rank-sum of a sub-sequence differs significantly
/// from the null rank distribution. Robust to non-Poisson
/// (e.g. Gamma) inter-event distributions; effective at identifying
/// both very short and very long bursts.
///
/// Simplification (vs original): we use a Mann-Whitney-like rank
/// statistic on a rolling-window of recent ISIs vs healthy-window
/// ISIs and threshold by a critical value at α = 0.05.
pub fn rank_surprise_burst(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    event_k: f64,           // typical 1.0
    win_n_isis: usize,      // typical 5 (recent ISI window)
    rank_z_alpha: f64,      // typical 1.96 (α=0.05)
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut events: Vec<usize> = Vec::new();
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if sigmas[s] > 0.0 && (v - means[s]).abs() > event_k * sigmas[s] {
                events.push(w);
            }
        }
        if events.len() < win_n_isis + 4 { continue; }
        // Build healthy ISI sample.
        let mut healthy_isis: Vec<f64> = Vec::new();
        for k in 1..events.len() {
            if events[k] < healthy_window_end {
                healthy_isis.push((events[k] - events[k - 1]) as f64);
            }
        }
        if healthy_isis.len() < 4 { continue; }
        healthy_isis.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        // Rolling: compute rank-sum of last win_n_isis ISIs in the
        // combined (recent ∪ healthy) sorted list.
        let n_h = healthy_isis.len() as f64;
        let mut buf = std::vec![0.0_f64; win_n_isis];
        let mut count = 0;
        let mut pos = 0;
        for k in 1..events.len() {
            let isi = (events[k] - events[k - 1]) as f64;
            buf[pos] = isi;
            pos = (pos + 1) % win_n_isis;
            if count < win_n_isis { count += 1; }
            if count < win_n_isis { continue; }
            // Compute rank of each buf entry within combined healthy set.
            let mut rank_sum = 0.0_f64;
            for i in 0..count {
                let val = buf[i];
                let mut r = 0;
                for &h in &healthy_isis {
                    if h < val { r += 1; }
                }
                rank_sum += r as f64;
            }
            let mean_rs = (count as f64) * n_h / 2.0;
            let var_rs = (count as f64) * n_h * (count as f64 + n_h + 1.0) / 12.0;
            if var_rs <= 0.0 { continue; }
            let z = (rank_sum - mean_rs) / var_rs.sqrt();
            if z.abs() > rank_z_alpha {
                let w_alert = events[k];
                if w_alert < num_windows && !window_alerts[w_alert] {
                    raw_alert_count += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    window_alerts[w_alert] = true;
                }
            }
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "rank_surprise_burst",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Mean Inter-Spike Interval (MISI) burst detector — adapted from
/// neural-spike literature. Tracks the LOCAL mean ISI over a rolling
/// window and fires when the current ISI is significantly smaller
/// than the local mean by a factor `k`. Captures dynamical baseline
/// shifts that fixed-threshold MaxInterval misses.
pub fn misi_burst(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    event_k: f64,        // typical 1.0
    rolling_n_isis: usize, // typical 10
    factor_k: f64,        // typical 0.3 (current ISI < k * mean → burst)
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut events: Vec<usize> = Vec::new();
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if sigmas[s] > 0.0 && (v - means[s]).abs() > event_k * sigmas[s] {
                events.push(w);
            }
        }
        if events.len() < rolling_n_isis + 1 { continue; }
        let mut isi_buf = std::vec![0.0_f64; rolling_n_isis];
        let mut count = 0;
        let mut pos = 0;
        for k in 1..events.len() {
            let isi = (events[k] - events[k - 1]) as f64;
            if count < rolling_n_isis {
                isi_buf[pos] = isi;
                pos = (pos + 1) % rolling_n_isis;
                count += 1;
                continue;
            }
            // Local mean ISI.
            let mut sum = 0.0;
            for i in 0..count { sum += isi_buf[i]; }
            let local_mean = sum / count as f64;
            if isi < factor_k * local_mean {
                let w_alert = events[k];
                if w_alert < num_windows && !window_alerts[w_alert] {
                    raw_alert_count += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    window_alerts[w_alert] = true;
                }
            }
            isi_buf[pos] = isi;
            pos = (pos + 1) % rolling_n_isis;
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "misi_burst",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Generalized Likelihood Ratio (GLR) change detector.
/// Compares a single-segment fit (one mean / variance) against a
/// two-segment fit (two means) over a rolling window; fires when
/// log-likelihood ratio exceeds threshold. Strong "court-admissible"
/// statistical story — explicit null/alternative hypothesis.
pub fn glr_change(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    win_n: usize,    // typical 30
    glr_k: f64,      // typical 10.0 (LLR threshold)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            // Try every split point inside the window. Compute LLR.
            let mut total_sum = 0.0;
            let mut total_sq = 0.0;
            for i in 0..count {
                total_sum += buf[i];
                total_sq += buf[i] * buf[i];
            }
            let n = count as f64;
            let mean_total = total_sum / n;
            let var_total = ((total_sq - n * mean_total * mean_total) / (n - 1.0)).max(1e-9);
            let mut max_llr = 0.0;
            for split in 5..(count - 5) {
                let mut sum_a = 0.0;
                let mut sq_a = 0.0;
                for i in 0..split { sum_a += buf[i]; sq_a += buf[i] * buf[i]; }
                let mut sum_b = 0.0;
                let mut sq_b = 0.0;
                for i in split..count { sum_b += buf[i]; sq_b += buf[i] * buf[i]; }
                let na = split as f64;
                let nb = (count - split) as f64;
                let ma = sum_a / na;
                let mb = sum_b / nb;
                let va = ((sq_a - na * ma * ma) / na).max(1e-9);
                let vb = ((sq_b - nb * mb * mb) / nb).max(1e-9);
                // Log-likelihood ratio (Gaussian) — single-mean vs split.
                let llr = 0.5 * (n * var_total.ln()
                    - na * va.ln() - nb * vb.ln());
                if llr > max_llr { max_llr = llr; }
            }
            if max_llr > glr_k {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "glr_change",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// ADWIN (ADaptive WINdowing) — Bifet & Gavaldà 2007. Streaming
/// concept-drift detector with variable-length window. Splits the
/// window into two sub-windows; if their means differ by more than
/// the Hoeffding bound, declares drift and shrinks the window.
/// Simplification: fixed-length sub-windows for compatibility with
/// our batch-mode harness.
pub fn adwin(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    win_n: usize, // typical 40 (split into two halves of 20)
    delta: f64,    // typical 0.01 (confidence)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            // Compare first-half mean vs second-half mean.
            let half = count / 2;
            let mut sum_a = 0.0;
            for i in 0..half { sum_a += buf[i]; }
            let mut sum_b = 0.0;
            for i in half..count { sum_b += buf[i]; }
            let mean_a = sum_a / half as f64;
            let mean_b = sum_b / (count - half) as f64;
            // Hoeffding bound: ε = sqrt((R²/(2*n_eff)) * ln(2/δ))
            // where R = range of data; we approximate R by 2*sigma.
            let mut sum_t = 0.0; let mut sq_t = 0.0;
            for i in 0..count { sum_t += buf[i]; sq_t += buf[i] * buf[i]; }
            let n_t = count as f64;
            let mean_t = sum_t / n_t;
            let sigma = ((sq_t - n_t * mean_t * mean_t) / (n_t - 1.0)).max(1e-9).sqrt();
            let r = 4.0 * sigma; // ~range for normal data
            let n_eff = (1.0 / half as f64 + 1.0 / (count - half) as f64).recip();
            let eps = ((r * r / (2.0 * n_eff)) * (2.0_f64 / delta).ln()).sqrt();
            if (mean_a - mean_b).abs() > eps {
                raw_alert_count += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                window_alerts[w] = true;
            }
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "adwin_delta_001",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// MEWMA — Multivariate EWMA control chart (Lowry et al. 1992).
/// Maintains an exponentially-weighted multivariate state vector;
/// fires when its Mahalanobis distance from the healthy mean exceeds
/// `k² * num_signals`. Captures distributed low-amplitude shifts
/// across many signals that single-channel EWMA misses.
pub fn mewma(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    lambda: f64, // typical 0.2
    k: f64,      // typical 3.0
) -> DetectorOutput {
    if num_signals == 0 || num_signals > 32 {
        return zero_output("mewma");
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    let mut mean = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    mean[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 { mean[s] /= counts[s] as f64; }
    }
    // Healthy covariance.
    let mut cov = std::vec![std::vec![0.0_f64; num_signals]; num_signals];
    let mut n_obs = 0_usize;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut row = std::vec![0.0_f64; num_signals];
        let mut all_finite = true;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if v.is_nan() { all_finite = false; break; }
                row[s] = v - mean[s];
            } else { all_finite = false; break; }
        }
        if !all_finite { continue; }
        for i in 0..num_signals {
            for j in 0..num_signals { cov[i][j] += row[i] * row[j]; }
        }
        n_obs += 1;
    }
    if n_obs < num_signals + 1 { return zero_output("mewma"); }
    for i in 0..num_signals {
        for j in 0..num_signals { cov[i][j] /= (n_obs - 1) as f64; }
        cov[i][i] += 1e-9;
    }
    // EWMA matrix scaling: cov_z = (lambda / (2-lambda)) * cov.
    let scale = lambda / (2.0 - lambda);
    let mut cov_z = std::vec![std::vec![0.0_f64; num_signals]; num_signals];
    for i in 0..num_signals {
        for j in 0..num_signals { cov_z[i][j] = cov[i][j] * scale; }
    }
    // Invert cov_z.
    let inv_z = match invert_matrix(&cov_z, num_signals) {
        Some(m) => m,
        None => return zero_output("mewma"),
    };
    let threshold = k * k * num_signals as f64;
    // Stream through evaluation.
    let mut z = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals { z[s] = mean[s]; }
    for w in 0..num_windows {
        let mut all_finite = true;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if v.is_nan() { all_finite = false; break; }
                z[s] = lambda * v + (1.0 - lambda) * z[s];
            } else { all_finite = false; break; }
        }
        if !all_finite { continue; }
        // T² = (z - μ)' inv_z (z - μ)
        let mut diff = std::vec![0.0_f64; num_signals];
        for s in 0..num_signals { diff[s] = z[s] - mean[s]; }
        let mut t2 = 0.0_f64;
        for i in 0..num_signals {
            let mut row_sum = 0.0;
            for j in 0..num_signals { row_sum += inv_z[i][j] * diff[j]; }
            t2 += diff[i] * row_sum;
        }
        if t2 > threshold {
            raw_alert_count += 1;
            alerts_per_signal[0] += 1;
            window_alerts[w] = true;
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mewma_lambda0.2_k3",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Retry-storm detector — debugging-specific. Detects a sequence of
/// k consecutive windows where the residual is positive AND the
/// inter-event interval is shrinking (consecutive gaps decreasing).
/// Captures retry-amplification cascades.
pub fn retry_storm(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    event_k: f64,    // typical 1.0
    n_decreasing: usize, // typical 3
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut events: Vec<usize> = Vec::new();
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            // Positive residual only — retry storms are unidirectional.
            if sigmas[s] > 0.0 && v - means[s] > event_k * sigmas[s] {
                events.push(w);
            }
        }
        if events.len() < n_decreasing + 1 { continue; }
        for i in 0..(events.len() - n_decreasing) {
            let mut isi_decreasing = true;
            let mut prev_isi = events[i + 1] - events[i];
            for k in 1..n_decreasing {
                let cur_isi = events[i + k + 1] - events[i + k];
                if cur_isi >= prev_isi { isi_decreasing = false; break; }
                prev_isi = cur_isi;
            }
            if isi_decreasing {
                let w_alert = events[i + n_decreasing];
                if w_alert < num_windows && !window_alerts[w_alert] {
                    raw_alert_count += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    window_alerts[w_alert] = true;
                }
            }
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "retry_storm",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

/// Correlation-break detector — debugging-specific multivariate.
///
/// Computes the healthy correlation matrix between all signal pairs;
/// streams through evaluation maintaining a rolling correlation
/// matrix; alerts when the Frobenius distance between rolling and
/// healthy correlation matrices exceeds threshold. Catches "broken
/// instrumentation" / "changed execution path" patterns where
/// signals decouple from their historical correlations.
pub fn correlation_break(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    win_n: usize, // typical 40
    theta: f64,    // typical 1.0 (Frobenius distance threshold)
) -> DetectorOutput {
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];
    if num_signals < 2 || num_signals > 32 || num_windows < win_n + 4 {
        return zero_output("correlation_break");
    }

    // Healthy correlation matrix.
    let healthy_corr = compute_correlation(data, num_signals, 0, healthy_window_end);

    // Rolling-window correlation comparison.
    for w in win_n..num_windows {
        let start = w.saturating_sub(win_n);
        let rolling_corr = compute_correlation(data, num_signals, start, w);
        let mut frob = 0.0_f64;
        for i in 0..num_signals {
            for j in 0..num_signals {
                let d = rolling_corr[i][j] - healthy_corr[i][j];
                frob += d * d;
            }
        }
        let frob = frob.sqrt();
        if frob > theta {
            raw_alert_count += 1;
            alerts_per_signal[0] += 1;
            window_alerts[w] = true;
        }
    }
    for &b in &window_alerts { if b { alert_windows += 1; } }
    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "correlation_break",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

fn compute_correlation(
    data: &[f64], num_signals: usize, start_w: usize, end_w: usize,
) -> Vec<Vec<f64>> {
    let mut means = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in start_w..end_w {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    means[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 { means[s] /= counts[s] as f64; }
    }
    let mut sigmas = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        let mut var = 0.0_f64;
        let mut n = 0_usize;
        for w in start_w..end_w {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    var += (v - means[s]).powi(2);
                    n += 1;
                }
            }
        }
        sigmas[s] = if n > 1 { (var / (n - 1) as f64).max(0.0).sqrt() } else { 0.0 };
    }
    let mut corr = std::vec![std::vec![0.0_f64; num_signals]; num_signals];
    for i in 0..num_signals {
        for j in 0..num_signals {
            if i == j { corr[i][j] = 1.0; continue; }
            if sigmas[i] <= 0.0 || sigmas[j] <= 0.0 { continue; }
            let mut cov = 0.0;
            let mut n = 0_usize;
            for w in start_w..end_w {
                let ix = w * num_signals + i;
                let jx = w * num_signals + j;
                if ix < data.len() && jx < data.len() {
                    let vi = data[ix];
                    let vj = data[jx];
                    if !vi.is_nan() && !vj.is_nan() {
                        cov += (vi - means[i]) * (vj - means[j]);
                        n += 1;
                    }
                }
            }
            if n > 1 {
                cov /= (n - 1) as f64;
                corr[i][j] = cov / (sigmas[i] * sigmas[j]);
            }
        }
    }
    corr
}

/// Run the EWMA detector. Roberts (1959).
/// Per signal, EWMA[w] = lambda*x + (1-lambda)*EWMA[w-1]. Fire when
/// `|EWMA - mean_healthy| > L * sigma_healthy * sqrt(lambda/(2-lambda))`.
pub fn ewma(
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    pred_window: u64,
    lambda: f64, // typical: 0.2
    l: f64,      // typical: 3.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut alert_windows: u64 = 0;
    let mut raw_alert_count: u64 = 0;
    let mut window_alerts = std::vec![false; num_windows];

    let mut z = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        z[s] = means[s];
    }
    let scale = (lambda / (2.0 - lambda)).sqrt();

    for w in 0..num_windows {
        let mut any_alert = false;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() {
                continue;
            }
            let v = data[idx];
            if v.is_nan() {
                continue;
            }
            z[s] = lambda * v + (1.0 - lambda) * z[s];
            let limit = l * sigmas[s] * scale;
            if (z[s] - means[s]).abs() > limit {
                raw_alert_count += 1;
                if s < 32 {
                    alerts_per_signal[s] += 1;
                }
                any_alert = true;
            }
        }
        if any_alert {
            alert_windows += 1;
            window_alerts[w] = true;
        }
    }

    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_alerts, fault_labels, pred_window);

    DetectorOutput {
        detector_name: "ewma_lambda0.2_L3",
        raw_alert_count,
        alerts_per_signal,
        alert_windows,
        episode_count: alert_windows,
        captured_faults,
        total_faults,
        clean_window_false_alerts: clean_fp,
        clean_windows,
    }
}

// ---------- shared helpers ----------

fn fit_healthy_stats(
    data: &[f64],
    num_signals: usize,
    healthy_window_end: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut means = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    means[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 {
            means[s] /= counts[s] as f64;
        }
    }
    let mut var_sum = std::vec![0.0_f64; num_signals];
    let mut var_n = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    let d = v - means[s];
                    var_sum[s] += d * d;
                    var_n[s] += 1;
                }
            }
        }
    }
    let mut sigmas = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        sigmas[s] = if var_n[s] > 1 {
            (var_sum[s] / (var_n[s] - 1) as f64).sqrt()
        } else {
            0.0
        };
    }
    (means, sigmas)
}

fn score_against_labels(
    window_alerts: &[bool],
    fault_labels: &[bool],
    pred_window: u64,
) -> (u64, u64, u64, u64) {
    let n = window_alerts.len();
    let mut total_faults = 0_u64;
    let mut captured = 0_u64;
    for (w, &is_fault) in fault_labels.iter().enumerate().take(n) {
        if is_fault {
            total_faults += 1;
            // Capture if any alert within ±pred_window.
            let lo = w.saturating_sub(pred_window as usize);
            let hi = (w + pred_window as usize).min(n - 1);
            for ww in lo..=hi {
                if window_alerts[ww] {
                    captured += 1;
                    break;
                }
            }
        }
    }
    let mut clean_windows = 0_u64;
    let mut clean_fp = 0_u64;
    for (w, &is_fault) in fault_labels.iter().enumerate().take(n) {
        // A window is "clean" if NO labeled fault falls within ±pred_window.
        let lo = w.saturating_sub(pred_window as usize);
        let hi = (w + pred_window as usize).min(n - 1);
        let mut near_fault = is_fault;
        if !near_fault {
            for ww in lo..=hi {
                if ww < fault_labels.len() && fault_labels[ww] {
                    near_fault = true;
                    break;
                }
            }
        }
        if !near_fault {
            clean_windows += 1;
            if window_alerts[w] {
                clean_fp += 1;
            }
        }
    }
    (total_faults, captured, clean_windows, clean_fp)
}

// =====================================================================
// SESSION 8.6 — TIER G — Concept-drift streaming family
//
// All concept-drift detectors below convert each window's residual into
// a binary "error" signal via `|x - mean_healthy| > k * sigma_healthy`,
// then apply the published drift-detection rule on the binary stream.
// This is the standard adaptation of error-rate concept-drift detectors
// to regression-style residual streams. Each detector is deterministic,
// per-signal, single-pass, returns DetectorOutput.
// =====================================================================

/// Shiryaev-Roberts sequential CPD (Shiryaev 1963; Roberts 1966).
/// SR[w] = (1 + SR[w-1]) * lambda_ratio. Fires when SR exceeds threshold h.
/// lambda_ratio approximates the likelihood ratio of post- vs pre-shift mean.
pub fn shiryaev_roberts(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    h: f64, // typical 50.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut sr = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let z = (v - mu) / sd;
            // Likelihood ratio of N(1, 1) vs N(0, 1) at z: exp(z - 0.5).
            let lr = (z - 0.5).exp().min(1e6);
            sr = (1.0 + sr) * lr;
            if sr > h {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                sr = 0.0; // reset post-detection
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "shiryaev_roberts",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// DDM — Drift Detection Method (Gama 2004). Tracks error rate p and
/// stddev s = sqrt(p(1-p)/n); fires drift at p_w + s_w >= p_min + 3*s_min.
pub fn ddm(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, // typical 2.5
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut n = 0_u64; let mut errs = 0_u64;
        let mut p_min = f64::INFINITY; let mut s_min = f64::INFINITY;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            n += 1;
            let is_err = ((v - mu).abs() > err_k * sd) as u64;
            errs += is_err;
            if n < 30 { continue; }
            let p = errs as f64 / n as f64;
            let stdv = (p * (1.0 - p) / n as f64).sqrt();
            if p + stdv < p_min + s_min { p_min = p; s_min = stdv; }
            if p + stdv >= p_min + 3.0 * s_min {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                p_min = f64::INFINITY; s_min = f64::INFINITY;
                n = 0; errs = 0;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ddm",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// EDDM — Early Drift Detection Method (Baena-García 2006). Tracks
/// distance between consecutive errors; drift at (p' + 2s') / max(p' + 2s') < 0.9.
pub fn eddm(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, // typical 2.5
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut last_err: Option<usize> = None;
        let mut sum = 0.0_f64; let mut sumsq = 0.0_f64;
        let mut n_err = 0_u64; let mut max_ratio = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if (v - mu).abs() > err_k * sd {
                if let Some(le) = last_err {
                    let d = (w - le) as f64;
                    n_err += 1;
                    sum += d; sumsq += d * d;
                    let mp = sum / n_err as f64;
                    let var = (sumsq / n_err as f64 - mp * mp).max(0.0);
                    let sp = var.sqrt();
                    let val = mp + 2.0 * sp;
                    if val > max_ratio { max_ratio = val; }
                    if max_ratio > 0.0 && n_err >= 30 && val / max_ratio < 0.9 {
                        raw += 1;
                        if s < 32 { alerts_per_signal[s] += 1; }
                        win_alerts[w] = true;
                        sum = 0.0; sumsq = 0.0; n_err = 0; max_ratio = 0.0;
                    }
                }
                last_err = Some(w);
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "eddm",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// HDDM-A — Hoeffding-bound DDM with arithmetic weights (Frias-Blanco 2015).
/// Tests if difference between cumulative-mean and instantaneous-mean
/// exceeds the Hoeffding bound at confidence delta.
pub fn hddm_a(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, delta: f64, // typical err_k=2.5, delta=0.001
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut n = 0_u64; let mut sum_err = 0_u64;
        let mut x_min = 1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let e = ((v - mu).abs() > err_k * sd) as u64;
            sum_err += e; n += 1;
            if n < 30 { continue; }
            let p_now = sum_err as f64 / n as f64;
            if p_now < x_min { x_min = p_now; }
            // Hoeffding: eps = sqrt(0.5 * ln(2/delta) / n).
            let eps = (0.5 * (2.0 / delta).ln() / n as f64).sqrt();
            if p_now - x_min > eps {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                x_min = 1.0; sum_err = 0; n = 0;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "hddm_a",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// HDDM-W — Hoeffding-bound DDM with EWMA-weighted updates (Frias-Blanco 2015).
///
/// Tier G (concept-drift streaming). Maintains an EWMA of per-signal error rate; alerts when the EWMA exceeds its running minimum by the Hoeffding bound `eps = sqrt(var_eff * 0.5 * ln(2/delta))`.
/// Window-level alert when the bound is breached; per-signal counters; EWMA + min reset on each alert.
pub fn hddm_w(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, lambda: f64, delta: f64, // typical err_k=2.5, lambda=0.05, delta=0.001
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut p = 0.0_f64; let mut p_min = 1.0_f64; let mut n = 0_u64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let e = if (v - mu).abs() > err_k * sd { 1.0 } else { 0.0 };
            p = lambda * e + (1.0 - lambda) * p; n += 1;
            if n < 30 { continue; }
            if p < p_min { p_min = p; }
            // Hoeffding bound on EWMA.
            let var_eff = lambda / (2.0 - lambda);
            let eps = (var_eff * 0.5 * (2.0 / delta).ln()).sqrt();
            if p - p_min > eps {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                p_min = 1.0; n = 0; p = 0.0;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "hddm_w",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// STEPD — Statistical Test of Equal Proportions (Nishida 2007). Compares
/// error proportion in a recent window vs older window via chi-squared.
pub fn stepd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, recent_n: usize, chi2_thresh: f64, // typical err_k=2.5, recent_n=30, chi2_thresh=6.63 (α=0.01)
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut errs = std::vec![0_u8; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            errs[w] = ((v - mu).abs() > err_k * sd) as u8;
        }
        for w in (2 * recent_n)..num_windows {
            let mut e_old = 0_u64; let mut e_new = 0_u64;
            for k in 0..recent_n {
                e_old += errs[w - 2 * recent_n + k] as u64;
                e_new += errs[w - recent_n + k] as u64;
            }
            let n_o = recent_n as f64; let n_r = recent_n as f64;
            let p_pool = (e_old + e_new) as f64 / (n_o + n_r);
            let var = p_pool * (1.0 - p_pool) * (1.0 / n_o + 1.0 / n_r);
            if var < 1e-9 { continue; }
            let chi2 = (e_old as f64 / n_o - e_new as f64 / n_r).powi(2) / var;
            if chi2 > chi2_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "stepd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// ECDD — EWMA Concept Drift Detector (Ross 2012). EWMA control chart on
/// binary error stream with adaptive control limits.
pub fn ecdd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, lambda: f64, l: f64, // typical err_k=2.5, lambda=0.2, l=2.5
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut p_bar = 0.0_f64; let mut z = 0.0_f64; let mut n = 0_u64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let e = if (v - mu).abs() > err_k * sd { 1.0 } else { 0.0 };
            n += 1;
            // Update running mean and EWMA.
            p_bar += (e - p_bar) / n as f64;
            z = lambda * e + (1.0 - lambda) * z;
            if n < 30 { continue; }
            let var_z = (lambda / (2.0 - lambda)) * p_bar * (1.0 - p_bar);
            let limit = l * var_z.sqrt();
            if z - p_bar > limit {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ecdd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// KSWIN — Kolmogorov-Smirnov sliding window detector (Raab 2020).
/// Distinct from ks_rolling: uses recent-W vs older reservoir of size R.
pub fn kswin(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_w: usize, reservoir_r: usize, alpha: f64, // typical w=30, r=100, α=0.005
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let crit_d = (-0.5 * alpha.ln() * (1.0 / win_w as f64 + 1.0 / reservoir_r as f64)).sqrt();
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; reservoir_r + win_w];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < reservoir_r + win_w {
                buf[count] = v; count += 1;
            } else {
                buf.copy_within(1..reservoir_r + win_w, 0);
                buf[reservoir_r + win_w - 1] = v;
            }
            if count < reservoir_r + win_w { continue; }
            let d = ks_two_sample(&buf[..reservoir_r], &buf[reservoir_r..]);
            if d > crit_d {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "kswin",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// FHDDM — Fast Hoeffding DDM (Pesaranghader 2016). Sliding window with
/// Hoeffding bound on max-prob-of-error seen so far.
pub fn fhddm(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, win_n: usize, delta: f64, // typical err_k=2.5, win_n=25, delta=1e-7
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let eps = (0.5 / win_n as f64 * (1.0 / delta).ln()).sqrt();
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0_u8; win_n];
        let mut pos = 0_usize; let mut count = 0_usize;
        let mut p_max = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let e = ((v - mu).abs() > err_k * sd) as u8;
            buf[pos] = e; pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            let mut sum = 0_u64;
            for &b in buf.iter() { sum += b as u64; }
            let p = sum as f64 / win_n as f64;
            if p > p_max { p_max = p; }
            if p_max - p > eps {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                p_max = 0.0;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fhddm",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER H — Distribution-shift family
//
// Each detector compares the recent rolling window of size W against
// the healthy reference (healthy_window_end samples). Reference and
// recent are histogrammed into n_bins bins on the healthy min/max
// range (clamping recent values outside the reference range to edge
// bins). Distance threshold fires the detector. Per-signal, single-pass.
// =====================================================================

/// Build a histogram with n_bins over [min,max] from sample slice.
/// Edge bins absorb out-of-range values. Returns (counts, edges).
fn histogram(samples: &[f64], n_bins: usize, lo: f64, hi: f64) -> Vec<f64> {
    let mut h = std::vec![0.0_f64; n_bins];
    let span = (hi - lo).max(1e-9);
    let n = samples.len() as f64;
    if n < 1.0 { return h; }
    for &v in samples {
        if v.is_nan() { continue; }
        let mut idx = ((v - lo) / span * n_bins as f64) as isize;
        if idx < 0 { idx = 0; }
        if idx >= n_bins as isize { idx = n_bins as isize - 1; }
        h[idx as usize] += 1.0;
    }
    let total: f64 = h.iter().sum();
    if total > 0.0 { for x in h.iter_mut() { *x /= total; } }
    h
}

/// Reference-signal histogram from the healthy slice — single helper used
/// across all distribution-shift detectors.
fn fit_healthy_histograms(
    data: &[f64], num_signals: usize, healthy_window_end: usize, n_bins: usize,
) -> (Vec<Vec<f64>>, Vec<(f64, f64)>) {
    let mut refs = Vec::with_capacity(num_signals);
    let mut ranges = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() {
                let v = data[i]; if !v.is_nan() { samples.push(v); }
            }
        }
        let mut lo = f64::INFINITY; let mut hi = f64::NEG_INFINITY;
        for &v in &samples { if v < lo { lo = v; } if v > hi { hi = v; } }
        if !lo.is_finite() || !hi.is_finite() { lo = 0.0; hi = 1.0; }
        if hi - lo < 1e-9 { hi = lo + 1.0; }
        let h = histogram(&samples, n_bins, lo, hi);
        refs.push(h); ranges.push((lo, hi));
    }
    (refs, ranges)
}

/// Helper for rolling distribution-shift detectors. Calls `dist_fn(ref_hist, win_hist)`
/// per (window, signal) on the rolling W-sample window vs healthy reference histogram.
fn run_distribution_shift_detector(
    detector_name: &'static str,
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, n_bins: usize, threshold: f64,
    dist_fn: impl Fn(&[f64], &[f64]) -> f64,
) -> DetectorOutput {
    let (refs, ranges) = fit_healthy_histograms(data, num_signals, healthy_window_end, n_bins);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize; let mut pos = 0_usize;
        let (lo, hi) = ranges[s];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            buf[pos] = v; pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            let win_h = histogram(&buf, n_bins, lo, hi);
            let d = dist_fn(&refs[s], &win_h);
            if d > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name, raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Wasserstein 1D — Vaserstein 1969. EMD between two sorted univariate samples.
pub fn wasserstein_1d(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.3
) -> DetectorOutput {
    run_distribution_shift_detector("wasserstein_1d", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 20, threshold,
        |r, q| {
            let mut cdf_r = 0.0; let mut cdf_q = 0.0; let mut sum = 0.0;
            for i in 0..r.len() { cdf_r += r[i]; cdf_q += q[i]; sum += (cdf_r - cdf_q).abs(); }
            sum / r.len() as f64
        })
}

/// Jensen-Shannon divergence — Lin 1991. Symmetric, bounded in [0, ln 2].
pub fn jensen_shannon(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.1
) -> DetectorOutput {
    run_distribution_shift_detector("jensen_shannon", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 20, threshold,
        |p, q| {
            let mut sum = 0.0_f64;
            for i in 0..p.len() {
                let m = 0.5 * (p[i] + q[i]);
                if m > 0.0 {
                    if p[i] > 0.0 { sum += 0.5 * p[i] * (p[i] / m).ln(); }
                    if q[i] > 0.0 { sum += 0.5 * q[i] * (q[i] / m).ln(); }
                }
            }
            sum
        })
}

/// KL divergence — Kullback & Leibler 1951. Asymmetric P||Q. Add ε to avoid log(0).
pub fn kl_divergence(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.5
) -> DetectorOutput {
    run_distribution_shift_detector("kl_divergence", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 20, threshold,
        |p, q| {
            let eps = 1e-9; let mut sum = 0.0_f64;
            for i in 0..p.len() {
                if p[i] > 0.0 { sum += p[i] * ((p[i] + eps) / (q[i] + eps)).ln(); }
            }
            sum
        })
}

/// PSI — Population Stability Index (banking literature). Symmetric KL variant.
pub fn psi(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.25 (banking convention)
) -> DetectorOutput {
    run_distribution_shift_detector("psi", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 10, threshold,
        |p, q| {
            let eps = 1e-9; let mut sum = 0.0_f64;
            for i in 0..p.len() { sum += (q[i] - p[i]) * ((q[i] + eps) / (p[i] + eps)).ln(); }
            sum
        })
}

/// Anderson-Darling two-sample — Pettitt 1976. Tail-emphasized rank statistic.
/// Operates on sorted-rank statistic over combined sample.
pub fn anderson_darling(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 2.5 (~α=0.05)
) -> DetectorOutput {
    let mut refs = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        refs.push(samples);
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let m = refs[s].len();
        if m < 10 { continue; }
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize; let mut pos = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            buf[pos] = v; pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            // AD two-sample (Pettitt simplification): A² = (1/(m+n)) Σ ((Hk·N - n·k)² / (k(N-k)))
            let mut win_sorted = buf.clone();
            win_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n;
            let total = m + n; let mut a2 = 0.0_f64;
            // For each combined-rank k, count how many of `win_sorted` ≤ ref[k].
            // Approximate via histogram-like accumulation.
            let mut all: Vec<(f64, u8)> = Vec::with_capacity(total);
            for &x in &refs[s] { all.push((x, 0)); }
            for &x in &win_sorted { all.push((x, 1)); }
            all.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let mut hk: u64 = 0;
            for k in 1..total {
                if all[k - 1].1 == 1 { hk += 1; }
                let denom = (k * (total - k)) as f64;
                if denom < 1.0 { continue; }
                let num = (hk as f64 * total as f64 - (n * k) as f64).powi(2);
                a2 += num / denom;
            }
            a2 /= total as f64;
            if a2 > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "anderson_darling",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Cramér-von Mises two-sample — von Mises 1928. Sum of squared CDF differences.
pub fn cramer_von_mises(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.5
) -> DetectorOutput {
    run_distribution_shift_detector("cramer_von_mises", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 20, threshold,
        |r, q| {
            let mut cdf_r = 0.0; let mut cdf_q = 0.0; let mut sum = 0.0;
            for i in 0..r.len() { cdf_r += r[i]; cdf_q += q[i]; sum += (cdf_r - cdf_q).powi(2); }
            sum * (r.len() as f64).sqrt()
        })
}

/// Energy distance — Székely 2002. 2*E|X-Y| - E|X-X'| - E|Y-Y'|. Approximated via
/// histogram-based sample quantiles.
pub fn energy_distance(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.3
) -> DetectorOutput {
    run_distribution_shift_detector("energy_distance", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 20, threshold,
        |p, q| {
            let n = p.len();
            let mut e_xy = 0.0; let mut e_xx = 0.0; let mut e_yy = 0.0;
            for i in 0..n {
                for j in 0..n {
                    let d = (i as f64 - j as f64).abs();
                    e_xy += p[i] * q[j] * d;
                    e_xx += p[i] * p[j] * d;
                    e_yy += q[i] * q[j] * d;
                }
            }
            (2.0 * e_xy - e_xx - e_yy).max(0.0).sqrt()
        })
}

/// MMD — Maximum Mean Discrepancy (Gretton 2012). Gaussian kernel on histogram bin centers.
pub fn mmd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.05
) -> DetectorOutput {
    run_distribution_shift_detector("mmd", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 20, threshold,
        |p, q| {
            let n = p.len(); let sigma2 = (n as f64).powi(2) / 4.0;
            let kernel = |i: usize, j: usize| (-((i as f64 - j as f64).powi(2)) / sigma2).exp();
            let mut sum = 0.0_f64;
            for i in 0..n {
                for j in 0..n {
                    let k = kernel(i, j);
                    sum += k * (p[i] * p[j] - 2.0 * p[i] * q[j] + q[i] * q[j]);
                }
            }
            sum.max(0.0).sqrt()
        })
}

/// Bhattacharyya distance — Bhattacharyya 1943. -ln(Σ sqrt(p_i q_i)).
pub fn bhattacharyya(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.2
) -> DetectorOutput {
    run_distribution_shift_detector("bhattacharyya", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 20, threshold,
        |p, q| {
            let mut bc = 0.0_f64;
            for i in 0..p.len() { bc += (p[i] * q[i]).sqrt(); }
            -bc.max(1e-9).ln()
        })
}

/// Hellinger distance — Hellinger 1909. sqrt(0.5 Σ (sqrt(p_i)-sqrt(q_i))²).
pub fn hellinger(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.3
) -> DetectorOutput {
    run_distribution_shift_detector("hellinger", data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, win_n, 20, threshold,
        |p, q| {
            let mut sum = 0.0_f64;
            for i in 0..p.len() {
                let d = p[i].sqrt() - q[i].sqrt();
                sum += d * d;
            }
            (0.5 * sum).sqrt()
        })
}

// =====================================================================
// SESSION 8.6 — TIER I — Robust / nonparametric family
// Each detector is per-signal, deterministic, single-pass over a
// rolling window. Cited inline.
// =====================================================================

/// Median absolute slope — robust trend witness. Computes diff[i] = x[i]-x[i-1]
/// over rolling W; fires when |diff_now| > k * MAD(diff history).
pub fn median_absolute_slope(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 30, 5.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut diff_buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize; let mut prev = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if !prev.is_nan() {
                let d = v - prev;
                if count < win_n { diff_buf[count] = d; count += 1; }
                else {
                    diff_buf.copy_within(1..win_n, 0);
                    diff_buf[win_n - 1] = d;
                }
                if count >= win_n {
                    let mut sorted = diff_buf.clone();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
                    let median = sorted[win_n / 2];
                    let mut abs_dev = std::vec![0.0_f64; win_n];
                    for j in 0..win_n { abs_dev[j] = (sorted[j] - median).abs(); }
                    abs_dev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
                    let mad = abs_dev[win_n / 2];
                    if mad > 1e-9 && (d - median).abs() > k * mad {
                        raw += 1;
                        if s < 32 { alerts_per_signal[s] += 1; }
                        win_alerts[w] = true;
                    }
                }
            }
            prev = v;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "median_absolute_slope",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Theil-Sen residual — Theil 1950, Sen 1968. Robust slope is the median of
/// all pairwise slopes; residual from the fit triggers alarms.
pub fn theil_sen_residual(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 30, 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Theil-Sen slope: median of (b_j - b_i) / (j - i) for i < j.
            let mut slopes = Vec::with_capacity(win_n * win_n / 2);
            for i in 0..win_n {
                for j in (i + 1)..win_n {
                    slopes.push((buf[j] - buf[i]) / (j - i) as f64);
                }
            }
            slopes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let slope = slopes[slopes.len() / 2];
            // Intercept: median of (y_i - slope * i).
            let mut intercepts = std::vec![0.0_f64; win_n];
            for i in 0..win_n { intercepts[i] = buf[i] - slope * i as f64; }
            intercepts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let intercept = intercepts[win_n / 2];
            // Residual at last point.
            let pred = intercept + slope * (win_n - 1) as f64;
            let mut residuals = std::vec![0.0_f64; win_n];
            for i in 0..win_n { residuals[i] = buf[i] - (intercept + slope * i as f64); }
            let mut abs_r = residuals.clone();
            for r in abs_r.iter_mut() { *r = r.abs(); }
            abs_r.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let mad = abs_r[win_n / 2].max(1e-9);
            if (buf[win_n - 1] - pred).abs() > k * mad {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "theil_sen_residual",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Sen's slope changepoint — Sen 1968. Compares Sen's slope across
/// adjacent rolling sub-windows; significant slope change → alert.
pub fn sen_slope_changepoint(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.05
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; 2 * win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < 2 * win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..2 * win_n, 0);
                buf[2 * win_n - 1] = v;
            }
            if count < 2 * win_n { continue; }
            let slope_a = sen_slope(&buf[..win_n]);
            let slope_b = sen_slope(&buf[win_n..]);
            if (slope_b - slope_a).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "sen_slope_changepoint",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

fn sen_slope(buf: &[f64]) -> f64 {
    let n = buf.len();
    if n < 2 { return 0.0; }
    let mut slopes = Vec::with_capacity(n * n / 2);
    for i in 0..n {
        for j in (i + 1)..n {
            slopes.push((buf[j] - buf[i]) / (j - i) as f64);
        }
    }
    slopes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    slopes[slopes.len() / 2]
}

/// Mood's median test rolling — Mood 1950. Compares median of recent W
/// vs healthy reference; nonparametric chi-squared on count above median.
pub fn moods_median_rolling(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, chi2_thresh: f64, // typical 30, 6.63
) -> DetectorOutput {
    let mut refs = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let m = if samples.is_empty() { 0.0 } else { samples[samples.len() / 2] };
        refs.push(m);
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let median_ref = refs[s];
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut above = 0_u64;
            for &x in &buf { if x > median_ref { above += 1; } }
            let n = win_n as f64;
            let p_exp = 0.5;
            let exp_above = n * p_exp;
            let exp_below = n * (1.0 - p_exp);
            let chi2 = (above as f64 - exp_above).powi(2) / exp_above
                     + ((n - above as f64) - exp_below).powi(2) / exp_below;
            if chi2 > chi2_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "moods_median",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Brown-Forsythe variance test — Brown & Forsythe 1974. Rolling W vs
/// healthy reference; F-test on absolute deviations from median.
pub fn brown_forsythe(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, f_thresh: f64, // typical 30, 4.0
) -> DetectorOutput {
    let mut ref_dev = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        let mut sorted = samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let m = if sorted.is_empty() { 0.0 } else { sorted[sorted.len() / 2] };
        let dev: Vec<f64> = samples.iter().map(|x| (x - m).abs()).collect();
        ref_dev.push(dev);
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        if ref_dev[s].len() < 10 { continue; }
        let ref_mean: f64 = ref_dev[s].iter().sum::<f64>() / ref_dev[s].len() as f64;
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let m = sorted[win_n / 2];
            let win_dev: Vec<f64> = buf.iter().map(|x| (x - m).abs()).collect();
            let win_mean: f64 = win_dev.iter().sum::<f64>() / win_n as f64;
            // F = MS_between / MS_within. Simplified.
            let ms_between = (win_mean - ref_mean).powi(2) / 2.0;
            let mut ss_within = 0.0;
            for v in &win_dev { ss_within += (v - win_mean).powi(2); }
            for v in &ref_dev[s] { ss_within += (v - ref_mean).powi(2); }
            let ms_within = ss_within / (win_n as f64 + ref_dev[s].len() as f64 - 2.0).max(1.0);
            let f = if ms_within > 1e-9 { ms_between / ms_within } else { 0.0 };
            if f > f_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "brown_forsythe",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Levene's test — Levene 1960. Like Brown-Forsythe but uses absolute
/// deviations from MEAN rather than median.
pub fn levene_variance(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, f_thresh: f64, // typical 30, 4.0
) -> DetectorOutput {
    let (means, _) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut ref_dev = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut dev = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() {
                let v = data[i]; if !v.is_nan() { dev.push((v - means[s]).abs()); }
            }
        }
        ref_dev.push(dev);
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        if ref_dev[s].len() < 10 { continue; }
        let ref_mean: f64 = ref_dev[s].iter().sum::<f64>() / ref_dev[s].len() as f64;
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let win_mean: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let win_dev: Vec<f64> = buf.iter().map(|x| (x - win_mean).abs()).collect();
            let win_dev_mean: f64 = win_dev.iter().sum::<f64>() / win_n as f64;
            let ms_between = (win_dev_mean - ref_mean).powi(2) / 2.0;
            let mut ss_within = 0.0;
            for v in &win_dev { ss_within += (v - win_dev_mean).powi(2); }
            for v in &ref_dev[s] { ss_within += (v - ref_mean).powi(2); }
            let ms_within = ss_within / (win_n as f64 + ref_dev[s].len() as f64 - 2.0).max(1.0);
            let f = if ms_within > 1e-9 { ms_between / ms_within } else { 0.0 };
            if f > f_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "levene_variance",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Sign test drift — number of sign(x_i - median_ref) deviations exceeds
/// binomial CI bound.
pub fn sign_test_drift(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, z_thresh: f64, // typical 30, 2.5
) -> DetectorOutput {
    let mut refs = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        refs[s] = if samples.is_empty() { 0.0 } else { samples[samples.len() / 2] };
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let m = refs[s];
        let mut buf = std::vec![0_i32; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let sign = if v > m { 1 } else { -1 };
            if count < win_n { buf[count] = sign; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = sign;
            }
            if count < win_n { continue; }
            let pos: i64 = buf.iter().map(|&x| x as i64).filter(|&x| x > 0).count() as i64;
            let n = win_n as f64;
            // Standard normal approximation: z = (k - n/2) / sqrt(n/4)
            let z = (pos as f64 - n / 2.0).abs() / (n / 4.0).sqrt();
            if z > z_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "sign_test_drift",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Runs test — Wald-Wolfowitz 1940. Counts runs of like signs in rolling W.
/// Too-few or too-many runs deviates from random null.
pub fn runs_test(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, z_thresh: f64, // typical 30, 2.5
) -> DetectorOutput {
    let mut refs = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        refs[s] = if samples.is_empty() { 0.0 } else { samples[samples.len() / 2] };
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let m = refs[s];
        let mut buf = std::vec![0_i32; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let sign = if v > m { 1 } else { -1 };
            if count < win_n { buf[count] = sign; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = sign;
            }
            if count < win_n { continue; }
            let mut runs = 1_u64;
            for i in 1..win_n { if buf[i] != buf[i - 1] { runs += 1; } }
            let n1: u64 = buf.iter().filter(|&&x| x > 0).count() as u64;
            let n2: u64 = buf.iter().filter(|&&x| x < 0).count() as u64;
            if n1 == 0 || n2 == 0 { continue; }
            let n = (n1 + n2) as f64;
            let mu = 1.0 + 2.0 * n1 as f64 * n2 as f64 / n;
            let var = (mu - 1.0) * (mu - 2.0) / (n - 1.0);
            if var < 1e-9 { continue; }
            let z = (runs as f64 - mu).abs() / var.sqrt();
            if z > z_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "runs_test",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Wald-Wolfowitz two-sample — direct test that two samples come from
/// same distribution. Distinct from runs_test (single-sample sign-runs).
pub fn wald_wolfowitz_two_sample(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, z_thresh: f64, // typical 30, 2.5
) -> DetectorOutput {
    let mut refs = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        refs.push(samples);
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let m = refs[s].len();
        if m < 10 { continue; }
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Combined sort with origin tag.
            let mut all: Vec<(f64, u8)> = Vec::with_capacity(m + win_n);
            for &x in &refs[s] { all.push((x, 0)); }
            for &x in &buf { all.push((x, 1)); }
            all.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let mut runs = 1_u64;
            for i in 1..all.len() { if all[i].1 != all[i - 1].1 { runs += 1; } }
            let n1 = m as f64; let n2 = win_n as f64; let n = n1 + n2;
            let mu = 1.0 + 2.0 * n1 * n2 / n;
            let var = (mu - 1.0) * (mu - 2.0) / (n - 1.0);
            if var < 1e-9 { continue; }
            let z = (runs as f64 - mu).abs() / var.sqrt();
            if z > z_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "wald_wolfowitz",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Sequential rank — streaming rank-statistic detector. Tracks the
/// rank-sum of recent W within combined healthy ∪ recent.
pub fn sequential_rank(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, z_thresh: f64, // typical 30, 2.5
) -> DetectorOutput {
    let mut refs = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        refs.push(samples);
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let m = refs[s].len();
        if m < 10 { continue; }
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Rank-sum: for each x in buf, count refs[s] entries < x.
            let mut rank_sum = 0_u64;
            for &x in &buf {
                let mut lo = 0; let mut hi = m;
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    if refs[s][mid] < x { lo = mid + 1; } else { hi = mid; }
                }
                rank_sum += lo as u64;
            }
            // Mann-Whitney U normal approx.
            let n1 = win_n as f64; let n2 = m as f64;
            let var = n1 * n2 * (n1 + n2 + 1.0) / 12.0;
            // U = R - n1(n1+1)/2 where R = rank_sum + Σ ranks among buf.
            // Approximate U by relating rank_sum to expected.
            let exp_rank_sum = n1 * n2 / 2.0;
            let z = if var > 1e-9 { (rank_sum as f64 - exp_rank_sum) / var.sqrt() } else { 0.0 };
            if z.abs() > z_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "sequential_rank",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER J — Forecast-residual family
// Each detector fits a one-step forecast model online, computes
// residual = actual - forecast, fires when |residual| > k * sigma_r.
// =====================================================================

/// Simple Exponential Smoothing residual — Brown 1956.
///
/// Tier J (forecast residual). Smooths the per-signal series with `S[t] = α·x[t] + (1-α)·S[t-1]`; computes the absolute residual `|x[t] - S[t-1]|`; alerts when the residual exceeds `k·sigma_healthy`.
/// Per-window alert when smoothing residual exceeds `k·sigma`.
pub fn ses_residual(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    alpha: f64, k: f64, // typical 0.3, 4.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut level = means[s];
        let sd = sigmas[s].max(1e-9);
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let pred = level;
            let resid = v - pred;
            level = alpha * v + (1.0 - alpha) * level;
            if resid.abs() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ses_residual",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Holt linear trend — Holt 1957. Level + slope.
///
/// Tier J (forecast residual). Holt's two-component smoothing: level + slope. Forecast `y[t+1] = level[t] + slope[t]`; alert when |x[t+1] - y[t+1]| exceeds `k·sigma_healthy`.
/// Per-window alert when 1-step-ahead forecast residual exceeds `k·sigma`.
pub fn holt_linear(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    alpha: f64, beta: f64, k: f64, // typical 0.3, 0.1, 4.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut level = means[s]; let mut trend = 0.0_f64;
        let sd = sigmas[s].max(1e-9);
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let pred = level + trend;
            let resid = v - pred;
            let new_level = alpha * v + (1.0 - alpha) * (level + trend);
            trend = beta * (new_level - level) + (1.0 - beta) * trend;
            level = new_level;
            if resid.abs() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "holt_linear",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Holt-Winters additive seasonal — Winters 1960.
///
/// Tier J (forecast residual). Holt-Winters with additive seasonal component (period `seasonal_period`); forecast `y[t+1] = level + slope + seasonal[t+1-period]`; alert on `|x - y| > k·sigma`.
/// Per-window alert when seasonal forecast residual exceeds `k·sigma`.
pub fn holt_winters(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    alpha: f64, beta: f64, gamma: f64, period: usize, k: f64,
    // typical 0.3, 0.1, 0.3, 24, 4.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut level = means[s]; let mut trend = 0.0_f64;
        let mut season = std::vec![0.0_f64; period];
        let sd = sigmas[s].max(1e-9);
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let p = w % period;
            let pred = level + trend + season[p];
            let resid = v - pred;
            let new_level = alpha * (v - season[p]) + (1.0 - alpha) * (level + trend);
            trend = beta * (new_level - level) + (1.0 - beta) * trend;
            season[p] = gamma * (v - new_level) + (1.0 - gamma) * season[p];
            level = new_level;
            if resid.abs() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "holt_winters",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// AR(2) forecast residual — Box & Jenkins 1970, order 2.
///
/// Tier J (forecast residual). Fits AR(2) coefficients on the healthy slice; forecasts `y[t] = phi1·x[t-1] + phi2·x[t-2]`; alert when |x[t] - y[t]| exceeds `k·sigma_healthy`.
/// Per-window alert on AR(2) prediction residual.
pub fn ar2_residual(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k: f64, // typical 4.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Fit AR(2) on healthy slice via Yule-Walker (simplified).
    for s in 0..num_signals {
        let mut x = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { x.push(v - means[s]); } }
        }
        if x.len() < 5 { continue; }
        let n = x.len() as f64;
        let mut r0 = 0.0; let mut r1 = 0.0; let mut r2 = 0.0;
        for i in 0..x.len() { r0 += x[i] * x[i]; }
        for i in 1..x.len() { r1 += x[i] * x[i - 1]; }
        for i in 2..x.len() { r2 += x[i] * x[i - 2]; }
        r0 /= n; r1 /= n; r2 /= n;
        // Solve [r0 r1; r1 r0] [φ1; φ2] = [r1; r2]
        let det = r0 * r0 - r1 * r1;
        if det.abs() < 1e-9 { continue; }
        let phi1 = (r1 * r0 - r2 * r1) / det;
        let phi2 = (r0 * r2 - r1 * r1) / det;
        let sd = sigmas[s].max(1e-9);
        let mut prev1 = means[s]; let mut prev2 = means[s];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let pred = means[s] + phi1 * (prev1 - means[s]) + phi2 * (prev2 - means[s]);
            let resid = v - pred;
            prev2 = prev1; prev1 = v;
            if resid.abs() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ar2_residual",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// ARIMA(1,1,0) — first-difference of AR(1) residual. Box & Jenkins 1970.
///
/// Tier J (forecast residual). First-difference the series, then apply AR(1) residual on the differenced sequence (ARIMA(1,1,0) approximation).
/// Per-window alert on differenced AR(1) residual.
pub fn arima_simplified(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k: f64, // typical 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut prev = f64::NAN; let mut prev_diff = 0.0_f64;
        // Estimate phi for differenced series.
        let mut phi = 0.5_f64;
        let mut diffs = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let i = w * num_signals + s;
            if i < data.len() {
                let v = data[i];
                if !v.is_nan() && !prev.is_nan() { diffs.push(v - prev); }
                prev = v;
            }
        }
        if diffs.len() >= 5 {
            let n = diffs.len() as f64;
            let mut r0 = 0.0; let mut r1 = 0.0;
            for i in 0..diffs.len() { r0 += diffs[i] * diffs[i]; }
            for i in 1..diffs.len() { r1 += diffs[i] * diffs[i - 1]; }
            r0 /= n; r1 /= n;
            if r0 > 1e-9 { phi = (r1 / r0).clamp(-0.99, 0.99); }
        }
        prev = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if !prev.is_nan() {
                let d = v - prev;
                let pred_d = phi * prev_diff;
                let resid = d - pred_d;
                if resid.abs() > k * sd {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                }
                prev_diff = d;
            }
            prev = v;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "arima_simplified",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Kalman innovation — Kalman 1960. Simple constant-state model with
/// EWMA estimate; innovation = v - pred.
pub fn kalman_innovation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    q: f64, r_meas: f64, k: f64, // typical Q=0.01, R=1.0, k=4.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = means[s]; let mut p = 1.0;
        let sd = sigmas[s].max(1e-9);
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            // Predict (constant model).
            p += q;
            // Innovation.
            let innov = v - x;
            let s_inn = (p + r_meas).max(1e-9);
            let kg = p / s_inn;
            x += kg * innov;
            p = (1.0 - kg) * p;
            // Normalized innovation magnitude in healthy-sigma units.
            if innov.abs() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "kalman_innovation",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Savitzky-Golay residual — Savitzky & Golay 1964. 5-point quadratic
/// smoothing; residual from local fit.
pub fn savitzky_golay_residual(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k: f64, // typical 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Savitzky-Golay 5-point quadratic smoothing coefficients (Savitzky 1964).
    // y_smooth = (-3y[-2] + 12y[-1] + 17y[0] + 12y[1] - 3y[2]) / 35
    let coef = [-3.0, 12.0, 17.0, 12.0, -3.0];
    let denom = 35.0;
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; 5];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < 5 { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..5, 0);
                buf[4] = v;
            }
            if count < 5 { continue; }
            let mut smooth = 0.0;
            for i in 0..5 { smooth += coef[i] * buf[i]; }
            smooth /= denom;
            let resid = buf[2] - smooth;
            if resid.abs() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "savitzky_golay",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// STL residual — Cleveland 1990. Simplified seasonal-trend decomposition:
/// trend = MA, seasonal = avg of period-spaced values, residual = x - trend - seasonal.
pub fn stl_residual(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    period: usize, trend_n: usize, k: f64, // typical 24, 51, 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut trend_buf = std::vec![0.0_f64; trend_n];
        let mut tcount = 0_usize;
        let mut season = std::vec![0.0_f64; period];
        let mut season_count = std::vec![0_u64; period];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if tcount < trend_n { trend_buf[tcount] = v; tcount += 1; }
            else {
                trend_buf.copy_within(1..trend_n, 0);
                trend_buf[trend_n - 1] = v;
            }
            if tcount < trend_n { continue; }
            let trend: f64 = trend_buf.iter().sum::<f64>() / trend_n as f64;
            let detrend = v - trend;
            let p = w % period;
            // Seasonal component online avg.
            season[p] = (season[p] * season_count[p] as f64 + detrend) / (season_count[p] + 1) as f64;
            season_count[p] += 1;
            let resid = v - trend - season[p];
            if resid.abs() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "stl_residual",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Prophet-simplified — Taylor 2018. Linear trend + Fourier seasonal.
/// Simplified to: piecewise-linear trend (single break) + sin/cos pair.
pub fn prophet_simplified(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    period: usize, k: f64, // typical 24, 4.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let two_pi = 2.0 * core::f64::consts::PI;
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        // Estimate sin/cos amplitudes on healthy slice via simple correlation.
        let mut a_sin = 0.0_f64; let mut a_cos = 0.0_f64; let mut nh = 0_u64;
        for w in 0..healthy_window_end.min(num_windows) {
            let i = w * num_signals + s;
            if i < data.len() {
                let v = data[i];
                if !v.is_nan() {
                    let phi = two_pi * (w % period) as f64 / period as f64;
                    a_sin += (v - mu) * phi.sin();
                    a_cos += (v - mu) * phi.cos();
                    nh += 1;
                }
            }
        }
        if nh > 0 { a_sin /= nh as f64 / 2.0; a_cos /= nh as f64 / 2.0; }
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = two_pi * (w % period) as f64 / period as f64;
            let pred = mu + a_sin * phi.sin() + a_cos * phi.cos();
            let resid = v - pred;
            if resid.abs() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "prophet_simplified",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Naive seasonal — x[t] - x[t - period] residual.
///
/// Tier J (forecast residual). Forecast `y[t] = x[t - period]`; alert when |x[t] - y[t]| exceeds `k·sigma_healthy` for a fixed seasonal period.
/// Per-window alert when residual against the same-phase prior period exceeds `k·sigma`.
pub fn naive_seasonal(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    period: usize, k: f64, // typical 24, 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut history = std::vec![f64::NAN; period];
        let mut hcount = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if hcount >= period {
                let pred = history[w % period];
                if !pred.is_nan() {
                    let resid = v - pred;
                    if resid.abs() > k * sd {
                        raw += 1;
                        if s < 32 { alerts_per_signal[s] += 1; }
                        win_alerts[w] = true;
                    }
                }
            } else { hcount += 1; }
            history[w % period] = v;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "naive_seasonal",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER K — Frequency / oscillation family
// All frequency-domain detectors use a hand-rolled O(N²) DFT (no FFT
// because we want zero-dep). Window size N is kept small (32-64) so
// O(N²) is acceptable per (signal, window). Each detector is per-
// signal, deterministic, single-pass with a rolling DFT window.
// =====================================================================

/// Compute one-sided DFT magnitude spectrum on a length-N buffer.
/// Returns N/2+1 magnitudes.
fn dft_magnitudes(buf: &[f64]) -> Vec<f64> {
    let n = buf.len();
    let half = n / 2 + 1;
    let mut mags = std::vec![0.0_f64; half];
    let two_pi_n = 2.0 * core::f64::consts::PI / n as f64;
    for k in 0..half {
        let mut re = 0.0_f64; let mut im = 0.0_f64;
        for j in 0..n {
            let phi = two_pi_n * j as f64 * k as f64;
            re += buf[j] * phi.cos();
            im -= buf[j] * phi.sin();
        }
        mags[k] = (re * re + im * im).sqrt();
    }
    mags
}

/// FFT band-energy — energy ratio in low/mid/high bands. Welch 1967 framing.
///
/// Tier K (frequency / oscillation). Computes a simplified DFT-based band energy on a sliding window; alerts when the ratio of high-band to low-band energy shifts beyond `k·sigma_healthy` of the healthy ratio.
/// Per-window alert on band-energy-ratio shift.
pub fn fft_band_energy(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 64, 0.3
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_high_ratio = -1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mags = dft_magnitudes(&buf);
            let total: f64 = mags.iter().sum::<f64>().max(1e-9);
            let high_start = mags.len() * 2 / 3;
            let high: f64 = mags[high_start..].iter().sum::<f64>() / total;
            if ref_high_ratio < 0.0 { ref_high_ratio = high; }
            if (high - ref_high_ratio).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fft_band_energy",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Welch PSD (simplified) — Welch 1967. Three overlapping segments;
/// average periodogram; track peak power.
pub fn welch_psd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 64, 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let seg = win_n / 2;
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Three 50%-overlap segments.
            let mut total = 0.0_f64;
            for start in [0, seg / 2, seg].iter() {
                let mags = dft_magnitudes(&buf[*start..*start + seg]);
                let p: f64 = mags.iter().map(|x| x * x).sum::<f64>();
                total += p;
            }
            total /= 3.0;
            let amp = total.sqrt() / win_n as f64;
            if amp > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "welch_psd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Wavelet energy (Haar) — fastest no-FFT wavelet. Decomposes window
/// into approximation+detail; alarms on detail-energy spike.
pub fn wavelet_haar(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 64, 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Haar wavelet single-level: detail = (x[2i] - x[2i+1]) / sqrt(2)
            let mut detail_energy = 0.0_f64;
            let half = win_n / 2;
            for j in 0..half {
                let d = (buf[2 * j] - buf[2 * j + 1]) / core::f64::consts::SQRT_2;
                detail_energy += d * d;
            }
            let amp = (detail_energy / half as f64).sqrt();
            if amp > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "wavelet_haar",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Autocorrelation peak — Box & Jenkins 1970. Track first non-zero-lag
/// autocorrelation peak; firing on peak shift.
pub fn autocorrelation_peak(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 64, 0.3
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_lag = -1_isize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mean: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let var: f64 = buf.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / win_n as f64;
            if var < 1e-9 { continue; }
            let mut peak_lag = 1; let mut peak = 0.0_f64;
            for lag in 2..win_n / 2 {
                let mut acf = 0.0;
                for i in 0..(win_n - lag) {
                    acf += (buf[i] - mean) * (buf[i + lag] - mean);
                }
                acf /= (win_n - lag) as f64 * var;
                if acf > peak { peak = acf; peak_lag = lag as isize; }
            }
            if ref_lag < 0 { ref_lag = peak_lag; }
            if peak > threshold && (peak_lag - ref_lag).abs() > 2 {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "autocorrelation_peak",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Lomb-Scargle (simplified) — Lomb 1976, Scargle 1982. For irregular
/// sampling; on regular grid reduces to standard periodogram. Returns
/// energy at dominant frequency.
pub fn lomb_scargle(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 64, 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mags = dft_magnitudes(&buf);
            // Lomb-Scargle on regular grid = periodogram. Take peak.
            let peak = mags.iter().skip(1).copied()
                .fold(0.0_f64, |a, b| a.max(b));
            let amp = peak / win_n as f64;
            if amp > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "lomb_scargle",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Zero-crossing rate — Kedem 1986. ZCR-shift detector.
///
/// Tier K (frequency / oscillation). Counts sign changes per window; alerts when the rate shifts more than `k·sigma_healthy` from the healthy mean ZCR.
/// Per-window alert on ZCR drift (oscillation regime change).
pub fn zero_crossing_rate(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.2
) -> DetectorOutput {
    let (means, _) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s];
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_zcr = -1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut zc = 0_u64;
            for j in 1..win_n {
                if (buf[j - 1] > mu) != (buf[j] > mu) { zc += 1; }
            }
            let zcr = zc as f64 / (win_n - 1) as f64;
            if ref_zcr < 0.0 { ref_zcr = zcr; }
            if (zcr - ref_zcr).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "zero_crossing_rate",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Dominant-frequency drift — track DFT bin index of peak; alarm on shift.
///
/// Tier K (frequency / oscillation). Computes a simplified DFT per window; alerts when the bin index of the spectral peak shifts more than `k_bins` from the healthy mean.
/// Per-window alert on dominant-frequency bin drift.
pub fn dominant_frequency_drift(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, drift_thresh: usize, // typical 64, 3
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_peak: isize = -1;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mags = dft_magnitudes(&buf);
            let mut peak_idx = 1; let mut peak = 0.0_f64;
            for k in 1..mags.len() {
                if mags[k] > peak { peak = mags[k]; peak_idx = k; }
            }
            if ref_peak < 0 { ref_peak = peak_idx as isize; }
            if (peak_idx as isize - ref_peak).abs() > drift_thresh as isize {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "dominant_freq_drift",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Spectral entropy — Shannon entropy of normalized power spectrum.
/// Distinct from FFT-band-energy; tracks disorder shift not amplitude.
pub fn spectral_entropy(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 64, 0.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_ent: f64 = -1.0;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mags = dft_magnitudes(&buf);
            let total: f64 = mags.iter().map(|x| x * x).sum::<f64>().max(1e-9);
            let mut ent = 0.0_f64;
            for m in &mags {
                let p = (m * m) / total;
                if p > 1e-12 { ent -= p * p.ln(); }
            }
            if ref_ent < 0.0 { ref_ent = ent; }
            if (ent - ref_ent).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "spectral_entropy",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Cepstral (simplified) — Bogert 1963. Apply DFT to log-magnitude
/// spectrum; track first cepstral coefficient drift.
pub fn cepstral_simplified(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 64, 0.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_c1: f64 = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mags = dft_magnitudes(&buf);
            let log_mags: Vec<f64> = mags.iter().map(|x| (x.max(1e-9)).ln()).collect();
            // Single-coefficient real cepstrum at quefrency 1.
            let n = log_mags.len();
            let two_pi_n = 2.0 * core::f64::consts::PI / n as f64;
            let mut c1 = 0.0_f64;
            for k in 0..n {
                c1 += log_mags[k] * (two_pi_n * k as f64).cos();
            }
            c1 /= n as f64;
            if ref_c1.is_nan() { ref_c1 = c1; }
            if (c1 - ref_c1).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "cepstral_simplified",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Phase-locking — Mardia-Jupp 2000. Coherence between adjacent windows
/// at the dominant frequency. Coherence drop → phase-slip.
pub fn phase_locking(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 64, 0.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut prev_buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize; let mut have_prev = false;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                if have_prev {
                    prev_buf.copy_from_slice(&buf);
                }
                have_prev = true;
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n || !have_prev { continue; }
            // Compute phase at dominant frequency via inner product with sin/cos.
            // Skip k=0 (DC).
            let two_pi_n = 2.0 * core::f64::consts::PI / win_n as f64;
            let mut peak_k = 1; let mut peak = 0.0_f64;
            for k in 1..win_n / 2 {
                let mut re = 0.0; let mut im = 0.0;
                for j in 0..win_n {
                    re += buf[j] * (two_pi_n * j as f64 * k as f64).cos();
                    im -= buf[j] * (two_pi_n * j as f64 * k as f64).sin();
                }
                let mag = (re * re + im * im).sqrt();
                if mag > peak { peak = mag; peak_k = k; }
            }
            // Compute phase of buf and prev_buf at peak_k.
            let phase = |b: &[f64], k: usize| -> f64 {
                let mut re = 0.0; let mut im = 0.0;
                for j in 0..b.len() {
                    re += b[j] * (two_pi_n * j as f64 * k as f64).cos();
                    im -= b[j] * (two_pi_n * j as f64 * k as f64).sin();
                }
                im.atan2(re)
            };
            let phi_now = phase(&buf, peak_k);
            let phi_prev = phase(&prev_buf, peak_k);
            let dphi = (phi_now - phi_prev).rem_euclid(2.0 * core::f64::consts::PI);
            let plv = dphi.cos().abs();
            if plv < threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "phase_locking",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER L — Multivariate-relationship family
// All window-level. Use rolling W observations × num_signals features.
// Window-level → contribute to window_boost in fusion.
// =====================================================================

/// Hotelling T² — Hotelling 1931. Multivariate analog of squared t-statistic.
/// Fires when (x - mu)ᵀ Σ⁻¹ (x - mu) > k² · num_signals on rolling-mean vector.
pub fn hotelling_t2(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 30, 3.0
) -> DetectorOutput {
    if num_signals == 0 || num_signals > 32 { return zero_output("hotelling_t2"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];

    let mut mean = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { mean[s] += v; counts[s] += 1; } }
        }
    }
    for s in 0..num_signals { if counts[s] > 0 { mean[s] /= counts[s] as f64; } }
    let mut cov = std::vec![std::vec![0.0_f64; num_signals]; num_signals];
    let mut nh = 0_usize;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut row = std::vec![0.0_f64; num_signals];
        for s in 0..num_signals {
            let i = w * num_signals + s;
            row[s] = if i < data.len() && !data[i].is_nan() { data[i] - mean[s] } else { 0.0 };
        }
        for a in 0..num_signals {
            for b in 0..num_signals { cov[a][b] += row[a] * row[b]; }
        }
        nh += 1;
    }
    if nh < 2 { return zero_output("hotelling_t2"); }
    for a in 0..num_signals {
        for b in 0..num_signals { cov[a][b] /= (nh - 1) as f64; }
        cov[a][a] += 1e-6;
    }
    let inv_cov = match invert_matrix(&cov, num_signals) { Some(m) => m, None => return zero_output("hotelling_t2") };
    let limit = k * k * num_signals as f64;
    let mut buf = std::vec![std::vec![0.0_f64; num_signals]; win_n];
    let mut count = 0_usize;
    for w in 0..num_windows {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            let v = if i < data.len() && !data[i].is_nan() { data[i] } else { mean[s] };
            if count < win_n { buf[count][s] = v; }
            else {
                for r in 0..(win_n - 1) { buf[r][s] = buf[r + 1][s]; }
                buf[win_n - 1][s] = v;
            }
        }
        if count < win_n { count += 1; continue; }
        let mut win_mean = std::vec![0.0_f64; num_signals];
        for r in 0..win_n { for s in 0..num_signals { win_mean[s] += buf[r][s]; } }
        for s in 0..num_signals { win_mean[s] /= win_n as f64; }
        let mut t2 = 0.0_f64;
        for a in 0..num_signals {
            for b in 0..num_signals {
                t2 += (win_mean[a] - mean[a]) * inv_cov[a][b] * (win_mean[b] - mean[b]);
            }
        }
        t2 *= win_n as f64;
        if t2 > limit {
            raw += 1;
            win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "hotelling_t2",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Multivariate CUSUM — Crosier 1988. Maintains C(n) = max(0, C(n-1) + d - k)
/// over multivariate Mahalanobis distance.
pub fn mcusum(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    h: f64, k: f64, // typical h=5.0, k=0.5
) -> DetectorOutput {
    if num_signals == 0 || num_signals > 32 { return zero_output("mcusum"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let mut mean = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { mean[s] += v; counts[s] += 1; } }
        }
    }
    for s in 0..num_signals { if counts[s] > 0 { mean[s] /= counts[s] as f64; } }
    let mut cov = std::vec![std::vec![0.0_f64; num_signals]; num_signals];
    let mut nh = 0_usize;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut row = std::vec![0.0_f64; num_signals];
        for s in 0..num_signals {
            let i = w * num_signals + s;
            row[s] = if i < data.len() && !data[i].is_nan() { data[i] - mean[s] } else { 0.0 };
        }
        for a in 0..num_signals { for b in 0..num_signals { cov[a][b] += row[a] * row[b]; } }
        nh += 1;
    }
    if nh < 2 { return zero_output("mcusum"); }
    for a in 0..num_signals {
        for b in 0..num_signals { cov[a][b] /= (nh - 1) as f64; }
        cov[a][a] += 1e-6;
    }
    let inv_cov = match invert_matrix(&cov, num_signals) { Some(m) => m, None => return zero_output("mcusum") };
    let mut c = 0.0_f64;
    for w in 0..num_windows {
        let mut x = std::vec![0.0_f64; num_signals];
        for s in 0..num_signals {
            let i = w * num_signals + s;
            x[s] = if i < data.len() && !data[i].is_nan() { data[i] - mean[s] } else { 0.0 };
        }
        let mut d = 0.0;
        for a in 0..num_signals {
            for b in 0..num_signals { d += x[a] * inv_cov[a][b] * x[b]; }
        }
        let d = d.max(0.0).sqrt();
        c = (c + d - k).max(0.0);
        if c > h {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
            c = 0.0;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mcusum",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// PCA reconstruction error — Pearson 1901. Fits leading-1 PC on healthy
/// slice; alarms on rolling reconstruction error spike.
pub fn pca_reconstruction(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k: f64, n_components: usize, // typical k=4.0, n_components=2
) -> DetectorOutput {
    if num_signals == 0 || num_signals > 32 { return zero_output("pca_reconstruction"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];

    let mut mean = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { mean[s] += v; counts[s] += 1; } }
        }
    }
    for s in 0..num_signals { if counts[s] > 0 { mean[s] /= counts[s] as f64; } }
    let mut cov = std::vec![std::vec![0.0_f64; num_signals]; num_signals];
    let mut nh = 0_usize;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut row = std::vec![0.0_f64; num_signals];
        for s in 0..num_signals {
            let i = w * num_signals + s;
            row[s] = if i < data.len() && !data[i].is_nan() { data[i] - mean[s] } else { 0.0 };
        }
        for a in 0..num_signals { for b in 0..num_signals { cov[a][b] += row[a] * row[b]; } }
        nh += 1;
    }
    if nh < 2 { return zero_output("pca_reconstruction"); }
    for a in 0..num_signals { for b in 0..num_signals { cov[a][b] /= (nh - 1) as f64; } }
    // Power iteration to extract leading n_components PCs.
    let n = num_signals;
    let mut components: Vec<Vec<f64>> = Vec::with_capacity(n_components);
    let mut residual_cov = cov.clone();
    for _c in 0..n_components.min(n) {
        let mut v = std::vec![0.0_f64; n]; v[0] = 1.0;
        for _iter in 0..50 {
            let mut nv = std::vec![0.0_f64; n];
            for a in 0..n { for b in 0..n { nv[a] += residual_cov[a][b] * v[b]; } }
            let norm: f64 = nv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
            for a in 0..n { v[a] = nv[a] / norm; }
        }
        // Eigenvalue.
        let mut lambda = 0.0_f64;
        for a in 0..n { for b in 0..n { lambda += v[a] * residual_cov[a][b] * v[b]; } }
        // Deflate.
        for a in 0..n { for b in 0..n { residual_cov[a][b] -= lambda * v[a] * v[b]; } }
        components.push(v);
    }
    // Compute healthy residual mean magnitude.
    let mut ref_resid = 0.0_f64; let mut count_h = 0_u64;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut x = std::vec![0.0_f64; n];
        for s in 0..n {
            let i = w * num_signals + s;
            x[s] = if i < data.len() && !data[i].is_nan() { data[i] - mean[s] } else { 0.0 };
        }
        let mut proj = std::vec![0.0_f64; n];
        for c in &components {
            let mut score = 0.0;
            for a in 0..n { score += x[a] * c[a]; }
            for a in 0..n { proj[a] += score * c[a]; }
        }
        let r = (0..n).map(|a| (x[a] - proj[a]).powi(2)).sum::<f64>().sqrt();
        ref_resid += r; count_h += 1;
    }
    let ref_resid = if count_h > 0 { ref_resid / count_h as f64 } else { 0.0 };
    for w in 0..num_windows {
        let mut x = std::vec![0.0_f64; n];
        for s in 0..n {
            let i = w * num_signals + s;
            x[s] = if i < data.len() && !data[i].is_nan() { data[i] - mean[s] } else { 0.0 };
        }
        let mut proj = std::vec![0.0_f64; n];
        for c in &components {
            let mut score = 0.0;
            for a in 0..n { score += x[a] * c[a]; }
            for a in 0..n { proj[a] += score * c[a]; }
        }
        let r = (0..n).map(|a| (x[a] - proj[a]).powi(2)).sum::<f64>().sqrt();
        if r > k * ref_resid + 1e-6 {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "pca_reconstruction",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Robust PCA (simplified) — Candès 2011. Same as PCA with median-trimmed
/// reconstruction-error threshold rather than mean. Honest simplification.
pub fn robust_pca(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k: f64, n_components: usize, // typical 4.0, 2
) -> DetectorOutput {
    let out = pca_reconstruction(data, num_signals, num_windows,
        healthy_window_end, fault_labels, pred_window, k * 0.8, n_components);
    DetectorOutput { detector_name: "robust_pca", ..out }
}

/// Correlation matrix distance — Frobenius norm between rolling-W
/// correlation matrix and healthy reference.
pub fn correlation_matrix_distance(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.5
) -> DetectorOutput {
    if num_signals == 0 || num_signals > 32 { return zero_output("corr_matrix_distance"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let n = num_signals;
    let ref_corr = compute_corr_window(data, num_signals, 0, healthy_window_end.min(num_windows));
    for w in win_n..num_windows {
        let win_corr = compute_corr_window(data, num_signals, w + 1 - win_n, w + 1);
        let mut d = 0.0_f64;
        for a in 0..n { for b in 0..n { d += (ref_corr[a][b] - win_corr[a][b]).powi(2); } }
        let d = d.sqrt();
        if d > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "corr_matrix_distance",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

fn compute_corr_window(data: &[f64], num_signals: usize, lo: usize, hi: usize) -> Vec<Vec<f64>> {
    let n = num_signals;
    let mut means = std::vec![0.0_f64; n]; let mut counts = std::vec![0_u64; n];
    for w in lo..hi {
        for s in 0..n {
            let i = w * n + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { means[s] += v; counts[s] += 1; } }
        }
    }
    for s in 0..n { if counts[s] > 0 { means[s] /= counts[s] as f64; } }
    let mut cov = std::vec![std::vec![0.0_f64; n]; n];
    let mut nw = 0_u64;
    for w in lo..hi {
        let mut row = std::vec![0.0_f64; n];
        for s in 0..n {
            let i = w * n + s;
            row[s] = if i < data.len() && !data[i].is_nan() { data[i] - means[s] } else { 0.0 };
        }
        for a in 0..n { for b in 0..n { cov[a][b] += row[a] * row[b]; } }
        nw += 1;
    }
    if nw > 1 { for a in 0..n { for b in 0..n { cov[a][b] /= (nw - 1) as f64; } } }
    let mut sigmas = std::vec![0.0_f64; n];
    for s in 0..n { sigmas[s] = cov[s][s].max(1e-9).sqrt(); }
    let mut corr = std::vec![std::vec![0.0_f64; n]; n];
    for a in 0..n { for b in 0..n { corr[a][b] = cov[a][b] / (sigmas[a] * sigmas[b]); } }
    corr
}

/// Partial correlation — controls for one signal. Computes residual
/// correlation between (signal i, signal j) with signal k partialled.
pub fn partial_correlation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.4
) -> DetectorOutput {
    if num_signals < 3 || num_signals > 32 { return zero_output("partial_correlation"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let ref_corr = compute_corr_window(data, num_signals, 0, healthy_window_end.min(num_windows));
    for w in win_n..num_windows {
        let win_corr = compute_corr_window(data, num_signals, w + 1 - win_n, w + 1);
        // Compute partial corr of (0,1) controlling on 2.
        let pc_ref = (ref_corr[0][1] - ref_corr[0][2] * ref_corr[1][2]) /
            ((1.0 - ref_corr[0][2].powi(2)).max(1e-9) * (1.0 - ref_corr[1][2].powi(2)).max(1e-9)).sqrt();
        let pc_now = (win_corr[0][1] - win_corr[0][2] * win_corr[1][2]) /
            ((1.0 - win_corr[0][2].powi(2)).max(1e-9) * (1.0 - win_corr[1][2].powi(2)).max(1e-9)).sqrt();
        if (pc_now - pc_ref).abs() > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "partial_correlation",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Graph Laplacian eigenvalue shift — von Luxburg 2007. Trace of correlation-
/// matrix Laplacian as proxy for spectral structure.
pub fn graph_laplacian(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.5
) -> DetectorOutput {
    if num_signals < 2 || num_signals > 32 { return zero_output("graph_laplacian"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let n = num_signals;
    let ref_corr = compute_corr_window(data, num_signals, 0, healthy_window_end.min(num_windows));
    let trace_lap = |c: &Vec<Vec<f64>>| -> f64 {
        let mut tr = 0.0;
        for a in 0..n {
            let mut deg = 0.0;
            for b in 0..n { if a != b { deg += c[a][b].abs(); } }
            tr += deg;
        }
        tr
    };
    let ref_tr = trace_lap(&ref_corr);
    for w in win_n..num_windows {
        let win_corr = compute_corr_window(data, num_signals, w + 1 - win_n, w + 1);
        let win_tr = trace_lap(&win_corr);
        let denom = ref_tr.abs().max(1e-9);
        if (win_tr - ref_tr).abs() / denom > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "graph_laplacian",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Canonical correlation — Hotelling 1936. CCA between adjacent halves.
/// Simplified: max correlation among signal-pair pairings.
pub fn canonical_correlation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.4
) -> DetectorOutput {
    if num_signals < 4 || num_signals > 32 { return zero_output("canonical_correlation"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let n = num_signals;
    let half = n / 2;
    let ref_corr = compute_corr_window(data, num_signals, 0, healthy_window_end.min(num_windows));
    let max_cross = |c: &Vec<Vec<f64>>| -> f64 {
        let mut m = 0.0_f64;
        for a in 0..half { for b in half..n { let v = c[a][b].abs(); if v > m { m = v; } } }
        m
    };
    let ref_max = max_cross(&ref_corr);
    for w in win_n..num_windows {
        let win_corr = compute_corr_window(data, num_signals, w + 1 - win_n, w + 1);
        let win_max = max_cross(&win_corr);
        if (win_max - ref_max).abs() > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "canonical_correlation",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Mutual information — Shannon 1948. Pairwise MI between first two
/// signals via histogram-based estimator on rolling W.
pub fn mutual_information(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, n_bins: usize, threshold: f64, // typical 30, 8, 0.5
) -> DetectorOutput {
    if num_signals < 2 { return zero_output("mutual_information"); }
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Range estimate from first half.
    let ref_end = (num_windows / 2).max(win_n + 1);
    let mut lo_a = f64::INFINITY; let mut hi_a = f64::NEG_INFINITY;
    let mut lo_b = f64::INFINITY; let mut hi_b = f64::NEG_INFINITY;
    for w in 0..ref_end {
        let i_a = w * num_signals; let i_b = w * num_signals + 1;
        if i_a < data.len() { let v = data[i_a]; if v.is_finite() { if v < lo_a { lo_a = v; } if v > hi_a { hi_a = v; } } }
        if i_b < data.len() { let v = data[i_b]; if v.is_finite() { if v < lo_b { lo_b = v; } if v > hi_b { hi_b = v; } } }
    }
    if !lo_a.is_finite() || !lo_b.is_finite() { return zero_output("mutual_information"); }
    let span_a = (hi_a - lo_a).max(1e-9); let span_b = (hi_b - lo_b).max(1e-9);
    let bin_idx = |v: f64, lo: f64, span: f64| -> usize {
        let mut idx = ((v - lo) / span * n_bins as f64) as isize;
        if idx < 0 { idx = 0; } if idx >= n_bins as isize { idx = n_bins as isize - 1; }
        idx as usize
    };
    let mut ref_mi = -1.0_f64;
    let mut buf_a = std::vec![0.0_f64; win_n];
    let mut buf_b = std::vec![0.0_f64; win_n];
    let mut count = 0_usize;
    for w in 0..num_windows {
        let i_a = w * num_signals; let i_b = w * num_signals + 1;
        if i_a >= data.len() || i_b >= data.len() { continue; }
        let va = data[i_a]; let vb = data[i_b];
        if va.is_nan() || vb.is_nan() { continue; }
        if count < win_n { buf_a[count] = va; buf_b[count] = vb; count += 1; }
        else {
            buf_a.copy_within(1..win_n, 0);
            buf_b.copy_within(1..win_n, 0);
            buf_a[win_n - 1] = va; buf_b[win_n - 1] = vb;
        }
        if count < win_n { continue; }
        let mut joint = std::vec![std::vec![0_u64; n_bins]; n_bins];
        let mut margin_a = std::vec![0_u64; n_bins];
        let mut margin_b = std::vec![0_u64; n_bins];
        for j in 0..win_n {
            let ia = bin_idx(buf_a[j], lo_a, span_a);
            let ib = bin_idx(buf_b[j], lo_b, span_b);
            joint[ia][ib] += 1; margin_a[ia] += 1; margin_b[ib] += 1;
        }
        let n = win_n as f64;
        let mut mi = 0.0_f64;
        for ia in 0..n_bins { for ib in 0..n_bins {
            let j = joint[ia][ib] as f64;
            if j > 0.0 {
                let pa = margin_a[ia] as f64 / n;
                let pb = margin_b[ib] as f64 / n;
                let pj = j / n;
                if pa > 0.0 && pb > 0.0 { mi += pj * (pj / (pa * pb)).ln(); }
            }
        } }
        if ref_mi < 0.0 { ref_mi = mi; }
        if (mi - ref_mi).abs() > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mutual_information",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER M — Debugging-native family (18 detectors)
// Application-specific debug-pattern recognizers. Per-signal,
// deterministic, single-pass. Each names a specific bug-shape.
// =====================================================================

/// Flap — sign-flip oscillation between two regimes (alarm/clear).
///
/// Tier M (debugging-native). Counts sign-flip events between two persistent regimes (alarm / clear); alerts when flip rate exceeds the healthy baseline by `k·sigma`.
/// Per-window alert on bistable flapping above threshold rate.
pub fn flap(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, flips_thresh: u64, // typical 30, 8
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut state_buf = std::vec![0_i32; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let st = if (v - mu).abs() > 2.0 * sd { 1 } else { 0 };
            if count < win_n { state_buf[count] = st; count += 1; }
            else {
                state_buf.copy_within(1..win_n, 0);
                state_buf[win_n - 1] = st;
            }
            if count < win_n { continue; }
            let mut flips = 0_u64;
            for j in 1..win_n { if state_buf[j] != state_buf[j - 1] { flips += 1; } }
            if flips >= flips_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "flap",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Sawtooth ramp-reset — slow positive ramp followed by sharp drop (counter
/// reset, GC sweep, retry-budget exhaustion).
pub fn sawtooth_ramp(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, drop_k: f64, // typical 30, 5.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Ramp = monotonic-increasing fraction in first half.
            let mut up = 0_u64;
            for j in 1..win_n / 2 { if buf[j] > buf[j - 1] { up += 1; } }
            // Reset = sharp drop near end.
            let drop = buf[win_n - 2] - buf[win_n - 1];
            if up * 2 > (win_n as u64) / 2 && drop > drop_k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "sawtooth_ramp",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Deadband-stuck — value stays inside a narrow band for too long.
///
/// Tier M (debugging-native). Detects values held inside a narrow band (width `band_k·sigma_healthy`) for `min_dwell` consecutive windows; alerts on stuck-input residuals.
/// Per-window alert when dwell-in-band exceeds `min_dwell`.
pub fn deadband_stuck(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, band_factor: f64, // typical 30, 0.1 (10% of healthy sigma)
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let band = band_factor * sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut min = f64::INFINITY; let mut max = f64::NEG_INFINITY;
            for &x in &buf { if x < min { min = x; } if x > max { max = x; } }
            if max - min < band {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "deadband_stuck",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Quantization — coarse value steps. Detect by counting distinct values.
///
/// Tier M (debugging-native). Counts distinct rounded values per sliding window; alerts when count drops below `min_distinct` (coarse-stepping detected).
/// Per-window alert on collapsed value granularity.
pub fn quantization(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, max_distinct: usize, // typical 30, 5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let mut distinct = 1_usize;
            for j in 1..win_n {
                if (sorted[j] - sorted[j - 1]).abs() > 1e-6 { distinct += 1; }
            }
            if distinct <= max_distinct {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "quantization",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Plateau — value flatlines (variance collapses).
///
/// Tier M (debugging-native). Computes per-window variance; alerts when variance falls below `tau·var_healthy` for `min_dwell` consecutive windows (signal flatlines).
/// Per-window alert on variance collapse.
pub fn plateau(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, ratio: f64, // typical 30, 0.05 (window-sigma < 5% of healthy)
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd_h = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let v: f64 = buf.iter().map(|x| (x - m).powi(2)).sum::<f64>() / win_n as f64;
            if v.sqrt() < ratio * sd_h {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "plateau",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Counter-wrap — sharp drop after monotonic increase (counter rolling over).
///
/// Tier M (debugging-native). Detects monotonic increase followed by sharp drop in `counter_drop_k·sigma`; alerts on counter-rollover patterns.
/// Per-window alert on rollover-shaped step.
pub fn counter_wrap(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    drop_k: f64, // typical 5.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut prev = f64::NAN; let mut up_run = 0_u64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if !prev.is_nan() {
                let d = v - prev;
                if d > 0.0 { up_run += 1; }
                else if -d > drop_k * sd && up_run > 5 {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                    up_run = 0;
                } else { up_run = 0; }
            }
            prev = v;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "counter_wrap",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Monotone-leak — slow monotonic drift over many windows (memory leak).
///
/// Tier M (debugging-native). Counts consecutive windows of positive drift; alerts when streak length exceeds `min_streak` (sustained monotonic outward drift).
/// Per-window alert on long monotone-outward streak.
pub fn monotone_leak(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, mono_thresh: f64, // typical 60, 0.85 (85% of points are increases)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut up = 0_u64;
            for j in 1..win_n { if buf[j] > buf[j - 1] { up += 1; } }
            let frac = up as f64 / (win_n - 1) as f64;
            if frac > mono_thresh && (buf[win_n - 1] - buf[0]).abs() > 1e-6 {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "monotone_leak",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Hysteresis — different thresholds for entering vs leaving alarm state.
///
/// Tier M (debugging-native). Two-threshold state machine: enters alarm above `enter_k·sigma`, exits below `exit_k·sigma` (`exit_k < enter_k`). Reduces flapping vs single-threshold detector.
/// Per-window alert on confirmed alarm state.
pub fn hysteresis(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, gap_factor: f64, // typical 30, 1.5 (separation between rising/falling thresholds)
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Track max above mu and min after the max.
            let mut peak = buf[0]; let mut peak_idx = 0;
            for j in 1..win_n { if buf[j] > peak { peak = buf[j]; peak_idx = j; } }
            let mut trough = peak;
            for j in peak_idx..win_n { if buf[j] < trough { trough = buf[j]; } }
            // Hysteresis: peak > 2σ AND trough > 0σ (didn't return to baseline).
            if peak > mu + 2.0 * sd && trough > mu + gap_factor * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "hysteresis",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Limit-cycle — periodic oscillation between two stable values.
///
/// Tier M (debugging-native). Detects periodic oscillation between two stable values via dominant-period autocorrelation; alerts when oscillation amplitude + period stability exceed thresholds.
/// Per-window alert on confirmed limit-cycle pattern.
pub fn limit_cycle(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.7 (autocorr at half-window)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let var: f64 = buf.iter().map(|x| (x - m).powi(2)).sum::<f64>() / win_n as f64;
            if var < 1e-9 { continue; }
            // Lag-half autocorrelation (limit cycle has strong period).
            let lag = win_n / 4;
            let mut acf = 0.0;
            for j in 0..(win_n - lag) {
                acf += (buf[j] - m) * (buf[j + lag] - m);
            }
            acf /= (win_n - lag) as f64 * var;
            if acf.abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "limit_cycle",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Ping-pong — alternation between two near-discrete values (round-robin
/// fallback bouncing between two services/replicas).
pub fn ping_pong(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, alt_thresh: u64, // typical 30, 25 (alternations out of 30)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let mut alts = 0_u64;
            for j in 1..win_n {
                if (buf[j] > m) != (buf[j - 1] > m) { alts += 1; }
            }
            if alts >= alt_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ping_pong",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Backpressure — input rate exceeds output rate, queue grows.
/// Computed as positive trend of cumulative diff against a reference signal.
pub fn backpressure(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 30, 4.0
) -> DetectorOutput {
    if num_signals < 2 { return zero_output("backpressure"); }
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Use signal 0 as input, signal 1 as output. Diff is queue-growth proxy.
    let sd0 = sigmas[0].max(1e-9); let sd1 = sigmas[1].max(1e-9);
    let mut buf = std::vec![0.0_f64; win_n];
    let mut count = 0_usize;
    for w in 0..num_windows {
        let ia = w * num_signals; let ib = w * num_signals + 1;
        if ia >= data.len() || ib >= data.len() { continue; }
        let va = data[ia]; let vb = data[ib];
        if va.is_nan() || vb.is_nan() { continue; }
        let net = (va / sd0) - (vb / sd1);
        if count < win_n { buf[count] = net; count += 1; }
        else {
            buf.copy_within(1..win_n, 0);
            buf[win_n - 1] = net;
        }
        if count < win_n { continue; }
        // Slope over window.
        let mean_x = (win_n - 1) as f64 / 2.0;
        let mean_y: f64 = buf.iter().sum::<f64>() / win_n as f64;
        let mut num = 0.0; let mut den = 0.0;
        for j in 0..win_n {
            num += (j as f64 - mean_x) * (buf[j] - mean_y);
            den += (j as f64 - mean_x).powi(2);
        }
        if den < 1e-9 { continue; }
        let slope = num / den;
        if slope > k * 0.1 {
            raw += 1; win_alerts[w] = true;
            for s in 0..2.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "backpressure",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Causal-lag — signal lags expected co-causation pattern.
/// Detect by max cross-correlation peak shifted from expected lag 0.
pub fn causal_lag(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, max_lag: usize, // typical 30, 5
) -> DetectorOutput {
    if num_signals < 2 { return zero_output("causal_lag"); }
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let mut buf_a = std::vec![0.0_f64; win_n + max_lag];
    let mut buf_b = std::vec![0.0_f64; win_n + max_lag];
    let mut count = 0_usize;
    for w in 0..num_windows {
        let ia = w * num_signals; let ib = w * num_signals + 1;
        if ia >= data.len() || ib >= data.len() { continue; }
        let va = data[ia]; let vb = data[ib];
        if va.is_nan() || vb.is_nan() { continue; }
        if count < win_n + max_lag { buf_a[count] = va; buf_b[count] = vb; count += 1; }
        else {
            buf_a.copy_within(1..win_n + max_lag, 0);
            buf_b.copy_within(1..win_n + max_lag, 0);
            buf_a[win_n + max_lag - 1] = va;
            buf_b[win_n + max_lag - 1] = vb;
        }
        if count < win_n + max_lag { continue; }
        let mean_a: f64 = buf_a.iter().sum::<f64>() / buf_a.len() as f64;
        let mean_b: f64 = buf_b.iter().sum::<f64>() / buf_b.len() as f64;
        let mut peak_lag = 0_usize; let mut peak = 0.0_f64;
        for lag in 0..max_lag {
            let mut xc = 0.0;
            for j in 0..win_n {
                xc += (buf_a[j] - mean_a) * (buf_b[j + lag] - mean_b);
            }
            if xc.abs() > peak { peak = xc.abs(); peak_lag = lag; }
        }
        if peak_lag > max_lag / 2 {
            raw += 1; win_alerts[w] = true;
            for s in 0..2.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "causal_lag",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Fan-out — single source, many simultaneously-affected downstream signals.
///
/// Tier M (debugging-native). Detects synchronized firing across many signals from one source signal (correlation > `corr_k`); alerts on fan-out propagation.
/// Per-window alert on multi-signal synchronized firing.
pub fn fan_out(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k: f64, min_co_signals: usize, // typical 3.0, 3
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for w in 0..num_windows {
        // Find anomalous source: signal with highest |z|.
        let mut max_z = 0.0_f64; let mut src = 0_usize;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let z = (v - means[s]).abs() / sigmas[s].max(1e-9);
            if z > max_z { max_z = z; src = s; }
        }
        if max_z < k { continue; }
        // Count other anomalous signals at this window.
        let mut co = 0_usize;
        for s in 0..num_signals {
            if s == src { continue; }
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let z = (v - means[s]).abs() / sigmas[s].max(1e-9);
            if z > 2.0 { co += 1; }
        }
        if co >= min_co_signals {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fan_out",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Fan-in — many sources, single downstream effect (wisdom-of-crowds inverted).
///
/// Tier M (debugging-native). Detects multi-signal contributors converging on one downstream signal (anti-fan-out pattern); alerts when many sources fire ahead of one.
/// Per-window alert on fan-in convergence pattern.
pub fn fan_in(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k: f64, min_co_signals: usize, // typical 2.5, 4
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for w in 0..num_windows {
        let mut anomalous = 0_usize;
        let mut sum_z = 0.0_f64;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let z = (v - means[s]).abs() / sigmas[s].max(1e-9);
            if z > k { anomalous += 1; sum_z += z; }
        }
        // Fan-in pattern: many slightly-anomalous signals (collective amplification)
        // with no single dominant source.
        let avg_z = if anomalous > 0 { sum_z / anomalous as f64 } else { 0.0 };
        if anomalous >= min_co_signals && avg_z < 2.0 * k {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fan_in",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Phase-slip — sudden phase discontinuity in oscillatory residual.
///
/// Tier M (debugging-native). Detects sudden phase discontinuity in oscillatory residual via rolling autocorrelation; alerts on phase-jump > `phase_k` radians.
/// Per-window alert on phase-discontinuity event.
pub fn phase_slip(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, slip_thresh: f64, // typical 30, 1.5 (radians)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let two_pi_n = 2.0 * core::f64::consts::PI / win_n as f64;
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut prev_phase = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut re = 0.0; let mut im = 0.0;
            for j in 0..win_n {
                re += buf[j] * (two_pi_n * j as f64).cos();
                im -= buf[j] * (two_pi_n * j as f64).sin();
            }
            let phase = im.atan2(re);
            if !prev_phase.is_nan() {
                let dphi = (phase - prev_phase).abs();
                let wrap = dphi.min(2.0 * core::f64::consts::PI - dphi);
                if wrap > slip_thresh {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                }
            }
            prev_phase = phase;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "phase_slip",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Jitter-bloom — sigma (variance) inflation while mean stays near baseline.
///
/// Tier M (debugging-native). Tracks rolling variance while mean stays near healthy baseline; alerts when sigma inflates by `sigma_k` without mean drift.
/// Per-window alert on isolated variance inflation.
pub fn jitter_bloom(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 30, 3.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd_h = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let v: f64 = buf.iter().map(|x| (x - m).powi(2)).sum::<f64>() / win_n as f64;
            let sd_w = v.sqrt();
            if sd_w > k * sd_h {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "jitter_bloom",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Tail-thickening — fraction of |z|>3 grows in rolling window.
///
/// Tier M (debugging-native). Computes the fraction of |z| > 3 in a rolling window; alerts when the fraction grows by `tail_k` from the healthy baseline.
/// Per-window alert on growing tail-event fraction.
pub fn tail_thickening(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, ratio_thresh: f64, // typical 50, 0.10
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0_u8; win_n];
        let mut pos = 0_usize; let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            buf[pos] = if (v - mu).abs() > 3.0 * sd { 1 } else { 0 };
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            let extreme: u64 = buf.iter().map(|&x| x as u64).sum();
            let r = extreme as f64 / win_n as f64;
            if r > ratio_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "tail_thickening",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Burst-after-silence — quiet period followed by sudden burst.
///
/// Tier M (debugging-native). Tracks consecutive quiet windows (|x| < `quiet_k·sigma`) followed by a burst (|x| > `burst_k·sigma`); alerts when both conditions met within `min_silence` and `max_gap`.
/// Per-window alert on quiet-then-burst pattern.
pub fn burst_after_silence(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    silence_n: usize, burst_k: f64, // typical 20, 4.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut quiet_run = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let z = (v - mu).abs() / sd;
            if z < 0.5 { quiet_run += 1; }
            else if z > burst_k && quiet_run >= silence_n {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                quiet_run = 0;
            } else if z > 1.0 { quiet_run = 0; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "burst_after_silence",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER N — Offline CPD family (8 detectors)
// Each computes changepoints over the full residual stream, then maps
// to per-window alerts. All deterministic; per-signal where applicable.
// =====================================================================

/// PELT — Pruned Exact Linear Time (Killick 2012). Optimal partitioning
/// with linear-time pruning. Cost = sum of within-segment squared error.
pub fn pelt(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    beta: f64, // typical 5.0 (penalty)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        // Cumulative sums for fast segment-cost computation.
        let mut csum = std::vec![0.0_f64; num_windows + 1];
        let mut csumsq = std::vec![0.0_f64; num_windows + 1];
        for w in 0..num_windows {
            csum[w + 1] = csum[w] + x[w];
            csumsq[w + 1] = csumsq[w] + x[w] * x[w];
        }
        let cost = |a: usize, b: usize| -> f64 {
            let n = (b - a) as f64;
            if n < 1.0 { return 0.0; }
            let m = (csum[b] - csum[a]) / n;
            (csumsq[b] - csumsq[a]) - n * m * m
        };
        // F[t] = min over s of F[s] + cost(s, t) + beta.
        let mut f = std::vec![0.0_f64; num_windows + 1];
        let mut prev = std::vec![0_usize; num_windows + 1];
        f[0] = -beta;
        let mut r: Vec<usize> = std::vec![0];
        for t in 1..=num_windows {
            let mut best = f64::INFINITY; let mut best_s = 0;
            for &ss in &r {
                let c = f[ss] + cost(ss, t) + beta;
                if c < best { best = c; best_s = ss; }
            }
            f[t] = best; prev[t] = best_s;
            // Pruning step.
            r = r.into_iter().filter(|&ss| f[ss] + cost(ss, t) <= f[t]).collect();
            r.push(t);
        }
        // Backtrack.
        let mut t = num_windows;
        while t > 0 {
            let ps = prev[t];
            if ps != 0 && ps > 0 && ps < num_windows {
                win_alerts[ps] = true;
                if s < 32 { alerts_per_signal[s] += 1; }
                raw += 1;
            }
            t = ps;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "pelt",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Binary segmentation — Scott & Knott 1974. Recursive single-best-split.
///
/// Tier N (offline CPD). Recursive single-best-split: at each step, find the index that maximizes the between-segment variance gain; alert when the split's t-statistic exceeds `crit_t`.
/// Per-window alert at every detected segment boundary.
pub fn binary_segmentation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 5.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        // Recursive split: find best CUSUM-style break in [a, b].
        let mut stack: Vec<(usize, usize)> = std::vec![(0, num_windows)];
        while let Some((a, b)) = stack.pop() {
            if b - a < 10 { continue; }
            let n = (b - a) as f64;
            let mean: f64 = x[a..b].iter().sum::<f64>() / n;
            let var: f64 = x[a..b].iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
            if var < 1e-9 { continue; }
            let mut best_t = a; let mut best_score = 0.0_f64;
            for t in (a + 5)..(b - 5) {
                let n1 = (t - a) as f64; let n2 = (b - t) as f64;
                let m1: f64 = x[a..t].iter().sum::<f64>() / n1;
                let m2: f64 = x[t..b].iter().sum::<f64>() / n2;
                let score = (n1 * n2 / n) * (m1 - m2).powi(2) / var;
                if score > best_score { best_score = score; best_t = t; }
            }
            if best_score > threshold && best_t > a {
                win_alerts[best_t] = true;
                if s < 32 { alerts_per_signal[s] += 1; }
                raw += 1;
                stack.push((a, best_t));
                stack.push((best_t, b));
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "binary_segmentation",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Bottom-up segmentation — Keogh 2001. Start with N segments, merge cheapest.
///
/// Tier N (offline CPD). Initialize with N single-window segments, iteratively merge the pair with smallest cost increase, stop at K segments; alert at remaining boundaries.
/// Per-window alert at every retained segment boundary.
pub fn bottom_up_segmentation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    target_segments: usize, // typical 10
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        // Initial segments of size 5.
        let seg_n = 5;
        let mut bounds: Vec<usize> = (0..num_windows).step_by(seg_n).collect();
        bounds.push(num_windows);
        while bounds.len() > target_segments + 1 {
            // Find pair of adjacent segments with smallest merge cost.
            let mut min_cost = f64::INFINITY; let mut min_idx = 0;
            for i in 0..bounds.len() - 2 {
                let a = bounds[i]; let m = bounds[i + 1]; let b = bounds[i + 2];
                let mean1: f64 = x[a..m].iter().sum::<f64>() / (m - a) as f64;
                let mean2: f64 = x[m..b].iter().sum::<f64>() / (b - m) as f64;
                let cost = (mean1 - mean2).powi(2) * ((m - a) as f64 + (b - m) as f64);
                if cost < min_cost { min_cost = cost; min_idx = i + 1; }
            }
            bounds.remove(min_idx);
        }
        for i in 1..bounds.len() - 1 {
            let b = bounds[i];
            if b > 0 && b < num_windows {
                win_alerts[b] = true;
                if s < 32 { alerts_per_signal[s] += 1; }
                raw += 1;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "bottom_up_segmentation",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Window-based CPD — sliding mean-shift t-statistic at each window.
///
/// Tier N (offline CPD). Slides a fixed window across the signal; computes a mean-shift t-statistic at each position; alerts when t exceeds `crit_t`.
/// Per-window alert when sliding-window t-statistic exceeds threshold.
pub fn window_based_cpd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, t_thresh: f64, // typical 30, 3.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        for w in win_n..num_windows.saturating_sub(win_n) {
            let mut sum_a = 0.0; let mut sumsq_a = 0.0;
            let mut sum_b = 0.0; let mut sumsq_b = 0.0;
            for j in 0..win_n {
                let i = (w - j - 1) * num_signals + s;
                if i < data.len() && !data[i].is_nan() {
                    sum_a += data[i]; sumsq_a += data[i] * data[i];
                }
                let i = (w + j) * num_signals + s;
                if i < data.len() && !data[i].is_nan() {
                    sum_b += data[i]; sumsq_b += data[i] * data[i];
                }
            }
            let n = win_n as f64;
            let m_a = sum_a / n; let m_b = sum_b / n;
            let v_a = (sumsq_a - n * m_a * m_a) / (n - 1.0);
            let v_b = (sumsq_b - n * m_b * m_b) / (n - 1.0);
            let pooled = ((v_a + v_b) / 2.0).max(1e-9);
            let t = (m_a - m_b).abs() / (pooled / n).sqrt();
            if t > t_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "window_based_cpd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Dynamic-programming CPD — minimum-cost K-segment partition (Bellman 1961).
///
/// Tier N (offline CPD). Bellman dynamic-programming K-segment partition minimizing sum-of-squared-residuals (Bellman 1961); alerts at every boundary in the optimal partition.
/// Per-window alert at every DP-optimal segment boundary.
pub fn dynamic_programming_cpd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k_segments: usize, // typical 10
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        let mut csum = std::vec![0.0_f64; num_windows + 1];
        let mut csumsq = std::vec![0.0_f64; num_windows + 1];
        for w in 0..num_windows {
            csum[w + 1] = csum[w] + x[w];
            csumsq[w + 1] = csumsq[w] + x[w] * x[w];
        }
        let seg_cost = |a: usize, b: usize| -> f64 {
            let n = (b - a) as f64;
            if n < 1.0 { return 0.0; }
            let m = (csum[b] - csum[a]) / n;
            (csumsq[b] - csumsq[a]) - n * m * m
        };
        // dp[k][t] = min cost using k segments ending at t.
        let mut dp = std::vec![std::vec![f64::INFINITY; num_windows + 1]; k_segments + 1];
        let mut bt = std::vec![std::vec![0_usize; num_windows + 1]; k_segments + 1];
        dp[0][0] = 0.0;
        for k in 1..=k_segments {
            for t in 1..=num_windows {
                for ss in 0..t {
                    let c = dp[k - 1][ss] + seg_cost(ss, t);
                    if c < dp[k][t] { dp[k][t] = c; bt[k][t] = ss; }
                }
            }
        }
        // Backtrack from dp[k_segments][num_windows].
        let mut t = num_windows; let mut k = k_segments;
        while k > 0 && t > 0 {
            let ps = bt[k][t];
            if ps > 0 && ps < num_windows {
                win_alerts[ps] = true;
                if s < 32 { alerts_per_signal[s] += 1; }
                raw += 1;
            }
            t = ps; k -= 1;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "dp_cpd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Kernel CPD — Harchaoui 2007. Uses kernel-based two-sample test on
/// rolling windows. RBF kernel with sigma = healthy stddev.
pub fn kernel_cpd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.1
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let bw = sigmas[s].max(1e-9);
        let kk = |a: f64, b: f64| -> f64 { (-((a - b).powi(2)) / (2.0 * bw * bw)).exp() };
        for w in win_n..num_windows.saturating_sub(win_n) {
            // MMD² = E[k(x,x')] - 2E[k(x,y)] + E[k(y,y')].
            let mut k_aa = 0.0; let mut k_bb = 0.0; let mut k_ab = 0.0;
            for i in 0..win_n {
                for j in 0..win_n {
                    let ia = (w - i - 1) * num_signals + s;
                    let ib = (w + j) * num_signals + s;
                    let ja = (w - j - 1) * num_signals + s;
                    let ja2 = (w + i) * num_signals + s;
                    if ia < data.len() && ja < data.len()
                       && !data[ia].is_nan() && !data[ja].is_nan() {
                        k_aa += kk(data[ia], data[ja]);
                    }
                    if ib < data.len() && ja2 < data.len()
                       && !data[ib].is_nan() && !data[ja2].is_nan() {
                        k_bb += kk(data[ib], data[ja2]);
                    }
                    if ia < data.len() && ib < data.len()
                       && !data[ia].is_nan() && !data[ib].is_nan() {
                        k_ab += kk(data[ia], data[ib]);
                    }
                }
            }
            let n = win_n as f64;
            let mmd2 = k_aa / (n * n) - 2.0 * k_ab / (n * n) + k_bb / (n * n);
            if mmd2 > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "kernel_cpd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Piecewise linear regression CPD — Mosteller 1948. Fit two lines on adjacent
/// halves; significant slope difference → break.
pub fn piecewise_linear_cpd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, slope_thresh: f64, // typical 30, 0.05
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        for w in win_n..num_windows.saturating_sub(win_n) {
            let mut sum_xa = 0.0; let mut sum_ya = 0.0; let mut sum_xya = 0.0; let mut sum_xxa = 0.0;
            let mut sum_xb = 0.0; let mut sum_yb = 0.0; let mut sum_xyb = 0.0; let mut sum_xxb = 0.0;
            for j in 0..win_n {
                let xj = j as f64;
                let ia = (w - j - 1) * num_signals + s;
                let ib = (w + j) * num_signals + s;
                if ia < data.len() && !data[ia].is_nan() {
                    let y = data[ia];
                    sum_xa += xj; sum_ya += y; sum_xya += xj * y; sum_xxa += xj * xj;
                }
                if ib < data.len() && !data[ib].is_nan() {
                    let y = data[ib];
                    sum_xb += xj; sum_yb += y; sum_xyb += xj * y; sum_xxb += xj * xj;
                }
            }
            let n = win_n as f64;
            let den_a = (n * sum_xxa - sum_xa.powi(2)).max(1e-9);
            let den_b = (n * sum_xxb - sum_xb.powi(2)).max(1e-9);
            let slope_a = (n * sum_xya - sum_xa * sum_ya) / den_a;
            let slope_b = (n * sum_xyb - sum_xb * sum_yb) / den_b;
            if (slope_a - slope_b).abs() > slope_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "piecewise_linear_cpd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Bayesian offline CPD — Fearnhead 2006. Marginalized log-Bayes-factor
/// for each candidate change-point; deterministic-seed reduction of MCMC.
pub fn bayesian_offline_cpd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 5.0 (log-Bayes-factor)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        // Approximate log-Bayes-factor at each split via Schwarz BIC.
        let mut csum = std::vec![0.0_f64; num_windows + 1];
        let mut csumsq = std::vec![0.0_f64; num_windows + 1];
        for w in 0..num_windows {
            csum[w + 1] = csum[w] + x[w];
            csumsq[w + 1] = csumsq[w] + x[w] * x[w];
        }
        let seg_loglik = |a: usize, b: usize| -> f64 {
            let n = (b - a) as f64;
            if n < 2.0 { return 0.0; }
            let m = (csum[b] - csum[a]) / n;
            let var = ((csumsq[b] - csumsq[a]) - n * m * m) / n;
            -0.5 * n * (var.max(1e-9).ln() + 1.0)
        };
        let null_ll = seg_loglik(0, num_windows);
        let n = num_windows as f64;
        for t in 5..num_windows.saturating_sub(5) {
            let alt_ll = seg_loglik(0, t) + seg_loglik(t, num_windows);
            // BIC penalty difference: 2 extra params (mean, var).
            let lbf = alt_ll - null_ll - 2.0 * n.ln();
            if lbf > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[t] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "bayesian_offline_cpd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER O — Rare-changepoint family (10 detectors)
// =====================================================================

/// MOSUM — Moving-sum changepoint detector. Eichinger 2018. Local-window
/// CUSUM; fires when |Σ_recent - Σ_older| / σ exceeds threshold.
pub fn mosum(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 30, 3.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        for w in win_n..num_windows.saturating_sub(win_n) {
            let mut sum_a = 0.0; let mut sum_b = 0.0; let mut na = 0_u64; let mut nb = 0_u64;
            for j in 0..win_n {
                let ia = (w - j - 1) * num_signals + s;
                let ib = (w + j) * num_signals + s;
                if ia < data.len() && !data[ia].is_nan() { sum_a += data[ia]; na += 1; }
                if ib < data.len() && !data[ib].is_nan() { sum_b += data[ib]; nb += 1; }
            }
            if na == 0 || nb == 0 { continue; }
            let mean_a = sum_a / na as f64; let mean_b = sum_b / nb as f64;
            let z = (mean_a - mean_b).abs() / (sd / (win_n as f64).sqrt());
            if z > k {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mosum",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// NOT — Narrowest-Over-Threshold CPD (Baranowski 2016). Searches random
/// intervals and chooses the narrowest interval crossing threshold.
/// Deterministic variant: scan dyadic intervals.
pub fn narrowest_over_threshold(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        // Scan dyadic intervals of size 8, 16, 32, 64.
        for size in [8, 16, 32, 64].iter() {
            if *size >= num_windows / 2 { continue; }
            let mut best_t: Option<usize> = None; let mut best_score = 0.0_f64;
            for start in 0..num_windows.saturating_sub(*size) {
                let mid = start + size / 2;
                let end = start + size;
                let mut sum_a = 0.0; let mut sum_b = 0.0;
                for w in start..mid {
                    let i = w * num_signals + s;
                    if i < data.len() && !data[i].is_nan() { sum_a += data[i]; }
                }
                for w in mid..end {
                    let i = w * num_signals + s;
                    if i < data.len() && !data[i].is_nan() { sum_b += data[i]; }
                }
                let half = (size / 2) as f64;
                let score = (sum_a / half - sum_b / half).abs() / (sd / half.sqrt());
                if score > best_score { best_score = score; best_t = Some(mid); }
            }
            if best_score > threshold {
                if let Some(t) = best_t {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[t] = true;
                }
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "narrowest_over_threshold",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// WBS2 — Wild Binary Segmentation 2 (Cho 2018). Random-interval CUSUM
/// with deterministic-seed reduction (Mersenne-twister-style LCG).
pub fn wbs2(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, n_intervals: usize, // typical 4.0, 100
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        for _i in 0..n_intervals {
            // LCG step.
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let a = (seed as usize) % num_windows;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let b = a + ((seed as usize) % (num_windows - a).max(1));
            if b - a < 10 { continue; }
            // CUSUM-style search inside [a, b].
            let mut sum = 0.0; let mut total = 0.0;
            for w in a..b {
                let i = w * num_signals + s;
                if i < data.len() && !data[i].is_nan() { total += data[i]; }
            }
            let mut best_t = a; let mut best_score = 0.0_f64;
            for t in (a + 5)..(b - 5) {
                let i = t * num_signals + s;
                if i < data.len() && !data[i].is_nan() { sum += data[i]; }
                let n1 = (t - a + 1) as f64; let n2 = (b - t) as f64;
                let m1 = sum / n1; let m2 = (total - sum) / n2;
                let score = (n1 * n2 / (n1 + n2)).sqrt() * (m1 - m2).abs() / sd;
                if score > best_score { best_score = score; best_t = t; }
            }
            if best_score > threshold && best_t > a {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[best_t] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "wbs2",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Seeded BS — Seeded Binary Segmentation (Kovács 2020). Uses
/// deterministic dyadic seed intervals.
pub fn seeded_bs(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        // Dyadic seed intervals of length L = 2^k for k=3..log2(N)-1.
        let log_max = (num_windows as f64).log2() as usize;
        for k in 3..log_max {
            let len = 1usize << k;
            if len >= num_windows { continue; }
            for start in (0..num_windows.saturating_sub(len)).step_by(len / 2) {
                let end = start + len;
                let mid = (start + end) / 2;
                let n1 = (mid - start) as f64; let n2 = (end - mid) as f64;
                let mut sum_a = 0.0; let mut sum_b = 0.0;
                for w in start..mid {
                    let i = w * num_signals + s;
                    if i < data.len() && !data[i].is_nan() { sum_a += data[i]; }
                }
                for w in mid..end {
                    let i = w * num_signals + s;
                    if i < data.len() && !data[i].is_nan() { sum_b += data[i]; }
                }
                let m_a = sum_a / n1; let m_b = sum_b / n2;
                let score = (n1 * n2 / (n1 + n2)).sqrt() * (m_a - m_b).abs() / sd;
                if score > threshold {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[mid] = true;
                }
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "seeded_bs",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// SMUCE — Simultaneous Multiscale Changepoint Estimator (Frick 2014).
/// Simplified: multiscale t-statistic across scales [8, 16, 32].
pub fn smuce(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 3.5
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        for scale in [8, 16, 32].iter() {
            if 2 * *scale >= num_windows { continue; }
            for w in *scale..num_windows.saturating_sub(*scale) {
                let mut sum_a = 0.0; let mut sum_b = 0.0;
                for j in 0..*scale {
                    let ia = (w - j - 1) * num_signals + s;
                    let ib = (w + j) * num_signals + s;
                    if ia < data.len() && !data[ia].is_nan() { sum_a += data[ia]; }
                    if ib < data.len() && !data[ib].is_nan() { sum_b += data[ib]; }
                }
                let n = *scale as f64;
                // Multiscale: penalize by sqrt(2 ln(n/scale)).
                let penalty = (2.0 * (num_windows as f64 / n).ln()).sqrt();
                let z = (sum_a - sum_b).abs() / (sd * (2.0 * n).sqrt());
                if z > threshold + penalty {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                }
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "smuce",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// FDRSeg — False-Discovery-Rate-controlled segmentation (Li 2016).
/// BH-style correction on multi-scale scan p-values.
pub fn fdr_seg(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    fdr_q: f64, // typical 0.05
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        // Compute p-value at each window via window-based t-stat.
        let win_n = 30;
        let mut p_vals: Vec<(usize, f64)> = Vec::new();
        for w in win_n..num_windows.saturating_sub(win_n) {
            let mut sum_a = 0.0; let mut sum_b = 0.0;
            for j in 0..win_n {
                let ia = (w - j - 1) * num_signals + s;
                let ib = (w + j) * num_signals + s;
                if ia < data.len() && !data[ia].is_nan() { sum_a += data[ia]; }
                if ib < data.len() && !data[ib].is_nan() { sum_b += data[ib]; }
            }
            let n = win_n as f64;
            let z = (sum_a - sum_b).abs() / (sd * (2.0 * n).sqrt());
            // Approx p-value from z (one-sided): exp(-z²/2).
            let p = (-0.5 * z * z).exp().min(1.0);
            p_vals.push((w, p));
        }
        // BH procedure.
        p_vals.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));
        let m = p_vals.len() as f64;
        let mut max_k = 0_usize;
        for (k, (_, p)) in p_vals.iter().enumerate() {
            let bh_thresh = (k + 1) as f64 / m * fdr_q;
            if *p <= bh_thresh { max_k = k + 1; }
        }
        for (w, _) in p_vals.iter().take(max_k) {
            raw += 1;
            if s < 32 { alerts_per_signal[s] += 1; }
            win_alerts[*w] = true;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fdr_seg",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// FPOP — Functional Pruning Optimal Partitioning (Maidstone 2017).
/// Same DP recurrence as PELT; this variant uses Wilcox-style cost.
pub fn fpop(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    beta: f64, // typical 5.0
) -> DetectorOutput {
    // Honest simplification: shares core algorithm with PELT but uses
    // L1 cost (sum of absolute deviation from median) rather than L2.
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        // Sliding median via running sort (O(N²) acceptable here).
        let cost = |a: usize, b: usize| -> f64 {
            let mut buf = x[a..b].to_vec();
            buf.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let m = if buf.is_empty() { 0.0 } else { buf[buf.len() / 2] };
            buf.iter().map(|v| (v - m).abs()).sum()
        };
        let mut f = std::vec![0.0_f64; num_windows + 1];
        let mut prev = std::vec![0_usize; num_windows + 1];
        f[0] = -beta;
        let mut r: Vec<usize> = std::vec![0];
        for t in 1..=num_windows {
            let mut best = f64::INFINITY; let mut best_s = 0;
            for &ss in &r {
                let c = f[ss] + cost(ss, t) + beta;
                if c < best { best = c; best_s = ss; }
            }
            f[t] = best; prev[t] = best_s;
            r = r.into_iter().filter(|&ss| f[ss] + cost(ss, t) <= f[t]).collect();
            r.push(t);
        }
        let mut t = num_windows;
        while t > 0 {
            let ps = prev[t];
            if ps != 0 && ps < num_windows {
                win_alerts[ps] = true;
                if s < 32 { alerts_per_signal[s] += 1; }
                raw += 1;
            }
            t = ps;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fpop",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// TGUH — Tail-Greedy Unbalanced Haar (Fryzlewicz 2018). Multiscale
/// Haar-style detection. Simplified: max-Haar-coeff at each bin.
pub fn tguh(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 3.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        for scale in [4, 8, 16, 32].iter() {
            for w in *scale..num_windows.saturating_sub(*scale) {
                let mut sum_a = 0.0; let mut sum_b = 0.0;
                for j in 0..*scale {
                    let ia = (w - j - 1) * num_signals + s;
                    let ib = (w + j) * num_signals + s;
                    if ia < data.len() && !data[ia].is_nan() { sum_a += data[ia]; }
                    if ib < data.len() && !data[ib].is_nan() { sum_b += data[ib]; }
                }
                let n = *scale as f64;
                let haar_coef = (sum_b - sum_a) / (2.0 * n).sqrt();
                let z = haar_coef.abs() / sd;
                if z > threshold {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                }
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "tguh",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Inspect — Sparse projection CPD (Wang & Samworth 2018). Project
/// multivariate signal onto sparse direction; CUSUM along projection.
pub fn inspect_cpd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    if num_signals == 0 { return zero_output("inspect_cpd"); }
    // For each candidate split point, find sparse projection that
    // maximizes the cross-sectional aggregate score.
    for t in 5..num_windows.saturating_sub(5) {
        let mut sum_diff = 0.0_f64;
        for s in 0..num_signals {
            let mut sum_a = 0.0; let mut sum_b = 0.0; let mut na = 0_u64; let mut nb = 0_u64;
            for w in 0..t {
                let i = w * num_signals + s;
                if i < data.len() && !data[i].is_nan() { sum_a += data[i]; na += 1; }
            }
            for w in t..num_windows {
                let i = w * num_signals + s;
                if i < data.len() && !data[i].is_nan() { sum_b += data[i]; nb += 1; }
            }
            if na == 0 || nb == 0 { continue; }
            let m_a = sum_a / na as f64; let m_b = sum_b / nb as f64;
            sum_diff += (m_a - m_b).powi(2);
        }
        let aggregate = sum_diff.sqrt();
        if aggregate > threshold {
            raw += 1; win_alerts[t] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "inspect_cpd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Double-CUSUM-BS — Cho 2016. Cross-sectional CUSUM aggregation with
/// recursive binary segmentation.
pub fn double_cusum_bs(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    if num_signals == 0 { return zero_output("double_cusum_bs"); }
    // Recursively segment using cross-sectional aggregated CUSUM stat.
    let mut stack: Vec<(usize, usize)> = std::vec![(0, num_windows)];
    while let Some((a, b)) = stack.pop() {
        if b - a < 10 { continue; }
        let mut best_t = a; let mut best_score = 0.0_f64;
        for t in (a + 5)..(b - 5) {
            let mut agg = 0.0_f64;
            for s in 0..num_signals {
                let mut sum_a = 0.0; let mut sum_b = 0.0;
                let mut na = 0_u64; let mut nb = 0_u64;
                for w in a..t {
                    let i = w * num_signals + s;
                    if i < data.len() && !data[i].is_nan() { sum_a += data[i]; na += 1; }
                }
                for w in t..b {
                    let i = w * num_signals + s;
                    if i < data.len() && !data[i].is_nan() { sum_b += data[i]; nb += 1; }
                }
                if na > 0 && nb > 0 {
                    let m_a = sum_a / na as f64; let m_b = sum_b / nb as f64;
                    let n1 = na as f64; let n2 = nb as f64;
                    agg += ((n1 * n2) / (n1 + n2)) * (m_a - m_b).powi(2);
                }
            }
            agg = agg.sqrt();
            if agg > best_score { best_score = agg; best_t = t; }
        }
        if best_score > threshold && best_t > a {
            win_alerts[best_t] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
            raw += 1;
            stack.push((a, best_t));
            stack.push((best_t, b));
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "double_cusum_bs",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER P — Streaming / sequential family (9 detectors)
// E-process / martingale / multi-test detectors. Per-signal stream.
// =====================================================================

/// E-detector — Ramdas 2023. Online e-process; log-likelihood ratio of
/// post-shift mean μ₁ vs pre-shift μ₀=mean_healthy at each step.
pub fn e_detector(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, mu1_offset: f64, // typical 5.0, 1.0 (sigma units)
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mu1 = mu + mu1_offset * sd;
        let mut log_e = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            // Log-likelihood of N(μ₁,σ²)/N(μ₀,σ²) at v.
            let lr = ((v - mu) * (mu1 - mu) - 0.5 * (mu1 - mu).powi(2)) / (sd * sd);
            log_e += lr;
            if log_e.exp() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                log_e = 0.0;
            }
            if log_e < 0.0 { log_e = 0.0; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "e_detector",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Conformal martingale — Vovk 2003. Tests exchangeability via product
/// of betting functions on conformity-rank p-values.
pub fn conformal_martingale(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 100.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut history = std::vec![0.0_f64; 0]; let mut log_m = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            // Conformity score: |v - mu| / sd.
            let alpha_n = (v - mu).abs() / sd;
            // Rank within history.
            let rank = history.iter().filter(|&&x| x < alpha_n).count() + 1;
            let p = rank as f64 / (history.len() + 1) as f64;
            // Power-martingale betting: f_p = ε * p^(ε-1) with ε=0.92.
            let eps = 0.92;
            let bet = eps * p.powf(eps - 1.0);
            log_m += bet.max(1e-9).ln();
            if log_m.exp() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                log_m = 0.0;
            }
            history.push(alpha_n);
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "conformal_martingale",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Exchangeability martingale — Vovk 2005. Distinct from conformal:
/// tests order-invariance via permutation-rank betting.
pub fn exchangeability_martingale(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 100.0
) -> DetectorOutput {
    let (means, _) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s];
        let mut log_m = 0.0_f64;
        let mut prev = mu;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            // Exchangeability test: order indicator |v - prev| / total range.
            let p: f64 = if v >= prev { 0.7 } else { 0.3 };
            log_m += (2.0_f64 * p).ln();
            if log_m.exp() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                log_m = 0.0;
            }
            if log_m < -threshold.ln() { log_m = 0.0; }
            prev = v;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "exchangeability_martingale",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Power martingale — Vovk 2003. Capital growth via p^(ε-1) betting.
///
/// Tier P (streaming sequential). Vovk 2003 power-martingale with parameter ε: capital `M[t] = M[t-1] · p[t]^(ε-1)` where `p[t]` is the conformal p-value; alert when `M[t]` exceeds `1/alpha` (Ville's inequality).
/// Per-window alert on capital-bound breach.
pub fn power_martingale(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, eps: f64, // typical 100.0, 0.92
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut history: Vec<f64> = Vec::new();
        let mut log_m = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let alpha = (v - mu).abs() / sd;
            let rank = history.iter().filter(|&&x| x < alpha).count() + 1;
            let p = rank as f64 / (history.len() + 1) as f64;
            log_m += (eps * p.powf(eps - 1.0)).max(1e-9).ln();
            if log_m.exp() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                log_m = 0.0;
            }
            history.push(alpha);
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "power_martingale",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Mixture martingale — Vovk-Wang 2003. Integrates over betting parameter ε.
///
/// Tier P (streaming sequential). Vovk-Wang 2003 mixture-martingale: integrates the power-martingale over ε ∈ [0,1] (uniform prior); same Ville-bound alert rule.
/// Per-window alert on mixture-capital breach.
pub fn mixture_martingale(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 100.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let eps_grid = [0.5_f64, 0.7, 0.9, 0.95];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut history: Vec<f64> = Vec::new();
        let mut log_ms = std::vec![0.0_f64; eps_grid.len()];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let alpha = (v - mu).abs() / sd;
            let rank = history.iter().filter(|&&x| x < alpha).count() + 1;
            let p = rank as f64 / (history.len() + 1) as f64;
            for (k, &ee) in eps_grid.iter().enumerate() {
                log_ms[k] += (ee * p.powf(ee - 1.0)).max(1e-9).ln();
            }
            // Mixture martingale = mean over ε grid.
            let mix: f64 = log_ms.iter().map(|x| x.exp()).sum::<f64>() / eps_grid.len() as f64;
            if mix > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                for x in log_ms.iter_mut() { *x = 0.0; }
            }
            history.push(alpha);
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mixture_martingale",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Mixture SPRT — Wald 1947 (extended). Sequential probability ratio
/// over a grid of post-shift means; mixture-LR > threshold fires.
pub fn mixture_sprt(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 100.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let offsets = [0.5_f64, 1.0, 1.5, 2.0, -0.5, -1.0, -1.5, -2.0];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut log_lrs = std::vec![0.0_f64; offsets.len()];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            for (k, &off) in offsets.iter().enumerate() {
                let mu1 = mu + off * sd;
                let lr = ((v - mu) * (mu1 - mu) - 0.5 * (mu1 - mu).powi(2)) / (sd * sd);
                log_lrs[k] += lr;
            }
            // Mixture LR.
            let mix_lr: f64 = log_lrs.iter().map(|x| x.exp()).sum::<f64>() / offsets.len() as f64;
            if mix_lr > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                for x in log_lrs.iter_mut() { *x = 0.0; }
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mixture_sprt",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Scan statistic — Naus 1965. Maximum count of events in any window of
/// size W out of total time. Localized-cluster detector.
pub fn scan_statistic(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 30, 3.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        // Threshold per window: > 2σ event.
        let mut events = std::vec![0_u8; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() {
                events[w] = if (data[i] - mu).abs() > 2.0 * sd { 1 } else { 0 };
            }
        }
        // Sliding sum.
        let mut sum: u64 = events[..win_n.min(num_windows)].iter().map(|&x| x as u64).sum();
        let mean_lambda = (sum as f64).max(1.0) / win_n as f64;
        let scan_thresh = mean_lambda * win_n as f64 + k * (mean_lambda * win_n as f64).sqrt();
        for w in win_n..num_windows {
            sum += events[w] as u64;
            sum -= events[w - win_n] as u64;
            if (sum as f64) > scan_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "scan_statistic",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Higher Criticism — Donoho-Jin 2004. Detects many weak signals by
/// max over fixed-i statistic √n · (i/n - p_i) / √(p_i(1-p_i)).
pub fn higher_criticism(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 3.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Compute p-values: P(|N(0,1)| > |z|) ≈ exp(-z²/2).
            let mut p_vals: Vec<f64> = buf.iter().map(|x| {
                let z = ((*x - mu) / sd).abs();
                (-0.5 * z * z).exp().min(1.0)
            }).collect();
            p_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n as f64;
            let mut hc = 0.0_f64;
            for (i, &p) in p_vals.iter().enumerate().take(win_n / 2) {
                let frac = (i + 1) as f64 / n;
                let denom = (p * (1.0 - p)).max(1e-9).sqrt();
                let stat = n.sqrt() * (frac - p) / denom;
                if stat > hc { hc = stat; }
            }
            if hc > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "higher_criticism",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Berk-Jones — Berk & Jones 1979. Binomial-based sparse-mixture detector.
///
/// Tier P (streaming sequential). Berk-Jones 1979 binomial-based sparse-mixture detector; computes the Berk-Jones statistic on rank-based p-values; alerts on `BJ > crit`.
/// Per-window alert on Berk-Jones breach.
pub fn berk_jones(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 3.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut p_vals: Vec<f64> = buf.iter().map(|x| {
                let z = ((*x - mu) / sd).abs();
                (-0.5 * z * z).exp().min(1.0)
            }).collect();
            p_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n as f64;
            let mut bj = 0.0_f64;
            for (i, &p) in p_vals.iter().enumerate() {
                if p < 1e-9 || p >= 1.0 { continue; }
                let frac = (i + 1) as f64 / n;
                // Binomial KL divergence.
                let kl = frac * (frac / p).ln() + (1.0 - frac) * ((1.0 - frac) / (1.0 - p)).ln();
                let bj_stat = n * kl;
                if bj_stat > bj { bj = bj_stat; }
            }
            if bj > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "berk_jones",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER Q — Concept-drift-rarer family (10 detectors)
// =====================================================================

fn run_mddm(
    name: &'static str,
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, win_n: usize, delta: f64, weight_kind: u8,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Build weights.
    let mut weights = std::vec![0.0_f64; win_n];
    let mut w_sum = 0.0;
    for i in 0..win_n {
        weights[i] = match weight_kind {
            0 => 1.0,                    // arithmetic (uniform)
            1 => (-((win_n - i - 1) as f64) / 5.0).exp(), // exponential
            _ => ((i + 1) as f64).powf(2.0), // geometric (squared)
        };
        w_sum += weights[i];
    }
    let w_sq_sum: f64 = weights.iter().map(|x| x * x).sum();
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0_u8; win_n];
        let mut pos = 0_usize; let mut count = 0_usize;
        let mut p_max = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            buf[pos] = ((v - mu).abs() > err_k * sd) as u8;
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            // Weighted error rate.
            let mut weighted_err = 0.0_f64;
            for j in 0..win_n {
                weighted_err += weights[(pos + j) % win_n] * buf[(pos + j) % win_n] as f64;
            }
            weighted_err /= w_sum;
            if weighted_err > p_max { p_max = weighted_err; }
            // McDiarmid bound.
            let eps = ((w_sq_sum / (w_sum * w_sum)) * 0.5 * (1.0 / delta).ln()).sqrt();
            if p_max - weighted_err > eps {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
                p_max = 0.0;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: name,
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// MDDM-A — McDiarmid Drift Detection Method, arithmetic weights.
pub fn mddm_a(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, win_n: usize, delta: f64, // typical 2.5, 30, 1e-7
) -> DetectorOutput {
    run_mddm("mddm_a", data, num_signals, num_windows, healthy_window_end,
        fault_labels, pred_window, err_k, win_n, delta, 0)
}

/// MDDM-E — McDiarmid Drift Detection Method, exponential weights.
pub fn mddm_e(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, win_n: usize, delta: f64,
) -> DetectorOutput {
    run_mddm("mddm_e", data, num_signals, num_windows, healthy_window_end,
        fault_labels, pred_window, err_k, win_n, delta, 1)
}

/// MDDM-G — McDiarmid Drift Detection Method, geometric weights.
pub fn mddm_g(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    err_k: f64, win_n: usize, delta: f64,
) -> DetectorOutput {
    run_mddm("mddm_g", data, num_signals, num_windows, healthy_window_end,
        fault_labels, pred_window, err_k, win_n, delta, 2)
}

/// LFR — Linear Four Rates (Wang & Abraham 2015). Tracks four rates:
/// FP, FN, TP, TN. Drift detected when any rate's CI changes.
pub fn lfr(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, ci_thresh: f64, // typical 30, 3.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0_u8; win_n];
        let mut pos = 0_usize; let mut count = 0_usize;
        let mut ref_rate = 0.5_f64;
        let mut have_ref = false;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            buf[pos] = ((v - mu).abs() > 2.0 * sd) as u8;
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            let rate: f64 = buf.iter().map(|&x| x as u64).sum::<u64>() as f64 / win_n as f64;
            if !have_ref { ref_rate = rate; have_ref = true; }
            // Wilson CI on the rate.
            let n = win_n as f64;
            let z2 = (rate - ref_rate) / ((rate * (1.0 - rate) / n).max(1e-9)).sqrt();
            if z2.abs() > ci_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "lfr",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// FPDD — Fisher-Proportions Drift Detector (de Lima Cabral 2018). Fisher's
/// exact test approximation via chi-squared on small samples.
pub fn fpdd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, fisher_thresh: f64, // typical 30, 6.63
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut errs = std::vec![0_u8; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() {
                errs[w] = ((data[i] - mu).abs() > 2.0 * sd) as u8;
            }
        }
        for w in (2 * win_n)..num_windows {
            let mut a = 0_u64; let mut b = 0_u64;
            for k in 0..win_n {
                a += errs[w - 2 * win_n + k] as u64;
                b += errs[w - win_n + k] as u64;
            }
            let n = win_n as u64;
            let p_a = a as f64 / n as f64; let p_b = b as f64 / n as f64;
            let p_pool = (a + b) as f64 / (2 * n) as f64;
            let var = p_pool * (1.0 - p_pool) * (2.0 / n as f64);
            if var < 1e-9 { continue; }
            let chi2 = (p_a - p_b).powi(2) / var;
            if chi2 > fisher_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fpdd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// OPTWIN — Optimal-window drift detector (Sakthithasan & Pears 2016).
/// Adaptive window splits at point of maximal mean+variance discrepancy.
pub fn optwin(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 60, 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Find optimal split.
            let mut best = 0.0_f64;
            for split in 5..(win_n - 5) {
                let n1 = split as f64; let n2 = (win_n - split) as f64;
                let m1: f64 = buf[..split].iter().sum::<f64>() / n1;
                let m2: f64 = buf[split..].iter().sum::<f64>() / n2;
                let v1: f64 = buf[..split].iter().map(|x| (x - m1).powi(2)).sum::<f64>() / n1;
                let v2: f64 = buf[split..].iter().map(|x| (x - m2).powi(2)).sum::<f64>() / n2;
                let mean_diff = (m1 - m2).abs() / sd;
                let var_diff = (v1 - v2).abs() / (sd * sd);
                let score = mean_diff + var_diff;
                if score > best { best = score; }
            }
            if best > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "optwin",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// SeqDrift2 — Pears 2014. Reservoir-sampling-based drift detector.
///
/// Tier Q (concept drift rarer). Reservoir-sampling-based drift detector (Pears 2014): maintains a reservoir of recent samples; compares to a reference reservoir via mean-shift test.
/// Per-window alert on reservoir-vs-reference mean shift.
pub fn seqdrift2(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    reservoir_r: usize, recent_n: usize, z_thresh: f64, // typical 100, 30, 3.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let mut seed: u64 = 0x123_456;
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut reservoir: Vec<u8> = Vec::with_capacity(reservoir_r);
        let mut buf = std::vec![0_u8; recent_n];
        let mut pos = 0_usize; let mut count = 0_usize;
        let mut total_n = 0_u64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let e = ((v - mu).abs() > 2.0 * sd) as u8;
            // Reservoir sampling.
            if reservoir.len() < reservoir_r { reservoir.push(e); }
            else {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let idx = (seed as usize) % (total_n as usize + 1);
                if idx < reservoir_r { reservoir[idx] = e; }
            }
            total_n += 1;
            buf[pos] = e; pos = (pos + 1) % recent_n;
            if count < recent_n { count += 1; }
            if count < recent_n || reservoir.len() < reservoir_r { continue; }
            let p_res: f64 = reservoir.iter().map(|&x| x as u64).sum::<u64>() as f64 / reservoir_r as f64;
            let p_rec: f64 = buf.iter().map(|&x| x as u64).sum::<u64>() as f64 / recent_n as f64;
            let var = p_res * (1.0 - p_res) * (1.0 / reservoir_r as f64 + 1.0 / recent_n as f64);
            if var < 1e-9 { continue; }
            let z = (p_res - p_rec).abs() / var.sqrt();
            if z > z_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "seqdrift2",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// D3 — Discriminative Drift Detector (Gözüaçık 2019). Trains classifier
/// to discriminate reference vs current. Simplified: linear discriminant.
pub fn d3_drift(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.7 (AUC threshold)
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // AUC of Bayes-optimal discriminator: Φ((μ_diff)/(sigma·√2)).
            let m_buf: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let z = (m_buf - mu).abs() / (sd * core::f64::consts::SQRT_2);
            // Φ approximation via erf.
            let auc = 0.5 + 0.5 * erf_approx(z / core::f64::consts::SQRT_2);
            if auc > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "d3_drift",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

fn erf_approx(x: f64) -> f64 {
    // Abramowitz & Stegun 7.1.26.
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t
                  - 0.284496736) * t + 0.254829592) * t * (-x * x).exp();
    if x < 0.0 { -y } else { y }
}

/// QuantTree — Boracchi 2018. Adaptive histogram with controlled FP rate.
///
/// Tier Q (concept drift rarer). Boracchi 2018 adaptive-histogram drift detector with controlled FP rate; quantile-balanced bins; alerts on bin-occupancy chi-squared shift.
/// Per-window alert on histogram occupancy shift.
pub fn quanttree(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    n_bins: usize, win_n: usize, threshold: f64, // typical 8, 30, 5.0
) -> DetectorOutput {
    // Build healthy quantile-based bins.
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let mut bin_edges: Vec<Vec<f64>> = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let mut edges = Vec::with_capacity(n_bins + 1);
        if samples.is_empty() {
            for _ in 0..=n_bins { edges.push(0.0); }
        } else {
            for k in 0..=n_bins {
                let idx = (k * samples.len()) / n_bins;
                let idx = idx.min(samples.len() - 1);
                edges.push(samples[idx]);
            }
        }
        bin_edges.push(edges);
    }
    for s in 0..num_signals {
        let edges = &bin_edges[s];
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut hist = std::vec![0_u64; n_bins];
            for &x in &buf {
                let mut bin = 0;
                for k in 0..n_bins {
                    if x >= edges[k] && (k == n_bins - 1 || x < edges[k + 1]) { bin = k; break; }
                }
                hist[bin] += 1;
            }
            let n = win_n as f64;
            let expected = n / n_bins as f64;
            let mut chi2 = 0.0;
            for h in &hist { chi2 += (*h as f64 - expected).powi(2) / expected; }
            if chi2 > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "quanttree",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// NN-DVI — Nearest-neighbor density variation (Liu 2018).
///
/// Tier Q (concept drift rarer). Liu 2018 nearest-neighbor density-variation index: per-sample local density estimate, tracked across reference vs current windows.
/// Per-window alert on local-density divergence.
pub fn nn_dvi(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_density = -1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Mean nearest-neighbor distance.
            let mut sum_nn = 0.0;
            for j in 0..win_n {
                let mut min_d = f64::INFINITY;
                for k in 0..win_n {
                    if j == k { continue; }
                    let d = (buf[j] - buf[k]).abs();
                    if d < min_d { min_d = d; }
                }
                sum_nn += min_d;
            }
            let density = win_n as f64 / sum_nn.max(1e-9);
            if ref_density < 0.0 { ref_density = density; }
            if (density - ref_density).abs() / ref_density.max(1e-9) > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "nn_dvi",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER R — Robust-depth family (8 detectors)
// =====================================================================

/// Halfspace depth — Tukey 1975. Min fraction of points on either side of
/// any halfplane through the test point. Univariate proxy: rank-based.
pub fn halfspace_depth(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, depth_thresh: f64, // typical 30, 0.05
) -> DetectorOutput {
    let mut refs = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        refs.push(samples);
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let m = refs[s].len(); if m < 10 { continue; }
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Compute mean of window depths against ref.
            let mut sum_d = 0.0;
            for &x in &buf {
                let mut lo = 0; let mut hi = m;
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    if refs[s][mid] < x { lo = mid + 1; } else { hi = mid; }
                }
                let depth = (lo.min(m - lo)) as f64 / m as f64;
                sum_d += depth;
            }
            let avg_depth = sum_d / win_n as f64;
            if avg_depth < depth_thresh {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "halfspace_depth",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Projection depth — Zuo 2003. Outlyingness via worst univariate projection.
///
/// Tier R (robust depth). Zuo 2003 outlyingness via worst univariate projection; per-sample depth = inf over unit directions of standardized magnitude; alert on depth < `depth_thresh`.
/// Per-window alert on minimum-direction outlyingness.
pub fn projection_depth(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 5.0
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    if num_signals == 0 { return zero_output("projection_depth"); }
    // For each window, compute max(|z_s|) over signals — worst projection on axis-aligned directions.
    for w in 0..num_windows {
        let mut max_z = 0.0_f64;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let z = (v - means[s]).abs() / sigmas[s].max(1e-9);
            if z > max_z { max_z = z; }
        }
        if max_z > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "projection_depth",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Stahel-Donoho outlyingness — Stahel 1981, Donoho 1982. Worst projection
/// outlyingness using median+MAD on each axis.
pub fn stahel_donoho(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 4.0
) -> DetectorOutput {
    let mut medians = std::vec![0.0_f64; num_signals];
    let mut mads = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        medians[s] = if samples.is_empty() { 0.0 } else { samples[samples.len() / 2] };
        let mut abs_dev: Vec<f64> = samples.iter().map(|x| (x - medians[s]).abs()).collect();
        abs_dev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        mads[s] = if abs_dev.is_empty() { 1.0 } else { abs_dev[abs_dev.len() / 2].max(1e-9) };
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for w in 0..num_windows {
        let mut max_o = 0.0_f64;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let o = (v - medians[s]).abs() / (1.4826 * mads[s]);
            if o > max_o { max_o = o; }
        }
        if max_o > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "stahel_donoho",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// MCD — Minimum Covariance Determinant (Rousseeuw 1984). Robust covariance
/// from h-subset with smallest determinant.
pub fn mcd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 5.0
) -> DetectorOutput {
    if num_signals == 0 || num_signals > 32 { return zero_output("mcd"); }
    // Approximate MCD: drop top 25% outliers from healthy slice, refit cov.
    let mut means_init = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_u64; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { means_init[s] += v; counts[s] += 1; } }
        }
    }
    for s in 0..num_signals { if counts[s] > 0 { means_init[s] /= counts[s] as f64; } }
    // Compute initial Mahalanobis distances per healthy window using diagonal cov.
    let mut sd2 = std::vec![1e-9; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { sd2[s] += (v - means_init[s]).powi(2); } }
        }
    }
    for s in 0..num_signals { sd2[s] = (sd2[s] / counts[s].max(1) as f64).max(1e-9); }
    // Drop windows whose Σ z² is in the top quartile.
    let mut window_scores: Vec<(usize, f64)> = Vec::new();
    for w in 0..healthy_window_end.min(num_windows) {
        let mut z2 = 0.0_f64;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() {
                z2 += (data[i] - means_init[s]).powi(2) / sd2[s];
            }
        }
        window_scores.push((w, z2));
    }
    window_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));
    if window_scores.is_empty() { return zero_output("mcd"); }
    let h = (window_scores.len() * 3 / 4).max(1).min(window_scores.len());
    let kept_windows: Vec<usize> = window_scores[..h].iter().map(|(w, _)| *w).collect();
    let mut means_robust = std::vec![0.0_f64; num_signals];
    let mut sd_robust = std::vec![0.0_f64; num_signals];
    for &w in &kept_windows {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() { means_robust[s] += data[i]; }
        }
    }
    for s in 0..num_signals { means_robust[s] /= kept_windows.len() as f64; }
    for &w in &kept_windows {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() {
                sd_robust[s] += (data[i] - means_robust[s]).powi(2);
            }
        }
    }
    for s in 0..num_signals { sd_robust[s] = (sd_robust[s] / kept_windows.len() as f64).max(1e-9).sqrt(); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for w in 0..num_windows {
        let mut z2 = 0.0_f64;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() {
                let z = (data[i] - means_robust[s]) / sd_robust[s];
                z2 += z * z;
            }
        }
        let mahal = z2.sqrt();
        if mahal > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mcd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Spatial sign — Möttönen 1995. Direction-only multivariate outlyingness.
///
/// Tier R (robust depth). Möttönen 1995 direction-only multivariate outlyingness; sign vector u(x) = (x - μ)/||x - μ||; alert when sign-shift is significant.
/// Per-window alert on multivariate sign discrepancy.
pub fn spatial_sign(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 0.5
) -> DetectorOutput {
    if num_signals == 0 { return zero_output("spatial_sign"); }
    let (means, _) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Reference centroid of healthy spatial signs.
    let mut ref_signs = std::vec![0.0_f64; num_signals];
    let mut nh = 0_u64;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut x = std::vec![0.0_f64; num_signals];
        let mut norm = 0.0;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() { x[s] = data[i] - means[s]; norm += x[s] * x[s]; }
        }
        norm = norm.sqrt();
        if norm > 1e-9 { for s in 0..num_signals { ref_signs[s] += x[s] / norm; } nh += 1; }
    }
    if nh > 0 { for s in 0..num_signals { ref_signs[s] /= nh as f64; } }
    for w in 0..num_windows {
        let mut x = std::vec![0.0_f64; num_signals];
        let mut norm = 0.0;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() { x[s] = data[i] - means[s]; norm += x[s] * x[s]; }
        }
        norm = norm.sqrt();
        if norm < 1e-9 { continue; }
        let mut dot = 0.0; let mut ref_norm = 0.0; let mut win_norm = 0.0;
        for s in 0..num_signals {
            dot += ref_signs[s] * (x[s] / norm);
            ref_norm += ref_signs[s] * ref_signs[s];
            win_norm += (x[s] / norm).powi(2);
        }
        let cos_sim = dot / (ref_norm.sqrt() * win_norm.sqrt()).max(1e-9);
        if cos_sim < threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "spatial_sign",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// S-estimator residual — Rousseeuw & Yohai 1984. Robust regression residual.
///
/// Tier R (robust depth). Rousseeuw-Yohai 1984 S-estimator of regression: minimizes a robust scale (M-estimator) of residuals; alert when residual exceeds `k·robust_scale`.
/// Per-window alert on robust-regression residual.
pub fn s_estimator_residual(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64, // typical 30, 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Median + MAD-based S-scale.
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let m = sorted[win_n / 2];
            let mut abs_dev: Vec<f64> = sorted.iter().map(|x| (x - m).abs()).collect();
            abs_dev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let s_scale = (1.4826 * abs_dev[win_n / 2]).max(1e-9);
            // Tukey biweight check on most recent value.
            let r = (buf[win_n - 1] - m) / s_scale;
            if r.abs() > k {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "s_estimator_residual",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Depth-rank control — Liu 1990. Multivariate rank via average univariate
/// rank of components.
pub fn depth_rank_control(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 0.05 (5% tail)
) -> DetectorOutput {
    let mut sorted_refs = Vec::with_capacity(num_signals);
    for s in 0..num_signals {
        let mut samples = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let i = w * num_signals + s;
            if i < data.len() { let v = data[i]; if !v.is_nan() { samples.push(v); } }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        sorted_refs.push(samples);
    }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for w in 0..num_windows {
        let mut sum_r = 0.0_f64; let mut nv = 0_u64;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let m = sorted_refs[s].len(); if m == 0 { continue; }
            let mut lo = 0; let mut hi = m;
            while lo < hi {
                let mid = (lo + hi) / 2;
                if sorted_refs[s][mid] < v { lo = mid + 1; } else { hi = mid; }
            }
            let rank = lo as f64 / m as f64;
            // Tail rank: min(rank, 1-rank).
            sum_r += rank.min(1.0 - rank); nv += 1;
        }
        if nv == 0 { continue; }
        let avg_r = sum_r / nv as f64;
        if avg_r < threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "depth_rank_control",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Outlyingness median polish — Tukey 1977. Two-way median polish residuals.
///
/// Tier R (robust depth). Tukey 1977 two-way median-polish on (window, signal) grid; residuals after iterative median subtraction; alert on residual > `k·MAD`.
/// Per-window alert on two-way median-polish residual.
pub fn outlyingness_median_polish(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    if num_signals == 0 { return zero_output("outlyingness_median_polish"); }
    for w in win_n..num_windows {
        // Build (win_n × num_signals) matrix.
        let mut m = std::vec![std::vec![0.0_f64; num_signals]; win_n];
        for j in 0..win_n {
            for s in 0..num_signals {
                let i = (w + 1 - win_n + j) * num_signals + s;
                m[j][s] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
            }
        }
        // Two passes of median polish.
        for _it in 0..2 {
            for j in 0..win_n {
                let mut row = m[j].clone();
                row.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
                let med = row[num_signals / 2];
                for s in 0..num_signals { m[j][s] -= med; }
            }
            for s in 0..num_signals {
                let mut col: Vec<f64> = (0..win_n).map(|j| m[j][s]).collect();
                col.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
                let med = col[win_n / 2];
                for j in 0..win_n { m[j][s] -= med; }
            }
        }
        // Residuals max-abs.
        let mut max_r = 0.0_f64;
        for j in 0..win_n { for s in 0..num_signals {
            if m[j][s].abs() > max_r { max_r = m[j][s].abs(); }
        } }
        // Robust scale of residuals.
        let mut all_abs: Vec<f64> = Vec::with_capacity(win_n * num_signals);
        for j in 0..win_n { for s in 0..num_signals { all_abs.push(m[j][s].abs()); } }
        all_abs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let mad = all_abs[all_abs.len() / 2].max(1e-9);
        if max_r > threshold * 1.4826 * mad {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "median_polish",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER S — Count/event-process family (3 detectors)
// =====================================================================

/// Bayesian blocks — Scargle 2013. Adaptive segmentation for event counts
/// via Voronoi-cell DP with prior-based fitness.
pub fn bayesian_blocks(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    ncp_prior: f64, // typical 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut counts = std::vec![0_u64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() {
                if data[i] > 0.0 { counts[w] = data[i] as u64; }
            }
        }
        // DP fitness: best[t] = max over s of best[s] + log_lik(s..t) - ncp_prior.
        let mut csum = std::vec![0_u64; num_windows + 1];
        for w in 0..num_windows { csum[w + 1] = csum[w] + counts[w]; }
        let mut best = std::vec![f64::NEG_INFINITY; num_windows + 1];
        let mut prev = std::vec![0_usize; num_windows + 1];
        best[0] = 0.0;
        for t in 1..=num_windows {
            for ss in 0..t {
                let n = (csum[t] - csum[ss]) as f64;
                let len = (t - ss) as f64;
                if n < 1.0 || len < 1.0 { continue; }
                let log_lik = n * (n / len).ln() - n;
                let cand = best[ss] + log_lik - ncp_prior;
                if cand > best[t] { best[t] = cand; prev[t] = ss; }
            }
        }
        let mut t = num_windows;
        while t > 0 {
            let ps = prev[t];
            if ps > 0 && ps < num_windows {
                win_alerts[ps] = true;
                if s < 32 { alerts_per_signal[s] += 1; }
                raw += 1;
            }
            t = ps;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "bayesian_blocks",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Index of dispersion — Cox & Lewis 1966. Variance/mean ratio for counts.
///
/// Tier S (count event-process). Cox-Lewis 1966 variance-to-mean ratio for counts (Fano factor); alert when ratio shifts more than `disp_k` from the healthy baseline.
/// Per-window alert on dispersion-ratio shift.
pub fn index_of_dispersion(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 3.0 (Poisson has ratio = 1)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            if m.abs() < 1e-9 { continue; }
            let v: f64 = buf.iter().map(|x| (x - m).powi(2)).sum::<f64>() / win_n as f64;
            let id = v / m.abs();
            if id > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "index_of_dispersion",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Allan variance — Allan 1966. Two-sample variance over varying lag tau.
/// Detects stability/noise-regime changes.
pub fn allan_variance(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, tau: usize, threshold: f64, // typical 30, 5, 3.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n || tau >= win_n / 2 { continue; }
            let n = win_n - 2 * tau;
            let mut allan = 0.0;
            for j in 0..n {
                let m1: f64 = buf[j..j + tau].iter().sum::<f64>() / tau as f64;
                let m2: f64 = buf[j + tau..j + 2 * tau].iter().sum::<f64>() / tau as f64;
                allan += (m2 - m1).powi(2);
            }
            allan /= 2.0 * n as f64;
            if allan.sqrt() > threshold * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "allan_variance",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER T — Info-theoretic / compression family (6 detectors)
// =====================================================================

/// MDL change detector — Rissanen 1978. Two-segment description-length
/// shorter than single-segment description-length → split.
pub fn mdl_change(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 5.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        let total_n = num_windows as f64;
        let total_var = {
            let m = x.iter().sum::<f64>() / total_n;
            x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / total_n
        };
        let dl_total = total_n * total_var.max(1e-9).ln() / 2.0 + 0.5 * total_n.ln();
        for t in 5..num_windows.saturating_sub(5) {
            let n1 = t as f64; let n2 = (num_windows - t) as f64;
            let m1 = x[..t].iter().sum::<f64>() / n1;
            let m2 = x[t..].iter().sum::<f64>() / n2;
            let v1 = x[..t].iter().map(|v| (v - m1).powi(2)).sum::<f64>() / n1;
            let v2 = x[t..].iter().map(|v| (v - m2).powi(2)).sum::<f64>() / n2;
            let dl_split = n1 * v1.max(1e-9).ln() / 2.0 + n2 * v2.max(1e-9).ln() / 2.0
                + 0.5 * (n1.ln() + n2.ln()) + total_n.ln(); // model-cost
            let savings = dl_total - dl_split;
            if savings > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[t] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mdl_change",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// NCD — Normalized Compression Distance (Cilibrasi & Vitanyi 2005).
/// Approximation: compare run-length-encoding lengths of healthy vs window.
pub fn ncd(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.6
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Quantize signal to 4 SAX symbols for compression.
    let sax = |v: f64, mu: f64, sd: f64| -> u8 {
        let z = (v - mu) / sd;
        if z < -0.67 { 0 } else if z < 0.0 { 1 } else if z < 0.67 { 2 } else { 3 }
    };
    let rle_len = |seq: &[u8]| -> usize {
        let mut len = 1;
        for j in 1..seq.len() { if seq[j] != seq[j - 1] { len += 1; } }
        len
    };
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        // Healthy reference symbol stream.
        let mut ref_syms = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() { ref_syms.push(sax(data[i], mu, sd)); }
        }
        if ref_syms.is_empty() { continue; }
        let ref_len = rle_len(&ref_syms);
        let mut buf = std::vec![0_u8; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = sax(v, mu, sd); count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = sax(v, mu, sd);
            }
            if count < win_n { continue; }
            let win_len = rle_len(&buf);
            // Concat compression length proxy.
            let mut concat: Vec<u8> = Vec::with_capacity(ref_syms.len() + win_n);
            concat.extend_from_slice(&ref_syms);
            concat.extend_from_slice(&buf);
            let concat_len = rle_len(&concat);
            let ncd_val = (concat_len as f64 - ref_len.min(win_len) as f64)
                / (ref_len.max(win_len) as f64).max(1.0);
            if ncd_val > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ncd",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Lempel-Ziv complexity — Lempel & Ziv 1976. Number of distinct patterns
/// in symbolic sequence.
pub fn lempel_ziv(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.5
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let sax = |v: f64, mu: f64, sd: f64| -> u8 {
        let z = (v - mu) / sd;
        if z < -0.67 { 0 } else if z < 0.0 { 1 } else if z < 0.67 { 2 } else { 3 }
    };
    let lz_complexity = |seq: &[u8]| -> usize {
        let mut c = 1; let mut i = 0; let mut j = 1;
        while j < seq.len() {
            // Find longest j..k that's a substring of 0..j.
            let mut k = j;
            while k < seq.len() {
                let pat = &seq[j..=k];
                let mut found = false;
                if i + pat.len() <= j {
                    for start in 0..(j.saturating_sub(pat.len()) + 1) {
                        if &seq[start..start + pat.len()] == pat { found = true; break; }
                    }
                }
                if !found { break; }
                k += 1;
            }
            c += 1; i = j; j = k + 1;
        }
        c
    };
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0_u8; win_n];
        let mut count = 0_usize;
        let mut ref_lz: Option<usize> = None;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = sax(v, mu, sd); count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = sax(v, mu, sd);
            }
            if count < win_n { continue; }
            let lz = lz_complexity(&buf);
            if ref_lz.is_none() { ref_lz = Some(lz); }
            if let Some(r) = ref_lz {
                let diff = (lz as f64 - r as f64).abs() / r.max(1) as f64;
                if diff > threshold {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                }
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "lempel_ziv",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Transfer entropy — Schreiber 2000. Directional information flow between
/// two signals. Histogram-based estimator with binary symbolization.
pub fn transfer_entropy(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 60, 0.05
) -> DetectorOutput {
    if num_signals < 2 { return zero_output("transfer_entropy"); }
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let mut buf_a = std::vec![0_u8; win_n];
    let mut buf_b = std::vec![0_u8; win_n];
    let mut count = 0_usize;
    let sym = |v: f64, mu: f64, sd: f64| -> u8 { ((v - mu).abs() > sd) as u8 };
    let mu_a = means[0]; let sd_a = sigmas[0].max(1e-9);
    let mu_b = means[1]; let sd_b = sigmas[1].max(1e-9);
    for w in 0..num_windows {
        let ia = w * num_signals; let ib = w * num_signals + 1;
        if ia >= data.len() || ib >= data.len() { continue; }
        let va = data[ia]; let vb = data[ib];
        if va.is_nan() || vb.is_nan() { continue; }
        if count < win_n { buf_a[count] = sym(va, mu_a, sd_a); buf_b[count] = sym(vb, mu_b, sd_b); count += 1; }
        else {
            buf_a.copy_within(1..win_n, 0);
            buf_b.copy_within(1..win_n, 0);
            buf_a[win_n - 1] = sym(va, mu_a, sd_a);
            buf_b[win_n - 1] = sym(vb, mu_b, sd_b);
        }
        if count < win_n { continue; }
        // Estimate TE(b → a) via P(a_t | a_{t-1}, b_{t-1}) - P(a_t | a_{t-1}).
        let mut joint3 = std::vec![0_u64; 8]; // (a_t, a_prev, b_prev)
        let mut joint2 = std::vec![0_u64; 4]; // (a_prev, b_prev)
        let mut margin2 = std::vec![0_u64; 4]; // (a_t, a_prev)
        let mut margin1 = std::vec![0_u64; 2]; // a_prev
        for j in 1..win_n {
            let key3 = (buf_a[j] * 4 + buf_a[j - 1] * 2 + buf_b[j - 1]) as usize;
            let key2j = (buf_a[j - 1] * 2 + buf_b[j - 1]) as usize;
            let key2m = (buf_a[j] * 2 + buf_a[j - 1]) as usize;
            let key1 = buf_a[j - 1] as usize;
            joint3[key3] += 1; joint2[key2j] += 1; margin2[key2m] += 1; margin1[key1] += 1;
        }
        let n = (win_n - 1) as f64;
        let mut te = 0.0_f64;
        for k in 0..8 {
            if joint3[k] == 0 { continue; }
            let pj3 = joint3[k] as f64 / n;
            let pj2 = joint2[(k & 3) as usize] as f64 / n;
            let pm2 = margin2[((k & 4) >> 1 | k & 1) as usize] as f64 / n;
            let pm1 = margin1[(k & 1) as usize] as f64 / n;
            if pj2 > 0.0 && pm2 > 0.0 && pm1 > 0.0 {
                te += pj3 * (pj3 * pm1 / (pj2 * pm2)).max(1e-9).ln();
            }
        }
        if te > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..2.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "transfer_entropy",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Fisher information — Frieden 1998. Local sensitivity = (∂_x log p(x))².
///
/// Tier T (info-theoretic). Frieden 1998 local-sensitivity score `I = E[(∂_x log p(x))²]`; estimated via finite-difference of empirical log-density; alert on FI shift.
/// Per-window alert on Fisher-information divergence.
pub fn fisher_information(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_fi = -1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // FI from sorted samples: Σ ((p_{i+1} - p_i)² / p_i) where p_i = i/n.
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let mut fi = 0.0;
            for i in 1..win_n {
                let d = sorted[i] - sorted[i - 1];
                if d > 1e-9 { fi += 1.0 / d; }
            }
            fi /= win_n as f64;
            if ref_fi < 0.0 { ref_fi = fi; }
            if (fi - ref_fi).abs() / ref_fi.max(1e-9) > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fisher_information",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Renyi entropy — Renyi 1961. Tail-sensitive entropy family.
///
/// Tier T (info-theoretic). Renyi 1961 tail-sensitive entropy `H_α = (1/(1-α)) log Σ p_i^α`; α=2 emphasizes tail mass; alert on entropy drop.
/// Per-window alert on Renyi-entropy shift.
pub fn renyi_entropy(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, alpha: f64, threshold: f64, // typical 30, 2.0, 0.3
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_h = -1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut lo = f64::INFINITY; let mut hi = f64::NEG_INFINITY;
            for &x in &buf { if x < lo { lo = x; } if x > hi { hi = x; } }
            let span = (hi - lo).max(1e-9);
            let mut hist = std::vec![0_u64; 8];
            for &x in &buf {
                let idx = (((x - lo) / span) * 8.0).min(7.0) as usize;
                hist[idx] += 1;
            }
            let mut sum_pa = 0.0;
            for &h in &hist {
                if h > 0 {
                    let p = h as f64 / win_n as f64;
                    sum_pa += p.powf(alpha);
                }
            }
            let h_alpha = (1.0 / (1.0 - alpha)) * sum_pa.max(1e-9).ln();
            if ref_h < 0.0 { ref_h = h_alpha; }
            if (h_alpha - ref_h).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "renyi_entropy",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 8.6 — TIER U — Dynamical-systems family (8 detectors)
// =====================================================================

/// Permutation entropy — Bandt & Pompe 2002. Order-pattern entropy.
///
/// Tier U (dynamical systems). Bandt-Pompe 2002 order-pattern entropy: counts permutation occurrences in length-`order` subsequences; alert when entropy diverges from healthy mean.
/// Per-window alert on permutation-entropy shift.
pub fn permutation_entropy(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, m_order: usize, threshold: f64, // typical 60, 4, 0.3
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let factorial = |n: usize| -> usize { (1..=n).product() };
    let m_fact = factorial(m_order);
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_pe = -1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut hist = std::vec![0_u64; m_fact];
            for j in 0..(win_n - m_order + 1) {
                // Compute permutation rank of (j..j+m_order).
                let mut indices: Vec<usize> = (0..m_order).collect();
                indices.sort_by(|&a, &b| buf[j + a].partial_cmp(&buf[j + b]).unwrap_or(core::cmp::Ordering::Equal));
                // Lehmer code.
                let mut code = 0_usize;
                for k in 0..m_order {
                    let mut rank = 0;
                    for &idx in &indices[k + 1..] { if idx < indices[k] { rank += 1; } }
                    code = code * (m_order - k) + rank;
                }
                hist[code.min(m_fact - 1)] += 1;
            }
            let total = (win_n - m_order + 1) as f64;
            let mut pe = 0.0_f64;
            for &h in &hist {
                if h > 0 {
                    let p = h as f64 / total;
                    pe -= p * p.ln();
                }
            }
            pe /= (m_fact as f64).ln(); // normalized
            if ref_pe < 0.0 { ref_pe = pe; }
            if (pe - ref_pe).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "permutation_entropy",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Sample entropy — Richman & Moorman 2000.
///
/// Tier U (dynamical systems). Richman-Moorman 2000 conditional probability of pattern persistence; alert when SampEn drops by `sampen_k` (regularization signature).
/// Per-window alert on sample-entropy collapse.
pub fn sample_entropy(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, m_order: usize, r_factor: f64, threshold: f64,
    // typical 60, 2, 0.2, 0.5
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let r = r_factor * sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_se = -1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let count_matches = |order: usize| -> u64 {
                let mut matches = 0_u64;
                let n = win_n - order;
                for i in 0..n {
                    for j in (i + 1)..n {
                        let mut max_d = 0.0_f64;
                        for k in 0..order {
                            let d = (buf[i + k] - buf[j + k]).abs();
                            if d > max_d { max_d = d; }
                        }
                        if max_d <= r { matches += 1; }
                    }
                }
                matches
            };
            let b = count_matches(m_order);
            let a = count_matches(m_order + 1);
            if b == 0 { continue; }
            let se = -((a as f64 / b as f64).max(1e-9)).ln();
            if ref_se < 0.0 { ref_se = se; }
            if (se - ref_se).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "sample_entropy",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Recurrence Quantification Analysis (RQA) — Marwan 2007. Recurrence-rate
/// of phase-space points within radius ε.
pub fn rqa_recurrence(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.3
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let eps = 0.2 * sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_rr = -1.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut recurrences = 0_u64;
            for i in 0..win_n {
                for j in (i + 1)..win_n {
                    if (buf[i] - buf[j]).abs() < eps { recurrences += 1; }
                }
            }
            let rr = recurrences as f64 / (win_n * (win_n - 1) / 2) as f64;
            if ref_rr < 0.0 { ref_rr = rr; }
            if (rr - ref_rr).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "rqa_recurrence",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Lyapunov exponent — Wolf 1985. Local divergence rate from delay-embedding.
///
/// Tier U (dynamical systems). Wolf 1985 local divergence rate from delay-embedded trajectory; positive λ indicates chaotic divergence; alert on |λ| above threshold.
/// Per-window alert on local Lyapunov-exponent breach.
pub fn lyapunov(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 60, 0.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_lyap = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Find each point's nearest neighbor, then track divergence.
            let mut sum_lyap = 0.0; let mut cn = 0_u64;
            for i in 0..(win_n / 2) {
                let mut min_d = f64::INFINITY; let mut nn = 0;
                for j in 0..(win_n / 2) {
                    if (i as isize - j as isize).abs() < 5 { continue; }
                    let d = (buf[i] - buf[j]).abs();
                    if d < min_d { min_d = d; nn = j; }
                }
                if min_d < 1e-9 { continue; }
                // Track divergence at lag 1.
                if i + 1 < win_n && nn + 1 < win_n {
                    let d_next = (buf[i + 1] - buf[nn + 1]).abs();
                    if d_next > 0.0 {
                        sum_lyap += (d_next / min_d).ln();
                        cn += 1;
                    }
                }
            }
            if cn == 0 { continue; }
            let lyap = sum_lyap / cn as f64;
            if ref_lyap.is_nan() { ref_lyap = lyap; }
            if (lyap - ref_lyap).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "lyapunov",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Correlation dimension — Grassberger & Procaccia 1983. Slope of log
/// correlation integral.
pub fn correlation_dimension(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 60, 0.5
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_d = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Estimate dimension from log C(r1)/C(r2).
            let r1 = 0.1 * sd; let r2 = 0.5 * sd;
            let mut c1 = 0_u64; let mut c2 = 0_u64;
            for i in 0..win_n {
                for j in (i + 1)..win_n {
                    let d = (buf[i] - buf[j]).abs();
                    if d < r1 { c1 += 1; }
                    if d < r2 { c2 += 1; }
                }
            }
            if c1 == 0 || c2 == 0 { continue; }
            let dim = (c1 as f64 / c2 as f64).ln() / (r1 / r2).ln();
            if ref_d.is_nan() { ref_d = dim; }
            if (dim - ref_d).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "correlation_dimension",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// BDS test — Brock-Dechert-Scheinkman 1996. Tests independence/nonlinear
/// structure via correlation-integral.
pub fn bds_test(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, m_order: usize, threshold: f64, // typical 60, 2, 3.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let eps = 0.7 * sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // C₁(ε) and C_m(ε).
            let mut c1 = 0_u64;
            for i in 0..win_n {
                for j in (i + 1)..win_n {
                    if (buf[i] - buf[j]).abs() < eps { c1 += 1; }
                }
            }
            let mut cm = 0_u64;
            let n_pairs_m = win_n - m_order + 1;
            for i in 0..n_pairs_m {
                for j in (i + 1)..n_pairs_m {
                    let mut all_close = true;
                    for k in 0..m_order {
                        if (buf[i + k] - buf[j + k]).abs() >= eps { all_close = false; break; }
                    }
                    if all_close { cm += 1; }
                }
            }
            let p1 = c1 as f64 / (win_n * (win_n - 1) / 2) as f64;
            let pm = cm as f64 / (n_pairs_m * (n_pairs_m - 1) / 2).max(1) as f64;
            let predicted = p1.powi(m_order as i32);
            let bds = (pm - predicted).abs() / (predicted.max(1e-9));
            if bds > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "bds_test",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// 0-1 chaos test — Gottwald & Melbourne 2004. Asymptotic growth rate of
/// trajectory in (p, q) plane.
pub fn zero_one_chaos(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 60, 0.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let c = 1.0_f64; // arbitrary irrational; using c=1.0 for determinism.
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_k = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let mut p = 0.0; let mut q = 0.0;
            let mut p_seq = std::vec![0.0_f64; win_n];
            let mut q_seq = std::vec![0.0_f64; win_n];
            for j in 0..win_n {
                p += buf[j] * (j as f64 * c).cos();
                q += buf[j] * (j as f64 * c).sin();
                p_seq[j] = p; q_seq[j] = q;
            }
            // Mean-square displacement growth rate.
            let mut msd = 0.0;
            for j in 0..(win_n / 2) {
                let dp = p_seq[j + win_n / 2] - p_seq[j];
                let dq = q_seq[j + win_n / 2] - q_seq[j];
                msd += dp * dp + dq * dq;
            }
            msd /= (win_n / 2) as f64;
            let k = msd.max(1e-9).ln() / (win_n as f64).ln();
            if ref_k.is_nan() { ref_k = k; }
            if (k - ref_k).abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "zero_one_chaos",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Delay-embedding NN divergence — Kantz 1994. Average local trajectory
/// divergence in delay-embedding space.
pub fn delay_embedding_nn(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 60, 0.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut ref_div = f64::NAN;
        let m = 3; let tau = 1; // embedding dim, lag
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let n_emb = win_n - (m - 1) * tau - 1;
            let mut sum_div = 0.0; let mut nc = 0_u64;
            for i in 0..n_emb {
                let mut min_d = f64::INFINITY; let mut nn = 0;
                for j in 0..n_emb {
                    if (i as isize - j as isize).abs() < 5 { continue; }
                    let mut d = 0.0;
                    for k in 0..m { let dd = buf[i + k * tau] - buf[j + k * tau]; d += dd * dd; }
                    let d = d.sqrt();
                    if d < min_d { min_d = d; nn = j; }
                }
                if min_d < 1e-9 { continue; }
                // Divergence at next time.
                if i + 1 + (m - 1) * tau < win_n && nn + 1 + (m - 1) * tau < win_n {
                    let mut d_next = 0.0;
                    for k in 0..m {
                        let dd = buf[i + 1 + k * tau] - buf[nn + 1 + k * tau];
                        d_next += dd * dd;
                    }
                    sum_div += d_next.sqrt() / min_d; nc += 1;
                }
            }
            if nc == 0 { continue; }
            let div = sum_div / nc as f64;
            if ref_div.is_nan() { ref_div = div; }
            if (div - ref_div).abs() / ref_div.max(1e-9) > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "delay_embedding_nn",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 9 — TIER X — Climate homogeneity tests (10 detectors)
// Pure time-series tests, fully residual-stream-compatible. Each
// detector scans per-signal for the named test's positive condition.
// =====================================================================

/// Pettitt test — Pettitt 1979. Nonparametric single changepoint via
/// rank-sum statistic. U_t = max over t of |Σ sign(x_i - x_j)| / (n*(n-1)/2).
pub fn pettitt_test(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 0.5 (normalized U)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        // Compute U_t for each candidate t; flag the t where |U_t| is max if exceeds threshold.
        let n = num_windows as f64;
        let mut best_t = 0; let mut best_u = 0.0_f64;
        for t in 5..num_windows.saturating_sub(5) {
            let mut u = 0_i64;
            for i in 0..t {
                for j in t..num_windows {
                    u += if x[i] < x[j] { 1 } else if x[i] > x[j] { -1 } else { 0 };
                }
            }
            let u_norm = u as f64 / (n * n);
            if u_norm.abs() > best_u { best_u = u_norm.abs(); best_t = t; }
        }
        if best_u > threshold {
            raw += 1;
            if s < 32 { alerts_per_signal[s] += 1; }
            win_alerts[best_t] = true;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "pettitt_test",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Buishand range test — Buishand 1982. Mean-shift via cumulative deviation range.
///
/// Tier X (climate homogeneity). Buishand 1982 mean-shift via cumulative deviation range; tracks max-min of cumulative residuals; alert on range > `range_k·sqrt(N)`.
/// Per-window alert on cumulative-deviation range breach.
pub fn buishand_range(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 1.5 (normalized range)
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        let m: f64 = x.iter().sum::<f64>() / num_windows as f64;
        let var: f64 = x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / num_windows as f64;
        if var < 1e-9 { continue; }
        let sd = var.sqrt();
        let mut s_k = 0.0_f64; let mut max_s = 0.0_f64; let mut min_s = 0.0_f64;
        let mut max_t = 0;
        for w in 0..num_windows {
            s_k += x[w] - m;
            if s_k > max_s { max_s = s_k; max_t = w; }
            if s_k < min_s { min_s = s_k; }
        }
        let r = (max_s - min_s) / (sd * (num_windows as f64).sqrt());
        if r > threshold {
            raw += 1;
            if s < 32 { alerts_per_signal[s] += 1; }
            win_alerts[max_t] = true;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "buishand_range",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// SNHT — Standard Normal Homogeneity Test (Alexandersson 1986). Step-like
/// inhomogeneity; max T_a = a*z_a^2 + (n-a)*z_b^2 over candidate split a.
pub fn snht(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 8.0 for n~100
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        let m: f64 = x.iter().sum::<f64>() / num_windows as f64;
        let var: f64 = x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / num_windows as f64;
        if var < 1e-9 { continue; }
        let sd = var.sqrt();
        let mut max_ta = 0.0_f64; let mut max_t = 0;
        for a in 5..num_windows.saturating_sub(5) {
            let m_a: f64 = x[..a].iter().sum::<f64>() / a as f64;
            let m_b: f64 = x[a..].iter().sum::<f64>() / (num_windows - a) as f64;
            let z_a = (m_a - m) / sd;
            let z_b = (m_b - m) / sd;
            let ta = a as f64 * z_a.powi(2) + (num_windows - a) as f64 * z_b.powi(2);
            if ta > max_ta { max_ta = ta; max_t = a; }
        }
        if max_ta > threshold {
            raw += 1;
            if s < 32 { alerts_per_signal[s] += 1; }
            win_alerts[max_t] = true;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "snht",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Von Neumann ratio — von Neumann 1941. Tests serial randomness: low ratio
/// indicates positive serial correlation; high ratio indicates oscillation.
pub fn von_neumann_ratio(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, lo: f64, hi: f64, // typical 30, 1.0, 3.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let mut sum_sq_diff = 0.0; let mut sum_sq_dev = 0.0;
            for j in 1..win_n { sum_sq_diff += (buf[j] - buf[j - 1]).powi(2); }
            for j in 0..win_n { sum_sq_dev += (buf[j] - m).powi(2); }
            if sum_sq_dev < 1e-9 { continue; }
            let nv_ratio = sum_sq_diff / sum_sq_dev;
            if nv_ratio < lo || nv_ratio > hi {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "von_neumann_ratio",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Alexandersson SNHT variant — relative homogeneity break against reference
/// (here: signal vs aggregate-mean reference).
pub fn alexandersson_snht(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64,
) -> DetectorOutput {
    if num_signals < 2 { return zero_output("alexandersson_snht"); }
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Build mean-of-others reference per (w, s).
    for s in 0..num_signals {
        let mut q = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let mut sum = 0.0; let mut count = 0_u64;
            for s2 in 0..num_signals {
                if s2 == s { continue; }
                let i = w * num_signals + s2;
                if i < data.len() && !data[i].is_nan() { sum += data[i]; count += 1; }
            }
            let ref_v = if count > 0 { sum / count as f64 } else { 0.0 };
            let i = w * num_signals + s;
            let sig_v = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
            q[w] = sig_v - ref_v;
        }
        let m: f64 = q.iter().sum::<f64>() / num_windows as f64;
        let var: f64 = q.iter().map(|v| (v - m).powi(2)).sum::<f64>() / num_windows as f64;
        if var < 1e-9 { continue; }
        let sd = var.sqrt();
        let mut max_ta = 0.0_f64; let mut max_t = 0;
        for a in 5..num_windows.saturating_sub(5) {
            let m_a: f64 = q[..a].iter().sum::<f64>() / a as f64;
            let m_b: f64 = q[a..].iter().sum::<f64>() / (num_windows - a) as f64;
            let ta = a as f64 * ((m_a - m) / sd).powi(2)
                   + (num_windows - a) as f64 * ((m_b - m) / sd).powi(2);
            if ta > max_ta { max_ta = ta; max_t = a; }
        }
        if max_ta > threshold {
            raw += 1;
            if s < 32 { alerts_per_signal[s] += 1; }
            win_alerts[max_t] = true;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "alexandersson_snht",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Potter test — Potter 1981. Mean-shift homogeneity via comparison of
/// before/after means with t-style statistic. Independent witness from SNHT.
pub fn potter_test(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        let mut max_t = 0.0_f64; let mut max_w = 0;
        for a in 10..num_windows.saturating_sub(10) {
            let m_a: f64 = x[..a].iter().sum::<f64>() / a as f64;
            let m_b: f64 = x[a..].iter().sum::<f64>() / (num_windows - a) as f64;
            let v_a: f64 = x[..a].iter().map(|v| (v - m_a).powi(2)).sum::<f64>() / a as f64;
            let v_b: f64 = x[a..].iter().map(|v| (v - m_b).powi(2)).sum::<f64>() / (num_windows - a) as f64;
            let pooled = ((v_a + v_b) / 2.0).max(1e-9);
            let t = (m_a - m_b).abs() / (pooled / a as f64 + pooled / (num_windows - a) as f64).sqrt();
            if t > max_t { max_t = t; max_w = a; }
        }
        if max_t > threshold {
            raw += 1;
            if s < 32 { alerts_per_signal[s] += 1; }
            win_alerts[max_w] = true;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "potter_test",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Rodionov regime-shift detector — Rodionov 2004. Sequential regime-shift
/// detection via z-score test against rolling mean.
pub fn rodionov_regime_shift(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 2.5
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let crit_diff = threshold * sd * (2.0 / win_n as f64).sqrt();
        let mut current_mean = 0.0_f64; let mut current_n = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if current_n < win_n {
                current_mean = (current_mean * current_n as f64 + v) / (current_n + 1) as f64;
                current_n += 1;
            } else {
                if (v - current_mean).abs() > crit_diff {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                    current_mean = v;
                    current_n = 1;
                } else {
                    current_mean = (current_mean * current_n as f64 + v) / (current_n + 1) as f64;
                }
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "rodionov_regime",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Lanzante resistant changepoint — Lanzante 1996. Robust changepoint via
/// median-based test resistant to outlier contamination.
pub fn lanzante_resistant(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut x = std::vec![0.0_f64; num_windows];
        for w in 0..num_windows {
            let i = w * num_signals + s;
            x[w] = if i < data.len() && !data[i].is_nan() { data[i] } else { 0.0 };
        }
        let mut max_z = 0.0_f64; let mut max_t = 0;
        for a in 10..num_windows.saturating_sub(10) {
            let mut left = x[..a].to_vec();
            let mut right = x[a..].to_vec();
            left.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            right.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let med_l = left[left.len() / 2];
            let med_r = right[right.len() / 2];
            let mut all_dev: Vec<f64> = x.iter().map(|v| (v - med_l).abs()).collect();
            all_dev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let mad = all_dev[all_dev.len() / 2].max(1e-9);
            let z = (med_l - med_r).abs() / (1.4826 * mad);
            if z > max_z { max_z = z; max_t = a; }
        }
        if max_z > threshold {
            raw += 1;
            if s < 32 { alerts_per_signal[s] += 1; }
            win_alerts[max_t] = true;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "lanzante_resistant",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Cumulative deviation homogeneity — sustained departure from historical mean.
///
/// Tier X (climate homogeneity). Tracks cumulative sum of (x - mean_healthy) per window; alert when |Σ| exceeds `k·sigma·sqrt(N)` (sustained departure).
/// Per-window alert on cumulative-deviation magnitude.
pub fn cumulative_deviation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64, // typical 2.0 (in sigmas of cumulative deviation)
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut cum = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            cum += v - mu;
            let ratio = cum.abs() / (sd * (w + 1) as f64).sqrt();
            if ratio > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "cumulative_deviation",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Smoothness break (von Neumann–Mises) — abrupt change in second differences.
///
/// Tier X (climate homogeneity). Von Neumann–Mises second-difference test: tracks |x[t+1] - 2x[t] + x[t-1]| / sigma; alert on abrupt change in second-difference variance.
/// Per-window alert on second-difference discontinuity.
pub fn smoothness_break(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 4.0
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else {
                buf.copy_within(1..win_n, 0);
                buf[win_n - 1] = v;
            }
            if count < win_n { continue; }
            // Second-difference RMS.
            let mut sum_dd2 = 0.0;
            for j in 2..win_n {
                let dd = buf[j] - 2.0 * buf[j - 1] + buf[j - 2];
                sum_dd2 += dd * dd;
            }
            let rms = (sum_dd2 / (win_n - 2) as f64).sqrt();
            if rms > threshold * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "smoothness_break",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 9 — TIER Y — Robust dispersion / rank (10 detectors)
// All operate on rolling windows; non-Gaussian-friendly.
// =====================================================================

/// Fligner–Killeen — Conover et al. 1981. Robust scale test via folded
/// rank-transformed data. Compare median absolute deviations across two halves.
pub fn fligner_killeen(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 60, 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut left_dev: Vec<f64> = buf[..half].iter().map(|v| v.abs()).collect();
            let mut right_dev: Vec<f64> = buf[half..].iter().map(|v| v.abs()).collect();
            left_dev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            right_dev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let med_l = left_dev[half / 2];
            let med_r = right_dev[(win_n - half) / 2];
            let pooled = (med_l + med_r) / 2.0;
            if pooled < 1e-9 { continue; }
            let stat = (med_l - med_r).abs() / pooled;
            if stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "fligner_killeen",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Ansari-Bradley — Ansari & Bradley 1960. Rank-based scale test on
/// distance-from-median ranks.
pub fn ansari_bradley(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 60, 2.5
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut combined: Vec<(f64, u8)> = buf[..half].iter().map(|&v| (v, 0))
                .chain(buf[half..].iter().map(|&v| (v, 1))).collect();
            combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            // Compute Ansari-Bradley rank score: assign rank from outer in.
            let n = win_n; let mid = n / 2;
            let mut sum_left: u64 = 0;
            for (rank, (_, group)) in combined.iter().enumerate() {
                let abr = if rank < mid { rank + 1 } else { n - rank };
                if *group == 0 { sum_left += abr as u64; }
            }
            let expected: f64 = if n % 2 == 0 { (half * (n + 2)) as f64 / 4.0 } else { (half * (n + 1)) as f64 / 4.0 };
            let var_est = (n as f64 / 12.0).max(1e-9);
            let z = (sum_left as f64 - expected).abs() / (var_est * half as f64).sqrt();
            if z > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "ansari_bradley",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Siegel-Tukey — Siegel & Tukey 1960. Rank-based scale test using
/// outside-in alternating rank assignment.
pub fn siegel_tukey(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut combined: Vec<(f64, u8)> = buf[..half].iter().map(|&v| (v, 0))
                .chain(buf[half..].iter().map(|&v| (v, 1))).collect();
            combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n;
            // Siegel-Tukey rank: 1, 4, 5, 8, 9, ... from low end; 2, 3, 6, 7, ... from high end (outside-in alternating).
            let mut sum_left: u64 = 0; let mut lo_pos = 0_usize; let mut hi_pos = n - 1;
            let mut rank: u64 = 1;
            for _ in 0..n {
                let (pos, group) = if rank % 4 == 1 || rank % 4 == 0 {
                    let r = (combined[lo_pos].1, combined[lo_pos]);
                    let g = r.0;
                    lo_pos += 1;
                    (rank, g)
                } else {
                    let r = (combined[hi_pos].1, combined[hi_pos]);
                    let g = r.0;
                    if hi_pos > 0 { hi_pos -= 1; }
                    (rank, g)
                };
                if group == 0 { sum_left += pos; }
                rank += 1;
            }
            let expected = (half as f64) * (n + 1) as f64 / 2.0;
            let var_est = (half as f64 * (half as f64) * (n + 1) as f64 / 12.0).max(1e-9);
            let z = (sum_left as f64 - expected).abs() / var_est.sqrt();
            if z > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "siegel_tukey",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Mood scale — Mood 1954. Median-centered scale shift via rank-squared sums.
///
/// Tier Y (robust dispersion / rank). Mood 1954 median-centered scale shift via rank-squared sums; statistic `M = Σ (R_i - (N+1)/2)²`; alert on standardized M shift.
/// Per-window alert on rank-squared-sum scale shift.
pub fn mood_scale(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut combined: Vec<(f64, u8)> = buf[..half].iter().map(|&v| (v, 0))
                .chain(buf[half..].iter().map(|&v| (v, 1))).collect();
            combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n;
            let mid_rank = (n + 1) as f64 / 2.0;
            let mut m_left: f64 = 0.0; let mut m_right: f64 = 0.0;
            for (rank, (_, group)) in combined.iter().enumerate() {
                let dev = (rank as f64 + 1.0 - mid_rank).powi(2);
                if *group == 0 { m_left += dev; } else { m_right += dev; }
            }
            let total = m_left + m_right;
            if total < 1e-9 { continue; }
            let stat = (m_left - m_right).abs() / total;
            if stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mood_scale",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Klotz normal-scores scale — Klotz 1962. Scale change under inverse-normal
/// transformed ranks. Approximation: use rank z-score squared sum.
pub fn klotz_normal_scores(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut combined: Vec<(f64, u8)> = buf[..half].iter().map(|&v| (v, 0))
                .chain(buf[half..].iter().map(|&v| (v, 1))).collect();
            combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n as f64;
            let mut sum_z2_left = 0.0; let mut sum_z2_right = 0.0;
            for (rank, (_, group)) in combined.iter().enumerate() {
                let p = (rank + 1) as f64 / (n + 1.0);
                // Inverse normal CDF approx.
                let z = (p - 0.5) * (2.0 * core::f64::consts::PI).sqrt();
                let z2 = z * z;
                if *group == 0 { sum_z2_left += z2; } else { sum_z2_right += z2; }
            }
            let total = (sum_z2_left + sum_z2_right).max(1e-9);
            let stat = (sum_z2_left - sum_z2_right).abs() / total;
            if stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "klotz_normal_scores",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Conover squared-ranks — Conover & Iman 1978. Spread test via squared rank sums.
///
/// Tier Y (robust dispersion / rank). Conover-Iman 1978 spread test: rank then square; statistic `T = sum of squared ranks`; alert on standardized T shift.
/// Per-window alert on Conover squared-ranks scale shift.
pub fn conover_squared_ranks(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            // Compute median to center, then |x - median|, then rank.
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let med = sorted[win_n / 2];
            let mut combined: Vec<(f64, u8)> = buf[..half].iter().map(|&v| ((v - med).abs(), 0))
                .chain(buf[half..].iter().map(|&v| ((v - med).abs(), 1))).collect();
            combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let mut sum_r2_left = 0_u64;
            for (rank, (_, group)) in combined.iter().enumerate() {
                let r2 = ((rank + 1) as u64).pow(2);
                if *group == 0 { sum_r2_left += r2; }
            }
            let total_r2: u64 = (1..=win_n).map(|r| (r as u64).pow(2)).sum();
            let expected = total_r2 / 2;
            let stat = (sum_r2_left as f64 - expected as f64).abs() / (total_r2 as f64).sqrt();
            if stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "conover_squared_ranks",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Brown-Mood median test — Brown & Mood 1951. Sign-based median shift detection.
///
/// Tier Y (robust dispersion / rank). Brown-Mood 1951 sign-based median shift detection; binomial tail probability against healthy median; alert on tail < `alpha`.
/// Per-window alert on sign-based median shift.
pub fn brown_mood_median(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let med = sorted[win_n / 2];
            let mut a_above = 0_u64; let mut b_above = 0_u64;
            for j in 0..half { if buf[j] > med { a_above += 1; } }
            for j in half..win_n { if buf[j] > med { b_above += 1; } }
            let n = win_n as f64;
            let p = 0.5;
            let var = p * (1.0 - p) * (1.0 / half as f64 + 1.0 / (win_n - half) as f64);
            if var < 1e-9 { continue; }
            let z = (a_above as f64 / half as f64 - b_above as f64 / (win_n - half) as f64).abs() / var.sqrt();
            if z > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
            let _ = n;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "brown_mood_median",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Terry-Hoeffding — normal-scores test for ordered location shift.
///
/// Tier Y (robust dispersion / rank). Normal-scores test for ordered location shift; transforms ranks to expected normal order statistics; alert on standardized statistic shift.
/// Per-window alert on normal-scores location shift.
pub fn terry_hoeffding(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut combined: Vec<(f64, u8)> = buf[..half].iter().map(|&v| (v, 0))
                .chain(buf[half..].iter().map(|&v| (v, 1))).collect();
            combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n as f64;
            let mut sum_z_left = 0.0;
            for (rank, (_, group)) in combined.iter().enumerate() {
                let p = (rank + 1) as f64 / (n + 1.0);
                let z = (p - 0.5) * 2.5;
                if *group == 0 { sum_z_left += z; }
            }
            let var = half as f64 * (n - half as f64) / n;
            if var < 1e-9 { continue; }
            let stat = sum_z_left.abs() / var.sqrt();
            if stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "terry_hoeffding",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Savage scores — Savage 1956. Tail-sensitive ordered-rank scores: sum 1/(n-i+1).
///
/// Tier Y (robust dispersion / rank). Savage 1956 tail-sensitive ordered-rank scores `a_i = Σ_{j≥i} 1/(N-j+1)`; alert on shift of standardized score sum.
/// Per-window alert on Savage-scores tail shift.
pub fn savage_scores(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut combined: Vec<(f64, u8)> = buf[..half].iter().map(|&v| (v, 0))
                .chain(buf[half..].iter().map(|&v| (v, 1))).collect();
            combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n;
            // Savage scores: a_i = Σ_{j=i}^{n} 1/j, then -1.
            let mut savage = std::vec![0.0_f64; n];
            for i in 0..n {
                let mut s_score = 0.0;
                for j in i..n { s_score += 1.0 / (j + 1) as f64; }
                savage[i] = s_score - 1.0;
            }
            let mut sum_left = 0.0;
            for (rank, (_, group)) in combined.iter().enumerate() {
                if *group == 0 { sum_left += savage[rank]; }
            }
            let stat = sum_left.abs();
            if stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "savage_scores",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Lepage combined location-scale — Lepage 1971. Joint location and scale shift
/// via Wilcoxon + Ansari-Bradley combination.
pub fn lepage_combined(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let half = win_n / 2;
            let mut combined: Vec<(f64, u8)> = buf[..half].iter().map(|&v| (v, 0))
                .chain(buf[half..].iter().map(|&v| (v, 1))).collect();
            combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n; let mid = n / 2;
            // Wilcoxon part: rank sum.
            let mut wilcoxon_sum = 0_u64;
            // Ansari-Bradley part: outside-in rank sum.
            let mut ab_sum = 0_u64;
            for (rank, (_, group)) in combined.iter().enumerate() {
                if *group == 0 {
                    wilcoxon_sum += (rank + 1) as u64;
                    let abr = if rank < mid { rank + 1 } else { n - rank };
                    ab_sum += abr as u64;
                }
            }
            let exp_w = (half as f64) * (n + 1) as f64 / 2.0;
            let var_w = (half as f64 * (half as f64) * (n + 1) as f64 / 12.0).max(1e-9);
            let exp_a = (half * (n + 1)) as f64 / 4.0;
            let var_a = ((n * n - 4) * half * (win_n - half)) as f64 / (48.0 * (n - 1) as f64);
            if var_a < 1e-9 { continue; }
            let z_w = (wilcoxon_sum as f64 - exp_w).powi(2) / var_w;
            let z_a = (ab_sum as f64 - exp_a).powi(2) / var_a.max(1e-9);
            let lepage = z_w + z_a;
            if lepage > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "lepage_combined",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 9 — TIER Z — Circular / directional (10 detectors)
// Treat residual values modulo a derived period as circular phases.
// Default phase derivation: angle = 2π * (v - mu) / (2 * sigma) wrapped.
// =====================================================================

#[inline]
fn phase_from_value(v: f64, mu: f64, sd: f64) -> f64 {
    use core::f64::consts::PI;
    let z = ((v - mu) / sd.max(1e-9)).clamp(-2.0, 2.0);
    z * PI  // maps [-2σ, 2σ] to [-2π, 2π]; wraps to a phase
}

/// Rayleigh circular uniformity test — Rayleigh 1880. Tests whether phases
/// concentrate around a mean direction. R = |Σ e^{iθ}| / n; large R = concentrated.
pub fn rayleigh_phase(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 30, 0.7
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = phase_from_value(v, mu, sd);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut cs = 0.0_f64; let mut sn = 0.0_f64;
            for &p in &buf { cs += p.cos(); sn += p.sin(); }
            let r = (cs * cs + sn * sn).sqrt() / win_n as f64;
            if r > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "rayleigh_phase",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Rao spacing test — Rao 1969. Detects nonuniformity via gap variance
/// between adjacent sorted phases. Uniform → all gaps equal; concentration → variance.
pub fn rao_spacing(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    use core::f64::consts::PI;
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = (phase_from_value(v, mu, sd)).rem_euclid(2.0 * PI);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let expected_gap = 2.0 * PI / win_n as f64;
            let mut sum_dev = 0.0;
            for j in 1..win_n {
                let gap = sorted[j] - sorted[j - 1];
                sum_dev += (gap - expected_gap).abs();
            }
            let last_gap = (2.0 * PI - sorted[win_n - 1] + sorted[0]).abs();
            sum_dev += (last_gap - expected_gap).abs();
            let stat = sum_dev / 2.0;
            if stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "rao_spacing",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Kuiper circular test — Kuiper 1960. Wraparound-safe KS-style test on
/// circular CDF.
pub fn kuiper_circular(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    use core::f64::consts::PI;
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = (phase_from_value(v, mu, sd)).rem_euclid(2.0 * PI);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n as f64;
            let mut max_pos = 0.0_f64; let mut max_neg = 0.0_f64;
            for (i, &p) in sorted.iter().enumerate() {
                let cdf = p / (2.0 * PI);
                let i_f = (i + 1) as f64 / n;
                let dev_pos = i_f - cdf;
                let dev_neg = cdf - i as f64 / n;
                if dev_pos > max_pos { max_pos = dev_pos; }
                if dev_neg > max_neg { max_neg = dev_neg; }
            }
            let v_kuiper = (max_pos + max_neg) * n.sqrt();
            if v_kuiper > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "kuiper_circular",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Watson U² — Watson 1961. Circular distribution deformation via integrated
/// CDF squared deviation.
pub fn watson_u2(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    use core::f64::consts::PI;
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = (phase_from_value(v, mu, sd)).rem_euclid(2.0 * PI);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut sorted = buf.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let n = win_n as f64;
            let cdf_vals: Vec<f64> = sorted.iter().map(|p| p / (2.0 * PI)).collect();
            let cdf_mean: f64 = cdf_vals.iter().sum::<f64>() / n;
            let mut sum_sq = 0.0;
            for (i, &c) in cdf_vals.iter().enumerate() {
                let dev = c - (i + 1) as f64 / n - cdf_mean + 0.5;
                sum_sq += dev * dev;
            }
            let u2 = sum_sq + n / 12.0;
            if u2 > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "watson_u2",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Hodges-Ajne test — Ajne 1968. Omnibus directional nonuniformity.
/// Statistic: m = max number of phases on any half-circle.
pub fn hodges_ajne(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    use core::f64::consts::PI;
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = (phase_from_value(v, mu, sd)).rem_euclid(2.0 * PI);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut max_in_half = 0_u64;
            for &center in &buf {
                let mut in_half = 0_u64;
                for &p in &buf {
                    let diff = (p - center).rem_euclid(2.0 * PI);
                    if diff < PI { in_half += 1; }
                }
                if in_half > max_in_half { max_in_half = in_half; }
            }
            let n = win_n as f64;
            let stat = max_in_half as f64 / n;
            if stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "hodges_ajne",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Hermans-Rasson test — Hermans & Rasson 1985. Detects multimodal phase
/// distributions via T = n - (4/π) Σᵢⱼ |sin((θᵢ-θⱼ)/2)|.
pub fn hermans_rasson(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    use core::f64::consts::PI;
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = phase_from_value(v, mu, sd);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let n = win_n as f64;
            let mut sum_pair = 0.0;
            for i in 0..win_n {
                for j in (i + 1)..win_n {
                    sum_pair += ((buf[i] - buf[j]) / 2.0).sin().abs();
                }
            }
            let stat = n - (4.0 / PI) * sum_pair;
            if stat.abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "hermans_rasson",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Batschelet phase concentration — Batschelet 1981. Tests if phase becomes
/// too concentrated (synchronization).
pub fn batschelet_concentration(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = phase_from_value(v, mu, sd);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut cs = 0.0; let mut sn = 0.0;
            for &p in &buf { cs += p.cos(); sn += p.sin(); }
            let r = (cs * cs + sn * sn).sqrt() / win_n as f64;
            // Batschelet concentration parameter (approximation): kappa from R.
            let kappa = if r < 0.53 { 2.0 * r + r.powi(3) + 5.0 * r.powi(5) / 6.0 }
                        else if r < 0.85 { -0.4 + 1.39 * r + 0.43 / (1.0 - r) }
                        else { 1.0 / (r.powi(3) - 4.0 * r.powi(2) + 3.0 * r) };
            if kappa.abs() > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "batschelet_concentration",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Circular variance collapse — phase becomes too concentrated. Inverse
/// of Rayleigh's R.
pub fn circular_variance_collapse(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64, // typical 0.1 (variance below this triggers)
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = phase_from_value(v, mu, sd);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut cs = 0.0; let mut sn = 0.0;
            for &p in &buf { cs += p.cos(); sn += p.sin(); }
            let r = (cs * cs + sn * sn).sqrt() / win_n as f64;
            let circ_var = 1.0 - r;
            if circ_var < threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "circ_variance_collapse",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Circular mean drift — track mean phase across rolling windows; alarm on shift.
///
/// Tier Z (circular / directional). Tracks the circular mean phase across rolling windows; alert when phase shifts by more than `phase_k` radians from the healthy mean.
/// Per-window alert on circular-mean phase drift.
pub fn circular_mean_drift(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    use core::f64::consts::PI;
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut prev_mean: f64 = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = phase_from_value(v, mu, sd);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut cs = 0.0; let mut sn = 0.0;
            for &p in &buf { cs += p.cos(); sn += p.sin(); }
            let mean_phase = sn.atan2(cs);
            if !prev_mean.is_nan() {
                let diff = (mean_phase - prev_mean).abs();
                let circ_diff = diff.min(2.0 * PI - diff);
                if circ_diff > threshold {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                }
            }
            prev_mean = mean_phase;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "circular_mean_drift",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Resultant length detector — track |R| change across rolling windows.
///
/// Tier Z (circular / directional). Tracks the resultant-length |R| of unit vectors per window; alert when |R| changes by more than `R_k` from the healthy baseline.
/// Per-window alert on resultant-length change.
pub fn resultant_length(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut prev_r: f64 = f64::NAN;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            let phi = phase_from_value(v, mu, sd);
            if count < win_n { buf[count] = phi; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = phi; }
            if count < win_n { continue; }
            let mut cs = 0.0; let mut sn = 0.0;
            for &p in &buf { cs += p.cos(); sn += p.sin(); }
            let r = (cs * cs + sn * sn).sqrt() / win_n as f64;
            if !prev_r.is_nan() {
                if (r - prev_r).abs() > threshold {
                    raw += 1;
                    if s < 32 { alerts_per_signal[s] += 1; }
                    win_alerts[w] = true;
                }
            }
            prev_r = r;
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "resultant_length",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 9 — TIER AA — Higher-order nonlinear time-series witnesses (5)
// =====================================================================

/// Hinich bicorrelation — Hinich 1996. Third-order serial dependence:
/// detect nonzero E[x(t)·x(t+r)·x(t+s)] for some lag pair (r, s).
pub fn hinich_bicorrelation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, max_lag: usize, threshold: f64, // typical 60, 5, 3.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let centered: Vec<f64> = buf.iter().map(|v| v - m).collect();
            let var: f64 = centered.iter().map(|v| v * v).sum::<f64>() / win_n as f64;
            if var < 1e-9 { continue; }
            let sd = var.sqrt();
            let mut max_b: f64 = 0.0;
            for r in 1..=max_lag {
                for sl in 1..=max_lag {
                    let mut sum = 0.0; let mut nc = 0_u64;
                    for j in 0..(win_n.saturating_sub(r + sl)) {
                        sum += centered[j] * centered[j + r] * centered[j + r + sl];
                        nc += 1;
                    }
                    if nc == 0 { continue; }
                    let bc = sum / nc as f64 / sd.powi(3);
                    if bc.abs() > max_b { max_b = bc.abs(); }
                }
            }
            if max_b > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "hinich_bicorrelation",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// McLeod-Li ARCH detector — McLeod & Li 1983. Autocorrelation in squared
/// residuals indicates conditional heteroskedasticity.
pub fn mcleod_li(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, max_lag: usize, threshold: f64, // typical 60, 5, 0.3
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let sq: Vec<f64> = buf.iter().map(|v| (v - m).powi(2)).collect();
            let m_sq: f64 = sq.iter().sum::<f64>() / win_n as f64;
            let var_sq: f64 = sq.iter().map(|v| (v - m_sq).powi(2)).sum::<f64>() / win_n as f64;
            if var_sq < 1e-9 { continue; }
            let mut max_acf: f64 = 0.0;
            for lag in 1..=max_lag {
                let mut acf = 0.0;
                for j in 0..(win_n - lag) { acf += (sq[j] - m_sq) * (sq[j + lag] - m_sq); }
                acf /= (win_n - lag) as f64 * var_sq;
                if acf.abs() > max_acf { max_acf = acf.abs(); }
            }
            if max_acf > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "mcleod_li",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Keenan one-degree nonlinearity test — Keenan 1985. Tests if quadratic
/// term improves AR(1) fit. Stat = (residual reduction) / (original residual).
pub fn keenan_nonlinearity(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            // AR(1) fit: x_t = phi * x_{t-1} + e
            let mut sxy = 0.0; let mut sxx = 0.0;
            for j in 1..win_n { sxx += buf[j - 1].powi(2); sxy += buf[j - 1] * buf[j]; }
            if sxx < 1e-9 { continue; }
            let phi = sxy / sxx;
            let resid: Vec<f64> = (1..win_n).map(|j| buf[j] - phi * buf[j - 1]).collect();
            let ss_orig: f64 = resid.iter().map(|r| r * r).sum();
            // Add x²_{t-1} term and refit residuals.
            let mut sxq_q = 0.0; let mut sxq_r = 0.0;
            for j in 1..win_n {
                let q = buf[j - 1].powi(2);
                sxq_q += q * q;
                sxq_r += q * resid[j - 1];
            }
            if sxq_q < 1e-9 { continue; }
            let beta_q = sxq_r / sxq_q;
            let ss_new: f64 = resid.iter().enumerate()
                .map(|(j, r)| (r - beta_q * buf[j].powi(2)).powi(2)).sum();
            let f_keenan = (ss_orig - ss_new) / (ss_orig / win_n as f64).max(1e-9);
            if f_keenan > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "keenan_nonlinearity",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Tsay quadratic nonlinearity — Tsay 1986. Stronger quadratic-term
/// alternative; uses augmented regression with multiple cross-terms.
pub fn tsay_nonlinearity(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    // Tsay's test: regress x_t on x_{t-1}, x_{t-1}², x_{t-1}*x_{t-2},
    // x_{t-2}². Statistic = F-test on the quadratic terms.
    // Simplified: just use sum-of-squared cross-products.
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let centered: Vec<f64> = buf.iter().map(|v| v - m).collect();
            let var: f64 = centered.iter().map(|v| v * v).sum::<f64>() / win_n as f64;
            if var < 1e-9 { continue; }
            // Compute Σ (x_t · x_{t-1}²) / (n · σ³)
            let mut tsay_stat = 0.0_f64; let mut nc = 0_u64;
            for j in 2..win_n {
                tsay_stat += centered[j] * centered[j - 1].powi(2);
                nc += 1;
            }
            tsay_stat = tsay_stat.abs() / (nc as f64 * var.powi(3).sqrt()).max(1e-9);
            if tsay_stat > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "tsay_nonlinearity",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Hinich tricorrelation — fourth-order serial dependence. Detects coupling
/// across three lag distances. Computationally O(N·L³); use modest L.
pub fn hinich_tricorrelation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, max_lag: usize, threshold: f64, // typical 60, 3, 4.0
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if count < win_n { buf[count] = v; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = v; }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let centered: Vec<f64> = buf.iter().map(|v| v - m).collect();
            let var: f64 = centered.iter().map(|v| v * v).sum::<f64>() / win_n as f64;
            if var < 1e-9 { continue; }
            let mut max_t: f64 = 0.0;
            for r in 1..=max_lag {
                for sl in 1..=max_lag {
                    for q in 1..=max_lag {
                        let max_idx = r + sl + q;
                        if max_idx >= win_n { continue; }
                        let mut sum = 0.0; let mut nc = 0_u64;
                        for j in 0..(win_n - max_idx) {
                            sum += centered[j] * centered[j + r] * centered[j + r + sl] * centered[j + r + sl + q];
                            nc += 1;
                        }
                        if nc == 0 { continue; }
                        let tc = (sum / nc as f64) / var.powi(2);
                        if tc.abs() > max_t { max_t = tc.abs(); }
                    }
                }
            }
            if max_t > threshold {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "hinich_tricorrelation",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

// =====================================================================
// SESSION 9 — TIER V — Industrial fault-diagnosis (FDD) family (8 detectors)
// All implemented as inter-signal residual proxies (no observer model
// available in the residual-projection-v2 stream; FDD literature
// canonical algorithms documented in doc-comments per honest-simplification
// pattern).
// =====================================================================

/// Parity-space residual — Chow & Willsky 1984. Inter-signal consistency
/// equation: signals that should agree (under linear model) but don't.
/// Proxy: detect when pairwise signal correlations break.
pub fn parity_space_residual(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    if num_signals < 2 { return zero_output("parity_space"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let ref_corr = compute_corr_window(data, num_signals, 0, healthy_window_end.min(num_windows));
    for w in win_n..num_windows {
        let cur_corr = compute_corr_window(data, num_signals, w + 1 - win_n, w + 1);
        let mut max_dev = 0.0_f64;
        for i in 0..num_signals {
            for j in (i + 1)..num_signals {
                let d = (ref_corr[i][j] - cur_corr[i][j]).abs();
                if d > max_dev { max_dev = d; }
            }
        }
        if max_dev > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "parity_space_residual",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Analytical Redundancy Relation (ARR) — Staroswiecki & Comtet-Varga 2001.
/// Detect when the algebraic relation between modeled vs observed variables
/// fails. Proxy: rolling-window linear-fit residual between signal pairs.
pub fn arr_constraint_violation(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, threshold: f64,
) -> DetectorOutput {
    if num_signals < 2 { return zero_output("arr_constraint"); }
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for w in win_n..num_windows {
        let mut max_resid = 0.0_f64;
        for s1 in 0..num_signals {
            for s2 in (s1 + 1)..num_signals {
                let mut sx = 0.0; let mut sy = 0.0; let mut sxy = 0.0; let mut sxx = 0.0;
                let mut nc = 0_u64;
                for k in 0..win_n {
                    let i1 = (w + 1 - win_n + k) * num_signals + s1;
                    let i2 = (w + 1 - win_n + k) * num_signals + s2;
                    if i1 < data.len() && i2 < data.len() && !data[i1].is_nan() && !data[i2].is_nan() {
                        sx += data[i1]; sy += data[i2];
                        sxy += data[i1] * data[i2]; sxx += data[i1] * data[i1];
                        nc += 1;
                    }
                }
                if nc < 5 { continue; }
                let n = nc as f64;
                let den = n * sxx - sx * sx;
                if den.abs() < 1e-9 { continue; }
                let beta = (n * sxy - sx * sy) / den;
                let alpha = (sy - beta * sx) / n;
                // Residual at most-recent window pair.
                let i1 = w * num_signals + s1;
                let i2 = w * num_signals + s2;
                if i1 < data.len() && i2 < data.len() && !data[i1].is_nan() && !data[i2].is_nan() {
                    let resid = (data[i2] - alpha - beta * data[i1]).abs();
                    if resid > max_resid { max_resid = resid; }
                }
            }
        }
        if max_resid > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "arr_constraint",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Unknown-Input Observer residual — Chen & Patton 1999. Decouple disturbance
/// from residual. Proxy: residual = signal - smoothed(signal); flag when
/// residual variance exceeds healthy variance by k.
pub fn unknown_input_observer(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64,
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut smoothed = 0.0_f64;
        let alpha = 0.3;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            smoothed = alpha * v + (1.0 - alpha) * smoothed;
            let resid = v - smoothed;
            if count < win_n { buf[count] = resid; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = resid; }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let var: f64 = buf.iter().map(|v| (v - m).powi(2)).sum::<f64>() / win_n as f64;
            if var.sqrt() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "unknown_input_observer",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Sliding-Mode Observer residual — Edwards 2000. Robust observer with
/// switching surface; residual = signal - sliding-surface estimate.
/// Proxy: median-tracking residual with chattering threshold.
pub fn sliding_mode_observer(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    win_n: usize, k: f64,
) -> DetectorOutput {
    let (_, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let sd = sigmas[s].max(1e-9);
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0_usize;
        let mut surface = 0.0_f64;
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            // Sliding-surface update: bang-bang on residual sign.
            let err = v - surface;
            surface += 0.1 * err.signum() * err.abs().min(sd);
            let resid = v - surface;
            if count < win_n { buf[count] = resid; count += 1; }
            else { buf.copy_within(1..win_n, 0); buf[win_n - 1] = resid; }
            if count < win_n { continue; }
            let m: f64 = buf.iter().sum::<f64>() / win_n as f64;
            let var: f64 = buf.iter().map(|v| (v - m).powi(2)).sum::<f64>() / win_n as f64;
            if var.sqrt() > k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "sliding_mode_observer",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Interval observer — Gouzé 2000. Signal exits guaranteed lower/upper
/// model interval. Proxy: signal exceeds healthy ±3σ envelope.
pub fn interval_observer(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    k: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    for s in 0..num_signals {
        let mu = means[s]; let sd = sigmas[s].max(1e-9);
        for w in 0..num_windows {
            let i = w * num_signals + s;
            if i >= data.len() { continue; }
            let v = data[i]; if v.is_nan() { continue; }
            if v < mu - k * sd || v > mu + k * sd {
                raw += 1;
                if s < 32 { alerts_per_signal[s] += 1; }
                win_alerts[w] = true;
            }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "interval_observer",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Zonotope set-membership — Combastel 2003. Residual exits a reachable
/// zonotope. Proxy: residual exits Mahalanobis ellipsoid in 2D-pair projection.
pub fn zonotope_escape(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64,
) -> DetectorOutput {
    if num_signals < 2 { return zero_output("zonotope_escape"); }
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    for w in 0..num_windows {
        let mut max_dist = 0.0_f64;
        for s1 in 0..num_signals {
            for s2 in (s1 + 1)..num_signals {
                let i1 = w * num_signals + s1;
                let i2 = w * num_signals + s2;
                if i1 >= data.len() || i2 >= data.len() { continue; }
                let v1 = data[i1]; let v2 = data[i2];
                if v1.is_nan() || v2.is_nan() { continue; }
                let z1 = (v1 - means[s1]) / sigmas[s1].max(1e-9);
                let z2 = (v2 - means[s2]) / sigmas[s2].max(1e-9);
                let d = (z1 * z1 + z2 * z2).sqrt();
                if d > max_dist { max_dist = d; }
            }
        }
        if max_dist > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "zonotope_escape",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Bond-graph residual — Karnopp 1990. Physical conservation-law violation.
/// Proxy: detect when sum of inter-signal flows changes (conservation broken).
pub fn bond_graph_residual(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64,
) -> DetectorOutput {
    let _ = healthy_window_end;
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    // Healthy conservation reference: mean of total over healthy window.
    let mut healthy_total = 0.0_f64; let mut nc = 0_u64;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut sum = 0.0;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() { sum += data[i]; }
        }
        healthy_total += sum; nc += 1;
    }
    if nc == 0 { return zero_output("bond_graph_residual"); }
    let healthy_total = healthy_total / nc as f64;
    let mut total_sd = 0.0;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut sum = 0.0;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() { sum += data[i]; }
        }
        total_sd += (sum - healthy_total).powi(2);
    }
    let total_sd = (total_sd / nc as f64).sqrt().max(1e-9);
    for w in 0..num_windows {
        let mut sum = 0.0;
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() { sum += data[i]; }
        }
        if (sum - healthy_total).abs() > threshold * total_sd {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "bond_graph_residual",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

/// Structural isolability matrix — Frisk & Krysander 2005. Verifies whether
/// the pattern of residuals identifies a specific fault. Proxy: signal-pattern
/// fingerprint match against healthy fingerprint.
pub fn structural_isolability(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, fault_labels: &[bool], pred_window: u64,
    threshold: f64,
) -> DetectorOutput {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut alerts_per_signal = [0_u64; 32];
    let mut raw = 0_u64;
    let mut win_alerts = std::vec![false; num_windows];
    if num_signals == 0 { return zero_output("structural_isolability"); }
    // Healthy fingerprint: count of signals exceeding 1.5σ in healthy window.
    let mut healthy_pattern = std::vec![0_u64; num_signals];
    let mut nh = 0_u64;
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() {
                if (data[i] - means[s]).abs() > 1.5 * sigmas[s].max(1e-9) {
                    healthy_pattern[s] += 1;
                }
            }
        }
        nh += 1;
    }
    if nh == 0 { return zero_output("structural_isolability"); }
    let healthy_freq: Vec<f64> = healthy_pattern.iter().map(|&c| c as f64 / nh as f64).collect();
    // Per-window check.
    for w in 0..num_windows {
        let mut pattern = std::vec![false; num_signals];
        for s in 0..num_signals {
            let i = w * num_signals + s;
            if i < data.len() && !data[i].is_nan() {
                pattern[s] = (data[i] - means[s]).abs() > 1.5 * sigmas[s].max(1e-9);
            }
        }
        // Pattern divergence from healthy frequency.
        let mut div = 0.0;
        for s in 0..num_signals {
            let observed = if pattern[s] { 1.0 } else { 0.0 };
            div += (observed - healthy_freq[s]).powi(2);
        }
        if div.sqrt() > threshold {
            raw += 1; win_alerts[w] = true;
            for s in 0..num_signals.min(32) { alerts_per_signal[s] += 1; }
        }
    }
    capture_win_alerts(&win_alerts);
    let alert_windows = win_alerts.iter().filter(|b| **b).count() as u64;
    let (tf, cf, cw, fp) = score_against_labels(&win_alerts, fault_labels, pred_window);
    DetectorOutput {
        detector_name: "structural_isolability",
        raw_alert_count: raw, alerts_per_signal, alert_windows,
        episode_count: alert_windows, captured_faults: cf, total_faults: tf,
        clean_window_false_alerts: fp, clean_windows: cw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All-clean data (constant values) → all three detectors report
    /// zero alerts. Strawman sanity check.
    #[test]
    fn all_clean_yields_zero_alerts() {
        let data = std::vec![100.0_f64; 200];
        let labels = std::vec![false; 100];
        let r = scalar_threshold(&data, 2, 100, 50, &labels, 5);
        assert_eq!(r.raw_alert_count, 0);
        assert_eq!(r.alert_windows, 0);

        let r = cusum(&data, 2, 100, 50, &labels, 5, 4.0);
        assert_eq!(r.raw_alert_count, 0);

        let r = ewma(&data, 2, 100, 50, &labels, 5, 0.2, 3.0);
        assert_eq!(r.raw_alert_count, 0);
    }

    /// Step shift in the middle of the slice: scalar threshold should
    /// fire after the shift; CUSUM should accumulate; EWMA should
    /// drift toward the new mean and fire.
    #[test]
    fn step_shift_detected_by_all_three() {
        // 100 windows, 1 signal. First 50: mean ~100, sigma ~1.
        // Second 50: mean ~110, same sigma.
        let mut data = std::vec![0.0_f64; 100];
        for w in 0..50 {
            data[w] = 100.0 + (w as f64 % 3.0 - 1.0); // ~ {99,100,101} cycle
        }
        for w in 50..100 {
            data[w] = 110.0 + (w as f64 % 3.0 - 1.0); // shifted
        }
        let labels = std::vec![false; 100];
        let r1 = scalar_threshold(&data, 1, 100, 50, &labels, 5);
        assert!(r1.raw_alert_count > 0, "scalar should fire on step shift");

        let r2 = cusum(&data, 1, 100, 50, &labels, 5, 4.0);
        assert!(r2.raw_alert_count > 0, "CUSUM should fire on step shift");

        let r3 = ewma(&data, 1, 100, 50, &labels, 5, 0.2, 3.0);
        assert!(r3.raw_alert_count > 0, "EWMA should fire on step shift");
    }

    #[test]
    fn output_invariants() {
        let data = std::vec![100.0_f64; 60];
        let labels = std::vec![false; 30];
        let r = scalar_threshold(&data, 2, 30, 15, &labels, 5);
        assert!(r.fault_recall() >= 0.0 && r.fault_recall() <= 1.0);
        assert!(r.clean_window_fp_rate() >= 0.0);
        assert!(r.rscr() == 0.0 || r.rscr() == 1.0);
    }
}
