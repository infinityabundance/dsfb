# Architecture — dsfb-chemical-engineering-py

Python bindings (pyo3, standalone): a thin abi3 wheel exposing the file-free read-only courts.

Unsafe posture: unsafe confined to the pyo3 binding boundary. It sits in the DSFB execution-vs-authority split (execution = `edge`/`cuda`; authority
= `atlas`/`corpus`; embedded = `core`; bindings = `py`; browser = `wasm`). It is deterministic,
hash-sealed where it emits evidence, and bounded by explicit non-claims. See the repo `README.md` for the
full crate map and `breadth_surface.toml` for the claim→artifact→reproduction→tier index.
