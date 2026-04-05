//! # Motif Descriptor (`τ_m`)
//!
//! A motif is a named structural pattern in the heuristics bank, or the distinguished
//! symbol `Unknown` when no bank entry matches. Section 3.1 of the DSSC paper.

/// A motif descriptor drawn from the heuristics bank, or `Unknown`.
///
/// `Unknown` is not an error: it is an epistemically honest output carrying a complete
/// structural descriptor in the accompanying `ProvenanceTag`. See Proposition 9.1
/// (Day-One Value) and Corollary 5.4 (No Silent Failure) of the DSSC paper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Motif {
    /// A named structural motif recognized from the heuristics bank.
    Named(String),
    /// No bank entry matched. The `ProvenanceTag` in the `Episode` carries the full
    /// structural descriptor `(σ, g, α)`. This is never silence — it is characterized.
    Unknown,
}

impl Motif {
    /// Construct a named motif.
    pub fn named(name: impl Into<String>) -> Self {
        Motif::Named(name.into())
    }

    /// `true` if this is the `Unknown` motif (no bank match).
    pub fn is_unknown(&self) -> bool { matches!(self, Motif::Unknown) }
}

impl std::fmt::Display for Motif {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Motif::Named(n) => write!(f, "{}", n),
            Motif::Unknown  => write!(f, "Unknown"),
        }
    }
}
