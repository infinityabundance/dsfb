use nalgebra::DMatrix;

pub fn degree_matrix(adjacency: &DMatrix<f64>) -> DMatrix<f64> {
    let n = adjacency.nrows();
    let mut degree = DMatrix::zeros(n, n);
    for row in 0..n {
        degree[(row, row)] = adjacency.row(row).iter().sum::<f64>();
    }
    degree
}

pub fn laplacian(adjacency: &DMatrix<f64>) -> DMatrix<f64> {
    degree_matrix(adjacency) - adjacency
}

pub fn frobenius_norm(matrix: &DMatrix<f64>) -> f64 {
    matrix.iter().map(|value| value * value).sum::<f64>().sqrt()
}

pub fn delta_norm(current: &DMatrix<f64>, previous: Option<&DMatrix<f64>>) -> f64 {
    previous
        .map(|matrix| frobenius_norm(&(current - matrix)))
        .unwrap_or(0.0)
}
