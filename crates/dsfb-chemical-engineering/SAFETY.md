# Safety statement — DSFB-Chemical-Engineering

**Advisory, read-only, no control or safety authority.** This software is NOT a controller, NOT a safety
instrumented function (SIS/SIF), and is NOT certified to IEC 61511 / IEC 61508 / ISA-84. It must never gate a
safety interlock. Every emitted label is an explicit **candidate**; episodes that meet no heuristic are
preserved as "unknown structural episode (evidence preserved)" rather than forced into a confident diagnosis.

- **Non-interference:** the pipeline is an observer with no feedback path; closed-loop plant stability is
  unaffected by construction (paper, Proposition 1).
- **Memory/UB:** `edge`/`atlas`/`corpus`/`core` `#![forbid(unsafe_code)]`; the embedded `core` is `no_std`,
  no-heap, `panic = "abort"`; the one audited `unsafe` block (wasm FFI marshalling) is exercised under Miri.
- **Determinism:** fixed-point quantisation + byte-exact replay; not claimed bit-reproducible for arbitrary
  floating-point pipelines.

This statement is alignment guidance, **not** a certification. See `paper/sections/limitations.tex` and the
paper's "Claims not made" section.
