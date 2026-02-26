use pyo3::prelude::*;
use ndarray::{Array1, Array2, Axis};

#[pyclass]
#[derive(Clone)]
pub struct HretObserver {
    m: usize,              // Number of channels
    g: usize,              // Number of groups
    group_mapping: Array1<usize>,  // g(k) for each k (0..g-1)
    rho: f64,              // Channel forgetting factor
    rho_g: Array1<f64>,    // Group forgetting factors
    beta_k: Array1<f64>,   // Channel sensitivities
    beta_g: Array1<f64>,   // Group sensitivities
    s_k: Array1<f64>,      // Channel envelopes
    s_g: Array1<f64>,      // Group envelopes
    k_k: Array2<f64>,      // Gains: p x M matrix
}

#[pymethods]
impl HretObserver {
    #[new]
    fn new(
        m: usize,
        g: usize,
        group_mapping: Vec<usize>,
        rho: f64,
        rho_g: Vec<f64>,
        beta_k: Vec<f64>,
        beta_g: Vec<f64>,
        k_k: Vec<Vec<f64>>,  // p rows, M columns
    ) -> PyResult<Self> {
        if group_mapping.len() != m {
            return Err(pyo3::exceptions::PyValueError::new_err("group_mapping length must equal m"));
        }
        let p = k_k.len();
        let k_k_flat: Vec<f64> = k_k.into_iter().flatten().collect();
        Ok(HretObserver {
            m,
            g,
            group_mapping: Array1::from(group_mapping),
            rho,
            rho_g: Array1::from(rho_g),
            beta_k: Array1::from(beta_k),
            beta_g: Array1::from(beta_g),
            s_k: Array1::zeros(m),
            s_g: Array1::zeros(g),
            k_k: Array2::from_shape_vec((p, m), k_k_flat).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?,
        })
    }

    fn update(&mut self, r: Vec<f64>) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)> {
        let r_arr = Array1::from(r);
        if r_arr.len() != self.m {
            return Err(pyo3::exceptions::PyValueError::new_err("residuals length must equal m"));
        }

        // Channel envelopes (eq. 8)
        self.s_k = self.rho * &self.s_k + (1.0 - self.rho) * r_arr.mapv(f64::abs);

        // Group envelopes (eq. 11)
        for gg in 0..self.g {
            let group_idx: Vec<usize> = self.group_mapping
            .iter()
            .enumerate()
            .filter_map(|(i, &x)| if x == gg { Some(i) } else { None })
            .collect();

            if !group_idx.is_empty() {
                let group_r = r_arr.select(Axis(0), &group_idx);
                let avg_abs_r = group_r.mapv(f64::abs).sum() / group_r.len() as f64;
                self.s_g[gg] = self.rho_g[gg] * self.s_g[gg] + (1.0 - self.rho_g[gg]) * avg_abs_r;
            }
        }

        // Trusts (eq. 9, 12)
        let ones_k = Array1::<f64>::ones(self.m);
        let ones_g = Array1::<f64>::ones(self.g);

        let w_k = &ones_k / (&ones_k + &self.beta_k * &self.s_k);
        let w_g = &ones_g / (&ones_g + &self.beta_g * &self.s_g);

        // Hierarchical composition (eq. 14-15)
        let w_g_mapped = w_g.select(Axis(0), &self.group_mapping.to_vec());
        let hat_w_k = &w_k * w_g_mapped;
        let sum_hat = hat_w_k.sum();
        let tilde_w_k = if sum_hat > 1e-10 { hat_w_k / sum_hat } else { Array1::zeros(self.m) };

        // Fusion correction (eq. 19): Delta_x = sum(tilde_w_k * K_k * r_k) [vectorized]
        let weighted_r = &tilde_w_k * r_arr;
        let delta_x = self.k_k.dot(&weighted_r);

        // Check convexity (for debugging; per Proposition 1)
        assert!(tilde_w_k.iter().all(|&w| w > 0.0 || w.abs() < 1e-10));
        assert!((tilde_w_k.sum() - 1.0).abs() < 1e-10);

        Ok((delta_x.to_vec(), tilde_w_k.to_vec(), self.s_k.to_vec(), self.s_g.to_vec()))
    }

    fn reset_envelopes(&mut self) {
        self.s_k = Array1::zeros(self.m);
        self.s_g = Array1::zeros(self.g);
    }
}

#[pymodule]
fn dsfb_hret(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HretObserver>()?;
    Ok(())
}
