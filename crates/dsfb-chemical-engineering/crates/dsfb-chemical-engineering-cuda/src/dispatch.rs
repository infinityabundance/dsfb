//! Backend dispatch: build residual lanes from a dataset, run the evidence factory on the CPU
//! reference (always available) or the CUDA device (when built with `--features cuda` and a device
//! is present), and cross-verify that the GPU evidence root matches the CPU reference byte-for-byte.
//!
//! # Data flow
//!
//! ```text
//! DataMatrix + n_base
//!     └─► build_lanes()       — compute residuals, hash the raw f64 layout
//!             └─► Lanes { data (sample-major), n_lanes, n_samples, input_sha256 }
//!                     └─► run()             — choose best available backend
//!                             ├─ try_cuda() — GPU path (feature = "cuda" and device available)
//!                             │       └─ cross-verify GPU == CPU; keeps CPU lanes on mismatch
//!                             └─ evidence_cpu() — CPU reference path (always available)
//!                                     └─► EvidenceRun { backend, lanes, elapsed_ns, ... }
//! ```
//!
//! # Correctness contract
//!
//! `cross_backend_verified` in `EvidenceRun` is `true` only when the GPU produced output that
//! compared equal (via `PartialEq` on `LaneEvidence`) to the CPU reference for every lane. On any
//! mismatch the court keeps the CPU reference lanes; on a mismatch the evidence root will be the
//! CPU root, not the GPU root. The CPU path sets `cross_backend_verified = true` unconditionally
//! because the CPU reference *is* the definition of correctness.

use crate::evidence::{lane_evidence_cpu, LaneEvidence};
use dsfb_chemical_engineering_edge::data::DataMatrix;
use dsfb_chemical_engineering_edge::Baseline;
use sha2::{Digest, Sha256};

/// The backend that produced a given `EvidenceRun`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// CPU reference path (`lane_evidence_cpu`). Always available; defines correctness.
    Cpu,
    /// CUDA GPU path (`evidence_kernel`). Available only when compiled with `--features cuda`
    /// and a CUDA-capable device is accessible at runtime.
    Cuda,
}

impl Backend {
    /// A stable string label used in the execution attestation and CLI output.
    pub fn label(&self) -> &'static str {
        match self {
            Backend::Cpu => "cpu-reference",
            Backend::Cuda => "cuda",
        }
    }
}

/// Residual lanes in sample-major layout: element `(sample i, lane L)` is at `data[i*n_lanes + L]`.
///
/// Sample-major layout means consecutive bytes in memory are consecutive lanes at the same time
/// step, which gives the CUDA `evidence_kernel` coalesced DRAM reads (128-byte cache lines) when
/// consecutive threads access consecutive lanes in the same warp.
///
/// Lanes are per-variable raw residuals `x_v(t) − baseline_mean_v` — the residual stream **plus**
/// the usually-discarded sub-threshold noise, which the evidence factory seals into the record.
pub struct Lanes {
    /// Flat f64 buffer, sample-major: `data[i * n_lanes + l]` is sample `i` of lane `l`.
    pub data: Vec<f64>,
    /// Number of lanes (== number of process variables in the data matrix).
    pub n_lanes: u32,
    /// Number of time steps (samples) per lane.
    pub n_samples: u32,
    /// SHA-256 of the raw f64 residual matrix (in sample-major LE layout), hex-encoded.
    /// Sealed into the `Passport` so the case file records exactly which bytes were analysed.
    pub input_sha256: String,
}

/// Build residual lanes from a `DataMatrix` by subtracting per-variable baseline means.
///
/// The first `n_base` samples are used by `Baseline::fit` to estimate the per-variable mean.
/// All `n` samples (including the baseline period) are then expressed as residuals and packed into
/// the sample-major `data` buffer. The SHA-256 of the residual matrix is computed over the raw
/// IEEE-754 bit representations of the doubles (matching the CPU and GPU hash inputs).
pub fn build_lanes(m: &DataMatrix, n_base: usize) -> Lanes {
    let baseline = Baseline::fit(m, n_base);
    let v = m.n_vars;
    let n = m.n_samples;
    let mut data = vec![0.0f64; v * n];
    for i in 0..n {
        let row = m.row(i);
        for (j, &mean) in baseline.mean.iter().enumerate() {
            // Residual = observed - baseline mean. Includes the noise floor below any control limit.
            data[i * v + j] = row[j] - mean;
        }
    }
    // Input digest over the raw f64 bits (sample-major LE), sealing the exact bytes analysed.
    // `to_bits()` gives the IEEE-754 bit pattern as u64; `to_le_bytes()` makes it endian-stable.
    let mut h = Sha256::new();
    for &x in &data {
        h.update(x.to_bits().to_le_bytes());
    }
    let input_sha256 = h.finalize().iter().map(|b| format!("{:02x}", b)).collect();
    Lanes {
        data,
        n_lanes: v as u32,
        n_samples: n as u32,
        input_sha256,
    }
}

