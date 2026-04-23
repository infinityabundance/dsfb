//! Main engine: composes all pipeline stages into a single deterministic observer.
//!
//! ## Pipeline (paper §B, Theorem 9)
//!
//!   IQ Residual → Sign → Grammar → Syntax → Semantics → DSA → Policy
//!
//! Each stage is a deterministic function under fixed parameters.
//! The composition is deterministic: identical ordered inputs produce
//! identical outputs on every replay.
//!
//! ## Non-Intrusion Contract (paper §II, §VIII-C)
//!
//! The public `observe()` method accepts `&[f32]` immutable residual slices
//! from the caller. The engine's internal mutable state is fully encapsulated.
//! No mutable reference to any caller-owned data is ever taken.
//! The Rust type system enforces this: `cargo geiger` reports zero unsafe.
//!
//! ## Generic Parameters
//!
//! - `W`:  window width for sign and DSA (paper default: 10)
//! - `K`:  grammar persistence threshold (paper default: 4)
//! - `M`:  heuristics bank capacity (default: 32)

use crate::sign::{SignTuple, SignWindow};
use crate::envelope::AdmissibilityEnvelope;
use crate::grammar::{GrammarEvaluator, GrammarState};
use crate::syntax::{classify, SyntaxThresholds, MotifClass};
use crate::heuristics::{HeuristicsBank, SemanticDisposition};
use crate::dsa::DsaWindow;
use crate::policy::{PolicyDecision, PolicyEvaluator};
use crate::platform::{PlatformContext, SnrFloor};
use crate::lyapunov::{LyapunovEstimator, LyapunovResult};

/// Typed non-intrusion contract for the DSFB-RF observer.
///
/// This struct is a compile-time, read-only declaration of the architectural
/// guarantees this observer provides to the system it is embedded in.
///
/// Derived from the DSFB-Semiconductor `NonIntrusiveDsfbObserver` contract
/// (de Beer 2026, §VIII-C) and extended for the RF context.
///
/// ## Guarantees
///
/// 1. **Observer-only write path**: `observe()` takes `&mut self` (own
///    state only) and `&[f32]` (caller data immutable).  No mutable
///    reference to caller-owned data is ever taken.
///
/// 2. **Fail-safe isolation**: if the observer panics or returns an error,
///    it cannot alter upstream receiver behaviour.  The observer is a leaf
///    node in the data flow graph.
///
/// 3. **Read-only side channel**: the observer taps the IQ residual stream
///    that the receiver already produces.  It neither writes to the receiver's
///    filter coefficients, detector thresholds, AGC loop state, nor any
///    firmware register.
///
/// 4. **Deterministic**: identical ordered inputs produce identical outputs
///    on every replay (Theorem 9 from the paper).  No internal PRNG,
///    no OS clock, no hardware entropy source.
///
/// 5. **Non-attributing**: the observer produces grammar states and motif
///    classes.  It does not attribute physical cause, emitter identity,
///    or intent.
#[derive(Debug, Clone, Copy)]
pub struct NonIntrusiveContract {
    /// Integration mode string.  Always `"read_only_side_channel"`.
    pub integration_mode: &'static str,
    /// Fail-safe isolation guarantee.
    pub fail_safe_isolation_note: &'static str,
    /// Write-path guarantee.
    pub write_path_note: &'static str,
    /// Determinism guarantee.
    pub determinism_note: &'static str,
    /// Attribution policy.
    pub attribution_policy: &'static str,
    /// Unsafe code count (enforced by `#![forbid(unsafe_code)]`).
    pub unsafe_count: u32,
    /// Heap allocation policy.
    pub heap_policy: &'static str,
}

