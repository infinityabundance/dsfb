//! Regime-switched admissibility envelopes.
//!
//! ## Theoretical basis
//!
//! A fixed-radius admissibility envelope ρ = const works well under
//! Wide-Sense Stationarity (WSS) — that is, while the nominal signal
//! regime is stable.  RF receivers routinely encounter **regime transitions**:
//!
//! - Preamble → data payload (burst-mode receivers)
//! - Acquisition → tracking (PLL lock transients, AGC settle)
//! - Idle → active (TDMA slot, radar duty cycle)
//! - Interference on → off (opportunistic spectrum sharing)
//!
//! The DSFB-Semiotics-Engine envelope module (de Beer 2026, §IV) models
//! five distinct envelope modes beyond the fixed baseline:
//!
//! | Mode | Physical RF scenario |
//! |---|---|
//! | Fixed | In-lock steady-state; nominal thermal-noise floor |
//! | Widening | Acquisition phase; PLL pull-in; AGC transient |
//! | Tightening | Post-fault recovery; channel-condition improvement |
//! | RegimeSwitched | Burst-mode: preamble vs payload; TDMA boundary |
//! | Aggregate | Worst-case across simultaneously active contexts |
//!
//! In addition, the semiotics-engine defines a **grammar trust scalar**
//! derived from the current boundary margin.  This is distinct from the
//! HRET channel-trust (which is residual-magnitude-based); the grammar trust
//! scalar is **geometry-based**, measuring how far inside the envelope the
//! current observation lies.
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - All state is stack-allocated `f32` scalars
//! - Widening / Tightening use EMA rate rather than open-loop ramp, so they
//!   are bounded and deterministic under any input sequence

use crate::envelope::AdmissibilityEnvelope;

/// Envelope operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EnvelopeMode {
    /// Fixed radius ρ = const.  Standard in-lock steady-state.
    Fixed,
    /// Widening: EMA-smoothed expansion toward ρ_max during acquisition /
    /// AGC transients.  Rate controlled by `widen_alpha` (EMA coefficient).
    Widening,
    /// Tightening: EMA-smoothed contraction toward ρ_base after a fault
    /// clears or channel conditions improve.
    Tightening,
    /// Regime-switched: the radius snaps between two pre-set levels depending
    /// on the active RF regime.  Maps naturally to burst-mode (preamble vs.
    /// payload) and TDMA boundary crossings.
    RegimeSwitched,
    /// Aggregate: takes the maximum of all `other_rho` values provided.
    /// Used when multiple envelope constraints are simultaneously active
    /// (e.g., regulatory mask + link-budget margin + observed-noise floor).
    Aggregate,
}

/// Regime labels for `RegimeSwitched` mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RfRegime {
    /// Burst preamble / synchronisation header — tolerates wider residuals.
    Preamble,
    /// Data payload — tighter envelope once lock is achieved.
    Payload,
    /// PLL acquisition / AGC settle — widest envelope.
    Acquisition,
    /// Steady-state in-lock — tightest envelope.
    InLock,
}

/// Parameters for a regime-switched envelope.
#[derive(Debug, Clone, Copy)]
pub struct RegimeEnvelopeParams {
    /// Base (tight) envelope radius ρ_base.
    pub rho_base: f32,
    /// Maximum (wide) envelope radius ρ_max used during widening mode or
    /// the "wide" regime in `RegimeSwitched`.
    pub rho_max: f32,
    /// EMA smoothing coefficient for widening (0 < α_widen < 1).
    /// Larger → faster widening.  Typical: 0.10.
    pub widen_alpha: f32,
    /// EMA smoothing coefficient for tightening (0 < α_tight < 1).
    /// Larger → faster tightening.  Typical: 0.05.
    pub tighten_alpha: f32,
    /// Boundary band fraction (semiotics-engine §IV: 4% of ρ).
    ///
    /// A sample within boundary_band of ρ_eff is classified as
    /// "boundary approach" for the grammar trust scalar.
    pub boundary_band_frac: f32,
    /// Slew threshold for abrupt slew detection as a fraction of ρ_eff.
    ///
    /// semiotics-engine default: 8% of ρ.
    pub slew_threshold_frac: f32,
}

