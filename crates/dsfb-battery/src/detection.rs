// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — Detection and grammar evaluation
//
// Implements grammar state evaluation (Definition 2), persistence-gated
// transitions (Proposition 3), reason code assignment (Section 5),
// full pipeline execution, and threshold-baseline comparison.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use crate::math::{compute_all_drifts, compute_all_residuals, compute_all_slews, compute_envelope};
use crate::types::{
    BatteryResidual, DetectionResult, EnvelopeParams, GrammarState, PipelineConfig, ReasonCode,
    SignTuple, Theorem1Result,
};
use thiserror::Error;

/// Errors arising from detection operations.
#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("math error: {0}")]
    Math(#[from] crate::math::MathError),
    #[error("no capacity data provided")]
    EmptyData,
    #[error("healthy window ({window}) exceeds data length ({len})")]
    InsufficientData { window: usize, len: usize },
}

/// Evaluate grammar state at a single cycle.
///
/// **Definition 2 (Paper):** Three-level classification:
///   Admissible — |r_k| ≤ ρ and no persistent outward drift
///   Boundary   — |r_k| > boundary_fraction × ρ, or persistent outward drift
///   Violation  — |r_k| > ρ
///
/// **Proposition 3 (Paper):** Persistence gating — drift and slew must
/// exceed their respective thresholds for L_d and L_s consecutive cycles
/// before triggering grammar transition.
///
/// `drift_persist_count` and `slew_persist_count` are the number of
/// consecutive prior cycles where drift/slew exceeded threshold.
pub fn evaluate_grammar_state(
    residual: f64,
    envelope: &EnvelopeParams,
    drift: f64,
    _slew: f64,
    drift_persist_count: usize,
    slew_persist_count: usize,
    config: &PipelineConfig,
) -> GrammarState {
    let abs_r = residual.abs();

    // Violation: residual exits envelope
    if abs_r > envelope.rho {
        return GrammarState::Violation;
    }

    // Boundary conditions (any of):
    //  1. Residual magnitude exceeds boundary fraction of envelope
    //  2. Persistent outward drift (Proposition 3: L_d consecutive cycles)
    //  3. Persistent slew with persistent drift (acceleration, Proposition 3)
    let near_boundary = abs_r > config.boundary_fraction * envelope.rho;
    let persistent_drift =
        drift.abs() > config.drift_threshold && drift_persist_count >= config.drift_persistence;
    let persistent_slew_and_drift = slew_persist_count >= config.slew_persistence
        && drift_persist_count >= config.drift_persistence;

    if near_boundary || persistent_drift || persistent_slew_and_drift {
        return GrammarState::Boundary;
    }

    GrammarState::Admissible
}

/// Compute the next persistence counter value for a boolean threshold condition.
///
/// This helper exists for addendum proof harnesses and wrapper layers. It
/// mirrors the current pipeline rule used in the batch evaluator:
/// increment while the threshold condition remains true, otherwise reset.
pub fn next_persistence_count(current: usize, threshold_condition_met: bool) -> usize {
    if threshold_condition_met {
        current + 1
    } else {
        0
    }
}

/// Assign a typed reason code based on sign tuple and grammar state.
///
/// **Section 5 (Paper):** Reason codes are typed structural interpretations
/// under declared conditions, not mechanistic certainty claims.
///
/// This implementation covers the two primary single-channel reason codes
/// applicable to the B0005 capacity-only dataset:
///   - SustainedCapacityFade: monotone outward drift without acceleration
///   - AcceleratingFadeKnee: persistent drift with persistent positive slew
pub fn assign_reason_code(
    sign: &SignTuple,
    grammar_state: GrammarState,
    drift_persist_count: usize,
    slew_persist_count: usize,
    config: &PipelineConfig,
) -> Option<ReasonCode> {
    if grammar_state == GrammarState::Admissible {
        return None;
    }

    // AcceleratingFadeKnee: both drift and slew persistent (Proposition 3)
    // Drift is outward (negative for capacity fade) and slew shows acceleration
    if drift_persist_count >= config.drift_persistence
        && slew_persist_count >= config.slew_persistence
        && sign.d.abs() > config.drift_threshold
        && sign.s.abs() > config.slew_threshold
    {
        return Some(ReasonCode::AcceleratingFadeKnee);
    }

    // SustainedCapacityFade: persistent drift without acceleration
    if drift_persist_count >= config.drift_persistence && sign.d.abs() > config.drift_threshold {
        return Some(ReasonCode::SustainedCapacityFade);
    }

    // Boundary from envelope approach without persistent drift
    if grammar_state == GrammarState::Boundary || grammar_state == GrammarState::Violation {
        return Some(ReasonCode::SustainedCapacityFade);
    }

    None
}

