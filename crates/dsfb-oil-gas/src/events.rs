/// DSFB Oil & Gas — Episode Aggregation
///
/// Collapses a sequence of AnnotatedStep into a compact Episode log.
/// Each Episode is a maximal contiguous run of identical GrammarState.
///
/// This module also computes EpisodeSummary statistics.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::types::{AnnotatedStep, Episode, EpisodeSummary, GrammarState};

/// Build an episode list from a slice of annotated steps.
///
/// A new episode begins whenever the GrammarState changes from the previous step.
pub fn aggregate_episodes(steps: &[AnnotatedStep]) -> Vec<Episode> {
    if steps.is_empty() {
        return Vec::new();
    }
    let mut episodes: Vec<Episode> = Vec::new();
    let mut current = build_episode_start(&steps[0]);

    for step in &steps[1..] {
        if step.state == current.state {
            // extend current episode
            current.end_ts = step.triple.timestamp;
            current.step_count += 1;
            current.peak_r     = current.peak_r    .max(step.triple.r    .abs());
            current.peak_delta = current.peak_delta.max(step.triple.delta.abs());
            current.peak_sigma = current.peak_sigma.max(step.triple.sigma.abs());
            current.drift_sign += step.triple.delta.signum();
        } else {
            // finalise and start new
            episodes.push(current);
            current = build_episode_start(step);
        }
    }
    episodes.push(current);
    episodes
}

fn build_episode_start(step: &AnnotatedStep) -> Episode {
    Episode {
        state: step.state,
        channel: step.channel.clone(),
        start_ts: step.triple.timestamp,
        end_ts:   step.triple.timestamp,
        step_count: 1,
        peak_r:     step.triple.r    .abs(),
        peak_delta: step.triple.delta.abs(),
        peak_sigma: step.triple.sigma.abs(),
        drift_sign: step.triple.delta.signum(),
        reason: step.reason.clone(),
    }
}

/// Compute summary statistics from the episode list and total step count.
pub fn summarise(channel: &str, steps: &[AnnotatedStep], episodes: &[Episode]) -> EpisodeSummary {
    let total_steps = steps.len();
    let total_episodes = episodes.len();
    let nominal_steps = steps.iter().filter(|s| s.state == GrammarState::Nominal).count();
    let non_nominal_episodes = episodes.iter().filter(|e| e.state != GrammarState::Nominal).count();

    let mut by_state: BTreeMap<GrammarState, usize> = BTreeMap::new();
    for step in steps {
        *by_state.entry(step.state).or_insert(0) += 1;
    }

    let ecc = if total_episodes > 0 {
        total_steps as f64 / total_episodes as f64
    } else {
        1.0
    };
    let edr = if total_steps > 0 {
        nominal_steps as f64 / total_steps as f64
    } else {
        1.0
    };

    EpisodeSummary {
        channel: channel.to_string(),
        total_steps,
        total_episodes,
        nominal_steps,
        non_nominal_episodes,
        episode_count_collapse: ecc,
        event_density_reduction: edr,
        by_state,
    }
}

/// Write an episode log to CSV.
///
/// Format: state,channel,start_ts,end_ts,step_count,peak_r,peak_delta,peak_sigma,reason
pub fn episodes_to_csv(episodes: &[Episode]) -> String {
    let mut out = String::from(
        "state,channel,start_ts,end_ts,step_count,peak_r,peak_delta,peak_sigma,drift_sign,reason\n"
    );
    for ep in episodes {
        out.push_str(&format!(
            "{},{},{:.4},{:.4},{},{:.4},{:.4},{:.4},{:.0},\"{}\"\n",
            ep.state,
            ep.channel,
            ep.start_ts,
            ep.end_ts,
            ep.step_count,
            ep.peak_r,
            ep.peak_delta,
            ep.peak_sigma,
            ep.drift_sign,
            ep.reason.as_str(),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AnnotatedStep, GrammarState, ReasonCode, ResidualTriple};

    fn make_step(ts: f64, state: GrammarState) -> AnnotatedStep {
        AnnotatedStep {
            triple: ResidualTriple { r: 0.0, delta: 0.0, sigma: 0.0, timestamp: ts },
            state,
            reason: ReasonCode::nominal(),
            channel: "test".to_string(),
        }
    }

    #[test]
    fn single_state_run_is_one_episode() {
        let steps: Vec<_> = (0..5).map(|i| make_step(i as f64, GrammarState::Nominal)).collect();
        let eps = aggregate_episodes(&steps);
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].step_count, 5);
    }

    #[test]
    fn state_change_creates_new_episode() {
        let mut steps: Vec<_> = (0..3).map(|i| make_step(i as f64, GrammarState::Nominal)).collect();
        steps.extend((3..6).map(|i| make_step(i as f64, GrammarState::DriftAccum)));
        let eps = aggregate_episodes(&steps);
        assert_eq!(eps.len(), 2);
        assert_eq!(eps[0].state, GrammarState::Nominal);
        assert_eq!(eps[1].state, GrammarState::DriftAccum);
    }

    #[test]
    fn ecc_gt_one_for_multi_step_episodes() {
        let steps: Vec<_> = (0..20).map(|i| make_step(i as f64, GrammarState::Nominal)).collect();
        let eps = aggregate_episodes(&steps);
        let summary = summarise("ch", &steps, &eps);
        assert!(summary.episode_count_collapse >= 1.0);
        assert_eq!(summary.event_density_reduction, 1.0);
    }
}
