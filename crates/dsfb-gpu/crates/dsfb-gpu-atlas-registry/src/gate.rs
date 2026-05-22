//! Gate — additional firing-gate applied after the comparator.
//!
//! A gate filters cells that would otherwise fire — examples: a
//! persistence gate that requires N consecutive firings before
//! admission, a clean-window-suppression gate that requires the
//! cell's neighbours to be quiet, etc. Pairs with persistence
//! (`P{N}` in the canonical name).
//!
//! At S1.1 the gate is an enum tag plus a `persistence_windows`
//! `u32` field carried by `DetectorTemplate` separately. Future
//! Atlas commits may attach richer per-gate parameters.

/// Per-cell firing gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gate {
    /// No gate; the comparator's decision passes through.
    None,
    /// Require `persistence_windows` consecutive firings.
    Persistence,
    /// Require clean cells on both sides of the firing range.
    CleanWindowBracket,
    /// Confuser-suppression gate — fires only when the confuser
    /// witness is NOT also firing.
    ConfuserSuppression,
}

impl Gate {
    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Persistence => "PERSISTENCE",
            Self::CleanWindowBracket => "CLEAN_WINDOW_BRACKET",
            Self::ConfuserSuppression => "CONFUSER_SUPPRESSION",
        }
    }
}
