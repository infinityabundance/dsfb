//! Safe Rust wrapper over the CUDA pipeline FFI.
//!
//! Two entry points are exposed:
//!
//! * [`build_gpu`] — the plain end-to-end pipeline call. Suitable for the
//!   CLI's `run-gpu` subcommand and for any caller that just wants a
//!   `CaseFile` back. Internally requests no CUDA-event timing.
//! * [`build_gpu_timed`] — the same call, but it also returns
//!   `PipelineTimingsFfi` describing per-stage microsecond timings.
//!   Used by `dsfb-gpu-debug bench --detail` to surface where each
//!   pipeline run is actually spending time.
//!
//! Both routes share the same Q16.16 kernels, the same locked launch
//! geometry, and the same `cuda/kernels.cu` host wrapper. The only
//! difference is whether the wrapper's `cudaEventRecord` calls fire. The
//! C++ side guards them behind a `want_timings = (timings_out != nullptr)`
//! flag, so the no-timing path has zero event overhead.
//!
//! Hash-chain construction reuses `dsfb_gpu_debug_core::casefile::
//! build_from_artifacts` so the CPU and GPU paths feed the same hashing
//! function with the same canonical bytes — which is exactly what makes
//! per-stage byte-equivalent comparison possible.

#![cfg(feature = "cuda")]

use core::ffi::c_int;
use std::vec;
use std::vec::Vec;

use dsfb_gpu_debug_core::candidate::{CandidateConfig, CandidateInterval};
use dsfb_gpu_debug_core::casefile::{build_from_artifacts_with_mode, CaseFile, EmissionMode};
use dsfb_gpu_debug_core::consensus::ConsensusCell;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::detector::{DetectorCell, DetectorThresholds};
use dsfb_gpu_debug_core::event::{GpuTraceEventCompact, TraceEvent};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::residual::{Baseline, ResidualCell};
use dsfb_gpu_debug_core::sign::SignCell;
use dsfb_gpu_debug_core::window::{compute_features, WindowFeature};

use crate::ffi::{
    dsfb_gpu_run_pipeline, dsfb_gpu_run_pipeline_batched, dsfb_gpu_run_pipeline_on_workspace,
    DetectorThresholdsFfi, PipelineTimingsFfi,
};
use crate::workspace::{
    BatchedGpuWorkspace, GpuWorkspace, GraphCaptureStatus,
    MAX_CANDIDATES_PER_ENTITY as WS_MAX_CANDIDATES_PER_ENTITY,
};
use crate::GpuError;

/// Per-entity capacity for the candidate output buffer.
///
/// At the v0 contract (128 windows per entity) the maximum possible
/// candidate count per entity is `floor(N / min_length_windows) = 128`,
/// but the canonical bank only ever admits a handful, so 16 is a
/// comfortable upper bound that fits in a single warp's worth of
/// register storage on the device. Re-exported from
/// `workspace::MAX_CANDIDATES_PER_ENTITY` so both modules agree on the
/// value without a circular dependency.
const MAX_CANDIDATES_PER_ENTITY: i32 = WS_MAX_CANDIDATES_PER_ENTITY;

/// Per-stage CUDA-event timings (microseconds), re-exported from the FFI
/// module so callers don't have to depend on `ffi` directly.
pub type PipelineTimings = PipelineTimingsFfi;

/// Run the GPU pipeline end-to-end and return the case file. The CUDA
/// path picks up at the residual stage; windowing runs on the CPU per
/// the v0 contract simplification.
///
/// # Errors
///
/// Returns `GpuError::CudaUnavailable` when the crate was built without
/// the `cuda` feature; `GpuError::KernelFailed(code)` for non-zero
/// `cudaError_t` from any stage of the kernel pipeline.
pub fn build_gpu(events: &[TraceEvent], contract: &Contract) -> Result<CaseFile, GpuError> {
    let (case, _) = build_gpu_inner(events, contract, false)?;
    Ok(case)
}

/// Run the GPU pipeline and return both the case file and per-stage
/// CUDA-event timings.
///
/// Equivalent to `build_gpu` except that the host wrapper records
/// `cudaEvent_t` markers around alloc / H2D / each kernel / D2H / free
/// and reports microsecond elapsed times for each. Useful for the
/// `dsfb-gpu-debug bench --detail` mode and for diagnosing where a
/// regression lives.
///
/// # Errors
///
/// Same as [`build_gpu`].
pub fn build_gpu_timed(
    events: &[TraceEvent],
    contract: &Contract,
) -> Result<(CaseFile, PipelineTimings), GpuError> {
    build_gpu_inner(events, contract, true)
}

fn build_gpu_inner(
    events: &[TraceEvent],
    contract: &Contract,
    want_timings: bool,
) -> Result<(CaseFile, PipelineTimings), GpuError> {
    // 1. CPU-side windowing (deferred from the GPU per spec v0). Same
    //    canonical bytes the CPU pipeline would produce; this is what
    //    keeps the `window_feature` hash byte-identical between
    //    backends.
    let features: Vec<WindowFeature> = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );

    // 2. Allocate output buffers sized for the contract grid.
    let total: usize = (contract.n_entities as usize) * (contract.n_windows as usize);
    let mut residuals: Vec<ResidualCell> = vec![ResidualCell::default(); total];
    let mut signs: Vec<SignCell> = vec![SignCell::default(); total];
    let mut detectors: Vec<DetectorCell> = vec![DetectorCell::default(); total];
    let mut consensus: Vec<ConsensusCell> = vec![ConsensusCell::default(); total];
    let candidate_capacity = (contract.n_entities as usize) * (MAX_CANDIDATES_PER_ENTITY as usize);
    let mut candidate_buf: Vec<CandidateInterval> =
        vec![CandidateInterval::default(); candidate_capacity];
    let mut candidate_count: Vec<i32> = vec![0i32; contract.n_entities as usize];

    let thresholds = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_config = CandidateConfig::CANONICAL;

    let mut timings = PipelineTimings::default();
    let timings_ptr: *mut PipelineTimings = if want_timings {
        std::ptr::from_mut::<PipelineTimings>(&mut timings)
    } else {
        std::ptr::null_mut()
    };

    // 3. Call into the FFI. The unsafe block is the only place where the
    //    pipeline transitions across the CUDA boundary, and it is the
    //    only `unsafe` in the entire CUDA crate. The pointers are
    //    derived from Rust Vec storage and remain valid for the
    //    duration of the call; sizes match the kernel's expectations
    //    exactly.
    let status: c_int = call_kernel(
        &features,
        &mut residuals,
        &mut signs,
        &mut detectors,
        &mut consensus,
        &mut candidate_buf,
        &mut candidate_count,
        contract,
        &thresholds,
        baseline,
        &candidate_config,
        timings_ptr,
    );

    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    // 4. Flatten the per-entity candidate slots into a single canonical-
    //    order vector. Walking entities in order, then taking the first
    //    `candidate_count[e]` slots, yields the same `(entity, start)`
    //    ordering the CPU candidate stage produces.
    let mut candidates: Vec<CandidateInterval> = Vec::with_capacity(candidate_capacity);
    for entity_id in 0..(contract.n_entities as usize) {
        let count = candidate_count[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(candidate_buf[base + i]);
        }
    }

    let case = build_from_artifacts_with_mode(
        events,
        contract,
        "cuda",
        EmissionMode::Audit,
        &features,
        &residuals,
        &signs,
        &detectors,
        &consensus,
        &candidates,
    );
    Ok((case, timings))
}

/// Run the GPU pipeline on a pre-allocated [`GpuWorkspace`].
///
/// This is the hot path for repeated invocations: the device buffers and
/// the host Vec storage have already been allocated, so the only
/// per-call overhead is the H2D / kernel launch / D2H sequence. Suitable
/// for the bench harness's measured iterations and for any deployment
/// that processes many fixtures in a row.
///
/// # Errors
///
/// `GpuError::InvalidInput` if the workspace was built for a different
/// fixture shape than the contract describes; otherwise the same error
/// surface as [`build_gpu`].
pub fn build_gpu_on_workspace(
    workspace: &mut GpuWorkspace,
    events: &[TraceEvent],
    contract: &Contract,
) -> Result<CaseFile, GpuError> {
    let (case, _) =
        build_gpu_on_workspace_inner(workspace, events, contract, EmissionMode::Audit, false)?;
    Ok(case)
}

/// Run the GPU pipeline on a workspace and emit the case file in
/// [`EmissionMode::Throughput`]. Identical kernel execution to
/// [`build_gpu_on_workspace`]; the only difference is the host-side
/// per-stage hash byte form, which uses the compact little-endian
/// representation in throughput mode.
///
/// # Errors
///
/// Same as [`build_gpu_on_workspace`].
pub fn build_gpu_throughput_on_workspace(
    workspace: &mut GpuWorkspace,
    events: &[TraceEvent],
    contract: &Contract,
) -> Result<CaseFile, GpuError> {
    let (case, _) =
        build_gpu_on_workspace_inner(workspace, events, contract, EmissionMode::Throughput, false)?;
    Ok(case)
}

/// Same as [`build_gpu_on_workspace`] but also returns per-stage
/// CUDA-event timings. Used by the bench's `--detail` path when a
/// workspace is in play.
///
/// # Errors
///
/// Same as [`build_gpu_on_workspace`].
pub fn build_gpu_timed_on_workspace(
    workspace: &mut GpuWorkspace,
    events: &[TraceEvent],
    contract: &Contract,
) -> Result<(CaseFile, PipelineTimings), GpuError> {
    build_gpu_on_workspace_inner(workspace, events, contract, EmissionMode::Audit, true)
}

fn build_gpu_on_workspace_inner(
    workspace: &mut GpuWorkspace,
    events: &[TraceEvent],
    contract: &Contract,
    mode: EmissionMode,
    want_timings: bool,
) -> Result<(CaseFile, PipelineTimings), GpuError> {
    workspace.assert_compatible(contract)?;

    // CPU-side windowing (deferred from the GPU per v0 contract).
    let features: Vec<WindowFeature> = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );

    let thresholds = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_config = CandidateConfig::CANONICAL;

    let mut timings = PipelineTimings::default();
    let timings_ptr: *mut PipelineTimings = if want_timings {
        std::ptr::from_mut::<PipelineTimings>(&mut timings)
    } else {
        std::ptr::null_mut()
    };

    // Safety: the workspace pointers are alive for the duration of this
    // call because we hold `&mut workspace`. The host buffers come from
    // the workspace itself (also borrowed mutably). The kernel function
    // mirrors the layouts statically verified on the C++ side.
    #[allow(unsafe_code)]
    let status = unsafe {
        dsfb_gpu_run_pipeline_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            features.as_ptr(),
            contract.n_entities as i32,
            contract.n_windows as i32,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref::<DetectorThresholdsFfi>(&thresholds),
            candidate_config.min_detector_count as i32,
            candidate_config.min_residual_q_raw,
            candidate_config.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            workspace.residuals.as_mut_ptr(),
            workspace.signs.as_mut_ptr(),
            workspace.detectors.as_mut_ptr(),
            workspace.consensus.as_mut_ptr(),
            workspace.candidate_buf.as_mut_ptr(),
            workspace.candidate_count.as_mut_ptr(),
            timings_ptr,
        )
    };

    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    // Flatten the per-entity candidate slots into a single canonical-
    // order vector. Walking entities in order, then taking the first
    // `candidate_count[e]` slots, yields the same `(entity, start)`
    // ordering the CPU candidate stage produces.
    let mut candidates: Vec<CandidateInterval> = Vec::with_capacity(workspace.candidate_buf.len());
    for entity_id in 0..(contract.n_entities as usize) {
        let count = workspace.candidate_count[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(workspace.candidate_buf[base + i]);
        }
    }

    let case = build_from_artifacts_with_mode(
        events,
        contract,
        "cuda",
        mode,
        &features,
        &workspace.residuals,
        &workspace.signs,
        &workspace.detectors,
        &workspace.consensus,
        &candidates,
    );
    Ok((case, timings))
}

/// S-PERF.13 — bulk pack of `TraceEvent[]` into
/// `GpuTraceEventCompact[]`. Replaces the prior scalar
/// `for ... zip()` loop in the D64 `_timed` dispatchers'
/// input-staging slot (renamed `features_us` →
/// `host_input_staging_us`; see the
/// `D64ThroughputHostStageTimings` doc + the
/// S-PERF.13-PREFLIGHT receipt at
/// `reports/s_perf_13_preflight_d64_feature_path_audit.txt`).
///
/// **Byte-identical-output contract (panel-locked S-PERF.13
/// N8 + P6)**: every event packed by this function matches
/// the scalar `GpuTraceEventCompact::from_trace_event`
/// output byte-for-byte. Enforced by (a) the AVX2 path's
/// explicit field arithmetic mirroring the const fn's
/// bit-mask + branchless-error-bit logic verbatim, (b) the
/// scalar fallback's direct call into the const fn, and
/// (c) the `s_perf_13_pack_equivalence_tests` module which
/// proves equivalence across n = {0, 1, 7, 8, 9, 64, 257,
/// 4096, 65_537}. Determinism: no atomics, no FP, no
/// order-dependent reduction.
///
/// **Approach A — Phase 2 (AVX2 + non-temporal stores)**:
/// when AVX2 is statically enabled (`cfg(target_feature =
/// "avx2")`) the helper delegates to
/// `pack_events_to_pinned_avx2`, which packs **two events
/// per 256-bit non-temporal store** (`_mm256_stream_si256`).
/// The non-temporal store bypasses L1/L2 so the pinned-
/// shadow writes do not pollute the cache during the bulk
/// pack; `_mm_sfence` retires the streaming stores before
/// the FFI dispatch's H2D copy reads the pinned shadow.
///
/// **Scalar fallback** (`pack_events_to_pinned_scalar`)
/// preserves Phase 1's chunk-and-unroll shape; runs on
/// non-AVX2 builds (any non-x86_64 target, or x86_64
/// builds without `+avx2` in the target feature set) and
/// serves as the byte-identical reference the unit tests
/// compare the AVX2 path against.
///
/// **Memory-bound floor**: at the canonical 256×4096 K=1
/// fixture the per-dispatch traffic is 4.2M × (48 B read +
/// 16 B write) ≈ 267 MB. At DDR4-3200 ~51 GB/s peak the
/// theoretical floor is ~5.2 ms. The pre-S-PERF.13 scalar
/// iterator measured ~6.3 ms (1.2× over floor). The Phase
/// 2 win comes from cache-bypass (non-temporal stores
/// eliminate L1/L2 eviction during the pack) and explicit
/// pointer arithmetic (no iterator-chain overhead).
#[inline(always)]
#[allow(clippy::inline_always)]
fn pack_events_to_pinned_simd(events: &[TraceEvent], pinned: &mut [GpuTraceEventCompact]) {
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        // SAFETY: `pack_events_to_pinned_avx2` reads at most
        // `events.len()` items from `events` and writes at
        // most `min(events.len(), pinned.len())` items to
        // `pinned`; bounds enforced inside the helper.
        // `_mm256_stream_si256` requires 32 B alignment; the
        // helper checks the dst pointer alignment at runtime
        // and falls back to `_mm256_storeu_si256` (unaligned)
        // when not 32 B aligned (e.g. Vec<T> in tests).
        #[allow(unsafe_code)]
        unsafe {
            pack_events_to_pinned_avx2(events, pinned);
        }
    }
    #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
    pack_events_to_pinned_scalar(events, pinned);
}

