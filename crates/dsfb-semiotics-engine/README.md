[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-semiotics-engine/dsfb_semiotics_engine_colab.ipynb)

# dsfb-semiotics-engine

`dsfb-semiotics-engine` is a standalone Rust crate inside the DSFB monorepo that implements a deterministic, auditable, synthetic reference artifact for the paper:

*DSFB Structural Semiotics Engine for General Systems: A Deterministic Endoduction Framework for Residual-Based Meaning Extraction*

The crate is intentionally conservative. It does not claim field validation, universal diagnosis, certification, or complete inverse recovery. It implements a reproducible computational companion that turns the paper’s layered objects into explicit Rust types, deterministic scenario generators, deterministic CSV ingestion, tabular artifacts, figures, a PDF report, and a zipped bundle that can be rerun from scratch locally or in Colab.

## Why This Crate Exists

The paper argues that residual evolution can be treated as a structured inferential object rather than as noise alone. This crate exists to make that claim inspectable:

- residuals are constructed explicitly from predicted and observed trajectories
- drift and slew are computed deterministically with documented discrete approximations
- signs, syntax, grammar, and semantics are exposed as separate typed layers
- all intermediate products are exported for auditability
- theorem-aligned synthetic demonstrations are kept distinct from illustrative examples and broader synthetic stress cases
- deterministic reruns are checked explicitly

The goal is not to impress by overclaiming. The goal is to give an aerospace-minded reviewer a deterministic artifact path they can inspect end to end.

## Conceptual Mapping to the Paper

The implementation follows the paper’s layered structure directly.

### Residuals

The residual layer constructs

\[
r(t) = y(t) - \hat{y}(t)
\]

from explicit observed and predicted trajectories.

### Drift and Slew

The syntax layer uses deterministic finite differences:

\[
d(t) = \frac{dr}{dt}, \qquad s(t) = \frac{d^2 r}{dt^2}
\]

In discrete time the crate uses one-sided differences at the boundaries and centered finite differences in the interior. This is a deterministic numerical choice, not a claim that the paper’s continuous-time objects have been solved in full generality.

### Signs

The sign layer constructs

\[
\sigma(t) = (r(t), d(t), s(t))
\]

as an explicit `SignTrajectory` with per-sample projected coordinates used in the figure export.
For figure-oriented visualization, the crate exports the deterministic projected sign coordinates

\[
\left[\|r(t)\|,\ \frac{r(t)\cdot d(t)}{\|r(t)\|},\ \|s(t)\|\right]
\]

with zero signed radial drift reported at exact zero residual norm. This projection is an auditable visualization device, not a latent-state embedding.

### Syntax

Syntax is represented through drift and slew structure, including:

- outward and inward drift fractions
- radial-sign dominance and radial-sign persistence
- drift-channel sign alignment across multi-channel drift vectors
- residual-norm path monotonicity and residual-norm trend alignment
- mean squared slew norm
- late slew-growth score
- localized slew spikes and spike strength
- boundary grazing episode and recovery counts
- grouped aggregate breach fraction when coordinated structure is configured
- trajectory labels such as `persistent-outward-drift`, `coordinated-outward-rise`, `discrete-event-like`, `curvature-rich-transition`, `inward-compatible-containment`, `near-boundary-recurrent`, `weakly-structured-baseline-like`, or a conservative `mixed-structured` fallback when the exported metrics do not justify a narrower rule-based summary

Outward and inward motion are computed from residual-envelope margin evolution and residual-aligned radial drift, not from the sign of a single channel. The monotonicity-style metrics are deterministic path summaries over residual norms rather than complete claims about every channel. The curvature-style metrics are deterministic summaries over slew norms rather than claims about differential geometry in full generality. When the syntax label remains `mixed-structured`, that is a conservative syntax-level non-commitment rather than anomaly language by itself.

### Grammar

Grammar is implemented through admissibility envelopes:

\[
\|r(t)\| \le \rho(t)
\]

The crate includes fixed, widening, tightening, regime-switched, and aggregate group envelopes. Grammar status is exported per step as `Admissible`, `Boundary`, or `Violation`.

### Detectability Bound

For configured theorem-aligned cases the crate evaluates the residual-envelope detectability bound

\[
t^\ast - t_0 \le \frac{\Delta_0}{\alpha - \kappa}
\]

