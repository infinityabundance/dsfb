# What Changes Operationally

This note is for engineers and operators, not for pitch language. The question is simple:
what does DSFB change in practice when it is added to a baseline monitoring stack?

## Synthetic Example

In the synthetic matched-magnitude cases, the primary residual magnitude can look similar across
two cases while the higher-order structure diverges. A threshold-style monitor sees “similar
size.” DSFB exposes:

- different higher-order slew structure
- different grammar trajectory
- different semantic interpretation outcome

Decision impact:

- the engineer learns that first-order residual size alone is not enough to treat the cases as
  operationally equivalent

## NASA Bearings Example

In the NASA Bearings run, Figure 13 now shows the baseline comparator alarm view beside the DSFB
grammar and semantic timelines. The baseline answers “when did an alarm appear?” DSFB adds:

- when boundary behavior started to accumulate
- when structural transitions appeared
- when the semantic interpretation narrowed or remained conservative

Decision impact:

- a reviewer can distinguish “alarm exists” from “alarm plus interpretable structural progression”
- a maintenance or monitoring engineer can forward a timeline instead of only an alarm count

## IMU / GPS-Denied Example

In `imu_thermal_drift_gps_denied`, the event timeline separates:

- blackout onset
- thermal-style drift accumulation
- a later abrupt mode-switch / estimator-stress event

Decision impact:

- the output supports a more legible response than “navigation residuals degraded”
- it helps separate slow drift structure from an abrupt transition structure

## What DSFB Adds

Relative to a baseline stack, DSFB can add:

- typed syntax rather than only scalar magnitude
- grammar reason codes and trust scalar rather than only alarm state
- constrained semantic disposition rather than untyped anomaly language

## What It Still Does Not Decide

DSFB still does not decide on its own:

- root cause
- corrective control action
- certification or safety approval
- whether a semantic label is unique

The operational value is earlier and clearer structural understanding, not autonomous authority.
