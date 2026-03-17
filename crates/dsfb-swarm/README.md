# dsfb-swarm

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-swarm/notebooks/dsfb_swarm_colab.ipynb)

`dsfb-swarm` is a standalone Rust crate that turns the paper *Deterministic Spectral Residual Inference for Swarm Interaction Networks: A DSFB Framework for Structural Phase Stability* into a reproducible empirical workflow. It simulates time-varying swarm interaction graphs, computes Laplacian spectra, predicts `lambda_k(t)` deterministically, measures residuals and residual derivatives, applies trust-gated interaction attenuation, compares against baselines, and writes a timestamped artifact bundle with figures, CSVs, JSON, Markdown, and PDF output.

## Citation

- de Beer, R. (2026). *Deterministic Spectral Residual Inference for Swarm Interaction Networks: A DSFB Framework for Structural Phase Stability (v1.0)*. Zenodo. DOI: [10.5281/zenodo.19073826](https://doi.org/10.5281/zenodo.19073826)

## Why this crate exists

The paper is about more than plotting `lambda_2(t)`. Its main point is that deterministic spectral prediction plus residual-centered diagnosis can reveal structural phase changes in a swarm interaction network before visible fragmentation is obvious. This crate exists to make those claims inspectable:

- it constructs a dynamic graph `G(t)` from interacting agents,
- it builds the Laplacian `L(t) = D(t) - A(t)`,
- it monitors `lambda_2(t)` and higher modes,
- it predicts those modes with deterministic predictors `hat lambda_k(t)`,
- it forms residuals `r_k(t) = lambda_k(t) - hat lambda_k(t)`,
- it computes residual drift, residual slew, and residual envelopes,
- it adds mode-shape residuals with eigenvector sign-ambiguity handling,
- it attenuates interactions through trust-gated edge weights,
- it compares the residual loop against simpler baselines,
- it exports enough raw and summarized data for paper-style interpretation.

The crate is intentionally self-contained. It includes its own `Cargo.toml`, local notebook, tests, and crate-local default output root so it can be run from `crates/dsfb-swarm` without requiring workspace edits elsewhere.

## What mathematical ideas it demonstrates

The implementation follows the paper’s intended objects and terminology:

- Dynamic swarm interaction graph `G(t)` with weighted adjacency `A(t)`.
- Degree matrix `D(t)` and Laplacian `L(t) = D(t) - A(t)`.
- Laplacian eigenvalues `lambda_k(t)`, especially algebraic connectivity `lambda_2(t)`.
- Deterministic spectral predictors:
  - zero-order hold,
  - first-order extrapolation,
  - smooth corrective extrapolation.
- Spectral residuals `r_{lambda_k}(t) = lambda_k(t) - hat lambda_k(t)`.
- Residual drift and residual slew as first and second finite-difference diagnostics.
- Residual envelopes and anomaly certificates.
- Finite-time detectability interpretation through persistent negative residual drift.
- Practical boundedness of the predictor-residual loop in nominal settings.
- Correlation between residual magnitude and Laplacian perturbation scale `||Delta L||_F`.
- Multi-mode residual stack `r_Lambda(t)` for `lambda_2 .. lambda_{m+1}`.
- Optional mode-shape residuals from eigenvector comparison with sign ambiguity handling.
- Trust-gated effective interaction weights `a_ij(t) = T_ij(t) * tilde a_ij(t)`.

The code does not claim a formal proof. It produces deterministic empirical evidence aligned with the paper’s framework.

## Scenario design

The crate runs four paper-facing scenarios:

1. `nominal`
   - Stable connected coordination.
   - Bounded deterministic excitation.
   - No persistent structural degradation.

2. `gradual_edge_degradation`
   - Bridge edges weaken progressively.
   - `lambda_2(t)` contracts over time.
   - Residual drift should become persistently negative before visible failure.

3. `adversarial_agent`
   - One agent injects inconsistent state and motion.
   - The disturbance is localized before trust gating attenuates its influence.
   - Multi-mode and mode-shape diagnostics are most useful here.

4. `communication_loss`
   - Cross-cluster bridge links collapse.
   - The graph fragments and `lambda_2(t)` falls sharply.
   - Trust and residual alarms should respond before or near fragmentation.

## Architecture overview

```text
crates/dsfb-swarm/
├── Cargo.toml
├── README.md
├── assets/
│   └── README_FIGURE_PLAN.md
├── examples/
│   └── quickstart.rs
├── notebooks/
│   └── dsfb_swarm_colab.ipynb
├── src/
│   ├── cli.rs
│   ├── config.rs
│   ├── error.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── math/
│   │   ├── baselines.rs
│   │   ├── envelopes.rs
│   │   ├── laplacian.rs
│   │   ├── metrics.rs
│   │   ├── predictor.rs
│   │   ├── residuals.rs
│   │   ├── spectrum.rs
│   │   └── trust.rs
│   ├── report/
│   │   ├── csv.rs
│   │   ├── json.rs
│   │   ├── manifest.rs
│   │   ├── mod.rs
│   │   └── plotting_data.rs
│   └── sim/
│       ├── agents.rs
│       ├── dynamics.rs
│       ├── graph.rs
│       ├── mod.rs
│       ├── runner.rs
│       └── scenarios.rs
└── tests/
    ├── metrics.rs
    ├── regression.rs
    └── smoke.rs
```

### Module responsibilities

- `sim/agents.rs`
  - deterministic initial swarm geometry,
  - local cluster layout with bridge nodes.

- `sim/graph.rs`
  - radius/k-nearest interaction graph construction,
  - pairwise disagreement measures,
  - edge export helpers.

- `sim/dynamics.rs`
  - consensus/alignment-style 2D motion,
  - scalar consensus state update,
  - deterministic bounded excitation.

- `sim/scenarios.rs`
  - paper-facing perturbation schedules,
  - edge weakening,
  - adversarial forcing,
  - communication-loss modulation.

- `math/spectrum.rs`
  - symmetric Laplacian eigendecomposition,
  - ordered eigenpairs,
  - mode-shape sign ambiguity handling.

- `math/predictor.rs`
  - deterministic spectral prediction for scalar and multi-mode monitoring.

- `math/residuals.rs`
  - residuals,
  - drift,
  - slew,
  - mode-shape residuals,
  - combined diagnostic scores.

- `math/envelopes.rs`
  - residual envelopes,
  - warmup calibration,
  - anomaly certificates,
  - persistent negative-drift detection.

- `math/trust.rs`
  - binary or smooth trust gating,
  - node and edge trust evolution,
  - trust-modulated effective adjacency.

- `math/baselines.rs`
  - state-norm thresholding,
  - disagreement-energy thresholding,
  - raw `lambda_2` thresholding.

- `math/metrics.rs`
  - detection lead time,
  - TPR/FPR,
  - trust suppression delay,
  - residual-to-topology correlation,
  - boundedness summary statistics.

- `report/*`
  - CSV/JSON emission,
  - figure rendering,
  - manifest creation,
  - Markdown/PDF report writing.

## CLI usage

Run all scenarios with default settings:

```bash
cd crates/dsfb-swarm
cargo run --release -- run
```

Run a single scenario:

```bash
cd crates/dsfb-swarm
cargo run --release -- run --scenario nominal
```

Run a paper-facing multi-mode diagnostic:

```bash
cd crates/dsfb-swarm
cargo run --release -- run --scenario adversarial-agent --multi-mode --modes 4 --mode-shapes
```

Run the benchmark suite across all scenarios:

```bash
cd crates/dsfb-swarm
cargo run --release -- benchmark --all-scenarios
```

Run a custom scaling and noise sweep:

```bash
cd crates/dsfb-swarm
cargo run --release -- benchmark \
  --all-scenarios \
  --sizes 20,50,100,200,500 \
  --noise 0.01,0.05,0.1,0.2 \
  --modes 4 \
  --mode-shapes
```

Quickstart example:

```bash
cd crates/dsfb-swarm
cargo run --release -- quickstart
```

Regenerate a compact report for the newest run:

```bash
cd crates/dsfb-swarm
cargo run --release -- report
```

### Optional config file

The CLI accepts JSON or TOML config patches through `--config`.

Example TOML:

```toml
[run]
scenario = "communication-loss"
steps = 240
agents = 48
multi_mode = true
monitored_modes = 5
mode_shapes = true
noise_level = 0.05
```

## Output directory structure

To avoid collateral writes elsewhere in the repository, the default output root is crate-local:

```text
crates/dsfb-swarm/output-dsfb-swarm/YYYY-MM-DD_HH-MM-SS/
```

Each run creates a fresh timestamped directory. No prior output is overwritten.

Core artifacts:

- `manifest.json`
- `run_config.json`
- `scenarios_summary.csv`
- `benchmark_summary.csv`
- `hero_benchmark_summary.csv`
- `time_series.csv`
- `detector_debug.csv`
- `spectra.csv`
- `residuals.csv`
- `trust.csv`
- `baselines.csv`
- `anomalies.json`
- `scenario_<name>_metrics.csv`
- `scenario_<name>_timeseries.csv`
- `figures/*.png`
- `report/dsfb_swarm_report.md`
- `report/dsfb_swarm_report.pdf`

Calibration-critical columns now exported in the CSV summaries:

- `scenarios_summary.csv`
  - `visible_failure_step`
  - `scalar_detection_step`
  - `multimode_detection_step`
  - `scalar_detection_lead_time`
  - `multimode_detection_lead_time`
  - `baseline_*_lead_time`
  - `multimode_minus_scalar_seconds`
  - `trust_drop_step`
  - `trust_suppression_delay`
  - `residual_topology_correlation`

- `benchmark_summary.csv`
  - the same lead-time and trust-delay fields across size/noise sweeps,
  - `best_baseline_name`, `best_baseline_lead_time`, and `lead_time_gain_vs_best_baseline`,
  - `dsfb_advantage_margin` as the exported headline margin against the best baseline,
  - `tpr_gain_vs_best_baseline` and `fpr_reduction_vs_best_baseline`,
  - `peak_mode_shape_norm` and `peak_stack_score` for multi-mode diagnostics,
  - runtime and residual-to-topology correlation for scaling studies.

- `hero_benchmark_summary.csv`
  - one strongest-evidence row per scenario for:
    - `gradual_edge_degradation`
    - `adversarial_agent`
    - `communication_loss`
  - scalar lead, multi lead, best-baseline lead, `dsfb_advantage_margin`, trust delay, TPR/FPR pairs, and a deterministic `winner` label.

- `time_series.csv`
  - calibrated scalar and multi-mode detector signals,
  - signal/combined ratios,
  - calibrated limits,
  - trust scores and trust alarm inputs,
  - baseline flags,
  - scalar-vs-multimode anomaly decisions at every step.

- `detector_debug.csv`
  - the same per-step detector trace as `time_series.csv`, retained as an explicitly named debug export for calibration audits and notebook-side inspection.

## What the figures and tables are meant to show

- `lambda2_timeseries.png`
  - Visible contraction or collapse of algebraic connectivity.

- `residual_timeseries.png`
  - Predictor versus observation together with the calibrated negative-residual detector signal and its crossing limit.

- `drift_slew.png`
  - Negative residual drift magnitude, drift threshold, and slew diagnostics for finite-time detectability.

- `trust_evolution.png`
  - Trust suppression of degraded or adversarial influence.

- `baseline_comparison.png`
  - Detection lead-time comparison versus non-predictive baselines.

- `scaling_curves.png`
  - Runtime growth with swarm size.

- `noise_stress_curves.png`
  - TPR/FPR behavior under bounded deterministic noise stress.

- `multimode_comparison.png`
  - Scalar-vs-multi lead times plus direct multi-mode detection advantage in seconds.

- `topology_snapshots.png`
  - Concrete graph changes corresponding to spectral warnings.

- `hero_leadtime_comparison.png`
  - The strongest calibrated scalar, multi-mode, and best-baseline lead times for the three headline scenarios, paired with the exported DSFB advantage margin and annotated by the computed winner.

- `hero_benchmark_table.png`
  - A compact publication-style table view of the hero benchmark rows with the winner and DSFB advantage margin exposed directly.

- `adversarial_trust_detection_focus.png`
  - A paper-ready adversarial timing view showing onset, multi-mode detection, scalar detection, trust-drop timing, and trust suppression on one aligned time axis.

## Reproducibility notes

- The swarm dynamics are deterministic except for bounded analytic pseudo-noise terms generated from fixed trigonometric expressions.
- Identical settings reproduce the same metrics and figures except for the timestamped folder name.
- The output root is local to this crate by default, which keeps the crate self-contained.
- The notebook rebuilds the crate and reruns the benchmark suite from scratch every time.

With the current calibration, the representative benchmark is designed to show:

- a clear scalar lead-time win over raw `lambda_2` thresholding in `communication_loss`,
- a clear multi-mode lead-time win over the best baseline in `gradual_edge_degradation`,
- stronger adversarial diagnosis from multi-mode plus trust than from scalar-only monitoring,
- explicit positive DSFB advantage margins in the hero rows where structural diagnosis matters,
- measurable trust suppression delay in adversarial and communication-loss cases,
- non-empty baseline and benchmark comparison tables rather than placeholder zeros.

## Colab notebook

The notebook at `notebooks/dsfb_swarm_colab.ipynb` is designed to:

- bootstrap Rust in Colab if needed,
- locate or clone the repository,
- build `dsfb-swarm` from source,
- run the benchmark suite from scratch,
- locate the newest timestamped output directory,
- load CSV/JSON artifacts,
- display the hero figures, adversarial timing view, and supporting tables inline,
- display the hero benchmark rows inline,
- assemble a notebook-side PDF report of the Colab results,
- copy the notebook itself into the run’s report folder,
- write a JSON manifest for notebook-produced artifacts,
- create a zip archive containing the full artifact bundle,
- surface direct download links for the PDF, zip archive, and manifest at the end of the run.

When the notebook completes, the run’s `report/` directory contains these Colab-specific outputs in addition to the Rust-generated report:

- `dsfb_swarm_colab_report.pdf`
- `dsfb_swarm_artifacts.zip`
- `colab_artifact_manifest.json`
- `dsfb_swarm_colab.ipynb`

The Colab badge and notebook bootstrap default to `https://github.com/infinityabundance/dsfb.git`. You can still override that at runtime with `DSFB_REPO_URL`, `DSFB_REPO_REF`, or `DSFB_REPO_ROOT`.

## Limitations

- The simulator is intentionally stylized rather than domain-specific.
- Trust is driven by deterministic residual and disagreement logic, not by a learned model.
- The PDF written by the Rust crate is a compact summary report; the notebook can regenerate a richer PDF bundle using the figure outputs.
- The benchmark suite is designed for demonstrative reproducibility, not for maximum numerical efficiency.
- Some scenario-level claims remain scenario-dependent: for example, adversarial runs often show earlier multi-mode detection without a clean `lambda_2`-based visible-failure threshold.

## Future extensions

- richer swarm kinematics and heading dynamics,
- explicit predictor correction loops per mode,
- stronger mode-shape alignment under clustered eigenvalues,
- more detailed graph-theoretic certificates,
- alternative trust policies and adversary models,
- tighter report/PDF integration with embedded figures.
