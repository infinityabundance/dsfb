# ISO/IEC 25010 Mapping

Status: Partial

This file maps the crate to selected software-quality characteristics. It is not a formal SQuaRE assessment.

Relevant crate components:
- `src/audit.rs`
- `src/lib.rs`
- `src/ffi.rs`
- `src/compliance.rs`

Traceability matrix:

| Quality characteristic | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Reliability / repeatability | `run_dsfb_pipeline`, `build_determinism_check` | Deterministic computation and repeated-run helper | Partial | No field-reliability statistics are claimed |
| Analysability | `stage2_detection_results.json`, operator overlay, compliance docs | Human-readable trace and mappings | Partial | Strong analyzability support, not independently rated |
| Interoperability | `src/ffi.rs`, `include/dsfb_battery_ffi.h`, `wrappers/plc/structured_text.st` | Narrow C ABI and PLC wrapper | Partial | Integration supports are additive only |
