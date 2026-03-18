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
        let candidate_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };
        if candidate_len > width && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else if current.is_empty() {
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
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
