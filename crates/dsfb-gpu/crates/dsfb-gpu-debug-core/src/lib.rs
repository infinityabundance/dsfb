//! Deterministic CPU reference and semantic authority for
//! `dsfb-gpu-debug` --- the court half of the **densorial /
//! tekmeric inference** posture.
//!
//! **Front-door identity (panel-locked)**: DSFB-GPU implements
//! densorial / tekmeric inference (deterministic inference
//! over evidence densors + evidence-based deterministic
//! adjudication), not neural inference over learned-weight
//! tensors. This crate hosts the CPU reference path, the
//! Q16.16 fixed-point primitive that pins CPU/GPU byte-
//! equality, and the bank module that owns admission
//! authority for `Episode` (the Semantic Non-Bypass Axiom:
//! only the bank's private `BankAdmissionToken` constructor
//! can mint admitted episodes; the GPU never does).
//!
//! ```text
//! neural inference:
//!   tensor → learned weights → probabilistic output
//!
//! densorial / tekmeric inference:
//!   densor → deterministic witness court → replayable case file
//! ```
//!
//! This crate is intentionally `#![no_std]` and dependency-free. It contains:
//!
//! * The Q16.16 fixed-point primitive (`fixed::Q16`) used by both the CPU
//!   reference and the CUDA kernels, so CPU↔GPU byte-equality is achievable
//!   per stage of the pipeline.
//! * A hand-rolled SHA-256 implementation in `hash` so that intermediate
//!   artifact hashes can be computed in a `no_std` context without pulling in
//!   external dependencies.
//!
//! As the workspace grows, additional modules (canonical serialization,
//! pipeline stages, heuristics bank, episode collapse, case file emission)
//! will live alongside these two foundations. Sections are added one at a
//! time so each is independently auditable.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Tests legitimately use `.unwrap()` and `.unwrap_err()` to assert
// reachability; allowing those clippy lints under `cfg(test)` keeps the
// production code strict while letting test bodies stay concise.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

#[cfg(test)]
extern crate std;

pub mod event;
pub mod fixed;
pub mod grammar;
pub mod hash;
pub mod motif;
pub mod verdict;

#[cfg(feature = "std")]
pub mod bank;
#[cfg(feature = "std")]
pub mod candidate;
#[cfg(feature = "std")]
pub mod casefile;
#[cfg(feature = "std")]
pub mod casefile_v2;
#[cfg(feature = "std")]
pub mod consensus;
#[cfg(feature = "std")]
pub mod contract;
#[cfg(feature = "std")]
pub mod detector;
#[cfg(feature = "std")]
pub mod fixture;
#[cfg(feature = "std")]
pub mod residual;
#[cfg(feature = "std")]
pub mod serialize;
#[cfg(feature = "std")]
pub mod sign;
#[cfg(feature = "std")]
pub mod window;

#[cfg(feature = "std")]
pub use bank::{bank_hash, BankMotif, Episode, HeuristicEntry, CANONICAL_BANK};
#[cfg(feature = "std")]
pub use candidate::{CandidateConfig, CandidateInterval};
#[cfg(feature = "std")]
pub use casefile::{
    build_cpu, build_cpu_throughput, build_from_artifacts_with_mode, compare as compare_case_files,
    emit as emit_case_file, CaseFile, EmissionMode, IntermediateHashes, COMPACT_INPUT_DOMAIN,
    TRACE_EVENT_LAYOUT_TAG,
};
#[cfg(feature = "std")]
pub use consensus::ConsensusCell;
#[cfg(feature = "std")]
pub use contract::{Contract, KernelSequence, NumericMode};
#[cfg(feature = "std")]
pub use detector::{DetectorCell, DetectorThresholds};
pub use event::TraceEvent;
pub use fixed::Q16;
pub use grammar::{GrammarState, ReasonCode};
pub use hash::{sha256, Sha256};
pub use motif::{registry_hash, MotifClass, MOTIF_CATALOG};
#[cfg(feature = "std")]
pub use residual::{Baseline, ResidualCell};
#[cfg(feature = "std")]
pub use sign::SignCell;
pub use verdict::FinalVerdict;
#[cfg(feature = "std")]
pub use window::WindowFeature;
