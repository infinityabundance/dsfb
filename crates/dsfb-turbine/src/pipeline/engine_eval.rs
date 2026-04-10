//! Single-engine DSFB evaluation pipeline.
//!
//! Runs the complete DSFB pipeline on one engine unit across all
//! informative channels, producing grammar-state trajectories,
//! episodes, reason codes, and audit traces.

use crate::core::config::DsfbConfig;
use crate::core::residual::{compute_baseline, compute_residuals, compute_drift, compute_slew, sign_at};
use crate::core::envelope::AdmissibilityEnvelope;
use crate::core::grammar::{GrammarEngine, GrammarState, aggregate_grammar, MAX_CHANNELS};
use crate::core::heuristics::{HeuristicsBank, EngineReasonCode};
use crate::core::regime::OperatingRegime;
use crate::core::episode::Episode;
use crate::core::audit::{AuditEntry, AuditTrail};
use crate::core::theorem::TheoremOneBound;
use crate::core::channels::ChannelId;

/// Expected upper bound for C-MAPSS engine-length capacity planning.
///
/// The evaluator remains dynamically sized; this constant is used only as a
/// capacity hint for the common dataset range rather than as a truncation bound.
const MAX_CYCLES: usize = 512;

/// Result of evaluating one engine unit.
#[derive(Debug)]
pub struct EngineEvalResult {
    /// Engine unit number.
    pub unit: u16,
    /// Total lifetime (cycles).
    pub total_cycles: u32,
    /// Per-cycle aggregate grammar state.
    pub grammar_trajectory: Vec<GrammarState>,
    /// Per-cycle aggregate reason code.
    pub reason_trajectory: Vec<EngineReasonCode>,
    /// First Boundary cycle (aggregate).
    pub first_boundary_cycle: Option<u32>,
    /// First Violation cycle (aggregate).
    pub first_violation_cycle: Option<u32>,
    /// Structural lead time: cycles between first Boundary and end-of-life.
    pub structural_lead_time: Option<u32>,
    /// Episodes (contiguous non-Admissible intervals).
    pub episodes: Vec<Episode>,
    /// Theorem 1 bound (computed from the most diagnostic channel).
    pub theorem_bound: Option<TheoremOneBound>,
    /// Per-channel first Boundary cycles.
    pub channel_boundary_cycles: Vec<(ChannelId, Option<u32>)>,
    /// Audit trail for the primary diagnostic channel.
    pub primary_audit: AuditTrail,
}

