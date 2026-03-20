use std::path::PathBuf;

use clap::CommandFactory;
use dsfb_semiotics_engine::dashboard::{DashboardReplay, DashboardReplayConfig};
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};
use dsfb_semiotics_engine::{CliArgs, ScenarioSelection};

#[test]
fn test_dashboard_replay_mode_initializes() {
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "nominal_stable",
    ));
    let bundle = engine.run_selected().unwrap();
    let replay = DashboardReplay::from_bundle(&bundle, DashboardReplayConfig::default()).unwrap();

    assert_eq!(replay.streams().len(), 1);
    assert!(!replay.streams()[0].events.is_empty());
}

#[test]
fn test_dashboard_help_exposes_mode() {
    let mut command = CliArgs::command();
    let help = command.render_long_help().to_string();

    assert!(help.contains("--dashboard-replay"));
    assert!(help.contains("--dashboard-width"));
    assert!(help.contains("--dashboard-height"));
    assert!(help.contains("--dashboard-max-frames"));
}

#[test]
fn test_dashboard_event_stream_schema_stable() {
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "nominal_stable",
    ));
    let bundle = engine.run_selected().unwrap();
    let replay = DashboardReplay::from_bundle(&bundle, DashboardReplayConfig::default()).unwrap();
    let value = serde_json::to_value(&replay.streams()[0].events[0]).unwrap();

    assert_eq!(
        value["schema_version"],
        "dsfb-semiotics-dashboard-event-stream/v1"
    );
    assert!(value.get("scenario_id").is_some());
    assert!(value.get("residual_norm").is_some());
    assert!(value.get("syntax_label").is_some());
    assert!(value.get("semantic_disposition").is_some());
    assert!(value.get("comparator_alarms").is_some());
}

#[test]
fn test_dashboard_handles_short_trajectory_without_panic() {
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig {
            steps: 2,
            ..Default::default()
        },
        "nominal_stable",
    ));
    let bundle = engine.run_selected().unwrap();
    let replay = DashboardReplay::from_bundle(&bundle, DashboardReplayConfig::default()).unwrap();

    assert!(replay.render_replay_ascii().is_ok());
}

#[test]
fn test_dashboard_handles_csv_replay_fixture_without_panic() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let args = CliArgs {
        all: false,
        scenario: None,
        input_mode: None,
        bank_source: dsfb_semiotics_engine::cli::args::BankSourceArg::Builtin,
        bank_path: None,
        strict_bank_validation: false,
        bank_validation_mode: dsfb_semiotics_engine::cli::args::BankValidationModeArg::Strict,
        sweep_family: None,
        sweep_points: 0,
        observed_csv: Some(crate_root.join("tests/fixtures/observed_fixture.csv")),
        predicted_csv: Some(crate_root.join("tests/fixtures/predicted_fixture.csv")),
        scenario_id: "fixture_csv".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        envelope_mode: dsfb_semiotics_engine::cli::args::EnvelopeModeArg::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "fixture".to_string(),
        output_dir: None,
        seed: 123,
        steps: 240,
        dt: 1.0,
        dashboard_replay: true,
        dashboard_width: 120,
        dashboard_height: 38,
        dashboard_max_frames: 1,
        dashboard_scenario: Some("fixture_csv".to_string()),
    };
    let selection = args.selection();
    let common = CommonRunConfig {
        seed: args.seed,
        steps: args.steps,
        dt: args.dt,
        output_root: None,
        bank: args.bank_config(),
    };
    let config = match selection {
        ScenarioSelection::Csv(input) => EngineConfig::csv(common, input),
        _ => unreachable!(),
    };
    let bundle = StructuralSemioticsEngine::new(config)
        .run_selected()
        .unwrap();
    let replay = DashboardReplay::from_bundle(&bundle, args.dashboard_config()).unwrap();

    assert!(replay.render_replay_ascii().is_ok());
}
