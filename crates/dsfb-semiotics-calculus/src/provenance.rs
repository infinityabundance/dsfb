//! # Provenance Tag (`τ_φ`)
//!
//! The provenance tag records the complete derivation tree that licensed a motif:
//! the sign evolution `σ`, grammar path `g`, and ADD invariant descriptor `α`.
//!
//! This is the type-level realization of Theorem 8.3 (Deterministic Auditability):
//! given a `ProvenanceTag` and the original trajectory, any observer re-running the
//! DSSC operational semantics with the same heuristics bank will reproduce the same
//! `Episode` exactly.
//!
//! In DO-178C DAL-A terms, the `ProvenanceTag` is the auditable trace from input to
//! output required for every safety-consequence output. No additional logging
//! infrastructure is required.

use crate::sign::ResidualSign;
use crate::grammar::GrammarState;

/// Provenance tag `φ` — the replayable derivation certificate for one episode.
///
/// Carries:
/// - `sign_sequence`: the observed sign evolution `(σ(k₀), …, σ(k*))`.
/// - `grammar_path`: the grammar state sequence `(g(k₀), …, g(k*))`.
/// - `add_descriptor`: a string encoding of the ADD algebraic invariants `α`.
/// - `step_range`: `(start_k, end_k)` — the trajectory window covered.
#[derive(Debug, Clone)]
pub struct ProvenanceTag {
    /// Observed sign sequence over the episode window.
    pub sign_sequence: Vec<ResidualSign>,
    /// Grammar state sequence over the episode window.
    pub grammar_path: Vec<GrammarState>,
    /// Serialized ADD algebraic invariant descriptor (growth invariant, reachability).
    /// Opaque string representation — structured ADD types belong in `dsfb-add`.
    pub add_descriptor: String,
    /// Trajectory step range `[start_k, end_k]` covered by this episode.
    pub step_range: (usize, usize),
}

impl ProvenanceTag {
    /// Construct a provenance tag from its components.
    pub fn new(
        sign_sequence: Vec<ResidualSign>,
        grammar_path: Vec<GrammarState>,
        add_descriptor: impl Into<String>,
        step_range: (usize, usize),
    ) -> Self {
        Self {
            sign_sequence,
            grammar_path,
            add_descriptor: add_descriptor.into(),
            step_range,
        }
    }

    /// Length of the episode window in steps.
    pub fn window_len(&self) -> usize {
        self.step_range.1.saturating_sub(self.step_range.0) + 1
    }

    /// `true` if the grammar path contains at least one `Violation` state.
    pub fn contains_violation(&self) -> bool {
        self.grammar_path.iter().any(|g| g.is_violation())
    }
}
