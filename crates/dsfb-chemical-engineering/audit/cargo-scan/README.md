# cargo-scan — status: no installable Rust CLI (coverage delegated, honestly)

`cargo-scan` (the Mozilla / UC-San-Diego unsafe-**effects** research tool) does **not** ship an installable
`cargo scan` Rust subcommand — it is a Python-driven research prototype with no stable crates.io CLI. Rather than
fabricate an output, this folder records that honestly.

The `unsafe`/effects surface this tool targets is instead covered by the siblings:
- **cargo-geiger** (`audit/cargo-geiger/`) — `unsafe` usage counts; **5 of 7 crates `#![forbid(unsafe_code)]`** (0
  `unsafe`); all first-party `unsafe` confined to two declared FFI boundaries (`cuda`, `wasm`).
- **Miri** (`audit/miri/`) — undefined behaviour on the interpreted paths.
- **Source ground-truth** — a direct `forbid(unsafe_code)` + `unsafe`-keyword grep over `src/` (the load-bearing
  posture in the geiger README), since geiger's matcher is unreliable on workspace path deps.

## What it does NOT certify
Nothing is claimed in this folder — it documents a tool that could not be run as a Rust CLI, with its intended
coverage delegated to the auditors above. No result was fabricated.
