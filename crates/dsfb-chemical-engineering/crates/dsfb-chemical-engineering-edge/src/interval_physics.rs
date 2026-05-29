//! `Interval` + `PhysicsInformedEnvelopeV1` (Wave-7 physics) — fuse **first-principles interval bounds** into
//! the admissibility envelope, so model–plant mismatch becomes first-class evidence.
//!
//! A purely statistical envelope answers "is this value unusual relative to the baseline?". A
//! physics-informed envelope adds an orthogonal, deterministic bound: "is this value even *physically
//! possible* given the first-principles relation and the (interval) uncertainty of its inputs?". Using
//! **interval arithmetic** (no floating-point ambiguity about the bound — the interval is the bound),
//! partial metering / parameter uncertainty propagates into an output interval, and a measured value outside
//! that interval is a **PhysicsViolation** — model–plant mismatch that the statistical envelope cannot
//! express. The two envelopes compose into four cells: within both, statistical-only, physics-violation, or
//! beyond both.
//!
//! Bounded (non-claims): the physics interval is only as good as the supplied relation + input intervals — a
//! `PhysicsViolation` flags an inconsistency between the model and the measurement, **not which is wrong** and
//! not a root cause. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// A closed real interval `[lo, hi]` with `lo ≤ hi`. The arithmetic is the standard interval extension, so a
/// propagated interval is a *guaranteed* enclosure of the true value given the input intervals.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

impl Interval {
    /// Construct `[lo, hi]`, swapping if given out of order (so a caller can pass `±` bounds either way).
    pub fn new(lo: f64, hi: f64) -> Self {
        if lo <= hi {
            Interval { lo, hi }
        } else {
            Interval { lo: hi, hi: lo }
        }
    }
    /// A degenerate point interval `[v, v]`.
    pub fn point(v: f64) -> Self {
        Interval { lo: v, hi: v }
    }
    pub fn width(self) -> f64 {
        self.hi - self.lo
    }
    pub fn contains(self, v: f64) -> bool {
        v >= self.lo && v <= self.hi
    }
    /// Scale by a scalar `k` (a point-interval multiply).
    pub fn scale(self, k: f64) -> Interval {
        self * Interval::point(k)
    }
}

impl std::ops::Add for Interval {
    type Output = Interval;
    fn add(self, o: Interval) -> Interval {
        Interval::new(self.lo + o.lo, self.hi + o.hi)
    }
}

impl std::ops::Sub for Interval {
    type Output = Interval;
    /// `[a,b] − [c,d] = [a−d, b−c]`.
    fn sub(self, o: Interval) -> Interval {
        Interval::new(self.lo - o.hi, self.hi - o.lo)
    }
}

impl std::ops::Mul for Interval {
    type Output = Interval;
    /// Interval multiplication: the hull of the four corner products (handles mixed signs correctly).
    fn mul(self, o: Interval) -> Interval {
        let c = [
            self.lo * o.lo,
            self.lo * o.hi,
            self.hi * o.lo,
            self.hi * o.hi,
        ];
        let lo = c.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = c.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        Interval { lo, hi }
    }
}

/// Where a value falls relative to the physics interval and the statistical envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvelopeCell {
    /// Inside both the physics interval and the statistical envelope (nominal).
    WithinBoth,
    /// Inside the physics interval but outside the statistical envelope — a statistical anomaly that is still
    /// physically plausible (e.g. an unusual but feasible operating point).
    StatisticalOnly,
    /// Outside the physics interval but *inside* the statistical envelope — physically impossible yet
    /// statistically unremarkable: the insidious case the statistical envelope alone would pass. This is the
    /// model–plant-mismatch evidence only a physics-informed envelope can give.
    PhysicsViolation,
    /// Outside both the physics interval and the statistical envelope.
    BeyondBoth,
}

impl EnvelopeCell {
    pub fn tag(self) -> &'static str {
        match self {
            EnvelopeCell::WithinBoth => "within_both",
            EnvelopeCell::StatisticalOnly => "statistical_only",
            EnvelopeCell::PhysicsViolation => "physics_violation",
            EnvelopeCell::BeyondBoth => "beyond_both",
        }
    }
}

/// A hash-sealed physics-informed envelope (schema v1): a first-principles interval composed with a
/// statistical envelope for one variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsInformedEnvelopeV1 {
    pub variable: String,
    pub physics_lo: f64,
    pub physics_hi: f64,
    pub stat_lo: f64,
    pub stat_hi: f64,
    /// Documented provenance of the physics interval (e.g. `"Q=UA·ΔT, UA∈[1100,1300], ΔT∈[9,11]"`).
    pub physics_basis: String,
    pub envelope_hash: String,
}

