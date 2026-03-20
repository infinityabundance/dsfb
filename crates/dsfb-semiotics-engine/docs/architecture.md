# Architecture Overview

`dsfb-semiotics-engine` keeps the paper-aligned layers explicit:

```text
observed + predicted
        |
        v
    residual layer
        |
        v
      sign layer
        |
        v
     syntax layer
        |
        v
     grammar layer
        |
        v
    semantics layer
        |
        v
 artifact / report / evaluation export
```

## Module Responsibilities

- `src/math`
  Deterministic numerical primitives, norms, envelopes, finite differences, and helper metrics.
- `src/engine`
  Typed layered engine objects, threshold settings, heuristic-bank governance, and orchestration.
- `src/evaluation`
  Post-run deterministic summaries, internal comparator baselines, and sweep summarization.
- `src/io`
  CSV ingest, output layout, schema markers, CSV/JSON writers, and zip packaging.
- `src/figures`
  Deterministic figure generation with captions aligned to the actual implementation.
- `src/report`
  Markdown and PDF artifact reporting with explicit limitations and provenance.
- `src/sim`
  Synthetic scenario generation and sweep members.

## Design Intent

- Keep deterministic engine logic separate from evaluation summaries.
- Keep semantics retrieval separate from syntax characterization.
- Keep export formatting separate from engine math.
- Keep public types explicit enough for line-by-line review.

## Extension Discipline

- Add syntax metrics only when their deterministic meaning can be stated in one sentence.
- Add heuristic-bank entries only through typed registry entries with provenance and compatibility notes.
- Prefer additive schema evolution over breaking output rewrites.
- Preserve the layered architecture even when adding new examples, reports, or evaluation summaries.
