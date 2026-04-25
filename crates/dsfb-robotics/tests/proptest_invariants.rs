//! Property-based invariants for the DSFB observer contract.
//!
//! Twelve properties are exercised, each corresponding to a
//! load-bearing guarantee of the DSFB observer contract. These are
//! NOT unit tests of specific inputs — they are randomised-input
//! tests that generate thousands of candidate inputs per invariant
//! and fail if any one of them violates the property.
//!
//! Only compiled with `--features std` (proptest needs `std`). The
//! `no_std` core has its own unit tests in `src/*.rs::tests`.

#![cfg(feature = "std")]

use dsfb_robotics::{
    balancing::{self, BalancingCombine},
    envelope::AdmissibilityEnvelope,
    grammar::{GrammarState, ReasonCode},
    math,
    observe, Episode,
};
use proptest::prelude::*;

// ---------- Strategies -----------------------------------------------------

/// Produce finite `f64` values in a bounded, physically-plausible
/// range for robotics residual norms (newton-metres or newtons).
fn finite_bounded() -> impl Strategy<Value = f64> {
    prop::num::f64::NORMAL.prop_map(|x| {
        // Clamp to a plausible robotics residual magnitude range.
        // Keep the strategy inside [−1e6, 1e6] to avoid denormal
        // explosions in the squared-norm paths.
        x.clamp(-1.0e6, 1.0e6)
    })
}

/// Variant of `finite_bounded` that also yields NaN / ±∞ occasionally
/// so properties can exercise the missingness-aware code paths.
fn maybe_non_finite() -> impl Strategy<Value = f64> {
    prop_oneof![
        9 => finite_bounded(),
        1 => Just(f64::NAN),
        1 => Just(f64::INFINITY),
        1 => Just(f64::NEG_INFINITY),
    ]
}

// ---------- Invariants -----------------------------------------------------

