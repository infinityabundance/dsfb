//! Pragmatic information gating for SOSA backplane efficiency.
//!
//! ## Theoretical Basis: Pragmatic Information (Atlan & Cohen 1998)
//!
//! Pragmatic information P is defined as information that changes the state
//! of a receiving system — it is the subset of Shannon information I(X) that
//! actually alters the receiver's behaviour or belief state.  Unlike syntactic
//! entropy, pragmatic information is *zero* when the receiver already knows
//! the content (redundant messages) and *zero* when the message is entirely
//! unpredictable (pure noise with nowhere to go).  It peaks at the edge of
//! the receiver's model capacity.
//!
//! **SOSA Backplane Gating:** The SOSA.MORA backplane resource is scarce.
//! DSFB applies pragmatic gating: only emit a grammar observation if it
//! carries pragmatic value — i.e. the grammar entropy has *changed* by more
//! than the gating threshold Δh.  This eliminates redundant Admissible-state
//! heartbeats (typically > 99% of samples) while preserving all state-
//! transition events.
//!
//! **Urgency Override:** When the grammar state is Violation or urgency ≥ 1.0,
//! the gate is bypassed: every sample is emitted regardless of redundancy.
//!
//! ## References
//!
//! Atlan, H. and Cohen, I.R. (1998) "Immune information, self-organisation
//!   and meaning," *Int. Immunol.* 10(6):711–717. doi:10.1093/intimm/10.6.711.
//!
//! Jumarie, G. (1990) *Relative Information: Theories and Applications.*
//!   Springer Series in Synergetics 47. doi:10.1007/978-3-642-84017-3.
//!
//! SOSA/MORA: Open Group SOSA/MORA Reference Architecture v1.1 (2021).
//!   https://publications.opengroup.org/c19f.

// ── Configuration ──────────────────────────────────────────────────────────

/// Configuration parameters for the pragmatic information gate.
#[derive(Debug, Clone, Copy)]
pub struct PragmaticConfig {
    /// Minimum grammar-entropy change required to emit (pragmatic threshold Δh).
    /// Below this change, the observation is suppressed as redundant.
    /// Typical: 0.05 nats (well within Admissible steady state).
    pub threshold: f32,
    /// Urgency level at which the gate is unconditionally bypassed.
    /// urgency ≥ urgency_override_level → always emit.
    pub urgency_override_level: f32,
    /// Whether to always emit the first observation after a reset.
    pub emit_on_first: bool,
}

impl Default for PragmaticConfig {
    fn default() -> Self {
        Self { threshold: 0.05, urgency_override_level: 0.8, emit_on_first: true }
    }
}

impl PragmaticConfig {
    /// Conservative config: wide threshold (fewer emissions).
    pub const fn conservative() -> Self {
        Self { threshold: 0.10, urgency_override_level: 0.9, emit_on_first: true }
    }

    /// Sensitive config: tight threshold (more emissions, less suppression).
    pub const fn sensitive() -> Self {
        Self { threshold: 0.01, urgency_override_level: 0.5, emit_on_first: true }
    }
}

// ── Pragmatic Gate ─────────────────────────────────────────────────────────

/// Pragmatic information gate for SOSA backplane traffic shaping.
///
/// Maintains a circular entropy history of depth S to track the running
/// grammar-entropy baseline and decide whether each new observation carries
/// sufficient pragmatic content to justify backplane emission.
///
/// ## Type Parameters
/// - `S`: History depth (≥ 4).  Entropy baseline is mean of last S values.
pub struct PragmaticGate<const S: usize> {
    /// Circular entropy history.
    history: [f32; S],
    /// Write head.
    head: usize,
    /// Number of valid samples in history.
    count: usize,
    /// Gate configuration.
    config: PragmaticConfig,
    /// Total observations presented to the gate.
    total_samples: u64,
    /// Total observations emitted (not suppressed).
    total_emitted: u64,
    /// Last emitted entropy value (for delta computation).
    last_emitted: f32,
}

impl<const S: usize> PragmaticGate<S> {
    /// Create a new gate with the given configuration.
    pub const fn new(config: PragmaticConfig) -> Self {
        Self {
            history: [0.0; S],
            head: 0,
            count: 0,
            config,
            total_samples: 0,
            total_emitted: 0,
            last_emitted: -1.0, // sentinel: no prior emission
        }
    }

    /// Create a gate with default configuration.
    pub const fn default_gate() -> Self {
        Self::new(PragmaticConfig { threshold: 0.05, urgency_override_level: 0.8, emit_on_first: true })
    }

