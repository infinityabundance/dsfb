# DSFB A-PNT Brief

## Problem

In GPS-denied or degraded-navigation intervals, residual monitors often collapse behavior into
threshold alarms. That is useful, but it can hide whether the residual history is drift-like,
slew-like, repeatedly boundary-grazing, or semantically underconstrained.

## What DSFB Does Differently

`dsfb-semiotics-engine` keeps the residual interpretation layered and deterministic:

- residual
- sign
- syntax
- grammar
- semantics

Instead of returning only "alarm/no alarm," it can expose:

- typed syntax
- grammar reason code and trust scalar
- constrained semantic disposition (`Match`, `CompatibleSet`, `Ambiguous`, `Unknown`)

## Concrete A-PNT Example

The crate includes `imu_thermal_drift_gps_denied`, a deterministic 3-axis illustrative scenario
with:

- GPS blackout beginning near `T+60 s`
- thermal-style drift onset near `T+75 s`
- a mode-switch pulse near `T+120 s`

The expected value is structural legibility:

- drift onset becomes visible before a generic "everything is bad" summary
- the mode-switch pulse appears as a separate abrupt structural event
- the exported event timeline makes the sequence forwardable to integrators and reviewers

## Integration Boundary

The crate now carries:

- a bounded live path
- first-class batch ingestion
- a C ABI and C++ wrapper
- a Python binding
- a real-time contract and timing report

## One Concrete Number

On the documented Linux x86_64 host, observed worst-case bounded `push_sample` latency in the
timing determinism report was `992276 ns` for the scalar path under the current measurement setup.

This is an observed host-side number only. It is not a certified WCET claim.