using explicit synthetic cases where the relevant quantities are known by construction. The output compares predicted upper bounds with observed first-crossing times. This is an empirical consistency check on synthetic demonstrations, not a proof of engineering performance.

### Deterministic Interpretability

The crate performs a deterministic reproducibility check for every executed scenario by rerunning the same layered pipeline and hashing the full materialized output, including residuals, drift, slew, sign objects, grammar states, detectability results, and semantic retrieval outputs. The resulting checks and run summary are exported in CSV and JSON, and the run metadata records whether the execution was synthetic or CSV-driven.

### Semantics

The semantics layer is a conservative typed heuristic bank, not a classifier. Each entry carries:

- `heuristic_id`
- `motif_label`
- scope conditions over syntax metrics
- admissibility requirements
- regime tags
- provenance and applicability notes
- retrieval priority
- compatibility / incompatibility metadata

Retrieval is constrained rather than purely threshold-labeled. Each candidate now exports explicit admissibility, regime, and scope-pass explanations in addition to the combined rationale text. The bank supports illustrative motifs such as:

- monotone drift -> gradual degradation candidate
- sustained outward drift with actual envelope breach -> persistent admissibility departure candidate
- localized slew spike -> discrete event candidate
- curvature-rich transition candidate
- curvature-led admissibility departure candidate
- mixed-regime transition candidate when explicit regime-shift tags and curvature-led departure structure coexist
- repeated envelope grazing -> near-boundary operation candidate
- recurrent boundary operation candidate when repeated recoverable boundary returns are present
- coordinated aggregate rise -> correlated degradation or common-mode disturbance candidate
- coordinated admissibility departure candidate for stronger grouped breach cases
- inward-compatible containment candidate
- inward recovery-compatible candidate for repeated returns to admissibility under inward-compatible motion
- balanced bounded oscillation -> bounded oscillatory operation candidate
- structured noisy trajectory candidate for admissible but visibly agitated residual evolution
- low-structure admissible evolution -> weakly structured baseline-compatible observation candidate
- explicit compatible sets when every matched pair is bank-compatible
- explicit ambiguity when matched heuristics conflict
- explicit `Unknown`, including whether the current outcome reflects low evidence or bank noncoverage, with an exported detail string explaining which case occurred

These are constrained heuristic retrieval outcomes only. They do not imply unique latent cause. In particular, the baseline-compatible path is a low-commitment description relative to the configured prediction and envelope, not a validated health classifier, and compatible sets remain jointly reportable motifs rather than collapsed diagnoses.

### CSV Ingestion Path

In addition to the bundled synthetic scenarios, the crate supports a deterministic CSV ingestion mode for externally supplied observed and predicted trajectories. The ingestion path:

- parses observed and predicted CSV files with explicit validation
- preserves channel names from headers or an optional override
- accepts an optional explicit `--time-column`
- falls back to deterministic sample times from row order and `--dt` when no explicit time column is supplied
- applies a user-configured admissibility envelope
- feeds the exact same residual -> sign -> syntax -> grammar -> semantics pipeline
- does not add any field-validation claim beyond the supplied trajectories and configured envelope

## Scenario Program

The crate ships deterministic synthetic scenarios covering:

- nominal stable behavior
- gradual degradation / monotone drift
- curvature onset
- abrupt event / slew spike
- oscillatory bounded behavior
- multiple theorem-aligned outward-drift exit cases
- inward-compatible invariance
- grouped correlated residual rise
- regime-switched envelopes
- noisy but structured deterministic residuals
- magnitude-matched admissible vs detectable comparison

Each scenario records:

- purpose
- theorem alignment note
- claim class
- limitation note

## Technical Architecture

The code is organized into explicit layers:

- `src/math/`
  Residual construction, finite differences, envelopes, detectability helpers, and deterministic metrics.
- `src/engine/`
  Residual, sign, syntax, grammar, semantics, typed run-config, and orchestration pipeline layers.
- `src/sim/`
  Deterministic synthetic scenario generators.
- `src/figures/`
  Publication-oriented PNG and SVG figure export.
- `src/report/`
  Markdown and PDF artifact report generation.
- `src/io/`
  Timestamped output layout, deterministic CSV ingestion, schema metadata, CSV/JSON export, and zip export.
- `tests/`
  Real tests for residual math, detectability, determinism, semantics, and output layout.

