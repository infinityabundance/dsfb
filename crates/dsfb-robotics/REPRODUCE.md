# Reproducibility recipe

This document is the **authoritative one-command reproduction recipe**
for the companion paper's headline results. Everything below is
deterministic: identical inputs produce identical outputs on any
supported platform (see `rust-toolchain.toml` for target support).

> **Phase status.** Commands that depend on Phases 4–6 are marked
> *(Phase 4+)* until those phases land. Phase 1 provides the crate
> scaffold; the commands under §1 (build) and §2 (core smoke tests)
> are live today.

---

## 0. Prerequisites

- `rustc` 1.85.1 (pinned by `rust-toolchain.toml`). Install rustup and
  let the toolchain file do the rest.
- `cargo-deny` for supply-chain audit (`cargo install cargo-deny`).
- `cargo-miri` (nightly) for the undefined-behaviour audit — *Phase 6*.
- `cargo-kani` for formal-verification harnesses — *Phase 6*.
- `cargo-fuzz` for libFuzzer targets — *Phase 6*.
- Python 3.11+ with `matplotlib`, `numpy`, `pandas` for figure
  regeneration — *Phase 5*.
- Optional: `latexmk` for rebuilding the companion paper PDF.

## 1. Build under every feature matrix (Phase 1 — live)

```bash
cd crates/dsfb-robotics

cargo check --no-default-features           # no_std + no_alloc + zero unsafe core
cargo check --features alloc                # + heap-backed convenience wrappers
cargo check --features std                  # + host-side tooling
cargo check --features std,serde            # + JSON serialization
cargo check --features std,paper_lock       # + paper-lock binary
```

All five configurations must succeed with **zero errors** on the pinned
toolchain. The only warnings expected are the host's AVX-512
target-feature warnings (environment-level, not crate-level).

## 2. Core smoke tests (Phase 1 — live)

```bash
cargo test --no-default-features --lib
```

Four Phase 1 smoke tests must pass:

- `phase1_smoke_tests::empty_input_returns_zero_episodes`
- `phase1_smoke_tests::non_empty_input_phase1_returns_zero_episodes`
- `phase1_smoke_tests::empty_episode_has_admissible_grammar`
- `phase1_smoke_tests::observe_never_writes_past_buffer`

## 3. Supply-chain audit (Phase 1 — live)

```bash
cargo deny --manifest-path crates/dsfb-robotics/Cargo.toml check
```

`cargo deny` must report **zero** violations. Licence allowlist is
Apache-2.0 / MIT / BSD-2/3-Clause / ISC / Unicode-DFS-2016; the
GPL / LGPL / AGPL family is denied.

## 4. Lint and format (Phase 1 — live)

```bash
cargo clippy --all-features -- -D warnings
cargo fmt --all -- --check
```

## 5. Integration tests, proptests, loom (Phase 2+ — *pending*)

```bash
cargo test --all-features
cargo test --test proptest_invariants       # ≥10 observer-contract invariants
RUSTFLAGS="--cfg loom" cargo test --test concurrency_observer
```

## 6. Miri UB audit (Phase 6 — *pending*)

Three separate configurations, matching the `dsfb-rf` audit layout:

```bash
# 6a. Strict-provenance, no_std + no_alloc core
MIRIFLAGS="-Zmiri-strict-provenance" \
  cargo +nightly miri test --no-default-features --lib

# 6b. Stacked-borrows, std + serde
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation" \
  cargo +nightly miri test --features std,serde --lib

# 6c. Tree-borrows, std + serde
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation -Zmiri-tree-borrows" \
  cargo +nightly miri test --features std,serde --lib
```

All three runs must be clean. Output is archived under
`audit/miri/miri_*.txt` with a consolidated `MIRI_AUDIT.md`.

## 7. Kani formal verification (Phase 6 — *pending*)

```bash
cargo kani
```

Kani must verify every harness in `src/kani_proofs.rs`:

