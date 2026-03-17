use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use printpdf::{BuiltinFont, Mm, PdfDocument};
use serde::Serialize;

use crate::config::{
    create_timestamped_run_directory, BenchmarkConfig, ReportArgsPatch, ResolvedCommand, RunConfig,
};
use crate::math::metrics::ScenarioSummary;
use crate::report::csv::write_csv_rows;
use crate::report::json::write_json_pretty;
use crate::report::manifest::{scenario_names, BenchmarkRow, RunManifest};
use crate::report::plotting_data::render_figures;
use crate::sim::runner::{run_scenario, ScenarioRun};
use crate::sim::scenarios::ScenarioDefinition;

pub mod csv;
pub mod json;
pub mod manifest;
pub mod plotting_data;

pub fn run_scenario_bundle(command: ResolvedCommand) -> Result<PathBuf> {
    let (command_name, config) = match command {
        ResolvedCommand::Run(config) => ("run".to_string(), config),
        ResolvedCommand::Quickstart(config) => ("quickstart".to_string(), config),
    };
    let run_dir = create_timestamped_run_directory(&config.output_root)?;
    let scenarios = execute_run_config(&config)?;
    let benchmark_rows = scenarios.iter().map(|scenario| BenchmarkRow::from(&scenario.summary)).collect::<Vec<_>>();
    write_outputs(
        &run_dir.run_dir,
        &command_name,
        &config,
        &scenarios,
        &benchmark_rows,
        scenario_names(&config.scenario.executable_scenarios()),
    )?;
    Ok(run_dir.run_dir)
}

pub fn run_benchmark_suite(config: BenchmarkConfig) -> Result<PathBuf> {
    let run_dir = create_timestamped_run_directory(&config.output_root)?;
    let mut scenarios = Vec::new();
    let mut benchmark_rows = Vec::new();
    for agents in &config.sizes {
        for noise_level in &config.noise_levels {
            for scenario in &config.scenarios {
                let run_config = RunConfig {
                    scenario: *scenario,
                    steps: config.steps,
                    agents: *agents,
                    dt: 0.08,
                    interaction_radius: 1.45,
                    k_neighbors: 4,
                    base_gain: 1.0,
                    noise_level: *noise_level,
                    warmup_steps: (config.steps / 5).max(12),
                    multi_mode: config.multi_mode,
                    monitored_modes: config.monitored_modes,
                    mode_shapes: config.mode_shapes,
                    predictor: config.predictor,
                    trust_mode: config.trust_mode,
                    output_root: config.output_root.clone(),
                    report_pdf: true,
                };
                let scenario_run = run_scenario(&run_config, ScenarioDefinition::from_kind(*scenario, config.steps))?;
                benchmark_rows.push(BenchmarkRow::from(&scenario_run.summary));
                if *agents == config.sizes[0] && (*noise_level - config.noise_levels[0]).abs() < 1.0e-12 {
                    scenarios.push(scenario_run);
                }
            }
        }
    }
    write_outputs(
        &run_dir.run_dir,
        "benchmark",
        &config,
        &scenarios,
        &benchmark_rows,
        config.scenarios.iter().map(|scenario| scenario.as_str().to_string()).collect(),
    )?;
    Ok(run_dir.run_dir)
}

