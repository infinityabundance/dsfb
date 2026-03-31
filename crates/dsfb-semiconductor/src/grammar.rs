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
    pub states: Vec<GrammarState>,
    pub reasons: Vec<GrammarReason>,
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
                (GrammarState::Boundary, GrammarReason::RecurrentBoundaryGrazing)
            } else {
                (GrammarState::Admissible, GrammarReason::Admissible)
            };

            states.push(state);
            reasons.push(reason);
        }

        traces.push(FeatureGrammarTrace {
            feature_index: residual_trace.feature_index,
            feature_name: residual_trace.feature_name.clone(),
            states,
            reasons,
        });
    }

    GrammarSet { traces }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nominal::{NominalFeature, NominalModel};
    use crate::residual::{ResidualFeatureTrace, ResidualSet};
    use crate::signs::{FeatureSigns, SignSet};

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
        let grammar = evaluate_grammar(&residuals, &signs, &nominal, &PipelineConfig::default());
        assert_eq!(grammar.traces[0].states[1], GrammarState::Violation);
        assert_eq!(grammar.traces[0].reasons[1], GrammarReason::EnvelopeViolation);
    }
}
