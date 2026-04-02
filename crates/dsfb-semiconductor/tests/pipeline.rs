use dsfb_semiconductor::baselines::{compute_baselines, ewma_series};
use dsfb_semiconductor::calibration::{
    run_secom_calibration, run_secom_dsa_calibration, CalibrationGrid,
};
use dsfb_semiconductor::config::PipelineConfig;
use dsfb_semiconductor::dataset::secom;
use dsfb_semiconductor::grammar::evaluate_grammar;
use dsfb_semiconductor::nominal::build_nominal_model;
use dsfb_semiconductor::phm2018_loader::run_phm2018_benchmark;
use dsfb_semiconductor::pipeline::run_secom_benchmark;
use dsfb_semiconductor::precursor::evaluate_dsa;
use dsfb_semiconductor::preprocessing::prepare_secom;
use dsfb_semiconductor::residual::compute_residuals;
use dsfb_semiconductor::signs::compute_signs;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipArchive;

fn write_fixture_dataset(root: &Path) -> PathBuf {
    let secom_root = root.join("secom");
    fs::create_dir_all(&secom_root).unwrap();
    fs::write(
        secom_root.join("secom.data"),
        "\
1.0 2.0 3.0\n\
1.0 2.2 3.0\n\
1.1 2.1 3.0\n\
1.2 NaN 3.0\n\
2.0 4.0 3.0\n\
2.5 4.5 3.0\n",
    )
    .unwrap();
    fs::write(
        secom_root.join("secom_labels.data"),
        "\
-1 \"01/01/2008 00:00:00\"\n\
-1 \"01/01/2008 00:10:00\"\n\
-1 \"01/01/2008 00:20:00\"\n\
-1 \"01/01/2008 00:30:00\"\n\
1 \"01/01/2008 00:40:00\"\n\
1 \"01/01/2008 00:50:00\"\n",
    )
    .unwrap();
    fs::write(secom_root.join("secom.names"), "fixture names\n").unwrap();
    root.to_path_buf()
}

fn test_config() -> PipelineConfig {
    PipelineConfig {
        healthy_pass_runs: 3,
        drift_window: 2,
        envelope_sigma: 2.0,
        boundary_fraction_of_rho: 0.5,
        state_confirmation_steps: 2,
        persistent_state_steps: 2,
        density_window: 3,
        ewma_alpha: 0.3,
        ewma_sigma_multiplier: 2.0,
        cusum_kappa_sigma_multiplier: 0.5,
        cusum_alarm_sigma_multiplier: 4.0,
        run_energy_sigma_multiplier: 3.0,
        pca_variance_explained: 0.95,
        pca_t2_sigma_multiplier: 3.0,
        pca_spe_sigma_multiplier: 3.0,
        drift_sigma_multiplier: 2.0,
        slew_sigma_multiplier: 2.0,
        grazing_window: 3,
        grazing_min_hits: 2,
        pre_failure_lookback_runs: 2,
        minimum_healthy_observations: 2,
        epsilon: 1.0e-9,
        dsa: dsfb_semiconductor::precursor::DsaConfig {
            window: 3,
            persistence_runs: 2,
            alert_tau: 1.5,
            corroborating_feature_count_min: 2,
        },
    }
}

fn write_phm2018_fixture_dataset(root: &Path) -> PathBuf {
    let phm_root = root.join("phm_data_challenge_2018");
    let train_dir = phm_root.join("train");
    let test_dir = phm_root.join("test");
    let fault_dir = train_dir.join("train_faults");
    let ttf_dir = train_dir.join("train_ttf");
    fs::create_dir_all(&fault_dir).unwrap();
    fs::create_dir_all(&ttf_dir).unwrap();
    fs::create_dir_all(&test_dir).unwrap();

    let mut train_csv =
        String::from("time,Tool,stage,Lot,runnum,recipe,recipe_step,sensor_a,sensor_b\n");
    for time in 0..60 {
        let sensor_a = if time < 40 {
            1.0
        } else {
            1.0 + (time - 39) as f64 * 0.35
        };
        let sensor_b = if time < 35 {
            0.5
        } else {
            0.5 + (time - 34) as f64 * 0.20
        };
        train_csv.push_str(&format!(
            "{time},T01,S01,L01,1,R01,1,{sensor_a:.3},{sensor_b:.3}\n"
        ));
    }
    fs::write(train_dir.join("L01_T01.csv"), train_csv).unwrap();
    fs::write(
        fault_dir.join("L01_T01_train_fault_data.csv"),
        "time,fault_name,Tool\n40,fault,T01\n",
    )
    .unwrap();
    fs::write(
        test_dir.join("L99_T99.csv"),
        "time,Tool,stage,Lot,runnum,recipe,recipe_step,sensor_a,sensor_b\n0,T99,S01,L99,1,R01,1,0.0,0.0\n",
    )
    .unwrap();

    root.to_path_buf()
}

