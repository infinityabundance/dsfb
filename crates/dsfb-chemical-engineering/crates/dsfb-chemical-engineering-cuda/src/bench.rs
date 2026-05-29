//! Throughput benchmark harness for the evidence factory and the memory roofline.
//!
//! Runs each workload multiple times and reports min/median/max/mean/stddev GB/s. The CUDA path
//! reports kernel-only time via CUDA events (host-device transfer excluded); the CPU reference
//! path measures wall time via `std::time::Instant`. Nsight (`nsys`/`ncu`) wraps the same binary
//! using `scripts/run_nsight.sh` to capture hardware performance counters.
//!
//! # Throughput metric
//!
//! GB/s is computed as `input_bytes / elapsed_ns` (== bytes/ns), where `input_bytes` is the size
//! of the raw f64 residual matrix. For the roofline kernel, the byte count is multiplied by
//! `iters` because the kernel reads the buffer that many times per timed call.
//!
//! NOTE: GB/s numbers reflect the workload as benchmarked, not a guaranteed performance bound.
//! The evidence kernel is SHA-256-compute-bound (each sample requires SHA-256 block updates),
//! not DRAM-bandwidth-bound; the roofline kernel provides the DRAM bandwidth ceiling for context.

use serde::{Deserialize, Serialize};

/// Generate a deterministic synthetic sample-major residual buffer of `n_lanes * n_samples` doubles.
///
/// The buffer is used for throughput benchmarks and profiling runs where the exact values do not
/// matter but the buffer must be non-trivial (to prevent the compiler or hardware from optimising
/// away the computation). The formula mixes:
/// - a slow drift term `0.001 * t + l * 0.01` (varies across time and lanes),
/// - a sine oscillation over that,
/// - a pseudo-random noise term derived from a multiply-step hash `(i * 2654435761 + l) % 97`.
///
/// The result is fully deterministic: the same `(n_lanes, n_samples)` always produces the same
/// bytes. The layout is sample-major: `v[i * n_lanes + l]` is sample `i` of lane `l`.
pub fn synthetic(n_lanes: usize, n_samples: usize) -> Vec<f64> {
    let mut v = vec![0.0f64; n_lanes * n_samples];
    for i in 0..n_samples {
        for l in 0..n_lanes {
            let t = i as f64;
            // sin(slow drift) + bounded pseudo-random noise in [0, 0.3).
            v[i * n_lanes + l] = (0.001 * t + l as f64 * 0.01).sin()
                + 0.3 * ((i * 2654435761 + l) % 97) as f64 / 97.0;
        }
    }
    v
}

/// Summary statistics for one benchmark workload, serialised into the per-run JSON report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchSummary {
    /// Short name identifying the workload ("evidence_factory" or "memory_roofline").
    pub workload: String,
    /// Backend that ran the workload ("cuda" or "cpu-reference").
    pub backend: String,
    /// Number of lanes (process variables) in the timed buffer.
    pub n_lanes: usize,
    /// Number of samples per lane in the timed buffer.
    pub n_samples: usize,
    /// Total bytes in the input buffer (or `input_bytes * iters` for the roofline).
    pub bytes: u64,
    /// Per-run throughput in GB/s (one entry per timed repetition, excluding warm-up).
    pub runs: Vec<f64>,
    /// Minimum GB/s across all runs.
    pub gbps_min: f64,
    /// Median GB/s (lower midpoint for even-length sorted arrays).
    pub gbps_median: f64,
    /// Maximum GB/s across all runs.
    pub gbps_max: f64,
    /// Arithmetic mean GB/s.
    pub gbps_mean: f64,
    /// Population standard deviation of GB/s.
    pub gbps_stddev: f64,
}

/// Compute summary statistics from a vector of per-run GB/s values.
///
/// Uses population variance (divides by `n`, not `n-1`) for the standard deviation.
/// Median is the lower midpoint for even-length arrays (`sorted[n/2]` with 0-based indexing).
fn summarize(
    workload: &str,
    backend: &str,
    n_lanes: usize,
    n_samples: usize,
    bytes: u64,
    gbps: Vec<f64>,
) -> BenchSummary {
    let mut sorted = gbps.clone();
    // Sort ascending to find min, median, max. `unwrap_or(Equal)` keeps the sort total-order-safe even
    // if a timing sample were ever non-finite (it never is here), matching the NaN-safe sorts elsewhere.
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len().max(1); // guard against empty; .max(1) prevents divide-by-zero below
    let mean = gbps.iter().sum::<f64>() / n as f64;
    // Population variance: mean of squared deviations.
    let var = gbps.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n as f64;
    BenchSummary {
        workload: workload.into(),
        backend: backend.into(),
        n_lanes,
        n_samples,
        bytes,
        gbps_min: *sorted.first().unwrap_or(&0.0),
        gbps_median: sorted[n / 2], // lower midpoint for even n
        gbps_max: *sorted.last().unwrap_or(&0.0),
        gbps_mean: mean,
        gbps_stddev: var.sqrt(),
        runs: gbps,
    }
}

