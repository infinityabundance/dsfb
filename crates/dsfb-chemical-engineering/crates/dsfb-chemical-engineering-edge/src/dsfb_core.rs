//! DSFB core engine — Drift–Slew Fusion Bootstrap residual semiotics.
//!
//! This is the deterministic, side-effect-free residual-interpretation core. It converts a stream
//! of residual samples `r_k = observed_k − expected_k` into a `(r, δ, σ)` triple (raw residual,
//! causal drift, slew), classifies each triple against a calibrated admissibility envelope, and
//! emits a typed grammar state with a reason code. Contiguous identical states aggregate into
//! episodes. Identical inputs + identical parameters always produce identical outputs.
//!
//! The engine is read-only: nothing here writes to any upstream estimator, controller, historian,
//! alarm limit, or setpoint. It structures the diagnostic exhaust those systems already produce.
//!
//! # Data flow (one channel, one step k)
//!
//! ```text
//! ResidualSample { observed, expected, timestamp }
//!        │
//!        ▼  r_k = observed − expected
//! ResidualProcessor
//!   ├── DriftEstimator  → δ_k = (1/w) Σ_{j=k−w+1}^{k} r_j   (causal windowed mean)
//!   └── SlewEstimator   → σ_k = (r_k − r_{k−1}) / Δt          (first finite difference, σ_0 = 0)
//!        │
//!        ▼  ResidualTriple { r, δ, σ, timestamp }
//! AdmissibilityEnvelope::normalise → (r̃, δ̃, σ̃) ∈ [−∞,+∞] (1 = at the limit)
//! CoordClass::classify each component → { Interior | Grazing | Outside }
//!        │
//!        ▼  EnvelopeEval
//! GrammarClassifier::classify → (GrammarState, ReasonCode)
//!        │
//!        ▼  AnnotatedStep { triple, state, reason, channel }
//! aggregate_episodes → Vec<Episode>   (contiguous-run RLE)
//! summarise          → EpisodeSummary (counts + nominal fraction)
//! ```
//!
//! # Determinism guarantee
//!
//! No random number generators, no hash maps with non-deterministic iteration order (BTreeMap is
//! used for all state-keyed aggregates), and no floating-point reductions whose order varies at
//! runtime. The `running_sum` in `DriftEstimator` accumulates in insertion order, which is fixed
//! by the input sequence — the same sequence always visits the ring buffer in the same order.
//!
//! # Novel contribution
//!
//! The upstream detectors/estimators (PCA, CUSUM, etc.) that generate residuals are prior art.
//! The novel contribution of DSFB is this drift–slew grammar + the admissibility envelope
//! classification + the typed episode calculus built on top of those prior-art detectors.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// Residual sample and triple
// ─────────────────────────────────────────────────────────────────────────────

/// A single residual observation `r_k = observed − expected` on a named channel.
///
/// The caller is responsible for normalising `observed` and `expected` to compatible engineering
/// units before constructing this struct. DSFB never rewrites or re-scales these values.
#[derive(Debug, Clone, PartialEq)]
pub struct ResidualSample {
    /// Monotone time index (samples or seconds).
    ///
    /// Must be non-decreasing across successive calls to `ResidualProcessor::process`; a
    /// non-positive inter-step delta falls back to `dt = 1.0` in the slew estimator.
    pub timestamp: f64,
    /// Observed value (caller-normalised engineering units, or a detector score).
    pub observed: f64,
    /// Expected value from an upstream estimator / detector threshold / baseline.
    pub expected: f64,
    /// Human-readable channel identifier (e.g. `"PCA-SPE"`, `"reactor_T_residual"`).
    pub channel: String,
}

impl ResidualSample {
    pub fn new(timestamp: f64, observed: f64, expected: f64, channel: impl Into<String>) -> Self {
        ResidualSample {
            timestamp,
            observed,
            expected,
            channel: channel.into(),
        }
    }
    /// Scalar residual `r_k = observed − expected`.
    #[inline]
    pub fn residual(&self) -> f64 {
        self.observed - self.expected
    }
}

/// The `(r, δ, σ)` triple produced by the DSFB decomposition at each step.
///
/// This is the information-complete forensic record for one sample: raw deviation, its
/// slow/sustained component (drift), and its fast/transient component (slew). All three are
/// needed to distinguish the eight grammar states — for example, DriftAccum requires only δ to
/// breach while r and σ stay inside; Compound requires both δ and σ to breach simultaneously.
///
/// Units for `r`, `delta`, and `sigma` match the caller's choice of engineering units; `sigma`
/// additionally carries a "per time unit" denominator from the `Δt` used in the slew estimator.
///
/// A non-finite `r` propagates `NaN` into `delta` and `sigma` as a deliberate sentinel — the
/// `GrammarClassifier` short-circuits to `SensorFault` on any non-finite component.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResidualTriple {
    /// Raw residual `r_k = observed − expected`.
    pub r: f64,
    /// Signed causal drift `δ_k` (windowed mean of residuals).
    pub delta: f64,
    /// Slew `σ_k = (r_k − r_{k−1}) / Δt` (first finite difference; units: [r units / time unit]).
    pub sigma: f64,
    /// Time stamp carried through for traceability.
    pub timestamp: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Drift / slew estimators
// ─────────────────────────────────────────────────────────────────────────────

/// Causal sliding-window drift estimator: `δ_k = (1/w) Σ r_j` over the last `w` residuals.
///
/// Uses an O(1)-per-step ring buffer with a scalar running sum: when the buffer is full, the
/// oldest sample is subtracted before the new one is added, keeping `running_sum` exact up to
/// floating-point rounding. The sum is accumulated in strict input order, which is deterministic
/// for a fixed input sequence — no reordering, no pairwise-tree reduction.
///
/// During the warm-up phase (`filled < window`) the mean is computed over the `filled` samples
/// actually seen so far, not over the full window width. This means early-sample δ values carry
/// higher variance; the paper's results start after the warm-up period.
pub struct DriftEstimator {
    window: usize,
    buffer: Vec<f64>,
    /// Write-head index into the ring (mod window). Points to the slot that will be overwritten
    /// on the next push, i.e. currently holds the oldest sample when the buffer is full.
    head: usize,
    /// Number of valid entries written so far; saturates at `window`.
    filled: usize,
    /// Running sum of the `filled` most-recent residuals.
    running_sum: f64,
}