/// Run the complete DSFB pipeline on a capacity sequence.
///
/// Implements the full semiotic analysis chain:
///   1. Residual construction (Definition 1)
///   2. Drift computation (windowed first difference)
///   3. Slew computation (windowed second difference)
///   4. Envelope parameterization from healthy window (Definition 3)
///   5. Grammar state evaluation at each cycle (Definition 2, Proposition 3)
///   6. Reason code assignment (Section 5)
///
/// Returns the envelope parameters and the per-cycle trajectory.
pub fn run_dsfb_pipeline(
    capacities: &[f64],
    config: &PipelineConfig,
) -> Result<(EnvelopeParams, Vec<BatteryResidual>), DetectionError> {
    let n = capacities.len();
    if n == 0 {
        return Err(DetectionError::EmptyData);
    }
    if config.healthy_window > n {
        return Err(DetectionError::InsufficientData {
            window: config.healthy_window,
            len: n,
        });
    }

    // Step 1: Compute envelope from healthy window (Definition 3)
    let healthy_data = &capacities[..config.healthy_window];
    let envelope = compute_envelope(healthy_data)?;

    // Step 2: Compute all residuals (Definition 1)
    let residuals = compute_all_residuals(capacities, envelope.mu);

    // Step 3: Compute all drifts (windowed first difference)
    let drifts = compute_all_drifts(&residuals, config.drift_window);

    // Step 4: Compute all slews (windowed second difference of drift)
    let slews = compute_all_slews(&drifts, config.drift_window);

    // Steps 5–6: Grammar state evaluation with persistence gating
    let mut trajectory = Vec::with_capacity(n);
    let mut drift_persist_count: usize = 0;
    let mut slew_persist_count: usize = 0;

    for k in 0..n {
        let sign = SignTuple {
            r: residuals[k],
            d: drifts[k],
            s: slews[k],
        };

        // Update persistence counters (Proposition 3)
        // Outward drift for capacity: negative drift means capacity is falling
        if drifts[k] < -config.drift_threshold {
            drift_persist_count += 1;
        } else {
            drift_persist_count = 0;
        }

        // Slew: negative slew means drift is accelerating (becoming more negative)
        if slews[k] < -config.slew_threshold {
            slew_persist_count += 1;
        } else {
            slew_persist_count = 0;
        }

        let grammar_state = evaluate_grammar_state(
            residuals[k],
            &envelope,
            drifts[k],
            slews[k],
            drift_persist_count,
            slew_persist_count,
            config,
        );

        let reason_code = assign_reason_code(
            &sign,
            grammar_state,
            drift_persist_count,
            slew_persist_count,
            config,
        );

        trajectory.push(BatteryResidual {
            cycle: k + 1,
            capacity_ah: capacities[k],
            sign,
            grammar_state,
            reason_code,
        });
    }

    Ok((envelope, trajectory))
}

/// Detect the first DSFB alarm cycle.
///
/// **Paper evaluation metric:**
///   k_alarm = inf { k : Γ_k ∈ {Boundary, Violation} }
///
/// Returns the cycle number (1-indexed) of the first non-Admissible state.
pub fn detect_dsfb_alarm(trajectory: &[BatteryResidual]) -> Option<usize> {
    trajectory.iter().find_map(|br| {
        if br.grammar_state != GrammarState::Admissible {
            Some(br.cycle)
        } else {
            None
        }
    })
}

/// Detect the end-of-life cycle using a capacity threshold.
///
/// **Paper evaluation metric:**
///   k_EOL = inf { k : C_k < C_EOL }
///
/// `eol_capacity` is the absolute end-of-life capacity threshold (Ah).
/// Returns the cycle number (1-indexed).
pub fn detect_eol(capacities: &[f64], eol_capacity: f64) -> Option<usize> {
    capacities
        .iter()
        .enumerate()
        .find_map(|(i, c)| if *c < eol_capacity { Some(i + 1) } else { None })
}

/// Detect alarm using a simple threshold baseline (comparator).
///
/// This is the blunt capacity-threshold crossing baseline from the paper's
/// detection comparison methodology (Section 8).
///
/// `threshold` is as a fraction of the first cycle's capacity, e.g. 0.85
/// means alarm when capacity drops below 85% of initial.
/// Returns the cycle number (1-indexed).
pub fn detect_threshold_alarm(capacities: &[f64], threshold_fraction: f64) -> Option<usize> {
    if capacities.is_empty() {
        return None;
    }
    let initial = capacities[0];
    let threshold = threshold_fraction * initial;
    capacities
        .iter()
        .enumerate()
        .find_map(|(i, c)| if *c < threshold { Some(i + 1) } else { None })
}

/// Build a DetectionResult for DSFB alarm.
pub fn build_dsfb_detection(
    trajectory: &[BatteryResidual],
    capacities: &[f64],
    eol_capacity: f64,
) -> DetectionResult {
    let alarm = detect_dsfb_alarm(trajectory);
    let eol = detect_eol(capacities, eol_capacity);
    let lead = match (alarm, eol) {
        (Some(a), Some(e)) => Some(e as i64 - a as i64),
        _ => None,
    };
    DetectionResult {
        method: String::from("DSFB Structural Alarm"),
        alarm_cycle: alarm,
        eol_cycle: eol,
        lead_time_cycles: lead,
    }
}

