# dsfb-dscd

Deterministic Structural Causal Dynamics (DSCD) on top of `dsfb` + `dsfb-add`.

## Quickstart

Run a Colab-friendly deterministic sweep:

```bash
cargo run -p dsfb-dscd -- --quick
```

Run a larger workstation sweep:

```bash
cargo run -p dsfb-dscd -- --full
```

Override scale controls explicitly:

```bash
cargo run -p dsfb-dscd -- --num-events 100000 --scaling-ns 4096,8192,16384,32768,65536,100000 --num-tau-samples 1001
```

The binary writes outputs to:

`output-dsfb-dscd/<YYYYMMDD_HHMMSS>/`

including:

- `threshold_scaling_summary.csv`
- `threshold_curve_N_<N>.csv`
- `graph_events.csv`
- `graph_edges.csv`
- `degree_distribution.csv`
- `interval_sizes.csv`
- `path_lengths.csv`
- `edge_provenance.csv`

## Notebook

Use:

`crates/dsfb-dscd/notebooks/dscd_plots.ipynb`

to regenerate DSCD paper figures directly from the generated CSVs.

All DSCD paper figures are reproducible from the `dsfb-dscd` outputs and notebook.
