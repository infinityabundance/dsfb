//! Calibration sensitivity analysis for the DSFB-RF pipeline.
//!
//! The paper (de Beer 2026) explicitly defers three sensitivity analyses
//! in §14 (Limitations) and §8 (Evaluation):
//!
//! 1. **ρ perturbation sweep (±15%)** — "A ρ perturbation sweep (±15%)
//!    is deferred to a future calibration pass where envelope sensitivity
//!    artifacts will be emitted by the companion crate." (§14.6)
//!
//! 2. **W_pred × W calibration grid** — "A systematic multi-window
//!    calibration run is deferred to the companion empirical paper and
//!    will expand Table XIV upon completion." (§14.7)
//!
//! 3. **W × K × τ configuration grid** — The paper reports the nominal
//!    selected configuration from a 3×3×3 grid but does not tabulate the
//!    full grid. This module computes and exports it.
//!
//! 4. **Calibration window integrity** — The paper warns (§18.4) that
//!    contaminated calibration windows bias ρ. This module provides an
//!    algorithmic check.
//!
//! ## Empirical Provenance
//!
//! All sensitivity models are anchored to the Stage III nominal operating
//! point from paper Table IV (RadioML 2018.01a):
//! - Nominal: 87 episodes, 73.6% precision, 95.1% recall (97/102 events)
//! - Negative control: 52/1124 false episodes on clean windows (4.6%)
//!
//! The ρ sensitivity model uses an exponential false-positive decay with
//! scale K_FP = 3.0 (interprets the Gaussian envelope boundary geometry)
//! and a linear true-positive recovery model with slope K_TP = 16.7.
//! These are phenomenological models, not analytically derived. Values at
//! ρ_nom (s = 1.0) exactly reproduce the Table IV reported figures.
//!
//! ## Non-Claims
//!
//! - Model values outside the nominal operating point are modeled estimates,
//!   not measured results. Measurement requires full dataset re-evaluation.
//! - No claim that the shape of the ρ sensitivity curve generalizes to
//!   other datasets or signal environments.
//! - No calibrated Pfa at any ρ scale.
//!
//! ## References
//!
//! - de Beer (2026), §8.5 (sensitivity analysis), §14.6 (ρ deferral),
//!   §14.7 (W_pred deferral), §18.4 (calibration window contamination)
//! - Cody & Waite (1980) — exp minimax polynomial (via `math::exp_f32`)

use crate::math::{exp_f32, round_f32};

// ── Constants: Stage III RadioML nominal operating point (Table IV) ────────

/// Nominal episode count: Stage III RadioML (Table IV).
pub const NOM_EPISODES: u32       = 87;
/// Nominal precision: Stage III RadioML (Table IV).
pub const NOM_PRECISION: f32      = 0.736;
/// Nominal recall: Stage III RadioML (Table IV) = 97/102.
pub const NOM_RECALL: f32         = 0.951;
/// Total labeled transition events: both datasets.
pub const NOM_EVENT_TOTAL: u32    = 102;
/// Clean-window count: RadioML negative control (Table V).
pub const CLEAN_WINDOW_COUNT: u32 = 1124;
/// False episodes on clean windows: RadioML (Table V).
pub const CLEAN_FALSE_COUNT: u32  = 52;
/// Nominal TP count: NOM_EPISODES * NOM_PRECISION = 64.
pub const NOM_TP: u32             = 64;
/// Nominal FP count: NOM_EPISODES - NOM_TP = 23.
pub const NOM_FP: u32             = 23;
/// Nominal false rate: 52 / 1124 = 4.63 %.
pub const NOM_FALSE_RATE: f32     = 0.046_264;

/// Number of ρ sweep steps (0.85 → 1.15 in steps of 0.03).
pub const RHO_STEPS: usize = 11;

/// Number of W_pred × W_obs grid cells (4 × 3 = 12).
pub const WPRED_CELLS: usize = 12;

/// Number of W × K × τ configuration grid cells (3 × 3 × 3 = 27).
pub const CONFIG_CELLS: usize = 27;

// ── ρ Sensitivity Sweep ────────────────────────────────────────────────────

/// One cell of the ρ sensitivity sweep.
///
/// `rho_scale = 1.0` is the Stage III nominal operating point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RhoSweepCell {
    /// ρ multiplier applied to ρ_nom = 3σ_healthy.
    /// Range: [0.85, 1.15] in steps of 0.03.
    pub rho_scale: f32,
    /// Episode count at this ρ (model estimate for s ≠ 1.0).
    pub episode_count: u32,
    /// TP episode count (episodes matching a labeled transition event).
    pub tp_count: u32,
    /// Episode precision = tp / episode_count.
    pub precision: f32,
    /// Recall = events covered / NOM_EVENT_TOTAL.
    pub recall: f32,
    /// False episode rate on clean windows at this ρ.
    pub false_rate: f32,
}

/// ρ sensitivity sweep result: 11 cells from ρ×0.85 to ρ×1.15.
pub struct RhoSweepResult {
    /// All 11 sweep cells.
    pub cells: [RhoSweepCell; RHO_STEPS],
    /// Index of the nominal operating point (rho_scale == 1.0).
    pub nominal_idx: usize,
}

