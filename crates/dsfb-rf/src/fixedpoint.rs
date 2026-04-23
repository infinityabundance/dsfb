//! Fixed-point ingress path for FPGA soft-core and bare-metal deployment.
//!
//! ## Context
//!
//! While `dsfb-rf` is purely `f32`-based in the observer hot path, a number
//! of deployment targets **lack a hardware FPU**:
//!
//! - RISC-V RV32I without the `F` extension (e.g., `riscv32imac-unknown-none-elf`)
//! - Cortex-M0/M0+ (ARMv6-M — no FPU)
//! - FPGA soft-cores (MicroBlaze without DSP48, PicoRV32)
//! - Custom ASICs / VLSI pipelines in e.g. C-UAS / EW front-ends
//!
//! In these contexts, the ADC lane residual arrives as a raw integer sample
//! and converting to `f32` before the observer introduces software FPU
//! overhead.  The `Q16.16` fixed-point format provides:
//!
//! - **16 integer bits**: supports residual norms up to 65535 (far beyond any
//!   normalised IQ residual)
//! - **16 fractional bits**: resolution of 2⁻¹⁶ ≈ 1.526 × 10⁻⁵ —
//!   comfortably below the 14-bit ADC quantisation step on typical SDRs
//! - **32-bit arithmetic**: fits in a single 32-bit ALU word; multiply in
//!   `i64` then shift avoids overflow
//!
//! ## Format conventions
//!
//! An `i32` in Q16.16 format represents the real number `x = raw / 2^16`.
//!
//! ```text
//! quantize(x)     = round(x × 2^16)     [f64 → Q16.16 i32]
//! dequantize(raw) = raw as f64 / 2^16   [Q16.16 i32 → f64]
//! to_f32(raw)     = raw as f32 / 65536.0 [Q16.16 i32 → f32 for observer]
//! ```
//!
//! Saturation on overflow prevents undefined behaviour and limits anomaly
//! injection from wildly out-of-range inputs.
//!
//! ## DSFB-Semiotics-Engine source
//!
//! The `quantize_q16_16` / `dequantize_q16_16` pair mirrors the reference
//! implementation in `dsfb-semiotics-engine/math/fixed_point.rs` (de Beer 2026).
//! The mode label `"fixed_q16_16"` is preserved for provenance traceability
//! in SigMF annotations.
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - All functions are `#[inline]` for zero-cost abstraction
//! - No `libm` dependency — arithmetic is pure integer + bit-shift

/// Quantise a `f64` value into Q16.16 fixed-point format (`i32`).
///
/// ## Formula
///
/// ```text
/// q = round(x × 2^16) = round(x × 65536.0)
/// ```
///
/// Saturates at `i32::MIN` / `i32::MAX` on overflow.
///
/// ## Accuracy
///
/// Representable range: −32768.0 … +32767.999985 (≈ ±32768).
/// Resolution: 1/65536 ≈ 1.526 × 10⁻⁵.
///
/// For normalised IQ residual norms in [0, 1] the quantisation error is
/// bounded by 0.5 × 2⁻¹⁶ ≈ 7.6 × 10⁻⁶, well below 14-bit ADC LSB.
#[inline]
pub fn quantize_q16_16(x: f64) -> i32 {
    const SCALE: f64 = 65536.0; // 2^16
    let scaled = x * SCALE;
    let rounded = if scaled >= 0.0 {
        scaled + 0.5
    } else {
        scaled - 0.5
    };
    // Saturating cast to i32
    if rounded >= i32::MAX as f64 {
        i32::MAX
    } else if rounded <= i32::MIN as f64 {
        i32::MIN
    } else {
        rounded as i32
    }
}

