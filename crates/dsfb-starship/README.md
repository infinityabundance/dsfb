# dsfb-starship

High-fidelity Starship-style 6-DoF hypersonic re-entry simulation demonstrating
Drift-Slew Fusion Bootstrap (DSFB) trust-adaptive IMU fusion during plasma blackout.

This crate is a deterministic re-entry simulation and analysis package. It models a Starship-class vehicle descending through hypersonic flight, injects faults into a redundant IMU set, and compares three navigation stacks:

- pure inertial propagation
- a simple GNSS-aided EKF baseline
- DSFB-based IMU fusion with GNSS aiding outside blackout

Use it when you want a reproducible end-to-end demo of DSFB under a harsh navigation scenario rather than a general-purpose flight dynamics library.

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-starship/starship_reentry_demo.ipynb)

> **Disclaimer (Important):** This is an illustrative simulation using representative physics and parameters. It is not calibrated to proprietary SpaceX models or actual flight data. Absolute performance numbers are not predictive of any specific vehicle.

## Citations

- de Beer, R. (2026). *Deterministic Drift--Slew Fusion Bootstrap for Navigation During Plasma Blackout in Hypersonic Re-Entry Vehicles (v1.0)*. Zenodo. DOI: [10.5281/zenodo.18711897](https://doi.org/10.5281/zenodo.18711897)
- de Beer, R. (2026c). *Drift--Slew Fusion Bootstrap: A Deterministic Residual-Based State Correction Framework*. Zenodo. DOI: [10.5281/zenodo.18706455](https://doi.org/10.5281/zenodo.18706455)
- de Beer, R. (2026a). *Slew-Aware Trust-Adaptive Nonlinear State Estimation for Oscillatory Systems With Drift and Corruption*. Zenodo. DOI: [10.5281/zenodo.18642887](https://doi.org/10.5281/zenodo.18642887)
- de Beer, R. (2026b). *Trust-Adaptive Multi-Diagnostic Weighting for Magnetically Confined Plasma State Estimation*. Zenodo. DOI: [10.5281/zenodo.18644561](https://doi.org/10.5281/zenodo.18644561)
- DSFB repository: [https://github.com/infinityabundance/dsfb](https://github.com/infinityabundance/dsfb)

## Features

- 6-DoF rigid-body translational and rotational dynamics
- Exponential atmosphere + altitude-dependent gravity
- Starship-like aerodynamic coefficients and heat-shield heating model
- Plasma blackout between configurable altitudes (default: 80 km to 40 km)
- Redundant IMU model with thermal drift ramp, Gaussian noise, and abrupt slew faults
- Three estimators:
  - Pure inertial baseline
  - Simple GNSS-aided EKF baseline
  - DSFB fusion layer + GNSS aiding outside blackout
- Output artifacts:
  - `starship_timeseries.csv`
  - `starship_summary.json`
  - PNG plots (altitude, log-scale position error, DSFB trust)
- Python bindings via PyO3, installable from wheels built by maturin

## What goes in and what comes out

Inputs:

- `SimConfig`: time step, horizon, blackout altitudes, entry conditions, DSFB trust parameters, and RNG seed
- optional output directory for `run_simulation`

Outputs:

- timestamped run directory under `output-dsfb-starship/`
- `starship_timeseries.csv` with truth, baseline, DSFB, and trust traces
- `starship_summary.json` with run configuration and aggregate metrics
- three PNG plots for altitude, position error, and DSFB trust
- Rust and Python APIs for running the same deterministic scenario programmatically

## Why this matters for reusable vehicles

The plasma blackout phase is one of the most demanding windows in hypersonic re-entry: several minutes of near-total loss of GPS and RF communication while the vehicle experiences extreme thermal gradients, aerodynamic transients, and potential sensor slew.

During this critical period, navigation must remain safe and bounded without external fixes. DSFB provides a deterministic, trust-adaptive solution that:

- Explicitly separates slow thermal drift from abrupt slew events
- Applies provably bounded corrections to prevent unsafe state jumps
- Gracefully attenuates faulty IMU channels while preserving trust in healthy ones
- Recovers quickly when high-quality measurements (e.g., Starlink reacquisition) become available again

By delivering predictable behavior when traditional filters are most vulnerable, DSFB can help improve landing precision, reduce refurbishment risk, and support faster turnaround for fully reusable vehicles like Starship.

## Build and Run

```bash
cargo run --release -p dsfb-starship
```

Outputs are always written under the workspace-root `output-dsfb-starship/` folder.
Each run creates a fresh timestamped directory (for example `output-dsfb-starship/20260220-143512`)
to prevent overwriting previous results.

Programmatically, the main entry point is `run_simulation(&SimConfig, output_dir)`, which validates the configuration, runs the scenario, writes artifacts, and returns a summary struct.

## Python / Colab

```bash
cd crates/dsfb-starship
python -m pip install -U maturin
# Colab/Linux: ensure patchelf is available for maturin wheel repair
apt-get -qq update && apt-get -qq install -y patchelf
python -m maturin build --release --out target/wheels
python -m pip install -U --force-reinstall target/wheels/dsfb_starship-*.whl
python -c "import dsfb_starship as m; print(m.default_config_json())"
```
