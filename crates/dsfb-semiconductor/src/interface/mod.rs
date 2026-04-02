use crate::grammar::layer::{build_grammar_states, GrammarState};
use crate::input::alarm_stream::{AlarmSample, AlarmStream};
use crate::input::residual_stream::{ResidualSample, ResidualStream};
use crate::policy::{derive_policy, PolicyDecision};
use crate::semantics::{match_semantics, minimal_heuristics_bank, Heuristic, SemanticMatch};
use crate::sign::{build_feature_signs, FeatureSignPoint};
use crate::syntax::{build_motifs, Motif, MotifTimelinePoint};
use std::sync::RwLock;

pub trait DSFBObserver {
    fn ingest(&self, residual: &ResidualSample);
    fn output(&self) -> Vec<PolicyDecision>;
}

#[derive(Debug, Default)]
pub struct ReadOnlyDsfbObserver {
    residuals: RwLock<Vec<ResidualSample>>,
    alarms: RwLock<Vec<AlarmSample>>,
}

#[derive(Debug, Clone)]
pub struct ObserverArtifacts {
    pub signs: Vec<FeatureSignPoint>,
    pub motifs: Vec<Motif>,
    pub motif_timeline: Vec<MotifTimelinePoint>,
    pub grammar_states: Vec<GrammarState>,
    pub heuristics: Vec<Heuristic>,
    pub semantic_matches: Vec<SemanticMatch>,
    pub policy_decisions: Vec<PolicyDecision>,
}

impl ReadOnlyDsfbObserver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_alarm_stream(alarm_stream: AlarmStream) -> Self {
        Self {
            residuals: RwLock::new(Vec::new()),
            alarms: RwLock::new(alarm_stream.samples().to_vec()),
        }
    }

    pub fn ingest_alarm(&self, alarm: &AlarmSample) {
        self.alarms.write().unwrap().push(alarm.clone());
        self.alarms.write().unwrap().sort_by(|left, right| {
            left.timestamp
                .total_cmp(&right.timestamp)
                .then_with(|| left.source.cmp(&right.source))
        });
    }

    pub fn residual_stream(&self) -> ResidualStream {
        ResidualStream::new(self.residuals.read().unwrap().clone())
    }

    pub fn alarm_stream(&self) -> AlarmStream {
        AlarmStream::new(self.alarms.read().unwrap().clone())
    }

    pub fn layered_output(&self) -> ObserverArtifacts {
        let residual_stream = self.residual_stream();
        let signs = build_feature_signs(&residual_stream);
        let syntax = build_motifs(&signs);
        let grammar_states = build_grammar_states(&signs, &syntax.timeline);
        let heuristics = minimal_heuristics_bank();
        let semantic_matches = match_semantics(&syntax.timeline, &grammar_states, &heuristics);
        let policy_decisions = derive_policy(&semantic_matches, &grammar_states);
        ObserverArtifacts {
            signs,
            motifs: syntax.motifs,
            motif_timeline: syntax.timeline,
            grammar_states,
            heuristics,
            semantic_matches,
            policy_decisions,
        }
    }
}

impl DSFBObserver for ReadOnlyDsfbObserver {
    fn ingest(&self, residual: &ResidualSample) {
        self.residuals.write().unwrap().push(residual.clone());
        self.residuals.write().unwrap().sort_by(|left, right| {
            left.timestamp
                .total_cmp(&right.timestamp)
                .then_with(|| left.feature_id.cmp(&right.feature_id))
        });
    }

    fn output(&self) -> Vec<PolicyDecision> {
        self.layered_output().policy_decisions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observer_replay_is_deterministic() {
        let observer = ReadOnlyDsfbObserver::new();
        let sample = ResidualSample {
            timestamp: 1.0,
            feature_id: "S059".into(),
            value: 2.0,
        };
        observer.ingest(&sample);
        observer.ingest(&ResidualSample {
            timestamp: 2.0,
            feature_id: "S059".into(),
            value: 3.0,
        });
        observer.ingest(&ResidualSample {
            timestamp: 3.0,
            feature_id: "S059".into(),
            value: 4.2,
        });
        let first = observer.layered_output();
        let second = observer.layered_output();
        assert_eq!(first.policy_decisions, second.policy_decisions);
        assert_eq!(first.semantic_matches, second.semantic_matches);
        assert_eq!(first.grammar_states, second.grammar_states);
    }
}
