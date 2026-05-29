//! The chemometric detector atlas (executed subset).
//!
//! # Role in the pipeline
//! Each detector is *fit* on a baseline normal-operating window and *scores* every sample,
//! emitting the common output schema (id, family, scope, raw/normalized score, threshold, signed
//! margin, breach). The `signed_margin` of each detector becomes a residual stream consumed by the
//! DSFB grammar. Detectors are organised into the four families from the project plan.
//!
//! The key quantity fed upstream is the *one-sided exceedance*:
//! ```text
//! exceedance = max(0, score / threshold − 1)   (= max(0, signed_margin))
//! ```
//! Values ≤ 0 mean the sample is within the control limit; values > 0 are proportional to how far
//! the sample has breached the limit.
//!
//! # Implemented families
//! - **A – Classical MSPC**: PCA Hotelling T², PCA SPE/Q, Shewhart max-|z|, Robust MAD-z.
//!   These are established multivariate statistical process control methods (prior art).
//! - **B – Dynamic/Temporal**: EWMA, CUSUM, Page–Hinkley, Mann–Kendall.
//!   Sequential detectors that accumulate information across time; they are sensitive to
//!   sustained shifts and slow drifts that single-sample tests can miss.
//! - **C – Nonlinear/Distributional**: PSI, KS-window, kNN, Spectral Entropy.
//!   Distribution-comparison detectors applied to the SPE residual-energy stream.
//! - **D – Process-Structure**: CoDrift, SensorBias.
//!   Use the cross-variable standardised row to flag structural anomalies.
//!
//! This module implements a defensible *executed* subset (~17 detectors). The full literature
//! corpus (54+ canonicalised detectors) is catalogued in `atlas.rs`; an `implementation_status`
//! field there honestly distinguishes executed detectors from catalogued prior-art entries.
//!
//! # Determinism / replay guarantee
//! All detectors are purely functional over their inputs: no RNG, no global mutable state, no
//! I/O. Given identical `DataMatrix` / `Baseline` / `PcaModel` inputs the outputs are
//! byte-reproducible across runs. Sort-based operations (e.g. `percentile`) use the stdlib
//! deterministic sort.

use crate::data::{Baseline, DataMatrix};
use crate::fft;
use crate::linalg::{self, PcaModel};
use serde::{Deserialize, Serialize};

/// Detector family taxonomy (project-plan families A–D).
///
/// Used in `DetectorOutput` to let downstream consumers (fusion, heuristics, reports) group
/// results by methodology without re-parsing detector IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum DetectorFamily {
    /// Family A: classical multivariate statistical process control (PCA-based, Shewhart).
    ClassicalMspc,
    /// Family B: sequential / adaptive detectors that operate on the time-ordered SPE stream.
    DynamicTemporal,
    /// Family C: distributional comparison detectors (histogram, rank, spectral).
    NonlinearDistributional,
    /// Family D: cross-variable structural detectors.
    ProcessStructure,
}

impl DetectorFamily {
    /// Short, stable string tag used in serialised output and report headers.
    /// The "A:" / "B:" prefix encodes the project-plan ordering for human readers.
    pub fn label(&self) -> &'static str {
        match self {
            DetectorFamily::ClassicalMspc => "A:ClassicalMSPC",
            DetectorFamily::DynamicTemporal => "B:DynamicTemporal",
            DetectorFamily::NonlinearDistributional => "C:NonlinearDistributional",
            DetectorFamily::ProcessStructure => "D:ProcessStructure",
        }
    }
}

/// One detector's output at a single time step (the common detector schema).
///
/// All per-sample detector results carry this uniform structure so that fusion, the DSFB grammar,
/// and report generation can treat every detector identically regardless of family.
///
/// # Score conventions
/// - `raw_score`: the detector's native statistic in its own units (e.g. T² value, CUSUM sum,
///   KS distance).  Units differ by detector; do not compare raw scores across detectors.
/// - `threshold`: the detector-specific control limit derived from the baseline window.
///   Always positive (clamped to ≥ 1e-12 by the `out` helper).
/// - `normalized_score = raw_score / threshold`: dimensionless ratio; 1.0 is exactly at limit.
/// - `signed_margin = normalized_score − 1`: the residual fed into the DSFB grammar.
///   Positive means the sample has exceeded the control limit; the magnitude is the fractional
///   overshoot (e.g. 0.5 means 50 % above the limit).
/// - `breach = normalized_score > 1.0`: Boolean convenience for the pipeline's alert path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorOutput {
    /// Stable string identifier of the originating detector (matches `Detector::id`).
    pub detector_id: String,
    pub family: DetectorFamily,
    /// Zero-based row index into the originating `DataMatrix` (preserves temporal ordering).
    pub time_index: usize,
    /// Human-readable description of the variable(s) this score summarises (e.g. "all",
    /// "scores", "residual", "single-var").
    pub variable_scope: String,
    pub raw_score: f64,
    pub normalized_score: f64,
    pub threshold: f64,
    /// `normalized_score − 1`: the residual stream consumed by DSFB. >0 means above the limit.
    pub signed_margin: f64,
    pub breach: bool,
}

/// Precomputed context shared by all detectors for one dataset run.
///
/// Computed once by the pipeline and passed by shared reference to every `Detector::run` call,
/// avoiding repeated computation of expensive operations (PCA projection, standardisation).
///
/// Lifetimes ensure the borrowed data outlives all detector calls within the same run.
pub struct DetectorContext<'a> {
    /// Full data matrix for the run (baseline + test rows combined).
    pub m: &'a DataMatrix,
    /// Number of rows in `m` that belong to the baseline / normal-operating window.
    /// Rows `0..n_base` are used to fit control limits; rows `n_base..m.n_samples` are test data.
    /// Detectors that fit limits from baseline data must clamp their slice to `[..n_base]`.
    pub n_base: usize,
    /// Pre-fitted baseline statistics (means, standard deviations) computed over `0..n_base`.
    pub baseline: &'a Baseline,
    /// PCA model fitted on the baseline window; used to project samples into score / loading space.
    pub pca: &'a PcaModel,
    /// Per-sample standardised rows: `z[i][j] = (x[i][j] − baseline.mean[j]) / baseline.std[j]`.
    /// Shape: `[m.n_samples][m.n_vars]`. Unitless (standard-deviation units).
    pub z: &'a [Vec<f64>],
    /// Per-sample Hotelling T² statistic (score space).
    /// Scalar per sample; units are dimensionless (chi-squared-like). Length == `m.n_samples`.
    pub t2: &'a [f64],
    /// Per-sample squared prediction error (Q / SPE) — the canonical residual-energy stream.
    /// Measures variance *unexplained* by the retained PCA components.
    /// Scalar per sample; units are squared original-variable units (summed across variables).
    /// Length == `m.n_samples`. This is the primary input for all Family-B/C/D detectors.
    pub spe: &'a [f64],
}

