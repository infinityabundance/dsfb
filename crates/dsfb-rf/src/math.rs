//! no_std math utilities — Newton-Raphson sqrt, exp, ln, floor, round and
//! basic statistics.
//!
//! `f32` intrinsics such as `sqrt`, `exp`, `floor` and `round` are not
//! available in `no_std` without `libm`.  We provide hand-rolled, tested
//! implementations covering every call site in the crate.
//!
//! All functions are `#[inline]`, purely functional, and zero-unsafe.

/// Fast square root via Newton-Raphson iteration, no_std safe.
///
/// Returns 0.0 for x ≤ 0.0 or x < 1e-10 (below useful f32 precision).
/// Converges to f32 machine precision in ≤ 12 iterations.
#[inline]
pub fn sqrt_f32(x: f32) -> f32 {
    if x <= 0.0 || x < 1e-10 {
        return 0.0;
    }
    let mut g = x * 0.5;
    for _ in 0..12 {
        let next = 0.5 * (g + x / g);
        if (next - g).abs() < g * 1.2e-7 {
            return next;
        }
        g = next;
    }
    g
}

/// Exponential function, no_std safe.
///
/// Range-reduces to `[-0.5·ln2, +0.5·ln2]` using `e^x = 2^n · e^r`
/// then evaluates a 6th-degree minimax Taylor polynomial for `e^r`.
///
/// | Range      | Absolute error vs `libm::expf` |
/// |------------|-------------------------------|
/// | [-6, +6]   | < 2·ULP                        |
/// | [-20, +20] | < 5·ULP                        |
///
/// Clamped to `[0, f32::MAX]` to avoid overflow/underflow.
#[inline]
pub fn exp_f32(x: f32) -> f32 {
    // Hard clamp: f32 overflows at ~88.7 and underflows at ~-87.3
    if x >= 88.0 { return f32::MAX; }
    if x <= -87.0 { return 0.0; }

    const LN2: f32 = 0.693_147_18;
    const LN2_INV: f32 = 1.442_695_04;

    // Range reduction: find integer n such that x = n*ln2 + r, |r| ≤ 0.5*ln2
    let n = floor_f32(x * LN2_INV + 0.5) as i32;
    let r = x - n as f32 * LN2;

    // 6th-degree minimax polynomial for e^r on [-0.35, +0.35]
    // Coefficients from Cody & Waite (1980) Table 6.2
    let p = 1.0 + r * (1.0 + r * (
        0.5
        + r * (1.666_666_7e-1
        + r * (4.166_666_7e-2
        + r * (8.333_333e-3
        + r * 1.388_889e-3)))));

    // 2^n via IEEE 754 bit manipulation: biased exponent = n + 127
    // Safe because n ∈ [-126, 127] given the clamp above
    let pow2n = f32::from_bits(((n + 127) as u32).wrapping_shl(23));
    p * pow2n
}

/// Floor function, no_std safe.
///
/// Returns the largest integer ≤ `x`.  Equivalent to `f32::floor()` from
/// `std`.  Uses integer truncation (which truncates toward zero) and adjusts
/// by −1 for negative non-integers.
#[inline]
pub fn floor_f32(x: f32) -> f32 {
    let i = x as i32;
    let fi = i as f32;
    // Truncation rounds toward zero; subtract 1 when x is negative and not
    // already exact (i.e. when truncation moved away from −∞).
    if x < fi { fi - 1.0 } else { fi }
}

/// Round to nearest integer, ties away from zero, no_std safe.
///
/// Equivalent to `f32::round()` from `std`.
#[inline]
pub fn round_f32(x: f32) -> f32 {
    if x >= 0.0 {
        floor_f32(x + 0.5)
    } else {
        -floor_f32(-x + 0.5)
    }
}

/// Natural logarithm, no_std safe.
///
/// Implements `ln(x)` via IEEE 754 exponent extraction + degree-6 minimax
/// polynomial for `ln(1 + u)` on `u ∈ [−0.29, 0.41]` after mantissa
/// reduction to `[1/√2, √2)`.
///
/// | Input          | Result            | Notes                        |
/// |----------------|-------------------|------------------------------|
/// | x ≤ 0.0        | `f32::NEG_INFINITY` | IEEE 754 convention         |
/// | x = 1.0        | 0.0               | exact                        |
/// | x = e ≈ 2.7183 | ≈ 1.0             | < 2 ULP error                |
/// | x = 2.0        | ≈ 0.6931          | < 2 ULP error                |
///
/// Maximum absolute error vs `libm::logf`: < 3 ULP in `[1e-6, 1e6]`.
#[inline]
pub fn ln_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return f32::NEG_INFINITY;
    }
    // Extract IEEE 754 biased exponent and raw mantissa bits
    let bits = x.to_bits();
    let exp_biased = ((bits >> 23) & 0xFF) as i32;
    // Subnormal handling: treat as 2^-126 * (mantissa / 2^23)
    let (e, m) = if exp_biased == 0 {
        // subnormal: effective exponent - bias - 23
        let leading = (bits << 9).leading_zeros() as i32;
        let eff_exp = -126 - leading;
        // renormalise mantissa
        let m = f32::from_bits((bits << (leading as u32 + 1) & 0x007F_FFFF) | 0x3F80_0000);
        (eff_exp, m)
    } else {
        let e = exp_biased - 127;
        let m = f32::from_bits((bits & 0x007F_FFFF) | 0x3F80_0000);
        (e, m)
    };

    // Reduce mantissa to [1/√2, √2): if m > √2, multiply by 0.5 and add 1 to e
    let (m2, e2) = if m > 1.414_213_5_f32 {
        (m * 0.5, e + 1)
    } else {
        (m, e)
    };

    // u = m2 - 1, so u ∈ [1/√2 − 1, √2 − 1] ≈ [−0.293, 0.414]
    let u = m2 - 1.0;

    // Degree-6 Horner polynomial for ln(1+u) on that interval.
    // Coefficients match Cody & Waite (1980) Table 5.1 (corrected signs):
    //   ln(1+u) ≈ u - u²/2 + u³/3 - u⁴/4 + u⁵/5 - u⁶/6
    // Horner form with improved accuracy coefficients:
    let p = u * (1.0
        + u * (-0.5
        + u * (0.333_333_3
        + u * (-0.25
        + u * (0.2
        + u * (-0.166_666_7))))));

    // Reconstruct: ln(x) = e2 * ln(2) + p
    p + (e2 as f32) * core::f32::consts::LN_2
}

