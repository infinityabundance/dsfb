//! `ControllerMaskingHeuristicV2` + `ValveStictionWitnessV1` (P62).
//!
//! Two formal fault-mechanism objects that sharpen the H6 (controller-compensation masking) and F1
//! (valve stiction) precursors into hash-sealed, multi-signal witnesses.
//!
//! - [`ControllerMaskingHeuristicV2`] — H6 fired on a single condition. V2 is the **conjunction of four
//!   independent signals** that together make masking suspectable: the process value is *stable*, the
//!   manipulated variable is *drifting*, control *effort is rising*, and the *residual energy is rising*.
//!   A controller silently compensating for a developing fault shows exactly this — a quiet PV bought by
//!   ever-harder actuation. Requiring all four (not any one) is what separates masking from benign tuning.
//! - [`ValveStictionWitnessV1`] — F1's residual motif made a formal witness over four named stiction
//!   signatures (sawtooth, deadband, limit-cycle, PV-lag).
//!
//! Both are advisory candidates (never root cause), additive, and off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The four independent masking signals (each a boolean the caller derives from the relevant stream).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskingSignals {
    /// Process value is stable (low variance / no envelope breach) — the symptom is *hidden*.
    pub pv_stable: bool,
    /// Manipulated variable is drifting monotonically — the controller is moving to hold PV.
    pub mv_drift: bool,
    /// Control effort (|MV − MV_baseline| or integral action) is rising over the window.
    pub effort_rising: bool,
    /// Residual energy (SPE/Q) is rising even though PV looks fine — the latent fault leaking through.
    pub residual_energy_rising: bool,
}

impl MaskingSignals {
    /// Derive the four signals from the relevant streams over a window, using simple deterministic
    /// rules (variance for stability; net sign of the first→last change for drift/rising). `pv` stable
    /// iff its sample variance is below `pv_var_tol`; the others "rise/drift" iff last − first > 0.
    pub fn from_streams(
        pv: &[f64],
        mv: &[f64],
        effort: &[f64],
        residual_energy: &[f64],
        pv_var_tol: f64,
    ) -> Self {
        fn rising(xs: &[f64]) -> bool {
            match (xs.first(), xs.last()) {
                (Some(&a), Some(&b)) if a.is_finite() && b.is_finite() => b - a > 0.0,
                _ => false,
            }
        }
        fn variance(xs: &[f64]) -> f64 {
            let fin: Vec<f64> = xs.iter().copied().filter(|x| x.is_finite()).collect();
            if fin.len() < 2 {
                return 0.0;
            }
            let mean = fin.iter().sum::<f64>() / fin.len() as f64;
            fin.iter().map(|&x| (x - mean) * (x - mean)).sum::<f64>() / fin.len() as f64
        }
        MaskingSignals {
            pv_stable: variance(pv) <= pv_var_tol,
            mv_drift: rising(mv),
            effort_rising: rising(effort),
            residual_energy_rising: rising(residual_energy),
        }
    }

    /// How many of the four signals hold.
    pub fn count(self) -> u8 {
        self.pv_stable as u8
            + self.mv_drift as u8
            + self.effort_rising as u8
            + self.residual_energy_rising as u8
    }
}

/// A hash-sealed controller-masking verdict (schema v2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerMaskingHeuristicV2 {
    pub episode_ref: String,
    pub signals: MaskingSignals,
    pub n_signals: u8,
    /// Masking is *suspected* only when all four signals hold (conjunction) — the V2 sharpening of H6.
    pub masking_suspected: bool,
    pub rationale: String,
    pub verdict_hash: String,
}

impl ControllerMaskingHeuristicV2 {
    fn seal(
        episode_ref: &str,
        s: MaskingSignals,
        n: u8,
        suspected: bool,
        rationale: &str,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"controller_masking_heuristic_v2");
        h.field("episode_ref", episode_ref.as_bytes());
        h.u64("pv_stable", s.pv_stable as u64);
        h.u64("mv_drift", s.mv_drift as u64);
        h.u64("effort_rising", s.effort_rising as u64);
        h.u64("residual_energy_rising", s.residual_energy_rising as u64);
        h.u64("n_signals", n as u64);
        h.u64("masking_suspected", suspected as u64);
        h.field("rationale", rationale.as_bytes());
        h.finalize_hex()
    }

    /// Evaluate the V2 heuristic: masking is suspected iff all four signals hold.
    pub fn evaluate(episode_ref: impl Into<String>, signals: MaskingSignals) -> Self {
        let episode_ref = episode_ref.into();
        let n_signals = signals.count();
        let masking_suspected = n_signals == 4;
        let rationale = if masking_suspected {
            "PV stable ∧ MV drifting ∧ effort rising ∧ residual energy rising — a quiet PV bought by \
             rising actuation; controller-compensation masking is the candidate (advisory, not root cause)."
                .to_string()
        } else {
            format!(
                "{n_signals}/4 masking signals — below the conjunction threshold; masking not suspected \
                 (a single signal is consistent with benign tuning/disturbance)."
            )
        };
        let verdict_hash = Self::seal(
            &episode_ref,
            signals,
            n_signals,
            masking_suspected,
            &rationale,
        );
        ControllerMaskingHeuristicV2 {
            episode_ref,
            signals,
            n_signals,
            masking_suspected,
            rationale,
            verdict_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.episode_ref,
            self.signals,
            self.n_signals,
            self.masking_suspected,
            &self.rationale,
        ) == self.verdict_hash
    }
}