/// S-PERF.13 scalar fallback. Phase 1 chunk-and-unroll
/// preserved for non-AVX2 hosts and as the byte-identical
/// reference the unit tests compare the AVX2 path against.
/// `#[allow(dead_code)]` because on AVX2 builds the
/// dispatcher routes through `pack_events_to_pinned_avx2`
/// and this fallback is unreachable; the
/// `s_perf_13_pack_equivalence_tests::pack_events_to_pinned_scalar`
/// shadow keeps the byte-identical reference exercised in
/// tests regardless of build target features.
#[inline(always)]
#[allow(dead_code)]
fn pack_events_to_pinned_scalar(events: &[TraceEvent], pinned: &mut [GpuTraceEventCompact]) {
    const CHUNK: usize = 8;
    let n = events.len().min(pinned.len());
    let full = (n / CHUNK) * CHUNK;
    let src_iter = events[..full].chunks_exact(CHUNK);
    let dst_iter = pinned[..full].chunks_exact_mut(CHUNK);
    for (src, dst) in src_iter.zip(dst_iter) {
        dst[0] = GpuTraceEventCompact::from_trace_event(&src[0]);
        dst[1] = GpuTraceEventCompact::from_trace_event(&src[1]);
        dst[2] = GpuTraceEventCompact::from_trace_event(&src[2]);
        dst[3] = GpuTraceEventCompact::from_trace_event(&src[3]);
        dst[4] = GpuTraceEventCompact::from_trace_event(&src[4]);
        dst[5] = GpuTraceEventCompact::from_trace_event(&src[5]);
        dst[6] = GpuTraceEventCompact::from_trace_event(&src[6]);
        dst[7] = GpuTraceEventCompact::from_trace_event(&src[7]);
    }
    for (dst, src) in pinned[full..n].iter_mut().zip(events[full..n].iter()) {
        *dst = GpuTraceEventCompact::from_trace_event(src);
    }
}

/// S-PERF.13 Phase 2 — AVX2 pack with adaptive store kind.
/// Processes events in pairs (2 × 16 B = 32 B = one 256-bit
/// store per pair). Uses `_mm256_stream_si256` (non-temporal,
/// bypasses L1/L2 caches) when the destination is 32-byte
/// aligned; otherwise falls back to `_mm256_storeu_si256`
/// (unaligned, works on any alignment but does not bypass
/// the cache). The production caller is the D64 `_timed`
/// dispatcher whose `events_pinned` buffer is allocated via
/// `cudaMallocHost`'s page-aligned allocator, so the
/// non-temporal store hits its fast path; the unaligned
/// fallback exists for the unit tests (which use
/// `Vec<GpuTraceEventCompact>` whose allocator only
/// guarantees the type's 8-byte alignment).
///
/// `_mm_sfence` retires non-temporal stores if any were
/// issued, before return, so the FFI dispatch that follows
/// reads coherent memory.
///
/// # Safety
///
/// Caller MUST guarantee:
///   - `events.as_ptr().add(i)` is valid for `i < events.len()`
///     (Rust slice invariant);
///   - `pinned.as_mut_ptr().add(i)` is valid for `i <
///     pinned.len()` (Rust slice invariant).
///
/// The pair-stride of 32 bytes is satisfied for any
/// `GpuTraceEventCompact[]` slice because the type's size
/// is 16 bytes (verified by the layout printout in the
/// S-PERF.13-PREFLIGHT receipt).
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[target_feature(enable = "avx2")]
#[inline]
#[allow(unsafe_code)]
unsafe fn pack_events_to_pinned_avx2(events: &[TraceEvent], pinned: &mut [GpuTraceEventCompact]) {
    use core::arch::x86_64::{
        __m256i, _mm256_set_epi32, _mm256_storeu_si256, _mm256_stream_si256, _mm_sfence,
    };

    let n = events.len().min(pinned.len());
    let src_base = events.as_ptr();
    let dst_base = pinned.as_mut_ptr();

    // 32-byte alignment check: streaming stores require it;
    // unaligned stores accept anything. cudaMallocHost gives
    // us page-aligned memory (4 KiB) so the production path
    // hits the fast non-temporal store. Vec<T> in tests gives
    // us 8-byte alignment so the fallback runs there.
    let dst_addr = dst_base as usize;
    let use_stream = (dst_addr % 32) == 0;

    let pairs = n / 2;
    for pair_idx in 0..pairs {
        let i = pair_idx * 2;
        // SAFETY: i+1 < n, both within slice bounds.
        let e0 = src_base.add(i);
        let e1 = src_base.add(i + 1);
        // The cast widens alignment from `GpuTraceEventCompact`
        // (8 B) to `__m256i` (32 B). The 32 B alignment is NOT
        // statically guaranteed; the runtime `use_stream` check
        // below picks the unaligned-store intrinsic when the
        // dst pointer is not 32 B aligned, so the cast itself is
        // sound as long as the stream-store path is gated on the
        // alignment check.
        #[allow(clippy::cast_ptr_alignment)]
        let d_pair = dst_base.add(i).cast::<__m256i>();

        // Event 0 fields (branchless error bit).
        let ts0 = (*e0).ts_ns;
        let entity0 = (*e0).entity_id;
        let lat0 = (*e0).latency_us;
        let err0_bit: u32 = u32::from((*e0).error_code != 0) << 31;
        let ee0: u32 = (entity0 & GpuTraceEventCompact::ENTITY_MASK) | err0_bit;

        // Event 1 fields.
        let ts1 = (*e1).ts_ns;
        let entity1 = (*e1).entity_id;
        let lat1 = (*e1).latency_us;
        let err1_bit: u32 = u32::from((*e1).error_code != 0) << 31;
        let ee1: u32 = (entity1 & GpuTraceEventCompact::ENTITY_MASK) | err1_bit;

        // Pack into a 256-bit register. `_mm256_set_epi32`
        // takes 8 i32s in HIGH-to-LOW order. Layout
        // (low → high bytes within the 32-byte store):
        //   [ts0_lo, ts0_hi, ee0, lat0, ts1_lo, ts1_hi, ee1, lat1]
        // Each event occupies 16 bytes (matches
        // GpuTraceEventCompact's repr(C) layout).
        #[allow(clippy::cast_possible_wrap)]
        let v: __m256i = _mm256_set_epi32(
            lat1 as i32,
            ee1 as i32,
            (ts1 >> 32) as i32,
            (ts1 & 0xFFFF_FFFF) as i32,
            lat0 as i32,
            ee0 as i32,
            (ts0 >> 32) as i32,
            (ts0 & 0xFFFF_FFFF) as i32,
        );
        if use_stream {
            _mm256_stream_si256(d_pair, v);
        } else {
            _mm256_storeu_si256(d_pair, v);
        }
    }

    // Tail: handle last event if n is odd. Falls through to
    // the const fn.
    if n & 1 != 0 {
        let i = n - 1;
        let src_ref = &*src_base.add(i);
        *dst_base.add(i) = GpuTraceEventCompact::from_trace_event(src_ref);
    }

    // Retire non-temporal stores so subsequent reads (the
    // FFI dispatch's H2D copy) see coherent memory. Cheap
    // when no non-temporal stores were issued.
    if use_stream {
        _mm_sfence();
    }
}

#[allow(clippy::too_many_arguments)]
fn call_kernel(
    features: &[WindowFeature],
    residuals: &mut [ResidualCell],
    signs: &mut [SignCell],
    detectors: &mut [DetectorCell],
    consensus: &mut [ConsensusCell],
    candidate_buf: &mut [CandidateInterval],
    candidate_count: &mut [i32],
    contract: &Contract,
    thresholds: &DetectorThresholdsFfi,
    baseline: Baseline,
    candidate_config: &CandidateConfig,
    timings_out: *mut PipelineTimingsFfi,
) -> c_int {
    // Why this `unsafe`: this is the FFI boundary. We're calling a foreign
    // function whose signature is mirrored verbatim in
    // `cuda/kernels.cu::dsfb_gpu_run_pipeline`. All host pointers come
    // from Rust slices we own; the slices outlive the call. Struct
    // layouts are statically verified on the C++ side via
    // `static_assert`. `timings_out` is nullable; the C++ side checks
    // for null before recording events.
    #[allow(unsafe_code)]
    unsafe {
        dsfb_gpu_run_pipeline(
            features.as_ptr(),
            contract.n_entities as i32,
            contract.n_windows as i32,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref::<DetectorThresholdsFfi>(thresholds),
            candidate_config.min_detector_count as i32,
            candidate_config.min_residual_q_raw,
            candidate_config.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            residuals.as_mut_ptr(),
            signs.as_mut_ptr(),
            detectors.as_mut_ptr(),
            consensus.as_mut_ptr(),
            candidate_buf.as_mut_ptr(),
            candidate_count.as_mut_ptr(),
            timings_out,
        )
    }
}

