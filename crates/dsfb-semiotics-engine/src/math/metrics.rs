use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct HashDigest {
    pub label: String,
    pub fnv1a_64_hex: String,
}

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
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    for sequence in sequences {
        for value in sequence {
            for byte in value.to_bits().to_le_bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(PRIME);
            }
        }
    }
    HashDigest {
        label: label.into(),
        fnv1a_64_hex: format!("{hash:016x}"),
    }
}

pub fn project_sign(residual: &[f64], drift: &[f64], slew: &[f64]) -> [f64; 3] {
    [
        residual.first().copied().unwrap_or_default(),
        drift.first().copied().unwrap_or_default(),
        slew.first().copied().unwrap_or_default(),
    ]
}

pub fn pairwise_abs_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().map(|value| value.abs()).sum::<f64>() / values.len() as f64
    }
}
