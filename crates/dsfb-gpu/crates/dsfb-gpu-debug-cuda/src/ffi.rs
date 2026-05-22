//! C ABI declarations for the CUDA kernels.
//!
//! `dsfb_gpu_run_pipeline` is the host-side wrapper defined in
//! `cuda/kernels.cu`. It owns the device-side allocations: the Rust caller
//! supplies host buffers sized to the contract's grid, and the wrapper
//! handles `cudaMalloc` / `cudaMemcpy` / kernel launches / `cudaFree`. The
//! return value is the raw `cudaError_t` (0 = success).

#![cfg(feature = "cuda")]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(missing_docs)]

use core::ffi::c_int;

use dsfb_gpu_debug_core::candidate::CandidateInterval;
use dsfb_gpu_debug_core::consensus::ConsensusCell;
use dsfb_gpu_debug_core::detector::{DetectorCell, DetectorCellWide};
use dsfb_gpu_debug_core::event::GpuTraceEventCompact;
use dsfb_gpu_debug_core::residual::ResidualCell;
use dsfb_gpu_debug_core::sign::SignCell;
use dsfb_gpu_debug_core::window::WindowFeature;

/// repr-C mirror of `cuda/layout.cuh::DetectorThresholds`. The Rust
/// `DetectorThresholds` from the core crate carries the same fields in the
/// same order but has additional methods; the FFI form drops the methods
/// and keeps only the data.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DetectorThresholdsFfi {
    pub spike_q16_raw: i32,
    pub sustain_q16_raw: i32,
    pub slew_shock_q16_raw: i32,
    pub plateau_min_q16_raw: i32,
    pub plateau_slew_max_q16_raw: i32,
    pub plateau_windows: u32,
    pub oscillation_window: u32,
    pub oscillation_alternations: u32,
    pub deadband_low_q16_raw: i32,
    pub deadband_high_q16_raw: i32,
    pub error_burst_q16_raw: i32,
    pub coupling_lat_q16_raw: i32,
    pub coupling_err_q16_raw: i32,
    pub variance_window: u32,
    pub variance_threshold_q16_raw: i32,
    pub ramp_window: u32,
    pub recovery_min_norm_q16_raw: i32,
    pub clean_band_q16_raw: i32,
    pub confuser_min_q16_raw: i32,
    pub fanout_drift_q16_raw: i32,
    pub entity_anomaly_factor_q16_raw: i32,
    pub history_window: u32,
}

impl From<&dsfb_gpu_debug_core::detector::DetectorThresholds> for DetectorThresholdsFfi {
    fn from(t: &dsfb_gpu_debug_core::detector::DetectorThresholds) -> Self {
        Self {
            spike_q16_raw: t.spike_q16_raw,
            sustain_q16_raw: t.sustain_q16_raw,
            slew_shock_q16_raw: t.slew_shock_q16_raw,
            plateau_min_q16_raw: t.plateau_min_q16_raw,
            plateau_slew_max_q16_raw: t.plateau_slew_max_q16_raw,
            plateau_windows: t.plateau_windows,
            oscillation_window: t.oscillation_window,
            oscillation_alternations: t.oscillation_alternations,
            deadband_low_q16_raw: t.deadband_low_q16_raw,
            deadband_high_q16_raw: t.deadband_high_q16_raw,
            error_burst_q16_raw: t.error_burst_q16_raw,
            coupling_lat_q16_raw: t.coupling_lat_q16_raw,
            coupling_err_q16_raw: t.coupling_err_q16_raw,
            variance_window: t.variance_window,
            variance_threshold_q16_raw: t.variance_threshold_q16_raw,
            ramp_window: t.ramp_window,
            recovery_min_norm_q16_raw: t.recovery_min_norm_q16_raw,
            clean_band_q16_raw: t.clean_band_q16_raw,
            confuser_min_q16_raw: t.confuser_min_q16_raw,
            fanout_drift_q16_raw: t.fanout_drift_q16_raw,
            entity_anomaly_factor_q16_raw: t.entity_anomaly_factor_q16_raw,
            history_window: t.history_window,
        }
    }
}

