//! C ABI to the CUDA kernels (compiled only under `--features cuda`).
//!
//! All `unsafe` in the crate is confined here. Each `extern "C"` symbol is a host-side wrapper
//! defined in `kernels.cu` that returns a `cudaError_t` (0 = success). The safe Rust wrappers in
//! this module validate buffer sizes before the call and convert error codes to `Result`.
//!
//! # ABI layout expectations
//!
//! `dsfb_chem_cuda_evidence` writes two output buffers:
//! - `h_digests`: `32 * n_lanes` bytes — lane 0's SHA-256 digest at offset 0, lane 1 at offset 32, etc.
//! - `h_summary`: `5 * n_lanes` i64s — for lane `l`, indices `[l*5 .. l*5+5]` are
//!   `[n_exceedances, oob_count, peak_exceedance, peak_abs_drift, peak_abs_slew]`.
//!
//! `dsfb_chem_cuda_roofline` is a pure memory-bandwidth kernel (no evidence arithmetic); it exists
//! only to establish the DRAM bandwidth ceiling for comparison with the evidence-kernel throughput.

#![cfg(feature = "cuda")]

use core::ffi::{c_char, c_int};

extern "C" {
    /// GPU evidence factory: runs `evidence_kernel` over `n_lanes * n_samples` doubles and fills
    /// `h_digests` (lane SHA-256 digests) and `h_summary` (integer summary stats). `elapsed_ms`
    /// receives the kernel-only time measured with CUDA events (host-device transfer excluded).
    fn dsfb_chem_cuda_evidence(
        h_x: *const f64,
        n_lanes: u32,
        n_samples: u32,
        h_digests: *mut u8,
        h_summary: *mut i64,
        elapsed_ms: *mut f32,
    ) -> c_int;

    /// V2-A batched evidence factory: runs `evidence_kernel_batched` over many datasets' lanes
    /// concatenated into one launch. `h_xcat` holds all lanes' samples lane-contiguous; lane `L`'s
    /// samples are `h_xcat[offsets[L] .. offsets[L]+nsamples[L]]`. Outputs use the same per-lane
    /// layout as `dsfb_chem_cuda_evidence`. `elapsed_ms` is the kernel-only time.
    fn dsfb_chem_cuda_evidence_batched(
        h_xcat: *const f64,
        total_elems: u64,
        h_offsets: *const u64,
        h_nsamples: *const u32,
        n_total_lanes: u32,
        h_digests: *mut u8,
        h_summary: *mut i64,
        elapsed_ms: *mut f32,
    ) -> c_int;

    /// V2-B segment-parallel evidence factory: one thread per (lane, segment). Writes a per-segment
    /// 32-byte digest and 5 partial-summary i64s; the host combines per lane. Segment `g` covers
    /// `xcat[seg_base[g] + seg_start[g] .. seg_base[g] + seg_end[g]]`, replaying `seg_warmup[g]`
    /// pre-segment samples to seed the drift ring.
    fn dsfb_chem_cuda_evidence_v2_segmented(
        h_xcat: *const f64,
        total_elems: u64,
        h_seg_base: *const u64,
        h_seg_start: *const u32,
        h_seg_end: *const u32,
        h_seg_warmup: *const u32,
        n_segments: u32,
        h_seg_digests: *mut u8,
        h_seg_summary: *mut i64,
        elapsed_ms: *mut f32,
    ) -> c_int;

    /// Memory roofline: stream-reads `n` doubles `iters` times; returns kernel-only elapsed time
    /// and a checksum over all partial sums (to prevent the compiler from eliminating the loads).
    fn dsfb_chem_cuda_roofline(
        h_x: *const f64,
        n: u64,
        iters: c_int,
        h_checksum: *mut f64,
        elapsed_ms: *mut f32,
    ) -> c_int;

    /// Copies the active CUDA device name into `buf` (null-terminated, at most `buflen-1` chars).
    fn dsfb_chem_cuda_device_name(buf: *mut c_char, buflen: c_int) -> c_int;
}

