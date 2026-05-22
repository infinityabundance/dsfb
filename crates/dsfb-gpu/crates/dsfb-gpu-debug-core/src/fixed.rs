//! Q16.16 signed fixed-point arithmetic.
//!
//! Why fixed-point: the CPU reference path and the CUDA kernels must produce
//! byte-identical outputs cell-for-cell so the hash chain in the case file is
//! the same regardless of whether evidence was produced on the host or the
//! device. Floating point makes byte-equivalence fragile across compilers,
//! drivers, fused-multiply-add behavior, and reduction order. Q16.16 with
//! explicit rounding sidesteps all of that.
//!
//! Why Q16.16 specifically: 16.16 strikes a balance between dynamic range
//! (i32 covers roughly ±32_767 in the integer half) and precision (1/65_536
//! in the fractional half) that fits this workload — latencies in
//! milliseconds, error-rate fractions, and smoothed-residual magnitudes all
//! sit comfortably inside that band once inputs are clamped at the boundary.
//!
//! Why these specific operations: `sat_add`, `sat_sub`, `sat_mul`, `sat_div`,
//! and `abs` are the only operations the downstream pipeline needs. No FMA,
//! no SIMD intrinsics. The same scalar code must run on the CPU and inside a
//! CUDA kernel, so the implementation is restricted to plain integer math
//! that both backends can express identically.
//!
//! Rounding rule: multiplication widens to i64, then applies round-half-to-
//! even (banker's rounding) at bit 15 before shifting right by 16. This rule
//! is symmetric, deterministic across architectures, and reduces the bias
//! that round-half-up would inject into long EWMA recurrences.

#![allow(clippy::module_name_repetitions)]

/// Signed Q16.16 fixed-point value.
///
/// Layout: a single `i32` where the high 16 bits are the integer part and
/// the low 16 bits are the fractional part. `#[repr(transparent)]` so the
/// type can cross the FFI boundary as a plain `int32_t` without conversion.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct Q16(pub i32);

impl Q16 {
    /// Q16.16 representation of zero. Used as an EWMA seed and as the
    /// "no evidence" axis value before any detector has fired.
    pub const ZERO: Q16 = Q16(0);

    /// Q16.16 representation of one. The fractional half is zero, the
    /// integer half is one. `1 << 16 == 65_536`.
    pub const ONE: Q16 = Q16(1 << 16);

    /// The smallest representable value. Saturating arithmetic clamps here
    /// rather than wrapping.
    pub const MIN: Q16 = Q16(i32::MIN);

    /// The largest representable value. Saturating arithmetic clamps here
    /// rather than wrapping.
    pub const MAX: Q16 = Q16(i32::MAX);

    /// Construct a Q16.16 value from an integer. Sign-extended into the
    /// high 16 bits. Caller is responsible for keeping `|x| ≤ 32_767`.
    #[must_use]
    pub const fn from_int(x: i16) -> Q16 {
        Q16((x as i32) << 16)
    }

    /// Construct a Q16.16 value from its raw `i32` bit pattern. Used by the
    /// contract loader (`ewma_alpha_q16_raw`) and by tests that need to pin
    /// a specific bit value.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Q16 {
        Q16(raw)
    }

    /// Return the raw `i32` representation. The hash-chain serializer
    /// writes raw words as zero-padded big-endian hex so the canonical bytes
    /// are stable regardless of host endianness.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Saturating addition. Both CPU and CUDA paths must use the same rule
    /// because per-cell results would otherwise disagree at the boundaries.
    #[must_use]
    pub const fn sat_add(self, b: Q16) -> Q16 {
        Q16(self.0.saturating_add(b.0))
    }

    /// Saturating subtraction. Mirrors `sat_add`.
    #[must_use]
    pub const fn sat_sub(self, b: Q16) -> Q16 {
        Q16(self.0.saturating_sub(b.0))
    }

    /// Saturating multiplication with round-half-to-even.
    ///
    /// The product is widened to i64 to avoid overflow before the rounding
    /// shift. Banker's rounding is applied at bit 15: ties round toward the
    /// even result. After rounding the value is shifted right by 16 (the
    /// fractional width) and saturated back to i32.
    ///
    /// This function is the load-bearing primitive for CPU↔GPU bit equality.
    /// Any divergence in its definition between the two backends would break
    /// stage-by-stage hash equivalence. The CUDA implementation in
    /// `cuda/common.cuh` is intentionally a line-for-line transliteration.
    #[must_use]
    pub const fn sat_mul(self, b: Q16) -> Q16 {
        let prod: i64 = (self.0 as i64) * (b.0 as i64);
        let rounded: i64 = round_half_even_i64_shift(prod, 16);
        Q16(saturate_i64_to_i32(rounded))
    }

    /// Saturating division. Returns `Q16::ZERO` when the divisor is zero.
    /// The pipeline never divides on the hot path; this exists for the
    /// occasional baseline computation and is included for completeness.
    #[must_use]
    pub const fn sat_div(self, b: Q16) -> Q16 {
        if b.0 == 0 {
            return Q16::ZERO;
        }
        // Shift numerator left by the fractional width first, then divide.
        // Performed in i64 to keep the pre-divide shift safe.
        let num: i64 = (self.0 as i64) << 16;
        Q16(saturate_i64_to_i32(num / (b.0 as i64)))
    }

    /// Absolute value, saturating. `Q16::MIN.abs()` returns `Q16::MAX`
    /// rather than panicking or wrapping.
    #[must_use]
    pub const fn abs(self) -> Q16 {
        Q16(self.0.saturating_abs())
    }

    /// Test for the additive identity. Used by axis gates that need to know
    /// "no evidence" without comparing two `Q16` values.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Linear interpolation: `self + alpha * (other - self)`, computed in
    /// saturating Q16.16. Convenience wrapper used by the EWMA recurrence.
    /// `alpha` is expected to live in `[0, ONE]`; values outside that band
    /// still produce a deterministic result but the EWMA interpretation
    /// stops being meaningful.
    #[must_use]
    pub const fn lerp(self, other: Q16, alpha: Q16) -> Q16 {
        let diff = other.sat_sub(self);
        let step = alpha.sat_mul(diff);
        self.sat_add(step)
    }
}

