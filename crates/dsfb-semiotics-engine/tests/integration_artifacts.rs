use std::fs;

use clap::Parser;
use dsfb_semiotics_engine::cli::args::{CsvInputConfig, ScenarioSelection};
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::types::{EnvelopeMode, GrammarState};
use dsfb_semiotics_engine::io::input::load_csv_trajectories;
use dsfb_semiotics_engine::io::output::create_output_layout;
use serde::Deserialize;
use tempfile::TempDir;
use zip::ZipArchive;

#[test]
fn output_path_creation_builds_expected_subdirectories() {
    let temp = TempDir::new().unwrap();
    let layout = create_output_layout(temp.path()).unwrap();

    assert!(layout.run_dir.starts_with(temp.path()));
    assert!(layout.figures_dir.exists());
    assert!(layout.csv_dir.exists());
    assert!(layout.json_dir.exists());
    assert!(layout.report_dir.exists());
}

#[test]
fn artifact_bundle_contains_manifest_report_zip_and_reproducibility_schema() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("gradual_degradation".to_string()),
    });

    let bundle = engine.run_single("gradual_degradation").unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    assert!(exported.manifest_path.exists());
    assert!(exported.report_pdf.exists());
    assert!(exported.zip_path.exists());
    assert!(exported.figure_paths.len() >= 24);

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&exported.manifest_path).unwrap()).unwrap();
    assert!(manifest.get("figure_paths").is_some());
    assert!(manifest.get("json_paths").is_some());
    let pdf_bytes = fs::read(&exported.report_pdf).unwrap();
    assert!(pdf_bytes.len() > 1_000_000);

    let reproducibility_summary = exported.run_dir.join("json/reproducibility_summary.json");
    let reproducibility_checks = exported.run_dir.join("json/reproducibility_checks.json");
    assert!(reproducibility_summary.exists());
    assert!(reproducibility_checks.exists());
}

#[test]
fn synthetic_zip_name_and_root_folder_use_synthetic_prefix() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("gradual_degradation".to_string()),
    });

    let bundle = engine.run_single("gradual_degradation").unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let zip_name = exported.zip_path.file_name().unwrap().to_string_lossy();
    assert!(zip_name.starts_with("synthetic-dsfb-semiotics-engine-"));

    let zip_root = exported
        .zip_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let file = std::fs::File::open(&exported.zip_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    assert!(!archive.is_empty());
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        assert!(entry.name().starts_with(&format!("{zip_root}/")));
    }
}

#[test]
fn export_artifacts_removes_stale_known_files_before_rewriting() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("gradual_degradation".to_string()),
    });

    let bundle = engine.run_single("gradual_degradation").unwrap();
    let first = export_artifacts(&bundle).unwrap();
    let stale_figure = first.run_dir.join("figures/stale_figure.tmp");
    let stale_csv = first.run_dir.join("csv/stale_table.csv");
    let stale_json = first.run_dir.join("json/stale_payload.json");
    let stale_report = first.run_dir.join("report/stale_note.md");
    let stale_zip = first.run_dir.join("stale_bundle.zip");
    fs::write(&stale_figure, "stale").unwrap();
    fs::write(&stale_csv, "stale").unwrap();
    fs::write(&stale_json, "stale").unwrap();
    fs::write(&stale_report, "stale").unwrap();
    fs::write(&stale_zip, "stale").unwrap();

    let second = export_artifacts(&bundle).unwrap();
    assert_eq!(first.run_dir, second.run_dir);
    assert!(!stale_figure.exists());
    assert!(!stale_csv.exists());
    assert!(!stale_json.exists());
    assert!(!stale_report.exists());
    assert!(!stale_zip.exists());
    assert!(second.manifest_path.exists());
    assert!(second.report_pdf.exists());
    assert!(second.zip_path.exists());
}

#[test]
fn export_artifacts_refuses_unexpected_root_entries() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("gradual_degradation".to_string()),
    });

    let bundle = engine.run_single("gradual_degradation").unwrap();
    let first = export_artifacts(&bundle).unwrap();
    let foreign_file = first.run_dir.join("foreign.txt");
    fs::write(&foreign_file, "unexpected").unwrap();

    let error = export_artifacts(&bundle).unwrap_err();
    assert!(error.to_string().contains("unexpected file"));
}

