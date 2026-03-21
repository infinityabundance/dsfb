# DSFB Structural Semiotics Engine Artifact Report

Timestamp: `2026-03-21_12-47-42`
Crate: `dsfb-semiotics-engine` v`0.1.0`
Artifact schema: `dsfb-semiotics-engine/v1`
Run configuration hash: `1edd43706015f568`
Input mode: `csv`
Bank version: `heuristic-bank/v3`
Bank schema: `dsfb-semiotics-engine-bank/v1`
Bank source: `builtin`
Bank content hash: `db08579813856a89`
Strict bank validation: `true`
Bank validation mode: `strict`
Git commit: `874b8cd231ec88bc9ed0ba6510d9fa3097105863`
Rust: `rustc 1.93.0 (254b59607 2026-01-19)`

## Definitions Used

- Residual: `r(t) = y(t) - y_hat(t)`
- Drift: `d(t) = dr/dt` via deterministic finite differences, optionally after a configured low-latency smoothing pass used only for derivative estimation.
- Slew: `s(t) = d^2r/dt^2` via deterministic second differences over the same configured derivative-preconditioning path.
- Sign tuple: `sigma(t) = (r(t), d(t), s(t))`.
- Smoothing profile: `safety_first` with mode `safety_first`, alpha `0.18000`, causal window `5`, estimated lag `2.0000` samples, and maximum settling horizon `4` samples. Raw residual exports remain unchanged; only derivative estimation uses this optional preconditioning path.
- Smoothing guidance note: Safety-first smoothing prioritizes jitter attenuation over immediacy. Use the exported lag bound as an integration aid rather than as a closed-loop stability guarantee.
- Sign projection used in Figure 03: deterministic projected sign coordinates `[||r(t)||, dot(r(t), d(t))/||r(t)||, ||s(t)||]`, reported as residual norm, signed radial drift, and slew norm, with zero radial drift reported at exact zero residual norm.
- Syntax metrics include outward and inward drift fractions from residual-norm and margin evolution, radial-sign dominance, radial-sign persistence, drift-channel sign alignment, residual-norm path monotonicity, residual-norm trend alignment, mean squared slew norm, late slew-growth score, slew spike count and strength, boundary grazing episodes, boundary recovery count, and grouped aggregate breach fraction when coordinated structure is configured. Labels such as `weakly-structured-baseline-like` and `mixed-structured` remain conservative summaries rather than health judgments.
- Grammar: admissibility checked pointwise against `||r(t)|| <= rho(t)`. Each grammar sample also carries a deterministic trust scalar in `[0,1]` derived from typed grammar severity, with lower trust reserved for stronger or more abrupt admissibility departures.
- Semantics: constrained retrieval over a typed heuristic bank with scope conditions, admissibility requirements, regime tags, provenance notes, and compatibility rules. The bank may be builtin or external, but the loaded bank version, source, content hash, and validation result are exported explicitly for audit. Compatible sets carry explicit pairwise compatibility notes, while `Unknown` carries an explicit low-evidence or bank-noncoverage detail string. Larger banks may use a deterministic admissibility/regime/group-breach index to narrow candidates before the exact typed scope and compatibility checks run.
- Detectability bound: `t* - t0 <= Delta0 / (alpha - kappa)` when configured assumptions hold.
- Evaluation: post-run deterministic summaries and simple internal deterministic comparators (residual threshold, moving-average trend, slew spike, envelope interaction, one-sided CUSUM, and a fixed innovation-style squared residual statistic) are reported separately from the core engine outputs.
- Comparator framing: these internal deterministic comparators are operator-legible analogies to threshold monitors, EKF innovation monitoring, chi-squared-style gating, and one-sided change detectors on the same controlled scenario families. They are not field benchmarks and do not support superiority claims by themselves.

## Reproducibility Summary

- Scenario count checked: 1
- Identical materializations: 1
- All identical: `true`
- Note: Per-scenario reproducibility is evaluated over full materialized outputs rather than reduced norm summaries.

- `nasa_milling_public_demo`: identical=`true`, hash1=`e7eb7bd9c915fc5c`, hash2=`e7eb7bd9c915fc5c`

## Evaluation Summary

- Scenario count: 1
- Boundary-interaction scenarios: 1
- Violation scenarios: 1
- Comparator trigger counts: baseline_cusum=1, baseline_envelope_interaction=1, baseline_innovation_chi_squared_style=1, baseline_moving_average_trend=1, baseline_residual_threshold=1, baseline_slew_spike=1
- Minimum trust scalar: 0
- Bank validation mode: `strict`
- Bank validation strict symmetry errors: 0
- Bank validation violations: 0
- Bank validation warnings: 0
- Bank validation regime-tag notes: 0
- Bank validation priority notes: 0
- Semantic disposition counts: Match=1
- Smoothing comparison rows exported: 1
- Retrieval scaling rows exported: 3
- Figure integrity checks exported: 13

