// common.cuh — shared constants and the deterministic fixed-point quantiser.
//
// MUST match src/evidence.rs exactly. The kernel TU is compiled with --fmad=false so that
// `x*SCALE + 0.5` is evaluated as two correctly-rounded IEEE-754 double operations, identical to
// the Rust reference (which performs no FMA contraction).
//
// Why --fmad=false?
//   Without this flag, the PTX assembler or the SASS back end may merge `x * SCALE + 0.5` into
//   a single DFMA instruction. DFMA has a single rounding step (result is only rounded once,
//   after the combined multiply-add), whereas two separate instructions each round individually.
//   Rust applies no FMA contraction, so the GPU must also use two rounded steps to agree.
//   With --fmad=false, both paths produce the same bit-identical int64 result for all finite
//   double inputs in the residual range, satisfying the cross-backend determinism contract.
#pragma once
#include <stdint.h>

// Fixed-point scale: 1 unit = 1 micro-unit of the original residual value.
// Must equal evidence::SCALE (1_000_000.0) in src/evidence.rs.
#define DSFB_SCALE      1000000.0

// Causal drift ring-buffer width in samples.
// Must equal evidence::DRIFT_WINDOW (16) in src/evidence.rs.
#define DSFB_DRIFT_WINDOW 16

// Round-half-away-from-zero of x*SCALE to int64.
// Non-finite inputs (NaN, ±inf) return 0 and set *finite = 0; all other inputs set *finite = 1.
// The implementation intentionally uses two separate IEEE-754 double operations (multiply, then
// add 0.5) rather than a fused form. With --fmad=false this is guaranteed at compile time.
__device__ __forceinline__ long long dsfb_quantize(double x, int* finite) {
    if (!isfinite(x)) { *finite = 0; return 0LL; }
    *finite = 1;
    double scaled = x * DSFB_SCALE;           /* step 1: multiply (correctly rounded) */
    if (scaled >= 0.0) return (long long)(scaled + 0.5);   /* step 2: add 0.5, truncate */
    return -(long long)(-scaled + 0.5);        /* negative branch: negate, add 0.5, negate */
}
