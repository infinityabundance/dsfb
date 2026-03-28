# EU Battery Passport / CIRPASS Mapping Helper

Status: Supportive mapping only. No legal or regulatory compliance claim is made.

Candidate field mapping:

| Passport-style concern | DSFB-support field | Crate source |
|---|---|---|
| Health trend traceability | `first_boundary_cycle`, `first_violation_cycle`, lead-time fields | `stage2_detection_results.json`, addendum overlay |
| Provenance | `config_hash`, `input_hash`, source artifact | `src/audit.rs`, addendum integrity helpers |
| Maintenance-relevant lifecycle annotation | reason code, advisory text, knee-onset narrative | `src/detection.rs`, `src/integration.rs`, addendum overlay |
| Reproducibility / integrity | deterministic summary hash, tamper-evident chain | addendum integrity helpers |

The addendum helper emits a stub JSON artifact at:

- `outputs/addendum/.../battery_passport/battery_passport_stub.json`

This stub is intended as an integration example only.