fn pdflatex_available() -> bool {
    Command::new("pdflatex")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[test]
fn pipeline_outputs_are_deterministic_for_fixed_input() {
    let data_temp = tempfile::tempdir().unwrap();
    let output_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());
    let config = test_config();

    let first =
        run_secom_benchmark(&data_root, Some(output_temp.path()), config.clone(), false).unwrap();
    let second = run_secom_benchmark(&data_root, Some(output_temp.path()), config, false).unwrap();

    let first_metrics: Value =
        serde_json::from_str(&fs::read_to_string(&first.metrics_path).unwrap()).unwrap();
    let second_metrics: Value =
        serde_json::from_str(&fs::read_to_string(&second.metrics_path).unwrap()).unwrap();
    let first_combined_csv =
        fs::read_to_string(first.run_dir.join("drsc_dsa_combined.csv")).unwrap();
    let second_combined_csv =
        fs::read_to_string(second.run_dir.join("drsc_dsa_combined.csv")).unwrap();
    let first_combined_png =
        fs::read(first.run_dir.join("figures").join("drsc_dsa_combined.png")).unwrap();
    let second_combined_png =
        fs::read(second.run_dir.join("figures").join("drsc_dsa_combined.png")).unwrap();
    assert_eq!(first_metrics, second_metrics);
    assert_eq!(first_combined_csv, second_combined_csv);
    assert_eq!(first_combined_png, second_combined_png);
    assert_ne!(first.run_dir, second.run_dir);
    let first_name = first.run_dir.file_name().unwrap().to_string_lossy();
    let second_name = second.run_dir.file_name().unwrap().to_string_lossy();
    assert!(first_name.contains("dsfb-semiconductor"));
    assert!(second_name.contains("dsfb-semiconductor"));
    assert!(first.report.markdown_path.exists());
    assert!(first.report.tex_path.exists());
    assert!(first.zip_path.exists());
    if pdflatex_available() {
        assert!(first
            .report
            .pdf_path
            .as_ref()
            .is_some_and(|path| path.exists()));
    }
}

#[test]
fn missing_values_are_imputed_without_nan_or_inf_propagation() {
    let data_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());
    let dataset = secom::load_from_root(&data_root).unwrap();
    let prepared = prepare_secom(&dataset, &test_config()).unwrap();
    let nominal = build_nominal_model(&prepared, &test_config());
    let residuals = compute_residuals(&prepared, &nominal);
    let signs = compute_signs(&prepared, &nominal, &residuals, &test_config());
    let grammar = evaluate_grammar(&residuals, &signs, &nominal, &test_config());

    for trace in &residuals.traces {
        assert!(trace.imputed_values.iter().all(|value| value.is_finite()));
        assert!(trace.residuals.iter().all(|value| value.is_finite()));
        assert!(trace.norms.iter().all(|value| value.is_finite()));
    }
    for trace in &signs.traces {
        assert!(trace.drift.iter().all(|value| value.is_finite()));
        assert!(trace.slew.iter().all(|value| value.is_finite()));
        assert!(trace.drift_threshold.is_finite());
        assert!(trace.slew_threshold.is_finite());
    }
    assert_eq!(
        residuals.traces[1].imputed_values[3],
        nominal.features[1].healthy_mean
    );
    assert_eq!(residuals.traces[1].residuals[3], 0.0);
    assert_eq!(grammar.traces.len(), residuals.traces.len());
}