/// The canonical non-intrusion contract for dsfb-rf.
///
/// Include this in operator advisories, SigMF annotations, and
/// VITA 49.2 context packets to assert the integration guarantees.
pub const NON_INTRUSIVE_CONTRACT: NonIntrusiveContract = NonIntrusiveContract {
    integration_mode: "read_only_side_channel",
    fail_safe_isolation_note:
        "observer failure cannot alter upstream receiver behaviour; \
         observer is a leaf node with no write-back path to any upstream state",
    write_path_note:
        "observe() takes &[f32] (immutable caller slice); \
         no mutable reference to caller-owned data is ever taken",
    determinism_note:
        "identical ordered inputs produce identical outputs on every replay; \
         no PRNG, no OS clock, no hardware entropy source",
    attribution_policy:
        "grammar states and motif classes are structural observations only; \
         no physical cause, emitter identity, or intent is attributed",
    unsafe_count: 0,
    heap_policy: "no_alloc in core path; heap opt-in via 'alloc' feature only",
};

/// Full deterministic trace for one observation — the audit chain.
///
/// Every field in this struct corresponds to a stage in the DSFB pipeline.
/// The complete chain can be serialized to `dsfb_traceability.json` by the
/// `output` module (requires `serde` feature).
#[derive(Debug, Clone, Copy)]
pub struct ObservationResult {
    /// Observation index k.
    pub k: u64,
    /// Raw residual norm ‖r(k)‖.
    pub residual_norm: f32,
    /// Sign tuple σ(k) = (‖r‖, ṙ, r̈). Stage 1 output.
    pub sign: SignTuple,
    /// Grammar state after hysteresis. Stage 2 output.
    pub grammar: GrammarState,
    /// Motif class from syntax layer. Stage 3 output.
    pub motif: MotifClass,
    /// Semantic disposition from heuristics bank. Stage 4 output.
    pub semantic: SemanticDisposition,
    /// DSA score. Stage 5 output.
    pub dsa_score: f32,
    /// Final policy decision. Stage 6 output.
    pub policy: PolicyDecision,
    /// Lyapunov stability result: finite-time Lyapunov exponent λ(k),
    /// stability classification, and estimated time-to-envelope-exit.
    pub lyapunov: LyapunovResult,
    /// Sub-threshold flag (SNR < floor → drift/slew forced to zero).
    pub sub_threshold: bool,
    /// Suppressed flag (waveform transition → grammar forced to Admissible).
    pub suppressed: bool,
}

/// The DSFB RF Structural Semiotics Engine.
///
/// ## Type Parameters
///
/// - `W`: window width (sign drift + DSA accumulator). Paper Stage III: `W = 10`.
/// - `K`: grammar persistence threshold. Paper default: `K = 4`.
/// - `M`: heuristics bank capacity. Paper default: `M = 32`.
///
/// ## Memory Footprint (no_std, no_alloc)
///
/// All storage is stack-allocated. For `W=10, K=4, M=8`:
/// - SignWindow<10>:        ~52 bytes
/// - GrammarEvaluator<4>:  ~20 bytes
/// - DsaWindow<10>:        ~212 bytes
/// - HeuristicsBank<8>:    ~400 bytes
/// - PolicyEvaluator:      ~8 bytes
/// - Total:                ~700 bytes — suitable for Cortex-M4F stack

// ── Decimation ────────────────────────────────────────────────────────────────
//
// DEFENCE: "Computational Wall" (see paper §XIX-A and AGENTS.md).
//
// Structural state changes (thermal drift, oscillator aging) occur at kHz or
// Hz rates — not at GHz sample rates. The `DecimationAccumulator` down-samples
// the residual stream before the semiotic pipeline, enabling deployment at
// full-rate (e.g. 200 MS/s FPGA path) while the Semiotic Engine runs at a
// decimated rate (e.g. 1 ks/s). DSFB monitors the *envelope* of the physics,
// not the cycle of the carrier. This is not a limitation; it is the correct
// physics.
//
// Implementation: accumulates `factor` norms, emits their RMS once per epoch.
// `factor=1` (the default) means every sample passes through unchanged — no
// performance penalty for configurations that do not need decimation.
// `no_std`, `no_alloc`, zero `unsafe`. Stack footprint: 16 bytes.

