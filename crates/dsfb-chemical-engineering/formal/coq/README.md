# DSFB grammar — Coq / Rocq formalisation

A second, independent machine-checked formalisation of the DSFB grammar/fusion **proof obligations**,
alongside the Lean 4 development in [`../lean/`](../lean/). Mirrors the `no_std` core
(`crates/dsfb-chemical-engineering-core`); the integer `classifyAxis` is formalised over `Z`. Verified with
**Rocq Prover 9.1.1** (`coqc`).

## Build / verify
```bash
sudo pacman -S rocq-stdlib        # provides List / ZArith / Lia (rocq-core alone is not enough)
cd formal/coq
coqc DsfbGrammar.v                # a clean compile (.vo produced, no errors) = every theorem verified
```

## What is proven (each maps to a `ProofObligationLedgerV1` row)
| Theorem (`DsfbGrammar.v`) | Obligation |
|---|---|
| `classify_total` | grammar totality |
| `valid_not_sensorFault` | a valid reading is never `sensorFault` |
| `outside_r_not_nominal` | an out-of-bound residual is never `nominal` |
| `deltaSigma_outside_is_compound` | the compound rule |
| `fused_sound` | **quorum soundness** |
| `compression_monotone` | **episode-compression monotonicity** |

The same obligations the Lean 4 development proves — now cross-checked by a *second, independent* prover
kernel, with `classifyAxis` additionally modelled over `Z`.

## Honest scope (non-claims)
Formalises the **grammar/fusion logic**, not the floating-point edge pipeline or any physical-process claim.
**Replay determinism** is not proven here — it stays empirical (the `verify-replay` 6/6 gate + golden hashes).
Nothing here asserts root cause, causality, or any control/safety authority.
