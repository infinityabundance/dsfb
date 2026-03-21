# DSFB Auditable INS Residual Interpretation Layer

Modern A-PNT and INS stacks often expose residual streams without a lightweight, auditable layer
that preserves whether a departure looked like persistent drift, bounded oscillation, abrupt
switching, or admissibility-qualified ambiguity. `dsfb-semiotics-engine` demonstrates that such a
layer can be implemented as a deterministic software component rather than as an opaque learned
classifier.

What the crate concretely demonstrates:

- a residual -> sign -> syntax -> grammar -> semantics pipeline with explicit typed artifacts
- strict-by-default heuristic-bank governance with exported validation reports
- bounded online memory through a fixed-capacity ring buffer
- a default bounded live window of `64` samples in the deployment-oriented path
- a C ABI, C++ wrapper, dashboard replay, and CSV-to-report path suitable for systems integration

Interface boundary:

- the engine can sit behind an opaque C handle or the single-header C++ wrapper
- grammar reasons and trust scalars are exported explicitly
- artifact generation remains separate from the bounded online path

Demonstration boundary:

- this crate is an auditable deterministic interpretation layer
- it is not formal certification, field validation, diagnosis, or unique latent-cause recovery
