# Online Path Allocation Audit

This note records the current allocation status of the bounded live path.

Scope:

- `OnlineStructuralEngine::new`
- `OnlineStructuralEngine::push_residual_sample`
- `OnlineStructuralEngine::push_residual_sample_batch`

## Current Findings

| Phase | Symbol | Allocation Behavior | Current Status |
|---|---|---|---|
| initialization | `OnlineStructuralEngine::new` | allowed | ring buffer, channel-name storage, bank registry, and retrieval index allocate here |
| per-sample | `OnlineStructuralEngine::push_residual_sample` | present | bounded `Vec`-backed residual/drift/slew/sign/status materialization still occurs |
| optional offline accumulation | `offline_history` | opt-in | disabled by default; outside bounded live-path contract |
| interface wrapper | `LiveEngineStatus` / FFI copy helpers | present | owned strings and selected heuristic ID vectors remain allocation-bearing |

## What Is Verified

- online history growth is bounded by the ring buffer capacity
- no unbounded `Vec` growth occurs in the hot bounded-history state itself
- optional offline accumulation remains separate and explicit

## What Is Not Yet Claimed

- no-allocation-after-init for the current hot path
- allocator-instrumented proof that every live step is allocation-free

## Regeneration / Cross-Check

The machine-readable companion appears in:

- [`docs/generated/real_time_contract_summary.json`](generated/real_time_contract_summary.json)

The live-path source under review is:

- [`src/live/mod.rs`](../src/live/mod.rs)
- [`src/live/contract.rs`](../src/live/contract.rs)
