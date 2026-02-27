# dsfb

[![crates.io](https://img.shields.io/crates/v/dsfb.svg)](https://crates.io/crates/dsfb)
[![docs.rs](https://docs.rs/dsfb/badge.svg)](https://docs.rs/dsfb)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/infinityabundance/dsfb/blob/main/LICENSE)
[![DSFB Notebook In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb/dsfb_simulation.ipynb)

Drift-Slew Fusion Bootstrap (DSFB): a trust-adaptive nonlinear state estimator for tracking position (`phi`), drift (`omega`), and slew (`alpha`) across multiple measurement channels.

In practical terms, this crate:

- takes one scalar measurement per channel at each time step
- predicts a three-state model `phi`, `omega`, `alpha`
- computes per-channel residuals against the predicted position
- tracks an exponential moving average of residual magnitude
- converts those residuals into normalized trust weights
- applies a bounded residual correction to the state estimate

Use `dsfb` when you want a small deterministic observer that can keep fusing redundant scalar channels while automatically downweighting channels whose residuals stop behaving like the rest.

## What goes in and what comes out

Inputs:

- `DsfbParams`: observer gains and trust parameters
- channel count: number of scalar measurement channels
- initial state: `DsfbState { phi, omega, alpha }`
- per-step data: `&[f64]` measurements plus `dt`

Outputs:

- corrected `DsfbState`
- per-channel trust weights through `trust_stats()` / `trust_weight()`
- per-channel residual-envelope state through `ema_residual()`

## Install

From crates.io:

```toml
[dependencies]
dsfb = "0.1.2"
```

To track unreleased changes, use Git:

```toml
[dependencies]
dsfb = { git = "https://github.com/infinityabundance/dsfb", branch = "main" }
```

## Quick Start

```rust
use dsfb::{DsfbObserver, DsfbParams, DsfbState};

let params = DsfbParams::new(0.5, 0.1, 0.01, 0.95, 0.1);
let mut observer = DsfbObserver::new(params, 2);
observer.init(DsfbState::new(0.0, 0.5, 0.0));

let dt = 0.01;
let measurements = [1.0, 1.05];
let state = observer.step(&measurements, dt);

println!("phi={}, omega={}, alpha={}", state.phi, state.omega, state.alpha);
```

At each call to `step`, DSFB predicts the next state, compares all channels to that prediction, and uses trust-weighted residual aggregation to decide how much the observer should move.

## Simulation Example

From workspace root:

```bash
cargo run --release -p dsfb --example drift_impulse
```

Outputs:
- `output-dsfb/<timestamp>/sim-dsfb.csv`
- metrics summary in console

Google Colab note:
- Click `Run all` first.
- If prompted for input data, click `Browse` in the file picker and upload `sim-dsfb.csv` (or your generated CSV file).

## Repository

Full documentation, notebooks, and verification scripts:
https://github.com/infinityabundance/dsfb

## Separate Crate In This Repo

For the separate synthetic benchmarking package used for fusion diagnostics paper workflows, see:
- `crates/dsfb-fusion-bench`
- `crates/dsfb-fusion-bench/README.md`

## Citation

> **de Beer, R.** (2026).  
> *Slew-Aware Trust-Adaptive Nonlinear State Estimation for Oscillatory Systems With Drift and Corruption* (v1.0).  
> Zenodo. https://doi.org/10.5281/zenodo.18642887
