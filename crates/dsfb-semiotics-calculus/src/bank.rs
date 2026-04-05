//! # Heuristics Bank (`τ_h`)
//!
//! The heuristics bank `h` is a finite partial function `M ⇀ 𝒫(Σ*)` mapping motif names
//! to sets of sign-sequence patterns. Section 3.1 and Theorem 7.2 (Bank Monotonicity)
//! of the DSSC paper.
//!
//! The Rust `HeuristicsBank::augment` method enforces the monotonicity invariant at the
//! type level: augmentation only adds or extends entries, never removes them. This is
//! the type-level proof of Theorem 7.2.

use std::collections::HashMap;
use crate::motif::Motif;
use crate::sign::ResidualSign;
use crate::grammar::GrammarState;

/// A pattern for matching against sign and grammar sequences.
///
/// A `MotifPattern` is a named structural signature: a sign predicate and a grammar
/// predicate that, together, license a `Motif::Named` output.
#[derive(Debug, Clone)]
pub struct MotifPattern {
    /// Human-readable name matching the motif name in the bank.
    pub name: String,
    /// Minimum number of consecutive non-nominal grammar steps required.
    pub min_persistence: usize,
    /// Whether the pattern requires a confirmed `Violation` state.
    pub requires_violation: bool,
    /// Minimum outward drift magnitude to trigger this pattern.
    pub min_drift: f64,
}

/// The heuristics bank: a monotone-growing finite collection of motif patterns.
///
/// # Monotonicity invariant (Theorem 7.2)
/// Every call to `augment` can only add new entries or extend existing ones.
/// No existing pattern is removed or narrowed. This invariant is enforced by the
/// `augment` implementation — there is no `remove` method.
#[derive(Debug, Clone, Default)]
pub struct HeuristicsBank {
    patterns: HashMap<String, Vec<MotifPattern>>,
}

impl HeuristicsBank {
    /// Construct an empty heuristics bank.
    ///
    /// An empty bank is a valid deployment state (Proposition 9.1 / Day-One Value).
    /// The observer is total with `h = ∅`; all outputs will be `Motif::Unknown`
    /// with complete provenance.
    pub fn new() -> Self { Self::default() }

    /// Augment the bank with a new pattern under the given motif name.
    ///
    /// Implements `augment(h, m, P)` from Definition 7.2 of the DSSC paper.
    /// The monotonicity invariant is enforced: this method only appends.
    pub fn augment(&mut self, name: impl Into<String>, pattern: MotifPattern) {
        self.patterns.entry(name.into()).or_default().push(pattern);
    }

    /// Number of named motifs in the bank.
    pub fn motif_count(&self) -> usize { self.patterns.len() }

    /// `true` if the bank is empty (Day-One deployment state).
    pub fn is_empty(&self) -> bool { self.patterns.is_empty() }

    /// Attempt to match a sign sequence and grammar path against the bank.
    ///
    /// Returns the first matching `Motif::Named` or `Motif::Unknown` if no pattern
    /// matches. Never returns `None` — the function is total (Theorem 3.1).
    pub fn match_episode(
        &self,
        signs: &[ResidualSign],
        grammar_path: &[GrammarState],
        persistence: usize,
    ) -> Motif {
        for (name, patterns) in &self.patterns {
            for p in patterns {
                if persistence >= p.min_persistence
                    && (!p.requires_violation
                        || grammar_path.iter().any(|g| g.is_violation()))
                    && signs.iter().any(|s| s.drift >= p.min_drift)
                {
                    return Motif::named(name.clone());
                }
            }
        }
        Motif::Unknown
    }
}
