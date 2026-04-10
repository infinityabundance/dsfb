# dsfb-turbine

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-turbine/notebooks/dsfb_turbine_colab.ipynb)

**DSFB Structural Semiotics Engine for Gas Turbine Jet Engine Health Monitoring**

A deterministic, read-only, observer-only augmentation layer for typed residual
interpretation over existing Engine Health Monitoring (EHM), Gas Path Analysis (GPA),
and Prognostics and Health Management (PHM) systems.

## Architectural Contract

| Property | Guarantee |
|----------|-----------|
| **Read-only** | Core inference consumes immutable slices and writes only to caller-provided output buffers |
| **Non-interfering** | The core engine exposes no callback or feedback path to upstream EHM/FADEC/GPA logic |
| **Deterministic** | The core engine uses no random seeds or training-dependent weights |
| **no_std** | `src/core/` is buildable without `std` |
| **no_alloc** | `src/core/` uses fixed-size stack data and caller-provided buffers rather than heap allocation |
| **no_unsafe** | `#![forbid(unsafe_code)]` is enforced in the library crate; examples and tests also forbid `unsafe` |

## What DSFB Does

DSFB reads health-parameter residuals that existing EHM systems already produce
and returns typed, auditable structural interpretations of what those residuals
mean over time. The embedded core is separated from the std-backed evaluation
tooling used for datasets, reporting, and figure generation. DSFB does not
predict RUL, modify FADEC logic, or replace any incumbent method.

## What DSFB Does NOT Do

- Does not predict Remaining Useful Life (RUL)
- Does not modify engine control or protection systems
- Does not claim superiority over GPA, Kalman filters, or ML prognostics
- Does not require changes to existing certified systems

## Datasets

Validated against five public datasets:

1. **C-MAPSS FD001** — 100 engines, single condition, HPC degradation (primary)
2. **C-MAPSS FD003** — 100 engines, single condition, HPC + fan degradation
3. **C-MAPSS FD002** — 260 engines, six conditions, HPC degradation
4. **C-MAPSS FD004** — 249 engines, six conditions, HPC + fan degradation
5. **N-CMAPSS** — Realistic flight conditions (Chao et al., 2021)

## Quick Start

```bash
# Run basic pipeline example (no data needed)
cargo run --example basic_pipeline

# Run the evaluation entrypoint with default paths (data/, output_full/)
cargo run --example cmapss_eval

# Reproduce the paper evaluation suite explicitly
cargo run --example cmapss_eval -- path/to/cmapss_data_dir output/

# Require benchmark data and abort instead of falling back to synthetic
cargo run --example cmapss_eval -- --require-real-data path/to/cmapss_data_dir output/
```

`cargo run --example cmapss_eval -- path/to/cmapss_data_dir output/`
reproduces the FD001/FD002/FD003 evaluation used in the paper when
`train_FD001.txt`, `train_FD002.txt`, and `train_FD003.txt` are present in
that directory.

`cargo run --example cmapss_eval -- --require-real-data path/to/cmapss_data_dir output/`
enforces a benchmark-only run and exits with an error instead of invoking the
synthetic demonstration path.

If `train_FD001.txt` is absent, the evaluation example falls back to a
synthetic demonstration rather than claiming a paper reproduction run.

## How To Read The Benchmark Outputs

The recommended C-MAPSS runs in this crate may produce visually "clean"
headline numbers, including 100% Boundary detection, 100% Violation
occurrence, 100% early warning, and 0% clean-window false alarms for selected
configurations. That behavior is expected in this workflow because the example
explicitly performs an in-benchmark parameter sweep and then reports what DSFB
can do when calibrated for that benchmark setting.

Those outputs should therefore be read narrowly. They are not presented as a
head-to-head performance comparison against incumbent prognostic methods, and
they are not evidence that DSFB is universally superior or deployment-ready.
Comparator baselines, uncertainty estimates, and broader robustness studies are
separate empirical questions and are out of scope for this crate's example
artifacts.

