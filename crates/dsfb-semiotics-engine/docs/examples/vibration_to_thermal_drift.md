# Vibration to Thermal Drift Example

The example `examples/vibration_to_thermal_drift.rs` demonstrates a physically grounded residual story:

- a high-frequency vibration-like phase with elevated slew
- a slower monotone drift-like phase that resembles thermal expansion or bias creep
- a bounded live interpretation trace that reports syntax, grammar reason, trust, and semantic retrieval changes

## Run

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --example vibration_to_thermal_drift
```

## Unit Interpretation

This example uses residual units that inherit directly from the source signal and treats them as millimeters:

- residual: `mm`
- drift: `mm/s`
- slew: `mm/s^2`

That is, the example discusses drift in millimeters/second and slew in millimeters/second^2.

If a deployment uses a unitless residual or another inherited unit system, the same layered interpretation logic still applies, but the exported numbers should be read in those inherited units rather than treated as universally physical by default.

## Why It Exists

The example is intentionally concrete. It helps reviewers and operators see how vibration-like slew-rich behavior can remain structurally distinct from a later drift-dominated phase without turning the crate into a field-validation claim.
