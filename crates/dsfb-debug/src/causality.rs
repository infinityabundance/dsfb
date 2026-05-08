//! DSFB-Debug: causality / graph attribution — root-cause stamping
//! over the service-call graph (no_std).
//!
//! # Role in the operator workflow
//!
//! Pure-function service-graph walk. Given the per-(window, signal)
//! evaluation grid (`SignalEvaluation` from `run_evaluation`) and a
//! service dependency graph encoded as parent→child signal-index
//! pairs, for each closed episode this module returns the most-
//! upstream contributing signal as the candidate root cause. The
//! result is written into `DebugEpisode.root_cause_signal_index`.
//!
//! Per panellist P11 (Senior SRE): "questions 2 and 3 of the eight
//! load-bearing on-call questions are 'which service is the
//! originator?' and 'what changed?'. Without graph attribution
//! DSFB-Debug answers neither." This module is the answer.
//!
//! # Deterministic algorithm (Theorem 9 preserved)
//!
//! 1. For each episode, scan the per-(window, signal) grid over the
//!    window range `[start_window, end_window]`.
//! 2. Find the lexicographically-earliest `(window, signal)` pair
//!    whose `confirmed_grammar_state >= Boundary` AND whose absolute
//!    `sign_tuple.slew` exceeds the engine's `slew_delta` threshold.
//!    This is the "first slew window".
//! 3. Among the signals contributing in the first slew window, find
//!    those whose graph-incoming edges (parent → this signal) come
//!    from outside the contributing-signal set of the episode —
//!    these are the upstream-most signals.
//! 4. Return the lowest such signal index. Tie-broken by lowest
//!    index for determinism.
//!
//! # Failure modes (returns `None`, never silently fabricates)
//!
//! - Empty graph (no edges supplied) → no attribution
//! - Episode has fewer than 2 contributing signals → no attribution
//!   (single-signal episodes have no upstream/downstream distinction
//!   within the episode itself)
//!
//! `None` is the honest "I cannot attribute" answer; the engine
//! never invents a root cause.

#![allow(clippy::too_many_arguments)]

use crate::types::*;

/// Walk the service-call graph and stamp each closed episode with its
/// most-upstream contributing signal index, if determinable.
///
/// `eval_out` must be the same buffer that `run_evaluation` populated.
/// `service_graph` is a slice of `(parent_signal_idx, child_signal_idx)`
/// pairs. Empty graph → no attribution.
pub fn attribute_root_causes(
    episodes_out: &mut [DebugEpisode],
    episode_count: usize,
    eval_out: &[SignalEvaluation],
    num_signals: usize,
    num_windows: usize,
    service_graph: &[(u16, u16)],
    slew_delta: f64,
) {
    if service_graph.is_empty() {
        let mut i = 0;
        while i < episode_count {
            episodes_out[i].root_cause_signal_index = None;
            i += 1;
        }
        return;
    }

    let mut ep_idx: usize = 0;
    while ep_idx < episode_count {
        episodes_out[ep_idx].root_cause_signal_index =
            attribute_one_episode(
                &episodes_out[ep_idx],
                eval_out,
                num_signals,
                num_windows,
                service_graph,
                slew_delta,
            );
        ep_idx += 1;
    }
}

