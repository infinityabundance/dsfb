//! # dsfb-robotics — DSFB Structural Semiotics Engine for Robotics Health Monitoring
//!
//! **What this crate is, in one paragraph.** A deterministic, `no_std`,
//! `no_alloc`, zero-`unsafe` *observer* that reads residual streams — joint
//! torque identification residuals, inverse-dynamics residuals, whole-body
//! MPC force residuals, centroidal-momentum observer residuals, bearing
//! envelope-spectrum residuals, health-index trajectories — which existing
//! robot control and prognostics pipelines already compute, and structures
//! them into a typed grammar of human-readable episodes. DSFB does **not**
//! replace inverse-dynamics identification, Kalman / Luenberger observers,
//! whole-body controllers, MPC, rainflow RUL estimators, or vibration-based
//! FDD classifiers — it **augments** them by giving operators a structural
//! view of what those systems discard. Removing DSFB leaves the upstream
//! control and prognostics stack unchanged.
//!
//! ---
//!
//! **Invariant Forge LLC** — Prior art under 35 U.S.C. § 102.
//! Commercial deployment requires a separate written license.
//! Reference implementation: Apache-2.0.
//! <licensing@invariantforge.net>
//!
//! ## Positioning — Augmentation, not competition
//!
//! DSFB **does not compete** with existing robotics sensing, kinematic
//! identification, whole-body balance control, or PHM methods. Existing
//! methods will continue to outperform DSFB at their own tasks — earlier
//! fault detection, lower false-alarm rates, better RUL accuracy, tighter
//! tracking control. DSFB's role is orthogonal: it reads the **residuals
//! those methods already produce and usually discard**, and structures
//! them into a human-readable grammar (Admissible / Boundary / Violation)
//! with typed episodes and provenance-tagged audit trails.
//!
//! This makes existing methods **more important**, not less — DSFB is
//! literally dependent on a functioning upstream observer chain to have
//! anything to interpret.
//!
//! ## Architectural Contract
//!
//! - **Observer-only.** Public API accepts `&[f64]` (immutable reference
//!   only). There is no mutable write path into any upstream data
//!   structure. Enforced by type signature.
//! - **`#![no_std]`.** Core modules link against neither the Rust standard
//!   library nor any OS runtime. Deployable on bare-metal MCUs (Cortex-M4F,
//!   RISC-V 32-bit) alongside a safety-gate companion to an industrial
//!   robot controller.
//! - **`no_alloc` in core.** All internal structures use fixed-capacity
//!   array-backed types. The canonical [`observe`] signature takes a
//!   caller-supplied `&mut [Episode]` output buffer. No heap allocation in
//!   any hot path of the default build.
//! - **Zero `unsafe`.** No `unsafe` blocks, no `UnsafeCell`, no `RefCell`
//!   in any observer code path. Enforced at compile time by
//!   `#![forbid(unsafe_code)]` below.
//!
//! ## Non-Claims (from companion paper §11)
//!
//! This crate does **not** provide:
//! - Fault classification (bearing fault type, root-cause identification)
//! - Calibrated Pd/Pfa or F1/ROC-AUC guarantees
//! - Earlier detection than incumbent threshold alarms, RMS monitors, or
//!   CUSUM/EWMA change-point detectors
//! - Hard real-time latency bounds under specific controller platforms
//! - RUL (remaining useful life) prediction
//! - ISO 10218-1/-2:2025 or IEC 61508 certification
//! - A replacement for any upstream observer, estimator, or controller
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | *(none)* | Core engine: `no_std` + `no_alloc` + zero unsafe |
//! | `alloc` | Opt-in heap via `alloc` crate for host-side convenience wrappers |
//! | `std` | Opt-in std library for pipeline and output modules |
//! | `serde` | JSON artefact serialization (requires `std`) |
//! | `paper_lock` | Headline-metric enforcement for deterministic reproducibility |
//! | `real_figures` | Real-dataset figure bank for the companion paper (requires `std`) |
//! | `experimental` | Exploratory extensions not validated in the companion paper |
//!
//! ## Minimal usage (bare-metal, `no_std` + `no_alloc`)
//!
//! ```
//! use dsfb_robotics::{Episode, observe};
//! let residuals: &[f64] = &[0.01, 0.02, 0.05, 0.12, 0.21];
//! let mut out = [Episode::empty(); 16];
//! let n = observe(residuals, &mut out);
//! for e in &out[..n] {
//!     // advisory only — no write-back, no upstream coupling
//!     let _ = (e.index, e.grammar, e.decision);
//! }
//! ```
//!
//! ## Streaming engine usage (per-observation API)
//!
//! ```
//! use dsfb_robotics::engine::DsfbRoboticsEngine;
//! use dsfb_robotics::platform::RobotContext;
//!
//! // W=8 drift window, K=4 persistence threshold
//! let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
//!
//! let residual_norm: f64 = 0.045; // ‖r(k)‖ from your upstream observer
//! let ep = eng.observe_one(residual_norm, false, RobotContext::ArmOperating, 0);
//! let _ = (ep.grammar, ep.decision);
//! // upstream robot controller: UNCHANGED
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// ---------------------------------------------------------------
// Conditional std/alloc imports — core does not require either.
// ---------------------------------------------------------------
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// ---------------------------------------------------------------
// Core modules — unconditionally no_std + no_alloc + zero unsafe
// ---------------------------------------------------------------