pub fn generate_report_for_existing_run(patch: ReportArgsPatch) -> Result<PathBuf> {
    let run_dir = match patch.run_dir {
        Some(path) => path,
        None => latest_run_dir(&patch.output_root)?,
    };
    let report_dir = run_dir.join("report");
    fs::create_dir_all(&report_dir)
        .with_context(|| format!("failed to create {}", report_dir.display()))?;
    let manifest_path = run_dir.join("manifest.json");
    let markdown = format!(
        "# dsfb-swarm report regeneration\n\nThis report bundle was regenerated for `{}`.\n\nManifest path: `{}`\n\nThe primary figures and tabular artifacts remain in the sibling directories `figures/` and the run root CSV/JSON exports.\n",
        run_dir.display(),
        manifest_path.display()
    );
    fs::write(report_dir.join("dsfb_swarm_report.md"), &markdown)
        .with_context(|| format!("failed to write regenerated markdown into {}", report_dir.display()))?;
    write_compact_pdf(
        &report_dir.join("dsfb_swarm_report.pdf"),
        &[
            "dsfb-swarm report regeneration".to_string(),
            format!("run directory: {}", run_dir.display()),
            format!("manifest: {}", manifest_path.display()),
            "See figures/*.png and CSV/JSON exports for the full empirical bundle.".to_string(),
        ],
    )?;
    Ok(run_dir)
}

fn execute_run_config(config: &RunConfig) -> Result<Vec<ScenarioRun>> {
    let scenarios = config
        .scenario
        .executable_scenarios()
        .into_iter()
        .map(|kind| run_scenario(config, ScenarioDefinition::from_kind(kind, config.steps)))
        .collect::<Result<Vec<_>>>()?;
    Ok(scenarios)
}

fn write_outputs<T>(
    run_dir: &Path,
    command_name: &str,
    config: &T,
    scenarios: &[ScenarioRun],
    benchmark_rows: &[BenchmarkRow],
    scenario_kinds: Vec<String>,
) -> Result<()>
where
    T: Serialize,
{
    let figures_dir = run_dir.join("figures");
    let report_dir = run_dir.join("report");
    fs::create_dir_all(&figures_dir)
        .with_context(|| format!("failed to create {}", figures_dir.display()))?;
    fs::create_dir_all(&report_dir)
        .with_context(|| format!("failed to create {}", report_dir.display()))?;

    render_figures(&figures_dir, scenarios, benchmark_rows)?;

    let summaries = scenarios.iter().map(|scenario| scenario.summary.clone()).collect::<Vec<_>>();
    let time_series = scenarios
        .iter()
        .flat_map(|scenario| scenario.time_series.clone())
        .collect::<Vec<_>>();
    let spectra = scenarios
        .iter()
        .flat_map(|scenario| scenario.spectra.clone())
        .collect::<Vec<_>>();
    let residuals = scenarios
        .iter()
        .flat_map(|scenario| scenario.residuals.clone())
        .collect::<Vec<_>>();
    let trust = scenarios
        .iter()
        .flat_map(|scenario| scenario.trust.clone())
        .collect::<Vec<_>>();
    let baselines = scenarios
        .iter()
        .flat_map(|scenario| scenario.baselines.clone())
        .collect::<Vec<_>>();
    let anomalies = scenarios
        .iter()
        .flat_map(|scenario| scenario.anomalies.clone())
        .collect::<Vec<_>>();

    write_json_pretty(&run_dir.join("run_config.json"), config)?;
    write_csv_rows(&run_dir.join("scenarios_summary.csv"), summaries.iter().cloned())?;
    write_csv_rows(&run_dir.join("benchmark_summary.csv"), benchmark_rows.iter().cloned())?;
    write_csv_rows(&run_dir.join("time_series.csv"), time_series.iter().cloned())?;
    write_csv_rows(&run_dir.join("spectra.csv"), spectra.iter().cloned())?;
    write_csv_rows(&run_dir.join("residuals.csv"), residuals.iter().cloned())?;
    write_csv_rows(&run_dir.join("trust.csv"), trust.iter().cloned())?;
    write_csv_rows(&run_dir.join("baselines.csv"), baselines.iter().cloned())?;
    write_json_pretty(&run_dir.join("anomalies.json"), &anomalies)?;

    for scenario in scenarios {
        write_csv_rows(
            &run_dir.join(format!("scenario_{}_metrics.csv", scenario.definition.name)),
            std::iter::once(scenario.summary.clone()),
        )?;
        write_csv_rows(
            &run_dir.join(format!("scenario_{}_timeseries.csv", scenario.definition.name)),
            scenario.time_series.iter().cloned(),
        )?;
    }

    let report_markdown = build_markdown_report(command_name, run_dir, &summaries, benchmark_rows);
    fs::write(report_dir.join("dsfb_swarm_report.md"), &report_markdown)
        .with_context(|| format!("failed to write report markdown under {}", report_dir.display()))?;
    write_compact_pdf(
        &report_dir.join("dsfb_swarm_report.pdf"),
        &build_pdf_lines(run_dir, &summaries, benchmark_rows),
    )?;

    let manifest = RunManifest {
        crate_name: "dsfb-swarm",
        crate_version: env!("CARGO_PKG_VERSION"),
        command: command_name.to_string(),
        timestamp: run_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string(),
        scenario_kinds,
        artifact_inventory: vec![
            "manifest.json".to_string(),
            "run_config.json".to_string(),
            "scenarios_summary.csv".to_string(),
            "benchmark_summary.csv".to_string(),
            "time_series.csv".to_string(),
            "spectra.csv".to_string(),
            "residuals.csv".to_string(),
            "trust.csv".to_string(),
            "baselines.csv".to_string(),
            "anomalies.json".to_string(),
            "figures/lambda2_timeseries.png".to_string(),
            "figures/residual_timeseries.png".to_string(),
            "figures/drift_slew.png".to_string(),
            "figures/trust_evolution.png".to_string(),
            "figures/baseline_comparison.png".to_string(),
            "figures/scaling_curves.png".to_string(),
            "figures/noise_stress_curves.png".to_string(),
            "figures/multimode_comparison.png".to_string(),
            "figures/topology_snapshots.png".to_string(),
            "report/dsfb_swarm_report.md".to_string(),
            "report/dsfb_swarm_report.pdf".to_string(),
        ],
    };
    write_json_pretty(&run_dir.join("manifest.json"), &manifest)?;
    Ok(())
}