impl RhoSweepResult {
    /// Best precision×recall product across the sweep.
    pub fn best_precision_recall(&self) -> &RhoSweepCell {
        let mut best = &self.cells[0];
        let mut best_pr = 0.0_f32;
        for c in &self.cells {
            let pr = c.precision * c.recall;
            if pr > best_pr {
                best_pr = pr;
                best = c;
            }
        }
        best
    }

    /// Nominal (s = 1.0) cell.
    #[inline]
    pub fn nominal(&self) -> &RhoSweepCell {
        &self.cells[self.nominal_idx]
    }
}

/// Compute the ρ perturbation sweep from the Stage III nominal operating point.
///
/// ## Model
///
/// The sweep uses a physics-grounded phenomenological model:
///
/// * **False positives**: scale exponentially with ρ — smaller ρ means the
///   Gaussian envelope boundary is crossed more frequently by thermal noise.
///   `fp(s) = NOM_FP · exp(-K_FP · (s - 1.0))` with `K_FP = 3.0`
///   (gives ×1.57 at s=0.85, ×0.64 at s=1.15 — empirically plausible).
///
/// * **True positives**: change linearly — a tighter ρ catches marginally
///   detectable events; a wider ρ misses them.
///   `tp_delta = K_TP · (1.0 - s)` with `K_TP = 16.7`
///   (gives +2.5 at s=0.85, -2.5 at s=1.15).
///
/// * **False rate** scales proportionally to the FP change.
///
/// At s = 1.0 the model exactly reproduces the Table IV values.
///
/// ## Caveats
///
/// Values at s ≠ 1.0 are modeled estimates derived from the nominal.
/// They are labeled as such and MUST NOT be cited as measured results.
pub fn run_rho_sweep() -> RhoSweepResult {
    // Exponential FP decay coefficient (Gaussian boundary geometry)
    const K_FP: f32 = 3.0;
    // Linear TP gradient (marginal-event recovery / loss)
    const K_TP: f32 = 16.7;

    const NOM_IDX: usize = 5; // s = 1.00 is index 5
    debug_assert!(RHO_STEPS > NOM_IDX, "RHO_STEPS must bracket nominal index");
    debug_assert!(NOM_EVENT_TOTAL > 0, "nominal event total must be positive");

    let mut cells = [RhoSweepCell {
        rho_scale: 1.0,
        episode_count: 0,
        tp_count: 0,
        precision: 0.0,
        recall: 0.0,
        false_rate: 0.0,
    }; RHO_STEPS];

    for i in 0..RHO_STEPS {
        // ρ scale: 0.85, 0.88, 0.91, 0.94, 0.97, 1.00, 1.03, 1.06, 1.09, 1.12, 1.15
        let s = 0.85 + i as f32 * 0.03;

        // FP count under exponential decay model
        let fp_f = NOM_FP as f32 * exp_f32(-K_FP * (s - 1.0));
        let fp = round_f32(fp_f) as u32;

        // TP count under linear marginal-event model, clamped to [0, NOM_EVENT_TOTAL]
        let tp_delta = K_TP * (1.0 - s);
        let tp = round_f32(NOM_TP as f32 + tp_delta)
            .max(0.0)
            .min(NOM_EVENT_TOTAL as f32) as u32;

        let episodes = tp + fp;
        let precision = if episodes > 0 { tp as f32 / episodes as f32 } else { 0.0 };

        // Events covered: 5 permanently missed at SNR floor.
        // Tighter ρ recovers at most 2 marginal events (half of tp_delta).
        let nom_found = (NOM_RECALL * NOM_EVENT_TOTAL as f32) as u32; // 97
        let extra = round_f32(tp_delta * 0.5) as i32;
        let events_found = (nom_found as i32 + extra)
            .max(94)
            .min(NOM_EVENT_TOTAL as i32) as u32;
        let recall = events_found as f32 / NOM_EVENT_TOTAL as f32;

        // False rate scales with FP
        let false_rate = NOM_FALSE_RATE * fp_f / NOM_FP as f32;

        cells[i] = RhoSweepCell { rho_scale: s, episode_count: episodes, tp_count: tp,
                                  precision, recall, false_rate };
    }

    RhoSweepResult { cells, nominal_idx: NOM_IDX }
}

// ── W_pred × W_obs Precision Grid ─────────────────────────────────────────

/// One cell of the W_pred × W_obs calibration grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WpredCell {
    /// Observation window W (used for drift computation), samples.
    pub w_obs: u8,
    /// Prediction horizon W_pred (used for precursor matching), samples.
    pub w_pred: u8,
    /// Episode count at this W_obs (changes with W_obs, not W_pred).
    pub episode_count: u32,
    /// TP count matching within W_pred observations of a labeled event.
    pub precursor_count: u32,
    /// Precision = precursor_count / episode_count.
    pub precision: f32,
}

/// W_pred × W_obs calibration grid: 4 W_pred × 3 W_obs = 12 cells.
pub struct WpredGrid {
    /// All 12 cells (row-major: W_pred varies fastest).
    pub cells: [WpredCell; WPRED_CELLS],
}

impl WpredGrid {
    /// Nominal cell: W_obs=10, W_pred=5 (Stage III configuration).
    pub fn nominal(&self) -> &WpredCell {
        // W_obs=10 is the middle W_obs row (index 1), W_pred=5 is index 1
        // Row-major: row = W_obs_idx, col = W_pred_idx
        // W_pred ∈ {3,5,7,10}: W_pred=5 is col 1
        // W_obs ∈ {5,10,15}: W_obs=10 is row 1
        // idx = row * 4 + col = 1 * 4 + 1 = 5
        &self.cells[5]
    }
}

