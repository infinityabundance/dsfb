# DSFB-Debug Non-Intrusion Contract — Version 1.0

## Formal Statement

The DSFB-Debug engine implements the mapping:

    O : T → E

where T is the set of telemetry observation sequences (traces, metrics, logs)
and E is the set of typed debugging episodes.

The codomain E ⊂ {Silent, Watch, Review, Escalate}* contains only advisory
outputs. No element of E can modify any element of T.

## Exclusion Table

The following write-path types DO NOT EXIST in the crate's public API:

| Excluded Type | Why Excluded |
|---|---|
| `&mut [f64]` (observations) | Cannot modify upstream telemetry data |
| `&mut [bool]` (labels) | Cannot modify upstream fault labels |
| `&mut AlertConfig` | Cannot modify alert thresholds |
| `&mut SamplingConfig` | Cannot modify trace sampling rates |
| `&mut ApplicationState` | Cannot modify application runtime |

## Compile-Time Enforcement

The Rust type system enforces this contract through three mechanisms:

1. **`#![forbid(unsafe_code)]`** — prevents circumventing the borrow checker
2. **All public functions accept only `&self` and `&[T]`** — shared immutable references
3. **No interior mutability types** (`RefCell`, `Cell`, `UnsafeCell`) in public API

## Verification

- `tests/integration.rs::test_observer_only_contract` — proves no mutable side-channel
- `cargo clippy -- -D warnings -D clippy::mut_from_ref` — catches leaked mutability
- CI enforces on every commit

## Deterministic Replay Guarantee (Theorem 9)

`engine.evaluate(input) == engine.evaluate(input)` — always.

Verified by `tests/integration.rs::test_theorem_9_deterministic_replay`.
