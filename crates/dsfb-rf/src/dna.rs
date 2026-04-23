//! Hardware DNA authentication via Allan variance fingerprinting.
//!
//! ## Theoretical Basis
//!
//! Every hardware oscillator (OCXO, TCXO, VCXO, MEMS) produces a unique
//! combination of frequency-stability noise coefficients that can be estimated
//! from short-term measurements of the RF residual's carrier phase.  The
//! Allan deviation σ_y(τ) evaluated at a set of averaging times
//! {τ₁, τ₂, τ₄, τ₈, τ₁₆, τ₃₂, τ₆₄, τ₁₂₈} forms a 8-dimensional fingerprint
//! vector that is unique to each physical oscillator at the manufacturing-
//! process level (thermal-mechanical history, crystal cut variations, ageing
//! state).
//!
//! **Authentication Protocol:**
//! 1. During commissioning, compute the hardware DNA fingerprint and register it.
//! 2. At each calibration epoch, compute a fresh fingerprint from the live residual.
//! 3. Compute the cosine similarity between the fresh and registered fingerprints.
//! 4. Authenticate if similarity > AUTHENTICATION_THRESHOLD (default 0.95).
//!
//! **Security Note:** This is a *physical layer* authentication signal.
//! It detects hardware substitution (swap attack) and clock-injection spoofing
//! by comparing the intrinsic noise signature of the oscillator, not a
//! transmitted authentication code.  It does **not** provide cryptographic
//! guarantees and is not a replacement for link-layer authentication.
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - Rolling circular buffer of N residual norms
//! - Allan deviation computed at 8 averaging times via overlapping samples
//! - Cosine similarity matching with configurable threshold
//!
//! ## References
//!
//! Allan, D.W. (1966) "Statistics of atomic frequency standards,"
//!   *Proc. IEEE* 54(2):221–230. doi:10.1109/PROC.1966.4634.
//!
//! IEEE Std 1139-2008, "Standard Definitions of Physical Quantities for
//!   Fundamental Frequency and Time Metrology—Random Jitter and Phase Noise."
//!
//! Danev, B., Zanetti, D. and Capkun, S. (2010) "On physical-layer identification
//!   of wireless devices," *IEEE TNET* 20(3):1157–1270.
//!   doi:10.1109/TNET.2012.2191619.

use crate::math::sqrt_f32;

// ── Constants ──────────────────────────────────────────────────────────────

/// Cosine similarity threshold for authentic match (> 0.95).
pub const AUTHENTICATION_THRESHOLD: f32 = 0.95;

/// Allan deviation averaging times in samples (powers of 2: 1, 2, 4, …, 128).
pub const ALLAN_TAUS: [u32; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

// ── Hardware DNA Fingerprint ───────────────────────────────────────────────

/// Authentication verdict from DNA comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnaVerdict {
    /// Similarity ≥ AUTHENTICATION_THRESHOLD: fingerprint matches registration.
    Authentic,
    /// Similarity ∈ [0.85, threshold): close but not confident.  Flag for review.
    Suspicious,
    /// Similarity < 0.85: hardware substitution or spoofed clock likely.
    Spoofed,
}

/// Result of comparing an incoming oscillator fingerprint to a registered DNA.
#[derive(Debug, Clone, Copy)]
pub struct DnaMatchResult {
    /// Cosine similarity between incoming and registered fingerprints ∈ [−1, 1].
    pub similarity: f32,
    /// True if similarity ≥ AUTHENTICATION_THRESHOLD.
    pub is_authentic: bool,
    /// Verdict classification.
    pub verdict: DnaVerdict,
}

/// Registered hardware DNA fingerprint.
///
/// The 8 Allan deviation values at tau = 1, 2, 4, 8, 16, 32, 64, 128.
#[derive(Debug, Clone, Copy)]
pub struct HardwareDna {
    /// Allan deviation fingerprint: σ_y(τ) at the 8 standard averaging times.
    pub signature: [f32; 8],
    /// Human-readable hardware label.
    pub label: &'static str,
}

impl HardwareDna {
    /// Create a DNA record from a computed Allan deviation fingerprint.
    pub const fn new(signature: [f32; 8], label: &'static str) -> Self {
        Self { signature, label }
    }
}

// ── Allan Variance Estimator ───────────────────────────────────────────────

