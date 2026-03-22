# Fixed-Point Deployment Evidence

This document records the current fixed-point evidence for the bounded live path. It is intentionally
scoped. It does not claim whole-crate embedded readiness.

Supporting artifacts:

- [generated/fixed_point_deployment_evidence_f64.json](generated/fixed_point_deployment_evidence_f64.json)
- [generated/fixed_point_deployment_evidence_numeric_fixed.json](generated/fixed_point_deployment_evidence_numeric_fixed.json)

## Supported Scope

Covered today:

- bounded `OnlineStructuralEngine` live path
- scalar `push_residual_sample`
- batch `push_residual_sample_batch`
- live syntax label
- live grammar state / reason code
- live semantic disposition
- live selected heuristic IDs
- live trust scalar

Not yet covered:

- full offline artifact pipeline
- report / PDF / paper figure generation under `numeric-fixed`
- whole-crate `no_std`
- target qualification or hardware sign-off

## Precision Bounds Used

The current evidence compares the `f64` report against the `fixed_q16_16` report at the live-status
level. For the exercised scenarios:

- syntax labels matched exactly
- grammar states and reason codes matched exactly
- semantic dispositions matched exactly
- selected heuristic IDs matched exactly
- trust-scalar drift stayed below `2.5e-5`
- residual-norm drift stayed below `6.5e-6`

These are conservative observed deltas from the committed generated artifacts above.

## Scenarios Exercised

The generated reports cover three live-path scenarios:

- `imu_thermal_drift_gps_denied`
- `regime_switch`
- `abrupt_event`

This gives one flight-relevant IMU-style case plus two additional live/demo paths with different
structural signatures.

## Consistency Results

### `imu_thermal_drift_gps_denied`

- `f64`: `curvature-rich-transition`, `Boundary`, `RecurrentBoundaryGrazing`, `Unknown`
- `fixed_q16_16`: same syntax / grammar / semantic outcome

### `regime_switch`

- `f64`: `discrete-event-like`, `Boundary`, `Boundary`, `Match`
- `fixed_q16_16`: same syntax / grammar / semantic outcome and same selected heuristic

### `abrupt_event`

- `f64`: `mixed-structured`, `Admissible`, `Admissible`, `Match`
- `fixed_q16_16`: same syntax / grammar / semantic outcome and same selected heuristic

## Interpretation

Within the tested bounded live scope, the fixed-point backend now reads as deployment evidence
rather than mere preparation:

- the live classifications remained consistent across the exercised scenarios
- numeric drift stayed small and did not change the exported live interpretation
- the tested scope is explicit and intentionally limited

## Conservative Claim Boundary

What can be claimed honestly:

- the bounded live path has matching fixed-point and `f64` outcomes on the committed exercised
  scenarios within the documented tolerances

What cannot be claimed honestly:

- whole-crate embedded readiness
- full offline/report-path equivalence
- hardware qualification
- universal numeric equivalence outside the tested scope

## Regeneration

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml \
  --bin dsfb-fixed-point-evidence -- \
  --output-json crates/dsfb-semiotics-engine/docs/generated/fixed_point_deployment_evidence_f64.json

cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml \
  --features numeric-fixed --bin dsfb-fixed-point-evidence -- \
  --output-json crates/dsfb-semiotics-engine/docs/generated/fixed_point_deployment_evidence_numeric_fixed.json
```
