# dsfb-starship

High-fidelity Starship-style 6-DoF hypersonic re-entry simulation demonstrating
Drift-Slew Fusion Bootstrap (DSFB) trust-adaptive IMU fusion during plasma blackout.

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

## Build and Run

```bash
cargo run --release -p dsfb-starship
```

Outputs are always written under the workspace-root `output-dsfb-starship/` folder.
Each run creates a fresh timestamped directory (for example `output-dsfb-starship/20260220-143512`)
to prevent overwriting previous results.

## Python / Colab

```bash
cd crates/dsfb-starship
python -m pip install -U "maturin[patchelf]" patchelf
python -m maturin build --release --out target/wheels
python -m pip install -U --force-reinstall target/wheels/dsfb_starship-*.whl
python -c "import dsfb_starship as m; print(m.default_config_json())"
```
