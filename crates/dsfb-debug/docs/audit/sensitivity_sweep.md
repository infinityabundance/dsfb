# Sensitivity sweep — Phase η.3

One-at-a-time variation: each parameter swept across 5
values; all other parameters held at `FusionConfig::ALL_DEFAULT`.
Per configuration: LO-CV aggregate across all 12 vendored
fixtures (verbatim from `run_fusion_evaluation` stdout).
Theorem 9 deterministic replay verified per configuration.

Source: Phase η.3 sweep harness (`tests/sensitivity_sweep.rs`).

## min_consensus

| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |
|------:|----------:|--------:|------------:|----------------:|:------:|
| 1 | 14.3056 | 0.7510 | 0.9167 | 4 | 12 / 12 |
| 3 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 5 | 14.3056 | 0.2047 | 0.9167 | 3 | 12 / 12 |
| 7 | 14.3056 | 0.1590 | 0.9167 | 3 | 12 / 12 |
| 9 | 14.3056 | 0.1317 | 0.9167 | 1 | 12 / 12 |

## margin_gate

| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |
|------:|----------:|--------:|------------:|----------------:|:------:|
| 0.10 | 14.3056 | 0.3539 | 0.9167 | 6 | 12 / 12 |
| 0.20 | 14.3056 | 0.3539 | 0.9167 | 5 | 12 / 12 |
| 0.30 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 0.40 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 0.50 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |

## scalar_k (3-sigma multiplier)

| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |
|------:|----------:|--------:|------------:|----------------:|:------:|
| 2.0 | 14.3056 | 0.3647 | 0.9167 | 4 | 12 / 12 |
| 2.5 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 3.0 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 3.5 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 4.0 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |

## cusum_h

| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |
|------:|----------:|--------:|------------:|----------------:|:------:|
| 2.0 | 14.3056 | 0.3708 | 0.9167 | 4 | 12 / 12 |
| 3.0 | 14.3056 | 0.3597 | 0.9167 | 4 | 12 / 12 |
| 4.0 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 5.0 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 6.0 | 14.3056 | 0.3507 | 0.9167 | 4 | 12 / 12 |

## ewma_lambda

| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |
|------:|----------:|--------:|------------:|----------------:|:------:|
| 0.05 | 14.3056 | 0.3539 | 0.9167 | 5 | 12 / 12 |
| 0.10 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 0.20 | 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |
| 0.30 | 14.3056 | 0.3513 | 0.9167 | 4 | 12 / 12 |
| 0.40 | 14.3056 | 0.3513 | 0.9167 | 4 | 12 / 12 |

## Honest empirical reading

Each parameter's response curve is read column-wise from
the per-table values above. Steep response = high
sensitivity (operator-side calibration matters); flat
response = low sensitivity (default works robustly).
Per Session-17 academic-honesty discipline, no parameter
setting is claimed superior on this 12-fixture surface
without LO-CV gate evidence; the sweep ledger is the
operator-side input to per-site calibration.