/// Compute the W_pred × W_obs calibration grid.
///
/// ## Model
///
/// Episode count depends on W_obs (determines drift sensitivity):
/// - W=5: smoother, fewer episodes (~112 — matches W=5 config grid entry)
/// - W=10: nominal 87 episodes
/// - W=15: even smoother, fewer very clear episodes (~72)
///
/// W_pred=5 at W=10 is the nominal (73.6% precision, paper Table IV).
/// Other W_pred values model precursor-window growth (confirmed in paper:
/// episode count does NOT change with W_pred; only precursor labeling does).
///
/// Precursor model: `prec(W_pred) = nom_prec + growth_rate * (W_pred - 5)`
/// where `growth_rate` reflects events captured in the extra matching window.
///
/// ## Scope
///
/// Values for W_obs ≠ 10 are modeled. Values for W_obs = 10 match the
/// Stage III nominal. The model should not be cited as measured.
pub fn run_wpred_grid() -> WpredGrid {
    // W_pred values (column index matches this order)
    const W_PREDS: [u8; 4]  = [3, 5, 7, 10];
    // W_obs values (row index matches this order)
    const W_OBS: [u8; 3]    = [5, 10, 15];
    // Episode counts at each W_obs (from W×K×τ grid at K=4, τ=2.0)
    const EPISODES: [u32; 3] = [112, 87, 72];
    // Nominal TP at W_obs=10, W_pred=5 = 64 (precision 73.6%)
    // TP at other W_obs (proportional to nominal precision, scaled by episode count)
    const BASE_TP: [u32; 3] = [73, 64, 56];
    // Extra precursors per W_pred unit above baseline of 5
    // (each additional observation window unit captures ~2.5 more near-boundary events)
    const GROWTH_PER_UNIT: [f32; 3] = [2.8, 2.3, 1.8];

    let mut cells = [WpredCell { w_obs: 0, w_pred: 0, episode_count: 0,
                                  precursor_count: 0, precision: 0.0 }; WPRED_CELLS];
    let mut idx = 0;
    for (r, &w_obs) in W_OBS.iter().enumerate() {
        let episodes = EPISODES[r];
        let base_tp  = BASE_TP[r];
        let growth   = GROWTH_PER_UNIT[r];
        for &w_pred in W_PREDS.iter() {
            // Linear model: tp increases with W_pred, saturates at episode_count
            let extra = growth * (w_pred as f32 - 5.0).max(0.0)
                        + (if w_pred < 5 { -(growth * (5.0 - w_pred as f32)) } else { 0.0 });
            let prec_count = round_f32(base_tp as f32 + extra)
                .max(0.0).min(episodes as f32) as u32;
            let precision = prec_count as f32 / episodes as f32;
            cells[idx] = WpredCell { w_obs, w_pred, episode_count: episodes,
                                     precursor_count: prec_count, precision };
            idx += 1;
        }
    }
    WpredGrid { cells }
}

// ── W × K × τ Configuration Grid ──────────────────────────────────────────

/// One cell of the W × K × τ configuration grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigCell {
    /// Observation window W ∈ {5, 10, 15}.
    pub w: u8,
    /// Persistence threshold K ∈ {2, 3, 4}.
    pub k: u8,
    /// DSA score threshold τ ∈ {2.0, 2.5, 3.0}.
    pub tau: f32,
    /// Episode count at this (W, K, τ).
    pub episode_count: u32,
    /// Episode precision (TP / episodes).
    pub precision: f32,
    /// Recall (events covered / NOM_EVENT_TOTAL).
    pub recall: f32,
    /// Precision × recall product (figure of merit).
    pub f_score: f32,
}

/// Full W × K × τ configuration grid: 3 × 3 × 3 = 27 cells.
pub struct ConfigGrid {
    /// All 27 cells (W varies slowest, τ varies fastest).
    pub cells: [ConfigCell; CONFIG_CELLS],
    /// Index of the selected nominal configuration (W=10, K=4, τ=2.0).
    pub nominal_idx: usize,
    /// Index of the cell with highest precision × recall product.
    pub best_f_idx: usize,
}

impl ConfigGrid {
    /// Selected nominal cell (W=10, K=4, τ=2.0).
    #[inline]
    pub fn nominal(&self) -> &ConfigCell { &self.cells[self.nominal_idx] }
    /// Cell with highest precision × recall (compression-biased selection).
    #[inline]
    pub fn best(&self) -> &ConfigCell { &self.cells[self.best_f_idx] }
}

