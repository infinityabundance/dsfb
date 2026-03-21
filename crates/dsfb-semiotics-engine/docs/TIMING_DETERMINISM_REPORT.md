# Timing Determinism Report

Schema: `dsfb-semiotics-timing-determinism/v1`
Platform: `linux-x86_64`
Rust: `rustc 1.93.0 (254b59607 2026-01-19)`
Numeric mode: `f64`
Iterations: `400` measured after `32` warmup iterations

This report records observed host-side timing behavior. It is not a certified WCET analysis.

| Measurement | Mean (ns) | Median (ns) | p95 (ns) | p99 (ns) | p99.9 (ns) | Max (ns) | Jitter (ns) |
|-------------|-----------|-------------|----------|----------|------------|----------|-------------|
| scalar_push_sample | 637359 | 616728 | 763010 | 981176 | 992276 | 992276 | 375548 |
| batch_push_sample | 1877424 | 1873025 | 1908592 | 1951953 | 2117250 | 2117250 | 244225 |
| grammar_admissible_path | 1373 | 1373 | 1403 | 1422 | 1433 | 1433 | 60 |
| grammar_violation_path | 1505 | 1473 | 1522 | 2314 | 3046 | 3046 | 1573 |
| semantic_retrieval_builtin_bank | 39455 | 38762 | 41457 | 62627 | 112379 | 112379 | 73617 |
| semantic_retrieval_enlarged_bank | 187992 | 187509 | 191617 | 197086 | 201735 | 201735 | 14226 |

## Notes

- `scalar_push_sample`: Observed bounded live scalar step on the current host after warmup.
- `batch_push_sample`: Observed bounded live batch step on the current host after warmup.
- `grammar_admissible_path`: Observed grammar evaluation on an admissible fixture.
- `grammar_violation_path`: Observed grammar evaluation on a violation-like fixture.
- `semantic_retrieval_builtin_bank`: Observed semantic retrieval on the builtin bank.
- `semantic_retrieval_enlarged_bank`: Observed semantic retrieval on an enlarged synthetic bank.
- These are host-side observed timing summaries, not certified WCET bounds. Median and tail metrics describe the measured platform only.