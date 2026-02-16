# DSFB - Drift-Slew Fusion Bootstrap

[![crates.io](https://img.shields.io/crates/v/dsfb.svg)](https://crates.io/crates/dsfb)
[![docs.rs](https://docs.rs/dsfb/badge.svg)](https://docs.rs/dsfb)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![DOI: Slew-Aware DSFB Paper](https://zenodo.org/badge/DOI/10.5281/zenodo.18642887.svg)](https://doi.org/10.5281/zenodo.18642887)
[![DOI: Fusion Diagnostics Paper](https://zenodo.org/badge/DOI/10.5281/zenodo.18644561.svg)](https://doi.org/10.5281/zenodo.18644561)

---

`DSFB Simulation Notebook:`
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb/dsfb_simulation.ipynb)

`Fusion Bench Figures Notebook:`
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-fusion-bench/dsfb_fusion_figures.ipynb)

`High-Rate Estimation Trust Figures Notebook:`
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-lcss-hret/dsfb_lcss_hret_figures.ipynb)

`HRET Correlated Group Figures Notebook:`
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-lcss-hret/dsfb-lcss-hret-correlated.ipynb)

---

A Rust implementation of the Drift-Slew Fusion Bootstrap (DSFB) algorithm for trust-adaptive nonlinear state estimation.

## Workspace Crates

This repository contains three separate crates for different paper workflows:

- `dsfb`:
  crate for the DSFB estimator itself
  workspace path: `crates/dsfb`
  crates.io: https://crates.io/crates/dsfb
  docs.rs: https://docs.rs/dsfb
- `dsfb-fusion-bench`:
  standalone synthetic benchmarking + plotting-data generator crate
  workspace path: `crates/dsfb-fusion-bench`
  local README: `crates/dsfb-fusion-bench/README.md`
- `dsfb-lcss-hret`:
  standalone IEEE L-CSS high-rate estimation trust analysis benchmarking crate
  workspace path: `crates/dsfb-lcss-hret`
  local README: `crates/dsfb-lcss-hret/README.md`
  isolated crate (not part of workspace) - compiles independently

Observer-theoretic framework for slew-aware trust-adaptive oscillatory state estimation under bounded disturbances.

## Overview

DSFB is a state estimation algorithm that tracks position (φ), velocity/drift (ω), and acceleration/slew (α) across multiple measurement channels with adaptive trust weighting. The algorithm dynamically adjusts trust weights for each channel based on exponential moving averages (EMA) of residuals, making it robust to impulse disturbances and measurement anomalies. 

### Key Features

- **Trust-adaptive fusion**: Automatically down-weights unreliable measurement channels
- **Drift-slew dynamics**: Tracks position, velocity (drift), and acceleration (slew)
- **O(M) complexity**: Efficient per-step computation for M channels
- **Deterministic**: Reproducible results with seed control
- **Pure Rust**: No external C dependencies

## Algorithm

The DSFB algorithm operates in discrete time with the following steps:

### Predict
```
φ⁻ = φ + ω·dt
ω⁻ = ω + α·dt
α⁻ = α
```

### Update Trust Weights
```
rₖ = yₖ - h(φ⁻)              # Residuals
sₖ = ρ·sₖ + (1-ρ)·|rₖ|       # EMA residuals
w̃ₖ = 1 / (σ₀ + sₖ)           # Raw trust weights
wₖ = w̃ₖ / Σⱼw̃ⱼ              # Normalized weights
```

### Correct
```
R = Σₖ wₖ·rₖ                 # Aggregate residual
φ = φ⁻ + k_φ·R
ω = ω⁻ + k_ω·R
α = α⁻ + k_α·R
```

## Installation

From crates.io:

```toml
[dependencies]
dsfb = "0.1.1"
```

To track unreleased changes, use Git:

```toml
[dependencies]
dsfb = { git = "https://github.com/infinityabundance/dsfb", branch = "main" }
```

Or install from source:

```bash
git clone https://github.com/infinityabundance/dsfb
cd dsfb
cargo build --release -p dsfb
```

## Usage

### Basic Example

```rust
use dsfb::{DsfbObserver, DsfbParams, DsfbState};

// Configure parameters
let params = DsfbParams::new(
    0.5,  // k_phi: position gain
    0.1,  // k_omega: velocity gain
    0.01, // k_alpha: acceleration gain
    0.95, // rho: EMA smoothing (0 < ρ < 1)
    0.1,  // sigma0: trust softness
);

// Create observer with 2 channels
let mut observer = DsfbObserver::new(params, 2);

// Initialize state
observer.init(DsfbState::new(0.0, 0.5, 0.0));

// Process measurements
let dt = 0.01;
let measurements = vec![1.0, 1.05];
let state = observer.step(&measurements, dt);

println!("φ={}, ω={}, α={}", state.phi, state.omega, state.alpha);

// Check trust weights
let w0 = observer.trust_weight(0);
let w1 = observer.trust_weight(1);
println!("Trust weights: {}, {}", w0, w1);
```

### Running the Simulation

The repository includes a complete simulation harness that demonstrates DSFB performance against baseline methods:

```bash
# Run simulation and generate CSV output
cargo run --release -p dsfb --example drift_impulse

# Output will be written to: output-dsfb/<timestamp>/sim-dsfb.csv
```

The simulation compares three methods:
1. **Mean Fusion**: Simple average of measurements
2. **Freq-Only Observer**: Observer without acceleration state
3. **DSFB Observer**: Full drift-slew fusion with trust adaptation

Simulation scenario:
- Two measurement channels
- Channel 2 has linear drift and impulse disturbance
- Impulse occurs at t=3.0s for 1.0s duration
- Metrics: RMS error, peak error during impulse, recovery time

## Fusion Benchmark Crate (Separate Artifact)

For the separate fusion diagnostics benchmarking artifact, use `dsfb-fusion-bench`:

```bash
cargo run --release -p dsfb-fusion-bench -- --run-default
cargo run --release -p dsfb-fusion-bench -- --run-sweep
```

Outputs are written under:
- `output-dsfb-fusion-bench/<timestamp>/` (from workspace root by default)
- includes `summary.csv`, `heatmap.csv`, `trajectories.csv`, and `sim-dsfb-fusion-bench.csv`

Companion notebook for figures:
- `crates/dsfb-fusion-bench/dsfb_fusion_figures.ipynb`

## High-Rate Estimation Trust Crate (Separate Artifact)

For the high-rate estimation trust analysis, use the standalone `dsfb-lcss-hret` crate:

```bash
# Run default benchmark configuration
cargo run --release --manifest-path crates/dsfb-lcss-hret/Cargo.toml -- --run-default

# Run parameter sweep
cargo run --release --manifest-path crates/dsfb-lcss-hret/Cargo.toml -- --run-sweep

# Customize parameters
cargo run --release --manifest-path crates/dsfb-lcss-hret/Cargo.toml -- \
  --run-default \
  --num-runs 500 \
  --time-steps 2000 \
  --seed 123
```

Outputs are written under:
- `output-dsfb-lcss-hret/<timestamp>/` (timestamped directories)
- includes `summary.csv`, `trajectories.csv`, and `heatmap.csv`

Companion notebook for IEEE-formatted figures:
- `crates/dsfb-lcss-hret/dsfb_lcss_hret_figures.ipynb`

Note: This crate is intentionally isolated (not part of the workspace) and compiles independently.

### Paper Verification Workflow

For a single-command run that builds the example, writes the simulation CSV, computes metrics from CSV, and generates plots:

```bash
./scripts/run_drift_impulse_verify.sh
```

This produces:
- `output-dsfb/<timestamp>/sim-dsfb.csv`
- `output-dsfb/<timestamp>/analysis/metrics.json`
- `output-dsfb/<timestamp>/analysis/metrics_summary.csv`
- `output-dsfb/<timestamp>/analysis/phi_estimates.png`
- `output-dsfb/<timestamp>/analysis/estimation_errors.png`
- `output-dsfb/<timestamp>/analysis/trust_weight_and_ema.png`

Recovery time is computed as steps after the impulse window until error stays below a near-baseline threshold for `hold_steps` consecutive samples. The threshold is:
- `baseline_rms * baseline_factor + baseline_margin`
- `baseline_rms` is RMS error before the impulse start

You can tune this definition:

```bash
./scripts/run_drift_impulse_verify.sh \
  --hold-steps 10 \
  --baseline-factor 1.10 \
  --baseline-margin 0.005
```

To verify against values reported in your paper, pass expected metrics and tolerance:

```bash
./scripts/run_drift_impulse_verify.sh \
  --expect-rms-mean <value> \
  --expect-rms-freqonly <value> \
  --expect-rms-dsfb <value> \
  --expect-peak-mean <value> \
  --expect-peak-freqonly <value> \
  --expect-peak-dsfb <value> \
  --expect-recovery-mean <value> \
  --expect-recovery-freqonly <value> \
  --expect-recovery-dsfb <value> \
  --tolerance 1e-3
```

### Visualizing Results

Use the Jupyter notebook to visualize simulation results:

```bash
# First, run the simulation to generate data
cargo run --release -p dsfb --example drift_impulse

# Then open the notebook
jupyter notebook crates/dsfb/dsfb_simulation.ipynb
```

Or use Google Colab:
`DSFB Simulation Notebook:`
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb/dsfb_simulation.ipynb)

`Fusion Bench Figures Notebook:`
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-fusion-bench/dsfb_fusion_figures.ipynb)

