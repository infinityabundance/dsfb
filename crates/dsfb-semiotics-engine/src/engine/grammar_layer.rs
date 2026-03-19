use crate::engine::types::{
    AdmissibilityEnvelope, DetectabilityBoundInputs, DetectabilityResult, GrammarStatus,
    ResidualTrajectory,
};
use crate::math::detectability::compute_detectability_result;
use crate::math::envelope::evaluate_grammar;

pub fn evaluate_grammar_layer(
    residual: &ResidualTrajectory,
    envelope: &AdmissibilityEnvelope,
) -> Vec<GrammarStatus> {
    evaluate_grammar(residual, envelope)
}

pub fn evaluate_detectability(
    residual: &ResidualTrajectory,
    grammar: &[GrammarStatus],
    bound_inputs: Option<DetectabilityBoundInputs>,
    reference: Option<&ResidualTrajectory>,
) -> DetectabilityResult {
    compute_detectability_result(
        &residual.scenario_id,
        residual,
        grammar,
        bound_inputs,
        reference,
    )
}