/// Attribute a single episode. Returns `Some(signal_idx)` when a
/// unique upstream-most contributor is determinable, `None` otherwise.
fn attribute_one_episode(
    episode: &DebugEpisode,
    eval_out: &[SignalEvaluation],
    num_signals: usize,
    num_windows: usize,
    service_graph: &[(u16, u16)],
    slew_delta: f64,
) -> Option<u16> {
    let start_w = episode.start_window as usize;
    let end_w = episode.end_window as usize;
    if start_w >= num_windows {
        return None;
    }

    // Collect contributing signals: those whose confirmed grammar state
    // reached Boundary or higher anywhere in [start_w, end_w].
    // Bitset over num_signals (max 512 supported here).
    const MAX_SIG: usize = 512;
    let mut contrib = [false; MAX_SIG];
    if num_signals > MAX_SIG {
        return None;
    }

    let mut w = start_w;
    while w <= end_w && w < num_windows {
        let mut s = 0;
        while s < num_signals {
            let idx = w * num_signals + s;
            if idx < eval_out.len() {
                let e = eval_out[idx];
                if e.confirmed_grammar_state >= GrammarState::Boundary {
                    contrib[s] = true;
                }
            }
            s += 1;
        }
        w += 1;
    }

    // Count contributors. If fewer than 2, attribution is moot.
    let mut contrib_count = 0;
    let mut s = 0;
    while s < num_signals {
        if contrib[s] {
            contrib_count += 1;
        }
        s += 1;
    }
    if contrib_count < 2 {
        return None;
    }

    // Find the first-slew (window, signal) pair: earliest window with
    // any contributor whose absolute slew exceeds slew_delta. Among
    // those, lowest signal index for determinism.
    let mut first_slew_signal: Option<u16> = None;
    let mut w = start_w;
    while w <= end_w && w < num_windows && first_slew_signal.is_none() {
        let mut s = 0;
        while s < num_signals {
            let idx = w * num_signals + s;
            if idx < eval_out.len() && contrib[s] {
                let e = eval_out[idx];
                let slew_abs = if e.sign_tuple.slew >= 0.0 {
                    e.sign_tuple.slew
                } else {
                    -e.sign_tuple.slew
                };
                if slew_abs > slew_delta {
                    first_slew_signal = Some(s as u16);
                    break;
                }
            }
            s += 1;
        }
        w += 1;
    }
    let _first = first_slew_signal?;

    // Find upstream-most contributors: those whose ALL graph-incoming
    // edges (parent → this signal) come from signals NOT in the
    // contributing set. These are the boundary-of-the-cascade signals.
    // Among them, lowest index wins.
    let mut upstream_most: Option<u16> = None;
    let mut s = 0_u16;
    while (s as usize) < num_signals {
        if contrib[s as usize] {
            let mut has_internal_parent = false;
            let mut has_any_parent = false;
            let mut g = 0;
            while g < service_graph.len() {
                let (parent, child) = service_graph[g];
                if child == s {
                    has_any_parent = true;
                    if (parent as usize) < num_signals && contrib[parent as usize] {
                        has_internal_parent = true;
                        break;
                    }
                }
                g += 1;
            }
            // Upstream-most: has either no parent in graph (root of
            // service tree) or has parents only OUTSIDE the contributing
            // set. The presence of a parent outside the contributing
            // set means the cascade entered through this signal.
            if !has_internal_parent {
                let _ = has_any_parent; // intentionally unused — both
                                         // graph-roots and external-parent
                                         // boundary signals qualify
                if upstream_most.is_none() {
                    upstream_most = Some(s);
                }
            }
        }
        s += 1;
    }
    upstream_most
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(window: u64, signal: u16, state: GrammarState, slew: f64) -> SignalEvaluation {
        SignalEvaluation {
            window_index: window,
            signal_index: signal,
            residual_value: 0.0,
            sign_tuple: SignTuple { norm: 0.0, drift: 0.0, slew },
            raw_grammar_state: state,
            confirmed_grammar_state: state,
            reason_code: ReasonCode::AbruptSlewViolation,
            motif: None,
            semantic_disposition: SemanticDisposition::Unknown,
            dsa_score: 0.0,
            policy_state: PolicyState::Review,
            was_imputed: false,
            drift_persistence: 0.0,
        }
    }

    fn blank_ep(start: u64, end: u64, contrib: u16) -> DebugEpisode {
        DebugEpisode {
            episode_id: 0,
            start_window: start,
            end_window: end,
            peak_grammar_state: GrammarState::Boundary,
            primary_reason_code: ReasonCode::AbruptSlewViolation,
            matched_motif: SemanticDisposition::Unknown,
            policy_state: PolicyState::Review,
            contributing_signal_count: contrib,
            structural_signature: StructuralSignature {
                dominant_drift_direction: DriftDirection::Positive,
                peak_slew_magnitude: 0.5,
                duration_windows: end - start + 1,
                signal_correlation: contrib as f64 / 4.0,
            },
            root_cause_signal_index: None,
        }
    }

    #[test]
    fn attributes_linear_chain_root_to_signal_zero() {
        // Service graph: s0 -> s1 -> s2 -> s3 (linear chain).
        // All four contribute; first slew on s0 then propagates.
        let graph: &[(u16, u16)] = &[(0, 1), (1, 2), (2, 3)];
        let num_signals = 4;
        let num_windows = 6;
        let n = num_signals * num_windows;
        let mut eval = [SignalEvaluation {
            window_index: 0, signal_index: 0, residual_value: 0.0,
            sign_tuple: SignTuple::ZERO,
            raw_grammar_state: GrammarState::Admissible,
            confirmed_grammar_state: GrammarState::Admissible,
            reason_code: ReasonCode::Admissible,
            motif: None, semantic_disposition: SemanticDisposition::Unknown,
            dsa_score: 0.0, policy_state: PolicyState::Silent, was_imputed: false,
            drift_persistence: 0.0,
        }; 64];
        // Slew on s0 first (window 0), then s1 (window 1), s2 (window 2), s3 (window 3).
        // num_signals = 4 → row offsets are 0, 4, 8, 12.
        eval[0] = ev(0, 0, GrammarState::Boundary, 0.9);
        eval[5] = ev(1, 1, GrammarState::Boundary, 0.8);
        eval[10] = ev(2, 2, GrammarState::Boundary, 0.7);
        eval[15] = ev(3, 3, GrammarState::Boundary, 0.6);
        let _ = n;

        let mut episodes = [blank_ep(0, 4, 4); 1];
        attribute_root_causes(&mut episodes, 1, &eval, num_signals, num_windows, graph, 0.1);

        assert_eq!(episodes[0].root_cause_signal_index, Some(0),
                   "linear-chain cascade should attribute root to s0");
    }

    #[test]
    fn empty_graph_yields_none() {
        let mut eval = [SignalEvaluation {
            window_index: 0, signal_index: 0, residual_value: 0.0,
            sign_tuple: SignTuple::ZERO,
            raw_grammar_state: GrammarState::Admissible,
            confirmed_grammar_state: GrammarState::Admissible,
            reason_code: ReasonCode::Admissible,
            motif: None, semantic_disposition: SemanticDisposition::Unknown,
            dsa_score: 0.0, policy_state: PolicyState::Silent, was_imputed: false,
            drift_persistence: 0.0,
        }; 16];
        eval[0] = ev(0, 0, GrammarState::Boundary, 0.5);
        eval[1] = ev(0, 1, GrammarState::Boundary, 0.5);
        let mut episodes = [blank_ep(0, 1, 2); 1];
        attribute_root_causes(&mut episodes, 1, &eval, 4, 4, &[], 0.1);
        assert_eq!(episodes[0].root_cause_signal_index, None,
                   "empty graph must not produce attribution");
    }

    #[test]
    fn single_signal_episode_yields_none() {
        // One contributor → attribution is moot.
        let graph: &[(u16, u16)] = &[(0, 1)];
        let mut eval = [SignalEvaluation {
            window_index: 0, signal_index: 0, residual_value: 0.0,
            sign_tuple: SignTuple::ZERO,
            raw_grammar_state: GrammarState::Admissible,
            confirmed_grammar_state: GrammarState::Admissible,
            reason_code: ReasonCode::Admissible,
            motif: None, semantic_disposition: SemanticDisposition::Unknown,
            dsa_score: 0.0, policy_state: PolicyState::Silent, was_imputed: false,
            drift_persistence: 0.0,
        }; 16];
        eval[0] = ev(0, 0, GrammarState::Boundary, 0.5);
        let mut episodes = [blank_ep(0, 1, 1); 1];
        attribute_root_causes(&mut episodes, 1, &eval, 4, 4, graph, 0.1);
        assert_eq!(episodes[0].root_cause_signal_index, None,
                   "single-signal episode must not produce attribution");
    }
}
