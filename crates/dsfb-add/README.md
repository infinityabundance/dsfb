# dsfb-add — Algebraic Deterministic Dynamics Sweep

`dsfb-add` is the empirical sweep and figure-generation companion for the Algebraic Deterministic Dynamics (ADD) stack. It runs deterministic lambda sweeps, exports structural observables as CSV, and feeds a Colab notebook that regenerates the paper-facing figures from those CSVs.

At a high level, the crate does three things:

1. It evolves four deterministic toy models across a shared lambda grid.
2. It writes reproducible sweep outputs into `output-dsfb-add/<timestamp>/` at the repo root.
3. It provides the numerical substrate for the ADD paper's figures: cross-layer response curves, transport transition summaries, structural-law fits, robustness checks, and finite-size scaling diagnostics.

The stack it sweeps is:

- Algebraic Echo Theory (AET)
- Topological Charge Propagation (TCP)
- Resonance Lattice Theory (RLT)
- Invariant Word-Length Thermodynamics (IWLT)

## Motivation

ADD extends the deterministic style of DSFB upward into structural dynamics. Instead of using stochastic primitives as the default language for memory, disorder, spread, and irreversibility, the ADD paper frames those effects through exact evolution rules on words, graphs, and trajectory-generated complexes.

`dsfb-add` is the empirical side of that argument. It is not a full physics engine and it does not claim microscopic fidelity. Its job is narrower and more useful for the paper: it provides a deterministic numerical laboratory in which the four layers can be swept together, perturbed in a controlled way, compared across finite trajectory lengths, and summarized with a small number of interpretable structural observables.

In practice that means the crate can answer questions like:

- how echo growth changes across lambda,
- how invariant word-length entropy tracks that echo growth,
- where the resonance transport transition occurs,
- how persistent-topology summaries respond across the same sweep,
- whether the AET-IWLT structural law survives deterministic rule perturbations,
- and whether those claims stabilize as `steps_per_run` increases.

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
  creates `output-dsfb-add/<timestamp>/` using `chrono::Utc::now()` and writes sweep, phase-boundary, and robustness CSV files.
- `sweep`:
  orchestrates baseline and perturbed sweeps, optional multi-`N` runs, phase-boundary extraction, and robustness summaries.
- `analysis/rlt_phase`:
  extracts `lambda_star`, transition brackets, and transition width from the RLT expansion curve.

`SimulationConfig::lambda_grid()` produces evenly spaced lambda values on `[lambda_min, lambda_max]`. With the default configuration the sweep is deterministic and reproducible across runs because each sub-theory derives all pseudo-random choices from `random_seed` and the lambda index.

If `multi_steps_per_run` is populated, the crate repeats the full sweep for every requested trajectory length. This is what supports the paper's finite-size scaling story: the exact same lambda grid and deterministic rules are re-run at multiple `N`, and the notebook then measures how regression slope, `R^2`, residual variance, and phase-boundary location stabilize.

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

Finite-size scaling run:

```bash
cargo run -p dsfb-add --bin dsfb_add_sweep -- --multi-steps 5000,10000,20000
```

If `config.json` exists in the current working directory, the binary loads it automatically. Otherwise it uses `SimulationConfig::default()`.

If `--multi-steps` is provided, those values override the single `steps_per_run` setting for that run.

Each run creates:

```text
output-dsfb-add/<YYYY-MM-DDTHH-MM-SSZ>/
```

inside the workspace root and writes the requested CSV outputs there.

## Using The Colab Notebook

Workflow:

1. Run the Rust sweep locally so the CSV files, point clouds, and trajectory examples are generated.
2. Zip, upload, or sync `output-dsfb-add/<timestamp>/` into your Colab environment, or let the notebook generate a fresh run in Colab.
3. Open `crates/dsfb-add/dsfb_add_sweep.ipynb` using the Colab badge in the main repo README.
4. Set `OUTPUT_DIR` only if you intentionally want an existing run directory; otherwise the notebook can bootstrap a fresh one.
5. Run the notebook cells to regenerate all PNG figures and derived summary CSVs in the same directory as the sweep outputs.

The notebook is structured so Rust remains the authoritative simulation layer and Colab remains the analysis and figure-generation layer. The Rust side produces deterministic sweeps and structural summaries; the notebook performs the paper-facing regression, finite-size scaling, residual diagnostics, universality comparison, and hero-figure assembly.

## Outputs

Expected runtime files:

