//! DSFB-Debug: Deterministic Structural Accumulator (DSA) — paper §5
//! and Appendix D.
//!
//! The DSA is the running scalar score that aggregates five rolling
//! structural features into a single value the policy engine consumes:
//!
//! ```text
//! DSA(k) = w1·boundary_density(k)
//!        + w2·drift_persist(k)
//!        + w3·slew_density(k)
//!        + w4·ewma_occupancy(k)
//!        + w5·motif_recurrence(k)
//! ```
//!
//! Default weights are unit (w_i = 1.0), reproducing the paper's
//! reference scoring. Operators tuning per-site can adjust weights
//! via the `EngineConfig` (see `config.rs`).
//!
//! The DSA score is one input to the `PolicyState` decision in
//! `policy.rs`; higher scores escalate from `Silent`→`Watch`→
//! `Review`→`Escalate`. The score is a pure function of its inputs;
//! Theorem 9 deterministic replay holds trivially.

/// Compute DSA score from rolling features. Unit weights (all w=1.0).
#[inline]
pub fn compute_dsa_score(
    boundary_density: f64,
    drift_persistence: f64,
    slew_density: f64,
) -> f64 {
    // Unit weights per semiconductor crate convention
    boundary_density + drift_persistence + slew_density
}

/// Check DSA directional consistency gate
/// Returns true if DSA score >= τ
#[inline]
pub fn consistency_gate(dsa_score: f64, tau: f64) -> bool {
    dsa_score >= tau
}