impl PhysicsInformedEnvelopeV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"physics_informed_envelope_v1");
        h.field("variable", self.variable.as_bytes());
        h.f64q("physics_lo", self.physics_lo);
        h.f64q("physics_hi", self.physics_hi);
        h.f64q("stat_lo", self.stat_lo);
        h.f64q("stat_hi", self.stat_hi);
        h.field("physics_basis", self.physics_basis.as_bytes());
        h.finalize_hex()
    }

    /// Build from a first-principles `physics` interval (typically the result of interval propagation) and a
    /// statistical envelope `[stat_lo, stat_hi]`.
    pub fn build(
        variable: impl Into<String>,
        physics: Interval,
        stat_lo: f64,
        stat_hi: f64,
        physics_basis: impl Into<String>,
    ) -> Self {
        let mut e = PhysicsInformedEnvelopeV1 {
            variable: variable.into(),
            physics_lo: physics.lo,
            physics_hi: physics.hi,
            stat_lo,
            stat_hi,
            physics_basis: physics_basis.into(),
            envelope_hash: String::new(),
        };
        e.envelope_hash = e.seal();
        e
    }

    /// Classify a measured value into one of the four cells.
    pub fn classify(&self, v: f64) -> EnvelopeCell {
        let in_physics = v >= self.physics_lo && v <= self.physics_hi;
        let in_stat = v >= self.stat_lo && v <= self.stat_hi;
        match (in_physics, in_stat) {
            (true, true) => EnvelopeCell::WithinBoth,
            (true, false) => EnvelopeCell::StatisticalOnly,
            (false, true) => EnvelopeCell::PhysicsViolation,
            (false, false) => EnvelopeCell::BeyondBoth,
        }
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.envelope_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_arithmetic_propagates_uncertainty() {
        // Q = U·A·ΔT with UA ∈ [1100, 1300] W/K and ΔT ∈ [9, 11] K → Q ∈ [9900, 14300] W.
        let ua = Interval::new(1100.0, 1300.0);
        let dt = Interval::new(9.0, 11.0);
        let q = ua * dt;
        assert!((q.lo - 9900.0).abs() < 1e-6 && (q.hi - 14300.0).abs() < 1e-6);
        // Mixed-sign multiply hull.
        let m = Interval::new(-2.0, 3.0) * Interval::new(-1.0, 4.0);
        assert_eq!((m.lo, m.hi), (-8.0, 12.0)); // corners: 2,-8,-3,12
                                                // add / sub.
        assert_eq!(
            Interval::new(1.0, 2.0) + Interval::new(3.0, 4.0),
            Interval::new(4.0, 6.0)
        );
        assert_eq!(
            Interval::new(5.0, 8.0) - Interval::new(1.0, 2.0),
            Interval::new(3.0, 7.0)
        );
    }

    #[test]
    fn physics_violation_is_distinct_from_statistical() {
        // Physics interval for a duty: [9900, 14300]; the learned statistical envelope drifted WIDE on the
        // high side: [10000, 20000] (so the baseline saw values physics actually forbids).
        let phys = Interval::new(9900.0, 14300.0);
        let env = PhysicsInformedEnvelopeV1::build(
            "duty_Q",
            phys,
            10000.0,
            20000.0,
            "Q=UA·ΔT, UA∈[1100,1300], ΔT∈[9,11]",
        );
        assert_eq!(env.classify(12000.0), EnvelopeCell::WithinBoth);
        // 9950 is physically possible but below the (drifted) statistical band → StatisticalOnly.
        assert_eq!(env.classify(9950.0), EnvelopeCell::StatisticalOnly);
        // 18000 W is impossible for this UA·ΔT yet INSIDE the learned envelope → PhysicsViolation (the
        // insidious case the statistical envelope alone would pass).
        assert_eq!(env.classify(18000.0), EnvelopeCell::PhysicsViolation);
        // 25000 W is outside both.
        assert_eq!(env.classify(25000.0), EnvelopeCell::BeyondBoth);
        assert!(env.envelope_hash.len() == 64 && env.verify());
    }

    #[test]
    fn envelope_is_tamper_evident() {
        let env =
            PhysicsInformedEnvelopeV1::build("x", Interval::new(0.0, 10.0), 2.0, 8.0, "basis");
        assert!(env.verify());
        let mut t = env.clone();
        t.physics_hi = 1e9; // forge a looser physics bound
        assert!(!t.verify());
    }
}