#[test]
fn benchmark_run_writes_expected_core_artifacts() {
    let data_temp = tempfile::tempdir().unwrap();
    let output_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());

    let artifacts =
        run_secom_benchmark(&data_root, Some(output_temp.path()), test_config(), false).unwrap();

    let expected_files = [
        "artifact_manifest.json",
        "baseline_comparison_summary.json",
        "benchmark_metrics.json",
        "dataset_summary.json",
        "density_metrics.csv",
        "dsa_grid_results.csv",
        "dsa_grid_summary.json",
        "dsa_feature_ranking.csv",
        "dsa_feature_ranking_recall_aware.csv",
        "dsa_feature_ranking_dsfb_aware.csv",
        "dsa_feature_ranking_burden_aware.csv",
        "dsa_feature_ranking_comparison.csv",
        "dsa_feature_cohorts.json",
        "dsa_feature_policy_overrides.json",
        "dsa_feature_policy_summary.csv",
        "dsa_cohort_results.csv",
        "dsa_cohort_results_recall_aware.csv",
        "dsa_cohort_results_dsfb_aware.csv",
        "dsa_cohort_summary.json",
        "dsa_cohort_summary_recall_aware.json",
        "dsa_cohort_summary_dsfb_aware.json",
        "dsa_cohort_precursor_quality.csv",
        "dsa_motif_policy_contributions.csv",
        "dsa_policy_contribution_analysis.csv",
        "dsa_recall_rescue_results.csv",
        "dsa_recall_critical_features.csv",
        "dsa_recall_recovery_efficiency.csv",
        "dsfb_single_change_iteration_log.csv",
        "optimization_log.json",
        "dsa_pareto_frontier.csv",
        "dsa_stage_a_candidates.csv",
        "dsa_stage_b_candidates.csv",
        "dsa_stage1_candidates.csv",
        "dsa_stage2_candidates.csv",
        "dsa_missed_failure_diagnostics.csv",
        "dsa_delta_target_assessment.json",
        "dsa_operator_baselines.json",
        "dsa_operator_delta_targets.json",
        "dsa_operator_delta_attainment_matrix.csv",
        "dsa_policy_operator_burden_contributions.csv",
        "failures_index.json",
        "missed_failure_priority.csv",
        "feature_motif_grounding.json",
        "feature_to_motif.json",
        "negative_control_report.json",
        "dsfb_heuristics_bank_minimal.json",
        "dsfb_heuristic_provenance.csv",
        "policy_decisions.csv",
        "policy_burden_summary.csv",
        "dsfb_feature_role_validation.csv",
        "dsfb_group_validation.csv",
        "dsa_cohort_results_burden_aware.csv",
        "dsa_cohort_summary_burden_aware.json",
        "dsa_heuristic_policy_failure_analysis.md",
        "dsa_parameter_manifest.json",
        "dsa_seed_feature_check.json",
        "dsfb_signs.csv",
        "dsfb_feature_signs.csv",
        "dsfb_motifs.csv",
        "dsfb_motif_labels_per_time.csv",
        "dsfb_feature_motif_timeline.csv",
        "feature_motif_timeline.csv",
        "dsfb_grammar_states.csv",
        "dsfb_feature_grammar_states.csv",
        "dsfb_envelope_interaction_summary.csv",
        "dsfb_heuristics_bank_expanded.json",
        "dsfb_semantic_matches.csv",
        "dsfb_semantic_ranked_candidates.csv",
        "dsfb_feature_policy_decisions.csv",
        "dsfb_group_definitions.json",
        "dsfb_group_signs.csv",
        "dsfb_group_grammar_states.csv",
        "dsfb_group_semantic_matches.csv",
        "dsfb_structural_delta_metrics.json",
        "drsc_dsa_combined.csv",
        "drsc_top_feature.csv",
        "dsa_top_feature.csv",
        "drifts.csv",
        "dsa_run_signals.csv",
        "cusum_baseline.csv",
        "run_energy_baseline.csv",
        "pca_fdc_baseline.csv",
        "ewma_baseline.csv",
        "engineering_report.md",
        "engineering_report.tex",
        "episode_precision_metrics.json",
        "feature_metrics.csv",
        "grammar_states.csv",
        "heuristics_bank.json",
        "lead_time_comparison.csv",
        "lead_time_explanation.json",
        "lead_time_metrics.csv",
        "parameter_manifest.json",
        "per_failure_run_signals.csv",
        "per_failure_run_dsa_signals.csv",
        "phm2018_support_status.json",
        "recurrent_boundary_stats.json",
        "recurrent_boundary_tradeoff_curve.csv",
        "recurrent_boundary_tradeoff_plot.png",
        "dsa_metrics.csv",
        "dsfb_metric_regrounding.csv",
        "dsa_vs_baselines.json",
        "missed_failure_root_cause.json",
        "paper_abstract_artifact.txt",
        "residuals.csv",
        "run_bundle.zip",
        "run_configuration.json",
        "secom_archive_layout.json",
        "slews.csv",
        "target_d_regression_analysis.json",
        "non_intrusive_interface_spec.md",
    ];

    for file in expected_files {
        assert!(
            artifacts.run_dir.join(file).exists(),
            "missing artifact {file}"
        );
    }
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("missingness_top20.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("benchmark_comparison.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("drsc_top_feature.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("drsc_dsa_combined.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("dsa_top_feature.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("grammar_timeline.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("top_feature_residual_norms.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("top_feature_drift.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("top_feature_ewma.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("top_feature_slew.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("dsfb_non_intrusive_architecture.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures")
        .join("dsfb_non_intrusive_architecture.svg")
        .exists());

    let archive = fs::File::open(artifacts.run_dir.join("run_bundle.zip")).unwrap();
    let mut zip = ZipArchive::new(archive).unwrap();
    assert!(zip.by_name("artifact_manifest.json").is_ok());
    assert!(zip.by_name("drsc_top_feature.csv").is_ok());
    assert!(zip.by_name("drsc_dsa_combined.csv").is_ok());
    assert!(zip.by_name("figures/missingness_top20.png").is_ok());
    assert!(zip.by_name("figures/benchmark_comparison.png").is_ok());
    assert!(zip.by_name("figures/drsc_top_feature.png").is_ok());
    assert!(zip.by_name("figures/drsc_dsa_combined.png").is_ok());
    assert!(zip.by_name("figures/dsa_top_feature.png").is_ok());
    assert!(zip.by_name("figures/grammar_timeline.png").is_ok());
    assert!(zip
        .by_name("figures/top_feature_residual_norms.png")
        .is_ok());
    assert!(zip.by_name("figures/top_feature_drift.png").is_ok());
    assert!(zip.by_name("figures/top_feature_ewma.png").is_ok());
    assert!(zip.by_name("figures/top_feature_slew.png").is_ok());
    assert!(zip
        .by_name("figures/dsfb_non_intrusive_architecture.png")
        .is_ok());
    assert!(zip
        .by_name("figures/dsfb_non_intrusive_architecture.svg")
        .is_ok());
    assert!(zip.by_name("dsa_metrics.csv").is_ok());
    assert!(zip.by_name("dsa_grid_results.csv").is_ok());
    assert!(zip.by_name("dsa_grid_summary.json").is_ok());
    assert!(zip.by_name("dsa_feature_ranking.csv").is_ok());
    assert!(zip.by_name("dsa_feature_ranking_recall_aware.csv").is_ok());
    assert!(zip.by_name("dsa_feature_ranking_dsfb_aware.csv").is_ok());
    assert!(zip.by_name("dsa_feature_ranking_burden_aware.csv").is_ok());
    assert!(zip.by_name("dsa_feature_ranking_comparison.csv").is_ok());
    assert!(zip.by_name("dsa_feature_cohorts.json").is_ok());
    assert!(zip.by_name("dsa_feature_policy_overrides.json").is_ok());
    assert!(zip.by_name("dsa_feature_policy_summary.csv").is_ok());
    assert!(zip.by_name("dsa_cohort_results.csv").is_ok());
    assert!(zip.by_name("dsa_cohort_results_recall_aware.csv").is_ok());
    assert!(zip.by_name("dsa_cohort_results_dsfb_aware.csv").is_ok());
    assert!(zip.by_name("dsa_cohort_summary.json").is_ok());
    assert!(zip.by_name("dsa_cohort_summary_recall_aware.json").is_ok());
    assert!(zip.by_name("dsa_cohort_summary_dsfb_aware.json").is_ok());
    assert!(zip.by_name("dsa_cohort_precursor_quality.csv").is_ok());
    assert!(zip.by_name("dsa_motif_policy_contributions.csv").is_ok());
    assert!(zip.by_name("dsa_policy_contribution_analysis.csv").is_ok());
    assert!(zip.by_name("dsa_recall_rescue_results.csv").is_ok());
    assert!(zip.by_name("dsa_recall_critical_features.csv").is_ok());
    assert!(zip.by_name("dsa_recall_recovery_efficiency.csv").is_ok());
    assert!(zip.by_name("dsa_pareto_frontier.csv").is_ok());
    assert!(zip.by_name("dsa_stage_a_candidates.csv").is_ok());
    assert!(zip.by_name("dsa_stage_b_candidates.csv").is_ok());
    assert!(zip.by_name("dsa_stage1_candidates.csv").is_ok());
    assert!(zip.by_name("dsa_stage2_candidates.csv").is_ok());
    assert!(zip.by_name("dsa_missed_failure_diagnostics.csv").is_ok());
    assert!(zip.by_name("dsa_delta_target_assessment.json").is_ok());
    assert!(zip.by_name("dsa_operator_baselines.json").is_ok());
    assert!(zip.by_name("dsa_operator_delta_targets.json").is_ok());
    assert!(zip
        .by_name("dsa_operator_delta_attainment_matrix.csv")
        .is_ok());
    assert!(zip
        .by_name("dsa_policy_operator_burden_contributions.csv")
        .is_ok());
    assert!(zip.by_name("recurrent_boundary_stats.json").is_ok());
    assert!(zip.by_name("recurrent_boundary_tradeoff_curve.csv").is_ok());
    assert!(zip.by_name("recurrent_boundary_tradeoff_plot.png").is_ok());
    assert!(zip.by_name("dsfb_metric_regrounding.csv").is_ok());
    assert!(zip.by_name("target_d_regression_analysis.json").is_ok());
    assert!(zip.by_name("missed_failure_root_cause.json").is_ok());
    assert!(zip.by_name("lead_time_comparison.csv").is_ok());
    assert!(zip.by_name("lead_time_explanation.json").is_ok());
    assert!(zip.by_name("episode_precision_metrics.json").is_ok());
    assert!(zip.by_name("paper_abstract_artifact.txt").is_ok());
    assert!(zip.by_name("optimization_log.json").is_ok());
    assert!(zip.by_name("non_intrusive_interface_spec.md").is_ok());
    assert!(zip.by_name("failures_index.json").is_ok());
    assert!(zip.by_name("feature_motif_grounding.json").is_ok());
    assert!(zip.by_name("dsfb_heuristics_bank_minimal.json").is_ok());
    assert!(zip.by_name("policy_decisions.csv").is_ok());
    assert!(zip.by_name("policy_burden_summary.csv").is_ok());
    assert!(zip.by_name("dsa_seed_feature_check.json").is_ok());
    assert!(zip.by_name("dsa_cohort_results_burden_aware.csv").is_ok());
    assert!(zip.by_name("dsa_cohort_summary_burden_aware.json").is_ok());
    assert!(zip.by_name("dsfb_signs.csv").is_ok());
    assert!(zip.by_name("dsfb_feature_signs.csv").is_ok());
    assert!(zip.by_name("dsfb_motifs.csv").is_ok());
    assert!(zip.by_name("dsfb_motif_labels_per_time.csv").is_ok());
    assert!(zip.by_name("dsfb_feature_motif_timeline.csv").is_ok());
    assert!(zip.by_name("feature_motif_timeline.csv").is_ok());
    assert!(zip.by_name("dsfb_grammar_states.csv").is_ok());
    assert!(zip.by_name("dsfb_feature_grammar_states.csv").is_ok());
    assert!(zip.by_name("dsfb_envelope_interaction_summary.csv").is_ok());
    assert!(zip.by_name("dsfb_heuristics_bank_expanded.json").is_ok());
    assert!(zip.by_name("dsfb_semantic_matches.csv").is_ok());
    assert!(zip.by_name("dsfb_semantic_ranked_candidates.csv").is_ok());
    assert!(zip.by_name("dsfb_feature_policy_decisions.csv").is_ok());
    assert!(zip.by_name("dsfb_group_definitions.json").is_ok());
    assert!(zip.by_name("dsfb_group_signs.csv").is_ok());
    assert!(zip.by_name("dsfb_group_grammar_states.csv").is_ok());
    assert!(zip.by_name("dsfb_group_semantic_matches.csv").is_ok());
    assert!(zip.by_name("dsfb_structural_delta_metrics.json").is_ok());
    assert!(zip.by_name("dsa_run_signals.csv").is_ok());
    assert!(zip.by_name("dsa_top_feature.csv").is_ok());
    assert!(zip.by_name("cusum_baseline.csv").is_ok());
    assert!(zip.by_name("run_energy_baseline.csv").is_ok());
    assert!(zip.by_name("per_failure_run_dsa_signals.csv").is_ok());
    if pdflatex_available() {
        assert!(artifacts.run_dir.join("engineering_report.pdf").exists());
        assert!(zip.by_name("engineering_report.pdf").is_ok());
    }

    let report = fs::read_to_string(artifacts.run_dir.join("engineering_report.md")).unwrap();
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.manifest_path).unwrap()).unwrap();
    let failure_case_paths = manifest
        .get("failure_case_paths")
        .and_then(Value::as_array)
        .expect("failure_case_paths should be present in the artifact manifest");
    assert!(
        !failure_case_paths.is_empty(),
        "failure_case_paths should not be empty"
    );
    for path in failure_case_paths {
        let entry = path
            .as_str()
            .and_then(|raw| std::path::Path::new(raw).file_name())
            .and_then(|name| name.to_str())
            .expect("failure_case_paths entries should have a filename");
        assert!(zip.by_name(entry).is_ok(), "missing zip entry for {entry}");
    }
    assert!(report.contains("## Artifact Inventory"));
    assert!(report.contains("## Deterministic Structural Accumulator (DSA)"));
    assert!(report.contains("## Feature-Cohort DSA Selection"));
    assert!(report.contains("## Heuristics-Governed DSA Policy Engine"));
    assert!(report.contains("## Semantics of Silence"));
    assert!(report.contains("## Predeclared Delta Target"));
    assert!(report.contains("## Recall Recovery Diagnostics"));
    assert!(report.contains("## Feature-Aware Heuristic Governance"));
    assert!(report.contains("## Missed-Failure Diagnostics"));
    assert!(report.contains("## Two-Stage Optimization Frontier"));
    assert!(report.contains("## Which Delta Matters on SECOM"));
    assert!(report.contains("## True DSFB Structural Semiotics Instantiation"));
    assert!(report.contains("## Non-Intrusive Integration Model"));
    assert!(report.contains("## Grouped / Coordinated Semiotics"));
    assert!(report.contains("## Missed Failure Analysis"));
    assert!(report.contains("## Feature -> Motif Grounding"));
    assert!(report.contains("## Heuristics With Justification"));
    assert!(report.contains("## DSFB vs EWMA Separation"));
    assert!(report.contains("## Executive Summary"));
    assert!(report.contains("## SECOM Structural Limitation"));
    assert!(report.contains("## Metric Re-Grounding"));
    assert!(report.contains("## Target D Regression Analysis"));
    assert!(report.contains("## Lead-Time Deficit Explanation"));
    assert!(report.contains("## Predeclared Operator Delta Targets"));
    assert!(report.contains("## Optimization Frontier"));
    assert!(report.contains("## Recall Recovery Efficiency"));
    assert!(report.contains("## Target Attainment Assessment"));
    assert!(report.contains("## Claims intentionally not made"));
    assert!(report.contains("## Legacy Nuisance Target Assessment"));
    assert!(report.contains("## Rating Delta Forecast"));
    assert!(report.contains(
        "## Deterministic Residual Stateflow Chart with Structural Accumulation (DRSC+DSA)"
    ));
    assert!(report.contains("DSFB Violation remains instantaneous envelope exit"));
    assert!(report.contains("drsc_dsa_combined.csv"));
    assert!(report.contains("figures/drsc_dsa_combined.png"));
    assert!(report.contains("dsa_vs_baselines.json"));
    assert!(report.contains("dsa_operator_delta_targets.json"));
    assert!(report.contains("dsa_operator_delta_attainment_matrix.csv"));
    assert!(report.contains("dsa_policy_operator_burden_contributions.csv"));
    assert!(manifest.get("drsc_dsa_combined_trace_path").is_some());
    assert!(manifest.get("drsc_dsa_combined_figure_path").is_some());
    assert!(manifest
        .get("dsa_motif_policy_contributions_path")
        .is_some());
    assert!(manifest.get("dsa_feature_policy_overrides_path").is_some());
    assert!(manifest
        .get("dsa_missed_failure_diagnostics_path")
        .is_some());
    assert!(manifest.get("dsa_recall_critical_features_path").is_some());
    assert!(manifest.get("dsa_delta_target_assessment_path").is_some());
    assert!(manifest.get("dsa_operator_baselines_path").is_some());
    assert!(manifest.get("dsa_operator_delta_targets_path").is_some());
    assert!(manifest
        .get("dsa_operator_delta_attainment_matrix_path")
        .is_some());
    assert!(manifest
        .get("dsa_policy_operator_burden_contributions_path")
        .is_some());
    assert!(manifest.get("recurrent_boundary_stats_path").is_some());
    assert!(manifest
        .get("recurrent_boundary_tradeoff_curve_path")
        .is_some());
    assert!(manifest
        .get("recurrent_boundary_tradeoff_plot_path")
        .is_some());
    assert!(manifest.get("dsfb_metric_regrounding_path").is_some());
    assert!(manifest.get("target_d_regression_analysis_path").is_some());
    assert!(manifest.get("missed_failure_root_cause_path").is_some());
    assert!(manifest.get("lead_time_comparison_path").is_some());
    assert!(manifest.get("lead_time_explanation_path").is_some());
    assert!(manifest.get("episode_precision_metrics_path").is_some());
    assert!(manifest.get("paper_abstract_artifact_path").is_some());
    assert!(manifest.get("optimization_log_path").is_some());
    assert!(manifest
        .get("dsa_recall_recovery_efficiency_path")
        .is_some());
    assert!(manifest.get("failures_index_path").is_some());
    assert!(manifest.get("failure_case_paths").is_some());
    assert!(manifest.get("feature_motif_grounding_path").is_some());
    assert!(manifest.get("dsfb_heuristics_bank_minimal_path").is_some());
    assert!(manifest.get("policy_decisions_path").is_some());
    assert!(manifest.get("policy_burden_summary_path").is_some());
    assert!(manifest.get("dsfb_vs_ewma_case_paths").is_some());
    assert!(manifest
        .get("dsa_feature_ranking_burden_aware_path")
        .is_some());
    assert!(manifest
        .get("dsa_feature_ranking_dsfb_aware_path")
        .is_some());
    assert!(manifest
        .get("dsa_cohort_results_burden_aware_path")
        .is_some());
    assert!(manifest.get("dsa_cohort_results_dsfb_aware_path").is_some());
    assert!(manifest
        .get("dsa_cohort_summary_burden_aware_path")
        .is_some());
    assert!(manifest.get("dsa_cohort_summary_dsfb_aware_path").is_some());
    assert!(manifest.get("dsfb_signs_path").is_some());
    assert!(manifest.get("dsfb_feature_signs_path").is_some());
    assert!(manifest.get("dsfb_motifs_path").is_some());
    assert!(manifest.get("dsfb_motif_labels_per_time_path").is_some());
    assert!(manifest.get("dsfb_feature_motif_timeline_path").is_some());
    assert!(manifest.get("dsfb_grammar_states_path").is_some());
    assert!(manifest.get("dsfb_feature_grammar_states_path").is_some());
    assert!(manifest
        .get("dsfb_envelope_interaction_summary_path")
        .is_some());
    assert!(manifest.get("dsfb_heuristics_bank_expanded_path").is_some());
    assert!(manifest.get("dsfb_semantic_matches_path").is_some());
    assert!(manifest
        .get("dsfb_semantic_ranked_candidates_path")
        .is_some());
    assert!(manifest.get("dsfb_feature_policy_decisions_path").is_some());
    assert!(manifest.get("dsfb_group_definitions_path").is_some());
    assert!(manifest.get("dsfb_group_signs_path").is_some());
    assert!(manifest.get("dsfb_group_grammar_states_path").is_some());
    assert!(manifest.get("dsfb_group_semantic_matches_path").is_some());
    assert!(manifest.get("dsfb_structural_delta_metrics_path").is_some());
    assert!(manifest.get("non_intrusive_interface_spec_path").is_some());
    assert!(manifest
        .get("non_intrusive_architecture_png_path")
        .is_some());
    assert!(manifest
        .get("non_intrusive_architecture_svg_path")
        .is_some());

    let feature_to_motif: Value = serde_json::from_str(
        &fs::read_to_string(artifacts.run_dir.join("feature_to_motif.json")).unwrap(),
    )
    .unwrap();
    let first_mapping = feature_to_motif
        .as_array()
        .and_then(|rows| rows.first())
        .expect("feature_to_motif.json should contain at least one row");
    assert!(first_mapping.get("feature_name").is_some());
    assert!(first_mapping.get("initial_role").is_some());
    assert!(first_mapping.get("motif_type").is_some());
}

#[test]
fn heuristics_bank_entries_include_operational_fields() {
    let data_temp = tempfile::tempdir().unwrap();
    let output_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());

    let artifacts =
        run_secom_benchmark(&data_root, Some(output_temp.path()), test_config(), false).unwrap();
    let heuristics: Value = serde_json::from_str(
        &fs::read_to_string(artifacts.run_dir.join("heuristics_bank.json")).unwrap(),
    )
    .unwrap();

    let first = &heuristics.as_array().unwrap()[0];
    for key in [
        "severity",
        "confidence",
        "alert_class_default",
        "requires_persistence",
        "requires_corroboration",
        "minimum_window",
        "minimum_hits",
        "maximum_allowed_fragmentation",
        "contributes_to_dsa_scoring",
        "contributes_to_dsa",
        "suppresses_alert",
        "promotes_alert",
        "recommended_action",
        "escalation_policy",
        "non_unique_warning",
        "known_limitations",
        "observed_point_hits",
        "observed_run_hits",
        "pre_failure_window_run_hits",
    ] {
        assert!(first.get(key).is_some(), "missing heuristics field {key}");
    }
}

