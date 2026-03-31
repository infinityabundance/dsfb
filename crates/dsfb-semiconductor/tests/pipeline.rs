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
        drift_sigma_multiplier: 2.0,
        slew_sigma_multiplier: 2.0,
        grazing_window: 3,
        grazing_min_hits: 2,
        pre_failure_lookback_runs: 2,
        minimum_healthy_observations: 2,
        epsilon: 1.0e-9,
    }
}

#[test]
fn pipeline_outputs_are_deterministic_for_fixed_input() {
    let data_temp = tempfile::tempdir().unwrap();
    let output_temp = tempfile::tempdir().unwrap();
    let data_root = write_fixture_dataset(data_temp.path());
    let config = test_config();

    let first = run_secom_benchmark(&data_root, Some(output_temp.path()), config.clone(), false).unwrap();
    let second = run_secom_benchmark(&data_root, Some(output_temp.path()), config, false).unwrap();

    let first_metrics: Value = serde_json::from_str(&fs::read_to_string(&first.metrics_path).unwrap()).unwrap();
    let second_metrics: Value =
        serde_json::from_str(&fs::read_to_string(&second.metrics_path).unwrap()).unwrap();
    assert_eq!(first_metrics, second_metrics);
    assert_ne!(first.run_dir, second.run_dir);
    assert!(first.report.markdown_path.exists());
    assert!(first.report.tex_path.exists());
    assert!(first.zip_path.exists());
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
    assert_eq!(residuals.traces[1].imputed_values[3], nominal.features[1].healthy_mean);
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
        "benchmark_metrics.json",
        "dataset_summary.json",
        "drifts.csv",
        "engineering_report.md",
        "engineering_report.tex",
        "feature_metrics.csv",
        "grammar_states.csv",
        "heuristics_bank.json",
        "parameter_manifest.json",
        "phm2018_support_status.json",
        "residuals.csv",
        "run_bundle.zip",
        "run_configuration.json",
        "slews.csv",
    ];

    for file in expected_files {
        assert!(artifacts.run_dir.join(file).exists(), "missing artifact {file}");
    }
    assert!(artifacts.run_dir.join("figures").join("benchmark_comparison.png").exists());
    assert!(artifacts.run_dir.join("figures").join("grammar_timeline.png").exists());
}
