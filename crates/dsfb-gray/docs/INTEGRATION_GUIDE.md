# Integration Guide

This document keeps the full DSFB scope intact and explains how the parts fit together in actual Rust workflows.

## Integration Paths

DSFB has three integration surfaces:

1. `Runtime observer`
   Ingest application telemetry as `ResidualSample` values and emit deterministic structural classifications.
2. `Static scanner`
   Scan crate source trees for structural motifs, assurance signals, and CI-facing findings.
3. `Attestation/export`
   Emit SARIF, in-toto, and DSSE artifacts so scan results can be retained, reviewed, and transported.

These are not separate products. The intended flow is:

`source motifs -> static findings -> derived static priors -> runtime observation -> audit trace -> attestation`

## Runtime Observer

The runtime path is library-first.

- Use `TelemetryAdapter<T>` to translate existing metrics, tracing snapshots, or domain records into `ResidualSample`.
- Feed those samples into `DsfbObserver`.
- Consume `ObservationResult`, especially:
  - `grammar_state`
  - `heuristic_match`
  - `reason_evidence`
  - `completed_episode`

`ReasonEvidence` is the important public hook for operator trust. It exposes the selected reason code, the matched heuristic, the explanatory description, the Rust-specific provenance string, and any applied static prior.

## Scan-To-Runtime Binding

The scanner no longer stops at a text report.

- `scan_crate_source(...)` produces a structured static report.
- `derive_static_priors_from_scan(...)` converts source-visible motifs into a bounded `StaticPriorSet`.
- `ObserverConfig::with_static_priors(...)` attaches that prior set to the runtime observer.

The bounded-prior rule is deliberate:

- static evidence can bias detection
- static evidence cannot override runtime evidence
- no prior lowers thresholds without a bounded clamp

This keeps the system context-aware without allowing the scanner to fabricate runtime truth.

## Canonical Audit

The scanner now emits one canonical broad audit.

- The evidence set is collected once.
- The score is computed once.
- Domain and standards interpretations are rendered as conclusion lenses at the end of the report.
- The audit improves code quality while also supporting compliance- and certification-oriented internal review without claiming certification.

## CI Usage

Typical CI flow:

1. Run `cargo test`.
2. Run `cargo clippy --all-targets -- -D warnings`.
3. Run `cargo run --bin dsfb-scan-crate -- <crate-root>`.
4. Publish the generated `output-dsfb-gray/dsfb-gray-<timestamp>/` folder as a build artifact.

The human-readable `.txt` report is for reviewers. The `.sarif.json`, `.intoto.json`, and `.dsse.json` outputs are for tooling, provenance, and audit retention.

## Trace Replay

The repository contains deterministic CSV outputs from the evaluation harness, and the public API is intentionally simple enough to support replay from recorded telemetry.

Current state:

- The crate supports replay-friendly ingestion through `TelemetryAdapter<T>`.
- The repository does not yet include a production trace artifact from a real external service.

That distinction matters. The replay path is real, but the current checked-in traces are still deterministic harness outputs, not production recordings.

## Non-Claims

- A static scan does not prove a live gray failure.
- A runtime episode does not prove physical root cause.
- A DSSE envelope proves provenance of the report, not certification of the crate.
- The DSFB audit is a guideline for improvement and review readiness, not a standards certificate.

Those boundaries are part of the design, not caveats added after the fact.
