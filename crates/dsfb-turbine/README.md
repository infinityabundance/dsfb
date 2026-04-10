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

## Math

For a single health channel and cycle index `k`, the paper-level object is the
residual sign

```text
sigma_k = (r_k, d_k, s_k)
```

with

```text
r_k = y_k - y_hat_k
```

where `y_k` is the observed health quantity and `y_hat_k` is the nominal
reference under the declared operating regime. In this crate, the default
benchmark path estimates that reference from the healthy window at the start of
each channel, so the implemented residual is formed relative to the healthy
window mean returned by `compute_baseline`.

The paper states the pointwise structural descriptors as

```text
d_k = r_k - r_{k-1}
s_k = d_k - d_{k-1}
```

The crate uses windowed forms for robustness against short transients:

```text
drift_k = (1 / W) * sum_{i=0}^{W-1} (r_{k-i} - r_{k-i-1})
slew_k  = (1 / W) * sum_{i=0}^{W-1} (drift_{k-i} - drift_{k-i-1})
```

where `W` is taken from `DsfbConfig::drift_window` and
`DsfbConfig::slew_window`. This is the behavior implemented by
`compute_drift` and `compute_slew`.

Admissibility is defined relative to a regime-conditioned envelope constructed
from healthy-window statistics:

```text
E = [mu - envelope_sigma * std, mu + envelope_sigma * std]
```

The operational test in the crate is therefore equivalent to checking whether
`|r_k| <= envelope_sigma * std`, together with the normalized position and gap
used by `AdmissibilityEnvelope`.

The grammar layer then maps each cycle into `Admissible`, `Boundary`, or
`Violation`. In the current implementation:

- `Admissible -> Boundary` occurs when the residual approaches the envelope or
  when outward drift persists beyond `persistence_threshold`
- `Boundary -> Violation` occurs when the envelope is exceeded or when
  nontrivial slew persists beyond `slew_persistence_threshold`
- `Violation` is terminal within a single evaluation pass

The paper's finite-exit bound is represented in `TheoremOneBound`:

```text
k_star - k_0 <= ceil(g_k0 / (eta - kappa))
g_k = epsilon_k - |r_k|
```

For the fixed-envelope benchmark runs in this crate, `kappa` is typically zero.
The reported theorem check is therefore an audit quantity for the observed
trajectory under the chosen configuration, not a blanket claim of universal
performance.

## Code

The crate follows the same separation described in the paper:

- `src/core/residual.rs`: baseline estimation, residual formation, drift, slew,
  and the `ResidualSign` tuple
- `src/core/envelope.rs`: admissibility envelope construction, normalized
  position, and envelope-status classification
- `src/core/grammar.rs`: deterministic grammar-state machine plus multi-channel
  aggregation
- `src/core/heuristics.rs`: typed reason codes and the heuristics bank used to
  attach structural interpretations
- `src/core/episode.rs`, `src/core/audit.rs`, and `src/core/theorem.rs`:
  operator-facing episodes, audit traces, and the finite-exit bound report
- `src/pipeline/engine_eval.rs`: std-backed orchestration from per-channel core
  operators to engine-level outputs
- `examples/basic_pipeline.rs` and `examples/cmapss_eval.rs`: runnable entry
  points for the minimal core path and the full benchmark path
- `tests/non_interference.rs`: crate-local checks for the observer-only,
  borrowed-input contract

Minimal core usage looks like this:

```rust
use dsfb_turbine::core::config::DsfbConfig;
use dsfb_turbine::core::envelope::AdmissibilityEnvelope;
use dsfb_turbine::core::grammar::GrammarEngine;
use dsfb_turbine::core::regime::OperatingRegime;
use dsfb_turbine::core::residual::{
    compute_baseline, compute_drift, compute_residuals, compute_slew, sign_at,
};

let values = [1580.0, 1580.2, 1580.1, 1580.4, 1580.9, 1581.4];
let config = DsfbConfig::cmapss_fd001_default();
let (mean, std) = compute_baseline(&values, &config);

let mut residuals = [0.0; 6];
let mut drift = [0.0; 6];
let mut slew = [0.0; 6];

compute_residuals(&values, mean, &mut residuals);
compute_drift(&residuals, config.drift_window, &mut drift);
compute_slew(&drift, config.slew_window, &mut slew);

let envelope = AdmissibilityEnvelope::from_baseline(
    mean,
    std,
    OperatingRegime::SeaLevelStatic,
    &config,
);
let mut grammar = GrammarEngine::new();

for k in 0..values.len() {
    let sign = sign_at(&residuals, &drift, &slew, k, 1);
    grammar.advance(&sign, &envelope, &config);
}
```

That snippet is intentionally narrow: it shows the deterministic core path on a
single channel. The std-backed pipeline layers build on the same operators for
dataset loading, fleet evaluation, reports, and figure generation.

## How To Run

Use the path that matches the claim you want to make:

- Core-only compile check:
  `cargo check --lib --no-default-features`
- Minimal deterministic walkthrough with synthetic local data:
  `cargo run --example basic_pipeline`
- Full benchmark reproduction from local C-MAPSS files:
  `cargo run --example cmapss_eval -- --require-real-data path/to/cmapss_data_dir output/`
- Fresh-runtime Colab reproduction and artifact packaging:
  open `notebooks/dsfb_turbine_colab.ipynb`

The strict `--require-real-data` mode is the reproducibility path for the paper
workflow. It aborts if the required `train_FD001.txt`, `train_FD002.txt`, and
`train_FD003.txt` files are not present. Running without that flag is useful
for local smoke testing, but it may fall back to the synthetic demo and should
not be described as a benchmark reproduction run.

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
