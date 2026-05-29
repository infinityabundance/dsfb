# Safety — dsfb-chemical-engineering-atlas

Advisory, read-only; **no control or safety-instrumented-function authority** (never gates an interlock).
Authority (no_std): the curated detector / H1–H6 heuristic / F1–F12 fault-signature records, with validation gates and the frozen atlas_hash_v1.

Unsafe/UB posture: #![forbid(unsafe_code)]. Determinism is fixed-point with byte-exact replay (not claimed bit-reproducible for
arbitrary floating-point pipelines). This is alignment guidance, not a certification — see the canonical
[`../../SAFETY.md`](../../SAFETY.md) and `paper/sections/limitations.tex`.