/// Raw GPU evidence output: per-lane 32-byte digests and 5 summary i64s (exc, oob, pke, pkd, pks).
///
/// Decoded by `dispatch::try_cuda` into a `Vec<LaneEvidence>` for comparison with the CPU reference.
pub struct GpuEvidence {
    /// Flat byte array: lane `l`'s 32-byte SHA-256 digest at `digests[l*32 .. l*32+32]`.
    pub digests: Vec<u8>,
    /// Flat i64 array: lane `l`'s summary at `summary[l*5 .. l*5+5]`
    /// = `[n_exceedances, oob_count, peak_exceedance, peak_abs_drift, peak_abs_slew]`.
    pub summary: Vec<i64>,
    /// Kernel-only elapsed time in milliseconds, as reported by CUDA events.
    pub elapsed_ms: f32,
}

/// Run the evidence factory on the GPU. Returns [`GpuEvidence`] or the raw `cudaError_t` on failure.
///
/// `x` must be in sample-major layout: element `(sample i, lane L)` is at `x[i * n_lanes + L]`.
/// This layout gives the kernel coalesced 128-byte reads because adjacent threads access adjacent
/// lanes within the same sample row.
///
/// # Panics
///
/// Panics if `x.len() != n_lanes * n_samples` (pre-flight size check before the `unsafe` call).
pub fn run_evidence(x: &[f64], n_lanes: u32, n_samples: u32) -> Result<GpuEvidence, i32> {
    assert_eq!(
        x.len(),
        (n_lanes as usize) * (n_samples as usize),
        "x must be n_lanes*n_samples"
    );
    // Allocate host-side output buffers; the C function writes into them via pointer.
    let mut digests = vec![0u8; (n_lanes as usize) * 32];
    let mut summary = vec![0i64; (n_lanes as usize) * 5];
    let mut elapsed_ms: f32 = 0.0;
    let rc = unsafe {
        dsfb_chem_cuda_evidence(
            x.as_ptr(),
            n_lanes,
            n_samples,
            digests.as_mut_ptr(),
            summary.as_mut_ptr(),
            &mut elapsed_ms as *mut f32,
        )
    };
    if rc != 0 {
        // rc is a cudaError_t; non-zero means the kernel failed or a CUDA call failed.
        return Err(rc);
    }
    Ok(GpuEvidence {
        digests,
        summary,
        elapsed_ms,
    })
}

/// Run the V2-A batched evidence factory: one launch over many datasets' lanes.
///
/// `xcat` holds every lane's samples lane-contiguous; `offsets[l]` is lane `l`'s start index into
/// `xcat` and `nsamples[l]` its sample count. Returns per-lane [`GpuEvidence`] in the same layout as
/// [`run_evidence`], so the digests are directly comparable to a per-dataset V1 run.
///
/// # Panics
///
/// Panics if `offsets.len() != nsamples.len()`, or if any lane's `offset + nsamples` exceeds
/// `xcat.len()` (a pre-flight bounds check before the `unsafe` call, so an out-of-range descriptor
/// can never read past the buffer on the device).
pub fn run_evidence_batched(
    xcat: &[f64],
    offsets: &[u64],
    nsamples: &[u32],
) -> Result<GpuEvidence, i32> {
    let n = offsets.len();
    assert_eq!(
        n,
        nsamples.len(),
        "offsets and nsamples must describe the same lanes"
    );
    for l in 0..n {
        let end = offsets[l] as usize + nsamples[l] as usize;
        assert!(
            end <= xcat.len(),
            "lane {l} descriptor [{}..{end}] exceeds xcat ({})",
            offsets[l],
            xcat.len()
        );
    }
    let mut digests = vec![0u8; n * 32];
    let mut summary = vec![0i64; n * 5];
    let mut elapsed_ms: f32 = 0.0;
    let rc = unsafe {
        dsfb_chem_cuda_evidence_batched(
            xcat.as_ptr(),
            xcat.len() as u64,
            offsets.as_ptr(),
            nsamples.as_ptr(),
            n as u32,
            digests.as_mut_ptr(),
            summary.as_mut_ptr(),
            &mut elapsed_ms as *mut f32,
        )
    };
    if rc != 0 {
        return Err(rc);
    }
    Ok(GpuEvidence {
        digests,
        summary,
        elapsed_ms,
    })
}

