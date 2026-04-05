//! # Grammar FSM (`τ_g`)
//!
//! The DSSC grammar is a deterministic finite-state machine (DFSM) over `Σ` with state
//! space `G = {Adm, Bdy, Vio}`. Transition function `δ: G × Σ → G` is defined by the
//! SOS rules (Grammar-Adm, Grammar-Bdy, Grammar-Vio) of Section 3.3.
//!
//! The Rust type system enforces totality: `GrammarFsm::step` returns a `GrammarState`
//! for every possible input — no `Option`, no `Result`. This is the type-level proof of
//! Theorem 3.1 (Determinism and Totality) for the grammar sub-system.

use crate::sign::ResidualSign;
use crate::envelope::{AdmissibilityEnvelope, EnvelopeRegion};

/// Grammar state `g ∈ G = {Adm, Bdy, Vio}`.
///
/// Maps to the three SOS grammar rules from Section 3.3 of the DSSC paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrammarState {
    /// `Adm`: residual is strictly inside the admissibility envelope.
    /// System behavior is structurally nominal.
    Admissible,
    /// `Bdy`: residual is within the δ-band of the boundary.
    /// Approach is detected; early-warning window is open.
    Boundary,
    /// `Vio`: residual has exited the admissibility envelope.
    /// Structural excursion is confirmed; endoductive operator fires.
    Violation,
}

impl GrammarState {
    /// `true` if this state indicates a non-nominal condition (Bdy or Vio).
    #[inline]
    pub fn is_non_nominal(&self) -> bool {
        !matches!(self, GrammarState::Admissible)
    }

    /// `true` if this is a confirmed violation (endoductive operator should fire).
    #[inline]
    pub fn is_violation(&self) -> bool {
        matches!(self, GrammarState::Violation)
    }
}

impl std::fmt::Display for GrammarState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarState::Admissible => write!(f, "Adm"),
            GrammarState::Boundary   => write!(f, "Bdy"),
            GrammarState::Violation  => write!(f, "Vio"),
        }
    }
}

/// The DSSC grammar deterministic finite-state machine.
///
/// Holds the current state and applies deterministic transitions per Section 3.3.
/// The FSM is total: every `(state, sign, envelope)` triple produces a unique next state.
#[derive(Debug, Clone)]
pub struct GrammarFsm {
    state: GrammarState,
    /// Persistence counter K(k): consecutive steps in the current state.
    persistence: usize,
}

impl GrammarFsm {
    /// Construct the FSM in the initial `Adm` state.
    pub fn new() -> Self {
        Self { state: GrammarState::Admissible, persistence: 0 }
    }

    /// Current grammar state.
    pub fn state(&self) -> GrammarState { self.state }

    /// Persistence counter: number of consecutive steps in the current state.
    pub fn persistence(&self) -> usize { self.persistence }

    /// Apply one deterministic transition step.
    ///
    /// This is the implementation of `δ: G × Σ → G` from the SOS rules.
    /// Returns the new state. Never returns `None`; the function is total.
    ///
    /// # Formal correspondence
    /// - `EnvelopeRegion::Interior` → Grammar-Adm rule → `Admissible`
    /// - `EnvelopeRegion::Boundary` → Grammar-Bdy rule → `Boundary`
    /// - `EnvelopeRegion::Exterior` → Grammar-Vio rule → `Violation`
    pub fn step(&mut self, sign: &ResidualSign, envelope: &AdmissibilityEnvelope) -> GrammarState {
        let region = envelope.classify(sign.magnitude);
        let next = match region {
            EnvelopeRegion::Interior => GrammarState::Admissible,
            EnvelopeRegion::Boundary => GrammarState::Boundary,
            EnvelopeRegion::Exterior => GrammarState::Violation,
        };
        if next == self.state {
            self.persistence = self.persistence.saturating_add(1);
        } else {
            self.persistence = 0;
        }
        self.state = next;
        next
    }
}

impl Default for GrammarFsm {
    fn default() -> Self { Self::new() }
}
