# Per-motif named-witness refinement (Phase ζ.8)

For each typed-confirmed episode in the LO-CV baseline pass,
the table reports: the motif's hand-curated
`primary_witness_detectors` (Phase 8 strict ensemble gate)
vs the empirically observed top-5 detectors firing within
the matched episode's window range.

**Refinement is RECOMMENDATION, not bank mutation.**
Phase ζ.9 separately gates any merge through LO-CV.

Source: Phase ζ.8 audit harness (parallel to Phase ζ.4
affinity refinement; same data, different view).

| Motif | Fixture | Curated witnesses | Observed top-5 |
|-------|---------|-------------------|----------------|
| `DeploymentRegressionSlew` | `tadbench_trainticket_F11` | `page_hinkley`, `pelt`, `pettitt_test` | `bayesian_offline_cpd` (1.00), `corr_matrix_distance` (1.00), `cumulative_deviation` (1.00), `dp_cpd` (1.00), `fpop` (1.00) |
| `AuthenticationFailureSpike` | `tadbench_trainticket_F11` | `poisson_burst`, `burst_after_silence`, `flap` | `anderson_darling` (1.00), `ar2_residual` (1.00), `arima_simplified` (1.00), `batschelet_concentration` (1.00), `bayesian_blocks` (1.00) |
| `AuthenticationFailureSpike` | `tadbench_trainticket_F11` | `poisson_burst`, `burst_after_silence`, `flap` | `arima_simplified` (1.00), `bayesian_blocks` (1.00), `bayesian_offline_cpd` (1.00), `burst_after_silence` (1.00), `canonical_correlation` (1.00) |
| `EnvelopeBreach` | `tadbench_trainticket_F11b` | `scalar_threshold_3sigma`, `cusum`, `page_hinkley` | `arima_simplified` (1.00), `bayesian_blocks` (1.00), `bond_graph_residual` (1.00), `cumulative_deviation` (1.00), `cusum` (1.00) |
| `DeploymentRegressionSlew` | `illinois_socialnetwork` | `page_hinkley`, `pelt`, `pettitt_test` | `cumulative_deviation` (1.00), `fpop` (1.00), `pelt` (1.00), `dp_cpd` (0.90), `page_hinkley` (0.80) |
| `EnvelopeBreach` | `aiops_challenge_2018_kpi` | `scalar_threshold_3sigma`, `cusum`, `page_hinkley` | `cusum` (1.00), `e_detector` (1.00), `interval_observer` (1.00), `mcd` (1.00), `spatial_sign` (1.00) |
| `EnvelopeBreach` | `multidim_localization_part1` | `scalar_threshold_3sigma`, `cusum`, `page_hinkley` | `cumulative_deviation` (1.00), `cusum` (1.00), `depth_rank_control` (1.00), `dp_cpd` (1.00), `fpop` (1.00) |
| `CascadingTimeoutSlew` | `defects4j_6project` | `correlation_break`, `lof`, `causal_lag` | `cumulative_deviation` (1.00), `e_detector` (1.00), `fpop` (1.00), `interval_observer` (1.00), `mcd` (1.00) |
| `ConfigDriftRegression` | `promise_defect_prediction` | `wasserstein_1d`, `ddm`, `kl_divergence` | `cumulative_deviation` (1.00), `dp_cpd` (0.86), `spatial_sign` (0.86), `pelt` (0.71), `mcusum` (0.57) |

## Witness coverage summary

- **Curated witness present in observed top-5**: 5 entries
- **Curated witness NOT in observed top-5**: 4 entries
- **No witness curation declared**: 0 entries
