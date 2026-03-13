use serde::Serialize;

use crate::config::KernelKind;
use crate::observer::ObserverSeries;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct KernelEvaluation {
    pub signal: f64,
    pub weight: f64,
    pub compatibility: f64,
}

pub fn evaluate_kernel(
    kind: KernelKind,
    source: &ObserverSeries,
    target: &ObserverSeries,
    corrected_time: usize,
    anchor_time: usize,
    delta: usize,
    resonance_threshold: f64,
) -> KernelEvaluation {
    let start = corrected_time.saturating_sub(delta);
    let end = anchor_time.min(corrected_time.saturating_add(delta));
    let mut weighted_signal = 0.0;
    let mut weight_sum = 0.0;
    let mut compatibility_sum = 0.0;

    for tau in start..=end {
        let distance = tau.abs_diff(corrected_time) as f64;
        let base_weight = match kind {
            KernelKind::Uniform => 1.0,
            KernelKind::Exponential | KernelKind::ResonanceGated => {
                let scale = delta.max(1) as f64;
                (-distance / scale).exp()
            }
        };
        let source_slope = slope_at(&source.estimate, tau);
        let target_slope = slope_at(&target.estimate, corrected_time);
        let slope_gap = (source_slope - target_slope).abs();
        let same_direction = source.correction_driver(tau).signum()
            == target.residual[corrected_time].signum()
            || target.residual[corrected_time].abs() < 1e-6;
        let compatibility = if slope_gap <= resonance_threshold && same_direction {
            1.0
        } else if slope_gap <= resonance_threshold * 1.8 {
            0.55
        } else {
            0.18
        };
        let gated_weight = match kind {
            KernelKind::ResonanceGated => base_weight * compatibility,
            _ => base_weight,
        };
        weighted_signal += gated_weight * source.correction_driver(tau);
        weight_sum += gated_weight;
        compatibility_sum += compatibility;
    }

    if weight_sum <= f64::EPSILON {
        return KernelEvaluation {
            signal: 0.0,
            weight: 0.0,
            compatibility: 0.0,
        };
    }

    KernelEvaluation {
        signal: weighted_signal / weight_sum,
        weight: weight_sum / ((end - start + 1) as f64),
        compatibility: compatibility_sum / ((end - start + 1) as f64),
    }
}

fn slope_at(series: &[f64], index: usize) -> f64 {
    if index == 0 {
        0.0
    } else {
        series[index] - series[index - 1]
    }
}
