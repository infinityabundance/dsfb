use crate::AddError;

/// Least-squares linear fit with a simple 95% slope confidence interval.
#[derive(Debug, Clone, Copy)]
pub struct LinearFit {
    pub slope: f64,
    pub intercept: f64,
    pub r2: f64,
    pub residual_variance: f64,
    pub mse_resid: f64,
    pub pearson_r: f64,
    pub spearman_rho: f64,
    pub slope_ci_low: f64,
    pub slope_ci_high: f64,
    pub sample_count: usize,
}

/// Residual and ratio diagnostics for the AET-IWLT structural law.
#[derive(Debug, Clone, Copy)]
pub struct StructuralLawDiagnostics {
    pub residual_mean: f64,
    pub residual_std: f64,
    pub residual_skew_approx: f64,
    pub residual_kurtosis_approx: f64,
    pub ratio_mean: f64,
    pub ratio_std: f64,
    pub ratio_min: f64,
    pub ratio_max: f64,
}

/// Fit `ys = slope * xs + intercept` and estimate a 95% confidence interval.
pub fn fit_with_ci(xs: &[f64], ys: &[f64]) -> Result<LinearFit, AddError> {
    if xs.len() != ys.len() {
        return Err(AddError::LengthMismatch {
            context: "structural law fit",
            expected: xs.len(),
            got: ys.len(),
        });
    }

    if xs.len() < 2 {
        return Err(AddError::InvalidConfig(
            "structural law fit requires at least two samples".to_string(),
        ));
    }

    let sample_count = xs.len();
    let x_mean = mean(xs);
    let y_mean = mean(ys);
    let sxx = xs
        .iter()
        .map(|x| {
            let dx = x - x_mean;
            dx * dx
        })
        .sum::<f64>();
    let sxy = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| (x - x_mean) * (y - y_mean))
        .sum::<f64>();

    let slope = if sxx > f64::EPSILON { sxy / sxx } else { 0.0 };
    let intercept = y_mean - slope * x_mean;

    let residuals: Vec<f64> = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| y - (slope * x + intercept))
        .collect();
    let sse = residuals.iter().map(|value| value * value).sum::<f64>();
    let mse_resid = sse / sample_count as f64;
    let residual_variance = if sample_count > 2 {
        sse / (sample_count - 2) as f64
    } else {
        0.0
    };
    let sst = ys
        .iter()
        .map(|y| {
            let dy = y - y_mean;
            dy * dy
        })
        .sum::<f64>();
    let r2 = if sst > f64::EPSILON {
        1.0 - sse / sst
    } else {
        1.0
    };

    // The lambda grid has O(10^2) points, so using a fixed t ~= 2.0 is a
    // reasonable 95% CI approximation for the slope.
    let slope_std_error = if sxx > f64::EPSILON {
        (residual_variance / sxx).sqrt()
    } else {
        0.0
    };
    let ci_half_width = 2.0 * slope_std_error;

    Ok(LinearFit {
        slope,
        intercept,
        r2,
        residual_variance,
        mse_resid,
        pearson_r: correlation(xs, ys),
        spearman_rho: spearman_correlation(xs, ys),
        slope_ci_low: slope - ci_half_width,
        slope_ci_high: slope + ci_half_width,
        sample_count,
    })
}

/// Compute residual and ratio diagnostics for a previously fitted law.
pub fn diagnostics_from_fit(
    xs: &[f64],
    ys: &[f64],
    fit: &LinearFit,
) -> Result<StructuralLawDiagnostics, AddError> {
    if xs.len() != ys.len() {
        return Err(AddError::LengthMismatch {
            context: "structural law diagnostics",
            expected: xs.len(),
            got: ys.len(),
        });
    }

    let residuals: Vec<f64> = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| y - (fit.slope * x + fit.intercept))
        .collect();
    let residual_mean = mean(&residuals);
    let residual_std = stddev(&residuals, residual_mean);
    let residual_skew_approx = standardized_moment(&residuals, residual_mean, residual_std, 3);
    let residual_kurtosis_approx = standardized_moment(&residuals, residual_mean, residual_std, 4);

    let ratios: Vec<f64> = xs
        .iter()
        .zip(ys.iter())
        .filter_map(|(x, y)| {
            if x.abs() <= f64::EPSILON {
                None
            } else {
                let ratio = y / x;
                ratio.is_finite().then_some(ratio)
            }
        })
        .collect();

    let ratio_mean = if ratios.is_empty() {
        0.0
    } else {
        mean(&ratios)
    };
    let ratio_std = if ratios.is_empty() {
        0.0
    } else {
        stddev(&ratios, ratio_mean)
    };
    let ratio_min = ratios.iter().copied().fold(f64::INFINITY, f64::min);
    let ratio_max = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    Ok(StructuralLawDiagnostics {
        residual_mean,
        residual_std,
        residual_skew_approx,
        residual_kurtosis_approx,
        ratio_mean,
        ratio_std,
        ratio_min: if ratios.is_empty() { 0.0 } else { ratio_min },
        ratio_max: if ratios.is_empty() { 0.0 } else { ratio_max },
    })
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn stddev(values: &[f64], mean_value: f64) -> f64 {
    (values
        .iter()
        .map(|value| {
            let delta = value - mean_value;
            delta * delta
        })
        .sum::<f64>()
        / values.len().max(1) as f64)
        .sqrt()
}

fn correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let x_mean = mean(xs);
    let y_mean = mean(ys);
    let covariance = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| (x - x_mean) * (y - y_mean))
        .sum::<f64>();
    let x_scale = xs
        .iter()
        .map(|x| {
            let delta = x - x_mean;
            delta * delta
        })
        .sum::<f64>()
        .sqrt();
    let y_scale = ys
        .iter()
        .map(|y| {
            let delta = y - y_mean;
            delta * delta
        })
        .sum::<f64>()
        .sqrt();

    if x_scale <= f64::EPSILON || y_scale <= f64::EPSILON {
        1.0
    } else {
        covariance / (x_scale * y_scale)
    }
}

fn spearman_correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let x_ranks = average_ranks(xs);
    let y_ranks = average_ranks(ys);
    correlation(&x_ranks, &y_ranks)
}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut ranks = vec![0.0; values.len()];
    let mut start = 0_usize;
    while start < indexed.len() {
        let mut end = start + 1;
        while end < indexed.len() && (indexed[end].1 - indexed[start].1).abs() <= 1.0e-12 {
            end += 1;
        }

        let average_rank = (start + 1 + end) as f64 / 2.0;
        for idx in start..end {
            ranks[indexed[idx].0] = average_rank;
        }

        start = end;
    }

    ranks
}

fn standardized_moment(values: &[f64], mean_value: f64, std_value: f64, power: i32) -> f64 {
    if std_value <= f64::EPSILON {
        return 0.0;
    }

    values
        .iter()
        .map(|value| (value - mean_value).powi(power))
        .sum::<f64>()
        / values.len().max(1) as f64
        / std_value.powi(power)
}
