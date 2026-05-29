//! The fixed-point evidence contract — the single source of truth that both the CPU reference and
//! the CUDA kernel must reproduce **byte-for-byte**.
//!
//! Residual values (and the usually-discarded sub-threshold noise) are quantised to fixed point so
//! the evidence is independent of floating-point associativity. From each lane (one residual
//! stream) the factory derives, per sample, the quantised residual `q`, the one-sided exceedance
//! `e = max(0,q)`, a causal windowed drift `d`, and the slew `s`. A SHA-256 lane digest is taken
//! over a canonical byte stream that **includes the raw IEEE-754 bits of the residual** — so even
//! the noise floor that conventional monitoring discards is sealed into the forensic record.
//!
//! Determinism contract (must hold on CPU and GPU):
//!   * quantisation is a single IEEE-754 double multiply + round-half-away-from-zero (no FMA);
//!   * drift/slew/exceedance are exact integer arithmetic;
//!   * the per-sample hashed record is `raw_bits(le u64) ‖ q ‖ e ‖ d ‖ s` (all little-endian),
//!     40 bytes/sample, fed to SHA-256 in sample order.
//!
//! The CUDA build is compiled with `--fmad=false`; Rust performs no implicit FMA here.
//!
//! # Per-sample record layout (40 bytes, little-endian throughout)
//!
//! ```text
//! offset  size  field
//! 0       8     raw IEEE-754 bits of xv (reinterpreted as u64, not converted)
//! 8       8     q  — quantised residual (i64)
//! 16      8     e  — one-sided exceedance max(0, q) (i64)
//! 24      8     d  — causal windowed drift: ring_sum / filled (i64, integer division)
//! 32      8     s  — slew q[i] - q[i-1], 0 at i=0 (i64)
//! ```
//!
//! # Why include raw IEEE-754 bits?
//!
//! Including the raw float bits (not just `q`) means the digest changes even when two different
//! residual values round to the same fixed-point integer. This seals sub-quantisation information
//! — including noise that conventional SPC/monitoring would silently discard — into the record.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Fixed-point scale factor (6 decimal places).
///
/// Every residual `x` is multiplied by this value before rounding to i64. A scale of 1e6 gives
/// one micro-unit of resolution. The same constant is defined as `DSFB_SCALE` in `common.cuh`.
pub const SCALE: f64 = 1_000_000.0;

/// Number of samples in the causal drift ring buffer.
///
/// The drift `d[i]` is the integer mean of the last `DRIFT_WINDOW` quantised residuals (or fewer
/// at the start of the stream). Using a fixed power-of-2 window allows the CUDA kernel to use
/// `%DSFB_DRIFT_WINDOW` modulo without a branch. Must match `DSFB_DRIFT_WINDOW` in `common.cuh`.
pub const DRIFT_WINDOW: usize = 16;

/// Human-readable contract identifier, sealed into the passport via [`crate::court::Passport`].
///
/// Any change to the evidence contract (scale, window size, record layout, or hash algorithm)
/// must be reflected in a new contract string so that stored case files can be unambiguously
/// identified as belonging to the old or new contract.
pub const CONTRACT_ID: &str =
    "dsfb-chem-cuda/evidence-contract/v1:scale=1e6,window=16,record=raw|q|e|d|s";

/// Deterministic round-half-away-from-zero of `x*SCALE` to i64 (matches the CUDA device code).
///
/// Returns `(q, finite)` where `q` is the quantised integer and `finite` is false iff `x` is
/// NaN or infinite. Non-finite inputs yield `q = 0` so the evidence arithmetic remains defined,
/// but the sample is counted separately via `oob_count`.
///
/// # Determinism note
///
/// This is intentionally two separate IEEE-754 double operations (`*` then `+`), not a fused
/// multiply-add. The CUDA kernel is compiled with `--fmad=false` for the same reason: both
/// paths produce the same correctly-rounded result for `x * 1_000_000.0`.
#[inline]
pub fn quantize(x: f64) -> (i64, bool) {
    if !x.is_finite() {
        return (0, false); // out-of-band: contributes 0 to integer evidence, flagged via `finite`
    }
    let scaled = x * SCALE;
    // Round half away from zero: add 0.5 before truncating toward zero.
    // The conditional on the sign of `scaled` reproduces the same logic as `dsfb_quantize` in
    // common.cuh, where the negative branch negates to work with a positive value.
    let q = if scaled >= 0.0 {
        (scaled + 0.5) as i64
    } else {
        -((-scaled + 0.5) as i64)
    };
    (q, true)
}

