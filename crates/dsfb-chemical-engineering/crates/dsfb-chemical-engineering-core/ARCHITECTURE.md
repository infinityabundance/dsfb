# Architecture — dsfb-chemical-engineering-core

Embedded grammar (no_std, no-heap, fixed-point): the residual triple + ring buffer + admissibility envelope + grammar state machine in scaled integers.

Unsafe posture: #![forbid(unsafe_code)] (no_std, no heap, panic=abort). It sits in the DSFB execution-vs-authority split (execution = `edge`/`cuda`; authority
= `atlas`/`corpus`; embedded = `core`; bindings = `py`; browser = `wasm`). It is deterministic,
hash-sealed where it emits evidence, and bounded by explicit non-claims. See the repo `README.md` for the
full crate map and `breadth_surface.toml` for the claim→artifact→reproduction→tier index.
