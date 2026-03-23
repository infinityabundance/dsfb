# Cost Model

This note documents the analytical cost accounting used by the crate-local cost report.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Scope

The cost model in this crate is architectural, not benchmark-driven. It is intended to remove ambiguity about buffer count, per-stage work, and likely integration burden without inventing GPU timings.

It supports three modes:

- minimal
- host-realistic
- full research/debug

These are implemented in `src/cost.rs` and emitted as `generated/cost_report.md` by the pipeline.

## Mode Definitions

### Minimal

The minimal mode corresponds to a stripped supervisory path:

- residual-like cue
- response field
- alpha modulation

This is the lowest-burden integration sketch and is useful for answering whether a small supervisory layer is plausible at all.

### Host-realistic

The host-realistic mode is the serious attachability target for this crate:

- residual
- depth disagreement
- normal disagreement
- motion disagreement
- neighborhood inconsistency
- trust
- alpha
- intervention

This mode excludes the synthetic visibility hint.

### Full research/debug

The full research/debug mode keeps additional cue exports and debug surfaces:

- visibility hint for explicit comparison
- thin proxy
- history instability proxy
- state labels

This mode is intended for ablation and report generation, not as a production-cost claim.

## Core Cost Statement

“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”

This is an architectural statement, not a measured production benchmark.

## Compatibility Statement

“The framework is compatible with tiled and asynchronous GPU execution.”

Again, this crate does not claim measured scheduling wins. It only claims architectural compatibility.

## Buffer Accounting

The crate cost model makes buffer questions explicit:

- which buffers are required in each mode
- bytes per pixel for each additional field
- which fields can be fused or omitted
- how footprints scale from 720p to 4K

This matters because one of the main blockers for systems reviewers is uncertainty about hidden memory or bandwidth burden.

## Stage Accounting

The model also exposes approximate stage groups:

- residual evaluation
- structural disagreement synthesis
- trust / grammar update
- alpha modulation
- optional debug writes

The reported counts are approximate arithmetic / read / write groups. They are not cycle-accurate.

## Reduction Opportunities

The crate explicitly documents three reduction strategies:

- half-resolution trust
- tile aggregation
- temporal reuse of proxy

These reductions are discussed because they are the most plausible first-step controls for a real implementation burden.

## What This Model Helps Decide

This model is intended to answer:

- what extra buffers are actually needed
- whether the extra work is local or globally coupled
- what the likely resolution scaling is
- which research/debug surfaces are optional in a deployment path

## What This Model Does Not Prove

- real GPU milliseconds on any hardware target
- final memory-system behavior on shipping architectures
- fusion decisions inside a specific commercial engine
- production-optimal pass scheduling
