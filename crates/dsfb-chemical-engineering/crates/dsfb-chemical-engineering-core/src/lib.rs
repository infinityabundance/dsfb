//! # dsfb-chemical-engineering-core
//!
//! A **`no_std`, no-heap, fixed-point** embedded core implementing the DSFB residual grammar — the part of
//! the chemical-engineering monitor that can run on a microcontroller next to the process, with **bounded,
//! statically-known memory** and **fully deterministic integer arithmetic** (no floating point, no
//! allocation, no `std`). Being dependency-free and `no_std`, the same core builds for both
//! **`thumbv7m-none-eabi`** (embedded; QEMU-verified — see `qemu-smoke/`) and **`wasm32-unknown-unknown`**
//! (the substrate for an in-browser court simulator — the Wave-7 WASM handoff).
//!
//! It mirrors the state machine of the edge crate's `dsfb_core` (`Nominal` / `DriftAccum` / `SlewSpike` /
//! `EnvViolation` / `BoundaryGrazing` / `Recovery` / `Compound` / `SensorFault`) over a residual triple
//! `(r, δ, σ)`, but in **scaled integers** (everything ×[`SCALE`]) so the classification is exact and
//! reproducible on any target with no FPU. The drift δ is a windowed mean of `r` over a const-generic ring
//! buffer (sustained accumulation); the slew σ is the first difference of `r` (rapid transient). Envelope
//! classification (`Interior` / `Grazing` / `Outside`) is done by exact integer comparison — no division in
//! the hot path, the grazing test promoted to `i128` to avoid overflow.
//!
//! ## What this crate deliberately is NOT
//! - **Not bit-identical to the edge float pipeline.** It is an embedded sibling using fixed-point and a
//!   simpler drift definition; the edge crate (and its replay-hash gate) remain the reference. Treat the two
//!   as the same *grammar*, calibrated independently — not as a cross-checked numeric pair.
//! - **Not a controller and not a safety function.** Read-only, advisory; emits a grammar token per sample,
//!   nothing else. No actuation, no SIS/IEC-61511 authority (the whole DSFB stack's standing boundary).
//! - **Not allocation- or panic-prone.** `#![forbid(unsafe_code)]`, no heap, fixed-capacity buffers; the hot
//!   path performs only integer add/sub/compare and one integer divide (the windowed mean).
//!
//! Under `cargo test` the crate links `std` (the default harness); for the target it is `#![no_std]`.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

/// Fixed-point scale: every engineering value is represented as `round(value × SCALE)` in an `i64`. Chosen
/// to match the CUDA evidence kernel's `SCALE = 1e6` determinism convention so the *numbers* are comparable
/// across the stack even though the embedded core is not claimed bit-identical to the float pipeline.
pub const SCALE: i64 = 1_000_000;

/// A fixed-point residual triple: raw residual `r`, drift `δ`, slew `σ`, each scaled by [`SCALE`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedTriple {
    pub r: i64,
    pub delta: i64,
    pub sigma: i64,
}

/// Per-axis classification of a value against an envelope axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordClass {
    /// Strictly inside: `|ṽ| < 1 − band`.
    Interior,
    /// In the grazing band but not breaching: `1 − band ≤ |ṽ| ≤ 1`.
    Grazing,
    /// Beyond the boundary: `|ṽ| > 1`.
    Outside,
}

/// Classify a scaled value `v` against the scaled axis bounds `[lo, hi]` with a grazing band `band_scaled`
/// (a fraction of the half-width, itself scaled by [`SCALE`], so `band_scaled ∈ [0, SCALE)`).
///
/// Exact integer logic, no division: working in doubled coordinates `d2 = 2v − (hi+lo)` and `width = hi − lo`,
/// - `Outside`  ⇔ `|d2| > width`
/// - `Grazing`  ⇔ `|d2|·SCALE ≥ width·(SCALE − band_scaled)` (promoted to `i128` so the product never overflows)
/// - `Interior` otherwise.
pub fn classify_axis(v: i64, lo: i64, hi: i64, band_scaled: i64) -> CoordClass {
    let d2 = 2 * (v as i128) - (hi as i128 + lo as i128);
    let width = hi as i128 - lo as i128; // caller guarantees hi > lo, so width > 0
    let ad2 = d2.abs();
    if ad2 > width {
        CoordClass::Outside
    } else if ad2 * SCALE as i128 >= width * (SCALE as i128 - band_scaled as i128) {
        CoordClass::Grazing
    } else {
        CoordClass::Interior
    }
}

