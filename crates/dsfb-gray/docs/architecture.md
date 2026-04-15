# DSFB Gray Architecture

## Purpose

`dsfb-gray` combines a deterministic runtime observer with a broad static crate
audit and portable attestation outputs. The project is intentionally broad: the
goal is to help Rust developers improve code quality while preserving
standards- and certification-relevant review signals in one place.

## Main Layers

### 1. Runtime Observer

The runtime observer accepts immutable `ResidualSample` values and derives:

- residual sign
- drift
- slew
- admissibility-envelope position
- grammar state
- reason code
- optional audit-trace events

This layer is the core deterministic interpretation engine.

### 2. Static Crate Audit

The static scanner walks a crate tree and emits one canonical broad audit that
includes:

- safety and panic surface signals
- verification and governance evidence
- Power-of-Ten-inspired review rules
- advanced structural checks
- heuristic provenance motifs
- score and subscores
- remediation and verification guidance

The audit is designed as a review-improvement instrument, not a compliance
certificate.

### 3. Static-To-Runtime Bridge

The scanner can derive bounded structural priors from static findings. Those
priors bias runtime interpretation conservatively without overriding runtime
evidence.

### 4. Attestation and Evidence Export

The scanner exports:

- text reports
- SARIF findings
- in-toto statements
- DSSE envelopes

These artifacts support CI retention, review traceability, and supply-chain
portability.

## Canonical Flow

1. Source motifs are scanned into a canonical broad audit.
2. The audit produces findings, evidence IDs, score, and subscores.
3. Optional static priors are derived from those findings.
4. Runtime telemetry is interpreted by the observer.
5. Runtime results and static artifacts can be retained together.

## Non-Certification Boundary

DSFB keeps standards- and certification-relevant checks in scope, but the crate
does not claim literal compliance or certification. The audit approximates
review surfaces and helps maintainers improve readiness and code quality.
