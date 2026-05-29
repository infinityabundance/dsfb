# Architecture — dsfb-chemical-engineering-corpus

Authority (no_std): the provenance-tiered soft-sensor dataset catalogue, with the frozen corpus_hash_v1.

Unsafe posture: #![forbid(unsafe_code)]. It sits in the DSFB execution-vs-authority split (execution = `edge`/`cuda`; authority
= `atlas`/`corpus`; embedded = `core`; bindings = `py`; browser = `wasm`). It is deterministic,
hash-sealed where it emits evidence, and bounded by explicit non-claims. See the repo `README.md` for the
full crate map and `breadth_surface.toml` for the claim→artifact→reproduction→tier index.