/// Per-lane derived evidence summary produced by the evidence factory (CPU or GPU path).
///
/// This is the court-facing record: all fields are integers or a hex digest — no floating-point —
/// so comparison between the CPU reference and the GPU output is exact (`PartialEq`/`Eq`).
///
/// The GPU path fills this struct by decoding the raw `GpuEvidence` output in `dispatch::try_cuda`;
/// the CPU path fills it directly in `lane_evidence_cpu`. Both paths must produce identical values
/// for `cross_backend_verified` to be `true` in the `EvidenceRun`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneEvidence {
    /// Zero-based lane index (== variable index in the data matrix).
    pub lane_id: u32,
    /// Number of samples processed in this lane.
    pub n_samples: u32,
    /// Number of samples where the quantised residual `q > 0` (one-sided exceedance count).
    pub n_exceedances: u32,
    /// Number of non-finite (NaN or ±inf) samples; these are sealed as `q=0` rather than dropped.
    pub oob_count: u32,
    /// Maximum `e = max(0, q)` seen across all samples (in fixed-point units, scale = 1e6).
    pub peak_exceedance: i64,
    /// Maximum `|d|` seen across all samples (integer drift in fixed-point units).
    pub peak_abs_drift: i64,
    /// Maximum `|s|` seen across all samples (integer slew in fixed-point units).
    pub peak_abs_slew: i64,
    /// SHA-256 of the canonical 40-bytes/sample record, hex-encoded (64 chars, lowercase).
    pub digest_hex: String,
}