impl DriftEstimator {
    pub fn new(window: usize) -> Self {
        assert!(window > 0, "drift window must be positive");
        DriftEstimator {
            window,
            buffer: vec![0.0; window],
            head: 0,
            filled: 0,
            running_sum: 0.0,
        }
    }
    /// Push one residual; return the current drift estimate δ_k.
    ///
    /// When the buffer is full (`filled == window`), the oldest sample occupies `buffer[head]`
    /// and is evicted from `running_sum` before the new value is written. This preserves the
    /// O(1) per-step cost regardless of window size.
    pub fn push(&mut self, r: f64) -> f64 {
        if self.filled == self.window {
            // Evict the oldest sample from the running sum before overwriting.
            self.running_sum -= self.buffer[self.head];
        }
        self.buffer[self.head] = r;
        self.running_sum += r;
        // Advance head with wrap-around (ring buffer semantics).
        self.head = (self.head + 1) % self.window;
        if self.filled < self.window {
            self.filled += 1;
        }
        // Warm-up: divide by `filled` (not `window`) until the buffer is full.
        self.running_sum / self.filled as f64
    }
    /// Reset to a clean state identical to `new(window)`. Use between independent replay runs.
    pub fn reset(&mut self) {
        self.buffer.iter_mut().for_each(|v| *v = 0.0);
        self.head = 0;
        self.filled = 0;
        self.running_sum = 0.0;
    }
}

/// Causal first-order finite-difference slew estimator: `σ_k = (r_k − r_{k−1})/Δt`, `σ_0 = 0`.
///
/// State is minimal: only the previous residual value is retained. The initial step always
/// produces σ = 0 (by convention, documented in the paper as the σ_0 = 0 boundary condition).
/// A zero or negative `dt` also returns 0 rather than ±∞ — callers should ensure timestamps
/// are strictly monotone; `ResidualProcessor` already enforces `dt ≥ 1.0` as a fallback.
pub struct SlewEstimator {
    /// The residual value at the previous step, `r_{k−1}`. `None` at initialisation (first step).
    prev_r: Option<f64>,
}

impl SlewEstimator {
    pub fn new() -> Self {
        SlewEstimator { prev_r: None }
    }
    /// Compute σ_k and update state. `dt` = elapsed time since the previous sample.
    ///
    /// Returns 0 on the first call (σ_0 = 0 boundary condition) and when `dt ≤ 0`
    /// (degenerate timestamp; caller should not pass non-positive dt in normal operation).
    pub fn push(&mut self, r: f64, dt: f64) -> f64 {
        let sigma = match self.prev_r {
            None => 0.0, // First sample: no previous residual to difference against.
            Some(prev) => {
                if dt > 0.0 {
                    (r - prev) / dt // Standard first finite difference, units: [r/time].
                } else {
                    0.0 // Guard against zero/negative dt; avoids ±∞ slew.
                }
            }
        };
        self.prev_r = Some(r);
        sigma
    }
    /// Reset to a clean state (prev_r = None). Use between independent replay runs.
    pub fn reset(&mut self) {
        self.prev_r = None;
    }
}

impl Default for SlewEstimator {
    fn default() -> Self {
        Self::new()
    }
}

/// Stateful processor converting `ResidualSample` → `ResidualTriple`. One instance per channel.
///
/// Owns the `DriftEstimator` and `SlewEstimator` for its channel, plus the previous timestamp
/// needed to compute `dt`. Processing is purely feed-forward: each call advances internal state
/// by exactly one step and never reads from future samples.
pub struct ResidualProcessor {
    drift: DriftEstimator,
    slew: SlewEstimator,
    /// Timestamp of the most-recently processed finite sample; `None` before the first call.
    prev_ts: Option<f64>,
}

