//! Admissibility envelopes: regime-conditioned bounds for residual classification.
//!
//! An [`AdmissibilityEnvelope`] defines the region of residual space considered
//! operationally acceptable under a given workload phase. When a residual sign
//! exits this envelope, the grammar layer transitions from `Admissible` to
//! `Boundary` or `Violation`.
//!
//! ## Failure Mode FM-03: Envelope Miscalibration Across Workload Phases
//!
//! Envelopes calibrated during steady-state may misclassify warmup or cooldown
//! transients as violations. Per-phase calibration is required.

use crate::regime::WorkloadPhase;
use crate::residual::ResidualSign;

/// Admissibility envelope for a single residual source under a specific
/// workload phase.
///
/// The envelope defines upper and lower bounds on the residual value,
/// drift, and slew. A residual sign is classified based on its position
/// relative to these bounds.
#[derive(Debug, Clone, Copy)]
pub struct AdmissibilityEnvelope {
    /// Lower bound on acceptable residual magnitude.
    pub residual_lower: f64,
    /// Upper bound on acceptable residual magnitude.
    pub residual_upper: f64,
    /// Maximum acceptable drift magnitude (absolute value).
    pub drift_limit: f64,
    /// Maximum acceptable slew magnitude (absolute value).
    pub slew_limit: f64,
    /// Workload phase this envelope applies to.
    pub phase: WorkloadPhase,
    /// Boundary fraction: the fraction of the envelope width at which
    /// the grammar transitions from Admissible to Boundary.
    /// Typically 0.8 (i.e., 80% of the way to the limit).
    pub boundary_fraction: f64,
}

/// Classification of a residual sign against an envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopePosition {
    /// Residual is well within the envelope interior.
    Interior,
    /// Residual is in the boundary zone (between boundary_fraction and 1.0).
    BoundaryZone,
    /// Residual has exited the envelope.
    Exterior,
}

impl AdmissibilityEnvelope {
    /// Create a new envelope with the given bounds.
    ///
    /// `boundary_fraction` is clamped to [0.5, 0.99].
    pub fn new(
        residual_lower: f64,
        residual_upper: f64,
        drift_limit: f64,
        slew_limit: f64,
        phase: WorkloadPhase,
        boundary_fraction: f64,
    ) -> Self {
        Self {
            residual_lower,
            residual_upper,
            drift_limit,
            slew_limit,
            phase,
            boundary_fraction: boundary_fraction.clamp(0.5, 0.99),
        }
    }

    /// Construct a symmetric envelope centered at zero with the given half-width.
    pub fn symmetric(
        half_width: f64,
        drift_limit: f64,
        slew_limit: f64,
        phase: WorkloadPhase,
    ) -> Self {
        Self::new(-half_width, half_width, drift_limit, slew_limit, phase, 0.8)
    }

    /// Classify a residual sign against this envelope.
    ///
    /// The classification considers residual magnitude, drift, and slew
    /// independently. The most severe classification wins.
    pub fn classify(&self, sign: &ResidualSign) -> EnvelopePosition {
        let r_pos = self.classify_scalar(sign.residual, self.residual_lower, self.residual_upper);
        let d_pos = self.classify_symmetric(sign.drift, self.drift_limit);
        let s_pos = self.classify_symmetric(sign.slew, self.slew_limit);

        // Return the most severe classification
        worst_position(worst_position(r_pos, d_pos), s_pos)
    }

    /// Classify a scalar value against asymmetric bounds [lower, upper].
    fn classify_scalar(&self, value: f64, lower: f64, upper: f64) -> EnvelopePosition {
        let range = upper - lower;
        if range <= 0.0 {
            return EnvelopePosition::Exterior;
        }
        let boundary_lower = lower + range * (1.0 - self.boundary_fraction) / 2.0;
        let boundary_upper = upper - range * (1.0 - self.boundary_fraction) / 2.0;

        if value < lower || value > upper {
            EnvelopePosition::Exterior
        } else if value < boundary_lower || value > boundary_upper {
            EnvelopePosition::BoundaryZone
        } else {
            EnvelopePosition::Interior
        }
    }