/// Build a DetectionResult for threshold baseline alarm.
pub fn build_threshold_detection(
    capacities: &[f64],
    threshold_fraction: f64,
    eol_capacity: f64,
) -> DetectionResult {
    let alarm = detect_threshold_alarm(capacities, threshold_fraction);
    let eol = detect_eol(capacities, eol_capacity);
    let lead = match (alarm, eol) {
        (Some(a), Some(e)) => Some(e as i64 - a as i64),
        _ => None,
    };
    DetectionResult {
        method: format!(
            "Threshold Baseline ({:.0}% of initial)",
            threshold_fraction * 100.0
        ),
        alarm_cycle: alarm,
        eol_cycle: eol,
        lead_time_cycles: lead,
    }
}

/// Verify Theorem 1 against observed detection results.
///
/// **Theorem 1 (Paper):** Under sustained outward drift η with envelope
/// expansion κ, the first envelope exit satisfies:
///   k* − k_0 ≤ ⌈ g_{k_0} / (η − κ) ⌉
///
/// For static envelope (κ = 0), this simplifies to ⌈ ρ / η ⌉.
///
/// This function estimates η from the observed drift sequence as the
/// median absolute drift over the post-healthy window, computes the
/// theoretical bound, and compares with the actual detection cycle.
pub fn verify_theorem1(
    envelope: &EnvelopeParams,
    trajectory: &[BatteryResidual],
    config: &PipelineConfig,
) -> Theorem1Result {
    // Estimate sustained outward drift α from the trajectory.
    // Use the median of absolute drift values where drift is outward
    // (negative for capacity fade), over the post-healthy-window region.
    let mut outward_drifts: Vec<f64> = trajectory
        .iter()
        .skip(config.healthy_window + config.drift_window)
        .filter(|br| br.sign.d < 0.0) // outward = capacity falling
        .map(|br| br.sign.d.abs())
        .collect();

    outward_drifts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let alpha = if outward_drifts.is_empty() {
        0.0
    } else {
        // Use median as robust estimate of sustained drift rate
        let mid = outward_drifts.len() / 2;
        outward_drifts[mid]
    };

    let kappa = 0.0; // Static envelope in Stage II

    let actual_detection = detect_dsfb_alarm(trajectory);

    if alpha <= kappa {
        return Theorem1Result {
            rho: envelope.rho,
            alpha,
            kappa,
            t_star: 0,
            actual_detection_cycle: actual_detection,
            bound_satisfied: None, // Cannot verify: sustained drift assumption not met
        };
    }

    // t* = ⌈ ρ / (α − κ) ⌉
    let t_star = (envelope.rho / (alpha - kappa)).ceil() as usize;

    // The actual detection lag is measured from end of healthy window
    let detection_lag = actual_detection.map(|a| {
        if a > config.healthy_window {
            a - config.healthy_window
        } else {
            0
        }
    });

    let bound_satisfied = detection_lag.map(|lag| t_star >= lag);

    Theorem1Result {
        rho: envelope.rho,
        alpha,
        kappa,
        t_star,
        actual_detection_cycle: actual_detection,
        bound_satisfied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_runs_on_constant_data() {
        // Constant capacity: should stay Admissible, no alarms
        let capacities: Vec<f64> = (0..50).map(|_| 2.0).collect();
        let config = PipelineConfig {
            healthy_window: 20,
            drift_window: 5,
            ..PipelineConfig::default()
        };
        // Constant data has zero std dev, which is an edge case.
        // We expect an error because the envelope sigma is zero.
        let result = run_dsfb_pipeline(&capacities, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_runs_on_slightly_noisy_data() {
        // Slightly noisy but stable capacity: should stay mostly Admissible
        let capacities: Vec<f64> = (0..50)
            .map(|i| 2.0 + 0.001 * ((i as f64 * 1.7).sin()))
            .collect();
        let config = PipelineConfig::default();
        let (env, traj) = run_dsfb_pipeline(&capacities, &config).unwrap();
        assert!(env.rho > 0.0);
        assert_eq!(traj.len(), 50);
    }

    #[test]
    fn test_detect_eol() {
        let capacities = vec![2.0, 1.9, 1.8, 1.7, 1.6, 1.5, 1.4, 1.3];
        // EOL at 1.5: first value strictly < 1.5 is 1.4 at index 6 → cycle 7
        let eol = detect_eol(&capacities, 1.5);
        assert_eq!(eol, Some(7));
    }

    #[test]
    fn test_detect_threshold_alarm() {
        let capacities = vec![2.0, 1.9, 1.8, 1.7, 1.6, 1.5];
        // 85% of initial = 1.7, first below at index 4 (capacity 1.6)
        let alarm = detect_threshold_alarm(&capacities, 0.85);
        assert_eq!(alarm, Some(5)); // cycle 5 = index 4 = 1.6 < 1.7
    }
}
