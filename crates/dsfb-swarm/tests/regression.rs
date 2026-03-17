use anyhow::Result;
use nalgebra::DMatrix;

use dsfb_swarm::config::TrustGateMode;
use dsfb_swarm::math::laplacian::laplacian;
use dsfb_swarm::math::residuals::compute_residual_stack;
use dsfb_swarm::math::spectrum::compute_spectrum;
use dsfb_swarm::math::trust::TrustModel;

#[test]
fn laplacian_construction_sanity() {
    let adjacency = DMatrix::from_row_slice(
        3,
        3,
        &[
            0.0, 1.0, 0.0, //
            1.0, 0.0, 1.0, //
            0.0, 1.0, 0.0,
        ],
    );
    let lap = laplacian(&adjacency);
    assert_eq!(lap[(0, 0)], 1.0);
    assert_eq!(lap[(1, 1)], 2.0);
    assert_eq!(lap[(0, 1)], -1.0);
    assert_eq!(lap[(1, 2)], -1.0);
}

#[test]
fn eigenvalues_are_sorted() {
    let adjacency = DMatrix::from_row_slice(
        4,
        4,
        &[
            0.0, 1.0, 0.0, 0.0, //
            1.0, 0.0, 1.0, 0.0, //
            0.0, 1.0, 0.0, 1.0, //
            0.0, 0.0, 1.0, 0.0,
        ],
    );
    let spectrum = compute_spectrum(&laplacian(&adjacency));
    assert!(spectrum
        .eigenvalues
        .windows(2)
        .all(|pair| pair[0] <= pair[1]));
    assert!(spectrum.lambda2 > 0.0);
}

#[test]
fn residual_computation_matches_toy_case() {
    let current_vectors = DMatrix::identity(4, 4);
    let previous_vectors = DMatrix::identity(4, 4);
    let residuals = compute_residual_stack(
        &[0.8, 1.2],
        &[1.0, 1.0],
        Some(&[-0.1, 0.1]),
        Some(&[0.0, 0.0]),
        &current_vectors,
        Some(&previous_vectors),
        0.5,
        true,
    );
    assert!((residuals.scalar_residual + 0.2).abs() < 1.0e-9);
    assert!((residuals.scalar_drift + 0.2).abs() < 1.0e-9);
    assert!((residuals.residuals[1] - 0.2).abs() < 1.0e-9);
}

#[test]
fn trust_update_suppresses_inconsistent_nodes() -> Result<()> {
    let adjacency = DMatrix::from_row_slice(
        3,
        3,
        &[
            0.0, 1.0, 1.0, //
            1.0, 0.0, 1.0, //
            1.0, 1.0, 0.0,
        ],
    );
    let disagreement = DMatrix::from_row_slice(
        3,
        3,
        &[
            0.0, 4.0, 4.0, //
            4.0, 0.0, 0.2, //
            4.0, 0.2, 0.0,
        ],
    );
    let mut trust = TrustModel::new(TrustGateMode::SmoothDecay, 3);
    let snapshot = trust.update(&adjacency, &disagreement, 1.5, &[0]);
    assert!(snapshot.node_trust[0] < snapshot.node_trust[1]);
    assert!(snapshot.node_trust[0] < snapshot.node_trust[2]);
    Ok(())
}
