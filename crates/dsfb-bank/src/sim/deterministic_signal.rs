#[derive(Debug, Clone, PartialEq)]
pub struct SignalTrace {
    pub id: String,
    pub values: Vec<f64>,
}

impl SignalTrace {
    pub fn new(id: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            id: id.into(),
            values,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

pub fn constant_signal(id: &str, value: f64, len: usize) -> SignalTrace {
    SignalTrace::new(id, vec![value; len])
}

pub fn linear_signal(id: &str, intercept: f64, slope: f64, len: usize) -> SignalTrace {
    let values = (0..len)
        .map(|index| intercept + slope * index as f64)
        .collect();
    SignalTrace::new(id, values)
}

pub fn step_signal(
    id: &str,
    before: f64,
    after: f64,
    step_index: usize,
    len: usize,
) -> SignalTrace {
    let values = (0..len)
        .map(|index| if index < step_index { before } else { after })
        .collect();
    SignalTrace::new(id, values)
}

pub fn spike_signal(
    id: &str,
    baseline: f64,
    amplitude: f64,
    spike_index: usize,
    len: usize,
) -> SignalTrace {
    let mut values = vec![baseline; len];
    if spike_index < len {
        values[spike_index] = baseline + amplitude;
    }
    SignalTrace::new(id, values)
}

pub fn periodic_signal(id: &str, pattern: &[f64], cycles: usize) -> SignalTrace {
    let mut values = Vec::with_capacity(pattern.len() * cycles);
    for _ in 0..cycles {
        values.extend_from_slice(pattern);
    }
    SignalTrace::new(id, values)
}

pub fn first_differences(values: &[f64]) -> Vec<f64> {
    let mut diffs = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let diff = if index == 0 {
            0.0
        } else {
            *value - values[index - 1]
        };
        diffs.push(diff);
    }
    diffs
}

pub fn second_differences(values: &[f64]) -> Vec<f64> {
    let first = first_differences(values);
    first_differences(&first)
}

pub fn residuals_against_reference(values: &[f64], reference: impl Fn(f64) -> f64) -> Vec<f64> {
    let mut residuals = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let residual = if index == 0 {
            0.0
        } else {
            *value - reference(values[index - 1])
        };
        residuals.push(residual);
    }
    residuals
}
