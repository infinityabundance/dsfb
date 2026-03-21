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
Because raw differentiation can amplify jitter, the sign generator also supports an optional deterministic low-latency smoothing pass before drift and slew estimation. The default posture remains conservative (`disabled`), the selected smoothing mode and parameters are exported in run metadata, and the raw residual export path remains unchanged.

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
Grammar export is not limited to a bare boolean outcome. Each sample also carries a typed grammar reason code and human-readable explanation such as:

- `Admissible`
- `Boundary`
- `RecurrentBoundaryGrazing`
- `SustainedOutwardDrift`
- `AbruptSlewViolation`
- `EnvelopeViolation`

These grammar reports describe structural inadmissibility relative to the configured envelope. They do not imply root-cause certainty.
Each grammar sample also exports a deterministic trust scalar in `[0,1]`. This trust value is a deployment-oriented interface derived from grammar severity and margin behavior, not a field-validated control law.

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

Retrieval is constrained rather than purely threshold-labeled. Each candidate now exports explicit admissibility, regime, and scope-pass explanations in addition to the combined rationale text. The bank may come from the compiled builtin registry or from a validated external JSON artifact. In either case the run metadata records the bank schema version, bank version, source kind, optional source path, content hash, and strict-validation mode. The bank supports illustrative motifs such as:

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
For larger banks the crate can also build a deterministic admissibility/regime/group-breach prefilter index before exact scope and compatibility checks run. That index narrows candidate sets without replacing the authoritative typed validation path, and the export surface records whether retrieval used the indexed or linear path.

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
To select an external bank artifact instead of the builtin fallback, set `CommonRunConfig { bank: BankRunConfig::external(path, strict), ..Default::default() }` before constructing the `EngineConfig`.

## Bounded Online Engine

In addition to the batch artifact pipeline, the crate now exposes a bounded online engine in [`src/live/`](src/live/) for replay and deployment-oriented integration surfaces.

- online history is retained in a fixed-capacity ring buffer with deterministic overwrite semantics
- the buffer capacity is explicit in `EngineSettings::online.history_buffer_capacity`
- run metadata and manifests export the configured online history buffer capacity
- optional offline accumulation remains separate so report/export workflows can still retain full histories without making the live path unbounded

This separation matters for long-endurance or embedded-style use because the live engine path no longer requires unbounded sign or residual history growth.
The public example [`examples/live_drop_in.rs`](examples/live_drop_in.rs) shows the intended one-sample-at-a-time bounded loop directly.

## Units and Physical Interpretation

Residual units are inherited from the upstream residual source. When the residual is physically meaningful, the downstream drift and slew quantities inherit those units in the usual way. For example:

- residual: `mm`
- drift: `mm/s`
- slew: `mm/s^2`

Equivalently, a residual measured in millimeters implies drift in millimeters/second and slew in millimeters/second^2.

The crate does not fake units when the residual source is abstract or normalized. In those cases the docs and examples treat the quantities as unitless or source-inherited rather than claiming a universal physical interpretation.

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
- smoothing comparison reports showing raw-versus-active derivative behavior
- retrieval indexing reports using deterministic candidate-count scaling proxies
- figure-source tables for every publication-style figure
- figure-integrity checks tying rendered figures back to exported source rows and emitted image files
- sweep summaries when sweep mode is used

The internal deterministic comparators are intentionally simple:

- residual threshold crossing only
- moving-average residual norm trend only
- slew spike detector only
- envelope interaction only
- one-sided CUSUM residual-norm detector only
- innovation-style squared residual statistic only

These are internal deterministic comparators for inspection. They are not field benchmarks and they do not support superiority claims.
They intentionally collapse structure into scalar triggers; the layered DSFB pipeline preserves syntax, grammar, and constrained semantic distinctions that these comparators do not retain.