/// Trait implemented by every executed detector.
///
/// Implementors are responsible for deriving their own control limit from the baseline slice
/// (`ctx.spe[..ctx.n_base]` or equivalent) and producing one `DetectorOutput` per sample row.
/// The output `Vec` must have length `ctx.m.n_samples` so downstream code can zip by index.
pub trait Detector {
    /// Stable string identifier; must match the corresponding entry in the TOML corpus.
    fn id(&self) -> &'static str;
    fn family(&self) -> DetectorFamily;
    /// Variable scope description used in `DetectorOutput::variable_scope`.  Defaults to "all"
    /// (whole-row score); override for detectors that target a sub-set of variables.
    fn scope(&self) -> &str {
        "all"
    }
    /// Score every sample in `ctx` and return one `DetectorOutput` per sample in temporal order.
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput>;
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Percentile (0..1) of a slice via deterministic sort.
///
/// `q = 0.0` returns the minimum; `q = 1.0` returns the maximum.
/// Non-finite values (NaN, ±inf) are silently dropped before sorting so that sensor dropouts
/// do not corrupt the control limit.
/// Sort order is deterministic (no RNG): given identical input slices the result is identical
/// across runs, which is required for byte-exact replay of pipeline outputs.
/// Returns 0.0 for empty or all-non-finite input (caller must apply `.max(1e-9)` if needed).
fn percentile(x: &[f64], q: f64) -> f64 {
    if x.is_empty() {
        return 0.0;
    }
    let mut v: Vec<f64> = x.iter().copied().filter(|a| a.is_finite()).collect();
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // Nearest-rank index: round to the closest position; saturates at the last element.
    let idx = ((v.len() - 1) as f64 * q).round() as usize;
    v[idx]
}

/// Convenience constructor: build a `DetectorOutput` from raw score + threshold.
///
/// This is the *only* place where `normalized_score`, `signed_margin`, and `breach` are derived,
/// keeping the arithmetic consistent across all detectors.
///
/// The threshold floor of 1e-12 prevents division-by-zero when a detector's baseline window is
/// degenerate (e.g. constant SPE series).  The caller should separately apply `.max(1e-9)` to
/// empirically derived thresholds before passing them here.
fn out(
    id: &str,
    fam: DetectorFamily,
    scope: &str,
    ti: usize,
    raw: f64,
    thr: f64,
) -> DetectorOutput {
    // Guard: floor the threshold at 1e-12 so it is genuinely positive (matches the doc invariant).
    // `f64::max` clamps any value below 1e-12 — including a non-positive or NaN threshold — up to the
    // floor, preventing ±inf/NaN normalised scores. All executed detectors already pass a positive
    // control limit (>= ~1e-9), so this is bit-identical for current inputs; it only hardens the surface.
    let thr = thr.max(1e-12);
    let norm = raw / thr;
    DetectorOutput {
        detector_id: id.to_string(),
        family: fam,
        time_index: ti,
        variable_scope: scope.to_string(),
        raw_score: raw,
        normalized_score: norm,
        threshold: thr,
        // signed_margin > 0 ↔ limit breach; this is the quantity fed to the DSFB grammar.
        signed_margin: norm - 1.0,
        breach: norm > 1.0,
    }
}

// ── Family A: Classical MSPC ────────────────────────────────────────────────────
//
// Detectors in this family are established prior art in multivariate statistical process control
// (MSPC). None of the implementations here are claimed as novel; they encode well-known
// estimators from the chemometrics literature (Jackson & Mudholkar, Nomikos & MacGregor, etc.).

/// PCA Hotelling T² detector with a baseline 99th-percentile control limit.
///
/// T² measures the Mahalanobis distance of the sample's PCA score vector from the centre of the
/// score space.  It is sensitive to sustained shifts *within* the PCA subspace (directions of
/// high baseline variance) but is blind to changes in the residual/complementary subspace.
///
/// Control limit: 99th percentile of the baseline T² distribution (empirical, not
/// chi-squared-approximated), computed over `ctx.t2[0..n_base]`.  Using the empirical
/// distribution avoids distributional assumptions on the input data.
pub struct PcaT2;
impl Detector for PcaT2 {
    fn id(&self) -> &'static str {
        "pca_t2"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::ClassicalMspc
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        // Limit derived from baseline rows only; test rows never influence the control limit.
        let thr = percentile(&ctx.t2[..ctx.n_base.min(ctx.t2.len())], 0.99).max(1e-9);
        ctx.t2
            .iter()
            .enumerate()
            .map(|(i, &v)| out(self.id(), self.family(), "scores", i, v, thr))
            .collect()
    }
}

/// PCA SPE (Squared Prediction Error, also called Q statistic) detector.
///
/// SPE measures variance *not* captured by the retained PCA components — i.e. the reconstruction
/// error in the original variable space.  It is complementary to T²: T² detects changes inside
/// the PCA subspace; SPE detects changes in the residual (unexplained) subspace.  Together they
/// form the classical MSPC monitoring pair.
///
/// Control limit: empirical 99th percentile of baseline SPE values.  All Family-B/C detectors
/// also operate on this same SPE stream, so the threshold here sets the primary scale reference.
pub struct PcaSpeQ;
impl Detector for PcaSpeQ {
    fn id(&self) -> &'static str {
        "pca_spe_q"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::ClassicalMspc
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let thr = percentile(&ctx.spe[..ctx.n_base.min(ctx.spe.len())], 0.99).max(1e-9);
        ctx.spe
            .iter()
            .enumerate()
            .map(|(i, &v)| out(self.id(), self.family(), "residual", i, v, thr))
            .collect()
    }
}

/// Shewhart-style multivariate max |z| detector.
///
/// For each sample, takes the maximum absolute standardised deviation across all variables:
/// `score = max_j |z[i][j]|` where `z[i][j] = (x[i][j] − mean[j]) / std[j]`.
///
/// The fixed 3σ limit is the traditional Shewhart individual-sample control limit (prior art).
/// This detector fires on *any single variable* exceeding 3 standard deviations; it does not
/// account for the multiplicity of variables (Bonferroni correction is not applied here).
///
/// Limitation: with many variables, the probability of at least one exceeding 3σ by chance
/// grows, potentially inflating false-positive rates.
pub struct ShewhartMaxZ;
impl Detector for ShewhartMaxZ {
    fn id(&self) -> &'static str {
        "shewhart_max_z"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::ClassicalMspc
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        ctx.z
            .iter()
            .enumerate()
            .map(|(i, row)| {
                // Largest absolute standardised deviation across all variables in this sample row.
                let mx = row.iter().fold(0.0f64, |a, &b| a.max(b.abs()));
                out(self.id(), self.family(), "all", i, mx, 3.0)
            })
            .collect()
    }
}

