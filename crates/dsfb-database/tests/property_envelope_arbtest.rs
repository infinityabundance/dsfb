//! Randomised property tests for `grammar::envelope::classify`.
//!
//! Companion to `proofs/kani_envelope.rs`: kani proves these properties
//! symbolically on all finite floats; `arbtest` exercises them with
//! shrinkable random cases so that in the (unusual) event of a proof
//! regression, test output names the smallest counterexample. This
//! redundancy is deliberate — it means a subtle refactor in the
//! envelope threshold logic surfaces both as a failing kani proof
//! and a failing `cargo test`, rather than only in one place.

use arbtest::arbtest;
use dsfb_database::grammar::envelope::{classify, Envelope};

#[test]
fn classify_is_total_on_finite_inputs() {
    arbtest(|u| {
        let ema: f64 = u.arbitrary()?;
        let instant: f64 = u.arbitrary()?;
        let drift: f64 = u.arbitrary()?;
        let slew: f64 = u.arbitrary()?;
        if !ema.is_finite() || !instant.is_finite() || !drift.is_finite() || !slew.is_finite() {
            return Ok(());
        }
        let env = classify(ema, instant, drift, slew);
        // Confirm the result is one of the three enum variants; a
        // non-matching variant would be caught by the compiler, but
        // this assertion documents the totality property explicitly.
        assert!(matches!(
            env,
            Envelope::Stable | Envelope::Drift | Envelope::Boundary
        ));
        Ok(())
    })
    .budget_ms(50);
}

#[test]
fn boundary_dominates_drift_property() {
    arbtest(|u| {
        let ema: f64 = u.arbitrary()?;
        let instant: f64 = u.arbitrary()?;
        let drift: f64 = u.arbitrary()?;
        let slew: f64 = u.arbitrary()?;
        if !ema.is_finite() || !instant.is_finite() || !drift.is_finite() || !slew.is_finite() {
            return Ok(());
        }
        if instant.abs() >= slew {
            assert_eq!(
                classify(ema, instant, drift, slew),
                Envelope::Boundary,
                "|instant|>=slew must classify as Boundary"
            );
        }
        Ok(())
    })
    .budget_ms(50);
}

#[test]
fn stable_below_both_thresholds_property() {
    arbtest(|u| {
        let ema: f64 = u.arbitrary()?;
        let instant: f64 = u.arbitrary()?;
        let drift: f64 = u.arbitrary()?;
        let slew: f64 = u.arbitrary()?;
        if !ema.is_finite() || !instant.is_finite() || !drift.is_finite() || !slew.is_finite() {
            return Ok(());
        }
        if ema.abs() < drift && instant.abs() < slew {
            assert_eq!(
                classify(ema, instant, drift, slew),
                Envelope::Stable,
                "strictly below both thresholds must classify as Stable"
            );
        }
        Ok(())
    })
    .budget_ms(50);
}

#[test]
fn threshold_monotonicity_property() {
    arbtest(|u| {
        let ema: f64 = u.arbitrary()?;
        let instant: f64 = u.arbitrary()?;
        let drift: f64 = u.arbitrary()?;
        let slew: f64 = u.arbitrary()?;
        let drift_looser: f64 = u.arbitrary()?;
        let slew_looser: f64 = u.arbitrary()?;
        if !ema.is_finite()
            || !instant.is_finite()
            || !drift.is_finite()
            || !slew.is_finite()
            || !drift_looser.is_finite()
            || !slew_looser.is_finite()
            || drift < 0.0
            || slew < 0.0
            || drift_looser < drift
            || slew_looser < slew
        {
            return Ok(());
        }
        if classify(ema, instant, drift, slew) == Envelope::Stable {
            assert_eq!(
                classify(ema, instant, drift_looser, slew_looser),
                Envelope::Stable,
                "loosening thresholds cannot flip Stable → non-Stable"
            );
        }
        Ok(())
    })
    .budget_ms(50);
}
