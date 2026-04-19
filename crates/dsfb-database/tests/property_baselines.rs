//! Randomised property tests for the published-baseline change-point
//! detectors used by the §7 bake-off (`baselines::{adwin, bocpd, pelt}`).
//!
//! Companion to the per-detector unit tests in `src/baselines/*.rs`. Those
//! tests exercise canonical fixtures (an obvious mean shift, a flat
//! series); these tests cover the *boundary* behaviours that a static
//! fixture cannot — degenerate inputs (empty, length-1, all-equal,
//! all-NaN, all-±∞) must not panic, and stationary uniform input must
//! not provoke an unbounded false-positive cascade.
//!
//! ### Read-only invariant
//!
//! Per the Pass-2 guardrails (`/home/one/.claude/plans/only-focus-on-dsfb-database-curious-scroll.md`)
//! the baseline source files (`src/baselines/{adwin,bocpd,pelt}.rs`) are
//! frozen. These tests call only the published `ChangePointDetector` trait
//! surface and never construct alternative configurations beyond the
//! `Default::default()` instances. If a property fails it is documented in
//! paper §44 (adversarial workload) or §36 (cross-firing) and the fix is
//! deferred to a future pass — the *contract* this file pins is that the
//! baselines we compare DSFB against do not crash on hostile input.
//!
//! ### Why arbtest, not proptest
//!
//! The crate already pulls in `arbtest = "0.3"` for
//! `tests/property_envelope_arbtest.rs`. Using the same library keeps the
//! dev-dependency surface minimal and the property style consistent
//! across the test suite.

use arbtest::arbtest;
use dsfb_database::baselines::{adwin::Adwin, bocpd::Bocpd, pelt::Pelt, ChangePointDetector};

fn detectors() -> Vec<Box<dyn ChangePointDetector>> {
    vec![
        Box::new(Adwin::default()),
        Box::new(Bocpd::default()),
        Box::new(Pelt::default()),
    ]
}

// ----------------------------------------------------------------------
// Degenerate-input no-panic (fixed cases — would be wasted on randomised
// shrinking because the failure modes are enumerable).
// ----------------------------------------------------------------------

#[test]
fn no_panic_on_empty_series() {
    let series: Vec<(f64, f64)> = Vec::new();
    for det in detectors() {
        let cps = det.detect(&series);
        assert!(
            cps.is_empty(),
            "{}: empty input must yield no change-points",
            det.name()
        );
    }
}

#[test]
fn no_panic_on_single_sample() {
    let series = vec![(0.0_f64, 0.0_f64)];
    for det in detectors() {
        let cps = det.detect(&series);
        assert!(
            cps.is_empty(),
            "{}: single-sample input must yield no change-points",
            det.name()
        );
    }
}

#[test]
fn no_panic_on_constant_series() {
    // 200 identical samples — variance is zero, Hoeffding/Bayesian/PELT
    // costs all collapse to degenerate values. None of the detectors
    // should panic, divide by zero, or invent change-points.
    let series: Vec<(f64, f64)> = (0..200).map(|i| (i as f64, 7.0)).collect();
    for det in detectors() {
        let cps = det.detect(&series);
        assert!(
            cps.is_empty(),
            "{}: constant series must yield no change-points (got {})",
            det.name(),
            cps.len()
        );
    }
}

#[test]
fn no_panic_on_all_nan_series() {
    let series: Vec<(f64, f64)> = (0..50).map(|i| (i as f64, f64::NAN)).collect();
    for det in detectors() {
        // We do not assert the *value* of the result — NaN inputs may
        // legitimately confuse a detector — but we do assert it does
        // not panic, hang, or otherwise fail to return.
        let _ = det.detect(&series);
    }
}

#[test]
fn no_panic_on_infinite_series() {
    for sign in [1.0_f64, -1.0_f64] {
        let series: Vec<(f64, f64)> = (0..50).map(|i| (i as f64, sign * f64::INFINITY)).collect();
        for det in detectors() {
            let _ = det.detect(&series);
        }
    }
}