#[test]
fn csv_ingest_mode_runs_through_same_pipeline() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(&observed_csv, "time,x,y\n0,1.0,2.0\n1,1.4,2.4\n2,1.9,2.9\n").unwrap();
    fs::write(
        &predicted_csv,
        "time,x,y\n0,0.9,1.9\n1,1.0,2.0\n2,1.1,2.1\n",
    )
    .unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_case".to_string(),
        channel_names: None,
        time_column: None,
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Csv(input.clone()),
    });

    let bundle = engine.run_csv(&input).unwrap();
    let scenario = &bundle.scenario_outputs[0];
    assert_eq!(scenario.record.id, "csv_case");
    assert_eq!(scenario.record.data_origin, "external-csv");
    assert_eq!(scenario.observed.channel_names, vec!["x", "y"]);
    assert_eq!(scenario.residual.samples.len(), 3);
}

#[test]
fn csv_loader_uses_named_time_column_when_requested() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(&observed_csv, "stamp,x\n10.0,1.0\n10.5,1.4\n11.0,1.9\n").unwrap();
    fs::write(&predicted_csv, "stamp,x\n10.0,0.9\n10.5,1.0\n11.0,1.1\n").unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_time_column".to_string(),
        channel_names: None,
        time_column: Some("stamp".to_string()),
        dt_fallback: 0.25,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };

    let (observed, _) = load_csv_trajectories(&input).unwrap();
    assert_eq!(observed.samples[0].time, 10.0);
    assert_eq!(observed.samples[1].time, 10.5);
    assert_eq!(observed.samples[2].time, 11.0);
}

#[test]
fn csv_loader_uses_dt_fallback_when_time_column_is_absent() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(&observed_csv, "x\n1.0\n1.4\n1.9\n").unwrap();
    fs::write(&predicted_csv, "x\n0.9\n1.0\n1.1\n").unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_dt_fallback".to_string(),
        channel_names: None,
        time_column: None,
        dt_fallback: 0.5,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };

    let (observed, _) = load_csv_trajectories(&input).unwrap();
    assert_eq!(observed.samples[0].time, 0.0);
    assert_eq!(observed.samples[1].time, 0.5);
    assert_eq!(observed.samples[2].time, 1.0);
}

#[test]
fn csv_loader_rejects_mismatched_rows() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(&observed_csv, "time,x\n0,1.0\n1,1.4\n2,1.9\n").unwrap();
    fs::write(&predicted_csv, "time,x\n0,0.9\n1,1.0\n").unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_mismatch".to_string(),
        channel_names: None,
        time_column: None,
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };

    let error = load_csv_trajectories(&input).unwrap_err();
    assert!(error.to_string().contains("row counts differ"));
}

#[test]
fn csv_loader_rejects_blank_channel_headers() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(&observed_csv, "time,,y\n0,1.0,2.0\n1,1.4,2.4\n").unwrap();
    fs::write(&predicted_csv, "time,,y\n0,0.9,1.9\n1,1.0,2.0\n").unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_bad_header".to_string(),
        channel_names: None,
        time_column: None,
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };

    let error = load_csv_trajectories(&input).unwrap_err();
    assert!(format!("{error:#}").contains("empty channel header"));
}

#[test]
fn csv_loader_rejects_missing_requested_time_column() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(&observed_csv, "stamp,x\n0.0,1.0\n1.0,1.4\n").unwrap();
    fs::write(&predicted_csv, "stamp,x\n0.0,0.9\n1.0,1.0\n").unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_missing_time".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };

    let error = load_csv_trajectories(&input).unwrap_err();
    assert!(format!("{error:#}").contains("requested time column"));
}

#[test]
fn csv_reproducibility_is_checked_and_identical() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(
        &observed_csv,
        "timestamp,x,y\n0.0,1.0,2.0\n0.5,1.4,2.4\n1.0,1.9,2.9\n",
    )
    .unwrap();
    fs::write(
        &predicted_csv,
        "timestamp,x,y\n0.0,0.9,1.9\n0.5,1.0,2.0\n1.0,1.1,2.1\n",
    )
    .unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_repro".to_string(),
        channel_names: None,
        time_column: Some("timestamp".to_string()),
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Csv(input.clone()),
    });

    let bundle = engine.run_csv(&input).unwrap();
    assert_eq!(bundle.run_metadata.input_mode, "csv");
    assert!(bundle.reproducibility_summary.all_identical);
}

#[test]
fn csv_cli_mode_exposes_external_data_surface() {
    let args = dsfb_semiotics_engine::cli::args::CliArgs::try_parse_from([
        "dsfb-semiotics-engine",
        "--input-mode",
        "csv",
        "--observed-csv",
        "observed.csv",
        "--predicted-csv",
        "predicted.csv",
        "--scenario-id",
        "csv_case",
        "--time-column",
        "timestamp",
        "--channel-names",
        "x,y",
    ])
    .unwrap();

    let selection = args.selection();
    match selection {
        ScenarioSelection::Csv(config) => {
            assert_eq!(config.scenario_id, "csv_case");
            assert_eq!(config.channel_names.unwrap(), vec!["x", "y"]);
            assert_eq!(config.time_column.as_deref(), Some("timestamp"));
        }
        other => panic!("expected CSV selection, got {other:?}"),
    }
}