/// Evaluates one engine unit through the complete DSFB pipeline.
///
/// # Arguments
/// - `unit`: engine unit number
/// - `channel_data`: Vec of (ChannelId, Vec<f64>) — sensor values per channel
/// - `config`: DSFB configuration
///
/// # Returns
/// Complete evaluation result with grammar trajectory, episodes, and audit.
pub fn evaluate_engine(
    unit: u16,
    channel_data: &[(ChannelId, Vec<f64>)],
    config: &DsfbConfig,
) -> EngineEvalResult {
    let total_cycles = channel_data.iter()
        .map(|(_, vals)| vals.len())
        .max()
        .unwrap_or(0) as u32;

    let bank = HeuristicsBank::default_gas_turbine();
    let regime = OperatingRegime::SeaLevelStatic; // FD001 single condition

    // ── Per-channel processing ──────────────────────────────────────
    let num_channels = channel_data.len().min(MAX_CHANNELS);
    let mut channel_engines: Vec<GrammarEngine> = vec![GrammarEngine::new(); num_channels];
    let mut channel_envelopes: Vec<AdmissibilityEnvelope> = Vec::with_capacity(num_channels);
    let mut channel_residuals: Vec<Vec<f64>> = Vec::with_capacity(num_channels);
    let mut channel_drifts: Vec<Vec<f64>> = Vec::with_capacity(num_channels);
    let mut channel_slews: Vec<Vec<f64>> = Vec::with_capacity(num_channels);
    let mut channel_boundary_cycles: Vec<(ChannelId, Option<u32>)> = Vec::new();

    for (ch_id, values) in channel_data.iter().take(num_channels) {
        let n = values.len();

        // Baseline from healthy window
        let (mean, std) = compute_baseline(values, config);
        let envelope = AdmissibilityEnvelope::from_baseline(mean, std, regime, config);
        channel_envelopes.push(envelope);

        // Residuals
        let mut resid = vec![0.0; n];
        compute_residuals(values, mean, &mut resid);
        
        // Drift
        let mut drift = vec![0.0; n];
        compute_drift(&resid, config.drift_window, &mut drift);
        
        // Slew
        let mut slew = vec![0.0; n];
        compute_slew(&drift, config.slew_window, &mut slew);

        channel_residuals.push(resid);
        channel_drifts.push(drift);
        channel_slews.push(slew);
        channel_boundary_cycles.push((*ch_id, None));
    }

    // ── Cycle-by-cycle grammar evaluation ───────────────────────────
    let n_cycles = total_cycles as usize;
    let capacity_hint = n_cycles.min(MAX_CYCLES);
    let mut grammar_trajectory = Vec::with_capacity(capacity_hint);
    let mut reason_trajectory = Vec::with_capacity(capacity_hint);
    let mut primary_audit = AuditTrail::new();
    let mut aggregate_first_boundary: Option<u32> = None;
    let mut aggregate_first_violation: Option<u32> = None;

    for k in 0..n_cycles {
        let cycle = (k + 1) as u32;

        // Advance each channel's grammar engine
        let mut states_buf = [GrammarState::Admissible; MAX_CHANNELS];
        for ch in 0..num_channels {
            if k < channel_residuals[ch].len() {
                let sign = sign_at(
                    &channel_residuals[ch],
                    &channel_drifts[ch],
                    &channel_slews[ch],
                    k, 1,
                );
                channel_engines[ch].advance(&sign, &channel_envelopes[ch], config);

                // Record per-channel first boundary
                if channel_engines[ch].first_boundary_cycle().is_some()
                    && channel_boundary_cycles[ch].1.is_none()
                {
                    channel_boundary_cycles[ch].1 = channel_engines[ch].first_boundary_cycle();
                }
            }
            states_buf[ch] = channel_engines[ch].state();
        }

        // Aggregate
        let agg_state = aggregate_grammar(
            &states_buf[..num_channels],
            config.channel_vote_fraction,
        );

        // Reason code from primary channel (channel 0)
        let reason = if num_channels > 0 && k < channel_residuals[0].len() {
            let env_stressed = channel_envelopes[0].classify_position(
                channel_residuals[0][k]
            ) != crate::core::envelope::EnvelopeStatus::Interior;

            bank.match_motif(
                channel_drifts[0][k],
                channel_slews[0][k],
                agg_state,
                env_stressed,
            )
        } else {
            EngineReasonCode::NoAnomaly
        };

        // Track aggregate transitions
        if agg_state.severity() >= GrammarState::Boundary.severity()
            && aggregate_first_boundary.is_none()
        {
            aggregate_first_boundary = Some(cycle);
        }
        if agg_state == GrammarState::Violation && aggregate_first_violation.is_none() {
            aggregate_first_violation = Some(cycle);
        }

        // Audit entry for primary channel
        if num_channels > 0 && k < channel_residuals[0].len() {
            let entry = AuditEntry {
                cycle,
                residual: channel_residuals[0][k],
                drift: channel_drifts[0][k],
                slew: channel_slews[0][k],
                envelope_position: channel_envelopes[0].normalized_position(
                    channel_residuals[0][k]
                ),
                envelope_status: channel_envelopes[0].classify_position(
                    channel_residuals[0][k]
                ),
                grammar_state: agg_state,
                reason_code: reason,
                drift_persistence: 0, // simplified
                slew_persistence: 0,
            };
            primary_audit.push(entry);
        }

        grammar_trajectory.push(agg_state);
        reason_trajectory.push(reason);
    }

    // ── Episode formation ───────────────────────────────────────────
    let episodes = form_episodes(
        unit,
        &grammar_trajectory,
        &reason_trajectory,
        if !channel_drifts.is_empty() { &channel_drifts[0] } else { &[] },
        if !channel_slews.is_empty() { &channel_slews[0] } else { &[] },
    );

    // ── Structural lead time ────────────────────────────────────────
    let structural_lead_time = aggregate_first_boundary.map(|fb| {
        if total_cycles > fb { total_cycles - fb } else { 0 }
    });

    // ── Theorem 1 bound ─────────────────────────────────────────────
    let theorem_bound = if !channel_drifts.is_empty() && !channel_residuals.is_empty() {
        compute_theorem_bound(
            &channel_residuals[0],
            &channel_drifts[0],
            &channel_envelopes[0],
            config,
            aggregate_first_boundary,
        )
    } else {
        None
    };

    EngineEvalResult {
        unit,
        total_cycles,
        grammar_trajectory,
        reason_trajectory,
        first_boundary_cycle: aggregate_first_boundary,
        first_violation_cycle: aggregate_first_violation,
        structural_lead_time,
        episodes,
        theorem_bound,
        channel_boundary_cycles,
        primary_audit,
    }
}

