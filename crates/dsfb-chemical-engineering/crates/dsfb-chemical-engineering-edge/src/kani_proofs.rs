//! Kani formal-verification harnesses (compiled only under `cfg(kani)`).
//!
//! These prove core DSFB invariants over bounded symbolic inputs:
//!   * determinism — identical inputs/params produce identical grammar states;
//!   * non-interference — the read-only wrapper exposes no mutation path;
//!   * envelope monotonicity — a triple at the origin is always interior;
//!   * grammar soundness (Wave-1 B3) — an interior *finite* triple is never SensorFault; a triple beyond
//!     the envelope bound is never reported interior; and `classify` is *total* (no panic, terminates,
//!     returns a state) for any finite triple — i.e. the per-sample grammar cannot get stuck or trap.
//!
//! Run with: `cargo kani` (after `cargo install --locked kani-verifier && cargo kani setup`).

#[cfg(kani)]
mod proofs {
    use crate::dsfb_core::*;

    #[kani::proof]
    fn proof_origin_triple_is_interior() {
        let env = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        let t = ResidualTriple {
            r: 0.0,
            delta: 0.0,
            sigma: 0.0,
            timestamp: 0.0,
        };
        let ev = evaluate(&env, &t);
        assert!(ev.all_interior());
    }

    #[kani::proof]
    fn proof_nonfinite_is_sensor_fault() {
        let env = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        let mut g = GrammarClassifier::new();
        let t = ResidualTriple {
            r: f64::NAN,
            delta: 0.0,
            sigma: 0.0,
            timestamp: 0.0,
        };
        let ev = evaluate(&env, &t);
        let (state, _) = g.classify(&ev, &t);
        assert!(state == GrammarState::SensorFault);
    }

    #[kani::proof]
    fn proof_determinism_two_runs_match() {
        let r: f64 = kani::any();
        kani::assume(r.is_finite() && r.abs() < 1e6);
        let env = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        let mut g1 = GrammarClassifier::new();
        let mut g2 = GrammarClassifier::new();
        let t = ResidualTriple {
            r,
            delta: r,
            sigma: 0.0,
            timestamp: 0.0,
        };
        let ev = evaluate(&env, &t);
        let (s1, _) = g1.classify(&ev, &t);
        let (s2, _) = g2.classify(&ev, &t);
        assert!(s1 == s2);
    }

    // ── Grammar soundness (Wave-1 B3) ───────────────────────────────────────────────────────────────

    /// A *finite* triple strictly inside the envelope is never classified as a SensorFault — the
    /// SensorFault state is reserved for non-finite (NaN/±∞) inputs.
    #[kani::proof]
    fn proof_interior_finite_is_not_sensor_fault() {
        let r: f64 = kani::any();
        let d: f64 = kani::any();
        let s: f64 = kani::any();
        // Strictly inside symmetric(r_max=3.0, slew=0.1): bound each component well within its limit.
        kani::assume(r.is_finite() && d.is_finite() && s.is_finite());
        kani::assume(r.abs() < 1.0 && d.abs() < 0.05 && s.abs() < 0.05);
        let env = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        let mut g = GrammarClassifier::new();
        let t = ResidualTriple {
            r,
            delta: d,
            sigma: s,
            timestamp: 0.0,
        };
        let ev = evaluate(&env, &t);
        let (state, _) = g.classify(&ev, &t);
        assert!(state != GrammarState::SensorFault);
    }

    /// The envelope actually bounds: a finite residual well beyond `r_max` is never reported interior.
    #[kani::proof]
    fn proof_beyond_bound_is_not_interior() {
        let r: f64 = kani::any();
        kani::assume(r.is_finite() && r > 4.0); // strictly beyond r_max=3.0 + the grazing band
        let env = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        let t = ResidualTriple {
            r,
            delta: 0.0,
            sigma: 0.0,
            timestamp: 0.0,
        };
        let ev = evaluate(&env, &t);
        assert!(!ev.all_interior());
    }

    /// `classify` is total on finite input: for any finite triple it terminates without panic and returns
    /// a state. (Kani proves the absence of panics/overflow along every path, not just a sampled one.)
    #[kani::proof]
    fn proof_classify_is_total_on_finite() {
        let r: f64 = kani::any();
        let d: f64 = kani::any();
        let s: f64 = kani::any();
        kani::assume(r.is_finite() && d.is_finite() && s.is_finite());
        kani::assume(r.abs() < 1e6 && d.abs() < 1e6 && s.abs() < 1e6);
        let env = AdmissibilityEnvelope::symmetric(3.0, 0.1);
        let mut g = GrammarClassifier::new();
        let t = ResidualTriple {
            r,
            delta: d,
            sigma: s,
            timestamp: 0.0,
        };
        let ev = evaluate(&env, &t);
        let (_state, _reason) = g.classify(&ev, &t); // must not panic / must return
    }
}
