# Requirements and Claim Traceability

This document maps high-value crate requirements and public claims to concrete implementation and test evidence. It is an audit-readiness aid, not a compliance certificate.

| Requirement ID | Requirement / Claim | Primary Implementation Evidence | Primary Test Evidence |
|----------------|---------------------|---------------------------------|-----------------------|
| `REQ-BOUNDED-LIVE-001` | Bounded online memory for live use | `src/live/mod.rs` ring buffer and bounded online engine | `tests/deployment_readiness.rs`, `src/live/mod.rs` unit tests |
| `REQ-TRUST-BOUND-002` | Trust scalar remains in `[0,1]` | `src/engine/types.rs`, `src/math/envelope.rs` | `tests/ruggedization_proofs.rs`, `proofs/kani/trust_scalar.rs` |
| `REQ-GOVERNANCE-003` | Strict bank validation is the default | `src/engine/config.rs`, `src/engine/bank.rs` | `tests/ruggedization_proofs.rs`, `tests/adoption_proofs.rs` |
| `REQ-FFI-BATCH-004` | Batch FFI ingestion preserves scalar semantics | `ffi/src/lib.rs`, `ffi/include/dsfb_semiotics_engine.h` | `ffi/src/lib.rs` unit tests, `tests/high_assurance_embedded.rs` |
| `REQ-CSV-REPLAY-005` | CSV replay stays deterministic and event-marked | `src/dashboard/csv_replay.rs` | `tests/dashboard_replay.rs` |
| `REQ-FIXED-POINT-006` | Experimental fixed-point numeric backend exists for the bounded live path | `src/math/fixed_point.rs`, `src/live/mod.rs`, `src/engine/settings.rs` | `tests/high_assurance_embedded.rs` |
| `REQ-SAFETY-SMOOTH-007` | Safety-first jitter robustness profile is typed and exported | `src/engine/settings.rs`, `src/math/smoothing.rs`, `src/report/artifact_report.rs` | `tests/high_assurance_embedded.rs`, `tests/ruggedization_proofs.rs` |
| `REQ-TIMING-008` | Timing determinism report includes tail metrics on a stated platform | `src/bin/dsfb-timing-determinism.rs`, `benches/execution_budget.rs` | `tests/high_assurance_embedded.rs` |
| `REQ-STATE-REPLAY-009` | Live engine state can be snapshotted and replayed one step exactly under documented conditions | `src/live/mod.rs`, `src/bin/dsfb-state-replay.rs` | `src/live/mod.rs` unit tests, `tests/high_assurance_embedded.rs` |
| `REQ-SUPPLY-CHAIN-010` | Supply-chain audit tooling is configured | `deny.toml`, `justfile`, crate-local CI workflow | `tests/high_assurance_embedded.rs` |

## Reading This Matrix

- Treat implementation evidence as the main code anchor.
- Treat tests as executable evidence for bounded properties.
- Treat README and companion docs as operator-facing explanations, not as the sole source of truth.
