//! `GpuWorkspace`: device-side and host-side buffer reuse.
//!
//! Why this exists: the v0 dispatcher (commit `874d075`) ran the full
//! pipeline by `cudaMalloc`-ing seven device buffers, copying inputs in,
//! launching the five kernels, copying every intermediate back, and then
//! `cudaFree`-ing all seven. Per-call host wall time was ~8 ms on the
//! canonical fixture; CUDA-event timings (O.1, commit `926aa28`) showed
//! the kernel-only portion at ~2.4 ms. The 5.6 ms gap was almost entirely
//! per-call allocator traffic (cudaMalloc × 7, Rust `Vec::new` × 6,
//! cudaFree × 7).
//!
//! A `GpuWorkspace` owns:
//!
//! * Seven device pointers — exactly the buffers the pipeline needs.
//!   Allocated once via `dsfb_gpu_workspace_alloc` and freed in `Drop`
//!   via `dsfb_gpu_workspace_free`. Lifetime is tied to the
//!   `GpuWorkspace` so a panic in user code still releases the device
//!   memory.
//! * Six host-side `Vec` buffers for the pipeline's output cells. Sized
//!   once at construction; reused on every dispatch.
//! * The dimensions the workspace was built for (`n_entities`,
//!   `n_windows`). Any call dispatched on this workspace must match.
//!
//! Determinism is unchanged: nothing about the kernel math depends on
//! whether the device buffers came from a fresh `cudaMalloc` or from
//! a reused workspace.

#![cfg(feature = "cuda")]

use core::ffi::c_int;
use std::vec;
use std::vec::Vec;

use dsfb_gpu_debug_core::candidate::{CandidateConfig, CandidateInterval};
use dsfb_gpu_debug_core::consensus::ConsensusCell;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::detector::{DetectorCell, DetectorThresholds};
use dsfb_gpu_debug_core::event::GpuTraceEventCompact;
use dsfb_gpu_debug_core::hash::sha256;
use dsfb_gpu_debug_core::residual::{Baseline, ResidualCell};
use dsfb_gpu_debug_core::sign::SignCell;
use dsfb_gpu_debug_core::window::WindowFeature;

use crate::ffi;
use crate::ffi::DetectorThresholdsFfi;
use crate::GpuError;

/// R.6c — outcome of an attempt to capture the Throughput-digests
/// pipeline into a `cudaGraphExec_t`.
///
/// The graph is **not** semantic — it only records the launch plan
/// (kernel sequence, dependency edges, kernel parameters captured at
/// trace time). The CPU bank module remains the only path that can
/// admit episodes; the GPU still emits evidence and digests. A
/// captured graph trades many small launches for one `cudaGraphLaunch`
/// on the same stream and produces byte-identical case files to the
/// non-graph (R.6b) path.
///
/// Capture can legitimately fail (older driver, stream busy, device
/// does not support graphs in the current mode). On failure the
/// dispatch wrapper demotes to the R.6b async path — that path is
/// the canonical fallback and is already byte-equivalent to the
/// pageable/sync reference.
#[derive(Debug, Clone)]
pub enum GraphCaptureStatus {
    /// Graph capture succeeded. `plan_hash` is the canonical SHA-256
    /// over the captured topology metadata (kernel sequence, scale,
    /// emission mode, layout version). Subsequent launches replay
    /// the captured graph.
    Captured {
        /// Canonical hash of the captured graph's launch plan.
        plan_hash: [u8; 32],
    },
    /// Graph capture did not produce a usable `cudaGraphExec_t`.
    /// `reason` is a short human-readable cause derived from the
    /// raw `cudaError_t`; the dispatch wrapper falls back to the
    /// R.6b async path.
    Demoted {
        /// Short cause string. Mostly for surfacing in case-file
        /// supplementary fields and test diagnostics.
        reason: String,
    },
}

/// R.6c — canonical metadata over which the graph plan hash is
/// computed. Layout is fixed `key=value\n` lines in the order below;
/// changing the order or the field set changes every recorded hash
/// and breaks reproducibility, so the order is part of the contract.
///
/// Deliberately excludes anything dynamic per dispatch: pointer
/// addresses, graph object handles, host wall-clock values. The
/// hash exists to certify "the graph captured here records exactly
/// this launch plan against exactly this scale and mode", not to
/// fingerprint a specific run.
const GRAPH_PLAN_VERSION: &str = "DSFB-GPU-DEBUG-GRAPH-V1";

/// R.6d — process-wide one-shot upload of the canonical
/// `DetectorThresholds` into the device's `__constant__
/// c_detector_thresholds` symbol. Returns `true` if the upload
/// succeeded (or had previously succeeded in this process),
/// `false` if it failed.
///
/// The motivation is correctness, not perf: multiple parallel
/// workspaces calling `cudaMemcpyToSymbol` simultaneously can
/// implicitly synchronize on the default stream and invalidate
/// other threads' active stream captures (R.6c). Because the
/// canonical thresholds are immutable, one upload per process is
/// both correct and sufficient.
fn ensure_canonical_thresholds_uploaded() -> bool {
    use std::sync::OnceLock;
    static UPLOAD_RESULT: OnceLock<bool> = OnceLock::new();
    *UPLOAD_RESULT.get_or_init(|| {
        let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
        #[allow(unsafe_code)]
        let status: core::ffi::c_int = unsafe {
            ffi::dsfb_gpu_upload_detector_thresholds(std::ptr::from_ref(&thresholds_ffi))
        };
        status == 0
    })
}

/// R.6c — compute the canonical graph plan hash for the
/// Throughput-digests pipeline at a given scale. The hash is
/// stable across two captures on the same workspace and changes
/// when n_entities or n_windows change. R.6d added
/// `uses_const_thresholds` to the metadata so the const-memory
/// detector kernel variant counts as a distinct topology — two
/// workspaces whose upload-at-construction outcome differs will
/// then surface different plan hashes.
fn compute_throughput_graph_plan_hash(
    contract: &Contract,
    uses_const_thresholds: bool,
) -> [u8; 32] {
    use core::fmt::Write;
    let mut text = String::with_capacity(512);
    text.push_str("graph_version=");
    text.push_str(GRAPH_PLAN_VERSION);
    text.push('\n');
    text.push_str("emission_mode=Throughput\n");
    text.push_str("numeric_mode=Q16.16\n");
    // `writeln!` into a `String` cannot fail; the trait surface
    // returns `core::fmt::Result` for parity with `Write` impls
    // that can fail, but `String`'s impl is infallible. Drop the
    // result explicitly so clippy is satisfied without an
    // unreachable panic path.
    let _ = writeln!(text, "n_catalogs={}", 1u32);
    let _ = writeln!(text, "n_entities={}", contract.n_entities);
    let _ = writeln!(text, "n_windows={}", contract.n_windows);
    // The captured topology mirrors the R.6b async FFI: H2D, five
    // pipeline kernels (legacy non-fused), four digest kernels,
    // D2H. Fused (R.4) and graph capture are deliberately separate
    // optimisations in R; if a future R subsection wires them
    // together the kernel_sequence string changes and the plan
    // hash with it.
    text.push_str(
        "kernel_sequence=\
         residual_field,drift_slew_sign,detector_motif,\
         consensus_grid,candidate_collapse,\
         digest_residual,digest_sign,digest_detector,digest_consensus\n",
    );
    text.push_str("uses_pre_alpha=false\n");
    text.push_str("uses_fused_residual_sign=false\n");
    text.push_str("uses_device_digests=true\n");
    text.push_str("candidate_layout_version=2\n");
    text.push_str("stage_digest_count=4\n");
    text.push_str(if uses_const_thresholds {
        "uses_const_thresholds=true\n"
    } else {
        "uses_const_thresholds=false\n"
    });
    sha256(text.as_bytes())
}

/// Per-entity candidate-buffer capacity. Mirrors the constant inside
/// `dispatch.rs`; kept here so workspace allocation matches what the
/// kernel expects without a circular module dependency.
pub(crate) const MAX_CANDIDATES_PER_ENTITY: i32 = 16;

/// R.8.5 — chosen tree-digest chunk size in bytes. 16 KiB at v0;
/// tuning this is a future-work lever once the win is measured.
/// Mirrored in `cuda/kernels.cu`'s tree-digest kernels indirectly
/// (the value is passed via FFI), and recorded in the case-file
/// metadata so replay catches a mode-mismatched receipt.
pub(crate) const TREE_DIGEST_CHUNK_SIZE: u32 = 16_384;

/// R.8.5 — header bytes the root kernel writes before the
/// concatenated leaf digests: 18-byte ASCII domain separator
/// "DSFB_STAGE_TREE_V1" + 3 × u32 little-endian metadata
/// (stage_id, chunk_size, chunk_count). Mirrors the layout in
/// `cuda/kernels.cu::tree_digest_root_kernel`.
pub(crate) const TREE_DIGEST_HEADER_BYTES: u64 = 18 + 4 + 4 + 4;

/// R.8.5 / R.9.b.3 — worst-case cell size in bytes across the four
/// staged cell types (residual, sign, detector, consensus). Used to
/// over-size the tree-digest scratch so any stage fits without
/// re-allocation.
///
/// At R.8.5 this was 64 bytes (covering up to `SignCell` at 40
/// bytes with headroom). R.9.b.3 bumps it to 264 to cover the
/// `DetectorCellWide` introduced by the D64 / wider profile path
/// (which feeds bytes into the same tree-digest scratch as the
/// narrow stages). The bump roughly quadruples the scratch
/// allocation for narrow-only workspaces; the cost is small in
/// absolute terms (a few MB at scale-large; trivial at canonical)
/// and avoids needing a parallel wide-only scratch.
pub(crate) const TREE_DIGEST_WORST_CELL_BYTES: u64 = 264;

/// Reusable host + device storage for the deterministic GPU pipeline.
///
/// Lifetimes: the device-side pointers are released in `Drop`, so a
/// workspace can be dropped freely once no more dispatch calls will use
/// it. Host-side `Vec` buffers ride along inside the struct so they are
/// allocated exactly once per workspace.
///
/// Sized for one fixture shape: a workspace built for a 16×128 contract
/// cannot be reused for a 256×1024 contract — `assert_compatible`
/// rejects that mismatch loudly so it never silently produces a corrupt
/// case file. Build a new workspace per scale point.
pub struct GpuWorkspace {
    // Device pointers. `*mut Foo` rather than `NonNull<Foo>` because the
    // foreign workspace_alloc routine writes them through `*mut *mut Foo`.
    d_features: *mut WindowFeature,
    d_residuals: *mut ResidualCell,
    d_signs: *mut SignCell,
    d_detectors: *mut DetectorCell,
    d_consensus: *mut ConsensusCell,
    d_candidates: *mut CandidateInterval,
    d_candidate_count: *mut i32,
    /// 4 × 32-byte slot for the Tier 3B per-stage device digests
    /// (residual, sign, detector, consensus). Allocated lazily via
    /// `dsfb_gpu_alloc_bytes` so the existing typed allocator ABI
    /// is unchanged; freed in `Drop` via `dsfb_gpu_free_bytes`.
    d_stage_digests: *mut u8,
    /// R.4 Pre-Alpha EWMA drift buffer: one i32 per (entity, window)
    /// cell. The fused R+S kernel reads from here instead of carrying
    /// the EWMA recurrence in registers. Allocated lazily as raw
    /// bytes through `dsfb_gpu_alloc_bytes` (size = `n_entities *
    /// n_windows * sizeof(i32)`); freed in `Drop`.
    d_drifts: *mut u8,

