# dsfb-computer-graphics

`dsfb-computer-graphics` is a crate-local Rust transition artifact for DSFB supervision in temporal reuse. It packages a deterministic Demo A scene, a fixed-alpha TAA baseline, a stronger residual-threshold baseline, a DSFB trust-gated path that only changes the supervisory control layer, generated figures, replayable metrics, and reviewer-facing summaries.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Scope Boundary

This crate is intentionally self-contained inside `crates/dsfb-computer-graphics`.

- Only files inside this crate are required for the demo, figures, metrics, report, reviewer summary, and completion note.
- The crate declares its own nested `[workspace]` so the validation commands can run from this directory without modifying the workspace root.
- If a broader workspace integration would normally be desirable, that limitation is documented here rather than solved by editing outside the crate.

## Crate Purpose

The purpose of this crate is to remove the main reasons a reviewer would hesitate:

- the demo is deterministic and quick to rerun
- the control-path change is explicit rather than hidden inside a larger renderer
- the DSFB state is exported as residual, proxy, trust, alpha, intervention, and structural-state artifacts
- the numeric result is computed and reported rather than left qualitative
- the GPU and transition framing is cost-honest and does not fabricate timings

## Quickstart

Run the full Demo A package:

```bash
cd crates/dsfb-computer-graphics
cargo run -- run-demo-a
```

Run the bounded Demo B adaptive-sampling study:

```bash
cd crates/dsfb-computer-graphics
cargo run -- run-demo-b
```

Run the crate-scoped validation commands:

```bash
cd crates/dsfb-computer-graphics
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Generated Artifacts

The crate writes outputs under `generated/`:

- `generated/frames/gt/`: canonical deterministic scene sequence
- `generated/frames/baseline/`: fixed-alpha baseline outputs
- `generated/frames/residual_baseline/`: residual-threshold baseline outputs
- `generated/frames/dsfb/`: DSFB outputs
- `generated/frames/residual/`, `proxy_*`, `trust/`, `alpha/`, `intervention/`, `state/`: exported DSFB state
- `generated/figures/fig_system_diagram.svg`
- `generated/figures/fig_trust_map.svg`
- `generated/figures/fig_before_after.svg`
- `generated/figures/fig_trust_vs_error.svg`
- `generated/metrics.json`
- `generated/report.md`
- `generated/reviewer_summary.md`
- `generated/completion_note.md`
- `generated/demo_b/metrics.json`
- `generated/demo_b/report.md`
- `generated/demo_b/figures/fig_demo_b_sampling.svg`

## Exact Numeric Demo Summary

For the default deterministic run checked into this crate:

- In this bounded synthetic setting, DSFB reduced ghost persistence duration from 12 to 0 frames relative to the fixed-alpha baseline.
- Against the residual-threshold baseline, DSFB reduced ghost persistence duration from 4 to 0 frames.

| Metric | Fixed-alpha baseline | Residual-threshold baseline | DSFB |
| --- | ---: | ---: | ---: |
| Ghost persistence frames | 12 | 4 | 0 |
| Peak ROI error | 0.39133 | 0.12452 | 0.06475 |
| Cumulative ROI error | 2.55778 | 0.70179 | 0.21577 |
| Average overall MAE | 0.01228 | 0.00373 | 0.00220 |

Additional event timing from the same run:

- reveal frame: 6
- trust-drop frame: 6
- trust-minimum frame: 6
- residual-baseline response frame: 6
- trust/error correlation at reveal: 0.9071

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

High trust keeps the blend closer to history preservation. Low trust pushes the blend toward current-frame replacement so revealed or unstable regions flush stale history earlier.

The underlying estimator is unchanged. The crate demonstrates a supervisory blend modulation layer, and it does not require replacing the underlying renderer or estimator.

## GPU Implementation Considerations

### Execution Model

The supervisory layer is structured around local per-pixel operations and can also be lifted to a per-tile realization. That makes it a plausible candidate for async-compute placement in a larger frame graph.

### Memory Layout

- residual buffer
- proxy buffer
- trust buffer
- optional history and tile-summary buffers

### Optimization Strategies

- half resolution trust
- tile aggregation
- temporal reuse of proxy

### Cost Table

| Operation group | Per-pixel / per-tile character | Memory footprint class | Reduction strategy |
| --- | --- | --- | --- |
| Residual evaluation | per-pixel local arithmetic | one scalar buffer | half resolution trust |
| Proxy synthesis | per-pixel with optional neighborhood lookups | packed proxy channels | compact packing and reuse |
| Grammar and trust update | per-pixel or per-tile aggregation | one trust buffer plus optional tile summaries | tile aggregation |
| Blend modulation | per-pixel scalar modulation | no extra color history beyond temporal reuse itself | fuse with existing resolve pass |

“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”

“The framework is compatible with tiled and asynchronous GPU execution.”

All cost claims in this crate are architectural or approximate. They are not measured production benchmarks.

## Mission and Transition Relevance

This crate is relevant to reliability and assurance in visual pipelines because it exposes replayable supervisory evidence rather than only a final image. That supports early detection of estimator failure modes, auditability, and after-action review for safety-adjacent or mission-adjacent visual systems.

The current crate is a synthetic feasibility artifact, not a fielded mission system. It illustrates a bounded feasibility demonstration for supervisory evidence in temporal reuse.

## Product Framing and Integration Surfaces

In product terms, this crate demonstrates the shape of an attachable supervisory trust layer: a middleware-style surface for temporal reuse supervision, logging, and adaptive compute routing that can sit on top of an existing estimator.

| Surface | Current crate coverage | Future extension |
| --- | --- | --- |
| TAA / temporal reuse | implemented in Demo A | extend to engine integration |
| adaptive sampling / SAR | implemented as bounded Demo B reveal-frame study | extend to temporal policy and broader sampling controllers |
| logging / QA | implemented through metrics, figures, and reports | extend to engine traces and regression pipelines |
| adaptive compute routing | partially illustrated through trust and intervention fields | extend to budget schedulers and cross-pass policy |

## What this crate does not claim

- It does not claim funding, licensing, or instant transition outcomes.
- It does not claim production-optimal TAA or temporal reconstruction.
- It does not claim measured GPU timings or hardware-specific wins.
- It does not claim readiness for mission deployment or safety certification.
- It does not claim that Demo A replaces a full commercial rendering stack.

## Limitations

- The scene is deterministic and synthetic rather than photoreal or field captured.
- The residual-threshold baseline is stronger than fixed alpha but still not a full commercial anti-ghosting stack.
- The structural grammar is simplified and scoped to this crate.
- Demo B is a bounded reveal-frame fixed-budget study rather than a temporal SAR system.

## Future Work

- Extend the artifact to richer scenes and engine-connected reprojection data while preserving replayability.
- Add broader comparative baselines such as variance gating, neighborhood clipping, or learned confidence predictors.
- Measure an actual GPU implementation and label the results explicitly as hardware measurements.
- Expand Demo B from a reveal-frame study into a temporal adaptive-sampling controller.

## Reviewer Notes

- The fastest path for a technical reviewer is `generated/reviewer_summary.md`.
- The canonical validated package for the default run is `generated/report.md`, `generated/metrics.json`, and the four figure files under `generated/figures/`.
- The crate-local boundary and validation checklist are recorded in `generated/completion_note.md`.