/// The four valve-stiction signatures (each a boolean the caller derives from the PV/OP streams).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StictionSignatures {
    /// Sawtooth in the PV residual (stick → ramp of OP → slip → PV jump).
    pub sawtooth: bool,
    /// Deadband: OP moves over a band with no PV response.
    pub deadband: bool,
    /// Sustained limit cycle (oscillation that does not decay).
    pub limit_cycle: bool,
    /// PV lags OP with a sharp catch-up slew after sustained drift.
    pub pv_lag: bool,
}

impl StictionSignatures {
    pub fn count(self) -> u8 {
        self.sawtooth as u8 + self.deadband as u8 + self.limit_cycle as u8 + self.pv_lag as u8
    }
}

/// A hash-sealed valve-stiction witness (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValveStictionWitnessV1 {
    pub episode_ref: String,
    pub signatures: StictionSignatures,
    pub n_signatures: u8,
    /// Stiction suspected when ≥ 2 of the 4 signatures co-occur (a single one has benign confusers).
    pub stiction_suspected: bool,
    pub witness_hash: String,
}

impl ValveStictionWitnessV1 {
    fn seal(episode_ref: &str, s: StictionSignatures, n: u8, suspected: bool) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"valve_stiction_witness_v1");
        h.field("episode_ref", episode_ref.as_bytes());
        h.u64("sawtooth", s.sawtooth as u64);
        h.u64("deadband", s.deadband as u64);
        h.u64("limit_cycle", s.limit_cycle as u64);
        h.u64("pv_lag", s.pv_lag as u64);
        h.u64("n_signatures", n as u64);
        h.u64("stiction_suspected", suspected as u64);
        h.finalize_hex()
    }

    pub fn build(episode_ref: impl Into<String>, signatures: StictionSignatures) -> Self {
        let episode_ref = episode_ref.into();
        let n_signatures = signatures.count();
        let stiction_suspected = n_signatures >= 2;
        let witness_hash = Self::seal(&episode_ref, signatures, n_signatures, stiction_suspected);
        ValveStictionWitnessV1 {
            episode_ref,
            signatures,
            n_signatures,
            stiction_suspected,
            witness_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.episode_ref,
            self.signatures,
            self.n_signatures,
            self.stiction_suspected,
        ) == self.witness_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masking_requires_all_four_signals() {
        let all = MaskingSignals {
            pv_stable: true,
            mv_drift: true,
            effort_rising: true,
            residual_energy_rising: true,
        };
        let v = ControllerMaskingHeuristicV2::evaluate("e", all);
        assert!(v.masking_suspected && v.n_signals == 4 && v.verify());
        let three = MaskingSignals {
            residual_energy_rising: false,
            ..all
        };
        let v3 = ControllerMaskingHeuristicV2::evaluate("e", three);
        assert!(!v3.masking_suspected && v3.n_signals == 3 && v3.verify());
    }

    #[test]
    fn masking_signals_from_streams() {
        // PV flat, MV/effort/residual all rising → all four signals.
        let pv = vec![1.0, 1.0, 1.0, 1.0];
        let mv = vec![0.0, 1.0, 2.0, 3.0];
        let effort = vec![0.0, 1.0, 2.0, 4.0];
        let re = vec![0.1, 0.2, 0.3, 0.5];
        let s = MaskingSignals::from_streams(&pv, &mv, &effort, &re, 1e-9);
        assert_eq!(s.count(), 4);
    }

    #[test]
    fn stiction_needs_two_signatures_and_self_verifies() {
        let two = StictionSignatures {
            sawtooth: true,
            deadband: false,
            limit_cycle: true,
            pv_lag: false,
        };
        let w = ValveStictionWitnessV1::build("idx=10..50", two);
        assert!(w.stiction_suspected && w.n_signatures == 2 && w.verify());
        let mut t = w.clone();
        t.stiction_suspected = false; // tamper
        assert!(!t.verify());
    }
}