fn latest_run_dir(output_root: &Path) -> Result<PathBuf> {
    let mut entries = fs::read_dir(output_root)
        .with_context(|| format!("failed to read {}", output_root.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();
    entries.pop().ok_or_else(|| anyhow::anyhow!("no run directories found under {}", output_root.display()))
}

fn build_markdown_report(
    command_name: &str,
    run_dir: &Path,
    summaries: &[ScenarioSummary],
    benchmark_rows: &[BenchmarkRow],
) -> String {
    let mut body = String::new();
    body.push_str("# DSFB-Swarm empirical report\n\n");
    body.push_str(&format!("Command: `{command_name}`\n\n"));
    body.push_str(&format!("Run directory: `{}`\n\n", run_dir.display()));
    body.push_str("## Mathematical framing\n\n");
    body.push_str(
        "The demonstrator evolves a dynamic interaction graph `G(t)` with adjacency `A(t)`, degree `D(t)`, and Laplacian `L(t) = D(t) - A(t)`. The monitored observables are the Laplacian eigenvalues, especially `lambda_2(t)`, together with deterministic predictors `hat lambda_k(t)`, residuals `r_k(t) = lambda_k(t) - hat lambda_k(t)`, residual drift, residual slew, residual envelopes, and trust-gated interaction attenuation.\n\n",
    );
    body.push_str("## Scenario summary\n\n");
    body.push_str("| scenario | visible failure | scalar detect | multi detect | scalar lead (s) | multi lead (s) | multi advantage (s) | trust delay (s) | corr(score, ||Delta L||_F) |\n");
    body.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for summary in summaries {
        body.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {:.4} |\n",
            summary.scenario,
            display_option_usize(summary.visible_failure_step),
            display_option_usize(summary.scalar_detection_step),
            display_option_usize(summary.multimode_detection_step),
            display_option(summary.scalar_detection_lead_time),
            display_option(summary.multimode_detection_lead_time),
            display_option(summary.multimode_minus_scalar_seconds),
            display_option(summary.trust_suppression_delay),
            summary.residual_topology_correlation
        ));
    }
    body.push_str("\n## Benchmark summary\n\n");
    body.push_str("| scenario | agents | noise | scalar lead (s) | multi lead (s) | baseline lambda2 lead (s) | multi advantage (s) | scalar TPR | scalar FPR | multi TPR | multi FPR | trust delay (s) |\n");
    body.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in benchmark_rows.iter().take(24) {
        body.push_str(&format!(
            "| {} | {} | {:.3} | {} | {} | {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {} |\n",
            row.scenario,
            row.agents,
            row.noise_level,
            display_option(row.scalar_detection_lead_time),
            display_option(row.multimode_detection_lead_time),
            display_option(row.baseline_lambda2_lead_time),
            display_option(row.multimode_minus_scalar_seconds),
            row.scalar_true_positive_rate,
            row.scalar_false_positive_rate,
            row.multimode_true_positive_rate,
            row.multimode_false_positive_rate,
            display_option(row.trust_suppression_delay)
        ));
    }
    body.push_str("\n## Figure inventory\n\n");
    for figure in [
        "figures/lambda2_timeseries.png",
        "figures/residual_timeseries.png",
        "figures/drift_slew.png",
        "figures/trust_evolution.png",
        "figures/baseline_comparison.png",
        "figures/scaling_curves.png",
        "figures/noise_stress_curves.png",
        "figures/multimode_comparison.png",
        "figures/topology_snapshots.png",
    ] {
        body.push_str(&format!("- `{figure}`\n"));
    }
    body.push_str("\n## Observed findings\n\n");
    for line in observed_findings(summaries) {
        body.push_str(&format!("- {line}\n"));
    }
    body
}

