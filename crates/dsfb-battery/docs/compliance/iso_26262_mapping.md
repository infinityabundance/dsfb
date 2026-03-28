# ISO 26262 Mapping

Status: Partial

This file maps the crate to an advisory diagnostic-monitor role. It is not an ASIL certification claim.

Relevant crate components:
- `src/detection.rs`
- `src/integration.rs`
- `src/audit.rs`

Traceability matrix:

| ISO 26262-oriented concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Diagnostic monitor behavior | `run_dsfb_pipeline`, `evaluate_grammar_state` | Structural deviation detection from residual behavior | Partial | Advisory only, not a safety mechanism replacement |
| Diagnostic traceability | `build_stage2_audit_trace` | Event-level audit trace and reason codes | Partial | Useful for analysis; coverage metrics are not quantified to ASIL evidence standards |
| Freshness / validity support | `build_validity_token` | Optional validity token helper | Partial | Token helper is outside the production artifact path |
| Freedom from interference | External integration discipline | Read-only and advisory notes | Partial | Must be preserved by the consuming system |
