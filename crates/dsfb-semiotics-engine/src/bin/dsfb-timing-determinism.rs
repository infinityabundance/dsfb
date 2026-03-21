#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use anyhow::Result;
use clap::{Parser, ValueHint};
use serde::Serialize;

use dsfb_semiotics_engine::engine::bank::HeuristicBankRegistry;
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::grammar_layer::evaluate_grammar_layer;
use dsfb_semiotics_engine::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};
use dsfb_semiotics_engine::engine::semantics::retrieve_semantics_with_registry;
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::{
    EnvelopeMode, GrammarStatus, ResidualSample, ResidualTrajectory, SyntaxCharacterization,
};
use dsfb_semiotics_engine::live::{numeric_mode_label, to_real, OnlineStructuralEngine};
use dsfb_semiotics_engine::math::envelope::{build_envelope, EnvelopeSpec};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Generate a host-side timing determinism report for bounded live steps, grammar evaluation, and semantic retrieval"
)]
struct TimingArgs {
    #[arg(long, value_hint = ValueHint::FilePath, default_value = "docs/TIMING_DETERMINISM_REPORT.md")]
    output_md: PathBuf,

    #[arg(long, value_hint = ValueHint::FilePath, default_value = "docs/timing_determinism_report.json")]
    output_json: PathBuf,

    #[arg(long, default_value_t = 400)]
    iterations: usize,

    #[arg(long, default_value_t = 32)]
    warmup: usize,
}

#[derive(Clone, Debug, Serialize)]
struct TimingMetricSummary {
    name: String,
    iterations: usize,
    mean_ns: u128,
    median_ns: u128,
    p95_ns: u128,
    p99_ns: u128,
    p999_ns: u128,
    max_ns: u128,
    jitter_ns: u128,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct TimingDeterminismReport {
    schema_version: String,
    platform: String,
    rust_version: String,
    numeric_mode: String,
    iterations: usize,
    warmup: usize,
    metrics: Vec<TimingMetricSummary>,
    note: String,
}

fn main() -> Result<()> {
    let args = TimingArgs::parse();
    let report = generate_report(args.iterations, args.warmup)?;
    if let Some(parent) = args.output_md.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = args.output_json.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.output_md, render_markdown(&report))?;
    fs::write(&args.output_json, serde_json::to_vec_pretty(&report)?)?;
    println!("timing_report_md={}", args.output_md.display());
    println!("timing_report_json={}", args.output_json.display());
    Ok(())
}

fn generate_report(iterations: usize, warmup: usize) -> Result<TimingDeterminismReport> {
    Ok(TimingDeterminismReport {
        schema_version: "dsfb-semiotics-timing-determinism/v1".to_string(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        rust_version: rust_version(),
        numeric_mode: numeric_mode_label().to_string(),
        iterations,
        warmup,
        metrics: vec![
            summarize("scalar_push_sample", measure_scalar_push(iterations, warmup)?, "Observed bounded live scalar step on the current host after warmup."),
            summarize("batch_push_sample", measure_batch_push(iterations, warmup)?, "Observed bounded live batch step on the current host after warmup."),
            summarize("grammar_admissible_path", measure_grammar_path(iterations, warmup, false), "Observed grammar evaluation on an admissible fixture."),
            summarize("grammar_violation_path", measure_grammar_path(iterations, warmup, true), "Observed grammar evaluation on a violation-like fixture."),
            summarize("semantic_retrieval_builtin_bank", measure_semantic_retrieval(iterations, warmup, false)?, "Observed semantic retrieval on the builtin bank."),
            summarize("semantic_retrieval_enlarged_bank", measure_semantic_retrieval(iterations, warmup, true)?, "Observed semantic retrieval on an enlarged synthetic bank."),
        ],
        note: "These are host-side observed timing summaries, not certified WCET bounds. Median and tail metrics describe the measured platform only.".to_string(),
    })
}

fn rust_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "rustc version unavailable".to_string())
}

fn summarize(name: &str, samples: Vec<u128>, note: &str) -> TimingMetricSummary {
    let mut sorted = samples;
    sorted.sort_unstable();
    let mean_ns = if sorted.is_empty() {
        0
    } else {
        sorted.iter().sum::<u128>() / sorted.len() as u128
    };
    let median_ns = percentile(&sorted, 0.50);
    let p95_ns = percentile(&sorted, 0.95);
    let p99_ns = percentile(&sorted, 0.99);
    let p999_ns = percentile(&sorted, 0.999);
    let max_ns = sorted.last().copied().unwrap_or_default();
    TimingMetricSummary {
        name: name.to_string(),
        iterations: sorted.len(),
        mean_ns,
        median_ns,
        p95_ns,
        p99_ns,
        p999_ns,
        max_ns,
        jitter_ns: max_ns.saturating_sub(median_ns),
        note: note.to_string(),
    }
}

fn percentile(sorted: &[u128], quantile: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() - 1) as f64 * quantile).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn measure_scalar_push(iterations: usize, warmup: usize) -> Result<Vec<u128>> {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "timing_scalar",
        vec!["residual".to_string()],
        1.0,
        EnvelopeSpec {
            name: "timing".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        EngineSettings::default(),
    )?;
    let mut samples = Vec::with_capacity(iterations);
    for step in 0..(warmup + iterations) {
        let started = Instant::now();
        let _ = engine.push_residual_sample(step as f64, &[to_real(step as f64 * 0.01)])?;
        let elapsed = started.elapsed().as_nanos();
        if step >= warmup {
            samples.push(elapsed);
        }
    }
    Ok(samples)
}