    /// R.6b — optional pinned (page-locked) host shadow for the
    /// input `WindowFeature` array. When `Some`, the async dispatch
    /// path computes features directly into this buffer instead of
    /// the dispatch-local Vec, so the H2D `cudaMemcpyAsync` can run
    /// truly asynchronously. `None` on the legacy sync path.
    pub(crate) features_pinned: Option<crate::pinned::PinnedHostBuf<WindowFeature>>,
    /// R.6b — pinned shadow for the `CandidateInterval` output
    /// buffer (D2H). Same shape as `candidate_buf`.
    pub(crate) candidates_pinned: Option<crate::pinned::PinnedHostBuf<CandidateInterval>>,
    /// R.6b — pinned shadow for per-entity `candidate_count` (D2H).
    pub(crate) candidate_count_pinned: Option<crate::pinned::PinnedHostBuf<i32>>,
    /// R.6b — pinned shadow for the 4 × 32-byte stage digest D2H.
    pub(crate) stage_digests_pinned: Option<crate::pinned::PinnedHostBuf<u8>>,
    /// R.6b — opaque CUDA stream handle (0 when no stream was
    /// created). Held as `u64` to keep the field type-system-clean
    /// on the Rust side; the C++ side reinterprets it as
    /// `cudaStream_t`.
    pub(crate) stream: u64,
    /// R.6c — opaque captured `cudaGraphExec_t` (0 when no graph
    /// has been captured for this workspace). When non-zero, the
    /// throughput-graph dispatch can replay the captured topology
    /// instead of issuing kernel + memcpy launches one at a time.
    /// Freed in `Drop` via `dsfb_gpu_destroy_throughput_graph`.
    /// The graph is NOT semantic — it only records the launch
    /// plan. The CPU bank still admits episodes; the GPU still
    /// emits evidence.
    pub(crate) graph_exec: u64,
    /// R.6c — canonical hash over the captured graph's topology
    /// (kernel sequence, scale, mode, layout version). Populated
    /// at capture time and recorded into the case file via the
    /// dispatch wrapper. `None` when no graph has been captured.
    /// The hash deliberately excludes pointer addresses, graph
    /// handles, and wall-clock values so the same topology on the
    /// same workspace yields the same hash run after run.
    pub(crate) graph_plan_hash: Option<[u8; 32]>,
    /// R.6d — true once `dsfb_gpu_upload_detector_thresholds`
    /// successfully populated the device-side `__constant__`
    /// `c_detector_thresholds` symbol from this workspace. When
    /// true, dispatch wrappers pass `use_const_thresholds=1` so
    /// the kernel reads the constant-memory copy; when false,
    /// they fall back to the param-passing variant. Set at
    /// workspace construction; never mutated afterwards (except
    /// by the test-only forcer described in R.6d.2).
    pub(crate) const_thresholds_uploaded: bool,
    /// R.8.5 — per-stage leaf-digest arena for the tree-digest
    /// path. Layout: `4 stages × tree_leaves_stride bytes`, where
    /// `tree_leaves_stride` is sized for the worst-case stage at
    /// the workspace's contract (`SignCell` is the largest cell
    /// type at v0). Each stage's per-catalog leaf digests live at
    /// `[stage_id * tree_leaves_stride .. stage_id *
    /// tree_leaves_stride + n_chunks * 32]`. Allocated when
    /// `tree_chunk_size` is non-zero; null otherwise.
    pub(crate) d_tree_leaves: *mut u8,
    /// R.8.5 — per-stage root-kernel scratch arena. Layout:
    /// `4 stages × tree_scratch_stride bytes`. Each per-stage
    /// region holds the header (30 bytes) concatenated with the
    /// stage's ordered leaf digests before the root SHA-256
    /// finalises. Sized at workspace construction; null when
    /// tree digest is disabled.
    pub(crate) d_tree_scratch: *mut u8,
    /// R.8.5 — chosen tree-digest chunk size in bytes (e.g. 16 384
    /// for 16 KiB). Recorded here so the dispatch wrapper can
    /// pass it to the FFI without re-deriving it from `Contract`.
    /// 0 means the tree-digest scratch buffers are not allocated.
    pub(crate) tree_chunk_size: u32,
    /// R.8.5 — per-stage stride within `d_tree_leaves`. Sized for
    /// the max possible `n_chunks` at the contract's worst-case
    /// stage (`SignCell`).
    pub(crate) tree_leaves_stride_bytes: u64,
    /// R.8.5 — per-stage stride within `d_tree_scratch`. Header
    /// (30 bytes) + max `n_chunks * 32` bytes.
    pub(crate) tree_scratch_stride_bytes: u64,
    /// R.9.b.2 — device-side wide-mask detector buffer.
    /// `n_cells × 264` bytes (`DetectorCellWide` = 264 bytes per
    /// cell with the `[u64; 32]` mask). Allocated lazily by the
    /// first wide-detector dispatch via
    /// `ensure_wide_detector_buffer`; null until then. The legacy
    /// `d_detectors` (12-byte cells) is unchanged and still used
    /// by every D16 dispatch path.
    pub(crate) d_detectors_wide: *mut dsfb_gpu_debug_core::detector::DetectorCellWide,
    /// R.10a — per-window axis-5 grid-sum precompute buffer
    /// (`n_windows × 8` bytes, i64). Filled by
    /// `axis5_grid_sum_kernel_wide` after the wide consensus kernel
    /// and consumed by the wide candidate kernel's flush loop so
    /// that loop becomes O(length) instead of O(length × n_entities).
    /// Allocated lazily by the D64 throughput dispatch via
    /// `ensure_axis5_grid_sum_buffer`; null until then.
    pub(crate) d_axis5_grid_sum: *mut u8,
    /// S-PERF.14 — Pre-Alpha drift EWMA precompute buffer
    /// (`n_entities × n_windows × 4` bytes, i32 per cell).
    /// Filled by `drift_ewma_precompute_kernel` (per-entity-
    /// serial walk that ONLY computes drift, not the full
    /// SignCell) and consumed by
    /// `drift_slew_sign_kernel_cellpar` (cell-parallel one-
    /// thread-per-cell main kernel that produces the SignCell
    /// output). The split is the S-PERF.14 launch-geometry
    /// repair: the legacy monolithic `drift_slew_sign_kernel`
    /// ran 1.6 ms at 2.1% occupancy on the post-S-PERF.13
    /// D64 pipeline (per the S-PERF.ROOF-PREFLIGHT receipt);
    /// the cellpar main kernel exposes 32 768 blocks at
    /// 256×4096 K=1 to the 80-SM device. Allocated lazily by
    /// the D64 throughput dispatch via `ensure_drift_buffer`;
    /// null until then. At 256×4096 K=1 the buffer is 4 MB.
    pub(crate) d_drift_buffer: *mut i32,
    /// R.10b — compact-wide-detector-digest-v1 arena
    /// (`n_cells × 18` bytes). Filled by
    /// `detector_wide_digest_pack_kernel_v1` after the wide
    /// detector kernel; the detector-stage tree digest reads from
    /// here instead of the 264-byte `DetectorCellWide` stride. At
    /// 256×4096 this shrinks the detector-stage digest input from
    /// ~277 MB to ~18 MB. Allocated lazily by the D64 throughput
    /// dispatch via `ensure_detector_digest_compact_buffer`.
    pub(crate) d_detector_digest_compact: *mut u8,
    /// R.10c — per-cell `fired` flag buffer (`n_cells × 1` byte).
    /// Filled by `candidate_fired_kernel_wide`; read by
    /// `candidate_boundary_kernel_wide`. Allows the per-entity
    /// boundary enumerator to walk 1-byte flags instead of 32-byte
    /// `ConsensusCell` records.
    pub(crate) d_candidate_fired: *mut u8,
    /// R.10c — per-entity candidate boundary scratch
    /// (`n_entities × MAX_CANDIDATES_PER_ENTITY × 8` bytes). Each
    /// 8-byte slot carries `(start_w: u32, end_w: u32)`. Filled by
    /// `candidate_boundary_kernel_wide`; read by
    /// `candidate_pack_kernel_wide`.
    pub(crate) d_candidate_boundaries: *mut u8,
    /// S-PERF.14c — per-entity intermediate run-boundary scratch
    /// (`n_entities × MAX_CANDIDATES_PER_ENTITY × 8` bytes). Same
    /// 8-byte `(start_w: u32, end_w: u32)` slot layout as
    /// `d_candidate_boundaries`. Filled by
    /// `candidate_boundary_precompute_kernel` (Pre-Alpha; per-
    /// entity serial walk that ONLY computes the run boundaries +
    /// length / max filter, no publication) and consumed by
    /// `candidate_boundary_cellpar_emit_kernel` (cellpar emit
    /// kernel that copies surviving runs into the legacy slot
    /// table). The split mirrors S-PERF.14a's
    /// `drift_ewma_precompute_kernel` + `drift_slew_sign_kernel_
    /// cellpar` pattern: the legacy monolithic
    /// `candidate_boundary_kernel_wide` ran 286 µs at 2.1 %
    /// occupancy on the post-S-PERF.14b D64 pipeline (per the
    /// post-S-PERF.14b ROOF receipt); the cellpar emit kernel
    /// exposes ≈ 4 096 (entity, slot) threads at 256 × 4 096 K=1
    /// to the 80-SM device. Allocated lazily by the D64 throughput
    /// dispatch via `ensure_candidate_run_buffer`; null until then.
    /// At 256 × 4 096 K=1 the buffer is 32 KB — negligible.
    pub(crate) d_candidate_run_buffer: *mut u8,
    /// S-PERF.14c — per-entity run-count scratch (`n_entities × 4`
    /// bytes). Filled by `candidate_boundary_precompute_kernel`,
    /// which writes the surviving-run count per entity after the
    /// length and max-per-entity filter. Consumed by
    /// `candidate_boundary_cellpar_emit_kernel`'s thread 0 of each
    /// (entity, catalog) block, which copies the count into the
    /// legacy `d_candidate_count` output. Allocated lazily by the
    /// D64 throughput dispatch via `ensure_candidate_run_buffer`;
    /// the two scratch slots are paired and allocated together,
    /// mirroring the R.10c `ensure_candidate_parallel_buffers`
    /// idiom. At 256 × 4 096 K=1 the buffer is 1 KB — trivial.
    pub(crate) d_candidate_run_count: *mut i32,

    /// R.11b — device-side `GpuTraceEventCompact[]` buffer. Sized lazily for
    /// the first event-count seen via `ensure_events_buffer`; can
    /// regrow if a larger event count is presented later. Replaces
    /// the host-side `compute_features` call in the dispatch's
    /// critical path: events are H2D'd here once per dispatch and
    /// `window_feature_kernel_structured` writes `d_features`
    /// on-device.
    pub(crate) d_events: *mut GpuTraceEventCompact,
    /// R.11b — current allocated capacity of `d_events` (number of
    /// `GpuTraceEventCompact` slots, NOT bytes). Tracked so subsequent
    /// dispatches with the same fixture shape skip the
    /// reallocation; a larger fixture triggers free+realloc.
    pub(crate) d_events_capacity: u64,
    /// R.11b — pinned host shadow for `d_events`. Mirrors the
    /// other pinned shadows so the H2D of events runs from
    /// page-locked memory and the async transfer doesn't fall back
    /// to the default-stream synchronisation pageable memory would
    /// force.
    pub(crate) events_pinned: Option<crate::pinned::PinnedHostBuf<GpuTraceEventCompact>>,

    /// Host-side per-cell `ResidualCell` buffer. Length = `n_entities *
    /// n_windows`.
    pub(crate) residuals: Vec<ResidualCell>,
    /// Host-side per-cell `SignCell` buffer.
    pub(crate) signs: Vec<SignCell>,
    /// Host-side per-cell `DetectorCell` buffer.
    pub(crate) detectors: Vec<DetectorCell>,
    /// Host-side per-cell `ConsensusCell` buffer.
    pub(crate) consensus: Vec<ConsensusCell>,
    /// Host-side per-entity candidate slot buffer. Length = `n_entities
    /// * MAX_CANDIDATES_PER_ENTITY`.
    pub(crate) candidate_buf: Vec<CandidateInterval>,
    /// Host-side per-entity candidate count. Length = `n_entities`.
    pub(crate) candidate_count: Vec<i32>,

    /// The number of entities this workspace was sized for.
    pub n_entities: u32,
    /// The number of windows this workspace was sized for.
    pub n_windows: u32,
}

// `GpuWorkspace` carries raw pointers to device memory, which makes the
// auto-derived `Send`/`Sync` impls unsound. The CUDA runtime is
// thread-affinity-aware (a device pointer is only valid in the context
// it was created in), so we conservatively forbid both: a workspace
// must stay on the thread that created it. This restriction matches
// how most CUDA-using Rust crates expose device handles.
//
// We don't `impl !Send for GpuWorkspace` explicitly because Rust's
// auto-trait rules already produce that conclusion for any type
// containing `*mut`.