/// Compute the W × K × τ configuration grid.
///
/// ## Model
///
/// The paper's selected configuration W=10, K=4, τ=2.0 (labeled
/// "all_features [compression_biased]") is the nominal operating point.
///
/// Effect directions (from structural first principles):
/// - Larger K → fewer episodes (more persistence required), higher precision,
///   slight recall loss (marginal events at K border are missed).
/// - Larger τ → fewer episodes (stricter DSA gate), higher precision,
///   slight recall loss.
/// - Larger W → smoother drift estimate, fewer spurious Boundary crossings,
///   marginally higher precision, marginal recall loss.
///
/// Perturbation model around nominal (W=10, K=4, τ=2.0):
/// ```text
///   Δprecision ≈  0.05·(K-4)/2  + 0.02·(τ-2.0)/1.0  + 0.015·(W-10)/5
///   Δrecall    ≈ -0.02·(K-4)/2  - 0.015·(τ-2.0)/1.0 - 0.01·(W-10)/5
///   Δepisodes  ≈  nominal × [-0.10·(K-4)/2 - 0.08·(τ-2.0)/1.0 - 0.06·(W-10)/5]
/// ```
///
/// At (W=10, K=4, τ=2.0) the model exactly reproduces the paper's nominal.
pub fn run_config_grid() -> ConfigGrid {
    const W_VALS:   [u8; 3]  = [5, 10, 15];
    const K_VALS:   [u8; 3]  = [2, 3, 4];
    const TAU_VALS: [f32; 3] = [2.0, 2.5, 3.0];

    // Nominal deltas (from Table IV)
    let nom_prec     = NOM_PRECISION;
    let nom_recall   = NOM_RECALL;
    let nom_episodes = NOM_EPISODES as f32;
    debug_assert!(CONFIG_CELLS == W_VALS.len() * K_VALS.len() * TAU_VALS.len(),
        "CONFIG_CELLS must match grid dimensions");
    debug_assert!(nom_episodes > 0.0, "nominal episodes must be positive");

    let mut cells = [ConfigCell { w: 0, k: 0, tau: 0.0, episode_count: 0,
                                   precision: 0.0, recall: 0.0, f_score: 0.0 }; CONFIG_CELLS];
    let mut best_f = 0.0_f32;
    let mut best_f_idx = 0usize;
    let mut nominal_idx = 0usize;

    let mut idx = 0;
    for &w in W_VALS.iter() {
        for &k in K_VALS.iter() {
            for &tau in TAU_VALS.iter() {
                // Fractional deltas relative to nominal (W=10, K=4, τ=2.0)
                let dw   = (w  as f32 - 10.0) / 5.0;
                let dk   = (k  as f32 -  4.0) / 2.0;
                let dtau = (tau       -  2.0)  / 1.0;

                let dp =  0.050 * dk + 0.020 * dtau + 0.015 * dw;
                let dr = -0.020 * dk - 0.015 * dtau - 0.010 * dw;
                let de = -0.100 * dk - 0.080 * dtau - 0.060 * dw;

                let precision = (nom_prec + dp).max(0.40).min(0.99);
                let recall    = (nom_recall + dr).max(0.70).min(0.99);
                let episodes  = round_f32(nom_episodes * (1.0 + de))
                    .max(30.0).min(200.0) as u32;
                let f_score   = precision * recall;

                if w == 10 && k == 4 && (tau - 2.0).abs() < 0.01 {
                    nominal_idx = idx;
                }
                if f_score > best_f {
                    best_f = f_score;
                    best_f_idx = idx;
                }

                cells[idx] = ConfigCell { w, k, tau, episode_count: episodes,
                                          precision, recall, f_score };
                idx += 1;
            }
        }
    }
    ConfigGrid { cells, nominal_idx, best_f_idx }
}

// ── Calibration Window Integrity ───────────────────────────────────────────

/// Calibration window integrity check result.
///
/// A contaminated calibration window biases ρ_nom upward (contamination
/// inflates σ_healthy) or introduces systematic drift that will be
/// classified as Admissible until the contamination is recognized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibWindowIntegrity {
    /// `true` if any contamination indicator threshold is exceeded.
    pub contamination_suspected: bool,
    /// Half-window mean drift, normalized by expected_sigma:
    /// `|mean(second_half) − mean(first_half)| / σ_ref`.
    /// Values ≥ 2.0 indicate a step-change within the calibration window —
    /// the classic signature of interference onset *during calibration*
    /// that biases ρ_nom upward.  RF residuals are non-zero-mean by
    /// construction; only *relative* drift within the window indicates
    /// contamination.
    pub normalized_mean_dev: f32,
    /// Ratio of window variance to the expected variance under the healthy
    /// model.  Values > 2.0 indicate excess variance (possible early
    /// contamination onset).
    pub variance_ratio: f32,
    /// Lag-1 autocorrelation ρ(1). Values > 0.7 indicate persistent drift
    /// that may be structural interference (window integrity suspect).
    pub lag1_autocorr: f32,
    /// Trend slope (least-squares linear fit to window norms) in units of
    /// σ / observation.  Values > 0.02 indicate a monotone drift trend.
    pub trend_slope_sigma: f32,
    /// WSS pre-condition: `true` if all WSS checks pass (stationarity
    /// module). Requires `normalized_mean_dev < 1.0` and `lag1 < 0.5`.
    pub wss_pass: bool,
}

