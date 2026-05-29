//! `RegimeEnvelopeV1` — a named, hash-sealed regime-conditioned admissibility envelope (P57).
//!
//! The pipeline already *calibrates* a per-regime admissibility envelope
//! ([`crate::dsfb_core::calibrate_regime_envelope`], used by `fusion::run_detector_dsfb_regime`):
//! given a regime's baseline residual triples it **relaxes — never tightens** the global default so
//! the regime's own nominal variability falls inside its envelope. That is the *functionality*. This
//! module formalizes it into a citable **authority object**: the calibrated bounds, plus the metadata
//! describing *what* the envelope governs (regime / phase / variable-group / detector-family), plus a
//! **`provenance_hash`** sealing *exactly how it was derived* (the global default + the calibration
//! parameters + a baseline summary). A Court Record can then cite which regime envelope classified a
//! sample and a verifier can re-derive its hash byte-for-byte.
//!
//! This is additive and off the replay path: the sealed object records a derivation the pipeline
//! already performs; it changes no existing sealed artifact.

use serde::{Deserialize, Serialize};

use crate::dsfb_core::{calibrate_regime_envelope, AdmissibilityEnvelope, ResidualTriple};
use crate::hashing::CanonicalHasher;

/// How a regime's envelope relates to the global default — disclosed in the sealed record so a reader
/// knows whether the bounds were regime-calibrated or fell back to the global default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionPolicy {
    /// The per-regime envelope only ever **relaxes** the global default (never tightens) — the
    /// `calibrate_regime_envelope` invariant. Used when the regime had ≥ `min_samples` baseline triples.
    RelaxNeverTighten,
    /// The regime had fewer than `min_samples` baseline triples, so the global default is used unchanged.
    GlobalDefaultFallback,
}

impl TransitionPolicy {
    /// Stable tag folded into the provenance hash (decoupled from any `Debug`/`Display` formatting).
    fn tag(self) -> &'static str {
        match self {
            TransitionPolicy::RelaxNeverTighten => "relax_never_tighten",
            TransitionPolicy::GlobalDefaultFallback => "global_default_fallback",
        }
    }
}

/// The calibration provenance: the exact inputs that produced the bounds. Stored in the record so the
/// object is **self-verifying** — `verify()` re-derives `provenance_hash` from these fields alone.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RegimeCalibrationProvenance {
    /// Number of baseline triples available for this regime (the calibration sample size).
    pub n_baseline_triples: usize,
    /// `min_samples` threshold below which the global default is used unchanged.
    pub min_samples: usize,
    /// Upper nearest-rank quantile used to widen each axis.
    pub q_high: f64,
    /// The global default envelope the per-regime bounds relaxed (never tightened).
    pub default_envelope: AdmissibilityEnvelope,
}

/// A named, hash-sealed regime-conditioned admissibility envelope (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegimeEnvelopeV1 {
    /// Operating-regime / batch label this envelope governs (e.g. `"penicillin:batch=growth"`).
    pub regime_id: String,
    /// Process phase within the regime, or `""` if not phase-resolved.
    pub phase_id: String,
    /// Variable group this envelope governs (e.g. `"reactor_temperatures"`), or `""`.
    pub var_group: String,
    /// Detector family the residual stream belongs to (e.g. `"process_structure"`), or `""`.
    pub family: String,
    /// The calibrated admissibility envelope (a superset of the global default on every axis).
    pub bounds: AdmissibilityEnvelope,
    /// Whether the bounds were regime-calibrated or fell back to the global default.
    pub transition_policy: TransitionPolicy,
    /// The calibration inputs (stored so the record is self-verifying).
    pub calibration: RegimeCalibrationProvenance,
    /// SHA-256 (via [`CanonicalHasher`]) over the metadata + bounds + calibration provenance — the
    /// seal a Court Record cites and a verifier re-derives.
    pub provenance_hash: String,
}

