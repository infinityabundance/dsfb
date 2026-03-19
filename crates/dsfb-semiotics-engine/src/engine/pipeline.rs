use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::cli::args::ScenarioSelection;
use crate::engine::grammar_layer::{evaluate_detectability, evaluate_grammar_layer};
use crate::engine::residual_layer::extract_residuals;
use crate::engine::semantics_layer::retrieve_semantics;
use crate::engine::sign_layer::construct_signs;
use crate::engine::syntax_layer::characterize_syntax;
use crate::engine::types::{
    CoordinatedResidualStructure, EngineOutputBundle, FigureArtifact, GroupResidualPoint,
    ReproducibilityCheck, ResidualTrajectory, RunMetadata, ScenarioOutput, SemanticMatchResult,
};
use crate::figures::plots::render_all_figures;
use crate::io::csv::write_rows;
use crate::io::json::write_pretty;
use crate::io::output::{create_output_layout, OutputLayout};
use crate::io::zip::zip_directory;
use crate::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use crate::math::envelope::build_envelope;
use crate::math::metrics::{fnv1a_hex, max_abs, pairwise_abs_mean};
use crate::report::artifact_report::build_markdown_report;
use crate::report::pdf::write_text_pdf;
use crate::sim::generators::{synthesize, ScenarioSynthesis};
use crate::sim::scenarios::{all_scenarios, ScenarioDefinition};

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub seed: u64,
    pub steps: usize,
    pub dt: f64,
    pub output_root: Option<PathBuf>,
    pub scenario_selection: ScenarioSelection,
}

#[derive(Clone, Debug)]
pub struct StructuralSemioticsEngine {
    config: EngineConfig,
}

#[derive(Clone, Debug)]
pub struct ExportedArtifacts {
    pub run_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub report_markdown: PathBuf,
    pub report_pdf: PathBuf,
    pub zip_path: PathBuf,
    pub figure_paths: Vec<PathBuf>,
}

pub fn run_all_demos(config: EngineConfig) -> Result<EngineOutputBundle> {
    StructuralSemioticsEngine::new(config).run_all()
}

pub fn run_scenario(config: EngineConfig, scenario_id: &str) -> Result<EngineOutputBundle> {
    StructuralSemioticsEngine::new(config).run_single(scenario_id)
}

pub fn export_artifacts(bundle: &EngineOutputBundle) -> Result<ExportedArtifacts> {
    let layout = OutputLayout {
        timestamp: bundle.run_metadata.timestamp.clone(),
        run_dir: bundle.run_dir.clone(),
        figures_dir: bundle.run_dir.join("figures"),
        csv_dir: bundle.run_dir.join("csv"),
        json_dir: bundle.run_dir.join("json"),
        report_dir: bundle.run_dir.join("report"),
    };

    let figure_artifacts = render_all_figures(bundle, &layout.figures_dir)?;
    write_tabular_artifacts(bundle, &layout)?;

    let manifest_path = layout.run_dir.join("manifest.json");
    let report_markdown_path = layout.report_dir.join("dsfb_semiotics_engine_report.md");
    let report_pdf_path = layout.report_dir.join("dsfb_semiotics_engine_report.pdf");
    let zip_path = layout.run_dir.join(format!(
        "dsfb-semiotics-engine-{}.zip",
        bundle.run_metadata.timestamp
    ));

    let report_manifest = crate::engine::types::ReportManifest {
        crate_name: bundle.run_metadata.crate_name.clone(),
        crate_version: bundle.run_metadata.crate_version.clone(),
        timestamp: bundle.run_metadata.timestamp.clone(),
        run_dir: layout.run_dir.display().to_string(),
        report_markdown: report_markdown_path.display().to_string(),
        report_pdf: report_pdf_path.display().to_string(),
        zip_archive: zip_path.display().to_string(),
        figure_paths: figure_artifacts
            .iter()
            .flat_map(|figure| [figure.png_path.clone(), figure.svg_path.clone()])
            .collect(),
        csv_paths: collect_relative_files(&layout.csv_dir)?,
        json_paths: collect_relative_files(&layout.json_dir)?,
        scenario_ids: bundle
            .scenario_outputs
            .iter()
            .map(|scenario| scenario.record.id.clone())
            .collect(),
        notes: vec![
            "Synthetic theorem-aligned demonstrations only.".to_string(),
            "Deterministic heuristic retrieval is conservative and auditable.".to_string(),
        ],
    };

    let markdown = build_markdown_report(bundle, &figure_artifacts, &report_manifest);
    std::fs::write(&report_markdown_path, &markdown)
        .with_context(|| format!("failed to write {}", report_markdown_path.display()))?;
    write_text_pdf(
        &report_pdf_path,
        "dsfb-semiotics-engine report",
        &markdown
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>(),
    )?;

    write_pretty(&manifest_path, &report_manifest)?;
    zip_directory(&layout.run_dir, &zip_path)?;

    Ok(ExportedArtifacts {
        run_dir: layout.run_dir,
        manifest_path,
        report_markdown: report_markdown_path,
        report_pdf: report_pdf_path,
        zip_path,
        figure_paths: figure_artifacts
            .iter()
            .flat_map(|figure| {
                [
                    PathBuf::from(&figure.png_path),
                    PathBuf::from(&figure.svg_path),
                ]
            })
            .collect(),
    })
}

