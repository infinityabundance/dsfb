# Embedded Integration Guide

Status: engineer-facing integration notes for an advisory DSFB layer. This file does not claim certification, qualification, or deployment approval.

## Scope

`dsfb-battery` is currently split into:

- a host-side `std` path for CSV loading, JSON export, plotting, reviewer bundles, and helper workflows
- a conservative `no_std + alloc` core boundary covering `src/types.rs`, `src/math.rs`, `src/detection.rs`, and `src/ffi.rs`

The crate is not presented as heapless or fully `no_std`. The host-side artifact stack remains `std`-based.

## Resource Evidence Table

The current resource evidence comes from the opt-in `dsfb-battery-resource-trace` helper and its compile-time size assertions. Timing values are host-environment measurements only. Memory and SWaP-C rows below are either exact `size_of` values, compile-time budgets, or explicit estimates.

| Item | Current source | Mode | Notes |
|---|---|---|---|
| `SignTuple` object budget | `src/resource_trace.rs` | compile-time asserted | budget capped at 32 bytes |
| `EnvelopeParams` object budget | `src/resource_trace.rs` | compile-time asserted | budget capped at 32 bytes |
| `BatteryResidual` object budget | `src/resource_trace.rs` | compile-time asserted | budget capped at 64 bytes |
| `PipelineConfig` object budget | `src/resource_trace.rs` | compile-time asserted | budget capped at 80 bytes |
| `Theorem1Result` object budget | `src/resource_trace.rs` | compile-time asserted | budget capped at 80 bytes |
| Hot-loop state estimate | `resource_trace.json` | estimated | derived from fixed windows, counters, and envelope/config state |
| Heuristics-bank serialized bytes | `resource_trace.json` | measured | exact JSON file size |
| Loaded heuristics-bank bytes | `resource_trace.json` | estimated | lower-bound estimate from serde serialization length |
| Host time per cycle | `resource_trace.json` | measured | current host only; not MCU timing |
| Heap allocation count | `resource_trace.json` | not measured | intentionally left unavailable rather than fabricated |
| Stack usage | `resource_trace.json` | not measured | exact stack depth is not currently profiled |

For reviewer bundles, prefer wording such as:

- "see generated `resource_trace.json` for current host measurements"
- "see `memory_budget_report.txt` for compile-time object-size budgets"
- "no target-MCU WCET claim is made"

## FFI and Static Linking

The crate already exposes `crate-type = ["rlib", "staticlib"]` and a narrow C ABI:

- `include/dsfb_battery_ffi.h`
- `src/ffi.rs`

Wrapper examples:

- `ffi/dsfb_battery_addendum_example.c`
- `wrappers/c/dsfb_battery_summary_example.c`

Static-link sketch:

```bash
cargo build --release --lib
cc -Iinclude wrappers/c/dsfb_battery_summary_example.c \
  target/release/libdsfb_battery.a \
  -lm \
  -o dsfb_battery_summary_example
```

The exposed surface is intentionally narrow:

- `dsfb_battery_default_config`
- `dsfb_battery_evaluate_grammar_state`
- `dsfb_battery_evaluate_step_status`
- `dsfb_battery_run_capacity_summary`

This is an integration boundary, not a proof of embedded deployment readiness.

## `no_std + alloc` Status

Current support:

- deterministic residual, drift, and slew calculations
- grammar-state evaluation and persistence logic
- theorem-summary computation
- narrow FFI wrapper compilation

Current non-support:

- full artifact export stack
- plotting and figure generation
- CSV ingestion
- reviewer-bundle orchestration

Conservative checklist before any target bring-up:

- keep DSFB output on a read-only advisory path
- confirm that `alloc` availability is acceptable for the intended target
- treat host-side timing reports as guidance only, not target timing evidence
- preserve fail-silent handling when upstream samples or derived windows are invalid
- keep external watchdog, actuation, and protection logic outside the DSFB crate boundary

## Validity Token / Watchdog Consumption

The existing integration and addendum helpers already use a validity/freshness token concept. The intended pattern is:

1. upstream telemetry or residual producer emits data
2. DSFB computes an advisory summary
3. host attaches a validity token or freshness rule
4. downstream consumer ignores stale or absent DSFB advisory output

Illustrative host-side policy:

```text
if token.present && token.stream_valid && token.age_s <= host_limit:
    publish advisory-only DSFB state to operator or monitor
else:
    suppress DSFB advisory output and fall back to host policy
```

This is an integration note only. The current crate does not prove watchdog behavior or system-level safety preservation.

## Certification-Facing Mapping Table

These rows describe supportive alignment only.

| Topic | Current crate evidence | Status | Notes |
|---|---|---|---|
| DO-178C DAL-C advisory-layer alignment | `docs/compliance/do178c_dal_mapping.md`, `src/audit.rs`, `src/detection.rs` | Partial | deterministic advisory computation and artifact traceability are mapped; no airborne qualification package is supplied |
| DO-311A battery-monitoring support | `docs/compliance/industry_standards_mapping.md` | Partial | advisory degradation-monitor role only |
| UL 1973 monitoring traceability support | `docs/compliance/industry_standards_mapping.md` | Partial | deterministic artifacts and reproducible outputs support review; no UL claim is made |
| UL 9540 integration planning | current crate docs only | Not currently mapped directly | external system packaging, installation, and fire-protection evidence are outside current crate scope |
| ICD / bounded interface support | `docs/addendum/icd.md`, `include/dsfb_battery_ffi.h` | Partial | narrow interface contract and transport-agnostic notes are present |
| Validity/freshness handling | `src/integration.rs`, `docs/engineer_extensions.md`, addendum overlay outputs | Partial | advisory freshness support exists; deployment enforcement remains host-system work |

## Reviewer Usage

For a reviewer-facing bundle without touching the production mono-cell figure path:

```bash
cargo run --release --bin sbir-demo -- \
  --cell B0005 \
  --multicell \
  --trace-resources
```

This writes an isolated bundle under `outputs/sbir_demo/...` with audit traces, addendum/compliance outputs, manifests, and markdown summaries. It does not regenerate or rename the current mono-cell production figures.