// ----------------------------------------------------------------------
// Stationary-input properties (randomised — every shrink finds a smaller
// counterexample if the property breaks, which is the entire reason for
// arbtest over a fixed unit case).
// ----------------------------------------------------------------------

/// Map an arbitrary byte stream to a bounded uniform real series in
/// [-1, 1]. The series is *stationary* by construction — every sample
/// is drawn from the same distribution, so no detector should fire
/// many change-points.
fn bounded_uniform_series(
    u: &mut arbtest::arbitrary::Unstructured<'_>,
) -> arbtest::arbitrary::Result<Vec<(f64, f64)>> {
    let raw_n: u8 = u.arbitrary()?;
    let n = (raw_n as usize).clamp(40, 240);
    let mut series = Vec::with_capacity(n);
    for i in 0..n {
        let raw: i8 = u.arbitrary()?;
        let v = (raw as f64) / 128.0;
        series.push((i as f64, v));
    }
    Ok(series)
}

#[test]
fn adwin_bounded_false_positive_on_stationary_uniform() {
    arbtest(|u| {
        let series = bounded_uniform_series(u)?;
        let n = series.len();
        let cps = Adwin::default().detect(&series);
        // The Hoeffding ε_cut on bounded data with δ=0.002 makes ADWIN
        // very conservative: empirically ≤ n/8 false positives on
        // uniform [-1, 1] for n ≤ 240. This is a loose ceiling — the
        // unit test in `src/baselines/adwin.rs` pins the strict
        // zero-false-positive case for *flat* input.
        assert!(
            cps.len() <= n / 8 + 1,
            "ADWIN: {} cps in {} stationary samples (limit {})",
            cps.len(),
            n,
            n / 8 + 1
        );
        Ok(())
    })
    .budget_ms(100);
}

#[test]
fn pelt_bounded_false_positive_on_stationary_uniform() {
    arbtest(|u| {
        let series = bounded_uniform_series(u)?;
        let n = series.len();
        let cps = Pelt::default().detect(&series);
        // PELT under BIC penalty `k·ln(n)·σ²` on uniform input is
        // self-tuning to scale; cap at n/10 + 1 for the loose
        // false-positive ceiling.
        assert!(
            cps.len() <= n / 10 + 1,
            "PELT: {} cps in {} stationary samples (limit {})",
            cps.len(),
            n,
            n / 10 + 1
        );
        Ok(())
    })
    .budget_ms(100);
}

#[test]
fn bocpd_bounded_false_positive_on_stationary_uniform() {
    arbtest(|u| {
        let series = bounded_uniform_series(u)?;
        let n = series.len();
        let cps = Bocpd::default().detect(&series);
        // BOCPD with map_drop_min=2 and λ=100 on stationary input is
        // dominated by the run-length prior (hazard ≈ 0.01), so
        // expected false positives ≈ n · 0.01. Cap at n/8 + 1 for
        // safety.
        assert!(
            cps.len() <= n / 8 + 1,
            "BOCPD: {} cps in {} stationary samples (limit {})",
            cps.len(),
            n,
            n / 8 + 1
        );
        Ok(())
    })
    .budget_ms(100);
}

// ----------------------------------------------------------------------
// Determinism property: the trait contract requires that `detect` is a
// pure function of the input series. Two calls with the same input must
// return byte-equal Vecs. Without this property the bake-off CSV
// fingerprint locks (`tests/deterministic_replay.rs`) would not be
// reachable for the baselines.
// ----------------------------------------------------------------------

#[test]
fn detectors_are_deterministic_on_arbitrary_input() {
    arbtest(|u| {
        let series = bounded_uniform_series(u)?;
        for det in detectors() {
            let a = det.detect(&series);
            let b = det.detect(&series);
            assert_eq!(
                a,
                b,
                "{}: detect() not deterministic (a={:?}, b={:?})",
                det.name(),
                a,
                b
            );
        }
        Ok(())
    })
    .budget_ms(100);
}