impl StructuralSemioticsEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    pub fn run_all(&self) -> Result<EngineOutputBundle> {
        self.execute(&all_scenarios())
    }

    pub fn run_single(&self, scenario_id: &str) -> Result<EngineOutputBundle> {
        let all = all_scenarios();
        let selected = all
            .into_iter()
            .find(|scenario| scenario.record.id == scenario_id)
            .ok_or_else(|| anyhow!("unknown scenario `{scenario_id}`"))?;
        self.execute(&[selected])
    }

    fn execute(&self, definitions: &[ScenarioDefinition]) -> Result<EngineOutputBundle> {
        let output_root = self
            .config
            .output_root
            .clone()
            .unwrap_or_else(default_output_root);
        let layout = create_output_layout(&output_root)?;

        let synthesized = definitions
            .iter()
            .map(|definition| {
                synthesize(
                    definition,
                    self.config.steps,
                    self.config.dt,
                    self.config.seed,
                )
            })
            .collect::<Vec<_>>();

        let base_outputs = definitions
            .iter()
            .zip(&synthesized)
            .map(|(definition, synthesized)| self.build_partial_output(definition, synthesized))
            .collect::<Result<Vec<_>>>()?;

        let residual_lookup = base_outputs
            .iter()
            .map(|output| (output.record.id.clone(), output.residual.clone()))
            .collect::<BTreeMap<_, _>>();

        let scenario_outputs = base_outputs
            .into_iter()
            .map(|mut output| {
                let reference = reference_for(&output.record.id)
                    .and_then(|reference_id| residual_lookup.get(reference_id));
                output.detectability = evaluate_detectability(
                    &output.residual,
                    &output.grammar,
                    bound_inputs_for(definitions, &output.record.id),
                    reference,
                );
                output.semantics = retrieve_semantics(
                    &output.record.id,
                    &output.syntax,
                    &output.grammar,
                    output.coordinated.as_ref(),
                );
                output
            })
            .collect::<Vec<_>>();

        let reproducibility_check =
            self.compute_reproducibility(definitions.first().context("no scenarios selected")?)?;

        Ok(EngineOutputBundle {
            run_metadata: run_metadata(
                &layout.timestamp,
                self.config.seed,
                self.config.steps,
                self.config.dt,
            ),
            run_dir: layout.run_dir,
            scenario_outputs,
            figure_artifacts: Vec::<FigureArtifact>::new(),
            reproducibility_check,
            report_manifest: None,
            tabular_inventory: BTreeMap::new(),
        })
    }

    fn build_partial_output(
        &self,
        definition: &ScenarioDefinition,
        synthesized: &ScenarioSynthesis,
    ) -> Result<ScenarioOutput> {
        let residual = extract_residuals(
            &synthesized.observed,
            &synthesized.predicted,
            &definition.record.id,
        );
        let drift = compute_drift_trajectory(&residual, self.config.dt, &definition.record.id);
        let slew = compute_slew_trajectory(&residual, self.config.dt, &definition.record.id);
        let sign = construct_signs(&residual, &drift, &slew);
        let envelope = build_envelope(&residual, &definition.envelope_spec, &definition.record.id);
        let grammar = evaluate_grammar_layer(&residual, &envelope);
        let syntax = characterize_syntax(&sign, &grammar);
        let coordinated = build_coordinated(definition, &residual)?;

        Ok(ScenarioOutput {
            record: synthesized.record.clone(),
            observed: synthesized.observed.clone(),
            predicted: synthesized.predicted.clone(),
            residual,
            drift,
            slew,
            sign,
            envelope,
            grammar,
            syntax,
            detectability: crate::engine::types::DetectabilityResult {
                scenario_id: definition.record.id.clone(),
                observed_crossing_step: None,
                observed_crossing_time: None,
                predicted_upper_bound: None,
                bound_satisfied: None,
                separation_at_exit: None,
                note: "Pending reference attachment.".to_string(),
            },
            semantics: SemanticMatchResult {
                scenario_id: definition.record.id.clone(),
                disposition: crate::engine::types::SemanticDisposition::Unknown,
                motif_summary: "Pending semantic retrieval.".to_string(),
                candidates: Vec::new(),
                selected_labels: Vec::new(),
                note: "Pending semantic retrieval.".to_string(),
            },
            coordinated,
        })
    }

    fn compute_reproducibility(
        &self,
        definition: &ScenarioDefinition,
    ) -> Result<ReproducibilityCheck> {
        let first = self.materialize_hash(definition)?;
        let second = self.materialize_hash(definition)?;
        Ok(ReproducibilityCheck {
            scenario_id: definition.record.id.clone(),
            first_hash: first.clone(),
            second_hash: second.clone(),
            identical: first == second,
            note: "The same deterministic scenario was synthesized twice with identical configuration and hashed over residual, drift, and slew trajectories.".to_string(),
        })
    }

    fn materialize_hash(&self, definition: &ScenarioDefinition) -> Result<String> {
        let synthesized = synthesize(
            definition,
            self.config.steps,
            self.config.dt,
            self.config.seed,
        );
        let residual = extract_residuals(
            &synthesized.observed,
            &synthesized.predicted,
            &definition.record.id,
        );
        let drift = compute_drift_trajectory(&residual, self.config.dt, &definition.record.id);
        let slew = compute_slew_trajectory(&residual, self.config.dt, &definition.record.id);
        Ok(fnv1a_hex(
            &definition.record.id,
            &[
                residual.samples.iter().map(|sample| sample.norm).collect(),
                drift.samples.iter().map(|sample| sample.norm).collect(),
                slew.samples.iter().map(|sample| sample.norm).collect(),
            ],
        )
        .fnv1a_64_hex)
    }
}