impl RegimeEnvelopeV1 {
    /// Compute the provenance hash over every semantic field, in a fixed order. f64 fields go through
    /// the quantizing `f64q` hasher so the seal is independent of floating-point byte representation
    /// (the same discipline as the canonical replay hash).
    fn compute_hash(
        regime_id: &str,
        phase_id: &str,
        var_group: &str,
        family: &str,
        bounds: &AdmissibilityEnvelope,
        policy: TransitionPolicy,
        calib: &RegimeCalibrationProvenance,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"regime_envelope_v1");
        h.field("regime_id", regime_id.as_bytes());
        h.field("phase_id", phase_id.as_bytes());
        h.field("var_group", var_group.as_bytes());
        h.field("family", family.as_bytes());
        h.field("transition_policy", policy.tag().as_bytes());
        // Calibrated bounds.
        h.f64q("r_min", bounds.r_min);
        h.f64q("r_max", bounds.r_max);
        h.f64q("delta_min", bounds.delta_min);
        h.f64q("delta_max", bounds.delta_max);
        h.f64q("sigma_min", bounds.sigma_min);
        h.f64q("sigma_max", bounds.sigma_max);
        h.f64q("grazing_band", bounds.grazing_band);
        // Calibration provenance (how the bounds were derived).
        h.u64("n_baseline_triples", calib.n_baseline_triples as u64);
        h.u64("min_samples", calib.min_samples as u64);
        h.f64q("q_high", calib.q_high);
        h.f64q("default_r_min", calib.default_envelope.r_min);
        h.f64q("default_r_max", calib.default_envelope.r_max);
        h.f64q("default_delta_min", calib.default_envelope.delta_min);
        h.f64q("default_delta_max", calib.default_envelope.delta_max);
        h.f64q("default_sigma_min", calib.default_envelope.sigma_min);
        h.f64q("default_sigma_max", calib.default_envelope.sigma_max);
        h.finalize_hex()
    }

    /// Seal a regime envelope from already-calibrated bounds + the calibration provenance.
    #[allow(clippy::too_many_arguments)]
    pub fn seal(
        regime_id: impl Into<String>,
        phase_id: impl Into<String>,
        var_group: impl Into<String>,
        family: impl Into<String>,
        bounds: AdmissibilityEnvelope,
        transition_policy: TransitionPolicy,
        calibration: RegimeCalibrationProvenance,
    ) -> Self {
        let regime_id = regime_id.into();
        let phase_id = phase_id.into();
        let var_group = var_group.into();
        let family = family.into();
        let provenance_hash = Self::compute_hash(
            &regime_id,
            &phase_id,
            &var_group,
            &family,
            &bounds,
            transition_policy,
            &calibration,
        );
        RegimeEnvelopeV1 {
            regime_id,
            phase_id,
            var_group,
            family,
            bounds,
            transition_policy,
            calibration,
            provenance_hash,
        }
    }

    /// Calibrate **and** seal from a regime's baseline triples — the formalization of
    /// `calibrate_regime_envelope`: run the relax-never-tighten calibration, record exactly which
    /// inputs produced the bounds, and seal the result. The `transition_policy` records whether the
    /// regime had enough baseline to calibrate or fell back to the global default.
    #[allow(clippy::too_many_arguments)]
    pub fn calibrate_and_seal(
        regime_id: impl Into<String>,
        phase_id: impl Into<String>,
        var_group: impl Into<String>,
        family: impl Into<String>,
        baseline_triples: &[ResidualTriple],
        default_envelope: AdmissibilityEnvelope,
        min_samples: usize,
        q_high: f64,
    ) -> Self {
        let bounds =
            calibrate_regime_envelope(baseline_triples, default_envelope, min_samples, q_high);
        let transition_policy = if baseline_triples.len() < min_samples {
            TransitionPolicy::GlobalDefaultFallback
        } else {
            TransitionPolicy::RelaxNeverTighten
        };
        let calibration = RegimeCalibrationProvenance {
            n_baseline_triples: baseline_triples.len(),
            min_samples,
            q_high,
            default_envelope,
        };
        Self::seal(
            regime_id,
            phase_id,
            var_group,
            family,
            bounds,
            transition_policy,
            calibration,
        )
    }

    /// Re-derive the provenance hash from the record's own fields and check it matches the stored
    /// `provenance_hash`. Self-contained — no external input needed (the calibration provenance is in
    /// the record). Returns `true` iff the seal is intact.
    pub fn verify(&self) -> bool {
        Self::compute_hash(
            &self.regime_id,
            &self.phase_id,
            &self.var_group,
            &self.family,
            &self.bounds,
            self.transition_policy,
            &self.calibration,
        ) == self.provenance_hash
    }

    /// Invariant the calibration guarantees: every axis of the sealed bounds is a **superset** of the
    /// global default (the relax-never-tighten property). A verifier can assert this independently of
    /// the hash, so a tightened (and therefore invalid) regime envelope is caught structurally.
    pub fn relaxes_default(&self) -> bool {
        let d = &self.calibration.default_envelope;
        let b = &self.bounds;
        b.r_min <= d.r_min
            && b.r_max >= d.r_max
            && b.delta_min <= d.delta_min
            && b.delta_max >= d.delta_max
            && b.sigma_min <= d.sigma_min
            && b.sigma_max >= d.sigma_max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triple(r: f64, d: f64, s: f64) -> ResidualTriple {
        ResidualTriple {
            r,
            delta: d,
            sigma: s,
            timestamp: 0.0,
        }
    }

    #[test]
    fn seal_is_deterministic_and_self_verifies() {
        let default = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        let baseline: Vec<ResidualTriple> =
            (0..64).map(|i| triple((i % 5) as f64, 0.1, 0.2)).collect();
        let a = RegimeEnvelopeV1::calibrate_and_seal(
            "penicillin:growth",
            "growth",
            "reactor",
            "process_structure",
            &baseline,
            default,
            16,
            0.99,
        );
        let b = RegimeEnvelopeV1::calibrate_and_seal(
            "penicillin:growth",
            "growth",
            "reactor",
            "process_structure",
            &baseline,
            default,
            16,
            0.99,
        );
        assert_eq!(
            a.provenance_hash, b.provenance_hash,
            "seal must be deterministic"
        );
        assert_eq!(a.provenance_hash.len(), 64);
        assert!(a.verify(), "record must self-verify");
        // The calibration relaxes (never tightens) the default on every axis.
        assert!(a.relaxes_default());
        assert_eq!(a.transition_policy, TransitionPolicy::RelaxNeverTighten);
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let default = AdmissibilityEnvelope::symmetric(2.0, 0.1);
        let baseline: Vec<ResidualTriple> = (0..32).map(|_| triple(1.0, 0.1, 0.2)).collect();
        let mut e =
            RegimeEnvelopeV1::calibrate_and_seal("r", "", "", "", &baseline, default, 8, 0.95);
        assert!(e.verify());
        // Mutate a bound without re-sealing → the stored hash no longer matches.
        e.bounds.r_max += 1.0;
        assert!(!e.verify(), "a tampered bound must fail verification");
    }

    #[test]
    fn fallback_when_too_few_baseline_samples() {
        let default = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        let baseline: Vec<ResidualTriple> = (0..4).map(|_| triple(1.0, 0.1, 0.2)).collect();
        let e = RegimeEnvelopeV1::calibrate_and_seal("r", "", "", "", &baseline, default, 16, 0.99);
        assert_eq!(e.transition_policy, TransitionPolicy::GlobalDefaultFallback);
        // Fallback returns the default unchanged → bounds equal the default (still a superset of it).
        assert!(e.relaxes_default());
        assert_eq!(e.bounds.r_max, default.r_max);
        assert!(e.verify());
    }
}