impl RegimeEnvelopeParams {
    /// Sensible defaults for a standard SDR receiver.
    pub const fn default_sdr(rho_base: f32) -> Self {
        Self {
            rho_base,
            rho_max: rho_base * 3.0,
            widen_alpha: 0.10,
            tighten_alpha: 0.05,
            boundary_band_frac: 0.04,   // 4 % per semiotics-engine §IV
            slew_threshold_frac: 0.08,  // 8 % per semiotics-engine §IV
        }
    }
}

/// Grammar-level trust scalar derived from envelope geometry.
///
/// This is a **deterministic, bounded scalar in [0, 1]** that downweights
/// a grammar contribution based on how close the residual norm is to the
/// envelope boundary.
///
/// Definition (semiotics-engine `trust_scalar_for()`):
///
/// ```text
/// margin = (ρ_eff − ‖r‖) / ρ_eff        (normalised inward distance)
/// T = clamp(margin / boundary_band_frac, 0, 1)
/// ```
///
/// Interpretation:
/// - T = 1.0 → residual deep inside envelope; grammar evidence fully trusted
/// - T = 0.0 → residual on or outside envelope boundary; grammar evidence suppressed
/// - Intermediate → proportional attenuation by proximity
///
/// This is distinct from HRET channel trust, which is magnitude-EMA-based.
/// Grammar trust is a **per-sample geometric score** with no memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrammarTrustScalar {
    /// Trust value T ∈ [0, 1].
    pub value: f32,
    /// Normalised inward margin = (ρ − ‖r‖) / ρ.
    pub margin: f32,
}

impl GrammarTrustScalar {
    /// Compute the grammar trust scalar for a given norm, effective radius, and band width.
    ///
    /// `band_frac` is the boundary_band_frac (default 0.04).
    pub fn compute(norm: f32, rho_eff: f32, band_frac: f32) -> Self {
        if rho_eff <= 1e-30 {
            return Self { value: 0.0, margin: 0.0 };
        }
        let margin = (rho_eff - norm) / rho_eff;
        // T = margin / band_frac, clamped to [0, 1]
        let value = if band_frac < 1e-12 {
            if margin >= 0.0 { 1.0 } else { 0.0 }
        } else {
            let raw = margin / band_frac;
            raw.max(0.0).min(1.0)
        };
        Self { value, margin }
    }

    /// Returns true if the trust scalar indicates full confidence.
    #[inline]
    pub fn is_fully_trusted(&self) -> bool { self.value >= 1.0 - 1e-6 }

    /// Returns true if grammar evidence is fully suppressed.
    #[inline]
    pub fn is_suppressed(&self) -> bool { self.value <= 1e-6 }
}

/// Regime-sensitive admissibility envelope with dynamic radius tracking.
///
/// Wraps `AdmissibilityEnvelope` and adds:
/// 1. Mode-dependent radius updates (widening / tightening EMA)
/// 2. Regime switching (snap between ρ_base and ρ_max)
/// 3. Grammar trust scalar computation
/// 4. Aggregate-mode maximum over multiple constraints
///
/// ## Stack footprint: ~48 bytes (all f32 + enum tags)
pub struct RegimeEnvelope {
    /// Current effective radius ρ_eff (updated per observation).
    rho_eff: f32,
    /// Operating mode.
    mode: EnvelopeMode,
    /// Parameters.
    params: RegimeEnvelopeParams,
    /// Consecutive boundary-approach count for RecurrentBoundaryGrazing.
    /// Resets on each non-boundary observation.
    consecutive_boundary: u8,
    /// Whether an abrupt slew was detected on the last observation.
    last_slew: bool,
}

impl RegimeEnvelope {
    /// Construct with given parameters, starting in Fixed mode at ρ_base.
    pub const fn new(params: RegimeEnvelopeParams) -> Self {
        Self {
            rho_eff: params.rho_base,
            mode: EnvelopeMode::Fixed,
            params,
            consecutive_boundary: 0,
            last_slew: false,
        }
    }

    /// Construct directly from a base AdmissibilityEnvelope.
    pub fn from_envelope(env: &AdmissibilityEnvelope) -> Self {
        let params = RegimeEnvelopeParams::default_sdr(env.rho);
        Self::new(params)
    }

    /// Set a different operating mode.
    pub fn set_mode(&mut self, mode: EnvelopeMode) {
        self.mode = mode;
    }

    /// Current effective envelope radius ρ_eff.
    #[inline]
    pub fn rho_eff(&self) -> f32 { self.rho_eff }

