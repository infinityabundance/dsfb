# Industry Standards Mapping

Status: Partial

This file unifies several battery-safety-oriented standards into a single DSFB monitoring-role mapping. It is not a certification claim.

Relevant crate components:
- `src/detection.rs`
- `src/audit.rs`
- `src/compliance.rs`

Traceability matrix:

| Standard | DSFB role | Crate evidence | Status | Notes |
|---|---|---|---|---|
| RTCA DO-311A | Advisory degradation monitor | Audit trace, reason codes, theorem summary | Partial | No RTCA qualification claim |
| IEC 62619 | Cell degradation trend support | Boundary/Violation states and lead-time reporting | Partial | No product compliance testing claim |
| MIL-PRF-32565C | Engineering monitoring support | Read-only audit trail and hashes | Partial | No military qualification claim |
| SAE J2929 | Hazard-observation support | Reason-code and hazard mapping scaffold | Partial | No vehicle-level safety validation claim |
| UL 1973 | Monitoring traceability support | Deterministic artifacts and reproducible outputs | Partial | No UL certification claim |
