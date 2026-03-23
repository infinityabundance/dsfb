# dsfb-computer-graphics

[![Open in Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-computer-graphics/colab/dsfb_computer_graphics_demo.ipynb)

Colab badge note: if the repository owner, repository name, branch, or notebook path changes, update the badge URL to the new GitHub location.

`dsfb-computer-graphics` is a crate-local Rust evaluation artifact for DSFB supervision in temporal reuse and bounded fixed-budget sampling control. The upgraded crate is designed to remove concrete reviewer blockers rather than rely on vague polish:

- host-realistic DSFB mode separated from visibility-assisted research mode
- stronger Demo A baselines, not only fixed alpha
- deterministic multi-scenario suite instead of a single favorable reveal
- explicit ablations showing which cues matter and which are expendable
- fixed-budget Demo B policies compared against nontrivial cheap alternatives
- attachability and cost surfaces documented as real interfaces, not hand-wavy prose
- blocker-oriented reports, reviewer bundle generation, and hard validation gates

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Scope Boundary

This crate is intentionally self-contained inside `crates/dsfb-computer-graphics`.

- Only files inside this crate are required for scene generation, Demo A, Demo B, metrics, figures, reports, notebook orchestration, and validation.
- The crate declares its own nested `[workspace]` so validation can be run from this directory without modifying the workspace root.
- If a broader workspace integration would normally be useful, that limitation is documented here rather than solved by editing outside the crate.

## Colab Notebook

The crate includes a real Google Colab notebook at `colab/dsfb_computer_graphics_demo.ipynb`.

The notebook:

- installs Rust and the small runtime dependencies it needs
- clones the repository and builds the crate
- creates a timestamped crate-local run directory under `output-dsfb-computer-graphics/`
- runs `cargo run -- run-all --output <run-dir>`
- generates a real PDF reviewer bundle and ZIP archive
- displays the major figures inline
- exposes `Download PDF` and `Download ZIP` controls for the current run

For notebook details and assumptions, see `docs/colab_notebook.md`.

## What This Crate Does

### Demo A

Demo A studies temporal reuse supervision. The crate runs:

- fixed-alpha baseline
- residual-threshold baseline
- neighborhood-clamped baseline
- depth/normal rejection baseline
- reactive-mask-style baseline
- strong heuristic baseline
- DSFB visibility-assisted research mode
- DSFB host-realistic mode
- DSFB ablations: no visibility, no thin proxy, no motion disagreement, no grammar, residual-only, and trust-without-alpha-modulation

The scenario suite contains:

- `thin_reveal`
- `fast_pan`
- `diagonal_reveal`
- `contrast_pulse`
- `stability_holdout`

### Demo B

Demo B is a fixed-budget allocation study. At equal total sample budget, the crate compares:

- uniform allocation
- edge-guided allocation
- residual-guided allocation
- contrast-guided allocation
- variance-guided allocation
- combined-heuristic allocation
- imported-trust allocation
- hybrid trust-plus-variance allocation

## Quickstart

Run the full reviewer package:

```bash
cd crates/dsfb-computer-graphics
cargo run -- run-all --output generated
```

Run only Demo A across the full scenario suite:

```bash
cd crates/dsfb-computer-graphics
cargo run -- run-demo-a --output generated
```

Run Demo A on one scenario only:

```bash
cd crates/dsfb-computer-graphics
cargo run -- run-demo-a --scenario thin_reveal --output generated/single_scenario
```

Run the canonical ablation package:

```bash
cd crates/dsfb-computer-graphics
cargo run -- run-ablations --output generated/ablations
```

Run only Demo B:

```bash
cd crates/dsfb-computer-graphics
cargo run -- run-demo-b --output generated
```

Validate a generated artifact directory:

```bash
cd crates/dsfb-computer-graphics
cargo run -- validate-artifacts --output generated
```

Run crate-scoped validation:

```bash
cd crates/dsfb-computer-graphics
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Output Structure

Running `run-all` writes a reviewer-oriented bundle like this:

```text
generated/
  artifact_manifest.json
  scene_manifest.json
  scenario_suite_manifest.json
  metrics.json
  report.md
  reviewer_summary.md
  five_mentor_audit.md
  check_signing_blockers.md
  ablation_report.md
  cost_report.md
  demo_b_decision_report.md
  completion_note.md
  figures/
  scenarios/
  demo_b/
```

The Colab notebook writes the same structure under a timestamped run directory inside `output-dsfb-computer-graphics/` and then adds:

- `artifacts_bundle.pdf`
- `output-dsfb-computer-graphics-YYYYMMDD-HHMMSS.zip`

The timestamped layout prevents accidental overwrite.

## Exact Numeric Demo Summary

For the current deterministic default run:

- On the canonical `thin_reveal` scenario, host-realistic DSFB reduced cumulative ROI MAE from `2.84366` for fixed alpha and `0.49435` for the strong heuristic baseline to `0.31904`.
- On the same scenario, ghost persistence fell from `12` frames for fixed alpha and `1` frame for the strong heuristic baseline to `0` frames for host-realistic DSFB.
- Across the five-scenario suite, host-realistic DSFB beat fixed alpha on `3` scenarios and beat the strong heuristic baseline on `3` scenarios, while `contrast_pulse` and `stability_holdout` are surfaced explicitly as bounded / neutral cases.
- For Demo B on the canonical `thin_reveal` sampling case, imported trust reduced ROI MAE from `0.17226` for uniform allocation to `0.03184` at the same total budget, versus `0.04977` for the combined-heuristic allocator.

Canonical Demo A table:

| Metric | Fixed alpha | Strong heuristic | Host-realistic DSFB |
| --- | ---: | ---: | ---: |
| Ghost persistence frames | 12 | 1 | 0 |
| Peak ROI error | 0.43507 | 0.13843 | 0.07198 |
| Cumulative ROI error | 2.84366 | 0.49435 | 0.31904 |
| Average non-ROI MAE | 0.01244 | 0.00062 | 0.00219 |

Canonical Demo B table:

| Metric | Uniform | Combined heuristic | Imported trust |
| --- | ---: | ---: | ---: |
| ROI MAE | 0.17226 | 0.04977 | 0.03184 |
| ROI RMSE | 0.17226 | 0.04977 | 0.03184 |
| ROI mean spp | 2.00 | 10.00 | 12.00 |

## DSFB Integration into Temporal Reuse

Baseline temporal blend equation:

```text
C_t(u) = alpha * C_t_current(u) + (1 - alpha) * C_{t-1}_reproj(u)
```

DSFB trust-modulated blend equation:

```text
C_t(u) = alpha_t(u) * C_t_current(u) + (1 - alpha_t(u)) * C_{t-1}_reproj(u)
alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - T_t(u))
```

High trust keeps the resolve closer to history preservation. Low trust pushes the resolve toward current-frame replacement. The underlying estimator is unchanged.

This crate demonstrates a supervisory blend modulation layer. It does not require replacing the underlying renderer or estimator.

For the explicit typed input/output surface, see `docs/integration_surface.md`.

## GPU Implementation Considerations

Execution model:

- per-pixel local supervision
- optional per-tile aggregation for reduced overhead
- async-compute-compatible decomposition

Memory layout:

- residual buffer
- trust buffer
- alpha buffer
- intervention buffer
- optional depth / normal / motion disagreement buffers
- optional tile summaries

Optimization strategies:

- half resolution trust
- tile aggregation
- temporal reuse of proxy

Approximate cost table:

| Operation group | Per-pixel / per-tile character | Memory footprint class | Reduction strategy |
| --- | --- | --- | --- |
| Residual evaluation | per-pixel local arithmetic | one scalar buffer | fuse with resolve where practical |
| Structural disagreement synthesis | per-pixel with neighborhood reads | several scalar buffers | tile aggregation |
| Trust and alpha update | per-pixel or per-tile | trust plus alpha | half-resolution trust |
| Debug / export surfaces | optional | debug-only expansion | disable in deployment path |

“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”

“The framework is compatible with tiled and asynchronous GPU execution.”

All cost claims in this crate are architectural or approximate. They are not measured production benchmarks.

For the full analytical model, see `docs/cost_model.md`.

## Mission and Transition Relevance

This crate is relevant to reliability and assurance in visual pipelines because it exposes replayable supervisory evidence instead of only a final image. That makes it relevant to:

- early detection of estimator failure
- auditable temporal-reuse behavior
- safety-adjacent or mission-adjacent visual stacks where bounded replay matters
- transition diligence where reviewers need to inspect evidence rather than trust claims

The crate is a synthetic feasibility artifact. It is not a fielded mission system.

## Product Framing and Integration Surfaces

In product terms, this crate demonstrates the shape of an attachable supervisory trust layer:

- temporal reuse supervision
- adaptive routing / fixed-budget allocation surface
- logging and traceability surface
- attachable middleware concept rather than renderer replacement

| Surface | Current crate coverage | Future extension |
| --- | --- | --- |
| TAA / temporal reuse | implemented in Demo A | connect to engine traces and reprojection buffers |
| adaptive sampling / SAR | implemented as bounded Demo B fixed-budget study | extend to temporal controllers and richer sample policies |
| logging / QA | metrics, figures, reports, manifests | engine-integrated trace and regression pipeline |
| adaptive compute routing | trust / intervention / allocation difficulty surfaces | cross-pass scheduling and runtime policy |

## Validation and Reviewer Reports

The main reviewer-facing outputs are:

- `generated/report.md`
- `generated/reviewer_summary.md`
- `generated/five_mentor_audit.md`
- `generated/check_signing_blockers.md`
- `generated/ablation_report.md`
- `generated/demo_b_decision_report.md`
- `generated/cost_report.md`

The strict artifact check is:

```bash
cargo run -- validate-artifacts --output generated
```

## What this crate does not claim

- It does not claim production readiness.
- It does not claim measured GPU benchmark wins.
- It does not claim superiority over every strong heuristic on every scenario.
- It does not claim field validation, engine deployment, or licensing closure.
- It does not claim that the synthetic visibility-assisted mode is a deployable cue source.

## Limitations

- The suite is still synthetic and deterministic rather than engine-captured or field-recorded.
- The strong heuristic baseline remains competitive on some scenarios; the crate surfaces that explicitly.
- The cost model is architectural, not hardware-profiled.
- Demo B is a bounded fixed-budget controller study, not a full temporal sampling policy.

## Future Work

- Run the same supervisory layer against richer external scene corpora or engine trace captures.
- Profile real GPU implementations and label those results explicitly as measured hardware data.
- Test half-resolution trust and tile aggregation on actual hardware.
- Package the host interface into an engine-adjacent prototype for transition diligence.