/// Benchmark the evidence factory at the given `(n_lanes, n_samples)` size over `runs` repetitions.
///
/// Tries the CUDA path first (one un-timed warm-up run, then `runs` timed runs). Falls back to
/// the CPU reference if CUDA is unavailable or all GPU runs fail. One warm-up run is performed on
/// the CPU path as well to exercise the instruction cache before timing.
///
/// The returned GB/s values are `input_bytes / kernel_elapsed_ns` where `input_bytes = n_lanes *
/// n_samples * 8`. For the CUDA path, elapsed time comes from CUDA events (kernel only).
pub fn bench_evidence(n_lanes: usize, n_samples: usize, runs: usize) -> BenchSummary {
    let x = synthetic(n_lanes, n_samples);
    let bytes = (x.len() * 8) as u64; // 8 bytes per f64
    let mut gbps = Vec::with_capacity(runs);
    #[cfg(feature = "cuda")]
    {
        use crate::ffi;
        // Un-timed warm-up: prime the GPU execution context and caches.
        let _ = ffi::run_evidence(&x, n_lanes as u32, n_samples as u32);
        for _ in 0..runs {
            match ffi::run_evidence(&x, n_lanes as u32, n_samples as u32) {
                Ok(ev) => {
                    let ns = ev.elapsed_ms as f64 * 1.0e6; // ms -> ns
                    gbps.push(if ns > 0.0 { bytes as f64 / ns } else { 0.0 });
                }
                Err(_) => break, // stop collecting on any CUDA error
            }
        }
        // Return the GPU summary if at least one timed run succeeded.
        if !gbps.is_empty() {
            return summarize("evidence_factory", "cuda", n_lanes, n_samples, bytes, gbps);
        }
    }
    // CPU reference timing: wall-clock via Instant, no CUDA events.
    use crate::dispatch::{evidence_cpu, Lanes};
    // `input_sha256` is left empty; it is not used during benchmarking.
    let lanes = Lanes {
        data: x,
        n_lanes: n_lanes as u32,
        n_samples: n_samples as u32,
        input_sha256: String::new(),
    };
    let _ = evidence_cpu(&lanes); // warm-up (discarded)
    for _ in 0..runs {
        let t0 = std::time::Instant::now();
        let _ = evidence_cpu(&lanes);
        let ns = t0.elapsed().as_nanos() as f64;
        gbps.push(if ns > 0.0 { bytes as f64 / ns } else { 0.0 });
    }
    summarize(
        "evidence_factory",
        "cpu-reference",
        n_lanes,
        n_samples,
        bytes,
        gbps,
    )
}

/// Benchmark the memory roofline (CUDA only). `iters` streaming passes per timed run.
///
/// Each timed call launches `roofline_kernel` `iters` times in a row before stopping the CUDA
/// event timer, so the per-call bytes transferred are `n_elems * 8 * iters`. One un-timed warm-up
/// call is issued before the timed runs. Returns `None` if CUDA is unavailable or any run fails.
///
/// The roofline result establishes the achievable DRAM read bandwidth for comparison against the
/// evidence-kernel throughput. The evidence kernel's compute requirements (SHA-256 per sample)
/// mean it will typically read far fewer bytes per unit time than the roofline ceiling.
#[cfg(feature = "cuda")]
pub fn bench_roofline(n_elems: usize, iters: i32, runs: usize) -> Option<BenchSummary> {
    use crate::ffi;
    // Generate a flat 1-lane synthetic buffer (lane dimension irrelevant for roofline).
    let x = synthetic(1, n_elems);
    // Each timed run streams the buffer `iters` times, so account for all passes in the byte count.
    let bytes_per_run = (x.len() * 8) as u64 * iters as u64;
    let _ = ffi::run_roofline(&x, iters); // un-timed warm-up
    let mut gbps = Vec::with_capacity(runs);
    for _ in 0..runs {
        let (_, ms) = ffi::run_roofline(&x, iters).ok()?; // propagate CUDA failure as None
        let ns = ms as f64 * 1.0e6; // ms -> ns
        gbps.push(if ns > 0.0 {
            bytes_per_run as f64 / ns
        } else {
            0.0
        });
    }
    Some(summarize(
        "memory_roofline",
        "cuda",
        1,
        n_elems,
        bytes_per_run,
        gbps,
    ))
}

/// Stub for non-CUDA builds: the roofline benchmark requires a GPU.
#[cfg(not(feature = "cuda"))]
pub fn bench_roofline(_n_elems: usize, _iters: i32, _runs: usize) -> Option<BenchSummary> {
    None
}
