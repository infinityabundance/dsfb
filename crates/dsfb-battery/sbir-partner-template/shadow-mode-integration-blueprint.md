# Shadow-Mode Integration Blueprint

Status: template only. Intended for planning discussion.

## Objective

Integrate `dsfb-battery` as a read-only advisory layer alongside an existing battery-monitoring or health-estimation stack without changing actuation, protection, or estimator tuning behavior.

## Conservative Integration Path

1. ingest partner telemetry or replay CSVs into a non-interfering host path
2. run DSFB offline or in shadow mode
3. emit advisory-only state, reason code, and optional validity/freshness token
4. log results for operator or engineering review
5. keep all host control decisions outside the DSFB boundary

## Initial Interfaces

- CSV or replay ingestion into the current Rust helper path
- static library linking through `include/dsfb_battery_ffi.h`
- optional wrapper examples under `ffi/` and `wrappers/c/`

## Evidence Outputs

- `audit_traces/.../stage2_detection_results.json`
- `compliance/...`
- `addendum/...`
- `resource_trace.json` when explicitly enabled
- `manifest.json` and `manifest.sha256`

## Out of Scope for This Template

- certification packages
- target-specific WCET claims
- direct control or protection logic
- field validation claims
