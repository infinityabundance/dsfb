# DO-178C DAL Advisory Mapping

Status: Partial

This file is a traceability scaffold for an advisory, non-interfering DSFB role. It is not a DO-178C compliance or certification claim.

Relevant crate components:
- `src/detection.rs::evaluate_grammar_state`
- `src/detection.rs::run_dsfb_pipeline`
- `src/audit.rs::build_stage2_audit_trace`
- `src/export.rs::export_audit_trace_json`

Traceability matrix:

| Objective-style concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Deterministic advisory computation | `src/detection.rs` | Fixed-order arithmetic and threshold/persistence logic | Partial | Deterministic within current implementation; no airborne qualification package is provided |
| Requirements-to-output traceability | `src/audit.rs`, `src/export.rs` | Audit trace, summary outcome, reason codes, hashes | Partial | Traceability exists at artifact level |
| Non-interference / advisory role | `src/audit.rs::InterfaceContract` | `read_only = true`, `advisory_only = true` | Partial | Supports DAL-advisory framing only |
| Bounded behavior | `PipelineConfig`, finite counters, per-cycle evaluation | Fixed windows and finite state values | Partial | No WCET qualification report is provided |