    /// Classify a scalar value against symmetric bounds [-limit, +limit].
    fn classify_symmetric(&self, value: f64, limit: f64) -> EnvelopePosition {
        if limit <= 0.0 {
            return if value.abs() > 0.0 {
                EnvelopePosition::Exterior
            } else {
                EnvelopePosition::Interior
            };
        }
        let abs_val = value.abs();
        let boundary = limit * self.boundary_fraction;

        if abs_val > limit {
            EnvelopePosition::Exterior
        } else if abs_val > boundary {
            EnvelopePosition::BoundaryZone
        } else {
            EnvelopePosition::Interior
        }
    }

    /// Envelope width (upper - lower) for the residual dimension.
    pub fn residual_width(&self) -> f64 {
        self.residual_upper - self.residual_lower
    }

    /// Fractional position of a residual value within the envelope [0.0, 1.0+].
    /// Values > 1.0 indicate exterior position.
    pub fn fractional_position(&self, residual: f64) -> f64 {
        let center = (self.residual_upper + self.residual_lower) / 2.0;
        let half_width = self.residual_width() / 2.0;
        if half_width <= 0.0 {
            return f64::INFINITY;
        }
        (residual - center).abs() / half_width
    }
}

/// Return the more severe of two envelope positions.
fn worst_position(a: EnvelopePosition, b: EnvelopePosition) -> EnvelopePosition {
    match (a, b) {
        (EnvelopePosition::Exterior, _) | (_, EnvelopePosition::Exterior) => {
            EnvelopePosition::Exterior
        }
        (EnvelopePosition::BoundaryZone, _) | (_, EnvelopePosition::BoundaryZone) => {
            EnvelopePosition::BoundaryZone
        }
        (EnvelopePosition::Interior, EnvelopePosition::Interior) => EnvelopePosition::Interior,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::residual::ResidualSource;

    fn sign(r: f64, d: f64, s: f64) -> ResidualSign {
        ResidualSign {
            residual: r,
            drift: d,
            slew: s,
            timestamp_ns: 0,
            source: ResidualSource::Latency,
        }
    }

    #[test]
    fn test_interior_classification() {
        let env = AdmissibilityEnvelope::symmetric(10.0, 1.0, 0.5, WorkloadPhase::SteadyState);
        assert_eq!(
            env.classify(&sign(0.0, 0.0, 0.0)),
            EnvelopePosition::Interior
        );
        assert_eq!(
            env.classify(&sign(5.0, 0.3, 0.1)),
            EnvelopePosition::Interior
        );
    }

    #[test]
    fn test_boundary_classification() {
        let env = AdmissibilityEnvelope::symmetric(10.0, 1.0, 0.5, WorkloadPhase::SteadyState);
        // 9.0 is 90% of half-width (10.0), beyond 80% boundary_fraction
        assert_eq!(
            env.classify(&sign(9.0, 0.0, 0.0)),
            EnvelopePosition::BoundaryZone
        );
    }

    #[test]
    fn test_exterior_classification() {
        let env = AdmissibilityEnvelope::symmetric(10.0, 1.0, 0.5, WorkloadPhase::SteadyState);
        assert_eq!(
            env.classify(&sign(11.0, 0.0, 0.0)),
            EnvelopePosition::Exterior
        );
        assert_eq!(
            env.classify(&sign(0.0, 1.5, 0.0)),
            EnvelopePosition::Exterior
        );
    }

    #[test]
    fn test_drift_triggers_boundary() {
        let env = AdmissibilityEnvelope::symmetric(10.0, 1.0, 0.5, WorkloadPhase::SteadyState);
        // residual is fine, but drift is at 0.9 (90% of limit=1.0, beyond 80%)
        assert_eq!(
            env.classify(&sign(0.0, 0.9, 0.0)),
            EnvelopePosition::BoundaryZone
        );
    }
}