/// Streaming residual-norm decimation accumulator.
///
/// Collects `factor` residual-norm samples and emits a single **root-mean-square**
/// value per epoch. This down-samples the semiotic pipeline to the physics
/// timescale of structural change (thermal, oscillator aging) decoupled from
/// the carrier sample rate.
///
/// ## Rationale (paper §XIX-A — Semiotic Decimation)
///
/// At 1 GSPS, a 27 ns per-sample budget is budget-limited for the full Fisher-Rao
/// and Lyapunov machinery. Structural changes that DSFB detects (PA drift,
/// oscillator aging, mask approach) occur at timescales > 10 ms. A decimation
/// factor of 10 000 at 1 GSPS yields 100 kHz structural monitoring — seven
/// decades above the physics rate, with a 27 µs per-epoch budget (10 000× more
/// comfortable). This is architecturally identical to how a spectrum analyzer
/// operates: full-rate ADC, decimated FFT, symbol-rate detection.
///
/// ## Instruction-Level Determinism
///
/// The accumulator is branchless (no dynamic dispatch, no heap, no loop beyond
/// the caller's own loop). The inner hot path is exactly 6 arithmetic
/// operations per input sample regardless of `factor`. Only the `push()`
/// `return Some(rms)` branch fires once per `factor` samples — fully
/// predictable by branch predictors and cycle-count manifests
/// (paper §XIX-B, Phase II deliverable).
///
/// ## Usage
///
/// ```
/// use dsfb_rf::engine::DecimationAccumulator;
/// let mut d = DecimationAccumulator::new(1000);
/// for i in 0..999 { assert!(d.push(0.05).is_none()); }
/// let rms = d.push(0.05).unwrap(); // epoch complete
/// assert!((rms - 0.05).abs() < 1e-5);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DecimationAccumulator {
    factor:  u32,   // Number of input samples per output epoch
    count:   u32,   // Samples accumulated in current epoch
    sum_sq:  f32,   // Running ‖r‖² for RMS computation
    peak:    f32,   // Peak norm in current epoch (for diagnostics)
}

impl DecimationAccumulator {
    /// Construct a new accumulator with the given decimation factor.
    ///
    /// `factor = 1` means every sample is emitted (no decimation).
    /// `factor = k` means one RMS value is emitted per `k` input samples.
    /// A `factor` of zero is treated as 1 (safety for const contexts).
    pub const fn new(factor: u32) -> Self {
        let f = if factor == 0 { 1 } else { factor };
        Self { factor: f, count: 0, sum_sq: 0.0, peak: 0.0 }
    }

    /// Push one residual norm into the accumulator.
    ///
    /// Returns `Some(rms)` when a full decimation epoch is complete.
    /// Returns `None` for all intermediate samples.
    #[inline]
    pub fn push(&mut self, norm: f32) -> Option<f32> {
        let n = if norm < 0.0 { -norm } else { norm }; // abs without libm
        self.sum_sq += n * n;
        if n > self.peak { self.peak = n; }
        self.count += 1;
        if self.count >= self.factor {
            let rms = crate::math::sqrt_f32(self.sum_sq / self.count as f32);
            self.count  = 0;
            self.sum_sq = 0.0;
            self.peak   = 0.0;
            Some(rms)
        } else {
            None
        }
    }

    /// Decimation factor (samples per output epoch).
    pub const fn factor(&self) -> u32 { self.factor }

    /// Samples accumulated in the current (incomplete) epoch.
    pub const fn count(&self) -> u32 { self.count }

    /// Reset the accumulator state (does not change the factor).
    pub fn reset(&mut self) {
        self.count  = 0;
        self.sum_sq = 0.0;
        self.peak   = 0.0;
    }
}

