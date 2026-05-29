//! `SpecLimitWitnessV1` (Wave-3 physics) — separates an **engineering spec / operating-limit breach** from
//! a **statistical anomaly**.
//!
//! DSFB's grammar fires on *learned* thresholds: a residual, T², or SPE crosses a baseline-calibrated bound.
//! That answers "is this statistically unusual?" — but an operator also needs the orthogonal question "did a
//! variable cross a **hard engineering limit**?" (a relief-valve setpoint, a vessel design pressure, a
//! minimum-flow trip point, a product spec). The two are independent: a process can sit deep inside its
//! statistical baseline yet breach a design limit (a slow set-point drift to an unsafe-but-stable value), or
//! ring the statistical alarm while comfortably within spec (benign transient). This witness reads the
//! *raw* series against declared limits and reports the spec status per sample, so a case file can say
//! "AboveUpperSpec **and** statistically anomalous" vs "statistical-only" vs "spec-breach-only" — a
//! distinction the statistical pipeline alone cannot draw.
//!
//! Bounded (non-claims): a spec breach is an *operating-limit event as declared by the supplied limits* — it
//! is **not** a safety trip, not an interlock action, and carries **no safety-instrumented-function (SIS,
//! IEC 61511) authority** (DSFB is advisory, read-only). The limits are an input, not validated by DSFB; a
//! breach is candidate evidence, never a control or shutdown decision. Additive + off the replay path;
//! deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Per-sample spec status against the declared limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecStatus {
    /// Inside both declared limits (or no limit on the breached side).
    WithinSpec,
    /// Below the declared lower spec/operating limit (LSL).
    BelowLowerSpec,
    /// Above the declared upper spec/operating limit (USL).
    AboveUpperSpec,
}

impl SpecStatus {
    /// Stable tag for rendering / downstream serialisation.
    pub fn tag(self) -> &'static str {
        match self {
            SpecStatus::WithinSpec => "within_spec",
            SpecStatus::BelowLowerSpec => "below_lower_spec",
            SpecStatus::AboveUpperSpec => "above_upper_spec",
        }
    }
    /// True iff this status is a breach (not within spec).
    pub fn is_breach(self) -> bool {
        !matches!(self, SpecStatus::WithinSpec)
    }
}

/// A declared engineering limit for one variable. Either bound is optional (a one-sided limit, e.g. a
/// minimum-flow trip with no upper bound). `limit_kind` documents the limit's *provenance* (e.g.
/// `"design_pressure"`, `"relief_setpoint"`, `"min_flow_trip"`, `"product_spec_USL"`) so the witness is
/// auditable; `unit` ties the limit to the unit-consistency court's vocabulary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecLimit {
    pub variable: String,
    #[serde(default)]
    pub unit: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub limit_kind: String,
}

/// Classify one raw value against optional lower/upper limits. A non-finite value is treated as
/// `WithinSpec` (it is a data-quality issue handled elsewhere, not a spec breach). The upper bound is
/// checked first, so a (mis)configuration with `lower > upper` deterministically reports `AboveUpperSpec`.
pub fn classify_sample(v: f64, lower: Option<f64>, upper: Option<f64>) -> SpecStatus {
    if !v.is_finite() {
        return SpecStatus::WithinSpec;
    }
    if let Some(u) = upper {
        if v > u {
            return SpecStatus::AboveUpperSpec;
        }
    }
    if let Some(l) = lower {
        if v < l {
            return SpecStatus::BelowLowerSpec;
        }
    }
    SpecStatus::WithinSpec
}

/// A hash-sealed spec-limit witness (schema v1) over one variable's raw series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecLimitWitnessV1 {
    pub variable: String,
    pub unit: String,
    pub limit_kind: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub n_samples: usize,
    pub n_within: usize,
    pub n_below: usize,
    pub n_above: usize,
    /// Index of the first breaching sample, if any (the spec-breach onset).
    pub first_breach_index: Option<usize>,
    /// Largest distance beyond the nearest breached limit across the series (0 if no breach). For an upper
    /// breach this is `max(v − upper)`; for a lower breach `max(lower − v)` — the worst overshoot magnitude.
    pub peak_excursion: f64,
    /// SHA-256 of the raw series (quantised f64), so the exact series is sealed.
    pub series_hash: String,
    pub witness_hash: String,
}

