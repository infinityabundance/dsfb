# Safety — dsfb-chemical-engineering-cuda

Advisory, read-only; **no control or safety-instrumented-function authority** (never gates an interlock).
GPU evidence factory + forensic court: on-GPU SHA-256 + Merkle evidence root, byte-exact to the CPU reference, digest-equivalence-gated.

Unsafe/UB posture: unsafe confined to the CUDA FFI boundary; the CPU-reference path is safe. Determinism is fixed-point with byte-exact replay (not claimed bit-reproducible for
arbitrary floating-point pipelines). This is alignment guidance, not a certification — see the canonical
[`../../SAFETY.md`](../../SAFETY.md) and `paper/sections/limitations.tex`.
