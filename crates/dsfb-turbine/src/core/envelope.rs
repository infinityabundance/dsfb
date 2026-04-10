//! Admissibility envelope construction and evaluation.
//!
//! The envelope `E_k` defines the region within which health-parameter
//! residuals are classified as operationally acceptable. Envelopes are
//! **regime-conditioned**: they depend on the declared operating context.
//!
//! Construction uses healthy-window statistics. Once declared, the envelope
//! is fixed for the evaluation — all subsequent classification is deterministic
//! against this fixed envelope.

use crate::core::config::DsfbConfig;
use crate::core::regime::OperatingRegime;

/// Admissibility envelope for a single channel.
#[derive(Debug, Clone, Copy)]
pub struct AdmissibilityEnvelope {
    /// Lower bound of the admissible region.
    pub lower: f64,
    /// Upper bound of the admissible region.
    pub upper: f64,
    /// The regime under which this envelope was constructed.
    pub regime: OperatingRegime,
    /// Baseline mean used for construction.
    pub baseline_mean: f64,
    /// Baseline standard deviation used for construction.
    pub baseline_std: f64,
}

impl AdmissibilityEnvelope {
    /// Constructs an envelope from healthy-window statistics.
    ///
    /// `envelope = [mean - sigma * std, mean + sigma * std]`
    ///
    /// The `sigma` multiplier is taken from `config.envelope_sigma`.
    #[must_use]
    pub fn from_baseline(
        mean: f64,
        std: f64,
        regime: OperatingRegime,
        config: &DsfbConfig,
    ) -> Self {
        let half_width = config.envelope_sigma * std;
        Self {
            lower: mean - half_width,
            upper: mean + half_width,
            regime,
            baseline_mean: mean,
            baseline_std: std,
        }
    }

    /// Tests whether a residual value is inside the admissible envelope.
    ///
    /// Note: the residual is `r_k = value_k - baseline_mean`, so we test
    /// against the envelope centered at zero with half-width `sigma * std`.
    #[must_use]
    pub fn is_admissible(&self, residual: f64) -> bool {
        let half_width = self.upper - self.baseline_mean;
        residual.abs() <= half_width
    }

    /// Returns the admissibility gap: distance from envelope boundary.
    ///
    /// Positive means inside (margin remaining).
    /// Negative means outside (envelope violated).
    #[must_use]
    pub fn gap(&self, residual: f64) -> f64 {
        let half_width = self.upper - self.baseline_mean;
        half_width - residual.abs()
    }

    /// Returns the normalized position within the envelope.
    ///
    /// 0.0 = at center, 1.0 = at boundary, >1.0 = outside.
    #[must_use]
    pub fn normalized_position(&self, residual: f64) -> f64 {
        let half_width = self.upper - self.baseline_mean;
        if half_width < 1e-15 {
            return if residual.abs() < 1e-15 { 0.0 } else { f64::MAX };
        }
        residual.abs() / half_width
    }
}

/// Envelope status classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeStatus {
    /// Residual is well within the envelope (normalized position < 0.7).
    Interior,
    /// Residual is approaching the boundary (0.7 <= normalized position < 1.0).
    Approaching,
    /// Residual is at or beyond the boundary (normalized position >= 1.0).
    Exceeded,
}

impl AdmissibilityEnvelope {
    /// Classifies a residual's position relative to the envelope.
    #[must_use]
    pub fn classify_position(&self, residual: f64) -> EnvelopeStatus {
        let pos = self.normalized_position(residual);
        if pos < 0.7 {
            EnvelopeStatus::Interior
        } else if pos < 1.0 {
            EnvelopeStatus::Approaching
        } else {
            EnvelopeStatus::Exceeded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_construction() {
        let config = DsfbConfig::default();
        let env = AdmissibilityEnvelope::from_baseline(
            100.0, 2.0, OperatingRegime::SeaLevelStatic, &config,
        );
        assert!(env.is_admissible(0.0));    // zero residual is admissible
        assert!(env.is_admissible(4.0));    // within 2.5 * 2.0 = 5.0
        assert!(!env.is_admissible(6.0));   // outside 5.0
    }

    #[test]
    fn test_gap_positive_inside() {
        let config = DsfbConfig::default();
        let env = AdmissibilityEnvelope::from_baseline(
            0.0, 1.0, OperatingRegime::SeaLevelStatic, &config,
        );
        assert!(env.gap(0.0) > 0.0);
        assert!(env.gap(2.5) < 1e-10); // exactly at boundary
    }

    #[test]
    fn test_classification() {
        let config = DsfbConfig::default();
        let env = AdmissibilityEnvelope::from_baseline(
            0.0, 1.0, OperatingRegime::SeaLevelStatic, &config,
        );
        assert_eq!(env.classify_position(0.0), EnvelopeStatus::Interior);
        assert_eq!(env.classify_position(2.0), EnvelopeStatus::Approaching);
        assert_eq!(env.classify_position(3.0), EnvelopeStatus::Exceeded);
    }
}
