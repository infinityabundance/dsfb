//! # Residual Sign (`τ_σ`)
//!
//! Encodes the triple `σ(k) = (‖r(k)‖, ṙ(k), r̈(k))` as defined in Section 2.1 of the
//! DSSC paper. The sign is the fundamental semiotic primitive: it captures magnitude,
//! first-order drift, and second-order slew of the residual at a single time step.
//!
//! The `ResidualSign` type is `Copy` and `Clone` because sign values are passed freely
//! through grammar transitions and provenance tags without ownership transfer.

/// The residual sign triple `σ(k) = (‖r(k)‖, ṙ(k), r̈(k))`.
///
/// All three components are `f64` to support arbitrary normed spaces projected to scalar
/// observables. Multi-channel residuals are handled by `Vec<ResidualSign>` (one per channel).
///
/// # Formal correspondence
/// | Field | Paper notation | Meaning |
/// |-------|---------------|---------|
/// | `magnitude` | `‖r(k)‖` | Residual norm at step k |
/// | `drift` | `ṙ(k) = r(k) − r(k−1)` | First discrete derivative |
/// | `slew` | `r̈(k) = ṙ(k) − ṙ(k−1)` | Second discrete derivative |
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResidualSign {
    /// Residual magnitude `‖r(k)‖ ≥ 0`.
    pub magnitude: f64,
    /// Drift component `ṙ(k)`: positive = moving outward, negative = moving inward.
    pub drift: f64,
    /// Slew component `r̈(k)`: rate of change of drift (acceleration).
    pub slew: f64,
}

impl ResidualSign {
    /// Construct a sign from a scalar residual value and its history.
    ///
    /// `r_prev` and `r_prev2` are `r(k-1)` and `r(k-2)` respectively.
    /// Pass `0.0` for steps before the trajectory begins.
    #[inline]
    pub fn from_scalar(r: f64, r_prev: f64, r_prev2: f64) -> Self {
        let drift = r - r_prev;
        let drift_prev = r_prev - r_prev2;
        Self {
            magnitude: r.abs(),
            drift,
            slew: drift - drift_prev,
        }
    }

    /// Returns `true` if the drift is outward (positive, moving away from origin).
    #[inline]
    pub fn is_drifting_outward(&self) -> bool {
        self.drift > 0.0
    }
}
