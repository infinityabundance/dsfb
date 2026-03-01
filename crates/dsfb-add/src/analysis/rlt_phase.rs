use crate::AddError;

#[derive(Debug, Clone, Copy)]
pub struct RltPhaseBoundary {
    pub lambda_star: Option<f64>,
    pub lambda_0_1: Option<f64>,
    pub lambda_0_9: Option<f64>,
    pub transition_width: Option<f64>,
}

pub fn analyze_rlt_phase_boundary(
    lambda_grid: &[f64],
    expansion_ratio: &[f64],
) -> Result<RltPhaseBoundary, AddError> {
    if lambda_grid.len() != expansion_ratio.len() {
        return Err(AddError::LengthMismatch {
            context: "rlt phase boundary",
            expected: lambda_grid.len(),
            got: expansion_ratio.len(),
        });
    }

    let lambda_star = first_crossing(lambda_grid, expansion_ratio, 0.5);
    let lambda_0_1 = first_crossing(lambda_grid, expansion_ratio, 0.1);
    let lambda_0_9 = first_crossing(lambda_grid, expansion_ratio, 0.9);
    let transition_width = match (lambda_0_1, lambda_0_9) {
        (Some(lo), Some(hi)) => Some(hi - lo),
        _ => None,
    };

    Ok(RltPhaseBoundary {
        lambda_star,
        lambda_0_1,
        lambda_0_9,
        transition_width,
    })
}

fn first_crossing(lambda_grid: &[f64], values: &[f64], threshold: f64) -> Option<f64> {
    lambda_grid
        .iter()
        .zip(values.iter())
        .find(|(_, value)| **value >= threshold)
        .map(|(lambda, _)| *lambda)
}
