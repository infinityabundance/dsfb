# dsfb-lcss-hret

[![crates.io](https://img.shields.io/crates/v/dsfb-lcss-hret.svg)](https://crates.io/crates/dsfb-lcss-hret)
[![docs.rs](https://docs.rs/dsfb-lcss-hret/badge.svg)](https://docs.rs/dsfb-lcss-hret)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-lcss-hret/dsfb_lcss_hret_figures.ipynb)

IEEE L-CSS figure generation for DSFB high-rate estimation trust analysis.

## Description

This crate generates benchmark data and publication-ready figures for an IEEE L-CSS (Letters of Control Systems Society) submission. It implements high-rate estimation trust analysis experiments for the Drift-Slew Fusion Bootstrap (DSFB) algorithm, comparing performance across different estimation methods with varying parameter configurations.

### Key Features

- **Standalone benchmarking**: Independent CLI tool for running experiments
- **Monte Carlo simulations**: Configurable number of runs for statistical analysis
- **Parameter sweep**: Systematic exploration of algorithm parameters
- **CSV output**: Structured data for reproducible figure generation
- **IEEE-formatted figures**: Colab notebook for publication-ready visualizations

## Building and Running

This is a standalone crate that compiles independently:

```bash
# Build the crate
cargo build --release --manifest-path crates/dsfb-lcss-hret/Cargo.toml

# Run with default benchmark
cargo run --release --manifest-path crates/dsfb-lcss-hret/Cargo.toml -- --run-default

# Run parameter sweep
cargo run --release --manifest-path crates/dsfb-lcss-hret/Cargo.toml -- --run-sweep

# Customize parameters
cargo run --release --manifest-path crates/dsfb-lcss-hret/Cargo.toml -- \
  --run-default \
  --num-runs 500 \
  --time-steps 2000 \
  --seed 123 \
  --output my-output-dir
```

## Output

The tool generates timestamped directories under `output-dsfb-lcss-hret/` (or the specified output directory) containing:

- `summary.csv` - Aggregate statistics for different estimation methods
- `trajectories.csv` - Time-series data for state estimates
- `heatmap.csv` - Parameter sweep results for visualization

## Plotting

Use the companion Jupyter/Colab notebook to generate publication-ready figures from the benchmark outputs:

**Google Colab (recommended):**
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-lcss-hret/dsfb_lcss_hret_figures.ipynb)

**Local Jupyter:**
```bash
jupyter notebook crates/dsfb-lcss-hret/dsfb_lcss_hret_figures.ipynb
```

The notebook generates three IEEE-formatted figures:
- **Figure 1**: Method comparison bar chart with error bars
- **Figure 2**: State estimation trajectory and error plots
- **Figure 3**: Parameter sweep heatmap

All figures are saved as both PDF (for publication) and PNG (for preview) at 300 DPI with IEEE single-column formatting (3.5" width).

## Related Crates

This crate is part of the DSFB (Drift-Slew Fusion Bootstrap) repository:
- [`dsfb`](https://crates.io/crates/dsfb) - Core DSFB estimator implementation
- `dsfb-fusion-bench` - Fusion diagnostics benchmarking tool

## Citation

If you use this work in your research, please cite:

```bibtex
@software{dsfb2026,
  author = {de Beer, Riaan},
  title = {DSFB: Drift-Slew Fusion Bootstrap},
  year = {2026},
  url = {https://github.com/infinityabundance/dsfb}
}
```

## License

Apache-2.0