/// Raw per-segment output of the V2-B segment-parallel kernel: `n_segments` 32-byte digests +
/// `5 * n_segments` partial-summary i64s, in segment-descriptor order. The caller combines these
/// per lane into the `evidence_root_v2` lane digests.
pub struct GpuSegmentEvidence {
    pub seg_digests: Vec<u8>,
    pub seg_summary: Vec<i64>,
    pub elapsed_ms: f32,
}

/// Run the V2-B segment-parallel kernel. The four descriptor slices must all have length
/// `n_segments`; for segment `g`, samples `xcat[seg_base[g]+seg_start[g] .. seg_base[g]+seg_end[g]]`
/// are emitted and `seg_warmup[g]` preceding samples are replayed to seed the drift ring.
///
/// # Panics
/// Panics if the descriptor slices disagree in length, or if any segment's `seg_base + seg_end`
/// exceeds `xcat.len()` or its warm-up would read before `xcat[0]` (pre-flight bounds checks).
pub fn run_evidence_v2_segmented(
    xcat: &[f64],
    seg_base: &[u64],
    seg_start: &[u32],
    seg_end: &[u32],
    seg_warmup: &[u32],
) -> Result<GpuSegmentEvidence, i32> {
    let n = seg_base.len();
    assert!(
        seg_start.len() == n && seg_end.len() == n && seg_warmup.len() == n,
        "segment descriptor slices must all be length n_segments"
    );
    for g in 0..n {
        let base = seg_base[g] as usize;
        let end = base + seg_end[g] as usize;
        assert!(
            end <= xcat.len(),
            "segment {g} end {end} exceeds xcat {}",
            xcat.len()
        );
        assert!(
            seg_start[g] >= seg_warmup[g],
            "segment {g} warm-up underflows lane start"
        );
    }
    let mut seg_digests = vec![0u8; n * 32];
    let mut seg_summary = vec![0i64; n * 5];
    let mut elapsed_ms: f32 = 0.0;
    let rc = unsafe {
        dsfb_chem_cuda_evidence_v2_segmented(
            xcat.as_ptr(),
            xcat.len() as u64,
            seg_base.as_ptr(),
            seg_start.as_ptr(),
            seg_end.as_ptr(),
            seg_warmup.as_ptr(),
            n as u32,
            seg_digests.as_mut_ptr(),
            seg_summary.as_mut_ptr(),
            &mut elapsed_ms as *mut f32,
        )
    };
    if rc != 0 {
        return Err(rc);
    }
    Ok(GpuSegmentEvidence {
        seg_digests,
        seg_summary,
        elapsed_ms,
    })
}

/// Run the memory-roofline kernel `iters` streaming passes over `x`.
///
/// Returns `(checksum, elapsed_ms)` where `checksum` is the sum of all elements across all passes
/// (used to prevent dead-code elimination) and `elapsed_ms` is the kernel-only time. The bandwidth
/// in GB/s is `(x.len() * 8 * iters) / (elapsed_ms * 1e6)`.
pub fn run_roofline(x: &[f64], iters: i32) -> Result<(f64, f32), i32> {
    let mut checksum = 0.0f64;
    let mut elapsed_ms = 0.0f32;
    let rc = unsafe {
        dsfb_chem_cuda_roofline(
            x.as_ptr(),
            x.len() as u64,
            iters,
            &mut checksum as *mut f64,
            &mut elapsed_ms as *mut f32,
        )
    };
    if rc != 0 {
        return Err(rc);
    }
    Ok((checksum, elapsed_ms))
}

/// Query the active CUDA device name (e.g. "NVIDIA GeForce RTX 4080 SUPER").
///
/// Returns a best-effort string; on failure returns a diagnostic containing the error code rather
/// than panicking, so the attestation can still be sealed with a device identifier.
pub fn device_name() -> String {
    let mut buf = vec![0i8; 256];
    let rc =
        unsafe { dsfb_chem_cuda_device_name(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int) };
    if rc != 0 {
        return format!("unknown-cuda-device(rc={rc})");
    }
    // Convert i8 slice to UTF-8, stopping at the first null terminator.
    let bytes: Vec<u8> = buf
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}