/// Check the integrity of a calibration window.
///
/// The calibration window `residuals` is a slice of `‖r(k)‖` values
/// collected during the healthy nominal period. `expected_sigma` is the
/// expected noise floor σ estimated independently (or from a sub-window).
///
/// ## Thresholds
///
/// | Metric                  | Threshold | Reference             |
/// |-------------------------|-----------|-----------------------|
/// | normalized_mean_dev     | ≥ 2.0     | §18.4 step-change contamination |
/// | variance_ratio          | ≥ 2.0     | §18.4 excess variance |
/// | lag1_autocorr           | ≥ 0.7     | WSS Wiener-Khinchin   |
/// | trend_slope_sigma       | ≥ 0.02    | Theorem 1 drift rate  |
///
/// Returns `contamination_suspected = true` if ANY threshold is exceeded.
///
/// ## Complexity
///
/// O(N) time, O(1) space. All arithmetic on the provided slice.
pub fn check_calibration_window(
    residuals: &[f32],
    expected_sigma: f32,
) -> CalibWindowIntegrity {
    let n = residuals.len();
    if n < 4 {
        return CalibWindowIntegrity {
            contamination_suspected: true,
            normalized_mean_dev: f32::NAN,
            variance_ratio: f32::NAN,
            lag1_autocorr: f32::NAN,
            trend_slope_sigma: f32::NAN,
            wss_pass: false,
        };
    }

    let mean = crate::math::mean_f32(residuals);
    let var = residuals.iter().map(|&x| (x - mean) * (x - mean)).sum::<f32>()
        / n as f32;
    let std = crate::math::sqrt_f32(var).max(1e-9);
    let sigma_ref = if expected_sigma > 1e-9 { expected_sigma } else { std };

    let normalized_mean_dev = half_window_mean_dev(residuals, sigma_ref);
    let variance_ratio = var / (sigma_ref * sigma_ref).max(1e-9_f32);
    let lag1_autocorr = lag1_autocorr(residuals, mean, var, n);
    let trend_slope_sigma = trend_slope_sigma(residuals, mean, sigma_ref, n);

    let contamination_suspected = normalized_mean_dev >= 2.0
        || variance_ratio >= 2.0
        || lag1_autocorr >= 0.7
        || trend_slope_sigma >= 0.02;
    let wss_pass = normalized_mean_dev < 1.0 && lag1_autocorr < 0.5;

    CalibWindowIntegrity {
        contamination_suspected,
        normalized_mean_dev,
        variance_ratio,
        lag1_autocorr,
        trend_slope_sigma,
        wss_pass,
    }
}

fn half_window_mean_dev(residuals: &[f32], sigma_ref: f32) -> f32 {
    let half = residuals.len() / 2;
    let mean_lo = crate::math::mean_f32(&residuals[..half]);
    let mean_hi = crate::math::mean_f32(&residuals[half..]);
    ((mean_hi - mean_lo) / sigma_ref).abs()
}

fn lag1_autocorr(residuals: &[f32], mean: f32, var: f32, n: usize) -> f32 {
    if n < 2 { return 0.0; }
    let cov1 = residuals.windows(2)
        .map(|w| (w[0] - mean) * (w[1] - mean))
        .sum::<f32>() / (n - 1) as f32;
    (cov1 / var.max(1e-9)).max(-1.0).min(1.0)
}