impl GpuWorkspace {
    /// Allocate device buffers and pre-allocate the host buffers sized
    /// for the contract's `n_entities × n_windows` grid.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::KernelFailed(code)` if any `cudaMalloc`
    /// returns a non-success status. The constructor rolls back any
    /// partial allocations before returning so no device memory is
    /// leaked on failure.
    #[allow(clippy::too_many_lines)]
    pub fn new(contract: &Contract) -> Result<Self, GpuError> {
        let total = (contract.n_entities as usize) * (contract.n_windows as usize);
        let candidate_capacity =
            (contract.n_entities as usize) * (MAX_CANDIDATES_PER_ENTITY as usize);

        let mut d_features: *mut WindowFeature = std::ptr::null_mut();
        let mut d_residuals: *mut ResidualCell = std::ptr::null_mut();
        let mut d_signs: *mut SignCell = std::ptr::null_mut();
        let mut d_detectors: *mut DetectorCell = std::ptr::null_mut();
        let mut d_consensus: *mut ConsensusCell = std::ptr::null_mut();
        let mut d_candidates: *mut CandidateInterval = std::ptr::null_mut();
        let mut d_candidate_count: *mut i32 = std::ptr::null_mut();

        // Safety: every out-pointer is a valid `*mut *mut T` derived
        // from a local; the C side either fills each one with a valid
        // device pointer (and returns 0) or zeroes them all (on
        // failure). The struct never observes an inconsistent state.
        #[allow(unsafe_code)]
        let status: c_int = unsafe {
            ffi::dsfb_gpu_workspace_alloc(
                contract.n_entities as i32,
                contract.n_windows as i32,
                MAX_CANDIDATES_PER_ENTITY,
                std::ptr::from_mut(&mut d_features),
                std::ptr::from_mut(&mut d_residuals),
                std::ptr::from_mut(&mut d_signs),
                std::ptr::from_mut(&mut d_detectors),
                std::ptr::from_mut(&mut d_consensus),
                std::ptr::from_mut(&mut d_candidates),
                std::ptr::from_mut(&mut d_candidate_count),
            )
        };

        if status != 0 {
            return Err(GpuError::KernelFailed(status));
        }

        // Allocate the 4×32 byte digest slot for the Tier 3B path.
        // Same rollback discipline as the typed buffers: on failure
        // free the typed allocations before returning.
        let mut d_stage_digests: *mut u8 = std::ptr::null_mut();
        #[allow(unsafe_code)]
        let dig_status: c_int =
            unsafe { ffi::dsfb_gpu_alloc_bytes(4 * 32, std::ptr::from_mut(&mut d_stage_digests)) };
        if dig_status != 0 {
            #[allow(unsafe_code)]
            unsafe {
                ffi::dsfb_gpu_workspace_free(
                    d_features,
                    d_residuals,
                    d_signs,
                    d_detectors,
                    d_consensus,
                    d_candidates,
                    d_candidate_count,
                );
            }
            return Err(GpuError::KernelFailed(dig_status));
        }

        // R.4 Pre-Alpha drift buffer: i32 per (entity, window).
        // Sized in raw bytes (4 bytes per i32). Allocated through the
        // same generic byte allocator; freed in Drop.
        let drift_bytes = (total as u64) * 4;
        let mut d_drifts: *mut u8 = std::ptr::null_mut();
        #[allow(unsafe_code)]
        let drift_status: c_int =
            unsafe { ffi::dsfb_gpu_alloc_bytes(drift_bytes, std::ptr::from_mut(&mut d_drifts)) };
        if drift_status != 0 {
            #[allow(unsafe_code)]
            unsafe {
                ffi::dsfb_gpu_workspace_free(
                    d_features,
                    d_residuals,
                    d_signs,
                    d_detectors,
                    d_consensus,
                    d_candidates,
                    d_candidate_count,
                );
                ffi::dsfb_gpu_free_bytes(d_stage_digests);
            }
            return Err(GpuError::KernelFailed(drift_status));
        }

        Ok(Self {
            d_features,
            d_residuals,
            d_signs,
            d_detectors,
            d_consensus,
            d_candidates,
            d_candidate_count,
            d_stage_digests,
            d_drifts,
            // R.6b: pinned shadows are opt-in via
            // `new_with_pinned_async`; the legacy `new` constructor
            // leaves them empty and the stream handle at 0.
            features_pinned: None,
            candidates_pinned: None,
            candidate_count_pinned: None,
            stage_digests_pinned: None,
            stream: 0,
            // R.6c: graph_exec and graph_plan_hash stay unset
            // until `try_capture_throughput_graph` succeeds.
            graph_exec: 0,
            graph_plan_hash: None,
            // R.6d: legacy `new()` does not opt into constant-memory
            // thresholds; `new_with_pinned_async()` is the path that
            // attempts the upload and flips this flag on success.
            const_thresholds_uploaded: false,
            // R.8.5: tree-digest scratch is allocated only when
            // `new_with_pinned_async` is called and the contract opts
            // in via chunk_size > 0. Legacy `new()` leaves these
            // null and zero.
            d_tree_leaves: std::ptr::null_mut(),
            d_tree_scratch: std::ptr::null_mut(),
            tree_chunk_size: 0,
            tree_leaves_stride_bytes: 0,
            tree_scratch_stride_bytes: 0,
            // R.9.b.2: wide-detector buffer is lazily allocated on
            // the first wide-dispatch call via
            // `ensure_wide_detector_buffer`. Legacy and pinned-async
            // constructors leave it null; the wide dispatch path
            // populates it only when a caller opts in.
            d_detectors_wide: std::ptr::null_mut(),
            // R.10a: axis-5 grid-sum buffer is lazily allocated on
            // the first D64 throughput dispatch via
            // `ensure_axis5_grid_sum_buffer`. Null until then.
            d_axis5_grid_sum: std::ptr::null_mut(),
            // S-PERF.14: Pre-Alpha drift EWMA precompute buffer
            // is lazily allocated on the first D64 throughput
            // dispatch via `ensure_drift_buffer`. Null until then.
            d_drift_buffer: std::ptr::null_mut(),
            // R.10b: compact-wide-detector-digest-v1 arena is
            // lazily allocated on the first D64 throughput dispatch
            // via `ensure_detector_digest_compact_buffer`. Null
            // until then.
            d_detector_digest_compact: std::ptr::null_mut(),
            // R.10c: parallel-candidate-collapse scratch buffers
            // (fired flags + boundary tuples). Both lazily
            // allocated on the first D64 throughput dispatch via
            // `ensure_candidate_parallel_buffers`.
            d_candidate_fired: std::ptr::null_mut(),
            d_candidate_boundaries: std::ptr::null_mut(),
            // S-PERF.14c: the candidate-boundary Pre-Alpha + cellpar
            // split scratch buffers stay null until the D64 _timed
            // throughput dispatch invokes `ensure_candidate_run_buffer`.
            d_candidate_run_buffer: std::ptr::null_mut(),
            d_candidate_run_count: std::ptr::null_mut(),
            // R.11b: GPU window-feature inputs. Lazily allocated
            // on the first D64 throughput dispatch via
            // `ensure_events_buffer(n_events)`. Capacity grows
            // when a larger event count arrives.
            d_events: std::ptr::null_mut(),
            d_events_capacity: 0,
            events_pinned: None,
            residuals: vec![ResidualCell::default(); total],
            signs: vec![SignCell::default(); total],
            detectors: vec![DetectorCell::default(); total],
            consensus: vec![ConsensusCell::default(); total],
            candidate_buf: vec![CandidateInterval::default(); candidate_capacity],
            candidate_count: vec![0i32; contract.n_entities as usize],
            n_entities: contract.n_entities,
            n_windows: contract.n_windows,
        })
    }

    /// R.6b — construct a workspace with pinned host shadows for
    /// the H2D/D2H buffers and a dedicated CUDA stream. The pinned
    /// shadows are populated by the async dispatch path
    /// (`build_gpu_throughput_pinned_async_on_workspace`) which
    /// uses `cudaMemcpyAsync` on the stream handle. The legacy
    /// pageable buffers and device pointers are allocated exactly
    /// as in `new()`, so any sync dispatch entry can still drive
    /// this workspace.
    ///
    /// Pinned shadow sizes:
    /// * features:       `n_entities * n_windows` × `WindowFeature`
    /// * candidates:     `n_entities * MAX_CANDIDATES_PER_ENTITY` × `CandidateInterval`
    /// * candidate_count: `n_entities` × `i32`
    /// * stage_digests:  `4 * 32` bytes
    ///
    /// All pinned shadows are freed in `Drop` via `cudaFreeHost`,
    /// and the stream is destroyed via `cudaStreamDestroy`.
    ///
    /// # Errors
    ///
    /// * `GpuError::KernelFailed(code)` if any device alloc, pinned
    ///   host alloc, or `cudaStreamCreate` returns a non-zero CUDA
    ///   error. The constructor rolls back any partial allocations
    ///   before returning.
    pub fn new_with_pinned_async(contract: &Contract) -> Result<Self, GpuError> {
        let mut ws = Self::new(contract)?;

        let total = (contract.n_entities as usize) * (contract.n_windows as usize);
        let candidate_capacity =
            (contract.n_entities as usize) * (MAX_CANDIDATES_PER_ENTITY as usize);

        // Pinned host shadows. `PinnedHostBuf::new` zero-initialises
        // the storage via `T::default()` — every cell type is
        // `Copy + Default`. Failures roll back via `ws` going out of
        // scope (its `Drop` releases everything allocated so far).
        ws.features_pinned = Some(crate::pinned::PinnedHostBuf::new(total)?);
        ws.candidates_pinned = Some(crate::pinned::PinnedHostBuf::new(candidate_capacity)?);
        ws.candidate_count_pinned = Some(crate::pinned::PinnedHostBuf::new(
            contract.n_entities as usize,
        )?);
        ws.stage_digests_pinned = Some(crate::pinned::PinnedHostBuf::new(4 * 32)?);

        // Stream creation. On failure the partial workspace drops
        // cleanly because the pinned shadows are normal Rust types.
        let mut stream_handle: u64 = 0;
        #[allow(unsafe_code)]
        let status: c_int =
            unsafe { ffi::dsfb_gpu_create_stream(std::ptr::from_mut(&mut stream_handle)) };
        if status != 0 {
            return Err(GpuError::KernelFailed(status));
        }
        ws.stream = stream_handle;

        // R.6d — opt-in constant-memory threshold upload. The
        // dispatch wrappers below will check `const_thresholds_uploaded`
        // and pass `use_const_thresholds=1` to the FFI on success.
        // If the upload fails (extremely rare on a working CUDA
        // context), the flag stays false and the param-passing
        // kernel variant runs — byte-equivalent fallback.
        //
        // The upload is gated behind a process-wide `OnceLock` so
        // multiple workspaces constructed in parallel (e.g. cargo
        // test) only upload once. cudaMemcpyToSymbol is
        // synchronous on the default stream — running it from N
        // parallel threads simultaneously would serialize and
        // could invalidate concurrent stream captures from other
        // tests. The values are canonical and immutable, so one
        // upload per process is correct.
        ws.const_thresholds_uploaded = ensure_canonical_thresholds_uploaded();

        // R.8.5 — tree-digest scratch arenas. Sized for the
        // largest cell type (`SignCell`, currently 40 bytes per
        // cell — the contract bounds `n_cells`). Chunk size is
        // locked to 16 KiB at v0; tunable later if a different
        // value produces a measurable win.
        ws.tree_chunk_size = TREE_DIGEST_CHUNK_SIZE;
        let n_cells = (contract.n_entities as u64) * (contract.n_windows as u64);
        let worst_stage_cell_bytes: u64 = TREE_DIGEST_WORST_CELL_BYTES;
        let worst_stage_bytes = n_cells * worst_stage_cell_bytes;
        let worst_n_chunks = worst_stage_bytes
            .div_ceil(u64::from(ws.tree_chunk_size))
            .max(1);
        ws.tree_leaves_stride_bytes = worst_n_chunks * 32;
        // Root-kernel header: 18-byte domain string + 3 × u32 LE
        // fields (stage_id, chunk_size, chunk_count) = 30 bytes.
        ws.tree_scratch_stride_bytes = TREE_DIGEST_HEADER_BYTES + worst_n_chunks * 32;
        let total_leaves_bytes = 4 * ws.tree_leaves_stride_bytes;
        let total_scratch_bytes = 4 * ws.tree_scratch_stride_bytes;
        let mut d_tree_leaves: *mut u8 = std::ptr::null_mut();
        #[allow(unsafe_code)]
        let leaves_status: c_int = unsafe {
            ffi::dsfb_gpu_alloc_bytes(total_leaves_bytes, std::ptr::from_mut(&mut d_tree_leaves))
        };
        if leaves_status != 0 {
            return Err(GpuError::KernelFailed(leaves_status));
        }
        ws.d_tree_leaves = d_tree_leaves;
        let mut d_tree_scratch: *mut u8 = std::ptr::null_mut();
        #[allow(unsafe_code)]
        let scratch_status: c_int = unsafe {
            ffi::dsfb_gpu_alloc_bytes(total_scratch_bytes, std::ptr::from_mut(&mut d_tree_scratch))
        };
        if scratch_status != 0 {
            return Err(GpuError::KernelFailed(scratch_status));
        }
        ws.d_tree_scratch = d_tree_scratch;

        Ok(ws)
    }

