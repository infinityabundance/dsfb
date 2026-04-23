//! Deterministic Structural Accumulator (DSA) score.
//!
//! ## Mathematical Definition (paper §B.5)
//!
//! DSA(k) = w₁·b(k) + w₂·d(k) + w₃·s(k) + w₄·e(k) + w₅·μ(k)
//!
//! where:
//!   b(k) = rolling boundary density (fraction of last W_dsa in Boundary)
//!   d(k) = outward drift persistence (fraction with ṙ > 0)
//!   s(k) = slew density (fraction with |r̈| > δ_s)
//!   e(k) = normalised EWMA alarm occupancy over last W_dsa
//!   μ(k) = motif recurrence frequency in last W_dsa
//!
//! Default weights: w_i = 1.0 (unit weights, paper Stage III config).
//!
//! Alert fires when DSA(k) ≥ τ for ≥ K consecutive observations
//! AND ≥ m feature channels co-activate (corroboration, Lemma 6).
//!
//! ## Corroboration (paper Lemma 6)
//!
//! False-episode rate decreases monotonically with corroboration count c(k).
//! The DSA score is a monotonically increasing function of c(k).

use crate::grammar::GrammarState;
use crate::sign::SignTuple;

/// DSA score value — a dimensionless accumulation metric ∈ [0, 5.0].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DsaScore(pub f32);

impl DsaScore {
    /// Zero DSA score.
    pub const ZERO: Self = Self(0.0);

    /// Raw score value.
    #[inline]
    pub fn value(&self) -> f32 { self.0 }

    /// Returns true if score meets or exceeds threshold τ.
    #[inline]
    pub fn meets_threshold(&self, tau: f32) -> bool { self.0 >= tau }
}

/// Fixed-capacity DSA window.
///
/// W_DSA = window width for accumulation (default 10, paper Stage III).
/// K = persistence threshold (default 4).
pub struct DsaWindow<const W: usize> {
    /// Circular buffer: was this observation a boundary-approach?
    boundary_flags: [bool; W],
    /// Circular buffer: was drift outward?
    drift_flags: [bool; W],
    /// Circular buffer: was slew above threshold?
    slew_flags: [bool; W],
    /// Circular buffer: was EWMA above threshold?
    ewma_flags: [bool; W],
    /// Circular buffer: did a named motif fire?
    motif_flags: [bool; W],
    /// Write head.
    head: usize,
    /// Count of valid observations (saturates at W).
    count: usize,
    /// EWMA accumulator for residual norm.
    ewma_norm: f32,
    /// EWMA smoothing weight λ (default 0.20, paper Stage III).
    lambda: f32,
    /// EWMA alarm threshold = healthy_mean_ewma + 3σ_ewma.
    ewma_threshold: f32,
    /// DSA component weights [w1..w5].
    weights: [f32; 5],
    /// Slew threshold δ_s for slew density computation.
    delta_s: f32,
}

impl<const W: usize> DsaWindow<W> {
    /// Create a new DSA window with paper Stage III defaults.
    ///
    /// λ = 0.20, unit weights, δ_s = 0.05.
    /// `ewma_threshold` should be set from healthy-window calibration.
    pub const fn new(ewma_threshold: f32) -> Self {
        Self {
            boundary_flags: [false; W],
            drift_flags: [false; W],
            slew_flags: [false; W],
            ewma_flags: [false; W],
            motif_flags: [false; W],
            head: 0,
            count: 0,
            ewma_norm: 0.0,
            lambda: 0.20,
            ewma_threshold,
            weights: [1.0; 5],
            delta_s: 0.05,
        }
    }

    /// Push one observation and compute the DSA score.
    ///
    /// `motif_fired`: true if a named (non-Unknown) motif was identified.
    pub fn push(
        &mut self,
        sign: &SignTuple,
        grammar: GrammarState,
        motif_fired: bool,
    ) -> DsaScore {
        // Update EWMA for norm
        self.ewma_norm = self.lambda * sign.norm + (1.0 - self.lambda) * self.ewma_norm;

        // Compute individual flags
        let b = grammar.requires_attention();
        let d = sign.drift > 0.0;
        let s = sign.slew.abs() > self.delta_s;
        let e = self.ewma_norm > self.ewma_threshold;
        let mu = motif_fired;

        // Write into circular buffers
        let h = self.head;
        self.boundary_flags[h] = b;
        self.drift_flags[h] = d;
        self.slew_flags[h] = s;
        self.ewma_flags[h] = e;
        self.motif_flags[h] = mu;

        self.head = (self.head + 1) % W;
        if self.count < W { self.count += 1; }

        // Compute density scores over filled window
        let n = self.count as f32;
        let b_score = self.boundary_flags[..self.count].iter().filter(|&&x| x).count() as f32 / n;
        let d_score = self.drift_flags[..self.count].iter().filter(|&&x| x).count() as f32 / n;
        let s_score = self.slew_flags[..self.count].iter().filter(|&&x| x).count() as f32 / n;
        let e_score = self.ewma_flags[..self.count].iter().filter(|&&x| x).count() as f32 / n;
        let mu_score = self.motif_flags[..self.count].iter().filter(|&&x| x).count() as f32 / n;

        let score = self.weights[0] * b_score
            + self.weights[1] * d_score
            + self.weights[2] * s_score
            + self.weights[3] * e_score
            + self.weights[4] * mu_score;

        DsaScore(score)
    }

