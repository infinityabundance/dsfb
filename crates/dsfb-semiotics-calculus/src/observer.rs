//! # Observer (`𝒪: Traj → Ep`)
//!
//! The DSSC observer is the functor `𝒪: Traj → Ep` proven in Theorem 3.2 of the DSSC
//! paper. It satisfies:
//!
//! 1. **Well-definedness** (SC-1): every trajectory maps to a unique episode sequence.
//! 2. **Stationarity**: grammar transitions are Markovian; the observer is time-shift invariant.
//! 3. **Compositionality**: `𝒪(r₁ · r₂) = 𝒪(r₁) ⊗ 𝒪(r₂)` (episode concatenation).
//! 4. **Purity** (SC-2): `Observer` holds no mutable references to the observed system.
//!    Rust's ownership system enforces this structurally — there is no `&mut System` here.
//!
//! The `Observer` struct owns only its configuration (envelope, bank, enduce operator).
//! It reads residual values passed to `observe_step` and returns episodes. It writes
//! to no external state. This is the Rust enforcement of Non-Interference (SC-2).

use crate::sign::ResidualSign;
use crate::grammar::GrammarFsm;
use crate::envelope::AdmissibilityEnvelope;
use crate::bank::HeuristicsBank;
use crate::enduce::{Enduce, DefaultEnduce};
use crate::episode::Episode;
use crate::grammar::GrammarState;

/// The DSSC observer: a pure, read-only structural interpretation engine.
///
/// # Non-interference guarantee (Theorem 3.2, property 4)
/// `Observer` takes residual values by shared reference (`&f64` / `f64` copy).
/// It has no `&mut` path to the observed system, enforced by the Rust borrow checker.
/// This is not a policy constraint — it is a structural impossibility.
///
/// # Usage
/// ```rust,ignore
/// let observer = Observer::new(envelope, HeuristicsBank::new());
/// for (k, &residual) in trajectory.iter().enumerate() {
///     if let Some(episode) = observer.observe_step(residual, k) {
///         println!("Episode at step {}: {}", k, episode);
///     }
/// }
/// ```
pub struct Observer<E: Enduce = DefaultEnduce> {
    envelope: AdmissibilityEnvelope,
    bank: HeuristicsBank,
    enduce: E,
    fsm: GrammarFsm,
    /// Accumulated signs for the current episode window.
    window_signs: Vec<ResidualSign>,
    window_grammar: Vec<GrammarState>,
    window_start: usize,
    /// ADD descriptor for the current window (domain-supplied; may be empty).
    add_descriptor: String,
}

impl Observer<DefaultEnduce> {
    /// Construct a default observer with the given envelope and heuristics bank.
    ///
    /// Uses `DefaultEnduce`, which returns `Motif::Unknown` for all inputs.
    /// This is the correct Day-One deployment configuration (Proposition 9.1).
    pub fn new(envelope: AdmissibilityEnvelope, bank: HeuristicsBank) -> Self {
        Self::with_enduce(envelope, bank, DefaultEnduce)
    }
}

impl<E: Enduce> Observer<E> {
    /// Construct an observer with a custom endoductive operator.
    pub fn with_enduce(envelope: AdmissibilityEnvelope, bank: HeuristicsBank, enduce: E) -> Self {
        Self {
            envelope,
            bank,
            enduce,
            fsm: GrammarFsm::new(),
            window_signs: Vec::new(),
            window_grammar: Vec::new(),
            window_start: 0,
            add_descriptor: String::new(),
        }
    }

    /// Supply an ADD algebraic descriptor for the current episode window.
    /// This is stored in the provenance tag of the next emitted episode.
    pub fn set_add_descriptor(&mut self, desc: impl Into<String>) {
        self.add_descriptor = desc.into();
    }

    /// Process one residual observation at step `k`.
    ///
    /// This is the implementation of one reduction step of the DSSC operational semantics
    /// (Section 3.3). It is total: every call returns either `None` (episode window still
    /// open) or `Some(Episode)` (episode emitted on grammar state transition to Vio or
    /// on a persistence-gate trigger).
    ///
    /// The residual `magnitude` is the scalar `‖r(k)‖`.
    /// `prev` and `prev2` are `r(k-1)` and `r(k-2)` for drift/slew computation.
    pub fn observe_step(
        &mut self,
        magnitude: f64,
        prev: f64,
        prev2: f64,
        k: usize,
    ) -> Option<Episode> {
        let sign = ResidualSign::from_scalar(magnitude, prev, prev2);
        let state = self.fsm.step(&sign, &self.envelope);

        self.window_signs.push(sign);
        self.window_grammar.push(state);

        // Emit an episode on confirmed Violation — endoductive operator fires.
        if state == GrammarState::Violation {
            let episode = self.enduce.enduce(
                &self.window_signs,
                &self.window_grammar,
                &self.bank,
                (self.window_start, k),
                &self.add_descriptor,
            );
            // Reset window for next episode
            self.window_signs.clear();
            self.window_grammar.clear();
            self.window_start = k + 1;
            self.add_descriptor.clear();
            return Some(episode);
        }

        // Return to Adm after non-nominal: reset window quietly (no episode emitted).
        if state == GrammarState::Admissible && self.window_signs.len() > 1 {
            self.window_signs.clear();
            self.window_grammar.clear();
            self.window_start = k + 1;
            self.add_descriptor.clear();
        }

        None
    }

    /// Process a complete trajectory slice, returning all emitted episodes.
    ///
    /// Implements `𝒪(r)` for a full trajectory. This is the monoidal-unit-respecting
    /// batch form: `𝒪(r₁ · r₂) = 𝒪(r₁) ⊗ 𝒪(r₂)` (Definition 3.4).
    pub fn observe_trajectory(&mut self, residuals: &[f64]) -> Vec<Episode> {
        let mut episodes = Vec::new();
        let mut prev = 0.0_f64;
        let mut prev2 = 0.0_f64;
        for (k, &r) in residuals.iter().enumerate() {
            if let Some(ep) = self.observe_step(r, prev, prev2, k) {
                episodes.push(ep);
            }
            prev2 = prev;
            prev = r;
        }
        episodes
    }
}
