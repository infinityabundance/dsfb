//! CUDA dispatch and FFI bridge for `dsfb-gpu-debug` --- the
//! **CUDA Evidence Factory** for the DSFB-GPU densorial /
//! tekmeric inference stack.
//!
//! **Front-door identity (panel-locked)**:
//!
//! > DSFB-GPU is a CUDA-accelerated deterministic evidence
//! > court: byte-exact witness-family kernels shade residual
//! > densors into canonical evidence bytes, then a CPU-side
//! > jurisprudence layer admits, challenges, contraindicates,
//! > and records them into replayable case files.
//!
//! This crate is the GPU half of that posture. It is not a
//! neural inference backend. It executes deterministic
//! witness-family kernels over residual densors and returns
//! canonical witness bytes, candidate summaries, and stage
//! digests to the CPU-side court (the `dsfb-gpu-debug-core`
//! and `dsfb-gpu-atlas-corpus` crates).
//!
//! The GPU has no semantic authority. It does not admit
//! episodes or assign final meaning. It produces byte-exact
//! evidence under a declared numeric, boundary, reduction,
//! parameter, and hashing contract.
//!
//! ```text
//! residual densors
//!   → CUDA deterministic witness families
//!   → witness densors / candidate summaries / stage digests
//!   → CPU court admission
//!   → replayable case file
//! ```
//!
//! Panel-locked non-claims: this crate does NOT claim peak
//! memory-bandwidth saturation, optimal kernel occupancy,
//! optimal multi-GPU scaling, or production CUDA performance.
//! Those are explicit future performance-campaign targets.
//! Current artifacts establish the deterministic-evidence-
//! factory shape, the byte-exact CPU/GPU equivalence, and
//! the Semantic Non-Bypass Axiom (the bank stage stays CPU-
//! side; episodes only enter the case file through the
//! bank-private admission token).
//!
//! Build behavior is split by the `cuda` feature flag:
//!
//! * Without `cuda`: the crate compiles to a thin shim. Every public entry
//!   point returns `GpuError::CudaUnavailable`. This lets the rest of the
//!   workspace build and run on hosts without `nvcc` installed, which is
//!   important because the CPU reference path is the v0 reproducibility
//!   target — the GPU path is a parity check.
//! * With `cuda`: `build.rs` invokes `nvcc` to compile the kernels under
//!   `cuda/kernels.cu` into a static archive, links it, and the dispatch
//!   module wires Rust callers to the C ABI functions exposed in `ffi.rs`.
//!
//! `unsafe` is restricted to the CUDA FFI boundary and the
//! resource-lifetime wrappers that own raw device handles: the
//! kernel-dispatch calls in `dispatch`, the workspace allocate /
//! free / readback paths in `workspace`, the `cudaMallocHost` /
//! `cudaFreeHost` owning handle in `pinned`, and the stream /
//! graph handle plumbing the throughput path needs. All semantic
//! code (case-file construction, hash chain, bank admission) remains
//! safe Rust. The unsafe surface is small, auditable, and never
//! reaches across module boundaries; each `unsafe` block sits next
//! to a comment naming the invariant it carries.

#![deny(missing_docs)]

/// Errors that the GPU pipeline can surface.
///
/// The most common variant on developer machines without an NVIDIA toolkit
/// is `CudaUnavailable`; the CLI translates this into exit code 2 so it can
/// be distinguished from a hash-chain divergence.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GpuError {
    /// The crate was built without the `cuda` feature, so no kernels are
    /// present. The CPU reference path still runs.
    CudaUnavailable,
    /// A kernel returned a non-zero error code. The contained value is the
    /// raw status from the FFI boundary.
    KernelFailed(i32),
    /// One of the pipeline inputs disagrees with the contract (bad
    /// dimensions, oversize buffer, null pointer at the FFI boundary).
    InvalidInput(&'static str),
}

#[cfg(feature = "cuda")]
mod ffi;

#[cfg(feature = "cuda")]
mod dispatch;

#[cfg(feature = "cuda")]
mod pinned;

#[cfg(feature = "cuda")]
mod workspace;

#[cfg(feature = "cuda")]
pub use dispatch::{
    build_gpu, build_gpu_batched_throughput, build_gpu_batched_throughput_device_digests,
    build_gpu_fused_throughput_digests_on_workspace, build_gpu_layer_a_batched,
    build_gpu_layer_a_on_workspace, build_gpu_on_workspace,
    build_gpu_throughput_device_digests_on_workspace, build_gpu_throughput_graph_or_demote,
    build_gpu_throughput_on_workspace, build_gpu_throughput_pinned_async_on_workspace,
    build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact,
    build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact,
    build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed,
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact,
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed,
    build_gpu_throughput_pinned_async_on_workspace_timed,
    build_gpu_throughput_pinned_async_on_workspace_tree,
    build_gpu_throughput_pinned_async_on_workspace_tree_compact, build_gpu_timed,
    build_gpu_timed_on_workspace, compact_densor_root_path1a_sweep_time,
    compact_densor_root_streaming_sweep_time, evaluate_detector_wide_d64_on_workspace,
    sha256_device, sha256_device_streaming, D64ThroughputHostStageTimings,
    D64ThroughputStageTimings, PipelineTimings, R8HostStageTimings, R8StageTimings,
};
#[cfg(feature = "cuda")]
pub use pinned::PinnedHostBuf;
#[cfg(feature = "cuda")]
pub use workspace::{BatchedGpuWorkspace, GpuWorkspace, GraphCaptureStatus};

#[cfg(not(feature = "cuda"))]
mod stubs {
    use super::GpuError;
    use dsfb_gpu_debug_core::casefile::CaseFile;
    use dsfb_gpu_debug_core::contract::Contract;
    use dsfb_gpu_debug_core::event::TraceEvent;

    /// Stub for non-CUDA builds. Always returns `GpuError::CudaUnavailable`.
    ///
    /// # Errors
    ///
    /// Always returns `GpuError::CudaUnavailable` on builds without the
    /// `cuda` feature.
    pub fn build_gpu(_events: &[TraceEvent], _contract: &Contract) -> Result<CaseFile, GpuError> {
        Err(GpuError::CudaUnavailable)
    }
}

#[cfg(not(feature = "cuda"))]
pub use stubs::build_gpu;

/// Placeholder entry point retained for the workspace smoke build.
///
/// # Errors
///
/// Returns `GpuError::CudaUnavailable` whenever the crate was built without
/// the `cuda` feature.
pub fn pipeline_available() -> Result<(), GpuError> {
    if cfg!(feature = "cuda") {
        Ok(())
    } else {
        Err(GpuError::CudaUnavailable)
    }
}
