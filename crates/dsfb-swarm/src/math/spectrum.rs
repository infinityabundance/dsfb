use nalgebra::{DMatrix, DVector, SymmetricEigen};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SpectralSnapshot {
    pub eigenvalues: Vec<f64>,
    pub eigenvectors: DMatrix<f64>,
    pub lambda2: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModeObservation {
    pub mode: usize,
    pub eigenvalue: f64,
    pub eigenvector_norm: f64,
}

pub fn compute_spectrum(laplacian: &DMatrix<f64>) -> SpectralSnapshot {
    let decomposition = SymmetricEigen::new(laplacian.clone());
    let mut pairs = (0..decomposition.eigenvalues.len())
        .map(|index| {
            (
                sanitize_eigenvalue(decomposition.eigenvalues[index]),
                decomposition.eigenvectors.column(index).into_owned(),
            )
        })
        .collect::<Vec<_>>();
    pairs.sort_by(|left, right| left.0.total_cmp(&right.0));

    let eigenvalues = pairs.iter().map(|pair| pair.0).collect::<Vec<_>>();
    let eigenvectors = DMatrix::from_columns(
        &pairs
            .iter()
            .map(|pair| pair.1.clone())
            .collect::<Vec<DVector<f64>>>(),
    );
    let lambda2 = eigenvalues.get(1).copied().unwrap_or(0.0);
    SpectralSnapshot {
        eigenvalues,
        eigenvectors,
        lambda2,
    }
}

pub fn monitored_modes(snapshot: &SpectralSnapshot, monitored_modes: usize) -> Vec<ModeObservation> {
    let available = snapshot.eigenvalues.len().saturating_sub(1);
    let count = monitored_modes.min(available);
    (0..count)
        .map(|offset| {
            let mode = offset + 2;
            let column = snapshot.eigenvectors.column(offset + 1);
            ModeObservation {
                mode,
                eigenvalue: snapshot.eigenvalues[offset + 1],
                eigenvector_norm: column.norm(),
            }
        })
        .collect()
}

pub fn mode_vector(snapshot: &SpectralSnapshot, mode: usize) -> Option<DVector<f64>> {
    if mode == 0 || mode > snapshot.eigenvectors.ncols() {
        return None;
    }
    Some(snapshot.eigenvectors.column(mode - 1).into_owned())
}

pub fn sign_ambiguous_distance(left: &DVector<f64>, right: &DVector<f64>) -> f64 {
    let same = (left - right).norm();
    let flipped = (left + right).norm();
    same.min(flipped)
}

fn sanitize_eigenvalue(value: f64) -> f64 {
    if value.abs() < 1.0e-10 {
        0.0
    } else {
        value
    }
}
