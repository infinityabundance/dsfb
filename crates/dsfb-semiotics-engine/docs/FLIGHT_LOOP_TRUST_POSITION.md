# Flight-Loop Trust Position

This note positions `dsfb-semiotics-engine` as it exists today for transition use. It is not a
generic future-work note. It answers where the current evidence supports use, where it supports
only bounded experiments, and where blind trust is not yet justified.

Companion references:

- [Real-Time Contract](REAL_TIME_CONTRACT.md)
- [Timing Determinism Report](TIMING_DETERMINISM_REPORT.md)
- [Target-Facing Timing Demo](TARGET_FACING_TIMING_DEMO.md)
- [Online Path Allocation Audit](ONLINE_PATH_ALLOCATION_AUDIT.md)
- [High-Assurance Embedded Notes](high_assurance_embedded.md)
- [ICD](ICD.md)

## Position Summary

Three scope labels matter:

- `observed bounded behavior`: measured software behavior under the documented build and fixture
  conditions
- `integration-ready under stated assumptions`: suitable for bounded monitor/advisory integration
  or direct loop experiments where the caller accepts the explicit evidence and gaps
- `not certifiable as-is`: not a certified flight-critical component and not justified for blind
  trust in a primary control role

## Suitable Today

### Evaluation harness use

Suitable today:

- synthetic and public-dataset evaluation harnesses
- replay, forensics, and artifact generation workflows
- batch-ingestion experiments for multi-axis logs
- paper/demo figure generation and operational timeline review

Why:

- the layered path is deterministic and reproducible
- artifacts and replay exports are auditable
- the theorem/code/test traceability path exists

### Bounded monitor / advisory use

Suitable today under stated assumptions:

- bounded live-path monitoring where the component informs, annotates, or constrains operator
  interpretation
- monitor/advisory surfaces that consume:
  - syntax label
  - grammar reason code
  - semantic disposition
  - trust scalar
- batch-ingestion monitor paths for IMU-style multi-axis data

Why:

- the bounded live path has an explicit contract in [REAL_TIME_CONTRACT.md](REAL_TIME_CONTRACT.md)
- observed host timing and a separate constrained-profile timing story exist in
  [TIMING_DETERMINISM_REPORT.md](TIMING_DETERMINISM_REPORT.md) and
  [TARGET_FACING_TIMING_DEMO.md](TARGET_FACING_TIMING_DEMO.md)
- panic and non-finite-output policies are documented and tested for the online path
- fixed-point evidence now exists for the tested live subset in
  [FIXED_POINT_DEPLOYMENT_EVIDENCE.md](FIXED_POINT_DEPLOYMENT_EVIDENCE.md)

## Suitable Only for Direct Flight-Loop Integration Experiments

Direct flight-loop integration experiments are reasonable only under explicit laboratory or
hardware-in-the-loop assumptions:

- the component remains observable and bounded behind a supervisory integration boundary
- timing is remeasured on the actual target platform
- the integration owner accepts that observed bounded behavior is not certified WCET
- the current online allocation gap is accepted and monitored
- non-finite input rejection behavior is acceptable to the host control stack

This is an `integration-ready under stated assumptions` posture, not a flight qualification claim.

## Not Yet Justified for Blind Trust

Not justified today:

- blind trust as a primary flight-critical control component
- claims of hard-real-time certification
- claims of certified WCET
- claims of zero-allocation-after-init runtime for the bounded hot path
- claims of target qualification based only on host timing
- claims of whole-crate fixed-point embedded readiness

Reasons:

- [ONLINE_PATH_ALLOCATION_AUDIT.md](ONLINE_PATH_ALLOCATION_AUDIT.md) still documents bounded
  per-sample allocations in the live path
- timing evidence is observed and target-facing, but not certification-grade
- fixed-point evidence is strong within the tested live subset only
- the crate remains `std`-bound overall

## What Evidence Would Still Be Required

To move from advisory use toward stronger flight-loop trust, the remaining evidence would need to
include:

- target-platform timing measurements on the actual processor / RTOS configuration
- stronger allocator instrumentation proving no heap allocation after initialization in the live
  path, or a tighter and accepted runtime-allocation envelope
- stronger proof or audit coverage for the bounded live ingress path
- broader fixed-point coverage across more scenarios and integration paths
- program-specific assurance evidence beyond crate-level deterministic software tests

## Bottom Line

Current honest position:

- strong enough for pilot evaluation
- strong enough for bounded advisory / monitor use under stated assumptions
- strong enough for direct flight-loop integration experiments when the integration owner accepts
  the documented gaps
- not certifiable as-is

That distinction is intentional. The crate shows observed bounded behavior and integration-oriented
discipline. It does not claim certified flight-loop trust.