/// Robust Hampel-style max-|z| detector using per-variable median and MAD.
///
/// Instead of mean/std (which are sensitive to outliers), this detector uses:
/// - location: per-variable **median** of the baseline window.
/// - scale: per-variable **MAD** (median absolute deviation of the baseline window).
///
/// Score per sample: `max_j |(x[i][j] − median[j]) / MAD[j]|`.
/// Control limit: **3.5** (the Hampel identifier threshold from Leys et al. 2013 — prior art).
///
/// More resistant to masking by influential baseline observations than `ShewhartMaxZ`.
/// The `fit` step runs once on the baseline; `run` then re-uses the stored medians and MADs.
pub struct RobustZMad {
    /// Per-variable baseline median; length == `DataMatrix::n_vars`.
    med: Vec<f64>,
    /// Per-variable baseline MAD; floored to 1e-12 to prevent division by zero on constant cols.
    mad: Vec<f64>,
}
impl RobustZMad {
    /// Fit per-variable median and MAD from the first `n_base` rows of `ctx_m`.
    ///
    /// Called once by `build_bank`; the fitted parameters are stored in the struct and reused
    /// for every sample scored by `run`.  Order of column iteration is deterministic (0..n_vars).
    pub fn fit(ctx_m: &DataMatrix, n_base: usize) -> Self {
        let n = n_base.min(ctx_m.n_samples);
        let v = ctx_m.n_vars;
        let mut med = vec![0.0; v];
        let mut mad = vec![1e-12; v]; // MAD defaults to 1e-12 to handle constant columns safely.
        for j in 0..v {
            let col: Vec<f64> = (0..n).map(|i| ctx_m.get(i, j)).collect();
            med[j] = linalg::median(&col);
            mad[j] = linalg::mad(&col);
        }
        RobustZMad { med, mad }
    }
}
impl Detector for RobustZMad {
    fn id(&self) -> &'static str {
        "robust_z_mad"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::ClassicalMspc
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        (0..ctx.m.n_samples)
            .map(|i| {
                let row = ctx.m.row(i);
                // Largest robust standardised deviation; analogous to ShewhartMaxZ but uses
                // median/MAD rather than mean/std, reducing sensitivity to baseline outliers.
                let mx = (0..ctx.m.n_vars).fold(0.0f64, |a, j| {
                    a.max(((row[j] - self.med[j]) / self.mad[j]).abs())
                });
                out(self.id(), self.family(), "all", i, mx, 3.5)
            })
            .collect()
    }
}

// ── Family B: Dynamic / Temporal (operate on the SPE residual-energy stream) ─────
//
// All Family-B detectors run on the scalar SPE stream (one value per sample).  Operating on the
// summary residual-energy stream rather than on raw variables keeps the problem univariate and
// makes the sequential statistics well-defined regardless of the number of process variables.

/// Exponentially Weighted Moving Average (EWMA) control chart on the SPE stream.
///
/// The EWMA smoother `z[t] = λ·SPE[t] + (1−λ)·z[t−1]` down-weights distant observations
/// exponentially, making it sensitive to sustained small shifts that a Shewhart chart misses.
///
/// Score at each sample: `|z[t] − mu_baseline|` (absolute deviation of the smoother from the
/// baseline mean).  Control limit: the theoretical EWMA steady-state standard deviation
/// `3σ√(λ/(2−λ))` — the standard Roberts (1959) derivation for a zero-mean shift (prior art).
///
/// `lambda = 0.2` (default) gives moderate smoothing; lower λ is more sensitive to slow drift.
pub struct Ewma {
    /// Smoothing factor λ ∈ (0,1].  λ=1 degenerates to the Shewhart chart.
    lambda: f64,
}
impl Default for Ewma {
    fn default() -> Self {
        Ewma { lambda: 0.2 }
    }
}
impl Detector for Ewma {
    fn id(&self) -> &'static str {
        "ewma_spe"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::DynamicTemporal
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let nb = ctx.n_base.min(s.len()).max(1);
        let mu = linalg::mean(&s[..nb]);
        let sd = linalg::std_dev(&s[..nb]);
        // Theoretical EWMA steady-state control limit: 3σ·√(λ/(2−λ)).
        // Derived from variance of the EWMA statistic under i.i.d. Gaussian input (prior art).
        let lim = 3.0 * sd * (self.lambda / (2.0 - self.lambda)).sqrt();
        // Initialise the smoother at the baseline mean so the first few samples are not
        // inflated by a zero-initialisation transient.
        let mut z = mu;
        s.iter()
            .enumerate()
            .map(|(i, &v)| {
                z = self.lambda * v + (1.0 - self.lambda) * z;
                out(
                    self.id(),
                    self.family(),
                    "residual",
                    i,
                    (z - mu).abs(),
                    lim.max(1e-9),
                )
            })
            .collect()
    }
}

/// Two-sided Cumulative Sum (CUSUM) control chart on the standardised SPE stream.
///
/// The CUSUM maintains two running accumulators:
/// - `S⁺[t] = max(0, S⁺[t−1] + z[t] − k)` — detects upward shifts.
/// - `S⁻[t] = max(0, S⁻[t−1] − z[t] − k)` — detects downward shifts.
///
/// where `z[t] = (SPE[t] − μ) / σ` is the standardised SPE sample.
///
/// Score: `max(S⁺, S⁻)`.  Control limit: `h = 5` (in σ units) — the standard Page (1954)
/// tabular CUSUM parameterisation (prior art).  Slack `k = 0.5σ` is the allowance for
/// indifference to shifts smaller than 0.5σ.
///
/// Unlike EWMA, the CUSUM resets its accumulators towards zero after exceedances end, making it
/// sensitive to the onset of new shifts rather than accumulating indefinitely.
///
/// The accumulators are initialised at 0.0 (cold start); the first few samples will not trigger
/// a breach unless the shift begins immediately.
pub struct Cusum;
impl Detector for Cusum {
    fn id(&self) -> &'static str {
        "cusum_spe"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::DynamicTemporal
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let nb = ctx.n_base.min(s.len()).max(1);
        let mu = linalg::mean(&s[..nb]);
        let sd = linalg::std_dev(&s[..nb]);
        let k = 0.5; // slack (reference value) in σ — indifference half-width
        let h = 5.0; // decision interval in σ — breach threshold
        let (mut sh, mut sl) = (0.0f64, 0.0f64); // upper and lower accumulators
        s.iter()
            .enumerate()
            .map(|(i, &v)| {
                let zz = (v - mu) / sd;
                sh = (sh + zz - k).max(0.0); // upper CUSUM; resets to 0 when negative
                sl = (sl - zz - k).max(0.0); // lower CUSUM; resets to 0 when negative
                let raw = sh.max(sl);
                out(self.id(), self.family(), "residual", i, raw, h)
            })
            .collect()
    }
}