#[test]
fn phm2018_benchmark_writes_expected_artifacts() {
    let secom_data_temp = tempfile::tempdir().unwrap();
    let secom_output_temp = tempfile::tempdir().unwrap();
    let phm_data_temp = tempfile::tempdir().unwrap();
    let phm_output_temp = tempfile::tempdir().unwrap();

    let secom_data_root = write_fixture_dataset(secom_data_temp.path());
    let secom_artifacts = run_secom_benchmark(
        &secom_data_root,
        Some(secom_output_temp.path()),
        test_config(),
        false,
    )
    .unwrap();
    let phm_data_root = write_phm2018_fixture_dataset(phm_data_temp.path());
    let phm_artifacts = run_phm2018_benchmark(
        &phm_data_root,
        phm_output_temp.path(),
        Some(&secom_artifacts.run_dir),
    )
    .unwrap();

    for file in [
        "artifact_manifest.json",
        "claim_alignment_report.json",
        "phm2018_early_warning_stats.json",
        "phm2018_lead_time_metrics.csv",
        "phm2018_run_details.json",
        "phm2018_structural_metrics.json",
        "phm2018_support_status.json",
        "run_bundle.zip",
    ] {
        assert!(
            phm_artifacts.run_dir.join(file).exists(),
            "missing PHM artifact {file}"
        );
    }

    let metrics: Value =
        serde_json::from_str(&fs::read_to_string(&phm_artifacts.early_warning_stats_path).unwrap())
            .unwrap();
    assert!(metrics.get("percent_runs_dsfb_earlier").is_some());
    assert!(metrics.get("percent_runs_equal").is_some());
    assert!(metrics.get("percent_runs_later").is_some());

    let claim_alignment: Value = serde_json::from_str(
        &fs::read_to_string(&phm_artifacts.claim_alignment_report_path).unwrap(),
    )
    .unwrap();
    assert!(claim_alignment.get("secom_supports").is_some());
    assert!(claim_alignment.get("secom_does_not_support").is_some());
    assert!(claim_alignment.get("phm2018_supports").is_some());
    assert!(claim_alignment.get("claims_not_made").is_some());

    let structural_metrics: Value =
        serde_json::from_str(&fs::read_to_string(&phm_artifacts.structural_metrics_path).unwrap())
            .unwrap();
    assert!(structural_metrics
        .get("runs_with_structure_before_threshold")
        .is_some());
    assert!(structural_metrics
        .get("percent_structure_before_threshold")
        .is_some());

    let archive = fs::File::open(&phm_artifacts.zip_path).unwrap();
    let mut zip = ZipArchive::new(archive).unwrap();
    assert!(zip.by_name("artifact_manifest.json").is_ok());
    assert!(zip.by_name("claim_alignment_report.json").is_ok());
    assert!(zip.by_name("phm2018_early_warning_stats.json").is_ok());
    assert!(zip.by_name("phm2018_lead_time_metrics.csv").is_ok());
    assert!(zip.by_name("phm2018_run_details.json").is_ok());
    assert!(zip.by_name("phm2018_structural_metrics.json").is_ok());
    assert!(zip.by_name("phm2018_support_status.json").is_ok());
}

