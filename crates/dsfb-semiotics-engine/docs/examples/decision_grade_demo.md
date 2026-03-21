# Decision-Grade Demo

This demo is intended to answer a practical question:

"At what time does the structural interpretation change enough that an operator or monitor would
act differently?"

The recommended path uses the synthetic A-PNT-oriented scenario:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario imu_thermal_drift_gps_denied
```

The resulting run exports an event timeline artifact:

- `csv/imu_thermal_drift_gps_denied_event_timeline.csv`
- `json/imu_thermal_drift_gps_denied_event_timeline.json`

## Time-Ordered Narrative

- `t ≈ 60 s`: the GPS-denied blackout window begins. Residuals are still modest, but the scenario
  transitions into the higher-risk interval where unaided IMU drift matters.
- `t ≈ 75 s`: thermal-style drift begins accumulating on the x-axis residual. The syntax layer now
  has a growing outward-structure basis rather than only bounded jitter.
- `t ≈ 120 s`: the mode-switch pulse injects an abrupt slew-rich event. Grammar and syntax should
  now show a materially different structural concern from the earlier slow drift.
- `t > 120 s`: semantic retrieval has enough structure to move from weak evidence toward a more
  constrained drift-compatible or event-compatible interpretation, while still remaining
  conservative if the bank does not justify a unique label.

## Decision Implication

Conservative interpretation only:

- before the blackout and early drift, a monitor may continue nominal observation
- once the grammar layer surfaces a sustained admissibility concern, a downstream integrator has a
  concrete trigger for heightened scrutiny, fallback logic, or operator notification
- once the mode-switch pulse and later semantic narrowing appear, the situation is no longer just
  "an alarm"; it is a typed structural transition with explicit timing and reason text

This is not a prescribed control law. It is a deterministic event narrative that makes the
operational meaning of the transition sequence legible.