#[test]
fn bank_mode_alias_exposes_external_bank_surface() {
    let args = dsfb_semiotics_engine::cli::args::CliArgs::try_parse_from([
        "dsfb-semiotics-engine",
        "--bank-mode",
        "external",
        "--bank-path",
        "heuristics.json",
    ])
    .unwrap();

    let bank = args.bank_config();
    match bank.source {
        dsfb_semiotics_engine::engine::config::BankSourceConfig::External(path) => {
            assert_eq!(path, std::path::PathBuf::from("heuristics.json"));
        }
        other => panic!("expected external bank source, got {other:?}"),
    }
}

#[test]
fn exported_report_mentions_projection_and_run_mode_for_csv_runs() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(&observed_csv, "x\n1.0\n1.4\n1.9\n").unwrap();
    fs::write(&predicted_csv, "x\n0.9\n1.0\n1.1\n").unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_report".to_string(),
        channel_names: None,
        time_column: None,
        dt_fallback: 0.5,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 0.5,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Csv(input.clone()),
    });

    let bundle = engine.run_csv(&input).unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let report = fs::read_to_string(exported.report_markdown).unwrap();
    assert!(report.contains("Input mode: `csv`"));
    assert!(report.contains("signed radial drift"));
    assert!(report.contains("Data origin: external-csv"));
}

#[test]
fn exported_report_and_csv_include_semantic_applicability_and_provenance() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 240,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("nominal_stable".to_string()),
    });

    let bundle = engine.run_single("nominal_stable").unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let report = fs::read_to_string(exported.report_markdown).unwrap();
    let semantic_csv =
        fs::read_to_string(exported.run_dir.join("csv/semantic_matches.csv")).unwrap();
    assert!(report.contains(
        "Syntax note: This syntax label is a low-commitment baseline-compatible summary"
    ));
    assert!(report.contains("applicability="));
    assert!(report.contains("provenance="));
    assert!(semantic_csv.contains("candidate_applicability_notes"));
    assert!(semantic_csv.contains("candidate_provenance_notes"));
    assert!(semantic_csv.contains("unknown_reason_detail"));
    assert!(semantic_csv.contains("compatibility_reasons"));
}

#[test]
fn report_explains_mixed_structured_noncommitment_when_semantics_still_match() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 240,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("oscillatory_bounded".to_string()),
    });

    let bundle = engine.run_single("oscillatory_bounded").unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let report = fs::read_to_string(exported.report_markdown).unwrap();
    assert!(report.contains("bounded-oscillatory-structured"));
    assert!(report.contains("bounded oscillatory operation candidate"));
}

#[test]
fn report_keeps_small_nonzero_metric_values_visible() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 180,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("curvature_onset".to_string()),
    });

    let bundle = engine.run_single("curvature_onset").unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let report = fs::read_to_string(exported.report_markdown).unwrap();
    assert!(report.contains("mean_squared_slew_norm="));
    assert!(report.contains("e-"));
}

#[derive(Debug, Deserialize)]
struct SemanticRetrievalSourceRow {
    scenario_id: String,
    heuristic_bank_entry_count: usize,
    heuristic_candidates_post_admissibility: usize,
    heuristic_candidates_post_regime: usize,
    heuristic_candidates_pre_scope: usize,
    heuristic_candidates_post_scope: usize,
    heuristics_rejected_by_admissibility: usize,
    heuristics_rejected_by_regime: usize,
    heuristics_rejected_by_scope: usize,
    heuristics_selected_final: usize,
    disposition_code: i32,
}

#[derive(Debug, Deserialize)]
struct FigureIntegrityCheckRecord {
    figure_id: String,
    nonempty_series: bool,
    nonzero_values_present: bool,
    consistent_with_source: bool,
    source_csv: String,
}

