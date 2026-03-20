use serde::Serialize;

use crate::engine::types::{SignProjectionMetadata, SignProjectionMethod};

#[derive(Clone, Debug, Serialize)]
pub struct HashDigest {
    pub label: String,
    pub fnv1a_64_hex: String,
}

const FNV1A_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x100000001b3;

pub fn euclidean_norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum::<f64>().sqrt()
}

pub fn max_abs(values: &[f64]) -> f64 {
    values.iter().copied().map(f64::abs).fold(0.0, f64::max)
}

pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

pub fn rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
    }
}

pub fn count_where<F>(values: &[f64], predicate: F) -> usize
where
    F: Fn(f64) -> bool,
{
    values
        .iter()
        .copied()
        .filter(|value| predicate(*value))
        .count()
}

pub fn argmax(values: &[f64]) -> Option<usize> {
    values
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
}

pub fn fnv1a_hex(label: impl Into<String>, sequences: &[Vec<f64>]) -> HashDigest {
    let mut hash = FNV1A_OFFSET_BASIS;
    for sequence in sequences {
        for value in sequence {
            for byte in value.to_bits().to_le_bytes() {
                hash = fnv1a_extend(hash, byte);
            }
        }
    }
    HashDigest {
        label: label.into(),
        fnv1a_64_hex: format!("{hash:016x}"),
    }
}

pub fn hash_serializable_hex<T: Serialize>(
    label: impl Into<String>,
    value: &T,
) -> anyhow::Result<HashDigest> {
    let bytes = serde_json::to_vec(value)?;
    let hash = bytes.into_iter().fold(FNV1A_OFFSET_BASIS, fnv1a_extend);
    Ok(HashDigest {
        label: label.into(),
        fnv1a_64_hex: format!("{hash:016x}"),
    })
}

pub fn dot_product(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right).map(|(l, r)| l * r).sum::<f64>()
}

pub fn radial_drift(residual: &[f64], drift: &[f64]) -> f64 {
    let norm = euclidean_norm(residual);
    if norm <= 1.0e-12 {
        0.0
    } else {
        dot_product(residual, drift) / norm
    }
}

pub fn signed_radial_drift(residual: &[f64], drift: &[f64]) -> f64 {
    radial_drift(residual, drift)
}

pub fn signed_aggregate_drift(residual: &[f64], drift: &[f64]) -> f64 {
    signed_radial_drift(residual, drift)
}

pub fn sign_with_deadband(value: f64, deadband: f64) -> i8 {
    if value > deadband {
        1
    } else if value < -deadband {
        -1
    } else {
        0
    }
}

pub fn scalar_derivative(values: &[f64], times: &[f64]) -> Vec<f64> {
    let count = values.len();
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![0.0];
    }

    (0..count)
        .map(|index| {
            if index == 0 {
                safe_difference(values[1], values[0], times[1] - times[0])
            } else if index + 1 == count {
                safe_difference(
                    values[count - 1],
                    values[count - 2],
                    times[count - 1] - times[count - 2],
                )
            } else {
                safe_difference(
                    values[index + 1],
                    values[index - 1],
                    times[index + 1] - times[index - 1],
                )
            }
        })
        .collect()
}

/// Returns the ratio of net residual-norm change to total residual-norm path variation.
/// A value near 1 means most variation supports a single net direction; it is not a claim
/// about monotonicity in every channel.
pub fn residual_norm_path_monotonicity(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let total_variation = values
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .sum::<f64>();
    if total_variation <= 1.0e-12 {
        0.0
    } else {
        ((values.last().copied().unwrap_or_default() - values[0]).abs() / total_variation)
            .clamp(0.0, 1.0)
    }
}

pub fn monotonicity_score(values: &[f64]) -> f64 {
    residual_norm_path_monotonicity(values)
}

/// Returns the fraction of nonzero residual-norm increments that align with the net
/// residual-norm trend sign over the sampled window.
pub fn trend_aligned_increment_fraction(values: &[f64], deadband: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let deltas = values
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect::<Vec<_>>();
    let net = values.last().copied().unwrap_or_default() - values[0];
    let trend_sign = sign_with_deadband(net, deadband);
    if trend_sign == 0 {
        return 0.0;
    }

    let active = deltas
        .iter()
        .map(|delta| sign_with_deadband(*delta, deadband))
        .filter(|sign| *sign != 0)
        .collect::<Vec<_>>();
    if active.is_empty() {
        0.0
    } else {
        active.iter().filter(|sign| **sign == trend_sign).count() as f64 / active.len() as f64
    }
}

pub fn monotone_alignment_fraction(values: &[f64], deadband: f64) -> f64 {
    trend_aligned_increment_fraction(values, deadband)
}

/// Returns the dominant share of nonzero radial-drift signs.
pub fn dominant_nonzero_sign_fraction(signs: &[i8]) -> f64 {
    let mut positive = 0usize;
    let mut negative = 0usize;
    for sign in signs {
        match sign {
            1 => positive += 1,
            -1 => negative += 1,
            _ => {}
        }
    }
    let active = positive + negative;
    if active == 0 {
        0.0
    } else {
        positive.max(negative) as f64 / active as f64
    }
}

pub fn dominant_sign_fraction(signs: &[i8]) -> f64 {
    dominant_nonzero_sign_fraction(signs)
}

