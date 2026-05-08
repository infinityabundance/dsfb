# Bank refinement LO-CV ledger (Phase ζ.9)

Phase ζ.5 default-on calibrated weighted consensus, gated
by `audit::loo_cv::refinement_passes_gate(baseline, refined)`
with ε = 0.5·stddev, no regression on any of {RSCR, FP rate,
fault recall, replay-holds count}.

Source: Phase ζ.9 LO-CV harness (`tests/loo_cross_validation.rs`).

## Refinement under test

Phase ζ.5 calibrated overrides: 10 family-tier detectors with
mean cross-fixture `healthy_firing_rate > 0.25` AND
`fault_firing_rate = 0.0` filtered to weight 0
(`pelt`, `fpop`, `spatial_sign`, `cumulative_deviation`,
`bayesian_blocks`, `mcd`, `inspect_cpd`, `stahel_donoho`,
`zonotope_escape`, `depth_rank_control`).

## Comparison

| Metric | Baseline mean | Refined mean | Delta | Baseline stddev |
|--------|--------------:|-------------:|------:|----------------:|
| RSCR | 14.3056 | 14.3056 | +0.0000 | 20.6310 |
| Clean-window FP rate | 0.3539 | 0.3539 | +0.0000 | 0.2826 |
| Fault recall | 0.9167 | 0.9167 | +0.0000 | 0.2764 |
| Total typed-confirmed episodes | 4 | 4 | +0 | — |
| Total fusion episodes | 18 | 18 | +0 | — |
| Total raw alerts | 35187 | 35187 | +0 | — |
| Replay holds | 12 / 12 | 12 / 12 | — | — |

## Verdict

**LO-CV gate: ACCEPT.** All metrics within tolerance
(mean ≥ baseline − 0.5·stddev on every metric;
replay-holds count preserved). Refinement promoted to
canonical default per the user-locked Session-17 choice
"Apply LO-CV-passing refinements to canonical bank".

## Honest empirical reading

**Observable delta on this cross-fixture surface is exactly zero.**

The 10 filtered family-tier detectors had `fault_firing_rate = 0`
across all 12 vendored fixtures (verbatim from
`docs/audit/detector_selectivity.md`); they were never routing
evidence on labelled-fault windows in the first place. Filtering
them on healthy windows, where they were firing, has no observable
effect on the cross-fixture LO-CV aggregates because the
`window_tier_mask`-routed bank-aware evidence depends on whether
a detector's tier bit is set during the matched episode's window
range — and these detectors' bits were not being set during the
episode windows that actually got typed-confirmed.

This is not failure: the gate trivially accepts (no regression
anywhere), and the infrastructure ships ready for operator-side
site calibration. Partner-data engagements where these detectors
DO contribute observable evidence will provide the empirical
lever the audit harness can then gate.