    /// Sanity-check that a dispatch on this workspace is using the same
    /// dimensions the workspace was allocated for. Returns an error if
    /// the contract mismatches.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::InvalidInput` when the contract's
    /// `n_entities` or `n_windows` differ from the values the workspace
    /// was built for. This prevents a workspace sized for one fixture
    /// shape from silently emitting garbage on a differently-shaped
    /// fixture.
    pub fn assert_compatible(&self, contract: &Contract) -> Result<(), GpuError> {
        if contract.n_entities == self.n_entities && contract.n_windows == self.n_windows {
            Ok(())
        } else {
            Err(GpuError::InvalidInput(
                "GpuWorkspace dimensions do not match the supplied contract",
            ))
        }
    }

    // Internal accessors used by `dispatch.rs` so the device pointers
    // never leak into safe public Rust.
    pub(crate) fn d_features(&self) -> *mut WindowFeature {
        self.d_features
    }
    pub(crate) fn d_residuals(&self) -> *mut ResidualCell {
        self.d_residuals
    }
    pub(crate) fn d_signs(&self) -> *mut SignCell {
        self.d_signs
    }
    pub(crate) fn d_detectors(&self) -> *mut DetectorCell {
        self.d_detectors
    }
    pub(crate) fn d_consensus(&self) -> *mut ConsensusCell {
        self.d_consensus
    }
    pub(crate) fn d_candidates(&self) -> *mut CandidateInterval {
        self.d_candidates
    }
    pub(crate) fn d_candidate_count(&self) -> *mut i32 {
        self.d_candidate_count
    }
    pub(crate) fn d_stage_digests(&self) -> *mut u8 {
        self.d_stage_digests
    }
    /// R.4 Pre-Alpha drift buffer. Cast on the C ABI side to
    /// `int32_t*`.
    pub(crate) fn d_drifts(&self) -> *mut u8 {
        self.d_drifts
    }

    /// R.6b — is this workspace configured for pinned/async
    /// dispatch? Returns `true` only when all four pinned shadows
    /// are populated AND a non-zero stream handle is held. The
    /// async dispatch entry checks this gate before attempting an
    /// async path.
    #[must_use]
    pub fn has_pinned_async(&self) -> bool {
        self.features_pinned.is_some()
            && self.candidates_pinned.is_some()
            && self.candidate_count_pinned.is_some()
            && self.stage_digests_pinned.is_some()
            && self.stream != 0
    }

    /// R.6b — opaque CUDA stream handle (0 when none). Round-trips
    /// through the async FFI; the Rust side never dereferences it.
    pub(crate) fn stream_handle(&self) -> u64 {
        self.stream
    }

    /// R.8.5 — accessor for the per-workspace tree-digest leaves
    /// arena. Null until `new_with_pinned_async` allocates it; the
    /// dispatch path checks `tree_chunk_size != 0` before using it.
    pub(crate) fn d_tree_leaves(&self) -> *mut u8 {
        self.d_tree_leaves
    }

    /// R.8.5 — accessor for the per-workspace tree-digest root
    /// scratch arena. Null until `new_with_pinned_async` allocates
    /// it.
    pub(crate) fn d_tree_scratch(&self) -> *mut u8 {
        self.d_tree_scratch
    }

    /// R.8.5 — chosen chunk size in bytes for this workspace. The
    /// canonical contract field — recorded in the case-file
    /// metadata when the tree-digest dispatch is used.
    #[must_use]
    pub fn tree_chunk_size(&self) -> u32 {
        self.tree_chunk_size
    }

    /// R.8.5 — per-stage stride in `d_tree_leaves`. Passed to the
    /// FFI so the kernel knows how to offset its leaf writes per
    /// stage (4 stages share one allocation).
    pub(crate) fn tree_leaves_stride_bytes(&self) -> u64 {
        self.tree_leaves_stride_bytes
    }

    /// R.8.5 — per-stage stride in `d_tree_scratch`. Passed to the
    /// FFI so the root kernel knows how to offset its concatenation
    /// scratch per stage.
    pub(crate) fn tree_scratch_stride_bytes(&self) -> u64 {
        self.tree_scratch_stride_bytes
    }

    /// R.8.5 — `true` when the tree-digest scratch arenas are
    /// allocated and the dispatch can use the parallel tree-digest
    /// path. Currently true whenever `new_with_pinned_async`
    /// succeeded; opt-out is a follow-up if a workspace ever needs
    /// the serial digest only.
    #[must_use]
    pub fn has_tree_digest(&self) -> bool {
        self.tree_chunk_size != 0 && !self.d_tree_leaves.is_null() && !self.d_tree_scratch.is_null()
    }

    /// S-PERF.11 — read the four `TreeSha256V1` stage-root digests
    /// (residual / sign / detector / consensus) that the D64
    /// throughput dispatcher most recently wrote into the pinned
    /// shadow.
    ///
    /// Returns `None` when no D64 throughput dispatch has populated
    /// the pinned `stage_digests` buffer yet (the buffer is
    /// zero-initialised at workspace construction, so a caller that
    /// reads before dispatch sees all zeros — the option wrapper
    /// only signals the buffer is absent, not the values).
    ///
    /// WHY this accessor exists: the
    /// `s_perf_11_pre_rewrite_root_capture` acceptance test needs to
    /// pin the four pre-rewrite root digests as `[u8; 32]`
    /// constants. The post-rewrite kernel
    /// (`tree_digest_leaf_kernel_v2`) MUST produce byte-identical
    /// roots — that is the `same_mode_digest_root_law` enforcement
    /// surface for S-PERF.11. Without this accessor, the test would
    /// have to reach into `pub(crate) stage_digests_pinned`, which
    /// integration tests cannot see.
    #[must_use]
    pub fn last_d64_stage_root_digests(&self) -> Option<[[u8; 32]; 4]> {
        let pinned = self.stage_digests_pinned.as_ref()?;
        let bytes = pinned.as_slice();
        if bytes.len() < 4 * 32 {
            return None;
        }
        let mut out = [[0u8; 32]; 4];
        for (i, slot) in out.iter_mut().enumerate() {
            slot.copy_from_slice(&bytes[i * 32..(i + 1) * 32]);
        }
        Some(out)
    }

    /// S-PERF.15.a Step 0 — D2H copy of the entire
    /// `d_detectors_wide` arena (`n_entities × n_windows ×
    /// n_catalogs × 264` bytes; 540 KB at canonical 16×128 K=1).
    /// Used by the panel-locked pre-fusion byte-capture harness to
    /// SHA-256 the post-dispatch wide-detector bytes and pin them
    /// as `PINNED_PRE_S_PERF_15_A_DETECTOR_CELLWIDE_ARENA_SHA256`.
    /// The fused-kernel byte-identity assertion then re-hashes the
    /// post-fusion arena and asserts equality against the pinned
    /// digest. Returns `None` if the wide-detector buffer was
    /// never allocated (i.e., no D64 / D128 / D205 wide-detector
    /// dispatch has run on this workspace yet).
    ///
    /// # Errors
    ///
    /// Returns `Err(GpuError::KernelFailed(code))` if the
    /// underlying `cudaMemcpy` fails. The host buffer is sized
    /// from the workspace's contract dimensions; callers should
    /// only invoke this immediately after a D64 dispatch.
    #[must_use = "the captured arena bytes are the test's load-bearing input"]
    #[allow(unsafe_code)]
    pub fn last_d64_detector_wide_arena_bytes(&self) -> Option<Result<Vec<u8>, GpuError>> {
        if self.d_detectors_wide.is_null() {
            return None;
        }
        let cell_size_bytes: usize =
            std::mem::size_of::<dsfb_gpu_debug_core::detector::DetectorCellWide>();
        let n_cells = (self.n_entities as usize) * (self.n_windows as usize);
        let total_bytes = n_cells * cell_size_bytes;
        let mut host = vec![0u8; total_bytes];
        // Safety: FFI requires raw pointers; total_bytes was
        // computed from the workspace's allocation dimensions, so
        // the destination buffer is large enough. The CUDA runtime
        // performs the bounds check on the device side.
        let status: c_int = unsafe {
            ffi::dsfb_gpu_memcpy_d2h_bytes(
                self.d_detectors_wide.cast::<u8>(),
                host.as_mut_ptr(),
                total_bytes as u64,
            )
        };
        if status != 0 {
            return Some(Err(GpuError::KernelFailed(status)));
        }
        Some(Ok(host))
    }

    /// S-PERF.15.b Step 0 — D2H copy of the entire
    /// `d_consensus` arena (`n_entities × n_windows ×
    /// n_catalogs × sizeof(ConsensusCell)` bytes; ~32 KB at
    /// canonical 16×128 K=1, ~32 MB at full 256×4096). Used by
    /// the panel-locked pre-fusion byte-capture harness to
    /// SHA-256 the post-dispatch ConsensusCell bytes and pin
    /// them as `PINNED_PRE_S_PERF_15_B_CONSENSUS_ARENA_SHA256`.
    /// The post-S-PERF.15.b 2-stage fused kernel MUST produce
    /// byte-identical ConsensusCell output.
    ///
    /// # Errors
    ///
    /// Returns `Err(GpuError::KernelFailed(code))` if `cudaMemcpy` fails.
    #[must_use = "the captured arena bytes are the test's load-bearing input"]
    #[allow(unsafe_code)]
    pub fn last_d64_consensus_arena_bytes(&self) -> Option<Result<Vec<u8>, GpuError>> {
        if self.d_consensus.is_null() {
            return None;
        }
        let cell_size_bytes: usize =
            std::mem::size_of::<dsfb_gpu_debug_core::consensus::ConsensusCell>();
        let n_cells = (self.n_entities as usize) * (self.n_windows as usize);
        let total_bytes = n_cells * cell_size_bytes;
        let mut host = vec![0u8; total_bytes];
        // Safety: total_bytes is derived from the workspace's
        // contract dimensions; the destination is large enough.
        let status: c_int = unsafe {
            ffi::dsfb_gpu_memcpy_d2h_bytes(
                self.d_consensus.cast::<u8>(),
                host.as_mut_ptr(),
                total_bytes as u64,
            )
        };
        if status != 0 {
            return Some(Err(GpuError::KernelFailed(status)));
        }
        Some(Ok(host))
    }

    /// S-PERF.15.b Step 0 — D2H copy of the entire
    /// `d_axis5_grid_sum` arena (`n_windows × n_catalogs × 8`
    /// bytes; 1 KB at canonical 16×128 K=1, 32 KB at full
    /// 256×4096). Used by the pre-fusion harness to pin the
    /// axis5 i64 per-window sums under
    /// `PINNED_PRE_S_PERF_15_B_AXIS5_GRID_SUM_ARENA_SHA256`.
    /// The 2-stage fused kernel pair MUST produce a
    /// byte-identical i64 sum stream (associativity of i64
    /// addition + canonical block-ascending merge guarantees
    /// this).
    ///
    /// # Errors
    ///
    /// Returns `Err(GpuError::KernelFailed(code))` if `cudaMemcpy` fails.
    #[must_use = "the captured arena bytes are the test's load-bearing input"]
    #[allow(unsafe_code)]
    pub fn last_d64_axis5_grid_sum_bytes(&self) -> Option<Result<Vec<u8>, GpuError>> {
        if self.d_axis5_grid_sum.is_null() {
            return None;
        }
        let total_bytes = (self.n_windows as usize) * 8;
        let mut host = vec![0u8; total_bytes];
        // Safety: total_bytes = n_windows × 8 matches the
        // workspace's d_axis5_grid_sum allocation (sized per
        // contract dimensions). The host buffer is large enough.
        let status: c_int = unsafe {
            ffi::dsfb_gpu_memcpy_d2h_bytes(
                self.d_axis5_grid_sum,
                host.as_mut_ptr(),
                total_bytes as u64,
            )
        };
        if status != 0 {
            return Some(Err(GpuError::KernelFailed(status)));
        }
        Some(Ok(host))
    }