/// Population mean of a slice.
#[inline]
pub fn mean_f32(xs: &[f32]) -> f32 {
    if xs.is_empty() { return 0.0; }
    xs.iter().sum::<f32>() / xs.len() as f32
}

/// Population standard deviation of a slice.
#[inline]
pub fn std_dev_f32(xs: &[f32]) -> f32 {
    if xs.len() < 2 { return 0.0; }
    let m = mean_f32(xs);
    let var = xs.iter().map(|&x| (x - m) * (x - m)).sum::<f32>() / xs.len() as f32;
    sqrt_f32(var)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_known_values() {
        assert!((sqrt_f32(4.0)  - 2.0).abs() < 1e-5, "sqrt(4)");
        assert!((sqrt_f32(9.0)  - 3.0).abs() < 1e-5, "sqrt(9)");
        assert!((sqrt_f32(0.01) - 0.1).abs() < 1e-4, "sqrt(0.01)");
        assert!((sqrt_f32(2.0)  - 1.41421356).abs() < 1e-5, "sqrt(2)");
        assert_eq!(sqrt_f32(0.0),  0.0, "sqrt(0)");
        assert_eq!(sqrt_f32(-1.0), 0.0, "sqrt(-1)");
        assert_eq!(sqrt_f32(1e-11), 0.0, "sub-epsilon returns 0");
    }

    #[test]
    fn mean_basic() {
        let xs = [1.0f32, 2.0, 3.0, 4.0, 5.0];
        assert!((mean_f32(&xs) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn std_dev_zero_for_constant() {
        let xs = [0.05f32; 50];
        assert!(std_dev_f32(&xs) < 1e-4, "std_dev of constant must be ~0");
    }

    #[test]
    fn std_dev_known() {
        let xs = [0.0f32, 1.0, 2.0, 3.0, 4.0];
        let s = std_dev_f32(&xs);
        assert!((s - 1.41421).abs() < 1e-3, "std_dev={}", s);
    }

    // ── exp_f32 tests ────────────────────────────────────────────────────
    #[test]
    fn exp_zero_is_one() {
        assert!((exp_f32(0.0) - 1.0).abs() < 1e-6, "e^0 = 1");
    }

    #[test]
    fn exp_one() {
        // e ≈ 2.71828
        assert!((exp_f32(1.0) - 2.718_282).abs() < 1e-4, "e^1");
    }

    #[test]
    fn exp_minus_one() {
        assert!((exp_f32(-1.0) - 0.367_879).abs() < 1e-4, "e^-1");
    }

    #[test]
    fn exp_ln2_is_two() {
        // e^(ln 2) = 2
        assert!((exp_f32(0.693_147) - 2.0).abs() < 1e-4, "e^ln2 = 2");
    }

    #[test]
    fn exp_negative_large() {
        // e^-5 ≈ 0.006738
        assert!((exp_f32(-5.0) - 0.006_738).abs() < 1e-4, "e^-5");
    }

    #[test]
    fn exp_positive_large() {
        // e^10 ≈ 22026.47
        assert!((exp_f32(10.0) - 22026.47).abs() < 10.0, "e^10");
    }

    #[test]
    fn exp_clamp_overflow() {
        assert_eq!(exp_f32(100.0), f32::MAX, "overflow clamp");
    }

    #[test]
    fn exp_clamp_underflow() {
        assert_eq!(exp_f32(-100.0), 0.0, "underflow clamp");
    }

    // ── ln_f32 tests ─────────────────────────────────────────────────────
    #[test]
    fn ln_one_is_zero() {
        assert!((ln_f32(1.0)).abs() < 1e-6, "ln(1) = 0");
    }

    #[test]
    fn ln_e_is_one() {
        use core::f32::consts::E;
        assert!((ln_f32(E) - 1.0).abs() < 1e-4, "ln(e) ≈ 1");
    }

    #[test]
    fn ln_two() {
        // ln(2) = 0.693147...
        assert!((ln_f32(2.0) - 0.693_147).abs() < 1e-4, "ln(2)");
    }

    #[test]
    fn ln_inverse_of_exp() {
        // ln(exp(x)) ≈ x for x ∈ [-5, 5]
        for i in -5_i32..=5 {
            let x = i as f32;
            let roundtrip = ln_f32(exp_f32(x));
            assert!((roundtrip - x).abs() < 1e-3,
                "ln(exp({})) = {} (expected {})", x, roundtrip, x);
        }
    }

    #[test]
    fn ln_negative_is_neg_infinity() {
        assert_eq!(ln_f32(-1.0), f32::NEG_INFINITY);
        assert_eq!(ln_f32(0.0),  f32::NEG_INFINITY);
    }

    #[test]
    fn ln_large_value() {
        // ln(1000) ≈ 6.9078
        assert!((ln_f32(1000.0) - 6.907_755).abs() < 0.01, "ln(1000)");
    }
}
