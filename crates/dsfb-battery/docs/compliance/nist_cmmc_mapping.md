# NIST 800-171 / CMMC 2.0 Mapping

Status: Partial

This file maps crate artifacts to integrity and audit-support concerns. It is not a control implementation or assessment report.

Relevant crate components:
- `src/audit.rs`
- `src/heuristics.rs`
- `config/heuristics_bank_v1.json`
- `config/heuristics_bank_v1.sha256`

Traceability matrix:

| Control-family concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Artifact integrity | `src/audit.rs`, `src/compliance.rs` | Config/input hashes and deterministic summary hashes | Partial | Integrity evidence exists for local artifacts |
| Configuration integrity | `config/heuristics_bank_v1.json`, `src/heuristics.rs` | Versioned heuristics bank and SHA-256 verification | Partial | No enterprise CM system is implemented in the crate |
| Audit traceability | `stage2_detection_results.json` audit trace | Event-level trace, reason codes, hashes | Partial | Traceability is present, immutability enforcement is external |
| Access control and media protection | Outside crate scope | None in crate | Not supported | These controls require deployment environment support |