/// A fixed-point admissibility envelope: scaled bounds on each axis plus a scaled grazing-band fraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedEnvelope {
    pub r_min: i64,
    pub r_max: i64,
    pub delta_min: i64,
    pub delta_max: i64,
    pub sigma_min: i64,
    pub sigma_max: i64,
    /// Grazing-band fraction scaled by [`SCALE`] (e.g. `0.1` → `100_000`); must be in `[0, SCALE)`.
    pub band_scaled: i64,
}

impl FixedEnvelope {
    /// Symmetric envelope for a zero-centred residual of scale `k` (scaled), mirroring the edge crate's
    /// ratios (r: ±k, δ: ±0.6k, σ: ±2k): drift is time-averaged (tighter), slew is a first difference (wider).
    /// These ratios are heuristic, not physics-derived — calibrate per channel for engineering-unit residuals.
    pub const fn symmetric(k_scaled: i64, band_scaled: i64) -> Self {
        FixedEnvelope {
            r_min: -k_scaled,
            r_max: k_scaled,
            delta_min: -(k_scaled * 3 / 5), // 0.6k
            delta_max: k_scaled * 3 / 5,
            sigma_min: -(k_scaled * 2),
            sigma_max: k_scaled * 2,
            band_scaled,
        }
    }

    /// Classify a triple against the envelope.
    pub fn eval(&self, t: &FixedTriple) -> EnvelopeEval {
        EnvelopeEval {
            r_class: classify_axis(t.r, self.r_min, self.r_max, self.band_scaled),
            delta_class: classify_axis(t.delta, self.delta_min, self.delta_max, self.band_scaled),
            sigma_class: classify_axis(t.sigma, self.sigma_min, self.sigma_max, self.band_scaled),
        }
    }
}

/// Per-axis classification result of evaluating a triple against an envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvelopeEval {
    pub r_class: CoordClass,
    pub delta_class: CoordClass,
    pub sigma_class: CoordClass,
}

impl EnvelopeEval {
    pub fn r_violated(&self) -> bool {
        self.r_class == CoordClass::Outside
    }
    pub fn delta_violated(&self) -> bool {
        self.delta_class == CoordClass::Outside
    }
    pub fn sigma_violated(&self) -> bool {
        self.sigma_class == CoordClass::Outside
    }
    /// No axis is `Outside` but at least one is `Grazing` (mirrors the edge `any_grazing` guard).
    pub fn any_grazing(&self) -> bool {
        !self.r_violated()
            && !self.delta_violated()
            && !self.sigma_violated()
            && (self.r_class == CoordClass::Grazing
                || self.delta_class == CoordClass::Grazing
                || self.sigma_class == CoordClass::Grazing)
    }
}

/// DSFB grammar state (mirrors the edge crate's `GrammarState`). One token is emitted per sample; the
/// selection order in [`GrammarClassifier::classify`] is what disambiguates overlapping breaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarState {
    /// Interior on every axis with no prior disturbance — the process is behaving.
    Nominal,
    /// The windowed-mean drift δ breached its envelope: a slow, *sustained* departure (bias, fouling, ramp).
    DriftAccum,
    /// The first-difference slew σ breached: a *rapid* transient (step, spike, valve kick).
    SlewSpike,
    /// The raw residual r breached its envelope: a gross level violation (takes priority over δ/σ-only).
    EnvViolation,
    /// No breach, but at least one axis sits in the grazing band — an early-warning margin state.
    BoundaryGrazing,
    /// First interior sample after any non-`Nominal` state — emitted *once*, then decays to `Nominal`.
    Recovery,
    /// Both δ and σ breach simultaneously — a compound disturbance (sustained + transient together).
    Compound,
    /// The reading was flagged invalid (the fixed-point analogue of a non-finite float); memory left untouched.
    SensorFault,
}

impl GrammarState {
    /// Stable 2-letter token (identical to the edge crate's tokens).
    pub fn token(self) -> &'static str {
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
    pub fn is_nominal(self) -> bool {
        matches!(self, GrammarState::Nominal)
    }
}

