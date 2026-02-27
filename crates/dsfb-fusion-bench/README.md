# dsfb-fusion-bench

[![crates.io](https://img.shields.io/crates/v/dsfb-fusion-bench.svg)](https://crates.io/crates/dsfb-fusion-bench)
[![docs.rs](https://docs.rs/dsfb-fusion-bench/badge.svg)](https://docs.rs/dsfb-fusion-bench)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/infinityabundance/dsfb/blob/main/LICENSE)
[![Fusion Bench Notebook In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-fusion-bench/dsfb_fusion_figures.ipynb)

Deterministic synthetic benchmarking package for DSFB fusion diagnostics. This crate generates reproducible synthetic state-estimation benchmarks and writes stable CSV/JSON outputs for paper figures and notebook analysis.

In practical terms, this crate is a benchmark runner for comparing DSFB against alternative reconstruction methods on the same synthetic diagnostic problem. It:

- generates deterministic simulated state trajectories and grouped measurements
- injects configurable corruption and fault structure
- runs multiple fusion/reconstruction methods over the same data
- measures error and timing
- writes stable CSV and JSON outputs for notebooks, figures, and paper tables

Use it when you want reproducible method-to-method comparisons rather than a live estimator embedded in an application.

## What goes in and what comes out

Inputs:

- built-in default or sweep TOML configs
- optional `--config`, `--seed`, `--methods`, and `--outdir` overrides
- method selection across `equal`, `cov_inflate`, `irls_huber`, `nis_hard`, `nis_soft`, and `dsfb`

Outputs:

- per-run timestamped output directory
- summary tables, heatmaps, trajectory traces, and manifest metadata
- deterministic benchmark artifacts suitable for Colab and offline plotting

## Install

Install from crates.io:

```bash
cargo install dsfb-fusion-bench
```

Or run from source:

```bash
git clone https://github.com/infinityabundance/dsfb
cd dsfb
cargo run --release -p dsfb-fusion-bench -- --run-default
```

## Quick Start

```bash
# Default deterministic benchmark run
cargo run --release -p dsfb-fusion-bench -- --run-default

# Alpha/beta sweep run
cargo run --release -p dsfb-fusion-bench -- --run-sweep
```

Optional flags:

```text
--config <path>
--outdir <path>
--seed <int>
--run-default
--run-sweep
--methods <comma-separated>
```

The CLI is the primary interface. It loads a benchmark configuration, synthesizes the measurements, runs the selected methods, and writes comparable outputs with a stable schema.

## Reproducibility

- Fixed RNG seeds (configurable in TOML)
- Deterministic output ordering
- Stable output schema version: `1.0.0`
- CPU-only execution
- Timestamped output folders to avoid overwriting prior runs

## Methods

- `equal`
- `cov_inflate`
- `irls_huber`
- `nis_hard`
- `nis_soft`
- `dsfb`

## Outputs

Outputs are written to `output-dsfb-fusion-bench/<YYYYMMDD_HHMMSS>/` by default:

- `summary.csv`
- `heatmap.csv`
- `trajectories.csv`
- `sim-dsfb-fusion-bench.csv`
- `manifest.json`
- `summary_sweep.csv` (sweep mode)

`sim-dsfb-fusion-bench.csv` has the same schema as `trajectories.csv`; it exists to keep crate-specific naming distinct from `dsfb` simulation outputs.

Core metrics in summaries:

- `peak_err`
- `rms_err`
- `false_downweight_rate`
- `baseline_wls_us`
- `overhead_us`
- `total_us`

## Notebook Workflow

Companion notebook:

- `crates/dsfb-fusion-bench/dsfb_fusion_figures.ipynb`

The notebook auto-detects the latest run in `output-dsfb-fusion-bench/` and falls back to legacy paths or uploaded files in Colab.

Google Colab note:
- Click `Run all` first.
- If prompted for data files, click `Browse` in the file picker and upload:
- `summary.csv`
- `heatmap.csv`
- `sim-dsfb-fusion-bench.csv` (or `trajectories.csv`)

## Repository

Full documentation, notebooks, and scripts:
https://github.com/infinityabundance/dsfb

## Separate Crate In This Repo

For the DSFB estimator crate itself, see:

- `crates/dsfb`
- `crates/dsfb/README.md`

## Citation

If you use `dsfb-fusion-bench` in your research, please cite:

> **de Beer, R.** (2026).  
> *Trust-Adaptive Multi-Diagnostic Weighting for Magnetically Confined Plasma State Estimation* (v1.0).  
> Zenodo. https://doi.org/10.5281/zenodo.18644561

## Publication Note

`dsfb-fusion-bench` is maintained alongside a separate paper workflow in this repository.