/// CPU reference: extract each lane's contiguous sample sequence and compute its evidence.
///
/// The `data` buffer is sample-major, so lane `l`'s samples are at strides of `n_lanes`.
/// This function de-interleaves the buffer into a per-lane `Vec<f64>` for [`lane_evidence_cpu`].
/// The CPU path is also used as the cross-verification reference after every GPU run.
pub fn evidence_cpu(lanes: &Lanes) -> Vec<LaneEvidence> {
    let nl = lanes.n_lanes as usize;
    let ns = lanes.n_samples as usize;
    (0..nl)
        .map(|l| {
            // Collect the strided sample sequence for lane `l` into a contiguous slice.
            let seq: Vec<f64> = (0..ns).map(|i| lanes.data[i * nl + l]).collect();
            lane_evidence_cpu(l as u32, &seq)
        })
        .collect()
}

/// Result of an evidence run, including which backend produced it and (for GPU) the cross-check.
///
/// This struct is handed to [`crate::court::CaseFile::assemble`] and then to `attest`. The
/// `lanes` field always contains the final evidence that the court will seal; on a GPU/CPU
/// mismatch (`cross_backend_verified = false`), the court falls back to the CPU reference lanes.
pub struct EvidenceRun {
    /// Which backend produced the `lanes` evidence.
    pub backend: Backend,
    /// Human-readable device identifier (CUDA device name or "cpu-reference").
    pub device: String,
    /// Kernel name (e.g. "evidence_kernel" or "lane_evidence_cpu").
    pub kernel: String,
    /// Per-lane evidence that will be sealed into the case file.
    pub lanes: Vec<LaneEvidence>,
    /// Kernel-only elapsed time in nanoseconds.
    pub elapsed_ns: u64,
    /// Number of bytes in the residual matrix (used for throughput calculation).
    pub bytes_processed: u64,
    /// `true` if this run's sealed evidence is self-consistent: either the CPU reference path ran
    /// (correct by definition) **or** a GPU run produced output that matched the CPU reference.
    /// This is the flag the court uses to decide a run is trustworthy. It is `true` on the CPU path
    /// even though no GPU was involved — see `gpu_cross_verified` to distinguish the two cases.
    pub cross_backend_verified: bool,
    /// `true` **only** when an actual GPU run was executed *and* cross-checked equal against the CPU
    /// reference. `false` on the pure CPU-reference path (no second backend existed to compare) and
    /// `false` when a GPU run mismatched (the court then falls back to CPU lanes). This separates
    /// "self-consistent" (`cross_backend_verified`) from "two independent backends agreed"
    /// (`gpu_cross_verified`), so a CPU-only run is not misread as cross-backend evidence.
    pub gpu_cross_verified: bool,
}

/// Run the evidence factory using the best available backend.
///
/// Tries the CUDA path first (if the `cuda` feature is active and a device is present). Falls
/// through to the CPU reference path if CUDA is unavailable or `try_cuda` returns `None`
/// (e.g. due to a CUDA error or missing device).
pub fn run(lanes: &Lanes) -> EvidenceRun {
    let bytes = (lanes.data.len() * std::mem::size_of::<f64>()) as u64;
    #[cfg(feature = "cuda")]
    {
        // Attempt the GPU path. Returns `None` on any CUDA error; the CPU path is always the fallback.
        if let Some(run) = try_cuda(lanes, bytes) {
            return run;
        }
    }
    // CPU reference path: always deterministic and correct by definition.
    let t0 = std::time::Instant::now();
    let lev = evidence_cpu(lanes);
    let elapsed_ns = t0.elapsed().as_nanos() as u64;
    EvidenceRun {
        backend: Backend::Cpu,
        device: "cpu-reference".into(),
        kernel: "lane_evidence_cpu".into(),
        lanes: lev,
        elapsed_ns,
        bytes_processed: bytes,
        cross_backend_verified: true, // CPU is the reference; it is always "verified against itself"
        gpu_cross_verified: false,    // no GPU ran on this path, so nothing was cross-checked
    }
}

