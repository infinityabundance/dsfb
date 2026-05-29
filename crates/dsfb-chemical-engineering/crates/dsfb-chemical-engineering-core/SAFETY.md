# Safety — dsfb-chemical-engineering-core

Advisory, read-only; **no control or safety-instrumented-function authority** (never gates an interlock).
Embedded grammar (no_std, no-heap, fixed-point): the residual triple + ring buffer + admissibility envelope + grammar state machine in scaled integers.

Unsafe/UB posture: #![forbid(unsafe_code)] (no_std, no heap, panic=abort). Determinism is fixed-point with byte-exact replay (not claimed bit-reproducible for
arbitrary floating-point pipelines). This is alignment guidance, not a certification — see the canonical
[`../../SAFETY.md`](../../SAFETY.md) and `paper/sections/limitations.tex`.
