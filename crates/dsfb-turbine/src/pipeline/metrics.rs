//! Fleet-level evaluation metrics.
//!
//! Aggregates per-engine results into fleet-wide statistics.

use crate::pipeline::engine_eval::EngineEvalResult;

/// Fleet-level evaluation summary.
#[derive(Debug)]
pub struct FleetMetrics {
    /// Total engines evaluated.
    pub total_engines: usize,
    /// Engines with at least one Boundary episode.
    pub engines_with_boundary: usize,
    /// Engines with at least one Violation episode.
    pub engines_with_violation: usize,
    /// Mean structural lead time (cycles before end-of-life).
    pub mean_lead_time: f64,
    /// Median structural lead time.
    pub median_lead_time: f64,
    /// Min structural lead time.
    pub min_lead_time: u32,
    /// Max structural lead time.
    pub max_lead_time: u32,
    /// Total episodes across fleet.
    pub total_episodes: usize,
    /// Mean episodes per engine.
    pub mean_episodes_per_engine: f64,
    /// Fraction of engines where Theorem 1 bound was satisfied.
    pub theorem_satisfaction_rate: f64,
    /// Engines where DSFB grammar transition preceded RUL=30 threshold.
    pub early_warning_count: usize,
}

/// Computes fleet-level metrics from individual engine results.
pub fn compute_fleet_metrics(results: &[EngineEvalResult]) -> FleetMetrics {
    let n = results.len();
    if n == 0 {
        return FleetMetrics {
            total_engines: 0,
            engines_with_boundary: 0,
            engines_with_violation: 0,
            mean_lead_time: 0.0,
            median_lead_time: 0.0,
            min_lead_time: 0,
            max_lead_time: 0,
            total_episodes: 0,
            mean_episodes_per_engine: 0.0,
            theorem_satisfaction_rate: 0.0,
            early_warning_count: 0,
        };
    }

    let engines_with_boundary = results.iter()
        .filter(|r| r.first_boundary_cycle.is_some())
        .count();
    let engines_with_violation = results.iter()
        .filter(|r| r.first_violation_cycle.is_some())
        .count();

    let mut lead_times: Vec<u32> = results.iter()
        .filter_map(|r| r.structural_lead_time)
        .collect();
    lead_times.sort();

    let mean_lead = if lead_times.is_empty() {
        0.0
    } else {
        lead_times.iter().sum::<u32>() as f64 / lead_times.len() as f64
    };
    let median_lead = if lead_times.is_empty() {
        0.0
    } else {
        lead_times[lead_times.len() / 2] as f64
    };
    let min_lead = lead_times.first().copied().unwrap_or(0);
    let max_lead = lead_times.last().copied().unwrap_or(0);

    let total_episodes: usize = results.iter()
        .map(|r| r.episodes.len())
        .sum();

    let theorem_satisfied = results.iter()
        .filter(|r| r.theorem_bound.as_ref().map_or(false, |t| t.bound_satisfied))
        .count();
    let theorem_evaluated = results.iter()
        .filter(|r| r.theorem_bound.is_some())
        .count();
    let theorem_rate = if theorem_evaluated > 0 {
        theorem_satisfied as f64 / theorem_evaluated as f64
    } else {
        0.0
    };

    // Early warning: DSFB boundary before RUL=30
    let early_warning = results.iter()
        .filter(|r| {
            if let Some(fb) = r.first_boundary_cycle {
                let rul_at_boundary = if r.total_cycles > fb {
                    r.total_cycles - fb
                } else { 0 };
                rul_at_boundary > 30 // Boundary detected with >30 cycles remaining
            } else {
                false
            }
        })
        .count();

    FleetMetrics {
        total_engines: n,
        engines_with_boundary,
        engines_with_violation,
        mean_lead_time: mean_lead,
        median_lead_time: median_lead,
        min_lead_time: min_lead,
        max_lead_time: max_lead,
        total_episodes,
        mean_episodes_per_engine: total_episodes as f64 / n as f64,
        theorem_satisfaction_rate: theorem_rate,
        early_warning_count: early_warning,
    }
}