## Operator-Legible Comparator Case Study

This compact table stays within the crate's conservative framing. It compares internal deterministic comparators on the same controlled scenario family and shows where scalar alarm logic triggers while DSFB retains syntax, grammar, and constrained semantic distinctions.

| Scenario | Threshold | Moving Average | CUSUM | Innovation-Style | DSFB Syntax | DSFB Grammar | DSFB Semantics |
|----------|-----------|----------------|-------|------------------|-------------|--------------|----------------|
| `nasa_milling_public_demo` | alarm @ 14.0000 | alarm @ 3.0000 | alarm @ 3.0000 | alarm @ 14.0000 | `mixed-structured` | `Violation/AbruptSlewViolation` | `Match (H-CURVATURE-RICH-TRANSITION)` |

Case study note: `oscillatory_bounded` and `noisy_structured` can both remain admissible while simple scalar triggers stay silent or under-resolved, but DSFB preserves bounded-oscillatory versus structured-noisy syntax and semantic retrieval. `abrupt_event` and `curvature_onset` can both alarm under scalar comparators, while DSFB still separates discrete-event structure from curvature-led departure.

## Scenario Summary

### External CSV Scenario (nasa_milling_public_demo)

- Scenario ID: `nasa_milling_public_demo`
- Data origin: external-csv
- Purpose: Run externally supplied observed and predicted trajectories through the same deterministic structural semiotics pipeline used for the synthetic demonstrations, without adding hidden preprocessing.
- Alignment: This path preserves the layered residual/sign/syntax/grammar/semantics structure, but it does not attach theorem-aligned synthetic guarantees unless the input design justifies them separately.
- Claim class: external-csv ingestion
- Violations observed: 6
- First exit time: 14.0000
- Grammar state: `Violation`
- Grammar reason: `AbruptSlewViolation`
- Grammar reason text: Residual norm breached the configured envelope with an abrupt increase relative to the previous sample.
- Grammar supporting metrics: margin=-0.742270435731951, radius=0.3, residual_norm=1.042270435731951, norm_delta=0.6093739346912381, trust=0
- Trust scalar: `0`
- Syntax metrics: outward=0.65217, inward=0.34783, residual_norm_path_monotonicity=0.36718, residual_norm_trend_alignment=0.50000, radial_sign_persistence=0.80952, radial_sign_dominance=0.86364, drift_channel_sign_alignment=0.56522, mean_squared_slew_norm=2.012e-6, late_slew_growth_score=0.28532, slew_spikes=1, spike_strength=0.04214, grazing_episodes=2, boundary_recoveries=4, coordinated_group_breach_fraction=0
- Syntax label: `mixed-structured`
- Syntax note: This syntax label is conservative non-commitment at the syntax layer: the exported deterministic metrics did not support a narrower syntax summary under the current rule set. A separate semantic match may still be returned when admissibility, regime, and typed-bank constraints justify one.
- Semantic disposition: `Match`
- Semantic retrieval audit: path=linear-fallback, bank_entries=15, prefilter_candidates=15, post_admissibility=7, post_regime=4, pre_scope=4, post_scope=1, rejected_by_admissibility=8, rejected_by_regime=3, rejected_by_scope=3, selected_final=1
- Selected heuristics: `H-CURVATURE-RICH-TRANSITION`
- Semantic resolution basis: Single qualified heuristic remained after admissibility, regime, and scope filtering.
- Semantic unknown reason class: n/a
- Semantic unknown reason detail: n/a
- Semantic compatibility note: Single heuristic bank entry (`H-CURVATURE-RICH-TRANSITION`) satisfied the constrained retrieval rules.
- Semantic compatibility reasons: none
- Semantic note: The returned motif remains an illustrative compatibility statement only. It is not a unique-cause diagnosis.
- Limitation note: Interpretation depends on the supplied predicted trajectory, the configured admissibility envelope, and the sampled times parsed from the CSV files or synthesized deterministically from --dt when no explicit time column is supplied.