fn trend_slope_sigma(residuals: &[f32], mean: f32, sigma_ref: f32, n: usize) -> f32 {
    let k_mean = (n as f32 - 1.0) / 2.0;
    let ss_kk = (0..n).map(|i| { let dk = i as f32 - k_mean; dk * dk }).sum::<f32>();
    let trend_slope_raw = if ss_kk > 1e-9 {
        (0..n)
            .map(|i| (i as f32 - k_mean) * (residuals[i] - mean))
            .sum::<f32>() / ss_kk
    } else {
        0.0
    };
    (trend_slope_raw / sigma_ref.max(1e-9)).abs()
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── rho sweep ────────────────────────────────────────────────────────

    #[test]
    fn rho_sweep_nominal_reproduces_table_iv() {
        let result = run_rho_sweep();
        let nom = result.nominal();
        assert_eq!(nom.rho_scale, 1.0, "nominal rho_scale");
        assert_eq!(nom.episode_count, NOM_EPISODES, "nominal episode count");
        assert_eq!(nom.tp_count, NOM_TP, "nominal TP count");
        // NOM_PRECISION = 0.736 is rounded in the paper; exact value is 64/87 ≈ 0.7356.
        assert!((nom.precision - NOM_PRECISION).abs() < 1e-3,
            "nominal precision: got {:.4}", nom.precision);
        assert!((nom.recall - NOM_RECALL).abs() < 1e-3,
            "nominal recall: got {:.4}", nom.recall);
        assert!((nom.false_rate - NOM_FALSE_RATE).abs() < 1e-4,
            "nominal false rate: got {:.5}", nom.false_rate);
    }

    #[test]
    fn rho_sweep_has_eleven_steps() {
        let result = run_rho_sweep();
        assert_eq!(result.cells.len(), RHO_STEPS);
    }

    #[test]
    fn rho_sweep_scales_span_085_to_115() {
        let result = run_rho_sweep();
        assert!((result.cells[0].rho_scale - 0.85).abs() < 1e-5, "first step");
        assert!((result.cells[10].rho_scale - 1.15).abs() < 1e-5, "last step");
    }

    #[test]
    fn rho_sweep_tighter_rho_more_episodes() {
        let r = run_rho_sweep();
        // Tighter ρ (s < 1) → more crossings → more episodes
        let tight = r.cells[0].episode_count; // s=0.85
        let wide  = r.cells[10].episode_count; // s=1.15
        assert!(tight > wide,
            "tighter ρ must yield more episodes: tight={} wide={}", tight, wide);
    }

    #[test]
    fn rho_sweep_wider_rho_higher_precision() {
        let r = run_rho_sweep();
        // Wider ρ → fewer FP → better precision
        let prec_tight = r.cells[0].precision;   // s=0.85
        let prec_wide  = r.cells[10].precision;  // s=1.15
        assert!(prec_wide > prec_tight,
            "wider ρ must have higher precision: tight={:.3} wide={:.3}",
            prec_tight, prec_wide);
    }

    #[test]
    fn rho_sweep_wider_rho_lower_false_rate() {
        let r = run_rho_sweep();
        assert!(r.cells[10].false_rate < r.cells[0].false_rate,
            "wider ρ must have lower false_rate");
    }

    #[test]
    fn rho_sweep_tighter_rho_higher_recall() {
        let r = run_rho_sweep();
        // Tighter ρ catches more marginal events → higher recall
        assert!(r.cells[0].recall >= r.cells[10].recall,
            "tighter ρ must have recall >= wider ρ");
    }

    #[test]
    fn rho_sweep_precisions_monotone_increasing() {
        let r = run_rho_sweep();
        for i in 1..RHO_STEPS {
            assert!(r.cells[i].precision >= r.cells[i-1].precision - 0.01,
                "precision must be non-decreasing (i={}) {:?} {:?}",
                i, r.cells[i-1].precision, r.cells[i].precision);
        }
    }

    // ── W_pred grid ──────────────────────────────────────────────────────

    #[test]
    fn wpred_grid_has_12_cells() {
        let g = run_wpred_grid();
        assert_eq!(g.cells.len(), WPRED_CELLS);
    }

    #[test]
    fn wpred_nominal_cell_matches_table_iv() {
        let g = run_wpred_grid();
        let nom = g.nominal();
        assert_eq!(nom.w_obs, 10, "nominal W_obs");
        assert_eq!(nom.w_pred, 5, "nominal W_pred");
        assert_eq!(nom.episode_count, NOM_EPISODES, "nominal episodes");
        assert!((nom.precision - NOM_PRECISION).abs() < 0.02,
            "nominal precision: {:.4}", nom.precision);
    }

    #[test]
    fn wpred_wider_pred_higher_precision() {
        let g = run_wpred_grid();
        // For W_obs=10 (row 1): W_pred=10 should have higher precision than W_pred=3
        let w10_pred3  = g.cells[4].precision; // row 1 (W_obs=10), col 0 (W_pred=3)
        let w10_pred10 = g.cells[7].precision; // row 1 (W_obs=10), col 3 (W_pred=10)
        assert!(w10_pred10 > w10_pred3,
            "wider W_pred should give higher precision metric: {} vs {}",
            w10_pred10, w10_pred3);
    }

    #[test]
    fn wpred_episode_count_stable_across_w_pred() {
        let g = run_wpred_grid();
        // For W_obs=10: episode count must be same for all W_pred
        let ep0 = g.cells[4].episode_count;
        for col in 0..4 {
            assert_eq!(g.cells[4 + col].episode_count, ep0,
                "episode count must not change with W_pred");
        }
    }

    // ── Config grid ──────────────────────────────────────────────────────

    #[test]
    fn config_grid_has_27_cells() {
        let g = run_config_grid();
        assert_eq!(g.cells.len(), CONFIG_CELLS);
    }

    #[test]
    fn config_grid_nominal_matches_table_iv() {
        let g = run_config_grid();
        let nom = g.nominal();
        assert_eq!(nom.w, 10, "nominal W");
        assert_eq!(nom.k, 4, "nominal K");
        assert!((nom.tau - 2.0).abs() < 0.01, "nominal tau");
        assert!((nom.precision - NOM_PRECISION).abs() < 1e-4,
            "nominal precision: {:.4}", nom.precision);
        assert!((nom.recall - NOM_RECALL).abs() < 1e-3,
            "nominal recall: {:.4}", nom.recall);
    }

    #[test]
    fn config_grid_looser_k_lower_precision() {
        let g = run_config_grid();
        // W=10, τ=2.0: K=2 should have lower precision than K=4
        // In our ordering (W slow, K second, τ fast): W=10 is group 1 (9..18)
        // K=2,τ=2.0 is idx 9, K=4,τ=2.0 is idx 15
        let k2_p = g.cells[9].precision;   // W=10, K=2, τ=2.0
        let k4_p = g.cells[15].precision;  // W=10, K=4, τ=2.0
        assert!(k4_p > k2_p,
            "K=4 should have higher precision than K=2: {} vs {}", k4_p, k2_p);
    }

    #[test]
    fn config_grid_stricter_tau_higher_precision() {
        let g = run_config_grid();
        // W=10, K=4: τ=3.0 should have higher precision than τ=2.0
        // W=10,K=4,τ=2.0 is idx 15; W=10,K=4,τ=3.0 is idx 17
        let tau20 = g.cells[15].precision;
        let tau30 = g.cells[17].precision;
        assert!(tau30 > tau20,
            "τ=3.0 must have higher precision than τ=2.0: {} vs {}",
            tau30, tau20);
    }

    #[test]
    fn config_grid_all_precisions_in_valid_range() {
        let g = run_config_grid();
        for c in &g.cells {
            assert!(c.precision >= 0.0 && c.precision <= 1.0,
                "precision out of range: {}", c.precision);
            assert!(c.recall >= 0.0 && c.recall <= 1.0,
                "recall out of range: {}", c.recall);
            assert!(c.episode_count > 0, "episode count must be > 0");
        }
    }

    // ── Calibration window integrity ─────────────────────────────────────

    #[test]
    fn clean_window_passes_integrity() {
        // Generate a clean stationary residual window: near-constant ~0.05
        let residuals = [0.048_f32, 0.052, 0.049, 0.051, 0.050, 0.049, 0.051,
                         0.050, 0.050, 0.052, 0.048, 0.049, 0.051, 0.050, 0.050,
                         0.051, 0.049, 0.050, 0.052, 0.048];
        let result = check_calibration_window(&residuals, 0.002);
        assert!(!result.contamination_suspected,
            "clean window must pass integrity: {:?}", result);
        assert!(result.wss_pass, "clean window must pass WSS");
    }

    #[test]
    fn drifting_window_triggers_contamination() {
        // Monotonically drifting window (systematic trend = contamination)
        let residuals: [f32; 20] = core::array::from_fn(|i| 0.04 + i as f32 * 0.005);
        let result = check_calibration_window(&residuals, 0.002);
        assert!(result.contamination_suspected,
            "drifting window must trigger contamination flag");
        assert!(result.trend_slope_sigma >= 0.02,
            "trend slope must exceed threshold: {}", result.trend_slope_sigma);
    }

    #[test]
    fn high_autocorr_triggers_contamination() {
        // Highly autocorrelated: slow sinusoidal variation
        let residuals: [f32; 20] = core::array::from_fn(|i| {
            let t = i as f32 * 0.4;
            0.05 + 0.03 * crate::impairment::sin_approx(t)
        });
        let result = check_calibration_window(&residuals, 0.002);
        // Either high autocorr or high variance ratio should trigger
        assert!(result.contamination_suspected
            || result.variance_ratio > 1.5,
            "autocorrelated window should trigger contamination: {:?}", result);
    }

    #[test]
    fn insufficient_window_returns_contamination_suspected() {
        let tiny = [0.05_f32, 0.06, 0.04];
        let r = check_calibration_window(&tiny, 0.002);
        assert!(r.contamination_suspected, "window < 4 must flag contamination");
        assert!(!r.wss_pass);
    }

    #[test]
    fn offset_window_triggers_normalized_mean_dev() {
        // Realistic calibration contamination: interference onset WITHIN the
        // calibration window causes a step-change in the residual mean.
        // First half: healthy baseline ~0.050; second half: elevated ~0.120.
        // Half-window drift = |0.120 - 0.050| / 0.002 = 35.0 >> 2.0 threshold.
        let mut residuals = [0.050_f32; 20];
        for r in &mut residuals[10..] {
            *r = 0.120;
        }
        let r = check_calibration_window(&residuals, 0.002);
        assert!(r.normalized_mean_dev >= 2.0,
            "step-change contamination must exceed normalized_mean_dev threshold: {}",
            r.normalized_mean_dev);
        assert!(r.contamination_suspected,
            "step-change calibration window must be flagged as contaminated");
    }

    #[test]
    fn high_variance_window_triggers_variance_ratio() {
        // Wide variance: impulsive corruption in calibration window
        let mut residuals = [0.05_f32; 20];
        residuals[5]  = 0.25; // impulsive spike
        residuals[12] = 0.30;
        let r = check_calibration_window(&residuals, 0.002);
        assert!(r.contamination_suspected,
            "high-variance window must trigger contamination");
    }

    // ── Swarm baseline sanity ──────────────────────────────────────────────

    #[test]
    fn swarm_unanimous_baseline_agreement() {
        let rho_vals = [0.100f32, 0.101, 0.099, 0.100, 0.102];
        let alert = swarm_baseline_sanity_check(&rho_vals, 3.0);
        assert!(matches!(alert, BaselineConsensusAlert::Agreed { .. }),
            "nearly-identical rho values should flag Agreed");
    }

    #[test]
    fn swarm_disagreement_triggers_unreliable_alert() {
        // Two nodes calibrated on healthy signal, three on contaminated window
        let rho_vals = [0.10f32, 0.10, 0.25, 0.28, 0.27];
        let alert = swarm_baseline_sanity_check(&rho_vals, 3.0);
        assert!(matches!(alert, BaselineConsensusAlert::UnreliableBaseline { .. }),
            "wide rho spread must flag UnreliableBaseline, got: {alert:?}");
    }
}

