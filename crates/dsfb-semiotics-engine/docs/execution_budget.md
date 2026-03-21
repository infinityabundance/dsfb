# Execution Budget

The crate now ships a formal benchmark target at [`benches/execution_budget.rs`](../benches/execution_budget.rs).

Measured benchmark families:

- one bounded online-engine step
- one bounded online-engine batch step
- one CSV replay-driver advance step
- one semantic retrieval call on the builtin bank
- one semantic retrieval call on an enlarged typed bank

Measured host-side medians from `cargo bench --bench execution_budget -- --sample-size 10`
on the development host used for this crate pass:

- bounded online engine step: about `1.63 ms`
- bounded online engine batch step: about `1.88 ms`
- CSV replay step: about `2.33 us`
- semantic retrieval, builtin bank: about `12.6 us`
- semantic retrieval, enlarged bank: about `74.2 us`

Batch-versus-scalar interpretation:

- on the documented host, one 3-sample batch step was slightly cheaper than three repeated scalar
  steps when compared against the mean timing figures from the timing determinism report
- that corresponds to an observed call-overhead reduction of roughly `1.8%` on this host
- the crate still recommends batch ingestion as the primary API for multi-axis or IMU-class feeds
  because the interface semantics stay deterministic while FFI call count drops materially

What this benchmark is and is not:

- It measures execution on the host that ran the benchmark.
- It is complemented by the timing determinism report for mean, median, p95, p99, p99.9, maximum, and jitter summaries.
- It does not claim deterministic timing guarantees across other hosts.
- It does not invent Cortex-M, ARM, or flight-computer numbers that were not actually measured.
- It is an execution-budget aid for reviewers, not a certification claim.

Rerun locally:

```bash
cargo bench --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bench execution_budget
```

Generate the separate timing determinism report:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-timing-determinism
```

The retrieval-scaling artifact exported by normal runs remains separate: that artifact reports
deterministic candidate-count scaling for indexed versus linear narrowing, while this benchmark
captures host-side execution budget on the measured platform.
