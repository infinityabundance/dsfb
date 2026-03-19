use crate::engine::types::{
    AdmissibilityEnvelope, EnvelopeMode, EnvelopeSample, GrammarState, GrammarStatus,
    ResidualTrajectory,
};

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

pub fn evaluate_grammar(
    residual: &ResidualTrajectory,
    envelope: &AdmissibilityEnvelope,
) -> Vec<GrammarStatus> {
    residual
        .samples
        .iter()
        .zip(&envelope.samples)
        .map(|(sample, env)| {
            let margin = env.radius - sample.norm;
            let state = if margin < 0.0 {
                GrammarState::Violation
            } else if margin <= 0.04 * env.radius.max(1.0) {
                GrammarState::Boundary
            } else {
                GrammarState::Admissible
            };
            GrammarStatus {
                scenario_id: residual.scenario_id.clone(),
                step: sample.step,
                time: sample.time,
                state,
                margin,
                radius: env.radius,
                residual_norm: sample.norm,
                regime: env.regime.clone(),
            }
        })
        .collect()
}