    /// Decide whether to emit the current observation.
    ///
    /// - `grammar_entropy`: Current grammar FSM state entropy ∈ [0, 1] (nats-normalised).
    ///   Can be derived from the grammar certainty / spread across states.
    /// - `urgency`: Operator urgency signal ∈ [0, 1].  Values ≥ `urgency_override_level`
    ///   bypass the gate unconditionally.
    ///
    /// Returns `true` if the observation should be emitted to the SOSA backplane.
    pub fn should_emit(&mut self, grammar_entropy: f32, urgency: f32) -> bool {
        self.total_samples += 1;

        // Push to history
        self.history[self.head] = grammar_entropy;
        self.head = (self.head + 1) % S;
        if self.count < S { self.count += 1; }

        // Rule 1: Urgency override
        if urgency >= self.config.urgency_override_level {
            self.total_emitted += 1;
            self.last_emitted = grammar_entropy;
            return true;
        }

        // Rule 2: Emit on first (after reset or startup)
        if self.config.emit_on_first && self.last_emitted < 0.0 {
            self.total_emitted += 1;
            self.last_emitted = grammar_entropy;
            return true;
        }

        // Rule 3: Pragmatic gate — emit if |Δh| > threshold
        let delta = (grammar_entropy - self.last_emitted).abs();
        if delta >= self.config.threshold {
            self.total_emitted += 1;
            self.last_emitted = grammar_entropy;
            return true;
        }

        false
    }

    /// Backplane efficiency = 1 − (emitted / total).
    /// Represents the fraction of observations successfully suppressed.
    pub fn backplane_efficiency(&self) -> f32 {
        if self.total_samples == 0 { return 1.0; }
        1.0 - (self.total_emitted as f32 / self.total_samples as f32)
    }

    /// Total observations presented.
    pub fn total_samples(&self) -> u64 { self.total_samples }

    /// Total observations emitted (not gated).
    pub fn total_emitted(&self) -> u64 { self.total_emitted }

    /// Running mean grammar entropy from history.
    pub fn mean_entropy(&self) -> f32 {
        if self.count == 0 { return 0.0; }
        let sum: f32 = self.history[..self.count.min(S)].iter().sum();
        sum / self.count.min(S) as f32
    }

    /// Reset all state (preserves config).
    pub fn reset(&mut self) {
        self.history = [0.0; S];
        self.head = 0;
        self.count = 0;
        self.total_samples = 0;
        self.total_emitted = 0;
        self.last_emitted = -1.0;
    }
}

impl<const S: usize> Default for PragmaticGate<S> {
    fn default() -> Self { Self::default_gate() }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_observation_always_emitted() {
        let mut gate = PragmaticGate::<8>::default_gate();
        assert!(gate.should_emit(0.2, 0.0), "first sample must always emit");
    }

    #[test]
    fn constant_stream_suppressed_after_first() {
        let mut gate = PragmaticGate::<8>::default_gate();
        let mut emitted = 0u32;
        for _ in 0..100 {
            if gate.should_emit(0.1, 0.0) { emitted += 1; }
        }
        // Only the first should emit (and possibly small noise)
        assert_eq!(emitted, 1, "constant entropy should suppress all after first: {}", emitted);
    }

    #[test]
    fn state_change_triggers_emission() {
        let mut gate = PragmaticGate::<8>::default_gate();
        gate.should_emit(0.1, 0.0); // first
        // Feed identical: suppressed
        for _ in 0..10 { gate.should_emit(0.1, 0.0); }
        // Large jump: must emit
        let emitted = gate.should_emit(0.9, 0.0);
        assert!(emitted, "large entropy change must trigger emission");
    }

    #[test]
    fn urgency_override_bypasses_gate() {
        let mut gate = PragmaticGate::<8>::default_gate();
        gate.should_emit(0.1, 0.0); // first (clears sentinel)
        // Feed identical entropy but with high urgency
        for _ in 0..5 {
            let e = gate.should_emit(0.1, 0.95); // urgency > 0.8
            assert!(e, "high urgency must bypass gate");
        }
    }

    #[test]
    fn backplane_efficiency_high_for_constant() {
        let mut gate = PragmaticGate::<8>::default_gate();
        for _ in 0..200 { gate.should_emit(0.1, 0.0); }
        let eff = gate.backplane_efficiency();
        assert!(eff > 0.98, "constant stream efficiency > 98%: {:.4}", eff);
    }

    #[test]
    fn reset_clears_state() {
        let mut gate = PragmaticGate::<8>::default_gate();
        for _ in 0..50 { gate.should_emit(0.1, 0.0); }
        gate.reset();
        assert_eq!(gate.total_samples(), 0);
        assert_eq!(gate.total_emitted(), 0);
        // After reset, first emission should work again
        assert!(gate.should_emit(0.1, 0.0), "first after reset must emit");
    }

    #[test]
    fn efficiency_reflects_emission_ratio() {
        let mut gate = PragmaticGate::<8>::default_gate();
        // Emit 1 out of 100 (first)
        for _ in 0..100 { gate.should_emit(0.2, 0.0); }
        let eff = gate.backplane_efficiency();
        // 1 emitted / 100 total = 99% efficiency
        assert!((eff - 0.99).abs() < 0.02, "efficiency: {:.4}", eff);
    }
}
