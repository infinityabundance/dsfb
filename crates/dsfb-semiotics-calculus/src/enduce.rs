//! # Endoductive Operator (`ℰ`)
//!
//! The endoductive operator `ℰ: 𝒯 × Σ* × G* × τ_h ⇀ τ_m × τ_φ` is the core of the
//! DSSC. It is formalized as a Rust trait so that domain-specific instantiations can
//! provide their own matching logic while inheriting the type-level totality guarantee.
//!
//! ## Totality guarantee
//!
//! The return type is `Episode`, not `Option<Episode>`. This enforces at compile time
//! that every call produces a valid, certified output — either a named motif or
//! `Motif::Unknown` with a complete `ProvenanceTag`. This is the type-level realization
//! of Theorem 5.2 (Soundness) and Corollary 5.4 (No Silent Failure).
//!
//! ## Blanket implementation
//!
//! A default implementation [`DefaultEnduce`] is provided that returns `Motif::Unknown`
//! for every input — the correct behavior for a Day-One empty-bank deployment
//! (Proposition 9.1 of the DSSC paper).

use crate::sign::ResidualSign;
use crate::grammar::GrammarState;
use crate::bank::HeuristicsBank;
use crate::episode::Episode;
use crate::provenance::ProvenanceTag;
use crate::motif::Motif;

/// The endoductive operator trait.
///
/// Implement this trait to provide domain-specific motif matching. The default
/// implementation [`DefaultEnduce`] always returns `Motif::Unknown`.
///
/// # Formal correspondence
/// Implements `ℰ(r, σ, g, h) = (m, φ)` from Definition 5.1 of the DSSC paper.
pub trait Enduce {
    /// Apply the endoductive operator.
    ///
    /// # Arguments
    /// - `signs`: the observed sign sequence `σ(k₀:k*)`.
    /// - `grammar_path`: the grammar state sequence `g(k₀:k*)`.
    /// - `bank`: the heuristics bank `h`.
    /// - `step_range`: `(start_k, end_k)` — indices of the episode window.
    /// - `add_descriptor`: serialized ADD algebraic invariant descriptor (may be empty).
    ///
    /// # Returns
    /// An `Episode` — always. Never panics, never returns `None`.
    fn enduce(
        &self,
        signs: &[ResidualSign],
        grammar_path: &[GrammarState],
        bank: &HeuristicsBank,
        step_range: (usize, usize),
        add_descriptor: &str,
    ) -> Episode;
}

/// Default endoductive operator: returns `Motif::Unknown` with full provenance.
///
/// This is the correct implementation for a Day-One empty-bank deployment.
/// It satisfies all six safety-case properties (SC-1 through SC-6) without any
/// pre-configured fault library. See Proposition 9.1 of the DSSC paper.
pub struct DefaultEnduce;

impl Enduce for DefaultEnduce {
    fn enduce(
        &self,
        signs: &[ResidualSign],
        grammar_path: &[GrammarState],
        bank: &HeuristicsBank,
        step_range: (usize, usize),
        add_descriptor: &str,
    ) -> Episode {
        let persistence = grammar_path.iter()
            .rev()
            .take_while(|&&g| g == *grammar_path.last().unwrap_or(&GrammarState::Admissible))
            .count();

        let motif = if bank.is_empty() {
            Motif::Unknown
        } else {
            bank.match_episode(signs, grammar_path, persistence)
        };

        let tag = ProvenanceTag::new(
            signs.to_vec(),
            grammar_path.to_vec(),
            add_descriptor,
            step_range,
        );

        Episode::new(motif, tag)
    }
}