- Candidate `H-CURVATURE-RICH-TRANSITION` (`curvature-rich transition candidate`): score=0.25856, regimes=fixed, regime_check=Regime check passed because available regimes `fixed` satisfied required tags `fixed|widening|regime_shifted` via `fixed`., admissibility=Admissibility check passed because this bank entry accepts any grammar state mix., scope=Scope check passed for syntax label `mixed-structured` because mean_squared_slew_norm=2.012e-6 >= 4.000e-9, late_slew_growth_score=0.28532 >= 0.15000, drift_channel_sign_alignment=0.56522 >= 0.30000, slew_spike_count=1 >= 1, slew_spike_strength=0.04214 >= 0.01000., metric_highlights=mean_squared_slew_norm=2.012e-6 | late_slew_growth_score=0.28532 | max_slew_norm=0.0049553, applicability=Use when slew-rich structure is material. This remains a motif statement, not a validated mechanism classifier., provenance=Illustrative mapping for residual trajectories whose interpretation is governed more by curvature than by monotone migration., rationale=Admissibility check passed because this bank entry accepts any grammar state mix. Regime check passed because available regimes `fixed` satisfied required tags `fixed|widening|regime_shifted` via `fixed`. Scope check passed for syntax label `mixed-structured` because mean_squared_slew_norm=2.012e-6 >= 4.000e-9, late_slew_growth_score=0.28532 >= 0.15000, drift_channel_sign_alignment=0.56522 >= 0.30000, slew_spike_count=1 >= 1, slew_spike_strength=0.04214 >= 0.01000. Use when slew-rich structure is material. This remains a motif statement, not a validated mechanism classifier.

## Figure Captions

- `figure_01_residual_prediction_observation_overview`: Residual, observation, and prediction overview for the gradual degradation case. Synthetic deterministic demonstration only.
- `figure_02_drift_and_slew_decomposition`: Residual norm, signed radial drift, and slew norm decomposition for a representative case. Synthetic deterministic demonstration only when the bundled scenario suite is used.
- `figure_03_sign_space_projection`: Projected sign trajectory using the deterministic coordinates [||r||, dot(r,d)/||r||, ||s||]. Synthetic deterministic demonstration only when the bundled scenario suite is used.
- `figure_04_syntax_comparison`: Syntax comparison between monotone drift and curvature-dominated trajectories. Synthetic deterministic demonstration only.
- `figure_05_envelope_exit_under_sustained_outward_drift`: Residual norm and admissibility envelope for the sustained outward-drift exit case. Synthetic theorem-aligned demonstration only.
- `figure_06_envelope_invariance_under_inward_drift`: Residual norm and admissibility envelope for the inward-compatible invariance case. Synthetic theorem-aligned demonstration only.
- `figure_07_exit_invariance_pair_common_envelope`: Exit-invariance pair under a common visualization envelope, contrasting outward drift with inward-compatible containment. Synthetic theorem-aligned demonstration only.
- `figure_08_residual_trajectory_separation`: Residual trajectory separation between magnitude-matched admissible and detectable cases. Synthetic theorem-aligned demonstration only.
- `figure_09_detectability_bound_comparison`: Run-specific detectability view. The exported figure preserves the paper-facing filename while using either multi-case bound-versus-observed timing summaries or single-run residual-versus-envelope context with windowed detectability ratios, depending on the executed run.
- `figure_10_deterministic_pipeline_flow`: Deterministic layered engine flow showing residual extraction, sign construction, syntax, grammar, and semantic retrieval as auditable maps.
- `figure_11_coordinated_group_semiotics`: Local versus aggregate envelopes for the grouped correlated case, supporting the grouped aggregate breach fraction used in the coordinated syntax and semantic summaries. Synthetic deterministic demonstration only.
- `figure_12_semantic_retrieval_heuristics_bank`: Run-specific constrained-retrieval process summary rendered from exported source rows. Panel 1 shows ranked post-regime candidate scores, panel 2 shows the deterministic filter funnel, and panel 3 shows stage-specific rejection counts. The figure remains within-run rather than cross-dataset.
- `figure_13_internal_baseline_comparators`: Run-specific internal deterministic comparator activity. The panels show first-trigger timing, onset ordering, and triggered-scenario counts within the executed run. These remain within-crate comparator views only, not field benchmarks.

## Figure Integrity Checks