The crate is intentionally standalone by using its own empty `[workspace]` section. That keeps it self-contained and avoids changing root workspace behavior.

An architecture note with a compact layer diagram is available in [`docs/architecture.md`](docs/architecture.md).

## Library API

The crate can be driven as a library through typed deterministic configuration objects rather than only through the CLI.

```rust
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};

let config = EngineConfig::synthetic_single(
    CommonRunConfig::default(),
    "nominal_stable",
);
let engine = StructuralSemioticsEngine::new(config);
let bundle = engine.run_selected()?;
let artifacts = export_artifacts(&bundle)?;
```

For CSV-driven runs, use `EngineConfig::csv(...)` with a validated `CsvInputConfig`. The exported run metadata and manifest both carry the additive schema marker `dsfb-semiotics-engine/v1`.

## Evaluation Harness

The crate keeps deterministic engine outputs separate from deterministic evaluation summaries.

The evaluation layer exports:

- run-level semantic disposition counts
- run-level syntax label counts
- run-level comparator trigger counts
- scenario-level summaries
- per-scenario reproducibility status
- heuristic-bank validation results
- artifact completeness checks
- sweep summaries when sweep mode is used

The internal deterministic comparators are intentionally simple:

- residual threshold crossing only
- moving-average residual norm trend only
- slew spike detector only
- envelope interaction only

These are internal deterministic comparators for inspection. They are not field benchmarks and they do not support superiority claims.

## Heuristic Bank Governance

The semantics bank is maintained through a governed registry rather than an ad hoc list.

Each run exports a validation report covering:

- bank version
- duplicate heuristic ID detection
- unknown compatibility-link targets
- missing provenance text
- scope-condition sanity notes

Compatibility gaps are exported explicitly as review notes. They do not silently disappear into retrieval behavior.

## Sweep Mode

The crate supports deterministic calibration-style synthetic sweeps:

- `gradual-drift-slope`
- `curvature-onset-timing`
- `spike-magnitude-duration`
- `oscillation-amplitude-frequency`
- `coordinated-rise-strength`
- `envelope-tightness`

Sweep runs use the same core pipeline and export the same artifact bundle shape, with additive sweep summary tables and a sweep figure. These are calibration-style synthetic studies only.

## Illustrative CSV Example

A small CSV-driven example is included for deterministic end-to-end review:

- data: [`examples/data/illustrative_observed.csv`](examples/data/illustrative_observed.csv) and [`examples/data/illustrative_predicted.csv`](examples/data/illustrative_predicted.csv)
- example binary: [`examples/run_csv_fixture.rs`](examples/run_csv_fixture.rs)
- walkthrough: [`docs/examples/illustrative_csv_example.md`](docs/examples/illustrative_csv_example.md)

These files are public and version-controlled in the crate so the CSV path can be rerun without network access. They are illustrative CSV inputs only, not field-validation data.

## Running Locally

Build the crate:

```bash
cargo build --manifest-path crates/dsfb-semiotics-engine/Cargo.toml
```

Run the full deterministic demonstration suite:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- --all
```

Run one scenario:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- --scenario outward_exit_case_a
```

Run CSV ingestion mode:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --input-mode csv \
  --observed-csv /path/to/observed.csv \
  --predicted-csv /path/to/predicted.csv \
  --scenario-id csv_case \
  --time-column timestamp \
  --envelope-mode fixed \
  --envelope-base 1.0 \
  --dt 0.5
```

Run a deterministic sweep:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --sweep-family oscillation-amplitude-frequency \
  --sweep-points 6
```

Override the output root:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- --all --output-dir /tmp/dsfb-semiotics-engine-artifacts
```

Set an explicit deterministic seed:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- --all --seed 123
```

Run tests:

```bash
cargo test --manifest-path crates/dsfb-semiotics-engine/Cargo.toml
```

Run the full crate-local quality gate:

```bash
cd crates/dsfb-semiotics-engine
just qa
```

This executes formatting, clippy, tests, and docs for the crate only. Contributor expectations and extension guidance are recorded in `CONTRIBUTING.md`.
Because this work is restricted to the crate directory, a crate-local GitHub Actions template is provided at `ci/github-actions-crate-quality-gate.yml` rather than installing a live repo-root workflow automatically.

Refresh snapshot fixtures intentionally with:

```bash
cd crates/dsfb-semiotics-engine
DSFB_UPDATE_SNAPSHOTS=1 cargo test --test snapshots
```

## Output Layout

By default the crate writes to:

```text
crates/dsfb-semiotics-engine/output-dsfb-semiotics-engine/<timestamp>/
```

where `<timestamp>` has the form `YYYY-MM-DD_HH-MM-SS`.

Each run emits:

- `figures/*.png`
- `figures/*.svg`
- `csv/*.csv`
- `json/*.json`
- `report/dsfb_semiotics_engine_report.md`
- `report/dsfb_semiotics_engine_report.pdf`
- `manifest.json`
- `dsfb-semiotics-engine-<timestamp>.zip`

The zip archive contains the generated run directory contents for convenient download.
The PDF report embeds the generated figure PNG artifacts and appends a deterministic artifact appendix covering the exported markdown, manifest, CSV, and JSON text artifacts.
Artifact export treats the timestamped run directory as owned scratch space: expected artifact subdirectories are cleaned before rewriting, and unexpected root-level files cause the export to fail rather than silently mixing stale results into a purportedly fresh run.
Both `run_metadata.json` and `manifest.json` carry the additive schema marker `dsfb-semiotics-engine/v1` so downstream review tooling can check the exported contract explicitly.

Additional evaluation artifacts include:

- `csv/evaluation_summary.csv`
- `csv/scenario_evaluations.csv`
- `csv/baseline_comparators.csv`
- `csv/heuristic_bank_validation.csv`
- `csv/artifact_completeness.csv`
- `json/evaluation_summary.json`
- `json/scenario_evaluations.json`
- `json/baseline_comparators.json`
- `json/heuristic_bank_validation.json`
- `json/artifact_completeness.json`
- `csv/sweep_results.csv` and `json/sweep_results.json` for sweep runs
- `csv/sweep_summary.csv` and `json/sweep_summary.json` for sweep runs

A schema overview is provided in [`docs/schema.md`](docs/schema.md).

## Figure Suite

The crate generates the original twelve paper-aligned figures plus additive evaluation figures:

1. residual vs prediction / observation overview
2. drift and slew decomposition
3. projected sign trajectory using the deterministic coordinates `[||r||, dot(r,d)/||r||, ||s||]`
4. syntax comparison
5. envelope exit under sustained outward drift
6. envelope invariance under inward-compatible drift
7. exit-invariance pair under a common envelope
8. residual trajectory separation
9. detectability bound comparison
10. deterministic pipeline flow
11. coordinated group semiotics with local versus aggregate envelope behavior
12. semantic retrieval / heuristics-bank summary
13. internal deterministic baseline comparator summary
14. sweep stability summary when sweep mode is executed

## Colab Notebook

The notebook lives at:

`crates/dsfb-semiotics-engine/dsfb_semiotics_engine_colab.ipynb`

It:

- installs Rust if needed
- locates or clones the repository
- rebuilds the crate from source
- reruns the Rust artifact generator
- loads the newest timestamped run
- displays summary tables inline
- displays all figures inline
- surfaces the PDF report path and zip path

The notebook does not reimplement the semiotic engine logic in Python.

## Limitations and Non-Claims

- All scenarios are synthetic deterministic constructions.
- CSV ingestion mode runs the same deterministic engine on supplied trajectories but does not, by itself, validate those inputs or their predictive model.
- The crate demonstrates theorem-aligned behavior under configured assumptions; it does not prove those assumptions hold in real systems.
- Envelope exit is treated as detectable departure from the configured admissibility grammar, not unique diagnosis.
- Heuristic semantic matches are constrained motif retrieval outputs only and may remain ambiguous or unknown.
- Internal comparator and sweep outputs are empirical aids for deterministic inspection only; they are not external benchmarks.
- No claim is made that this crate replaces probabilistic monitoring, validates all domains, or achieves certification.

## Why This Is Useful in Deterministic Engineering Diagnostics

Even with those limits, the crate is useful because it makes a disciplined path visible:

- the inference path is deterministic
- the intermediate objects are auditable
- the figures come from exported artifacts rather than hand-drawn illustrations
- repeated runs create fresh timestamped folders instead of overwriting prior evidence
- the output bundle is structured for review, replay, and cautious extension

That is a reasonable computational companion for a methodology paper aimed at deterministic, auditable engineering diagnostics.