The purpose of these runs is different: to show the structural information that
DSFB can expose when it is well-tuned for a residual stream. DSFB is not
intended to compete with or replace existing probabilistic EHM, GPA, or PHM
methods. It is intended to augment them by attaching deterministic, typed,
human-readable structural meaning to residual behavior that is otherwise often
thresholded away, aggregated, or left uninterpreted.

## Build Modes

Typical usage in this crate is split into two layers:

- `src/core/`: the embedded-facing engine, intended to remain `no_std`, `no_alloc`, and `no_unsafe`
- `src/dataset/`, `src/pipeline/`, `src/report/`, `src/figures/`, and executable examples: std-backed research and evaluation tooling

For a core-only build, disable default features:

```bash
cargo check --lib --no-default-features
```

## Google Colab

The crate ships with a Colab notebook at
`notebooks/dsfb_turbine_colab.ipynb`. The notebook:

- clones the repository into a fresh Colab runtime
- installs Rust and the small Python dependencies needed for artifact packaging
- downloads the public NASA C-MAPSS benchmark archive used by the crate example
- runs `cargo run --example cmapss_eval -- --require-real-data <data-dir> <output-dir>`
- stages generated SVG figures into a dedicated figures folder
- builds a single PDF containing all generated figures
- builds a zip archive containing the run artifacts for download

The notebook writes run outputs under `crates/dsfb-turbine/colab_output/`
inside the fresh clone. It recursively extracts the downloaded archive and
requires `train_FD001.txt`, `train_FD002.txt`, and `train_FD003.txt` to be
present before the evaluation step. If those files are not recovered, the
notebook aborts rather than using the crate's synthetic fallback path. The
notebook enforces this by invoking the example's `--require-real-data` mode.

For empirical clarity: the downloaded NASA C-MAPSS archive is the public
benchmark simulation dataset used by the paper workflow. It is not field-engine
telemetry.

## Crate Structure

```
src/
  lib.rs                    # Library root: forbid unsafe; no_std when std feature is disabled
  core/                     # Core engine (no_std, no_alloc)
    residual.rs             # Residual sign: (r, d, s) computation
    envelope.rs             # Admissibility envelope construction
    grammar.rs              # Grammar state machine (Admissible/Boundary/Violation)
    heuristics.rs           # Heuristics bank (typed degradation motifs)
    regime.rs               # Operating-regime classification
    episode.rs              # Episode formation
    audit.rs                # Deterministic audit trace
    config.rs               # Configuration (version-locked to paper)
    theorem.rs              # Theorem 1 finite-exit bound
    sensitivity.rs          # Parameter sensitivity sweep
    channels.rs             # C-MAPSS sensor channel mapping
  dataset/                  # Data loading (std-gated, alloc-using)
    cmapss.rs               # C-MAPSS parser
    ncmapss.rs              # N-CMAPSS placeholder
  pipeline/                 # Evaluation orchestration (std-gated, alloc-using)
    engine_eval.rs          # Single-engine pipeline
    fleet.rs                # Fleet-level orchestrator
    metrics.rs              # Fleet metrics computation
  figures/                  # SVG figure generation (std-gated, alloc-using)
  report/                   # Text report generation (std-gated, alloc-using)
examples/
  basic_pipeline.rs         # Minimal core-engine walkthrough
  cmapss_eval.rs            # Full executable evaluation entrypoint
notebooks/
  dsfb_turbine_colab.ipynb  # Fresh-runtime Colab runner and artifact packager
```

## License

Apache-2.0. Commercial deployment requires separate license from Invariant Forge LLC.

## Citation

de Beer, R. (2026). DSFB Structural Semiotics Engine for Gas Turbine Jet Engine
Health Monitoring. Invariant Forge LLC.

## Prior Art

This work constitutes prior art under 35 U.S.C. § 102, timestamped via
Zenodo DOI and crates.io publication.
