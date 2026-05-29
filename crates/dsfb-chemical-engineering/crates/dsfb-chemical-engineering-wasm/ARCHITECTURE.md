# Architecture — dsfb-chemical-engineering-wasm

Browser what-if Chemical Court simulator: replays a residual stream under an operator-amended admissibility envelope over immutable evidence (raw extern "C" exports, no wasm-bindgen).

Unsafe posture: #![deny(unsafe_code)] with one audited linear-memory FFI block (exercised under Miri). It sits in the DSFB execution-vs-authority split (execution = `edge`/`cuda`; authority
= `atlas`/`corpus`; embedded = `core`; bindings = `py`; browser = `wasm`). It is deterministic,
hash-sealed where it emits evidence, and bounded by explicit non-claims. See the repo `README.md` for the
full crate map and `breadth_surface.toml` for the claim→artifact→reproduction→tier index.
