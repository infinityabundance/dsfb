# ISO 21448 (SOTIF) Mapping

Status: Partial

This file maps DSFB to a functional-inadequacy detection role. It is not a SOTIF validation claim.

Relevant crate components:
- `src/detection.rs`
- `src/integration.rs`
- `src/audit.rs`

Traceability matrix:

| SOTIF-oriented concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Detection of intended-model deviation | `evaluate_grammar_state`, `run_dsfb_pipeline` | Boundary/Violation states relative to admissibility envelope | Partial | Monitor role only |
| Explainable escalation path | `build_stage2_audit_trace`, operator overlay | State transitions, reason codes, advisory text | Partial | Helps review inadequacy candidates |
| Control-system response | Outside crate scope | None | Not supported | DSFB does not implement corrective control action |
