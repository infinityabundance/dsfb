/// DSFB Oil & Gas — Admissibility Envelope Evaluation
///
/// Evaluates whether a ResidualTriple is inside, on the boundary of,
/// or outside the calibrated admissibility envelope.
/// No side-effects; pure classification functions only.

use crate::types::{AdmissibilityEnvelope, ResidualTriple};

/// Classification of a normalised coordinate against the unit interval [−1, 1].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordClass {
    Interior,  // |x̃| < 1 − ε_b
    Grazing,   // 1 − ε_b ≤ |x̃| ≤ 1
    Outside,   // |x̃| > 1
}

impl CoordClass {
    pub fn classify(norm_val: f64, grazing_band: f64) -> Self {
        let abs_v = norm_val.abs();
        if abs_v > 1.0 {
            CoordClass::Outside
        } else if abs_v >= 1.0 - grazing_band {
            CoordClass::Grazing
        } else {
            CoordClass::Interior
        }
    }
}

/// Result of evaluating a triple against an envelope.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeEval {
    pub r_class: CoordClass,
    pub delta_class: CoordClass,
    pub sigma_class: CoordClass,
    pub norm_r:     f64,
    pub norm_delta: f64,
    pub norm_sigma: f64,
}

impl EnvelopeEval {
    /// True if the raw residual is outside its bounds.
    pub fn r_violated(&self)     -> bool { self.r_class     == CoordClass::Outside }
    /// True if drift is outside its bounds.
    pub fn delta_violated(&self) -> bool { self.delta_class == CoordClass::Outside }
    /// True if slew is outside its bounds.
    pub fn sigma_violated(&self) -> bool { self.sigma_class == CoordClass::Outside }
    /// True if any coordinate is grazing but none is outside.
    pub fn any_grazing(&self) -> bool {
        !self.r_violated()
            && !self.delta_violated()
            && !self.sigma_violated()
            && (self.r_class     == CoordClass::Grazing
                || self.delta_class == CoordClass::Grazing
                || self.sigma_class == CoordClass::Grazing)
    }
    /// True if all coordinates are fully interior.
    pub fn all_interior(&self) -> bool {
        self.r_class     == CoordClass::Interior
            && self.delta_class == CoordClass::Interior
            && self.sigma_class == CoordClass::Interior
    }
}

/// Evaluate a triple against an envelope; returns classification for each axis.
pub fn evaluate(env: &AdmissibilityEnvelope, triple: &ResidualTriple) -> EnvelopeEval {
    let (nr, nd, ns) = env.normalise(triple);
    EnvelopeEval {
        r_class:     CoordClass::classify(nr, env.grazing_band),
        delta_class: CoordClass::classify(nd, env.grazing_band),
        sigma_class: CoordClass::classify(ns, env.grazing_band),
        norm_r:     nr,
        norm_delta: nd,
        norm_sigma: ns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AdmissibilityEnvelope, ResidualTriple};

    fn make_triple(r: f64, delta: f64, sigma: f64) -> ResidualTriple {
        ResidualTriple { r, delta, sigma, timestamp: 0.0 }
    }

    #[test]
    fn zero_triple_is_interior() {
        let env = AdmissibilityEnvelope::default_pipeline();
        let ev = evaluate(&env, &make_triple(0.0, 0.0, 0.0));
        assert!(ev.all_interior());
    }

    #[test]
    fn r_outside_is_detected() {
        let env = AdmissibilityEnvelope::default_pipeline();
        let ev = evaluate(&env, &make_triple(100.0, 0.0, 0.0));
        assert!(ev.r_violated());
    }

    #[test]
    fn delta_outside_is_detected() {
        let env = AdmissibilityEnvelope::default_pipeline();
        let ev = evaluate(&env, &make_triple(0.0, 100.0, 0.0));
        assert!(ev.delta_violated());
    }

    #[test]
    fn sigma_outside_is_detected() {
        let env = AdmissibilityEnvelope::default_pipeline();
        let ev = evaluate(&env, &make_triple(0.0, 0.0, 100.0));
        assert!(ev.sigma_violated());
    }
}
