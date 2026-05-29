# cargo-fuzz — coverage-guided fuzzing of the deterministic DSFB core

[`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) drives **libFuzzer** (with **AddressSanitizer**) against
pure, deterministic DSFB entry points. Where the Kani harnesses (`audit/kani/`) prove totality over a *bounded
symbolic* window, fuzzing is the **empirical companion**: it beats on the *same* functions over the full input
domain — arbitrary bytes reinterpreted as `f64`/`i64`, including NaN, ±∞, subnormals, `i64::MIN/MAX`, and inverted
bounds — with coverage feedback steering toward new branches. The harness crate is
`crates/dsfb-chemical-engineering-edge/fuzz/` (a detached sub-workspace; never built by `cargo test --workspace`).

## Targets and results (real captured runs — 60 s each)

Toolchain: `cargo-fuzz 0.13.1`, `+nightly`, libFuzzer + AddressSanitizer. Each campaign started from an empty
corpus. **No input larger than 4096 bytes.** Full logs: `*.txt` in this folder.

| Target | Function under test | Executions | exec/s | Coverage | Verdict |
|---|---|---|---|---|---|
| `grammar_classify` | `dsfb_core::evaluate` → `GrammarClassifier::classify` (float path) | **91,954,977** | ~1.51 M | 43 edges (saturated) | **0 crashes / panics** |
| `parse_unit` | `unit_consistency::parse_unit(&str)` (unit lexer) | **47,636,768** | ~0.78 M | 249 edges, 248-input corpus | **0 crashes / panics** |
| `core_fixedpoint_classify` | `core::FixedEnvelope::eval` → `GrammarClassifier::classify` (`no_std` i64 path) | **85,640,660** | ~1.40 M | 55 edges | **0 crashes / panics** |

**Total: 225,232,405 executions, 0 crashes, 0 panics, 0 ASan errors (no memory error / leak / UB).**

## What each target establishes
- **`grammar_classify`** — the float grammar classifier is *total and panic-free over the entire IEEE-754 domain*,
  not merely the `|r,δ,σ| < 1e6` finite window the Kani proof bounds. The fuzzer reaches NaN/±∞/subnormal triples
  (which route to `SensorFault`) without ever tripping a panic, overflow, or trap. This is the empirical leg of the
  totality claim Kani proves symbolically.
- **`parse_unit`** — the engineering-unit string lexer (a data-ingestion boundary fed by historian headers /
  roles sidecars) never panics on arbitrary valid UTF-8: no slicing on non-`char` boundaries, no index overflow.
  The richer coverage (249 edges, 248 retained inputs) reflects the fuzzer discovering the unit-table branches.
- **`core_fixedpoint_classify`** — backs the `no_std` core's documented overflow-safety claim ("promoted to `i128`
  so the product never overflows") *empirically*: fully-arbitrary `i64` envelope bounds (incl. inverted `lo > hi`
  and out-of-`[0,SCALE)` band fractions) and triple coordinates flow through `eval → classify` with zero overflow
  panics. This is the embedded sibling of the float `grammar_classify` target.

## What this does NOT certify
- **Fuzzing is bounded-time sampling, not a proof.** 60 s / ~10⁸ executions per target found nothing; it does not
  *prove* the absence of a deeper input that would. The proof obligation lives in `audit/kani/` (bounded, every-path)
  and `formal/` (Lean 4 / Coq). Fuzzing and proving are complementary, not redundant.
- **Coverage is structural, not semantic.** Saturated edge coverage means the fuzzer exercised the reachable
  branches; it does not assert the *classification is correct*, only that it does not crash. Correctness is the
  job of the unit tests + the replay-digest gate.
- **Scope is the pure, deterministic core.** The CSV/JSON/TOML *file* loaders (`load_csv_slice`,
  `RolesDoc::load`, the MANIFEST parser) are I/O-bound and not yet fuzzed here; they are an obvious next target
  (file-backed harness) and are noted as future work. The CUDA/GPU path is out of scope (no sanitizer over FFI).
- **No data flows in.** The targets synthesise inputs from fuzzer bytes; no dataset (synthetic or controlled) is read.

## Reproduce
```fish
cd crates/dsfb-chemical-engineering-edge
cargo +nightly fuzz build
cargo +nightly fuzz run grammar_classify         -- -max_total_time=60
cargo +nightly fuzz run parse_unit               -- -max_total_time=60
cargo +nightly fuzz run core_fixedpoint_classify -- -max_total_time=60
```
The `fuzz/corpus/` and `fuzz/artifacts/` directories are git-ignored (regenerable); the harness *sources*
(`fuzz/fuzz_targets/*.rs`) and `fuzz/Cargo.toml` are committed and reviewable.