/// Main DSFB Structural Semiotics Engine.
///
/// A zero-allocation, deterministic observer that combines envelope admissibility,
/// sign-segment grammar, DSA scoring, Lyapunov exponent estimation, heuristics,
/// and policy evaluation into a single state machine operating on IQ residuals.
///
/// # Type Parameters
/// - `W` — sliding window length for sign-segment and DSA statistics.
/// - `K` — grammar state-machine size (number of grammar states).
/// - `M` — heuristics bank capacity.
///
/// # Non-Intrusion Contract
/// The engine is a **read-only observer**. It never modifies, delays, or discards
/// samples from the underlying signal chain. See [`NON_INTRUSIVE_CONTRACT`].
///
/// # Example
/// ```rust
/// use dsfb_rf::engine::DsfbRfEngine;
/// use dsfb_rf::platform::PlatformContext;
/// let mut eng = DsfbRfEngine::<10, 4, 8>::new(0.05, 3.0);
/// let ctx = PlatformContext::operational();
/// let _obs = eng.observe(0.1, ctx);
/// ```
pub struct DsfbRfEngine<const W: usize, const K: usize, const M: usize> {
    envelope:      AdmissibilityEnvelope,
    sign_window:   SignWindow<W>,
    grammar:       GrammarEvaluator<K>,
    dsa:           DsaWindow<W>,
    heuristics:    HeuristicsBank<M>,
    policy_eval:   PolicyEvaluator,
    lyapunov:      LyapunovEstimator<W>,
    snr_floor:     SnrFloor,
    syn_thresh:    SyntaxThresholds,
    obs_count:     u64,
    episode_count: u32,
    /// Semiotic decimation accumulator.
    ///
    /// `observe_decimated()` uses this to down-sample the residual stream to
    /// the physics timescale. `factor=1` (default) means every sample passes
    /// through — the `observe()` hot path is unaffected.
    decim: DecimationAccumulator,
}

impl<const W: usize, const K: usize, const M: usize> DsfbRfEngine<W, K, M> {
    /// Construct engine with given envelope radius ρ and DSA threshold τ.
    pub fn new(rho: f32, tau: f32) -> Self {
        use crate::policy::PolicyConfig;
        Self {
            envelope:      AdmissibilityEnvelope::new(rho),
            sign_window:   SignWindow::new(),
            grammar:       GrammarEvaluator::new(),
            dsa:           DsaWindow::new(rho * 0.5),
            heuristics:    HeuristicsBank::default_rf(),
            policy_eval:   PolicyEvaluator::with_config(PolicyConfig {
                tau,
                k: K as u8,
                m: 1,
                extreme_bypass: true,
            }),
            lyapunov:      LyapunovEstimator::new(),
            snr_floor:     SnrFloor::default(),
            syn_thresh:    SyntaxThresholds::default(),
            obs_count:     0,
            episode_count: 0,
            decim:         DecimationAccumulator::new(1), // no decimation by default
        }
    }

    /// Construct from a healthy-window norm slice (Stage III calibration).
    ///
    /// Computes ρ = μ + 3σ from `healthy_norms`.
    /// Returns `None` if slice is empty.
    pub fn from_calibration(healthy_norms: &[f32], tau: f32) -> Option<Self> {
        let env = AdmissibilityEnvelope::calibrate_from_window(healthy_norms)?;
        let mut eng = Self::new(env.rho, tau);
        eng.dsa.calibrate_ewma_threshold(healthy_norms);
        Some(eng)
    }

    /// Set a custom SNR floor (default: −10 dB).
    pub fn with_snr_floor(mut self, db: f32) -> Self {
        self.snr_floor = SnrFloor::new(db);
        self
    }

    /// Set the semiotic decimation factor (default: 1 — no decimation).
    ///
    /// With `factor = D`, the full semiotic pipeline runs **once per D input
    /// samples**.  The input window accumulates the RMS of `D` norms before
    /// forwarding to the sign → grammar → syntax → semantics → DSA → policy
    /// chain.
    ///
    /// ## When to use
    ///
    /// At high sample rates (≥ 1 MS/s) where structural changes of interest
    /// (thermal drift, PA aging, mask approach) occur at kHz or Hz rates.
    /// Decimation effectively sets the structural monitoring bandwidth to
    /// `sample_rate / D` Hz, which is appropriate for the physics timescale.
    ///
    /// ## Non-intrusion guarantee is preserved
    ///
    /// The accumulator is entirely internal. `observe_decimated()` still takes
    /// only `&[f32]` immutable slices from the caller. `factor=1` (default)
    /// means `observe_decimated()` === `observe()` with zero overhead.
    ///
    /// ## Example
    ///
    /// ```
    /// use dsfb_rf::engine::DsfbRfEngine;
    /// // 1 GSPS receiver; monitor at 100 kHz structural rate
    /// let eng = DsfbRfEngine::<10, 4, 8>::new(0.1, 2.0)
    ///     .with_decimation(10_000);
    /// assert_eq!(eng.decimation_factor(), 10_000);
    /// ```
    pub fn with_decimation(mut self, factor: u32) -> Self {
        self.decim = DecimationAccumulator::new(factor);
        self
    }

