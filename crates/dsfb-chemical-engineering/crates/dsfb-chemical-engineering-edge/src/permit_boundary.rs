//! `PermitBoundaryWitnessV1` (Wave-3 physics) — a permit-relevant boundary witness for the environmental /
//! consent variables a plant must keep inside an operating permit (stack NOx/SOx/CO, effluent pH, BOD/COD,
//! discharge temperature, opacity).
//!
//! This reuses [`crate::spec_limit::SpecLimitWitnessV1`] for the breach facts (no duplicated machinery) and
//! adds two things a permit context needs that a generic spec limit does not: (1) documented *permit
//! provenance* — which permit parameter and which (illustrative) regulatory basis the boundary represents;
//! and (2) the early-warning metric operators actually track — the **tightest approach to the boundary**
//! across the run (`min_margin_to_limit`), so "we came within 3 % of the NOx consent" is first-class
//! evidence, not just "we breached / we didn't".
//!
//! Bounded (non-claims, stronger than a spec limit): this is **NOT regulatory compliance certification**,
//! not a Continuous-Emissions-Monitoring-System (CEMS) of record, and not a permit-reporting instrument. It
//! is an advisory, read-only proxy indicator over whatever series + (illustrative) limit the operator
//! supplies; the permit boundary is an input DSFB does not validate against the actual consent document. A
//! boundary touch/breach here is candidate evidence to *prompt a look at the system of record*, never a
//! compliance determination. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;
use crate::spec_limit::{SpecLimit, SpecLimitWitnessV1};

/// A hash-sealed permit-boundary witness (schema v1) wrapping the spec-limit breach facts with permit
/// provenance and a headroom metric.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermitBoundaryWitnessV1 {
    /// The permit parameter this boundary governs (e.g. `"stack_NOx_ppm"`, `"effluent_pH"`, `"discharge_T"`).
    pub permit_parameter: String,
    /// A documented, *illustrative* statement of the regulatory basis (e.g. `"operating-permit daily limit
    /// (illustrative; not the consent of record)"`). Provenance only — never validated by DSFB.
    pub regulatory_basis: String,
    /// The underlying spec-limit breach record (per-sample classification, counts, onset, peak excursion).
    pub breach: SpecLimitWitnessV1,
    /// Smallest *signed* distance to the nearest permit bound across all finite samples: positive is
    /// headroom (the closest the process came without breaching), negative is the worst breach depth. This
    /// is the permit early-warning number — a small positive value is a "near-miss" worth attention.
    pub min_margin_to_limit: f64,
    /// The fixed non-claim, sealed into the record so it can never be silently dropped.
    pub non_claim: String,
    pub permit_hash: String,
}

impl PermitBoundaryWitnessV1 {
    const NON_CLAIM: &'static str =
        "advisory proxy indicator only; NOT regulatory compliance certification, NOT a CEMS of record; \
         the permit boundary is an unvalidated input; a touch/breach prompts a look at the system of record";

    /// Smallest signed distance to the nearest declared bound across finite samples. For a sample `v`:
    /// the margin is `min((upper − v), (v − lower))` over whichever bounds are present; with no bound the
    /// margin is `+∞` (no permit boundary to approach). The series minimum is the tightest approach.
    fn min_margin(limit: &SpecLimit, series: &[f64]) -> f64 {
        let mut m = f64::INFINITY;
        for &v in series {
            if !v.is_finite() {
                continue;
            }
            let mut sample = f64::INFINITY;
            if let Some(u) = limit.upper {
                sample = sample.min(u - v);
            }
            if let Some(l) = limit.lower {
                sample = sample.min(v - l);
            }
            m = m.min(sample);
        }
        m
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"permit_boundary_witness_v1");
        h.field("permit_parameter", self.permit_parameter.as_bytes());
        h.field("regulatory_basis", self.regulatory_basis.as_bytes());
        // Bind to the inner breach record by its seal (which already covers the series + limits + tallies).
        h.field("breach_witness_hash", self.breach.witness_hash.as_bytes());
        h.f64q("min_margin_to_limit", self.min_margin_to_limit);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    /// Build + seal a permit-boundary witness from a permit-relevant series and its (illustrative) limit.
    pub fn build(
        permit_parameter: impl Into<String>,
        regulatory_basis: impl Into<String>,
        limit: &SpecLimit,
        series: &[f64],
    ) -> Self {
        let mut w = PermitBoundaryWitnessV1 {
            permit_parameter: permit_parameter.into(),
            regulatory_basis: regulatory_basis.into(),
            breach: SpecLimitWitnessV1::build(limit, series),
            min_margin_to_limit: Self::min_margin(limit, series),
            non_claim: Self::NON_CLAIM.to_string(),
            permit_hash: String::new(),
        };
        w.permit_hash = w.seal();
        w
    }

