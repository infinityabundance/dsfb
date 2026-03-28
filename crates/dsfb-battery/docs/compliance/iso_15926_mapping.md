# ISO 15926 Mapping

Status: Partial

This file maps DSFB entities to lifecycle-information-style semantic concepts. It is not a formal ISO 15926 data model implementation.

Relevant crate components:
- `src/types.rs`
- `src/audit.rs`

Traceability matrix:

| DSFB concept | ISO 15926-style semantic role | Crate component | Status | Notes |
|---|---|---|---|---|
| Capacity sample | Observed lifecycle property value | `BatteryResidual.capacity_ah` | Partial | No formal ontology serialization is emitted |
| Sign tuple | Structured condition descriptor | `SignTuple` | Partial | Mapped conceptually only |
| Grammar state | Lifecycle condition classification | `GrammarState` | Partial | Finite-state semantics are explicit |
| Audit event | Traceable lifecycle event | `AuditEvent` | Partial | Artifact-level mapping only |