/// Run the deterministic-inference pipeline on `n_catalogs`
/// independent fixtures in a single batched kernel launch (O.16,
/// Tier 2). Returns one `CaseFile` per catalog, in catalog-major
/// order.
///
/// All catalogs share the same `contract` (and therefore the same
/// canonical bank, detector registry, kernel sequence, and so on).
/// Per-catalog output bytes depend only on per-catalog input bytes;
/// corrupting one catalog's events only changes its own per-stage
/// hashes and case file.
///
/// The pipeline emits case files in [`EmissionMode::Throughput`]
/// because batching is exclusively a performance lever; the audit
/// path remains the single-catalog `build_gpu_on_workspace` route.
///
/// # Errors
///
/// `GpuError::InvalidInput` if `catalogs.len() != workspace.n_catalogs`
/// or any catalog's event-count does not match the contract grid.
/// `GpuError::KernelFailed(code)` for non-zero `cudaError_t` from the
/// kernel pipeline.
pub fn build_gpu_batched_throughput(
    workspace: &mut BatchedGpuWorkspace,
    catalogs: &[&[TraceEvent]],
    contract: &Contract,
) -> Result<Vec<CaseFile>, GpuError> {
    // 1. Shape checks.
    if catalogs.len() != workspace.n_catalogs as usize {
        return Err(GpuError::InvalidInput(
            "BatchedGpuWorkspace.n_catalogs does not match catalogs.len()",
        ));
    }
    if contract.n_entities != workspace.n_entities || contract.n_windows != workspace.n_windows {
        return Err(GpuError::InvalidInput(
            "BatchedGpuWorkspace dimensions do not match the supplied contract",
        ));
    }

    let n_catalogs = workspace.n_catalogs as usize;
    let per_catalog = workspace.per_catalog_cells();

    // 2. CPU-side windowing for each catalog. Writes feature cells into
    //    the workspace's catalog-major host buffer at offset
    //    `catalog_id * per_catalog`.
    let window_size_ns = u64::from(contract.window_size_ms) * 1_000_000;
    for (c_idx, &events) in catalogs.iter().enumerate() {
        let features_for_catalog = compute_features(
            events,
            contract.n_windows,
            contract.n_entities,
            window_size_ns,
        );
        let dst = &mut workspace.features[c_idx * per_catalog..(c_idx + 1) * per_catalog];
        dst.copy_from_slice(&features_for_catalog);
    }

    let thresholds = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_config = CandidateConfig::CANONICAL;

    // 3. Single batched FFI call. The unsafe block is the only place
    //    where we cross the CUDA boundary in the batched path.
    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        dsfb_gpu_run_pipeline_batched(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.features.as_ptr(),
            workspace.n_catalogs as i32,
            workspace.n_entities as i32,
            workspace.n_windows as i32,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref::<DetectorThresholdsFfi>(&thresholds),
            candidate_config.min_detector_count as i32,
            candidate_config.min_residual_q_raw,
            candidate_config.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            workspace.residuals.as_mut_ptr(),
            workspace.signs.as_mut_ptr(),
            workspace.detectors.as_mut_ptr(),
            workspace.consensus.as_mut_ptr(),
            workspace.candidate_buf.as_mut_ptr(),
            workspace.candidate_count.as_mut_ptr(),
            std::ptr::null_mut(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    // 4. Per-catalog case-file assembly. The pipeline stages all live
    //    in catalog-major buffers; for each catalog we slice into its
    //    contiguous window and feed `build_from_artifacts_with_mode`
    //    exactly as the single-catalog path does.
    let per_candidate_slot = workspace.per_catalog_candidate_slots();
    let mut cases: Vec<CaseFile> = Vec::with_capacity(n_catalogs);
    for c_idx in 0..n_catalogs {
        let cell_range = c_idx * per_catalog..(c_idx + 1) * per_catalog;
        let count_range =
            c_idx * (workspace.n_entities as usize)..(c_idx + 1) * (workspace.n_entities as usize);
        let cand_range = c_idx * per_candidate_slot..(c_idx + 1) * per_candidate_slot;

        // Flatten the per-entity candidate slots for this catalog.
        let mut candidates: Vec<CandidateInterval> = Vec::new();
        let counts = &workspace.candidate_count[count_range];
        let slots = &workspace.candidate_buf[cand_range];
        for entity_id in 0..(workspace.n_entities as usize) {
            let count = counts[entity_id] as usize;
            let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
            for i in 0..count {
                candidates.push(slots[base + i]);
            }
        }

        let case = build_from_artifacts_with_mode(
            catalogs[c_idx],
            contract,
            "cuda",
            EmissionMode::Throughput,
            &workspace.features[cell_range.clone()],
            &workspace.residuals[cell_range.clone()],
            &workspace.signs[cell_range.clone()],
            &workspace.detectors[cell_range.clone()],
            &workspace.consensus[cell_range],
            &candidates,
        );
        cases.push(case);
    }
    Ok(cases)
}

/// Hash `bytes` with the `__device__` SHA-256 in `cuda/sha256.cuh`.
///
/// Used by `tests/device_sha256_self_test.rs` to verify byte-equality
/// with `dsfb_gpu_debug_core::hash::sha256` over three known-vector
/// inputs. The Tier 3B digest kernels use the same `__device__`
/// routine internally; if this self-test fails, the Tier 3B path is
/// not safe to enable. Single thread on device, so the wall time
/// is dominated by the one-off `cudaMalloc` / `cudaMemcpy` for the
/// input bytes.
///
/// # Errors
///
/// Returns `GpuError::KernelFailed(code)` for any non-zero
/// `cudaError_t` from the FFI.
pub fn sha256_device(bytes: &[u8]) -> Result<[u8; 32], GpuError> {
    let mut out = [0u8; 32];
    #[allow(unsafe_code)]
    let status = unsafe {
        crate::ffi::dsfb_gpu_sha256_self_test(bytes.as_ptr(), bytes.len() as u64, out.as_mut_ptr())
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }
    Ok(out)
}

/// Hash `bytes` with the streaming `__device__` SHA-256
/// (init/update/finalize) in `cuda/sha256.cuh`.
///
/// S-PERF.14b.1 Step 1 byte-stream equivalence helper. The
/// Rust acceptance test
/// `tests/s_perf_14b_1_streaming_sha_self_test.rs` cross-
/// validates `sha256_device_streaming(bytes)` against
/// `sha256_device(bytes)` (one-shot) over three known-vector
/// inputs (empty / 55 B / 64 KB) BEFORE the streaming helpers
/// are allowed to be consumed by the rewritten
/// `compact_densor_digest_v1_root_kernel_blockcoop`. If
/// streaming ≠ one-shot for any of the three vectors, the
/// streaming primitives are not safe to use in production —
/// the rewrite is blocked until the divergence is fixed.
///
/// # Errors
///
/// Returns `GpuError::KernelFailed(code)` for any non-zero
/// `cudaError_t` from the FFI.
pub fn sha256_device_streaming(bytes: &[u8]) -> Result<[u8; 32], GpuError> {
    let mut out = [0u8; 32];
    #[allow(unsafe_code)]
    let status = unsafe {
        crate::ffi::dsfb_gpu_sha256_streaming_self_test(
            bytes.as_ptr(),
            bytes.len() as u64,
            out.as_mut_ptr(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }
    Ok(out)
}

/// S-PERF.14b.1 v3 tile-size sweep wrapper: measures the mean
/// per-iteration wall (in nanoseconds) of the inactive Path 1b
/// v2 streaming root kernel at the given `tile_bytes`. The
/// kernel is launched `n_warmup` warm-up iterations + `n_timed`
/// timed iterations with cudaEvent_t bracketing; the mean per
/// timed iteration is returned.
///
/// Per the panel-locked backend-selection discipline (2026-05-18
/// post-v2-seal correction), Path 1b v2 is INACTIVE in
/// production until a tile size beats Path 1a's pinned 925.2 µs
/// per-call wall. This wrapper drives the sweep harness that
/// reports whether such a tile size exists.
///
/// # Errors
///
/// Returns `GpuError::KernelFailed(code)` for any non-zero
/// `cudaError_t` from the FFI (including
/// `cudaErrorInvalidValue` for zero-sized inputs).
pub fn compact_densor_root_streaming_sweep_time(
    n_chunks_per_catalog: u32,
    chunk_size: u32,
    stage_id: u32,
    n_catalogs: u32,
    tile_bytes: u32,
    n_warmup: i32,
    n_timed: i32,
) -> Result<u64, GpuError> {
    let mut mean_ns: u64 = 0;
    #[allow(unsafe_code)]
    let status = unsafe {
        crate::ffi::dsfb_gpu_compact_densor_root_streaming_sweep_time(
            n_chunks_per_catalog,
            chunk_size,
            stage_id,
            n_catalogs,
            tile_bytes,
            n_warmup,
            n_timed,
            &mut mean_ns,
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }
    Ok(mean_ns)
}

/// S-PERF.14b.1 v3 Path 1a baseline wrapper. Measures the
/// active production backend
/// (`compact_densor_digest_v1_root_kernel_blockcoop`) with
/// the same fixture-shape parameters the streaming sweep
/// uses, so the v3 sweep harness can compare per-stage walls
/// apples-to-apples.
///
/// # Errors
///
/// Returns `GpuError::KernelFailed(code)` for any non-zero
/// `cudaError_t` from the FFI.
pub fn compact_densor_root_path1a_sweep_time(
    n_chunks_per_catalog: u32,
    chunk_size: u32,
    stage_id: u32,
    n_catalogs: u32,
    n_warmup: i32,
    n_timed: i32,
) -> Result<u64, GpuError> {
    let mut mean_ns: u64 = 0;
    #[allow(unsafe_code)]
    let status = unsafe {
        crate::ffi::dsfb_gpu_compact_densor_root_path1a_sweep_time(
            n_chunks_per_catalog,
            chunk_size,
            stage_id,
            n_catalogs,
            n_warmup,
            n_timed,
            &mut mean_ns,
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }
    Ok(mean_ns)
}

/// Tier 3B (O.17) single-catalog dispatch: runs the pipeline on a
/// pre-existing workspace, then runs the four `__device__` SHA-256
/// digest kernels (residual / sign / detector / consensus) on the
/// device. The residual / sign / detector cell buffers are *not*
/// copied back to the host; their per-stage hash contributions to
/// the chain come from the precomputed device digests.
///
/// Determinism: the device digests are byte-equal to the host's
/// `hash_*_compact` digests by construction (pinned by
/// `device_sha256_self_test` and the cross-mode equivalence test).
/// The bank stage receives the consensus grid (copied back to host
/// because the axis-5 entity-locality gate requires it) and the
/// candidate intervals.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if `workspace` dimensions disagree
///   with `contract`.
/// * `GpuError::KernelFailed(code)` for any non-zero `cudaError_t`.
pub fn build_gpu_throughput_device_digests_on_workspace(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
) -> Result<CaseFile, GpuError> {
    workspace.assert_compatible(contract)?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let mut stage_digests_host = [0u8; 4 * 32];

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_digests_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            features.as_ptr(),
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            // R.3b: consensus D2H stripped from the Tier 3B path. After
            // R.5 the bank's axis-5 evidence (entity_avg_q, grid_avg_q)
            // lives inside each CandidateInterval, so the consensus
            // grid is no longer needed host-side. Passing null tells
            // the C++ wrapper to skip the cudaMemcpy entirely.
            std::ptr::null_mut(),
            workspace.candidate_buf.as_mut_ptr(),
            workspace.candidate_count.as_mut_ptr(),
            stage_digests_host.as_mut_ptr(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    // Flatten per-entity candidate slots to a contiguous Vec the bank
    // can iterate over.
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = workspace.candidate_count[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(workspace.candidate_buf[base + i]);
        }
    }

    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    Ok(
        dsfb_gpu_debug_core::casefile::build_throughput_from_artifacts_and_device_digests(
            events,
            contract,
            "cuda",
            &features,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            // R.3b: bank no longer reads consensus after R.5 (axis-5
            // moved into CandidateInterval). Pass an empty slice so
            // any future regression that re-introduces consensus
            // iteration in the bank surfaces as an out-of-bounds
            // panic in the test sweep rather than silently passing.
            &[],
            &candidates,
        ),
    )
}

/// Tier 3B (O.17) batched dispatch: runs K catalogs through the
/// pipeline + parallel device digest kernels. K SHA-256 streams run
/// concurrently as K independent blocks on the device. Per-catalog
/// stage digests are copied back; residual / sign / detector cells
/// stay on the device for each catalog.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if `catalogs.len() != workspace.n_catalogs`
///   or workspace dimensions disagree with `contract`.
/// * `GpuError::KernelFailed(code)` for any non-zero `cudaError_t`.
#[allow(clippy::too_many_lines)]
pub fn build_gpu_batched_throughput_device_digests(
    workspace: &mut BatchedGpuWorkspace,
    catalogs: &[&[TraceEvent]],
    contract: &Contract,
) -> Result<Vec<CaseFile>, GpuError> {
    if catalogs.len() != workspace.n_catalogs as usize {
        return Err(GpuError::InvalidInput(
            "Catalog slice length does not match BatchedGpuWorkspace n_catalogs",
        ));
    }
    if contract.n_entities != workspace.n_entities || contract.n_windows != workspace.n_windows {
        return Err(GpuError::InvalidInput(
            "BatchedGpuWorkspace dimensions do not match the supplied contract",
        ));
    }

    let n_catalogs = catalogs.len();
    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let per_catalog = workspace.per_catalog_cells();
    let per_candidate_slot = workspace.per_catalog_candidate_slots();

    // Compose all catalogs' WindowFeature arrays into the workspace's
    // catalog-major host features buffer.
    for (c_idx, events) in catalogs.iter().enumerate() {
        let features = compute_features(
            events,
            contract.n_windows,
            contract.n_entities,
            u64::from(contract.window_size_ms) * 1_000_000,
        );
        let dst = &mut workspace.features[c_idx * per_catalog..(c_idx + 1) * per_catalog];
        dst.copy_from_slice(&features);
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;
    let mut stage_digests_host = vec![0u8; 4 * 32 * n_catalogs];

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_batched_throughput_digests(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            workspace.features.as_ptr(),
            i32::try_from(n_catalogs).unwrap_or(i32::MAX),
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            // R.3b: consensus D2H stripped from the batched Tier 3B
            // path. After R.5 the bank's axis-5 evidence rides inside
            // each CandidateInterval; passing null tells the C++
            // wrapper to skip the per-catalog consensus cudaMemcpy.
            std::ptr::null_mut(),
            workspace.candidate_buf.as_mut_ptr(),
            workspace.candidate_count.as_mut_ptr(),
            stage_digests_host.as_mut_ptr(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    // Per-catalog case-file assembly. Stage digests are stage-major:
    // [K residual][K sign][K detector][K consensus]. Slice them per
    // catalog and feed each to the Tier 3B casefile entry.
    let mut cases: Vec<CaseFile> = Vec::with_capacity(n_catalogs);
    for c_idx in 0..n_catalogs {
        let cell_range = c_idx * per_catalog..(c_idx + 1) * per_catalog;
        let count_range =
            c_idx * (workspace.n_entities as usize)..(c_idx + 1) * (workspace.n_entities as usize);
        let cand_range = c_idx * per_candidate_slot..(c_idx + 1) * per_candidate_slot;

        let mut candidates: Vec<CandidateInterval> = Vec::new();
        let counts = &workspace.candidate_count[count_range];
        let slots = &workspace.candidate_buf[cand_range];
        for entity_id in 0..(workspace.n_entities as usize) {
            let count = counts[entity_id] as usize;
            let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
            for i in 0..count {
                candidates.push(slots[base + i]);
            }
        }

        let mut residual_digest = [0u8; 32];
        let mut sign_digest = [0u8; 32];
        let mut detector_digest = [0u8; 32];
        let mut consensus_digest = [0u8; 32];
        // Stage-major digest layout: [K residual][K sign][K detector][K consensus].
        // Each stage block is `32 * n_catalogs` bytes; within a block we step by
        // `c_idx * 32` to reach this catalog's slot.
        let stage_block = 32 * n_catalogs;
        let r_base = c_idx * 32;
        let s_base = stage_block + c_idx * 32;
        let d_base = 2 * stage_block + c_idx * 32;
        let cn_base = 3 * stage_block + c_idx * 32;
        residual_digest.copy_from_slice(&stage_digests_host[r_base..r_base + 32]);
        sign_digest.copy_from_slice(&stage_digests_host[s_base..s_base + 32]);
        detector_digest.copy_from_slice(&stage_digests_host[d_base..d_base + 32]);
        consensus_digest.copy_from_slice(&stage_digests_host[cn_base..cn_base + 32]);

        let _ = cell_range; // R.3b: consensus slice no longer indexed.
        let case =
            dsfb_gpu_debug_core::casefile::build_throughput_from_artifacts_and_device_digests(
                catalogs[c_idx],
                contract,
                "cuda",
                &workspace.features[c_idx * per_catalog..(c_idx + 1) * per_catalog],
                residual_digest,
                sign_digest,
                detector_digest,
                consensus_digest,
                // R.3b: bank no longer reads consensus after R.5.
                &[],
                &candidates,
            );
        cases.push(case);
    }
    Ok(cases)
}

/// R.3a — Layer A (device evidence fabric) single-catalog dispatch.
///
/// What's different from `build_gpu_throughput_device_digests_on_workspace`:
/// returns a `CompactCaseSummary` (no admitted episodes, no
/// `final_case_file_hash`) instead of a `CaseFile`. Layer A's whole
/// reason for existing is to measure the evidence-fabric cost in
/// isolation — without the host bank stage. The Semantic Non-Bypass
/// Axiom holds by type: a caller of Layer A literally cannot obtain
/// admitted episodes from this function.
///
/// R.3a uses the same Tier 3B FFI under the hood; the difference is
/// purely host-side (skip the bank stage; emit a summary instead of a
/// case file). R.5 will refine the FFI to also stop copying the
/// consensus grid back when the bank stage is not needed. Until then,
/// the consensus D2H still happens — Layer A pays for it but does not
/// use the bytes.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if `workspace` dimensions disagree
///   with `contract`.
/// * `GpuError::KernelFailed(code)` for any non-zero `cudaError_t`.
pub fn build_gpu_layer_a_on_workspace(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
) -> Result<dsfb_gpu_debug_core::casefile::CompactCaseSummary, GpuError> {
    workspace.assert_compatible(contract)?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let mut stage_digests_host = [0u8; 4 * 32];

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_digests_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            features.as_ptr(),
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            // R.3b: consensus D2H stripped from Layer A. After R.5 the
            // bank's axis-5 evidence rides inside each CandidateInterval
            // (entity_avg_q, grid_avg_q), so the consensus grid is no
            // longer needed host-side. Null tells the C++ wrapper to
            // skip the cudaMemcpy entirely — this is the largest single
            // D2H win Layer A pays for at scaled fixtures.
            std::ptr::null_mut(),
            workspace.candidate_buf.as_mut_ptr(),
            workspace.candidate_count.as_mut_ptr(),
            stage_digests_host.as_mut_ptr(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = workspace.candidate_count[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(workspace.candidate_buf[base + i]);
        }
    }

    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    Ok(
        dsfb_gpu_debug_core::casefile::build_compact_summary_from_device_digests(
            events,
            contract,
            "cuda",
            &features,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            &candidates,
        ),
    )
}

/// R.3a — Layer A batched dispatch. Runs K independent catalogs
/// through the Tier 3B device-digests FFI and returns K
/// `CompactCaseSummary` records (one per catalog). No bank stage runs
/// for any catalog; this is the pure evidence-fabric throughput path.
///
/// Same host-side restructuring as the single-catalog Layer A: the
/// FFI is the existing batched Tier 3B path, but instead of folding
/// the resulting digests into K full case files, we fold them into K
/// compact summaries. The Semantic Non-Bypass Axiom holds at the type
/// level — the function cannot emit admitted episodes.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if `catalogs.len() != workspace.n_catalogs`
///   or workspace dimensions disagree with `contract`.
/// * `GpuError::KernelFailed(code)` for any non-zero `cudaError_t`.
#[allow(clippy::too_many_lines)]
pub fn build_gpu_layer_a_batched(
    workspace: &mut BatchedGpuWorkspace,
    catalogs: &[&[TraceEvent]],
    contract: &Contract,
) -> Result<Vec<dsfb_gpu_debug_core::casefile::CompactCaseSummary>, GpuError> {
    if catalogs.len() != workspace.n_catalogs as usize {
        return Err(GpuError::InvalidInput(
            "Catalog slice length does not match BatchedGpuWorkspace n_catalogs",
        ));
    }
    if contract.n_entities != workspace.n_entities || contract.n_windows != workspace.n_windows {
        return Err(GpuError::InvalidInput(
            "BatchedGpuWorkspace dimensions do not match the supplied contract",
        ));
    }

    let n_catalogs = catalogs.len();
    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let per_catalog = workspace.per_catalog_cells();
    let per_candidate_slot = workspace.per_catalog_candidate_slots();

    // Compose all catalogs' WindowFeature arrays into the workspace's
    // catalog-major host features buffer. Same pattern as the Tier 3B
    // batched case-file path.
    for (c_idx, events) in catalogs.iter().enumerate() {
        let features = compute_features(
            events,
            contract.n_windows,
            contract.n_entities,
            u64::from(contract.window_size_ms) * 1_000_000,
        );
        let dst = &mut workspace.features[c_idx * per_catalog..(c_idx + 1) * per_catalog];
        dst.copy_from_slice(&features);
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;
    let mut stage_digests_host = vec![0u8; 4 * 32 * n_catalogs];

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_batched_throughput_digests(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            workspace.features.as_ptr(),
            i32::try_from(n_catalogs).unwrap_or(i32::MAX),
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            // R.3b: consensus D2H stripped from batched Layer A. Same
            // rationale as the single-catalog Layer A path above —
            // bank's axis-5 evidence rides in CandidateInterval after
            // R.5, so the per-catalog consensus cudaMemcpy is skipped.
            // At K=64 large fixture this drops 2 MB * 64 = 128 MB per
            // batched dispatch from the D2H bill.
            std::ptr::null_mut(),
            workspace.candidate_buf.as_mut_ptr(),
            workspace.candidate_count.as_mut_ptr(),
            stage_digests_host.as_mut_ptr(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let mut summaries: Vec<dsfb_gpu_debug_core::casefile::CompactCaseSummary> =
        Vec::with_capacity(n_catalogs);
    let stage_block = 32 * n_catalogs;
    for c_idx in 0..n_catalogs {
        let cell_range = c_idx * per_catalog..(c_idx + 1) * per_catalog;
        let count_range =
            c_idx * (workspace.n_entities as usize)..(c_idx + 1) * (workspace.n_entities as usize);
        let cand_range = c_idx * per_candidate_slot..(c_idx + 1) * per_candidate_slot;

        let mut candidates: Vec<CandidateInterval> = Vec::new();
        let counts = &workspace.candidate_count[count_range];
        let slots = &workspace.candidate_buf[cand_range];
        for entity_id in 0..(workspace.n_entities as usize) {
            let count = counts[entity_id] as usize;
            let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
            for i in 0..count {
                candidates.push(slots[base + i]);
            }
        }

        let mut residual_digest = [0u8; 32];
        let mut sign_digest = [0u8; 32];
        let mut detector_digest = [0u8; 32];
        let mut consensus_digest = [0u8; 32];
        let r_base = c_idx * 32;
        let s_base = stage_block + c_idx * 32;
        let d_base = 2 * stage_block + c_idx * 32;
        let cn_base = 3 * stage_block + c_idx * 32;
        residual_digest.copy_from_slice(&stage_digests_host[r_base..r_base + 32]);
        sign_digest.copy_from_slice(&stage_digests_host[s_base..s_base + 32]);
        detector_digest.copy_from_slice(&stage_digests_host[d_base..d_base + 32]);
        consensus_digest.copy_from_slice(&stage_digests_host[cn_base..cn_base + 32]);

        let summary = dsfb_gpu_debug_core::casefile::build_compact_summary_from_device_digests(
            catalogs[c_idx],
            contract,
            "cuda",
            &workspace.features[cell_range],
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            &candidates,
        );
        summaries.push(summary);
    }
    Ok(summaries)
}

/// R.4 fused Throughput dispatch — same evidence chain and case-file
/// output as [`build_gpu_throughput_device_digests_on_workspace`], but
/// the residual and sign stages run as a Pre-Alpha EWMA kernel
/// followed by a cell-parallel fused R+S kernel. The fused path is
/// byte-preserving: the resulting `CaseFile.hashes` are bit-identical
/// to the un-fused reference on the same fixture and contract,
/// pinned by `fused_throughput_equivalence`.
///
/// At canonical 16×128 the parallelism win is small (sign was 16
/// threads, becomes 2 048). At scaled 256×4 096 the win is
/// substantial (sign was 256 threads, becomes 1 M+). The same R.5
/// candidate bytes, the same R.3b consensus-D2H strip, and the same
/// Audit mode hold.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if `workspace` dimensions disagree
///   with `contract`.
/// * `GpuError::KernelFailed(code)` for any non-zero `cudaError_t`.
pub fn build_gpu_fused_throughput_digests_on_workspace(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
) -> Result<CaseFile, GpuError> {
    workspace.assert_compatible(contract)?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let mut stage_digests_host = [0u8; 4 * 32];

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_fused_throughput_digests_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            workspace.d_drifts(),
            features.as_ptr(),
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            // R.3b: consensus D2H stripped from the fused path too —
            // axis-5 rides inside CandidateInterval after R.5.
            std::ptr::null_mut(),
            workspace.candidate_buf.as_mut_ptr(),
            workspace.candidate_count.as_mut_ptr(),
            stage_digests_host.as_mut_ptr(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = workspace.candidate_count[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(workspace.candidate_buf[base + i]);
        }
    }

    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    Ok(
        dsfb_gpu_debug_core::casefile::build_throughput_from_artifacts_and_device_digests(
            events,
            contract,
            "cuda",
            &features,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            // R.3b: bank no longer reads consensus after R.5.
            &[],
            &candidates,
        ),
    )
}

/// R.6b — opt-in pinned/async Throughput-digests dispatch.
///
/// Same evidence chain and case-file output as
/// [`build_gpu_throughput_device_digests_on_workspace`] (the
/// post-R.3b sync path), but the workspace's pinned host shadows
/// are used for H2D/D2H and every transfer + kernel launch goes
/// through the workspace's CUDA stream. A single
/// `cudaStreamSynchronize` at the end of the FFI call ensures the
/// caller observes a fully-drained workspace on return.
///
/// The workspace must be constructed with
/// [`GpuWorkspace::new_with_pinned_async`]; otherwise this entry
/// returns `GpuError::InvalidInput`. Single-stream by design — no
/// concurrent overlap with other dispatches and no double-buffering
/// in R.6b. CUDA Graph capture lands in R.6c.
///
/// Byte equivalence: a single CUDA stream executes its work in
/// program order, identical to default-stream behaviour. The case
/// file is bit-identical to the sync dispatch on the same fixture,
/// pinned by `r6b_pinned_async_equivalence`.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if `workspace` was not built with
///   pinned shadows + stream, or if dimensions disagree with
///   `contract`.
/// * `GpuError::KernelFailed(code)` for any non-zero `cudaError_t`
///   from a transfer, kernel launch, or `cudaStreamSynchronize`.
///
/// # Panics
///
/// The body uses `.expect()` on the pinned-shadow `Option`s after
/// the `has_pinned_async()` gate at the top of the function has
/// returned `true`. The expects are unreachable on any well-formed
/// workspace; they would only panic on a malformed workspace that
/// passes `has_pinned_async` but lacks one of the pinned shadows —
/// which the constructor invariant rules out.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
) -> Result<CaseFile, GpuError> {
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;

    // Compute features into the workspace's pinned shadow. The
    // computed Vec is dropped immediately after copy; the pinned
    // buffer persists across dispatches as part of the workspace.
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    {
        let pinned = workspace
            .features_pinned
            .as_mut()
            .expect("has_pinned_async guarantees features_pinned is Some");
        pinned.as_mut_slice().copy_from_slice(&features);
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let features_ptr = workspace
        .features_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_digests_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            features_ptr,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            // R.3b: consensus D2H stripped in this dispatch path
            // exactly as in the post-R.5 sync version.
            std::ptr::null_mut(),
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            // R.6d: opt-in constant-memory detector kernel when
            // the workspace successfully uploaded thresholds at
            // construction. Byte-equivalent fallback otherwise.
            c_int::from(workspace.has_const_thresholds()),
            // R.8: pass null — this entry does not collect per-stage
            // timings. The `_timed` variant below opts in.
            std::ptr::null_mut(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    // Flatten per-entity candidate slots from the pinned shadows.
    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    Ok(
        dsfb_gpu_debug_core::casefile::build_throughput_from_artifacts_and_device_digests(
            events,
            contract,
            "cuda",
            &features,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            // R.3b: bank no longer reads consensus after R.5.
            &[],
            &candidates,
        ),
    )
}

/// R.6c — Throughput-digests dispatch that prefers a captured
/// CUDA Graph when available and gracefully demotes to the R.6b
/// async path when graph capture is not. Byte-identical to the
/// R.6b reference under both branches: a captured launch only
/// replays the same kernel + memcpy topology recorded by R.6b's
/// FFI minus its terminal `cudaStreamSynchronize`.
///
/// The graph is **not** semantic. It only records the launch
/// plan against the workspace's pinned shadows and stream. The
/// CPU bank still admits episodes; the GPU still emits evidence.
/// The returned `GraphCaptureStatus` is recorded into the case
/// file's supplementary fields so the audit chain documents
/// which path produced the bytes.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if the workspace was not built
///   with `new_with_pinned_async`.
/// * `GpuError::KernelFailed` if the graph launch or the demoted
///   R.6b dispatch returns a non-zero CUDA error.
///
/// # Panics
///
/// Unreachable in practice: the `has_pinned_async` gate at the
/// top of the function guarantees the pinned shadow `expect`s
/// after a successful capture all succeed.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_graph_or_demote(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
) -> Result<(CaseFile, GraphCaptureStatus), GpuError> {
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }

    // Idempotent attempt to capture. If a graph already exists on
    // this workspace this returns `Captured` immediately; the
    // first call on a fresh workspace either succeeds or returns
    // `Demoted` with a cudaError reason.
    let status = workspace.try_capture_throughput_graph(contract)?;

    let plan_hash = match status {
        GraphCaptureStatus::Demoted { ref reason } => {
            // Fall back to the R.6b async path. Its case file is
            // byte-identical to the pageable/sync reference and
            // the graph_plan_hash supplementary fields stay None.
            let case = build_gpu_throughput_pinned_async_on_workspace(events, contract, workspace)?;
            return Ok((
                case,
                GraphCaptureStatus::Demoted {
                    reason: reason.clone(),
                },
            ));
        }
        GraphCaptureStatus::Captured { plan_hash } => plan_hash,
    };

    // Captured branch: stage features into the pinned shadow,
    // launch the captured graph on the workspace's stream, then
    // hash the host shadows into a CaseFile through the same
    // post-pipeline routine R.6b uses.
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    {
        let pinned = workspace
            .features_pinned
            .as_mut()
            .expect("has_pinned_async guarantees features_pinned is Some");
        pinned.as_mut_slice().copy_from_slice(&features);
    }

    let stream = workspace.stream_handle();
    let graph_exec = workspace.graph_exec();
    debug_assert!(graph_exec != 0, "Captured status implies graph_exec != 0");

    // Safety: graph_exec and stream are owned by `workspace` and
    // remain valid for the duration of this call. The FFI calls
    // `cudaGraphLaunch` then `cudaStreamSynchronize`, so on
    // return every pinned shadow is valid to read.
    #[allow(unsafe_code)]
    let launch_status: c_int =
        unsafe { crate::ffi::dsfb_gpu_launch_throughput_graph(graph_exec, stream) };
    if launch_status != 0 {
        return Err(GpuError::KernelFailed(launch_status));
    }

    // Flatten per-entity candidate slots from the pinned shadows.
    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("has_pinned_async guarantees candidate_count_pinned is Some")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("has_pinned_async guarantees candidates_pinned is Some")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("has_pinned_async guarantees stage_digests_pinned is Some")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    let case = dsfb_gpu_debug_core::casefile::build_throughput_from_artifacts_and_device_digests(
        events,
        contract,
        "cuda",
        &features,
        residual_digest,
        sign_digest,
        detector_digest,
        consensus_digest,
        // R.3b: bank no longer reads consensus after R.5.
        &[],
        &candidates,
    );
    Ok((case, GraphCaptureStatus::Captured { plan_hash }))
}

/// R.8 — host-side segment timings paired with the device per-stage
/// breakdown from `R8StageTimingsFfi`. Together they form the full
/// "where is time going" picture that R.7's 2.9× speedup at
/// 256x4096 K=64 doesn't surface on its own.
///
/// All fields are microseconds. `features_us` is the host wall time
/// of `compute_features`; `bank_and_finalize_us` is the host wall
/// time of `casefile::build_throughput_from_artifacts_and_device_digests`
/// which includes both the bank-admission loop and the case-file
/// hash-chain construction. Splitting those two further would
/// require core-crate plumbing that R.8 deliberately avoids per the
/// "no contract changes" rule in the plan.
#[derive(Copy, Clone, Debug, Default)]
pub struct R8HostStageTimings {
    /// Host wall time of the windowing / feature-aggregation step
    /// (`dsfb_gpu_debug_core::window::compute_features`).
    pub features_us: f32,
    /// Host wall time of the bank admission + case-file hash chain
    /// finalisation step
    /// (`casefile::build_throughput_from_artifacts_and_device_digests`).
    pub bank_and_finalize_us: f32,
}

/// R.8 — public alias for the FFI struct that carries cudaEvent-derived
/// per-stage microseconds back from the C++ wrapper. Re-exported under
/// the cleaner `R8StageTimings` name so external callers (the bench,
/// any future internal user) do not need to refer to the `Ffi` suffix.
/// All fields are documented on the underlying type in `ffi.rs`.
pub use crate::ffi::R8StageTimingsFfi as R8StageTimings;

/// R.8 — same byte-for-byte dispatch as
/// `build_gpu_throughput_pinned_async_on_workspace`, but the C++
/// wrapper records cudaEvent timings around each kernel + memcpy
/// and the Rust side times the host segments (`compute_features`,
/// bank admission + finalisation). Returns the case file plus
/// both timing structs so the bench can produce the per-stage
/// percent-of-time table required by R.8.
///
/// # Errors
///
/// Same as the non-timed entry; this routine only adds the
/// instrumentation. The case file's bytes are identical to the
/// non-timed path under the same inputs and the same workspace
/// const-thresholds state.
///
/// # Panics
///
/// Unreachable: the `has_pinned_async` gate keeps the
/// `expect()` calls on pinned shadows reachable only when the
/// pinned shadows are `Some`.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace_timed(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
) -> Result<(CaseFile, R8StageTimings, R8HostStageTimings), GpuError> {
    use std::time::Instant;
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;

    // ---- host: feature generation timing -------------------------
    let t_features = Instant::now();
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    // Convert via u64 → f32 to dodge clippy's u128 → f32 precision
    // warning. The host-side feature-generation wall is well under
    // 1 second at every supported scale (~10 µs canonical, ~1 ms
    // scale-large) so the u64-narrowing cast is loss-free in
    // practice.
    #[allow(clippy::cast_precision_loss)]
    let features_us = (t_features.elapsed().as_nanos() as u64) as f32 / 1_000.0_f32;

    {
        let pinned = workspace
            .features_pinned
            .as_mut()
            .expect("has_pinned_async guarantees features_pinned is Some");
        pinned.as_mut_slice().copy_from_slice(&features);
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let features_ptr = workspace
        .features_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();

    let mut device_timings = R8StageTimings::default();

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_digests_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            features_ptr,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            std::ptr::null_mut(),
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            c_int::from(workspace.has_const_thresholds()),
            std::ptr::from_mut(&mut device_timings),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    // ---- host: bank admission + case-file hash chain timing -------
    let t_bank = Instant::now();
    let case = dsfb_gpu_debug_core::casefile::build_throughput_from_artifacts_and_device_digests(
        events,
        contract,
        "cuda",
        &features,
        residual_digest,
        sign_digest,
        detector_digest,
        consensus_digest,
        &[],
        &candidates,
    );
    // Same u128 → u64 → f32 narrowing as `features_us` above; the
    // post-launch host work is also well under 1 second so the
    // cast is loss-free.
    #[allow(clippy::cast_precision_loss)]
    let bank_and_finalize_us = (t_bank.elapsed().as_nanos() as u64) as f32 / 1_000.0_f32;

    let host_timings = R8HostStageTimings {
        features_us,
        bank_and_finalize_us,
    };

    Ok((case, device_timings, host_timings))
}

/// R.8.5 — Throughput-mode tree-digest dispatch.
///
/// Same residual / sign / detector / consensus / candidate kernels
/// as `build_gpu_throughput_pinned_async_on_workspace`, but the 4
/// single-thread `*_digest_kernel_batched` kernels are replaced
/// with the deterministic block-parallel tree digest from R.8.5
/// (`cuda/kernels.cu::tree_digest_leaf_kernel` + `_root_kernel`).
///
/// Produces a case file whose 4 stage hashes are
/// INTENTIONALLY DIFFERENT from the serial-digest path's stage
/// hashes — the tree digest commits to (canonical chunked stage
/// bytes || domain separator) rather than (canonical stage
/// bytes) alone. The case file's supplementary metadata records
/// `digest_mode = tree_sha256_v1`, `chunk_size`, and
/// `chunk_count` per stage so replay catches a mode-mismatched
/// receipt at validation time.
///
/// K=1 only at v0. K>1 batched tree digest needs the batched FFI
/// extended with leaf-arena strides; deferred to a follow-up.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if the workspace was not built with
///   `new_with_pinned_async` (no pinned shadows + stream) or the
///   contract's dimensions disagree with the workspace's allocation.
/// * `GpuError::KernelFailed` if any kernel returns a non-zero
///   `cudaError_t`. The dispatch returns immediately on error; the
///   case file is not built in that case.
///
/// # Panics
///
/// Unreachable in practice: the `has_pinned_async` gate keeps the
/// pinned shadow `expect()` calls reachable only when the shadows
/// are `Some`.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace_tree(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
) -> Result<CaseFile, GpuError> {
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }
    if !workspace.has_tree_digest() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace tree-digest scratch is not allocated; \
             call GpuWorkspace::new_with_pinned_async(contract) on a fresh workspace",
        ));
    }

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;

    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    {
        let pinned = workspace
            .features_pinned
            .as_mut()
            .expect("has_pinned_async guarantees features_pinned is Some");
        pinned.as_mut_slice().copy_from_slice(&features);
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let features_ptr = workspace
        .features_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();
    let tree_chunk_size = workspace.tree_chunk_size();
    let tree_leaves_stride = workspace.tree_leaves_stride_bytes();
    let tree_scratch_stride = workspace.tree_scratch_stride_bytes();
    let d_tree_leaves = workspace.d_tree_leaves();
    let d_tree_scratch = workspace.d_tree_scratch();

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_tree_digests_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            d_tree_leaves,
            tree_leaves_stride,
            d_tree_scratch,
            tree_scratch_stride,
            features_ptr,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            c_int::from(workspace.has_const_thresholds()),
            tree_chunk_size,
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    // Flatten per-entity candidate slots from the pinned shadows.
    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    // Hand the device-computed stage digests to the shared
    // case-file builder. The builder hashes them in canonical order
    // into the chain just like the serial-digest path; only the
    // bytes feeding the chain differ.
    Ok(
        dsfb_gpu_debug_core::casefile::build_throughput_from_artifacts_and_device_digests(
            events,
            contract,
            "cuda",
            &features,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            &[],
            &candidates,
        ),
    )
}

/// R.11 — Throughput-mode dispatch + compact verdict finalizer.
///
/// Same kernel path as
/// `build_gpu_throughput_pinned_async_on_workspace_tree` (R.8.5
/// tree digest on the GPU side), but the host post-launch step
/// uses
/// `dsfb_gpu_debug_core::casefile::build_throughput_compact_verdict_from_device_digests`
/// with caller-supplied `FixtureHashes`, skipping the ~250 MB of
/// scalar SHA-256 work that the non-compact builder did per
/// dispatch (re-hashing ~4 M events + ~1 M window features).
///
/// The case file emitted here is **byte-identical** to the one
/// `build_gpu_throughput_pinned_async_on_workspace_tree` would
/// have produced for the same fixture, because `FixtureHashes`
/// carries the same canonical hashes the non-compact path would
/// have re-computed. The compact builder does not weaken any
/// chain commitment — it only reuses precomputed values.
///
/// **Semantic Non-Bypass Axiom**: unchanged. Episodes are still
/// admitted via the bank module's private `BankAdmissionToken`
/// constructor; the compact path only changes WHERE the input
/// commitment bytes come from (precomputed vs. recomputed), not
/// who can mint an admitted episode.
///
/// # Errors
///
/// Same as the non-compact tree-digest dispatch.
///
/// # Panics
///
/// Unreachable in practice — the `has_pinned_async` and
/// `has_tree_digest` gates keep the pinned-shadow `expect()`
/// calls reachable only when the shadows are `Some`.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace_tree_compact(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
    fixture: &dsfb_gpu_debug_core::casefile::FixtureHashes,
) -> Result<CaseFile, GpuError> {
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }
    if !workspace.has_tree_digest() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace tree-digest scratch is not allocated; \
             call GpuWorkspace::new_with_pinned_async(contract) on a fresh workspace",
        ));
    }

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;

    // `compute_features` still has to run because the GPU pipeline
    // needs the populated `WindowFeature[]` to consume. The
    // FixtureHashes saves us from re-hashing those features, not
    // from generating them. At v0 fixture generation is a small
    // fraction of wall (1.6 % at 256x4096 K=1 per R.8); a future
    // optimisation could cache the features themselves on the
    // workspace, but that is out of R.11's scope.
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    {
        let pinned = workspace
            .features_pinned
            .as_mut()
            .expect("has_pinned_async guarantees features_pinned is Some");
        pinned.as_mut_slice().copy_from_slice(&features);
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let features_ptr = workspace
        .features_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();
    let tree_chunk_size = workspace.tree_chunk_size();
    let tree_leaves_stride = workspace.tree_leaves_stride_bytes();
    let tree_scratch_stride = workspace.tree_scratch_stride_bytes();
    let d_tree_leaves = workspace.d_tree_leaves();
    let d_tree_scratch = workspace.d_tree_scratch();

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_tree_digests_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors(),
            workspace.d_consensus(),
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            d_tree_leaves,
            tree_leaves_stride,
            d_tree_scratch,
            tree_scratch_stride,
            features_ptr,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            c_int::from(workspace.has_const_thresholds()),
            tree_chunk_size,
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    // The compact verdict builder skips the ~250 MB of host
    // SHA-256 work that the non-compact builder did. Same chain
    // semantics; the FixtureHashes contract guarantees the bytes
    // committed to are identical.
    Ok(
        dsfb_gpu_debug_core::casefile::build_throughput_compact_verdict_from_device_digests(
            contract,
            "cuda",
            fixture,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            &candidates,
        ),
    )
}

/// R.9.b.2 — GPU wide-mask detector dispatch (D64).
///
/// Computes features on the host, uploads them to the workspace's
/// pinned shadow, runs residual + drift/slew sign + the D64
/// wide-mask detector kernel on the GPU, and copies the resulting
/// `DetectorCellWide[]` back to the caller. Stops at the detector
/// stage so the parity test against the CPU reference is precise.
///
/// **Byte equivalence**: every output cell's `DetectorMask2048`
/// matches the CPU `dsfb_gpu_debug_core::detector::evaluate_wide(
/// DetectorProfile::D64, ...)` byte-for-byte. The R.9.b parity
/// test pins this invariant at the canonical fixture; the same
/// math runs at every scale.
///
/// **Memory budget honesty**: `DetectorCellWide` is 264 bytes per
/// cell. At canonical 16×128 that's 540 KB per catalog; at
/// 256×4096 it's ~270 MB. K > 1 batched wide dispatch is R.9.c+
/// work — for now the buffer is sized for K = 1 single-catalog,
/// and `ensure_wide_detector_buffer` returns
/// `GpuError::KernelFailed` if the device allocation refuses.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if the workspace was not built with
///   `new_with_pinned_async`.
/// * `GpuError::KernelFailed` if the wide-detector buffer allocation
///   refuses or any kernel returns a non-zero `cudaError_t`.
///
/// # Panics
///
/// Unreachable in practice — every pinned-shadow `expect()` after
/// the `has_pinned_async` gate.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn evaluate_detector_wide_d64_on_workspace(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
) -> Result<Vec<dsfb_gpu_debug_core::detector::DetectorCellWide>, GpuError> {
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }
    workspace.ensure_wide_detector_buffer()?;
    // R.10a — axis-5 grid-sum precompute buffer used by the wide
    // candidate kernel's flush loop.
    workspace.ensure_axis5_grid_sum_buffer()?;
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer that
    // backs the cell-parallel `drift_slew_sign_kernel_cellpar`
    // split. Workspace-resident i32 [n_entities × n_windows]
    // per catalog (4 MB at canonical 256×4096). Replaces the
    // legacy monolithic `drift_slew_sign_kernel`'s register-
    // carried EWMA state.
    workspace.ensure_drift_buffer()?;
    // R.10b — compact-wide-detector-digest-v1 arena that feeds the
    // detector-stage tree digest. Wide cells stay on device; only
    // the bytes the digest hashes change shape (264 → 18 B/cell).
    workspace.ensure_detector_digest_compact_buffer()?;
    // R.10c — parallel-candidate-collapse scratch buffers
    // (fired flags + boundary tuples). Inputs to the new
    // candidate_fired/boundary/pack kernels that replaced the
    // entity-serial wide candidate-collapse kernel.
    workspace.ensure_candidate_parallel_buffers()?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let n_cells = contract.n_entities as usize * contract.n_windows as usize;

    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    {
        let pinned = workspace
            .features_pinned
            .as_mut()
            .expect("has_pinned_async guarantees features_pinned is Some");
        pinned.as_mut_slice().copy_from_slice(&features);
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;

    let features_ptr = workspace
        .features_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let stream = workspace.stream_handle();

    // Host output buffer for the wide cells. 270 MB at scale-large;
    // we allocate on heap because the size is unknown at compile time
    // and stack-allocating ~270 MB would blow the stack.
    let mut host_wide: Vec<dsfb_gpu_debug_core::detector::DetectorCellWide> =
        vec![dsfb_gpu_debug_core::detector::DetectorCellWide::default(); n_cells];

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_evaluate_detector_wide_d64_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            workspace.d_detectors_wide(),
            features_ptr,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            host_wide.as_mut_ptr(),
            stream,
            0, // R.9.b.2: const-thresholds flag reserved for future use
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    Ok(host_wide)
}

/// R.9.b.3 — full D64 Throughput dispatch (wide detector +
/// projected consensus + projected candidate + tree digest of
/// wide stage bytes + compact verdict finalizer).
///
/// Routes the same pinned/async stream as
/// `build_gpu_throughput_pinned_async_on_workspace_tree_compact`
/// but uses the wide-mask GPU kernels with on-device OR projection
/// to the canonical 16-motif basis. The bank ABI is unchanged —
/// `CandidateInterval::union_mask` arrives at the host with the
/// projected u16 mask folded into its u32 layout, exactly as the
/// D16 path produces it.
///
/// **Semantic Non-Bypass**: every admitted episode goes through
/// `bank_collapse` which mints the `BankAdmissionToken`. R.9.b.3
/// does not introduce a new admission path; it only changes what
/// detector evidence the bank sees.
///
/// **Case-file divergence from D16**: the chain's detector,
/// consensus, candidate, and episode hashes all differ from the
/// D16 path because the OR projection produces a richer mask
/// (⊇ canonical 16-mask). The contract's
/// `detector_registry_hash` should be pinned to
/// `DetectorProfile::D64.registry_hash()` by the caller so the
/// `detector_registry` chain link matches the active profile.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if the workspace lacks pinned
///   shadows or tree-digest scratch.
/// * `GpuError::KernelFailed` if any kernel returns a non-zero
///   `cudaError_t` or the wide-detector buffer allocation refuses.
///
/// # Panics
///
/// Unreachable — every `expect()` after the workspace gates.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
    fixture: &dsfb_gpu_debug_core::casefile::FixtureHashes,
) -> Result<CaseFile, GpuError> {
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }
    if !workspace.has_tree_digest() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace tree-digest scratch is not allocated; \
             call GpuWorkspace::new_with_pinned_async(contract) on a fresh workspace",
        ));
    }
    workspace.ensure_wide_detector_buffer()?;
    // R.10a — axis-5 grid-sum precompute buffer used by the wide
    // candidate kernel's flush loop.
    workspace.ensure_axis5_grid_sum_buffer()?;
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer that
    // backs the cell-parallel `drift_slew_sign_kernel_cellpar`
    // split. Workspace-resident i32 [n_entities × n_windows]
    // per catalog (4 MB at canonical 256×4096). Replaces the
    // legacy monolithic `drift_slew_sign_kernel`'s register-
    // carried EWMA state.
    workspace.ensure_drift_buffer()?;
    // R.10b — compact-wide-detector-digest-v1 arena that feeds the
    // detector-stage tree digest. Wide cells stay on device; only
    // the bytes the digest hashes change shape (264 → 18 B/cell).
    workspace.ensure_detector_digest_compact_buffer()?;
    // R.10c — parallel-candidate-collapse scratch buffers
    // (fired flags + boundary tuples). Inputs to the new
    // candidate_fired/boundary/pack kernels that replaced the
    // entity-serial wide candidate-collapse kernel.
    workspace.ensure_candidate_parallel_buffers()?;
    // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch
    // (`d_candidate_run_buffer` + `d_candidate_run_count`). The
    // D64 _timed FFI swap replaces the legacy single-kernel
    // `candidate_boundary_kernel_wide` with the new
    // `candidate_boundary_precompute_kernel` + `candidate_boundary_cellpar_emit_kernel`
    // pair to break the 2.1 %-occupancy ceiling.
    workspace.ensure_candidate_run_buffer()?;
    // R.11b — events buffer + pinned host shadow sized for the
    // actual input event count. compute_features no longer runs on
    // the host; the kernel builds WindowFeature[] on-device from
    // the events H2D'd here.
    workspace.ensure_events_buffer(events.len() as u64)?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let window_size_ns: u64 = u64::from(contract.window_size_ms) * 1_000_000;

    // R.11b — copy events into the pinned shadow. The dispatch's
    // FFI then triggers a single cudaMemcpyAsync H2D from this
    // page-locked source. The pinned shadow is sized exactly to
    // the workspace's current event capacity; copy the leading
    // `events.len()` entries.
    {
        let pinned = workspace
            .events_pinned
            .as_mut()
            .expect("ensure_events_buffer guarantees events_pinned is Some");
        let slice = pinned.as_mut_slice();
        // R.11c — pack each event into its 16-byte compact form as
        // we copy. The audit `TraceEvent[]` slice is untouched; the
        // pinned shadow holds the throughput-mode projection.
        for (dst, src) in slice.iter_mut().zip(events.iter()).take(events.len()) {
            *dst = GpuTraceEventCompact::from_trace_event(src);
        }
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let events_ptr = workspace
        .events_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();
    let tree_chunk_size = workspace.tree_chunk_size();
    let tree_leaves_stride = workspace.tree_leaves_stride_bytes();
    let tree_scratch_stride = workspace.tree_scratch_stride_bytes();
    let d_tree_leaves = workspace.d_tree_leaves();
    let d_tree_scratch = workspace.d_tree_scratch();
    let d_detectors_wide = workspace.d_detectors_wide();
    // Cast workspace's i64 grid-sum buffer pointer. cudaMalloc
    // returns ≥256-byte aligned memory; the localised allow keeps
    // clippy happy without a global override.
    #[allow(clippy::cast_ptr_alignment)]
    let d_axis5_grid_sum = workspace.d_axis5_grid_sum().cast::<i64>();
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer.
    let d_drift_buffer = workspace.d_drift_buffer();
    // R.10b — compact-wide-detector-digest-v1 arena pointer + the
    // two u16 metadata values the pack kernel folds into each
    // 18-byte compact record.
    let d_detector_digest_compact = workspace.d_detector_digest_compact();
    let d_candidate_fired = workspace.d_candidate_fired();
    let d_candidate_boundaries = workspace.d_candidate_boundaries();
    // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch pointers
    // (workspace-resident; ~33 KB at canonical 256 × 4 096 K=1). Allocated
    // by `workspace.ensure_candidate_run_buffer()` above; passed through
    // the D64 _timed FFI so the new `candidate_boundary_precompute_kernel`
    // can write run boundaries here and the `candidate_boundary_cellpar_emit_kernel`
    // can publish them into the legacy `d_candidate_boundaries` slot table.
    let d_candidate_run_buffer = workspace.d_candidate_run_buffer();
    let d_candidate_run_count = workspace.d_candidate_run_count();
    let d_events = workspace.d_events();
    let n_events_u64 = events.len() as u64;
    // R.11b — `ticks_per_event_ns` follows the structured-fixture
    // derivation: `(n_windows × window_size_ns) / n_events`. Both
    // `synthesize` and `synthesize_scaled` produce events whose
    // `ts_ns = i × ticks_per_event_ns`. The kernel uses this stride
    // to find candidate event indices per cell.
    let ticks_per_event_ns: u64 = if n_events_u64 == 0 {
        1
    } else {
        (u64::from(contract.n_windows) * window_size_ns) / n_events_u64
    };
    let profile_id_i32 = DetectorProfile::D64.active_detector_count() as i32;
    let wide_mask_words_used_i32 = DetectorProfile::D64.mask_word_count() as i32;

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_d64_tree_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            d_drift_buffer,
            d_detectors_wide,
            workspace.d_consensus(),
            d_axis5_grid_sum,
            d_detector_digest_compact,
            d_candidate_fired,
            d_candidate_boundaries,
            // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch.
            d_candidate_run_buffer,
            d_candidate_run_count,
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            d_tree_leaves,
            tree_leaves_stride,
            d_tree_scratch,
            tree_scratch_stride,
            d_events,
            events_ptr,
            n_events_u64,
            ticks_per_event_ns,
            window_size_ns,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            profile_id_i32,
            wide_mask_words_used_i32,
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            tree_chunk_size,
            0, // digest_mode_id = 0 (TreeSha256V1 — S-PERF.11 default path)
            std::ptr::null_mut(),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    Ok(
        dsfb_gpu_debug_core::casefile::build_throughput_compact_verdict_from_device_digests(
            contract,
            "cuda",
            fixture,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            &candidates,
        ),
    )
}

