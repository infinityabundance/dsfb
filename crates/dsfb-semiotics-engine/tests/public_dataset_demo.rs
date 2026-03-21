use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::dashboard::{CsvReplayDriver, DashboardReplayConfig};
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use serde_json::Value;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fetch_script() -> PathBuf {
    crate_root().join("tools/fetch_public_dataset.py")
}

fn preprocess_script() -> PathBuf {
    crate_root().join("tools/preprocess_public_dataset.py")
}

fn ensure_fetch_ran() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let status = Command::new("python3")
            .arg(fetch_script())
            .arg("--dataset")
            .arg("nasa_milling")
            .arg("--dataset")
            .arg("nasa_bearings")
            .status()
            .unwrap();
        assert!(status.success());
    });
}

fn ensure_preprocess_ran() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        ensure_fetch_ran();
        let status = Command::new("python3")
            .arg(preprocess_script())
            .arg("--dataset")
            .arg("nasa_milling")
            .arg("--dataset")
            .arg("nasa_bearings")
            .status()
            .unwrap();
        assert!(status.success());
    });
}

fn ensure_full_pipeline_ran() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let output = Command::new(env!("CARGO_BIN_EXE_dsfb-public-dataset-demo"))
            .arg("--dataset")
            .arg("nasa_milling")
            .arg("--dataset")
            .arg("nasa_bearings")
            .arg("--phase")
            .arg("all")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    });
}

fn dataset_root(dataset: &str) -> PathBuf {
    crate_root()
        .join("artifacts/public_dataset_demo")
        .join(dataset)
        .join("latest")
}

fn sample_root(dataset: &str) -> PathBuf {
    crate_root()
        .join("examples/public_dataset_demo")
        .join(dataset)
}

fn processed_root(dataset: &str) -> PathBuf {
    crate_root().join("data/processed").join(dataset)
}

fn processed_input(dataset: &str) -> CsvInputConfig {
    CsvInputConfig {
        observed_csv: processed_root(dataset).join("observed.csv"),
        predicted_csv: processed_root(dataset).join("predicted.csv"),
        scenario_id: format!("{dataset}_public_demo"),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: if dataset == "nasa_milling" {
            0.30
        } else {
            0.35
        },
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: format!("{dataset}_envelope"),
    }
}

#[test]
fn test_dataset_fetch_script_exists() {
    assert!(fetch_script().is_file());
}

#[test]
fn test_dataset_fetch_script_runs() {
    ensure_fetch_ran();
}

#[test]
fn test_dataset_files_exist_after_fetch() {
    ensure_fetch_ran();
    assert!(crate_root()
        .join("data/public_dataset/raw/nasa_milling_raw_summary.csv")
        .is_file());
    assert!(crate_root()
        .join("data/public_dataset/raw/nasa_bearings_raw_summary.csv")
        .is_file());
}

#[test]
fn test_dataset_integrity_check_passes() {
    ensure_fetch_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let metadata = fs::read_to_string(
            crate_root()
                .join("data/public_dataset/raw")
                .join(format!("{dataset}_source_metadata.json")),
        )
        .unwrap();
        let value: Value = serde_json::from_str(&metadata).unwrap();
        assert_eq!(value["dataset"], dataset);
        assert!(value["record_count"].as_u64().unwrap() > 0);
        assert!(value["source_url"].as_str().unwrap().contains("NASA"));
    }
}

#[test]
fn test_preprocessing_script_exists() {
    assert!(preprocess_script().is_file());
}

#[test]
fn test_preprocessing_produces_csv() {
    ensure_preprocess_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(processed_root(dataset).join("observed.csv").is_file());
        assert!(processed_root(dataset).join("predicted.csv").is_file());
        assert!(processed_root(dataset).join("metadata.json").is_file());
    }
}

