//! Episode formation: the operator-facing output object.
//!
//! An [`Episode`] represents a contiguous period during which the grammar
//! state held a specific classification. Episodes are the primary output
//! consumed by operators, dashboards, and automated response systems.

use crate::grammar::GrammarState;
use crate::residual::ResidualSource;
use crate::ReasonCode;

/// A structural episode: a contiguous period of a specific grammar state.
///
/// This is the operator-facing output of the DSFB engine. Each episode
/// carries the full audit context needed to understand why the classification
/// was made and what structural pattern was detected.
#[derive(Debug, Clone)]
pub struct Episode {
    /// Monotonic timestamp when this episode began (nanoseconds).
    pub start_ts: u64,
    /// Monotonic timestamp when this episode ended (nanoseconds).
    /// `None` if the episode is still open.
    pub end_ts: Option<u64>,
    /// Grammar state during this episode.
    pub grammar_state: GrammarState,
    /// Reason code from the heuristics bank match.
    pub reason_code: ReasonCode,
    /// Which residual source(s) contributed to this classification.
    pub primary_source: ResidualSource,
    /// Maximum absolute drift observed during this episode.
    pub max_drift: f64,
    /// Maximum absolute slew observed during this episode.
    pub max_slew: f64,
    /// Maximum absolute residual observed during this episode.
    pub max_residual: f64,
    /// Number of observation samples in this episode.
    pub sample_count: u32,
}

impl Episode {
    /// Duration of this episode in nanoseconds, or `None` if still open.
    pub fn duration_ns(&self) -> Option<u64> {
        self.end_ts.map(|end| end.saturating_sub(self.start_ts))
    }

    /// Whether this episode is still open (no end timestamp).
    pub fn is_open(&self) -> bool {
        self.end_ts.is_none()
    }

    /// Whether this episode represents a structural anomaly
    /// (Boundary or Violation).
    pub fn is_anomalous(&self) -> bool {
        matches!(
            self.grammar_state,
            GrammarState::Boundary | GrammarState::Violation
        )
    }
}

/// Builder for constructing episodes incrementally as observations arrive.
pub struct EpisodeBuilder {
    current: Option<Episode>,
}

impl EpisodeBuilder {
    /// Create a new episode builder.
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Open a new episode at the given timestamp with the given state.
    pub fn open(
        &mut self,
        timestamp_ns: u64,
        grammar_state: GrammarState,
        reason_code: ReasonCode,
        source: ResidualSource,
    ) {
        self.current = Some(Episode {
            start_ts: timestamp_ns,
            end_ts: None,
            grammar_state,
            reason_code,
            primary_source: source,
            max_drift: 0.0,
            max_slew: 0.0,
            max_residual: 0.0,
            sample_count: 0,
        });
    }

    /// Update the current open episode with a new observation.
    pub fn update(&mut self, residual: f64, drift: f64, slew: f64) {
        if let Some(ref mut ep) = self.current {
            ep.max_drift = ep.max_drift.max(drift.abs());
            ep.max_slew = ep.max_slew.max(slew.abs());
            ep.max_residual = ep.max_residual.max(residual.abs());
            ep.sample_count += 1;
        }
    }

    /// Close the current episode and return it.
    pub fn close(&mut self, timestamp_ns: u64) -> Option<Episode> {
        if let Some(mut ep) = self.current.take() {
            ep.end_ts = Some(timestamp_ns);
            Some(ep)
        } else {
            None
        }
    }

    /// Whether an episode is currently open.
    pub fn is_open(&self) -> bool {
        self.current.is_some()
    }

    /// Reference to the current open episode, if any.
    pub fn current(&self) -> Option<&Episode> {
        self.current.as_ref()
    }
}

impl Default for EpisodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episode_lifecycle() {
        let mut builder = EpisodeBuilder::new();
        assert!(!builder.is_open());

        builder.open(
            1000,
            GrammarState::Boundary,
            ReasonCode::SustainedLatencyDrift,
            ResidualSource::Latency,
        );
        assert!(builder.is_open());

        builder.update(5.0, 0.3, 0.01);
        builder.update(6.0, 0.4, 0.02);

        let ep = builder.close(3000).unwrap();
        assert_eq!(ep.start_ts, 1000);
        assert_eq!(ep.end_ts, Some(3000));
        assert_eq!(ep.grammar_state, GrammarState::Boundary);
        assert_eq!(ep.sample_count, 2);
        assert!((ep.max_drift - 0.4).abs() < 1e-10);
        assert!(ep.is_anomalous());
    }
}
