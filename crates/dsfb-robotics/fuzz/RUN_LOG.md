# Fuzz Harness Run Log

This log records actual fuzz-target executions of the dsfb-robotics
fuzz harnesses. Every entry is a one-shot record of: the target, the
iteration count, the wall-clock duration, the host platform, and the
findings (crashes / non-finite-output / panics).

The targets exist at:
- [`fuzz/fuzz_targets/engine_roundtrip.rs`](fuzz_targets/engine_roundtrip.rs)
  — interprets the fuzz input as a stream of little-endian f64s and
  runs `dsfb_robotics::observe` end-to-end. Asserts no panic, no
  out-of-bounds output, structurally-valid grammar / decision labels.
- [`fuzz/fuzz_targets/grammar_fsm.rs`](fuzz_targets/grammar_fsm.rs)
  — exercises the grammar FSM directly with arbitrary residual norms.

Reproduce locally with:

```bash
cd crates/dsfb-robotics
cargo +nightly fuzz run engine_roundtrip --release -- -runs=1000000
cargo +nightly fuzz run grammar_fsm --release -- -runs=1000000
```

A small seed corpus is committed at [`fuzz/corpus/engine_roundtrip/`](corpus/engine_roundtrip/)
and [`fuzz/corpus/grammar_fsm/`](corpus/grammar_fsm/) so subsequent
runs start from known-interesting inputs (monotonic ramp, bounce,
NaN/Inf-laced).

## Run history

| Date (UTC) | Target | Iterations | Duration | Host | Findings |
|---|---|---|---|---|---|
| 2026-04-25 | `engine_roundtrip` | 1 000 000 | 3 s | Linux x86_64 (CachyOS, Rust nightly-2025-11-21, libfuzzer-sys 0.4) | **0 crashes / 0 panics / 0 non-finite-output assertions** |
| 2026-04-25 | `grammar_fsm`        | 1 000 000 | 0 s | Linux x86_64 (CachyOS, Rust nightly-2025-11-21, libfuzzer-sys 0.4) | **0 crashes / 0 panics / 0 non-finite-output assertions** |

Both runs cleanly terminated at the requested iteration count with
no failure artefacts. The `fuzz/artifacts/` directory remains empty
because no input triggered a property assertion.

## Why these targets are sufficient

The engine has zero `unsafe`, no FFI surface, and immutable-only input
references on the public API (`&[f64]` not `&mut [f64]`). The
combination Miri × 3 alias models (tree borrows / stacked borrows /
no-std core) + Loom × 3 thread interleavings + Kani 6 harnesses + 188
unit / integration / proptest tests already provides strong soundness
coverage. The fuzz harness is the closing-the-loop check: does any
arbitrary byte stream interpreted as f64 input produce a panic or a
malformed output? After 1 M iterations × 2 targets: no.

This is documentation of a measurement, not a certification. The
fuzz harness is regenerable: a future code change that introduces an
unsoundness is expected to surface here on the next run; a clean log
has a finite shelf life and should be re-run before any tagged
release.
