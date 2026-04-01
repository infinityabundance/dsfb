use crate::config::PipelineConfig;
use crate::nominal::NominalModel;
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GrammarState {
    Admissible,
    Boundary,
    Violation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GrammarReason {
    Admissible,
    SustainedOutwardDrift,
    AbruptSlewViolation,
    RecurrentBoundaryGrazing,
    EnvelopeViolation,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureGrammarTrace {
    pub feature_index: usize,
    pub feature_name: String,
    pub raw_states: Vec<GrammarState>,
    pub raw_reasons: Vec<GrammarReason>,
    pub states: Vec<GrammarState>,
    pub reasons: Vec<GrammarReason>,
    pub persistent_boundary: Vec<bool>,
    pub persistent_violation: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GrammarSet {
    pub traces: Vec<FeatureGrammarTrace>,
}

pub fn evaluate_grammar(
    residuals: &ResidualSet,
    signs: &SignSet,
    nominal: &NominalModel,
    config: &PipelineConfig,
) -> GrammarSet {
    let mut traces = Vec::with_capacity(residuals.traces.len());

    for (residual_trace, sign_trace) in residuals.traces.iter().zip(&signs.traces) {
        let feature = &nominal.features[residual_trace.feature_index];
        let (raw_states, raw_reasons) =
            evaluate_raw_trace(residual_trace, sign_trace, feature, config);
        let (states, reasons) =
            apply_hysteresis(&raw_states, &raw_reasons, config.state_confirmation_steps);
        let persistent_boundary = persistent_mask(
            &states,
            GrammarState::Boundary,
            config.persistent_state_steps,
        );
        let persistent_violation = persistent_mask(
            &states,
            GrammarState::Violation,
            config.persistent_state_steps,
        );

        traces.push(FeatureGrammarTrace {
            feature_index: residual_trace.feature_index,
            feature_name: residual_trace.feature_name.clone(),
            raw_states,
            raw_reasons,
            states,
            reasons,
            persistent_boundary,
            persistent_violation,
        });
    }

    GrammarSet { traces }
}

fn evaluate_raw_trace(
    residual_trace: &crate::residual::ResidualFeatureTrace,
    sign_trace: &crate::signs::FeatureSigns,
    feature: &crate::nominal::NominalFeature,
    config: &PipelineConfig,
) -> (Vec<GrammarState>, Vec<GrammarReason>) {
    let mut states = Vec::with_capacity(residual_trace.norms.len());
    let mut reasons = Vec::with_capacity(residual_trace.norms.len());

    for index in 0..residual_trace.norms.len() {
        let zone_start = index.saturating_sub(config.grazing_window.saturating_sub(1));
        let zone_hits = residual_trace.norms[zone_start..=index]
            .iter()
            .filter(|value| **value > config.boundary_fraction_of_rho * feature.rho)
            .count();

        let norm = residual_trace.norms[index];
        let drift = sign_trace.drift[index];
        let slew = sign_trace.slew[index].abs();

        let (state, reason) = if !feature.analyzable {
            (GrammarState::Admissible, GrammarReason::Admissible)
        } else if norm > feature.rho {
            (GrammarState::Violation, GrammarReason::EnvelopeViolation)
        } else if norm > config.boundary_fraction_of_rho * feature.rho
            && drift >= sign_trace.drift_threshold
            && slew >= sign_trace.slew_threshold
        {
            (GrammarState::Boundary, GrammarReason::AbruptSlewViolation)
        } else if norm > config.boundary_fraction_of_rho * feature.rho
            && drift >= sign_trace.drift_threshold
        {
            (GrammarState::Boundary, GrammarReason::SustainedOutwardDrift)
        } else if norm > config.boundary_fraction_of_rho * feature.rho
            && zone_hits >= config.grazing_min_hits
        {
            (
                GrammarState::Boundary,
                GrammarReason::RecurrentBoundaryGrazing,
            )
        } else {
            (GrammarState::Admissible, GrammarReason::Admissible)
        };

        states.push(state);
        reasons.push(reason);
    }

    (states, reasons)
}

fn apply_hysteresis(
    raw_states: &[GrammarState],
    raw_reasons: &[GrammarReason],
    confirmation_steps: usize,
) -> (Vec<GrammarState>, Vec<GrammarReason>) {
    if raw_states.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut states = Vec::with_capacity(raw_states.len());
    let mut reasons = Vec::with_capacity(raw_states.len());
    let mut current_state = raw_states[0];
    let mut current_reason = raw_reasons[0];
    let mut candidate_state: Option<GrammarState> = None;
    let mut candidate_count = 0usize;

    for (&raw_state, &raw_reason) in raw_states.iter().zip(raw_reasons) {
        if raw_state == current_state {
            candidate_state = None;
            candidate_count = 0;
            if raw_state != GrammarState::Admissible {
                current_reason = raw_reason;
            } else {
                current_reason = GrammarReason::Admissible;
            }
        } else if candidate_state == Some(raw_state) {
            candidate_count += 1;
        } else {
            candidate_state = Some(raw_state);
            candidate_count = 1;
        }

        if let Some(next_state) = candidate_state {
            if candidate_count >= confirmation_steps {
                current_state = next_state;
                current_reason = if current_state == GrammarState::Admissible {
                    GrammarReason::Admissible
                } else {
                    raw_reason
                };
                candidate_state = None;
                candidate_count = 0;
            }
        }

        states.push(current_state);
        reasons.push(if current_state == GrammarState::Admissible {
            GrammarReason::Admissible
        } else {
            current_reason
        });
    }

    (states, reasons)
}

fn persistent_mask(
    states: &[GrammarState],
    target: GrammarState,
    minimum_steps: usize,
) -> Vec<bool> {
    let mut out = Vec::with_capacity(states.len());
    let mut consecutive = 0usize;

    for &state in states {
        if state == target {
            consecutive += 1;
            out.push(consecutive >= minimum_steps);
        } else {
            consecutive = 0;
            out.push(false);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nominal::{NominalFeature, NominalModel};
    use crate::residual::{ResidualFeatureTrace, ResidualSet};
    use crate::signs::{FeatureSigns, SignSet};

    fn test_config() -> PipelineConfig {
        PipelineConfig {
            state_confirmation_steps: 1,
            persistent_state_steps: 2,
            ..PipelineConfig::default()
        }
    }

    #[test]
    fn violation_state_wins_over_boundary() {
        let residuals = ResidualSet {
            traces: vec![ResidualFeatureTrace {
                feature_index: 0,
                feature_name: "S001".into(),
                imputed_values: vec![0.0, 2.0],
                residuals: vec![0.0, 2.0],
                norms: vec![0.0, 2.0],
                threshold_alarm: vec![false, true],
            }],
        };
        let signs = SignSet {
            traces: vec![FeatureSigns {
                feature_index: 0,
                feature_name: "S001".into(),
                drift: vec![0.0, 1.0],
                slew: vec![0.0, 1.0],
                drift_threshold: 0.1,
                slew_threshold: 0.1,
            }],
        };
        let nominal = NominalModel {
            features: vec![NominalFeature {
                feature_index: 0,
                feature_name: "S001".into(),
                healthy_mean: 0.0,
                healthy_std: 0.5,
                rho: 1.5,
                healthy_observations: 10,
                analyzable: true,
            }],
        };
        let grammar = evaluate_grammar(&residuals, &signs, &nominal, &test_config());
        assert_eq!(grammar.traces[0].raw_states[1], GrammarState::Violation);
        assert_eq!(grammar.traces[0].states[1], GrammarState::Violation);
        assert_eq!(
            grammar.traces[0].raw_reasons[1],
            GrammarReason::EnvelopeViolation
        );
    }

    #[test]
    fn hysteresis_requires_confirmation_before_state_change() {
        let raw_states = vec![
            GrammarState::Admissible,
            GrammarState::Boundary,
            GrammarState::Admissible,
            GrammarState::Boundary,
            GrammarState::Boundary,
        ];
        let raw_reasons = vec![
            GrammarReason::Admissible,
            GrammarReason::SustainedOutwardDrift,
            GrammarReason::Admissible,
            GrammarReason::SustainedOutwardDrift,
            GrammarReason::SustainedOutwardDrift,
        ];
        let (states, _) = apply_hysteresis(&raw_states, &raw_reasons, 2);
        assert_eq!(
            states,
            vec![
                GrammarState::Admissible,
                GrammarState::Admissible,
                GrammarState::Admissible,
                GrammarState::Admissible,
                GrammarState::Boundary,
            ]
        );
    }

    #[test]
    fn persistence_mask_starts_after_minimum_consecutive_steps() {
        let states = vec![
            GrammarState::Admissible,
            GrammarState::Boundary,
            GrammarState::Boundary,
            GrammarState::Boundary,
            GrammarState::Admissible,
        ];
        let mask = persistent_mask(&states, GrammarState::Boundary, 2);
        assert_eq!(mask, vec![false, false, true, true, false]);
    }
}