    /// Current mode.
    #[inline]
    pub fn mode(&self) -> EnvelopeMode { self.mode }

    /// Update the envelope for one observation of residual norm.
    ///
    /// Adjusts ρ_eff according to the current mode, then computes and
    /// returns the grammar trust scalar.
    ///
    /// `other_rho` is only used in `Aggregate` mode (max over all provided
    /// values); pass an empty slice for other modes.
    pub fn update(
        &mut self,
        norm: f32,
        regime: RfRegime,
        other_rho: &[f32],
    ) -> EnvelopeUpdateResult {
        self.rho_eff = self.compute_rho_eff(regime, other_rho);

        let band = self.params.boundary_band_frac * self.rho_eff;
        let in_boundary_band = norm > (self.rho_eff - band).max(0.0) && norm <= self.rho_eff;
        let above_envelope = norm > self.rho_eff;
        if in_boundary_band {
            self.consecutive_boundary = self.consecutive_boundary.saturating_add(1);
        } else {
            self.consecutive_boundary = 0;
        }
        let recurrent_boundary_grazing = self.consecutive_boundary >= 2;

        let trust = GrammarTrustScalar::compute(norm, self.rho_eff, self.params.boundary_band_frac);
        EnvelopeUpdateResult {
            rho_eff: self.rho_eff,
            mode: self.mode,
            grammar_trust: trust,
            in_boundary_band,
            above_envelope,
            recurrent_boundary_grazing,
        }
    }

    fn compute_rho_eff(&self, regime: RfRegime, other_rho: &[f32]) -> f32 {
        match self.mode {
            EnvelopeMode::Fixed => self.params.rho_base,
            EnvelopeMode::Widening => {
                let a = self.params.widen_alpha;
                let r = a * self.params.rho_max + (1.0 - a) * self.rho_eff;
                r.max(self.params.rho_base).min(self.params.rho_max)
            }
            EnvelopeMode::Tightening => {
                let a = self.params.tighten_alpha;
                let r = a * self.params.rho_base + (1.0 - a) * self.rho_eff;
                r.max(self.params.rho_base).min(self.params.rho_max)
            }
            EnvelopeMode::RegimeSwitched => match regime {
                RfRegime::Preamble | RfRegime::Acquisition => self.params.rho_max,
                RfRegime::Payload | RfRegime::InLock => self.params.rho_base,
            },
            EnvelopeMode::Aggregate => {
                let mut max_rho = self.params.rho_base;
                for &r in other_rho {
                    if r > max_rho { max_rho = r; }
                }
                max_rho
            }
        }
    }

    /// Update with explicit delta_norm for slew detection.
    ///
    /// Returns `(EnvelopeUpdateResult, abrupt_slew)`.
    pub fn update_with_slew(
        &mut self,
        norm: f32,
        regime: RfRegime,
        other_rho: &[f32],
        delta_norm: f32,
    ) -> (EnvelopeUpdateResult, bool) {
        let result = self.update(norm, regime, other_rho);
        let slew_threshold = self.params.slew_threshold_frac * self.rho_eff;
        let abrupt_slew = delta_norm.abs() > slew_threshold;
        self.last_slew = abrupt_slew;
        (result, abrupt_slew)
    }

    /// Reset to initial state (Fixed mode, ρ_base).
    pub fn reset(&mut self) {
        self.rho_eff = self.params.rho_base;
        self.mode = EnvelopeMode::Fixed;
        self.consecutive_boundary = 0;
        self.last_slew = false;
    }
}

