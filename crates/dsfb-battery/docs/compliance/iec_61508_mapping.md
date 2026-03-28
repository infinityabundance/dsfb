# IEC 61508 Mapping

Status: Partial

This file provides a compatibility-style mapping for the DSFB advisory monitor. It is not an IEC 61508 SIL claim.

Relevant crate components:
- `src/detection.rs`
- `src/audit.rs`
- `src/ffi.rs`

Traceability matrix:

| IEC 61508-oriented concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Deterministic behavior | `run_dsfb_pipeline`, `evaluate_grammar_state` | Fixed arithmetic order and finite state outputs | Partial | Current implementation is deterministic but not SIL-certified |
| Separation from actuation/control | `InterfaceContract.read_only`, `advisory_only` | Read-only, advisory contract | Partial | External integration must preserve non-interference |
| Traceability of outputs | `build_stage2_audit_trace` | Audit-trace artifact with hashes | Partial | Useful for safety cases, not a complete safety lifecycle |
| Tool and process qualification | Outside crate scope | None | Not supported | No IEC 61508 process qualification is claimed |