/// Page–Hinkley one-sided change detector on the SPE stream.
///
/// The Page–Hinkley (PH) test detects an upward mean-shift in a stream by tracking:
/// - `cum[t] = Σ_{s≤t} (SPE[s] − μ − δ)` — cumulative sum adjusted by the tolerance δ.
/// - `min_cum` — running minimum of `cum` up to time t.
/// - Score: `PH[t] = cum[t] − min_cum[t]`.  A breach at `PH > λ` signals that the cumulative
///   deviation has grown beyond the minimum by more than the threshold.
///
/// Parameters derived from the baseline:
/// - `delta = 0.5 · σ_baseline` — tolerance for the magnitude of the targeted upward shift.
/// - `lambda = 8.0 · σ_baseline` — detection threshold; larger λ reduces false positives.
///
/// This is a one-sided (upward) change detector; it will not fire on downward shifts in SPE,
/// which is appropriate because decreasing SPE is not anomalous.
///
/// NOTE: the threshold `lambda` is expressed in the same units as SPE (not standardised), so its
/// value scales with the baseline SPE level.
pub struct PageHinkley;
impl Detector for PageHinkley {
    fn id(&self) -> &'static str {
        "page_hinkley_spe"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::DynamicTemporal
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let nb = ctx.n_base.min(s.len()).max(1);
        let mu = linalg::mean(&s[..nb]);
        let sd = linalg::std_dev(&s[..nb]);
        let delta = 0.5 * sd; // tolerance: half of baseline std
        let lambda = 8.0 * sd; // detection threshold: 8× baseline std
        let mut cum = 0.0f64;
        let mut min_cum = 0.0f64; // tracks the running minimum of cum
        s.iter()
            .enumerate()
            .map(|(i, &v)| {
                cum += v - mu - delta;
                // Update the running minimum; the PH statistic is how far cum has risen above it.
                if cum < min_cum {
                    min_cum = cum;
                }
                let ph = cum - min_cum;
                out(self.id(), self.family(), "residual", i, ph, lambda)
            })
            .collect()
    }
}

/// Windowed Mann–Kendall monotone-trend test on the SPE stream.
///
/// The Mann–Kendall (MK) statistic `S` counts concordant minus discordant pairs in the window:
/// `S = Σ_{a<b} sign(SPE[b] − SPE[a])`.
/// Under the null hypothesis of no trend, S is approximately normal with variance
/// `Var(S) = n(n−1)(2n+5) / 18` (no ties assumed — prior art).
/// Score: normalised `Z = max(0, |S| − 1) / √Var(S)` (continuity-corrected one-sample test).
/// Control limit: 1.96 (≈ α = 0.05 two-tailed critical value for the standard normal).
///
/// The test is applied to a sliding window of width `win` ending at each sample, so the score
/// at position `i` reflects whether the last `win` SPE values show a monotone trend.
///
/// Minimum window size for the test: 4 samples (returns score = 0 if window is smaller).
/// Time complexity per sample: O(win²) due to the all-pairs comparison.
pub struct MannKendall {
    /// Sliding window width (number of SPE samples).  Default: 30.
    win: usize,
}
impl Default for MannKendall {
    fn default() -> Self {
        MannKendall { win: 30 }
    }
}
impl Detector for MannKendall {
    fn id(&self) -> &'static str {
        "mann_kendall_spe"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::DynamicTemporal
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let w = self.win.min(s.len().max(1));
        (0..s.len())
            .map(|i| {
                // Window: up to `win` samples ending at and including sample i.
                let lo = i.saturating_sub(w - 1);
                let window = &s[lo..=i];
                let n = window.len();
                // MK test requires at least 4 observations to give a meaningful variance.
                if n < 4 {
                    return out(self.id(), self.family(), "residual", i, 0.0, 1.96);
                }
                // S = Σ sign(window[b] − window[a]) for all a < b pairs.
                let mut sgn = 0i64;
                for a in 0..n {
                    for b in a + 1..n {
                        sgn += (window[b] - window[a]).signum() as i64;
                    }
                }
                // Theoretical variance of S under i.i.d. no-ties hypothesis. Factors are cast to f64
                // *before* multiplying: the integer form `n*(n-1)*(2n+5)` would overflow `usize` for a
                // very large window (the field is configurable in principle), whereas this is overflow-free
                // and bit-identical for in-range `n` (the integer product fits exactly in an f64 mantissa).
                let nf = n as f64;
                let var_s = nf * (nf - 1.0) * (2.0 * nf + 5.0) / 18.0;
                // Continuity-corrected standardised statistic.
                let z = if var_s > 0.0 {
                    (sgn.abs() as f64 - 1.0).max(0.0) / var_s.sqrt()
                } else {
                    0.0
                };
                out(self.id(), self.family(), "residual", i, z, 1.96)
            })
            .collect()
    }
}

// ── Family C: Nonlinear / Distributional ────────────────────────────────────────
//
// These detectors compare a sliding window of the SPE stream to the baseline SPE distribution,
// detecting changes in distributional shape that are invisible to moment-based detectors.
// All are established prior-art methods; none are claimed as novel.