/// Dequantise a Q16.16 `i32` back to `f64`.
///
/// ## Formula
///
/// ```text
/// x = raw / 2^16 = raw as f64 / 65536.0
/// ```
///
/// Round-trip accuracy: |dequantize(quantize(x)) − x| ≤ 2⁻¹⁷ ≈ 7.6 × 10⁻⁶.
#[inline]
pub fn dequantize_q16_16(raw: i32) -> f64 {
    raw as f64 / 65536.0
}

/// Convert a Q16.16 `i32` directly to an `f32` for the observer hot path.
///
/// Equivalent to `dequantize_q16_16(raw) as f32` but avoids the intermediate
/// `f64` on platforms where `f32` is the native float type.
#[inline]
pub fn q16_16_to_f32(raw: i32) -> f32 {
    raw as f32 / 65536.0_f32
}

/// Quantise an `f32` into Q16.16.
///
/// Convenience wrapper over `quantize_q16_16` for callers already working
/// in `f32` (avoids widening to `f64` on soft-float platforms).
#[inline]
pub fn quantize_f32(x: f32) -> i32 {
    quantize_q16_16(x as f64)
}

/// Multiply two Q16.16 values and return a Q16.16 result.
///
/// Internally uses `i64` to prevent overflow during the multiply.
///
/// ## Formula
///
/// ```text
/// result = (a as i64 * b as i64) >> 16
/// ```
///
/// Saturates on overflow (result > i32::MAX or < i32::MIN).
#[inline]
pub fn mul_q16_16(a: i32, b: i32) -> i32 {
    let prod = a as i64 * b as i64;
    let shifted = prod >> 16;
    if shifted > i32::MAX as i64 {
        i32::MAX
    } else if shifted < i32::MIN as i64 {
        i32::MIN
    } else {
        shifted as i32
    }
}

/// Add two Q16.16 values with saturation.
#[inline]
pub fn add_q16_16(a: i32, b: i32) -> i32 {
    a.saturating_add(b)
}

/// Provenance mode label for SigMF annotation.
///
/// Include this literal in any SigMF `dsfb:quantization_mode` annotation
/// field to identify the Q16.16 fixed-point ingress path.
pub const MODE_LABEL: &str = "fixed_q16_16";

/// Fractional bits in the Q16.16 format.
pub const FRAC_BITS: u32 = 16;

/// Scale factor: 2^FRAC_BITS.
pub const SCALE: i32 = 1 << FRAC_BITS; // 65536

/// Maximum representable value in Q16.16 as `f64`.
pub const MAX_VALUE: f64 = i32::MAX as f64 / 65536.0;

/// Minimum representable value in Q16.16 as `f64`.
pub const MIN_VALUE: f64 = i32::MIN as f64 / 65536.0;

