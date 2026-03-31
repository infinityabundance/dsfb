use dsfb_semiconductor::baselines::{compute_baselines, ewma_series};
use dsfb_semiconductor::calibration::{
    run_secom_calibration, run_secom_precursor_calibration, CalibrationGrid,
};
use dsfb_semiconductor::config::PipelineConfig;
use dsfb_semiconductor::dataset::secom;
use dsfb_semiconductor::grammar::evaluate_grammar;
use dsfb_semiconductor::nominal::build_nominal_model;
use dsfb_semiconductor::pipeline::run_secom_benchmark;
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
        drift_sigma_multiplier: 2.0,
        slew_sigma_multiplier: 2.0,
        grazing_window: 3,
        grazing_min_hits: 2,
        pre_failure_lookback_runs: 2,
        minimum_healthy_observations: 2,
        epsilon: 1.0e-9,
        precursor: dsfb_semiconductor::precursor::PrecursorConfig {
            window: 3,
            persistence_runs: 2,
            boundary_density_tau: 0.3,
            drift_persistence_tau: 0.3,
            transition_cluster_tau: 2,
            ewma_occupancy_tau: 0.8,
            alert_tau: 1.5,
        },
    }
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
    assert_eq!(first_metrics, second_metrics);
    assert_ne!(first.run_dir, second.run_dir);
    let first_name = first.run_dir.file_name().unwrap().to_string_lossy();
    let second_name = second.run_dir.file_name().unwrap().to_string_lossy();
    assert!(first_name.contains("dsfb-semiconductor"));
    assert!(second_name.contains("dsfb-semiconductor"));
    assert!(first.report.markdown_path.exists());
    assert!(first.report.tex_path.exists());
    assert!(first.zip_path.exists());
    if pdflatex_available() {
        assert!(first.report.pdf_path.as_ref().is_some_and(|path| path.exists()));
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
        "drsc_top_feature.csv",
        "drifts.csv",
        "ewma_baseline.csv",
        "engineering_report.md",
        "engineering_report.tex",
        "feature_metrics.csv",
        "grammar_states.csv",
        "heuristics_bank.json",
        "lead_time_metrics.csv",
        "parameter_manifest.json",
        "per_failure_run_signals.csv",
        "per_failure_run_precursor_signals.csv",
        "phm2018_support_status.json",
        "precursor_metrics.csv",
        "precursor_vs_baselines_summary.json",
        "residuals.csv",
        "run_bundle.zip",
        "run_configuration.json",
        "secom_archive_layout.json",
        "slews.csv",
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

    let archive = fs::File::open(artifacts.run_dir.join("run_bundle.zip")).unwrap();
    let mut zip = ZipArchive::new(archive).unwrap();
    assert!(zip.by_name("artifact_manifest.json").is_ok());
    assert!(zip.by_name("drsc_top_feature.csv").is_ok());
    assert!(zip.by_name("figures/missingness_top20.png").is_ok());
    assert!(zip.by_name("figures/benchmark_comparison.png").is_ok());
    assert!(zip.by_name("figures/drsc_top_feature.png").is_ok());
    assert!(zip.by_name("figures/grammar_timeline.png").is_ok());
    assert!(zip.by_name("figures/top_feature_residual_norms.png").is_ok());
    assert!(zip.by_name("figures/top_feature_drift.png").is_ok());
    assert!(zip.by_name("figures/top_feature_ewma.png").is_ok());
    assert!(zip.by_name("figures/top_feature_slew.png").is_ok());
    assert!(zip.by_name("precursor_metrics.csv").is_ok());
    assert!(zip.by_name("per_failure_run_precursor_signals.csv").is_ok());
    if pdflatex_available() {
        assert!(artifacts.run_dir.join("engineering_report.pdf").exists());
        assert!(zip.by_name("engineering_report.pdf").is_ok());
    }

    let report = fs::read_to_string(artifacts.run_dir.join("engineering_report.md")).unwrap();
    assert!(report.contains("## Artifact Inventory"));
    assert!(report.contains("engineering_report.pdf"));
    assert!(report.contains("run_bundle.zip"));
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
fn precursor_calibration_writes_expected_artifacts() {
    let data_temp = tempfile::tempdir().unwrap();
    let output_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());

    let artifacts = run_secom_precursor_calibration(
        &data_root,
        Some(output_temp.path()),
        test_config(),
        false,
    )
    .unwrap();

    for file in [
        "precursor_calibration_grid.csv",
        "precursor_calibration_run_configuration.json",
        "precursor_parameter_grid_manifest.json",
    ] {
        assert!(
            artifacts.run_dir.join(file).exists(),
            "missing precursor calibration artifact {file}"
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
