# Capability Ladder

This document keeps the broad DSFB scope explicit without collapsing the project into vague marketing language.

## Phase A: Structural Scanner and Attestation

- Operator question:
  What can fail here, based on the source tree I actually ship?
- Primary artifact:
  `*_scan.txt`, `*.sarif.json`, `*.intoto.json`, `*.dsse.json`
- Success metric:
  Findings are traceable to evidence spans and can be retained in CI.
- Technical risk retired:
  Blindness to source-visible structural hazards and missing provenance.

## Phase B: Runtime Observer

- Operator question:
  Is a structural failure beginning now in the telemetry stream?
- Primary artifact:
  `ObservationResult`, `Episode`, `AuditTrace`
- Success metric:
  Deterministic, replayable structural classifications with typed reason codes.
- Technical risk retired:
  Reliance on scalar alarms that discard trajectory structure.

## Phase C: Static-To-Runtime Binding

- Operator question:
  What should the runtime watch more closely because of the codebase it is attached to?
- Primary artifact:
  `StaticPriorSet`, derived scanner priors, runtime-applied prior evidence
- Success metric:
  Runtime outputs record when a structural prior influenced heuristic matching.
- Technical risk retired:
  Scanner/runtime disconnect where static knowledge never reaches live observation.

## Phase D: Domain / Hazard Interpretation Lenses

- Operator question:
  What do these findings mean in my mission context?
- Primary artifact:
  One canonical scan report with domain-specific conclusion lenses at the end
- Success metric:
  The same evidence can be interpreted coherently for cloud-native, distributed, industrial, supply-chain, or certification-preparation review without fragmenting the audit.
- Technical risk retired:
  Over-generalized findings with no operational context.

## Why This Stays Broad

The project is intentionally broad. The discipline is not scope reduction; it is evidence separation.

- Source-visible structure belongs to the scanner.
- Live telemetry structure belongs to the observer.
- Provenance belongs to the attestation layer.
- Domain meaning belongs to the concluding interpretation lenses.

That separation keeps the scope wide while preventing overclaiming.