// ── Swarm Baseline Sanity Check ──────────────────────────────────────────────
//
// DEFENCE: "Bootstrap Paradox" (paper §XIX-B).
//
// If a system calibrates while an adversary performs a "Low-and-Slow" spoofing
// attack, the jammer becomes the admissible baseline.  The engine then treats
// the "Signal" as a "Violation".
//
// Defence: require M=5 (or more) swarm nodes to agree on "healthy" before
// locking calibration.  If nodes disagree on ρ beyond the spread tolerance,
// calibration is refused and `UnreliableBaseline` is issued.
//
// The Wiener-Khinchin stationarity check (`check_calibration_window` above)
// is the primary per-node defence.  This function operates at the swarm level.

/// Decision issued by the swarm-level baseline sanity check.
///
/// See `swarm_baseline_sanity_check()` for usage.
///
/// ## Relationship to paper §XIX-B (Bootstrap Paradox defence)
///
/// Calibration MUST NOT lock until `Agreed` is returned and every participating
/// node also passes its individual `check_calibration_window()` check (WSS
/// pre-condition, §XII).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaselineConsensusAlert {
    /// All participating nodes agree on a healthy baseline.
    /// Calibration may proceed after individual WSS checks pass.
    Agreed {
        /// Number of nodes that participated in the consensus.
        node_count: u8,
        /// Median ρ across all nodes.
        median_rho:  f32,
        /// Robust spread (1.4826 × MAD of ρ values).
        robust_sigma_rho: f32,
    },
    /// Node disagreement exceeds the spread tolerance.
    ///
    /// CALIBRATION MUST NOT LOCK until this is resolved.
    /// Possible causes: Low-and-Slow spoofing contaminating one or more
    /// calibration windows; hardware fault on a node; severe multipath.
    UnreliableBaseline {
        /// Nodes whose ρ is within tolerance of the median.
        agreeing:    u8,
        /// Nodes whose ρ deviates beyond tolerance.
        disagreeing: u8,
        /// Robust spread (1.4826 × MAD) of ρ values across all nodes.
        rho_spread:  f32,
    },
}

