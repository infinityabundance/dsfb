# Audit summary — dsfb-robotics

This directory contains the crate's independent-audit artefacts. Every
file is reproducible from `scripts/run_audit.sh`; all results below
are bit-exactly deterministic under the pinned `rust-toolchain.toml`.

## Gray audit

| | Result |
|---|---|
| **Overall score** | **90.3 %** — `strong assurance posture` |
| Safety Surface | 100 % (15/15 checks) |
| Verification Evidence | 100 % (5/5) |
| Build / Tooling Complexity | 100 % (6/6) |
| Lifecycle / Governance | 100 % (13/13) |
| NASA/JPL Power of Ten | 70 % (6.5/10 rules applied; 2 indeterminate) |
| Advanced Structural Checks | 91.3 % (20/23) |
| Reference: dsfb-rf (gold standard) | 91.4 % |

**Residual findings** are concentrated in a small, documented set:

- **P10-3** (no dynamic allocation after initialisation) — flagged on
  `String::with_capacity` / `Vec::with_capacity` in the
  feature-gated `std,paper_lock` binary path. The `no_std` + `no_alloc`
  core is zero-allocation; the report-emission binary legitimately
  allocates.
- **P10-5** (≥ 2 assertions per function on average) — density is
  0.74/function (up from 0.05/function in dsfb-rf). The
  `debug_assert!` injection pass is conservative to avoid over-
  saturation; the remaining gap is in small accessor helpers that
  do not benefit from additional runtime assertions.
- **P10-7** — *indeterminate* after the `.ok()` / `let _ =` →
  `eprintln!` / `print!` rewrite pass; the binary now relies on the
  CLI tool's panic-on-broken-pipe semantics for stdout/stderr.
- **P10-8** — 4 `#[cfg(feature = ...)]` sites (required for the
  `alloc` / `std` / `serde` / `paper_lock` feature matrix).
- **SAFE-STATE** — explicitly documented `_ => None` safe-state arm
  in [`src/datasets/mod.rs::from_slug`](../src/datasets/mod.rs).
- **ITER-UNB** — `std::env::args().collect()` in main; bounded by
  the OS `ARG_MAX`, documented inline.
- **H-SERDE-01** — flagged because the crate opts into
  `serde,serde_json` for JSON report emission behind the
  feature-gated `paper_lock` path. The core crate is serde-free.
- **H-ALLOC-01** — `Vec::with_capacity(cal_len)` in
  `paper_lock::calibrated_envelope`; `cal_len` is strictly bounded by
  `residuals.len() / 5`, an initialisation-time allocation.

The latest scan lives in `audit/dsfb-gray-<timestamp>/`. Reproduce
with:

```bash
cargo run -p dsfb-gray --release --bin dsfb-scan-crate -- \
  crates/dsfb-robotics \
  --out-dir crates/dsfb-robotics/audit/
```

## Miri × 3 configurations

See [`miri/MIRI_AUDIT.md`](miri/MIRI_AUDIT.md). All three configs
clean:

- `no_std` strict-provenance — 139 tests pass (15.9 s)
- `std+serde` stacked-borrows — 139 tests pass (6.6 s)
- `std+serde` tree-borrows — 139 tests pass (7.7 s)

SHA-256 of each report archived in [`miri/RUN_MANIFEST.json`](miri/RUN_MANIFEST.json).

## Kani × 4 proofs

See [`kani/KANI_AUDIT.md`](kani/KANI_AUDIT.md). All four **verified
successful** in ~1.3 s of CBMC time:

- `proof_engine_observe_bounded` — 585 checks, 0 failures
- `proof_grammar_severity_is_total_order`
- `proof_policy_from_grammar_is_total`
- `proof_envelope_violation_is_monotone_in_norm`

The `observe`-purity property is covered by proptest's
`observe_is_deterministic` (256 inputs/invocation) and the paper-lock
binary's `fixture_output_is_bit_exact_across_repeat_invocations`
integration test (full CLI re-invocations for all 14 datasets, three
times each).

## Stock test suite

- **146** lib unit tests (`--no-default-features`, `--features std,serde`,
  `--features std,paper_lock`)
- **12** proptest invariants (`tests/proptest_invariants.rs`)
- **2** concurrency-observer baselines (`tests/concurrency_observer.rs`)
- **11** adapter-pipeline integration tests (`tests/adapter_pipeline.rs`)
- **9** paper-lock CLI integration tests (`tests/paper_lock_binary.rs`)

Total: **180 tests**, all passing. `cargo clippy --all-features -D
warnings` clean. Zero `unsafe` (enforced at crate root via
`#![forbid(unsafe_code)]`). Zero `.unwrap()` / `.expect()` /
`panic!` / `todo!` / `unimplemented!` in production code.

## Provenance and reproduction

The 14 real-world residual streams in `data/processed/*.csv` are
reproducible from `scripts/preprocess_datasets.py` against the raw
corpora fetched from the per-dataset URLs documented in
`docs/<slug>_oracle_protocol.md`. The scan above was generated
against the **exact** preprocessed CSVs with SHA-256 recorded in
`data/processed/PROCESSED_MANIFEST.json`.