/// R.9.d.1 — full D128 Throughput dispatch.
///
/// Mirrors `build_gpu_throughput_pinned_async_on_workspace_d64_tree_
/// compact` but routes through the D128 kernels: 16 motifs × 8
/// threshold-scaled variants per cell. Bridge invariants from R.9.b
/// extended: D128.V0..V3 scales mirror D64.V0..V3 bit-for-bit, so
/// D128 OR-projection ⊇ D64 OR-projection ⊇ canonical D16. The
/// bank ABI is unchanged.
///
/// **Scope-locked**: this dispatch does NOT apply the R.10b
/// compact-wide-detector-digest pack. The detector tree-digest
/// hashes the full 264-byte `DetectorCellWide` stride. R.10b for
/// D128 is intentionally deferred to R.9.d.1-followup so the
/// post-D128 R.12b sweep can measure where the new bottleneck is
/// before optimising blind.
///
/// **Case-file divergence from D64**: the chain's
/// `detector_registry` link binds to
/// `DetectorProfile::D128.registry_hash()` if the caller pinned it
/// on the contract; the consensus / candidate / final hashes also
/// differ because the OR-projected mask sees more cells fire under
/// D128's wider variant set than under D64's.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if the workspace lacks pinned
///   shadows, tree-digest scratch, or the events buffer.
/// * `GpuError::KernelFailed` on any non-zero `cudaError_t` or
///   on wide-detector / events / axis-5 / candidate-parallel
///   buffer allocation refusal.
///
/// # Panics
///
/// Unreachable — every `expect()` runs after a workspace gate
/// proves the pinned shadow is `Some`.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
    fixture: &dsfb_gpu_debug_core::casefile::FixtureHashes,
) -> Result<CaseFile, GpuError> {
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }
    if !workspace.has_tree_digest() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace tree-digest scratch is not allocated; \
             call GpuWorkspace::new_with_pinned_async(contract) on a fresh workspace",
        ));
    }
    workspace.ensure_wide_detector_buffer()?;
    workspace.ensure_axis5_grid_sum_buffer()?;
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer that
    // backs the cell-parallel `drift_slew_sign_kernel_cellpar`
    // split. Workspace-resident i32 [n_entities × n_windows]
    // per catalog (4 MB at canonical 256×4096). Replaces the
    // legacy monolithic `drift_slew_sign_kernel`'s register-
    // carried EWMA state.
    workspace.ensure_drift_buffer()?;
    workspace.ensure_candidate_parallel_buffers()?;
    workspace.ensure_events_buffer(events.len() as u64)?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let window_size_ns: u64 = u64::from(contract.window_size_ms) * 1_000_000;

    // Pack events into the R.11c compact form and stage them in the
    // pinned host shadow before the FFI's H2D.
    {
        let pinned = workspace
            .events_pinned
            .as_mut()
            .expect("ensure_events_buffer guarantees events_pinned is Some");
        let slice = pinned.as_mut_slice();
        for (dst, src) in slice.iter_mut().zip(events.iter()).take(events.len()) {
            *dst = GpuTraceEventCompact::from_trace_event(src);
        }
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let events_ptr = workspace
        .events_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();
    let tree_chunk_size = workspace.tree_chunk_size();
    let tree_leaves_stride = workspace.tree_leaves_stride_bytes();
    let tree_scratch_stride = workspace.tree_scratch_stride_bytes();
    let d_tree_leaves = workspace.d_tree_leaves();
    let d_tree_scratch = workspace.d_tree_scratch();
    let d_detectors_wide = workspace.d_detectors_wide();
    #[allow(clippy::cast_ptr_alignment)]
    let d_axis5_grid_sum = workspace.d_axis5_grid_sum().cast::<i64>();
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer.
    let _d_drift_buffer = workspace.d_drift_buffer(); // allocated for D64 workspace compatibility; D128 FFI does not consume it
    let d_candidate_fired = workspace.d_candidate_fired();
    let d_candidate_boundaries = workspace.d_candidate_boundaries();
    let d_events = workspace.d_events();
    let n_events_u64 = events.len() as u64;
    let ticks_per_event_ns: u64 = if n_events_u64 == 0 {
        1
    } else {
        (u64::from(contract.n_windows) * window_size_ns) / n_events_u64
    };

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_d128_tree_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            d_detectors_wide,
            workspace.d_consensus(),
            d_axis5_grid_sum,
            d_candidate_fired,
            d_candidate_boundaries,
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            d_tree_leaves,
            tree_leaves_stride,
            d_tree_scratch,
            tree_scratch_stride,
            d_events,
            events_ptr,
            n_events_u64,
            ticks_per_event_ns,
            window_size_ns,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            tree_chunk_size,
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    Ok(
        dsfb_gpu_debug_core::casefile::build_throughput_compact_verdict_from_device_digests(
            contract,
            "cuda",
            fixture,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            &candidates,
        ),
    )
}

