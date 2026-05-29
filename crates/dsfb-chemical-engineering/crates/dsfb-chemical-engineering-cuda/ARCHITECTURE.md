# Architecture — dsfb-chemical-engineering-cuda

GPU evidence factory + forensic court: on-GPU SHA-256 + Merkle evidence root, byte-exact to the CPU reference, digest-equivalence-gated.

Unsafe posture: unsafe confined to the CUDA FFI boundary; the CPU-reference path is safe. It sits in the DSFB execution-vs-authority split (execution = `edge`/`cuda`; authority
= `atlas`/`corpus`; embedded = `core`; bindings = `py`; browser = `wasm`). It is deterministic,
hash-sealed where it emits evidence, and bounded by explicit non-claims. See the repo `README.md` for the
full crate map and `breadth_surface.toml` for the claim→artifact→reproduction→tier index.
