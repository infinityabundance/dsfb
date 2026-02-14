# dsfb

[![crates.io](https://img.shields.io/crates/v/dsfb.svg)](https://crates.io/crates/dsfb)
[![docs.rs](https://docs.rs/dsfb/badge.svg)](https://docs.rs/dsfb)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/infinityabundance/dsfb/blob/main/LICENSE)

Drift-Slew Fusion Bootstrap (DSFB): a trust-adaptive nonlinear state estimator for tracking position (`phi`), drift (`omega`), and slew (`alpha`) across multiple measurement channels.

## Install

From crates.io:

```toml
[dependencies]
dsfb = "0.1"
```

Before first crates.io release, use Git:

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

## Simulation Example

From workspace root:

```bash
cargo run --release -p dsfb --example drift_impulse
```

Outputs:
- `out/sim.csv`
- metrics summary in console

## Repository

Full documentation, notebooks, and verification scripts:
https://github.com/infinityabundance/dsfb

## Citation

> **de Beer, R.** (2026).  
> *Slew-Aware Trust-Adaptive Nonlinear State Estimation for Oscillatory Systems With Drift and Corruption* (v1.0).  
> Zenodo. https://doi.org/10.5281/zenodo.18642887