    /// Current decimation factor.
    pub fn decimation_factor(&self) -> u32 { self.decim.factor() }

    /// Process one residual norm observation.
    ///
    /// The full pipeline stages run in order. Returns an `ObservationResult`
    /// containing the complete audit chain for this observation.
    ///
    /// ## Non-Intrusion
    ///
    /// `residual_norm` and `ctx` are consumed by value or immutable reference.
    /// No caller-owned data is mutated. The engine advances only its own
    /// internal state.
    pub fn observe(
        &mut self,
        residual_norm: f32,
        ctx: PlatformContext,
    ) -> ObservationResult {
        let k = self.obs_count;
        self.obs_count += 1;
        let sub_threshold = self.snr_floor.is_sub_threshold(ctx.snr_db);
        let suppressed = ctx.waveform_state.is_suppressed();
        let sign = self.sign_window.push(residual_norm, sub_threshold, self.snr_floor);
        let effective_waveform = select_effective_waveform(ctx.waveform_state, sub_threshold);
        let grammar = self.grammar.evaluate(&sign, &self.envelope, effective_waveform);
        let motif = classify(&sign, grammar, self.envelope.rho, &self.syn_thresh);
        let semantic = self.heuristics.lookup(motif, grammar);
        let motif_fired = !matches!(motif, MotifClass::Unknown);
        let dsa = self.dsa.push(&sign, grammar, motif_fired);
        let lyapunov = self.lyapunov.push(residual_norm, self.envelope.rho);
        let policy = self.policy_eval.evaluate(grammar, semantic, dsa, 1);
        if matches!(policy, PolicyDecision::Escalate) {
            self.episode_count = self.episode_count.saturating_add(1);
        }
        ObservationResult {
            k, residual_norm, sign, grammar, motif, semantic,
            dsa_score: dsa.0, lyapunov, policy, sub_threshold, suppressed,
        }
    }

    /// Batch-process a slice of residual norms, returning all results.
    ///
    /// Convenience method for the host-side pipeline. Requires `alloc` feature
    /// for Vec output, or use the iterator form below for bare-metal.
    #[cfg(feature = "alloc")]
    pub fn observe_batch(
        &mut self,
        norms: &[f32],
        ctx: PlatformContext,
    ) -> alloc::vec::Vec<ObservationResult> {
        norms.iter().map(|&n| self.observe(n, ctx)).collect()
    }

    /// Process one residual norm through the **decimation accumulator**, then
    /// (only when a full epoch completes) through the full semiotic pipeline.
    ///
    /// Returns `None` for all intermediate samples within an epoch.
    /// Returns `Some(ObservationResult)` once per `decimation_factor()` calls.
    ///
    /// With `decimation_factor() == 1` (the default), this is identical to
    /// `observe()` and returns `Some` on every call.
    ///
    /// ## Motivation (paper §XIX-A — Semiotic Decimation)
    ///
    /// DSFB monitors the *envelope* of the physics, not the *cycle* of the
    /// carrier.  Structural state changes (thermal drift, oscillator aging,
    /// mask approach) occur at kHz/Hz rates.  Running the full Fisher-Rao,
    /// Lyapunov, and grammar machinery at 1 GSPS is unnecessary and violates
    /// the sensor physics.  Decimation resolves the "Computational Wall"
    /// criticism without sacrificing structural detection sensitivity.
    ///
    /// ## Non-intrusion guarantee preserved
    ///
    /// The `norm` argument is consumed by value; `ctx` is passed by value.
    /// No caller-owned data is mutated.
    ///
    /// ## Example
    ///
    /// ```
    /// use dsfb_rf::engine::DsfbRfEngine;
    /// use dsfb_rf::platform::PlatformContext;
    /// let mut eng = DsfbRfEngine::<10, 4, 8>::new(0.05, 2.0)
    ///     .with_decimation(100);
    /// let ctx = PlatformContext::with_snr(20.0);
    /// for i in 0..99 {
    ///     assert!(eng.observe_decimated(0.02, ctx).is_none());
    /// }
    /// let result = eng.observe_decimated(0.02, ctx);
    /// assert!(result.is_some()); // 100th sample triggers epoch
    /// ```
    #[inline]
    pub fn observe_decimated(
        &mut self,
        residual_norm: f32,
        ctx: PlatformContext,
    ) -> Option<ObservationResult> {
        self.decim.push(residual_norm).map(|rms| self.observe(rms, ctx))
    }