- `proof_observe_bounded_output` — `observe()` writes ≤ `out.len()` episodes.
- `proof_grammar_fsm_total` — every grammar transition is defined for every input.
- `proof_envelope_monotonicity` — `τ` threshold crossings are monotonic in `ρ`.
- `proof_observe_pure` — identical inputs produce identical outputs.

## 8. Fuzzing (Phase 6 — *pending*)

```bash
cd fuzz
cargo +nightly fuzz run engine_roundtrip -- -runs=1000000
cargo +nightly fuzz run grammar_fsm      -- -runs=1000000
```

Corpora are checked in under `fuzz/corpus/`.

## 9. Paper-lock — ten real-world datasets (Phase 4+ — *pending*)

```bash
for ds in cwru ims cmapss kuka_lwr femto_st \
          panda_gaz dlr_justin ur10_kufieta \
          cheetah3 icub_pushrecovery; do
  cargo run --release --bin paper-lock --features std,paper_lock -- "$ds"
done
```

Each dataset requires its full corpus at the path documented in the
per-dataset oracle-protocol (see `docs/<dataset>_oracle_protocol.md`).
`paper-lock` **never** silently substitutes a synthetic fixture; if
the dataset is absent, it exits with code 64 (EX_USAGE) and prints
the fetch instructions for that dataset's licence tier.

Redistributable slices are shipped in `data/slices/` under each
dataset's upstream licence (see `data/slices/SLICE_MANIFEST.json`).
Datasets under data-use agreements (DLR Justin, iCub) ship as
**manifest-only pointers with SHA-256 checksums** — the user must
obtain the full corpus from the upstream provider under their terms.

Three consecutive runs must produce **bit-identical** outputs on the
same slice set.

## 10. Figures and Colab notebook (Phase 5 — *pending*)

```bash
python3 scripts/figures_real.py
```

Regenerates every figure referenced in the companion paper to
pixel-identity. Output goes to `dsfb-robotics-output/figs/`.

Alternative one-click reproduction via Colab:
<https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-robotics/colab/dsfb_robotics_reproduce.ipynb>

The notebook bundles only the in-tree slices and must complete
end-to-end in under five minutes on free-tier Colab.

## 11. Rebuild the companion paper (Phase 8 — *pending*)

```bash
cd paper && latexmk -pdf dsfb_robotics.tex
```

Produces `paper/dsfb_robotics.pdf`. The `latexdiff` between the paper
before and after the Phase 8 patches must show **only additions** and
measured-value substitutions for the `TBD — companion crate` lines —
no existing line is deleted other than within the replaced TBD
sentences themselves.

## 12. dsfb-gray audit (Phase 9 — *pending*)

```bash
cargo run -p dsfb-gray --release -- \
  --crate-path crates/dsfb-robotics \
  --output crates/dsfb-robotics/audit/dsfb_robotics_scan.txt
```

Target: **overall ≥ 95 %**, every section ≥ 90 %. The final SARIF
output is archived at `audit/dsfb_robotics_scan.sarif.json`.

---

## Determinism guarantees

1. **Toolchain is pinned** (`rust-toolchain.toml` → rustc 1.85.1).
2. **Release profile** uses `lto = true`, `codegen-units = 1`,
   `panic = "abort"` (see `Cargo.toml`) so the final binary is
   deterministic in layout.
3. **No `RNG`** is seeded in the core engine. Any randomness used for
   dataset stratification is seeded deterministically from the
   dataset's SHA-256 (see `scripts/prepare_slices.py`, Phase 5).
4. **No floating-point non-associativity traps** in the hot path —
   the engine accumulates `f64` in a fixed summation order; use
   `cargo miri` (Phase 6) to double-check under strict provenance.

## Verifying an external reproduction

A third-party reproducer signals success by posting:

1. `audit/dsfb_robotics_scan.sarif.json` hash.
2. `paper-lock` output JSONs for each of the ten datasets.
3. `miri` transcript hashes for each of the three configurations.
4. `kani` result hashes for each harness.

These collectively fingerprint the reproducible build.