#[test]
fn calibration_grid_writes_expected_artifacts() {
    let data_temp = tempfile::tempdir().unwrap();
    let output_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());

    let artifacts = run_secom_calibration(
        &data_root,
        Some(output_temp.path()),
        CalibrationGrid {
            healthy_pass_runs: vec![3, 4],
            drift_window: vec![2],
            envelope_sigma: vec![2.0],
            boundary_fraction_of_rho: vec![0.5],
            state_confirmation_steps: vec![2],
            persistent_state_steps: vec![2],
            density_window: vec![3],
            ewma_alpha: vec![0.3],
            ewma_sigma_multiplier: vec![2.0],
            cusum_kappa_sigma_multiplier: vec![0.5],
            cusum_alarm_sigma_multiplier: vec![4.0],
            run_energy_sigma_multiplier: vec![3.0],
            pca_variance_explained: vec![0.95],
            pca_t2_sigma_multiplier: vec![3.0],
            pca_spe_sigma_multiplier: vec![3.0],
            drift_sigma_multiplier: vec![2.0],
            slew_sigma_multiplier: vec![2.0],
            grazing_window: vec![3],
            grazing_min_hits: vec![2],
            pre_failure_lookback_runs: vec![2],
        },
        false,
    )
    .unwrap();

    for file in [
        "calibration_grid_results.csv",
        "calibration_best_by_metric.json",
        "calibration_report.md",
        "calibration_run_configuration.json",
        "parameter_grid_manifest.json",
    ] {
        assert!(
            artifacts.run_dir.join(file).exists(),
            "missing calibration artifact {file}"
        );
    }
}

