# Changelog

All notable changes to `dsfb-gray` should be recorded here.

The release discipline for this repository is:

- document public API additions and behavior changes
- document scoring-method changes explicitly
- document generated-artifact format changes explicitly
- avoid silent semver-significant changes to observer, scanner, or attestation outputs

## Unreleased

## 0.1.0 — 2026-04-16

Initial crates.io release.

- Deterministic static crate auditing via `dsfb-scan-crate` with `dsfb-assurance-score-v1` scoring.
- Attestation export: SARIF 2.1.0, in-toto v1, DSSE envelope.
- 12-entry heuristic motif bank for structural code-quality interpretation.
- Runtime observer with `TelemetryAdapter`, `StaticPriorSet`, `ReasonEvidence`.
- Fault-injection harness: clock drift, partial partition, channel backpressure, async starvation.
- Reproducible public-artifact generation through `dsfb-regenerate-public-artifacts`.
- Core observer modules available in `no_std` mode.