/// Population Stability Index (PSI) of a sliding SPE window vs the baseline distribution.
///
/// PSI is a symmetric KL-divergence variant used in credit/risk modelling for population drift
/// (prior art).  For each sample, a window of `win` SPE values is histogrammed into `bins` bins
/// whose edges are derived from the baseline quantiles.  The PSI score is:
/// `PSI = Σ_b (p_cur[b] − p_base[b]) · ln(p_cur[b] / p_base[b])`
/// where `p_base` and `p_cur` are the normalised histogram proportions.
///
/// Control limit: **0.25** — a conventional rule-of-thumb boundary (< 0.10 stable, 0.10–0.25
/// minor shift, > 0.25 significant distributional change) from credit-scoring practice.
///
/// Implementation notes:
/// - Bin edges use equal-probability (quantile) spacing on the baseline, so each baseline bin
///   starts with equal expected proportion — avoiding sparse-tail artefacts.
/// - Each bin proportion is floored at 1e-4 to prevent log(0) when a sliding-window bin is empty.
pub struct Psi {
    /// Sliding window width in samples.  Default: 40.
    win: usize,
    /// Number of histogram bins.  Default: 10.
    bins: usize,
}
impl Default for Psi {
    fn default() -> Self {
        Psi { win: 40, bins: 10 }
    }
}
impl Detector for Psi {
    fn id(&self) -> &'static str {
        "psi_spe"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::NonlinearDistributional
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let nb = ctx.n_base.min(s.len()).max(2);
        // Bin edges from baseline quantiles: equal-probability bins so each baseline bin ≈ 1/bins.
        let edges: Vec<f64> = (0..=self.bins)
            .map(|b| percentile(&s[..nb], b as f64 / self.bins as f64))
            .collect();
        // Helper: bin `data` into the fixed baseline edges and return normalised proportions.
        // Proportions are floored at 1e-4 to keep the PSI sum finite.
        let hist = |data: &[f64]| -> Vec<f64> {
            let mut h = vec![0.0; self.bins];
            for &v in data {
                // Assign each value to the first bin whose right edge is ≥ v; default to last bin.
                let mut bi = self.bins - 1;
                for b in 0..self.bins {
                    if v <= edges[b + 1] {
                        bi = b;
                        break;
                    }
                }
                h[bi] += 1.0;
            }
            let tot: f64 = h.iter().sum::<f64>().max(1.0);
            h.iter().map(|c| (c / tot).max(1e-4)).collect()
        };
        let base_h = hist(&s[..nb]);
        let w = self.win.min(s.len().max(1));
        (0..s.len())
            .map(|i| {
                let lo = i.saturating_sub(w - 1);
                let cur_h = hist(&s[lo..=i]);
                // PSI = Σ_b (cur − base)·ln(cur/base); take abs to get a non-negative score.
                let psi: f64 = base_h
                    .iter()
                    .zip(&cur_h)
                    .map(|(b, c)| (c - b) * (c / b).ln())
                    .sum();
                out(self.id(), self.family(), "residual", i, psi.abs(), 0.25)
            })
            .collect()
    }
}

/// Two-sample Kolmogorov–Smirnov (KS) detector: sliding SPE window vs baseline.
///
/// The KS statistic is the supremum of the absolute difference between the empirical CDFs of the
/// baseline and the sliding window:  `D = sup_x |F_base(x) − F_win(x)|`.
/// It is sensitive to any change in the distribution (location, scale, shape) without assuming
/// a parametric family — prior art (Kolmogorov 1933, Smirnov 1948).
///
/// Control limit: the approximate two-sample critical value at α ≈ 0.05:
/// `c_α = 1.36 · √((n_base + win) / (n_base · win))`  (from the Kolmogorov asymptotic table).
/// This is an approximation; exact tabulation would require a lookup table.
///
/// The baseline CDF is pre-sorted once; the sliding window is re-sorted each sample — O(win·log(win))
/// per sample.  The CDFs are merged in O(n_base + win) using the standard merge-walk.
pub struct KsWindow {
    /// Sliding window width in samples.  Default: 40.
    win: usize,
}
impl Default for KsWindow {
    fn default() -> Self {
        KsWindow { win: 40 }
    }
}
impl Detector for KsWindow {
    fn id(&self) -> &'static str {
        "ks_spe"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::NonlinearDistributional
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let nb = ctx.n_base.min(s.len()).max(2);
        // Sort the baseline once; reused for all sliding-window comparisons. Filter non-finite first:
        // the current Rust sort panics on a total-order violation, which NaN triggers even via
        // `unwrap_or` — so a quality-gated NaN becomes a gap in the empirical CDF (the denominators
        // below use the finite counts), not a crash. No-op on finite data → replay unchanged.
        let mut base: Vec<f64> = s[..nb].iter().copied().filter(|x| x.is_finite()).collect();
        base.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let w = self.win.min(s.len().max(1));
        // Asymptotic two-sample KS critical value (α ≈ 0.05).
        let crit = 1.36 * ((nb + w) as f64 / (nb * w) as f64).sqrt();
        (0..s.len())
            .map(|i| {
                let lo = i.saturating_sub(w - 1);
                // Sort the current window; this allocation is unavoidable for the CDF merge.
                let mut cur: Vec<f64> = s[lo..=i]
                    .iter()
                    .copied()
                    .filter(|x| x.is_finite())
                    .collect();
                cur.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                // Compute KS = max |F_base − F_cur| via a merge-walk over both sorted arrays.
                // ia and ib advance through baseline and current CDFs respectively.
                let (mut ia, mut ib, mut d) = (0usize, 0usize, 0.0f64);
                while ia < base.len() && ib < cur.len() {
                    if base[ia] <= cur[ib] {
                        ia += 1;
                    } else {
                        ib += 1;
                    }
                    let fa = ia as f64 / base.len() as f64;
                    let fb = ib as f64 / cur.len() as f64;
                    d = d.max((fa - fb).abs());
                }
                out(self.id(), self.family(), "residual", i, d, crit.max(1e-6))
            })
            .collect()
    }
}

/// k-Nearest Neighbour (kNN) distance anomaly detector on the scalar SPE stream.
///
/// For each sample, computes the average absolute distance to its `k` nearest neighbours in the
/// baseline SPE set:  `kNN(v) = (1/k) Σ_{i∈kNN(v)} |v − base[i]|`.
///
/// This is a one-dimensional instance of the local-outlier proximity score (prior art).  Although
/// operating on a scalar (SPE), it captures non-Gaussian tail behaviour and multi-modal baselines
/// that a threshold on a single percentile would miss.
///
/// Control limit: 99th percentile of the kNN scores computed on the baseline itself (leave-one-in
/// rather than leave-one-out — the nearest neighbour of a baseline sample may be itself, yielding
/// a score of 0, so the limit is derived from the self-scored distribution which is conservatively
/// permissive).
///
/// Time complexity per sample: O(n_base) for distance computation + O(n_base·log(n_base)) for the
/// sort — the baseline sort is repeated for every test sample.
pub struct KnnDistance {
    /// Number of nearest neighbours to average.  Default: 5.
    k: usize,
}
impl Default for KnnDistance {
    fn default() -> Self {
        KnnDistance { k: 5 }
    }
}
impl Detector for KnnDistance {
    fn id(&self) -> &'static str {
        "knn_spe"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::NonlinearDistributional
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let nb = ctx.n_base.min(s.len()).max(self.k + 1);
        let base = &s[..nb];
        // Closure: average distance from `v` to its k nearest baseline points.
        let knn = |v: f64| -> f64 {
            // Filter non-finite distances before sorting (NaN would panic the total-order sort); a
            // NaN baseline point simply drops out of the neighbour set. No-op on finite data.
            let mut d: Vec<f64> = base
                .iter()
                .map(|b| (v - b).abs())
                .filter(|x| x.is_finite())
                .collect();
            d.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)); // ascending — smallest distances first
            d.iter().take(self.k).sum::<f64>() / self.k as f64
        };
        // Derive the limit from the baseline's self-kNN distribution.
        let base_scores: Vec<f64> = base.iter().map(|&v| knn(v)).collect();
        let thr = percentile(&base_scores, 0.99).max(1e-9);
        s.iter()
            .enumerate()
            .map(|(i, &v)| out(self.id(), self.family(), "residual", i, knn(v), thr))
            .collect()
    }
}

