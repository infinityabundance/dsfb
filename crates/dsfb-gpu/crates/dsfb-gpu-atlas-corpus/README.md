# dsfb-gpu-atlas-corpus

Literature detector corpus and deterministic canonicalisation court for
DSFB-GPU-Atlas.

This crate is host-only and has no external Rust dependencies. It
ships the T.10-frozen corpus anchor, deterministic court surfaces,
provenance-bound artifact emitters, and invariant tests that keep the
claim boundary explicit.

## Scope

- Maintains the literature detector corpus and canonical IDs.
- Emits deterministic text and JSON court artifacts under versioned
  hash domains.
- Records constraints, contraindications, coverage holes, activation
  planning, and related audit surfaces.

## Non-claims

The Atlas corpus records deterministic witness authority. It does not
claim learned detector usefulness, medical diagnosis, root-cause
certainty, production CUDA performance, or probabilistic inference.

## Publish order

Publish after `dsfb-gpu-debug-core = 0.1.0` is visible on crates.io.
`dsfb-gpu-atlas-registry` depends on this crate.