/// Reason code attached to a classification (a subset of the edge crate's, carrying drift/slew *direction*
/// so a downstream reader can tell an upward drift from a downward one without re-deriving it from the triple).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasonCode {
    /// Paired with `Nominal` — no disturbance.
    Nominal,
    /// `DriftAccum` with δ ≥ 0 — upward drift.
    DriftPositive,
    /// `DriftAccum` with δ < 0 — downward drift.
    DriftNegative,
    /// `SlewSpike` with σ ≥ 0 — upward transient.
    SlewPositive,
    /// `SlewSpike` with σ < 0 — downward transient.
    SlewNegative,
    /// `EnvViolation` — the raw residual r is out of bounds.
    Violation,
    /// `BoundaryGrazing` — at least one axis is within the grazing band.
    Grazing,
    /// `Recovery` — first interior step after a disturbance.
    Recovery,
    /// `Compound` — δ and σ breach together.
    Compound,
    /// `SensorFault` — invalid / out-of-band reading.
    OobSensor,
}

/// One-step-memory grammar classifier (mirrors the edge `GrammarClassifier`).
#[derive(Debug, Clone, Copy)]
pub struct GrammarClassifier {
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

    /// Classify an evaluated triple. `valid = false` signals a bad sensor reading (the fixed-point analogue
    /// of a non-finite float): emit `SensorFault` without touching the one-step memory. The state-selection
    /// order is identical to the edge crate's.
    pub fn classify(
        &mut self,
        eval: &EnvelopeEval,
        t: &FixedTriple,
        valid: bool,
    ) -> (GrammarState, ReasonCode) {
        use GrammarState::*;
        if !valid {
            return (SensorFault, ReasonCode::OobSensor);
        }
        let new_state = if eval.delta_violated() && eval.sigma_violated() {
            Compound
        } else if eval.r_violated() {
            EnvViolation
        } else if eval.sigma_violated() {
            SlewSpike
        } else if eval.delta_violated() {
            DriftAccum
        } else if eval.any_grazing() {
            BoundaryGrazing
        } else if self.prev_state != Nominal {
            Recovery
        } else {
            Nominal
        };
        let reason = match new_state {
            Nominal => ReasonCode::Nominal,
            DriftAccum => {
                if t.delta >= 0 {
                    ReasonCode::DriftPositive
                } else {
                    ReasonCode::DriftNegative
                }
            }
            SlewSpike => {
                if t.sigma >= 0 {
                    ReasonCode::SlewPositive
                } else {
                    ReasonCode::SlewNegative
                }
            }
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

    pub fn reset(&mut self) {
        self.prev_state = GrammarState::Nominal;
    }
}

/// A fixed-capacity ring buffer of scaled residuals — the only "memory" the core keeps, with statically
/// known size `N` (no heap). Tracks a running sum so the windowed mean is O(1).
#[derive(Debug, Clone, Copy)]
pub struct RingBuffer<const N: usize> {
    buf: [i64; N],
    len: usize,
    head: usize,
    sum: i64,
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        RingBuffer {
            buf: [0; N],
            len: 0,
            head: 0,
            sum: 0,
        }
    }
}

impl<const N: usize> RingBuffer<N> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a value, evicting the oldest once full. Maintains the running `sum`.
    pub fn push(&mut self, v: i64) {
        if self.len == N {
            self.sum -= self.buf[self.head];
        } else {
            self.len += 1;
        }
        self.buf[self.head] = v;
        self.sum += v;
        // `N.max(1)` guards the degenerate `N == 0` case so the modulo can never divide by zero (a
        // zero-capacity ring is meaningless but must not panic on a `no_std`/`panic="abort"` target).
        self.head = (self.head + 1) % N.max(1);
    }