/// Attempt to run the evidence factory on the GPU.
///
/// Returns `None` if the CUDA call fails (any non-zero `cudaError_t`), allowing the caller to
/// fall back to the CPU path without propagating an error.
///
/// After the GPU run, the CPU reference is computed and the two evidence sets are compared with
/// `==` (which uses `LaneEvidence`'s derived `PartialEq`, comparing every integer field and the
/// hex digest string). If any lane differs, `cross_backend_verified` is `false` and the court
/// keeps the CPU reference lanes — the GPU result is not sealed into the case file.
#[cfg(feature = "cuda")]
fn try_cuda(lanes: &Lanes, bytes: u64) -> Option<EvidenceRun> {
    use crate::ffi;
    // Launch the evidence kernel. Returns Err(cudaError_t) on failure.
    let gpu = ffi::run_evidence(&lanes.data, lanes.n_lanes, lanes.n_samples).ok()?;
    // Decode raw GPU output into LaneEvidence structs for comparison.
    let mut lev = Vec::with_capacity(lanes.n_lanes as usize);
    for l in 0..lanes.n_lanes as usize {
        // Digest: 32 bytes at offset l*32 in the flat digests buffer.
        let dig = &gpu.digests[l * 32..l * 32 + 32];
        // Summary: 5 i64s at offset l*5 in the flat summary buffer.
        // Layout: [n_exceedances, oob_count, peak_exceedance, peak_abs_drift, peak_abs_slew].
        let s = &gpu.summary[l * 5..l * 5 + 5];
        lev.push(LaneEvidence {
            lane_id: l as u32,
            n_samples: lanes.n_samples,
            // The kernel writes these counts as `long long`. They are bounded by `n_samples` (a u32),
            // so they always fit u32 — but rather than a silent `as u32` truncation, convert checked:
            // a value that does not fit means a corrupt GPU result, so we bail out of the GPU path
            // (`ok()?` → `None`) and `run()` falls back to the authoritative CPU reference.
            n_exceedances: u32::try_from(s[0]).ok()?,
            oob_count: u32::try_from(s[1]).ok()?,
            peak_exceedance: s[2],
            peak_abs_drift: s[3],
            peak_abs_slew: s[4],
            // Format each byte as two lowercase hex digits, producing a 64-char string.
            digest_hex: dig.iter().map(|b| format!("{:02x}", b)).collect(),
        });
    }
    // Cross-verify: run the CPU reference and compare every lane's evidence byte-for-byte.
    // This is the court's primary correctness guarantee.
    let cpu = evidence_cpu(lanes);
    let verified = cpu == lev;
    // The attestation must describe the run that produced the *sealed* evidence. On a match the GPU
    // evidence is sealed, so we report the device + the kernel-only GPU time (ms → ns). On a MISMATCH the
    // court seals the CPU reference lanes instead, so the attestation reports the CPU backend (not a GPU
    // throughput for evidence the GPU did not produce); `elapsed_ns = 0` signals that no GPU timing
    // applies to the sealed CPU evidence. `cross_backend_verified` stays `false` on mismatch, so the run
    // is still flagged (a GPU that disagrees with the reference is a real fault to surface).
    let (backend, device, kernel, elapsed_ns, sealed) = if verified {
        (
            Backend::Cuda,
            ffi::device_name(),
            "evidence_kernel".to_string(),
            (gpu.elapsed_ms as f64 * 1.0e6) as u64,
            lev,
        )
    } else {
        (
            Backend::Cpu,
            "cpu-reference (gpu cross-check mismatch fallback)".to_string(),
            "lane_evidence_cpu".to_string(),
            0u64,
            cpu,
        )
    };
    Some(EvidenceRun {
        backend,
        device,
        kernel,
        lanes: sealed,
        elapsed_ns,
        bytes_processed: bytes,
        cross_backend_verified: verified,
        // A GPU run happened here, so `gpu_cross_verified` carries the real two-backend result:
        // `true` iff the GPU output matched the CPU reference, `false` on any mismatch.
        gpu_cross_verified: verified,
    })
}