    /// S-PERF.15.b Step 0 — D2H copy of the entire
    /// `d_candidate_fired` arena (`n_entities × n_windows ×
    /// n_catalogs × 1` byte; 2 KB at canonical, 1 MB at full
    /// scale). Used by the pre-fusion harness to pin the
    /// downstream cascade input under
    /// `PINNED_PRE_S_PERF_15_B_CANDIDATE_FIRED_ARENA_SHA256`.
    /// The 2-stage fused kernel pair's downstream
    /// candidate_fired bytes MUST be byte-identical or the
    /// cascade has drifted upstream.
    ///
    /// # Errors
    ///
    /// Returns `Err(GpuError::KernelFailed(code))` if `cudaMemcpy` fails.
    #[must_use = "the captured arena bytes are the test's load-bearing input"]
    #[allow(unsafe_code)]
    pub fn last_d64_candidate_fired_arena_bytes(&self) -> Option<Result<Vec<u8>, GpuError>> {
        if self.d_candidate_fired.is_null() {
            return None;
        }
        let n_cells = (self.n_entities as usize) * (self.n_windows as usize);
        let total_bytes = n_cells;
        let mut host = vec![0u8; total_bytes];
        // Safety: total_bytes = n_cells × 1 matches the workspace's
        // d_candidate_fired allocation; the host buffer is large
        // enough.
        let status: c_int = unsafe {
            ffi::dsfb_gpu_memcpy_d2h_bytes(
                self.d_candidate_fired,
                host.as_mut_ptr(),
                total_bytes as u64,
            )
        };
        if status != 0 {
            return Some(Err(GpuError::KernelFailed(status)));
        }
        Some(Ok(host))
    }

    /// S-PERF.15.c Step 0 — D2H copy of the entire
    /// `d_candidates` arena
    /// (`n_entities × MAX_CANDIDATES_PER_ENTITY × n_catalogs ×
    /// sizeof::<CandidateInterval>` bytes; 12 288 bytes at
    /// canonical 16×16 max_per_entity × 1 catalog with 48-byte
    /// CandidateInterval). Used by the panel-locked pre-rewrite
    /// byte-capture harness to SHA-256 the post-dispatch
    /// `CandidateInterval` arena bytes and pin them as
    /// `PINNED_PRE_S_PERF_15_C_CANDIDATE_PACK_BYTES`. The
    /// post-S-PERF.15.c `candidate_pack_kernel_wide_blockcoop`
    /// MUST produce byte-identical bytes here or this pin fires.
    ///
    /// # Errors
    ///
    /// Returns `Err(GpuError::KernelFailed(code))` if `cudaMemcpy` fails.
    #[must_use = "the captured arena bytes are the test's load-bearing input"]
    #[allow(unsafe_code)]
    pub fn last_d64_candidates_arena_bytes(&self) -> Option<Result<Vec<u8>, GpuError>> {
        if self.d_candidates.is_null() {
            return None;
        }
        let n_slots = (self.n_entities as usize) * (MAX_CANDIDATES_PER_ENTITY as usize);
        let total_bytes = n_slots * core::mem::size_of::<CandidateInterval>();
        let mut host = vec![0u8; total_bytes];
        // Safety: total_bytes = n_slots × sizeof(CandidateInterval)
        // matches the workspace's d_candidates allocation in new();
        // the host buffer is large enough.
        let status: c_int = unsafe {
            ffi::dsfb_gpu_memcpy_d2h_bytes(
                self.d_candidates.cast::<u8>(),
                host.as_mut_ptr(),
                total_bytes as u64,
            )
        };
        if status != 0 {
            return Some(Err(GpuError::KernelFailed(status)));
        }
        Some(Ok(host))
    }

    /// S-PERF.15.c Step 0 — D2H copy of the entire
    /// `d_candidate_count` arena
    /// (`n_entities × n_catalogs × sizeof::<i32>` bytes; 64 bytes
    /// at canonical 16 entities × 1 catalog). Used by the
    /// panel-locked pre-rewrite byte-capture harness to SHA-256
    /// the upstream `candidate_boundary` cascade's per-entity
    /// count output and pin it as
    /// `PINNED_PRE_S_PERF_15_C_CANDIDATE_COUNT_BYTES`. The
    /// `candidate_pack_kernel_wide_blockcoop` READS this buffer
    /// but does not write it; pinning is defense-in-depth that
    /// the upstream cascade (candidate_boundary + candidate_fired)
    /// remains byte-stable across the S-PERF.15.c swap.
    ///
    /// # Errors
    ///
    /// Returns `Err(GpuError::KernelFailed(code))` if `cudaMemcpy` fails.
    #[must_use = "the captured arena bytes are the test's load-bearing input"]
    #[allow(unsafe_code)]
    pub fn last_d64_candidate_count_arena_bytes(&self) -> Option<Result<Vec<u8>, GpuError>> {
        if self.d_candidate_count.is_null() {
            return None;
        }
        let n_entries = self.n_entities as usize;
        let total_bytes = n_entries * core::mem::size_of::<i32>();
        let mut host = vec![0u8; total_bytes];
        // Safety: total_bytes = n_entries × sizeof(i32) matches
        // the workspace's d_candidate_count allocation in new();
        // the host buffer is large enough.
        let status: c_int = unsafe {
            ffi::dsfb_gpu_memcpy_d2h_bytes(
                self.d_candidate_count.cast::<u8>(),
                host.as_mut_ptr(),
                total_bytes as u64,
            )
        };
        if status != 0 {
            return Some(Err(GpuError::KernelFailed(status)));
        }
        Some(Ok(host))
    }

    /// S-PERF.15.a Step 0 — D2H copy of the entire
    /// `d_detector_digest_compact` arena
    /// (`n_entities × n_windows × n_catalogs × 18` bytes; 36 KB
    /// at canonical 16×128 K=1). Used by the panel-locked
    /// pre-fusion byte-capture harness to SHA-256 the
    /// post-dispatch 18-byte-compact pack and pin it as
    /// `PINNED_PRE_S_PERF_15_A_DETECTOR_COMPACT_PACK_ARENA_SHA256`.
    /// Returns `None` if the compact-pack buffer was never
    /// allocated (i.e., no D64 _timed dispatch has run via the
    /// `ensure_detector_digest_compact_buffer` path).
    ///
    /// # Errors
    ///
    /// Returns `Err(GpuError::KernelFailed(code))` if the
    /// underlying `cudaMemcpy` fails.
    #[must_use = "the captured arena bytes are the test's load-bearing input"]
    #[allow(unsafe_code)]
    pub fn last_d64_detector_compact_pack_arena_bytes(&self) -> Option<Result<Vec<u8>, GpuError>> {
        // R.10b pinned compact-record size = 18 bytes per cell.
        // Declared at the top of the function (before the
        // null-check return) so the const lives at the start of
        // the scope per clippy's `items_after_statements` rule.
        const COMPACT_BYTES_PER_CELL: usize = 18;
        if self.d_detector_digest_compact.is_null() {
            return None;
        }
        let n_cells = (self.n_entities as usize) * (self.n_windows as usize);
        let total_bytes = n_cells * COMPACT_BYTES_PER_CELL;
        let mut host = vec![0u8; total_bytes];
        // Safety: FFI requires raw pointers; total_bytes was
        // computed from the workspace's contract dimensions.
        let status: c_int = unsafe {
            ffi::dsfb_gpu_memcpy_d2h_bytes(
                self.d_detector_digest_compact,
                host.as_mut_ptr(),
                total_bytes as u64,
            )
        };
        if status != 0 {
            return Some(Err(GpuError::KernelFailed(status)));
        }
        Some(Ok(host))
    }

    /// R.9.b.2 — accessor for the lazily-allocated wide-detector
    /// device buffer. Returns null until
    /// `ensure_wide_detector_buffer` has succeeded at least once
    /// on this workspace.
    pub(crate) fn d_detectors_wide(&self) -> *mut dsfb_gpu_debug_core::detector::DetectorCellWide {
        self.d_detectors_wide
    }

    /// R.9.b.2 — lazily allocate the wide-detector device buffer
    /// for this workspace's contract dimensions. Idempotent: a
    /// second call on a workspace that already has the buffer is
    /// a no-op. Sized for one catalog at the workspace's contract;
    /// K > 1 batched wide dispatch is R.9.c+ work.
    ///
    /// **S-PERF.15.d Direction A.1 zero-init contract** (panel-
    /// locked 2026-05-18 post-Step-0d byte-counter trace): the
    /// freshly-allocated `d_detectors_wide` buffer is immediately
    /// zero-initialised via `cudaMemset`. This is load-bearing
    /// for the post-S-PERF.15.d rewrite of
    /// `detector_motif_fused_d64_kernel`, which skips writing the
    /// cold `mask[1..31]` lanes (248 B per cell at D64) per
    /// dispatch. Because those bytes start as stable zero from
    /// this one-time init and are never written by the rewritten
    /// kernel, the full `DetectorCellWide` arena
    /// SHA-256 (`PINNED_PRE_S_PERF_15_D_DETECTOR_CELLWIDE_ARENA_SHA256`)
    /// remains byte-identical across the workspace's lifetime —
    /// preserving Pin 1 of the S-PERF.15.d Step 0 contract without
    /// requiring a contract rebaseline. The per-dispatch
    /// wide-arena write traffic drops 264 B → 16 B per cell
    /// (16.5x per-cell, ~2.6x total DRAM write per Step 0d
    /// measurement) while the byte-identity contract holds.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::KernelFailed(code)` if `cudaMalloc`
    /// refuses (e.g. OOM at 270 MB for 256×4096), or if the
    /// post-allocation `cudaMemset` fails (S-PERF.15.d
    /// zero-init). The dispatch caller can then honestly skip
    /// the wide row.
    pub fn ensure_wide_detector_buffer(&mut self) -> Result<(), GpuError> {
        if !self.d_detectors_wide.is_null() {
            return Ok(());
        }
        let n_cells = u64::from(self.n_entities) * u64::from(self.n_windows);
        let bytes = n_cells
            * (core::mem::size_of::<dsfb_gpu_debug_core::detector::DetectorCellWide>() as u64);
        let mut ptr: *mut u8 = std::ptr::null_mut();
        // Safety: the FFI writes a valid device pointer or returns
        // non-zero. We never dereference the byte pointer; it's
        // immediately cast to the typed wide-detector pointer.
        #[allow(unsafe_code)]
        let status: c_int =
            unsafe { ffi::dsfb_gpu_alloc_bytes(bytes, std::ptr::from_mut(&mut ptr)) };
        if status != 0 {
            return Err(GpuError::KernelFailed(status));
        }
        // S-PERF.15.d Direction A.1 one-time zero-init. The
        // rewritten detector_motif_fused_d64_kernel relies on
        // mask[1..31] starting as zero and staying zero (since the
        // kernel skips those writes per dispatch). Without this
        // memset, the cold lanes would contain whatever cudaMalloc
        // returned (typically zero on a fresh allocation, but
        // implementation-defined) and Pin 1
        // (PINNED_PRE_S_PERF_15_D_DETECTOR_CELLWIDE_ARENA_SHA256)
        // would NOT be byte-identical to the pre-rewrite legacy
        // capture.
        //
        // Safety: `ptr` is a valid device byte pointer of length
        // `bytes` per the just-succeeded alloc. cudaMemset is
        // documented to write `bytes` bytes starting at `ptr`.
        #[allow(unsafe_code)]
        let memset_status: c_int = unsafe { ffi::dsfb_gpu_memset_bytes(ptr, 0, bytes) };
        if memset_status != 0 {
            // Free the freshly-allocated buffer before propagating
            // the error so the workspace doesn't leak a
            // half-initialised wide arena.
            #[allow(unsafe_code)]
            unsafe {
                ffi::dsfb_gpu_free_bytes(ptr);
            }
            return Err(GpuError::KernelFailed(memset_status));
        }
        // The allocator returns 256-byte aligned device memory
        // (cudaMalloc guarantees ≥ 256-byte alignment); the cast
        // is safe in practice, hence the localised allow.
        #[allow(clippy::cast_ptr_alignment)]
        let typed = ptr.cast::<dsfb_gpu_debug_core::detector::DetectorCellWide>();
        self.d_detectors_wide = typed;
        Ok(())
    }

