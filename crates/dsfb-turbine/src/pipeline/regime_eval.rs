//! P2: Regime-conditioned engine evaluation for multi-condition datasets.
//!
//! FD002/FD004 have six operating conditions. Sensor values differ
//! substantially across regimes, so a single global baseline produces
//! false alarms. This module constructs per-regime baselines and
//! envelopes, then evaluates each cycle against the envelope matching
//! its operating regime.

use crate::core::config::DsfbConfig;
use crate::core::residual::{compute_drift, compute_slew, sign_at};
use crate::core::envelope::AdmissibilityEnvelope;
use crate::core::grammar::{GrammarEngine, GrammarState, aggregate_grammar, MAX_CHANNELS};
use crate::core::heuristics::{HeuristicsBank, EngineReasonCode};
use crate::core::regime::OperatingRegime;
use crate::core::episode::Episode;
use crate::core::audit::{AuditEntry, AuditTrail};
use crate::core::channels::ChannelId;
use crate::dataset::cmapss::CmapssDataset;
use crate::pipeline::engine_eval::EngineEvalResult;
use crate::pipeline::metrics::{compute_fleet_metrics, FleetMetrics};

/// Maximum number of regimes we track.
const MAX_REGIMES: usize = 8;

/// Per-regime baseline statistics for one channel.
#[derive(Debug, Clone, Copy)]
struct RegimeBaseline {
    regime_id: u8,
    mean: f64,
    std: f64,
    count: usize,
}