/// R.9.d.2.1 — D205 wide-mask throughput dispatch.
///
/// Mirrors `build_gpu_throughput_pinned_async_on_workspace_d128_
/// tree_compact` kernel-for-kernel; the only difference is that
/// the three profile-specific kernels
/// (`detector_motif_kernel_wide_d205`,
/// `consensus_grid_kernel_wide_d205`,
/// `candidate_pack_kernel_wide_d205`) replace their D128
/// counterparts and use a 13-variant scaled-threshold table with
/// an active-bit gate `det_id < 205`.
///
/// **D205 is a scaling-ladder byte-equivalence proof, NOT a new
/// performance headline.** The D64 ≈55× full-pipeline campaign
/// reduction at the courthouse-factory workload remains the R.13
/// headline. R.9.d.2.1 closes the asymmetry left by R.9.d.2
/// (CPU D205 landed; GPU D205 deferred): every supported
/// profile now has a CPU + GPU byte-equivalent execution path.
///
/// Bridge invariants (panel-locked, pinned by acceptance tests):
/// the D205 GPU mask matches the D205 CPU mask cell-for-cell;
/// D205 V0-only projection equals canonical D16; D205 OR ⊇ D128
/// OR ⊇ D64 OR ⊇ canonical D16; high bits ≥ 205 are
/// deterministically zero; D205 popcount never exceeds 205.
///
/// **Scope-locked**: this dispatch does NOT apply the R.10b
/// compact-wide-detector-digest pack. The detector tree-digest
/// hashes the full 264-byte `DetectorCellWide` stride. R.10b for
/// D205 is intentionally deferred (it would be a separate
/// optimisation commit). The same workspace fields used by D128
/// are reused — `DetectorCellWide` is profile-independent at 264
/// bytes per cell.
///
/// **Case-file divergence from D128**: the chain's
/// `detector_registry` link binds to
/// `DetectorProfile::D205.registry_hash()` if the caller pinned
/// it on the contract; consensus / candidate / final hashes also
/// differ because D205's additional V8..V12 variants can set
/// firings the D128 mask did not, while keeping V0..V7
/// byte-identical.
///
/// # Errors
///
/// * `GpuError::InvalidInput` if the workspace lacks pinned
///   shadows, tree-digest scratch, or the events buffer.
/// * `GpuError::KernelFailed` on any non-zero `cudaError_t` or
///   on wide-detector / events / axis-5 / candidate-parallel
///   buffer allocation refusal.
///
/// # Panics
///
/// Unreachable — every `expect()` runs after a workspace gate
/// proves the pinned shadow is `Some`.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
    fixture: &dsfb_gpu_debug_core::casefile::FixtureHashes,
) -> Result<CaseFile, GpuError> {
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }
    if !workspace.has_tree_digest() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace tree-digest scratch is not allocated; \
             call GpuWorkspace::new_with_pinned_async(contract) on a fresh workspace",
        ));
    }
    workspace.ensure_wide_detector_buffer()?;
    workspace.ensure_axis5_grid_sum_buffer()?;
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer that
    // backs the cell-parallel `drift_slew_sign_kernel_cellpar`
    // split. Workspace-resident i32 [n_entities × n_windows]
    // per catalog (4 MB at canonical 256×4096). Replaces the
    // legacy monolithic `drift_slew_sign_kernel`'s register-
    // carried EWMA state.
    workspace.ensure_drift_buffer()?;
    workspace.ensure_candidate_parallel_buffers()?;
    workspace.ensure_events_buffer(events.len() as u64)?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let window_size_ns: u64 = u64::from(contract.window_size_ms) * 1_000_000;

    // Pack events into the R.11c compact form and stage them in the
    // pinned host shadow before the FFI's H2D.
    {
        let pinned = workspace
            .events_pinned
            .as_mut()
            .expect("ensure_events_buffer guarantees events_pinned is Some");
        let slice = pinned.as_mut_slice();
        for (dst, src) in slice.iter_mut().zip(events.iter()).take(events.len()) {
            *dst = GpuTraceEventCompact::from_trace_event(src);
        }
    }

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let events_ptr = workspace
        .events_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();
    let tree_chunk_size = workspace.tree_chunk_size();
    let tree_leaves_stride = workspace.tree_leaves_stride_bytes();
    let tree_scratch_stride = workspace.tree_scratch_stride_bytes();
    let d_tree_leaves = workspace.d_tree_leaves();
    let d_tree_scratch = workspace.d_tree_scratch();
    let d_detectors_wide = workspace.d_detectors_wide();
    #[allow(clippy::cast_ptr_alignment)]
    let d_axis5_grid_sum = workspace.d_axis5_grid_sum().cast::<i64>();
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer.
    let _d_drift_buffer = workspace.d_drift_buffer(); // allocated for D64 workspace compatibility; D205 FFI does not consume it
    let d_candidate_fired = workspace.d_candidate_fired();
    let d_candidate_boundaries = workspace.d_candidate_boundaries();
    let d_events = workspace.d_events();
    let n_events_u64 = events.len() as u64;
    let ticks_per_event_ns: u64 = if n_events_u64 == 0 {
        1
    } else {
        (u64::from(contract.n_windows) * window_size_ns) / n_events_u64
    };

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_d205_tree_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            d_detectors_wide,
            workspace.d_consensus(),
            d_axis5_grid_sum,
            d_candidate_fired,
            d_candidate_boundaries,
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            d_tree_leaves,
            tree_leaves_stride,
            d_tree_scratch,
            tree_scratch_stride,
            d_events,
            events_ptr,
            n_events_u64,
            ticks_per_event_ns,
            window_size_ns,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            tree_chunk_size,
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    Ok(
        dsfb_gpu_debug_core::casefile::build_throughput_compact_verdict_from_device_digests(
            contract,
            "cuda",
            fixture,
            residual_digest,
            sign_digest,
            detector_digest,
            consensus_digest,
            &candidates,
        ),
    )
}