`IEEE L-CSS High-Rate Estimation Trust Figures Notebook:`
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-lcss-hret/dsfb_lcss_hret_figures.ipynb)

In Google Colab, click `Run all` first. If a notebook asks for CSV files, click `Browse` in the file picker and upload the required CSVs from your local machine.

The notebook displays:
- Position estimates vs ground truth
- Error curves for all methods
- Trust weight adaptation over time
- EMA residual tracking
- Performance metrics table

## API Reference

### `DsfbState`

State vector for the observer:
- `phi: f64` - Position/phase
- `omega: f64` - Velocity/frequency (drift)
- `alpha: f64` - Acceleration/slew

### `DsfbParams`

Algorithm parameters:
- `k_phi: f64` - Position correction gain
- `k_omega: f64` - Velocity correction gain
- `k_alpha: f64` - Acceleration correction gain
- `rho: f64` - EMA smoothing factor (0 < ρ < 1, typical: 0.95)
- `sigma0: f64` - Trust softness parameter (typical: 0.1)

### `DsfbObserver`

Main observer struct:
- `new(params: DsfbParams, channels: usize) -> Self` - Create observer
- `init(&mut self, initial_state: DsfbState)` - Set initial state
- `step(&mut self, measurements: &[f64], dt: f64) -> DsfbState` - Process one time step
- `state(&self) -> DsfbState` - Get current state
- `trust_weight(&self, channel: usize) -> f64` - Get trust weight for channel
- `ema_residual(&self, channel: usize) -> f64` - Get EMA residual for channel