impl ResidualProcessor {
    pub fn new(drift_window: usize) -> Self {
        ResidualProcessor {
            drift: DriftEstimator::new(drift_window),
            slew: SlewEstimator::new(),
            prev_ts: None,
        }
    }
    /// Process one sample, returning the `(r, δ, σ)` triple for that step.
    ///
    /// Non-finite residuals (`NaN`, `±Inf`) return a sentinel triple `{r=NaN, δ=NaN, σ=NaN}`
    /// WITHOUT advancing the ring buffer, slew state, or `prev_ts`. This prevents a single
    /// bad sensor reading from poisoning all subsequent drift estimates.
    ///
    /// The `dt` fallback of 1.0 is used both on the very first sample (no previous timestamp)
    /// and when the timestamp appears to go backwards or is identical to the previous. In
    /// practice this makes σ_0 = 0 (the slew estimator also returns 0 on the first call) and
    /// guards against malformed data without panicking.
    pub fn process(&mut self, sample: &ResidualSample) -> ResidualTriple {
        let r = sample.residual();
        if !r.is_finite() {
            // Short-circuit: return NaN sentinel and leave all state unchanged.
            return ResidualTriple {
                r,
                delta: f64::NAN,
                sigma: f64::NAN,
                timestamp: sample.timestamp,
            };
        }
        let dt = match self.prev_ts {
            None => 1.0, // First sample: no elapsed time available; slew estimator returns 0.
            Some(prev) => {
                let d = sample.timestamp - prev;
                if d <= 0.0 {
                    1.0 // Non-positive delta (clock reversal or duplicate ts): fall back to 1.
                } else {
                    d
                }
            }
        };
        self.prev_ts = Some(sample.timestamp);
        let delta = self.drift.push(r);
        let sigma = self.slew.push(r, dt);
        ResidualTriple {
            r,
            delta,
            sigma,
            timestamp: sample.timestamp,
        }
    }
    pub fn reset(&mut self) {
        self.drift.reset();
        self.slew.reset();
        self.prev_ts = None;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Admissibility envelope
// ─────────────────────────────────────────────────────────────────────────────

/// Admissibility bounds for one DSFB channel: a product set
/// `[r_min,r_max] × [δ_min,δ_max] × [σ_min,σ_max]` plus a normalised grazing band `ε_b ∈ (0,1)`.
///
/// The envelope defines the region of "acceptable" (r, δ, σ) space for this channel under normal
/// process conditions. All three axis limits must be independently calibrated from process history;
/// the factory constants in `symmetric` and `new` are illustrative only and must be replaced for
/// real deployments.
///
/// The `grazing_band` is the fraction of each half-width treated as a transition zone: if a
/// normalised coordinate falls in `[1 − ε_b, 1]` (absolute value), the coordinate is classified
/// `Grazing` rather than `Outside`. This avoids high-frequency state chatter near the boundary.
///
/// `Copy` is derived intentionally: the envelope is immutable once calibrated and is cheap to
/// clone into replay harnesses.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdmissibilityEnvelope {
    /// Lower and upper bound on raw residual r (engineering units of the channel).
    pub r_min: f64,
    pub r_max: f64,
    /// Lower and upper bound on drift δ (same units as r; typically a tighter range than r).
    pub delta_min: f64,
    pub delta_max: f64,
    /// Lower and upper bound on slew σ (units: [r units / time unit]).
    pub sigma_min: f64,
    pub sigma_max: f64,
    /// Normalised width of the grazing zone on each boundary face; must be in (0, 1).
    pub grazing_band: f64,
}

impl AdmissibilityEnvelope {
    pub fn new(
        r_min: f64,
        r_max: f64,
        delta_min: f64,
        delta_max: f64,
        sigma_min: f64,
        sigma_max: f64,
        grazing_band: f64,
    ) -> Self {
        assert!(r_min < r_max, "r bounds must be ordered");
        assert!(delta_min < delta_max, "delta bounds must be ordered");
        assert!(sigma_min < sigma_max, "sigma bounds must be ordered");
        assert!(
            (0.0..1.0).contains(&grazing_band),
            "grazing_band must be in (0,1)"
        );
        AdmissibilityEnvelope {
            r_min,
            r_max,
            delta_min,
            delta_max,
            sigma_min,
            sigma_max,
            grazing_band,
        }
    }

    /// Symmetric envelope for a zero-centred residual statistic of scale O(1) under nominal
    /// operation, where `k` is the breach multiple.
    ///
    /// The ratios (r: ±k, δ: ±0.6k, σ: ±2k) reflect the heuristic that drift is a
    /// time-averaged quantity (so its envelope is narrower than raw r) while slew is a
    /// first-difference (amplifying noise, so its envelope is wider). Used primarily for
    /// detector-margin streams in `fusion::run_detector_dsfb`.
    ///
    /// NOTE: these scaling ratios are not derived from process physics; deployments that
    /// feed engineering-unit residuals directly must calibrate each axis independently.
    pub fn symmetric(k: f64, grazing_band: f64) -> Self {
        Self::new(-k, k, -k * 0.6, k * 0.6, -k * 2.0, k * 2.0, grazing_band)
    }

    /// Map a triple into normalised coordinates `(r̃, δ̃, σ̃)` where ±1 corresponds to the
    /// envelope boundary on each axis.
    ///
    /// The linear map `ṽ = (v − mid) / half` places the envelope centre at 0 and the
    /// boundary at ±1. Values with |ṽ| > 1 are outside the envelope; |ṽ| < 1 − ε_b is
    /// strictly interior. `NaN` inputs propagate transparently.
    pub fn normalise(&self, t: &ResidualTriple) -> (f64, f64, f64) {
        let norm = |v: f64, lo: f64, hi: f64| {
            let mid = (hi + lo) / 2.0;
            let half = (hi - lo) / 2.0;
            (v - mid) / half
        };
        (
            norm(t.r, self.r_min, self.r_max),
            norm(t.delta, self.delta_min, self.delta_max),
            norm(t.sigma, self.sigma_min, self.sigma_max),
        )
    }
}

/// Nearest-rank quantile of `xs` at fraction `q ∈ [0,1]` (deterministic; sorts a copy of the
/// input). Returns 0.0 for an empty slice. Callers should filter non-finite values beforehand.
fn nearest_rank_quantile(xs: &[f64], q: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut v: Vec<f64> = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((q * v.len() as f64).ceil() as usize).clamp(1, v.len());
    v[rank - 1]
}

/// Calibrate a per-regime admissibility envelope from that regime's baseline triples, **relaxing
/// (never tightening)** the supplied `default` envelope. Returns `default` unchanged when fewer
/// than `min_samples` baseline triples are available for the regime.
///
/// Each axis bound is widened to cover the regime's own baseline variability: the upper bound
/// becomes `max(default_upper, q_high(values))` and the lower bound `min(default_lower,
/// −q_high(|values|))`, at the `q_high` nearest-rank quantile. The result is a superset of `default`
/// on every axis, so **no regime's admissibility is ever tightened** relative to the global
/// envelope (a regime's nominal variability falls inside its own envelope, and any excursion the
/// global envelope would breach still breaches).
///
/// NOTE — the *net* effect on fused-episode baseline coverage is empirical, not strictly monotone:
/// the grammar's `Recovery` transition is path-dependent, so widening one regime's envelope can
/// re-route the state sequence and shift episode boundaries. Measured effect (paper's regime
/// result): baseline false positives fall on the non-stationary penicillin fed-batch (phase-aligned
/// regime, 0.54→0.39); the batch-level gas-sensor regime label does not capture that dataset's
/// within-baseline gas heterogeneity and does not improve its baseline FP. Fully deterministic
/// (fixed quantile, sorted copies, no RNG).
pub fn calibrate_regime_envelope(
    baseline_triples: &[ResidualTriple],
    default: AdmissibilityEnvelope,
    min_samples: usize,
    q_high: f64,
) -> AdmissibilityEnvelope {
    if baseline_triples.len() < min_samples {
        return default;
    }
    let rs: Vec<f64> = baseline_triples
        .iter()
        .map(|t| t.r)
        .filter(|x| x.is_finite())
        .collect();
    let da: Vec<f64> = baseline_triples
        .iter()
        .map(|t| t.delta.abs())
        .filter(|x| x.is_finite())
        .collect();
    let sa: Vec<f64> = baseline_triples
        .iter()
        .map(|t| t.sigma.abs())
        .filter(|x| x.is_finite())
        .collect();
    let r_hi = nearest_rank_quantile(&rs, q_high);
    let d_hi = nearest_rank_quantile(&da, q_high);
    let s_hi = nearest_rank_quantile(&sa, q_high);
    // new() re-asserts ordering; every bound only widens, so the invariants still hold.
    AdmissibilityEnvelope::new(
        default.r_min,
        default.r_max.max(r_hi),
        default.delta_min.min(-d_hi),
        default.delta_max.max(d_hi),
        default.sigma_min.min(-s_hi),
        default.sigma_max.max(s_hi),
        default.grazing_band,
    )
}

/// Classification of a single normalised coordinate against the `[−1, 1]` interval.
///
/// `Outside` takes precedence over `Grazing` when building `EnvelopeEval`: a coordinate
/// with |ṽ| > 1.0 is definitively breaching, not merely near the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordClass {
    /// Strictly inside the envelope: |ṽ| < 1 − grazing_band.
    Interior,
    /// Within the grazing band but not breaching: 1 − grazing_band ≤ |ṽ| ≤ 1.
    Grazing,
    /// Beyond the envelope boundary: |ṽ| > 1.
    Outside,
}

