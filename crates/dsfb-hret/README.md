# dsfb-hret

**Hierarchical Residual-Envelope Trust (HRET)** — a deterministic extension of DSFB for grouped multi-sensor fusion with correlated disturbance handling.

dsfb-hret implements the HRET algorithm described in:

R. de Beer (2026).
Hierarchical Residual-Envelope Trust: A Deterministic Framework for Grouped Multi-Sensor Fusion.
https://doi.org/10.5281/zenodo.18783283

`dsfb-hret` implements an `HretObserver` that fuses residuals from multiple sensor channels arranged into groups. Each channel and group maintains an exponentially-smoothed residual envelope, and trust weights are computed hierarchically — channel-level weights modulated by group-level weights — before being normalized to form a convex combination. The resulting fusion correction `Δx` can be fed directly into a state observer or navigation filter.

This crate exposes the observer as both a native Rust library (`rlib`) and a Python extension module (`cdylib`) via [PyO3](https://pyo3.rs).

---

## Key concepts

| Symbol | Meaning |
|---|---|
| `m` | Number of sensor channels |
| `g` | Number of channel groups |
| `s_k` | Per-channel residual envelope (eq. 8) |
| `s_g` | Per-group residual envelope (eq. 11) |
| `w_k`, `w_g` | Channel and group trust weights (eq. 9, 12) |
| `w̃_k` | Normalized hierarchical weights (eq. 14–15) |
| `Δx` | Fusion correction vector (eq. 19) |

Trust weights decay as residual envelopes grow, so channels (or whole groups) producing large or correlated residuals are automatically down-weighted without any hard thresholding.

---

## Installation

### Python

Build and install the extension module with [maturin](https://github.com/PyO3/maturin):

```bash
pip install maturin
maturin develop --release
```

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
dsfb-hret = "0.1"
```

---

## Usage

### Python

```python
from dsfb_hret import HretObserver

# 3 channels, 2 groups: channels 0+1 → group 0, channel 2 → group 1
obs = HretObserver(
    m=3,
    g=2,
    group_mapping=[0, 0, 1],
    rho=0.95,           # channel forgetting factor
    rho_g=[0.9, 0.85],  # per-group forgetting factors
    beta_k=[1.0, 1.0, 1.0],   # channel sensitivities
    beta_g=[1.0, 1.0],        # group sensitivities
    k_k=[[1.0, 0.5, 0.5],     # gain matrix (p × m)
         [0.0, 1.0, 0.0]],
)

residuals = [0.05, 0.12, 0.30]
delta_x, weights, s_k, s_g = obs.update(residuals)

print("Fusion correction:", delta_x)
print("Normalized weights:", weights)
print("Channel envelopes:", s_k)
print("Group envelopes:  ", s_g)

# Reset envelopes (e.g. after a mode switch)
obs.reset_envelopes()
```

### Rust

```rust
use dsfb_hret::HretObserver;

let mut obs = HretObserver::new(
    2, 1,
    vec![0, 0],
    0.95,
    vec![0.9],
    vec![1.0, 1.0],
    vec![1.0],
    vec![vec![1.0, 1.0]],
).unwrap();

let (delta_x, weights, s_k, s_g) = obs.update(vec![0.1, 0.2]).unwrap();
```

---

## API

### `HretObserver::new`

```
new(m, g, group_mapping, rho, rho_g, beta_k, beta_g, k_k) -> HretObserver
```

| Parameter | Type | Description |
|---|---|---|
| `m` | `usize` | Number of sensor channels |
| `g` | `usize` | Number of groups |
| `group_mapping` | `[usize; m]` | Group index for each channel (values in `0..g`) |
| `rho` | `f64` | Channel envelope forgetting factor (`0 < rho < 1`) |
| `rho_g` | `[f64; g]` | Per-group envelope forgetting factors |
| `beta_k` | `[f64; m]` | Per-channel trust sensitivity |
| `beta_g` | `[f64; g]` | Per-group trust sensitivity |
| `k_k` | `[[f64; m]; p]` | Observer gain matrix (`p × m`) |

### `HretObserver::update`

```
update(r: [f64; m]) -> (delta_x, tilde_w_k, s_k, s_g)
```

Advances the observer one step with residual vector `r`. Returns the fusion correction `Δx`, the normalized weights `w̃_k`, and the current channel and group envelopes.

### `HretObserver::reset_envelopes`

Zeroes all channel and group envelopes. Useful after fault recovery or filter re-initialization.

---

## Part of the `dsfb` workspace

This crate is one component of the [dsfb](https://github.com/infinityabundance/dsfb) workspace. See the root repository for the full architecture.

---

## Citation

If you use this crate in academic work, please cite the accompanying preprint:

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

---

## License

Apache-2.0