/// Saturate an `i64` to `i32`. Standalone so both the CPU and CUDA paths use
/// the same clamp rule.
#[must_use]
const fn saturate_i64_to_i32(x: i64) -> i32 {
    if x > i32::MAX as i64 {
        i32::MAX
    } else if x < i32::MIN as i64 {
        i32::MIN
    } else {
        x as i32
    }
}

/// Right shift with round-half-to-even at the cut.
///
/// `bits` is expected to be in `1..=32`. For `bits == 0` the input is
/// returned unchanged. The rule: take the bits about to be discarded; if
/// the top discarded bit is 0 the result rounds down (truncation); if the
/// top discarded bit is 1 and any of the remaining discarded bits is 1 the
/// result rounds away from zero (round-up for positive, round-down for
/// negative magnitudes); if the top discarded bit is 1 and all remaining
/// discarded bits are 0 (exact midpoint), the result is rounded toward the
/// even neighbor.
#[must_use]
const fn round_half_even_i64_shift(value: i64, bits: u32) -> i64 {
    if bits == 0 {
        return value;
    }
    let truncated: i64 = value >> bits;
    let halfway_bit: i64 = 1i64 << (bits - 1);
    let discarded: i64 = value & ((1i64 << bits) - 1);
    let remainder_below_half: i64 = discarded & (halfway_bit - 1);
    let at_half: bool = discarded == halfway_bit && remainder_below_half == 0;
    if at_half {
        // Exact midpoint: round toward even.
        if truncated & 1 == 0 {
            truncated
        } else {
            truncated.saturating_add(1)
        }
    } else if discarded > halfway_bit {
        truncated.saturating_add(1)
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_have_expected_raw_bits() {
        assert_eq!(Q16::ZERO.raw(), 0);
        assert_eq!(Q16::ONE.raw(), 1 << 16);
        assert_eq!(Q16::MIN.raw(), i32::MIN);
        assert_eq!(Q16::MAX.raw(), i32::MAX);
    }

    #[test]
    fn from_int_round_trips_through_integer_part() {
        for x in [-32_767_i16, -1, 0, 1, 32_767] {
            let q = Q16::from_int(x);
            assert_eq!(q.raw() >> 16, i32::from(x));
            assert_eq!(q.raw() & 0xFFFF, 0);
        }
    }

    #[test]
    fn sat_add_clamps_at_max() {
        assert_eq!(Q16::MAX.sat_add(Q16::ONE), Q16::MAX);
        assert_eq!(Q16::MIN.sat_add(Q16::from_int(-1)), Q16::MIN);
        assert_eq!(Q16::from_int(2).sat_add(Q16::from_int(3)), Q16::from_int(5));
    }

    #[test]
    fn sat_sub_clamps_at_min() {
        assert_eq!(Q16::MIN.sat_sub(Q16::ONE), Q16::MIN);
        assert_eq!(Q16::MAX.sat_sub(Q16::from_int(-1)), Q16::MAX);
        assert_eq!(Q16::from_int(5).sat_sub(Q16::from_int(3)), Q16::from_int(2));
    }

    #[test]
    fn sat_mul_is_one_identity() {
        for x in [-100, -1, 0, 1, 100, 32_767] {
            let q = Q16::from_int(x);
            assert_eq!(q.sat_mul(Q16::ONE), q);
            assert_eq!(Q16::ONE.sat_mul(q), q);
        }
    }

    #[test]
    fn sat_mul_half_times_half_is_quarter() {
        // 0.5 raw == 0x8000 == 32_768. 0.5 * 0.5 = 0.25 raw == 0x4000.
        let half = Q16::from_raw(0x8000);
        let quarter = Q16::from_raw(0x4000);
        assert_eq!(half.sat_mul(half), quarter);
    }

    #[test]
    fn sat_mul_handles_negative_signs() {
        let a = Q16::from_int(-3);
        let b = Q16::from_int(4);
        assert_eq!(a.sat_mul(b), Q16::from_int(-12));
        assert_eq!(b.sat_mul(a), Q16::from_int(-12));
        assert_eq!(a.sat_mul(a), Q16::from_int(9));
    }

    #[test]
    fn sat_mul_saturates_on_overflow() {
        // 30_000 * 30_000 = 9e8 — far beyond i32::MAX / 65_536 ≈ 32_768.
        let big = Q16::from_int(30_000);
        assert_eq!(big.sat_mul(big), Q16::MAX);
        assert_eq!(big.sat_mul(Q16::from_int(-30_000)), Q16::MIN);
    }

    #[test]
    fn sat_div_one_is_identity() {
        for x in [-100, -1, 0, 1, 100] {
            let q = Q16::from_int(x);
            assert_eq!(q.sat_div(Q16::ONE), q);
        }
    }

    #[test]
    fn sat_div_by_zero_returns_zero() {
        assert_eq!(Q16::from_int(5).sat_div(Q16::ZERO), Q16::ZERO);
        assert_eq!(Q16::from_int(-5).sat_div(Q16::ZERO), Q16::ZERO);
    }

    #[test]
    fn abs_saturates_at_min() {
        assert_eq!(Q16::MIN.abs(), Q16::MAX);
        assert_eq!(Q16::from_int(-7).abs(), Q16::from_int(7));
        assert_eq!(Q16::from_int(7).abs(), Q16::from_int(7));
    }

    #[test]
    fn lerp_at_zero_returns_self() {
        let a = Q16::from_int(10);
        let b = Q16::from_int(20);
        assert_eq!(a.lerp(b, Q16::ZERO), a);
    }

    #[test]
    fn lerp_at_one_returns_other() {
        let a = Q16::from_int(10);
        let b = Q16::from_int(20);
        assert_eq!(a.lerp(b, Q16::ONE), b);
    }

    #[test]
    fn lerp_at_half_returns_midpoint() {
        let a = Q16::from_int(10);
        let b = Q16::from_int(20);
        let half = Q16::from_raw(0x8000);
        assert_eq!(a.lerp(b, half), Q16::from_int(15));
    }

    #[test]
    fn ewma_recurrence_with_locked_alpha_is_stable() {
        // alpha = 0.125 (0x2000 raw). Constant input x. The EWMA should
        // monotonically approach x without ever overshooting it.
        //
        // With banker's rounding and alpha = 1/8, convergence stalls when
        // the per-step delta drops to 4 raw Q16 units (4 * 8192 = 32_768,
        // which is exactly halfway and rounds to the even neighbor 0). So
        // the asymptotic gap is 4 raw units rather than 0 — a known
        // property of the discrete recurrence we are intentionally pinning.
        let alpha = Q16::from_raw(0x2000);
        let x = Q16::from_int(100);
        let mut ewma = Q16::ZERO;
        let mut last = Q16::MIN;
        for _ in 0..200 {
            ewma = ewma.lerp(x, alpha);
            // Each step is closer to x than the previous one (monotone non-decreasing).
            assert!(ewma.raw() >= last.raw());
            assert!(ewma.raw() <= x.raw());
            last = ewma;
        }
        // After convergence, the EWMA sits within the rounding floor of x.
        let final_gap = x.sat_sub(ewma).raw();
        assert!(
            final_gap <= 4,
            "expected gap ≤ 4 raw units, got {final_gap}"
        );
    }

    #[test]
    fn round_half_even_breaks_ties_to_even() {
        // 0.5 rounds to 0 (even); 1.5 rounds to 2 (even); 2.5 rounds to 2; 3.5 rounds to 4.
        // Test by constructing values whose discarded bits are exactly the halfway pattern.
        let cases: [(i64, i64); 4] = [
            (0b0_1000, 0),  // 0.5 ↦ 0
            (0b1_1000, 2),  // 1.5 ↦ 2
            (0b10_1000, 2), // 2.5 ↦ 2
            (0b11_1000, 4), // 3.5 ↦ 4
        ];
        for (input, expected) in cases {
            assert_eq!(
                round_half_even_i64_shift(input, 4),
                expected,
                "input={input:#b}"
            );
        }
    }

    #[test]
    fn round_half_even_handles_negative_midpoints() {
        // -0.5 ↦ 0 (even); -1.5 ↦ -2; -2.5 ↦ -2; -3.5 ↦ -4.
        let cases: [(i64, i64); 4] = [
            (-(0b0_1000_i64), 0),
            (-(0b1_1000_i64), -2),
            (-(0b10_1000_i64), -2),
            (-(0b11_1000_i64), -4),
        ];
        for (input, expected) in cases {
            assert_eq!(
                round_half_even_i64_shift(input, 4),
                expected,
                "input={input}"
            );
        }
    }
}