impl CoordClass {
    /// Classify a single normalised value.
    ///
    /// Checks `Outside` first so that `a > 1.0` is never ambiguous with the grazing condition
    /// `a >= 1.0 - grazing_band` (which would also be true for values just above 1.0 when
    /// `grazing_band` is small).
    pub fn classify(norm_val: f64, grazing_band: f64) -> Self {
        let a = norm_val.abs();
        if a > 1.0 {
            CoordClass::Outside
        } else if a >= 1.0 - grazing_band {
            CoordClass::Grazing
        } else {
            CoordClass::Interior
        }
    }
}

/// Result of evaluating a triple against an admissibility envelope.
///
/// Carries both the per-axis class and the normalised coordinates so downstream consumers
/// (the `GrammarClassifier` and forensic report renderers) can use whichever is appropriate
/// without recomputing the normalisation.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeEval {
    pub r_class: CoordClass,
    pub delta_class: CoordClass,
    pub sigma_class: CoordClass,
    /// Normalised r (1.0 = at the r_max boundary; −1.0 = at r_min).
    pub norm_r: f64,
    /// Normalised δ (same convention as norm_r).
    pub norm_delta: f64,
    /// Normalised σ (same convention as norm_r).
    pub norm_sigma: f64,
}

impl EnvelopeEval {
    /// True when the raw residual r is outside the envelope (|r̃| > 1).
    pub fn r_violated(&self) -> bool {
        self.r_class == CoordClass::Outside
    }
    /// True when drift δ is outside the envelope.
    pub fn delta_violated(&self) -> bool {
        self.delta_class == CoordClass::Outside
    }
    /// True when slew σ is outside the envelope.
    pub fn sigma_violated(&self) -> bool {
        self.sigma_class == CoordClass::Outside
    }
    /// True when no coordinate is `Outside` but at least one is `Grazing`.
    ///
    /// The guard `!r_violated() && !delta_violated() && !sigma_violated()` is deliberate:
    /// a coordinate that is simultaneously `Outside` (breach) and within the grazing band
    /// of another axis should yield the breach state, not `BoundaryGrazing`.
    pub fn any_grazing(&self) -> bool {
        !self.r_violated()
            && !self.delta_violated()
            && !self.sigma_violated()
            && (self.r_class == CoordClass::Grazing
                || self.delta_class == CoordClass::Grazing
                || self.sigma_class == CoordClass::Grazing)
    }
    /// True when all three coordinates are strictly inside the envelope.
    pub fn all_interior(&self) -> bool {
        self.r_class == CoordClass::Interior
            && self.delta_class == CoordClass::Interior
            && self.sigma_class == CoordClass::Interior
    }
}