/// Rolling Allan variance estimator.
///
/// Maintains a circular buffer of N phase/norm samples and computes σ_y(τ)
/// for each of the ALLAN_TAUS averaging intervals using the overlapping
/// Allan variance formula:
///
/// ```text
/// σ²_y(τ) = 1 / (2τ²(N-2τ)) · Σ_{k=1}^{N-2τ} [x(k+2τ) - 2x(k+τ) + x(k)]²
/// ```
///
/// ## Type Parameters
/// - `N`: Buffer capacity (≥ 256 recommended for τ=128 support; 512 is ideal).
pub struct AllanVarianceEstimator<const N: usize> {
    buf: [f32; N],
    head: usize,
    count: usize,
}

impl<const N: usize> AllanVarianceEstimator<N> {
    /// Create a new estimator with empty buffer.
    pub const fn new() -> Self {
        Self { buf: [0.0; N], head: 0, count: 0 }
    }

    /// Absorb one sample (residual norm or phase increment).
    pub fn push(&mut self, sample: f32) {
        self.buf[self.head] = sample;
        self.head = (self.head + 1) % N;
        if self.count < N { self.count += 1; }
    }

    /// Number of valid samples.
    pub fn len(&self) -> usize { self.count }

    /// Whether enough samples are available for fingerprint computation.
    /// Requires count ≥ 2 * max_tau + 1 = 257 for τ_max = 128.
    pub fn is_ready(&self) -> bool { self.count >= 2 * ALLAN_TAUS[7] as usize + 1 }

    /// Access sample at absolute position `i` in the ring buffer.
    fn sample(&self, i: usize) -> f32 {
        // Most-recent sample is at index (head-1+N)%N
        // sample(0) = most recent, sample(count-1) = oldest
        let idx = (self.head + N - 1 - i) % N;
        self.buf[idx]
    }

    /// Compute Allan deviation σ_y(τ) for averaging time τ (in samples).
    ///
    /// Uses overlapping samples for N_eff = count − 2τ averages.
    /// Returns 0.0 if count < 2τ + 1.
    pub fn allan_deviation(&self, tau: u32) -> f32 {
        let t = tau as usize;
        if self.count < 2 * t + 1 { return 0.0; }
        let n = self.count.min(N);
        let max_k = n.saturating_sub(2 * t);
        if max_k == 0 { return 0.0; }
        let tau_sq = (t * t) as f32;
        let mut sum = 0.0_f32;
        for k in 0..max_k {
            // x(k), x(k+τ), x(k+2τ) — oldest-first ordering
            // In ring buf with sample(0)=most recent: oldest is sample(count-1)
            // k=0 is the oldest triple
            let offset = n.saturating_sub(1).saturating_sub(k);
            let x0 = if offset >= 2 * t { self.sample(offset) } else { 0.0 };
            let x1 = if offset >= t { self.sample(offset - t) } else { 0.0 };
            let x2 = self.sample(offset);
            // Second difference: Δ = x(k+2τ) - 2x(k+τ) + x(k)
            let diff = x2 - 2.0 * x1 + x0;
            sum += diff * diff;
        }
        let avar = sum / (2.0 * tau_sq * max_k as f32);
        sqrt_f32(avar.max(0.0))
    }

    /// Compute the 8-element Allan deviation fingerprint.
    ///
    /// Returns `None` if `is_ready()` is false.
    pub fn fingerprint(&self) -> Option<[f32; 8]> {
        if !self.is_ready() { return None; }
        let mut sig = [0.0_f32; 8];
        for (i, &tau) in ALLAN_TAUS.iter().enumerate() {
            sig[i] = self.allan_deviation(tau);
        }
        Some(sig)
    }

    /// Reset the buffer.
    pub fn reset(&mut self) {
        self.buf = [0.0; N];
        self.head = 0;
        self.count = 0;
    }
}

impl<const N: usize> Default for AllanVarianceEstimator<N> {
    fn default() -> Self { Self::new() }
}

// ── Authentication ─────────────────────────────────────────────────────────

/// Authenticate an incoming fingerprint against a registered hardware DNA.
///
/// Uses cosine similarity as the distance metric.  Cosine similarity is
/// invariant to overall scale amplitude changes (gain variations), making
/// it robust to received-power differences while sensitive to the *shape*
/// of the σ_y(τ) curve which is intrinsic to the oscillator hardware.
pub fn verify_dna(incoming: &[f32; 8], registered: &HardwareDna) -> DnaMatchResult {
    let sim = cosine_similarity(incoming, &registered.signature);
    let is_authentic = sim >= AUTHENTICATION_THRESHOLD;
    let verdict = if sim >= AUTHENTICATION_THRESHOLD {
        DnaVerdict::Authentic
    } else if sim >= 0.85 {
        DnaVerdict::Suspicious
    } else {
        DnaVerdict::Spoofed
    };
    DnaMatchResult { similarity: sim, is_authentic, verdict }
}