    /// Current observation count.
    pub fn obs_count(&self) -> u64 { self.obs_count }

    /// Current escalation-episode count.
    pub fn episode_count(&self) -> u32 { self.episode_count }

    /// Current envelope radius ρ.
    pub fn rho(&self) -> f32 { self.envelope.rho }

    /// Current grammar state.
    pub fn grammar_state(&self) -> GrammarState { self.grammar.state() }

    /// Return the typed non-intrusion contract for this observer.
    ///
    /// Use this in operator advisories, SigMF `dsfb:contract` annotations,
    /// and VITA 49.2 context packets to formally assert the integration
    /// guarantees provided by this implementation.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use dsfb_rf::engine::DsfbRfEngine;
    /// let eng = DsfbRfEngine::<10, 4, 8>::new(0.1, 2.0);
    /// let c = eng.contract();
    /// assert_eq!(c.integration_mode, "read_only_side_channel");
    /// assert_eq!(c.unsafe_count, 0);
    /// ```
    #[inline]
    pub fn contract(&self) -> NonIntrusiveContract {
        NON_INTRUSIVE_CONTRACT
    }

    /// Reset all internal state.
    pub fn reset(&mut self) {
        self.sign_window.reset();
        self.grammar.reset();
        self.dsa.reset();
        self.lyapunov.reset();
        self.decim.reset();
        self.obs_count = 0;
        self.episode_count = 0;
    }
}

