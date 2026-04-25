//! `libm`-free f64 helpers for the `no_std` + `no_alloc` core.
//!
//! The default Rust `f64::sqrt` is a `std`-only method (backed by `libm`
//! on `no_std` targets). To avoid pulling `libm` or `std` into the core,
//! the handful of transcendental operations DSFB needs are implemented
//! here in portable f64 arithmetic with bounded iteration counts.

/// Absolute value of an `f64`.
///
/// Equivalent to `f64::abs`, but available in `no_std` without a
/// dependency on `libm`.
#[inline]
#[must_use]
pub fn abs_f64(x: f64) -> f64 {
    debug_assert!(!x.is_nan(), "abs_f64 called on NaN");
    if x.is_sign_negative() { -x } else { x }
}

/// Newton-Raphson square root of a non-negative `f64`.
///
/// Returns `None` if `x` is negative or non-finite. For `x = 0` returns
/// `Some(0.0)`. Converges to within 1 ULP of the true square root in at
/// most 64 iterations for any finite non-negative input (bounded-loop
/// guarantee for JPL Power-of-Ten Rule 2).
///
/// # Determinism
///
/// Pure function; identical input produces identical output. No floating-
/// point mode dependencies.
#[must_use]
pub fn sqrt_f64(x: f64) -> Option<f64> {
    if !x.is_finite() || x < 0.0 {
        return None;
    }
    if x == 0.0 {
        return Some(0.0);
    }

    // Initial estimate: a bit-trick that halves the exponent and gives
    // a rough approximation accurate to ~1 bit. Newton iteration then
    // converges quadratically.
    let bits = x.to_bits();
    let mut y = f64::from_bits((bits >> 1).wrapping_add(0x1FF8_0000_0000_0000));

    // Bounded Newton iteration: y_{n+1} = 0.5 * (y_n + x / y_n).
    // Convergence check tightens each iteration; 64 is a safe upper
    // bound across all finite non-negative f64 inputs.
    let mut prev = 0.0_f64;
    let mut i = 0_u8;
    while i < 64 {
        if y == prev {
            break;
        }
        prev = y;
        y = 0.5 * (y + x / y);
        i = i.saturating_add(1);
    }

    debug_assert!(y.is_finite(), "sqrt_f64 produced non-finite result");
    debug_assert!(y >= 0.0, "sqrt_f64 produced negative result");
    Some(y)
}

/// Windowed arithmetic mean of a non-empty slice of `f64`.
///
/// Returns `None` if the slice is empty or contains only non-finite
/// values. Non-finite values (NaN, ±∞) are skipped (missingness-aware),
/// matching the Stage III calibration protocol.
#[must_use]
pub fn finite_mean(xs: &[f64]) -> Option<f64> {
    debug_assert!(xs.len() <= usize::MAX / 2, "slice length unreasonable");
    let mut sum = 0.0_f64;
    let mut n = 0_usize;
    for &x in xs {
        if x.is_finite() {
            sum += x;
            n += 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f64)
    }
}

/// Windowed variance of a non-empty slice of `f64` (population form, `/N`).
///
/// Returns `None` if the slice is empty or contains no finite values.
/// Non-finite values are skipped. Uses the two-pass algorithm (mean then
/// deviation-squared) for numerical stability over Welford's online form
/// because the calibration window is small and bounded.
#[must_use]
pub fn finite_variance(xs: &[f64]) -> Option<f64> {
    let mean = finite_mean(xs)?;
    let mut ssq = 0.0_f64;
    let mut n = 0_usize;
    for &x in xs {
        if x.is_finite() {
            let d = x - mean;
            ssq += d * d;
            n += 1;
        }
    }
    debug_assert!(n > 0, "finite_variance: finite_mean returned Some but no finite samples");
    Some(ssq / n as f64)
}

/// Clamp a value to the inclusive range `[lo, hi]`.
///
/// Returns `lo` if `x < lo`, `hi` if `x > hi`, else `x`. `NaN` inputs
/// return `NaN` (explicit short-circuit so the caller can decide).
#[inline]
#[must_use]
pub fn clamp_f64(x: f64, lo: f64, hi: f64) -> f64 {
    debug_assert!(lo <= hi, "clamp_f64 called with lo > hi");
    if x.is_nan() {
        return x;
    }
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_zero() {
        assert_eq!(sqrt_f64(0.0), Some(0.0));
    }

    #[test]
    fn sqrt_perfect_squares() {
        for (x, r) in [(1.0, 1.0), (4.0, 2.0), (9.0, 3.0), (16.0, 4.0), (10_000.0, 100.0)] {
            let got = sqrt_f64(x).expect("finite non-negative");
            assert!((got - r).abs() < 1e-12, "sqrt({}) = {} (expected {})", x, got, r);
        }
    }

    #[test]
    fn sqrt_non_squares_within_ulp() {
        for x in [2.0_f64, 3.0, 5.0, 7.0, 0.5, 0.1, 1e-6, 1e6] {
            let got = sqrt_f64(x).expect("finite non-negative");
            // Cross-check: got * got should equal x within 2 ULPs.
            let back = got * got;
            let rel_err = abs_f64((back - x) / x);
            assert!(rel_err < 1e-14, "sqrt({}) back-check: rel_err = {}", x, rel_err);
        }
    }

    #[test]
    fn sqrt_negative_is_none() {
        assert_eq!(sqrt_f64(-1.0), None);
        assert_eq!(sqrt_f64(-0.5), None);
    }

    #[test]
    fn sqrt_non_finite_is_none() {
        assert_eq!(sqrt_f64(f64::NAN), None);
        assert_eq!(sqrt_f64(f64::INFINITY), None);
        assert_eq!(sqrt_f64(f64::NEG_INFINITY), None);
    }

    #[test]
    fn abs_basic() {
        assert_eq!(abs_f64(1.0), 1.0);
        assert_eq!(abs_f64(-1.0), 1.0);
        assert_eq!(abs_f64(0.0), 0.0);
        assert_eq!(abs_f64(-0.0), 0.0);
    }

    #[test]
    fn finite_mean_skips_non_finite() {
        let xs = [1.0, 2.0, f64::NAN, 3.0, f64::INFINITY, 4.0];
        let m = finite_mean(&xs).expect("some finite values");
        assert!((m - 2.5).abs() < 1e-12, "mean = {}", m);
    }

    #[test]
    fn finite_mean_empty_is_none() {
        assert_eq!(finite_mean(&[]), None);
    }

    #[test]
    fn finite_mean_all_nan_is_none() {
        assert_eq!(finite_mean(&[f64::NAN, f64::NAN]), None);
    }

    #[test]
    fn finite_variance_constant_is_zero() {
        assert_eq!(finite_variance(&[5.0; 10]), Some(0.0));
    }

    #[test]
    fn finite_variance_known() {
        // [1, 2, 3, 4, 5]: mean = 3, var = (4+1+0+1+4)/5 = 2.0
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let v = finite_variance(&xs).expect("finite");
        assert!((v - 2.0).abs() < 1e-12, "var = {}", v);
    }

    #[test]
    fn clamp_in_range() {
        assert_eq!(clamp_f64(0.5, 0.0, 1.0), 0.5);
        assert_eq!(clamp_f64(-0.5, 0.0, 1.0), 0.0);
        assert_eq!(clamp_f64(1.5, 0.0, 1.0), 1.0);
    }

    #[test]
    fn clamp_nan_propagates() {
        assert!(clamp_f64(f64::NAN, 0.0, 1.0).is_nan());
    }
}
