# Detector subset optimization — Phase η.5

Greedy forward by selectivity rank: progressively include
the top-K detectors (sorted by cross-fixture mean selectivity
from Phase ζ.3 audit). Per K: full LO-CV across 12 fixtures.

Stop criterion: 95% recall plateau — smallest K where mean
fault recall reaches ≥ 95% of the all-detectors baseline.

**Total detectors:** 203  
**Baseline mean fault recall:** 0.9167  
**95% threshold:** 0.8708

## Subset trajectory

| K | Recall | RSCR | FP rate | Typed-confirmed | Replay |
|--:|-------:|-----:|--------:|----------------:|:------:|
| 5 | 0.9167 | 14.3056 | 0.0370 | 1 | 12 / 12 |
| 10 | 0.9167 | 14.3056 | 0.0370 | 1 | 12 / 12 |
| 15 | 0.9167 | 14.3056 | 0.1095 | 1 | 12 / 12 |
| 25 | 0.9167 | 14.3056 | 0.1095 | 1 | 12 / 12 |
| 50 | 0.9167 | 14.3056 | 0.1311 | 1 | 12 / 12 |
| 100 | 0.9167 | 14.3056 | 0.2763 | 2 | 12 / 12 |
| 150 | 0.9167 | 14.3056 | 0.2933 | 3 | 12 / 12 |
| 203 | 0.9167 | 14.3056 | 0.3539 | 4 | 12 / 12 |

## Verdict

**Minimal sufficient subset: K = 5** (smallest tested K where
mean fault recall reaches ≥ 0.8708). The remaining 198 detectors
contribute either redundantly (covered by top-5) or not at all
on the current 12-fixture surface.

## Top-15 detectors (the operational core)

1. `buishand_range`
2. `bottom_up_segmentation`
3. `wbs2`
4. `mcusum`
5. `dp_cpd`
6. `bayesian_offline_cpd`
7. `e_detector`
8. `mdl_change`
9. `log_isi_burst`
10. `max_interval_burst`
11. `alexandersson_snht`
12. `allan_variance`
13. `anderson_darling`
14. `ansari_bradley`
15. `ar1_residual`

## Honest empirical reading

Greedy-by-selectivity is a tractable proxy for true greedy
forward selection (which would require ~205 × ~30 LO-CV runs).
It captures the minimal-sufficient-subset story for cases
where detector contributions are roughly independent. For
highly correlated detector subsets (where greedy may include
redundant evidence), true greedy forward would prune more
aggressively. Partner-data engagements with sharper selectivity
signal can refine this curve.
