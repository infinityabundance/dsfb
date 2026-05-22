//! Four-state grammar used by the episode collapser.
//!
//! Modeled after the `dsfb-debug` grammar (Admissible / Boundary /
//! Violation) plus a `Recovery` state we use specifically for the
//! shock-and-recovery motif. The state machine is intentionally tiny and
//! deterministic — there is no learned transition, no probability, just
//! a closed-form function of the consensus cell's axis values.

/// The grammar's discrete states. Ordering is meaningful: higher values
/// indicate more severity, so a single `max()` over a window yields the
/// peak grammar state for an interval.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default)]
#[repr(u8)]
pub enum GrammarState {
    /// Cell is within the deadband: no axis lit up.
    #[default]
    Admissible = 0,
    /// One or more axes elevated but no single axis at violation level.
    Boundary = 1,
    /// At least one axis crossed the violation threshold; episode-eligible.
    Violation = 2,
    /// Drift descending while norm still above the recovery floor.
    Recovery = 3,
}

/// Reason codes attached to a grammar transition. Carried into the
/// `Episode` so an operator audit can see *why* the grammar flagged a
/// cell, not just that it did.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[repr(u8)]
pub enum ReasonCode {
    /// Sentinel: no reason because the cell is admissible.
    #[default]
    Admissible = 0,
    /// Boundary entered because residual or drift is elevated.
    BoundaryApproach = 1,
    /// Drift sustained above the violation threshold for several windows.
    SustainedOutwardDrift = 2,
    /// Single-window slew shock.
    AbruptSlewViolation = 3,
    /// Multiple boundary cells with no clear violation — graze.
    RecurrentBoundaryGrazing = 4,
    /// Envelope-magnitude violation (norm itself crossed the high band).
    EnvelopeViolation = 5,
    /// Drift descending after a peak — recovery edge.
    DriftWithRecovery = 6,
    /// One-shot boundary crossing that did not re-enter on the next
    /// cell.
    SingleCrossing = 7,
}

impl ReasonCode {
    /// Severity rank used for deterministic tie-break when several
    /// reasons coexist. Higher = more severe.
    #[must_use]
    pub const fn severity(self) -> u8 {
        match self {
            Self::Admissible => 0,
            Self::SingleCrossing => 1,
            Self::BoundaryApproach => 2,
            Self::RecurrentBoundaryGrazing => 3,
            Self::DriftWithRecovery => 4,
            Self::AbruptSlewViolation => 5,
            Self::SustainedOutwardDrift => 6,
            Self::EnvelopeViolation => 7,
        }
    }
}