fn build_coordinated(
    definition: &ScenarioDefinition,
    residual: &ResidualTrajectory,
) -> Result<Option<CoordinatedResidualStructure>> {
    if definition.groups.is_empty() {
        return Ok(None);
    }
    let aggregate_spec = definition
        .aggregate_envelope_spec
        .as_ref()
        .context("grouped scenario missing aggregate envelope")?;

    let mut points = Vec::new();
    for group in &definition.groups {
        for sample in &residual.samples {
            let member_values = group
                .member_indices
                .iter()
                .filter_map(|index| sample.values.get(*index).copied())
                .collect::<Vec<_>>();
            let aggregate_abs_mean = pairwise_abs_mean(&member_values);
            let local_max_abs = max_abs(&member_values);
            let (aggregate_radius, _, _) = aggregate_spec.radius_at(sample.step, sample.time);
            points.push(GroupResidualPoint {
                scenario_id: residual.scenario_id.clone(),
                group_id: group.group_id.clone(),
                step: sample.step,
                time: sample.time,
                aggregate_abs_mean,
                local_max_abs,
                aggregate_radius,
                aggregate_margin: aggregate_radius - aggregate_abs_mean,
            });
        }
    }

    Ok(Some(CoordinatedResidualStructure {
        scenario_id: residual.scenario_id.clone(),
        groups: definition.groups.clone(),
        points,
    }))
}

