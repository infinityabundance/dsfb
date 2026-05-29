//! # dsfb-chemical-engineering-cuda
//!
//! CUDA-accelerated DSFB-Chemical-Engineering **evidence factory** and **forensic court**.
//!
//! The evidence factory quantises residual streams (and the usually-discarded noise) to fixed point
//! and seals each lane into an on-GPU SHA-256 digest under a locked deterministic contract
//! (`--fmad=false`, integer evidence arithmetic). The CPU reference path produces the identical
//! evidence, so the GPU result can be **cross-verified byte-for-byte**. The court then assembles a
//! hash-linked **case file** (passport, per-lane evidence, Merkle + evidence root, challenge docket,
//! precedent chain) and an **execution attestation** (backend, device, timing, throughput) sealed
//! over the reproducible evidence root. Replay re-derives the root and confirms it matches.
//!
//! Doctrine: *the GPU produces evidence; the court decides what that evidence is allowed to mean.*
//!
//! Unsafe code is confined to [`ffi`] (CUDA C ABI), present only under `--features cuda`.
//!
//! # Module map
//!
//! | Module | Role |
//! |--------|------|
//! | [`evidence`] | Fixed-point contract + CPU reference implementation + determinism tests |
//! | [`dispatch`] | Backend selection, lane construction, GPU/CPU cross-verification |
//! | [`court`] | Passport, case file, attestation, challenge docket, replay verification |
//! | [`ffi`] | `unsafe` CUDA C ABI wrappers (cuda feature only) |
//! | [`bench`] | GB/s throughput benchmark harness (evidence factory + roofline) |
//! | [`cli`] | Command-line driver (demo, bench, verify-replay, device, profile) |
//!
//! # Determinism guarantee
//!
//! The core invariant: **every build of this crate, on any backend, produces the same evidence root
//! for the same input.** This is enforced by:
//! - Fixed-point quantisation (no floating-point accumulation in evidence arithmetic).
//! - `--fmad=false` on the CUDA kernel compilation unit (disables fused multiply-add on the GPU),
//!   ensuring the single `x * SCALE` multiply is correctly rounded, identical to Rust.
//! - On-GPU SHA-256 (FIPS 180-4) that matches the host `sha2` crate byte-for-byte.
//! - The `dispatch` module cross-verifies GPU vs. CPU evidence roots after every GPU run.

// Enforce, with the compiler, the invariant the docs state: ALL `unsafe` in this crate lives in the
// `ffi` module (the CUDA C ABI boundary). The crate denies unsafe everywhere; only `ffi` opts back in.
#![deny(unsafe_code)]

pub mod bench;
pub mod cli;
pub mod court;
pub mod digest_equivalence;
pub mod dispatch;
pub mod evidence;

/// Safe wrappers around the CUDA C ABI (`extern "C"` symbols from kernels.cu).
/// Compiled only when the `cuda` feature is active; all `unsafe` in this crate lives here.
#[cfg(feature = "cuda")]
#[allow(unsafe_code)] // the ONE permitted place: CUDA C ABI wrappers (see module docs).
pub mod ffi;

/// Re-export the edge crate so downstream users can reach shared types without an extra dep.
pub use dsfb_chemical_engineering_edge as edge;

/// Crate version string, sourced from Cargo.toml at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Whether this build includes the CUDA backend.
///
/// `true` when compiled with `--features cuda`; `false` for the CPU-only build.
/// The CPU reference path is always available regardless of this flag.
pub const CUDA_ENABLED: bool = cfg!(feature = "cuda");
