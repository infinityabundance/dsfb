use anyhow::{anyhow, Result};

use crate::engine::types::{
    AdmissibilityEnvelope, EnvelopeMode, EnvelopeSample, GrammarReasonCode, GrammarState,
    GrammarStatus, ResidualTrajectory, TrustScalar,
};

/// Typed envelope configuration used by both synthetic and CSV-driven runs.
#[derive(Clone, Debug)]
pub struct EnvelopeSpec {
    pub name: String,
    pub mode: EnvelopeMode,
    pub base_radius: f64,
    pub slope: f64,
    pub switch_step: Option<usize>,
    pub secondary_slope: Option<f64>,
    pub secondary_base: Option<f64>,
}

impl EnvelopeSpec {
    /// Validates that the envelope parameters are explicit and numerically well-formed.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(anyhow!("envelope specification requires a non-empty name"));
        }
        if !self.base_radius.is_finite() || self.base_radius <= 0.0 {
            return Err(anyhow!(
                "envelope base radius must be positive and finite; got {}",
                self.base_radius
            ));
        }
        if !self.slope.is_finite() {
            return Err(anyhow!("envelope slope must be finite; got {}", self.slope));
        }
        match self.mode {
            EnvelopeMode::RegimeSwitched => {
                let secondary_slope = self.secondary_slope.ok_or_else(|| {
                    anyhow!("regime-switched envelopes require an explicit secondary slope")
                })?;
                let secondary_base = self.secondary_base.ok_or_else(|| {
                    anyhow!("regime-switched envelopes require an explicit secondary base radius")
                })?;
                self.switch_step.ok_or_else(|| {
                    anyhow!("regime-switched envelopes require an explicit switch step")
                })?;
                if !secondary_slope.is_finite() {
                    return Err(anyhow!(
                        "regime-switched secondary slope must be finite; got {}",
                        secondary_slope
                    ));
                }
                if !secondary_base.is_finite() || secondary_base <= 0.0 {
                    return Err(anyhow!(
                        "regime-switched secondary base must be positive and finite; got {}",
                        secondary_base
                    ));
                }
            }
            EnvelopeMode::Fixed
            | EnvelopeMode::Widening
            | EnvelopeMode::Tightening
            | EnvelopeMode::Aggregate => {}
        }
        Ok(())
    }

    // TRACE:DEFINITION:DEF-ENVELOPE-RADIUS:Admissibility envelope radius:Maps configured envelope mode to per-sample radius, derivative bound, and regime label.
    pub fn radius_at(&self, step: usize, time: f64) -> (f64, f64, String) {
        match self.mode {
            EnvelopeMode::Fixed => (self.base_radius, 0.0, "fixed".to_string()),
            EnvelopeMode::Widening => (
                self.base_radius + self.slope * time,
                self.slope,
                "widening".to_string(),
            ),
            EnvelopeMode::Tightening => (
                (self.base_radius - self.slope * time).max(0.05),
                -self.slope,
                "tightening".to_string(),
            ),
            EnvelopeMode::RegimeSwitched => {
                let switch_step = self.switch_step.unwrap_or(0);
                if step < switch_step {
                    (
                        self.base_radius + self.slope * time,
                        self.slope,
                        "regime_nominal".to_string(),
                    )
                } else {
                    let secondary_slope = self.secondary_slope.unwrap_or(self.slope);
                    let secondary_base = self
                        .secondary_base
                        .unwrap_or(self.base_radius + self.slope * time);
                    let local_time = time - switch_step as f64;
                    (
                        secondary_base + secondary_slope * local_time,
                        secondary_slope,
                        "regime_shifted".to_string(),
                    )
                }
            }
            EnvelopeMode::Aggregate => (
                self.base_radius + self.slope * time,
                self.slope,
                "aggregate".to_string(),
            ),
        }
    }
}

// TRACE:ALGORITHM:ALG-ENVELOPE-BUILD:Envelope materialization:Builds the typed admissibility envelope trajectory used by grammar evaluation.
pub fn build_envelope(
    residual: &ResidualTrajectory,
    spec: &EnvelopeSpec,
    scenario_id: &str,
) -> AdmissibilityEnvelope {
    let samples = residual
        .samples
        .iter()
        .map(|sample| {
            let (radius, derivative_bound, regime) = spec.radius_at(sample.step, sample.time);
            EnvelopeSample {
                step: sample.step,
                time: sample.time,
                radius,
                derivative_bound,
                regime,
            }
        })
        .collect::<Vec<_>>();

    AdmissibilityEnvelope {
        scenario_id: scenario_id.to_string(),
        name: spec.name.clone(),
        mode: spec.mode,
        samples,
    }
}

