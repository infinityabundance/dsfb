# W3C SSN/SOSA Mapping

Status: Partial

This file maps DSFB signals into observation-style semantics. It does not claim conformance to a deployed ontology stack.

Relevant crate components:
- `src/types.rs`
- `src/audit.rs`
- `src/compliance.rs`

Traceability matrix:

| DSFB entity | SSN/SOSA-style role | Crate component | Status | Notes |
|---|---|---|---|---|
| Capacity measurement | Observation result | `BatteryResidual.capacity_ah` | Partial | No RDF or ontology export is emitted |
| Residual / drift / slew | Measurement derivative / observed property | `SignTuple` | Partial | Mapped at schema-concept level |
| Grammar state | Observation interpretation/classification | `GrammarState` | Partial | Explicit finite classification |
| Heuristic interpretation | Observation interpretation note | `ReasonCode`, `FailureModeObservation` | Partial | Advisory semantics only |