    /// R.9.b.2 — `true` after `ensure_wide_detector_buffer` has
    /// allocated the wide buffer.
    #[must_use]
    pub fn has_wide_detector_buffer(&self) -> bool {
        !self.d_detectors_wide.is_null()
    }

    /// R.10a — accessor for the lazily-allocated axis-5 grid-sum
    /// buffer used by the D64 throughput pipeline. Returns null
    /// until `ensure_axis5_grid_sum_buffer` has succeeded at least
    /// once on this workspace.
    pub(crate) fn d_axis5_grid_sum(&self) -> *mut u8 {
        self.d_axis5_grid_sum
    }

    /// R.10a — lazily allocate the axis-5 grid-sum device buffer
    /// for this workspace's contract dimensions. Idempotent. The
    /// buffer holds `n_windows × 8` bytes of i64 per-window sums of
    /// `axis7_consensus_q` across entities, written by
    /// `axis5_grid_sum_kernel_wide` and consumed by the wide
    /// candidate-collapse kernel's flush loop.
    ///
    /// At the 256×4096 K=1 scale-large point the buffer is 32 KB —
    /// negligible — so this allocator never fails in practice.
    /// Returned as `GpuError::KernelFailed(code)` when it does.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::KernelFailed(code)` if `cudaMalloc`
    /// refuses.
    pub fn ensure_axis5_grid_sum_buffer(&mut self) -> Result<(), GpuError> {
        if !self.d_axis5_grid_sum.is_null() {
            return Ok(());
        }
        // 8 bytes per window for i64 per-window grid-sum. K > 1
        // batched D64 throughput is R.9.d+ work; one catalog here.
        let bytes = u64::from(self.n_windows) * 8;
        let mut ptr: *mut u8 = std::ptr::null_mut();
        // Safety: the FFI writes a valid device pointer or returns
        // non-zero. We never dereference the raw bytes from Rust.
        #[allow(unsafe_code)]
        let status: c_int =
            unsafe { ffi::dsfb_gpu_alloc_bytes(bytes, std::ptr::from_mut(&mut ptr)) };
        if status != 0 {
            return Err(GpuError::KernelFailed(status));
        }
        self.d_axis5_grid_sum = ptr;
        Ok(())
    }

    /// R.10a — `true` after `ensure_axis5_grid_sum_buffer` has
    /// allocated the buffer. Used by the D64 throughput dispatch
    /// to gate the FFI call.
    #[must_use]
    pub fn has_axis5_grid_sum_buffer(&self) -> bool {
        !self.d_axis5_grid_sum.is_null()
    }

    /// S-PERF.14 — accessor for the lazily-allocated Pre-Alpha
    /// drift EWMA precompute buffer. Returns null until
    /// `ensure_drift_buffer` has succeeded at least once on
    /// this workspace.
    pub(crate) fn d_drift_buffer(&self) -> *mut i32 {
        self.d_drift_buffer
    }

    /// S-PERF.14 — lazily allocate the drift EWMA precompute
    /// device buffer for this workspace's contract dimensions.
    /// Idempotent. The buffer holds `n_entities × n_windows × 4`
    /// bytes of i32 per-cell drift state, written by
    /// `drift_ewma_precompute_kernel` (the Pre-Alpha kernel)
    /// and consumed by `drift_slew_sign_kernel_cellpar` (the
    /// cell-parallel main SignCell-producing kernel).
    ///
    /// At the 256×4096 K=1 scale-large point the buffer is
    /// 4 MB — modest. K > 1 batched D64 throughput is R.9.d+
    /// work; this allocator sizes only for one catalog.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::KernelFailed(code)` if `cudaMalloc`
    /// refuses.
    pub fn ensure_drift_buffer(&mut self) -> Result<(), GpuError> {
        if !self.d_drift_buffer.is_null() {
            return Ok(());
        }
        // 4 bytes per cell (i32 drift) × n_entities × n_windows.
        let bytes = u64::from(self.n_entities) * u64::from(self.n_windows) * 4;
        let mut ptr: *mut u8 = std::ptr::null_mut();
        // Safety: the FFI writes a valid device pointer or returns
        // non-zero. We never dereference the raw bytes from Rust.
        #[allow(unsafe_code)]
        let status: c_int =
            unsafe { ffi::dsfb_gpu_alloc_bytes(bytes, std::ptr::from_mut(&mut ptr)) };
        if status != 0 {
            return Err(GpuError::KernelFailed(status));
        }
        #[allow(clippy::cast_ptr_alignment)]
        let typed = ptr.cast::<i32>();
        self.d_drift_buffer = typed;
        Ok(())
    }

    /// S-PERF.14 — `true` after `ensure_drift_buffer` has
    /// allocated the buffer. Used by the D64 throughput dispatch
    /// to gate the FFI call.
    #[must_use]
    pub fn has_drift_buffer(&self) -> bool {
        !self.d_drift_buffer.is_null()
    }

    /// R.10b — accessor for the compact-wide-detector-digest-v1
    /// device arena. Returns null until
    /// `ensure_detector_digest_compact_buffer` has succeeded.
    pub(crate) fn d_detector_digest_compact(&self) -> *mut u8 {
        self.d_detector_digest_compact
    }

    /// R.10b — fixed byte width of one compact-wide-detector-digest-v1
    /// record. Must agree with the kernel-side
    /// `DETECTOR_WIDE_DIGEST_COMPACT_V1_BYTES`. Surfaced here so the
    /// workspace allocator does not silently drift if the v1 layout
    /// is ever extended (a future v2 ships a different constant + a
    /// different pack kernel).
    pub(crate) const DETECTOR_WIDE_DIGEST_COMPACT_V1_BYTES: u64 = 18;

    /// R.10b — lazily allocate the compact-wide-detector-digest-v1
    /// arena (`n_cells × 18` bytes) for this workspace's contract
    /// dimensions. Idempotent.
    ///
    /// At 256×4096 this is ~18 MB on device. K > 1 batched D64
    /// throughput is R.9.d+ work; one catalog is allocated here.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::KernelFailed(code)` if `cudaMalloc`
    /// refuses.
    pub fn ensure_detector_digest_compact_buffer(&mut self) -> Result<(), GpuError> {
        if !self.d_detector_digest_compact.is_null() {
            return Ok(());
        }
        let n_cells = u64::from(self.n_entities) * u64::from(self.n_windows);
        let bytes = n_cells * Self::DETECTOR_WIDE_DIGEST_COMPACT_V1_BYTES;
        let mut ptr: *mut u8 = std::ptr::null_mut();
        // Safety: the FFI writes a valid device pointer or returns
        // non-zero. We never dereference the raw bytes from Rust.
        #[allow(unsafe_code)]
        let status: c_int =
            unsafe { ffi::dsfb_gpu_alloc_bytes(bytes, std::ptr::from_mut(&mut ptr)) };
        if status != 0 {
            return Err(GpuError::KernelFailed(status));
        }
        self.d_detector_digest_compact = ptr;
        Ok(())
    }

    /// R.10b — `true` after `ensure_detector_digest_compact_buffer`
    /// has allocated the arena. The D64 throughput dispatch must
    /// gate on this before passing the pointer to the FFI.
    #[must_use]
    pub fn has_detector_digest_compact_buffer(&self) -> bool {
        !self.d_detector_digest_compact.is_null()
    }

    /// R.10c — accessor for the per-cell `fired` flag buffer.
    /// Returns null until `ensure_candidate_parallel_buffers` has
    /// succeeded.
    pub(crate) fn d_candidate_fired(&self) -> *mut u8 {
        self.d_candidate_fired
    }

    /// R.10c — accessor for the per-entity candidate boundary
    /// scratch (`(start_w, end_w)` tuples). Returns null until
    /// `ensure_candidate_parallel_buffers` has succeeded.
    pub(crate) fn d_candidate_boundaries(&self) -> *mut u8 {
        self.d_candidate_boundaries
    }

    /// R.10c — lazily allocate both parallel-candidate-collapse
    /// scratch buffers (`d_candidate_fired` + `d_candidate_boundaries`).
    /// Idempotent. Sized for one catalog at the workspace's
    /// contract.
    ///
    /// At 256×4096 the fired buffer is 1 MB and the boundary
    /// buffer is 32 KB — negligible compared to the wide detector
    /// arena. K > 1 batched D64 throughput is R.9.d+ work.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::KernelFailed(code)` if either `cudaMalloc`
    /// refuses. On partial-failure (first alloc succeeds, second
    /// fails) the first buffer is freed before returning.
    pub fn ensure_candidate_parallel_buffers(&mut self) -> Result<(), GpuError> {
        if !self.d_candidate_fired.is_null() && !self.d_candidate_boundaries.is_null() {
            return Ok(());
        }
        let n_cells = u64::from(self.n_entities) * u64::from(self.n_windows);
        // fired: 1 byte per cell.
        if self.d_candidate_fired.is_null() {
            let mut ptr: *mut u8 = std::ptr::null_mut();
            #[allow(unsafe_code)]
            let status: c_int =
                unsafe { ffi::dsfb_gpu_alloc_bytes(n_cells, std::ptr::from_mut(&mut ptr)) };
            if status != 0 {
                return Err(GpuError::KernelFailed(status));
            }
            self.d_candidate_fired = ptr;
        }
        // boundaries: 8 bytes per (entity × max_per_entity) slot.
        if self.d_candidate_boundaries.is_null() {
            let slots = u64::from(self.n_entities) * (MAX_CANDIDATES_PER_ENTITY as u64);
            let bytes = slots * 8;
            let mut ptr: *mut u8 = std::ptr::null_mut();
            #[allow(unsafe_code)]
            let status: c_int =
                unsafe { ffi::dsfb_gpu_alloc_bytes(bytes, std::ptr::from_mut(&mut ptr)) };
            if status != 0 {
                // Roll back the fired buffer so a retry can re-attempt
                // both allocations cleanly.
                #[allow(unsafe_code)]
                unsafe {
                    let _ = ffi::dsfb_gpu_free_bytes(self.d_candidate_fired);
                }
                self.d_candidate_fired = std::ptr::null_mut();
                return Err(GpuError::KernelFailed(status));
            }
            self.d_candidate_boundaries = ptr;
        }
        Ok(())
    }

    /// R.10c — `true` after both parallel-candidate buffers are
    /// allocated.
    #[must_use]
    pub fn has_candidate_parallel_buffers(&self) -> bool {
        !self.d_candidate_fired.is_null() && !self.d_candidate_boundaries.is_null()
    }

    /// S-PERF.14c — accessor for the per-entity intermediate
    /// run-boundary scratch (`(start_w, end_w)` tuples produced by
    /// the Pre-Alpha precompute kernel). Returns null until
    /// `ensure_candidate_run_buffer` has succeeded.
    pub(crate) fn d_candidate_run_buffer(&self) -> *mut u8 {
        self.d_candidate_run_buffer
    }

    /// S-PERF.14c — accessor for the per-entity surviving-run
    /// count scratch (the count produced by the Pre-Alpha
    /// precompute kernel after the length + max-per-entity filter).
    /// Returns null until `ensure_candidate_run_buffer` has
    /// succeeded.
    pub(crate) fn d_candidate_run_count(&self) -> *mut i32 {
        self.d_candidate_run_count
    }

