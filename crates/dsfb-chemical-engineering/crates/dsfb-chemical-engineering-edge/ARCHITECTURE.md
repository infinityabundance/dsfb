# Architecture — dsfb-chemical-engineering-edge

CPU execution: the residual pipeline, drift/slew/envelope grammar, detector bank, deterministic quorum fusion, heuristics, reports/figures, and the Chemical Court Record v1 bundle.

Unsafe posture: #![forbid(unsafe_code)]. It sits in the DSFB execution-vs-authority split (execution = `edge`/`cuda`; authority
= `atlas`/`corpus`; embedded = `core`; bindings = `py`; browser = `wasm`). It is deterministic,
hash-sealed where it emits evidence, and bounded by explicit non-claims. See the repo `README.md` for the
full crate map and `breadth_surface.toml` for the claim→artifact→reproduction→tier index.