/// Evaluates one engine unit with regime-conditioned envelopes.
///
/// For each sensor channel:
/// 1. Group the first `healthy_window` cycles by regime
/// 2. Compute per-regime (mean, std)
/// 3. Construct per-regime envelopes
/// 4. At each cycle, evaluate the residual against the envelope for that cycle's regime
pub fn evaluate_engine_regime_conditioned(
    unit: u16,
    dataset: &CmapssDataset,
    channels: &[ChannelId],
    config: &DsfbConfig,
) -> EngineEvalResult {
    // Extract all rows for this unit, sorted by cycle
    let mut unit_rows: Vec<_> = dataset.rows.iter()
        .filter(|r| r.unit == unit)
        .collect();
    unit_rows.sort_by_key(|r| r.cycle);

    let total_cycles = unit_rows.len() as u32;
    if total_cycles == 0 {
        return empty_result(unit);
    }

    let bank = HeuristicsBank::default_gas_turbine();
    let num_channels = channels.len().min(MAX_CHANNELS);

    // ── Per-channel, per-regime baseline computation ────────────────
    // For each channel, compute baselines per regime from healthy window
    let hw = config.healthy_window.min(unit_rows.len());

    struct ChannelState {
        baselines: [RegimeBaseline; MAX_REGIMES],
        num_regimes: usize,
        envelopes: [AdmissibilityEnvelope; MAX_REGIMES],
        // Full-series computed values
        residuals: Vec<f64>,
        drifts: Vec<f64>,
        slews: Vec<f64>,
        grammar: GrammarEngine,
    }

    let mut ch_states: Vec<ChannelState> = Vec::with_capacity(num_channels);

    for &ch_id in channels.iter().take(num_channels) {
        let sensor_idx = ch_id.cmapss_sensor_index();

        // Collect per-regime samples from healthy window
        let mut regime_sums: [(f64, f64, usize, u8); MAX_REGIMES] = [(0.0, 0.0, 0, 0); MAX_REGIMES];
        let mut n_regimes = 0usize;

        for row in unit_rows.iter().take(hw) {
            let regime = OperatingRegime::from_cmapss_settings(
                row.op_settings[0], row.op_settings[1], row.op_settings[2]);
            let rid = match regime {
                OperatingRegime::MultiCondition { regime_id } => regime_id,
                _ => 0,
            };

            // Find or create regime slot
            let mut slot = None;
            for s in 0..n_regimes {
                if regime_sums[s].3 == rid {
                    slot = Some(s);
                    break;
                }
            }
            let s = match slot {
                Some(s) => s,
                None => {
                    if n_regimes >= MAX_REGIMES { continue; }
                    let s = n_regimes;
                    regime_sums[s].3 = rid;
                    n_regimes += 1;
                    s
                }
            };

            let val = row.sensors[sensor_idx];
            regime_sums[s].0 += val;
            regime_sums[s].2 += 1;
        }

        // Compute per-regime means
        for s in 0..n_regimes {
            if regime_sums[s].2 > 0 {
                regime_sums[s].0 /= regime_sums[s].2 as f64;
            }
        }

        // Second pass: compute per-regime variance
        for row in unit_rows.iter().take(hw) {
            let regime = OperatingRegime::from_cmapss_settings(
                row.op_settings[0], row.op_settings[1], row.op_settings[2]);
            let rid = match regime {
                OperatingRegime::MultiCondition { regime_id } => regime_id,
                _ => 0,
            };
            for s in 0..n_regimes {
                if regime_sums[s].3 == rid {
                    let d = row.sensors[sensor_idx] - regime_sums[s].0;
                    regime_sums[s].1 += d * d;
                    break;
                }
            }
        }

        // Build baselines and envelopes
        let mut baselines = [RegimeBaseline { regime_id: 0, mean: 0.0, std: 1.0, count: 0 }; MAX_REGIMES];
        let mut envelopes = [AdmissibilityEnvelope::from_baseline(0.0, 1.0, OperatingRegime::Unknown, config); MAX_REGIMES];

        for s in 0..n_regimes {
            let mean = regime_sums[s].0;
            let var = if regime_sums[s].2 > 1 {
                regime_sums[s].1 / (regime_sums[s].2 - 1) as f64
            } else { 1.0 };
            let std = sqrt_no_std(var).max(1e-10);

            baselines[s] = RegimeBaseline {
                regime_id: regime_sums[s].3,
                mean,
                std,
                count: regime_sums[s].2,
            };
            envelopes[s] = AdmissibilityEnvelope::from_baseline(
                baselines[s].mean,
                baselines[s].std,
                OperatingRegime::MultiCondition { regime_id: baselines[s].regime_id },
                config,
            );
        }

        // Compute regime-conditioned residuals for every cycle
        let n = unit_rows.len();
        let mut residuals = vec![0.0f64; n];
        for (k, row) in unit_rows.iter().enumerate() {
            let regime = OperatingRegime::from_cmapss_settings(
                row.op_settings[0], row.op_settings[1], row.op_settings[2]);
            let rid = match regime {
                OperatingRegime::MultiCondition { regime_id } => regime_id,
                _ => 0,
            };
            // Use exact regime baseline when available; otherwise fall back to
            // the most-supported healthy-window regime for this channel.
            let found_mean = find_regime_slot(&baselines, n_regimes, rid)
                .map_or(0.0, |idx| baselines[idx].mean);
            residuals[k] = row.sensors[sensor_idx] - found_mean;
        }

        // Drift and slew
        let mut drifts = vec![0.0f64; n];
        let mut slews = vec![0.0f64; n];
        compute_drift(&residuals, config.drift_window, &mut drifts);
        compute_slew(&drifts, config.slew_window, &mut slews);

        ch_states.push(ChannelState {
            baselines,
            num_regimes: n_regimes,
            envelopes,
            residuals,
            drifts,
            slews,
            grammar: GrammarEngine::new(),
        });
    }

    // ── Cycle-by-cycle grammar evaluation ───────────────────────────
    let n = unit_rows.len();
    let mut grammar_trajectory = Vec::with_capacity(n);
    let mut reason_trajectory = Vec::with_capacity(n);
    let mut primary_audit = AuditTrail::new();
    let mut agg_first_boundary: Option<u32> = None;
    let mut agg_first_violation: Option<u32> = None;
    let mut channel_boundary_cycles: Vec<(ChannelId, Option<u32>)> =
        channels.iter().take(num_channels).map(|&c| (c, None)).collect();

    for k in 0..n {
        let cycle = (k + 1) as u32;
        let row = &unit_rows[k];
        let regime = OperatingRegime::from_cmapss_settings(
            row.op_settings[0], row.op_settings[1], row.op_settings[2]);
        let rid = match regime {
            OperatingRegime::MultiCondition { regime_id } => regime_id,
            _ => 0,
        };

        let mut states_buf = [GrammarState::Admissible; MAX_CHANNELS];

        for (ci, cs) in ch_states.iter_mut().enumerate() {
            // Use exact regime envelope when available; otherwise fall back to
            // the regime most represented in the healthy window.
            let env_idx = find_regime_slot(&cs.baselines, cs.num_regimes, rid).unwrap_or(0);

            let sign = sign_at(&cs.residuals, &cs.drifts, &cs.slews, k, 1);
            cs.grammar.advance(&sign, &cs.envelopes[env_idx], config);
            states_buf[ci] = cs.grammar.state();

            if cs.grammar.first_boundary_cycle().is_some() && channel_boundary_cycles[ci].1.is_none() {
                channel_boundary_cycles[ci].1 = cs.grammar.first_boundary_cycle();
            }
        }

        let agg_state = aggregate_grammar(&states_buf[..num_channels], config.channel_vote_fraction);

        let reason = if !ch_states.is_empty() {
            let env_stressed = {
                let cs = &ch_states[0];
                let ei = find_regime_slot(&cs.baselines, cs.num_regimes, rid).unwrap_or(0);
                cs.envelopes[ei].classify_position(cs.residuals[k])
                    != crate::core::envelope::EnvelopeStatus::Interior
            };
            bank.match_motif(ch_states[0].drifts[k], ch_states[0].slews[k], agg_state, env_stressed)
        } else {
            EngineReasonCode::NoAnomaly
        };

        if agg_state.severity() >= GrammarState::Boundary.severity() && agg_first_boundary.is_none() {
            agg_first_boundary = Some(cycle);
        }
        if agg_state == GrammarState::Violation && agg_first_violation.is_none() {
            agg_first_violation = Some(cycle);
        }

        if !ch_states.is_empty() {
            let cs = &ch_states[0];
            let ei = find_regime_slot(&cs.baselines, cs.num_regimes, rid).unwrap_or(0);
            primary_audit.push(AuditEntry {
                cycle,
                residual: cs.residuals[k],
                drift: cs.drifts[k],
                slew: cs.slews[k],
                envelope_position: cs.envelopes[ei].normalized_position(cs.residuals[k]),
                envelope_status: cs.envelopes[ei].classify_position(cs.residuals[k]),
                grammar_state: agg_state,
                reason_code: reason,
                drift_persistence: 0,
                slew_persistence: 0,
            });
        }

        grammar_trajectory.push(agg_state);
        reason_trajectory.push(reason);
    }

    let episodes = form_episodes_simple(unit, &grammar_trajectory, &reason_trajectory,
        if !ch_states.is_empty() { &ch_states[0].drifts } else { &[] },
        if !ch_states.is_empty() { &ch_states[0].slews } else { &[] });

    let structural_lead_time = agg_first_boundary.map(|fb| total_cycles.saturating_sub(fb));

    EngineEvalResult {
        unit,
        total_cycles,
        grammar_trajectory,
        reason_trajectory,
        first_boundary_cycle: agg_first_boundary,
        first_violation_cycle: agg_first_violation,
        structural_lead_time,
        episodes,
        theorem_bound: None,
        channel_boundary_cycles,
        primary_audit,
    }
}