/// `libm`-free f64 helpers for `no_std` + `no_alloc` core.
pub mod math;

/// Robot operating context: commissioning, operating, stance, swing, maintenance.
pub mod platform;

/// Residual sign tuple σ(k) = (‖r‖, ṙ, r̈).
pub mod sign;

/// Admissibility envelope `E(k) = {r : ‖r‖ ≤ ρ(k)}`.
pub mod envelope;

/// Grammar FSM: `Admissible | Boundary[ReasonCode] | Violation`.
pub mod grammar;

/// Canonical [`Episode`] struct emitted by the observer.
pub mod episode;

/// Advisory policy layer: grammar → decision.
pub mod policy;

/// Heuristics bank: typed robotics motifs.
pub mod heuristics;

/// Syntax layer: classify sign tuples into named motifs (see
/// [`heuristics::RoboticsMotif`] for the typed motif catalogue).
pub mod syntax;

/// Shared residual helper for kinematic-identification datasets.
pub mod kinematics;

/// Shared residual helper for balancing datasets.
pub mod balancing;

/// Healthy-window envelope calibration.
pub mod calibration;

/// Wide-sense-stationarity check for calibration windows.
pub mod stationarity;

/// Uncertainty budget per GUM JCGM 100:2008.
pub mod uncertainty;

/// Streaming DSFB engine orchestrator. See
/// [`engine::DsfbRoboticsEngine`] and [`grammar::GrammarEvaluator`]
/// for the canonical per-sample pipeline.
pub mod engine;

/// Per-dataset residual adapters across PHM (CWRU, IMS, FEMTO-ST),
/// kinematics (KUKA LWR-IV+, Franka Panda Gaz, DLR-class
/// Giacomuzzo, UR10 Polydoros), and balancing (MIT Mini-Cheetah,
/// iCub push-recovery, ANYmal, Unitree G1, ergoCub Sorrentino,
/// plus the LeRobot ALOHA / Mobile-ALOHA / SO-100 / DROID / OpenX
/// teleoperation slates). See [`datasets::DatasetId`] for the
/// canonical slug enumeration.
pub mod datasets;

/// Paper-lock driver: per-dataset DSFB evaluation, deterministic
/// JSON emission, bit-exact reproducibility gate. Feature-gated on
/// `paper_lock` (which pulls in `std` + `serde` + `serde_json`).
#[cfg(feature = "paper_lock")]
pub mod paper_lock;

// Kani formal-verification harnesses — compiled only when the crate is
// built with `#[cfg(kani)]` (which Kani itself sets). Invisible in
// stock `cargo build` output. See `src/kani_proofs.rs` for the
// harness inventory.
#[cfg(kani)]
mod kani_proofs;

// ---------------------------------------------------------------
// Public flat re-exports — the most-used types at crate root so that
// `use dsfb_robotics::{Episode, observe, GrammarState, DsfbRoboticsEngine};`
// is idiomatic.
// ---------------------------------------------------------------

pub use crate::engine::DsfbRoboticsEngine;
pub use crate::envelope::AdmissibilityEnvelope;
pub use crate::episode::Episode;
pub use crate::grammar::{GrammarState, ReasonCode};
pub use crate::platform::RobotContext;
pub use crate::policy::PolicyDecision;
pub use crate::sign::{SignTuple, SignWindow};

// ---------------------------------------------------------------
// Top-level convenience observe()
// ---------------------------------------------------------------