/// Evaluate a triple against an envelope, returning the per-axis classification and
/// normalised coordinates. Pure function; no state mutation.
pub fn evaluate(env: &AdmissibilityEnvelope, t: &ResidualTriple) -> EnvelopeEval {
    let (nr, nd, ns) = env.normalise(t);
    EnvelopeEval {
        r_class: CoordClass::classify(nr, env.grazing_band),
        delta_class: CoordClass::classify(nd, env.grazing_band),
        sigma_class: CoordClass::classify(ns, env.grazing_band),
        norm_r: nr,
        norm_delta: nd,
        norm_sigma: ns,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Grammar states and reason codes
// ─────────────────────────────────────────────────────────────────────────────

/// Typed grammar state emitted by the DSFB automaton per step. Annotation tokens only; no actuation.
///
/// These are the eight states of the DSFB residual grammar (paper Table 1). They form a partial
/// severity ordering, but the ordering is not encoded as a numeric severity to avoid implying a
/// metric — states represent qualitatively different causal patterns, not just magnitudes.
///
/// Precedence in `GrammarClassifier::classify` (highest wins):
/// `SensorFault` > `Compound` > `EnvViolation` > `SlewSpike` > `DriftAccum` > `BoundaryGrazing`
/// > `Recovery` > `Nominal`.
///
/// `Ord` is derived to enable `BTreeMap` keying (for deterministic iteration order in episode
/// summaries); the derived order is the declaration order and does NOT encode severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GrammarState {
    /// All three coordinates are strictly inside the admissibility envelope.
    Nominal,
    /// Drift δ is outside the envelope while r and σ are not: sustained slow accumulation.
    DriftAccum,
    /// Slew σ is outside the envelope while δ is not: rapid transient onset.
    SlewSpike,
    /// Raw residual r is outside the envelope (δ and σ may or may not also breach).
    EnvViolation,
    /// No coordinate is `Outside`, but at least one is in the grazing band.
    BoundaryGrazing,
    /// Previous step was non-nominal; current step re-enters the strict interior.
    Recovery,
    /// Both δ and σ are simultaneously outside the envelope: concurrent drift + slew breach.
    Compound,
    /// A non-finite sensor value was received; internal state not updated.
    SensorFault,
}

impl GrammarState {
    pub fn token(&self) -> &'static str {
        match self {
            GrammarState::Nominal => "NOM",
            GrammarState::DriftAccum => "DA",
            GrammarState::SlewSpike => "SS",
            GrammarState::EnvViolation => "EV",
            GrammarState::BoundaryGrazing => "BG",
            GrammarState::Recovery => "RC",
            GrammarState::Compound => "CP",
            GrammarState::SensorFault => "SF",
        }
    }
    pub fn is_nominal(&self) -> bool {
        *self == GrammarState::Nominal
    }
    pub fn is_non_nominal(&self) -> bool {
        !self.is_nominal()
    }
}

impl fmt::Display for GrammarState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// Structured reason code attached to each classification.
///
/// Provides the operator-facing one-line description of why a particular grammar state was
/// assigned. Reason codes are more specific than states (e.g. DriftAccum splits into
/// `DriftPositive` / `DriftNegative` based on the sign of δ) but do not imply root cause —
/// they describe the pattern of the `(r, δ, σ)` signal, not the process physics that caused it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasonCode {
    Nominal,
    /// δ > 0 and outside the envelope: positive-direction drift accumulation.
    DriftPositive,
    /// δ < 0 and outside the envelope: negative-direction drift accumulation.
    DriftNegative,
    /// σ > 0 and outside the envelope: rapid upward transient.
    SlewRising,
    /// σ < 0 and outside the envelope: rapid downward transient.
    SlewFalling,
    /// r outside the envelope regardless of δ or σ.
    Violation,
    /// Within the grazing band of at least one boundary face.
    Grazing,
    /// State returning to the envelope interior from a prior non-nominal step.
    Recovery,
    /// Both δ and σ outside the envelope simultaneously.
    Compound,
    /// Non-finite sensor value received; step masked.
    OobSensor,
}

impl ReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ReasonCode::Nominal => "Operating within calibrated envelope",
            ReasonCode::DriftPositive => "Sustained positive drift outside calibrated delta bounds",
            ReasonCode::DriftNegative => "Sustained negative drift outside calibrated delta bounds",
            ReasonCode::SlewRising => "Rapid rising transient outside calibrated sigma bounds",
            ReasonCode::SlewFalling => "Rapid falling transient outside calibrated sigma bounds",
            ReasonCode::Violation => "Raw residual outside calibrated r bounds",
            ReasonCode::Grazing => "State within grazing band of envelope boundary",
            ReasonCode::Recovery => {
                "State returned to envelope interior following non-Nominal episode"
            }
            ReasonCode::Compound => "Simultaneous drift and slew exit from envelope",
            ReasonCode::OobSensor => {
                "Non-finite sensor value (NaN or inf); observation masked pending review"
            }
        }
    }
    pub fn drift(sign: f64) -> Self {
        if sign >= 0.0 {
            ReasonCode::DriftPositive
        } else {
            ReasonCode::DriftNegative
        }
    }
    pub fn slew(sign: f64) -> Self {
        if sign >= 0.0 {
            ReasonCode::SlewRising
        } else {
            ReasonCode::SlewFalling
        }
    }
}

