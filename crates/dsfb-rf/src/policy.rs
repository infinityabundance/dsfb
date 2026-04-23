//! Policy engine: Silent | Watch | Review | Escalate.
//!
//! The policy engine is the final stage in the deterministic pipeline:
//!
//!   IQ Residual → Sign → Syntax → Grammar → Semantics → Policy
//!
//! It maps (grammar_state, semantic_disposition, dsa_score, corroboration)
//! → PolicyDecision, subject to persistence and fragmentation constraints.
//!
//! ## Policy Rules (paper §VIII, §B.5)
//!
//! - Silent:   grammar Admissible, DSA < τ, or persistence gate failed
//! - Watch:    motif active, DSA < τ or persistence < K
//! - Review:   persistence ≥ K AND motif class = Review-grade
//! - Escalate: persistence ≥ K AND Violation-class motif or Violation grammar

use crate::grammar::GrammarState;
use crate::heuristics::SemanticDisposition;
use crate::dsa::DsaScore;

/// The operator-facing policy decision.
///
/// This is the terminal output of the DSFB pipeline. It is the single
/// value the integration layer presents to the operator or upstream
/// alerting system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PolicyDecision {
    /// No structural activity detected. Nominal operation.
    Silent,
    /// Structural activity below escalation threshold. Continue monitoring.
    Watch,
    /// Persistent structural episode. Operator review warranted.
    Review,
    /// Violation-class episode. Immediate operator attention required.
    Escalate,
}

impl PolicyDecision {
    /// Returns true if this decision requires operator action.
    #[inline]
    pub fn requires_action(&self) -> bool {
        matches!(self, PolicyDecision::Review | PolicyDecision::Escalate)
    }

    /// Numeric level for metric computation (0–3).
    #[inline]
    pub fn level(&self) -> u8 {
        *self as u8
    }
}

/// Policy configuration — the Stage III fixed protocol parameters.
#[derive(Debug, Clone, Copy)]
pub struct PolicyConfig {
    /// DSA score threshold τ. Default 2.0 (paper Stage III).
    pub tau: f32,
    /// Persistence count K. Default 4 (paper Stage III).
    pub k: u8,
    /// Minimum corroboration count m. Default 1 (paper Stage III).
    pub m: u8,
    /// When `true`, a [`GrammarState::Violation`] causes an immediate
    /// [`PolicyDecision::Escalate`] without waiting for K persistence
    /// observations.  This is the **magnitude-gated bypass** described in
    /// paper §L item 9 (hypersonic detection latency defence): an extreme
    /// violation (residual norm >> ρ, triggering `Violation` directly) must
    /// not be delayed by the hysteresis confirmation window.
    ///
    /// Default: `true`.  Set to `false` only if false-escalation suppression
    /// is more important than latency (e.g., benign laboratory environments
    /// with high transient artefact rates).
    pub extreme_bypass: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self { tau: 2.0, k: 4, m: 1, extreme_bypass: true }
    }
}

impl PolicyConfig {
    /// Paper Stage III fixed configuration.
    pub const STAGE_III: Self = Self { tau: 2.0, k: 4, m: 1, extreme_bypass: true };
}

/// Policy evaluator with persistence tracking.
///
/// Maintains a consecutive-observation persistence counter.
/// Fires Review/Escalate only when DSA ≥ τ for ≥ K consecutive observations
/// with ≥ m corroborating channels.
pub struct PolicyEvaluator {
    config: PolicyConfig,
    /// Consecutive observations with DSA ≥ τ.
    persistence: u8,
    /// Whether last-committed decision was Review or Escalate (for fragmentation guard).
    episode_open: bool,
}

impl PolicyEvaluator {
    /// Create a new evaluator with Stage III defaults.
    pub const fn new() -> Self {
        Self {
            config: PolicyConfig::STAGE_III,
            persistence: 0,
            episode_open: false,
        }
    }

    /// Create with custom configuration.
    pub const fn with_config(config: PolicyConfig) -> Self {
        Self { config, persistence: 0, episode_open: false }
    }

