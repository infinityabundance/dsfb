# dsfb-hret

[![crates.io](https://img.shields.io/crates/v/dsfb-hret.svg)](https://crates.io/crates/dsfb-hret)
[![docs.rs](https://docs.rs/dsfb-hret/badge.svg)](https://docs.rs/dsfb-hret)
[![Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Open in Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-hret/hret_empirical_validation.ipynb)

Hierarchical Residual-Envelope Trust (HRET)

Deterministic extension of DSFB for grouped multi-sensor fusion with correlated disturbance suppression.

Paper
https://zenodo.org/records/18783283
DOI: 10.5281/zenodo.18783283

Install

[dependencies]
dsfb-hret = "0.1"

cargo add dsfb-hret

Usage (Rust)

use dsfb_hret::HretObserver;

let mut observer = HretObserver::new(
    10,                             // M channels
    2,                              // G groups
    vec![0,0,0,0,0,1,1,1,1,1],     // group mapping
    0.95,                           // rho
    vec![0.9, 0.9],                 // rho_g
    vec![1.0; 10],                  // beta_k
    vec![1.0, 1.0],                 // beta_g
    vec![vec![0.5; 10]; 1]          // K_k (p=1)
).unwrap();

let residuals = vec![0.1; 10];
let (delta_x, weights, s_k, s_g) = observer.update(residuals).unwrap();

println!("Correction: {:?}", delta_x);
println!("Weights: {:?}", weights);

Python / Colab

Full validation notebook (toy sims, MC re-entry dispersion, baselines):

https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-hret/hret_empirical_validation.ipynb

References

1. de Beer, R. (2026). Hierarchical Residual-Envelope Trust: A Deterministic Framework for Grouped Multi-Sensor Fusion (v1.0). Zenodo. https://doi.org/10.5281/zenodo.18783283

Related DSFB papers:
[1] Deterministic DSFB for Hypersonic Re-Entry Navigation. https://doi.org/10.5281/zenodo.18711897
[2] DSFB Core Framework. https://doi.org/10.5281/zenodo.18706455
[3] Trust-Adaptive Multi-Diagnostic Weighting for Plasma Estimation. https://doi.org/10.5281/zenodo.18644561
[4] Slew-Aware Trust-Adaptive Nonlinear Estimation. https://doi.org/10.5281/zenodo.18642887
