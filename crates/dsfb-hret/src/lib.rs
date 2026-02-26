#![allow(clippy::useless_conversion)] // False positive from PyO3-generated PyResult signature.

use ndarray::{Array1, Array2};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

const WEIGHT_SUM_EPS: f64 = 1e-12;
pub type HretUpdate = (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HretError {
    message: String,
}

impl HretError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for HretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HretError {}

#[derive(Clone, Debug)]
#[pyclass]
pub struct HretObserver {
    m: usize,
    g: usize,
    group_mapping: Array1<usize>,
    group_indices: Vec<Vec<usize>>,
    rho: f64,
    rho_g: Array1<f64>,
    beta_k: Array1<f64>,
    beta_g: Array1<f64>,
    s_k: Array1<f64>,
    s_g: Array1<f64>,
    k_k: Array2<f64>,
}

impl HretObserver {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        m: usize,
        g: usize,
        group_mapping: Vec<usize>,
        rho: f64,
        rho_g: Vec<f64>,
        beta_k: Vec<f64>,
        beta_g: Vec<f64>,
        k_k: Vec<Vec<f64>>,
    ) -> Result<Self, HretError> {
        validate_positive("m", m)?;
        validate_positive("g", g)?;
        validate_len("group_mapping", m, group_mapping.len())?;
        validate_len("rho_g", g, rho_g.len())?;
        validate_len("beta_k", m, beta_k.len())?;
        validate_len("beta_g", g, beta_g.len())?;
        validate_forgetting_factor("rho", rho)?;
        validate_forgetting_factors("rho_g", &rho_g)?;
        validate_non_negative_finite("beta_k", &beta_k)?;
        validate_non_negative_finite("beta_g", &beta_g)?;

        let mut group_indices = vec![Vec::new(); g];
        for (channel_idx, &group_idx) in group_mapping.iter().enumerate() {
            if group_idx >= g {
                return Err(HretError::new(format!(
                    "group_mapping[{channel_idx}] = {group_idx} is out of range 0..{g}",
                )));
            }
            group_indices[group_idx].push(channel_idx);
        }

        if k_k.is_empty() {
            return Err(HretError::new("k_k must contain at least one gain row"));
        }

        let p = k_k.len();
        let mut k_k_flat = Vec::with_capacity(p * m);
        for (row_idx, row) in k_k.into_iter().enumerate() {
            validate_len(&format!("k_k[{row_idx}]"), m, row.len())?;
            for (col_idx, value) in row.into_iter().enumerate() {
                if !value.is_finite() {
                    return Err(HretError::new(format!(
                        "k_k[{row_idx}][{col_idx}] must be finite (got {value})",
                    )));
                }
                k_k_flat.push(value);
            }
        }

        let k_k = Array2::from_shape_vec((p, m), k_k_flat).map_err(|e| {
            HretError::new(format!(
                "failed to build gain matrix with shape ({p}, {m}): {e}",
            ))
        })?;

        Ok(Self {
            m,
            g,
            group_mapping: Array1::from(group_mapping),
            group_indices,
            rho,
            rho_g: Array1::from(rho_g),
            beta_k: Array1::from(beta_k),
            beta_g: Array1::from(beta_g),
            s_k: Array1::zeros(m),
            s_g: Array1::zeros(g),
            k_k,
        })
    }

    pub fn update(&mut self, residuals: Vec<f64>) -> Result<HretUpdate, HretError> {
        validate_len("residuals", self.m, residuals.len())?;
        validate_finite("residuals", &residuals)?;

        let r_arr = Array1::from(residuals);

        // Channel envelopes (eq. 8)
        self.s_k = self.rho * &self.s_k + (1.0 - self.rho) * r_arr.mapv(f64::abs);

        // Group envelopes (eq. 11)
        for (group_idx, channels) in self.group_indices.iter().enumerate() {
            if channels.is_empty() {
                continue;
            }

            let avg_abs_r =
                channels.iter().map(|&i| r_arr[i].abs()).sum::<f64>() / channels.len() as f64;
            self.s_g[group_idx] = self.rho_g[group_idx] * self.s_g[group_idx]
                + (1.0 - self.rho_g[group_idx]) * avg_abs_r;
        }

        // Trusts (eq. 9, 12)
        let w_k =
            Array1::from_iter((0..self.m).map(|i| 1.0 / (1.0 + self.beta_k[i] * self.s_k[i])));
        let w_g =
            Array1::from_iter((0..self.g).map(|i| 1.0 / (1.0 + self.beta_g[i] * self.s_g[i])));

        // Hierarchical composition (eq. 14-15)
        let w_g_mapped =
            Array1::from_iter(self.group_mapping.iter().map(|&group_idx| w_g[group_idx]));
        let hat_w_k = &w_k * &w_g_mapped;
        let sum_hat = hat_w_k.sum();
        let tilde_w_k = if sum_hat > WEIGHT_SUM_EPS {
            hat_w_k / sum_hat
        } else {
            Array1::from_elem(self.m, 1.0 / self.m as f64)
        };

        // Fusion correction (eq. 19): Delta_x = K * (tilde_w ⊙ r)
        let weighted_r = &tilde_w_k * &r_arr;
        let delta_x = self.k_k.dot(&weighted_r);

        debug_assert!(tilde_w_k.iter().all(|&w| w >= -1e-12));
        debug_assert!((tilde_w_k.sum() - 1.0).abs() < 1e-8);

        Ok((
            delta_x.to_vec(),
            tilde_w_k.to_vec(),
            self.s_k.to_vec(),
            self.s_g.to_vec(),
        ))
    }

    pub fn reset_envelopes(&mut self) {
        self.s_k.fill(0.0);
        self.s_g.fill(0.0);
    }

    pub fn channel_count(&self) -> usize {
        self.m
    }

    pub fn group_count(&self) -> usize {
        self.g
    }

    pub fn group_mapping_vec(&self) -> Vec<usize> {
        self.group_mapping.to_vec()
    }
}