/// R.9.c-diagnostic — host-side timing partner for the D64
/// throughput tree-compact path. Mirrors `R8HostStageTimings` but
/// applies to the D64 throughput dispatch: feature generation on
/// the host before the FFI and bank admission + case-file
/// finalisation on the host after.
///
/// Together with `D64ThroughputStageTimings` this is the full
/// "where is the time going" picture for the D64 throughput path,
/// which the R.9.b.3 timing report identified as the next target
/// for analysis. Splitting bank vs case-file further would require
/// core-crate plumbing intentionally avoided in this diagnostic-only
/// commit.
#[derive(Copy, Clone, Debug, Default)]
pub struct D64ThroughputHostStageTimings {
    /// Host wall time of the input pack-to-pinned staging step
    /// (TraceEvent[] -> GpuTraceEventCompact[] copy into the
    /// workspace's pinned events shadow before the FFI dispatch).
    ///
    /// **Renamed at S-PERF.13** (panel-locked 2026-05-18,
    /// post-S-PERF.13-PREFLIGHT). Previously named `features_us`,
    /// which mislabelled the slot: R.11b already moved
    /// `window::compute_features` to device via
    /// `window_feature_kernel_structured`. This slot has never
    /// timed host feature math on the D64 `_timed` path; it has
    /// always measured input event materialisation into the
    /// pinned shadow. See
    /// `reports/s_perf_13_preflight_d64_feature_path_audit.txt`
    /// for the full audit verdict
    /// (`FeaturePathMixedHostStagingDeviceCompute`).
    pub host_input_staging_us: f32,
    /// Host wall time of
    /// `casefile::build_throughput_compact_verdict_from_device_digests`,
    /// which folds the 4 device digests + compact candidate
    /// descriptors + precomputed `FixtureHashes` into the 12-link
    /// chain. Includes the bank admission loop.
    pub bank_and_finalize_us: f32,
}

