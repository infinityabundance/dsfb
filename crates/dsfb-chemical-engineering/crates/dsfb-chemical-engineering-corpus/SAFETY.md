# Safety — dsfb-chemical-engineering-corpus

Advisory, read-only; **no control or safety-instrumented-function authority** (never gates an interlock).
Authority (no_std): the provenance-tiered soft-sensor dataset catalogue, with the frozen corpus_hash_v1.

Unsafe/UB posture: #![forbid(unsafe_code)]. Determinism is fixed-point with byte-exact replay (not claimed bit-reproducible for
arbitrary floating-point pipelines). This is alignment guidance, not a certification — see the canonical
[`../../SAFETY.md`](../../SAFETY.md) and `paper/sections/limitations.tex`.
