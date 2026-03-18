use serde::Serialize;

use crate::detectability::{DetectabilitySummary, SemanticStatus};
use crate::heuristics::{AmbiguityTier, HeuristicRanking};

#[derive(Clone, Debug, Serialize)]
pub struct SemanticAssessment {
    pub semantic_status: SemanticStatus,
    pub semantic_reason: String,
}

pub fn assess_semantic_status(
    detectability: &DetectabilitySummary,
    ranking: Option<&HeuristicRanking>,
) -> SemanticAssessment {
    let ambiguity_dominates = ranking
        .map(|entry| {
            matches!(
                entry.ambiguity_tier,
                AmbiguityTier::NearTie | AmbiguityTier::Ambiguous | AmbiguityTier::Unavailable
            )
        })
        .unwrap_or(false);

    let semantic_status = if detectability.first_crossing_step.is_none() {
        SemanticStatus::NotDetected
    } else if ambiguity_dominates {
        match detectability.semantic_status {
            SemanticStatus::Degraded => SemanticStatus::DegradedAmbiguous,
            SemanticStatus::NotDetected => SemanticStatus::NotDetected,
            _ => SemanticStatus::Ambiguous,
        }
    } else {
        detectability.semantic_status
    };

    let semantic_reason = if detectability.first_crossing_step.is_none() {
        "No mathematical pointwise crossing was observed, so the final semantic status is not_detected."
            .to_string()
    } else if ambiguity_dominates {
        let tier = ranking
            .map(|entry| entry.ambiguity_tier.as_str())
            .unwrap_or("unavailable");
        match semantic_status {
            SemanticStatus::DegradedAmbiguous => format!(
                "{} The retrieval layer is also {} rather than unambiguous, so the final semantic status is degraded_ambiguous.",
                detectability.semantic_reason, tier
            ),
            SemanticStatus::Ambiguous => format!(
                "{} The retrieval layer is also {} rather than unambiguous, so the final semantic status is ambiguous.",
                detectability.semantic_reason, tier
            ),
            _ => detectability.semantic_reason.clone(),
        }
    } else {
        detectability.semantic_reason.clone()
    };

    SemanticAssessment {
        semantic_status,
        semantic_reason,
    }
}
