use anyhow::{bail, Result};
use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lattice {
    pub label: String,
    pub sites: usize,
    pub masses: Vec<f64>,
    pub springs: Vec<f64>,
}

impl Lattice {
    pub fn monatomic_fixed_chain(sites: usize, mass: f64, spring: f64) -> Result<Self> {
        if sites < 2 {
            bail!("lattice must contain at least two sites");
        }
        if mass <= 0.0 || spring <= 0.0 {
            bail!("mass and spring constants must be strictly positive");
        }
        Ok(Self {
            label: "nominal_monatomic_chain".to_string(),
            sites,
            masses: vec![mass; sites],
            springs: vec![spring; sites + 1],
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.masses.len() != self.sites {
            bail!("masses length does not match site count");
        }
        if self.springs.len() != self.sites + 1 {
            bail!("springs must contain sites + 1 edge values");
        }
        if self.masses.iter().any(|mass| *mass <= 0.0) {
            bail!("all masses must be strictly positive");
        }
        if self.springs.iter().any(|spring| *spring <= 0.0) {
            bail!("all springs must be strictly positive");
        }
        Ok(())
    }

    pub fn stiffness_matrix(&self) -> Result<DMatrix<f64>> {
        self.validate()?;
        let mut stiffness = DMatrix::<f64>::zeros(self.sites, self.sites);
        for i in 0..self.sites {
            stiffness[(i, i)] += self.springs[i] + self.springs[i + 1];
            if i + 1 < self.sites {
                let coupling = self.springs[i + 1];
                stiffness[(i, i + 1)] -= coupling;
                stiffness[(i + 1, i)] -= coupling;
            }
        }
        Ok(stiffness)
    }

    pub fn dynamical_matrix(&self) -> Result<DMatrix<f64>> {
        let stiffness = self.stiffness_matrix()?;
        let mut dynamical = DMatrix::<f64>::zeros(self.sites, self.sites);
        for i in 0..self.sites {
            let mass_i = self.masses[i].sqrt();
            for j in 0..self.sites {
                let mass_j = self.masses[j].sqrt();
                dynamical[(i, j)] = stiffness[(i, j)] / (mass_i * mass_j);
            }
        }
        Ok(dynamical)
    }
}
