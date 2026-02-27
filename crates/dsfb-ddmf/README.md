# dsfb-ddmf

`dsfb-ddmf` implements a **Deterministic Disturbance Modeling Framework (DDMF)** for residual-envelope fusion systems built on top of the core `dsfb` crate.

In practical terms, this crate:

- generates deterministic disturbance sequences for pointwise-bounded, drift-type, slew-rate-bounded, impulsive, and persistent-elevated regimes
- runs the residual-envelope recursion `s[n+1] = rho s[n] + (1-rho)|r[n]|`
- maps envelope state into deterministic trust weights `w[n] = 1 / (1 + beta s[n])`
- provides single-channel and light multi-channel simulations
- runs seeded Monte Carlo disturbance sweeps with an explicit default batch size of `x360`
- writes reproducible CSV and JSON outputs for Colab and offline analysis

Use `dsfb-ddmf` when you want disturbance-side analysis for DSFB-style residual-envelope systems rather than a full observer or application demo.

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-ddmf/dsfb_ddmf_colab_latest.ipynb)

## Reference papers

- de Beer, R. (2026). *Deterministic Disturbance Modeling Framework for Residual-Envelope Fusion Systems*. Local manuscript / DOI pending.
- de Beer, R. (2026). *Slew-Aware Trust-Adaptive Nonlinear State Estimation for Oscillatory Systems With Drift and Corruption*. Zenodo. DOI: [10.5281/zenodo.18642887](https://doi.org/10.5281/zenodo.18642887)
- de Beer, R. (2026). *Hierarchical Residual-Envelope Trust: A Deterministic Framework for Grouped Multi-Sensor Fusion*. Zenodo. DOI: [10.5281/zenodo.18783283](https://doi.org/10.5281/zenodo.18783283)
- DSFB repository: [https://github.com/infinityabundance/dsfb](https://github.com/infinityabundance/dsfb)

## What this crate provides

- A Rust API for residual-envelope disturbance analysis.
- Deterministic disturbance generators with no random behavior inside each generator.
- A single-channel DDMF kernel and a simple grouped multi-channel extension.
- A seeded Monte Carlo sweep harness for disturbance-space exploration.
- A CLI that writes reproducible outputs under `output-dsfb-ddmf/<timestamp>/`.
- Colab-ready notebook assets in both `.ipynb` and `.py` form.

## What goes in and what comes out

Inputs to `SimulationConfig`:

- `n_steps`: simulation length
- `rho`: envelope forgetting factor in `(0, 1)`
- `beta`: trust sensitivity parameter
- `disturbance_kind`: deterministic disturbance class and parameters
- `epsilon_bound`: optional bounded residual contribution

Outputs from `run_simulation`:

- `r`: residual trace
- `d`: disturbance trace
- `s`: envelope trace
- `w`: trust trace

Outputs from the Monte Carlo CLI:

- `results.csv`
- `summary.json`
- `single_run_impulse.csv`
- `single_run_persistent.csv`

The Colab notebook then reads those files and saves Plotly figures such as:

- `envelope_impulse_vs_persistent.png`
- `envelope_impulse_vs_persistent.pdf`

## DDMF kernel summary

For each channel:

- Residual decomposition: `r_k[n] = epsilon_k[n] + d_k[n]`
- Envelope recursion: `s_k[n+1] = rho s_k[n] + (1-rho)|r_k[n]|`
- Trust mapping: `w_k[n] = 1 / (1 + beta_k s_k[n])`

This crate keeps the disturbance-side analysis deterministic. It does not introduce stochastic noise models or statistical hypothesis tests.

## Disturbance classes

`DisturbanceKind` supports:

- `PointwiseBounded { d }`
- `Drift { b, s_max }`
- `SlewRateBounded { s_max }`
- `Impulsive { amplitude, start, len }`
- `PersistentElevated { r_nom, r_high, step_time }`

These map directly to the DDMF categories discussed in the paper: bounded, drift-type, slew-rate-limited, impulsive, and sustained elevated disturbance regimes.

## Build and run

From the workspace root:

```bash
cargo build -p dsfb-ddmf
cargo run -p dsfb-ddmf --bin monte_carlo
```

From the crate directory:

```bash
cd crates/dsfb-ddmf
cargo build
cargo run --bin monte_carlo
```

The Monte Carlo default is `x360` runs. You can override that explicitly:

```bash
cargo run --bin monte_carlo -- --runs 360 --steps 180
```

All runtime outputs are written under:

```text
output-dsfb-ddmf/YYYYMMDD_HHMMSS/
```

## Colab workflow

Notebook files live in this crate directory:

- `dsfb_ddmf_colab.ipynb`
- `dsfb_ddmf_colab_latest.ipynb`
- `dsfb_ddmf_colab.py`

The notebook:

- builds the Rust crate in release mode
- runs the Monte Carlo CLI with `--runs 360`
- detects the latest output directory
- loads `results.csv` and the example trajectory CSVs into pandas
- generates Plotly figures for envelope, trust, and Monte Carlo summaries
- saves PNG and PDF figures back into the same output directory

## Integration with the DSFB workspace

`dsfb-ddmf` depends on the existing `dsfb` crate via workspace path dependency and reuses `dsfb::TrustStats` as the bridge type for envelope/trust state summaries.

Conceptually:

- `dsfb` remains the core residual-envelope fusion crate
- `dsfb-hret` extends trust logic to grouped hierarchical fusion
- `dsfb-ddmf` adds deterministic disturbance classes, simulation, and Monte Carlo reproducibility tooling on top of those residual-envelope ideas

## Limitations

- DDMF is deterministic and structural; it does not claim probabilistic optimality.
- Slew-rate-only disturbances without a magnitude bound are intentionally treated as inadmissible / unbounded regimes.
- The crate analyzes envelope and trust behavior, not full observer-state stability.

## License

Apache-2.0