- `aet_sweep.csv`
- `aet_sweep_perturbed.csv`
- `aet_sweep_N<steps>.csv`
- `aet_sweep_perturbed_N<steps>.csv`
- `tcp_sweep.csv`
- `tcp_sweep_N<steps>.csv`
- `rlt_sweep.csv`
- `rlt_sweep_perturbed.csv`
- `rlt_sweep_N<steps>.csv`
- `rlt_sweep_perturbed_N<steps>.csv`
- `iwlt_sweep.csv`
- `iwlt_sweep_perturbed.csv`
- `iwlt_sweep_N<steps>.csv`
- `iwlt_sweep_perturbed_N<steps>.csv`
- `tcp_points/lambda_<idx>_run_<r>.csv`
- `tcp_points_N<steps>/lambda_<idx>_run_<r>.csv`
- `rlt_examples/trajectory_bounded_lambda_<idx>.csv`
- `rlt_examples/trajectory_expanding_lambda_<idx>.csv`
- `rlt_examples_N<steps>/trajectory_bounded_lambda_<idx>.csv`
- `rlt_examples_N<steps>/trajectory_expanding_lambda_<idx>.csv`
- `rlt_phase_boundary.csv`
- `robustness_metrics.csv`
- `tcp_ph_summary.csv` (written by the Colab notebook after persistent-homology post-processing)
- `aet_iwlt_law_summary.csv` (written by the Colab notebook after regression analysis)
- `aet_iwlt_scaling_summary.csv` (written by the Colab notebook after finite-size scaling analysis)
- `aet_iwlt_diagnostics_summary.csv` (written by the Colab notebook after residual, ratio, and log-log diagnostics)

Expected notebook figure outputs:

- `fig_aet_echo_slope_vs_lambda.png`
- `fig_aet_robustness.png`
- `fig_iwlt_entropy_density_vs_lambda.png`
- `fig_iwlt_robustness.png`
- `fig_rlt_escape_rate_vs_lambda.png`
- `fig_rlt_expansion_ratio_vs_lambda.png`
- `fig_rlt_expansion_ratio_vs_lambda_zoom.png`
- `fig_rlt_robustness.png`
- `fig_rlt_trajectory_bounded.png`
- `fig_rlt_trajectory_expanding.png`
- `fig_tcp_betti1_mean_vs_lambda.png`
- `fig_tcp_total_persistence_vs_lambda.png`
- `fig_aet_iwlt_structural_law.png`
- `fig_aet_iwlt_universality.png`
- `fig_aet_iwlt_scaling_slope_vs_N.png`
- `fig_aet_iwlt_scaling_r2_vs_N.png`
- `fig_aet_iwlt_scaling_resid_vs_N.png`
- `fig_aet_iwlt_residuals_vs_echo.png`
- `fig_aet_iwlt_residual_hist.png`
- `fig_aet_iwlt_ratio_vs_lambda.png`
- `fig_aet_iwlt_ratio_hist.png`
- `fig_aet_iwlt_loglog.png`
- `fig_cross_layer_summary_vs_lambda.png`
- `fig_rlt_phase_lambda_star_vs_N.png`
- `fig_rlt_phase_width_vs_N.png`
- `fig_hero_add_stack.png`

`tcp_sweep.csv` includes coarse Rust-side topological proxies (`betti0`, `betti1`, `l_tcp`) plus radius statistics. The notebook augments those proxies with `ripser`-based H1 summary statistics computed from the exported per-lambda run clouds, with total persistence treated as the main smooth TCP observable.

The perturbed sweep CSVs are small deterministic robustness experiments: they nudge the update laws without changing the overall structural regime picture. `rlt_phase_boundary.csv` extracts the transport transition location and width, `robustness_metrics.csv` quantifies baseline-vs-perturbed deviations, and the AET-IWLT summary/diagnostic CSVs support the structural-law analysis directly.

Taken together, the outputs are meant to support the numerical section of the ADD paper:

- baseline and perturbed sweeps show the main structural response,
- multi-`N` runs show finite-size convergence,
- phase-boundary summaries quantify the RLT transport transition,
- structural-law summaries quantify the AET-IWLT coupling,
- and the hero figure condenses the stack into a single paper-facing panel.

## Relationship To The DSFB / ADD Papers

The DSFB crate provides the deterministic observer philosophy already present in this monorepo. The ADD paper extends that philosophy into structural dynamics: irreducible word growth, deterministic topological complexity, resonance spread, and entropy production without stochastic assumptions.

`dsfb-add` turns that argument into a repeatable experiment. Its outputs are the empirical curves used to study echo slopes, entropy densities, resonance spreads, and topology-vs-lambda structure for the ADD stack.