/// Result of one `RegimeEnvelope::update()` call.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeUpdateResult {
    /// Current effective envelope radius after mode update.
    pub rho_eff: f32,
    /// Mode that produced this result.
    pub mode: EnvelopeMode,
    /// Grammar trust scalar T ∈ [0, 1].
    pub grammar_trust: GrammarTrustScalar,
    /// True if the residual norm falls within the boundary band.
    ///
    /// Boundary band = (ρ_eff − 4%·ρ_eff, ρ_eff].
    pub in_boundary_band: bool,
    /// True if the residual norm is above ρ_eff (envelope violation).
    pub above_envelope: bool,
    /// True if ≥ 2 consecutive samples were in the boundary band.
    ///
    /// Corroborates `ReasonCode::RecurrentBoundaryGrazing` in the grammar layer.
    pub recurrent_boundary_grazing: bool,
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> RegimeEnvelopeParams {
        RegimeEnvelopeParams {
            rho_base: 0.10,
            rho_max: 0.30,
            widen_alpha: 0.20,
            tighten_alpha: 0.10,
            boundary_band_frac: 0.04,
            slew_threshold_frac: 0.08,
        }
    }

    #[test]
    fn fixed_mode_constant_rho() {
        let mut env = RegimeEnvelope::new(params());
        for _ in 0..50 {
            let r = env.update(0.05, RfRegime::InLock, &[]);
            assert!((r.rho_eff - 0.10).abs() < 1e-6);
        }
    }

    #[test]
    fn widening_mode_expands() {
        let mut env = RegimeEnvelope::new(params());
        env.set_mode(EnvelopeMode::Widening);
        let mut rho_prev = env.rho_eff();
        for _ in 0..30 {
            let r = env.update(0.05, RfRegime::Acquisition, &[]);
            assert!(r.rho_eff >= rho_prev - 1e-9, "rho must not decrease in widening mode");
            rho_prev = r.rho_eff;
        }
        assert!(rho_prev > 0.10, "rho should have grown above rho_base");
    }

    #[test]
    fn tightening_mode_contracts() {
        let mut env = RegimeEnvelope::new(params());
        env.rho_eff = 0.29; // start near max
        env.set_mode(EnvelopeMode::Tightening);
        let mut rho_prev = env.rho_eff();
        for _ in 0..40 {
            let r = env.update(0.05, RfRegime::InLock, &[]);
            assert!(r.rho_eff <= rho_prev + 1e-6, "rho must not increase in tightening mode");
            rho_prev = r.rho_eff;
        }
        assert!(rho_prev < 0.29, "rho should have contracted");
    }

    #[test]
    fn regime_switched_snaps() {
        let mut env = RegimeEnvelope::new(params());
        env.set_mode(EnvelopeMode::RegimeSwitched);

        let r_acq = env.update(0.05, RfRegime::Acquisition, &[]);
        assert!((r_acq.rho_eff - 0.30).abs() < 1e-6);

        let r_lock = env.update(0.05, RfRegime::InLock, &[]);
        assert!((r_lock.rho_eff - 0.10).abs() < 1e-6);
    }

    #[test]
    fn aggregate_mode_takes_max() {
        let mut env = RegimeEnvelope::new(params());
        env.set_mode(EnvelopeMode::Aggregate);
        let r = env.update(0.05, RfRegime::InLock, &[0.15, 0.25, 0.20]);
        assert!((r.rho_eff - 0.25).abs() < 1e-6);
    }

    #[test]
    fn grammar_trust_full_inside() {
        let p = params(); // boundary_band_frac = 0.04
        let mut env = RegimeEnvelope::new(p);
        // norm = 0.00 is deep inside → trust = 1
        let r = env.update(0.0, RfRegime::InLock, &[]);
        assert!(r.grammar_trust.is_fully_trusted());
    }

    #[test]
    fn grammar_trust_zero_at_boundary() {
        let p = params();
        let mut env = RegimeEnvelope::new(p);
        // norm = rho_eff = 0.10 → margin = 0 → trust = 0
        let r = env.update(0.10, RfRegime::InLock, &[]);
        assert!(r.grammar_trust.is_suppressed(), "trust={}", r.grammar_trust.value);
    }

    #[test]
    fn recurrent_boundary_grazing_after_two() {
        let mut env = RegimeEnvelope::new(params());
        // norm in boundary band: (0.096, 0.100]
        let r1 = env.update(0.098, RfRegime::InLock, &[]);
        assert!(!r1.recurrent_boundary_grazing, "only 1 sample — not recurring yet");
        let r2 = env.update(0.097, RfRegime::InLock, &[]);
        assert!(r2.recurrent_boundary_grazing, "2 consecutive should trigger grazing");
    }

    #[test]
    fn abrupt_slew_detection() {
        let mut env = RegimeEnvelope::new(params());
        // slew_threshold_frac = 0.08, rho_base = 0.10 → threshold = 0.008
        let (_, slew) = env.update_with_slew(0.05, RfRegime::InLock, &[], 0.001);
        assert!(!slew, "0.001 < 0.008: no slew");
        let (_, slew) = env.update_with_slew(0.05, RfRegime::InLock, &[], 0.02);
        assert!(slew, "0.02 > 0.008: abrupt slew detected");
    }
}
