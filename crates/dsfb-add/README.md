# dsfb-add — Algebraic Deterministic Dynamics Sweep

This crate runs deterministic parameter sweeps for Algebraic Deterministic Dynamics (ADD) and writes structural diagnostics as CSV under `output-dsfb-add/<timestamp>/` at the repo root.

It is the empirical companion to the ADD paper: a lightweight Rust implementation of four toy-but-structured models that mirror the paper's stack:

- Algebraic Echo Theory (AET)
- Topological Charge Propagation (TCP)
- Resonance Lattice Theory (RLT)
- Invariant Word-Length Thermodynamics (IWLT)

## Motivation

ADD extends the deterministic style of DSFB upward into structural dynamics. Instead of using stochastic primitives as the default language for memory, disorder, spread, and irreversibility, the ADD paper frames those effects through exact evolution rules on words, graphs, and trajectory-generated complexes.

`dsfb-add` is the empirical side of that argument. It does not try to be a full physics engine. Instead, it gives the repo a reproducible sweep harness that scans a deterministic lambda grid (`num_lambda = 360` by default), computes structural diagnostics for each sub-theory, and produces CSV outputs that can be turned into paper-ready figures in Google Colab.

## Architecture

The crate is split into four simulation modules plus shared configuration, output, and orchestration code.

- `aet`:
  deterministic word evolution on a finite alphabet with terminating, confluent local rewrite rules. It tracks irreducible echo length growth and average increment statistics.
- `tcp`:
  deterministic 2D trajectory generation indexed by lambda. Rust writes per-lambda point clouds and coarse topological proxies; the Colab notebook can then compute richer persistent-homology diagnostics from those point clouds.
- `rlt`:
  deterministic walks on a synthetic resonance lattice. It measures graph escape rate and expansion ratio from the visited component.
- `iwlt`:
  append-only symbolic evolution with local length-non-increasing rewrites. It tracks minimal representative length, entropy density, and average increment.
- `config`:
  defines `SimulationConfig`, including the lambda sweep bounds, step count, seed, and per-subtheory toggles.
- `output`:
  creates `output-dsfb-add/<timestamp>/` using `chrono::Utc::now()` and writes the sweep CSV files.
- `sweep`:
  orchestrates all enabled sub-theories, writes outputs, and exposes the `SweepResult` aggregate.

`SimulationConfig::lambda_grid()` produces evenly spaced lambda values on `[lambda_min, lambda_max]`. With the default configuration the sweep is deterministic and reproducible across runs because each sub-theory derives all pseudo-random choices from `random_seed` and the lambda index.

The crate also depends on the workspace `dsfb` crate. That dependency is used to build a small deterministic drive signal shared across the ADD toy models, so the sweep remains aligned with the DSFB repository's observer-first philosophy without modifying any existing DSFB source code.

## Running The Sweep

From the repo root:

```bash
cargo run -p dsfb-add --bin dsfb_add_sweep
```

Optional config override:

```bash
cargo run -p dsfb-add --bin dsfb_add_sweep -- --config crates/dsfb-add/config.json
```

If `config.json` exists in the current working directory, the binary loads it automatically. Otherwise it uses `SimulationConfig::default()`.

Each run creates:

```text
output-dsfb-add/<YYYY-MM-DDTHH-MM-SSZ>/
```

inside the workspace root and writes the requested CSV outputs there.

## Using The Colab Notebook

Workflow:

1. Run the Rust sweep locally so the CSV files and TCP point clouds are generated.
2. Zip, upload, or sync `output-dsfb-add/<timestamp>/` into your Colab environment.
3. Open `crates/dsfb-add/dsfb_add_sweep.ipynb` using the Colab badge in the main repo README.
4. Set `OUTPUT_DIR` in the notebook to the uploaded timestamped folder.
5. Run the notebook cells to generate Plotly PNG figures in the same directory as the CSVs.

The notebook is structured so Rust remains the authoritative simulation layer and Colab remains the figure-generation layer.

## Outputs

Expected runtime files:

- `aet_sweep.csv`
- `tcp_sweep.csv`
- `rlt_sweep.csv`
- `iwlt_sweep.csv`
- `tcp_points/lambda_<idx>_run_<r>.csv`
- `rlt_examples/trajectory_bounded_lambda_<idx>.csv`
- `rlt_examples/trajectory_expanding_lambda_<idx>.csv`
- `tcp_ph_summary.csv` (written by the Colab notebook after persistent-homology post-processing)

Expected notebook figure outputs:

- `fig_aet_echo_slope_vs_lambda.png`
- `fig_iwlt_entropy_density_vs_lambda.png`
- `fig_rlt_escape_rate_vs_lambda.png`
- `fig_rlt_expansion_ratio_vs_lambda.png`
- `fig_rlt_expansion_ratio_vs_lambda_zoom.png`
- `fig_rlt_trajectory_bounded.png`
- `fig_rlt_trajectory_expanding.png`
- `fig_tcp_betti1_mean_vs_lambda.png`
- `fig_tcp_total_persistence_vs_lambda.png`
- `fig_cross_layer_summary_vs_lambda.png`

`tcp_sweep.csv` includes coarse Rust-side topological proxies (`betti0`, `betti1`, `l_tcp`) plus radius statistics. The notebook augments those proxies with `ripser`-based H1 summary statistics computed from the exported per-lambda run clouds. The RLT example trajectories provide a compact visual contrast between bounded recurrence and expanding transport.

## Relationship To The DSFB / ADD Papers

The DSFB crate provides the deterministic observer philosophy already present in this monorepo. The ADD paper extends that philosophy into structural dynamics: irreducible word growth, deterministic topological complexity, resonance spread, and entropy production without stochastic assumptions.

`dsfb-add` turns that argument into a repeatable experiment. Its outputs are the empirical curves used to study echo slopes, entropy densities, resonance spreads, and topology-vs-lambda structure for the ADD stack.
