// DSFB Oil & Gas — Library Root
//
// Deterministic, read-only residual structuring framework for upstream
// and midstream oil and gas systems.
//
// # Positioning
// This crate does NOT replace RTTM, Kalman filters, SPC/CUSUM, ML
// condition-monitoring systems, or any SCADA/DCS/historian system.
// It structures the residuals those systems already produce into typed,
// human-readable grammar episodes.
//
// # Feature flags
// | Feature | Enables |
// |---------|---------|
// | *(none)*| Pure `no_std`/`no_alloc` core |
// | `alloc` | Vec/String types; grammar engine, episode aggregation, domain frames |
// | `std` (default, implies `alloc`) | CSV loaders, error types, report formatting, binary |
//
// # TRL: 3.  Computational validation on real Petrobras 3W (9,087 steps, 12 instances,
// # 6 fault types), Equinor Volve 15/9-F-15 (5,326 depth-steps, TQA), and RPDBCS
// # ESPset (6,032 vibration snapshots, 11 ESP units).  Hardware-in-the-loop and
// # field-trial validation are Phase II (TRL 4-5).

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// ── Core modules (always available, no heap, no unsafe) ──────────────────────
pub mod types;
pub mod residual;
pub mod envelope;
pub mod grammar;
pub mod integration;

// ── Alloc-gated modules ───────────────────────────────────────────────────────
#[cfg(feature = "alloc")]
pub mod events;
#[cfg(feature = "alloc")]
pub mod oilwell;
#[cfg(feature = "alloc")]
pub mod pipeline;
#[cfg(feature = "alloc")]
pub mod subsea;
#[cfg(feature = "alloc")]
pub mod drilling;
#[cfg(feature = "alloc")]
pub mod drilling_real;
#[cfg(feature = "alloc")]
pub mod rotating;
#[cfg(feature = "alloc")]
pub mod rotating_real;
#[cfg(feature = "alloc")]
pub mod report;

// ── Std-gated modules ─────────────────────────────────────────────────────────
#[cfg(feature = "std")]
pub mod loaders;
#[cfg(feature = "std")]
pub mod error;
#[cfg(feature = "std")]
pub mod figure_pipeline;
#[cfg(feature = "std")]
pub mod figure_traces;

// ── Kani formal-verification harnesses (compiled only by `cargo kani`) ────────
#[cfg(kani)]
mod kani_proofs;

// ── Re-exports — core (always) ────────────────────────────────────────────────
pub use types::{AdmissibilityEnvelope, GrammarState, ReasonCode, ResidualTriple};
pub use residual::SlewEstimator;
pub use envelope::{evaluate, CoordClass, EnvelopeEval};
pub use grammar::GrammarClassifier;
pub use integration::NonIntrusiveGuarantee;

// ── Re-exports — alloc ────────────────────────────────────────────────────────
#[cfg(feature = "alloc")]
pub use types::{AnnotatedStep, DsfbDomainFrame, Episode, EpisodeSummary, ResidualSample};
#[cfg(feature = "alloc")]
pub use residual::{DriftEstimator, ResidualProcessor};
#[cfg(feature = "alloc")]
pub use grammar::{DeterministicDsfb, DsfbEngine};
#[cfg(feature = "alloc")]
pub use integration::{deterministic_replay, process_read_only, ReadOnlySlice};
#[cfg(feature = "alloc")]
pub use events::{aggregate_episodes, episodes_to_csv, summarise};
#[cfg(feature = "alloc")]
pub use report::{format_episodes_table, format_summary, noise_compression_ratio};
#[cfg(feature = "alloc")]
pub use oilwell::OilwellFrame;
#[cfg(feature = "alloc")]
pub use pipeline::PipelineFrame;
#[cfg(feature = "alloc")]
pub use subsea::SubseaFrame;
#[cfg(feature = "alloc")]
pub use drilling::DrillingFrame;
#[cfg(feature = "alloc")]
pub use drilling_real::VolveFrame;
#[cfg(feature = "alloc")]
pub use rotating::RotatingFrame;
#[cfg(feature = "alloc")]
pub use rotating_real::EspFrame;

// ── Re-exports — std ─────────────────────────────────────────────────────────
#[cfg(feature = "std")]
pub use loaders::{load_drilling_csv, load_esp_csv, load_oilwell_csv, load_pipeline_csv, load_rotating_csv, load_subsea_csv, load_volve_csv};
#[cfg(feature = "std")]
pub use error::DsfbError;
#[cfg(feature = "std")]
pub use figure_pipeline::{generate_all_figures, FigureGenerationResult};
#[cfg(feature = "std")]
pub use figure_traces::{export_grammar_traces, GrammarTraceExport};
