# Per-motif refinement report (Phases ζ.4 + ζ.8)

For each motif that typed on a confirmed positive fixture,
the table reports: current hand-curated affinity-tier mask
vs observed tier-firing on the matched episode; current
named-witness list vs observed top-K detectors-by-firing.

**Refinements are RECOMMENDATIONS, not bank mutations.**
Phase ζ.9 separately gates any merge through leave-one-
fixture-out cross-validation (`audit::loo_cv::refinement_passes_gate`).

Source: Phase ζ.4 + ζ.8 audit harness.

| Motif | Fixture | Curated mask | Observed mask | Divergence | Top witnesses (observed) |
|-------|---------|-------------:|--------------:|-----------|-----|
| `DeploymentRegressionSlew` | `tadbench_trainticket_F11` | 0x0340c203 | 0x015dd49f | Overlap | `bayesian_offline_cpd` (1.00), `corr_matrix_distance` (1.00), `cumulative_deviation` (1.00), `dp_cpd` (1.00), `fpop` (1.00) |
| `AuthenticationFailureSpike` | `tadbench_trainticket_F11` | 0x0e002021 | 0x055ff78d | Overlap | `anderson_darling` (1.00), `ar2_residual` (1.00), `arima_simplified` (1.00), `batschelet_concentration` (1.00), `bayesian_blocks` (1.00) |
| `AuthenticationFailureSpike` | `tadbench_trainticket_F11` | 0x0e002021 | 0x015df481 | Overlap | `arima_simplified` (1.00), `bayesian_blocks` (1.00), `bayesian_offline_cpd` (1.00), `burst_after_silence` (1.00), `canonical_correlation` (1.00) |
| `EnvelopeBreach` | `tadbench_trainticket_F11b` | 0x03040013 | 0x014de483 | Overlap | `arima_simplified` (1.00), `bayesian_blocks` (1.00), `bond_graph_residual` (1.00), `cumulative_deviation` (1.00), `cusum` (1.00) |
| `DeploymentRegressionSlew` | `illinois_socialnetwork` | 0x0340c203 | 0x055ff79f | Overlap | `cumulative_deviation` (1.00), `fpop` (1.00), `pelt` (1.00), `dp_cpd` (0.90), `page_hinkley` (0.80) |
| `EnvelopeBreach` | `aiops_challenge_2018_kpi` | 0x03040013 | 0x0557f79f | Overlap | `cusum` (1.00), `e_detector` (1.00), `interval_observer` (1.00), `mcd` (1.00), `spatial_sign` (1.00) |
| `EnvelopeBreach` | `multidim_localization_part1` | 0x03040013 | 0x015df417 | Overlap | `cumulative_deviation` (1.00), `cusum` (1.00), `depth_rank_control` (1.00), `dp_cpd` (1.00), `fpop` (1.00) |
| `CascadingTimeoutSlew` | `defects4j_6project` | 0x01403046 | 0x055ff79f | Overlap | `cumulative_deviation` (1.00), `e_detector` (1.00), `fpop` (1.00), `interval_observer` (1.00), `mcd` (1.00) |
| `ConfigDriftRegression` | `promise_defect_prediction` | 0x03020180 | 0x055ff79f | Overlap | `cumulative_deviation` (1.00), `dp_cpd` (0.86), `spatial_sign` (0.86), `pelt` (0.71), `mcusum` (0.57) |

## Summary by divergence

- **Overlap (partial)**: 9 motif(s)