    /// Reset the DSA window (e.g., after post-transition guard expires).
    pub fn reset(&mut self) {
        self.boundary_flags = [false; W];
        self.drift_flags = [false; W];
        self.slew_flags = [false; W];
        self.ewma_flags = [false; W];
        self.motif_flags = [false; W];
        self.head = 0;
        self.count = 0;
        self.ewma_norm = 0.0;
    }

    /// Calibrate the EWMA threshold from healthy-window observations.
    ///
    /// Sets ewma_threshold = mean_ewma_norm + 3 * std_ewma_norm over healthy window.
    pub fn calibrate_ewma_threshold(&mut self, healthy_norms: &[f32]) {
        if healthy_norms.is_empty() {
            return;
        }
        // Run EWMA over healthy window to get steady-state distribution
        let mut ewma = 0.0_f32;
        let mut ewma_vals = [0.0_f32; 256];
        let clip = healthy_norms.len().min(256);
        for (i, &n) in healthy_norms[..clip].iter().enumerate() {
            ewma = self.lambda * n + (1.0 - self.lambda) * ewma;
            ewma_vals[i] = ewma;
        }
        let n = clip as f32;
        let mean = ewma_vals[..clip].iter().sum::<f32>() / n;
        let var = ewma_vals[..clip].iter()
            .map(|&x| (x - mean) * (x - mean))
            .sum::<f32>() / n;
        self.ewma_threshold = mean + 3.0 * crate::math::sqrt_f32(var);
        self.ewma_norm = 0.0; // reset accumulator after calibration
    }
}

/// Multi-channel DSA corroboration accumulator.
///
/// Implements Lemma 6: false-episode rate decreases monotonically
/// with corroboration count c(k) (number of channels simultaneously
/// in Boundary/Violation grammar state).
pub struct CorroborationAccumulator<const K: usize> {
    /// Rolling buffer of corroboration counts c(k).
    counts: [u8; K],
    head: usize,
    filled: usize,
    /// Minimum consecutive K observations at or above threshold τ.
    persistence_threshold: u8,
}

impl<const K: usize> CorroborationAccumulator<K> {
    /// New accumulator. `persistence_threshold` = minimum c(k) to qualify.
    pub const fn new(persistence_threshold: u8) -> Self {
        Self {
            counts: [0; K],
            head: 0,
            filled: 0,
            persistence_threshold,
        }
    }

    /// Push a new corroboration count and return whether the accumulator
    /// fires (K consecutive observations ≥ persistence_threshold).
    pub fn push(&mut self, count: u8) -> bool {
        self.counts[self.head] = count;
        self.head = (self.head + 1) % K;
        if self.filled < K { self.filled += 1; }

        if self.filled < K {
            return false;
        }
        // All K slots must be ≥ persistence_threshold
        self.counts.iter().all(|&c| c >= self.persistence_threshold)
    }

    /// Reset.
    pub fn reset(&mut self) {
        self.counts = [0; K];
        self.head = 0;
        self.filled = 0;
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GrammarState, ReasonCode};

    #[test]
    fn dsa_zero_for_clean_signal() {
        let mut w = DsaWindow::<10>::new(1.0);
        let sign = SignTuple::new(0.01, 0.0, 0.0);
        for _ in 0..10 {
            let score = w.push(&sign, GrammarState::Admissible, false);
            assert!(score.value() < 0.5, "clean signal DSA should be low");
        }
    }

    #[test]
    fn dsa_rises_for_sustained_boundary() {
        let mut w = DsaWindow::<10>::new(0.05);
        let sign = SignTuple::new(0.07, 0.005, 0.0);
        let grammar = GrammarState::Boundary(ReasonCode::SustainedOutwardDrift);
        let mut last = DsaScore::ZERO;
        for _ in 0..10 {
            last = w.push(&sign, grammar, true);
        }
        assert!(last.value() > 1.5, "sustained boundary DSA should be elevated: {}", last.value());
    }

    #[test]
    fn corroboration_fires_after_k_consecutive() {
        let mut acc = CorroborationAccumulator::<4>::new(1);
        // First 3 pushes: not yet K filled
        assert!(!acc.push(2));
        assert!(!acc.push(2));
        assert!(!acc.push(2));
        // 4th push fills K
        assert!(acc.push(2));
    }

    #[test]
    fn corroboration_requires_all_k_above_threshold() {
        let mut acc = CorroborationAccumulator::<4>::new(2);
        acc.push(3); acc.push(3); acc.push(0); // one below threshold
        let fires = acc.push(3);
        assert!(!fires, "should not fire with one slot below threshold");
    }

    #[test]
    fn dsa_threshold_check() {
        let score = DsaScore(2.5);
        assert!(score.meets_threshold(2.0));
        assert!(!score.meets_threshold(3.0));
    }
}
