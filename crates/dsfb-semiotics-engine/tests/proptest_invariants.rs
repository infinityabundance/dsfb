use std::path::PathBuf;

use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::engine::bank::{HeuristicBankMetadata, HeuristicBankRegistry};
use dsfb_semiotics_engine::engine::grammar_layer::evaluate_grammar_layer;
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::syntax_layer::characterize_syntax_with_coordination_configured;
use dsfb_semiotics_engine::engine::types::{
    AdmissibilityRequirement, EnvelopeMode, GrammarState, HeuristicBankEntry, HeuristicProvenance,
    HeuristicScopeConditions, ResidualSample, ResidualTrajectory,
};
use dsfb_semiotics_engine::io::input::load_csv_trajectories;
use dsfb_semiotics_engine::io::schema::HEURISTIC_BANK_SCHEMA_VERSION;
use dsfb_semiotics_engine::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use dsfb_semiotics_engine::math::envelope::{build_envelope, EnvelopeSpec};
use dsfb_semiotics_engine::math::metrics::{
    euclidean_norm, hash_serializable_hex, project_sign, residual_norm_path_monotonicity,
    scalar_derivative, sign_with_deadband, trend_aligned_increment_fraction,
};
use dsfb_semiotics_engine::sim::scenarios::all_scenarios;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed, TestRunner};

const PROPTEST_SEED: u64 = 0xD5FB_5EED_2026_0320;
const SMOKE_CASES: u32 = 64;
const RESEARCH_CASES: u32 = 256;
const HIGH_RISK_CASES: u32 = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PropertyBudget {
    Smoke,
    Research,
    Stress,
}

fn property_budget() -> PropertyBudget {
    match std::env::var("DSFB_PROPTEST_MODE")
        .unwrap_or_else(|_| "research".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "smoke" => PropertyBudget::Smoke,
        "stress" => PropertyBudget::Stress,
        _ => PropertyBudget::Research,
    }
}

fn standard_cases() -> u32 {
    match property_budget() {
        PropertyBudget::Smoke => SMOKE_CASES,
        PropertyBudget::Research | PropertyBudget::Stress => RESEARCH_CASES,
    }
}

fn high_risk_cases() -> u32 {
    match property_budget() {
        PropertyBudget::Smoke => SMOKE_CASES,
        PropertyBudget::Research | PropertyBudget::Stress => HIGH_RISK_CASES,
    }
}

fn deterministic_runner(cases: u32) -> TestRunner {
    TestRunner::new(Config {
        cases,
        failure_persistence: None,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(PROPTEST_SEED),
        ..Config::default()
    })
}

fn increasing_times(len: usize, increments: &[u16]) -> Vec<f64> {
    let mut time = 0.0;
    let mut times = Vec::with_capacity(len.max(1));
    times.push(time);
    for increment in increments.iter().take(len.saturating_sub(1)) {
        time += f64::from(*increment) * 0.25;
        times.push(time);
    }
    times
}

fn residual_trajectory_from_values(
    scenario_id: &str,
    values: &[f64],
    times: &[f64],
) -> ResidualTrajectory {
    ResidualTrajectory {
        scenario_id: scenario_id.to_string(),
        channel_names: vec!["x".to_string()],
        samples: values
            .iter()
            .enumerate()
            .map(|(step, value)| ResidualSample {
                step,
                time: times[step],
                values: vec![*value],
                norm: value.abs(),
            })
            .collect(),
    }
}

