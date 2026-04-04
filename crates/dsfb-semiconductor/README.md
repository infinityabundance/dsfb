# dsfb-semiconductor

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb)

`dsfb-semiconductor` is the empirical software companion for the DSFB semiconductor paper. It instantiates the paper's bounded claim: DSFB is a deterministic, non-intrusive augmentation layer over existing semiconductor monitoring signals, not a replacement for incumbent SPC/APC/FDC infrastructure.

The operator-facing result in the saved SECOM row is bounded and concrete:

- `28,607 -> 71` structured review episodes (`-99.75%`)
- `0.36% -> 80.3%` event relevance (`220.8x`)
- `10,554 -> 3,854` investigation-worthy decisions (`-63.5%`)
- `104 / 104` failure-labeled runs preserved

The crate is intentionally read-only and side-channel by design:

- SPC, EWMA, threshold logic, APC, and controller behavior remain authoritative and unchanged
- DSFB consumes immutable residual streams, alarm streams, and metadata only
- DSFB emits advisory interpretations only; there is no control-output or threshold-write API
- identical inputs replay to identical outputs
- if DSFB fails, upstream plant behavior is unchanged

The current crate turns real semiconductor datasets into inspectable DSFB artifacts:

- nominal reference summaries
- residual traces
- drift traces
- slew traces
- admissibility-envelope / grammar-state traces
- DSA structural traces, scores, consistency flags, and policy-governed feature states
- provenance-aware heuristics-bank entries with active alert-governance fields
- explicit DSFB feature-scaffold sign / syntax / grammar / semantics / policy traces for `S059`, `S133`, `S123`, `S540`, `S128`, and `S104`, plus grouped semiotics for `{S059,S133}`, `{S123,S540,S128}`, and `{S104}`
- lead-time, sliding-window density, and pass-run nuisance proxy metrics
- calibration grid artifacts over the fixed DSFB parameter surface
- a bounded DSA calibration grid over fixed deterministic threshold and persistence settings
- a deterministic residual stateflow chart (DRSC) plus aligned trace CSV for the top boundary-activity feature
- a publication-quality Deterministic Residual Stateflow Chart with Structural Accumulation (DRSC+DSA) plus aligned trace CSV for that same selected feature window
- all notebook-parity PNG figures directly from the crate
- a generated `dsfb_traceability.json` artifact exposing the chain `Residual -> Sign -> Motif -> Grammar -> Semantic -> Policy`
- an engineering report in Markdown, LaTeX, and PDF when `pdflatex` is available; the PDF includes the generated figures and an artifact inventory
- a ZIP bundle of the full run directory

Operator-facing docs bundled in the crate:

- `docs/sbir_one_page.md`
- `docs/fab_data_integration.md`
- `docs/compute_cost.md`
- `docs/failure_modes.md`

## What math from the paper is instantiated

This crate implements the paper's core operator-facing objects with explicit saved parameters:

- nominal reference from an initial healthy passing window
- residuals `r(k) = x(k) - x_hat(k)`
- drift from a finite window difference on residual norms
- slew as the first difference of drift
- admissibility envelope radius `rho = sigma_multiplier * healthy_std`
- grammar states `Admissible`, `Boundary`, and `Violation`
- hysteretic state confirmation together with persistent boundary / violation traces
- a separate deterministic DSA feature layer built from rolling raw-boundary density, outward drift persistence, slew density, normalized EWMA occupancy, motif recurrence, and a directional-consistency gate
- a heuristics-governed DSA policy engine that maps active motifs into feature-level `Silent`, `Watch`, `Review`, and `Escalate` states
- explicit semantics of silence: transient, fragmented, or unsupported structure can remain `Silent` even when the numeric structural score is nonzero
- policy-gated feature-level DSA alerts that require both the numeric persistence condition `dsa_score >= tau` for at least `K` runs and the motif-policy reduction to `Review` or `Escalate`
- run-level DSA aggregation as cross-feature corroboration `feature_count_review_or_escalate(k) >= m`, together with emitted watch/review/escalate feature counts and a stricter `strict_escalate_run_alert(k)` trace
- a provenance-aware heuristics bank built from observed grammar motifs, severity tags, action notes, action-governance fields, and limitations
- five explicit deterministic comparators: a raw residual-magnitude threshold, a univariate EWMA residual-norm comparator, a positive CUSUM residual-norm comparator, a run-level residual-energy comparator, and a PCA T2/SPE multivariate FDC comparator
- per-failure-run earliest-signal tracking and lead-time deltas against the scalar comparators for both the DSFB state logic and the DSA layer
- sliding-window density summaries for boundary / violation / threshold / EWMA occupancy
- pass-run nuisance proxies derived from SECOM pass labels
- a deterministic SECOM calibration workflow over explicit parameter grids
- a bounded DSA calibration workflow over `W`, `K`, `tau`, and corroboration count `m`
- a deterministic feature-ranking step plus explicit DSA cohorts over `top_4`, `top_8`, `top_16`, and `all_features`
- a grammar-conditioned feature-scaffold semiotics pass over selected SECOM channels that emits `dsfb_feature_signs.csv`, `dsfb_feature_motif_timeline.csv`, `dsfb_feature_grammar_states.csv`, `dsfb_semantic_matches.csv`, `dsfb_feature_policy_decisions.csv`, and grouped semiotics artifacts
- a deterministic residual stateflow chart (DRSC) that synchronizes residual/drift/slew structure, confirmation-filtered grammar state, a DSA overlay band, and the run-level comparator overlay for one emitted feature trace without redefining DSFB state semantics
- a publication-oriented DRSC+DSA figure that keeps the same selected feature but simplifies the view into four aligned grayscale panels: normalized residual / drift / slew, persistent deterministic DSFB state, DSA activation, and run-level threshold / EWMA trigger timing
- a separate DSA structural-focus figure that exposes rolling DSA inputs, DSA score, persistence gating, and feature-level comparator bands for that same emitted feature trace

The implementation is intentionally simple and deterministic. It is designed for auditability and reproducibility, not for inflated benchmark claims.

## Supported datasets

### SECOM

Implemented and verified as the first real-data benchmark path.

- Source: UCI Machine Learning Repository SECOM dataset
- Access mode: automated download via `fetch-secom`, or manual placement of `secom.zip`
- Role in this crate: real-data benchmark for residual structure, drift, slew, grammar, and motif extraction
- Non-claim: SECOM is anonymized and instance-level; it does not by itself validate chamber-mechanism attribution or full run-to-failure prognostics

Archive-layout note:

- The current distributed `secom.data` parses as `590` numeric columns.
- The UCI metadata text in `secom.names` states `591` attributes.
- This crate uses the `590` numeric columns actually present in `secom.data` and reads pass/fail labels plus timestamps separately from `secom_labels.data`.
- The exact resolved note is emitted to `secom_archive_layout.json` in every run bundle.

Expected raw-data location after fetch or manual placement:

```text
crates/dsfb-semiconductor/data/raw/secom/
  secom.zip
  secom.data
  secom_labels.data
  secom.names
```

### PHM 2018 ion mill etch

Implemented as a parallel, bounded degradation-oriented benchmark path.

- The crate loads continuous degradation trajectories from the extracted PHM 2018 dataset already staged under `crates/dsfb-semiconductor/data/`
- The PHM path runs the same DSFB stack shape used in the crate: residual -> sign -> syntax -> grammar -> semantics -> policy
- The comparison baseline is explicit and narrow: `run_energy_scalar_threshold`
- The saved claim is bounded: PHM 2018 is used for degradation-oriented structure-emergence and timing analysis, not for burden-compression claims
- The current saved PHM run at `crates/dsfb-semiconductor/output-dsfb-semiconductor/20260402_192526_564_dsfb-semiconductor_phm2018/` reports `14` comparable runs, mean lead delta `+58,572.43`, median lead delta `+26,746.0`, and a mixed split of `45% earlier / 5% equal / 50% later`

Expected manual archive path:

```text
crates/dsfb-semiconductor/data/raw/phm2018/phm_data_challenge_2018.tar.gz
```

Probe command:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- probe-phm2018
```

## Exact run instructions

Fetch SECOM into the crate-local raw-data directory:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- fetch-secom
```