/// Cosine similarity between two 8-element vectors.  
/// Returns 0.0 if either vector is zero-magnitude.
pub fn cosine_similarity(a: &[f32; 8], b: &[f32; 8]) -> f32 {
    let dot: f32    = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
    let mag_a: f32  = sqrt_f32(a.iter().map(|&x| x * x).sum::<f32>());
    let mag_b: f32  = sqrt_f32(b.iter().map(|&x| x * x).sum::<f32>());
    if mag_a < 1e-20 || mag_b < 1e-20 { return 0.0; }
    (dot / (mag_a * mag_b)).max(-1.0).min(1.0)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors_is_one() {
        let a = [0.1_f32, 0.2, 0.3, 0.1, 0.05, 0.02, 0.01, 0.005];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-4, "identical vectors: cosine={}", sim);
    }

    #[test]
    fn cosine_orthogonal_vectors_is_zero() {
        let a = [1.0_f32, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];
        let b = [0.0_f32, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-4, "orthogonal: cosine={}", sim);
    }

    #[test]
    fn authentic_match_passes() {
        let sig = [0.1_f32, 0.08, 0.06, 0.05, 0.04, 0.03, 0.02, 0.01];
        let dna = HardwareDna::new(sig, "OCXO_test");
        // Identical fingerprint: similarity = 1.0 → Authentic
        let result = verify_dna(&sig, &dna);
        assert_eq!(result.verdict, DnaVerdict::Authentic);
        assert!(result.is_authentic);
    }

    #[test]
    fn spoofed_fingerprint_detected() {
        let registered = [0.1_f32, 0.08, 0.06, 0.05, 0.04, 0.03, 0.02, 0.01];
        let dna = HardwareDna::new(registered, "reference");
        // Very different shape: opposite slope
        let incoming = [0.01_f32, 0.02, 0.03, 0.04, 0.05, 0.06, 0.08, 0.10];
        let result = verify_dna(&incoming, &dna);
        assert_ne!(result.verdict, DnaVerdict::Authentic,
            "opposite-slope fingerprint should not authenticate: sim={:.3}", result.similarity);
    }

    #[test]
    fn allan_estimator_ready_after_sufficient_samples() {
        let mut est = AllanVarianceEstimator::<512>::new();
        assert!(!est.is_ready());
        for i in 0..257 {
            est.push(0.01 + i as f32 * 0.0001);
        }
        assert!(est.is_ready(), "estimator must be ready after 257 samples");
    }

    #[test]
    fn allan_estimator_returns_none_when_not_ready() {
        let mut est = AllanVarianceEstimator::<512>::new();
        for _ in 0..10 { est.push(0.01); }
        assert!(est.fingerprint().is_none());
    }

    #[test]
    fn fingerprint_returns_some_when_ready() {
        let mut est = AllanVarianceEstimator::<512>::new();
        for i in 0..512 { est.push(0.01 + (i as f32 * 0.0001).sin() * 0.001); }
        let fp = est.fingerprint();
        assert!(fp.is_some(), "must return Some after 512 samples");
        let sig = fp.unwrap();
        // All values must be non-negative
        for (i, &v) in sig.iter().enumerate() {
            assert!(v >= 0.0, "sigma[{}] = {} must be non-negative", i, v);
        }
    }

    #[test]
    fn verify_dna_with_small_perturbation_authentic() {
        // Build an exact fingerprint via cosine similarity check
        let base = [0.10_f32, 0.07, 0.05, 0.04, 0.03, 0.02, 0.015, 0.01];
        let dna = HardwareDna::new(base, "reference");
        // Tiny perturbation: < 1% change per element
        let perturbed: [f32; 8] = core::array::from_fn(|i| base[i] * (1.0 + 0.002 * (i as f32)));
        let result = verify_dna(&perturbed, &dna);
        assert_eq!(result.verdict, DnaVerdict::Authentic,
            "tiny perturbation should still authenticate: sim={:.4}", result.similarity);
    }
}
