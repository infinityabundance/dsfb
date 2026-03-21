//! Minimal fixed-point helpers for the bounded live-engine path.
//!
//! This module does not claim a full fixed-point rewrite of the crate. It provides a documented
//! and auditable experimental backend for the online residual/sign/syntax/grammar deployment path.

/// Stable machine-readable label for the experimental fixed-point backend.
pub const FIXED_POINT_NUMERIC_MODE: &str = "fixed_q16_16";
/// Fractional bits used by the experimental fixed-point backend.
pub const FIXED_POINT_FRACTIONAL_BITS: u32 = 16;
const FIXED_POINT_SCALE: f64 = (1u64 << FIXED_POINT_FRACTIONAL_BITS) as f64;

// TRACE:ASSUMPTION:ASM-FIXED-POINT-QUANTIZATION:Fixed-point ingress quantization:The experimental embedded backend quantizes online residual inputs to q16.16 with saturating conversion before the conservative layered path runs.
/// Quantizes a floating-point value into the experimental q16.16 fixed-point representation.
#[must_use]
pub fn quantize_q16_16(value: f64) -> i64 {
    let scaled = (value * FIXED_POINT_SCALE).round();
    if scaled.is_nan() {
        0
    } else if scaled > i64::MAX as f64 {
        i64::MAX
    } else if scaled < i64::MIN as f64 {
        i64::MIN
    } else {
        scaled as i64
    }
}

/// Dequantizes a q16.16 fixed-point value back into `f64`.
#[must_use]
pub fn dequantize_q16_16(value: i64) -> f64 {
    value as f64 / FIXED_POINT_SCALE
}

/// Returns a conservative documentation string describing the current overflow policy.
#[must_use]
pub const fn fixed_point_overflow_policy() -> &'static str {
    "saturating ingress quantization to q16.16; downstream typed engine logic consumes the quantized values deterministically"
}

#[cfg(test)]
mod tests {
    use super::{dequantize_q16_16, fixed_point_overflow_policy, quantize_q16_16};

    #[test]
    fn q16_16_roundtrip_is_reasonable() {
        let quantized = quantize_q16_16(1.25);
        let restored = dequantize_q16_16(quantized);
        assert!((restored - 1.25).abs() < 1.0e-4);
    }

    #[test]
    fn overflow_policy_note_is_nonempty() {
        assert!(fixed_point_overflow_policy().contains("saturating"));
    }
}