    /// S-PERF.14c — lazily allocate both candidate-boundary Pre-
    /// Alpha + cellpar split scratch buffers
    /// (`d_candidate_run_buffer` + `d_candidate_run_count`).
    /// Idempotent. Sized for one catalog at the workspace's
    /// contract.
    ///
    /// At 256 × 4 096 K=1 the run-buffer is 32 KB and the
    /// run-count is 1 KB — negligible compared to the existing
    /// R.10c scratch and to the wide detector arena. K > 1
    /// batched D64 throughput is later work; this allocator
    /// sizes only for one catalog. The split lets the cellpar
    /// emit kernel expose ≈ 4 096 (entity, slot) threads at
    /// canonical scale to the 80-SM device, breaking the
    /// 2.1 %-occupancy ceiling the legacy single-kernel
    /// `candidate_boundary_kernel_wide` paid (8 blocks × 32
    /// threads = 256 threads total; ROOF post-S-PERF.14b
    /// receipt).
    ///
    /// # Errors
    ///
    /// Returns `GpuError::KernelFailed(code)` if either `cudaMalloc`
    /// refuses. On partial failure (first alloc succeeds, second
    /// fails) the first buffer is freed before returning so a
    /// retry can re-attempt both allocations cleanly.
    pub fn ensure_candidate_run_buffer(&mut self) -> Result<(), GpuError> {
        if !self.d_candidate_run_buffer.is_null() && !self.d_candidate_run_count.is_null() {
            return Ok(());
        }
        // run_buffer: 8 bytes per (entity × max_per_entity) slot —
        // same layout as `d_candidate_boundaries`.
        if self.d_candidate_run_buffer.is_null() {
            let slots = u64::from(self.n_entities) * (MAX_CANDIDATES_PER_ENTITY as u64);
            let bytes = slots * 8;
            let mut ptr: *mut u8 = std::ptr::null_mut();
            #[allow(unsafe_code)]
            let status: c_int =
                unsafe { ffi::dsfb_gpu_alloc_bytes(bytes, std::ptr::from_mut(&mut ptr)) };
            if status != 0 {
                return Err(GpuError::KernelFailed(status));
            }
            self.d_candidate_run_buffer = ptr;
        }
        // run_count: 4 bytes per entity (i32 count after filter).
        if self.d_candidate_run_count.is_null() {
            let bytes = u64::from(self.n_entities) * 4;
            let mut ptr: *mut u8 = std::ptr::null_mut();
            #[allow(unsafe_code)]
            let status: c_int =
                unsafe { ffi::dsfb_gpu_alloc_bytes(bytes, std::ptr::from_mut(&mut ptr)) };
            if status != 0 {
                // Roll back the run buffer so a retry can re-attempt
                // both allocations cleanly.
                #[allow(unsafe_code)]
                unsafe {
                    let _ = ffi::dsfb_gpu_free_bytes(self.d_candidate_run_buffer);
                }
                self.d_candidate_run_buffer = std::ptr::null_mut();
                return Err(GpuError::KernelFailed(status));
            }
            #[allow(clippy::cast_ptr_alignment)]
            let typed = ptr.cast::<i32>();
            self.d_candidate_run_count = typed;
        }
        Ok(())
    }

    /// S-PERF.14c — `true` after both candidate-boundary Pre-Alpha +
    /// cellpar split scratch buffers are allocated.
    #[must_use]
    pub fn has_candidate_run_buffer(&self) -> bool {
        !self.d_candidate_run_buffer.is_null() && !self.d_candidate_run_count.is_null()
    }

    /// R.11b — accessor for the lazily-allocated device-side
    /// `GpuTraceEventCompact[]` buffer. Returns null until
    /// `ensure_events_buffer` has succeeded for some n_events.
    pub(crate) fn d_events(&self) -> *mut GpuTraceEventCompact {
        self.d_events
    }

    /// R.11b — lazily allocate (or regrow) the device-side
    /// `GpuTraceEventCompact[]` buffer and the matching pinned host shadow,
    /// sized for `n_events` events. Idempotent for the same or
    /// smaller event count; on a larger count the existing buffer
    /// is freed and replaced.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::KernelFailed(code)` if `cudaMalloc`
    /// refuses, or `GpuError::CudaUnavailable` propagated from
    /// `PinnedHostBuf::new` if the pinned-host allocation fails.
    pub fn ensure_events_buffer(&mut self, n_events: u64) -> Result<(), GpuError> {
        // Already sized correctly: nothing to do.
        if !self.d_events.is_null() && self.d_events_capacity >= n_events {
            // The pinned shadow length must match the device
            // capacity exactly so `copy_from_slice` doesn't truncate
            // a larger incoming slice; the matching pinned shadow
            // was sized to `d_events_capacity` at the previous
            // allocation. The dispatch caller copies only the first
            // n_events slots, so a larger capacity is harmless.
            return Ok(());
        }
        // Free any prior allocation. This is safe even when the
        // pointer is null (the FFI no-ops on null).
        if !self.d_events.is_null() {
            // Safety: the pointer was obtained from
            // `dsfb_gpu_alloc_bytes`; freeing it via the matching
            // FFI is the documented contract.
            #[allow(unsafe_code)]
            unsafe {
                let _ = ffi::dsfb_gpu_free_bytes(self.d_events.cast::<u8>());
            }
            self.d_events = std::ptr::null_mut();
            self.d_events_capacity = 0;
        }
        let bytes = n_events * (core::mem::size_of::<GpuTraceEventCompact>() as u64);
        let mut ptr: *mut u8 = std::ptr::null_mut();
        #[allow(unsafe_code)]
        let status: c_int =
            unsafe { ffi::dsfb_gpu_alloc_bytes(bytes, std::ptr::from_mut(&mut ptr)) };
        if status != 0 {
            return Err(GpuError::KernelFailed(status));
        }
        // The allocator returns 256-byte aligned device memory; the
        // cast to `*mut GpuTraceEventCompact` (which has 8-byte alignment
        // requirements) is safe in practice.
        #[allow(clippy::cast_ptr_alignment)]
        let typed = ptr.cast::<GpuTraceEventCompact>();
        self.d_events = typed;
        self.d_events_capacity = n_events;

        // Resize the pinned host shadow to match. PinnedHostBuf
        // owns its allocation and drops it on replacement.
        self.events_pinned = Some(crate::pinned::PinnedHostBuf::new(n_events as usize)?);
        Ok(())
    }

    /// R.11b — `true` after `ensure_events_buffer` has allocated
    /// at least one event-shaped buffer.
    #[must_use]
    pub fn has_events_buffer(&self) -> bool {
        !self.d_events.is_null() && self.events_pinned.is_some()
    }

    /// R.6d — was `dsfb_gpu_upload_detector_thresholds` successful
    /// at workspace construction? When true, dispatch wrappers
    /// pass `use_const_thresholds=1` to the FFI and the kernel
    /// reads the constant-memory copy. When false, the param-
    /// passing kernel variant runs (byte-equivalent fallback).
    #[must_use]
    pub fn has_const_thresholds(&self) -> bool {
        self.const_thresholds_uploaded
    }

    /// R.6d — test-only forcer for the byte-equivalence fallback
    /// tests in `r6d_const_thresholds.rs`. Lets the suite exercise
    /// both branches deterministically without actually failing
    /// `cudaMemcpyToSymbol`. Production code never calls this;
    /// the workspace's natural upload-at-construction path is the
    /// only setter in normal operation.
    #[doc(hidden)]
    pub fn force_const_thresholds_uploaded_for_test(&mut self, value: bool) {
        self.const_thresholds_uploaded = value;
    }

    /// R.6c — does this workspace hold a captured graph exec?
    /// Returns `true` after a successful
    /// `try_capture_throughput_graph`. The dispatch wrapper uses
    /// the answer to decide whether to launch the graph or
    /// demote to R.6b.
    #[must_use]
    pub fn has_graph(&self) -> bool {
        self.graph_exec != 0
    }

    /// R.6c — accessor for the canonical graph plan hash recorded
    /// at capture time. `None` until capture has succeeded at
    /// least once on this workspace. The hash is the same value
    /// the dispatch wrapper writes into the case file's
    /// supplementary fields.
    #[must_use]
    pub fn graph_plan_hash(&self) -> Option<[u8; 32]> {
        self.graph_plan_hash
    }

    /// R.6c — attempt to capture the Throughput-digests pipeline
    /// into a `cudaGraphExec_t` against this workspace's pinned
    /// shadows and stream. Returns `GraphCaptureStatus::Captured`
    /// on success and `GraphCaptureStatus::Demoted` when the
    /// driver / device refuses capture (older driver, stream not
    /// supported, etc.) — the dispatch wrapper then demotes to
    /// the R.6b async path. Idempotent: a second call on a
    /// workspace that already holds a graph returns the existing
    /// `Captured` status without re-entering capture mode.
    ///
    /// # Errors
    ///
    /// * `GpuError::InvalidInput` if the workspace was not built
    ///   with `new_with_pinned_async` (no pinned shadows + stream
    ///   to bind the graph against), or if the contract's
    ///   dimensions disagree with the workspace's allocation.
    ///
    /// Note: a CUDA capture failure is **not** propagated as
    /// `Err`. Capture is best-effort; the wrapper returns
    /// `Demoted` so the caller can fall back. Only structural
    /// configuration mistakes surface as `Err`.
    ///
    /// # Panics
    ///
    /// Unreachable in practice: the `has_pinned_async` gate above
    /// guarantees every `expect()` on the pinned shadow accessors
    /// succeeds. If `has_pinned_async()` returned `true` then by
    /// construction all four pinned shadows are `Some`.
    #[allow(clippy::expect_used, clippy::too_many_lines)]
    pub fn try_capture_throughput_graph(
        &mut self,
        contract: &Contract,
    ) -> Result<GraphCaptureStatus, GpuError> {
        self.assert_compatible(contract)?;
        if !self.has_pinned_async() {
            return Err(GpuError::InvalidInput(
                "GpuWorkspace was not built with pinned shadows + stream; \
                 call GpuWorkspace::new_with_pinned_async(contract)",
            ));
        }

        // Idempotent: if a graph is already captured for this
        // workspace, return its plan hash and do not re-capture.
        if self.graph_exec != 0 {
            if let Some(plan_hash) = self.graph_plan_hash {
                return Ok(GraphCaptureStatus::Captured { plan_hash });
            }
        }

        // Compute the canonical plan hash from the contract's
        // scale and the const-thresholds upload status. The hash
        // is independent of any per-launch state (events, pointer
        // values, time) so it is stable run-over-run; it changes
        // only when the captured topology metadata changes (scale
        // or R.6d const-thresholds branch).
        let uses_const = self.const_thresholds_uploaded;
        let plan_hash = compute_throughput_graph_plan_hash(contract, uses_const);

        // Pin static parameter buffers so the C++ side reads them
        // from stable addresses during capture.
        let thresholds_ffi = DetectorThresholdsFfi::from(&DetectorThresholds::CANONICAL);
        let baseline = Baseline::CANONICAL;
        let candidate_cfg = CandidateConfig::CANONICAL;

        let h_features_ptr = self
            .features_pinned
            .as_ref()
            .expect("has_pinned_async guarantees features_pinned is Some")
            .as_ptr();
        let h_candidates_ptr = self
            .candidates_pinned
            .as_mut()
            .expect("has_pinned_async guarantees candidates_pinned is Some")
            .as_mut_ptr();
        let h_count_ptr = self
            .candidate_count_pinned
            .as_mut()
            .expect("has_pinned_async guarantees candidate_count_pinned is Some")
            .as_mut_ptr();
        let h_digests_ptr = self
            .stage_digests_pinned
            .as_mut()
            .expect("has_pinned_async guarantees stage_digests_pinned is Some")
            .as_mut_ptr();

        let n_entities = self.n_entities as i32;
        let n_windows = self.n_windows as i32;
        let stream = self.stream;

        let mut graph_exec: u64 = 0;
        // Safety: every pointer comes from a workspace field that
        // is alive for the duration of the FFI call. The C side
        // either fills `graph_exec` with a live `cudaGraphExec_t`
        // and returns 0, or sets it to 0 and returns a non-zero
        // `cudaError_t` — the Rust side never dereferences the
        // handle on the failure branch.
        #[allow(unsafe_code)]
        let status: c_int = unsafe {
            ffi::dsfb_gpu_try_capture_throughput_graph(
                std::ptr::from_mut(&mut graph_exec),
                self.d_features,
                self.d_residuals,
                self.d_signs,
                self.d_detectors,
                self.d_consensus,
                self.d_candidates,
                self.d_candidate_count,
                self.d_stage_digests,
                h_features_ptr,
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
                h_candidates_ptr,
                h_count_ptr,
                h_digests_ptr,
                stream,
                c_int::from(uses_const),
            )
        };

        if status != 0 || graph_exec == 0 {
            // Capture refused or returned no exec. Surface as
            // Demoted; the caller falls back to R.6b. The error
            // code is part of the reason string so a CI run on
            // a graph-incapable host records it verbatim.
            return Ok(GraphCaptureStatus::Demoted {
                reason: format!("cuda graph capture refused (cudaError {status})"),
            });
        }

        self.graph_exec = graph_exec;
        self.graph_plan_hash = Some(plan_hash);
        Ok(GraphCaptureStatus::Captured { plan_hash })
    }

    /// R.6c — opaque captured `cudaGraphExec_t` handle for the
    /// throughput pipeline. 0 when no graph is captured.
    pub(crate) fn graph_exec(&self) -> u64 {
        self.graph_exec
    }

