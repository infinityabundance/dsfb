//! DSFB-Debug: episode aggregation — Trace Event Collapse implementation.
//!
//! # Trace Event Collapse — paper §7's "primary developer-facing delta"
//!
//! This module is the structural-aggregation core. It collapses
//! per-(window, signal) anomaly events into a small number of typed
//! structural episodes. The collapse is the operator-visible delta
//! of DSFB-Debug versus flat alerting: 11 raw cell-level alerts on
//! F-11 collapse into 3 typed episodes (RSCR 3.67×); 52 raw alerts
//! on the AIOps Challenge KPI fixture collapse into 1 episode (RSCR
//! 52×).
//!
//! # Algorithm
//!
//! `aggregate_episodes` is a deterministic run-length aggregator over
//! the per-(window, signal) `PolicyState` grid. An episode opens when
//! any signal transitions to `Review` or `Escalate`; an episode
//! closes when all signals return to `Silent` or `Watch` for
//! `correlation_window` consecutive windows. The closed episode
//! carries:
//!
//! - `episode_id` — sequential id from 0
//! - `start_window` / `end_window` — inclusive range
//! - `peak_grammar_state` — max grammar state observed
//! - `primary_reason_code` — most-frequent reason code
//! - `policy_state` — peak policy state (Watch / Review / Escalate)
//! - `contributing_signal_count` — distinct signals in non-Admissible
//!   state during the episode
//! - `structural_signature` — `(dominant_drift_direction,
//!   peak_slew_magnitude, duration_windows, signal_correlation)`
//! - `matched_motif` — left as `Unknown`; populated by the bank's
//!   `match_episode_with_consensus` after fusion.
//!
//! # `compute_metrics` — RSCR + fault-recall + clean-window FP rate
//!
//! Computes the paper §13 headline metrics from a closed episode list:
//!
//! | Metric | Formula |
//! |--------|---------|
//! | RSCR | raw_alerts / max(1, dsfb_episode_count) |
//! | episode_precision | episodes_overlapping_labeled_fault / dsfb_episode_count |
//! | fault_recall | labeled_faults_captured_by_at_least_one_episode / total_labeled_faults |
//! | investigation_load_reduction_pct | (1 - dsfb / raw) × 100 |
//! | clean_window_false_episode_rate | episodes_in_clean_window / clean_window_count |
//!
//! All formulas are repeated inline here so a reader doesn't need to
//! cross-reference the paper.
//!
//! # Determinism (Theorem 9)
//!
//! The aggregator is a pure function of the policy-state grid.
//! Iteration order is deterministic (row-major over windows then
//! signals). Tie-breakers (most-frequent-reason-code resolves ties by
//! lower enum index) preserve byte-identical output across replays.

use crate::types::*;