/// R.9.c-diagnostic — per-stage cudaEvent timings for the D64
/// Throughput tree pipeline. Mirrors
/// `cuda/kernels.cu::D64ThroughputStageTimings` byte-for-byte under
/// `repr(C)`. Populated when the caller passes a non-null
/// `*mut D64ThroughputStageTimingsFfi` to the D64 throughput FFI;
/// when null, the FFI does no event work and pays zero overhead
/// — the existing R.9.b.3 wall time stays the canonical baseline.
///
/// All fields are CUDA-event-derived microseconds, recorded against
/// the pinned/async stream. The field order is the launch order of
/// the current D64 pipeline (residual → sign → wide detector →
/// wide consensus → wide candidate → 4 tree digests → D2H). Each
/// digest is reported individually because the R.9.b.3 16× wall-time
/// regression has several plausible culprits and we need per-kernel
/// granularity to distinguish them.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct D64ThroughputStageTimingsFfi {
    /// `cudaMemcpyAsync` H2D for `WindowFeature[]`.
    pub h2d_us: f32,
    /// `residual_field_kernel`.
    pub residual_us: f32,
    /// `drift_slew_sign_kernel` (per-entity EWMA recurrence).
    pub sign_us: f32,
    /// `detector_motif_kernel_wide_d64` — the wide-mask detector
    /// kernel writing 264-byte `DetectorCellWide` cells. Suspected
    /// dominant cost at full scale; this field confirms or
    /// falsifies that.
    pub detector_wide_us: f32,
    /// `consensus_grid_kernel_wide` — reads wide detector cells,
    /// projects to canonical u16 on-device, writes
    /// `ConsensusCell[]`.
    pub consensus_wide_us: f32,
    /// R.10a — `axis5_grid_sum_kernel_wide`: per-window i64 sum of
    /// `axis7_consensus_q` across entities. Hoisted out of the
    /// candidate kernel's flush loop so that loop drops from
    /// O(length × n_entities) to O(length).
    pub axis5_grid_sum_us: f32,
    /// `candidate_collapse_kernel_wide` — entity-serial walker.
    /// Post-R.10a the flush loop reads the precomputed grid sums
    /// instead of re-scanning the entity axis per candidate.
    pub candidate_wide_us: f32,
    /// Tree-digest stage 0 (residual cells, 16 B/cell).
    pub residual_digest_us: f32,
    /// Tree-digest stage 1 (sign cells, 20 B/cell).
    pub sign_digest_us: f32,
    /// Tree-digest stage 2 (wide detector cells, 264 B/cell).
    /// The R.9.c-compaction experiment isolated this stage's cost
    /// as ~wash on wall time even when shrunk 16×, so this field
    /// quantifies that empirically.
    pub detector_digest_us: f32,
    /// Tree-digest stage 3 (consensus cells, 32 B/cell).
    pub consensus_digest_us: f32,
    /// `cudaMemcpyAsync` D2H for candidates + counts +
    /// 4×32-byte stage digests.
    pub d2h_us: f32,
    /// Sum of the above: device-side wall time from the H2D
    /// `cudaEventRecord` to the D2H `cudaEventRecord` on the
    /// captured stream.
    pub total_device_us: f32,
}

/// R.8 — per-stage timings for the pinned/async Throughput-digests
/// pipeline. Mirrors `cuda/kernels.cu::R8StageTimings` byte-for-byte
/// under `repr(C)`. Populated when the caller passes a non-null
/// `*mut R8StageTimingsFfi` to the R.6b async FFI; ignored when null.
/// All fields are CUDA-event-derived microseconds.
///
/// Field order is the launch order: H2D → residual → drift/slew sign →
/// detector → consensus → candidate-collapse → 4 digest kernels → D2H.
/// `total_device_us` is the wall time from the H2D event to the D2H
/// event on the captured stream.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct R8StageTimingsFfi {
    /// `cudaMemcpyAsync` H2D for `WindowFeature[]`.
    pub h2d_us: f32,
    /// `residual_field_kernel`.
    pub residual_us: f32,
    /// `drift_slew_sign_kernel` (per-entity EWMA recurrence).
    pub sign_us: f32,
    /// `detector_motif_kernel` (param or const variant).
    pub detector_us: f32,
    /// `consensus_grid_kernel`.
    pub consensus_us: f32,
    /// `candidate_collapse_kernel`.
    pub candidate_us: f32,
    /// 4 per-stage device SHA-256 digest kernels combined (Tier 3B).
    pub digests_us: f32,
    /// `cudaMemcpyAsync` D2H for candidates + count + 4 stage digests.
    pub d2h_us: f32,
    /// Sum of the above; effectively the device-side wall time.
    pub total_device_us: f32,
}

/// Per-stage CUDA-event timings in microseconds. Mirrors the
/// `PipelineTimings` struct emitted by the host wrapper in
/// `cuda/kernels.cu`. All fields are floats because `cudaEventElapsedTime`
/// returns milliseconds as a `float`; the C++ side converts to µs before
/// writing. `total_us` includes alloc and free; the per-stage fields are
/// pairwise differences between consecutive event records.
///
/// Field order is fixed and must match the C ABI of the C++ struct in
/// `kernels.cu` verbatim.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct PipelineTimingsFfi {
    /// Wall time spent in the seven `cudaMalloc` calls.
    pub alloc_us: f32,
    /// Wall time for the single host-to-device copy of the
    /// `WindowFeature` array.
    pub h2d_us: f32,
    /// `residual_field` kernel time (from end of H2D to end of kernel 1).
    pub k1_residual_us: f32,
    /// `drift_slew_sign` kernel time.
    pub k2_sign_us: f32,
    /// `detector_motif` kernel time.
    pub k3_detector_us: f32,
    /// `consensus_grid` kernel time.
    pub k4_consensus_us: f32,
    /// `candidate_collapse` kernel time.
    pub k5_candidate_us: f32,
    /// Wall time for the six device-to-host copies of intermediates.
    pub d2h_us: f32,
    /// Wall time spent in `cudaFree` for the seven device buffers.
    pub free_us: f32,
    /// Total wall time on the device side, from the first event to the
    /// last. Equal to the sum of the per-stage fields ± event-record
    /// granularity.
    pub total_us: f32,
}