    /// R.3b: host-side consensus buffer accessor for the
    /// "consensus-D2H-stripped" acceptance test. Returns the
    /// workspace's host shadow of the consensus grid. Pre-R.3b every
    /// Tier 3B dispatch overwrote this buffer with the consensus
    /// cells from the device; post-R.3b the dispatch passes null and
    /// the buffer is never touched. A test that poisons this buffer
    /// with a sentinel pattern and re-reads after a dispatch is the
    /// load-bearing observable that the D2H is actually gone.
    #[must_use]
    pub fn consensus(&self) -> &[ConsensusCell] {
        &self.consensus
    }

    /// R.3b: mutable accessor paired with [`consensus`]. Tests use
    /// this to poison the host buffer with a sentinel before a
    /// dispatch; production callers should never write into it.
    #[must_use]
    pub fn consensus_mut(&mut self) -> &mut [ConsensusCell] {
        &mut self.consensus
    }
}

/// K-catalog reusable host + device storage for the batched
/// deterministic GPU pipeline (O.16, Tier 2).
///
/// Lays out every per-cell buffer as `n_catalogs * n_entities * n_windows`
/// in catalog-major order. Each catalog occupies a contiguous slice of
/// `n_entities * n_windows` elements at offset `catalog_id *
/// n_entities * n_windows`. The kernels read `blockIdx.z` to select the
/// catalog and offset their indices accordingly.
///
/// Independence: catalog[j]'s output bytes are a function only of
/// catalog[j]'s input bytes. Two catalogs running in the same batch
/// cannot influence each other; corrupting the input of catalog[j]
/// only changes case[j].
///
/// Lifetime: device buffers are freed in `Drop`. The same FFI
/// `dsfb_gpu_workspace_free` is used because it is layout-agnostic.
///
/// Sized for one `(n_catalogs, n_entities, n_windows)` triple; rebuild
/// the workspace if any dimension changes.
pub struct BatchedGpuWorkspace {
    d_features: *mut WindowFeature,
    d_residuals: *mut ResidualCell,
    d_signs: *mut SignCell,
    d_detectors: *mut DetectorCell,
    d_consensus: *mut ConsensusCell,
    d_candidates: *mut CandidateInterval,
    d_candidate_count: *mut i32,
    /// `4 × 32 × n_catalogs` bytes for Tier 3B per-catalog per-stage
    /// digests. Stage-major layout (`[K residual][K sign][K detector]
    /// [K consensus]`). Allocated lazily via `dsfb_gpu_alloc_bytes`.
    d_stage_digests: *mut u8,

    /// Host-side reusable storage, all sized for the full batch.
    pub(crate) features: Vec<WindowFeature>,
    pub(crate) residuals: Vec<ResidualCell>,
    pub(crate) signs: Vec<SignCell>,
    pub(crate) detectors: Vec<DetectorCell>,
    pub(crate) consensus: Vec<ConsensusCell>,
    pub(crate) candidate_buf: Vec<CandidateInterval>,
    pub(crate) candidate_count: Vec<i32>,

    /// Number of catalogs the workspace was built for.
    pub n_catalogs: u32,
    /// Number of entities per catalog.
    pub n_entities: u32,
    /// Number of windows per catalog.
    pub n_windows: u32,
}

impl BatchedGpuWorkspace {
    /// Allocate device + host buffers sized for the given batch shape.
    ///
    /// # Errors
    ///
    /// `GpuError::KernelFailed(code)` if any `cudaMalloc` fails. The
    /// constructor rolls back partial allocations before returning.
    pub fn new(n_catalogs: u32, contract: &Contract) -> Result<Self, GpuError> {
        let per_catalog: usize = (contract.n_entities as usize) * (contract.n_windows as usize);
        let total: usize = (n_catalogs as usize) * per_catalog;
        let candidate_capacity_per_catalog: usize =
            (contract.n_entities as usize) * (MAX_CANDIDATES_PER_ENTITY as usize);
        let candidate_total: usize = (n_catalogs as usize) * candidate_capacity_per_catalog;

        let mut d_features: *mut WindowFeature = std::ptr::null_mut();
        let mut d_residuals: *mut ResidualCell = std::ptr::null_mut();
        let mut d_signs: *mut SignCell = std::ptr::null_mut();
        let mut d_detectors: *mut DetectorCell = std::ptr::null_mut();
        let mut d_consensus: *mut ConsensusCell = std::ptr::null_mut();
        let mut d_candidates: *mut CandidateInterval = std::ptr::null_mut();
        let mut d_candidate_count: *mut i32 = std::ptr::null_mut();

        // Safety: every out-pointer is a valid `*mut *mut T`. The C
        // side either writes all of them (success, returns 0) or zeroes
        // all of them (failure path).
        #[allow(unsafe_code)]
        let status: c_int = unsafe {
            ffi::dsfb_gpu_workspace_alloc_batched(
                n_catalogs as i32,
                contract.n_entities as i32,
                contract.n_windows as i32,
                MAX_CANDIDATES_PER_ENTITY,
                std::ptr::from_mut(&mut d_features),
                std::ptr::from_mut(&mut d_residuals),
                std::ptr::from_mut(&mut d_signs),
                std::ptr::from_mut(&mut d_detectors),
                std::ptr::from_mut(&mut d_consensus),
                std::ptr::from_mut(&mut d_candidates),
                std::ptr::from_mut(&mut d_candidate_count),
            )
        };
        if status != 0 {
            return Err(GpuError::KernelFailed(status));
        }

        // Tier 3B digest buffer: 4 stages × 32 bytes × K catalogs.
        let mut d_stage_digests: *mut u8 = std::ptr::null_mut();
        #[allow(unsafe_code)]
        let dig_status: c_int = unsafe {
            ffi::dsfb_gpu_alloc_bytes(
                4 * 32 * u64::from(n_catalogs),
                std::ptr::from_mut(&mut d_stage_digests),
            )
        };
        if dig_status != 0 {
            #[allow(unsafe_code)]
            unsafe {
                ffi::dsfb_gpu_workspace_free(
                    d_features,
                    d_residuals,
                    d_signs,
                    d_detectors,
                    d_consensus,
                    d_candidates,
                    d_candidate_count,
                );
            }
            return Err(GpuError::KernelFailed(dig_status));
        }

        Ok(Self {
            d_features,
            d_residuals,
            d_signs,
            d_detectors,
            d_consensus,
            d_candidates,
            d_candidate_count,
            d_stage_digests,
            features: vec![WindowFeature::default(); total],
            residuals: vec![ResidualCell::default(); total],
            signs: vec![SignCell::default(); total],
            detectors: vec![DetectorCell::default(); total],
            consensus: vec![ConsensusCell::default(); total],
            candidate_buf: vec![CandidateInterval::default(); candidate_total],
            candidate_count: vec![0i32; (n_catalogs as usize) * (contract.n_entities as usize)],
            n_catalogs,
            n_entities: contract.n_entities,
            n_windows: contract.n_windows,
        })
    }

    /// Number of cells per catalog (`n_entities * n_windows`).
    #[must_use]
    pub const fn per_catalog_cells(&self) -> usize {
        (self.n_entities as usize) * (self.n_windows as usize)
    }

    /// Number of candidate slots per catalog.
    #[must_use]
    pub const fn per_catalog_candidate_slots(&self) -> usize {
        (self.n_entities as usize) * (MAX_CANDIDATES_PER_ENTITY as usize)
    }

    pub(crate) fn d_features(&self) -> *mut WindowFeature {
        self.d_features
    }
    pub(crate) fn d_residuals(&self) -> *mut ResidualCell {
        self.d_residuals
    }
    pub(crate) fn d_signs(&self) -> *mut SignCell {
        self.d_signs
    }
    pub(crate) fn d_detectors(&self) -> *mut DetectorCell {
        self.d_detectors
    }
    pub(crate) fn d_consensus(&self) -> *mut ConsensusCell {
        self.d_consensus
    }
    pub(crate) fn d_candidates(&self) -> *mut CandidateInterval {
        self.d_candidates
    }
    pub(crate) fn d_candidate_count(&self) -> *mut i32 {
        self.d_candidate_count
    }
    pub(crate) fn d_stage_digests(&self) -> *mut u8 {
        self.d_stage_digests
    }
}

impl Drop for BatchedGpuWorkspace {
    fn drop(&mut self) {
        // Safety: same contract as `GpuWorkspace::Drop`. The
        // workspace_free routine is layout-agnostic (it just frees
        // every non-null pointer it is given) so it works for both
        // single-catalog and batched workspaces. The digest byte buffer
        // is freed separately via `dsfb_gpu_free_bytes`.
        #[allow(unsafe_code)]
        unsafe {
            ffi::dsfb_gpu_workspace_free(
                self.d_features,
                self.d_residuals,
                self.d_signs,
                self.d_detectors,
                self.d_consensus,
                self.d_candidates,
                self.d_candidate_count,
            );
            ffi::dsfb_gpu_free_bytes(self.d_stage_digests);
        }
    }
}

impl Drop for GpuWorkspace {
    fn drop(&mut self) {
        // Safety: each pointer was either populated by the matching
        // workspace_alloc call (and thus refers to a live device
        // allocation) or is null. `dsfb_gpu_workspace_free` accepts
        // both, never touches an already-freed pointer, and returns a
        // status we intentionally ignore — there is nothing useful to
        // do with a free-time error during Drop. The digest byte
        // buffer is freed separately via `dsfb_gpu_free_bytes`.
        #[allow(unsafe_code)]
        unsafe {
            ffi::dsfb_gpu_workspace_free(
                self.d_features,
                self.d_residuals,
                self.d_signs,
                self.d_detectors,
                self.d_consensus,
                self.d_candidates,
                self.d_candidate_count,
            );
            ffi::dsfb_gpu_free_bytes(self.d_stage_digests);
            // R.4: free the Pre-Alpha drift buffer.
            ffi::dsfb_gpu_free_bytes(self.d_drifts);
            // R.8.5: free the tree-digest scratch arenas (null when
            // tree digest was never enabled on this workspace).
            ffi::dsfb_gpu_free_bytes(self.d_tree_leaves);
            ffi::dsfb_gpu_free_bytes(self.d_tree_scratch);
            // R.9.b.2: free the wide-detector buffer (null when the
            // wide dispatch was never invoked on this workspace).
            ffi::dsfb_gpu_free_bytes(self.d_detectors_wide.cast::<u8>());
            // R.10a: free the axis-5 grid-sum buffer (null when the
            // D64 throughput dispatch never ran on this workspace).
            ffi::dsfb_gpu_free_bytes(self.d_axis5_grid_sum);
            // S-PERF.14: free the Pre-Alpha drift EWMA precompute
            // buffer (null when the D64 throughput dispatch never
            // ran on this workspace).
            ffi::dsfb_gpu_free_bytes(self.d_drift_buffer.cast::<u8>());
            // R.10b: free the compact-wide-detector-digest-v1
            // arena (null when the D64 throughput dispatch never
            // ran on this workspace).
            ffi::dsfb_gpu_free_bytes(self.d_detector_digest_compact);
            // R.10c: free the parallel-candidate-collapse scratch
            // buffers (null when the D64 throughput dispatch never
            // ran on this workspace).
            ffi::dsfb_gpu_free_bytes(self.d_candidate_fired);
            ffi::dsfb_gpu_free_bytes(self.d_candidate_boundaries);
            // S-PERF.14c: free the candidate-boundary Pre-Alpha +
            // cellpar split scratch buffers (null when the D64
            // throughput dispatch never ran on this workspace).
            ffi::dsfb_gpu_free_bytes(self.d_candidate_run_buffer);
            ffi::dsfb_gpu_free_bytes(self.d_candidate_run_count.cast::<u8>());
            // R.11b: free the device-side GpuTraceEventCompact[] buffer
            // (null when the D64 throughput dispatch never ran on
            // this workspace). The pinned host shadow drops itself.
            ffi::dsfb_gpu_free_bytes(self.d_events.cast::<u8>());
            // R.6c: destroy any captured graph exec BEFORE the
            // stream is destroyed — the exec keeps an internal
            // reference to topology recorded against this stream.
            if self.graph_exec != 0 {
                let _ = ffi::dsfb_gpu_destroy_throughput_graph(self.graph_exec);
            }
            // R.6b: destroy the CUDA stream (no-op when 0). The
            // pinned host shadows drop themselves via PinnedHostBuf's
            // `Drop` impl on the way out of this function.
            if self.stream != 0 {
                let _ = ffi::dsfb_gpu_destroy_stream(self.stream);
            }
        }
    }
}