#[test]
fn semantic_retrieval_audit_counts_are_stage_consistent() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 240,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("nominal_stable".to_string()),
    });

    let bundle = engine.run_single("nominal_stable").unwrap();
    let audit = &bundle.scenario_outputs[0].semantics.retrieval_audit;
    assert_eq!(
        audit.heuristic_bank_entry_count,
        audit.heuristic_candidates_post_admissibility + audit.heuristics_rejected_by_admissibility
    );
    assert_eq!(
        audit.heuristic_candidates_post_admissibility,
        audit.heuristic_candidates_post_regime + audit.heuristics_rejected_by_regime
    );
    assert_eq!(
        audit.heuristic_candidates_post_regime,
        audit.heuristic_candidates_pre_scope
    );
    assert_eq!(
        audit.heuristic_candidates_pre_scope,
        audit.heuristic_candidates_post_scope + audit.heuristics_rejected_by_scope
    );
    assert!(audit.heuristics_selected_final <= audit.heuristic_candidates_post_scope);
}

#[test]
fn semantic_retrieval_figure_source_uses_exported_admissibility_counts() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 240,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        bank: dsfb_semiotics_engine::engine::config::BankRunConfig::default(),
        scenario_selection: ScenarioSelection::Single("nominal_stable".to_string()),
    });

    let bundle = engine.run_single("nominal_stable").unwrap();
    let scenario = &bundle.scenario_outputs[0];
    let exported = export_artifacts(&bundle).unwrap();
    let source_rows = fs::read_to_string(
        exported
            .run_dir
            .join("csv/figure_12_semantic_retrieval_source.csv"),
    )
    .unwrap();
    let semantic_matches =
        fs::read_to_string(exported.run_dir.join("csv/semantic_matches.csv")).unwrap();
    let report = fs::read_to_string(exported.report_markdown).unwrap();
    let integrity_rows =
        fs::read_to_string(exported.run_dir.join("json/figure_integrity_checks.json")).unwrap();

    assert!(semantic_matches.contains("heuristic_candidates_post_admissibility"));
    assert!(semantic_matches.contains("heuristics_rejected_by_scope"));
    assert!(report.contains("## Figure Integrity Checks"));
    assert!(report.contains("figure_12_semantic_retrieval_heuristics_bank"));

    let mut reader = csv::Reader::from_reader(source_rows.as_bytes());
    let rows = reader
        .deserialize::<SemanticRetrievalSourceRow>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let nominal = rows
        .iter()
        .find(|row| row.scenario_id == "nominal_stable")
        .unwrap();
    let non_admissible_grammar_count = scenario
        .grammar
        .iter()
        .filter(|status| !matches!(status.state, GrammarState::Admissible))
        .count();

    assert_eq!(
        nominal.heuristic_bank_entry_count,
        scenario
            .semantics
            .retrieval_audit
            .heuristic_bank_entry_count
    );
    assert_eq!(
        nominal.heuristic_candidates_post_admissibility,
        scenario
            .semantics
            .retrieval_audit
            .heuristic_candidates_post_admissibility
    );
    assert_eq!(
        nominal.heuristic_candidates_post_regime,
        scenario
            .semantics
            .retrieval_audit
            .heuristic_candidates_post_regime
    );
    assert_eq!(
        nominal.heuristic_candidates_pre_scope,
        scenario
            .semantics
            .retrieval_audit
            .heuristic_candidates_pre_scope
    );
    assert_eq!(
        nominal.heuristic_candidates_post_scope,
        scenario
            .semantics
            .retrieval_audit
            .heuristic_candidates_post_scope
    );
    assert_eq!(
        nominal.heuristics_rejected_by_admissibility,
        scenario
            .semantics
            .retrieval_audit
            .heuristics_rejected_by_admissibility
    );
    assert_eq!(
        nominal.heuristics_rejected_by_regime,
        scenario
            .semantics
            .retrieval_audit
            .heuristics_rejected_by_regime
    );
    assert_eq!(
        nominal.heuristics_rejected_by_scope,
        scenario
            .semantics
            .retrieval_audit
            .heuristics_rejected_by_scope
    );
    assert_eq!(
        nominal.heuristics_selected_final,
        scenario.semantics.retrieval_audit.heuristics_selected_final
    );
    assert_eq!(non_admissible_grammar_count, 0);
    assert!(nominal.heuristic_candidates_post_admissibility > 0);
    assert!(nominal.disposition_code >= 0);

    let integrity =
        serde_json::from_str::<Vec<FigureIntegrityCheckRecord>>(&integrity_rows).unwrap();
    let figure_12 = integrity
        .iter()
        .find(|check| check.figure_id == "figure_12_semantic_retrieval_heuristics_bank")
        .unwrap();
    assert!(figure_12.nonempty_series);
    assert!(figure_12.nonzero_values_present);
    assert!(figure_12.consistent_with_source);
    assert!(figure_12
        .source_csv
        .ends_with("figure_12_semantic_retrieval_heuristics_bank_source.csv"));
}