impl fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single time-step's complete DSFB annotation record.
///
/// Bundles the numeric triple with the symbolic grammar classification and reason code.
/// This is the unit of exchange between the per-channel engine and the multi-channel fusion
/// layer (`fusion.rs`). The `channel` field enables correct routing when step slices from
/// different channels are handled together.
#[derive(Debug, Clone)]
pub struct AnnotatedStep {
    /// The `(r, δ, σ)` numeric decomposition for this step.
    pub triple: ResidualTriple,
    /// Grammar state assigned by `GrammarClassifier::classify`.
    pub state: GrammarState,
    /// Structured reason code for the assigned state.
    pub reason: ReasonCode,
    /// Channel identifier propagated from the originating `ResidualSample`.
    pub channel: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Grammar automaton
// ─────────────────────────────────────────────────────────────────────────────

/// Stateful one-step grammar classifier.
///
/// Holds only `prev_state`, the grammar state from the previous step. This single bit of history
/// is sufficient to determine whether the current step should be labelled `Recovery` (the process
/// has just re-entered the interior after a non-nominal episode) or `Nominal` (steady-state).
///
/// Precedence (highest wins): `Compound` > `EnvViolation` > `SlewSpike` > `DriftAccum`
/// > `BoundaryGrazing` > `Recovery` > `Nominal`.
///
/// A non-finite triple short-circuits to `SensorFault` without touching `prev_state`, so a bad
/// sensor reading cannot mask a subsequent `Recovery` transition — the automaton resumes from
/// the last known valid state once finite values reappear.
///
/// NOTE: `prev_state` is updated at the end of `classify`. When `Recovery` is emitted, the
/// recorded `prev_state` is set to `Nominal`, not `Recovery`, so the next step does not generate
/// another spurious `Recovery` if the process remains well-behaved.
pub struct GrammarClassifier {
    /// The grammar state produced at the immediately preceding finite step.
    prev_state: GrammarState,
}

impl Default for GrammarClassifier {
    fn default() -> Self {
        GrammarClassifier {
            prev_state: GrammarState::Nominal,
        }
    }
}

impl GrammarClassifier {
    pub fn new() -> Self {
        Self::default()
    }
    /// Classify one step and update `prev_state`.
    ///
    /// Returns `(GrammarState, ReasonCode)`. The state is determined by the precedence ladder:
    ///
    /// 1. Non-finite components → `SensorFault` (short-circuit; `prev_state` unchanged).
    /// 2. δ outside **and** σ outside → `Compound` (concurrent drift + slew breach).
    /// 3. r outside → `EnvViolation` (raw residual breach, irrespective of δ/σ individually).
    /// 4. σ outside only → `SlewSpike` (fast transient without sustained drift).
    /// 5. δ outside only → `DriftAccum` (slow sustained drift without slew spike).
    /// 6. Any coordinate in the grazing band → `BoundaryGrazing`.
    /// 7. `prev_state` was non-nominal → `Recovery` (first interior step after an episode).
    /// 8. Otherwise → `Nominal`.
    ///
    /// `prev_state` is set to `Nominal` when `Recovery` is emitted, preventing consecutive
    /// Recovery labels when the process stays inside the envelope.
    pub fn classify(
        &mut self,
        eval: &EnvelopeEval,
        t: &ResidualTriple,
    ) -> (GrammarState, ReasonCode) {
        use GrammarState::*;
        if !t.r.is_finite() || !t.delta.is_finite() || !t.sigma.is_finite() {
            // Non-finite input: emit SensorFault without touching prev_state.
            return (SensorFault, ReasonCode::OobSensor);
        }
        let new_state = if eval.delta_violated() && eval.sigma_violated() {
            // Both slow (drift) and fast (slew) components breach simultaneously → Compound.
            Compound
        } else if eval.r_violated() {
            // Raw residual outside envelope (may or may not have δ/σ involvement).
            EnvViolation
        } else if eval.sigma_violated() {
            // Fast transient only: slew breaches but drift does not.
            SlewSpike
        } else if eval.delta_violated() {
            // Slow accumulation only: drift breaches but slew does not.
            DriftAccum
        } else if eval.any_grazing() {
            // No breach, but at least one axis is near a boundary.
            BoundaryGrazing
        } else if self.prev_state != Nominal {
            // First step fully inside the interior following a non-nominal episode.
            Recovery
        } else {
            Nominal
        };
        let reason = match new_state {
            Nominal => ReasonCode::Nominal,
            DriftAccum => ReasonCode::drift(t.delta), // Sign of δ distinguishes direction.
            SlewSpike => ReasonCode::slew(t.sigma),   // Sign of σ distinguishes direction.
            EnvViolation => ReasonCode::Violation,
            BoundaryGrazing => ReasonCode::Grazing,
            Recovery => ReasonCode::Recovery,
            Compound => ReasonCode::Compound,
            SensorFault => ReasonCode::OobSensor,
        };
        // Store Nominal (not Recovery) so the next interior step does not re-emit Recovery.
        self.prev_state = if new_state == Recovery {
            Nominal
        } else {
            new_state
        };
        (new_state, reason)
    }
    /// Reset to the initial state (prev_state = Nominal). Use between independent replay runs.
    pub fn reset(&mut self) {
        self.prev_state = GrammarState::Nominal;
    }
}

/// Full single-channel DSFB engine composing all stages in sequence:
/// `ResidualSample` → `ResidualProcessor` → `ResidualTriple` → `evaluate` → `EnvelopeEval`
/// → `GrammarClassifier` → `AnnotatedStep` (appended to `history`).
///
/// One instance corresponds to one monitored channel. Multiple channels are handled by
/// multiple independent `DeterministicDsfb` instances; there is no cross-channel coupling
/// in this struct — that coupling is the responsibility of `fusion.rs`.
pub struct DeterministicDsfb {
    /// Owns the drift and slew estimators for this channel.
    processor: ResidualProcessor,
    /// Immutable calibrated admissibility envelope for this channel.
    envelope: AdmissibilityEnvelope,
    /// One-step memory for `Recovery` detection.
    classifier: GrammarClassifier,
    /// Ordered log of all annotated steps since the last `reset()` call.
    history: Vec<AnnotatedStep>,
    /// Channel label propagated into every `AnnotatedStep`.
    channel: String,
}

impl DeterministicDsfb {
    pub fn with_window(
        envelope: AdmissibilityEnvelope,
        drift_window: usize,
        channel: impl Into<String>,
    ) -> Self {
        DeterministicDsfb {
            processor: ResidualProcessor::new(drift_window),
            envelope,
            classifier: GrammarClassifier::new(),
            history: Vec::new(),
            channel: channel.into(),
        }
    }
    /// Process one sample through the full pipeline and return the annotated step.
    ///
    /// Also appends the step to `self.history` for later batch analysis.
    /// Order of operations is fixed: process → evaluate → classify. No stage feeds back
    /// to an earlier stage; the pipeline is strictly feed-forward.
    pub fn ingest_sample(&mut self, sample: &ResidualSample) -> AnnotatedStep {
        let env = self.envelope; // Copy; classify against this channel's own envelope.
        self.ingest_sample_env(sample, &env)
    }