/// Returns adjacent agreement across the active nonzero sign sequence.
pub fn adjacent_sign_agreement_fraction(signs: &[i8]) -> f64 {
    let active = signs
        .iter()
        .copied()
        .filter(|sign| *sign != 0)
        .collect::<Vec<_>>();
    if active.len() < 2 {
        return 0.0;
    }

    let same_direction = active
        .windows(2)
        .filter(|window| window[0] == window[1])
        .count();
    same_direction as f64 / (active.len() - 1) as f64
}

pub fn persistence_fraction(signs: &[i8]) -> f64 {
    adjacent_sign_agreement_fraction(signs)
}

/// Returns within-sample sign alignment across active drift channels.
/// A value near 1 indicates most active drift channels share the same sign.
pub fn within_sample_sign_alignment(values: &[f64], deadband: f64) -> f64 {
    let signs = values
        .iter()
        .map(|value| sign_with_deadband(*value, deadband))
        .filter(|sign| *sign != 0)
        .collect::<Vec<_>>();
    if signs.is_empty() {
        0.0
    } else {
        (signs.iter().map(|sign| *sign as f64).sum::<f64>().abs() / signs.len() as f64)
            .clamp(0.0, 1.0)
    }
}

pub fn channel_sign_coherence(values: &[f64], deadband: f64) -> f64 {
    within_sample_sign_alignment(values, deadband)
}

pub fn positive_fraction(values: &[f64], deadband: f64) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().filter(|value| **value > deadband).count() as f64 / values.len() as f64
    }
}

pub fn standard_deviation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mu = mean(values);
    let variance = values
        .iter()
        .map(|value| {
            let centered = value - mu;
            centered * centered
        })
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt()
}

pub fn episode_count(flags: &[bool]) -> usize {
    let mut count = 0usize;
    let mut active = false;
    for flag in flags {
        if *flag && !active {
            count += 1;
            active = true;
        } else if !*flag {
            active = false;
        }
    }
    count
}

pub fn recovery_count(flags: &[bool]) -> usize {
    let mut count = 0usize;
    let mut active = false;
    for flag in flags {
        if *flag {
            active = true;
        } else if active {
            count += 1;
            active = false;
        }
    }
    count
}

/// Returns the sum of positive normalized excess above a deterministic threshold.
/// This is used as a compact spike-strength summary rather than a claim about impulse energy.
pub fn positive_excess_strength(values: &[f64], threshold: f64) -> f64 {
    let normalizer = threshold.abs().max(1.0e-12);
    values
        .iter()
        .map(|value| ((*value - threshold).max(0.0)) / normalizer)
        .sum::<f64>()
}

/// Returns a deterministic early-to-late slew-growth score derived from the slew-norm
/// baseline, peak, and tail averages.
pub fn late_slew_growth_score(values: &[f64]) -> f64 {
    if values.len() < 4 {
        return 0.0;
    }

    let baseline_len = (values.len() / 4).max(2);
    let tail_len = baseline_len;
    let baseline = mean(&values[..baseline_len]);
    let terminal = mean(&values[values.len() - tail_len..]);
    let peak = values.iter().copied().fold(0.0, f64::max);
    let onset_gain = normalized_positive_rise(peak, baseline);
    let sustained_gain = normalized_positive_rise(terminal, baseline);
    (0.6 * onset_gain + 0.4 * sustained_gain).clamp(0.0, 1.0)
}

pub fn curvature_onset_score(values: &[f64]) -> f64 {
    late_slew_growth_score(values)
}

pub fn project_sign(residual: &[f64], drift: &[f64], slew: &[f64]) -> [f64; 3] {
    [
        euclidean_norm(residual),
        signed_radial_drift(residual, drift),
        euclidean_norm(slew),
    ]
}

pub fn sign_projection_metadata() -> SignProjectionMetadata {
    SignProjectionMetadata {
        method: SignProjectionMethod::AggregateNormSignedRadialDrift,
        axis_labels: [
            "||r(t)||".to_string(),
            "signed radial drift".to_string(),
            "||s(t)||".to_string(),
        ],
        note: "Deterministic aggregate projection using residual norm, signed radial drift `dot(r(t), d(t))/||r(t)||` with zero reported at exact zero residual norm, and slew norm. This is not a latent-state embedding.".to_string(),
    }
}

pub fn pairwise_abs_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().map(|value| value.abs()).sum::<f64>() / values.len() as f64
    }
}

/// Formats scalar metrics with enough precision to keep small nonzero values visible in
/// reports and summary strings without making moderate-size values noisy.
pub fn format_metric(value: f64) -> String {
    let magnitude = value.abs();
    if magnitude <= 1.0e-12 {
        "0".to_string()
    } else if magnitude >= 100.0 {
        format!("{value:.3}")
    } else if magnitude >= 1.0 {
        format!("{value:.4}")
    } else if magnitude >= 1.0e-2 {
        format!("{value:.5}")
    } else if magnitude >= 1.0e-4 {
        format!("{value:.7}")
    } else {
        format!("{value:.3e}")
    }
}

fn safe_difference(upper: f64, lower: f64, delta_t: f64) -> f64 {
    if delta_t.abs() <= 1.0e-12 {
        0.0
    } else {
        (upper - lower) / delta_t
    }
}

fn fnv1a_extend(hash: u64, byte: u8) -> u64 {
    let hash = hash ^ byte as u64;
    hash.wrapping_mul(FNV1A_PRIME)
}

fn normalized_positive_rise(upper: f64, lower: f64) -> f64 {
    let rise = (upper - lower).max(0.0);
    let normalizer = upper.abs() + lower.abs() + 1.0e-12;
    (rise / normalizer).clamp(0.0, 1.0)
}
