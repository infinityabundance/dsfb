# dsfb-gpu-atlas-registry

Detector algebra and S1.2 registry generator for DSFB-GPU-Atlas.

This crate generates the 162-spec literature-bound detector registry
from the T.10 corpus anchor and a bounded parameter grid. It is
host-only and does not link to CUDA.

## Scope

- Defines deterministic detector algebra types.
- Generates `DetectorSpec` records from the corpus-bound S1.2 grid.
- Emits `registry_hash_v2` over canonical registry bytes.
- Verifies that registry-bound specs carry the live corpus hash.

## Non-claims

This crate does not execute detector kernels, admit episodes, generate
unbounded detector counts, or claim measured detector usefulness. It
spells and verifies registry authority for later execution surfaces.

## Publish order

Publish after both `dsfb-gpu-debug-core = 0.1.0` and
`dsfb-gpu-atlas-corpus = 0.1.0` are visible on crates.io.