    /// True iff any sample crossed the permit boundary.
    pub fn breached(&self) -> bool {
        self.breach.is_spec_breach()
    }

    /// True iff the process never breached but came within `frac`·(nothing) — i.e. a near-miss: no breach yet
    /// `0 ≤ min_margin_to_limit ≤ tol`. `tol` is in the variable's own units (the caller knows the scale).
    pub fn is_near_miss(&self, tol: f64) -> bool {
        !self.breached()
            && self.min_margin_to_limit.is_finite()
            && self.min_margin_to_limit >= 0.0
            && self.min_margin_to_limit <= tol
    }

    /// Re-derive the inner breach seal and the permit seal and check both — catches tampering of the breach
    /// record, the headroom metric, the provenance, or the non-claim.
    pub fn verify(&self, series: &[f64]) -> bool {
        self.breach.verify(series)
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.permit_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn permit_limit() -> SpecLimit {
        // Illustrative stack-NOx upper consent at 50 ppm (no lower bound).
        SpecLimit {
            variable: "stack_NOx".into(),
            unit: "-".into(),
            lower: None,
            upper: Some(50.0),
            limit_kind: "permit_NOx_ppm".into(),
        }
    }

    #[test]
    fn records_breach_and_headroom_and_self_verifies() {
        // Rides up to a 55 ppm breach (excursion 5) at index 3; tightest pre-breach approach was 50−48 = 2.
        let series = vec![40.0, 45.0, 48.0, 55.0, 47.0];
        let w = PermitBoundaryWitnessV1::build(
            "stack_NOx_ppm",
            "operating-permit daily limit (illustrative)",
            &permit_limit(),
            &series,
        );
        assert!(w.breached());
        assert_eq!(w.breach.n_above, 1);
        assert!((w.breach.peak_excursion - 5.0).abs() < 1e-12);
        // min margin is the worst breach depth: 50 − 55 = −5.
        assert!((w.min_margin_to_limit - (-5.0)).abs() < 1e-12);
        assert!(w.permit_hash.len() == 64 && w.verify(&series));
        assert!(w.non_claim.contains("NOT regulatory compliance"));
    }

    #[test]
    fn near_miss_is_flagged_without_a_breach() {
        // Peaks at 49 ppm — never breaches the 50 ppm consent, but the 1 ppm headroom is a near-miss.
        let series = vec![40.0, 45.0, 49.0, 46.0];
        let w = PermitBoundaryWitnessV1::build(
            "stack_NOx_ppm",
            "illustrative",
            &permit_limit(),
            &series,
        );
        assert!(!w.breached());
        assert!((w.min_margin_to_limit - 1.0).abs() < 1e-12);
        assert!(w.is_near_miss(2.0)); // within 2 ppm of the boundary
        assert!(!w.is_near_miss(0.5)); // but not within 0.5 ppm
    }

    #[test]
    fn tampering_the_headroom_or_nonclaim_breaks_the_seal() {
        let series = vec![40.0, 49.0];
        let mut w = PermitBoundaryWitnessV1::build("p", "illustrative", &permit_limit(), &series);
        assert!(w.verify(&series));
        w.min_margin_to_limit = 9.9; // forge a rosier headroom
        assert!(!w.verify(&series));
        let mut w2 = PermitBoundaryWitnessV1::build("p", "illustrative", &permit_limit(), &series);
        w2.non_claim = "fully compliant, certified".into(); // forge away the non-claim
        assert!(!w2.verify(&series));
    }
}
