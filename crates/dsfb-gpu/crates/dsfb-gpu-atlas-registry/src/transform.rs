//! Transform — the second algebra coordinate.
//!
//! The transform names the input signal a detector consumes
//! BEFORE its statistic is computed. Pairs with
//! [`crate::DetectorFamily`] in the algebra grammar:
//!
//! ```text
//!   {FAMILY}__{TRANSFORM}__W{WINDOW}__{STATISTIC}__{COMPARATOR}__P{PERSISTENCE}
//! ```
//!
//! Wire names are uppercase snake-case constants — no `Debug`
//! derive coupling so a Rust-version variant rename cannot
//! silently shift the canonical naming.

/// Pre-statistic signal transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transform {
    /// Raw signal value (no transformation).
    Raw,
    /// Residual (signal minus baseline).
    Residual,
    /// First-difference / drift signal.
    Drift,
    /// Slew (rate-of-change) signal.
    Slew,
    /// Absolute value of the signal.
    Abs,
    /// Sign of the signal (sgn).
    Signed,
    /// Squared signal.
    Squared,
    /// Logarithmic transform (defined where applicable).
    Log,
}

impl Transform {
    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::Raw => "RAW",
            Self::Residual => "RESIDUAL",
            Self::Drift => "DRIFT",
            Self::Slew => "SLEW",
            Self::Abs => "ABS",
            Self::Signed => "SIGNED",
            Self::Squared => "SQUARED",
            Self::Log => "LOG",
        }
    }
}