fn write_tabular_artifacts(bundle: &EngineOutputBundle, layout: &OutputLayout) -> Result<()> {
    let scenario_catalog = bundle
        .scenario_outputs
        .iter()
        .map(|scenario| scenario.record.clone())
        .collect::<Vec<_>>();
    write_rows(
        layout.csv_dir.join("scenario_catalog.csv").as_path(),
        scenario_catalog.clone(),
    )?;
    write_rows(
        layout.csv_dir.join("detectability_bounds.csv").as_path(),
        bundle
            .scenario_outputs
            .iter()
            .map(|scenario| scenario.detectability.clone()),
    )?;
    write_rows(
        layout.csv_dir.join("semantic_matches.csv").as_path(),
        bundle
            .scenario_outputs
            .iter()
            .map(|scenario| semantic_csv_row(&scenario.semantics)),
    )?;
    write_rows(
        layout.csv_dir.join("reproducibility_check.csv").as_path(),
        std::iter::once(bundle.reproducibility_check.clone()),
    )?;

    let grammar_rows = bundle
        .scenario_outputs
        .iter()
        .flat_map(|scenario| scenario.grammar.clone())
        .collect::<Vec<_>>();
    write_rows(
        layout.csv_dir.join("grammar_events.csv").as_path(),
        grammar_rows,
    )?;

    let pipeline_rows = bundle
        .scenario_outputs
        .iter()
        .map(|scenario| scenario.syntax.clone())
        .collect::<Vec<_>>();
    write_rows(
        layout.csv_dir.join("pipeline_summary.csv").as_path(),
        pipeline_rows,
    )?;

    for scenario in &bundle.scenario_outputs {
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_timeseries.csv", scenario.record.id))
                .as_path(),
            scenario
                .observed
                .samples
                .iter()
                .zip(&scenario.predicted.samples)
                .map(|(observed, predicted)| {
                    time_series_row(&scenario.record.id, observed, predicted)
                }),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_residual.csv", scenario.record.id))
                .as_path(),
            scenario.residual.samples.iter().map(|sample| {
                vector_norm_row(
                    &scenario.record.id,
                    sample.step,
                    sample.time,
                    &sample.values,
                    sample.norm,
                )
            }),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_drift.csv", scenario.record.id))
                .as_path(),
            scenario.drift.samples.iter().map(|sample| {
                vector_norm_row(
                    &scenario.record.id,
                    sample.step,
                    sample.time,
                    &sample.values,
                    sample.norm,
                )
            }),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_slew.csv", scenario.record.id))
                .as_path(),
            scenario.slew.samples.iter().map(|sample| {
                vector_norm_row(
                    &scenario.record.id,
                    sample.step,
                    sample.time,
                    &sample.values,
                    sample.norm,
                )
            }),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_sign.csv", scenario.record.id))
                .as_path(),
            scenario
                .sign
                .samples
                .iter()
                .map(|sample| sign_csv_row(&scenario.record.id, sample)),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_envelope.csv", scenario.record.id))
                .as_path(),
            scenario.envelope.samples.clone(),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_grammar.csv", scenario.record.id))
                .as_path(),
            scenario.grammar.clone(),
        )?;
        if let Some(coordinated) = &scenario.coordinated {
            write_rows(
                layout
                    .csv_dir
                    .join(format!("{}_coordinated.csv", scenario.record.id))
                    .as_path(),
                coordinated.points.clone(),
            )?;
        }
    }

    write_pretty(
        layout.json_dir.join("run_metadata.json").as_path(),
        &bundle.run_metadata,
    )?;
    write_pretty(
        layout.json_dir.join("scenario_catalog.json").as_path(),
        &scenario_catalog,
    )?;
    write_pretty(
        layout.json_dir.join("reproducibility_check.json").as_path(),
        &bundle.reproducibility_check,
    )?;
    write_pretty(
        layout.json_dir.join("scenario_outputs.json").as_path(),
        &bundle.scenario_outputs,
    )?;

    Ok(())
}

fn run_metadata(timestamp: &str, seed: u64, steps: usize, dt: f64) -> RunMetadata {
    RunMetadata {
        crate_name: "dsfb-semiotics-engine".to_string(),
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_version: command_stdout("rustc", &["--version"]),
        git_commit: command_stdout("git", &["rev-parse", "HEAD"]),
        timestamp: timestamp.to_string(),
        seed,
        steps,
        dt,
        cli_args: std::env::args().collect(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn bound_inputs_for(
    definitions: &[ScenarioDefinition],
    scenario_id: &str,
) -> Option<crate::engine::types::DetectabilityBoundInputs> {
    definitions
        .iter()
        .find(|definition| definition.record.id == scenario_id)
        .and_then(|definition| definition.detectability_inputs.clone())
}

fn reference_for(scenario_id: &str) -> Option<&'static str> {
    match scenario_id {
        "outward_exit_case_a" | "outward_exit_case_b" | "outward_exit_case_c" => {
            Some("nominal_stable")
        }
        "magnitude_matched_detectable" => Some("magnitude_matched_admissible"),
        _ => None,
    }
}

fn default_output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output-dsfb-semiotics-engine")
}

fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
}

fn collect_relative_files(dir: &PathBuf) -> Result<Vec<String>> {
    let mut files = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path().display().to_string())
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