fn build_pdf_lines(run_dir: &Path, summaries: &[ScenarioSummary], benchmark_rows: &[BenchmarkRow]) -> Vec<String> {
    let mut lines = vec![
        "DSFB-Swarm compact PDF report".to_string(),
        format!("run directory: {}", run_dir.display()),
        "core observables: lambda_2(t), r_lambda(t), drift, slew, envelopes, trust".to_string(),
        String::new(),
        "scenario summaries:".to_string(),
    ];
    for summary in summaries {
        lines.push(format!(
            "{} | fail={} | scalar={} | multi={} | scalar_lead={} | multi_lead={}",
            summary.scenario,
            display_option_usize(summary.visible_failure_step),
            display_option_usize(summary.scalar_detection_step),
            display_option_usize(summary.multimode_detection_step),
            display_option(summary.scalar_detection_lead_time),
            display_option(summary.multimode_detection_lead_time),
        ));
    }
    lines.push(String::new());
    lines.push("benchmark snapshot:".to_string());
    for row in benchmark_rows.iter().take(12) {
        lines.push(format!(
            "{} | N={} | noise={:.3} | scalar lead={} | multi advantage={} | trust delay={}",
            row.scenario,
            row.agents,
            row.noise_level,
            display_option(row.scalar_detection_lead_time),
            display_option(row.multimode_minus_scalar_seconds),
            display_option(row.trust_suppression_delay),
        ));
    }
    lines.push(String::new());
    lines.push("see report markdown and figures/*.png for the full artifact bundle.".to_string());
    lines
}

