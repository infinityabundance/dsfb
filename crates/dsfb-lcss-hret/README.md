# dsfb-lcss-hret

IEEE L-CSS figure generation for DSFB high-rate estimation trust analysis.

## Description

This crate generates benchmark data and figures for an IEEE L-CSS (Letters of Control Systems Society) submission. It implements high-rate estimation trust analysis experiments for the Drift-Slew Fusion Bootstrap (DSFB) algorithm.

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

See the companion Jupyter/Colab notebook `dsfb_lcss_hret_figures.ipynb` for generating publication-ready figures from the benchmark outputs.

## License

Apache-2.0