#[pymethods]
impl HretObserver {
    #[new]
    #[pyo3(signature = (m, g, group_mapping, rho, rho_g, beta_k, beta_g, k_k))]
    #[allow(clippy::too_many_arguments)]
    fn py_new(
        m: usize,
        g: usize,
        group_mapping: Vec<usize>,
        rho: f64,
        rho_g: Vec<f64>,
        beta_k: Vec<f64>,
        beta_g: Vec<f64>,
        k_k: Vec<Vec<f64>>,
    ) -> PyResult<Self> {
        Self::new(m, g, group_mapping, rho, rho_g, beta_k, beta_g, k_k)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    }

    #[pyo3(name = "update")]
    #[allow(clippy::useless_conversion)]
    fn py_update(&mut self, residuals: Vec<f64>) -> PyResult<HretUpdate> {
        self.update(residuals)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    }

    #[pyo3(name = "reset_envelopes")]
    fn py_reset_envelopes(&mut self) {
        self.reset_envelopes();
    }

    #[getter]
    fn m(&self) -> usize {
        self.channel_count()
    }

    #[getter]
    fn g(&self) -> usize {
        self.group_count()
    }

    #[getter]
    fn group_mapping(&self) -> Vec<usize> {
        self.group_mapping_vec()
    }

    fn __repr__(&self) -> String {
        format!(
            "HretObserver(m={}, g={}, p={})",
            self.m,
            self.g,
            self.k_k.nrows()
        )
    }
}

fn validate_positive(field: &str, value: usize) -> Result<(), HretError> {
    if value == 0 {
        return Err(HretError::new(format!("{field} must be > 0 (got 0)")));
    }
    Ok(())
}

fn validate_len(field: &str, expected: usize, got: usize) -> Result<(), HretError> {
    if expected != got {
        return Err(HretError::new(format!(
            "{field} length mismatch: expected {expected}, got {got}",
        )));
    }
    Ok(())
}

fn validate_forgetting_factor(field: &str, value: f64) -> Result<(), HretError> {
    if !value.is_finite() || value <= 0.0 || value >= 1.0 {
        return Err(HretError::new(format!(
            "{field} must be finite and in (0, 1); got {value}",
        )));
    }
    Ok(())
}

fn validate_forgetting_factors(field: &str, values: &[f64]) -> Result<(), HretError> {
    for (idx, value) in values.iter().copied().enumerate() {
        if !value.is_finite() || value <= 0.0 || value >= 1.0 {
            return Err(HretError::new(format!(
                "{field}[{idx}] must be finite and in (0, 1); got {value}",
            )));
        }
    }
    Ok(())
}

fn validate_non_negative_finite(field: &str, values: &[f64]) -> Result<(), HretError> {
    for (idx, value) in values.iter().copied().enumerate() {
        if !value.is_finite() || value < 0.0 {
            return Err(HretError::new(format!(
                "{field}[{idx}] must be finite and >= 0; got {value}",
            )));
        }
    }
    Ok(())
}

fn validate_finite(field: &str, values: &[f64]) -> Result<(), HretError> {
    for (idx, value) in values.iter().copied().enumerate() {
        if !value.is_finite() {
            return Err(HretError::new(format!(
                "{field}[{idx}] must be finite; got {value}",
            )));
        }
    }
    Ok(())
}

#[pymodule]
fn dsfb_hret(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HretObserver>()?;
    Ok(())
}

#[cfg(test)]
mod tests;
