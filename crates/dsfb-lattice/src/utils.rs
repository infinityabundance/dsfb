use std::f64::consts::PI;
use std::path::Path;

use nalgebra::DMatrix;

pub fn time_points(length: usize, dt: f64) -> Vec<f64> {
    (0..length).map(|step| step as f64 * dt).collect()
}

pub fn padded_range(min: f64, max: f64) -> (f64, f64) {
    if !min.is_finite() || !max.is_finite() {
        return (-1.0, 1.0);
    }
    if (max - min).abs() < 1.0e-12 {
        let pad = if max.abs() < 1.0 { 1.0 } else { 0.1 * max.abs() };
        return (min - pad, max + pad);
    }
    let pad = 0.08 * (max - min).abs();
    (min - pad, max + pad)
}

pub fn min_max(values: &[f64]) -> (f64, f64) {
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    if !min.is_finite() || !max.is_finite() {
        (0.0, 1.0)
    } else {
        (min, max)
    }
}

pub fn max_abs(values: &[f64]) -> f64 {
    values.iter().map(|value| value.abs()).fold(0.0_f64, f64::max)
}

pub fn offdiag_energy(matrix: &DMatrix<f64>) -> f64 {
    let mut total = 0.0;
    for row in 0..matrix.nrows() {
        for column in 0..matrix.ncols() {
            if row != column {
                total += matrix[(row, column)].powi(2);
            }
        }
    }
    total.sqrt()
}

pub fn covariance_trace(matrix: &DMatrix<f64>) -> f64 {
    (0..matrix.nrows()).map(|index| matrix[(index, index)]).sum()
}

pub fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[derive(Clone, Debug)]
pub struct DeterministicRng {
    state: u64,
    cached_gaussian: Option<f64>,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed,
            cached_gaussian: None,
        }
    }

    pub fn next_f64(&mut self) -> f64 {
        let bits = self.next_u64() >> 11;
        bits as f64 / ((1_u64 << 53) as f64)
    }

    pub fn next_gaussian(&mut self) -> f64 {
        if let Some(value) = self.cached_gaussian.take() {
            return value;
        }

        let u1 = self.next_f64().max(1.0e-12);
        let u2 = self.next_f64();
        let radius = (-2.0 * u1.ln()).sqrt();
        let angle = 2.0 * PI * u2;
        let z0 = radius * angle.cos();
        let z1 = radius * angle.sin();
        self.cached_gaussian = Some(z1);
        z0
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

pub fn escape_pdf_text(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

pub fn wrap_text(input: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in input.split_whitespace() {
        for chunk in split_token(word, width) {
            let candidate_len = if current.is_empty() {
                chunk.len()
            } else {
                current.len() + 1 + chunk.len()
            };
            if candidate_len > width && !current.is_empty() {
                lines.push(current);
                current = chunk;
            } else if current.is_empty() {
                current = chunk;
            } else {
                current.push(' ');
                current.push_str(&chunk);
            }
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn split_token(token: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![token.to_string()];
    }

    let characters: Vec<char> = token.chars().collect();
    if characters.len() <= width {
        return vec![token.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < characters.len() {
        let end = (start + width).min(characters.len());
        chunks.push(characters[start..end].iter().collect());
        start = end;
    }
    chunks
}
