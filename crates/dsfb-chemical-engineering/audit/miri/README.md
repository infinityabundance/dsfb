# Miri — undefined-behaviour interpreter

[Miri](https://github.com/rust-lang/miri) executes Rust under an interpreter that detects **undefined behaviour**
(out-of-bounds access, use-after-free, invalid-value / misaligned reads, data races, some leaks) that an ordinary
optimised build silently accepts. Run on the three `no_std`, `#![forbid(unsafe_code)]` crates Miri can fully
interpret (`cargo +nightly miri test -p dsfb-chemical-engineering-<core|atlas|corpus>`), plus the new
`#![forbid(unsafe_code)]` execution-substrate crate `dsfb-densor-runtime`, whose tests are pure and deterministic
(`cargo +nightly miri test -p dsfb-densor-runtime`).

## Result — no UB on any interpreted crate

| Crate | Miri unit tests | Verdict | Log |
|---|---|---|---|
| `core` (embedded grammar) | **8 / 8 pass** | no UB (exit 0) | `miri-core.txt` |
| `atlas` (authority) | **7 / 7 pass** | no UB (exit 0) | `miri-atlas.txt` |
| `corpus` (data corpus) | **7 / 7 pass** | no UB (exit 0) | `miri-corpus.txt` |
| `dsfb-densor-runtime` (execution substrate) | **8 / 8 pass** | no UB (exit 0) | `miri-densor-runtime.txt` |

`core`'s suite covers the boundary-exact classifier (`classify_axis_boundaries_are_exact`), the deterministic
engine, the bounded ring buffer, the sensor-fault / memory-preservation path, the slew-spike / drift-accum
detectors, and the recovery-once path; `atlas` exercises the detector/heuristic authority gates (incl. the frozen
`atlas_hash_v1`); `corpus` exercises the soft-sensor catalogue gates (incl. `corpus_hash_v1`);
`dsfb-densor-runtime` exercises the seal/receipt determinism + tamper-evidence + the two authority-gate refusals.
The first three are `no_std`, no-heap, `panic="abort"`, `#![forbid(unsafe_code)]`; `dsfb-densor-runtime` is `std`
but likewise `#![forbid(unsafe_code)]` with allocation-light, deterministic tests — all four sit within the surface
Miri can fully interpret, and **all pass with no UB reported**.

## What it does NOT certify / scope
- Miri **cannot execute real foreign/GPU FFI**, so the `cuda` `--features cuda` host↔device path is out of Miri
  scope (only CPU-reference paths are interpretable); tests that shell out or touch the filesystem are
  isolated/skipped.
- A clean Miri run proves no UB **on the executed paths**, not over all inputs — the bounded-exhaustive,
  every-path guarantee is Kani's job (`audit/kani/`). The five `#![forbid(unsafe_code)]` crates have no
  first-party `unsafe` for Miri to flag anyway; the only load-bearing UB surfaces are the two declared FFI
  boundaries (`cuda`, `wasm`), which sit outside Miri's reach and are covered by the CPU-reference equivalence
  tests + source review instead.
