use crate::config::PipelineConfig;
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EwmaFeatureTrace {
    pub feature_index: usize,
    pub feature_name: String,
    pub ewma: Vec<f64>,
    pub healthy_mean: f64,
    pub healthy_std: f64,
    pub threshold: f64,
    pub alarm: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CusumFeatureTrace {
    pub feature_index: usize,
    pub feature_name: String,
    pub cusum: Vec<f64>,
    pub healthy_mean: f64,
    pub healthy_std: f64,
    pub kappa: f64,
    pub alarm_threshold: f64,
    pub alarm: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunEnergyTrace {
    pub energy: Vec<f64>,
    pub healthy_mean: f64,
    pub healthy_std: f64,
    pub threshold: f64,
    pub analyzable_feature_count: usize,
    pub alarm: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PcaFdcTrace {
    pub t2: Vec<f64>,
    pub spe: Vec<f64>,
    pub t2_healthy_mean: f64,
    pub t2_healthy_std: f64,
    pub spe_healthy_mean: f64,
    pub spe_healthy_std: f64,
    pub t2_threshold: f64,
    pub spe_threshold: f64,
    pub analyzable_feature_count: usize,
    pub healthy_observation_count: usize,
    pub retained_components: usize,
    pub explained_variance_fraction: f64,
    pub target_variance_explained: f64,
    pub alarm: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaselineSet {
    pub ewma: Vec<EwmaFeatureTrace>,
    pub cusum: Vec<CusumFeatureTrace>,
    pub run_energy: RunEnergyTrace,
    pub pca_fdc: PcaFdcTrace,
}

pub fn compute_baselines(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    config: &PipelineConfig,
) -> BaselineSet {
    let ewma = residuals
        .traces
        .iter()
        .zip(&nominal.features)
        .map(|(trace, feature)| {
            let ewma = ewma_series(&trace.norms, config.ewma_alpha);
            let healthy_ewma = dataset
                .healthy_pass_indices
                .iter()
                .filter_map(|&idx| ewma.get(idx).copied())
                .collect::<Vec<_>>();
            let healthy_mean = mean(&healthy_ewma).unwrap_or(0.0);
            let healthy_std = sample_std(&healthy_ewma, healthy_mean).unwrap_or(0.0);
            let threshold = if feature.analyzable {
                healthy_mean + config.ewma_sigma_multiplier * healthy_std.max(config.epsilon)
            } else {
                0.0
            };
            let alarm = ewma
                .iter()
                .map(|value| feature.analyzable && *value > threshold)
                .collect::<Vec<_>>();

            EwmaFeatureTrace {
                feature_index: trace.feature_index,
                feature_name: trace.feature_name.clone(),
                ewma,
                healthy_mean,
                healthy_std,
                threshold,
                alarm,
            }
        })
        .collect::<Vec<_>>();

    let cusum = residuals
        .traces
        .iter()
        .zip(&nominal.features)
        .map(|(trace, feature)| {
            let healthy_norms = dataset
                .healthy_pass_indices
                .iter()
                .filter_map(|&idx| trace.norms.get(idx).copied())
                .collect::<Vec<_>>();
            let healthy_mean = mean(&healthy_norms).unwrap_or(0.0);
            let healthy_std = sample_std(&healthy_norms, healthy_mean).unwrap_or(0.0);
            let sigma = healthy_std.max(config.epsilon);
            let kappa = if feature.analyzable {
                config.cusum_kappa_sigma_multiplier * sigma
            } else {
                0.0
            };
            let alarm_threshold = if feature.analyzable {
                config.cusum_alarm_sigma_multiplier * sigma
            } else {
                0.0
            };
            let cusum = positive_cusum_series(&trace.norms, healthy_mean, kappa);
            let alarm = cusum
                .iter()
                .map(|value| feature.analyzable && *value > alarm_threshold)
                .collect::<Vec<_>>();

            CusumFeatureTrace {
                feature_index: trace.feature_index,
                feature_name: trace.feature_name.clone(),
                cusum,
                healthy_mean,
                healthy_std,
                kappa,
                alarm_threshold,
                alarm,
            }
        })
        .collect::<Vec<_>>();

    let analyzable_feature_indices = nominal
        .features
        .iter()
        .filter(|feature| feature.analyzable)
        .map(|feature| feature.feature_index)
        .collect::<Vec<_>>();
    let run_energy_series = (0..dataset.labels.len())
        .map(|run_index| {
            if analyzable_feature_indices.is_empty() {
                return 0.0;
            }
            analyzable_feature_indices
                .iter()
                .map(|&feature_index| {
                    let sigma = nominal.features[feature_index]
                        .healthy_std
                        .max(config.epsilon);
                    let residual = residuals.traces[feature_index].residuals[run_index];
                    let z = residual / sigma;
                    z * z
                })
                .sum::<f64>()
                / analyzable_feature_indices.len() as f64
        })
        .collect::<Vec<_>>();
    let healthy_run_energy = dataset
        .healthy_pass_indices
        .iter()
        .filter_map(|&idx| run_energy_series.get(idx).copied())
        .collect::<Vec<_>>();
    let run_energy_healthy_mean = mean(&healthy_run_energy).unwrap_or(0.0);
    let run_energy_healthy_std =
        sample_std(&healthy_run_energy, run_energy_healthy_mean).unwrap_or(0.0);
    let run_energy_threshold = run_energy_healthy_mean
        + config.run_energy_sigma_multiplier * run_energy_healthy_std.max(config.epsilon);
    let run_energy_alarm = run_energy_series
        .iter()
        .map(|value| !analyzable_feature_indices.is_empty() && *value > run_energy_threshold)
        .collect::<Vec<_>>();

    let pca_fdc = compute_pca_fdc(
        dataset,
        nominal,
        residuals,
        config,
        &analyzable_feature_indices,
    );

    BaselineSet {
        ewma,
        cusum,
        run_energy: RunEnergyTrace {
            energy: run_energy_series,
            healthy_mean: run_energy_healthy_mean,
            healthy_std: run_energy_healthy_std,
            threshold: run_energy_threshold,
            analyzable_feature_count: analyzable_feature_indices.len(),
            alarm: run_energy_alarm,
        },
        pca_fdc,
    }
}

pub fn ewma_series(values: &[f64], alpha: f64) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(values.len());
    let mut state = values[0];
    out.push(state);
    for value in &values[1..] {
        state = alpha * *value + (1.0 - alpha) * state;
        out.push(state);
    }
    out
}

pub fn positive_cusum_series(values: &[f64], target_mean: f64, kappa: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(values.len());
    let mut state = 0.0;
    for value in values {
        state = (state + (*value - target_mean - kappa)).max(0.0);
        out.push(state);
    }
    out
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn sample_std(values: &[f64], mean: f64) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let variance = values
        .iter()
        .map(|value| {
            let centered = *value - mean;
            centered * centered
        })
        .sum::<f64>()
        / (values.len() as f64 - 1.0);
    Some(variance.sqrt())
}

fn compute_pca_fdc(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    config: &PipelineConfig,
    analyzable_feature_indices: &[usize],
) -> PcaFdcTrace {
    let run_count = dataset.labels.len();
    let healthy_observation_count = dataset.healthy_pass_indices.len();
    if analyzable_feature_indices.is_empty() || healthy_observation_count < 2 {
        return PcaFdcTrace {
            t2: vec![0.0; run_count],
            spe: vec![0.0; run_count],
            t2_healthy_mean: 0.0,
            t2_healthy_std: 0.0,
            spe_healthy_mean: 0.0,
            spe_healthy_std: 0.0,
            t2_threshold: 0.0,
            spe_threshold: 0.0,
            analyzable_feature_count: analyzable_feature_indices.len(),
            healthy_observation_count,
            retained_components: 0,
            explained_variance_fraction: 0.0,
            target_variance_explained: config.pca_variance_explained,
            alarm: vec![false; run_count],
        };
    }

    let healthy_standardized = dataset
        .healthy_pass_indices
        .iter()
        .map(|&run_index| {
            analyzable_feature_indices
                .iter()
                .map(|&feature_index| {
                    standardized_residual(nominal, residuals, feature_index, run_index, config)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let column_means = column_means(&healthy_standardized);
    let centered_healthy = healthy_standardized
        .iter()
        .map(|row| {
            row.iter()
                .zip(&column_means)
                .map(|(value, mean)| *value - *mean)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let gram = gram_matrix(&centered_healthy);
    let (eigenvalues, eigenvectors) =
        jacobi_eigen_symmetric(&gram, 64 * gram.len().max(1).pow(2), 1.0e-10);
    let mut components = eigenvalues
        .iter()
        .copied()
        .zip(eigenvectors)
        .filter(|(eigenvalue, _)| *eigenvalue > config.epsilon)
        .collect::<Vec<_>>();
    components
        .sort_by(|(lhs, _), (rhs, _)| rhs.partial_cmp(lhs).unwrap_or(std::cmp::Ordering::Equal));

    let total_variance = components.iter().map(|(value, _)| *value).sum::<f64>();
    let mut retained = Vec::new();
    let mut cumulative_variance = 0.0;
    if total_variance > config.epsilon {
        for (eigenvalue, sample_eigenvector) in components {
            cumulative_variance += eigenvalue;
            let loading = sample_to_feature_loading(
                &centered_healthy,
                &sample_eigenvector,
                eigenvalue,
                config.epsilon,
            );
            retained.push((eigenvalue, loading));
            if cumulative_variance / total_variance >= config.pca_variance_explained {
                break;
            }
        }
    }

    let explained_variance_fraction = if total_variance > config.epsilon {
        retained.iter().map(|(value, _)| *value).sum::<f64>() / total_variance
    } else {
        0.0
    };

    let mut t2 = Vec::with_capacity(run_count);
    let mut spe = Vec::with_capacity(run_count);
    for run_index in 0..run_count {
        let centered = analyzable_feature_indices
            .iter()
            .enumerate()
            .map(|(local_index, &feature_index)| {
                standardized_residual(nominal, residuals, feature_index, run_index, config)
                    - column_means[local_index]
            })
            .collect::<Vec<_>>();
        let (t2_value, spe_value) = pca_scores(&centered, &retained, config.epsilon);
        t2.push(t2_value);
        spe.push(spe_value);
    }

    let healthy_t2 = dataset
        .healthy_pass_indices
        .iter()
        .filter_map(|&run_index| t2.get(run_index).copied())
        .collect::<Vec<_>>();
    let healthy_spe = dataset
        .healthy_pass_indices
        .iter()
        .filter_map(|&run_index| spe.get(run_index).copied())
        .collect::<Vec<_>>();
    let t2_healthy_mean = mean(&healthy_t2).unwrap_or(0.0);
    let t2_healthy_std = sample_std(&healthy_t2, t2_healthy_mean).unwrap_or(0.0);
    let spe_healthy_mean = mean(&healthy_spe).unwrap_or(0.0);
    let spe_healthy_std = sample_std(&healthy_spe, spe_healthy_mean).unwrap_or(0.0);
    let t2_threshold =
        t2_healthy_mean + config.pca_t2_sigma_multiplier * t2_healthy_std.max(config.epsilon);
    let spe_threshold =
        spe_healthy_mean + config.pca_spe_sigma_multiplier * spe_healthy_std.max(config.epsilon);
    let alarm = (0..run_count)
        .map(|run_index| t2[run_index] > t2_threshold || spe[run_index] > spe_threshold)
        .collect::<Vec<_>>();

    PcaFdcTrace {
        t2,
        spe,
        t2_healthy_mean,
        t2_healthy_std,
        spe_healthy_mean,
        spe_healthy_std,
        t2_threshold,
        spe_threshold,
        analyzable_feature_count: analyzable_feature_indices.len(),
        healthy_observation_count,
        retained_components: retained.len(),
        explained_variance_fraction,
        target_variance_explained: config.pca_variance_explained,
        alarm,
    }
}

fn standardized_residual(
    nominal: &NominalModel,
    residuals: &ResidualSet,
    feature_index: usize,
    run_index: usize,
    config: &PipelineConfig,
) -> f64 {
    let sigma = nominal.features[feature_index]
        .healthy_std
        .max(config.epsilon);
    residuals.traces[feature_index].residuals[run_index] / sigma
}

fn column_means(matrix: &[Vec<f64>]) -> Vec<f64> {
    if matrix.is_empty() {
        return Vec::new();
    }
    let width = matrix[0].len();
    let mut means = vec![0.0; width];
    for row in matrix {
        for (index, value) in row.iter().enumerate() {
            means[index] += *value;
        }
    }
    for mean in &mut means {
        *mean /= matrix.len() as f64;
    }
    means
}

fn gram_matrix(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = matrix.len();
    let mut gram = vec![vec![0.0; n]; n];
    for row_index in 0..n {
        for col_index in row_index..n {
            let value = dot(&matrix[row_index], &matrix[col_index]) / (n as f64 - 1.0);
            gram[row_index][col_index] = value;
            gram[col_index][row_index] = value;
        }
    }
    gram
}

fn jacobi_eigen_symmetric(
    matrix: &[Vec<f64>],
    max_iterations: usize,
    tolerance: f64,
) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = matrix.len();
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    let mut a = matrix.to_vec();
    let mut v = vec![vec![0.0; n]; n];
    for index in 0..n {
        v[index][index] = 1.0;
    }

    for _ in 0..max_iterations {
        let mut p = 0usize;
        let mut q = 0usize;
        let mut max_off_diagonal = 0.0_f64;
        for row in 0..n {
            for col in (row + 1)..n {
                let magnitude = a[row][col].abs();
                if magnitude > max_off_diagonal {
                    max_off_diagonal = magnitude;
                    p = row;
                    q = col;
                }
            }
        }
        if max_off_diagonal <= tolerance {
            break;
        }

        let theta = 0.5 * (2.0 * a[p][q]).atan2(a[q][q] - a[p][p]);
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        a[p][p] = cos_theta * cos_theta * app - 2.0 * sin_theta * cos_theta * apq
            + sin_theta * sin_theta * aqq;
        a[q][q] = sin_theta * sin_theta * app
            + 2.0 * sin_theta * cos_theta * apq
            + cos_theta * cos_theta * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for k in 0..n {
            if k == p || k == q {
                continue;
            }
            let akp = a[k][p];
            let akq = a[k][q];
            a[k][p] = cos_theta * akp - sin_theta * akq;
            a[p][k] = a[k][p];
            a[k][q] = sin_theta * akp + cos_theta * akq;
            a[q][k] = a[k][q];
        }

        for row in &mut v {
            let vip = row[p];
            let viq = row[q];
            row[p] = cos_theta * vip - sin_theta * viq;
            row[q] = sin_theta * vip + cos_theta * viq;
        }
    }

    let eigenvalues = (0..n).map(|index| a[index][index]).collect::<Vec<_>>();
    let eigenvectors = (0..n)
        .map(|column| v.iter().map(|row| row[column]).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    (eigenvalues, eigenvectors)
}

fn sample_to_feature_loading(
    centered_healthy: &[Vec<f64>],
    sample_eigenvector: &[f64],
    eigenvalue: f64,
    epsilon: f64,
) -> Vec<f64> {
    let singular = (eigenvalue * (centered_healthy.len() as f64 - 1.0))
        .max(epsilon)
        .sqrt();
    let feature_count = centered_healthy.first().map(|row| row.len()).unwrap_or(0);
    let mut loading = vec![0.0; feature_count];
    for (sample_index, row) in centered_healthy.iter().enumerate() {
        let weight = sample_eigenvector[sample_index];
        for (feature_index, value) in row.iter().enumerate() {
            loading[feature_index] += value * weight;
        }
    }
    for value in &mut loading {
        *value /= singular;
    }
    let norm = l2_norm(&loading).max(epsilon);
    for value in &mut loading {
        *value /= norm;
    }
    loading
}

fn pca_scores(
    centered: &[f64],
    retained_components: &[(f64, Vec<f64>)],
    epsilon: f64,
) -> (f64, f64) {
    if retained_components.is_empty() {
        return (0.0, squared_norm(centered));
    }
    let mut reconstructed = vec![0.0; centered.len()];
    let mut t2 = 0.0;
    for (eigenvalue, loading) in retained_components {
        let score = dot(centered, loading);
        t2 += score * score / eigenvalue.max(epsilon);
        for (index, value) in loading.iter().enumerate() {
            reconstructed[index] += score * value;
        }
    }
    let mut residual = vec![0.0; centered.len()];
    for (index, value) in centered.iter().enumerate() {
        residual[index] = *value - reconstructed[index];
    }
    (t2, squared_norm(&residual))
}

fn dot(lhs: &[f64], rhs: &[f64]) -> f64 {
    lhs.iter().zip(rhs).map(|(left, right)| left * right).sum()
}

fn squared_norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum()
}

fn l2_norm(values: &[f64]) -> f64 {
    squared_norm(values).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ewma_series_matches_recursive_definition() {
        let ewma = ewma_series(&[1.0, 3.0, 5.0], 0.5);
        assert_eq!(ewma, vec![1.0, 2.0, 3.5]);
    }

    #[test]
    fn positive_cusum_accumulates_only_above_target_plus_kappa() {
        let cusum = positive_cusum_series(&[1.0, 2.0, 4.0, 3.0, 1.5], 1.0, 0.5);
        assert_eq!(cusum, vec![0.0, 0.5, 3.0, 4.5, 4.5]);
    }

    #[test]
    fn jacobi_eigen_symmetric_recovers_simple_diagonalization() {
        let matrix = vec![vec![3.0, 1.0], vec![1.0, 3.0]];
        let (mut eigenvalues, _eigenvectors) = jacobi_eigen_symmetric(&matrix, 64, 1.0e-12);
        eigenvalues.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap());
        assert!((eigenvalues[0] - 2.0).abs() < 1.0e-6);
        assert!((eigenvalues[1] - 4.0).abs() < 1.0e-6);
    }

    #[test]
    fn pca_scores_are_finite_without_retained_components() {
        let (t2, spe) = pca_scores(&[1.0, -2.0], &[], 1.0e-9);
        assert_eq!(t2, 0.0);
        assert_eq!(spe, 5.0);
    }
}