/// R.9.c-diagnostic — public alias for the FFI struct that carries
/// cudaEvent-derived per-stage microseconds back from the C++
/// wrapper. Re-exported under the cleaner
/// `D64ThroughputStageTimings` name so external callers don't see
/// the `Ffi` suffix. Field docs live on the underlying type.
pub use crate::ffi::D64ThroughputStageTimingsFfi as D64ThroughputStageTimings;

/// R.9.c-diagnostic — same dispatch as
/// `build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact`,
/// but the C++ wrapper records cudaEvents around each kernel and
/// memcpy, and the Rust side times the host segments. Returns the
/// case file plus both timing structs.
///
/// **No byte-changes**: the case-file output is identical to the
/// non-timed entry; the events ride along on the captured stream
/// and the elapsed-time math runs after `cudaStreamSynchronize`,
/// outside the byte path. This is the load-bearing property that
/// keeps the R.9.b.3 reproducibility invariants intact.
///
/// # Errors
///
/// Same as the non-timed entry; this routine only adds the
/// instrumentation.
///
/// # Panics
///
/// Unreachable: the `has_pinned_async` gate keeps the
/// `expect()` calls on pinned shadows reachable only when the
/// pinned shadows are `Some`.
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
    fixture: &dsfb_gpu_debug_core::casefile::FixtureHashes,
) -> Result<
    (
        CaseFile,
        D64ThroughputStageTimings,
        D64ThroughputHostStageTimings,
    ),
    GpuError,
> {
    use std::time::Instant;
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }
    if !workspace.has_tree_digest() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace tree-digest scratch is not allocated; \
             call GpuWorkspace::new_with_pinned_async(contract) on a fresh workspace",
        ));
    }
    workspace.ensure_wide_detector_buffer()?;
    // R.10a — axis-5 grid-sum precompute buffer used by the wide
    // candidate kernel's flush loop.
    workspace.ensure_axis5_grid_sum_buffer()?;
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer that
    // backs the cell-parallel `drift_slew_sign_kernel_cellpar`
    // split. Workspace-resident i32 [n_entities × n_windows]
    // per catalog (4 MB at canonical 256×4096). Replaces the
    // legacy monolithic `drift_slew_sign_kernel`'s register-
    // carried EWMA state.
    workspace.ensure_drift_buffer()?;
    // R.10b — compact-wide-detector-digest-v1 arena that feeds the
    // detector-stage tree digest. Wide cells stay on device; only
    // the bytes the digest hashes change shape (264 → 18 B/cell).
    workspace.ensure_detector_digest_compact_buffer()?;
    // R.10c — parallel-candidate-collapse scratch buffers
    // (fired flags + boundary tuples). Inputs to the new
    // candidate_fired/boundary/pack kernels that replaced the
    // entity-serial wide candidate-collapse kernel.
    workspace.ensure_candidate_parallel_buffers()?;
    // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch.
    workspace.ensure_candidate_run_buffer()?;
    // R.11b — events buffer + pinned host shadow sized for the
    // actual input event count. compute_features no longer runs on
    // the host; the kernel builds WindowFeature[] on-device.
    workspace.ensure_events_buffer(events.len() as u64)?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let window_size_ns: u64 = u64::from(contract.window_size_ms) * 1_000_000;

    // ---- host: input pack-to-pinned staging timing ---------------
    //
    // **S-PERF.13 (panel-locked 2026-05-18) — this slot measures
    // INPUT STAGING, NOT host compute_features.** R.11b moved
    // feature math to device under `window_feature_kernel_structured`
    // (launched inside the FFI path at cuda/kernels.cu around line
    // 5026-5028). What remains here is the
    // `TraceEvent` (48 B) -> `GpuTraceEventCompact` (16 B)
    // projection into the pinned events shadow before the FFI
    // dispatch. The historical `features_us` field name was renamed
    // to `host_input_staging_us` at S-PERF.13 seal time after the
    // S-PERF.13-PREFLIGHT audit verdict
    // (`FeaturePathMixedHostStagingDeviceCompute`); see
    // `reports/s_perf_13_preflight_d64_feature_path_audit.txt` for
    // the full evidence chain.
    //
    // The pack loop below (`for (dst, src) in slice.iter_mut().zip(...)`)
    // is the SIMD-acceleration target for S-PERF.13: at the canonical
    // 256x4096 K=1 fixture (~4.2M events x 16 B = ~67 MB) the manual
    // iterator chain emits scalar stores that inhibit
    // autovectorization. The byte-identical packed-shadow output
    // contract is what permits SIMD/chunk-unroll without perturbing
    // any downstream hash or episode count.
    let t_input_staging = Instant::now();
    {
        let pinned = workspace
            .events_pinned
            .as_mut()
            .expect("ensure_events_buffer guarantees events_pinned is Some");
        let slice = pinned.as_mut_slice();
        // S-PERF.13 — bulk SIMD-friendly pack. Replaces the prior
        // scalar `for ... zip()` loop. The helper packs in chunks
        // of 8 with hand-unrolled stores; LLVM can prove the
        // chunk length and emit AVX2 vector stores on x86_64
        // Haswell+. R.11c projection semantics are unchanged
        // (every chunk-element is packed by the same
        // `GpuTraceEventCompact::from_trace_event` const fn);
        // pinned-shadow output is byte-identical to the scalar
        // path — panel-locked S-PERF.13 N8 + P6 contract.
        pack_events_to_pinned_simd(events, slice);
    }
    #[allow(clippy::cast_precision_loss)]
    let host_input_staging_us = (t_input_staging.elapsed().as_nanos() as u64) as f32 / 1_000.0_f32;

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let events_ptr = workspace
        .events_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();
    let tree_chunk_size = workspace.tree_chunk_size();
    let tree_leaves_stride = workspace.tree_leaves_stride_bytes();
    let tree_scratch_stride = workspace.tree_scratch_stride_bytes();
    let d_tree_leaves = workspace.d_tree_leaves();
    let d_tree_scratch = workspace.d_tree_scratch();
    let d_detectors_wide = workspace.d_detectors_wide();
    #[allow(clippy::cast_ptr_alignment)]
    let d_axis5_grid_sum = workspace.d_axis5_grid_sum().cast::<i64>();
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer.
    let d_drift_buffer = workspace.d_drift_buffer();
    let d_detector_digest_compact = workspace.d_detector_digest_compact();
    let d_candidate_fired = workspace.d_candidate_fired();
    let d_candidate_boundaries = workspace.d_candidate_boundaries();
    // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch pointers
    // (workspace-resident; ~33 KB at canonical 256 × 4 096 K=1). Allocated
    // by `workspace.ensure_candidate_run_buffer()` above; passed through
    // the D64 _timed FFI so the new `candidate_boundary_precompute_kernel`
    // can write run boundaries here and the `candidate_boundary_cellpar_emit_kernel`
    // can publish them into the legacy `d_candidate_boundaries` slot table.
    let d_candidate_run_buffer = workspace.d_candidate_run_buffer();
    let d_candidate_run_count = workspace.d_candidate_run_count();
    let d_events = workspace.d_events();
    let n_events_u64 = events.len() as u64;
    let ticks_per_event_ns: u64 = if n_events_u64 == 0 {
        1
    } else {
        (u64::from(contract.n_windows) * window_size_ns) / n_events_u64
    };
    let profile_id_i32 = DetectorProfile::D64.active_detector_count() as i32;
    let wide_mask_words_used_i32 = DetectorProfile::D64.mask_word_count() as i32;

    let mut device_timings = D64ThroughputStageTimings::default();

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_d64_tree_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            d_drift_buffer,
            d_detectors_wide,
            workspace.d_consensus(),
            d_axis5_grid_sum,
            d_detector_digest_compact,
            d_candidate_fired,
            d_candidate_boundaries,
            // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch.
            d_candidate_run_buffer,
            d_candidate_run_count,
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            d_tree_leaves,
            tree_leaves_stride,
            d_tree_scratch,
            tree_scratch_stride,
            d_events,
            events_ptr,
            n_events_u64,
            ticks_per_event_ns,
            window_size_ns,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            profile_id_i32,
            wide_mask_words_used_i32,
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            tree_chunk_size,
            0, // digest_mode_id = 0 (TreeSha256V1 — S-PERF.11 default path)
            std::ptr::from_mut(&mut device_timings),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    // ---- host: bank admission + case-file finalize timing --------
    let t_bank = Instant::now();
    let case = dsfb_gpu_debug_core::casefile::build_throughput_compact_verdict_from_device_digests(
        contract,
        "cuda",
        fixture,
        residual_digest,
        sign_digest,
        detector_digest,
        consensus_digest,
        &candidates,
    );
    #[allow(clippy::cast_precision_loss)]
    let bank_and_finalize_us = (t_bank.elapsed().as_nanos() as u64) as f32 / 1_000.0_f32;

    let host_timings = D64ThroughputHostStageTimings {
        host_input_staging_us,
        bank_and_finalize_us,
    };

    Ok((case, device_timings, host_timings))
}

/// S-PERF.12 — D64 Throughput pipeline using the
/// **CompactDensorDigestV1** throughput-mode digest. Mirrors
/// [`build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed`]
/// in every respect except the digest mode: the four per-stage
/// tree digests are produced by
/// [`cuda::compact_densor_digest_v1_leaf_kernel`] +
/// [`cuda::compact_densor_digest_v1_root_kernel`] which hash a
/// deterministic XOR-fold-by-4 compact projection of each
/// chunk rather than the raw chunk bytes. Per-stage root
/// digests under this mode are NOT byte-identical to
/// `TreeSha256V1` — the canonical domain header
/// (`DSFB_STAGE_COMPACT_DENSOR_V1`, 28 bytes) and the per-leaf
/// SHA inputs are both different — and S-PERF.10's
/// `digest_mode_non_aliasing_law` covers this: each declared
/// throughput-digest mode owns its own root-byte projection.
///
/// Same court semantics as the TreeSha256V1 path:
///
/// * Same candidate descriptors (the bank stage runs on
///   `CandidateInterval` byte slices, not on digest bytes).
/// * R.12b episode counts 13 / 89 / 1917 preserved.
/// * Audit mode (SerialSha256) is untouched.
///
/// # Errors
///
/// Same as the tree-compact-timed entry: returns
/// [`GpuError::InvalidInput`] if the workspace lacks pinned/
/// async + tree-digest scratch arenas, and [`GpuError::KernelFailed`]
/// on any CUDA error.
///
/// # Panics
///
/// Unreachable: same `expect()` discipline as
/// [`build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed`].
#[allow(clippy::expect_used, clippy::too_many_lines)]
pub fn build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
    events: &[TraceEvent],
    contract: &Contract,
    workspace: &mut GpuWorkspace,
    fixture: &dsfb_gpu_debug_core::casefile::FixtureHashes,
) -> Result<
    (
        CaseFile,
        D64ThroughputStageTimings,
        D64ThroughputHostStageTimings,
    ),
    GpuError,
