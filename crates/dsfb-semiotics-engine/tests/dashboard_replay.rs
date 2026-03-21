use std::fs;
use std::path::{Path, PathBuf};

use clap::CommandFactory;
use dsfb_semiotics_engine::cli::args::{
    BankSourceArg, BankValidationModeArg, CliArgs, CsvInputConfig, EnvelopeModeArg,
    ScenarioSelection,
};
use dsfb_semiotics_engine::dashboard::{
    CsvReplayDriver, CsvReplayRunState, DashboardReplay, DashboardReplayConfig,
    CSV_REPLAY_STATE_SCHEMA_VERSION,
};
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use tempfile::tempdir;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn csv_fixture() -> CsvInputConfig {
    CsvInputConfig {
        observed_csv: crate_root().join("tests/fixtures/observed_fixture.csv"),
        predicted_csv: crate_root().join("tests/fixtures/predicted_fixture.csv"),
        scenario_id: "fixture_csv".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 0.5,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "fixture_envelope".to_string(),
    }
}

fn csv_replay_config() -> DashboardReplayConfig {
    DashboardReplayConfig {
        max_frames: Some(4),
        source_label: Some("fixture_csv_pair".to_string()),
        ..Default::default()
    }
}

fn write_csv_pair(
    dir: &Path,
    observed_rows: &[(usize, f64, &[f64])],
    predicted_rows: &[(usize, f64, &[f64])],
) -> (PathBuf, PathBuf) {
    let observed = dir.join("observed.csv");
    let predicted = dir.join("predicted.csv");
    let header = "step,time,ax,ay\n";

    let observed_body = observed_rows
        .iter()
        .map(|(step, time, values)| format!("{step},{time},{},{}\n", values[0], values[1]))
        .collect::<String>();
    let predicted_body = predicted_rows
        .iter()
        .map(|(step, time, values)| format!("{step},{time},{},{}\n", values[0], values[1]))
        .collect::<String>();

    fs::write(&observed, format!("{header}{observed_body}")).unwrap();
    fs::write(&predicted, format!("{header}{predicted_body}")).unwrap();
    (observed, predicted)
}

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
fn test_dashboard_help_mentions_dashboard_csv_replay() {
    let mut command = CliArgs::command();
    let help = command.render_long_help().to_string();

    assert!(help.contains("--dashboard-replay-csv"));
    assert!(help.contains("--dashboard-playback-speed"));
    assert!(help.contains("--dashboard-start-paused"));
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
    assert!(value.get("event_markers").is_some());
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
    let args = CliArgs {
        all: false,
        scenario: None,
        input_mode: None,
        bank_source: BankSourceArg::Builtin,
        bank_path: None,
        strict_bank_validation: false,
        bank_validation_mode: BankValidationModeArg::Strict,
        sweep_family: None,
        sweep_points: 0,
        observed_csv: Some(crate_root().join("tests/fixtures/observed_fixture.csv")),
        predicted_csv: Some(crate_root().join("tests/fixtures/predicted_fixture.csv")),
        scenario_id: "fixture_csv".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        envelope_mode: EnvelopeModeArg::Fixed,
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
        dashboard_replay: false,
        dashboard_replay_csv: true,
        dashboard_width: 120,
        dashboard_height: 38,
        dashboard_max_frames: 2,
        dashboard_playback_speed: 1.0,
        dashboard_start_paused: false,
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
        ScenarioSelection::Csv(input) => EngineConfig::csv(common.clone(), input.clone()),
        _ => unreachable!(),
    };
    let bundle = StructuralSemioticsEngine::new(config)
        .run_selected()
        .unwrap();
    let ScenarioSelection::Csv(input) = args.selection() else {
        unreachable!()
    };
    let replay = CsvReplayDriver::from_bundle_and_csv_input(
        &bundle,
        &common,
        &input,
        &EngineSettings::default(),
        args.dashboard_config(),
    )
    .unwrap();

    assert!(replay
        .render_current_frame_ascii()
        .contains("REPLAY MODE: CSV"));
}

#[test]
fn test_dashboard_csv_replay_mode_initializes() {
    let driver = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        csv_fixture(),
        EngineSettings::default(),
        csv_replay_config(),
    )
    .unwrap();

    assert_eq!(driver.stream().input_mode, "csv");
    assert_eq!(
        driver.timing_state().schema_version,
        CSV_REPLAY_STATE_SCHEMA_VERSION
    );
    assert_eq!(driver.timing_state().current_frame_index, 0);
    assert!(driver
        .render_current_frame_ascii()
        .contains("Playback Speed"));
}

#[test]
fn test_dashboard_csv_replay_pause_resume() {
    let mut driver = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        csv_fixture(),
        EngineSettings::default(),
        csv_replay_config(),
    )
    .unwrap();

    assert_eq!(driver.timing_state().run_state, CsvReplayRunState::Running);
    driver.pause();
    assert!(driver.is_paused());
    assert_eq!(driver.timing_state().run_state, CsvReplayRunState::Paused);
    driver.resume();
    assert!(!driver.is_paused());
    assert_eq!(driver.timing_state().run_state, CsvReplayRunState::Running);
}

