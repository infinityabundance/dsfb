# Real-Time Contract

This document defines the current bounded online-path integration contract for
`dsfb-semiotics-engine`.

It is a deployment aid, not a certification claim.

Covered symbols:

- `OnlineStructuralEngine::new`
- `OnlineStructuralEngine::push_residual_sample`
- `OnlineStructuralEngine::push_residual_sample_batch`
- `ffi::dsfb_semiotics_engine_push_sample`
- `ffi::dsfb_semiotics_engine_push_sample_batch`

This contract covers the bounded live path only. It does not cover offline artifact generation,
PDF rendering, ZIP packaging, dashboard UI rendering, or notebook orchestration.

## Assumptions

- numeric backend: `f64` for the checked-in contract summary in
  [`docs/generated/real_time_contract_summary.json`](generated/real_time_contract_summary.json)
- default history buffer capacity: `64`
- bounded live path only; optional offline history remains explicitly separate
- same build, same settings, same bank, and same numeric backend are assumed for state-exact replay
- timing values are host-observed measurements, not certified WCET

## Memory Budget

The bounded online path retains only the most recent `N` residual samples in the fixed-capacity
ring buffer used by [`src/live/mod.rs`](../src/live/mod.rs).

Measured/derived profile points from the generated summary:

| Numeric Mode | Channels | Buffer Capacity | Estimated Bounded Bytes | Notes |
|---|---:|---:|---:|---|
| `f64` | 1 | 32 | 3200 | bounded live handle + ring slots + retained value storage |
| `f64` | 1 | 64 | 4992 | default single-channel profile |
| `f64` | 3 | 64 | 6064 | default 3-axis profile |
| `f64` | 3 | 128 | 10672 | enlarged bounded profile |

Computation method:

- engine handle stack bytes from `size_of::<OnlineStructuralEngine>()`
- ring slot bytes from `size_of::<Option<ResidualSample>>() * history_buffer_capacity`
- retained value heap bytes from `size_of::<f64>() * channel_count * history_buffer_capacity`
- channel-name storage bytes from `size_of::<String>() * channel_count`

Excluded from this growth budget:

- builtin or external heuristic-bank registry storage
- retrieval-index storage
- optional offline accumulation
- report, figure, PDF, JSON, CSV, and ZIP artifact structures

## Allocation Policy

Current allocation policy for the covered live path:

- initialization-time allocation: allowed
- bounded ring-buffer allocation: initialization only
- per-sample heap allocation: still present
- optional offline accumulation: explicit opt-in and out of contract

The current explicit gap is that `push_residual_sample` still materializes bounded `Vec`-backed
residual, drift, slew, sign, and status structures on each step. This is documented in
[`docs/ONLINE_PATH_ALLOCATION_AUDIT.md`](ONLINE_PATH_ALLOCATION_AUDIT.md) and in the generated
summary JSON. No-allocation-after-init is therefore not yet claimed.

## Panic Policy

Source-audited policy for the covered path:

- no intentional `panic!`, `unwrap`, or `expect` in the non-test bounded step path
- invalid inputs return structured `Result` errors instead of panicking
- Invalid inputs return structured `Result` errors instead of panicking.
- zero-capacity history creation returns an error
- malformed batch lengths return an error
- snapshot schema mismatch returns an error

This is an engineering policy and source audit, not a formal proof that every future edit remains
panic-free unless the tests continue to pass.

## NaN / Inf Policy

Ingress and output guards for the covered path:

- non-finite sample time is rejected at ingress
- non-finite residual values are rejected at ingress
- externally visible live status checks `time`, `residual_norm`, `drift_norm`, `slew_norm`,
  `trust_scalar`, and all projected sign coordinates for finiteness before returning
- if a non-finite value appears internally, the live path returns a structured error instead of
  emitting NaN or Inf through the live status

This policy applies to externally visible live status values. It does not claim that every internal
floating-point intermediate across the whole crate is formally proven finite.

## Timing Budget

Observed host-side timing from [`docs/TIMING_DETERMINISM_REPORT.md`](TIMING_DETERMINISM_REPORT.md):

| Measurement | Median | p99 | p99.9 | Max Observed | Notes |
|---|---:|---:|---:|---:|---|
| `scalar_push_sample` | `616728 ns` | `981176 ns` | `992276 ns` | `992276 ns` | bounded online scalar step |
| `batch_push_sample` | `1873025 ns` | `1951953 ns` | `2117250 ns` | `2117250 ns` | row-major batch path |
| `grammar_admissible_path` | `1373 ns` | `1422 ns` | `1433 ns` | `1433 ns` | admissible grammar fixture |
| `grammar_violation_path` | `1473 ns` | `2314 ns` | `3046 ns` | `3046 ns` | violation-like grammar fixture |
| `semantic_retrieval_builtin_bank` | `38762 ns` | `62627 ns` | `112379 ns` | `112379 ns` | builtin bank |
| `semantic_retrieval_enlarged_bank` | `187509 ns` | `197086 ns` | `201735 ns` | `201735 ns` | enlarged stress bank |

Interpretation rules:

- these are observed measurements on the stated host platform only
- they are not a certified WCET bound
- they are useful for integration budgeting and regression review
- any real platform claim must be remeasured on the target build and hardware

## Failure Handling

The covered path fails conservatively:

- invalid `dt` or invalid envelope configuration prevents initialization
- invalid sample width or non-finite inputs return `Err(...)` and do not intentionally panic
- snapshot binary corruption or schema mismatch returns `Err(...)`
- FFI callers should read numeric status codes first and display strings second

## Machine-Readable Summary

The companion machine-readable summary is committed at:

- [`docs/generated/real_time_contract_summary.json`](generated/real_time_contract_summary.json)

Regenerate it with:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-real-time-contract
```

## Non-Claims

This contract does not claim:

- certified WCET
- hard-real-time certification
- zero-allocation-after-init for the current live path
- full `no_std` compatibility
- field qualification or control-loop validation
