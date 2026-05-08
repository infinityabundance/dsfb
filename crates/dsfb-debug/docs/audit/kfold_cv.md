# K-fold cross-validation (K = 4) — Phase η.2

Source: Phase η.2 K-fold harness (`src/audit/loo_cv.rs::aggregate_kfold_cv`).

**Folds:** 4
**Fixtures per fold:** 3 (last fold may be smaller)
**All folds Theorem 9 replay holds:** true

## Per-fold test-set aggregates

| Fold | Test fixtures | RSCR | FP rate | Fault recall | Replay |
|-----:|---------------|-----:|--------:|-------------:|:------:|
| 0 | `tadbench_trainticket_F04`, `tadbench_trainticket_F11`, `tadbench_trainticket_F11b` | 3.5556 | 0.3172 | 1.0000 | 3 / 3 |
| 1 | `tadbench_trainticket_F19`, `illinois_socialnetwork`, `aiops_challenge_2018_kpi` | 25.3333 | 0.4235 | 0.6667 | 3 / 3 |
| 2 | `lo2_oauth2_endoductive`, `multidim_localization_part1`, `deeptralog_F01` | 6.3333 | 0.3750 | 1.0000 | 3 / 3 |
| 3 | `defects4j_6project`, `bugsinpy_6project`, `promise_defect_prediction` | 22.0000 | 0.3000 | 1.0000 | 3 / 3 |

## Cross-fold aggregate

| Metric | Cross-fold mean | Cross-fold stddev |
|--------|----------------:|------------------:|
| RSCR | 14.3056 | 9.4860 |
| Clean-window FP rate | 0.3539 | 0.0488 |
| Fault recall | 0.9167 | 0.1443 |

## Honest empirical reading

K-fold CV averages over multiple test-set configurations
(in contrast to LO-1 which holds out one fixture at a time).
With N = 12 fixtures and K = 4, each fold tests on a multi-fixture set (lower variance per
fold than LO-1, but fewer folds than LO-1 — the two views
triangulate the cross-validation noise floor).
