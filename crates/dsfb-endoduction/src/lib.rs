//! # dsfb-endoduction
//!
//! Empirical evaluation of the Thermodynamic Precursor Visibility Principle
//! on the NASA IMS bearing run-to-failure dataset using DSFB structural
//! residual analysis.
//!
//! This crate implements a residual-structure analysis pipeline that tests
//! whether structured residual analysis reveals precursor behavior earlier
//! or more clearly than conventional scalar diagnostics. It does **not**
//! prove any thermodynamic law; it evaluates a falsifiable hypothesis on
//! real data.
#![forbid(unsafe_code)]

pub mod admissibility;
pub mod baseline;
pub mod baselines;
pub mod cli;
pub mod data;
pub mod evaluation;
pub mod figures;
pub mod grammar;
pub mod report;
pub mod residual;
pub mod trust;
pub mod types;

pub use types::{Config, RunManifest, WindowMetrics};