/// Compute the lane evidence + digest for one residual stream — the **CPU reference implementation**.
///
/// This function is the authoritative definition of the evidence contract. The CUDA `evidence_kernel`
/// in `kernels.cu` must produce byte-for-byte identical results for the same input `x`.
///
/// # Arguments
///
/// * `lane_id` — zero-based lane index, stored in the returned struct and used to key the case file.
/// * `x` — a contiguous slice of residual values for this lane, in time order (sample 0 first).
///
/// # Algorithm (sequential over samples — causal, no look-ahead)
///
/// 1. Quantise `xv` to `q` via [`quantize`] (round-half-away-from-zero, no FMA).
/// 2. Maintain a causal ring buffer of the last `DRIFT_WINDOW` `q` values; `d = ring_sum / filled`
///    (integer truncation toward zero). During the first `DRIFT_WINDOW` samples, `filled` grows
///    from 1, so the denominator is always the actual number of samples in the window.
/// 3. `e = max(0, q)` — one-sided: only positive deviations count as exceedances.
/// 4. `s = q[i] - q[i-1]`, with `s = 0` at `i = 0` (no prior sample).
/// 5. Feed the canonical 40-byte record `(raw_bits ‖ q ‖ e ‖ d ‖ s)` to SHA-256 in sample order.
pub fn lane_evidence_cpu(lane_id: u32, x: &[f64]) -> LaneEvidence {
    let n = x.len();
    let mut hasher = Sha256::new();
    // Ring buffer state for causal windowed drift.
    let mut ring = [0i64; DRIFT_WINDOW];
    let mut ring_sum: i64 = 0;
    let mut filled = 0usize; // number of slots currently occupied (< DRIFT_WINDOW at stream start)
    let mut head = 0usize; // index of the oldest slot, updated circularly
    let mut prev_q: i64 = 0; // previous sample's q, for slew computation
    let mut n_exc = 0u32;
    let mut oob = 0u32;
    let (mut pk_e, mut pk_d, mut pk_s) = (0i64, 0i64, 0i64);

    for (i, &xv) in x.iter().enumerate() {
        let (q, finite) = quantize(xv);
        if !finite {
            // Non-finite value: sealed as q=0, counted separately so the digest still changes.
            oob += 1;
        }
        // Causal windowed drift via integer ring buffer (exact, no floating point).
        // Once the buffer is full, evict the oldest value before inserting the new one.
        if filled == DRIFT_WINDOW {
            ring_sum -= ring[head]; // evict oldest
        } else {
            filled += 1; // still filling: grow the active window
        }
        ring[head] = q;
        ring_sum += q;
        head = (head + 1) % DRIFT_WINDOW; // advance head to the next slot (oldest after next eviction)
        let d = ring_sum / filled as i64; // integer division (truncation toward zero, as in the GPU)
        let e = if q > 0 { q } else { 0 };
        let s = if i == 0 { 0 } else { q - prev_q };
        prev_q = q;

        if e > 0 {
            n_exc += 1;
        }
        if e > pk_e {
            pk_e = e;
        }
        if d.abs() > pk_d {
            pk_d = d.abs();
        }
        if s.abs() > pk_s {
            pk_s = s.abs();
        }

        // Canonical per-sample record: raw IEEE-754 bits ‖ q ‖ e ‖ d ‖ s (all LE, 40 bytes total).
        // `xv.to_bits()` reinterprets the double as u64 without conversion — bit-exact.
        // This matches `__double_as_longlong(xv)` in the CUDA kernel (same bit pattern).
        hasher.update(xv.to_bits().to_le_bytes());
        hasher.update(q.to_le_bytes());
        hasher.update(e.to_le_bytes());
        hasher.update(d.to_le_bytes());
        hasher.update(s.to_le_bytes());
    }
    let digest = hasher.finalize();
    LaneEvidence {
        lane_id,
        n_samples: n as u32,
        n_exceedances: n_exc,
        oob_count: oob,
        peak_exceedance: pk_e,
        peak_abs_drift: pk_d,
        peak_abs_slew: pk_s,
        // Lowercase hex, 64 chars. The GPU path produces the same string via byte-by-byte formatting.
        digest_hex: digest.iter().map(|b| format!("{:02x}", b)).collect(),
    }
}

/// V2-B segment size: samples per independently-hashed segment in the Merkle-segment format.
///
/// Hashing fixed-size segments independently shortens the per-thread SHA dependency chain from
/// `n_samples` to `SEGMENT_SIZE`, enabling intra-lane GPU parallelism (see
/// `docs/cuda_evidence_kernel_v2_design.md`). The size trades two opposing costs: smaller segments =
/// more threads (higher occupancy → throughput) but a larger share of the fixed
/// `DRIFT_WINDOW`(=16)-sample **halo warm-up**, which is pure re-computation (halo fraction ≈ 16/seg).
/// Measured on the deep 1024×8192 case after the SHA micro-opt, kernel time / SM-throughput are
/// 2.56 ms/30.4% (512), **2.40 ms/32.6% (256)**, 2.31 ms/34.2% (128, 12.5% halo), 2.24 ms/35.6%
/// (64, 25% halo). **256 is the knee** — most of the speedup at a modest 6.25% halo, without the
/// segment-count / host-combine blow-up below it. The size is part of the V2 contract id.
pub const SEGMENT_SIZE: usize = 256;

/// V2-B contract identifier (distinct from [`CONTRACT_ID`] — V2-B is a separate evidence format).
pub const CONTRACT_ID_V2: &str =
    "dsfb-chem-cuda/evidence-contract/v2:scale=1e6,window=16,record=raw|q|e|d|s,merkle-seg=256";