/// Spectral entropy shift detector on a sliding SPE window.
///
/// The spectral entropy of a short time-series segment quantifies the flatness of its power
/// spectrum: high entropy → broadband / noise-like; low entropy → narrow-band / periodic.
/// Changes in spectral entropy indicate structural changes in the frequency content of the
/// SPE residual (e.g. emergence of oscillatory components associated with valve stiction or
/// controller limit cycles).
///
/// Score: `|(H[window] − μ_base) / σ_base|` — standardised deviation of the current window's
/// spectral entropy from the baseline mean entropy.  Control limit: 3 (standard deviations).
///
/// The actual entropy computation is delegated to `fft::spectral_entropy` (see `fft.rs`).
///
/// Baseline statistics are derived from windows starting at index `w` (the first full window
/// within the baseline), skipping early partial windows.  Fallback defaults (μ=0.5, σ=0.1)
/// are used when the baseline is too short to compute stable estimates — NOTE: these defaults
/// are heuristic and may not match the actual SPE entropy distribution.
pub struct SpectralEntropy {
    /// Sliding window width in samples; should be a power of two for FFT efficiency.  Default: 32.
    win: usize,
}
impl Default for SpectralEntropy {
    fn default() -> Self {
        SpectralEntropy { win: 32 }
    }
}
impl Detector for SpectralEntropy {
    fn id(&self) -> &'static str {
        "spectral_entropy_spe"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::NonlinearDistributional
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let w = self.win.min(s.len().max(2));
        // Closure: spectral entropy of the `win`-sample window ending at index `i`.
        let ent_at = |i: usize| -> f64 {
            let lo = i.saturating_sub(w - 1);
            fft::spectral_entropy(&s[lo..=i])
        };
        let nb = ctx.n_base.min(s.len()).max(2);
        // Collect entropy values for baseline windows that are fully within the baseline region
        // (starting from index `w` to avoid partial-window artefacts at the beginning).
        let base_ent: Vec<f64> = (w..nb).map(ent_at).collect();
        // Fallback defaults if the baseline is too short to produce reliable statistics.
        let mu = if base_ent.is_empty() {
            0.5
        } else {
            linalg::mean(&base_ent)
        };
        let sd = if base_ent.len() < 2 {
            0.1
        } else {
            linalg::std_dev(&base_ent)
        };
        (0..s.len())
            .map(|i| {
                let e = ent_at(i);
                // Standardised absolute deviation from the baseline entropy mean.
                out(
                    self.id(),
                    self.family(),
                    "residual",
                    i,
                    ((e - mu) / sd).abs(),
                    3.0,
                )
            })
            .collect()
    }
}

// ── Family D: Process-structure ─────────────────────────────────────────────────
//
// Family-D detectors examine the *cross-variable structure* of each sample row using the
// standardised z-matrix (z[i][j] = (x[i][j] − mean[j]) / std[j]).  They produce one scalar
// score per sample that summarises a structural property across all variables simultaneously.

/// Co-drift detector: counts variables simultaneously drifting in the same sign.
///
/// For each sample row, counts how many variables have `z > 2` (positive co-drift) and how
/// many have `z < −2` (negative co-drift).  Score = `max(pos_count, neg_count)`.
/// Control limit: **3** — a score ≥ 3 means at least 3 variables are simultaneously deviating
/// beyond 2σ in the same direction, which is unlikely under independent normal behaviour.
///
/// This pattern is consistent with a process-wide upset (e.g. a common-cause disturbance) or
/// a thermal excursion driving correlated drift across multiple sensors.  In isolation it is not
/// diagnostic — it only flags structural co-movement; the heuristics layer provides candidate
/// labels.
pub struct CoDrift;
impl Detector for CoDrift {
    fn id(&self) -> &'static str {
        "co_drift"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::ProcessStructure
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        ctx.z
            .iter()
            .enumerate()
            .map(|(i, row)| {
                // Count variables drifting positively (>2σ) and negatively (<−2σ).
                let pos = row.iter().filter(|&&v| v > 2.0).count();
                let neg = row.iter().filter(|&&v| v < -2.0).count();
                // Take the larger count (most prevalent direction of co-drift).
                out(self.id(), self.family(), "all", i, pos.max(neg) as f64, 3.0)
            })
            .collect()
    }
}

/// Sensor-bias candidate detector: isolation of a single outlying variable.
///
/// For each sample, sorts absolute standardised deviations `|z[j]|` in descending order and
/// checks whether the top variable is isolated:
/// - `top`: largest `|z[j]|` in the row.
/// - `second`: second-largest `|z[j]|`.
/// - Score: `top` if `second < 2.0`, else `0.0`.
///
/// The isolation condition (`second < 2.0`) requires all other variables to be within ≈ 2σ
/// while the leading variable is large.  If two or more variables are simultaneously elevated,
/// the score is zeroed — that pattern is more consistent with a process upset (see CoDrift).
///
/// Control limit: **4.0** — the top variable must exceed 4σ to produce a breach.
/// NOTE: "sensor bias candidate" is the label used when this detector fires alongside H1 in the
/// heuristics layer.  The detector itself only measures isolation; the causal interpretation
/// (instrument vs process) requires operator verification.
pub struct SensorBias;
impl Detector for SensorBias {
    fn id(&self) -> &'static str {
        "sensor_bias"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::ProcessStructure
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        ctx.z
            .iter()
            .enumerate()
            .map(|(i, row)| {
                // Sort absolute deviations descending to find the top-two outliers. Filter non-finite
                // first (a NaN variable would panic the total-order sort); it simply isn't a candidate
                // outlier. No-op on finite data → replay unchanged.
                let mut sorted: Vec<f64> = row
                    .iter()
                    .map(|v| v.abs())
                    .filter(|x| x.is_finite())
                    .collect();
                sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)); // largest first
                let top = sorted.first().copied().unwrap_or(0.0);
                let second = sorted.get(1).copied().unwrap_or(0.0);
                // Isolation score: large top, small second.
                // If any other variable is also elevated (≥ 2σ), the pattern is not isolated.
                let raw = if second < 2.0 { top } else { 0.0 };
                out(self.id(), self.family(), "single-var", i, raw, 4.0)
            })
            .collect()
    }
}