Run the full SECOM benchmark with default parameters:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- run-secom --fetch-if-missing
```

Run the deterministic SECOM calibration grid:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- calibrate-secom \
  --healthy-pass-runs-grid 80,100,120 \
  --drift-window-grid 3,5 \
  --boundary-fraction-of-rho-grid 0.4,0.5 \
  --state-confirmation-steps-grid 1,2 \
  --persistent-state-steps-grid 1,2 \
  --density-window-grid 10 \
  --pre-failure-lookback-runs-grid 10,20
```

Run the bounded deterministic DSA calibration grid:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- calibrate-secom-dsa --fetch-if-missing
```

The bounded DSA grid is fixed at:

- `W ∈ {5, 10, 15}`
- `K ∈ {2, 3, 4}`
- `tau ∈ {2.0, 2.5, 3.0}`
- `m ∈ {2, 3, 5}`

The current crate default for `run-secom` is the bounded-grid best-recall point:

- `W = 5`
- `K = 2`
- `tau = 2.0`

Key configurable parameters:

- `--healthy-pass-runs`
- `--drift-window`
- `--envelope-sigma`
- `--boundary-fraction-of-rho`
- `--ewma-alpha`
- `--ewma-sigma-multiplier`
- `--drift-sigma-multiplier`
- `--slew-sigma-multiplier`
- `--grazing-window`
- `--grazing-min-hits`
- `--pre-failure-lookback-runs`
- `--state-confirmation-steps`
- `--persistent-state-steps`
- `--density-window`
- `--dsa-window`
- `--dsa-persistence-runs`
- `--dsa-alert-tau`
- `--dsa-corroborating-feature-count-min`
- `--cusum-kappa-sigma-multiplier`
- `--cusum-alarm-sigma-multiplier`
- `--run-energy-sigma-multiplier`

Calibration-grid arguments:

- `--healthy-pass-runs-grid`
- `--drift-window-grid`
- `--envelope-sigma-grid`
- `--boundary-fraction-of-rho-grid`
- `--ewma-alpha-grid`
- `--ewma-sigma-multiplier-grid`
- `--cusum-kappa-sigma-multiplier-grid`
- `--cusum-alarm-sigma-multiplier-grid`
- `--run-energy-sigma-multiplier-grid`
- `--drift-sigma-multiplier-grid`
- `--slew-sigma-multiplier-grid`
- `--grazing-window-grid`
- `--grazing-min-hits-grid`
- `--pre-failure-lookback-runs-grid`
- `--state-confirmation-steps-grid`
- `--persistent-state-steps-grid`
- `--density-window-grid`
- `--dsa-window-grid`
- `--dsa-persistence-runs-grid`
- `--dsa-alert-tau-grid`
- `--dsa-corroborating-feature-count-min-grid`

Current implemented baselines:

- residual-threshold baseline: `|r(k)| > rho`
- EWMA baseline: univariate EWMA on residual norms with explicit `alpha` and healthy-window thresholding
- CUSUM baseline: positive residual-norm CUSUM with fixed healthy-window `kappa` and alarm-threshold multipliers
- run-energy baseline: mean squared residual z-energy across analyzable features with a fixed healthy-window threshold
- PCA T2/SPE baseline: deterministic multivariate FDC comparator with a healthy-window PCA fit and fixed T2/SPE sigma thresholds

DSFB state-layer distinction:

- `DSFB Violation`: hard envelope exit `|r(k)| > rho`
- `DSA`: persistence-constrained structural accumulation from rolling structural features

Current baseline classes not implemented:

- lightweight ML anomaly baselines

## Output structure

All benchmark runs write to a repo-level timestamped directory and do not reuse an existing run folder:

```text
output-dsfb-semiconductor/<timestamp>_dsfb-semiconductor_<dataset>/
```

The current SECOM pipeline writes:

```text
artifact_manifest.json
baseline_comparison_summary.json
benchmark_metrics.json
dataset_summary.json
density_metrics.csv
drsc_top_feature.csv
drsc_dsa_combined.csv
dsa_top_feature.csv
drifts.csv
cusum_baseline.csv
run_energy_baseline.csv
ewma_baseline.csv
engineering_report.md
engineering_report.tex
engineering_report.pdf          # when pdflatex is available; includes figures and artifact inventory
feature_metrics.csv
figures/
grammar_states.csv
heuristics_bank.json
lead_time_metrics.csv
parameter_manifest.json
dsa_parameter_manifest.json
dsa_feature_ranking.csv
dsa_feature_ranking_recall_aware.csv
dsa_feature_ranking_dsfb_aware.csv
dsa_feature_ranking_burden_aware.csv
dsa_feature_ranking_comparison.csv
dsa_seed_feature_check.json
dsa_feature_cohorts.json
dsa_feature_policy_overrides.json
dsa_feature_policy_summary.csv
dsa_recall_rescue_results.csv
dsa_recall_critical_features.csv
dsa_recall_recovery_efficiency.csv
dsfb_single_change_iteration_log.csv
optimization_log.json
dsa_operator_baselines.json
dsa_operator_delta_targets.json
dsa_operator_delta_attainment_matrix.csv
dsa_policy_operator_burden_contributions.csv
dsa_feature_policy_summary.csv
dsa_feature_policy_overrides.json
failures_index.json
missed_failure_priority.csv
feature_to_motif.json
negative_control_report.json
dsfb_feature_role_validation.csv
dsfb_group_validation.csv
dsfb_heuristic_provenance.csv
policy_decisions.csv
policy_burden_summary.csv
episode_precision_metrics.json
recurrent_boundary_stats.json
recurrent_boundary_tradeoff_curve.csv
recurrent_boundary_tradeoff_plot.png
lead_time_comparison.csv
lead_time_explanation.json
missed_failure_root_cause.json
target_d_regression_analysis.json
dsfb_structural_delta_metrics.json
non_intrusive_interface_spec.md
dsa_delta_target_assessment.json
dsa_pareto_frontier.csv
dsa_stage_a_candidates.csv
dsa_stage_b_candidates.csv
dsa_missed_failure_diagnostics.csv
dsa_cohort_results.csv
dsa_cohort_results_recall_aware.csv
dsa_cohort_results_dsfb_aware.csv
dsa_cohort_results_burden_aware.csv
dsa_cohort_summary.json
dsa_cohort_summary_recall_aware.json
dsa_cohort_summary_dsfb_aware.json
dsa_cohort_summary_burden_aware.json
dsa_cohort_precursor_quality.csv
dsa_cohort_failure_analysis.md   # emitted when no cohort satisfies the primary success condition
dsa_heuristic_policy_failure_analysis.md
dsa_motif_policy_contributions.csv
dsa_policy_contribution_analysis.csv
dsa_rating_delta_forecast.json
dsa_rating_delta_failure_analysis.md  # emitted when the rating-delta primary success condition is not met
dsa_grid_results.csv
dsa_grid_summary.json
per_failure_run_signals.csv
per_failure_run_dsa_signals.csv
phm2018_support_status.json
dsa_metrics.csv
dsa_run_signals.csv
dsa_vs_baselines.json
dsfb_group_definitions.json
pca_fdc_baseline.csv
residuals.csv
run_bundle.zip
run_configuration.json
secom_archive_layout.json
slews.csv
```

Notable generated figure artifacts now include:

- `figures/dsfb_unified_value_figure.png`
- `figures/dsfb_non_intrusive_architecture.png`
- `figures/dsfb_non_intrusive_architecture.svg`

The PHM 2018 benchmark writes a parallel run bundle with:

```text
artifact_manifest.json
claim_alignment_report.json
phm2018_early_warning_stats.json
phm2018_lead_time_metrics.csv
phm2018_run_details.json
phm2018_structural_metrics.json
phm2018_support_status.json
run_bundle.zip
```

## Current heuristics-governed DSA result

The latest audited SECOM run at `crates/dsfb-semiconductor/output-dsfb-semiconductor/20260402_183530_113_dsfb-semiconductor_secom/` keeps the empirical claim narrow and operator-focused.

- Ranking formula: `candidate_score = z(dsfb_raw_boundary_points) - z(dsfb_raw_violation_points) + z(ewma_alarm_points) - I(missing_fraction > 0.50) * 2.0`
- Recall-aware ranking formula: `candidate_score_recall = z(pre_failure_run_hits) + z(motif_precision_proxy) + z(ewma_alarm_points) + 0.5 * z(dsfb_raw_boundary_points) + 0.5 * z(recall_rescue_contribution) - 0.5 * z(dsfb_raw_violation_points) - I(missing_fraction > 0.50) * 2.0`
- DSFB-aware ranking formula: `candidate_score_dsfb = z(pre_failure_run_hits) + z(motif_precision_proxy) + 0.5 * z(recall_rescue_contribution) + 0.5 * z(semantic_persistence_contribution) + 0.5 * z(grouped_semantic_support) + 0.25 * z(dsfb_raw_boundary_points) - z(operator_burden_contribution) - 0.5 * z(violation_overdominance_penalty) - I(missing_fraction > 0.50) * 2.0`
- Seed-feature check: `S059` ranked 1 and `S044` ranked 6; `S061`, `S222`, `S354`, and `S173` ranked 19, 31, 49, and 88 respectively, so only `S059` reached `top_4` and only `S059` plus `S044` reached `top_8`
- Full bounded cohort grid evaluated: `405` saved rows across `top_4`, `top_8`, `top_16`, and `all_features` with `W in {5,10,15}`, `K in {2,3,4}`, `tau in {2.0,2.5,3.0}`, and `m in {1,2,3,5}` where valid
- The heuristics bank is now active policy, not passive reporting: `pre_failure_slow_drift` defaults to `Review`, `recurrent_boundary_approach` to `Watch`, and `transient_excursion` to `Silent`, with deterministic persistence, corroboration, and fragmentation gates
- The operator baselines are explicit and saved: numeric-only DSA provides the investigation-point baseline (`10554` Review/Escalate points), raw boundary provides the episode baseline (`28607` episodes), and the prior policy-governed DSA row provides the pass-run review-burden baseline
- The current selected configuration preserves full recall at `104/104` while reducing investigation-worthy burden to `3854` Review/Escalate points and compressing episodes to `71`
- The strongest SECOM deltas are now explicit and baseline-named: `63.5%` investigation-load reduction versus numeric-only DSA, `99.8%` episode reduction versus raw boundary episodes, and `80.3%` episode precision versus a `0.36%` raw boundary precision proxy (`220.8x`)
- Honest nuisance reporting is also explicit: nuisance reduction versus EWMA is only `19.3%`, so the crate does not claim a generic `40%+` nuisance delta on SECOM
- The structural limitation is now surfaced directly: `recurrent_boundary_approach` carries both useful precursor signal and dominant pass-run nuisance, so aggressive suppression trades away recall
- All four ranking strategies converge to the same selected all-feature row; the best ranked cohort remains much lower-nuisance but materially lower-recall than the selected all-feature configuration
- Feature-aware bounded rescue remains explicit and saved; in the latest run it recovers all four baseline-missed failures, including failure run `2` through low-burden semantic support on `S092`
- Semantics of silence remain measurable on the selected row: `4093` feature points are suppressed to `Silent`, leaving `187` watch points, `3560` review points, and `819` escalate points
- The new DSFB scaffold is additive rather than rhetorical: `S059` is supported as a recurrent-boundary precursor, `S123` is partially supported as a transition / instability feature, `S540` and `S128` stay corroborative, `S104` behaves as a watch-only sentinel, and `S133` remains semantically ambiguous rather than a clean slow-drift precursor
- Grouped semiotics is now explicitly audited in `dsfb_group_definitions.json`; the scaffold groups remain corroborative structure rather than root-cause claims
- Motif contribution is still not generic: the operator-facing value comes more from investigation-load reduction and episode compression than from run-level nuisance alone, and the strongest remaining blocker is failure coverage rather than burden
- The best reachable Pareto point under the latest saved deterministic sweep is still bounded: DSA materially lowers investigation-worthy burden and compresses episodes while now reaching equal-threshold coverage, but it still trails threshold and EWMA on mean lead and it does not preserve precursor quality or pass-run review-burden reductions

The current figure set includes:

- `figures/missingness_top20.png`
- `figures/drsc_top_feature.png`
- `figures/drsc_dsa_combined.png`
- `figures/dsa_top_feature.png`
- `figures/benchmark_comparison.png`
- `figures/grammar_timeline.png`
- `figures/top_feature_residual_norms.png`
- `figures/top_feature_drift.png`
- `figures/top_feature_ewma.png`
- `figures/top_feature_slew.png`
- `figures/dsfb_unified_value_figure.png`
- `figures/dsfb_non_intrusive_architecture.png`

The DRSC figure is an operator-facing synchronized chart for the top boundary-activity feature in the run. Its layers are:

- normalized residual / drift / slew structure
- confirmation-filtered persistent grammar state band
- feature-level DSA score with persistence-constrained alert shading
- normalized admissibility-envelope occupancy together with normalized EWMA occupancy and normalized run-energy occupancy

The emitted DRSC also annotates the first persistent boundary, the first persistent violation when present in the selected window, and the failure-labeled run. This crate does not currently implement a trust scalar, so the DRSC lower layer is an admissibility overlay rather than a trust plot.

The emitted DRSC+DSA figure is the publication-oriented version of that same selected feature window. It keeps the data deterministic and grayscale-safe while reducing the view to four aligned panels:

- normalized residual / drift / slew using fixed threshold-normalized formulas `residual / rho`, `drift / drift_threshold`, and `slew / slew_threshold`
- the actual persistent DSFB state band with the display aliases `Admissible`, `Boundary`, and `Violation`
- a binary DSA layer rendered as feature-level DSA alert plus corroborated run-level DSA alert
- run-level threshold and EWMA any-feature trigger timing

This figure is intended to make the current crate value legible in one glance. It does not by itself claim earlier precursor timing than scalar baselines unless that is actually visible in the saved run bundle.

The emitted DSA structural-focus figure is separate from DRSC on purpose. It expands the structural inputs for the same selected feature window while showing:

- rolling boundary density, drift persistence, slew density, normalized EWMA occupancy, and motif recurrence
- DSA score with the fixed `tau` line and persistence-gated alert shading
- feature-level DSA / boundary / violation / threshold / EWMA / CUSUM alert bands plus the run-level residual-energy and PCA T2/SPE alarm bands in one aligned view

The crate emits these PNG figures directly; the notebook simply renders the contents of `run_dir/figures/*.png` inline. No notebook-only plotting logic is required to obtain them.

The calibration pipeline writes:

```text
output-dsfb-semiconductor/<timestamp>_dsfb-semiconductor_secom_calibration/
  calibration_best_by_metric.json
  calibration_grid_results.csv
  calibration_report.md
  calibration_run_configuration.json
  parameter_grid_manifest.json
```

The bounded DSA calibration pipeline writes:

```text
output-dsfb-semiconductor/<timestamp>_dsfb-semiconductor_secom_dsa_calibration/
  dsa_grid_results.csv
  dsa_grid_summary.json
  dsa_calibration_report.md
  dsa_calibration_run_configuration.json
  dsa_parameter_grid_manifest.json
```

## Reproducibility discipline

- Every meaningful threshold and window is saved to `parameter_manifest.json`
- Fixed DSA weights, the corroborated run-level aggregation choice, the consistency rule, the primary-success recall tolerance, and the optimization-priority order are saved to `dsa_parameter_manifest.json`
- Dataset source and output root are saved to `run_configuration.json`
- Calibration grids are saved verbatim to `parameter_grid_manifest.json`
- The bounded DSA calibration grid is saved verbatim to `dsa_parameter_grid_manifest.json`
- Missing values are preserved at load time and then deterministically imputed with the healthy-window nominal mean before residual construction
- Repeated runs with the same inputs and parameters produce the same metrics, traces, and calibration rows, modulo different timestamped output directories

## Current empirical boundary

The crate establishes deterministic structural artifact generation on real semiconductor data, not a blanket superiority claim over scalar baselines.

- `DSFB Violation` and feature-level `DSA` are intentionally different signals: violation is a hard envelope exit, while feature-level DSA is a persistence-constrained structural accumulator.
- The primary run-level DSA comparison signal is fixed cross-feature corroboration: `feature_count_review_or_escalate(k) >= m`.
- The authoritative comparison artifact for the DSA layer is `dsa_vs_baselines.json`.
- The legacy one-run nuisance/recall sweep gate saved in `dsa_parameter_manifest.json` is: DSA pass-run nuisance below EWMA nuisance and DSA failure recall within `1` run of threshold recall. The stronger operator-relevant target is now logged separately in `dsa_delta_target_assessment.json`.
- The current selected SECOM row under `output-dsfb-semiconductor/20260402_183530_113_dsfb-semiconductor_secom/` reports DSA recall `104/104`, investigation-load reduction `0.6348`, episode reduction `0.9975`, pass-run nuisance reduction versus EWMA `0.1933`, mean lead `17.97`, and episode precision `0.8028`.
- On that saved run, the heuristics-governed policy layer strongly reduces operator-facing investigation burden (`10554 -> 3854`) and compresses raw structural episodes (`28607 -> 71`) while keeping threshold-level recall. The operator burden success story is therefore real, but SECOM still does not support a strong early-warning claim or a `40%+` nuisance claim versus EWMA.
- The current bounded cohort DSA grid under `output-dsfb-semiconductor/20260402_062248_909_dsfb-semiconductor_secom/` contains `405` saved rows over cohort, `W`, `K`, `tau`, and `m` for each ranking strategy. The compression-biased, recall-aware, burden-aware, and DSFB-aware rankings all converge to the same `all_features (W=10, K=4, tau=2.0, m=1)` configuration, while narrower cohorts still trade away too much recall for burden reduction.
- The lead-time, density, and nuisance values remain proxy metrics on SECOM labels, not fab-qualified false-alarm or economic metrics.
- The DRSC and DSA figures are deterministic and replayable from saved traces, but they are operator-facing visualizations of current rule-based state evolution, not probabilistic explanation layers.

## no_std kernel

The DSFB computation kernel compiles without the standard library and
requires only `alloc`. This enables deployment on Cortex-M microcontrollers,
FPGAs, and RTOS targets where no OS is available.

**Kernel modules available without std:**
`process_context`, `units`, `signs`, `sign`, `grammar`, `grammar::layer`,
`syntax`, `policy`, `semantics`, `config`, `nominal`, `residual`, `input`

**Verified in CI** against `thumbv7em-none-eabi` (Cortex-M4/M7, no OS):

```sh
cargo check --lib --no-default-features --target thumbv7em-none-eabi
```

The I/O layer (CLI, pipeline, plotting, dataset adapters, SECOM evaluation)
requires `std` and is compiled with the default feature set.

## Caveats and non-claims

- This crate does not claim SEMI standards compliance or completed qualification.
- This crate does not claim universal superiority over SPC, EWMA/CUSUM, multivariate FDC, or ML baselines.
- The current comparator set is still bounded: a univariate residual-magnitude threshold, univariate EWMA and positive CUSUM residual-norm comparators, a run-level residual-energy comparator, and a deterministic PCA T2/SPE multivariate FDC comparator.
- The current nuisance analysis is a pass-run proxy on SECOM labels, not a fab-qualified false-alarm-rate study.
- The current lead-time analysis is bounded to fixed lookback windows on the available labels.
- SECOM is real semiconductor data, but it is not a deployment validation dataset.
- PHM 2018 claims are intentionally bounded to degradation-oriented timing and structure-emergence analysis against the saved `run_energy_scalar_threshold` baseline; this crate does not claim PHM burden reduction or universal DSFB timing superiority.
- This crate does not claim Kani verification for `dsfb-semiconductor`.
- The computation kernel (`sign`, `grammar`, `syntax`, `semantics`, `policy`, `process_context`, `units`) supports `no_std` with `alloc`, verified in CI against `thumbv7em-none-eabi`. No claim of `no_alloc`, SIMD, rayon, or parallel-acceleration support. The I/O layer (CLI, pipeline, dataset adapters) requires `std`.
- This crate does not claim SEMI E125 compatibility.
- PDF generation depends on `pdflatex` being installed in the runtime.
- The notebook file is wired to the current CLI and output paths, but this README does not claim that a live Colab execution was performed in this environment.

## Notebook

The Colab notebook lives at:

[`crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb`](/home/one/dsfb/crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb)

It is wired to:

- bootstrap a Rust environment in Colab
- fetch or reuse the real SECOM dataset
- run the crate end to end
- inspect archive-layout, lead-time, nuisance, and PHM-support summary artifacts
- display the generated figures inline
- optionally run the bounded calibration grid
- surface the PDF report and ZIP bundle for download
