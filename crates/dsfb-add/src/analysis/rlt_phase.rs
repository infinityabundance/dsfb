use crate::AddError;

/// Summary of the RLT transport transition for a single lambda sweep.
#[derive(Debug, Clone, Copy)]
pub struct RltPhaseBoundary {
    pub lambda_star: Option<f64>,
    pub lambda_0_1: Option<f64>,
    pub lambda_0_9: Option<f64>,
    pub transition_width: Option<f64>,
    pub max_derivative: Option<f64>,
}

pub fn analyze_rlt_phase_boundary(
    lambda_grid: &[f64],
    expansion_ratio: &[f64],
    escape_rate: &[f64],
) -> Result<RltPhaseBoundary, AddError> {
    if lambda_grid.len() != expansion_ratio.len() {
        return Err(AddError::LengthMismatch {
            context: "rlt phase boundary",
            expected: lambda_grid.len(),
            got: expansion_ratio.len(),
        });
    }
    if lambda_grid.len() != escape_rate.len() {
        return Err(AddError::LengthMismatch {
            context: "rlt phase boundary escape_rate",
            expected: lambda_grid.len(),
            got: escape_rate.len(),
        });
    }

    let lambda_star = first_crossing(lambda_grid, escape_rate, 0.5);
    let lambda_0_1 = first_crossing(lambda_grid, expansion_ratio, 0.1);
    let lambda_0_9 = first_crossing(lambda_grid, expansion_ratio, 0.9);
    let transition_width = match (lambda_0_1, lambda_0_9) {
        (Some(lo), Some(hi)) => Some(hi - lo),
        _ => None,
    };
    let max_derivative = max_derivative(lambda_grid, expansion_ratio);

    Ok(RltPhaseBoundary {
        lambda_star,
        lambda_0_1,
        lambda_0_9,
        transition_width,
        max_derivative,
    })
}

fn first_crossing(lambda_grid: &[f64], values: &[f64], threshold: f64) -> Option<f64> {
    lambda_grid
        .iter()
        .zip(values.iter())
        .find(|(_, value)| **value >= threshold)
        .map(|(lambda, _)| *lambda)
}

fn max_derivative(lambda_grid: &[f64], values: &[f64]) -> Option<f64> {
    let mut max_value: Option<f64> = None;

    for (lambda_pair, value_pair) in lambda_grid.windows(2).zip(values.windows(2)) {
        let delta_lambda = lambda_pair[1] - lambda_pair[0];
        if delta_lambda.abs() <= f64::EPSILON {
            continue;
        }

        let derivative = ((value_pair[1] - value_pair[0]) / delta_lambda).abs();
        max_value = Some(match max_value {
            Some(current) => current.max(derivative),
            None => derivative,
        });
    }

    max_value
}
