//! Comparator — fires the per-cell predicate against a threshold.
//!
//! `High` fires when statistic > threshold; `Low` fires when
//! statistic < threshold; `TwoSided` fires on either side at
//! their respective thresholds. Used as the fifth coordinate in
//! the algebra grammar.

/// Per-cell comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Comparator {
    /// Fires when statistic > threshold.
    High,
    /// Fires when statistic < threshold.
    Low,
    /// Fires on either side at the respective threshold.
    TwoSided,
}

impl Comparator {
    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Low => "LOW",
            Self::TwoSided => "TWO_SIDED",
        }
    }
}
