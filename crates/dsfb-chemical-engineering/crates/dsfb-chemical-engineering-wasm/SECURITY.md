# Security — dsfb-chemical-engineering-wasm

Browser what-if Chemical Court simulator: replays a residual stream under an operator-amended admissibility envelope over immutable evidence (raw extern "C" exports, no wasm-bindgen).

Unsafe posture: #![deny(unsafe_code)] with one audited linear-memory FFI block (exercised under Miri). This crate follows the workspace security policy — see the canonical
[`../../SECURITY.md`](../../SECURITY.md). Machine-readable audits (dsfb-gray, cargo-audit, cargo-geiger,
Miri, …) are in [`../../audit/`](../../audit/). Report vulnerabilities per the repo policy.