> {
    use std::time::Instant;
    workspace.assert_compatible(contract)?;
    if !workspace.has_pinned_async() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace was not built with pinned shadows + stream; \
             call GpuWorkspace::new_with_pinned_async(contract)",
        ));
    }
    if !workspace.has_tree_digest() {
        return Err(GpuError::InvalidInput(
            "GpuWorkspace tree-digest scratch is not allocated; \
             call GpuWorkspace::new_with_pinned_async(contract) on a fresh workspace",
        ));
    }
    workspace.ensure_wide_detector_buffer()?;
    workspace.ensure_axis5_grid_sum_buffer()?;
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer that
    // backs the cell-parallel `drift_slew_sign_kernel_cellpar`
    // split. Workspace-resident i32 [n_entities × n_windows]
    // per catalog (4 MB at canonical 256×4096). Replaces the
    // legacy monolithic `drift_slew_sign_kernel`'s register-
    // carried EWMA state.
    workspace.ensure_drift_buffer()?;
    workspace.ensure_detector_digest_compact_buffer()?;
    workspace.ensure_candidate_parallel_buffers()?;
    // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch.
    workspace.ensure_candidate_run_buffer()?;
    workspace.ensure_events_buffer(events.len() as u64)?;

    let n_entities = contract.n_entities as i32;
    let n_windows = contract.n_windows as i32;
    let window_size_ns: u64 = u64::from(contract.window_size_ms) * 1_000_000;

    // ---- host: input pack-to-pinned staging timing ---------------
    //
    // **S-PERF.13 (panel-locked 2026-05-18) — this slot measures
    // INPUT STAGING, NOT host compute_features.** R.11b moved
    // feature math to device under `window_feature_kernel_structured`
    // (launched inside the FFI path). The pack loop below is the
    // SIMD-acceleration target for S-PERF.13; the byte-identical
    // packed-shadow output contract permits SIMD/chunk-unroll
    // without perturbing any downstream hash or episode count.
    // See the tree-compact-timed sibling above + the
    // S-PERF.13-PREFLIGHT receipt for the full audit chain.
    let t_input_staging = Instant::now();
    {
        let pinned = workspace
            .events_pinned
            .as_mut()
            .expect("ensure_events_buffer guarantees events_pinned is Some");
        let slice = pinned.as_mut_slice();
        // S-PERF.13 — bulk SIMD-friendly pack (see the
        // tree-compact-timed sibling above for the full doc + the
        // panel-locked byte-identical-output contract).
        pack_events_to_pinned_simd(events, slice);
    }
    #[allow(clippy::cast_precision_loss)]
    let host_input_staging_us = (t_input_staging.elapsed().as_nanos() as u64) as f32 / 1_000.0_f32;

    let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
    let baseline = Baseline::CANONICAL;
    let candidate_cfg = CandidateConfig::CANONICAL;

    let events_ptr = workspace
        .events_pinned
        .as_ref()
        .expect("guarded above")
        .as_ptr();
    let candidates_ptr = workspace
        .candidates_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let count_ptr = workspace
        .candidate_count_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let digests_ptr = workspace
        .stage_digests_pinned
        .as_mut()
        .expect("guarded above")
        .as_mut_ptr();
    let stream = workspace.stream_handle();
    let tree_chunk_size = workspace.tree_chunk_size();
    let tree_leaves_stride = workspace.tree_leaves_stride_bytes();
    let tree_scratch_stride = workspace.tree_scratch_stride_bytes();
    let d_tree_leaves = workspace.d_tree_leaves();
    let d_tree_scratch = workspace.d_tree_scratch();
    let d_detectors_wide = workspace.d_detectors_wide();
    #[allow(clippy::cast_ptr_alignment)]
    let d_axis5_grid_sum = workspace.d_axis5_grid_sum().cast::<i64>();
    // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer.
    let d_drift_buffer = workspace.d_drift_buffer();
    let d_detector_digest_compact = workspace.d_detector_digest_compact();
    let d_candidate_fired = workspace.d_candidate_fired();
    let d_candidate_boundaries = workspace.d_candidate_boundaries();
    // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch pointers
    // (workspace-resident; ~33 KB at canonical 256 × 4 096 K=1). Allocated
    // by `workspace.ensure_candidate_run_buffer()` above; passed through
    // the D64 _timed FFI so the new `candidate_boundary_precompute_kernel`
    // can write run boundaries here and the `candidate_boundary_cellpar_emit_kernel`
    // can publish them into the legacy `d_candidate_boundaries` slot table.
    let d_candidate_run_buffer = workspace.d_candidate_run_buffer();
    let d_candidate_run_count = workspace.d_candidate_run_count();
    let d_events = workspace.d_events();
    let n_events_u64 = events.len() as u64;
    let ticks_per_event_ns: u64 = if n_events_u64 == 0 {
        1
    } else {
        (u64::from(contract.n_windows) * window_size_ns) / n_events_u64
    };
    let profile_id_i32 = DetectorProfile::D64.active_detector_count() as i32;
    let wide_mask_words_used_i32 = DetectorProfile::D64.mask_word_count() as i32;

    let mut device_timings = D64ThroughputStageTimings::default();

    #[allow(unsafe_code)]
    let status: c_int = unsafe {
        crate::ffi::dsfb_gpu_run_pipeline_throughput_d64_tree_async_on_workspace(
            workspace.d_features(),
            workspace.d_residuals(),
            workspace.d_signs(),
            d_drift_buffer,
            d_detectors_wide,
            workspace.d_consensus(),
            d_axis5_grid_sum,
            d_detector_digest_compact,
            d_candidate_fired,
            d_candidate_boundaries,
            // S-PERF.14c — Pre-Alpha + cellpar split intermediate scratch.
            d_candidate_run_buffer,
            d_candidate_run_count,
            workspace.d_candidates(),
            workspace.d_candidate_count(),
            workspace.d_stage_digests(),
            d_tree_leaves,
            tree_leaves_stride,
            d_tree_scratch,
            tree_scratch_stride,
            d_events,
            events_ptr,
            n_events_u64,
            ticks_per_event_ns,
            window_size_ns,
            n_entities,
            n_windows,
            contract.ewma_alpha_q16_raw,
            baseline.latency_us,
            baseline.error_rate_q16_raw,
            std::ptr::from_ref(&thresholds_ffi),
            candidate_cfg.min_detector_count as i32,
            candidate_cfg.min_residual_q_raw,
            candidate_cfg.min_length_windows as i32,
            MAX_CANDIDATES_PER_ENTITY,
            profile_id_i32,
            wide_mask_words_used_i32,
            candidates_ptr,
            count_ptr,
            digests_ptr,
            stream,
            tree_chunk_size,
            1, // digest_mode_id = 1 (CompactDensorDigestV1 — S-PERF.12 path)
            std::ptr::from_mut(&mut device_timings),
        )
    };
    if status != 0 {
        return Err(GpuError::KernelFailed(status));
    }

    let count_slice = workspace
        .candidate_count_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let slots = workspace
        .candidates_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut candidates: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..(workspace.n_entities as usize) {
        let count = count_slice[entity_id] as usize;
        let base = entity_id * MAX_CANDIDATES_PER_ENTITY as usize;
        for i in 0..count {
            candidates.push(slots[base + i]);
        }
    }

    let stage_digests_host = workspace
        .stage_digests_pinned
        .as_ref()
        .expect("guarded above")
        .as_slice();
    let mut residual_digest = [0u8; 32];
    let mut sign_digest = [0u8; 32];
    let mut detector_digest = [0u8; 32];
    let mut consensus_digest = [0u8; 32];
    residual_digest.copy_from_slice(&stage_digests_host[0..32]);
    sign_digest.copy_from_slice(&stage_digests_host[32..64]);
    detector_digest.copy_from_slice(&stage_digests_host[64..96]);
    consensus_digest.copy_from_slice(&stage_digests_host[96..128]);

    let t_bank = Instant::now();
    // S-PERF.12 — the CaseFile builder is unchanged: it folds
    // the four per-stage digests into the throughput-mode case
    // file under the existing TreeSha256V1 chain in the case
    // file's metadata. The CompactDensorDigestV1 mode identity
    // is recorded ONLY in the S-PERF.12 corpus receipt (not in
    // the case file), so all prior case-file hashes stay
    // byte-identical. The four stage digests under
    // CompactDensorDigestV1 differ from TreeSha256V1 → the
    // case-file hashes also differ, which is expected and
    // panel-acknowledged via `digest_mode_non_aliasing_law`.
    let case = dsfb_gpu_debug_core::casefile::build_throughput_compact_verdict_from_device_digests(
        contract,
        "cuda",
        fixture,
        residual_digest,
        sign_digest,
        detector_digest,
        consensus_digest,
        &candidates,
    );
    #[allow(clippy::cast_precision_loss)]
    let bank_and_finalize_us = (t_bank.elapsed().as_nanos() as u64) as f32 / 1_000.0_f32;

    let host_timings = D64ThroughputHostStageTimings {
        host_input_staging_us,
        bank_and_finalize_us,
    };

    Ok((case, device_timings, host_timings))
}

#[cfg(test)]
mod s_perf_13_pack_equivalence_tests {
    //! S-PERF.13 — CPU-side equivalence tests for the
    //! [`pack_events_to_pinned_simd`] helper. These tests pin
    //! the panel-locked N8 + P6 contract: the bulk SIMD pack
    //! MUST produce byte-identical output to the scalar
    //! `for ... zip()` baseline for any input length. No GPU,
    //! no FFI, no clock dependence — pure host-side
    //! determinism check that runs under every workspace gate
    //! including the pre-commit hook.
    use super::*;
    use dsfb_gpu_debug_core::event::TraceEvent;

    /// Reference scalar pack — the pre-S-PERF.13 baseline.
    fn pack_events_to_pinned_scalar(events: &[TraceEvent], pinned: &mut [GpuTraceEventCompact]) {
        for (dst, src) in pinned.iter_mut().zip(events.iter()).take(events.len()) {
            *dst = GpuTraceEventCompact::from_trace_event(src);
        }
    }

    fn synthetic_events(n: usize) -> Vec<TraceEvent> {
        let mut events = Vec::with_capacity(n);
        let mut state: u64 = 0xD5FB_D5FB_D5FB_D5FB;
        for i in 0..n {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let ts_ns = (i as u64) * 1000 + (state >> 16);
            let entity_id = (i as u32) % 1024;
            let latency_us = ((state >> 32) as u32) % 100_000;
            let error_code: u16 = u16::from((state & (1 << 7)) != 0);
            // TraceEvent::new takes all 10 fields positionally;
            // GpuTraceEventCompact::from_trace_event reads only
            // (ts_ns, entity_id, error_code, latency_us), so
            // the auxiliary fields (route_id / span_id /
            // parent_span_id / status_code / event_kind / flags)
            // are filled with deterministic-but-irrelevant values.
            events.push(TraceEvent::new(
                ts_ns,
                entity_id,
                (i as u32) % 16,
                state,
                state.wrapping_shr(8),
                latency_us,
                200,
                error_code,
                0,
                0,
            ));
        }
        events
    }

    /// Panel-locked N8 + P6 contract — byte-equivalence
    /// across every length covering chunk boundaries
    /// (empty, tail-only, exact-chunk, multi-chunk + tail,
    /// canonical-scale and large-scale fixtures).
    #[test]
    fn packed_bytes_byte_identical_to_old_path() {
        for &n in &[0_usize, 1, 7, 8, 9, 64, 257, 4096, 65_537] {
            let events = synthetic_events(n);
            let mut scalar_out = vec![GpuTraceEventCompact::default(); n];
            let mut simd_out = vec![GpuTraceEventCompact::default(); n];
            pack_events_to_pinned_scalar(&events, &mut scalar_out);
            pack_events_to_pinned_simd(&events, &mut simd_out);
            assert_eq!(
                scalar_out, simd_out,
                "SIMD pack diverged from scalar baseline at n_events={n}; \
                 panel-locked S-PERF.13 N8 + P6 contract violated"
            );
        }
    }

    /// Defense-in-depth: helper must not write past
    /// `events.len()` even when the pinned buffer is larger.
    #[test]
    fn pack_does_not_write_past_events_len() {
        let events = synthetic_events(17);
        let mut pinned = vec![GpuTraceEventCompact::default(); 64];
        pack_events_to_pinned_simd(&events, &mut pinned);
        let mut expected_head = vec![GpuTraceEventCompact::default(); 17];
        pack_events_to_pinned_scalar(&events, &mut expected_head);
        assert_eq!(&pinned[..17], &expected_head[..]);
        for (i, slot) in pinned.iter().enumerate().skip(17) {
            assert_eq!(
                *slot,
                GpuTraceEventCompact::default(),
                "pinned shadow slot {i} written past events.len()"
            );
        }
    }

    /// Mutation sensitivity: changing any single event's
    /// bytes MUST surface as a different packed-shadow byte.
    #[test]
    fn changing_any_event_byte_changes_packed_output() {
        let n = 32;
        let baseline_events = synthetic_events(n);
        let mut baseline = vec![GpuTraceEventCompact::default(); n];
        pack_events_to_pinned_simd(&baseline_events, &mut baseline);

        for mutate_idx in 0..n {
            let mut mutated_events = baseline_events.clone();
            mutated_events[mutate_idx].latency_us =
                mutated_events[mutate_idx].latency_us.wrapping_add(1);
            let mut mutated = vec![GpuTraceEventCompact::default(); n];
            pack_events_to_pinned_simd(&mutated_events, &mut mutated);
            assert_ne!(
                baseline, mutated,
                "mutating event index {mutate_idx} produced byte-identical packed output; \
                 SIMD pack may be skipping a chunk boundary"
            );
        }
    }
}