/// CPU reference for the **V2-B Merkle-segment** lane digest — the authoritative format definition.
///
/// **NON-equivalent to V1 by design.** The per-sample records (`raw|q|e|d|s`) and every summary field
/// are computed *identically* to [`lane_evidence_cpu`] — the drift ring buffer still spans the whole
/// stream causally, so the records are bit-for-bit the V1 records. What changes is the *sealing*: the
/// record stream is partitioned into `seg`-sample segments, each hashed independently into a 32-byte
/// segment digest, and the lane digest is SHA-256 over the concatenated segment digests (a one-level
/// Merkle root). This shortens the per-thread SHA dependency chain from `n_samples` to `seg`, which is
/// what lets a GPU kernel parallelise the hash *within* a lane.
///
/// Because the lane digest is now a hash-of-hashes, it differs from V1's single-pass digest for the
/// same input (even with one segment, V2-B double-hashes) — so this is a new evidence format
/// (`evidence_root_v2`), opt-in and clearly labelled, never a drop-in replacement for V1.
pub fn lane_evidence_v2_cpu(lane_id: u32, x: &[f64], seg: usize) -> LaneEvidence {
    assert!(seg > 0, "segment size must be positive");
    let n = x.len();
    // Drift/slew state — identical to the V1 reference (records must match V1 byte-for-byte).
    let mut ring = [0i64; DRIFT_WINDOW];
    let mut ring_sum: i64 = 0;
    let mut filled = 0usize;
    let mut head = 0usize;
    let mut prev_q: i64 = 0;
    let mut n_exc = 0u32;
    let mut oob = 0u32;
    let (mut pk_e, mut pk_d, mut pk_s) = (0i64, 0i64, 0i64);
    // Segment digests accumulate here as raw 32-byte blocks, concatenated in segment order.
    let mut seg_digests: Vec<u8> = Vec::new();
    let mut seg_hasher = Sha256::new();
    let mut in_seg = 0usize; // samples fed to the current segment hasher

    for (i, &xv) in x.iter().enumerate() {
        let (q, finite) = quantize(xv);
        if !finite {
            oob += 1;
        }
        if filled == DRIFT_WINDOW {
            ring_sum -= ring[head];
        } else {
            filled += 1;
        }
        ring[head] = q;
        ring_sum += q;
        head = (head + 1) % DRIFT_WINDOW;
        let d = ring_sum / filled as i64;
        let e = if q > 0 { q } else { 0 };
        let s = if i == 0 { 0 } else { q - prev_q };
        prev_q = q;
        if e > 0 {
            n_exc += 1;
        }
        if e > pk_e {
            pk_e = e;
        }
        if d.abs() > pk_d {
            pk_d = d.abs();
        }
        if s.abs() > pk_s {
            pk_s = s.abs();
        }
        // Feed the canonical 40-byte record to the CURRENT segment hasher (identical bytes to V1).
        seg_hasher.update(xv.to_bits().to_le_bytes());
        seg_hasher.update(q.to_le_bytes());
        seg_hasher.update(e.to_le_bytes());
        seg_hasher.update(d.to_le_bytes());
        seg_hasher.update(s.to_le_bytes());
        in_seg += 1;
        if in_seg == seg {
            // Segment full: finalise its digest, append the raw 32 bytes, start a fresh segment.
            let sd = std::mem::replace(&mut seg_hasher, Sha256::new()).finalize();
            seg_digests.extend_from_slice(&sd);
            in_seg = 0;
        }
    }
    // Flush a trailing partial segment (when n is not a multiple of seg).
    if in_seg > 0 {
        let sd = seg_hasher.finalize();
        seg_digests.extend_from_slice(&sd);
    }
    // lane_root_v2 = SHA-256 over the concatenated segment digests (one-level Merkle root).
    let mut root_h = Sha256::new();
    root_h.update(&seg_digests);
    let root = root_h.finalize();
    LaneEvidence {
        lane_id,
        n_samples: n as u32,
        n_exceedances: n_exc,
        oob_count: oob,
        peak_exceedance: pk_e,
        peak_abs_drift: pk_d,
        peak_abs_slew: pk_s,
        digest_hex: root.iter().map(|b| format!("{:02x}", b)).collect(),
    }
}