impl SpecLimitWitnessV1 {
    fn hash_series(series: &[f64]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"spec_limit_series_v1");
        for &v in series {
            h.f64q("v", v);
        }
        h.finalize_hex()
    }

    /// Seal over the witness's own fields (called after the struct is built with its `series_hash` set and
    /// a placeholder `witness_hash`). `witness_hash` is intentionally excluded from its own digest.
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"spec_limit_witness_v1");
        h.field("variable", self.variable.as_bytes());
        h.field("unit", self.unit.as_bytes());
        h.field("limit_kind", self.limit_kind.as_bytes());
        // Encode each optional bound as (present-flag, value) so `None` and `Some(0.0)` never collide.
        h.u64("has_lower", self.lower.is_some() as u64);
        h.f64q("lower", self.lower.unwrap_or(0.0));
        h.u64("has_upper", self.upper.is_some() as u64);
        h.f64q("upper", self.upper.unwrap_or(0.0));
        h.u64("n_samples", self.n_samples as u64);
        h.u64("n_within", self.n_within as u64);
        h.u64("n_below", self.n_below as u64);
        h.u64("n_above", self.n_above as u64);
        h.u64("has_first_breach", self.first_breach_index.is_some() as u64);
        h.u64(
            "first_breach_index",
            self.first_breach_index.unwrap_or(0) as u64,
        );
        h.f64q("peak_excursion", self.peak_excursion);
        h.field("series_hash", self.series_hash.as_bytes());
        h.finalize_hex()
    }

    /// Classify a raw series against the declared limit and seal the witness.
    pub fn build(limit: &SpecLimit, series: &[f64]) -> Self {
        let (mut n_within, mut n_below, mut n_above) = (0usize, 0usize, 0usize);
        let mut first_breach_index = None;
        let mut peak_excursion = 0.0f64;
        for (i, &v) in series.iter().enumerate() {
            match classify_sample(v, limit.lower, limit.upper) {
                SpecStatus::WithinSpec => n_within += 1,
                SpecStatus::BelowLowerSpec => {
                    n_below += 1;
                    first_breach_index.get_or_insert(i);
                    if let Some(l) = limit.lower {
                        peak_excursion = peak_excursion.max(l - v);
                    }
                }
                SpecStatus::AboveUpperSpec => {
                    n_above += 1;
                    first_breach_index.get_or_insert(i);
                    if let Some(u) = limit.upper {
                        peak_excursion = peak_excursion.max(v - u);
                    }
                }
            }
        }
        let mut w = SpecLimitWitnessV1 {
            variable: limit.variable.clone(),
            unit: limit.unit.clone(),
            limit_kind: limit.limit_kind.clone(),
            lower: limit.lower,
            upper: limit.upper,
            n_samples: series.len(),
            n_within,
            n_below,
            n_above,
            first_breach_index,
            peak_excursion,
            series_hash: Self::hash_series(series),
            witness_hash: String::new(),
        };
        w.witness_hash = w.seal();
        w
    }

    /// True iff any sample breached a declared limit.
    pub fn is_spec_breach(&self) -> bool {
        self.n_below + self.n_above > 0
    }

    /// Re-derive the seal (including the series hash, supplied again) and check it matches — catches
    /// tampering of any tally or the hash.
    pub fn verify(&self, series: &[f64]) -> bool {
        // The supplied series must hash to the sealed `series_hash`; then re-sealing over `self` (which uses
        // that same stored hash) must reproduce `witness_hash`. Together this catches a tampered series, a
        // tampered tally/limit, or a tampered hash.
        Self::hash_series(series) == self.series_hash && self.seal() == self.witness_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limit(lower: Option<f64>, upper: Option<f64>) -> SpecLimit {
        SpecLimit {
            variable: "reactor_P".into(),
            unit: "bar".into(),
            lower,
            upper,
            limit_kind: "design_pressure".into(),
        }
    }

    #[test]
    fn classifies_against_one_and_two_sided_limits() {
        assert_eq!(
            classify_sample(5.0, Some(0.0), Some(10.0)),
            SpecStatus::WithinSpec
        );
        assert_eq!(
            classify_sample(11.0, Some(0.0), Some(10.0)),
            SpecStatus::AboveUpperSpec
        );
        assert_eq!(
            classify_sample(-1.0, Some(0.0), Some(10.0)),
            SpecStatus::BelowLowerSpec
        );
        // One-sided (min-flow trip, no upper bound): only the lower side can breach.
        assert_eq!(
            classify_sample(100.0, Some(2.0), None),
            SpecStatus::WithinSpec
        );
        assert_eq!(
            classify_sample(1.0, Some(2.0), None),
            SpecStatus::BelowLowerSpec
        );
        // Non-finite is not a spec breach (it is a data-quality issue handled elsewhere).
        assert_eq!(
            classify_sample(f64::NAN, Some(0.0), Some(10.0)),
            SpecStatus::WithinSpec
        );
    }

    #[test]
    fn witness_counts_breaches_and_peak_excursion_and_self_verifies() {
        // Design pressure 10 bar; the series rides up and overshoots to 12.5 (excursion 2.5) at index 4.
        let series = vec![8.0, 9.0, 9.5, 10.0, 12.5, 11.0];
        let w = SpecLimitWitnessV1::build(&limit(Some(0.0), Some(10.0)), &series);
        assert_eq!(w.n_above, 2); // 12.5 and 11.0 both exceed 10
        assert_eq!(w.n_below, 0);
        assert_eq!(w.n_within, 4);
        assert_eq!(w.first_breach_index, Some(4));
        assert!((w.peak_excursion - 2.5).abs() < 1e-12);
        assert!(w.is_spec_breach());
        assert_eq!(w.witness_hash.len(), 64);
        assert!(w.verify(&series));
        // Determinism + tamper-evidence.
        assert_eq!(
            SpecLimitWitnessV1::build(&limit(Some(0.0), Some(10.0)), &series),
            w
        );
        assert!(!w.verify(&[0.0; 6]));
    }

    #[test]
    fn statistical_anomaly_can_sit_within_spec() {
        // A variable parked at a benign-but-unusual value: no declared limit is crossed, so the spec
        // witness reports NO breach even though a statistical detector might fire. This is the whole point:
        // the two questions are independent.
        let series = vec![5.0, 5.0, 5.0, 7.0, 7.0]; // a step that a statistical detector flags
        let w = SpecLimitWitnessV1::build(&limit(Some(0.0), Some(10.0)), &series);
        assert!(!w.is_spec_breach());
        assert_eq!(w.n_within, 5);
        assert_eq!(w.first_breach_index, None);
        assert_eq!(w.peak_excursion, 0.0);
    }
}