// TRACE:ALGORITHM:ALG-GRAMMAR-EVALUATION:Admissibility grammar evaluation:Assigns admissible, boundary, and violation states with typed reason codes and supporting metrics.
pub fn evaluate_grammar(
    residual: &ResidualTrajectory,
    envelope: &AdmissibilityEnvelope,
) -> Vec<GrammarStatus> {
    let mut boundary_streak = 0usize;
    residual
        .samples
        .iter()
        .zip(&envelope.samples)
        .enumerate()
        .map(|(index, (sample, env))| {
            let margin = env.radius - sample.norm;
            let boundary_band = 0.04 * env.radius.max(1.0);
            let state = if margin < 0.0 {
                GrammarState::Violation
            } else if margin <= boundary_band {
                GrammarState::Boundary
            } else {
                GrammarState::Admissible
            };
            boundary_streak = if matches!(state, GrammarState::Boundary) {
                boundary_streak + 1
            } else {
                0
            };
            let previous_norm = index
                .checked_sub(1)
                .and_then(|previous| residual.samples.get(previous))
                .map(|previous| previous.norm)
                .unwrap_or(sample.norm);
            let norm_delta = sample.norm - previous_norm;
            let abrupt_threshold = 0.08 * env.radius.max(1.0);
            let (reason_code, reason_text) = match state {
                GrammarState::Admissible => (
                    GrammarReasonCode::Admissible,
                    "Residual norm remained inside the configured admissibility envelope."
                        .to_string(),
                ),
                GrammarState::Boundary if boundary_streak >= 2 => (
                    GrammarReasonCode::RecurrentBoundaryGrazing,
                    "Residual norm remained near the configured envelope boundary across consecutive samples."
                        .to_string(),
                ),
                GrammarState::Boundary => (
                    GrammarReasonCode::Boundary,
                    "Residual norm approached the configured admissibility boundary without breaching it."
                        .to_string(),
                ),
                GrammarState::Violation if norm_delta > abrupt_threshold => (
                    GrammarReasonCode::AbruptSlewViolation,
                    "Residual norm breached the configured envelope with an abrupt increase relative to the previous sample."
                        .to_string(),
                ),
                GrammarState::Violation if norm_delta > 0.0 => (
                    GrammarReasonCode::SustainedOutwardDrift,
                    "Residual norm breached the configured envelope during continued outward growth."
                        .to_string(),
                ),
                GrammarState::Violation => (
                    GrammarReasonCode::EnvelopeViolation,
                    "Residual norm remained outside the configured admissibility envelope."
                        .to_string(),
                ),
            };
            let trust_scalar = trust_scalar_for(reason_code, margin, env.radius, boundary_band);
            GrammarStatus {
                scenario_id: residual.scenario_id.clone(),
                step: sample.step,
                time: sample.time,
                state,
                reason_code,
                rule_category: match state {
                    GrammarState::Admissible => "admissible",
                    GrammarState::Boundary => "boundary",
                    GrammarState::Violation => "violation",
                }
                .to_string(),
                reason_text,
                supporting_metric_summary: format!(
                    "margin={}, radius={}, residual_norm={}, norm_delta={}, trust={}",
                    margin,
                    env.radius,
                    sample.norm,
                    norm_delta,
                    trust_scalar.value()
                ),
                margin,
                radius: env.radius,
                residual_norm: sample.norm,
                trust_scalar,
                regime: env.regime.clone(),
            }
        })
        .collect()
}

// TRACE:CLAIM:CLM-TRUST-SEVERITY-MAPPING:Trust scalar from grammar severity:Maps grammar reason and envelope gap to a bounded deterministic trust scalar.
fn trust_scalar_for(
    reason_code: GrammarReasonCode,
    margin: f64,
    radius: f64,
    boundary_band: f64,
) -> TrustScalar {
    let normalized_gap = if margin < 0.0 {
        (-margin / radius.max(1.0e-12)).clamp(0.0, 1.0)
    } else if boundary_band <= 1.0e-12 {
        0.0
    } else {
        ((boundary_band - margin.max(0.0)) / boundary_band).clamp(0.0, 1.0)
    };
    let base_severity = match reason_code {
        GrammarReasonCode::Admissible => 0.0,
        GrammarReasonCode::Boundary => 0.20,
        GrammarReasonCode::RecurrentBoundaryGrazing => 0.35,
        GrammarReasonCode::SustainedOutwardDrift => 0.65,
        GrammarReasonCode::EnvelopeViolation => 0.75,
        GrammarReasonCode::AbruptSlewViolation => 0.85,
    };
    TrustScalar::new(1.0 - (base_severity + 0.15 * normalized_gap).clamp(0.0, 1.0))
}
