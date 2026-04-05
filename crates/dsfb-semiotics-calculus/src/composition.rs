//! # Composition Operators (Section 7 of the DSSC paper)
//!
//! Three composition operators are defined:
//! 1. `GrammarFusion` — hierarchical product of two grammar FSMs.
//! 2. Heuristics bank augmentation — handled by `HeuristicsBank::augment` (monotone).
//! 3. `CrossStreamObserver` — fused observation over two residual streams.

use crate::grammar::{GrammarFsm, GrammarState};
use crate::envelope::AdmissibilityEnvelope;
use crate::sign::ResidualSign;

/// Hierarchical grammar fusion `G₁ ⋈ G₂` (Definition 7.1).
///
/// The product FSM state is `(g₁, g₂)`. Transitions apply independently to each
/// component. Determinism of the product is guaranteed because both components are
/// deterministic (Proposition 7.1).
pub struct GrammarFusion {
    fsm1: GrammarFsm,
    fsm2: GrammarFsm,
}

impl GrammarFusion {
    /// Construct the product FSM, both initialized to `Adm`.
    pub fn new() -> Self {
        Self { fsm1: GrammarFsm::new(), fsm2: GrammarFsm::new() }
    }

    /// Apply one step to both FSMs and return the joint state.
    pub fn step(
        &mut self,
        sign1: &ResidualSign, env1: &AdmissibilityEnvelope,
        sign2: &ResidualSign, env2: &AdmissibilityEnvelope,
    ) -> (GrammarState, GrammarState) {
        let g1 = self.fsm1.step(sign1, env1);
        let g2 = self.fsm2.step(sign2, env2);
        (g1, g2)
    }

    /// `true` if either component is in violation (joint non-nominal detection).
    pub fn is_joint_violation(&self) -> bool {
        self.fsm1.state().is_violation() || self.fsm2.state().is_violation()
    }
}

impl Default for GrammarFusion {
    fn default() -> Self { Self::new() }
}