// ── Phase-C executed batch (catalogued → executed; governed atlas re-freeze) ────────────────────
//
// Four prior-art detectors promoted from the catalogue to the executed bank. Each is
// deterministic-by-construction (a fixed recursion / a fixed-iteration `PcaModel::fit` / fixed
// landmarks+window) so the byte-exact replay hash stays reproducible, and each derives its control
// limit from the baseline window only. None is claimed novel.

/// Multivariate EWMA (MEWMA) — Lowry, Woodall, Champ & Rigdon 1992 (prior art).
///
/// Smooths the *standardised vector* `z_t` with `Z_t = λ·z_t + (1−λ)·Z_{t−1}` and scores `‖Z_t‖²`
/// scaled by the EWMA steady-state factor `(2−λ)/λ`, so the statistic is comparable to a Hotelling T²
/// but carries memory: it catches small *sustained* multivariate shifts a memoryless per-sample T²
/// misses. Control limit = empirical 0.99 quantile of the baseline statistic (no distributional
/// assumption). `λ = 0.2` (fixed). Deterministic; no RNG. Non-claim: prior-art MSPC statistic.
pub struct Mewma {
    lambda: f64,
}
impl Default for Mewma {
    fn default() -> Self {
        Mewma { lambda: 0.2 }
    }
}
impl Detector for Mewma {
    fn id(&self) -> &'static str {
        "mewma"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::DynamicTemporal
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let (n, nv) = (ctx.m.n_samples, ctx.m.n_vars);
        let factor = (2.0 - self.lambda) / self.lambda; // steady-state variance scaling of the EWMA vector
        let mut zbar = vec![0.0f64; nv]; // EWMA vector, init at the standardised baseline mean (0)
        let mut stat = Vec::with_capacity(n);
        for row in ctx.z.iter().take(n) {
            let mut ss = 0.0;
            for j in 0..nv {
                zbar[j] = self.lambda * row[j] + (1.0 - self.lambda) * zbar[j];
                ss += zbar[j] * zbar[j];
            }
            stat.push(ss * factor);
        }
        // Warm-up: the EWMA needs a few samples to settle; suppress breaches (and exclude from the control
        // limit) until primed, so a cold-start transient is never read as a (spurious) episode.
        let nb = ctx.n_base.min(n);
        let warm = 16usize.min(nb);
        let thr = percentile(&stat[warm..nb], 0.99).max(1e-9);
        stat.iter()
            .enumerate()
            .map(|(i, &v)| {
                out(
                    self.id(),
                    self.family(),
                    "scores",
                    i,
                    if i < warm { 0.0 } else { v },
                    thr,
                )
            })
            .collect()
    }
}

/// Dynamic PCA (DPCA) — Ku, Storer & Georgakis 1995 (prior art).
///
/// Augments each standardised row with its lag-1 predecessor `[z_t, z_{t−1}]` and fits PCA on the
/// augmented baseline, so serial correlation (process dynamics) is part of the model. The score is the
/// SPE of the augmented test row — sensitive to *temporal* structure a static SPE (which treats rows as
/// i.i.d.) cannot see. Reuses the crate's deterministic [`PcaModel::fit`] (fixed init + 60 iterations,
/// the same routine the pipeline trusts for replay), so it is byte-reproducible. Lag order fixed at 1.
/// Non-claim: prior-art DPCA; the lag order is not auto-selected.
pub struct Dpca;
impl Detector for Dpca {
    fn id(&self) -> &'static str {
        "dpca"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::DynamicTemporal
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let (n, nv) = (ctx.m.n_samples, ctx.m.n_vars);
        // Augmented row [z_t, z_{t−1}]; row 0 repeats z_0 (no predecessor ⇒ ~zero dynamic residual).
        let aug = |i: usize| -> Vec<f64> {
            let prev = i.saturating_sub(1);
            let mut a = Vec::with_capacity(2 * nv);
            a.extend_from_slice(&ctx.z[i]);
            a.extend_from_slice(&ctx.z[prev]);
            a
        };
        let nb = ctx.n_base.min(n).max(2);
        let base_aug: Vec<Vec<f64>> = (1..nb).map(aug).collect(); // skip row 0 (degenerate predecessor)
        let k = ctx.pca.k.max(1);
        let dmodel = PcaModel::fit(&base_aug, 2 * nv, k, 60);
        let stat: Vec<f64> = (0..n).map(|i| dmodel.spe(&aug(i))).collect();
        // Warm-up: row 0 has no predecessor and the model is fit from row 1; suppress until primed.
        let warm = 16usize.min(nb);
        let thr = percentile(&stat[warm..nb], 0.99).max(1e-9);
        stat.iter()
            .enumerate()
            .map(|(i, &v)| {
                out(
                    self.id(),
                    self.family(),
                    "dynamic-residual",
                    i,
                    if i < warm { 0.0 } else { v },
                    thr,
                )
            })
            .collect()
    }
}

/// MOSUM (Moving-Sum changepoint) — Bauer & Hackl 1978 (prior art).
///
/// A fixed-width moving sum of the standardised SPE: `M_t = |Σ_{i=t−w+1}^{t} z_i| / √w` with
/// `z_i = (SPE_i − μ)/σ` over the baseline. Unlike an unbounded CUSUM, the bounded window makes it
/// sensitive to *localised* level shifts and self-resetting once a shift scrolls out of the window.
/// `w = 20` (fixed); control limit = empirical 0.99 quantile of the baseline statistic. Deterministic.
/// Non-claim: prior-art changepoint statistic.
pub struct Mosum {
    w: usize,
}
impl Default for Mosum {
    fn default() -> Self {
        Mosum { w: 20 }
    }
}
impl Detector for Mosum {
    fn id(&self) -> &'static str {
        "mosum"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::DynamicTemporal
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let s = ctx.spe;
        let n = s.len();
        let nb = ctx.n_base.min(n).max(1);
        let mu = linalg::mean(&s[..nb]);
        let sd = linalg::std_dev(&s[..nb]).max(1e-12);
        let w = self.w.max(1);
        let wf = (w as f64).sqrt();
        let stat: Vec<f64> = (0..n)
            .map(|t| {
                let lo = t + 1 - w.min(t + 1);
                let sum: f64 = (lo..=t).map(|i| (s[i] - mu) / sd).sum();
                sum.abs() / wf
            })
            .collect();
        // Warm-up: the moving window is not full until t = w−1; suppress the partial-window prefix.
        let warm = w.min(nb);
        let thr = percentile(&stat[warm..nb], 0.99).max(1e-9);
        stat.iter()
            .enumerate()
            .map(|(i, &v)| {
                out(
                    self.id(),
                    self.family(),
                    "residual",
                    i,
                    if i < warm { 0.0 } else { v },
                    thr,
                )
            })
            .collect()
    }
}