The comparison framing is intentionally operator-legible. These internal monitors are analogous in spirit to threshold detectors, innovation-style monitoring, and change detectors, but they remain within-crate deterministic comparisons on shared scenario families rather than external benchmarks.
More concretely, the report and docs frame them as conservative within-crate analogies to threshold monitors, EKF innovation monitoring, chi-squared-style gating, and one-sided change detectors. They are presented to help an operator compare alarm behavior and lost structural resolution, not to claim benchmark superiority.
Each exported report now includes an `Operator-Legible Comparator Case Study` table for a curated scenario family so a reviewer can see, in one place, which scalar comparators alarm and where DSFB retains bounded-oscillatory, structured-noisy, discrete-event, or curvature-led distinctions instead of collapsing structure into a single trigger.

## Heuristic Bank Governance

The semantics bank is maintained through a governed registry rather than an ad hoc list.

The runtime supports two deterministic bank sources:

- builtin bank: compiled into the crate for offline reference runs and tests
- external bank: loaded from a typed JSON artifact under the schema marker `dsfb-semiotics-engine-bank/v1`

External-bank loading does not bypass typing or validation. The artifact is parsed into the same typed registry used by the builtin bank, normalized deterministically after parse, and then validated before the engine runs. The crate records whether the active bank came from the builtin registry or an external artifact, together with the bank version, content hash, and validation mode.
The typed external-bank artifact format is documented in [docs/bank_schema.md](docs/bank_schema.md).

Each run exports a validation report covering:

- bank version
- bank schema version
- bank source kind and optional source path
- bank content hash
- validation mode (`strict` or `permissive`)
- duplicate heuristic ID detection
- unknown compatibility-link targets
- self-link detection
- compatibility / incompatibility overlap detection
- missing provenance text
- empty or duplicated regime-tag notes
- retrieval-priority sanity notes
- scope-condition sanity notes
- optional strict-mode symmetry failures for compatibility and incompatibility links

Strict governance is now the default posture for both builtin and external banks. Compatibility gaps are exported explicitly and, under the default strict mode, missing reverse links, unknown references, contradictions, duplicate IDs, and incomplete required metadata fail the run. Permissive mode is explicit opt-in through `--bank-validation-mode permissive` and is intended for authoring or review only; permissive runs are exported as not governance-clean in the manifest and report. The compatibility alias `--strict-bank-validation` is still accepted for users carrying older scripts.
This strict validation default is part of the runtime contract rather than a documentation-only preference.

The builtin bank is no longer treated as the only operational path. A validated external JSON bank can be swapped at startup without recompiling the engine logic, which allows motif-library updates to remain data-driven and separately versioned from the core engine implementation.

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

A minimal external-bank walkthrough is also included at [`docs/examples/external_bank_example.md`](docs/examples/external_bank_example.md).

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

Run with an external bank artifact instead of the builtin fallback:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario nominal_stable \
  --bank-mode external \
  --bank-path crates/dsfb-semiotics-engine/tests/fixtures/external_bank_minimal.json
```

Strict governance is the default. To opt into a review-only permissive run, add `--bank-validation-mode permissive`.

Run a deterministic sweep:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --sweep-family oscillation-amplitude-frequency \
  --sweep-points 6
```

Render the deterministic `ratatui` replay dashboard for one scenario:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario nominal_stable \
  --dashboard-replay \
  --dashboard-max-frames 4
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

This executes formatting, clippy, tests, docs, snapshots, fixed-seed property tests, and dashboard replay smoke coverage for the crate only. Contributor expectations and extension guidance are recorded in `CONTRIBUTING.md`.
Because this work is restricted to the crate directory, crate-local GitHub Actions workflow templates are provided at `.github/workflows/crate-quality-gate.yml` and `ci/github-actions-crate-quality-gate.yml` rather than installing a live repo-root workflow automatically.
The fixed-seed property-test surface can also be run directly with `cargo test --test proptest_invariants`.
The crate-local workflow mirrors also include a `numeric-f32` compile check and a small FFI smoke compile for the checked-in C example.
Property budgets are controlled with `DSFB_PROPTEST_MODE=smoke|research|stress`, where the default research-grade budget is 256 cases and the high-risk dashboard/near-threshold properties use 512 cases.

The crate currently ships one additive Cargo feature flag:

- `external-bank` (enabled by default): enables external heuristic-bank JSON loading alongside the builtin fallback
- `numeric-f32`: narrows the bounded live-engine ingestion surface to `f32` while the internal deterministic math remains explicitly widened for conservative reproducibility; manifests record the selected numeric mode

Check the crate in `numeric-f32` mode:

```bash
cargo check --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --features numeric-f32
```

Run the bounded online failure-injection example:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --example synthetic_failure_injection
```

Run the physically grounded vibration-to-thermal-drift example:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --example vibration_to_thermal_drift
```

Run the bounded live drop-in example:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --example live_drop_in
```

Build the legacy-integration FFI crate and header surface:

```bash
cargo test --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -p dsfb-semiotics-engine-ffi
```

The checked-in header lives at [`ffi/include/dsfb_semiotics_engine.h`](ffi/include/dsfb_semiotics_engine.h), with minimal examples at [`ffi/examples/minimal_ffi.c`](ffi/examples/minimal_ffi.c) and [`ffi/examples/minimal_ffi.cpp`](ffi/examples/minimal_ffi.cpp).
The FFI ABI is intentionally code-oriented: `DsfbCurrentStatus` carries numeric syntax / grammar / semantic codes plus trust, while dedicated helpers copy human-readable labels and the stable last-error string into caller-owned buffers. Ownership rules and shared/static library build notes are documented in [`docs/examples/ffi_integration.md`](docs/examples/ffi_integration.md).

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
Both `run_metadata.json` and `manifest.json` carry the additive schema marker `dsfb-semiotics-engine/v1` so downstream review tooling can check the exported contract explicitly. They also export a deterministic `run_configuration_hash` alongside the explicit settings dump and bank provenance.
Those metadata files also export the bounded online-history buffer capacity and the selected numeric mode.
Heuristic-bank artifacts use the separate schema marker `dsfb-semiotics-engine-bank/v1`, and the resolved bank descriptor is exported at `json/loaded_heuristic_bank_descriptor.json`.

Additional evaluation artifacts include:

- `csv/figure_01_residual_prediction_observation_overview_source.csv` through `csv/figure_13_internal_baseline_comparators_source.csv`
- matching `json/<figure-id>_source.json` files for each rendered publication-style figure
- `csv/evaluation_summary.csv`
- `csv/scenario_evaluations.csv`
- `csv/baseline_comparators.csv`
- `csv/comparator_results.csv`
- `csv/heuristic_bank_validation.csv`
- `csv/bank_validation_report.csv`
- `csv/artifact_completeness.csv`
- `csv/smoothing_comparison_report.csv`
- `csv/retrieval_latency_report.csv`
- `csv/figure_09_detectability_source.csv`
- `csv/figure_12_semantic_retrieval_source.csv`
- `csv/figure_13_internal_baseline_comparators_source.csv`
- `csv/figure_integrity_checks.csv`
- `csv/figure_integrity_report.csv`
- `json/evaluation_summary.json`
- `json/scenario_evaluations.json`
- `json/baseline_comparators.json`
- `json/comparator_results.json`
- `json/semantic_matches.json`
- `json/heuristic_bank_validation.json`
- `json/bank_validation_report.json`
- `json/loaded_heuristic_bank_descriptor.json`
- `json/artifact_completeness.json`
- `json/smoothing_comparison_report.json`
- `json/retrieval_latency_report.json`
- `json/figure_09_detectability_source.json`
- `json/figure_12_semantic_retrieval_source.json`
- `json/figure_13_internal_baseline_comparators_source.json`
- `json/figure_integrity_checks.json`
- `json/figure_integrity_report.json`
- `csv/sweep_results.csv` and `json/sweep_results.json` for sweep runs
- `csv/sweep_summary.csv` and `json/sweep_summary.json` for sweep runs
- `csv/figure_14_sweep_stability_summary_source.csv` and `json/figure_14_sweep_stability_summary_source.json` for sweep runs
- `csv/figure_14_sweep_stability_source.csv` and `json/figure_14_sweep_stability_source.json` for sweep runs

A schema overview is provided in [`docs/schema.md`](docs/schema.md).
Bank-artifact notes are summarized in [`docs/bank_schema.md`](docs/bank_schema.md), and future embedded-core extraction notes are recorded in [`docs/embedded_core_roadmap.md`](docs/embedded_core_roadmap.md).

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
    panel 1: leading candidate score
    panel 2: typed-bank entries remaining after admissibility filtering
    panel 3: final disposition code (`Unknown=0`, `Ambiguous=1`, `CompatibleSet=2`, `Match=3`)
13. internal deterministic baseline comparator summary
14. sweep stability summary when sweep mode is executed

Every publication-style figure is now paired with a machine-readable figure-source table whose rows carry the exact panel ids, series labels, plotted coordinates, generation timestamp, and additive figure metadata used by the renderer. The artifact pipeline also emits figure-integrity records that check panel counts, expected-versus-observed panel identities, emitted PNG/SVG presence, integer-like behavior for count-labeled panels, and source-table consistency.

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
- displays resolved artifact paths for debugging
- renders one-click download buttons for the PDF report and ZIP bundle when those artifacts exist
- shows a clear warning instead of a broken button when an expected artifact is missing

The notebook does not reimplement the semiotic engine logic in Python.

## FFI And Legacy Integration

For legacy control stacks and C or C++ hosts, the crate includes a minimal nested FFI crate at [`ffi/`](ffi/).

- it exposes a small C ABI around the bounded online engine
- the exported surface supports create, destroy, push-sample, current-status query, and reset
- grammar state and grammar reason are exposed explicitly through C-friendly enums
- the current-status ABI also exports a trust scalar and semantic disposition code instead of relying only on strings
- the header is checked in so downstream users do not need Rust tooling merely to inspect the ABI

This is a minimal integration path for experimentation and interoperability. It is not a certification claim.

## Synthetic Failure Injection Example

[`examples/synthetic_failure_injection.rs`](examples/synthetic_failure_injection.rs) provides a dead-simple deterministic example that starts from a nominal oscillatory signal, injects a linear degradation term, pushes the signal through the bounded online engine, and prints a time-stamped interpretation trace.

The example demonstrates:

- nominal behavior
- structural change detection
- grammar escalation
- semantic retrieval under the current bank

The printed wording is illustrative and depends on the configured heuristic bank, but the example is intended to be operator-readable and easy to rerun locally.

## Vibration To Thermal Drift Example

[`examples/vibration_to_thermal_drift.rs`](examples/vibration_to_thermal_drift.rs) and [`docs/examples/vibration_to_thermal_drift.md`](docs/examples/vibration_to_thermal_drift.md) provide a more physically grounded walkthrough. The example treats the residual as a bearing-gap signal in millimeters and explicitly discusses residual, drift, and slew in millimeters, millimeters/second, and millimeters/second^2 as the signal transitions from vibration-like high-frequency behavior into slower thermal-like drift.

## Limitations and Non-Claims

- All scenarios are synthetic deterministic constructions.
- CSV ingestion mode runs the same deterministic engine on supplied trajectories but does not, by itself, validate those inputs or their predictive model.
- The crate demonstrates theorem-aligned behavior under configured assumptions; it does not prove those assumptions hold in real systems.
- Envelope exit is treated as detectable departure from the configured admissibility grammar, not unique diagnosis.
- Heuristic semantic matches are constrained motif retrieval outputs only and may remain ambiguous or unknown.
- Builtin-bank and external-bank runs may differ when the bank artifact version or content differs.
- Internal comparator and sweep outputs are empirical aids for deterministic inspection only; they are not external benchmarks.
- The crate is not `no_std` today and is not packaged for direct embedded deployment.
- No claim is made that this crate replaces probabilistic monitoring, validates all domains, or achieves certification.

## Why This Is Useful in Deterministic Engineering Diagnostics

Even with those limits, the crate is useful because it makes a disciplined path visible:

- the inference path is deterministic
- the intermediate objects are auditable
- the figures come from exported artifacts rather than hand-drawn illustrations
- repeated runs create fresh timestamped folders instead of overwriting prior evidence
- the output bundle is structured for review, replay, and cautious extension

That is a reasonable computational companion for a methodology paper aimed at deterministic, auditable engineering diagnostics.