fn write_compact_pdf(path: &Path, lines: &[String]) -> Result<()> {
    let (document, page1, layer1) =
        PdfDocument::new("dsfb-swarm report", Mm(210.0), Mm(297.0), "Layer 1");
    let font = document.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_size = 8.5;
    let left_margin_mm = 14.0;
    let top_margin_mm = 282.0;
    let bottom_margin_mm = 16.0;
    let line_step_mm = 5.2;
    let max_chars = 88;

    let mut current_layer = document.get_page(page1).get_layer(layer1);
    let mut y = top_margin_mm;
    let mut page_index = 1usize;
    for source_line in lines {
        for line in wrap_text(source_line, max_chars) {
            if y < bottom_margin_mm {
                page_index += 1;
                let layer_name = format!("Layer {page_index}");
                let (page, layer) = document.add_page(Mm(210.0), Mm(297.0), layer_name);
                current_layer = document.get_page(page).get_layer(layer);
                y = top_margin_mm;
            }
            current_layer.use_text(line, font_size, Mm(left_margin_mm), Mm(y), &font);
            y -= line_step_mm;
        }
    }
    document
        .save(&mut BufWriter::new(
            File::create(path).with_context(|| format!("failed to create {}", path.display()))?,
        ))
        .with_context(|| format!("failed to save {}", path.display()))
}

fn wrap_text(line: &str, max_chars: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    let mut wrapped = Vec::new();
    let mut current = String::new();
    for token in line.split_whitespace() {
        if token.len() > max_chars {
            if !current.is_empty() {
                wrapped.push(std::mem::take(&mut current));
            }
            let mut start = 0;
            while start < token.len() {
                let end = (start + max_chars).min(token.len());
                wrapped.push(token[start..end].to_string());
                start = end;
            }
            continue;
        }

        let next_len = if current.is_empty() {
            token.len()
        } else {
            current.len() + 1 + token.len()
        };
        if next_len > max_chars && !current.is_empty() {
            wrapped.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(token);
    }
    if !current.is_empty() {
        wrapped.push(current);
    }
    if wrapped.is_empty() {
        wrapped.push(String::new());
    }
    wrapped
}

fn display_option(value: Option<f64>) -> String {
    value
        .map(|number| format!("{number:.3}"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn display_option_usize(value: Option<usize>) -> String {
    value
        .map(|number| number.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn observed_findings(summaries: &[ScenarioSummary]) -> Vec<String> {
    let mut findings = Vec::new();
    if let Some(summary) = summaries.iter().find(|summary| summary.scenario == "communication_loss") {
        findings.push(format!(
            "communication_loss reached visible failure at step {} with scalar detection at step {} and lead time {} s; raw lambda2 thresholding lagged to {} s in the representative run.",
            display_option_usize(summary.visible_failure_step),
            display_option_usize(summary.scalar_detection_step),
            display_option(summary.scalar_detection_lead_time),
            display_option(summary.baseline_lambda2_lead_time),
        ));
    }
    if let Some(summary) = summaries
        .iter()
        .find(|summary| summary.scenario == "gradual_edge_degradation")
    {
        findings.push(format!(
            "gradual_edge_degradation produced scalar detection at step {} and multi-mode detection at step {}, with lead times {} s and {} s before the visibility threshold.",
            display_option_usize(summary.scalar_detection_step),
            display_option_usize(summary.multimode_detection_step),
            display_option(summary.scalar_detection_lead_time),
            display_option(summary.multimode_detection_lead_time),
        ));
    }
    if let Some(summary) = summaries.iter().find(|summary| summary.scenario == "adversarial_agent") {
        findings.push(format!(
            "adversarial_agent showed multi-mode advantage of {} s over scalar-only detection and trust suppression delay {} s; visible failure remained scenario-dependent in the representative run.",
            display_option(summary.multimode_minus_scalar_seconds),
            display_option(summary.trust_suppression_delay),
        ));
    }
    if let Some(summary) = summaries.iter().find(|summary| summary.scenario == "nominal") {
        findings.push(format!(
            "nominal remained bounded with scalar FPR {:.3} and multi-mode FPR {:.3} in the representative run.",
            summary.scalar_false_positive_rate,
            summary.multimode_false_positive_rate,
        ));
    }
    if findings.is_empty() {
        findings.push("No scenario summaries were available.".to_string());
    }
    findings
}
