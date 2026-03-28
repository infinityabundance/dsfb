# STC Support Scaffold

Status: Partial

This file describes what the crate can contribute to an STC-oriented evidence package. It is not an STC approval claim.

Relevant crate components:
- `src/audit.rs::build_stage2_audit_trace`
- `src/compliance.rs::run_compliance_workflow`
- `src/compliance.rs::build_stc_traceability_support`

Traceability matrix:

| Evidence need | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Configuration identification | `src/audit.rs`, `src/compliance.rs` | `config_hash` in audit trace and compliance traceability support JSON | Partial | Hashes support traceability but do not replace configuration management process evidence |
| Input-data identification | `src/audit.rs`, `src/compliance.rs` | `input_hash` in audit trace and compliance traceability support JSON | Partial | Input hashing is local artifact evidence only |
| Output reproducibility | `src/compliance.rs::build_determinism_check` | Repeated-run summary hash comparison | Partial | Local same-toolchain reproducibility check only |
| Artifact traceability | `stage2_detection_results.json`, `stc_traceability_support.json` | Deterministic summary fields and hashable artifacts | Partial | No STC review package template or approval workflow is supplied |