/// Resolution (1 LSB) of the Q16.16 format in real units.
pub const RESOLUTION: f64 = 1.0 / 65536.0;

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_zero() {
        let raw = quantize_q16_16(0.0);
        let back = dequantize_q16_16(raw);
        assert_eq!(raw, 0);
        assert!((back - 0.0).abs() < 1e-10);
    }

    #[test]
    fn round_trip_one() {
        let raw = quantize_q16_16(1.0);
        assert_eq!(raw, 65536);
        let back = dequantize_q16_16(raw);
        assert!((back - 1.0).abs() < 1e-10, "back={}", back);
    }

    #[test]
    fn round_trip_negative() {
        let raw = quantize_q16_16(-1.0);
        assert_eq!(raw, -65536);
        let back = dequantize_q16_16(raw);
        assert!((back - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn round_trip_fractional() {
        let x = 0.12345_f64;
        let raw = quantize_q16_16(x);
        let back = dequantize_q16_16(raw);
        assert!(
            (back - x).abs() < RESOLUTION + 1e-14,
            "round-trip error {} (expected < {})", (back - x).abs(), RESOLUTION
        );
    }

    #[test]
    fn saturation_on_overflow() {
        let raw = quantize_q16_16(1_000_000.0);
        assert_eq!(raw, i32::MAX, "must saturate at i32::MAX");
        let raw_neg = quantize_q16_16(-1_000_000.0);
        assert_eq!(raw_neg, i32::MIN, "must saturate at i32::MIN");
    }

    #[test]
    fn multiply_integers() {
        // 2.0 × 3.0 = 6.0 in Q16.16
        let a = quantize_q16_16(2.0);
        let b = quantize_q16_16(3.0);
        let c = mul_q16_16(a, b);
        let back = dequantize_q16_16(c);
        assert!((back - 6.0).abs() < 1e-4, "2×3={}", back);
    }

    #[test]
    fn multiply_fractions() {
        // 0.5 × 0.5 = 0.25
        let a = quantize_q16_16(0.5);
        let b = quantize_q16_16(0.5);
        let c = mul_q16_16(a, b);
        let back = dequantize_q16_16(c);
        assert!((back - 0.25).abs() < 1e-4, "0.5×0.5={}", back);
    }

    #[test]
    fn add_saturates() {
        let big = i32::MAX / 2 + 10;
        let sum = add_q16_16(big, big);
        assert_eq!(sum, i32::MAX, "overflow must saturate");
    }

    #[test]
    fn f32_conversion_matches() {
        let x = 0.0456_f32;
        let raw = quantize_f32(x);
        let back = q16_16_to_f32(raw);
        assert!((back - x).abs() < 2.0 * RESOLUTION as f32, "f32 round-trip error");
    }

    #[test]
    fn constants_consistent() {
        assert_eq!(SCALE, 65536);
        assert_eq!(FRAC_BITS, 16);
        assert!((MAX_VALUE - (i32::MAX as f64 / 65536.0)).abs() < 1e-10);
        assert!((MIN_VALUE - (i32::MIN as f64 / 65536.0)).abs() < 1e-10);
        assert!((RESOLUTION - 1.0 / 65536.0).abs() < 1e-20);
    }

    #[test]
    fn mode_label_is_canonical() {
        assert_eq!(MODE_LABEL, "fixed_q16_16");
    }

    // ── Periodic resync ────────────────────────────────────────────────────

    #[test]
    fn periodic_resync_triggers_at_period() {
        let cfg = PeriodicResyncConfig { period: 1000, max_drift_ulps: 100 };
        assert!(cfg.should_resync(999));
        assert!(!cfg.should_resync(998));
    }

    #[test]
    fn apply_resync_clamps_to_envelope() {
        // Q16.16 value that has drifted slightly above the envelope rho
        let rho_q = quantize_f32(0.10);
        let drifted = rho_q + 200; // small ulp drift
        let (clamped, corrected) = apply_periodic_resync(drifted, rho_q, 100);
        assert!(corrected, "value outside tolerance should be corrected");
        assert!(clamped <= rho_q, "clamped value must not exceed rho_q");
    }

    #[test]
    fn apply_resync_within_tolerance_unchanged() {
        let rho_q = quantize_f32(0.10);
        let close  = rho_q + 50; // within 100 ulp tolerance
        let (val, corrected) = apply_periodic_resync(close, rho_q, 100);
        assert!(!corrected, "within-tolerance value must not be corrected");
        assert_eq!(val, close, "within-tolerance value must be unchanged");
    }
}

// ── Periodic State Resynchronisation ────────────────────────────────────────
//
// DEFENCE: "Fixed-Point Precision Loss" (paper §XIX-F).
//
// Over billions of fixed-point accumulator steps, rounding errors perform a
// random walk.  The Allan deviation of the rounding noise has a floor slope of
// −0.5 (white noise) in log-log, meaning cumulative drift grows as √N .
// At N = 1×10⁹ samples in Q16.16, worst-case cumulative error ≈ √(10⁹) × 2⁻¹⁶
// ≈ 0.49 — well into the signal band.
//
// Defence: every `period` samples, compare the Q16.16 accumulator to the
// current admissibility envelope and re-zero if drift exceeds `max_drift_ulps`
// (Unit in the Last Place at fractional bit 16 = 1/65536 ≈ 1.5 × 10⁻⁵).
// This bounds the random walk to a finite error budget without changing the
// mathematical invariant of the pipeline — a necessary periodic perturbation
// correction equivalent to re-painting the "zero" of a measurement instrument.
//
// The `test_long_duration_stability` test in `tests/long_duration_stability.rs`
// validates this defence over 1 000 000 observations (≈ 15 seconds at 64 kSPS).

/// Configuration for periodic fixed-point accumulator resynchronisation.
///
/// ## Usage
///
/// At every observation step, call `should_resync(obs_count % period)`.
/// When `true`, call `apply_periodic_resync()` on each Q16.16 accumulator
/// that carries a running sum over an extended session.
///
/// ## Default
///
/// `period = 65536` (one full Q16.16 fractional cycle, ≈ 1 s at 64 kSPS).
/// `max_drift_ulps = 32` (½ LSB of a 16-bit ADC ≈ 7.6 × 10⁻⁶).
///
/// # Examples
///
/// ```
/// use dsfb_rf::fixedpoint::{PeriodicResyncConfig, apply_periodic_resync,
///                             quantize_f32};
/// let cfg = PeriodicResyncConfig { period: 1000, max_drift_ulps: 50 };
/// let rho_q = quantize_f32(0.10);
/// let drifted = rho_q + 80;
/// if cfg.should_resync(999) {
///     let (v, _) = apply_periodic_resync(drifted, rho_q, cfg.max_drift_ulps);
///     let _ = v; // re-zeroed value
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PeriodicResyncConfig {
    /// Number of observations between resync opportunities.
    pub period: u32,
    /// Maximum allowed accumulator drift in Q16.16 ULPs before correction.
    /// 1 ULP = 1/65536 ≈ 1.53 × 10⁻⁵ in f32 terms.
    pub max_drift_ulps: i32,
}

impl PeriodicResyncConfig {
    /// Default configuration: period 65536, tolerance 32 ULPs.
    pub const DEFAULT: Self = Self { period: 65536, max_drift_ulps: 32 };

    /// Returns `true` when `obs_mod_period == period - 1` (epoch boundary).
    ///
    /// Call as: `cfg.should_resync(obs_count % cfg.period)`.
    #[inline]
    pub fn should_resync(&self, obs_mod_period: u32) -> bool {
        obs_mod_period == self.period.saturating_sub(1)
    }
}

/// Apply a periodic resynchronisation correction to one Q16.16 accumulator.
///
/// If `|accumulator - reference_q| > max_drift_ulps`, the accumulator is
/// **clamped to `reference_q`** and the second return value is `true` (correction applied).
/// Otherwise the accumulator is returned unchanged (`false`).
///
/// ## Arguments
///
/// * `accumulator`  — current Q16.16 accumulator value.
/// * `reference_q`  — the current admissibility envelope as Q16.16 (ρ_Q).
/// * `max_drift_ulps` — ULP tolerance before triggering correction.
///
/// ## Returns
///
/// `(corrected_value, was_corrected)`
///
/// # Examples
///
/// ```
/// use dsfb_rf::fixedpoint::{apply_periodic_resync, quantize_f32};
/// let rho_q = quantize_f32(0.10);
/// let (v, fixed) = apply_periodic_resync(rho_q + 200, rho_q, 100);
/// assert!(fixed && v == rho_q);
/// ```
#[inline]
pub fn apply_periodic_resync(
    accumulator:    i32,
    reference_q:    i32,
    max_drift_ulps: i32,
) -> (i32, bool) {
    let drift = accumulator.wrapping_sub(reference_q);
    let abs_drift = drift.unsigned_abs() as i32;
    if abs_drift > max_drift_ulps {
        (reference_q, true)
    } else {
        (accumulator, false)
    }
}