extern "C" {
    /// Batched allocation: allocates seven device buffers sized for
    /// `n_catalogs * n_entities * n_windows` cells each. Pairs with
    /// `dsfb_gpu_workspace_free` (the free routine is layout-agnostic).
    pub fn dsfb_gpu_workspace_alloc_batched(
        n_catalogs: i32,
        n_entities: i32,
        n_windows: i32,
        max_candidates_per_entity: i32,
        d_features_out: *mut *mut WindowFeature,
        d_residuals_out: *mut *mut ResidualCell,
        d_signs_out: *mut *mut SignCell,
        d_detectors_out: *mut *mut DetectorCell,
        d_consensus_out: *mut *mut ConsensusCell,
        d_candidates_out: *mut *mut CandidateInterval,
        d_candidate_count_out: *mut *mut i32,
    ) -> c_int;

    /// Batched dispatch: runs K independent catalogs through the
    /// deterministic-inference kernels in a single graph of launches.
    /// `h_features` carries `n_catalogs * n_entities * n_windows`
    /// WindowFeature records laid out catalog-major; outputs follow the
    /// same layout. Each kernel reads its catalog index from
    /// `blockIdx.z`. Per-catalog outputs are byte-identical to a
    /// single-catalog dispatch on the same inputs.
    pub fn dsfb_gpu_run_pipeline_batched(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        h_features: *const WindowFeature,
        n_catalogs: i32,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_residuals: *mut ResidualCell,
        h_signs: *mut SignCell,
        h_detectors: *mut DetectorCell,
        h_consensus: *mut ConsensusCell,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        timings_out: *mut PipelineTimingsFfi,
    ) -> c_int;

    /// Allocate the seven device buffers a pipeline run needs and write
    /// the resulting pointers through the out-parameters. Pairs with
    /// `dsfb_gpu_workspace_free`. Returns 0 on success; on failure the
    /// already-allocated buffers are rolled back and every out-pointer is
    /// set to null.
    pub fn dsfb_gpu_workspace_alloc(
        n_entities: i32,
        n_windows: i32,
        max_candidates_per_entity: i32,
        d_features_out: *mut *mut WindowFeature,
        d_residuals_out: *mut *mut ResidualCell,
        d_signs_out: *mut *mut SignCell,
        d_detectors_out: *mut *mut DetectorCell,
        d_consensus_out: *mut *mut ConsensusCell,
        d_candidates_out: *mut *mut CandidateInterval,
        d_candidate_count_out: *mut *mut i32,
    ) -> c_int;

    /// Free every non-null device pointer in the workspace. Best-effort;
    /// returns the last non-success cudaError seen.
    pub fn dsfb_gpu_workspace_free(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
    ) -> c_int;

    /// Run the deterministic pipeline on a pre-allocated workspace. Same
    /// outputs as `dsfb_gpu_run_pipeline` but skips the per-call
    /// cudaMalloc/cudaFree storm. `timings_out` may be null; when set,
    /// `alloc_us` and `free_us` are written as 0 because no allocation
    /// occurs here.
    pub fn dsfb_gpu_run_pipeline_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        h_features: *const WindowFeature,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_residuals: *mut ResidualCell,
        h_signs: *mut SignCell,
        h_detectors: *mut DetectorCell,
        h_consensus: *mut ConsensusCell,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        timings_out: *mut PipelineTimingsFfi,
    ) -> c_int;

    /// Run the full deterministic pipeline on the GPU. All host buffers are
    /// allocated by the caller and must be sized exactly:
    ///
    /// * `h_features`, `h_residuals`, `h_signs`, `h_detectors`, `h_consensus`
    ///   — each `n_entities * n_windows` elements.
    /// * `h_candidates` — `n_entities * max_candidates_per_entity` elements.
    /// * `h_candidate_count_per_entity` — `n_entities` elements.
    ///
    /// `timings_out` may be null. When non-null the wrapper records CUDA
    /// events around each stage and writes per-stage microsecond timings
    /// before returning. Event creation/destruction is fully inside the
    /// `if (want_timings)` branch on the C++ side, so passing `null`
    /// reduces the function to its non-instrumented path with zero
    /// CUDA-event overhead.
    ///
    /// Returns 0 on success or a non-zero `cudaError_t` on failure.
    pub fn dsfb_gpu_run_pipeline(
        h_features: *const WindowFeature,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_residuals: *mut ResidualCell,
        h_signs: *mut SignCell,
        h_detectors: *mut DetectorCell,
        h_consensus: *mut ConsensusCell,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        timings_out: *mut PipelineTimingsFfi,
    ) -> c_int;

    /// Tier 3B self-test: hash `len` bytes from `host_data` on the GPU
    /// using the `__device__ SHA-256` defined in `cuda/sha256.cuh`. The
    /// 32-byte big-endian digest is written to `out_digest`. The Rust
    /// acceptance test asserts byte-equality with the host
    /// `dsfb_gpu_debug_core::hash::sha256` implementation over three
    /// known-vector inputs (empty, 55 bytes, 64 KiB). Single thread on
    /// device; the SHA-256 algorithm is inherently serial.
    pub fn dsfb_gpu_sha256_self_test(host_data: *const u8, len: u64, out_digest: *mut u8) -> c_int;

    /// S-PERF.14b.1 streaming self-test: hashes `len` bytes from
    /// `host_data` using the streaming SHA-256 init/update/finalize
    /// helpers defined in `cuda/sha256.cuh` (per-byte updates,
    /// stress-testing the buffering + tail-padding logic). The
    /// 32-byte big-endian digest is written to `out_digest`. The
    /// Rust acceptance test asserts byte-equality with
    /// `dsfb_gpu_sha256_self_test` (one-shot) over three known-
    /// vector inputs (empty, 55 bytes, 64 KiB) BEFORE the streaming
    /// helpers are allowed to be consumed by the rewritten
    /// `compact_densor_digest_v1_root_kernel_blockcoop`. Single
    /// thread on device; SHA-256 is inherently serial within a
    /// stream.
    pub fn dsfb_gpu_sha256_streaming_self_test(
        host_data: *const u8,
        len: u64,
        out_digest: *mut u8,
    ) -> c_int;

    /// Generic device-byte allocator used by the workspace digest
    /// buffer extension. `size` bytes are `cudaMalloc`'d and written
    /// through `out_ptr`. Returns 0 on success or the raw `cudaError_t`.
    pub fn dsfb_gpu_alloc_bytes(size: u64, out_ptr: *mut *mut u8) -> c_int;

    /// Generic device-byte free. Accepts a null pointer (returns
    /// `cudaSuccess`).
    pub fn dsfb_gpu_free_bytes(ptr: *mut u8) -> c_int;

    /// S-PERF.15.a Step 0 — synchronous device-to-host byte copy
    /// helper. Required by the panel-locked pre-fusion byte-capture
    /// harness: the harness D2Hs the post-dispatch
    /// `d_detectors_wide` + `d_detector_digest_compact` arenas,
    /// SHA-256s them, and asserts byte-equality against the
    /// `PINNED_PRE_S_PERF_15_A_*_ARENA_SHA256` constants. Returns
    /// `cudaSuccess` (= 0) on success. `size == 0` is a no-op; null
    /// `d_src` or `h_dst` with non-zero size returns
    /// `cudaErrorInvalidValue`.
    pub fn dsfb_gpu_memcpy_d2h_bytes(d_src: *const u8, h_dst: *mut u8, size: u64) -> c_int;

    /// S-PERF.15.d Step 1 — synchronous `cudaMemset` device-byte
    /// helper. Required by the panel-locked Direction A.1 (D64
    /// hot-lane projection with one-time zero-init): the
    /// `d_detectors_wide` buffer is zero-initialised exactly once
    /// at allocation time so that the rewritten
    /// `detector_motif_fused_d64_kernel` can skip writing the cold
    /// `mask[1..31]` lanes per dispatch without corrupting the
    /// byte-identity contract (`PINNED_PRE_S_PERF_15_D_DETECTOR_CELLWIDE_ARENA_SHA256`
    /// stays byte-identical because the cold bytes remain stable
    /// zero across the buffer's lifetime). `value` is the byte
    /// pattern (low 8 bits of `i32`, per `cudaMemset` semantics).
    /// Returns `cudaSuccess` (= 0) on success. `size == 0` is a
    /// no-op; null `d_dst` with non-zero size returns
    /// `cudaErrorInvalidValue`.
    pub fn dsfb_gpu_memset_bytes(d_dst: *mut u8, value: c_int, size: u64) -> c_int;

    /// S-PERF.14b.1 v3 tile-size sweep entry point. Allocates a
    /// synthetic leaf buffer of `n_chunks_per_catalog * 32 *
    /// n_catalogs` bytes (cuda-memset to 0xA5; digest values are
    /// NOT examined — only kernel wall-time is measured), runs
    /// the streaming_blockcoop kernel `n_warmup` warm-up
    /// iterations + `n_timed` timed iterations with the given
    /// `tile_bytes`, and writes the mean per-iteration wall in
    /// nanoseconds to `out_mean_ns`. Timing uses `cudaEvent_t`
    /// (sub-µs precision; sufficient for tile-size comparison
    /// without requiring sudo ncu). Path 1b v2 is INACTIVE in
    /// production until a tile size beats Path 1a's pinned
    /// 925.2 µs per-call wall; this entry point is the sweep
    /// harness that finds (or fails to find) such a tile size.
    pub fn dsfb_gpu_compact_densor_root_streaming_sweep_time(
        n_chunks_per_catalog: u32,
        chunk_size: u32,
        stage_id: u32,
        n_catalogs: u32,
        tile_bytes: u32,
        n_warmup: c_int,
        n_timed: c_int,
        out_mean_ns: *mut u64,
    ) -> c_int;

    /// S-PERF.14b.1 v3 Path 1a baseline measurement (apples-
    /// to-apples comparator). Same harness shape as the
    /// streaming sweep above but invokes
    /// `compact_densor_digest_v1_root_kernel_blockcoop` (the
    /// active production backend). Used by the v3 sweep test
    /// to produce per-stage Path 1a walls so the streaming
    /// kernel comparison can be honest at every stage shape
    /// (residual / sign / detector-compact-pack / consensus)
    /// rather than relying on a single averaged Path 1a number.
    pub fn dsfb_gpu_compact_densor_root_path1a_sweep_time(
        n_chunks_per_catalog: u32,
        chunk_size: u32,
        stage_id: u32,
        n_catalogs: u32,
        n_warmup: c_int,
        n_timed: c_int,
        out_mean_ns: *mut u64,
    ) -> c_int;

    /// R.6a — pinned host-byte allocator. `size` bytes are
    /// `cudaMallocHost`'d (page-locked) and written through
    /// `out_ptr`. Pinned host memory is the prerequisite for
    /// `cudaMemcpyAsync` (without staging through a pageable copy),
    /// which R.6b will use for stream-based double-buffered dispatch.
    /// Returns 0 on success or the raw `cudaError_t` (and zeroes
    /// `*out_ptr` on failure).
    pub fn dsfb_gpu_alloc_pinned_bytes(size: u64, out_ptr: *mut *mut u8) -> c_int;

    /// R.6a — pinned host-byte free. Mirror of
    /// `dsfb_gpu_alloc_pinned_bytes`. Accepts a null pointer
    /// (returns `cudaSuccess`). Calls `cudaFreeHost` rather than
    /// `cudaFree`; mixing the two is a runtime error.
    pub fn dsfb_gpu_free_pinned_bytes(ptr: *mut u8) -> c_int;

    /// R.6b — create a CUDA stream. The returned handle is opaque
    /// (`cudaStream_t` as `u64`); the Rust side never dereferences
    /// it. Round-trip the handle through
    /// `dsfb_gpu_run_pipeline_throughput_digests_async_on_workspace`
    /// and free it with `dsfb_gpu_destroy_stream`. Returns 0 on
    /// success or the raw `cudaError_t`; on failure `*out_stream`
    /// is set to 0.
    pub fn dsfb_gpu_create_stream(out_stream: *mut u64) -> c_int;

    /// R.6b — destroy a CUDA stream created via
    /// `dsfb_gpu_create_stream`. Accepts handle = 0 (no-op).
    pub fn dsfb_gpu_destroy_stream(stream: u64) -> c_int;

    /// R.8.5 — Throughput tree-digest dispatch. Mirrors
    /// `dsfb_gpu_run_pipeline_throughput_digests_async_on_workspace`
    /// but replaces the 4 single-thread `*_digest_kernel_batched`
    /// kernels with the parallel tree-digest pair (one leaf block
    /// per chunk + one root block per catalog). Produces a digest
    /// that is INTENTIONALLY DIFFERENT from the serial-digest
    /// output — the case file records `digest_mode = tree_sha256_v1`
    /// plus the chunk_size + chunk_count so replay catches a
    /// mode-mismatched receipt.
    ///
    /// The caller supplies two workspace-owned scratch arenas:
    /// `d_tree_leaves` holds per-stage leaf digests, striped at
    /// `tree_leaves_stride_bytes` per stage; `d_tree_scratch` holds
    /// the root kernel's per-stage concatenation scratch striped
    /// at `tree_scratch_stride_bytes` per stage. Both are allocated
    /// once at `GpuWorkspace::new_with_pinned_async` and sized for
    /// the contract's worst case.
    ///
    /// K=1 only at v0. The catalog axis is locked to 1 inside the
    /// FFI; batched-K tree digest is a follow-up.
    #[allow(clippy::too_many_arguments)]
    pub fn dsfb_gpu_run_pipeline_throughput_tree_digests_async_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        d_tree_leaves: *mut u8,
        tree_leaves_stride_bytes: u64,
        d_tree_scratch: *mut u8,
        tree_scratch_stride_bytes: u64,
        h_features: *const WindowFeature,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
        stream_handle: u64,
        use_const_thresholds: c_int,
        tree_chunk_size: u32,
    ) -> c_int;

    /// R.6d — upload `DetectorThresholds` into the device-side
    /// `__constant__ c_detector_thresholds` symbol. Synchronous;
    /// not bound to any stream so it cannot accidentally be baked
    /// into a captured CUDA Graph. Callers upload once at workspace
    /// construction; the dispatch wrappers then prefer the
    /// `_const` kernel variant. On failure callers retain the
    /// param-passing path (the FFI signatures accept a
    /// `use_const_thresholds` flag).
    ///
    /// Returns 0 on success or the raw `cudaError_t`. Idempotent:
    /// uploading the same canonical thresholds twice is a no-op
    /// from the device's perspective.
    pub fn dsfb_gpu_upload_detector_thresholds(h_thresholds: *const DetectorThresholdsFfi)
        -> c_int;

    /// R.6b — explicit-stream async variant of
    /// `dsfb_gpu_run_pipeline_throughput_digests_on_workspace`. Same
    /// pipeline, same outputs; uses `cudaMemcpyAsync` and
    /// stream-bound kernel launches throughout, then issues a
    /// single `cudaStreamSynchronize` before returning so the
    /// caller's contract (host buffers populated on return) is
    /// preserved. `stream_handle = 0` falls back to the default
    /// stream (matching the legacy sync wrapper's behaviour).
    #[allow(clippy::too_many_arguments)]
    pub fn dsfb_gpu_run_pipeline_throughput_digests_async_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        h_features: *const WindowFeature,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_consensus: *mut ConsensusCell,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
        stream_handle: u64,
        // R.6d — when non-zero, launch the `detector_motif_kernel_const`
        // variant which reads thresholds from `c_detector_thresholds`
        // (uploaded once at workspace construction via
        // `dsfb_gpu_upload_detector_thresholds`). When zero, falls back
        // to the original `detector_motif_kernel` with thresholds
        // passed by value.
        use_const_thresholds: c_int,
        // R.8 — when non-null, the wrapper records cudaEvent_t markers
        // around each kernel + each memcpy and writes per-stage
        // microsecond timings into the struct. When null, no event
        // overhead.
        stage_timings_out: *mut R8StageTimingsFfi,
    ) -> c_int;

    /// Tier 3B single-catalog dispatch: runs the pipeline + 4 digest
    /// kernels on a pre-existing workspace. Copies back consensus
    /// (needed by the bank stage's axis-5 entity-locality gate),
    /// candidates, counts, and 4 × 32-byte stage digests. The
    /// residual / sign / detector cell buffers stay on the device.
    pub fn dsfb_gpu_run_pipeline_throughput_digests_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        h_features: *const WindowFeature,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_consensus: *mut ConsensusCell,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
    ) -> c_int;

    /// R.4 fused single-catalog dispatch: same evidence chain as
    /// `dsfb_gpu_run_pipeline_throughput_digests_on_workspace`, but
    /// the residual + sign stages run as a Pre-Alpha EWMA kernel
    /// followed by a cell-parallel fused R+S kernel instead of the
    /// legacy entity-serial sign kernel. Detector / consensus /
    /// candidate / digest kernels are unchanged. Byte-preserving.
    ///
    /// `d_drifts` is the workspace's Pre-Alpha drift buffer
    /// (`int32_t*`, sized `n_entities * n_windows`); the C++ side
    /// uses it to stage the EWMA recurrence so the fused R+S kernel
    /// can read it cell-locally.
    pub fn dsfb_gpu_run_pipeline_fused_throughput_digests_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        d_drifts: *mut u8,
        h_features: *const WindowFeature,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_consensus: *mut ConsensusCell,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
    ) -> c_int;

    /// R.6c — opt-in CUDA Graph capture for the Throughput-digests
    /// pipeline. Wraps the R.6b async kernel sequence in a
    /// `cudaStreamBeginCapture`/`cudaStreamEndCapture` pair and
    /// instantiates the captured topology into a `cudaGraphExec_t`
    /// returned as an opaque `u64` through `out_graph_exec`. The
    /// host buffers passed here are the workspace's pinned shadows
    /// — the captured graph references those exact pointers, so
    /// refreshing the pinned `h_features` between launches feeds
    /// fresh inputs into the graph without re-capture.
    ///
    /// Capture can legitimately fail (driver / device does not
    /// support graphs, etc.). On failure the wrapper drains
    /// capture mode and returns the raw `cudaError_t`; the Rust
    /// caller demotes to the R.6b async path.
    ///
    /// Implementation note: the C++ wrapper creates a private
    /// scratch stream for `cudaStreamBeginCapture` and uses
    /// `cudaStreamCaptureModeThreadLocal`. This ensures (a) the
    /// workspace's launch stream is never put into capture mode,
    /// so the demoted fallback can use it unmodified, and (b)
    /// concurrent CUDA work on other threads (cargo test
    /// parallelism, etc.) does not invalidate this capture or
    /// vice-versa. `stream_handle` is accepted for API symmetry
    /// with the R.6b async FFI but is not used inside capture —
    /// the resulting `cudaGraphExec_t` is launchable on any
    /// stream by design.
    #[allow(clippy::too_many_arguments)]
    pub fn dsfb_gpu_try_capture_throughput_graph(
        out_graph_exec: *mut u64,
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        h_features: *const WindowFeature,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
        stream_handle: u64,
        // R.6d — non-zero bakes the `detector_motif_kernel_const`
        // variant into the captured topology; zero bakes the
        // param-passing variant. The caller is responsible for
        // uploading thresholds via
        // `dsfb_gpu_upload_detector_thresholds` BEFORE this call
        // when passing 1, because the upload is intentionally
        // kept outside the captured graph.
        use_const_thresholds: c_int,
    ) -> c_int;

    /// R.6c — launch a previously captured graph on the given
    /// stream and `cudaStreamSynchronize` before returning so the
    /// host pinned shadows are valid to read on return. Returns 0
    /// on success or the raw `cudaError_t`. Rejects
    /// `graph_exec = 0` or `stream_handle = 0` with
    /// `cudaErrorInvalidValue`.
    pub fn dsfb_gpu_launch_throughput_graph(graph_exec: u64, stream_handle: u64) -> c_int;

    /// R.9.b.3 — full D64 Throughput pipeline (wide detector +
    /// wide consensus + wide candidate + tree digest of wide
    /// stage bytes). Mirrors `dsfb_gpu_run_pipeline_throughput_
    /// tree_digests_async_on_workspace` but routes through the
    /// wide-mask kernels with on-device projection to the
    /// canonical 16-motif basis (the bank ABI is unchanged).
    /// Wide detector cells stay on device; only candidates +
    /// counts + 4 × 32-byte stage digests cross PCIe.
    ///
    /// R.9.c-diagnostic: when `timings_out` is non-null, the FFI
    /// records per-stage cudaEvents on the captured stream and
    /// writes the elapsed-time breakdown into the
    /// `D64ThroughputStageTimingsFfi` struct. Null = zero overhead
    /// and the byte output is identical to the pre-diagnostic
    /// path.
    #[allow(clippy::too_many_arguments)]
    pub fn dsfb_gpu_run_pipeline_throughput_d64_tree_async_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        // S-PERF.14 — Pre-Alpha drift EWMA precompute buffer.
        // Sized n_entities × n_windows × i32 per catalog.
        // Written by `drift_ewma_precompute_kernel`; consumed
        // by `drift_slew_sign_kernel_cellpar`. Allocated by
        // `GpuWorkspace::ensure_drift_buffer()`.
        d_drift_buffer: *mut i32,
        d_detectors_wide: *mut DetectorCellWide,
        d_consensus: *mut ConsensusCell,
        d_axis5_grid_sum: *mut i64,
        d_detector_digest_compact: *mut u8,
        d_candidate_fired: *mut u8,
        d_candidate_boundaries: *mut u8,
        // S-PERF.14c — per-entity intermediate run-boundary
        // scratch (`n_entities × MAX_CANDIDATES_PER_ENTITY × 8`
        // bytes). Allocated by
        // `GpuWorkspace::ensure_candidate_run_buffer()`. Written
        // by `candidate_boundary_precompute_kernel`; consumed by
        // `candidate_boundary_cellpar_emit_kernel`.
        d_candidate_run_buffer: *mut u8,
        // S-PERF.14c — per-entity surviving-run count scratch
        // (`n_entities × 4` bytes). Allocated by
        // `GpuWorkspace::ensure_candidate_run_buffer()`. Written
        // by `candidate_boundary_precompute_kernel`; consumed by
        // thread 0 of each (entity, catalog) cellpar emit block.
        d_candidate_run_count: *mut i32,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        d_tree_leaves: *mut u8,
        tree_leaves_stride_bytes: u64,
        d_tree_scratch: *mut u8,
        tree_scratch_stride_bytes: u64,
        d_events: *mut GpuTraceEventCompact,
        h_events: *const GpuTraceEventCompact,
        n_events: u64,
        ticks_per_event_ns: u64,
        window_size_ns: u64,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        profile_id: i32,
        wide_mask_words_used: i32,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
        stream_handle: u64,
        tree_chunk_size: u32,
        digest_mode_id: i32,
        timings_out: *mut D64ThroughputStageTimingsFfi,
    ) -> c_int;

    /// R.9.d.1 — full D128 Throughput pipeline. Mirrors the D64
    /// throughput FFI but routes through the D128-specific kernels
    /// (`detector_motif_kernel_wide_d128`, `consensus_grid_kernel_
    /// wide_d128`, `candidate_pack_kernel_wide_d128`) so the bank
    /// sees the OR-projected u16 mask over 8 variants per motif
    /// instead of 4. The bank ABI is unchanged — only the wider
    /// detector evidence drives more candidate firings.
    ///
    /// R.9.d.1 deliberately omits the R.10b compact-wide-detector-
    /// digest pack: the D128 detector tree-digest hashes the full
    /// 264-byte `DetectorCellWide` stride. Compact-pack for D128
    /// (a 28 B/cell `[header, mask_word_0, mask_word_1]` form) is
    /// R.9.d.1-followup work, gated by the post-D128 R.12b sweep.
    ///
    /// No optional `D64ThroughputStageTimings*` here — R.9.d.1
    /// keeps the surface narrow; the D128 saturation bench
    /// measures host wall-time directly.
    #[allow(clippy::too_many_arguments)]
    pub fn dsfb_gpu_run_pipeline_throughput_d128_tree_async_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors_wide: *mut DetectorCellWide,
        d_consensus: *mut ConsensusCell,
        d_axis5_grid_sum: *mut i64,
        d_candidate_fired: *mut u8,
        d_candidate_boundaries: *mut u8,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        d_tree_leaves: *mut u8,
        tree_leaves_stride_bytes: u64,
        d_tree_scratch: *mut u8,
        tree_scratch_stride_bytes: u64,
        d_events: *mut GpuTraceEventCompact,
        h_events: *const GpuTraceEventCompact,
        n_events: u64,
        ticks_per_event_ns: u64,
        window_size_ns: u64,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
        stream_handle: u64,
        tree_chunk_size: u32,
    ) -> c_int;

    /// R.9.d.2.1 — D205 wide-mask throughput dispatch. Mirrors the
    /// D128 entry-point signature exactly; the only difference is
    /// the underlying CUDA kernels (detector_motif / consensus_grid
    /// / candidate_pack are D205-specific). The same workspace
    /// fields are used (D205's 264-byte `DetectorCellWide` cell is
    /// identical to D128's). D205 is a scaling-ladder
    /// byte-equivalence proof, NOT a new performance headline.
    pub fn dsfb_gpu_run_pipeline_throughput_d205_tree_async_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors_wide: *mut DetectorCellWide,
        d_consensus: *mut ConsensusCell,
        d_axis5_grid_sum: *mut i64,
        d_candidate_fired: *mut u8,
        d_candidate_boundaries: *mut u8,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        d_tree_leaves: *mut u8,
        tree_leaves_stride_bytes: u64,
        d_tree_scratch: *mut u8,
        tree_scratch_stride_bytes: u64,
        d_events: *mut GpuTraceEventCompact,
        h_events: *const GpuTraceEventCompact,
        n_events: u64,
        ticks_per_event_ns: u64,
        window_size_ns: u64,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
        stream_handle: u64,
        tree_chunk_size: u32,
    ) -> c_int;

    /// R.9.b.2 — wide-mask detector dispatch (D64). Runs the
    /// standard residual → drift/slew sign → wide-detector
    /// pipeline on the workspace's device buffers, then copies
    /// the resulting `DetectorCellWide[]` (264 bytes per cell)
    /// back to the host. Stops at the detector stage so the
    /// parity claim against the CPU `evaluate_wide(D64, ...)`
    /// reference is precise. Full pipeline integration is R.9.b.3.
    ///
    /// `use_const_thresholds` is accepted for API symmetry with
    /// the throughput dispatch path; the wide kernel currently
    /// passes thresholds by value and ignores the flag — caller
    /// should pass 0.
    ///
    /// Returns 0 on success or the raw `cudaError_t` from the
    /// first failing CUDA call.
    pub fn dsfb_gpu_evaluate_detector_wide_d64_on_workspace(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors_wide: *mut DetectorCellWide,
        h_features: *const WindowFeature,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        h_detectors_wide: *mut DetectorCellWide,
        stream_handle: u64,
        use_const_thresholds: c_int,
    ) -> c_int;

    /// R.6c — destroy a graph exec created by
    /// `dsfb_gpu_try_capture_throughput_graph`. Accepts
    /// `graph_exec = 0` (no-op, returns `cudaSuccess`).
    pub fn dsfb_gpu_destroy_throughput_graph(graph_exec: u64) -> c_int;

    /// Tier 3B batched dispatch: K-catalog pipeline + parallel digest
    /// kernels. Residual / sign / detector cells stay on device;
    /// per-catalog consensus + candidates + counts + (4 × 32) stage
    /// digests come back. The digest buffer layout is stage-major:
    /// `[K × residual][K × sign][K × detector][K × consensus]` for a
    /// total of `4 × 32 × K` bytes.
    pub fn dsfb_gpu_run_pipeline_batched_throughput_digests(
        d_features: *mut WindowFeature,
        d_residuals: *mut ResidualCell,
        d_signs: *mut SignCell,
        d_detectors: *mut DetectorCell,
        d_consensus: *mut ConsensusCell,
        d_candidates: *mut CandidateInterval,
        d_candidate_count: *mut i32,
        d_stage_digests: *mut u8,
        h_features: *const WindowFeature,
        n_catalogs: i32,
        n_entities: i32,
        n_windows: i32,
        alpha_q16_raw: i32,
        baseline_latency_us: u32,
        baseline_error_rate_q_raw: i32,
        h_thresholds: *const DetectorThresholdsFfi,
        min_detector_count: i32,
        min_residual_q_raw: i32,
        min_length_windows: i32,
        max_candidates_per_entity: i32,
        h_consensus: *mut ConsensusCell,
        h_candidates: *mut CandidateInterval,
        h_candidate_count_per_entity: *mut i32,
        h_stage_digests: *mut u8,
    ) -> c_int;
}
