# IMU Thermal Drift Under GPS-Denied Blackout

The synthetic scenario `imu_thermal_drift_gps_denied` is an A-PNT-oriented illustrative case for
reviewers who want a concrete residual story instead of only abstract drift language.

Run it:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario imu_thermal_drift_gps_denied
```

Scenario assumptions:

- three residual channels representing IMU x/y/z angular-rate residuals in `rad/s`
- GPS-denied blackout begins at approximately `T+60 s`
- slowly accelerating thermal drift begins on the x axis at approximately `T+75 s`
- one abrupt navigation-mode switch event is injected near `T+120 s`
- y and z axes remain low-level jitter-like residuals for contrast

Inherited units:

- residual: `rad/s`
- drift: `rad/s^2`
- slew: `rad/s^3`

Expected structural interpretation, conservatively framed:

- the drifting axis is consistent with persistent outward migration structure during the blackout
- the mode-switch pulse is consistent with abrupt slew-rich transition structure
- the grammar layer should surface the admissibility interaction and reason text explicitly
- semantics may return a conservative drift-compatible or event-compatible motif, or remain
  ambiguity-qualified, depending on the current bank contents

This is a synthetic A-PNT illustration, not a claim of GPS-denied field validation.
