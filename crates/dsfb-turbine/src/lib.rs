//! # DSFB Structural Semiotics Engine for Gas Turbine Jet Engine Health Monitoring
//!
//! A deterministic, read-only, observer-only augmentation layer for typed residual
//! interpretation over existing Engine Health Monitoring (EHM), Gas Path Analysis (GPA),
//! and Prognostics and Health Management (PHM) systems.
//!
//! ## Architectural Contract
//!
//! 1. **Read-only**: All input data is accepted as `&[f64]` immutable slices.
//!    No mutable reference to upstream data is ever created.
//! 2. **Non-interfering**: No write path, callback, or feedback channel exists
//!    from DSFB to any upstream EHM/FADEC/GPA system.
//! 3. **Deterministic**: No random seeds, no stochastic sampling, no training-dependent
//!    weights. Given identical inputs and configuration, outputs are identical.
//! 4. **no_std / no_alloc**: The `core` module operates without heap allocation,
//!    suitable for embedded avionics and safety-critical environments.
//! 5. **no_unsafe**: `#![forbid(unsafe_code)]` is enforced for the library crate.
//!
//! ## Build Boundary
//!
//! This crate uses the common split architecture:
//! - `core`: `no_std`, `no_alloc`, deterministic inference primitives
//! - `dataset`, `pipeline`, `figures`, `report`: std-gated evaluation tooling
//!
//! ## Non-Interference Invariant
//!
//! If DSFB is removed, the upstream EHM/GPA/FADEC system is unchanged.
//! DSFB does not modify, replace, or interact with any engine control,
//! protection, or estimation logic.
//!
//! ## What DSFB Does NOT Do
//!
//! - Does not predict Remaining Useful Life (RUL)
//! - Does not modify FADEC, EHM, or GPA systems
//! - Does not change thrust management, fuel scheduling, or control loops
//! - Does not replace Kalman filters, particle filters, or any estimation pipeline
//! - Does not claim superiority over any incumbent method
//!
//! ## What DSFB Does
//!
//! - Converts health-parameter residual streams into typed structural episodes
//! - Provides deterministic, auditable reason codes for each classification
//! - Formalizes regime-conditioned admissibility envelopes
//! - Enables early-warning detection of structural degradation trajectory changes
//! - Produces audit-ready trace chains from raw residual to typed conclusion

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

// ═══════════════════════════════════════════════════════════════════════
// CORE ENGINE — no_std, no_alloc, always compiled
// ═══════════════════════════════════════════════════════════════════════
pub mod core;

// ═══════════════════════════════════════════════════════════════════════
// STD-GATED MODULES — dataset loading, evaluation pipelines, figures, reporting
// ═══════════════════════════════════════════════════════════════════════
#[cfg(feature = "std")]
pub mod dataset;
#[cfg(feature = "std")]
pub mod pipeline;
#[cfg(feature = "std")]
pub mod figures;
#[cfg(feature = "std")]
pub mod report;

/// Crate version for paper-lock and reproducibility.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Paper DOI for reproducibility lock.
pub const PAPER_DOI: &str = "TBD-zenodo-gas-turbine";

/// Non-interference contract version.
pub const NON_INTERFERENCE_CONTRACT: &str = "v1.0-read-only-observer-only";
