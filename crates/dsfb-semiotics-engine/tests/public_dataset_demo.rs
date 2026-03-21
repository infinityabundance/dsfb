use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::dashboard::{CsvReplayDriver, DashboardReplayConfig};
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use dsfb_semiotics_engine::figures::source::FigureSourceTable;
use serde_json::Value;
use zip::ZipArchive;

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

fn latest_zip_path(dataset: &str) -> PathBuf {
    fs::read_dir(dataset_root(dataset))
        .unwrap()
        .find_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension().unwrap_or_default() == "zip").then_some(path)
        })
        .unwrap_or_else(|| panic!("missing zip archive for {dataset}"))
}

fn processed_root(dataset: &str) -> PathBuf {
    crate_root().join("data/processed").join(dataset)
}

fn latest_source_table(dataset: &str, figure_id: &str) -> FigureSourceTable {
    let source_path = dataset_root(dataset)
        .join("json")
        .join(format!("{figure_id}_source.json"));
    serde_json::from_str(&fs::read_to_string(source_path).unwrap()).unwrap()
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
fn test_public_dataset_committed_raw_summary_cache_exists() {
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(crate_root()
            .join("data/public_dataset/raw")
            .join(format!("{dataset}_raw_summary.csv"))
            .is_file());
        assert!(crate_root()
            .join("data/public_dataset/raw")
            .join(format!("{dataset}_source_metadata.json"))
            .is_file());
    }
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
fn test_public_dataset_zip_name_has_dataset_prefix() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let zip_path = latest_zip_path(dataset);
        let file_name = zip_path.file_name().unwrap().to_string_lossy();
        assert!(file_name.starts_with(&format!("{dataset}-dsfb-semiotics-engine-")));
    }
}

#[test]
fn test_public_dataset_zip_root_folder_matches_zip_name() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let zip_path = latest_zip_path(dataset);
        let zip_stem = zip_path.file_stem().unwrap().to_string_lossy().to_string();
        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        assert!(!archive.is_empty());
        for index in 0..archive.len() {
            let entry = archive.by_index(index).unwrap();
            assert!(entry.name().starts_with(&format!("{zip_stem}/")));
        }
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
fn test_public_dataset_detectability_figure_uses_observed_event_fallback() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let table = latest_source_table(dataset, "figure_09_detectability_bound_comparison");
        if dataset == "nasa_bearings" {
            assert_eq!(
                table.panel_ids,
                vec![
                    "primary_magnitude_similarity",
                    "meta_residual_divergence",
                    "outcome_consequence"
                ]
            );
            assert!(table.rows.iter().any(|row| {
                row.panel_id == "primary_magnitude_similarity"
                    && row.series_id == "stable_primary_window"
            }));
        } else {
            assert!(table.rows.iter().any(|row| {
                row.panel_id == "detectability_context"
                    && row.series_kind == "segment"
                    && matches!(
                        row.series_id.as_str(),
                        "first_boundary_time" | "first_violation_time"
                    )
            }));
            assert!(table.rows.iter().any(|row| {
                row.panel_id == "detectability_window_ratio"
                    && row.series_kind == "bar"
                    && row.series_id == "window_max_ratio"
            }));
        }
    }
}

#[test]
fn test_public_dataset_group_figure_uses_multi_channel_fallback() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let table = latest_source_table(dataset, "figure_11_coordinated_group_semiotics");
        assert!(table
            .rows
            .iter()
            .any(|row| { row.panel_id == "local_channels" && row.series_id == "local_channel_1" }));
        assert!(table.rows.iter().any(|row| {
            row.panel_id == "aggregate_group" && row.series_id == "aggregate_abs_mean"
        }));
    }
}

#[test]
fn test_public_dataset_semantic_retrieval_uses_compact_tick_labels() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let table = latest_source_table(dataset, "figure_12_semantic_retrieval_heuristics_bank");
        if dataset == "nasa_bearings" {
            assert_eq!(
                table.panel_ids,
                vec![
                    "semantic_score_timeline",
                    "semantic_candidate_count_timeline",
                    "semantic_disposition_timeline"
                ]
            );
            assert!(table.rows.iter().all(|row| row.series_kind == "line"));
        } else {
            let labels = table
                .rows
                .iter()
                .filter(|row| row.series_kind == "bar")
                .map(|row| row.x_tick_label.as_str())
                .collect::<Vec<_>>();
            assert!(labels.iter().any(|label| label.contains("nasa")));
            assert!(!labels.iter().any(|label| label.contains("_public_demo")));
        }
    }
}

#[test]
fn test_public_dataset_comparator_source_preserves_figure_source_table() {
    ensure_full_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let table = latest_source_table(dataset, "figure_13_internal_baseline_comparators");
        assert_eq!(table.figure_id, "figure_13_internal_baseline_comparators");
        if dataset == "nasa_bearings" {
            assert!(table
                .rows
                .iter()
                .any(|row| row.panel_id == "baseline_alarm_timing"));
            assert!(table
                .rows
                .iter()
                .any(|row| row.panel_id == "dsfb_grammar_timeline"));
            assert!(table
                .rows
                .iter()
                .any(|row| row.panel_id == "dsfb_semantic_timeline"));
        } else {
            assert!(table.rows.iter().any(|row| row.series_kind == "bar"));
        }
        assert!(dataset_root(dataset)
            .join("json/figure_13_internal_baseline_comparators_legacy_source.json")
            .is_file());
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
