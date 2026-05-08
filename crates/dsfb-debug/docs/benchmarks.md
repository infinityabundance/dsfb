# Fusion timing benchmarks — Phase η.6

Hand-rolled microbenchmark using `std::time::Instant`. No
`criterion` Cargo dependency (zero-runtime-deps discipline
preserved). Per fixture: 1 warm-up iteration (not measured)
+ 10 measured iterations; reports median + min + max
wall-clock µs and derived ns-per-cell latency.

Source: Phase η.6 bench harness (`tests/timing_benchmarks.rs`).
Theorem 9 deterministic replay verified per iteration.

Build flags: `cargo test --features "std paper-lock"` (debug build;
release build is typically 5-20× faster but not the documented
baseline because cargo-test defaults to debug).

## Per-fixture timing

| Fixture | Signals | Windows | Cells | Median µs | Min µs | Max µs | ns/cell |
|---------|--------:|--------:|------:|----------:|-------:|-------:|--------:|
| `tadbench_trainticket_F11b` | 6 | 4 | 24 | 1435 | 1397 | 1572 | 59791.7 |
| `tadbench_trainticket_F11` | 16 | 431 | 6896 | 16292250 | 16027667 | 16399686 | 2362565.3 |

## Honest empirical reading

The end-to-end `run_fusion_evaluation` per-cell latency on
the F-11 fixture (6,896 cells, 205 detectors, 9-axis fusion)
is the published debug-build benchmark. Production-deployment
sizing should:

1. Run `cargo test --release --features "std paper-lock"` for
   the realistic latency (debug builds run 5-20× slower).
2. Multiply ns/cell × num_signals × evaluation_rate to derive
   required CPU budget per pipeline.
3. The 9-axis bank-aware fusion and Theorem 9 deterministic-
   replay verification both contribute to the per-cell cost
   reported here; they are non-optional in `paper-lock` mode.

Per Session-17 academic-honesty discipline, no production-
latency claim is made — the numbers in this table are
verbatim test stdout on the run-time machine (cargo-test
debug build); a partner deployment should re-run the bench
on their hardware and capture site-specific numbers.