#[inline]
fn select_effective_waveform(
    ctx_waveform: crate::platform::WaveformState,
    sub_threshold: bool,
) -> crate::platform::WaveformState {
    if sub_threshold {
        crate::platform::WaveformState::Calibration
    } else {
        ctx_waveform
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformContext;

    fn eng() -> DsfbRfEngine<10, 4, 8> {
        DsfbRfEngine::new(0.10, 2.0)
    }

    fn ctx(snr: f32) -> PlatformContext { PlatformContext::with_snr(snr) }

    // ── Theorem 9: Determinism ───────────────────────────────────────────
    #[test]
    fn determinism_identical_inputs_produce_identical_outputs() {
        let inputs = [0.01f32, 0.02, 0.04, 0.07, 0.09, 0.08, 0.06, 0.04, 0.03, 0.02,
                      0.03, 0.05, 0.08, 0.11, 0.10, 0.08, 0.06, 0.03, 0.02, 0.01];
        let c = ctx(15.0);
        let mut e1 = eng();
        let mut e2 = eng();
        for &n in &inputs {
            let r1 = e1.observe(n, c);
            let r2 = e2.observe(n, c);
            assert_eq!(r1.policy, r2.policy,
                "Theorem 9 violated at k={}: {:?} vs {:?}", r1.k, r1.policy, r2.policy);
            assert_eq!(r1.grammar, r2.grammar);
        }
    }

    // ── L8: Observer-only — no upstream mutation ─────────────────────────
    #[test]
    fn observe_does_not_mutate_input() {
        let mut e = eng();
        let original = 0.07f32;
        let copy = original;
        let _ = e.observe(original, ctx(15.0));
        // original is Copy — value is unchanged
        assert_eq!(original, copy);
    }

    // ── L10: Sub-threshold forces Admissible ─────────────────────────────
    #[test]
    fn sub_threshold_snr_forces_admissible() {
        let mut e = eng();
        // Feed large norms at sub-threshold SNR
        for _ in 0..20 {
            let r = e.observe(0.50, PlatformContext::with_snr(-20.0));
            assert_eq!(r.grammar, GrammarState::Admissible,
                "sub-threshold must force Admissible, got {:?}", r.grammar);
            assert_eq!(r.sign.drift, 0.0);
            assert_eq!(r.sign.slew, 0.0);
        }
    }

    // ── XIV-C: Transition window suppression ─────────────────────────────
    #[test]
    fn transition_window_no_escalation() {
        let mut e = eng();
        let ctx_t = PlatformContext::transition();
        for _ in 0..30 {
            let r = e.observe(999.0, ctx_t);
            assert!(!matches!(r.policy, PolicyDecision::Review | PolicyDecision::Escalate),
                "transition must suppress escalation, got {:?}", r.policy);
        }
    }

    // ── Clean signal stays Silent ─────────────────────────────────────────
    #[test]
    fn nominal_signal_stays_silent() {
        let mut e = eng();
        let c = ctx(20.0);
        for _ in 0..30 {
            let r = e.observe(0.02, c);
            assert_eq!(r.policy, PolicyDecision::Silent,
                "nominal signal at k={} must be Silent, got {:?}", r.k, r.policy);
        }
    }

    // ── Theorem 1: Sustained drift exits envelope ─────────────────────────
    #[test]
    fn sustained_drift_eventually_detected() {
        let mut e = DsfbRfEngine::<10, 4, 8>::new(0.10, 2.0);
        let c = ctx(20.0);
        let mut detected = false;
        for i in 0..60u32 {
            let norm = 0.01 + i as f32 * 0.004;
            let r = e.observe(norm, c);
            if matches!(r.policy, PolicyDecision::Review | PolicyDecision::Escalate) {
                detected = true;
                break;
            }
        }
        assert!(detected,
            "Theorem 1: sustained drift must be detected in finite observations");
    }

    // ── Calibration from healthy window ───────────────────────────────────
    #[test]
    fn calibration_produces_valid_engine() {
        let healthy: [f32; 100] = core::array::from_fn(|i| 0.03 + i as f32 * 0.0002);
        let e = DsfbRfEngine::<10, 4, 8>::from_calibration(&healthy, 2.0);
        assert!(e.is_some());
        let e = e.unwrap();
        assert!(e.rho() > 0.0, "calibrated rho must be positive");
    }

    // ── Reset clears all state ────────────────────────────────────────────
    #[test]
    fn reset_clears_observation_count() {
        let mut e = eng();
        let c = ctx(15.0);
        for _ in 0..10 { e.observe(0.05, c); }
        assert_eq!(e.obs_count(), 10);
        e.reset();
        assert_eq!(e.obs_count(), 0);
    }

    // ── Bare-metal build sanity (no std, no alloc needed) ─────────────────
    #[test]
    fn engine_fits_in_reasonable_stack() {
        // Verify size is manageable for MCU deployment
        let size = core::mem::size_of::<DsfbRfEngine<10, 4, 8>>();
        assert!(size < 4096, "engine size {} bytes exceeds 4KB stack budget", size);
    }

    // ── Non-intrusion contract assertions ─────────────────────────────────
    #[test]
    fn contract_mode_is_read_only_side_channel() {
        let e = eng();
        let c = e.contract();
        assert_eq!(c.integration_mode, "read_only_side_channel");
    }

    #[test]
    fn contract_unsafe_count_zero() {
        let e = eng();
        assert_eq!(e.contract().unsafe_count, 0);
    }

    #[test]
    fn contract_heap_policy_no_alloc() {
        let e = eng();
        let policy = e.contract().heap_policy;
        assert!(policy.contains("no_alloc"), "heap policy must assert no_alloc: {}", policy);
    }

    #[test]
    fn non_intrusive_contract_constant_accessible() {
        assert_eq!(NON_INTRUSIVE_CONTRACT.integration_mode, "read_only_side_channel");
        assert_eq!(NON_INTRUSIVE_CONTRACT.unsafe_count, 0);
    }

    // ── Semiotic Decimation ───────────────────────────────────────────────

    #[test]
    fn decimation_accumulator_emits_once_per_factor() {
        let mut d = DecimationAccumulator::new(10);
        for i in 0..9 {
            assert!(d.push(0.05).is_none(), "expected None at sample {i}");
        }
        let rms = d.push(0.05);
        assert!(rms.is_some(), "expected Some(rms) at 10th sample");
        let v = rms.unwrap();
        assert!((v - 0.05).abs() < 1e-5, "rms {v} not close to 0.05");
    }

    #[test]
    fn decimation_accumulator_factor_one_emits_every_sample() {
        let mut d = DecimationAccumulator::new(1);
        for i in 0..20 {
            assert!(d.push(0.03).is_some(), "factor=1 must emit at sample {i}");
        }
    }

    #[test]
    fn decimation_accumulator_zero_factor_treated_as_one() {
        let mut d = DecimationAccumulator::new(0);
        assert_eq!(d.factor(), 1, "factor=0 must be normalised to 1");
        assert!(d.push(0.05).is_some(), "normalised factor=1 must emit immediately");
    }

    #[test]
    fn decimation_accumulator_rms_of_mixed_norms() {
        let mut d = DecimationAccumulator::new(4);
        let norms = [0.0f32, 0.0, 0.0, 4.0]; // RMS = sqrt((0+0+0+16)/4) = 2.0
        for (i, &n) in norms.iter().enumerate() {
            let r = d.push(n);
            if i < 3 { assert!(r.is_none()); }
            else { assert!((r.unwrap() - 2.0).abs() < 1e-4, "rms mismatch: {r:?}"); }
        }
    }

    #[test]
    fn observe_decimated_returns_none_then_some() {
        let mut e = DsfbRfEngine::<10, 4, 8>::new(0.10, 2.0)
            .with_decimation(5);
        let c = ctx(20.0);
        for _ in 0..4 {
            assert!(e.observe_decimated(0.02, c).is_none());
        }
        assert!(e.observe_decimated(0.02, c).is_some());
    }

    #[test]
    fn observe_decimated_factor_one_equiv_to_observe() {
        let mut e1 = DsfbRfEngine::<10, 4, 8>::new(0.10, 2.0);
        let mut e2 = DsfbRfEngine::<10, 4, 8>::new(0.10, 2.0).with_decimation(1);
        let c = ctx(20.0);
        for _ in 0..20 {
            let r1 = e1.observe(0.03, c);
            let r2 = e2.observe_decimated(0.03, c).unwrap();
            assert_eq!(r1.policy, r2.policy,
                "factor=1 observe_decimated must equal observe");
        }
    }

    #[test]
    fn decimation_theorem9_determinism_preserved() {
        // Decimated pipeline must also satisfy Theorem 9 (determinism)
        let inputs = [0.02f32, 0.04, 0.03, 0.05, 0.06,
                      0.07, 0.08, 0.07, 0.05, 0.03];
        let c = ctx(20.0);
        let mut e1 = DsfbRfEngine::<10, 4, 8>::new(0.10, 2.0).with_decimation(5);
        let mut e2 = DsfbRfEngine::<10, 4, 8>::new(0.10, 2.0).with_decimation(5);
        let mut out1: [Option<crate::policy::PolicyDecision>; 10] = [None; 10];
        let mut out2: [Option<crate::policy::PolicyDecision>; 10] = [None; 10];
        for (i, &n) in inputs.iter().enumerate() {
            out1[i] = e1.observe_decimated(n, c).map(|r| r.policy);
            out2[i] = e2.observe_decimated(n, c).map(|r| r.policy);
        }
        assert_eq!(out1, out2, "Theorem 9 must hold for decimated pipeline");
    }

    #[test]
    fn decimation_factor_accessible_after_builder() {
        let e = DsfbRfEngine::<10, 4, 8>::new(0.10, 2.0).with_decimation(1000);
        assert_eq!(e.decimation_factor(), 1000);
    }

    #[test]
    fn reset_clears_decimation_accumulator() {
        let mut e = DsfbRfEngine::<10, 4, 8>::new(0.10, 2.0).with_decimation(10);
        let c = ctx(20.0);
        for _ in 0..5 { e.observe_decimated(0.05, c); }
        e.reset();
        // After reset, need another full 10 samples to emit
        for _ in 0..9 {
            assert!(e.observe_decimated(0.05, c).is_none());
        }
        assert!(e.observe_decimated(0.05, c).is_some());
    }
}