fn fixed_envelope(
    residual: &ResidualTrajectory,
    radius: f64,
) -> dsfb_semiotics_engine::engine::types::AdmissibilityEnvelope {
    build_envelope(
        residual,
        &EnvelopeSpec {
            name: "proptest".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: radius,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        &residual.scenario_id,
    )
}

fn syntax_from_values(
    values: &[f64],
    times: &[f64],
) -> dsfb_semiotics_engine::engine::types::SyntaxCharacterization {
    let residual = residual_trajectory_from_values("proptest_syntax", values, times);
    let drift = compute_drift_trajectory(&residual, 1.0, "proptest_syntax");
    let slew = compute_slew_trajectory(&residual, 1.0, "proptest_syntax");
    let sign = dsfb_semiotics_engine::engine::sign_layer::construct_signs(&residual, &drift, &slew);
    let envelope = fixed_envelope(&residual, 10.0);
    let grammar = evaluate_grammar_layer(&residual, &envelope);
    characterize_syntax_with_coordination_configured(
        &sign,
        &grammar,
        None,
        &EngineSettings::default().syntax,
    )
}

fn minimal_scope() -> HeuristicScopeConditions {
    HeuristicScopeConditions {
        min_outward_drift_fraction: None,
        max_outward_drift_fraction: None,
        min_inward_drift_fraction: None,
        max_inward_drift_fraction: None,
        max_curvature_energy: None,
        min_curvature_energy: None,
        max_curvature_onset_score: None,
        min_curvature_onset_score: None,
        min_directional_persistence: None,
        min_sign_consistency: None,
        min_channel_coherence: None,
        min_aggregate_monotonicity: None,
        max_aggregate_monotonicity: None,
        min_slew_spike_count: None,
        max_slew_spike_count: None,
        min_slew_spike_strength: None,
        max_slew_spike_strength: None,
        min_boundary_grazing_episodes: None,
        max_boundary_grazing_episodes: None,
        min_boundary_recovery_count: None,
        min_coordinated_group_breach_fraction: None,
        max_coordinated_group_breach_fraction: None,
        require_group_breach: false,
    }
}

fn valid_registry_with_links(compatible: bool, incompatible: bool) -> HeuristicBankRegistry {
    let mut left = HeuristicBankEntry {
        heuristic_id: "H-A".to_string(),
        motif_label: "a".to_string(),
        short_label: "a".to_string(),
        scope_conditions: minimal_scope(),
        admissibility_requirements: AdmissibilityRequirement::Any,
        regime_tags: vec!["fixed".to_string()],
        provenance: HeuristicProvenance {
            source: "generated".to_string(),
            note: "generated".to_string(),
        },
        applicability_note: "generated".to_string(),
        retrieval_priority: 1,
        compatible_with: Vec::new(),
        incompatible_with: Vec::new(),
        directional_incompatibility_exceptions: Vec::new(),
    };
    let mut right = left.clone();
    right.heuristic_id = "H-B".to_string();
    right.motif_label = "b".to_string();
    right.short_label = "b".to_string();

    if compatible {
        left.compatible_with.push("H-B".to_string());
        right.compatible_with.push("H-A".to_string());
    }
    if incompatible {
        left.incompatible_with.push("H-B".to_string());
        right.incompatible_with.push("H-A".to_string());
    }

    HeuristicBankRegistry {
        metadata: HeuristicBankMetadata {
            schema_version: HEURISTIC_BANK_SCHEMA_VERSION.to_string(),
            bank_version: "generated/v1".to_string(),
            note: "generated for property validation".to_string(),
        },
        entries: vec![left, right],
    }
}

fn csv_config(scenario_id: &str, dt_fallback: f64) -> CsvInputConfig {
    CsvInputConfig {
        observed_csv: PathBuf::from("observed.csv"),
        predicted_csv: PathBuf::from("predicted.csv"),
        scenario_id: scenario_id.to_string(),
        channel_names: None,
        time_column: None,
        dt_fallback,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "fixture".to_string(),
    }
}

#[test]
fn prop_norm_nonnegative() {
    let strategy =
        proptest::collection::vec(proptest::collection::vec(-500i32..=500, 1..=6), 1..=8);
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |rows| {
            for row in rows {
                let values = row
                    .into_iter()
                    .map(|value| f64::from(value) * 1.0e-3)
                    .collect::<Vec<_>>();
                let norm = euclidean_norm(&values);
                prop_assert!(norm.is_finite());
                prop_assert!(norm >= 0.0);
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_projected_sign_finite() {
    let strategy = (1usize..=6).prop_flat_map(|len| {
        (
            proptest::collection::vec(-25i32..=25, len),
            proptest::collection::vec(-50i32..=50, len),
            proptest::collection::vec(-50i32..=50, len),
        )
    });
    let mut runner = deterministic_runner(high_risk_cases());
    runner
        .run(&strategy, |(residual_raw, drift_raw, slew_raw)| {
            let residual = residual_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-6)
                .collect::<Vec<_>>();
            let drift = drift_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-3)
                .collect::<Vec<_>>();
            let slew = slew_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-3)
                .collect::<Vec<_>>();
            let projection = project_sign(&residual, &drift, &slew);

            prop_assert!(projection.iter().all(|value| value.is_finite()));
            prop_assert!(projection[0] >= 0.0);
            prop_assert!(projection[2] >= 0.0);
            Ok(())
        })
        .unwrap();
}

#[test]
// TRACE:CLAIM:CLM-TEST-CONSTANT-DRIFT-ZERO:Executable constant-drift evidence:Property test confirms constant scalar paths yield zero drift.
fn prop_constant_signal_zero_drift() {
    let strategy = (1usize..=10, -500i32..=500).prop_flat_map(|(len, value_raw)| {
        proptest::collection::vec(1u16..=8, len.saturating_sub(1)).prop_map(move |increments| {
            let times = increasing_times(len, &increments);
            (times, f64::from(value_raw) * 1.0e-2)
        })
    });
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |(times, value)| {
            let values = vec![value; times.len()];
            let residual = residual_trajectory_from_values("constant", &values, &times);
            let drift = compute_drift_trajectory(&residual, 1.0, "constant");
            let derivative = scalar_derivative(&values, &times);

            prop_assert!(derivative.iter().all(|item| item.abs() <= 1.0e-12));
            prop_assert!(drift
                .samples
                .iter()
                .all(|sample| sample.norm.abs() <= 1.0e-12));
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_linear_or_affine_signal_constant_drift() {
    let strategy = (2usize..=10, -200i32..=200, -100i32..=100).prop_flat_map(
        |(len, intercept_raw, slope_raw)| {
            proptest::collection::vec(1u16..=8, len - 1).prop_map(move |increments| {
                let times = increasing_times(len, &increments);
                let intercept = f64::from(intercept_raw) * 0.1;
                let slope = f64::from(slope_raw) * 0.05;
                (times, intercept, slope)
            })
        },
    );
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |(times, intercept, slope)| {
            let values = times
                .iter()
                .map(|time| intercept + slope * *time)
                .collect::<Vec<_>>();
            let derivative = scalar_derivative(&values, &times);

            prop_assert!(derivative.iter().all(|value| value.is_finite()));
            prop_assert!(derivative
                .iter()
                .all(|value| (*value - slope).abs() <= 1.0e-9));
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_affine_signal_zero_slew() {
    let strategy = (3usize..=10, -200i32..=200, -100i32..=100).prop_flat_map(
        |(len, intercept_raw, slope_raw)| {
            proptest::collection::vec(1u16..=8, len - 1).prop_map(move |increments| {
                let times = increasing_times(len, &increments);
                let intercept = f64::from(intercept_raw) * 0.1;
                let slope = f64::from(slope_raw) * 0.05;
                (times, intercept, slope)
            })
        },
    );
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |(times, intercept, slope)| {
            let values = times
                .iter()
                .map(|time| intercept + slope * *time)
                .collect::<Vec<_>>();
            let residual = residual_trajectory_from_values("affine", &values, &times);
            let slew = compute_slew_trajectory(&residual, 1.0, "affine");

            prop_assert!(slew.samples.iter().all(|sample| sample.norm.is_finite()));
            prop_assert!(slew
                .samples
                .iter()
                .all(|sample| sample.norm.abs() <= 1.0e-10));
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_monotonicity_metrics_bounded() {
    let strategy = proptest::collection::vec(-500i32..=500, 1..=16);
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |values_raw| {
            let values = values_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-2)
                .collect::<Vec<_>>();
            let path_monotonicity = residual_norm_path_monotonicity(&values);
            let trend_alignment = trend_aligned_increment_fraction(&values, 1.0e-9);

            prop_assert!(path_monotonicity.is_finite());
            prop_assert!(trend_alignment.is_finite());
            prop_assert!((0.0..=1.0).contains(&path_monotonicity));
            prop_assert!((0.0..=1.0).contains(&trend_alignment));
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_outward_inward_fractions_bounded() {
    let strategy = (3usize..=12).prop_flat_map(|len| {
        (
            proptest::collection::vec(-200i32..=200, len),
            proptest::collection::vec(1u16..=8, len - 1),
        )
    });
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |(values_raw, increments)| {
            let times = increasing_times(values_raw.len(), &increments);
            let values = values_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-2)
                .collect::<Vec<_>>();
            let syntax = syntax_from_values(&values, &times);

            prop_assert!(syntax.outward_drift_fraction.is_finite());
            prop_assert!(syntax.inward_drift_fraction.is_finite());
            prop_assert!((0.0..=1.0).contains(&syntax.outward_drift_fraction));
            prop_assert!((0.0..=1.0).contains(&syntax.inward_drift_fraction));
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_no_nan_or_inf_in_exported_metrics() {
    let strategy = (3usize..=12).prop_flat_map(|len| {
        (
            proptest::collection::vec(-50i32..=50, len),
            proptest::collection::vec(1u16..=8, len - 1),
        )
    });
    let mut runner = deterministic_runner(high_risk_cases());
    runner
        .run(&strategy, |(values_raw, increments)| {
            let times = increasing_times(values_raw.len(), &increments);
            let values = values_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-4)
                .collect::<Vec<_>>();
            let syntax = syntax_from_values(&values, &times);
            let serialized = serde_json::to_string(&syntax).unwrap();

            prop_assert!(!serialized.contains("NaN"));
            prop_assert!(!serialized.contains("inf"));
            prop_assert!(!serialized.contains("null"));
            Ok(())
        })
        .unwrap();
}

#[test]
// TRACE:CLAIM:CLM-TEST-REPRODUCIBILITY-HASH:Executable reproducibility evidence:Property test confirms identical inputs keep deterministic layered hashes stable.
fn prop_reproducibility_hash_stable_for_identical_inputs() {
    let strategy = (1usize..=4).prop_flat_map(|outer_len| {
        proptest::collection::vec(proptest::collection::vec(-200i32..=200, 0..=6), outer_len)
    });
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |nested_raw| {
            let nested = nested_raw
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|value| f64::from(value) * 1.0e-3)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let first = hash_serializable_hex("materialized", &nested).unwrap();
            let second = hash_serializable_hex("materialized", &nested).unwrap();

            prop_assert_eq!(first.fnv1a_64_hex, second.fnv1a_64_hex);
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_csv_validation_invariants() {
    let strategy = (0u16..=4, prop::bool::ANY);
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |(dt_raw, empty_id)| {
            let scenario_id = if empty_id { "" } else { "csv_case" };
            let config = csv_config(scenario_id, f64::from(dt_raw) * 0.5);
            let valid = config.validate().is_ok();

            prop_assert_eq!(valid, !empty_id && dt_raw > 0);
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_bank_graph_reference_consistency_for_valid_generated_cases() {
    let strategy = (prop::bool::ANY, prop::bool::ANY).prop_filter(
        "compatibility and incompatibility cannot both be enabled for the same pair",
        |(compatible, incompatible)| !(*compatible && *incompatible),
    );
    let mut runner = deterministic_runner(standard_cases());
    runner
        .run(&strategy, |(compatible, incompatible)| {
            let report = valid_registry_with_links(compatible, incompatible).validation_report();

            prop_assert!(report.duplicate_ids.is_empty());
            prop_assert!(report.unknown_link_targets.is_empty());
            prop_assert!(report.compatibility_conflicts.is_empty());
            prop_assert!(report.missing_compatibility_links.is_empty());
            prop_assert!(report.missing_incompatibility_links.is_empty());
            prop_assert!(report.valid);
            Ok(())
        })
        .unwrap();
}

#[test]
fn prop_boundary_transition_well_defined_near_threshold() {
    let strategy = (960i32..=990, 990i32..=1040, 1000i32..=1080);
    let mut runner = deterministic_runner(high_risk_cases());
    runner
        .run(&strategy, |(below_raw, edge_raw, above_raw)| {
            let mut values = vec![
                f64::from(below_raw) * 1.0e-3,
                f64::from(edge_raw) * 1.0e-3,
                f64::from(above_raw) * 1.0e-3,
            ];
            values.sort_by(f64::total_cmp);
            let times = [0.0, 1.0, 2.0];
            let residual = residual_trajectory_from_values("boundary", &values, &times);
            let grammar = evaluate_grammar_layer(&residual, &fixed_envelope(&residual, 1.0));
            let severities = grammar
                .iter()
                .map(|status| match status.state {
                    GrammarState::Admissible => 0,
                    GrammarState::Boundary => 1,
                    GrammarState::Violation => 2,
                })
                .collect::<Vec<_>>();

            prop_assert!(grammar.iter().all(|status| status.margin.is_finite()));
            prop_assert!(grammar.iter().all(|status| status.radius.is_finite()));
            prop_assert!(severities.windows(2).all(|window| window[0] <= window[1]));
            prop_assert_ne!(grammar[0].state, GrammarState::Violation);
            prop_assert_ne!(grammar[2].state, GrammarState::Admissible);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_property_case_budget_defaults_are_research_grade() {
    assert_eq!(standard_cases(), RESEARCH_CASES);
    assert_eq!(high_risk_cases(), HIGH_RISK_CASES);
}

#[test]
fn test_proptest_seed_is_fixed_and_documented() {
    assert_eq!(PROPTEST_SEED, 0xD5FB_5EED_2026_0320);
}

#[test]
fn test_sign_deadband_smoke_symmetry() {
    for deadband in [1.0e-9, 1.0e-6, 1.0e-3] {
        for value in [
            -10.0,
            -1.0e-2,
            -deadband,
            -deadband / 2.0,
            0.0,
            deadband / 2.0,
            deadband,
            1.0e-2,
            10.0,
        ] {
            let sign = sign_with_deadband(value, deadband);
            let mirrored = sign_with_deadband(-value, deadband);
            assert!(matches!(sign, -1..=1));
            assert_eq!(sign, -mirrored);
        }
    }
}

#[test]
fn test_property_fixture_bank_roundtrip_is_stable() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/external_bank_minimal.json");
    let first = HeuristicBankRegistry::load_external_json(path.as_path(), true).unwrap();
    let second = HeuristicBankRegistry::load_external_json(path.as_path(), true).unwrap();

    assert_eq!(first.1.content_hash, second.1.content_hash);
    assert!(first.2.valid);
    assert!(second.2.valid);
}

#[test]
fn test_property_csv_loader_fixture_stays_deterministic() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let observed_csv = crate_root.join("tests/fixtures/observed_fixture.csv");
    let predicted_csv = crate_root.join("tests/fixtures/predicted_fixture.csv");
    let config = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "fixture_csv".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "fixture".to_string(),
    };
    let first = load_csv_trajectories(&config).unwrap();
    let second = load_csv_trajectories(&config).unwrap();

    assert_eq!(first.0.samples.len(), second.0.samples.len());
    assert_eq!(first.1.samples.len(), second.1.samples.len());
}

#[test]
fn test_property_scenario_catalog_still_contains_reference_cases() {
    let ids = all_scenarios()
        .into_iter()
        .map(|scenario| scenario.record.id)
        .collect::<Vec<_>>();
    assert!(ids.iter().any(|id| id == "oscillatory_bounded"));
    assert!(ids.iter().any(|id| id == "noisy_structured"));
}
