# dsfb-hret

`dsfb-hret` implements **Hierarchical Residual-Envelope Trust (HRET)**, a deterministic fusion method for combining multiple residual channels when some sensors share a common disturbance source.

In practical terms, this crate:

- accepts per-channel residuals at each update step
- tracks per-channel and per-group residual envelopes over time
- converts those envelopes into trust weights
- downweights channels that belong to a degraded group
- normalizes the result into convex channel weights
- applies a gain matrix to produce a fused state correction vector

Use it when your estimator has several measurement channels, some channels belong to the same subsystem or fault domain, and you want a deterministic alternative to covariance-heavy trust adaptation.

Reference paper:

R. de Beer (2026).  
*Hierarchical Residual-Envelope Trust: A Deterministic Framework for Grouped Multi-Sensor Fusion.*  
https://doi.org/10.5281/zenodo.18783283

## What this crate provides

- A Rust API for stateful HRET updates.
- A Python extension module (`dsfb_hret`) via PyO3.
- Envelope memory for residual magnitude tracking.
- Hierarchical trust computation across channels and groups.
- Convex weight normalization for stable fusion.
- A fused correction output `Delta_x = K * (tilde_w ⊙ r)`.

## What goes in and what comes out

Inputs to `HretObserver::new`:

- `m`: number of residual channels
- `g`: number of channel groups
- `group_mapping`: which group each channel belongs to
- `rho`: channel envelope forgetting factor
- `rho_g`: group envelope forgetting factors
- `beta_k`: channel trust sensitivity parameters
- `beta_g`: group trust sensitivity parameters
- `k_k`: gain matrix with shape `(p, m)`

Input to `update`:

- `residuals`: one scalar residual per channel

Outputs from `update`:

- `delta_x`: fused correction vector of length `p`
- `weights`: normalized per-channel fusion weights
- `s_k`: updated channel envelopes
- `s_g`: updated group envelopes

## Model summary

Given channel residuals `r_k`:

- Channel envelopes: `s_k`
- Group envelopes: `s_g`
- Channel trust: `w_k = 1 / (1 + beta_k * s_k)`
- Group trust: `w_g = 1 / (1 + beta_g * s_g)`
- Hierarchical trust: `hat_w_k = w_k * w_g(group(k))`
- Convex normalization: `tilde_w_k = hat_w_k / sum(hat_w_k)`
- Correction: `Delta_x = K * (tilde_w ⊙ r)`

The main effect is that a single noisy channel is penalized by its own envelope, while a disturbance shared by several related channels also reduces trust at the group level.

## Installation

### Rust

```toml
[dependencies]
dsfb-hret = "0.1.1"
```

### Python (local build)

```bash
python -m pip install maturin
maturin develop --release
```

## Rust usage

```rust
use dsfb_hret::HretObserver;

let mut obs = HretObserver::new(
    3,
    2,
    vec![0, 0, 1],          // group mapping
    0.95,                   // rho
    vec![0.9, 0.85],        // rho_g
    vec![1.0, 1.0, 1.0],    // beta_k
    vec![1.0, 1.0],         // beta_g
    vec![
        vec![1.0, 0.5, 0.5],
        vec![0.0, 1.0, 0.0],
    ],                      // K (p x m)
).unwrap();

let (delta_x, weights, s_k, s_g) = obs.update(vec![0.05, 0.12, 0.30]).unwrap();
assert_eq!(weights.len(), 3);
obs.reset_envelopes();
```

## Python usage

```python
from dsfb_hret import HretObserver

obs = HretObserver(
    m=3,
    g=2,
    group_mapping=[0, 0, 1],
    rho=0.95,
    rho_g=[0.9, 0.85],
    beta_k=[1.0, 1.0, 1.0],
    beta_g=[1.0, 1.0],
    k_k=[[1.0, 0.5, 0.5], [0.0, 1.0, 0.0]],
)

delta_x, weights, s_k, s_g = obs.update([0.05, 0.12, 0.30])
print(delta_x, weights)
```

## Input validation behavior

`HretObserver::new` validates:

- `m > 0`, `g > 0`
- all vector lengths (`group_mapping`, `rho_g`, `beta_k`, `beta_g`, `k_k` rows)
- `group_mapping` values in `0..g`
- `rho` and each `rho_g[i]` in `(0, 1)`
- finite gains/residuals and non-negative `beta_k`, `beta_g`
- non-empty gain matrix

Invalid inputs return `HretError` (Rust) or `ValueError` (Python).

## Notebook validation workflow

The empirical notebook is in:

`hret_hypersonic_validation.ipynb`

It contains:

- toy correlated-fault simulation
- hypersonic re-entry Monte Carlo
- HRET baseline comparisons and sensitivity hooks

## Citation

```bibtex
@misc{debeer2026hret,
  author    = {de Beer, Riaan},
  title     = {Hierarchical Residual-Envelope Trust: A Deterministic Framework for Grouped Multi-Sensor Fusion},
  year      = {2026},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.18783283},
  url       = {https://doi.org/10.5281/zenodo.18783283}
}
```

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18783283.svg)](https://doi.org/10.5281/zenodo.18783283)

## License

Apache-2.0