proptest! {
    /// P1: `observe` never writes past the output-buffer length.
    ///
    /// This is the primary memory-safety guarantee of the public API.
    /// It is reinforced at compile time by `#![forbid(unsafe_code)]`
    /// (so an out-of-bounds write would be caught by the bounds
    /// checker); proptest confirms it holds across arbitrary inputs.
    #[test]
    fn observe_never_writes_past_output(
        residuals in prop::collection::vec(finite_bounded(), 0..64),
        out_cap in 0usize..32,
    ) {
        let mut out = vec![Episode::empty(); out_cap];
        let n = observe(&residuals, &mut out);
        prop_assert!(n <= out.len(), "observe returned n={} > out.len()={}", n, out.len());
    }

    /// P2: `observe` writes exactly `min(residuals.len(), out.len())`
    /// when all inputs are finite.
    #[test]
    fn observe_fills_to_capacity_on_finite_inputs(
        residuals in prop::collection::vec(finite_bounded(), 0..64),
        out_cap in 0usize..32,
    ) {
        let mut out = vec![Episode::empty(); out_cap];
        let n = observe(&residuals, &mut out);
        prop_assert_eq!(n, residuals.len().min(out.len()));
    }

    /// P3: Every emitted episode has a recognised grammar label.
    #[test]
    fn emitted_episodes_have_valid_grammar_labels(
        residuals in prop::collection::vec(maybe_non_finite(), 0..64),
    ) {
        let mut out = vec![Episode::empty(); residuals.len()];
        let n = observe(&residuals, &mut out);
        for e in &out[..n] {
            prop_assert!(
                matches!(e.grammar, "Admissible" | "Boundary" | "Violation"),
                "unknown grammar label: {}",
                e.grammar
            );
        }
    }

    /// P4: Every emitted episode has a recognised decision label.
    #[test]
    fn emitted_episodes_have_valid_decision_labels(
        residuals in prop::collection::vec(maybe_non_finite(), 0..64),
    ) {
        let mut out = vec![Episode::empty(); residuals.len()];
        let n = observe(&residuals, &mut out);
        for e in &out[..n] {
            prop_assert!(
                matches!(e.decision, "Silent" | "Review" | "Escalate"),
                "unknown decision label: {}",
                e.decision
            );
        }
    }

    /// P5: `observe` is deterministic — identical inputs produce
    /// identical outputs. This is the cornerstone of paper-lock's
    /// bit-exact reproducibility guarantee.
    #[test]
    fn observe_is_deterministic(
        residuals in prop::collection::vec(maybe_non_finite(), 0..64),
    ) {
        let mut out_a = vec![Episode::empty(); residuals.len()];
        let mut out_b = vec![Episode::empty(); residuals.len()];
        let n_a = observe(&residuals, &mut out_a);
        let n_b = observe(&residuals, &mut out_b);
        prop_assert_eq!(n_a, n_b);
        prop_assert_eq!(&out_a[..n_a], &out_b[..n_b]);
    }

    /// P6: Non-finite residuals never produce Violation — the
    /// missingness-aware below-floor rule.
    #[test]
    fn non_finite_inputs_never_escalate(
        len in 1usize..64,
    ) {
        let residuals = vec![f64::NAN; len];
        let mut out = vec![Episode::empty(); len];
        let n = observe(&residuals, &mut out);
        for e in &out[..n] {
            prop_assert_eq!(e.grammar, "Admissible");
            prop_assert_eq!(e.decision, "Silent");
        }
    }

    /// P7: `AdmissibilityEnvelope::is_violation` is strictly monotone
    /// in `norm` for a fixed (non-infinite) multiplier.
    #[test]
    fn envelope_violation_is_monotone_in_norm(
        rho in 0.0_f64..100.0,
        norm_a in 0.0_f64..100.0,
        norm_b in 0.0_f64..100.0,
    ) {
        let env = AdmissibilityEnvelope::new(rho);
        if norm_a < norm_b {
            // If the smaller norm is a violation, the larger must be too.
            prop_assert!(!env.is_violation(norm_a, 1.0) || env.is_violation(norm_b, 1.0));
        }
    }

    /// P8: `sqrt_f64` is a left-inverse for squaring over
    /// finite non-negative `f64` inputs.
    #[test]
    fn sqrt_f64_is_left_inverse_of_square(
        x in 0.0_f64..1.0e6,
    ) {
        let got = math::sqrt_f64(x).expect("finite non-negative");
        let back = got * got;
        if x == 0.0 {
            prop_assert_eq!(got, 0.0);
        } else {
            let rel = (back - x).abs() / x;
            prop_assert!(rel < 1e-12, "relative error {} for x={}", rel, x);
        }
    }

    /// P9: GrammarState severity is a total ordering consistent with
    /// `Admissible < Boundary < Violation`.
    #[test]
    fn grammar_severity_is_consistent(
        reason_idx in 0u8..4,
    ) {
        let reason = match reason_idx {
            0 => ReasonCode::SustainedOutwardDrift,
            1 => ReasonCode::AbruptSlewViolation,
            2 => ReasonCode::RecurrentBoundaryGrazing,
            _ => ReasonCode::EnvelopeViolation,
        };
        let adm = GrammarState::Admissible.severity();
        let bnd = GrammarState::Boundary(reason).severity();
        let vio = GrammarState::Violation.severity();
        prop_assert!(adm < bnd);
        prop_assert!(bnd < vio);
    }

    /// P10: `finite_mean` of a constant-value slice equals that
    /// constant. Sanity check for the missingness-aware statistics.
    #[test]
    fn finite_mean_of_constant_equals_constant(
        c in -1.0e6_f64..1.0e6,
        len in 1usize..64,
    ) {
        let xs = vec![c; len];
        let mu = math::finite_mean(&xs).expect("finite");
        let rel = if c.abs() > 1e-9 { (mu - c).abs() / c.abs() } else { (mu - c).abs() };
        prop_assert!(rel < 1e-12);
    }

    /// P11: `balancing::combine_channels(SumOfSquares)` is non-negative
    /// for all finite inputs (it is a norm).
    #[test]
    fn balancing_combine_is_non_negative(
        rf in finite_bounded(),
        rx in finite_bounded(),
    ) {
        let r = balancing::combine_channels(rf, rx, BalancingCombine::SumOfSquares)
            .expect("finite inputs yield finite norm");
        prop_assert!(r >= 0.0, "combined residual was negative: {}", r);
    }

    /// P12: Observer contract — the input residual slice is NEVER
    /// mutated by `observe`. Enforced at compile time by the `&[f64]`
    /// signature; this property is a belt-and-braces dynamic check.
    #[test]
    fn observer_never_mutates_input(
        residuals in prop::collection::vec(maybe_non_finite(), 0..64),
    ) {
        let before: Vec<u64> = residuals.iter().map(|x| x.to_bits()).collect();
        let mut out = vec![Episode::empty(); residuals.len()];
        let _n = observe(&residuals, &mut out);
        let after: Vec<u64> = residuals.iter().map(|x| x.to_bits()).collect();
        prop_assert_eq!(before, after, "observe must not mutate input slice");
    }
}
