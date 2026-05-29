# Safety — dsfb-chemical-engineering-edge

Advisory, read-only; **no control or safety-instrumented-function authority** (never gates an interlock).
CPU execution: the residual pipeline, drift/slew/envelope grammar, detector bank, deterministic quorum fusion, heuristics, reports/figures, and the Chemical Court Record v1 bundle.

Unsafe/UB posture: #![forbid(unsafe_code)]. Determinism is fixed-point with byte-exact replay (not claimed bit-reproducible for
arbitrary floating-point pipelines). This is alignment guidance, not a certification — see the canonical
[`../../SAFETY.md`](../../SAFETY.md) and `paper/sections/limitations.tex`.