#[test]
fn test_preprocessing_is_deterministic() {
    ensure_preprocess_ran();
    let before = fs::read_to_string(processed_root("nasa_milling").join("observed.csv")).unwrap();
    let status = Command::new("python3")
        .arg(preprocess_script())
        .arg("--dataset")
        .arg("nasa_milling")
        .status()
        .unwrap();
    assert!(status.success());
    let after = fs::read_to_string(processed_root("nasa_milling").join("observed.csv")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn test_processed_csv_matches_expected_schema() {
    ensure_preprocess_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let observed = fs::read_to_string(processed_root(dataset).join("observed.csv")).unwrap();
        let header = observed.lines().next().unwrap();
        assert!(header.starts_with("step,time,"));
        assert!(header.split(',').count() >= 4);
    }
}

#[test]
fn test_full_public_dataset_pipeline_runs() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(dataset_root(dataset).is_dir());
    }
}

#[test]
fn test_pdf_generated() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(dataset_root(dataset)
            .join("report/dsfb_semiotics_engine_report.pdf")
            .is_file());
    }
}

#[test]
fn test_pngs_generated() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let figures_dir = dataset_root(dataset).join("figures");
        assert!(figures_dir.is_dir());
        assert!(fs::read_dir(figures_dir).unwrap().any(|entry| entry
            .unwrap()
            .path()
            .extension()
            .unwrap_or_default()
            == "png"));
    }
}

#[test]
fn test_zip_generated() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(fs::read_dir(dataset_root(dataset))
            .unwrap()
            .any(|entry| entry.unwrap().path().extension().unwrap_or_default() == "zip"));
    }
}

#[test]
fn test_replay_csv_generated() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(dataset_root(dataset)
            .join("replay/replay_events.csv")
            .is_file());
    }
}

#[test]
fn test_dashboard_replay_works_with_dataset_output() {
    ensure_preprocess_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let replay = CsvReplayDriver::from_csv_run(
            CommonRunConfig::default(),
            processed_input(dataset),
            EngineSettings::default(),
            DashboardReplayConfig::default(),
        )
        .unwrap();
        assert!(!replay.stream().events.is_empty());
    }
}

#[test]
fn test_replay_file_exists_and_loads() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let replay =
            fs::read_to_string(dataset_root(dataset).join("replay/replay_events.csv")).unwrap();
        assert!(replay.contains("syntax_label"));
        assert!(replay.contains("grammar_state"));
    }
}

#[test]
fn test_replay_has_non_empty_event_sequence() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let replay =
            fs::read_to_string(dataset_root(dataset).join("replay/replay_events.csv")).unwrap();
        let lines = replay.lines().collect::<Vec<_>>();
        assert!(lines.len() > 2);
        assert!(lines.iter().skip(1).any(|line| line.contains("syntax")));
    }
}

#[test]
fn test_sample_artifacts_exist() {
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(sample_root(dataset).join("manifest.json").is_file());
        assert!(sample_root(dataset)
            .join("report/dsfb_semiotics_engine_report.pdf")
            .is_file());
        assert!(sample_root(dataset)
            .join("replay/replay_events.csv")
            .is_file());
    }
}

#[test]
fn test_sample_pdf_readable() {
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let pdf = sample_root(dataset).join("report/dsfb_semiotics_engine_report.pdf");
        assert!(pdf.is_file());
        assert!(fs::metadata(pdf).unwrap().len() > 0);
    }
}

#[test]
fn test_sample_png_exists() {
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let figures_dir = sample_root(dataset).join("figures");
        assert!(fs::read_dir(figures_dir).unwrap().any(|entry| entry
            .unwrap()
            .path()
            .extension()
            .unwrap_or_default()
            == "png"));
    }
}

#[test]
fn test_public_dataset_demo_docs_exist() {
    assert!(crate_root().join("docs/public_dataset_demo.md").is_file());
}

#[test]
fn test_docs_include_commands() {
    let docs = fs::read_to_string(crate_root().join("docs/public_dataset_demo.md")).unwrap();
    assert!(docs.contains("dsfb-public-dataset-demo"));
    assert!(docs.contains("just demo-public-dataset"));
    assert!(docs.contains("--dashboard-replay-csv"));
}

#[test]
fn test_docs_reference_artifacts() {
    let docs = fs::read_to_string(crate_root().join("docs/public_dataset_demo.md")).unwrap();
    assert!(docs.contains("examples/public_dataset_demo/nasa_milling"));
    assert!(docs.contains("examples/public_dataset_demo/nasa_bearings"));
    assert!(docs.contains("artifacts/public_dataset_demo/nasa_milling/latest"));
}
