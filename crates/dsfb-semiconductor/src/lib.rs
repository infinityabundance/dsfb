//! `dsfb-semiconductor` — deterministic DSFB kernel and SECOM benchmark companion.
//!
//! # Non-Intrusion Guarantees
//!
//! DSFB is a **read-only supervisory system** operating on residual streams.
//!
//! | Guarantee | Enforcement |
//! |-----------|-------------|
//! | No mutation of upstream data | Observer API accepts only `&[f64]` (shared ref) |
//! | No control-path influence | No write path into any upstream data structure |
//! | Deterministic outputs under identical inputs | Pure function composition over fixed params |
//! | Removable without system impact | Advisory outputs only; zero coupling |
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

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

// ── Minimal observer API ───────────────────────────────────────────────────────────────

/// A structured episode produced by the DSFB observer layer.
///
/// Advisory only. No upstream state is modified.
#[derive(Debug, Clone, PartialEq)]
pub struct Episode {
    /// Sample index within the input slice.
    pub index: usize,
    /// Squared residual norm (`|x - nominal|²`), avoiding `sqrt` for kernel compatibility.
    pub residual_norm_sq: f64,
    /// Rolling drift estimate (mean first-difference of absolute residuals over last 5 samples).
    pub drift: f64,
    /// Grammar state: `"Admissible"`, `"Boundary"`, or `"Violation"`.
    pub grammar: &'static str,
    /// Advisory decision: `"Silent"`, `"Review"`, or `"Escalate"`.
    pub decision: &'static str,
}

/// A list of [`Episode`] values returned by [`observe`].
pub type Episodes = Vec<Episode>;

/// Read-only observation of a raw residual slice.
///
/// Accepts a **shared reference only** — no write-back, no upstream coupling,
/// no side effects. Deterministic: identical inputs produce identical outputs.
///
/// NaN or non-finite samples are treated as imputed (missing data) and always
/// return `grammar = "Admissible"`, `decision = "Silent"`.
///
/// # Key guarantees
///
/// - No mutable access to any upstream structure.
/// - Deterministic: identical ordered inputs → identical episode sequence.
/// - No side effects of any kind.
///
/// # Example
///
/// ```
/// let residuals: &[f64] = &[0.1, 0.2, 0.5, 1.2, 2.1];
/// let episodes = dsfb_semiconductor::observe(residuals);
/// for e in &episodes {
///     // advisory only — no write-back, no coupling
///     println!("index={} grammar={} decision={}", e.index, e.grammar, e.decision);
/// }
/// ```
pub fn observe(residuals: &[f64]) -> Episodes {
    if residuals.is_empty() {
        return Episodes::new();
    }

    const DRIFT_WINDOW: usize = 5;

    // Nominal estimate from first 20 % of samples (at least 1, at most all).
    let nominal_len = (residuals.len() / 5).max(1).min(residuals.len());
    let mut nominal_sum = 0.0f64;
    let mut nominal_count = 0usize;
    for x in &residuals[..nominal_len] {
        if x.is_finite() {
            nominal_sum += x;
            nominal_count += 1;
        }
    }
    let nominal_mean = if nominal_count == 0 {
        0.0
    } else {
        nominal_sum / nominal_count as f64
    };

    // Threshold: rho^2 = (3*std)^2 = 9 * var.  Avoids sqrt for no_std compat.
    let mut var_sum = 0.0f64;
    for x in &residuals[..nominal_len] {
        if x.is_finite() {
            let d = x - nominal_mean;
            var_sum += d * d;
        }
    }
    let var = var_sum / nominal_count.max(1) as f64;
    let rho_sq = (9.0 * var).max(1e-18);
    let boundary_rho_sq = 0.25 * rho_sq; // (0.5 * rho)^2

    let mut episodes = Episodes::with_capacity(residuals.len());

    for i in 0..residuals.len() {
        let x = residuals[i];

        // Imputed (NaN / inf) samples never trigger violation.
        if !x.is_finite() {
            episodes.push(Episode {
                index: i,
                residual_norm_sq: 0.0,
                drift: 0.0,
                grammar: "Admissible",
                decision: "Silent",
            });
            continue;
        }

        let r = x - nominal_mean;
        let r_sq = r * r;

        // Rolling drift: mean first-difference of |residual| over last DRIFT_WINDOW samples.
        let drift = if i == 0 {
            0.0
        } else {
            let start = i.saturating_sub(DRIFT_WINDOW);
            let count = i - start;
            let mut d_sum = 0.0f64;
            for j in start..i {
                let a = if residuals[j].is_finite() {
                    (residuals[j] - nominal_mean).abs()
                } else {
                    0.0
                };
                let b = if residuals[j + 1].is_finite() {
                    (residuals[j + 1] - nominal_mean).abs()
                } else {
                    0.0
                };
                d_sum += b - a;
            }
            d_sum / count as f64
        };

        let grammar = if r_sq > rho_sq {
            "Violation"
        } else if r_sq > boundary_rho_sq && drift > 0.0 {
            "Boundary"
        } else {
            "Admissible"
        };

        let decision = match grammar {
            "Violation" => "Escalate",
            "Boundary" => "Review",
            _ => "Silent",
        };

        episodes.push(Episode {
            index: i,
            residual_norm_sq: r_sq,
            drift,
            grammar,
            decision,
        });
    }

    episodes
}

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