#[test]
fn test_dashboard_csv_replay_single_step() {
    let mut config = csv_replay_config();
    config.start_paused = true;
    let mut driver = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        csv_fixture(),
        EngineSettings::default(),
        config,
    )
    .unwrap();

    assert_eq!(driver.timing_state().current_frame_index, 0);
    assert!(driver.single_step());
    assert_eq!(driver.timing_state().current_frame_index, 1);
    assert!(driver.is_paused());
}

#[test]
fn test_dashboard_csv_replay_rate_control() {
    let mut fast = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        csv_fixture(),
        EngineSettings::default(),
        csv_replay_config(),
    )
    .unwrap();
    fast.set_playback_speed(4.0).unwrap();

    let mut slow = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        csv_fixture(),
        EngineSettings::default(),
        csv_replay_config(),
    )
    .unwrap();
    slow.set_playback_speed(0.25).unwrap();

    assert!(fast.advance(0.5).unwrap() >= 1);
    assert_eq!(slow.advance(0.5).unwrap(), 0);
    assert!(fast.timing_state().current_frame_index > slow.timing_state().current_frame_index);
}

#[test]
fn test_dashboard_csv_replay_end_of_stream_behavior() {
    let mut driver = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        csv_fixture(),
        EngineSettings::default(),
        DashboardReplayConfig {
            max_frames: Some(3),
            ..csv_replay_config()
        },
    )
    .unwrap();

    while !driver.is_ended() {
        let _ = driver.advance(1.0).unwrap();
        if driver.timing_state().current_frame_index + 1 >= driver.stream().events.len() {
            break;
        }
    }
    let _ = driver.advance(10.0).unwrap();

    assert!(
        driver.is_ended()
            || driver.timing_state().current_frame_index + 1 >= driver.stream().events.len()
    );
}

#[test]
fn test_dashboard_csv_replay_handles_short_fixture_without_panic() {
    let temp = tempdir().unwrap();
    let (observed, predicted) = write_csv_pair(
        temp.path(),
        &[(0, 0.0, &[1.0, 0.5])],
        &[(0, 0.0, &[0.98, 0.49])],
    );
    let driver = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        CsvInputConfig {
            observed_csv: observed,
            predicted_csv: predicted,
            scenario_id: "short_csv".to_string(),
            channel_names: None,
            time_column: Some("time".to_string()),
            dt_fallback: 0.5,
            envelope_mode: EnvelopeMode::Fixed,
            envelope_base: 1.0,
            envelope_slope: 0.0,
            envelope_switch_step: None,
            envelope_secondary_slope: None,
            envelope_secondary_base: None,
            envelope_name: "short".to_string(),
        },
        EngineSettings::default(),
        DashboardReplayConfig {
            max_frames: Some(1),
            ..csv_replay_config()
        },
    )
    .unwrap();

    assert!(driver
        .render_current_frame_ascii()
        .contains("REPLAY MODE: CSV"));
}

#[test]
fn test_dashboard_csv_replay_uses_deterministic_timing_state() {
    let mut driver_a = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        csv_fixture(),
        EngineSettings::default(),
        csv_replay_config(),
    )
    .unwrap();
    let mut driver_b = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        csv_fixture(),
        EngineSettings::default(),
        csv_replay_config(),
    )
    .unwrap();

    driver_a.advance(1.0).unwrap();
    driver_b.advance(1.0).unwrap();

    assert_eq!(
        serde_json::to_value(driver_a.timing_state()).unwrap(),
        serde_json::to_value(driver_b.timing_state()).unwrap()
    );
}

#[test]
fn test_dashboard_event_markers_emitted_for_state_changes() {
    let temp = tempdir().unwrap();
    let (observed, predicted) = write_csv_pair(
        temp.path(),
        &[
            (0, 0.0, &[1.0, 0.5]),
            (1, 0.5, &[1.2, 0.6]),
            (2, 1.0, &[1.7, 0.85]),
            (3, 1.5, &[2.1, 1.05]),
        ],
        &[
            (0, 0.0, &[0.98, 0.49]),
            (1, 0.5, &[1.0, 0.50]),
            (2, 1.0, &[1.05, 0.52]),
            (3, 1.5, &[1.08, 0.54]),
        ],
    );
    let mut driver = CsvReplayDriver::from_csv_run(
        CommonRunConfig::default(),
        CsvInputConfig {
            observed_csv: observed,
            predicted_csv: predicted,
            scenario_id: "marker_csv".to_string(),
            channel_names: None,
            time_column: Some("time".to_string()),
            dt_fallback: 0.5,
            envelope_mode: EnvelopeMode::Fixed,
            envelope_base: 0.55,
            envelope_slope: 0.0,
            envelope_switch_step: None,
            envelope_secondary_slope: None,
            envelope_secondary_base: None,
            envelope_name: "marker".to_string(),
        },
        EngineSettings::default(),
        csv_replay_config(),
    )
    .unwrap();

    let _ = driver.advance(2.0).unwrap();

    assert!(!driver.emitted_markers().is_empty());
    assert!(driver
        .emitted_markers()
        .iter()
        .any(|marker| marker.contains("grammar") || marker.contains("trust threshold")));
}
