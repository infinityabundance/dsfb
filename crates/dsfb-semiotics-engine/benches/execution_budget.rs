use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::dashboard::{CsvReplayDriver, DashboardReplayConfig};
use dsfb_semiotics_engine::engine::bank::HeuristicBankRegistry;
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};
use dsfb_semiotics_engine::engine::semantics::retrieve_semantics_with_registry;
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::{EnvelopeMode, GrammarStatus, SyntaxCharacterization};
use dsfb_semiotics_engine::live::{to_real, OnlineStructuralEngine};
use dsfb_semiotics_engine::math::envelope::EnvelopeSpec;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn csv_fixture() -> CsvInputConfig {
    CsvInputConfig {
        observed_csv: fixture_path("observed_fixture.csv"),
        predicted_csv: fixture_path("predicted_fixture.csv"),
        scenario_id: "bench_csv".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 0.5,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "bench_envelope".to_string(),
    }
}

fn benchmark_live_step(c: &mut Criterion) {
    c.bench_function("online_engine_step", |bench| {
        bench.iter_batched(
            || {
                OnlineStructuralEngine::with_builtin_bank(
                    "bench_online",
                    vec!["residual".to_string()],
                    1.0,
                    EnvelopeSpec {
                        name: "bench".to_string(),
                        mode: EnvelopeMode::Fixed,
                        base_radius: 1.0,
                        slope: 0.0,
                        switch_step: None,
                        secondary_slope: None,
                        secondary_base: None,
                    },
                    EngineSettings::default(),
                )
                .unwrap()
            },
            |mut engine| {
                for index in 0..31 {
                    let _ = engine
                        .push_residual_sample(index as f64, &[to_real(index as f64 * 0.01)])
                        .unwrap();
                }
                let _ = engine.push_residual_sample(31.0, &[to_real(0.42)]).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

fn benchmark_csv_replay_step(c: &mut Criterion) {
    c.bench_function("csv_replay_step", |bench| {
        bench.iter_batched(
            || {
                CsvReplayDriver::from_csv_run(
                    CommonRunConfig::default(),
                    csv_fixture(),
                    EngineSettings::default(),
                    DashboardReplayConfig {
                        max_frames: Some(4),
                        ..Default::default()
                    },
                )
                .unwrap()
            },
            |mut driver| {
                let _ = driver.advance(0.5).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

fn benchmark_semantic_retrieval(c: &mut Criterion) {
    let bundle = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "gradual_degradation",
    ))
    .run_selected()
    .unwrap();
    let scenario = &bundle.scenario_outputs[0];
    let syntax = scenario.syntax.clone();
    let grammar = scenario.grammar.clone();
    let coordinated = scenario.coordinated.clone();

    let small_registry = HeuristicBankRegistry::builtin();
    let large_registry = enlarged_registry(&small_registry, 128);

    c.bench_function("semantic_retrieval_builtin_bank", |bench| {
        bench.iter(|| {
            let _ = retrieval_call(
                "bench_small",
                &syntax,
                &grammar,
                coordinated.as_ref(),
                &small_registry,
            );
        });
    });

    c.bench_function("semantic_retrieval_enlarged_bank", |bench| {
        bench.iter(|| {
            let _ = retrieval_call(
                "bench_large",
                &syntax,
                &grammar,
                coordinated.as_ref(),
                &large_registry,
            );
        });
    });
}

fn retrieval_call(
    scenario_id: &str,
    syntax: &SyntaxCharacterization,
    grammar: &[GrammarStatus],
    coordinated: Option<&dsfb_semiotics_engine::CoordinatedResidualStructure>,
    registry: &HeuristicBankRegistry,
) -> dsfb_semiotics_engine::engine::types::SemanticMatchResult {
    retrieve_semantics_with_registry(
        scenario_id,
        syntax,
        grammar,
        coordinated,
        registry,
        &dsfb_semiotics_engine::engine::settings::SemanticRetrievalSettings::default(),
    )
}

fn enlarged_registry(
    registry: &HeuristicBankRegistry,
    target_entries: usize,
) -> HeuristicBankRegistry {
    let mut clone = registry.clone();
    let original = registry.entries.clone();
    let mut index = 0usize;
    while clone.entries.len() < target_entries {
        let mut entry = original[index % original.len()].clone();
        entry.heuristic_id = format!("{}-BENCH-{}", entry.heuristic_id, index);
        entry.compatible_with.clear();
        entry.incompatible_with.clear();
        entry.directional_incompatibility_exceptions.clear();
        clone.entries.push(entry);
        index += 1;
    }
    clone
}

criterion_group!(
    execution_budget,
    benchmark_live_step,
    benchmark_csv_replay_step,
    benchmark_semantic_retrieval
);
criterion_main!(execution_budget);