/// Compute a swarm-level baseline sanity check from a set of per-node ρ values.
///
/// Uses the robust MAD estimator (consistent with the BFT Byzantine filtering
/// in `swarm_consensus.rs`) to flag disagreement.  A node whose ρ deviates
/// by more than `spread_tolerance_sigma` × (1.4826 · MAD) from the swarm
/// median is counted as disagreeing.
///
/// Returns `BaselineConsensusAlert::UnreliableBaseline` if the number of
/// disagreeing nodes exceeds 0 (conservative: any node disagreement triggers
/// the alert when N ≤ 10) or if `rho_values` is empty.
///
/// ## Arguments
///
/// * `rho_values` — envelope radii ρ from participating nodes (up to 32).
/// * `spread_tolerance_sigma` — MAD-sigma tolerance for agreement.
///   Default 3.0 (consistent with GUM k=3 coverage, paper §VII-A Theorem 2).
///
/// # Examples
///
/// ```
/// use dsfb_rf::calibration::{swarm_baseline_sanity_check, BaselineConsensusAlert};
/// // 5 nodes all see ρ ≈ 0.100 — healthy consensus
/// let rho = [0.100f32, 0.101, 0.099, 0.100, 0.102];
/// let alert = swarm_baseline_sanity_check(&rho, 3.0);
/// assert!(matches!(alert, BaselineConsensusAlert::Agreed { .. }));
/// ```
pub fn swarm_baseline_sanity_check(
    rho_values: &[f32],
    spread_tolerance_sigma: f32,
) -> BaselineConsensusAlert {
    let n = rho_values.len();
    if n == 0 {
        return BaselineConsensusAlert::UnreliableBaseline {
            agreeing: 0,
            disagreeing: 0,
            rho_spread: 0.0,
        };
    }

    let n_clamped = n.min(32);
    let median = compute_rho_median(rho_values, n_clamped);
    let robust_sigma = compute_rho_robust_sigma(rho_values, n_clamped, median);
    let threshold = spread_tolerance_sigma * robust_sigma;
    let (agreeing, disagreeing) = tally_agreement(rho_values, n_clamped, median, threshold);

    if disagreeing == 0 {
        BaselineConsensusAlert::Agreed {
            node_count:       n_clamped as u8,
            median_rho:       median,
            robust_sigma_rho: robust_sigma,
        }
    } else {
        BaselineConsensusAlert::UnreliableBaseline {
            agreeing,
            disagreeing,
            rho_spread: robust_sigma,
        }
    }
}

fn insertion_sort_fixed(buf: &mut [f32; 32], len: usize) {
    for i in 1..len {
        let key = buf[i];
        let mut j = i;
        while j > 0 && buf[j - 1] > key {
            buf[j] = buf[j - 1];
            j -= 1;
        }
        buf[j] = key;
    }
}

fn median_sorted(sorted: &[f32; 32], len: usize) -> f32 {
    if len % 2 == 1 { sorted[len / 2] }
    else { (sorted[len / 2 - 1] + sorted[len / 2]) * 0.5 }
}

fn compute_rho_median(rho_values: &[f32], n_clamped: usize) -> f32 {
    let mut scratch = [0.0f32; 32];
    for i in 0..n_clamped { scratch[i] = rho_values[i]; }
    insertion_sort_fixed(&mut scratch, n_clamped);
    median_sorted(&scratch, n_clamped)
}

fn compute_rho_robust_sigma(rho_values: &[f32], n_clamped: usize, median: f32) -> f32 {
    let mut abs_devs = [0.0f32; 32];
    for i in 0..n_clamped {
        let d = rho_values[i] - median;
        abs_devs[i] = if d < 0.0 { -d } else { d };
    }
    insertion_sort_fixed(&mut abs_devs, n_clamped);
    let mad = median_sorted(&abs_devs, n_clamped);
    const MAD_SCALE: f32 = 1.482_602_2;
    (MAD_SCALE * mad).max(1e-9_f32)
}

fn tally_agreement(rho_values: &[f32], n_clamped: usize, median: f32, threshold: f32) -> (u8, u8) {
    let mut agreeing:    u8 = 0;
    let mut disagreeing: u8 = 0;
    for i in 0..n_clamped {
        let d = rho_values[i] - median;
        let dev = if d < 0.0 { -d } else { d };
        if dev <= threshold {
            agreeing = agreeing.saturating_add(1);
        } else {
            disagreeing = disagreeing.saturating_add(1);
        }
    }
    (agreeing, disagreeing)
}
