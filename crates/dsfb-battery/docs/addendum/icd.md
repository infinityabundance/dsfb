# Interface Control Document

Status: Engineering ICD for an advisory DSFB interface. This is not a bus-specific deployment claim.

## Role

DSFB is positioned here as a read-only, advisory, non-interfering interpretive layer. It does not claim control authority.

## Inputs

Production mono-cell path:

| Field | Type | Units | Source |
|---|---|---|---|
| `cycle` | `usize` | cycle index | CSV / upstream host |
| `capacity_ah` | `f64` | Ah | production mono-cell dataset |

Addendum / integration helper path:

| Field | Type | Units | Source |
|---|---|---|---|
| `residual` | `f64` | channel units | upstream residual producer |
| `drift` | `f64` | units/cycle | upstream residual producer or DSFB preprocessing |
| `slew` | `f64` | units/cycle² | upstream residual producer or DSFB preprocessing |
| `envelope_rho` | `f64` | channel units | host-declared admissibility envelope |
| `drift_counter`, `slew_counter` | `usize` | count | host-maintained persistence counters or helper logic |

## Outputs

| Field | Type | Meaning |
|---|---|---|
| `grammar_state` | enum | `Admissible`, `Boundary`, `Violation` |
| `reason_code` | optional enum | typed interpretation under current rules |
| `t_star` | `usize` | theorem-derived bound summary |
| `validity_token` | struct | freshness / output-present helper for advisory consumers |
| `tri_state_color` | string / code | `Green`, `Yellow`, `Red` |
| `advisory_text` | string | operator-facing overlay wording |

## Timing Expectations

- The current production crate is batch-oriented over full sequences.
- The addendum ICD also documents a step-oriented wrapper surface through `src/ffi.rs`.
- The per-update rule set is constant-work with fixed counters and fixed-width windows, but no hard real-time certification claim is made.

## Invalid-Stream Behavior

- The current production audit contract sets `fail_silent_on_invalid_stream = true`.
- The emitted interface contract also distinguishes `fail_silent_defined = true` and `fail_silent_enforced = true`.
- During an invalid interval, normal classification output is suppressed and the audit trace emits `invalid_stream_gap` instead.
- The addendum validity token is a freshness/helper signal only.
- A consuming system can ignore advisory DSFB output when the validity token is absent or stale.

## Transport Agnosticism

- The crate does not require a specific bus stack.
- The mission-bus mapping in `docs/addendum/mission_bus_mapping.md` is conceptual and additive.

## Non-Interference Posture

- `read_only = true`
- `advisory_only = true`
- `requires_cloud_connectivity = false`
- `requires_model_retraining = false`

These are implementation-scoped properties of the emitted artifacts and wrapper notes, not certification claims.