## Testing

```bash
# Run unit tests
cargo test -p dsfb

# Run with verbose output
cargo test -p dsfb -- --nocapture

# Run specific test
cargo test -p dsfb test_observer_creation
```

## Release Checklist

Before publishing a new version to crates.io:

```bash
# Verify package contents
cargo package --list -p dsfb

# Verify publishability without uploading
cargo publish --dry-run -p dsfb
```

Then tag and publish explicitly:

```bash
# Optional: tag release
git tag v0.1.0
git push origin v0.1.0

# Publish (intentional/manual step)
cargo publish -p dsfb
```

## Performance

- **Time complexity**: O(M) per step for M channels
- **Space complexity**: O(M) for channel statistics
- **Deterministic**: Given fixed seed, produces identical results

## Citation

If you use DSFB in your research, please cite:

> **de Beer, R.** (2026).  
> *Slew-Aware Trust-Adaptive Nonlinear State Estimation for Oscillatory Systems With Drift and Corruption* (v1.0).  
> Zenodo. https://doi.org/10.5281/zenodo.18642887

> **de Beer, R.** (2026).  
> *Trust-Adaptive Multi-Diagnostic Weighting for Magnetically Confined Plasma State Estimation* (v1.0).  
> Zenodo. https://doi.org/10.5281/zenodo.18644561

```bibtex
@software{dsfb2026,
  author = {de Beer, Riaan},
  title = {DSFB: Drift-Slew Fusion Bootstrap},
  year = {2026},
  url = {https://github.com/infinityabundance/dsfb}
}
```

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## References

For theoretical background, see:
- [Slew-Aware Trust-Adaptive Nonlinear State Estimation](docs/Slew-Aware%20Trust-Adaptive%20Nonlinear%20State%20Estimation.pdf)

## Repository Structure

```
dsfb/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── dsfb/
│   │   ├── Cargo.toml      # Publishable estimator crate
│   │   ├── src/
│   │   │   ├── lib.rs      # Public API
│   │   │   ├── observer.rs # DSFB observer implementation
│   │   │   ├── state.rs    # State representation
│   │   │   ├── params.rs   # Parameters
│   │   │   ├── trust.rs    # Trust weight calculations
│   │   │   └── sim.rs      # Simulation harness
│   │   ├── examples/
│   │   │   └── drift_impulse.rs
│   │   ├── dsfb_simulation.ipynb  # DSFB simulation notebook
│   │   └── sim.csv         # Sample CSV for notebook fallback
│   ├── dsfb-fusion-bench/
│       ├── Cargo.toml      # Benchmarking crate
│       ├── src/            # Simulation + methods + metrics + IO
│       ├── configs/        # Reproducible run configs
│       └── dsfb_fusion_figures.ipynb
│   └── dsfb-lcss-hret/
│       ├── Cargo.toml      # IEEE L-CSS benchmarking crate (isolated)
│       ├── src/
│       │   └── main.rs     # CLI and benchmark logic
│       ├── README.md       # Crate documentation
│       └── dsfb_lcss_hret_figures.ipynb
├── output-dsfb/            # Timestamped simulation outputs
├── output-dsfb-fusion-bench/  # Timestamped benchmark outputs
├── output-dsfb-lcss-hret/  # Timestamped IEEE L-CSS outputs
├── docs/                   # Documentation
├── README.md               # This file
├── LICENSE                 # Apache 2.0 license
└── CITATION.cff            # Citation metadata
```
