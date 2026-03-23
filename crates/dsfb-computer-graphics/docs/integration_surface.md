# Integration Surface

This document describes the attachable host-style interface implemented inside `src/host.rs` and used by the upgraded Demo A / Demo B artifact flow.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Purpose

The point of the integration surface is to show that DSFB in this crate is not only a paper-like confidence story. It is expressed as typed inputs and outputs that match the shape of a real temporal supervision layer:

- current frame inputs
- history / reprojection inputs
- local supervisory cues
- trust and intervention outputs
- alpha modulation for temporal reuse
- optional difficulty surfaces for fixed-budget routing

The crate does not claim engine integration already exists. It demonstrates the expected attachment points honestly and explicitly.

## Required Inputs

The host-realistic path consumes the following inputs per frame:

- `current_color`: the current-frame color buffer
- `reprojected_history`: the history buffer already reprojected into current-frame space
- `motion_vectors`: the reprojection / motion map
- `current_depth`: current depth buffer
- `reprojected_depth`: previous depth buffer reprojected into current-frame space
- `current_normals`: current normal buffer or synthetic normal equivalent
- `reprojected_normals`: previous normal buffer reprojected into current-frame space

These are represented directly by `HostTemporalInputs` in `src/host.rs`.

## Optional Inputs

The research/debug mode in this crate also supports optional cue channels:

- `visibility_hint`: synthetic visibility-like cue used only for explicit comparison / ablation
- `thin_hint`: crate-local thin-structure hint used to separate research/debug behavior from host-realistic behavior

The visibility hint is intentionally documented as a research cue rather than a deployment claim.

## Outputs

The supervisory pass produces:

- `residual`: local current-vs-history discrepancy
- `trust`: supervisory trust field in `[0, 1]`
- `alpha`: temporal blend modulation field
- `intervention`: hazard / response-strength field
- `proxies`: residual, visibility, depth, normal, motion, neighborhood, thin, and instability proxy channels
- `state`: simplified structural-state labels

These are returned by `HostSupervisionOutputs` and are consumed by the pipeline as real artifacts, not only transient debug values.

## Temporal Reuse Attachment

### Baseline equation

```text
C_t(u) = alpha * C_t_current(u) + (1 - alpha) * C_{t-1}_reproj(u)
```

### DSFB-modulated equation

```text
C_t(u) = alpha_t(u) * C_t_current(u) + (1 - alpha_t(u)) * C_{t-1}_reproj(u)
alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - T_t(u))
```

High trust keeps the resolve closer to history preservation. Low trust increases current-frame replacement pressure. The underlying estimator is unchanged.

In other words: the crate demonstrates a supervisory modulation layer rather than a replacement renderer.

## Sampling / Routing Attachment

Demo B uses the same supervisory surface as a difficulty source for fixed-budget allocation. The crate demonstrates three categories of allocation control:

- imported temporal trust
- sampling-native cheap heuristics
- hybrid trust plus variance

That means the same interface can feed:

- temporal blend modulation
- adaptive sampling or reconstruction budgeting
- logging / traceability surfaces

## Buffer Semantics

| Buffer | Meaning | Typical precision | Notes |
| --- | --- | --- | --- |
| current color | current-frame estimate | RGB16F or RGB32F equivalent | existing host buffer |
| reprojected history | history in current-frame space | RGB16F or RGB32F equivalent | existing host buffer |
| motion vectors | reprojection map | RG16F / RG32F equivalent | existing host buffer |
| depth | current / reprojected depth | R32F equivalent | existing host buffer |
| normals | current / reprojected normals | RGB10A2 / RGB16F equivalent | optional but useful |
| residual | local discrepancy field | R16F / R32F | supervisory |
| trust | supervisory trust | R16F / R32F | supervisory |
| alpha | blend modulation | R16F / R32F | can be fused into resolve |
| intervention | response strength | R16F / R32F | optional runtime debug |
| tile summary | reduced-resolution or per-tile aggregate | small structured buffer | optional optimization |

## Execution Order

One plausible pass sequence is:

1. reproject history, depth, and optional normals
2. compute residual and local disagreement proxies
3. classify simplified structural state
4. combine cues into trust and intervention
5. derive alpha modulation
6. apply temporal resolve
7. optionally emit trust / intervention to logging or routing consumers

This is exactly the order reflected by the system diagram and the crate pipeline.

## Reduced-Resolution / Per-Tile Opportunities

The following are honest candidates for reduced-resolution or per-tile realization:

- trust field
- intervention field
- neighborhood / motion aggregation
- optional sampling-budget map

The following are most naturally kept per pixel:

- residual
- final alpha modulation
- temporal resolve output

## GPU Pass Decomposition

Likely pass decomposition in an engine context:

- pass 1: reprojection and local cue synthesis
- pass 2: trust / intervention / alpha update
- pass 3: temporal resolve
- pass 4: optional logging, tile summaries, or routing export

The framework is compatible with tiled and asynchronous GPU execution, but this crate does not claim that those passes have been benchmarked on hardware.

## What This Document Proves

- the input/output surface is concrete and typed
- the attachment point in a temporal pipeline is explicit
- trust, alpha, intervention, and routing surfaces are all real crate outputs

## What This Document Does Not Prove

- production engine integration is already complete
- the current interface is optimal for every renderer
- the pass decomposition has been tuned on real hardware