/// Evaluates an entire multi-condition fleet with regime-conditioned envelopes.
pub fn evaluate_fleet_regime_conditioned(
    dataset: &CmapssDataset,
    channels: &[ChannelId],
    config: &DsfbConfig,
) -> (Vec<EngineEvalResult>, FleetMetrics) {
    let units = dataset.units();
    let mut results = Vec::with_capacity(units.len());
    for &unit in &units {
        let result = evaluate_engine_regime_conditioned(unit, dataset, channels, config);
        results.push(result);
    }
    let metrics = compute_fleet_metrics(&results);
    (results, metrics)
}

fn empty_result(unit: u16) -> EngineEvalResult {
    EngineEvalResult {
        unit, total_cycles: 0,
        grammar_trajectory: Vec::new(), reason_trajectory: Vec::new(),
        first_boundary_cycle: None, first_violation_cycle: None,
        structural_lead_time: None, episodes: Vec::new(),
        theorem_bound: None, channel_boundary_cycles: Vec::new(),
        primary_audit: AuditTrail::new(),
    }
}

fn find_regime_slot(
    baselines: &[RegimeBaseline; MAX_REGIMES],
    num_regimes: usize,
    regime_id: u8,
) -> Option<usize> {
    let mut fallback = None;
    let mut max_count = 0usize;

    for (idx, baseline) in baselines.iter().take(num_regimes).enumerate() {
        if baseline.regime_id == regime_id {
            return Some(idx);
        }
        if baseline.count > max_count {
            max_count = baseline.count;
            fallback = Some(idx);
        }
    }

    fallback
}

fn form_episodes_simple(unit: u16, grammar: &[GrammarState], reasons: &[EngineReasonCode], drifts: &[f64], slews: &[f64]) -> Vec<Episode> {
    let mut episodes = Vec::new();
    let n = grammar.len();
    let mut i = 0;
    while i < n {
        if grammar[i].severity() >= GrammarState::Boundary.severity() {
            let start = i;
            let mut peak = grammar[i];
            let mut peak_reason = if i < reasons.len() { reasons[i] } else { EngineReasonCode::NoAnomaly };
            let mut md = 0.0f64;
            let mut ms = 0.0f64;
            while i < n && grammar[i].severity() >= GrammarState::Boundary.severity() {
                if grammar[i].severity() > peak.severity() { peak = grammar[i]; }
                if i < reasons.len() && reasons[i].is_anomalous() { peak_reason = reasons[i]; }
                if i < drifts.len() && drifts[i].abs() > md { md = drifts[i].abs(); }
                if i < slews.len() && slews[i].abs() > ms { ms = slews[i].abs(); }
                i += 1;
            }
            episodes.push(Episode { unit, start_cycle: (start+1) as u32, end_cycle: i as u32, peak_state: peak, reason_code: peak_reason, max_drift: md, max_slew: ms, duration_cycles: (i-start) as u32, trigger_channel: 0 });
        } else { i += 1; }
    }
    episodes
}

fn sqrt_no_std(x: f64) -> f64 {
    if x <= 0.0 { return 0.0; }
    let mut g = x;
    for _ in 0..50 {
        let n = 0.5 * (g + x / g);
        if (n - g).abs() < 1e-15 { return n; }
        g = n;
    }
    g
}