/// Forms episodes from grammar trajectory.
fn form_episodes(
    unit: u16,
    grammar: &[GrammarState],
    reasons: &[EngineReasonCode],
    drifts: &[f64],
    slews: &[f64],
) -> Vec<Episode> {
    let mut episodes = Vec::new();
    let n = grammar.len();
    let mut i = 0;

    while i < n {
        if grammar[i].severity() >= GrammarState::Boundary.severity() {
            let start = i;
            let mut peak_state = grammar[i];
            let mut peak_reason = if i < reasons.len() { reasons[i] } else { EngineReasonCode::NoAnomaly };
            let mut max_drift = 0.0f64;
            let mut max_slew = 0.0f64;

            while i < n && grammar[i].severity() >= GrammarState::Boundary.severity() {
                if grammar[i].severity() > peak_state.severity() {
                    peak_state = grammar[i];
                }
                if i < reasons.len() && reasons[i].is_anomalous() {
                    peak_reason = reasons[i];
                }
                if i < drifts.len() && drifts[i].abs() > max_drift {
                    max_drift = drifts[i].abs();
                }
                if i < slews.len() && slews[i].abs() > max_slew {
                    max_slew = slews[i].abs();
                }
                i += 1;
            }

            episodes.push(Episode {
                unit,
                start_cycle: (start + 1) as u32,
                end_cycle: i as u32,
                peak_state,
                reason_code: peak_reason,
                max_drift,
                max_slew,
                duration_cycles: (i - start) as u32,
                trigger_channel: 0,
            });
        } else {
            i += 1;
        }
    }

    episodes
}

/// Computes Theorem 1 bound from observed trajectory.
fn compute_theorem_bound(
    residuals: &[f64],
    drifts: &[f64],
    envelope: &AdmissibilityEnvelope,
    config: &DsfbConfig,
    first_boundary: Option<u32>,
) -> Option<TheoremOneBound> {
    if residuals.len() < config.healthy_window + config.drift_window {
        return None;
    }

    // Find onset of sustained outward drift
    let start = config.healthy_window;
    let mut drift_onset: Option<usize> = None;
    let mut consecutive = 0u32;

    for k in start..drifts.len() {
        if drifts[k].abs() > config.drift_floor {
            consecutive += 1;
            if consecutive >= config.persistence_threshold as u32 && drift_onset.is_none() {
                drift_onset = Some(k - config.persistence_threshold + 1);
            }
        } else {
            consecutive = 0;
        }
    }

    let onset_idx = drift_onset?;
    let initial_gap = envelope.gap(residuals[onset_idx]);
    
    // Estimate sustained drift rate (median absolute drift in the persistence window)
    let mut drift_sum = 0.0;
    let mut drift_count = 0u32;
    for k in onset_idx..drifts.len().min(onset_idx + config.persistence_threshold * 2) {
        if drifts[k].abs() > config.drift_floor {
            drift_sum += drifts[k].abs();
            drift_count += 1;
        }
    }
    let avg_drift = if drift_count > 0 { drift_sum / drift_count as f64 } else { 0.0 };

    Some(TheoremOneBound::compute(
        initial_gap.max(0.0),
        avg_drift,
        0.0, // Fixed envelope: no expansion
        first_boundary,
        Some((onset_idx + 1) as u32),
    ))
}
