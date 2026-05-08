# Bootstrap confidence intervals (Phase η.1)

Cross-fixture LO-CV aggregates with 95% percentile-based
bootstrap confidence intervals. Fixture-level resampling
with replacement; deterministic LCG sampler (no `rand`
dependency); fixed seed for reproducibility.

Source: Phase η.1 audit harness (`src/audit/bootstrap.rs`).

**Iterations:** 1000  
**Fixtures resampled per iteration:** 12  
**LCG seed:** 0x61736566656c7900

## 95% confidence intervals

| Metric | Point estimate | 95% CI lower | 95% CI upper | CI width |
|--------|---------------:|-------------:|-------------:|---------:|
| RSCR (structural mean) | 14.3056 | 5.9444 | 21.9167 | 15.9722 |
| Clean-window FP rate | 0.3539 | 0.2084 | 0.4933 | 0.2849 |
| Fault recall | 0.9167 | 0.7500 | 1.0000 | 0.2500 |
| Typed-confirmed / fixture | 0.3333 | 0.0833 | 0.5833 | 0.5000 |

## Honest empirical reading

With N = 12 bounded fixtures, the bootstrap CI is wide by
construction — the empirical surface does not yet support
tight aggregate claims. The CI width is itself the honest
read: more fixtures (Phase II partner data) shrink the
interval; cross-fixture variance dominates the cross-
validation noise. The point estimates are the operator
anchor; the CI bounds are the publication-honest range.
