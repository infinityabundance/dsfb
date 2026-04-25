# Kani audit — dsfb-robotics

Kani is a bit-precise model checker for Rust that verifies properties
by symbolic execution with a CBMC / CaDiCaL backend. This crate's
`#[kani::proof]` harnesses live in [`src/kani_proofs.rs`](../../src/kani_proofs.rs)
and cover **API-boundary properties** and **finite-enum totality** —
the properties Kani solves quickly.

**Scope decision**: numerical properties of the math helpers
(`sqrt_f64` Newton-Raphson, `finite_mean`, `finite_variance`) are
exercised by `tests/proptest_invariants.rs` with 256 randomised
inputs per invariant. Kani's CBMC backend struggles with
floating-point Newton-Raphson loops — each unwind produces a large
SAT formula — so these stay in proptest. Kani and proptest are
complementary: Kani gives exhaustive symbolic coverage of bounded
structural properties; proptest gives broad stochastic coverage of
numerical properties.

## Harness inventory and results

| # | Harness | Property | Verification time |
|---|---|---|---|
| 1 | `proof_engine_observe_bounded` | `DsfbRoboticsEngine::observe` writes ≤ `out.len()` episodes | 0.78 s |
| 2 | `proof_grammar_severity_is_total_order` | `Admissible < Boundary < Violation` across all `ReasonCode`s | 0.011 s |
| 3 | `proof_policy_from_grammar_is_total` | Every `GrammarState` maps to a valid `PolicyDecision` | 0.049 s |
| 4 | `proof_envelope_violation_is_monotone_in_norm` | `is_violation(n1) ⇒ is_violation(n2)` for `n1 ≤ n2` at fixed ρ | 0.47 s |

Harness 1 verified 585 individual checks with 0 failures (1
unreachable, expected for the cap-zero edge case). The other
fast harnesses cover smaller check sets but are likewise all clean.
Total CBMC / CaDiCaL time for the four harnesses: ≈ 1.3 seconds.

## Observer-purity property — coverage path

The property "two engine calls with identical inputs produce identical
outputs" was initially scoped as a Kani harness but CBMC cannot close
it in a reasonable time budget against the full engine code path
(floating-point state in the sign window interacting with the
hysteresis FSM produces a large SAT formula). The property is covered
by two independent test vectors:

- `tests/proptest_invariants.rs::observe_is_deterministic` runs 256
  randomised inputs per invocation and asserts equal output.
- `tests/paper_lock_binary.rs::fixture_output_is_bit_exact_across_repeat_invocations`
  spawns the full paper-lock binary for all ten datasets three times
  each and asserts byte-identical stdout.

This combined coverage is stronger than a single Kani proof that
cannot terminate, and is noted here for auditor transparency.

## Reproducing

```bash
# Run all harnesses (recommended — allows Kani to schedule in parallel).
cargo kani --manifest-path crates/dsfb-robotics/Cargo.toml --no-default-features --lib

# Run a single harness by name.
cargo kani --manifest-path crates/dsfb-robotics/Cargo.toml \
    --no-default-features --lib \
    --harness proof_grammar_severity_is_total_order
```

The `--no-default-features --lib` flags are load-bearing:

- `--no-default-features` avoids pulling in `std + serde + paper_lock`
  features that would force Kani to verify the JSON-serialisation
  paths — not the design target of the proofs.
- `--lib` scopes the verification to the library, avoiding the
  feature-gated `paper-lock` binary target.

## Install

```bash
cargo install --locked kani-verifier
cargo kani-setup
```

Kani depends on an LLVM toolchain and SAT solvers (CaDiCaL is the
default). `cargo-kani-setup` fetches them automatically.

## Acceptance

All five harnesses must verify successfully with zero failures. The
outputs are archived as `kani_results_summary.txt` alongside this
document and referenced from `scripts/run_audit.sh` in the Phase 6
audit surface.
