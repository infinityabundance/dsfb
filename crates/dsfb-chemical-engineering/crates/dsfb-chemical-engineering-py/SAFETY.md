# Safety — dsfb-chemical-engineering-py

Advisory, read-only; **no control or safety-instrumented-function authority** (never gates an interlock).
Python bindings (pyo3, standalone): a thin abi3 wheel exposing the file-free read-only courts.

Unsafe/UB posture: unsafe confined to the pyo3 binding boundary. Determinism is fixed-point with byte-exact replay (not claimed bit-reproducible for
arbitrary floating-point pipelines). This is alignment guidance, not a certification — see the canonical
[`../../SAFETY.md`](../../SAFETY.md) and `paper/sections/limitations.tex`.
