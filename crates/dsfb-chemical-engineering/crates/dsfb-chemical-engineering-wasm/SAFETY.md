# Safety — dsfb-chemical-engineering-wasm

Advisory, read-only; **no control or safety-instrumented-function authority** (never gates an interlock).
Browser what-if Chemical Court simulator: replays a residual stream under an operator-amended admissibility envelope over immutable evidence (raw extern "C" exports, no wasm-bindgen).

Unsafe/UB posture: #![deny(unsafe_code)] with one audited linear-memory FFI block (exercised under Miri). Determinism is fixed-point with byte-exact replay (not claimed bit-reproducible for
arbitrary floating-point pipelines). This is alignment guidance, not a certification — see the canonical
[`../../SAFETY.md`](../../SAFETY.md) and `paper/sections/limitations.tex`.