#[derive(Clone, Debug, Serialize)]
struct TimeSeriesCsvRow {
    scenario_id: String,
    step: usize,
    time: f64,
    observed_ch1: Option<f64>,
    observed_ch2: Option<f64>,
    observed_ch3: Option<f64>,
    observed_ch4: Option<f64>,
    predicted_ch1: Option<f64>,
    predicted_ch2: Option<f64>,
    predicted_ch3: Option<f64>,
    predicted_ch4: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct VectorNormCsvRow {
    scenario_id: String,
    step: usize,
    time: f64,
    ch1: Option<f64>,
    ch2: Option<f64>,
    ch3: Option<f64>,
    ch4: Option<f64>,
    norm: f64,
}

#[derive(Clone, Debug, Serialize)]
struct SignCsvRow {
    scenario_id: String,
    step: usize,
    time: f64,
    residual_ch1: Option<f64>,
    residual_ch2: Option<f64>,
    residual_ch3: Option<f64>,
    residual_ch4: Option<f64>,
    drift_ch1: Option<f64>,
    drift_ch2: Option<f64>,
    drift_ch3: Option<f64>,
    drift_ch4: Option<f64>,
    slew_ch1: Option<f64>,
    slew_ch2: Option<f64>,
    slew_ch3: Option<f64>,
    slew_ch4: Option<f64>,
    residual_norm: f64,
    drift_norm: f64,
    slew_norm: f64,
    projection_1: f64,
    projection_2: f64,
    projection_3: f64,
}

#[derive(Clone, Debug, Serialize)]
struct SemanticMatchCsvRow {
    scenario_id: String,
    disposition: String,
    motif_summary: String,
    selected_labels: String,
    candidate_labels: String,
    note: String,
}

fn time_series_row(
    scenario_id: &str,
    observed: &crate::engine::types::VectorSample,
    predicted: &crate::engine::types::VectorSample,
) -> TimeSeriesCsvRow {
    TimeSeriesCsvRow {
        scenario_id: scenario_id.to_string(),
        step: observed.step,
        time: observed.time,
        observed_ch1: value_at(&observed.values, 0),
        observed_ch2: value_at(&observed.values, 1),
        observed_ch3: value_at(&observed.values, 2),
        observed_ch4: value_at(&observed.values, 3),
        predicted_ch1: value_at(&predicted.values, 0),
        predicted_ch2: value_at(&predicted.values, 1),
        predicted_ch3: value_at(&predicted.values, 2),
        predicted_ch4: value_at(&predicted.values, 3),
    }
}

fn vector_norm_row(
    scenario_id: &str,
    step: usize,
    time: f64,
    values: &[f64],
    norm: f64,
) -> VectorNormCsvRow {
    VectorNormCsvRow {
        scenario_id: scenario_id.to_string(),
        step,
        time,
        ch1: value_at(values, 0),
        ch2: value_at(values, 1),
        ch3: value_at(values, 2),
        ch4: value_at(values, 3),
        norm,
    }
}

fn sign_csv_row(scenario_id: &str, sample: &crate::engine::types::SignSample) -> SignCsvRow {
    SignCsvRow {
        scenario_id: scenario_id.to_string(),
        step: sample.step,
        time: sample.time,
        residual_ch1: value_at(&sample.residual, 0),
        residual_ch2: value_at(&sample.residual, 1),
        residual_ch3: value_at(&sample.residual, 2),
        residual_ch4: value_at(&sample.residual, 3),
        drift_ch1: value_at(&sample.drift, 0),
        drift_ch2: value_at(&sample.drift, 1),
        drift_ch3: value_at(&sample.drift, 2),
        drift_ch4: value_at(&sample.drift, 3),
        slew_ch1: value_at(&sample.slew, 0),
        slew_ch2: value_at(&sample.slew, 1),
        slew_ch3: value_at(&sample.slew, 2),
        slew_ch4: value_at(&sample.slew, 3),
        residual_norm: sample.residual_norm,
        drift_norm: sample.drift_norm,
        slew_norm: sample.slew_norm,
        projection_1: sample.projection[0],
        projection_2: sample.projection[1],
        projection_3: sample.projection[2],
    }
}

fn semantic_csv_row(result: &crate::engine::types::SemanticMatchResult) -> SemanticMatchCsvRow {
    SemanticMatchCsvRow {
        scenario_id: result.scenario_id.clone(),
        disposition: format!("{:?}", result.disposition),
        motif_summary: result.motif_summary.clone(),
        selected_labels: result.selected_labels.join(" | "),
        candidate_labels: result
            .candidates
            .iter()
            .map(|candidate| format!("{}:{:.3}", candidate.label, candidate.score))
            .collect::<Vec<_>>()
            .join(" | "),
        note: result.note.clone(),
    }
}

fn value_at(values: &[f64], index: usize) -> Option<f64> {
    values.get(index).copied()
}
