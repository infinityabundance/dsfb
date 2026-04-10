//! Theorem 1: Finite-time envelope exit bound computation.
//!
//! Given sustained outward drift η and envelope expansion rate κ,
//! the theorem bounds the number of cycles until envelope exit:
//!
//!   k* - k₀ ≤ ⌈g_{k₀} / (η - κ)⌉
//!
//! This module computes and reports the bound for comparison with
//! observed grammar-transition timing.

/// Theorem 1 bound computation result.
#[derive(Debug, Clone, Copy)]
pub struct TheoremOneBound {
    /// Initial admissibility gap (g_{k₀}).
    pub initial_gap: f64,
    /// Observed minimum sustained outward drift rate (η).
    pub drift_rate: f64,
    /// Observed maximum envelope expansion rate (κ). Typically 0.0 for fixed envelopes.
    pub envelope_expansion_rate: f64,
    /// Computed upper bound on exit time (cycles from onset of sustained drift).
    pub exit_bound_cycles: u32,
    /// Observed actual transition cycle (if available).
    pub observed_transition_cycle: Option<u32>,
    /// Onset cycle of sustained drift (k₀).
    pub drift_onset_cycle: Option<u32>,
    /// Whether the theorem bound is satisfied (observed ≤ bound).
    pub bound_satisfied: bool,
}

impl TheoremOneBound {
    /// Computes the Theorem 1 bound from observed trajectory statistics.
    ///
    /// # Arguments
    /// - `initial_gap`: admissibility gap at drift onset
    /// - `drift_rate`: minimum sustained outward drift per cycle (η)
    /// - `envelope_expansion_rate`: maximum envelope expansion per cycle (κ)
    /// - `observed_transition`: actual transition cycle (1-based)
    /// - `drift_onset`: cycle at which sustained drift began (1-based)
    #[must_use]
    pub fn compute(
        initial_gap: f64,
        drift_rate: f64,
        envelope_expansion_rate: f64,
        observed_transition: Option<u32>,
        drift_onset: Option<u32>,
    ) -> Self {
        let net_rate = drift_rate - envelope_expansion_rate;
        let exit_bound = if net_rate > 1e-15 {
            let raw = initial_gap / net_rate;
            ceil_nonnegative_to_u32(raw)
        } else {
            u32::MAX // No finite exit under these conditions
        };

        let bound_satisfied = match (observed_transition, drift_onset) {
            (Some(obs), Some(onset)) if obs >= onset => {
                (obs - onset) <= exit_bound
            }
            _ => false,
        };

        Self {
            initial_gap,
            drift_rate,
            envelope_expansion_rate,
            exit_bound_cycles: exit_bound,
            observed_transition_cycle: observed_transition,
            drift_onset_cycle: drift_onset,
            bound_satisfied,
        }
    }
}

/// Computes `ceil(x)` for finite nonnegative values without relying on `std`.
#[must_use]
fn ceil_nonnegative_to_u32(x: f64) -> u32 {
    if !x.is_finite() {
        return u32::MAX;
    }
    if x <= 0.0 {
        return 0;
    }

    let truncated = x as u32;
    if x > truncated as f64 {
        truncated.saturating_add(1)
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theorem_bound() {
        let bound = TheoremOneBound::compute(
            10.0,  // initial gap
            0.5,   // drift rate
            0.0,   // no envelope expansion
            Some(25), // observed at cycle 25
            Some(5),  // drift onset at cycle 5
        );
        // Bound: ceil(10.0 / 0.5) = 20 cycles
        assert_eq!(bound.exit_bound_cycles, 20);
        // Observed: 25 - 5 = 20 <= 20 → satisfied
        assert!(bound.bound_satisfied);
    }

    #[test]
    fn test_theorem_bound_not_satisfied() {
        let bound = TheoremOneBound::compute(
            5.0,
            0.1,
            0.0,
            Some(100),
            Some(5),
        );
        // Bound: ceil(5.0 / 0.1) = 50
        // Observed: 100 - 5 = 95 > 50 → not satisfied
        assert!(!bound.bound_satisfied);
    }
}