fn measure_batch_push(iterations: usize, warmup: usize) -> Result<Vec<u128>> {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "timing_batch",
        vec!["x".to_string(), "y".to_string()],
        1.0,
        EnvelopeSpec {
            name: "timing".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        EngineSettings::default(),
    )?;
    let mut samples = Vec::with_capacity(iterations);
    for batch in 0..(warmup + iterations) {
        let times = [batch as f64, batch as f64 + 0.5, batch as f64 + 1.0];
        let values = [
            to_real(0.10 + batch as f64 * 0.001),
            to_real(0.01),
            to_real(0.12 + batch as f64 * 0.001),
            to_real(0.015),
            to_real(0.14 + batch as f64 * 0.001),
            to_real(0.02),
        ];
        let started = Instant::now();
        let _ = engine.push_residual_sample_batch(&times, &values)?;
        let elapsed = started.elapsed().as_nanos();
        if batch >= warmup {
            samples.push(elapsed);
        }
    }
    Ok(samples)
}

fn measure_grammar_path(iterations: usize, warmup: usize, violating: bool) -> Vec<u128> {
    let base = if violating {
        vec![0.1, 0.9, 1.3, 1.4]
    } else {
        vec![0.1, 0.3, 0.4, 0.5]
    };
    let residual = ResidualTrajectory {
        scenario_id: if violating {
            "timing_violation".to_string()
        } else {
            "timing_admissible".to_string()
        },
        channel_names: vec!["x".to_string()],
        samples: base
            .into_iter()
            .enumerate()
            .map(|(step, value)| ResidualSample {
                step,
                time: step as f64,
                values: vec![value],
                norm: value.abs(),
            })
            .collect(),
    };
    let envelope = build_envelope(
        &residual,
        &EnvelopeSpec {
            name: "timing".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        &residual.scenario_id,
    );
    let mut samples = Vec::with_capacity(iterations);
    for step in 0..(warmup + iterations) {
        let started = Instant::now();
        let _ = evaluate_grammar_layer(&residual, &envelope);
        let elapsed = started.elapsed().as_nanos();
        if step >= warmup {
            samples.push(elapsed);
        }
    }
    samples
}

fn measure_semantic_retrieval(
    iterations: usize,
    warmup: usize,
    enlarged: bool,
) -> Result<Vec<u128>> {
    let bundle = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "gradual_degradation",
    ))
    .run_selected()?;
    let scenario = &bundle.scenario_outputs[0];
    let registry = if enlarged {
        enlarged_registry(&HeuristicBankRegistry::builtin(), 128)
    } else {
        HeuristicBankRegistry::builtin()
    };
    let mut samples = Vec::with_capacity(iterations);
    for step in 0..(warmup + iterations) {
        let started = Instant::now();
        retrieval_call(&scenario.syntax, &scenario.grammar, &registry);
        let elapsed = started.elapsed().as_nanos();
        if step >= warmup {
            samples.push(elapsed);
        }
    }
    Ok(samples)
}

fn retrieval_call(
    syntax: &SyntaxCharacterization,
    grammar: &[GrammarStatus],
    registry: &HeuristicBankRegistry,
) {
    let _ = retrieve_semantics_with_registry(
        "timing_semantics",
        syntax,
        grammar,
        None,
        registry,
        &dsfb_semiotics_engine::engine::settings::SemanticRetrievalSettings::default(),
    );
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
        entry.heuristic_id = format!("{}-TIMING-{}", entry.heuristic_id, index);
        entry.compatible_with.clear();
        entry.incompatible_with.clear();
        entry.directional_incompatibility_exceptions.clear();
        clone.entries.push(entry);
        index += 1;
    }
    clone
}

fn render_markdown(report: &TimingDeterminismReport) -> String {
    let mut lines = Vec::new();
    lines.push("# Timing Determinism Report".to_string());
    lines.push(String::new());
    lines.push(format!("Schema: `{}`", report.schema_version));
    lines.push(format!("Platform: `{}`", report.platform));
    lines.push(format!("Rust: `{}`", report.rust_version));
    lines.push(format!("Numeric mode: `{}`", report.numeric_mode));
    lines.push(format!(
        "Iterations: `{}` measured after `{}` warmup iterations",
        report.iterations, report.warmup
    ));
    lines.push(String::new());
    lines.push("This report records observed host-side timing behavior. It is not a certified WCET analysis.".to_string());
    lines.push(String::new());
    lines.push("| Measurement | Mean (ns) | Median (ns) | p95 (ns) | p99 (ns) | p99.9 (ns) | Max (ns) | Jitter (ns) |".to_string());
    lines.push("|-------------|-----------|-------------|----------|----------|------------|----------|-------------|".to_string());
    for metric in &report.metrics {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            metric.name,
            metric.mean_ns,
            metric.median_ns,
            metric.p95_ns,
            metric.p99_ns,
            metric.p999_ns,
            metric.max_ns,
            metric.jitter_ns
        ));
    }
    lines.push(String::new());
    lines.push("## Notes".to_string());
    lines.push(String::new());
    for metric in &report.metrics {
        lines.push(format!("- `{}`: {}", metric.name, metric.note));
    }
    lines.push(format!("- {}", report.note));
    lines.join("\n")
}
