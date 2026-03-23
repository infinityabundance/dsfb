# Cost Model

This document describes the analytical cost and memory model used by the crate reports.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Scope

The cost model in this crate is architectural. It is not a measured GPU benchmark. Its job is to make the integration burden explicit enough for diligence:

- which buffers exist
- which stages are local
- what the approximate read/write pressure looks like
- what changes between minimum, host-realistic, and research/debug modes

## Modes

### Minimal

Smallest decision-facing path:

- residual-like local discrepancy
- trust / intervention proxy
- alpha modulation

This corresponds to the lowest-burden attachment path and is used in the timing report as the minimum cost reference.

### Host-Realistic

Current minimum serious path:

- residual
- depth disagreement
- normal disagreement
- neighborhood inconsistency
- thin proxy
- history instability
- grammar/state contribution
- trust, intervention, alpha

Important current decision:

- motion disagreement is not part of the minimum path anymore
- it remains available as an optional motion-augmented extension

That change is deliberate. The current suite does not justify treating motion disagreement as mandatory in the minimum path.

### Full Research / Debug

Comparison-only path:

- synthetic visibility hint
- optional motion disagreement
- thin proxy exports
- history instability exports
- structural-state exports

This mode exists for ablation, trust diagnostics, and report generation. It is not a deployment claim.

## Current Trust and Cost Interaction

The current trust behavior is near-binary / gate-like in this crate. That matters for cost because it makes two reduction ideas more credible:

- half-resolution trust or intervention
- per-tile trust aggregation followed by alpha upsampling

The crate does not claim these are already tuned on hardware. It only shows that the dataflow is compatible with them.

## Core Statements

“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”

“The framework is compatible with tiled and asynchronous GPU execution.”

These are architecture statements, not measured deployment claims.

## What The Cost Model Helps Decide

- whether the supervision is local enough to be a realistic GPU pass candidate
- whether the minimum path is materially smaller than the debug path
- what the memory scaling looks like at larger resolutions
- which buffers are plausibly droppable outside analysis mode

## What The Cost Model Does Not Prove

- real GPU milliseconds
- cache behavior on NVIDIA, AMD, or Intel hardware
- production pass scheduling quality
- shipping-engine memory-system efficiency