/// Merkle-style root over lane digests: SHA-256 of the concatenated raw 32-byte digests, in lane order.
///
/// "Merkle-style" here means a single-level hash over all lane digests, not a binary tree. This is
/// sufficient because lanes are ordered and the count is fixed by the passport. The root changes if
/// any lane's digest changes, or if the lane order changes.
///
/// The raw 32-byte digests are hashed (not their hex representations) for compactness and to avoid
/// any ambiguity about hex encoding.
pub fn merkle_root(lanes: &[LaneEvidence]) -> String {
    let mut h = Sha256::new();
    for l in lanes {
        // Hash the raw 32-byte digest, not its hex, for compactness/stability.
        let raw = hex_to_bytes(&l.digest_hex);
        h.update(raw);
    }
    let d = h.finalize();
    d.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Decode a lowercase hex string to raw bytes.
///
/// Invalid hex nibbles silently decode as 0 (should not occur for well-formed digests).
/// Used throughout the court to convert hex-encoded hashes back to bytes for re-hashing.
pub(crate) fn hex_to_bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_round_half_away() {
        assert_eq!(quantize(0.0000005).0, 1); // 0.5 -> 1
        assert_eq!(quantize(-0.0000005).0, -1);
        assert_eq!(quantize(1.0).0, 1_000_000);
        assert_eq!(quantize(f64::NAN), (0, false));
    }

    #[test]
    fn lane_evidence_is_deterministic() {
        let x: Vec<f64> = (0..256).map(|i| (i as f64 * 0.1).sin()).collect();
        let a = lane_evidence_cpu(0, &x);
        let b = lane_evidence_cpu(0, &x);
        assert_eq!(a, b);
        assert_eq!(a.digest_hex.len(), 64);
    }

    #[test]
    fn exceedance_counts_only_positive() {
        let x = vec![-1.0, 2.0, -3.0, 4.0];
        let e = lane_evidence_cpu(0, &x);
        assert_eq!(e.n_exceedances, 2);
        assert_eq!(e.peak_exceedance, 4_000_000);
    }

    #[test]
    fn v2_merkle_segment_is_deterministic_and_summary_matches_v1() {
        let x: Vec<f64> = (0..2000).map(|i| (i as f64 * 0.07).sin() * 3.0).collect();
        let a = lane_evidence_v2_cpu(0, &x, SEGMENT_SIZE);
        let b = lane_evidence_v2_cpu(0, &x, SEGMENT_SIZE);
        assert_eq!(a, b, "V2-B must be deterministic");
        assert_eq!(a.digest_hex.len(), 64);
        // V2-B changes only the *sealing*: every summary field must equal the V1 reference.
        let v1 = lane_evidence_cpu(0, &x);
        assert_eq!(a.n_exceedances, v1.n_exceedances);
        assert_eq!(a.oob_count, v1.oob_count);
        assert_eq!(a.peak_exceedance, v1.peak_exceedance);
        assert_eq!(a.peak_abs_drift, v1.peak_abs_drift);
        assert_eq!(a.peak_abs_slew, v1.peak_abs_slew);
    }

    #[test]
    fn v2_digest_differs_from_v1_even_with_a_single_segment() {
        // V2-B is a hash-of-hashes, so it differs from V1's single-pass digest for the same input —
        // even when one segment covers the whole stream. This is what makes it a distinct format.
        let x: Vec<f64> = (0..300).map(|i| (i as f64 * 0.1).cos()).collect();
        let v1 = lane_evidence_cpu(0, &x);
        let v2_one_seg = lane_evidence_v2_cpu(0, &x, 100_000); // seg >> n: a single segment
        let v2_small = lane_evidence_v2_cpu(0, &x, 64); // many segments
        assert_ne!(
            v2_one_seg.digest_hex, v1.digest_hex,
            "V2-B is not V1 (double-hashed)"
        );
        assert_ne!(
            v2_small.digest_hex, v2_one_seg.digest_hex,
            "segmentation changes the root"
        );
        // Same summary regardless of segment size (sealing-only change).
        assert_eq!(v2_one_seg.peak_exceedance, v1.peak_exceedance);
        assert_eq!(v2_small.n_exceedances, v1.n_exceedances);
    }
}
