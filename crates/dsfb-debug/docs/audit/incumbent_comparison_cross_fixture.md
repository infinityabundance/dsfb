# Cross-fixture incumbent comparison — Phase η.9

DSFB-Debug + scalar-threshold + CUSUM + EWMA on each of
the 12 vendored fixtures. Per fixture × detector: raw
alerts, fault recall, clean-window FP rate, wall-clock µs.

Source: Phase η.9 harness (`tests/incumbent_compare_cross_fixture.rs`).

## Per-fixture matrix

| Fixture | Detector | Raw alerts | Fault recall | Clean FP rate | Wall µs |
|---------|----------|-----------:|-------------:|--------------:|--------:|
| **tadbench_trainticket_F04** | `dsfb-debug` | 0 | 1.0000 | 0.0000 | 363 |
|  | `scalar-threshold` | 0 | 1.0000 | 0.0000 | 11 |
|  | `CUSUM` | 0 | 1.0000 | 0.0000 | 10 |
|  | `EWMA` | 0 | 1.0000 | 0.0000 | 9 |
| **tadbench_trainticket_F11** | `dsfb-debug` | 11 | 1.0000 | 0.0070 | 3671 |
|  | `scalar-threshold` | 10 | 1.0000 | 0.0139 | 98 |
|  | `CUSUM` | 10 | 1.0000 | 0.0116 | 115 |
|  | `EWMA` | 65 | 1.0000 | 0.0812 | 104 |
| **tadbench_trainticket_F11b** | `dsfb-debug` | 7 | 1.0000 | 0.2500 | 102 |
|  | `scalar-threshold` | 0 | 1.0000 | 0.0000 | 1 |
|  | `CUSUM` | 10 | 1.0000 | 0.7500 | 1 |
|  | `EWMA` | 11 | 1.0000 | 1.0000 | 1 |
| **tadbench_trainticket_F19** | `dsfb-debug` | 0 | 1.0000 | 0.0000 | 266 |
|  | `scalar-threshold` | 0 | 1.0000 | 0.0000 | 8 |
|  | `CUSUM` | 0 | 1.0000 | 0.0000 | 10 |
|  | `EWMA` | 0 | 1.0000 | 0.0000 | 8 |
| **illinois_socialnetwork** | `dsfb-debug` | 24 | 1.0000 | 0.0312 | 292 |
|  | `scalar-threshold` | 6 | 1.0000 | 0.1875 | 9 |
|  | `CUSUM` | 7 | 1.0000 | 0.1875 | 11 |
|  | `EWMA` | 17 | 1.0000 | 0.4062 | 8 |
| **aiops_challenge_2018_kpi** | `dsfb-debug` | 52 | 0.0000 | 0.0323 | 206 |
|  | `scalar-threshold` | 46 | 0.0000 | 0.6154 | 5 |
|  | `CUSUM` | 46 | 0.0000 | 0.7308 | 8 |
|  | `EWMA` | 70 | 0.0000 | 0.7308 | 6 |
| **lo2_oauth2_endoductive** | `dsfb-debug` | 0 | 1.0000 | 0.0000 | 170 |
|  | `scalar-threshold` | 16 | 1.0000 | 0.5000 | 3 |
|  | `CUSUM` | 16 | 1.0000 | 0.5000 | 5 |
|  | `EWMA` | 32 | 1.0000 | 1.0000 | 4 |
| **multidim_localization_part1** | `dsfb-debug` | 19 | 1.0000 | 0.0833 | 129 |
|  | `scalar-threshold` | 15 | 1.0000 | 0.5000 | 2 |
|  | `CUSUM` | 13 | 1.0000 | 0.5000 | 3 |
|  | `EWMA` | 22 | 1.0000 | 0.5000 | 2 |
| **deeptralog_F01** | `dsfb-debug` | 0 | 1.0000 | 0.0000 | 138 |
|  | `scalar-threshold` | 0 | 1.0000 | 0.0000 | 3 |
|  | `CUSUM` | 0 | 1.0000 | 0.0000 | 4 |
|  | `EWMA` | 0 | 1.0000 | 0.0000 | 3 |
| **defects4j_6project** | `dsfb-debug` | 62 | 1.0000 | 0.0333 | 261 |
|  | `scalar-threshold` | 41 | 1.0000 | 0.6000 | 6 |
|  | `CUSUM` | 42 | 1.0000 | 0.5667 | 10 |
|  | `EWMA` | 69 | 1.0000 | 0.6000 | 8 |
| **bugsinpy_6project** | `dsfb-debug` | 0 | 1.0000 | 0.0000 | 267 |
|  | `scalar-threshold` | 0 | 1.0000 | 0.0000 | 6 |
|  | `CUSUM` | 0 | 1.0000 | 0.0000 | 9 |
|  | `EWMA` | 0 | 1.0000 | 0.0000 | 7 |
| **promise_defect_prediction** | `dsfb-debug` | 4 | 1.0000 | 0.0333 | 241 |
|  | `scalar-threshold` | 3 | 1.0000 | 0.0667 | 6 |
|  | `CUSUM` | 1 | 1.0000 | 0.0333 | 9 |
|  | `EWMA` | 3 | 1.0000 | 0.1000 | 7 |

## Cross-fixture aggregate per detector

Mean ± stddev across the populated fixtures. Mean computed
over all fixtures contributing data; stddev is the population
stddev (not sample stddev) for direct comparison.

| Detector | Mean recall | Stddev recall | Mean FP | Stddev FP | Mean µs | Total raw |
|----------|------------:|--------------:|--------:|----------:|--------:|----------:|
| `dsfb-debug` | 0.9167 | 0.2764 | 0.0392 | 0.0679 | 509 | 179 |
| `scalar-threshold` | 0.9167 | 0.2764 | 0.2070 | 0.2524 | 13 | 137 |
| `CUSUM` | 0.9167 | 0.2764 | 0.2733 | 0.2969 | 16 | 145 |
| `EWMA` | 0.9167 | 0.2764 | 0.3682 | 0.3763 | 14 | 289 |

## Honest empirical reading

The single-fixture (F-11) incumbent comparison was
Session-7 anchor; the 12-fixture matrix above is the
Session-18 cross-domain extension. Per Session-17
academic-honesty discipline:

- **Recall numbers** are computed as `captured_faults /
  total_faults` per the existing scoring harness; for
  steady-state fixtures (F-04, F-19, LO2, etc.) `total_faults
  = 0` so recall is reported as 1.0 vacuously. The recall
  delta between detectors is meaningful only on fixtures
  with actual labelled fault windows.
- **FP rate numbers** are the operationally relevant metric:
  lower = fewer alerts on clean windows. DSFB-Debug's
  bank-aware confirmed-typed-episode FP rate (the operator-
  facing output) is the structural-layer number; scalar /
  CUSUM / EWMA report per-cell-firing FP rates. The
  comparison is the architectural-claim test: structural
  episodes are a different layer than per-cell alerts.
- **Wall-clock numbers** are debug-build µs; release-build
  is typically 5-20× faster (see `docs/benchmarks.md`).