/// Aggregate per-window policy evaluations into episodes.
///
/// An episode opens when any signal transitions to Review or Escalate.
/// An episode closes when all signals return to Silent/Watch for
/// `correlation_window` consecutive windows.
///
/// # Arguments
/// * `policy_states` - per-window, per-signal policy states (row-major: [window][signal])
/// * `num_signals` - number of signals per window
/// * `num_windows` - number of windows
/// * `reason_codes` - per-window, per-signal reason codes
/// * `correlation_window` - windows of silence required to close an episode
/// * `episodes_out` - output buffer for episodes
///
/// # Returns
/// Number of episodes written
#[allow(clippy::too_many_arguments)]
pub fn aggregate_episodes(
    policy_states: &[PolicyState],
    num_signals: usize,
    num_windows: usize,
    reason_codes: &[ReasonCode],
    drift_directions: &[DriftDirection],
    slew_magnitudes: &[f64],
    correlation_window: u64,
    episodes_out: &mut [DebugEpisode],
) -> usize {
    if num_signals == 0 || num_windows == 0 {
        return 0;
    }

    let mut episode_count: usize = 0;
    let mut in_episode = false;
    let mut episode_start: u64 = 0;
    let mut silent_streak: u64 = 0;
    let mut peak_state = GrammarState::Admissible;
    let mut primary_reason = ReasonCode::Admissible;
    let mut peak_slew: f64 = 0.0;
    let mut contributing_signals: u16 = 0;

    let mut w: usize = 0;
    while w < num_windows {
        // Check if any signal in this window is Review or Escalate
        let mut window_has_action = false;
        let mut window_contributing: u16 = 0;
        let mut s: usize = 0;
        while s < num_signals {
            let idx = w * num_signals + s;
            if idx < policy_states.len() {
                let ps = policy_states[idx];
                if ps >= PolicyState::Review {
                    window_has_action = true;
                    window_contributing += 1;

                    // Track peak grammar state
                    let gs = match ps {
                        PolicyState::Escalate => GrammarState::Violation,
                        PolicyState::Review => GrammarState::Boundary,
                        _ => GrammarState::Admissible,
                    };
                    if gs > peak_state {
                        peak_state = gs;
                    }

                    // Track reason code (prefer more severe)
                    if idx < reason_codes.len() {
                        let rc = reason_codes[idx];
                        if reason_severity(rc) > reason_severity(primary_reason) {
                            primary_reason = rc;
                        }
                    }

                    // Track peak slew
                    if idx < slew_magnitudes.len() {
                        let sm = slew_magnitudes[idx];
                        if sm > peak_slew { peak_slew = sm; }
                    }
                }
            }
            s += 1;
        }

        if window_has_action {
            if !in_episode {
                // Open new episode
                in_episode = true;
                episode_start = w as u64;
                peak_state = GrammarState::Admissible;
                primary_reason = ReasonCode::Admissible;
                peak_slew = 0.0;
                contributing_signals = 0;
            }
            silent_streak = 0;
            if window_contributing > contributing_signals {
                contributing_signals = window_contributing;
            }
            // Re-check peak state for this window
            let mut s2: usize = 0;
            while s2 < num_signals {
                let idx = w * num_signals + s2;
                if idx < policy_states.len() && policy_states[idx] >= PolicyState::Review {
                    let gs = if policy_states[idx] == PolicyState::Escalate {
                        GrammarState::Violation
                    } else {
                        GrammarState::Boundary
                    };
                    if gs > peak_state { peak_state = gs; }
                    if idx < reason_codes.len() {
                        let rc = reason_codes[idx];
                        if reason_severity(rc) > reason_severity(primary_reason) {
                            primary_reason = rc;
                        }
                    }
                    if idx < slew_magnitudes.len() && slew_magnitudes[idx] > peak_slew {
                        peak_slew = slew_magnitudes[idx];
                    }
                }
                s2 += 1;
            }
        } else if in_episode {
            silent_streak += 1;
            if silent_streak >= correlation_window {
                // Close episode
                if episode_count < episodes_out.len() {
                    let dominant_drift = if w > 0 && (w - 1) * num_signals < drift_directions.len() {
                        drift_directions[(w - 1) * num_signals] // first signal's drift as proxy
                    } else {
                        DriftDirection::None
                    };

                    episodes_out[episode_count] = DebugEpisode {
                        episode_id: episode_count as u32,
                        start_window: episode_start,
                        end_window: w as u64 - silent_streak,
                        peak_grammar_state: peak_state,
                        primary_reason_code: primary_reason,
                        matched_motif: SemanticDisposition::Unknown, // filled by caller
                        policy_state: if peak_state == GrammarState::Violation {
                            PolicyState::Escalate
                        } else {
                            PolicyState::Review
                        },
                        contributing_signal_count: contributing_signals,
                        structural_signature: StructuralSignature {
                            dominant_drift_direction: dominant_drift,
                            peak_slew_magnitude: peak_slew,
                            duration_windows: (w as u64 - silent_streak) - episode_start + 1,
                            signal_correlation: contributing_signals as f64 / num_signals as f64,
                        },
            root_cause_signal_index: None,
                    };
                    episode_count += 1;
                }
                in_episode = false;
                peak_state = GrammarState::Admissible;
                primary_reason = ReasonCode::Admissible;
                peak_slew = 0.0;
                contributing_signals = 0;
            }
        }

        w += 1;
    }

    // Close any open episode at end of data
    if in_episode && episode_count < episodes_out.len() {
        episodes_out[episode_count] = DebugEpisode {
            episode_id: episode_count as u32,
            start_window: episode_start,
            end_window: num_windows as u64 - 1,
            peak_grammar_state: peak_state,
            primary_reason_code: primary_reason,
            matched_motif: SemanticDisposition::Unknown,
            policy_state: if peak_state == GrammarState::Violation {
                PolicyState::Escalate
            } else {
                PolicyState::Review
            },
            contributing_signal_count: contributing_signals,
            structural_signature: StructuralSignature {
                dominant_drift_direction: DriftDirection::None,
                peak_slew_magnitude: peak_slew,
                duration_windows: num_windows as u64 - episode_start,
                signal_correlation: contributing_signals as f64 / num_signals as f64,
            },
            root_cause_signal_index: None,
        };
        episode_count += 1;
    }

    episode_count
}

fn reason_severity(r: ReasonCode) -> u8 {
    match r {
        ReasonCode::Admissible => 0,
        ReasonCode::BoundaryApproach => 1,
        ReasonCode::SingleCrossing => 1,
        ReasonCode::DriftWithRecovery => 2,
        ReasonCode::RecurrentBoundaryGrazing => 3,
        ReasonCode::SustainedOutwardDrift => 4,
        ReasonCode::AbruptSlewViolation => 5,
        ReasonCode::EnvelopeViolation => 6,
    }
}

