# DSFB grammar — Lean 4 formalisation

Machine-checked Lean 4 proofs of the DSFB grammar/fusion **proof obligations** (the checklist sealed by
`dsfb_chemical_engineering_edge::proof_obligations::ProofObligationLedgerV1`). Mirrors the `no_std` core
(`crates/dsfb-chemical-engineering-core`) over Lean `Int` (unbounded — so a proof here covers every machine
width). **Pure Lean 4 core, no Mathlib** → builds offline against the pinned toolchain.

## Build / verify
```bash
cd formal/lean
lake build          # needs the Lean toolchain (lean-toolchain pins leanprover/lean4:v4.29.1)
```
The Lean4 VS Code extension (`leanprover.lean4`) installs the toolchain automatically on first open; from a
shell, `elan` + `lake build` do the same. A clean build = every theorem below verified.

## What is proven (each maps to a `ProofObligationLedgerV1` row)
| Theorem (`DsfbGrammar.lean`) | Obligation | Also Kani? |
|---|---|---|
| `classify_total` | grammar totality (every input → exactly one state) | yes |
| `valid_not_sensorFault` | a valid reading is never `SensorFault` | yes |
| `outside_r_not_nominal` | an out-of-bound residual is never `nominal` | yes |
| `deltaSigma_outside_is_compound` | the compound rule (δ∧σ outside ⇒ `compound`) | — |
| `fused_sound` + `not_fused_below_quorum` | **quorum soundness** (was open) | — |
| `compression_monotone` | **episode-compression monotonicity** (was open) | — |

The three Kani-checked obligations are re-proven here **unbounded** (Kani checks them on a bounded domain);
the two previously-open obligations (quorum soundness, compression monotonicity) are now proven outright.

## Honest scope (non-claims)
This formalises the **grammar/fusion logic**, not the floating-point edge pipeline and not any physical-process
claim. **Replay determinism** is *not* proven here — it stays empirical (the `verify-replay` 6/6 gate + the
golden hashes); a Lean/Coq proof of it remains future work. A **Coq** port (`coqc`) is not included: this
sandbox has the Lean toolchain but not Coq's compiler. Nothing here asserts root cause, causality, or any
control/safety authority.
