// cuda/common.cuh
//
// Q16.16 fixed-point primitives, mirroring `dsfb-gpu-debug-core/src/fixed.rs`
// line-for-line. The CPU and GPU paths must produce byte-identical per-cell
// outputs, so any change here has to be made in lockstep with the Rust
// counterpart.
//
// Why these specific operations: only `sat_add`, `sat_sub`, `sat_mul`,
// `sat_div`, and `abs` are needed by the downstream pipeline. No FMA, no
// CUDA intrinsics that might differ between architectures, no warp shuffles
// for accumulation. The kernels are launched with a fixed geometry (one
// thread per entity) so reductions are serial and ordering-stable.
//
// Rounding rule: multiplication widens to int64_t, then applies
// round-half-to-even at bit 15 before shifting right by 16 and saturating
// back to int32_t. The implementation is identical to
// `fixed.rs::round_half_even_i64_shift`.

#pragma once

#include <cstdint>

#ifndef DSFB_GPU_FIXED_POINT
// Be explicit: this file relies on the contract pinning fixed-point arithmetic.
// build.rs passes -DDSFB_GPU_FIXED_POINT; if it is missing we want the build
// to fail loudly rather than silently disagree with the CPU side.
#error "DSFB_GPU_FIXED_POINT must be defined for the deterministic numeric contract"
#endif

namespace dsfb {

// Saturate an int64_t down to int32_t with the same rule as the Rust
// `saturate_i64_to_i32`.
__device__ __host__ __forceinline__ int32_t saturate_i64_to_i32(int64_t x) {
    if (x > (int64_t)INT32_MAX) return INT32_MAX;
    if (x < (int64_t)INT32_MIN) return INT32_MIN;
    return (int32_t)x;
}

// Right shift with round-half-to-even at the cut. Direct transliteration of
// `fixed.rs::round_half_even_i64_shift`.
__device__ __host__ __forceinline__ int64_t round_half_even_i64_shift(int64_t value, int bits) {
    if (bits == 0) return value;
    int64_t truncated = value >> bits;
    int64_t halfway_bit = (int64_t)1 << (bits - 1);
    int64_t discarded = value & (((int64_t)1 << bits) - 1);
    int64_t remainder_below_half = discarded & (halfway_bit - 1);
    bool at_half = (discarded == halfway_bit) && (remainder_below_half == 0);
    if (at_half) {
        return (truncated & 1) ? (truncated + 1) : truncated;
    }
    if (discarded > halfway_bit) {
        return truncated + 1;
    }
    return truncated;
}

// Saturating Q16.16 addition.
__device__ __host__ __forceinline__ int32_t q16_sat_add(int32_t a, int32_t b) {
    int64_t r = (int64_t)a + (int64_t)b;
    return saturate_i64_to_i32(r);
}

// Saturating Q16.16 subtraction.
__device__ __host__ __forceinline__ int32_t q16_sat_sub(int32_t a, int32_t b) {
    int64_t r = (int64_t)a - (int64_t)b;
    return saturate_i64_to_i32(r);
}

// Saturating Q16.16 multiplication with banker's rounding. Load-bearing for
// CPU↔GPU byte-equality; this is the only place the GPU touches the Q16
// numeric domain.
__device__ __host__ __forceinline__ int32_t q16_sat_mul(int32_t a, int32_t b) {
    int64_t prod = (int64_t)a * (int64_t)b;
    int64_t rounded = round_half_even_i64_shift(prod, 16);
    return saturate_i64_to_i32(rounded);
}

// Saturating Q16.16 absolute value. Note the special case for INT32_MIN to
// match Rust's `i32::saturating_abs`.
__device__ __host__ __forceinline__ int32_t q16_abs(int32_t a) {
    if (a == INT32_MIN) return INT32_MAX;
    return a < 0 ? -a : a;
}

// Saturating Q16.16 linear interpolation: `a + alpha * (b - a)`. Used by
// the EWMA recurrence in the drift_slew_sign kernel.
__device__ __host__ __forceinline__ int32_t q16_lerp(int32_t a, int32_t b, int32_t alpha) {
    int32_t diff = q16_sat_sub(b, a);
    int32_t step = q16_sat_mul(alpha, diff);
    return q16_sat_add(a, step);
}

}  // namespace dsfb