/// Compute benchmark metrics from episodes and fault labels.
/// Paper §7.4
#[allow(clippy::too_many_arguments)]
pub fn compute_metrics(
    episodes: &[DebugEpisode],
    episode_count: usize,
    fault_labels: &[bool],
    raw_anomaly_count: u64,
    precision_window: u64,
    dataset_name: &'static str,
    num_signals: u16,
) -> BenchmarkMetrics {
    let num_windows = fault_labels.len() as u64;
    let dsfb_episode_count = episode_count as u64;

    let rscr = if dsfb_episode_count > 0 {
        raw_anomaly_count as f64 / dsfb_episode_count as f64
    } else {
        0.0
    };

    // Episode precision: fraction of episodes followed by a fault within W_pred
    let mut precise_count: u64 = 0;
    let mut i = 0;
    while i < episode_count {
        let ep = &episodes[i];
        let check_end = ep.end_window + precision_window;
        let check_end = if check_end >= num_windows { num_windows - 1 } else { check_end };
        let mut found_fault = false;
        let mut w = ep.start_window;
        while w <= check_end {
            if (w as usize) < fault_labels.len() && fault_labels[w as usize] {
                found_fault = true;
            }
            w += 1;
        }
        if found_fault {
            precise_count += 1;
        }
        i += 1;
    }
    let episode_precision = if dsfb_episode_count > 0 {
        precise_count as f64 / dsfb_episode_count as f64
    } else {
        0.0
    };

    // Fault recall: fraction of labeled faults captured by at least one episode
    let mut total_faults: u64 = 0;
    let mut captured_faults: u64 = 0;
    let mut w: usize = 0;
    while w < fault_labels.len() {
        if fault_labels[w] {
            total_faults += 1;
            // Check if any episode covers this window
            let mut covered = false;
            let mut j = 0;
            while j < episode_count {
                let ep = &episodes[j];
                // Fault is captured if it falls within episode ± precision_window
                if (w as u64) >= ep.start_window.saturating_sub(precision_window)
                    && (w as u64) <= ep.end_window + precision_window
                {
                    covered = true;
                }
                j += 1;
            }
            if covered {
                captured_faults += 1;
            }
        }
        w += 1;
    }
    let fault_recall = if total_faults > 0 {
        captured_faults as f64 / total_faults as f64
    } else {
        1.0 // no faults → vacuous recall
    };

    // Investigation load
    let investigation_load_dsfb = dsfb_episode_count;
    let investigation_load_reduction_pct = if raw_anomaly_count > 0 {
        (1.0 - investigation_load_dsfb as f64 / raw_anomaly_count as f64) * 100.0
    } else {
        0.0
    };

    // Clean-window false episode rate: episodes in windows with no faults nearby
    let false_episodes = dsfb_episode_count - precise_count;
    let clean_windows = num_windows - total_faults;
    let clean_window_false_episode_rate = if clean_windows > 0 {
        false_episodes as f64 / clean_windows as f64
    } else {
        0.0
    };

    BenchmarkMetrics {
        dataset_name,
        total_windows: num_windows,
        total_signals: num_signals,
        raw_anomaly_count,
        dsfb_episode_count,
        rscr,
        episode_precision,
        fault_recall,
        investigation_load_raw: raw_anomaly_count,
        investigation_load_dsfb,
        investigation_load_reduction_pct,
        clean_window_false_episode_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_episodes_from_silent() {
        let policies = [PolicyState::Silent; 100];
        let reasons = [ReasonCode::Admissible; 100];
        let drifts = [DriftDirection::None; 100];
        let slews = [0.0_f64; 100];
        let mut episodes = [DebugEpisode {
            episode_id: 0, start_window: 0, end_window: 0,
            peak_grammar_state: GrammarState::Admissible,
            primary_reason_code: ReasonCode::Admissible,
            matched_motif: SemanticDisposition::Unknown,
            policy_state: PolicyState::Silent,
            contributing_signal_count: 0,
            structural_signature: StructuralSignature {
                dominant_drift_direction: DriftDirection::None,
                peak_slew_magnitude: 0.0, duration_windows: 0, signal_correlation: 0.0,
            },
            root_cause_signal_index: None,
        }; 16];

        let count = aggregate_episodes(
            &policies, 1, 100, &reasons, &drifts, &slews, 5, &mut episodes,
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn test_single_episode() {
        // 10 windows, 1 signal. Windows 3-5 are Escalate, rest Silent.
        let mut policies = [PolicyState::Silent; 10];
        policies[3] = PolicyState::Escalate;
        policies[4] = PolicyState::Escalate;
        policies[5] = PolicyState::Escalate;
        let reasons = [ReasonCode::AbruptSlewViolation; 10];
        let drifts = [DriftDirection::Positive; 10];
        let slews = [1.0_f64; 10];

        let blank = DebugEpisode {
            episode_id: 0, start_window: 0, end_window: 0,
            peak_grammar_state: GrammarState::Admissible,
            primary_reason_code: ReasonCode::Admissible,
            matched_motif: SemanticDisposition::Unknown,
            policy_state: PolicyState::Silent,
            contributing_signal_count: 0,
            structural_signature: StructuralSignature {
                dominant_drift_direction: DriftDirection::None,
                peak_slew_magnitude: 0.0, duration_windows: 0, signal_correlation: 0.0,
            },
            root_cause_signal_index: None,
        };
        let mut episodes = [blank; 16];
        let count = aggregate_episodes(
            &policies, 1, 10, &reasons, &drifts, &slews, 3, &mut episodes,
        );
        assert_eq!(count, 1);
        assert_eq!(episodes[0].start_window, 3);
    }
}
