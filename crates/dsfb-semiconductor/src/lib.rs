//! `dsfb-semiconductor` — deterministic DSFB kernel and SECOM benchmark companion.
//!
//! # Feature flags
//!
//! | Feature | Default | Effect |
//! |---------|---------|--------|
//! | `std`   | yes     | Enables CLI, I/O, plotting, networking, and all dataset adapters. |
//! | *(none)* | —      | Kernel-only build: sign, grammar, syntax, semantics, policy, process_context, units. Suitable for bare-metal / RTOS / FPGA deployments. |
//!
//! # `no_std` kernel surface
//!
//! When compiled with `--no-default-features`, the following modules are
//! available and require only `alloc`:
//!
//! - [`process_context`] — recipe-step admissibility LUT and maintenance hysteresis
//! - [`units`] — type-safe physical quantity newtypes
//! - [`signs`] — residual sign computation (drift, slew)
//! - [`sign`] — streaming sign point construction
//! - [`grammar`] — three-state admissibility FSM with hysteresis
//! - [`grammar::layer`] — six-state streaming grammar
//! - [`syntax`] — motif classifier
//! - [`policy`] — decision ranking
//! - [`semantics`] — heuristics bank lookup
//! - [`config`] — pipeline configuration
//! - [`nominal`] — healthy-window model
//! - [`residual`] — residual set construction
//! - [`input`] — residual and alarm stream types

#![cfg_attr(not(feature = "std"), no_std)]

// When std is disabled, pull in alloc for Vec, String, BTreeMap, format!, etc.
#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

// ── Kernel modules (always compiled) ───────────────────────────────────────────────────
pub mod config;
pub mod grammar;
pub mod input;
pub mod nominal;
pub mod policy;
pub mod process_context;
pub mod residual;
pub mod semantics;
pub mod sign;
pub mod signs;
pub mod syntax;
pub mod units;

// ── std-only modules ────────────────────────────────────────────────────────────
#[cfg(feature = "std")]
pub mod baselines;
#[cfg(feature = "std")]
pub mod calibration;
#[cfg(feature = "std")]
pub mod cli;
#[cfg(feature = "std")]
pub mod cohort;
#[cfg(feature = "std")]
pub mod dataset;
#[cfg(feature = "std")]
pub mod error;
#[cfg(feature = "std")]
pub mod failure_driven;
#[cfg(feature = "std")]
pub mod heuristics;
#[cfg(feature = "std")]
pub mod interface;
#[cfg(feature = "std")]
pub mod metrics;
#[cfg(feature = "std")]
pub mod missingness;
#[cfg(feature = "std")]
pub mod multivariate_observer;
#[cfg(feature = "std")]
pub mod non_intrusive;
#[cfg(feature = "std")]
pub mod output_paths;
#[cfg(feature = "std")]
pub mod phm2018_loader;
#[cfg(feature = "std")]
pub mod pipeline;
#[cfg(feature = "std")]
pub mod plots;
#[cfg(feature = "std")]
pub mod precursor;
#[cfg(feature = "std")]
pub mod preprocessing;
#[cfg(feature = "std")]
pub mod report;
#[cfg(feature = "std")]
pub mod secom_addendum;
#[cfg(feature = "std")]
pub mod semiotics;
#[cfg(feature = "std")]
pub mod signature;
#[cfg(feature = "std")]
pub mod traceability;
#[cfg(feature = "std")]
pub mod unified_value_figure;
