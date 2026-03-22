# Target-Facing Timing Demo

This document is a target-facing bounded-execution demonstration. It is distinct from the broader
host benchmark report in [TIMING_DETERMINISM_REPORT.md](TIMING_DETERMINISM_REPORT.md).

Supporting artifact:

- [generated/target_facing_timing_demo.json](generated/target_facing_timing_demo.json)

## Purpose

The goal here is not to claim certified WCET. The goal is to show one tighter, more deployment-like
timing story than the general host benchmark suite.

## Constrained-Profile Assumptions

Measured profile:

- profile name: `constrained_host_profile`
- build expectation: `cargo run --release --bin dsfb-target-facing-timing-demo`
- bounded live path only
- builtin bank only
- `history_buffer_capacity=16`
- `offline_history_enabled=false`
- `safety_first` smoothing enabled
- no dashboard, report, PDF, or offline artifact work in the measured path
- 3-axis batch path used as an IMU-style advisory/monitor ingress surrogate

This is a `target-facing demonstration`, not an actual embedded-target certification artifact.

## Measured Online Paths

The demo measures:

- `single_axis_monitor_step`
- `imu_like_batch_step`
- `stress_violation_batch_step`

All numbers below are observed on the documented host under the constrained profile above.

## Observed Bounds

| Measurement | Median (ns) | p95 (ns) | p99 (ns) | Max (ns) |
|---|---:|---:|---:|---:|
| `single_axis_monitor_step` | 95,858 | 104,505 | 141,985 | 175,928 |
| `imu_like_batch_step` | 409,403 | 571,625 | 593,386 | 616,028 |
| `stress_violation_batch_step` | 193,370 | 294,048 | 383,504 | 412,078 |

Interpretation:

- the 3-axis IMU-style batch step stayed below `616,028 ns` in this observed constrained-profile
  run
- the stress-oriented violation-driving batch stayed below `412,078 ns` in this observed run
- these are observed bounds on the measured host only

## Measurement Method

The demo binary repeatedly executes the bounded live path after warmup and records wall-clock
latency per step with `Instant::now()`. It emits median, p95, p99, max, and jitter summaries to
the generated JSON artifact.

## Distinction From Other Timing Claims

- `host benchmarks`: [TIMING_DETERMINISM_REPORT.md](TIMING_DETERMINISM_REPORT.md) covers broader
  host-side timing behavior across more paths
- `target-facing demo`: this document narrows to a constrained deployment envelope and a smaller
  set of bounded live paths
- `certified WCET`: not claimed

## Regeneration

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --release \
  --bin dsfb-target-facing-timing-demo -- \
  --output-json crates/dsfb-semiotics-engine/docs/generated/target_facing_timing_demo.json
```

This command regenerates the machine-readable timing summary used here.
