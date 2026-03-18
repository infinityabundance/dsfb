use anyhow::{bail, Result};
use nalgebra::{DMatrix, SymmetricEigen};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct SpectrumAnalysis {
    pub eigenvalues: Vec<f64>,
    pub frequencies: Vec<f64>,
    pub eigenvectors: DMatrix<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpectralComparison {
    pub experiment: String,
    pub delta_norm_2: f64,
    pub max_abs_shift: f64,
    pub max_shift_ratio: f64,
    pub bound_satisfied: bool,
    pub per_mode_abs_shift: Vec<f64>,
}

pub fn analyze_symmetric(matrix: &DMatrix<f64>) -> Result<SpectrumAnalysis> {
    if matrix.nrows() != matrix.ncols() {
        bail!("matrix must be square");
    }

    let decomposition = SymmetricEigen::new(matrix.clone());
    let mut order: Vec<usize> = (0..decomposition.eigenvalues.len()).collect();
    order.sort_by(|left, right| {
        decomposition.eigenvalues[*left]
            .partial_cmp(&decomposition.eigenvalues[*right])
            .unwrap()
    });

    let mut eigenvalues = Vec::with_capacity(order.len());
    let mut frequencies = Vec::with_capacity(order.len());
    let mut eigenvectors = DMatrix::<f64>::zeros(matrix.nrows(), matrix.ncols());
    for (sorted_column, original_column) in order.into_iter().enumerate() {
        let eigenvalue = decomposition.eigenvalues[original_column];
        eigenvalues.push(eigenvalue);
        frequencies.push(eigenvalue.max(0.0).sqrt());
        eigenvectors
            .column_mut(sorted_column)
            .copy_from(&decomposition.eigenvectors.column(original_column));
    }

    Ok(SpectrumAnalysis {
        eigenvalues,
        frequencies,
        eigenvectors,
    })
}

pub fn spectral_norm_2(matrix: &DMatrix<f64>) -> Result<f64> {
    let decomposition = SymmetricEigen::new(matrix.clone());
    Ok(decomposition
        .eigenvalues
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max))
}

pub fn compare_spectra(
    experiment: impl Into<String>,
    nominal: &SpectrumAnalysis,
    perturbed: &SpectrumAnalysis,
    delta: &DMatrix<f64>,
) -> Result<SpectralComparison> {
    if nominal.eigenvalues.len() != perturbed.eigenvalues.len() {
        bail!("nominal and perturbed spectra must have the same dimension");
    }

    let delta_norm_2 = spectral_norm_2(delta)?;
    let per_mode_abs_shift: Vec<f64> = nominal
        .eigenvalues
        .iter()
        .zip(perturbed.eigenvalues.iter())
        .map(|(lambda, lambda_prime)| (lambda_prime - lambda).abs())
        .collect();

    let max_abs_shift = per_mode_abs_shift
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let max_shift_ratio = if delta_norm_2 > 0.0 {
        max_abs_shift / delta_norm_2
    } else {
        0.0
    };
    let bound_satisfied = per_mode_abs_shift
        .iter()
        .all(|shift| *shift <= delta_norm_2 + 1.0e-10);

    Ok(SpectralComparison {
        experiment: experiment.into(),
        delta_norm_2,
        max_abs_shift,
        max_shift_ratio,
        bound_satisfied,
        per_mode_abs_shift,
    })
}
