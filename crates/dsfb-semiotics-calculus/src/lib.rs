//! # DSFB Structural Semiotics Calculus (`dsfb-semiotics-calculus`)
//!
//! **Invariant Forge LLC** — April 2026 — Version 0.1.0
//!
//! This crate is the Rust type-level realization of the *DSFB Structural Semiotics Calculus*
//! (DSSC), a typed algebraic framework for deterministic, non-interfering, auditable
//! structural interpretation of residual trajectories.
//!
//! ## What this crate provides
//!
//! The crate encodes the core DSSC types and traits exactly as they appear in the companion
//! paper *"DSFB Structural Semiotics Calculus: Formal Syntax, Composition Rules, and Provable
//! Properties of Endoductive Inference over Residual Trajectories"* (Invariant Forge LLC,
//! DOI: 10.5281/zenodo.19446580). The type system enforces at compile time:
//!
//! - **Non-interference** (SC-2): `Observer` is a pure function over `Trajectory`; it holds
//!   no mutable references to the observed system.
//! - **Totality** (SC-1): `Enduce::enduce` returns `Episode` for every input, never panics,
//!   never returns `None`. An empty heuristics bank yields `Motif::Unknown`.
//! - **Auditability** (SC-3): Every `Episode` carries a `ProvenanceTag` recording the full
//!   `(sign_sequence, grammar_path, add_descriptor)` derivation.
//!
//! ## IP Notice
//!
//! The Apache 2.0 license applies to this software artifact as an executable and distributable
//! work. It does not constitute a license to the underlying theoretical framework, mathematical
//! architecture, formal constructions, or supervisory methods described in the companion paper,
//! which constitute proprietary Background IP of Invariant Forge LLC (Delaware LLC No. 10529072).
//! Commercial deployment requires a separate written license. Inquiries: licensing@invariantforge.net

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]

pub mod sign;
pub mod envelope;
pub mod grammar;
pub mod motif;
pub mod provenance;
pub mod episode;
pub mod enduce;
pub mod bank;
pub mod observer;
pub mod composition;
pub mod figures;

// Re-export the primary public API
pub use sign::ResidualSign;
pub use envelope::{AdmissibilityEnvelope, EnvelopeFamily};
pub use grammar::{GrammarState, GrammarFsm};
pub use motif::Motif;
pub use provenance::ProvenanceTag;
pub use episode::Episode;
pub use enduce::Enduce;
pub use bank::{HeuristicsBank, MotifPattern};
pub use observer::Observer;