#[test]
fn dsa_calibration_writes_expected_artifacts() {
    let data_temp = tempfile::tempdir().unwrap();
    let output_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());

    let artifacts = run_secom_dsa_calibration(
        &data_root,
        Some(output_temp.path()),
        test_config(),
        dsfb_semiconductor::precursor::DsaCalibrationGrid::bounded_default(),
        false,
    )
    .unwrap();

    for file in [
        "dsa_grid_results.csv",
        "dsa_grid_summary.json",
        "dsa_calibration_report.md",
        "dsa_calibration_run_configuration.json",
        "dsa_parameter_grid_manifest.json",
    ] {
        assert!(
            artifacts.run_dir.join(file).exists(),
            "missing dsa calibration artifact {file}"
        );
    }
}

#[test]
fn ewma_baseline_reacts_to_sustained_elevation() {
    let data_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());
    let dataset = secom::load_from_root(&data_root).unwrap();
    let config = test_config();
    let prepared = prepare_secom(&dataset, &config).unwrap();
    let nominal = build_nominal_model(&prepared, &config);
    let residuals = compute_residuals(&prepared, &nominal);
    let baselines = compute_baselines(&prepared, &nominal, &residuals, &config);

    let feature = &baselines.ewma[0];
    assert_eq!(ewma_series(&[0.0, 1.0, 1.0], 0.5), vec![0.0, 0.5, 0.75]);
    assert!(feature.ewma.iter().all(|value| value.is_finite()));
    assert!(feature.threshold.is_finite());
    assert!(feature.alarm.iter().any(|flag| *flag));
}