    /// Windowed mean (scaled), via integer division of the running sum by the current length. 0 when empty.
    pub fn mean(&self) -> i64 {
        if self.len == 0 {
            0
        } else {
            self.sum / self.len as i64
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// The full single-channel embedded engine: ring buffer (drift) + previous sample (slew) + envelope +
/// classifier, all stack-allocated with statically-known size `N`. One instance per monitored channel.
#[derive(Debug, Clone, Copy)]
pub struct DsfbCore<const N: usize> {
    env: FixedEnvelope,
    ring: RingBuffer<N>,
    prev_r: Option<i64>,
    classifier: GrammarClassifier,
}

impl<const N: usize> DsfbCore<N> {
    /// Create a channel engine with the given fixed-point envelope and a window of `N` samples.
    pub fn new(env: FixedEnvelope) -> Self {
        DsfbCore {
            env,
            ring: RingBuffer::new(),
            prev_r: None,
            classifier: GrammarClassifier::new(),
        }
    }

    /// Process one scaled residual sample and return its grammar state + reason. `valid = false` signals a
    /// bad reading (the fixed-point analogue of a non-finite float): emits `SensorFault` and leaves the ring,
    /// the previous sample, and the classifier memory untouched. Otherwise:
    /// - the drift δ is the windowed mean of `r` (sustained accumulation),
    /// - the slew σ is `r − previous r` (rapid transient; 0 on the first sample).
    pub fn step(&mut self, r_scaled: i64, valid: bool) -> (GrammarState, ReasonCode) {
        if !valid {
            let dummy = FixedTriple {
                r: 0,
                delta: 0,
                sigma: 0,
            };
            let bad = EnvelopeEval {
                r_class: CoordClass::Interior,
                delta_class: CoordClass::Interior,
                sigma_class: CoordClass::Interior,
            };
            return self.classifier.classify(&bad, &dummy, false);
        }
        self.ring.push(r_scaled);
        let delta = self.ring.mean();
        let sigma = match self.prev_r {
            Some(p) => r_scaled - p,
            None => 0,
        };
        self.prev_r = Some(r_scaled);
        let t = FixedTriple {
            r: r_scaled,
            delta,
            sigma,
        };
        let eval = self.env.eval(&t);
        self.classifier.classify(&eval, &t, true)
    }

    /// Reset to the initial state (empty window, no previous sample, Nominal memory) for a fresh replay.
    pub fn reset(&mut self) {
        self.ring = RingBuffer::new();
        self.prev_r = None;
        self.classifier.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience: a value `x` in engineering units → scaled fixed-point.
    fn fx(x: f64) -> i64 {
        (x * SCALE as f64).round() as i64
    }

    #[test]
    fn classify_axis_boundaries_are_exact() {
        // Axis [-1, 1] (scaled), 10% grazing band. Half-width = 1; interior threshold |v| < 0.9.
        let (lo, hi, band) = (fx(-1.0), fx(1.0), fx(0.1));
        assert_eq!(classify_axis(fx(0.0), lo, hi, band), CoordClass::Interior);
        assert_eq!(classify_axis(fx(0.89), lo, hi, band), CoordClass::Interior);
        assert_eq!(classify_axis(fx(0.95), lo, hi, band), CoordClass::Grazing);
        assert_eq!(classify_axis(fx(1.0), lo, hi, band), CoordClass::Grazing); // at the boundary, not outside
        assert_eq!(classify_axis(fx(1.01), lo, hi, band), CoordClass::Outside);
        assert_eq!(classify_axis(fx(-1.5), lo, hi, band), CoordClass::Outside);
    }

    #[test]
    fn nominal_zero_centred_signal_stays_nominal() {
        let mut core = DsfbCore::<8>::new(FixedEnvelope::symmetric(fx(3.0), fx(0.1)));
        // Small zero-centred wiggles: r, mean, and first-diff all stay well inside the envelope.
        for &x in &[0.1, -0.1, 0.05, -0.05, 0.1, 0.0, -0.1, 0.05] {
            let (state, _) = core.step(fx(x), true);
            assert!(state.is_nominal(), "expected NOM, got {}", state.token());
        }
    }

    #[test]
    fn sustained_drift_triggers_drift_accum() {
        // δ envelope is ±0.6·3 = ±1.8. A signal parked at +2.5 makes the windowed mean exceed 1.8 → DriftAccum
        // (raw r 2.5 < 3.0 so r does not breach; the slow mean does).
        let mut core = DsfbCore::<4>::new(FixedEnvelope::symmetric(fx(3.0), fx(0.1)));
        let mut saw_drift = false;
        for _ in 0..8 {
            let (state, reason) = core.step(fx(2.5), true);
            if state == GrammarState::DriftAccum {
                saw_drift = true;
                assert_eq!(reason, ReasonCode::DriftPositive);
            }
        }
        assert!(
            saw_drift,
            "a sustained +2.5 residual must drive the windowed mean past the δ envelope"
        );
    }

    #[test]
    fn isolated_spike_triggers_slew_spike() {
        // Quiet, then a single big jump: the first difference σ breaches (±2·3 = ±6) while the windowed mean
        // (one big sample among many) stays inside the δ envelope → SlewSpike.
        let mut core = DsfbCore::<16>::new(FixedEnvelope::symmetric(fx(3.0), fx(0.1)));
        for _ in 0..10 {
            core.step(fx(0.0), true);
        }
        let (state, reason) = core.step(fx(8.0), true); // jump of +8 > σ_max 6 (and r 8 > r_max 3)
                                                        // r also breaches here (8 > 3) so the state is EnvViolation, not SlewSpike — verify the engine picks
                                                        // the r-violation branch, matching the edge ordering.
        assert_eq!(state, GrammarState::EnvViolation);
        assert_eq!(reason, ReasonCode::Violation);
        // A jump that breaches σ but not r: from 0 to 2.5 is +2.5 < σ_max 6, no spike. Construct a true
        // slew-only case with a wider r envelope.
        let mut core2 = DsfbCore::<16>::new(FixedEnvelope {
            r_min: fx(-100.0),
            r_max: fx(100.0),
            ..FixedEnvelope::symmetric(fx(3.0), fx(0.1))
        });
        for _ in 0..10 {
            core2.step(fx(0.0), true);
        }
        let (s2, r2) = core2.step(fx(8.0), true); // +8 breaches σ (±6) but r 8 < 100
        assert_eq!(s2, GrammarState::SlewSpike);
        assert_eq!(r2, ReasonCode::SlewPositive);
    }

    #[test]
    fn invalid_reading_is_sensor_fault_and_preserves_memory() {
        let mut core = DsfbCore::<4>::new(FixedEnvelope::symmetric(fx(3.0), fx(0.1)));
        core.step(fx(0.0), true);
        let before = core.ring.len();
        let (state, reason) = core.step(0, false); // bad reading
        assert_eq!(state, GrammarState::SensorFault);
        assert_eq!(reason, ReasonCode::OobSensor);
        assert_eq!(
            core.ring.len(),
            before,
            "a sensor fault must not push into the ring"
        );
        assert!(
            core.prev_r.is_some(),
            "a sensor fault must not clobber the previous sample"
        );
    }

    #[test]
    fn recovery_emits_once_after_a_breach() {
        // Wide r envelope + a large window (N=16), so a single +8 spike breaches only σ (the first
        // difference), not r and not the windowed mean. The sequence after a quiet fill is then exactly:
        //   spike σ=+8 → SlewSpike ; next σ=−8 → SlewSpike ; next σ=0 (interior) → Recovery ; next → Nominal.
        let mut core = DsfbCore::<16>::new(FixedEnvelope {
            r_min: fx(-100.0),
            r_max: fx(100.0),
            ..FixedEnvelope::symmetric(fx(3.0), fx(0.1))
        });
        for _ in 0..16 {
            assert!(core.step(fx(0.0), true).0.is_nominal());
        }
        assert_eq!(core.step(fx(8.0), true).0, GrammarState::SlewSpike); // σ = +8 breaches (±6)
        assert_eq!(core.step(fx(0.0), true).0, GrammarState::SlewSpike); // σ = 0−8 = −8 still breaches
        assert_eq!(core.step(fx(0.0), true).0, GrammarState::Recovery); // σ = 0, re-enter the interior
        assert_eq!(core.step(fx(0.0), true).0, GrammarState::Nominal); // Recovery is emitted only once
    }

    #[test]
    fn ring_buffer_is_bounded_and_means_correctly() {
        let mut rb = RingBuffer::<3>::new();
        for v in [10, 20, 30, 40] {
            rb.push(v);
        }
        assert_eq!(rb.len(), 3); // bounded at N=3 (10 evicted)
        assert_eq!(rb.mean(), (20 + 30 + 40) / 3);
    }

    #[test]
    fn engine_is_deterministic_across_resets() {
        let env = FixedEnvelope::symmetric(fx(3.0), fx(0.1));
        let seq = [0.1, 2.5, 2.5, 8.0, 0.0, -0.1, 0.0];
        let run = |mut c: DsfbCore<8>| -> [&'static str; 7] {
            let mut out = ["", "", "", "", "", "", ""];
            for (i, &x) in seq.iter().enumerate() {
                out[i] = c.step(fx(x), true).0.token();
            }
            out
        };
        let a = run(DsfbCore::<8>::new(env));
        let b = run(DsfbCore::<8>::new(env));
        assert_eq!(
            a, b,
            "identical inputs must yield identical token sequences"
        );
    }
}