    /// Evaluate policy for one observation.
    ///
    /// The integration contract: this method has `&mut self` (the evaluator
    /// maintains persistence state), but accepts the upstream observables
    /// as immutable references. There is no write path into upstream data.
    pub fn evaluate(
        &mut self,
        grammar: GrammarState,
        disposition: SemanticDisposition,
        dsa: DsaScore,
        corroboration_count: u8,
    ) -> PolicyDecision {
        // DSA threshold and corroboration gate
        let dsa_active = dsa.meets_threshold(self.config.tau);
        let corroborated = corroboration_count >= self.config.m;

        // Magnitude-gated extreme bypass (paper §L item 9, hypersonic defence):
        // An immediate Violation grammar state bypasses the K-persistence
        // hysteresis gate and escalates on the first observation.  This
        // prevents multi-window confirmation delay when the residual norm
        // far exceeds ρ (the grammar only assigns Violation for |r| ≫ ρ_eff).
        if self.config.extreme_bypass && grammar.is_violation() && corroborated {
            self.persistence = self.persistence.saturating_add(1);
            self.episode_open = true;
            return PolicyDecision::Escalate;
        }

        // Update persistence counter
        if dsa_active && corroborated && grammar.requires_attention() {
            self.persistence = self.persistence.saturating_add(1);
        } else {
            self.persistence = 0;
            self.episode_open = false;
        }

        // Decision logic
        if !grammar.requires_attention() || !corroborated {
            PolicyDecision::Silent
        } else if !dsa_active || self.persistence < self.config.k {
            PolicyDecision::Watch
        } else if grammar.is_violation()
            || matches!(disposition,
                SemanticDisposition::AbruptOnsetEvent
                | SemanticDisposition::PreTransitionCluster)
        {
            self.episode_open = true;
            PolicyDecision::Escalate
        } else {
            self.episode_open = true;
            PolicyDecision::Review
        }
    }

    /// Reset the evaluator (e.g., after a post-transition guard window).
    pub fn reset(&mut self) {
        self.persistence = 0;
        self.episode_open = false;
    }

    /// Returns true if an episode is currently open.
    #[inline]
    pub fn episode_open(&self) -> bool {
        self.episode_open
    }

    /// Current persistence count.
    #[inline]
    pub fn persistence(&self) -> u8 {
        self.persistence
    }
}

impl Default for PolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GrammarState, ReasonCode};
    use crate::heuristics::SemanticDisposition;
    use crate::dsa::DsaScore;

    fn boundary() -> GrammarState {
        GrammarState::Boundary(ReasonCode::SustainedOutwardDrift)
    }

    #[test]
    fn clean_signal_is_silent() {
        let mut p = PolicyEvaluator::new();
        let d = p.evaluate(
            GrammarState::Admissible,
            SemanticDisposition::Unknown,
            DsaScore(0.1),
            0,
        );
        assert_eq!(d, PolicyDecision::Silent);
    }

    #[test]
    fn watch_before_persistence_threshold() {
        let mut p = PolicyEvaluator::new();
        // K=4, so first 3 should be Watch
        for _ in 0..3 {
            let d = p.evaluate(boundary(), SemanticDisposition::PreTransitionCluster,
                DsaScore(3.0), 1);
            assert_eq!(d, PolicyDecision::Watch,
                "should be Watch before K=4 persistence");
        }
    }

    #[test]
    fn escalate_after_k_consecutive_with_pre_transition() {
        let mut p = PolicyEvaluator::new();
        let mut last = PolicyDecision::Silent;
        for _ in 0..5 {
            last = p.evaluate(boundary(), SemanticDisposition::PreTransitionCluster,
                DsaScore(3.0), 1);
        }
        assert_eq!(last, PolicyDecision::Escalate);
    }

    #[test]
    fn review_for_corroborating_drift() {
        let mut p = PolicyEvaluator::new();
        let mut last = PolicyDecision::Silent;
        for _ in 0..5 {
            last = p.evaluate(boundary(), SemanticDisposition::CorroboratingDrift,
                DsaScore(3.0), 1);
        }
        assert_eq!(last, PolicyDecision::Review);
    }

    #[test]
    fn violation_always_escalates_after_persistence() {
        let mut p = PolicyEvaluator::new();
        let mut last = PolicyDecision::Silent;
        for _ in 0..5 {
            last = p.evaluate(GrammarState::Violation,
                SemanticDisposition::Unknown, DsaScore(3.0), 1);
        }
        assert_eq!(last, PolicyDecision::Escalate);
    }

    #[test]
    fn policy_resets_on_clean_window() {
        let mut p = PolicyEvaluator::new();
        // Build up persistence
        for _ in 0..5 {
            p.evaluate(boundary(), SemanticDisposition::PreTransitionCluster,
                DsaScore(3.0), 1);
        }
        // Clean observation resets
        p.evaluate(GrammarState::Admissible, SemanticDisposition::Unknown,
            DsaScore(0.1), 0);
        assert_eq!(p.persistence(), 0);
        assert!(!p.episode_open());
    }

    #[test]
    fn requires_action_only_for_review_escalate() {
        assert!(!PolicyDecision::Silent.requires_action());
        assert!(!PolicyDecision::Watch.requires_action());
        assert!(PolicyDecision::Review.requires_action());
        assert!(PolicyDecision::Escalate.requires_action());
    }
}