/// RBF Maximum Mean Discrepancy (MMD) — Gretton, Borgwardt, Rasch, Schölkopf & Smola 2012 (prior art).
///
/// A kernel two-sample statistic between a sliding window of recent standardised rows and a fixed set of
/// baseline *landmarks*, with an RBF kernel `k(a,b) = exp(−γ‖a−b‖²)`. `MMD²` is large when the recent
/// distribution drifts away — in *any* nonlinear feature — from the baseline, a distributional/nonlinear
/// complement to the linear residual detectors. To stay scalable + deterministic on the executed pipeline
/// (a full kernel-PCA eigendecomposition does **not** scale to multi-thousand-row datasets and remains
/// catalogued), the baseline is summarised by `L = 32` evenly-spaced landmarks and the window is fixed at
/// `w = 16`, with `γ = 1/(2·n_vars)`. Control limit = empirical 0.99 quantile of the baseline statistic.
/// Deterministic (fixed landmarks/window/γ; no RNG). Non-claim: a scalable landmark approximation of MMD,
/// not the full-sample statistic, and not a kernel-PCA reconstruction.
pub struct Mmd {
    w: usize,
    l: usize,
}
impl Default for Mmd {
    fn default() -> Self {
        Mmd { w: 16, l: 32 }
    }
}
impl Detector for Mmd {
    fn id(&self) -> &'static str {
        "mmd"
    }
    fn family(&self) -> DetectorFamily {
        DetectorFamily::NonlinearDistributional
    }
    fn run(&self, ctx: &DetectorContext) -> Vec<DetectorOutput> {
        let (n, nv) = (ctx.m.n_samples, ctx.m.n_vars.max(1));
        let nb = ctx.n_base.min(n).max(1);
        let gamma = 1.0 / (2.0 * nv as f64);
        let rbf = |a: &[f64], b: &[f64]| -> f64 {
            let mut d = 0.0;
            for j in 0..nv {
                let e = a[j] - b[j];
                d += e * e;
            }
            (-gamma * d).exp()
        };
        // Deterministic landmarks: L evenly-spaced baseline rows (indices j·nb/L).
        let l = self.l.min(nb).max(1);
        let land: Vec<&Vec<f64>> = (0..l).map(|j| &ctx.z[(j * nb) / l]).collect();
        // Landmark self-term mean_{ij} k(l_i,l_j) — constant across samples, computed once.
        let mut land_self = 0.0;
        for a in &land {
            for b in &land {
                land_self += rbf(a, b);
            }
        }
        land_self /= (l * l) as f64;
        let w = self.w.max(1);
        let stat: Vec<f64> = (0..n)
            .map(|t| {
                let lo = t + 1 - w.min(t + 1);
                let win: Vec<&Vec<f64>> = (lo..=t).map(|i| &ctx.z[i]).collect();
                let m = win.len() as f64;
                // mean_{ij} k(win_i,win_j)
                let mut ww = 0.0;
                for a in &win {
                    for b in &win {
                        ww += rbf(a, b);
                    }
                }
                ww /= m * m;
                // mean_{ij} k(win_i,land_j)
                let mut wl = 0.0;
                for a in &win {
                    for b in &land {
                        wl += rbf(a, b);
                    }
                }
                wl /= m * l as f64;
                // MMD² = E[k(x,x')] − 2E[k(x,y)] + E[k(y,y')]; clamp tiny negatives from rounding.
                (ww - 2.0 * wl + land_self).max(0.0)
            })
            .collect();
        // Warm-up: the sliding window is not full until t = w−1; suppress the partial-window prefix.
        let warm = w.min(nb);
        let thr = percentile(&stat[warm..nb], 0.99).max(1e-9);
        stat.iter()
            .enumerate()
            .map(|(i, &v)| {
                out(
                    self.id(),
                    self.family(),
                    "distributional",
                    i,
                    if i < warm { 0.0 } else { v },
                    thr,
                )
            })
            .collect()
    }
}

/// Build the executed detector bank for a dataset, fitting baseline-dependent detectors.
///
/// Returns a `Vec<Box<dyn Detector>>` containing all 18 executed detectors in a fixed,
/// deterministic order.  This order is **replay-critical** (it sets the per-detector timeline order
/// that feeds `canonical_replay_hash`), so do not reorder it.  It is, however, **independent** of the
/// corpus `canonical_id` sequence in `atlas.rs`: the relationship between the executed bank and the
/// atlas is *set membership* (every id here is catalogued there), not order equality — see the
/// authority subset gate in `tests/atlas_authority_gate.rs`.
///
/// `RobustZMad` is the only detector that requires fitting before `run` is called; all others
/// derive their parameters lazily inside `run` from `ctx.n_base`.
pub fn build_bank(m: &DataMatrix, n_base: usize) -> Vec<Box<dyn Detector>> {
    vec![
        Box::new(PcaT2),
        Box::new(PcaSpeQ),
        Box::new(ShewhartMaxZ),
        Box::new(RobustZMad::fit(m, n_base)),
        Box::new(Ewma::default()),
        Box::new(Cusum),
        Box::new(PageHinkley),
        Box::new(MannKendall::default()),
        Box::new(Psi::default()),
        Box::new(KsWindow::default()),
        Box::new(KnnDistance::default()),
        Box::new(SpectralEntropy::default()),
        Box::new(CoDrift),
        Box::new(SensorBias),
        // Phase-C executed batch — APPENDED (never inserted) so the existing 14 keep their
        // replay-critical timeline positions; these add new timelines after them.
        Box::new(Mewma::default()),
        Box::new(Dpca),
        Box::new(Mosum::default()),
        Box::new(Mmd::default()),
    ]
}