- `figure_01_residual_prediction_observation_overview`: panels=2/2, rows=`69`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_01_residual_prediction_observation_overview_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_01_residual_prediction_observation_overview_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_01_residual_prediction_observation_overview.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_01_residual_prediction_observation_overview.svg`
- `figure_02_drift_and_slew_decomposition`: panels=3/3, rows=`69`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_02_drift_and_slew_decomposition_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_02_drift_and_slew_decomposition_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_02_drift_and_slew_decomposition.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_02_drift_and_slew_decomposition.svg`
- `figure_03_sign_space_projection`: panels=1/1, rows=`26`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_03_sign_space_projection_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_03_sign_space_projection_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_03_sign_space_projection.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_03_sign_space_projection.svg`
- `figure_04_syntax_comparison`: panels=1/1, rows=`46`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_04_syntax_comparison_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_04_syntax_comparison_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_04_syntax_comparison.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_04_syntax_comparison.svg`
- `figure_05_envelope_exit_under_sustained_outward_drift`: panels=1/1, rows=`47`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_05_envelope_exit_under_sustained_outward_drift_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_05_envelope_exit_under_sustained_outward_drift_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_05_envelope_exit_under_sustained_outward_drift.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_05_envelope_exit_under_sustained_outward_drift.svg`
- `figure_06_envelope_invariance_under_inward_drift`: panels=1/1, rows=`47`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_06_envelope_invariance_under_inward_drift_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_06_envelope_invariance_under_inward_drift_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_06_envelope_invariance_under_inward_drift.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_06_envelope_invariance_under_inward_drift.svg`
- `figure_07_exit_invariance_pair_common_envelope`: panels=1/1, rows=`69`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_07_exit_invariance_pair_common_envelope_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_07_exit_invariance_pair_common_envelope_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_07_exit_invariance_pair_common_envelope.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_07_exit_invariance_pair_common_envelope.svg`
- `figure_08_residual_trajectory_separation`: panels=1/1, rows=`69`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_08_residual_trajectory_separation_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_08_residual_trajectory_separation_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_08_residual_trajectory_separation.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_08_residual_trajectory_separation.svg`
- `figure_09_detectability_bound_comparison`: panels=2/2, rows=`54`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_09_detectability_bound_comparison_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_09_detectability_bound_comparison_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_09_detectability_bound_comparison.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_09_detectability_bound_comparison.svg`
- `figure_10_deterministic_pipeline_flow`: panels=1/1, rows=`13`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_10_deterministic_pipeline_flow_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_10_deterministic_pipeline_flow_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_10_deterministic_pipeline_flow.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_10_deterministic_pipeline_flow.svg`
- `figure_11_coordinated_group_semiotics`: panels=2/2, rows=`139`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_11_coordinated_group_semiotics_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_11_coordinated_group_semiotics_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_11_coordinated_group_semiotics.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_11_coordinated_group_semiotics.svg`
- `figure_12_semantic_retrieval_heuristics_bank`: panels=3/3, rows=`12`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_12_semantic_retrieval_heuristics_bank_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_12_semantic_retrieval_heuristics_bank_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_12_semantic_retrieval_heuristics_bank.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_12_semantic_retrieval_heuristics_bank.svg`
- `figure_13_internal_baseline_comparators`: panels=3/3, rows=`18`, nonempty_series=`true`, nonzero_values_present=`true`, count_like_panels_integerlike=`true`, png_present=`true`, svg_present=`true`, consistent_with_source=`true`
  source_csv=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/csv/figure_13_internal_baseline_comparators_source.csv`, source_json=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/json/figure_13_internal_baseline_comparators_source.json`, png=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_13_internal_baseline_comparators.png`, svg=`/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/figures/figure_13_internal_baseline_comparators.svg`

## Limitations and Non-Claims

- Synthetic scenarios in this run are deterministic constructions intended to illustrate theorem-aligned behavior and auditable pipeline structure. CSV-ingested runs reuse the same pipeline without adding external validation claims.
- CSV ingestion mode, when used, applies the same deterministic layers to user-supplied trajectories but does not add validation claims beyond the supplied inputs and configured envelope.
- Envelope exits demonstrate detectable departure from the configured admissibility grammar, not unique identification of latent physical cause.
- Heuristic semantic matches are constrained typed-bank retrieval outcomes only; they are allowed to remain explicit compatible sets, ambiguous, or unknown.
- Builtin-bank and external-bank runs may differ when the bank artifact version, content, or validation policy differs. The run metadata records which bank was used.
- The current crate is not `no_std` and is not packaged for direct embedded deployment. A future embedded-core extraction path is documented separately.
- No certification claim is made. The artifact is aligned with deterministic and auditable engineering evaluation logic only.

## Manifest Summary

- Run directory: `/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42`
- Manifest schema: `dsfb-semiotics-engine/v1`
- Manifest run configuration hash: `1edd43706015f568`
- Manifest bank source: `builtin`
- Manifest bank version: `heuristic-bank/v3`
- Manifest bank content hash: `db08579813856a89`
- Figure files: 26
- CSV files: 41
- JSON files: 35
- PDF report: `/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/report/dsfb_semiotics_engine_report.pdf`
- Zip archive: `/home/one/dsfb/crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/generated/2026-03-21_12-47-42/nasa_milling-dsfb-semiotics-engine-2026-03-21_12-47-42.zip`
- Artifact completeness: complete=`true`, markdown=`true`, pdf=`true`, zip=`true`, manifest=`true`
- PDF companion content: rendered markdown report, embedded figure artifacts, full artifact inventory, and appended text-based CSV/JSON/manifest/report sources.