/// Read-only one-shot DSFB observation of a residual slice.
///
/// Constructs a default-parameter engine (`W = 8`, `K = 4`, envelope
/// radius ρ calibrated from the **first 20 %** of the input under the
/// paper's Stage III protocol) and streams `residuals` into `out`.
///
/// Returns the number of episodes written. Never writes past
/// `out.len()`. Callers that need a custom drift window, persistence
/// threshold, or a pre-computed envelope should use
/// [`DsfbRoboticsEngine`] directly.
///
/// This is the advertised `no_alloc` entry point:
/// `observe(&[f64], &mut [Episode]) -> usize`.
///
/// # Determinism
///
/// Pure function; identical ordered inputs produce identical outputs.
/// No global state, no allocation, no side effects, no panic paths.
///
/// # Non-finite input samples
///
/// Treated as below-floor (missingness-aware): they always produce
/// `grammar = "Admissible"`, `decision = "Silent"` and are not
/// counted toward drift or envelope statistics.
///
/// # Edge cases
///
/// - Empty input or empty output buffer → `0`.
/// - Calibration window (first 20 %) contains no finite samples →
///   all episodes `Admissible` / `Silent` (the engine runs with a
///   zero-radius envelope, which is then suppressed by the
///   non-finite-input fall-through).
pub fn observe(residuals: &[f64], out: &mut [Episode]) -> usize {
    debug_assert!(residuals.len() <= usize::MAX / 2, "residuals slice unreasonably large");
    debug_assert!(out.len() <= usize::MAX / 2, "output buffer unreasonably large");

    if residuals.is_empty() || out.is_empty() {
        return 0;
    }

    // Stage III calibration: use the first 20 % of the input as the
    // healthy window (bounded below at 1, above at residuals.len()).
    let cal_len = (residuals.len() / 5).max(1).min(residuals.len());
    let cal_slice = &residuals[..cal_len];

    // Compute finite-valued norms for calibration.
    let mut cal_buf = [0.0_f64; 64];
    let mut cal_n = 0_usize;
    let mut i = 0_usize;
    while i < cal_slice.len() && cal_n < cal_buf.len() {
        let x = cal_slice[i];
        if x.is_finite() {
            cal_buf[cal_n] = crate::math::abs_f64(x);
            cal_n += 1;
        }
        i += 1;
    }
    let envelope = if cal_n == 0 {
        // Fall back to a permissive envelope; the non-finite fall-through
        // in the engine will still produce all Admissible episodes.
        AdmissibilityEnvelope::new(f64::INFINITY)
    } else {
        AdmissibilityEnvelope::calibrate_from_window(&cal_buf[..cal_n])
            .unwrap_or_else(|| AdmissibilityEnvelope::new(f64::INFINITY))
    };

    let mut eng = DsfbRoboticsEngine::<8, 4>::from_envelope(envelope);
    eng.observe(residuals, out, RobotContext::ArmOperating)
}

// ---------------------------------------------------------------
// Top-level smoke tests — crate-level invariants
// ---------------------------------------------------------------
#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn empty_input_returns_zero_episodes() {
        let mut out = [Episode::empty(); 4];
        assert_eq!(observe(&[], &mut out), 0);
    }

    #[test]
    fn empty_output_buffer_returns_zero() {
        let mut out: [Episode; 0] = [];
        assert_eq!(observe(&[0.1, 0.2, 0.3], &mut out), 0);
    }

    #[test]
    fn calibration_then_drop_is_admissible() {
        // Calibration window (first 20 %) has residual magnitude around
        // 0.01 → envelope radius ρ ≈ 0.01. Subsequent samples at 0.001
        // are well below the boundary-approach band (0.5·ρ = 0.005)
        // and must stay Admissible / Silent.
        let mut residuals = [0.001_f64; 32];
        for v in residuals.iter_mut().take(6) {
            *v = 0.01;
        }
        let mut out = [Episode::empty(); 32];
        let n = observe(&residuals, &mut out);
        assert_eq!(n, 32);
        // After the calibration samples settle, steady-state residuals
        // below the boundary band must be Admissible / Silent.
        let tail_admissible = out[10..n].iter().all(|e| e.grammar == "Admissible");
        assert!(tail_admissible, "tail episodes must be Admissible once residuals drop below boundary band");
    }

    #[test]
    fn observe_respects_output_capacity() {
        let residuals = [0.02_f64; 32];
        let mut small = [Episode::empty(); 4];
        let n = observe(&residuals, &mut small);
        assert_eq!(n, 4);
    }

    #[test]
    fn episode_index_matches_input_position() {
        let residuals = [0.02_f64; 16];
        let mut out = [Episode::empty(); 16];
        let n = observe(&residuals, &mut out);
        for (i, e) in out[..n].iter().enumerate() {
            assert_eq!(e.index, i);
        }
    }

    #[test]
    fn stepwise_jump_eventually_escalates() {
        // Calibration window (first 20 %, i.e. 0.01 constants) → rho ≈ 0.01.
        // Followed by 0.5 residuals → escalated.
        let mut residuals = [0.01_f64; 32];
        for v in &mut residuals[6..] {
            *v = 0.5;
        }
        let mut out = [Episode::empty(); 32];
        let n = observe(&residuals, &mut out);
        assert_eq!(n, 32);
        let escalated = out[..n].iter().filter(|e| e.decision == "Escalate").count();
        assert!(escalated >= 20, "expected many Escalate episodes, got {}", escalated);
    }

    #[test]
    fn non_finite_inputs_stay_admissible() {
        let residuals = [f64::NAN; 16];
        let mut out = [Episode::empty(); 16];
        let n = observe(&residuals, &mut out);
        assert_eq!(n, 16);
        for e in &out[..n] {
            assert_eq!(e.grammar, "Admissible");
        }
    }
}