    /// As [`ingest_sample`], but classifies the triple against an explicitly supplied `envelope`
    /// rather than `self.envelope`. The drift/slew processor and the classifier's one-step memory
    /// advance identically; only the admissibility bounds differ. Used by regime-conditioned fusion
    /// (`fusion::run_detector_dsfb_regime`), where the envelope is selected per sample from that
    /// sample's operating regime. With `envelope == self.envelope` the output is byte-identical to
    /// `ingest_sample`.
    pub fn ingest_sample_env(
        &mut self,
        sample: &ResidualSample,
        envelope: &AdmissibilityEnvelope,
    ) -> AnnotatedStep {
        let triple = self.processor.process(sample);
        let eval = evaluate(envelope, &triple);
        let (state, reason) = self.classifier.classify(&eval, &triple);
        let step = AnnotatedStep {
            triple,
            state,
            reason,
            channel: self.channel.clone(),
        };
        self.history.push(step.clone());
        step
    }
    /// Read-only access to the complete step log since the last `reset()`.
    pub fn history(&self) -> &[AnnotatedStep] {
        &self.history
    }
    /// Reset all internal state. After calling this, `ingest_sample` behaves as if the engine
    /// were freshly constructed — useful for independent replay runs with the same parameters.
    pub fn reset(&mut self) {
        self.processor.reset();
        self.classifier.reset();
        self.history.clear();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Episode aggregation
// ─────────────────────────────────────────────────────────────────────────────

/// A maximal contiguous run of identical grammar states on one channel (run-length element).
///
/// Episodes are the primary unit of analysis in DSFB forensic reports: they compress the raw
/// step sequence into a compact list of named intervals, each characterised by its dominant
/// grammar state and peak signal magnitudes. Multiple consecutive steps in the same state
/// collapse into a single episode, reducing the data volume while preserving the causal
/// structure.
///
/// `drift_sign` is +1.0 or −1.0 (finalised by `aggregate_episodes` after all steps are seen)
/// and encodes the net direction of drift accumulation over the episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Grammar state that all steps in this run share.
    pub state: GrammarState,
    pub channel: String,
    /// Timestamp of the first step in the run.
    pub start_ts: f64,
    /// Timestamp of the last step in the run (equals `start_ts` for single-step episodes).
    pub end_ts: f64,
    /// Number of steps in the run.
    pub step_count: usize,
    /// Maximum absolute value of r seen within this episode (tracks worst-case raw deviation).
    pub peak_r: f64,
    /// Maximum absolute value of δ seen within this episode.
    pub peak_delta: f64,
    /// Maximum absolute value of σ seen within this episode.
    pub peak_sigma: f64,
    /// Net drift direction: +1.0 if the running sum of δ over the episode is ≥ 0, else −1.0.
    pub drift_sign: f64,
    /// Reason code from the first step that opened this episode.
    pub reason: ReasonCode,
}

impl Episode {
    /// Elapsed time from the first to the last step in the episode (same units as timestamp).
    pub fn duration(&self) -> f64 {
        self.end_ts - self.start_ts
    }
}

/// Compress a flat step sequence into a list of contiguous-run episodes (run-length encoding).
///
/// Iterates `steps` once in order. When the current step has the same `(state, channel)` pair
/// as the open episode, it extends that episode (updating `end_ts`, `step_count`, and peaks).
/// When the pair changes, a new episode is opened. This single-pass, ordered traversal is what
/// makes the output deterministic: episode identity depends only on the input order.
///
/// `drift_sign` is accumulated as a running sum of δ values during the pass and then
/// finalised to ±1.0 in a second pass over the output vector. This two-phase approach avoids
/// a sqrt/sign operation per step and produces a single canonical net direction per episode.
///
/// Non-finite values for r, δ, σ are excluded from peak comparisons (treated as 0) so that
/// occasional `SensorFault` steps do not corrupt episode-level statistics.
pub fn aggregate_episodes(steps: &[AnnotatedStep]) -> Vec<Episode> {
    let mut out: Vec<Episode> = Vec::new();
    for s in steps {
        let r = s.triple.r;
        let d = s.triple.delta;
        let sg = s.triple.sigma;
        if let Some(last) = out.last_mut() {
            if last.state == s.state && last.channel == s.channel {
                // Extend the open episode.
                last.end_ts = s.triple.timestamp;
                last.step_count += 1;
                // Update peaks using absolute magnitude comparisons; skip non-finite values.
                if r.is_finite() && r.abs() > last.peak_r.abs() {
                    last.peak_r = r;
                }
                if d.is_finite() && d.abs() > last.peak_delta.abs() {
                    last.peak_delta = d;
                }
                if sg.is_finite() && sg.abs() > last.peak_sigma.abs() {
                    last.peak_sigma = sg;
                }
                // Accumulate δ into running sum for net drift direction (finalised below).
                if d.is_finite() {
                    last.drift_sign += d;
                }
                continue;
            }
        }
        // State or channel changed: open a new episode.
        out.push(Episode {
            state: s.state,
            channel: s.channel.clone(),
            start_ts: s.triple.timestamp,
            end_ts: s.triple.timestamp,
            step_count: 1,
            peak_r: if r.is_finite() { r } else { 0.0 },
            peak_delta: if d.is_finite() { d } else { 0.0 },
            peak_sigma: if sg.is_finite() { sg } else { 0.0 },
            drift_sign: if d.is_finite() { d } else { 0.0 },
            reason: s.reason,
        });
    }
    // Second pass: collapse the running sum into a ±1.0 sign.
    for e in &mut out {
        e.drift_sign = if e.drift_sign >= 0.0 { 1.0 } else { -1.0 };
    }
    out
}

/// Scalar summary of one channel's complete annotation run.
///
/// Provides the first-level digest metrics reported in the paper: compression ratio
/// (`episode_count_collapse`), fractional availability (`nominal_fraction`), and the
/// per-state step histogram (`by_state`). `BTreeMap` is used for `by_state` to guarantee
/// deterministic serialisation order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeSummary {
    pub channel: String,
    pub total_steps: usize,
    pub total_episodes: usize,
    pub nominal_steps: usize,
    /// Number of episodes whose state is non-nominal (excludes Nominal episodes).
    pub non_nominal_episodes: usize,
    /// `total_steps / total_episodes`: average episode length in steps.
    ///
    /// A value of 1.0 means every step is its own episode (no repetition); higher values
    /// indicate fewer, longer episodes and thus greater compression of the raw alarm stream.
    pub episode_count_collapse: f64,
    /// Fraction of steps in the Nominal state: `nominal_steps / total_steps`.
    pub nominal_fraction: f64,
    /// Step count broken down by grammar state (BTreeMap for deterministic ordering).
    pub by_state: BTreeMap<GrammarState, usize>,
}

/// Build an `EpisodeSummary` from a flat step sequence and a pre-computed episode list.
///
/// `steps` and `episodes` must be derived from the same annotation run on `channel`; there
/// is no internal consistency check. Both are iterated once each, so the total cost is O(N).
pub fn summarise(channel: &str, steps: &[AnnotatedStep], episodes: &[Episode]) -> EpisodeSummary {
    let mut by_state: BTreeMap<GrammarState, usize> = BTreeMap::new();
    let mut nominal_steps = 0usize;
    for s in steps {
        *by_state.entry(s.state).or_insert(0) += 1;
        if s.state.is_nominal() {
            nominal_steps += 1;
        }
    }
    let total_steps = steps.len();
    let total_episodes = episodes.len();
    let non_nominal_episodes = episodes.iter().filter(|e| e.state.is_non_nominal()).count();
    EpisodeSummary {
        channel: channel.to_string(),
        total_steps,
        total_episodes,
        nominal_steps,
        non_nominal_episodes,
        // Guard against division by zero on empty episode lists.
        episode_count_collapse: if total_episodes > 0 {
            total_steps as f64 / total_episodes as f64
        } else {
            0.0
        },
        nominal_fraction: if total_steps > 0 {
            nominal_steps as f64 / total_steps as f64
        } else {
            0.0
        },
        by_state,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Non-interference contract + deterministic replay
// ─────────────────────────────────────────────────────────────────────────────

/// Type-system enforcer of the read-only non-interference contract.
///
/// Wraps an owned `Vec<T>` and exposes only shared (`&[T]`) access, making it impossible
/// at the type level for DSFB processing code to mutate the input data. This is a
/// documentation and enforcement device: the pipeline would also be read-only if it held
/// `&[T]`, but `ReadOnlySlice` makes the intent explicit in function signatures and prevents
/// accidental future additions of `&mut` access.
pub struct ReadOnlySlice<T> {
    inner: Vec<T>,
}

impl<T> ReadOnlySlice<T> {
    /// Wrap an owned vector; no data is copied.
    pub fn wrap(data: Vec<T>) -> Self {
        ReadOnlySlice { inner: data }
    }
    /// Shared slice view — the only access path to the data.
    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// Feed a `ReadOnlySlice` into any consumer that accepts `&[T]`.
///
/// The callback `f` receives only a shared reference, enforcing that the source data cannot
/// be modified by the consumer even if the consumer holds a mutable closure environment.
pub fn process_read_only<T, R, F: FnOnce(&[T]) -> R>(source: &ReadOnlySlice<T>, f: F) -> R {
    f(source.as_slice())
}

/// Documentation marker for the non-intrusive guarantee.
///
/// Carrying this struct (or its `DESCRIPTION` constant) in reports makes the architectural
/// constraint machine-readable: DSFB is a passive observer that writes nothing upstream.
pub struct NonIntrusiveGuarantee;
impl NonIntrusiveGuarantee {
    pub const DESCRIPTION: &'static str = "DSFB is a read-only observer. It writes to no upstream register, setpoint, alarm limit, \
         historian tag, or control variable. Its removal restores the pre-deployment baseline exactly.";
}

/// Replay a recorded residual sequence and return the grammar state for each step.
///
/// Constructs a fresh `DeterministicDsfb` engine from the given parameters and processes the
/// entire `samples` slice in order. Because all state is initialised identically every call,
/// and there is no RNG or non-deterministic iteration, identical `(samples, envelope,
/// drift_window)` triples always produce identical output — byte-exact replay is guaranteed.
pub fn deterministic_replay(
    samples: &[ResidualSample],
    envelope: AdmissibilityEnvelope,
    drift_window: usize,
) -> Vec<GrammarState> {
    let mut eng = DeterministicDsfb::with_window(envelope, drift_window, "replay");
    samples.iter().map(|s| eng.ingest_sample(s).state).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_zero_is_always_nominal() {
        let mut eng =
            DeterministicDsfb::with_window(AdmissibilityEnvelope::symmetric(3.0, 0.1), 5, "x");
        for i in 0..50 {
            let s = ResidualSample::new(i as f64, 0.0, 0.0, "x");
            assert_eq!(eng.ingest_sample(&s).state, GrammarState::Nominal);
        }
    }

    #[test]
    fn nan_emits_sensor_fault_without_poisoning() {
        let mut eng =
            DeterministicDsfb::with_window(AdmissibilityEnvelope::symmetric(3.0, 0.1), 5, "x");
        for i in 0..5 {
            eng.ingest_sample(&ResidualSample::new(i as f64, 0.0, 0.0, "x"));
        }
        let sf = eng.ingest_sample(&ResidualSample::new(5.0, f64::NAN, 0.0, "x"));
        assert_eq!(sf.state, GrammarState::SensorFault);
        let next = eng.ingest_sample(&ResidualSample::new(6.0, 0.0, 0.0, "x"));
        assert_ne!(next.state, GrammarState::SensorFault);
        assert!(next.triple.delta.is_finite());
    }

    #[test]
    fn replay_is_deterministic() {
        let samples: Vec<_> = (0..40)
            .map(|i| {
                let v = if i == 20 {
                    100.0
                } else {
                    (i as f64 * 0.3).sin()
                };
                ResidualSample::new(i as f64, v, 0.0, "x")
            })
            .collect();
        let env = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        assert_eq!(
            deterministic_replay(&samples, env, 5),
            deterministic_replay(&samples, env, 5)
        );
    }
}
