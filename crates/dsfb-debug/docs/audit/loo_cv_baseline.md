# Leave-one-fixture-out cross-validation: Baseline (Phase ζ.2)

Source: Phase ζ.2 LO-CV harness (`src/audit/loo_cv.rs`).

**Fixtures observed:** 12
**Fixtures with deterministic replay verified:** 12 / 12

## Cross-fixture aggregate

| Metric | Mean | Stddev | LO-CV gate floor (mean−0.5·stddev) |
|--------|-----:|-------:|----------------------------------:|
| RSCR | 14.3056 | 20.6310 | 3.9901 |
| Clean-window FP rate | 0.3539 | 0.2826 | (ceiling: 0.4952) |
| Fault recall | 0.9167 | 0.2764 | 0.7785 |

**Total raw alerts:** 35187
**Total episodes:** 18
**Total typed-confirmed episodes:** 4

## Per-fixture LO-CV deltas

Delta = (fixture metric) − (mean over other N-1 fixtures).
Large positive deltas indicate the fixture pulls the mean upward;
large negative deltas indicate the fixture pulls it downward.

### rscr

| Fixture | Delta |
|---------|------:|
| `tadbench_trainticket_F04` | -15.6061 |
| `tadbench_trainticket_F11` | -11.6061 |
| `tadbench_trainticket_F11b` | -7.9697 |
| `tadbench_trainticket_F19` | -15.6061 |
| `illinois_socialnetwork` | +10.5758 |
| `aiops_challenge_2018_kpi` | +41.1212 |
| `lo2_oauth2_endoductive` | -15.6061 |
| `multidim_localization_part1` | +5.1212 |
| `deeptralog_F01` | -15.6061 |
| `defects4j_6project` | +52.0303 |
| `bugsinpy_6project` | -15.6061 |
| `promise_defect_prediction` | -11.2424 |

### clean_window_fp_rate

| Fixture | Delta |
|---------|------:|
| `tadbench_trainticket_F04` | -0.2043 |
| `tadbench_trainticket_F11` | -0.3481 |
| `tadbench_trainticket_F11b` | +0.4321 |
| `tadbench_trainticket_F19` | -0.2406 |
| `illinois_socialnetwork` | +0.0571 |
| `aiops_challenge_2018_kpi` | +0.4111 |
| `lo2_oauth2_endoductive` | +0.2957 |
| `multidim_localization_part1` | +0.1594 |
| `deeptralog_F01` | -0.3861 |
| `defects4j_6project` | +0.3776 |
| `bugsinpy_6project` | -0.3133 |
| `promise_defect_prediction` | -0.2406 |

### fault_recall

| Fixture | Delta |
|---------|------:|
| `tadbench_trainticket_F04` | +0.0909 |
| `tadbench_trainticket_F11` | +0.0909 |
| `tadbench_trainticket_F11b` | +0.0909 |
| `tadbench_trainticket_F19` | +0.0909 |
| `illinois_socialnetwork` | +0.0909 |
| `aiops_challenge_2018_kpi` | -1.0000 |
| `lo2_oauth2_endoductive` | +0.0909 |
| `multidim_localization_part1` | +0.0909 |
| `deeptralog_F01` | +0.0909 |
| `defects4j_6project` | +0.0909 |
| `bugsinpy_6project` | +0.0909 |
| `promise_defect_prediction` | +0.0909 |

