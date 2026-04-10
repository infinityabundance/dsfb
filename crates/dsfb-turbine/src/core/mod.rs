//! Core DSFB engine — `no_std`, `no_alloc`, `no_unsafe`.
//!
//! This module contains the complete interpretive engine:
//! - Residual sign computation (residual, drift, slew)
//! - Admissibility envelope evaluation
//! - Grammar-state machine (Admissible / Boundary / Violation)
//! - Reason-code classification
//! - Heuristics bank (typed degradation motifs)
//! - Operating-regime classification
//! - Audit-trace generation
//! - Sensitivity sweep support
//! - Theorem-1 finite-exit bound computation
//!
//! All types in this module use fixed-size stack allocation.
//! No `Vec`, `String`, `Box`, `HashMap`, or any heap type is used.

pub mod residual;
pub mod envelope;
pub mod grammar;
pub mod heuristics;
pub mod regime;
pub mod episode;
pub mod audit;
pub mod config;
pub mod theorem;
pub mod sensitivity;
pub mod channels;

// ─── Re-exports for ergonomic use ──────────────────────────────────
pub use residual::ResidualSign;
pub use envelope::AdmissibilityEnvelope;
pub use grammar::{GrammarState, GrammarEngine};
pub use heuristics::{EngineReasonCode, HeuristicsBank};
pub use regime::OperatingRegime;
pub use episode::Episode;
pub use audit::AuditEntry;
pub use config::DsfbConfig;
pub use theorem::TheoremOneBound;
pub use channels::{ChannelId, SensorReading};
