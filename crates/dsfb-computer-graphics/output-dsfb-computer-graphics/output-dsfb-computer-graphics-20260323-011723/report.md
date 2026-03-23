# DSFB Computer Graphics Report

## Overview

This crate is a bounded transition artifact for temporal reuse supervision. It packages a deterministic scene, a fixed-alpha baseline, a stronger residual-threshold baseline, a DSFB supervisory path, real generated figures, and replayable metrics so a reviewer can evaluate the behavior quickly.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

What is demonstrated: a deterministic reveal event in which stale temporal history persists on thin geometry for the fixed-alpha baseline, while the DSFB supervisory signal lowers trust, raises the current-frame blend weight, and reduces persistence error.

What is not demonstrated: production-optimal tuning, field readiness, GPU benchmarks, or superiority against a full commercial temporal reconstruction stack.

What remains future work: engine integration, broader scenes, measured hardware studies, and larger comparative baselines.

## Numeric Demo Summary

| Metric | Fixed-alpha baseline | Residual-threshold baseline | DSFB |
| --- | ---: | ---: | ---: |
| Ghost persistence frames | 12 | 4 | 0 |
| Peak ROI error | 0.39133 | 0.12452 | 0.06475 |
| Cumulative ROI error | 2.55778 | 0.70179 | 0.21577 |
| Average overall MAE | 0.01228 | 0.00373 | 0.00220 |

- Reveal frame: 6
- Trust-drop frame: 6
- Trust-minimum frame: 6
- Residual-baseline response frame: 6
- Trust/error correlation at reveal: 0.9071

In this bounded synthetic setting, DSFB reduced ghost persistence duration from 12 to 0 frames relative to the fixed-alpha baseline.

Against the residual-threshold baseline, DSFB reduced ghost persistence duration from 4 to 0 frames.

## Canonical Scene

The canonical sequence contains a moving foreground object, a deterministic disocclusion event, a one-pixel vertical structure, a one-pixel diagonal structure, and a persistence ROI derived from the revealed thin pixels.

- Resolution: 160 x 96
- Frame count: 18
- Persistence mask pixels: 50

## DSFB State Exports

The crate exports first-class DSFB state rather than only the final gated image. Under `generated/frames/`, the run writes residual, trust, alpha, intervention, residual-proxy, visibility-proxy, motion-edge-proxy, thin-proxy, and simplified structural-state images for every frame.

The simplified structural-state field is crate-scoped and intentionally honest rather than universal. It uses the labels `nominal`, `disocclusion-like`, `unstable-history`, and `motion-edge` as a bounded grammar for this artifact.

## DSFB Integration into Temporal Reuse

Baseline temporal blend equation with a fixed blend weight:

```text
C_t(u) = alpha * C_t_current(u) + (1 - alpha) * C_{t-1}_reproj(u)
```

DSFB trust-modulated blend equation with the same underlying estimator:

```text
C_t(u) = alpha_t(u) * C_t_current(u) + (1 - alpha_t(u)) * C_{t-1}_reproj(u)
alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - T_t(u))
```

High trust means the supervisory layer keeps the blend close to the history-preserving setting. Low trust means the supervisory layer increases the current-frame weight so revealed or unstable regions flush stale history sooner.

The underlying estimator is unchanged. The crate demonstrates a supervisory blend modulation layer that can sit on top of an existing temporal reuse path without replacing the underlying renderer or estimator.

## Figures

- `fig_system_diagram.svg`: Inputs → Residuals → Proxies → Grammar → Trust → Intervention.
- `fig_trust_map.svg`: trust overlay on the actual reveal frame with disocclusion and motion-edge highlights.
- `fig_before_after.svg`: baseline fixed-alpha output versus DSFB on the same comparison frame and ROI.
- `fig_trust_vs_error.svg`: frame index on the x-axis, ROI error on the left y-axis, DSFB ROI trust on the right y-axis.

## GPU Implementation Considerations

### Execution Model

The supervisory path is organized around local per-pixel operations and can also be lifted to a per-tile realization. Residuals, proxies, trust, and blend modulation all depend on local evidence plus bounded temporal history, which makes the design a plausible async-compute candidate in a larger frame graph.

### Memory Layout

- Residual buffer: scalar discrepancy between current and reprojected history.
- Proxy buffer: residual, visibility, motion-edge, and thin-structure cues.
- Trust buffer: scalar supervisory field used to derive alpha modulation.
- Optional history and tile-summary buffers: bounded temporal memory plus coarse reduction outputs.

### Optimization Strategies

- half resolution trust
- tile aggregation
- temporal reuse of proxy

### Cost Table

| Operation group | Per-pixel / per-tile character | Memory footprint class | Reduction strategy |
| --- | --- | --- | --- |
| Residual evaluation | per-pixel local arithmetic | one scalar buffer | half resolution trust when full precision is unnecessary |
| Proxy synthesis | per-pixel with optional neighborhood lookups | packed proxy channels | reuse and compact proxy packing |
| Grammar and trust update | per-pixel or per-tile aggregation | one trust buffer plus optional tile summaries | tile aggregation |
| Blend modulation | per-pixel scalar modulation | no extra color history beyond temporal reuse itself | fuse with existing resolve pass |

“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”

“The framework is compatible with tiled and asynchronous GPU execution.”

All cost discussion in this crate is architectural or approximate. It is not presented as measured production benchmarking.

## Mission and Transition Relevance

This artifact is relevant to reliability and assurance in visual pipelines because it surfaces replayable residual, proxy, trust, and intervention evidence rather than only a final image. That supports early detection of estimator failure modes, bounded auditability, and after-action review for safety-adjacent or mission-adjacent visual systems.

The crate is a synthetic feasibility artifact, not a fielded mission system. It illustrates a bounded feasibility demonstration for supervisory evidence in temporal reuse rather than deployment readiness.

## Product Framing and Integration Surfaces

In product terms, this implementation demonstrates the shape of an attachable supervisory trust layer: a middleware-style surface that can modulate temporal reuse, emit traces, and expose a routing signal for adaptive compute without replacing the base estimator.

| Surface | Current crate coverage | Future extension |
| --- | --- | --- |
| TAA / temporal reuse | implemented in Demo A | extend to engine integration and richer reprojection paths |
| adaptive sampling / SAR | implemented as bounded Demo B fixed-budget reveal-frame study | extend to temporal policy and broader sampling controllers |
| logging / QA | implemented through generated metrics, figures, and reports | extend to engine traces, fleet replay, and automated regression checks |
| adaptive compute routing | partially illustrated through trust and intervention fields | extend to budget schedulers and cross-pass policy |

## What this crate does not claim

- It does not claim funding, licensing, or instant transition outcomes.
- It does not claim production-optimal TAA or temporal reconstruction.
- It does not claim measured GPU timings or hardware-specific performance wins.
- It does not claim readiness for mission deployment or safety certification.

## Limitations

- The scene is deterministic and synthetic rather than photoreal or field captured.
- The residual-threshold baseline is stronger than fixed alpha but still not a full commercial anti-ghosting stack.
- The structural grammar is intentionally simplified and scoped to this crate.
- Demo B is bounded to a reveal-frame fixed-budget study.

## Future Work

- Extend the crate to richer scenes and engine-connected reprojection data while preserving replayability.
- Add additional comparative baselines such as variance gating, neighborhood clipping, or learned confidence predictors.
- Measure an actual GPU implementation and label it explicitly as measured hardware data.
