//! # Episode (`τ_m × τ_φ`)
//!
//! A typed episode is the pair `(m, φ)` produced by the endoductive operator `ℰ`.
//! Section 3.1 of the DSSC paper.

use crate::motif::Motif;
use crate::provenance::ProvenanceTag;

/// A typed episode: the output of the endoductive operator `ℰ(r, σ, g, h) = (m, φ)`.
///
/// Every `Episode` is either a named motif or `Unknown`, always with a complete
/// provenance tag. There is no "null" episode — the type enforces No Silent Failure
/// (Corollary 5.4 of the DSSC paper).
#[derive(Debug, Clone)]
pub struct Episode {
    /// The motif descriptor: named structural pattern or `Unknown`.
    pub motif: Motif,
    /// The provenance tag: complete, replayable derivation certificate.
    pub provenance: ProvenanceTag,
}

impl Episode {
    /// Construct an episode from a motif and provenance tag.
    pub fn new(motif: Motif, provenance: ProvenanceTag) -> Self {
        Self { motif, provenance }
    }

    /// `true` if the motif is `Unknown` (no bank match; structural descriptor available).
    pub fn is_unknown(&self) -> bool { self.motif.is_unknown() }
}

impl std::fmt::Display for Episode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Episode({}, steps {}–{})",
            self.motif,
            self.provenance.step_range.0,
            self.provenance.step_range.1)
    }
}