#[test]
fn dsa_outputs_are_finite_and_reproducible() {
    let data_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());
    let config = test_config();
    let dataset = secom::load_from_root(&data_root).unwrap();
    let prepared = prepare_secom(&dataset, &config).unwrap();
    let nominal = build_nominal_model(&prepared, &config);
    let residuals = compute_residuals(&prepared, &nominal);
    let signs = compute_signs(&prepared, &nominal, &residuals, &config);
    let baselines = compute_baselines(&prepared, &nominal, &residuals, &config);
    let grammar = evaluate_grammar(&residuals, &signs, &nominal, &config);

    let first = evaluate_dsa(
        &prepared,
        &nominal,
        &residuals,
        &signs,
        &baselines,
        &grammar,
        &config.dsa,
        config.pre_failure_lookback_runs,
    )
    .unwrap();
    let second = evaluate_dsa(
        &prepared,
        &nominal,
        &residuals,
        &signs,
        &baselines,
        &grammar,
        &config.dsa,
        config.pre_failure_lookback_runs,
    )
    .unwrap();

    let first_json = serde_json::to_value(&first.summary).unwrap();
    let second_json = serde_json::to_value(&second.summary).unwrap();
    assert_eq!(first_json, second_json);
    assert_eq!(first.run_signals.corroborating_feature_count_min, 2);
    assert!(first
        .run_signals
        .primary_run_signal
        .contains("feature_count_review_or_escalate"));

    for trace in &first.traces {
        assert!(trace
            .boundary_density_w
            .iter()
            .all(|value| value.is_finite()));
        assert!(trace
            .drift_persistence_w
            .iter()
            .all(|value| value.is_finite()));
        assert!(trace.slew_density_w.iter().all(|value| value.is_finite()));
        assert!(trace.ewma_occupancy_w.iter().all(|value| value.is_finite()));
        assert!(trace
            .motif_recurrence_w
            .iter()
            .all(|value| value.is_finite()));
        assert!(trace
            .fragmentation_proxy_w
            .iter()
            .all(|value| value.is_finite()));
        assert!(trace.dsa_score.iter().all(|value| value.is_finite()));
    }
}
