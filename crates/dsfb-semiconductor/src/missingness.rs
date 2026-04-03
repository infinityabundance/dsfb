//! Missingness-aware grammar: invalidates drift computation when sensor data
//! is absent for more than [`MAX_CONSECUTIVE_MISSING_RUNS`] consecutive runs.
//!
//! # Hardware Reality Check
//! Semiconductor sensors fail.  An MFC with a broken transducer will produce
//! a flat line of zeros, not NaN — and a naive DSFB engine will interpret
//! a sustained zero residual as "nominal" when the truth is "unknown."
//!
//! More dangerously, imputed values (mean-fill) can accumulate into a
//! spurious drift signal over a long outage window, causing the engine to
//! escalate a phantom anomaly that exists only in the imputation model.
//!
//! # Policy
//! * If a feature's sensor is missing for `> MAX_CONSECUTIVE_MISSING_RUNS`
//!   consecutive runs, the drift value `d` is **invalidated** and set to
//!   [`DriftValidity::Unknown`].
//! * Grammar transitions from [`DriftValidity::Unknown`] features are
//!   **suppressed** — the feature is held at its last valid grammar state with
//!   a `suppressed_by_missingness` flag set to `true`.
//! * The outage event is recorded verbatim in the traceability manifest,
//!   preserving the audit trail.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum number of consecutive missing runs before drift is invalidated.
pub const MAX_CONSECUTIVE_MISSING_RUNS: usize = 3;

// ─── Drift Validity ───────────────────────────────────────────────────────────

/// Marks whether the first-difference (drift) value for a run is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftValidity {
    /// Drift computed from two consecutive non-missing values.
    Valid,
    /// Drift invalidated due to missing data beyond the permitted window.
    Unknown,
}

// ─── Feature Missingness Tracker ─────────────────────────────────────────────

/// Per-feature missingness tracker.
///
/// Accumulates consecutive missing runs and emits [`DriftValidity::Unknown`]
/// once the threshold is exceeded.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureMissingnessTracker {
    /// Feature identifier.
    pub feature_id: String,
    /// Current consecutive missing run count.
    pub consecutive_missing: usize,
    /// Total missing runs across the entire run sequence.
    pub total_missing: usize,
    /// Run indices where missingness triggered drift invalidation.
    pub invalidation_events: Vec<usize>,
}

impl FeatureMissingnessTracker {
    pub fn new(feature_id: impl Into<String>) -> Self {
        Self {
            feature_id: feature_id.into(),
            ..Default::default()
        }
    }

    /// Update the tracker for a single run.
    ///
    /// * `is_missing` — whether the sensor value for this run is absent.
    /// * `run_index` — zero-based run counter (used in invalidation events).
    ///
    /// Returns the [`DriftValidity`] for this run.
    pub fn update(&mut self, is_missing: bool, run_index: usize) -> DriftValidity {
        if is_missing {
            self.consecutive_missing += 1;
            self.total_missing += 1;
        } else {
            self.consecutive_missing = 0;
        }

        if self.consecutive_missing > MAX_CONSECUTIVE_MISSING_RUNS {
            self.invalidation_events.push(run_index);
            DriftValidity::Unknown
        } else {
            DriftValidity::Valid
        }
    }

    /// Returns `true` if drift is currently invalidated.
    #[must_use]
    pub fn is_invalidated(&self) -> bool {
        self.consecutive_missing > MAX_CONSECUTIVE_MISSING_RUNS
    }
}

// ─── Missingness-Aware Run Record ─────────────────────────────────────────────

/// The annotated record for a single run after missingness processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingnessAwareRecord {
    pub run_index: usize,
    pub feature_id: String,
    /// The observed or imputed value.
    pub value: f64,
    /// Whether the original observation was missing.
    pub is_missing: bool,
    /// Drift validity for this point.
    pub drift_validity: DriftValidity,
    /// Whether the grammar transition at this point is suppressed.
    pub suppressed_by_missingness: bool,
}

// ─── Missingness-Aware Grammar Filter ────────────────────────────────────────

/// Applies the missingness policy across all features in a run sequence.
///
/// Call [`MissingnessAwareGrammar::process`] once per feature with the
/// raw imputed-value vector and the corresponding `is_imputed` mask.
///
/// The returned [`MissingnessAwareRecord`] vector can be used to gate the
/// downstream grammar layer: any record with
/// `suppressed_by_missingness = true` should be held at the previous
/// grammar state rather than allowing a new transition.
#[derive(Debug, Default)]
pub struct MissingnessAwareGrammar {
    trackers: HashMap<String, FeatureMissingnessTracker>,
}

impl MissingnessAwareGrammar {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a feature's run sequence and return annotated records.
    ///
    /// # Arguments
    /// * `feature_id` — sensor identifier.
    /// * `values` — slice of imputed sensor values (one per run).
    /// * `is_imputed` — parallel slice indicating which values were imputed
    ///   (i.e., the original sensor reading was missing).
    pub fn process(
        &mut self,
        feature_id: &str,
        values: &[f64],
        is_imputed: &[bool],
    ) -> Vec<MissingnessAwareRecord> {
        assert_eq!(
            values.len(),
            is_imputed.len(),
            "values and is_imputed must have equal length"
        );

        let tracker = self
            .trackers
            .entry(feature_id.to_string())
            .or_insert_with(|| FeatureMissingnessTracker::new(feature_id));

        // Reset between calls — each call processes a fresh run sequence.
        tracker.consecutive_missing = 0;
        tracker.invalidation_events.clear();

        values
            .iter()
            .zip(is_imputed.iter())
            .enumerate()
            .map(|(run_index, (&value, &is_missing))| {
                let drift_validity = tracker.update(is_missing, run_index);
                MissingnessAwareRecord {
                    run_index,
                    feature_id: feature_id.to_string(),
                    value,
                    is_missing,
                    drift_validity,
                    suppressed_by_missingness: drift_validity == DriftValidity::Unknown,
                }
            })
            .collect()
    }

    /// Return all feature trackers for serialisation into the traceability
    /// manifest.
    pub fn trackers(&self) -> &HashMap<String, FeatureMissingnessTracker> {
        &self.trackers
    }

    /// Return a summary for embedding in the run manifest JSON.
    pub fn summary(&self) -> MissingSummary {
        let total_features = self.trackers.len();
        let features_with_invalidations = self
            .trackers
            .values()
            .filter(|t| !t.invalidation_events.is_empty())
            .count();
        let total_invalidation_events: usize = self
            .trackers
            .values()
            .map(|t| t.invalidation_events.len())
            .sum();
        let total_missing_observations: usize =
            self.trackers.values().map(|t| t.total_missing).sum();

        MissingSummary {
            total_features,
            features_with_invalidations,
            total_invalidation_events,
            total_missing_observations,
            max_consecutive_missing_threshold: MAX_CONSECUTIVE_MISSING_RUNS,
        }
    }
}

// ─── Summary ────────────────────────────────────────────────────────────────

/// Compact summary of missingness across all features — emitted in the
/// run manifest JSON for audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingSummary {
    pub total_features: usize,
    pub features_with_invalidations: usize,
    pub total_invalidation_events: usize,
    pub total_missing_observations: usize,
    pub max_consecutive_missing_threshold: usize,
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_valid_below_threshold() {
        let mut t = FeatureMissingnessTracker::new("S001");
        assert_eq!(t.update(true, 0), DriftValidity::Valid);
        assert_eq!(t.update(true, 1), DriftValidity::Valid);
        assert_eq!(t.update(true, 2), DriftValidity::Valid);
        // exactly at threshold: still Valid
        assert_eq!(t.consecutive_missing, 3);
    }

    #[test]
    fn tracker_invalidates_after_threshold() {
        let mut t = FeatureMissingnessTracker::new("S002");
        for i in 0..=3 {
            t.update(true, i);
        }
        // 4th consecutive missing → Unknown
        assert_eq!(t.update(true, 4), DriftValidity::Unknown);
        assert!(t.is_invalidated());
    }

    #[test]
    fn tracker_resets_on_valid_observation() {
        let mut t = FeatureMissingnessTracker::new("S003");
        for i in 0..10 {
            t.update(true, i);
        }
        // Valid observation resets streak
        assert_eq!(t.update(false, 10), DriftValidity::Valid);
        assert_eq!(t.consecutive_missing, 0);
        assert!(!t.is_invalidated());
    }

    #[test]
    fn grammar_filter_suppresses_after_threshold() {
        let mut grammar = MissingnessAwareGrammar::new();
        let values: Vec<f64> = vec![0.0; 8];
        // First 4 are missing, rest are present
        let is_imputed = vec![true, true, true, true, false, false, false, false];
        let records = grammar.process("S001", &values, &is_imputed);

        // Runs 0-2: consecutive missing ≤ 3 → Valid
        assert_eq!(records[0].drift_validity, DriftValidity::Valid);
        assert_eq!(records[2].drift_validity, DriftValidity::Valid);
        // Run 3: 4th consecutive missing → Unknown
        assert_eq!(records[3].drift_validity, DriftValidity::Unknown);
        assert!(records[3].suppressed_by_missingness);
        // After valid observation, back to Valid
        assert_eq!(records[4].drift_validity, DriftValidity::Valid);
        assert!(!records[4].suppressed_by_missingness);
    }

    #[test]
    fn summary_counts_invalidated_features() {
        let mut grammar = MissingnessAwareGrammar::new();
        // Feature with >3 consecutive missing
        let values = vec![0.0; 5];
        let all_missing = vec![true; 5];
        grammar.process("S_BAD", &values, &all_missing);

        // Feature with no missingness
        let none_missing = vec![false; 5];
        grammar.process("S_GOOD", &values, &none_missing);

        let summary = grammar.summary();
        assert_eq!(summary.total_features, 2);
        assert_eq!(summary.features_with_invalidations, 1);
    }
}
