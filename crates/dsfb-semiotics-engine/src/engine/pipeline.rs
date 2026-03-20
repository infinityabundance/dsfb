use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::cli::args::{CsvInputConfig, ScenarioSelection};
use crate::engine::grammar_layer::{evaluate_detectability, evaluate_grammar_layer};
use crate::engine::residual_layer::extract_residuals;
use crate::engine::semantics_layer::retrieve_semantics;
use crate::engine::sign_layer::construct_signs;
use crate::engine::syntax_layer::characterize_syntax_with_coordination;
use crate::engine::types::{
    CoordinatedResidualStructure, DetectabilityBoundInputs, EngineOutputBundle, FigureArtifact,
    GroupDefinition, GroupResidualPoint, ObservedTrajectory, PredictedTrajectory,
    ReproducibilityCheck, ReproducibilitySummary, ResidualTrajectory, RunMetadata, ScenarioOutput,
    ScenarioRecord,
};
use crate::figures::plots::render_all_figures;
use crate::io::csv::write_rows;
use crate::io::input::load_csv_trajectories;
use crate::io::json::write_pretty;
use crate::io::output::{create_output_layout, prepare_clean_export_layout, OutputLayout};
use crate::io::zip::zip_directory;
use crate::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use crate::math::envelope::{build_envelope, EnvelopeSpec};
use crate::math::metrics::{format_metric, hash_serializable_hex, max_abs, pairwise_abs_mean};
use crate::report::artifact_report::build_markdown_report;
use crate::report::pdf::{write_artifact_pdf, PdfTextArtifact};
use crate::sim::generators::synthesize;
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

#[derive(Clone, Debug)]
struct PreparedScenario {
    record: ScenarioRecord,
    observed: ObservedTrajectory,
    predicted: PredictedTrajectory,
    envelope_spec: EnvelopeSpec,
    detectability_inputs: Option<DetectabilityBoundInputs>,
    groups: Vec<GroupDefinition>,
    aggregate_envelope_spec: Option<EnvelopeSpec>,
}

#[derive(Clone, Debug, Serialize)]
struct PartialScenarioOutput {
    record: ScenarioRecord,
    observed: ObservedTrajectory,
    predicted: PredictedTrajectory,
    residual: ResidualTrajectory,
    drift: crate::engine::types::DriftTrajectory,
    slew: crate::engine::types::SlewTrajectory,
    sign: crate::engine::types::SignTrajectory,
    envelope: crate::engine::types::AdmissibilityEnvelope,
    grammar: Vec<crate::engine::types::GrammarStatus>,
    syntax: crate::engine::types::SyntaxCharacterization,
    coordinated: Option<CoordinatedResidualStructure>,
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
    prepare_clean_export_layout(&layout)?;

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
        input_mode: bundle.run_metadata.input_mode.clone(),
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
            "Synthetic and CSV-driven runs share the same deterministic engine layers.".to_string(),
            "Semantic outputs are constrained heuristic retrieval results, not unique-cause claims.".to_string(),
        ],
    };

    let markdown = build_markdown_report(bundle, &figure_artifacts, &report_manifest);
    std::fs::write(&report_markdown_path, &markdown)
        .with_context(|| format!("failed to write {}", report_markdown_path.display()))?;
    write_pretty(&manifest_path, &report_manifest)?;
    let text_artifacts =
        collect_pdf_text_artifacts(&layout, &report_manifest, &markdown, &manifest_path)?;
    write_artifact_pdf(
        &report_pdf_path,
        "dsfb-semiotics-engine report",
        &markdown
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>(),
        &figure_artifacts,
        &report_manifest,
        &text_artifacts,
    )?;
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
        let definitions = all_scenarios();
        let prepared = definitions
            .iter()
            .map(|definition| self.prepare_synthetic(definition))
            .collect::<Vec<_>>();
        self.execute_prepared(&prepared)
    }

    pub fn run_single(&self, scenario_id: &str) -> Result<EngineOutputBundle> {
        let definition = all_scenarios()
            .into_iter()
            .find(|scenario| scenario.record.id == scenario_id)
            .ok_or_else(|| anyhow!("unknown scenario `{scenario_id}`"))?;
        self.execute_prepared(&[self.prepare_synthetic(&definition)])
    }

    pub fn run_csv(&self, input: &CsvInputConfig) -> Result<EngineOutputBundle> {
        self.execute_prepared(&[self.prepare_csv(input)?])
    }

    fn execute_prepared(&self, prepared: &[PreparedScenario]) -> Result<EngineOutputBundle> {
        if prepared.is_empty() {
            return Err(anyhow!("no scenarios selected"));
        }

        let output_root = self
            .config
            .output_root
            .clone()
            .unwrap_or_else(default_output_root);
        let layout = create_output_layout(&output_root)?;

        let first_partial = prepared
            .iter()
            .map(|scenario| self.build_partial_output(scenario))
            .collect::<Result<Vec<_>>>()?;
        let first_lookup = first_partial
            .iter()
            .map(|scenario| (scenario.record.id.clone(), scenario.residual.clone()))
            .collect::<BTreeMap<_, _>>();
        let scenario_outputs = prepared
            .iter()
            .zip(first_partial)
            .map(|(scenario, partial)| self.finalize_output(scenario, partial, &first_lookup))
            .collect::<Vec<_>>();

        let second_partial = prepared
            .iter()
            .map(|scenario| self.build_partial_output(scenario))
            .collect::<Result<Vec<_>>>()?;
        let second_lookup = second_partial
            .iter()
            .map(|scenario| (scenario.record.id.clone(), scenario.residual.clone()))
            .collect::<BTreeMap<_, _>>();
        let scenario_outputs_second = prepared
            .iter()
            .zip(second_partial)
            .map(|(scenario, partial)| self.finalize_output(scenario, partial, &second_lookup))
            .collect::<Vec<_>>();

        let reproducibility_checks = scenario_outputs
            .iter()
            .zip(&scenario_outputs_second)
            .map(|(first, second)| compare_outputs(first, second))
            .collect::<Result<Vec<_>>>()?;
        let reproducibility_summary = summarize_reproducibility(&reproducibility_checks);
        let reproducibility_check = reproducibility_checks
            .first()
            .cloned()
            .context("missing reproducibility result")?;

        Ok(EngineOutputBundle {
            run_metadata: run_metadata(
                &layout.timestamp,
                input_mode_label(&self.config.scenario_selection),
                self.config.seed,
                self.config.steps,
                self.config.dt,
            ),
            run_dir: layout.run_dir,
            scenario_outputs,
            figure_artifacts: Vec::<FigureArtifact>::new(),
            reproducibility_check,
            reproducibility_checks,
            reproducibility_summary,
            report_manifest: None,
            tabular_inventory: BTreeMap::new(),
        })
    }

    fn prepare_synthetic(&self, definition: &ScenarioDefinition) -> PreparedScenario {
        let synthesis = synthesize(
            definition,
            self.config.steps,
            self.config.dt,
            self.config.seed,
        );
        PreparedScenario {
            record: synthesis.record,
            observed: synthesis.observed,
            predicted: synthesis.predicted,
            envelope_spec: definition.envelope_spec.clone(),
            detectability_inputs: definition.detectability_inputs.clone(),
            groups: definition.groups.clone(),
            aggregate_envelope_spec: definition.aggregate_envelope_spec.clone(),
        }
    }

    fn prepare_csv(&self, input: &CsvInputConfig) -> Result<PreparedScenario> {
        let (observed, predicted) = load_csv_trajectories(input)?;
        Ok(PreparedScenario {
            record: ScenarioRecord {
                id: input.scenario_id.clone(),
                title: format!("External CSV Scenario ({})", input.scenario_id),
                data_origin: "external-csv".to_string(),
                purpose: "Run externally supplied observed and predicted trajectories through the same deterministic structural semiotics pipeline used for the synthetic demonstrations, without adding hidden preprocessing.".to_string(),
                theorem_alignment: "This path preserves the layered residual/sign/syntax/grammar/semantics structure, but it does not attach theorem-aligned synthetic guarantees unless the input design justifies them separately.".to_string(),
                claim_class: "external-csv ingestion".to_string(),
                limitations: "Interpretation depends on the supplied predicted trajectory, the configured admissibility envelope, and the sampled times parsed from the CSV files or synthesized deterministically from --dt when no explicit time column is supplied.".to_string(),
            },
            observed,
            predicted,
            envelope_spec: EnvelopeSpec {
                name: input.envelope_name.clone(),
                mode: input.envelope_mode,
                base_radius: input.envelope_base,
                slope: input.envelope_slope,
                switch_step: input.envelope_switch_step,
                secondary_slope: input.envelope_secondary_slope,
                secondary_base: input.envelope_secondary_base,
            },
            detectability_inputs: None,
            groups: Vec::new(),
            aggregate_envelope_spec: None,
        })
    }

    fn build_partial_output(&self, prepared: &PreparedScenario) -> Result<PartialScenarioOutput> {
        let residual =
            extract_residuals(&prepared.observed, &prepared.predicted, &prepared.record.id);
        let drift = compute_drift_trajectory(&residual, self.config.dt, &prepared.record.id);
        let slew = compute_slew_trajectory(&residual, self.config.dt, &prepared.record.id);
        let sign = construct_signs(&residual, &drift, &slew);
        let envelope = build_envelope(&residual, &prepared.envelope_spec, &prepared.record.id);
        let coordinated = build_coordinated(
            &prepared.record.id,
            &prepared.groups,
            prepared.aggregate_envelope_spec.as_ref(),
            &residual,
        )?;
        let grammar = evaluate_grammar_layer(&residual, &envelope);
        let syntax = characterize_syntax_with_coordination(&sign, &grammar, coordinated.as_ref());

        Ok(PartialScenarioOutput {
            record: prepared.record.clone(),
            observed: prepared.observed.clone(),
            predicted: prepared.predicted.clone(),
            residual,
            drift,
            slew,
            sign,
            envelope,
            grammar,
            syntax,
            coordinated,
        })
    }

    fn finalize_output(
        &self,
        prepared: &PreparedScenario,
        partial: PartialScenarioOutput,
        residual_lookup: &BTreeMap<String, ResidualTrajectory>,
    ) -> ScenarioOutput {
        let reference = reference_for(&partial.record.id)
            .and_then(|reference_id| residual_lookup.get(reference_id));
        let detectability = evaluate_detectability(
            &partial.residual,
            &partial.grammar,
            prepared.detectability_inputs.clone(),
            reference,
        );
        let semantics = retrieve_semantics(
            &partial.record.id,
            &partial.syntax,
            &partial.grammar,
            partial.coordinated.as_ref(),
        );

        ScenarioOutput {
            record: partial.record,
            observed: partial.observed,
            predicted: partial.predicted,
            residual: partial.residual,
            drift: partial.drift,
            slew: partial.slew,
            sign: partial.sign,
            envelope: partial.envelope,
            grammar: partial.grammar,
            syntax: partial.syntax,
            detectability,
            semantics,
            coordinated: partial.coordinated,
        }
    }
}

fn compare_outputs(
    first: &ScenarioOutput,
    second: &ScenarioOutput,
) -> Result<ReproducibilityCheck> {
    let first_hash = hash_serializable_hex(format!("{}-first", first.record.id), first)?;
    let second_hash = hash_serializable_hex(format!("{}-second", second.record.id), second)?;
    Ok(ReproducibilityCheck {
        scenario_id: first.record.id.clone(),
        first_hash: first_hash.fnv1a_64_hex.clone(),
        second_hash: second_hash.fnv1a_64_hex.clone(),
        identical: first_hash.fnv1a_64_hex == second_hash.fnv1a_64_hex,
        materialized_components: vec![
            "observed".to_string(),
            "predicted".to_string(),
            "residual".to_string(),
            "drift".to_string(),
            "slew".to_string(),
            "sign".to_string(),
            "envelope".to_string(),
            "grammar".to_string(),
            "syntax".to_string(),
            "detectability".to_string(),
            "semantics".to_string(),
            "coordinated".to_string(),
        ],
        note: "Scenario output was materialized twice under identical deterministic configuration and hashed over full layered outputs, including grammar and semantics.".to_string(),
    })
}

fn summarize_reproducibility(checks: &[ReproducibilityCheck]) -> ReproducibilitySummary {
    let identical_count = checks.iter().filter(|check| check.identical).count();
    ReproducibilitySummary {
        scenario_count: checks.len(),
        identical_count,
        all_identical: identical_count == checks.len(),
        note: "Per-scenario reproducibility is evaluated over full materialized outputs rather than reduced norm summaries.".to_string(),
    }
}

fn build_coordinated(
    scenario_id: &str,
    groups: &[GroupDefinition],
    aggregate_spec: Option<&EnvelopeSpec>,
    residual: &ResidualTrajectory,
) -> Result<Option<CoordinatedResidualStructure>> {
    if groups.is_empty() {
        return Ok(None);
    }
    let aggregate_spec = aggregate_spec.context("grouped scenario missing aggregate envelope")?;

    let mut points = Vec::new();
    for group in groups {
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
                scenario_id: scenario_id.to_string(),
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
        scenario_id: scenario_id.to_string(),
        groups: groups.to_vec(),
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
        bundle
            .reproducibility_checks
            .iter()
            .map(reproducibility_csv_row),
    )?;
    write_rows(
        layout.csv_dir.join("reproducibility_summary.csv").as_path(),
        std::iter::once(bundle.reproducibility_summary.clone()),
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

    write_rows(
        layout.csv_dir.join("pipeline_summary.csv").as_path(),
        bundle
            .scenario_outputs
            .iter()
            .map(|scenario| scenario.syntax.clone()),
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
        layout
            .json_dir
            .join("reproducibility_checks.json")
            .as_path(),
        &bundle.reproducibility_checks,
    )?;
    write_pretty(
        layout
            .json_dir
            .join("reproducibility_summary.json")
            .as_path(),
        &bundle.reproducibility_summary,
    )?;
    write_pretty(
        layout.json_dir.join("scenario_outputs.json").as_path(),
        &bundle.scenario_outputs,
    )?;

    Ok(())
}

fn run_metadata(
    timestamp: &str,
    input_mode: &str,
    seed: u64,
    steps: usize,
    dt: f64,
) -> RunMetadata {
    RunMetadata {
        crate_name: "dsfb-semiotics-engine".to_string(),
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_version: command_stdout("rustc", &["--version"]),
        git_commit: command_stdout("git", &["rev-parse", "HEAD"]),
        timestamp: timestamp.to_string(),
        input_mode: input_mode.to_string(),
        seed,
        steps,
        dt,
        cli_args: std::env::args().collect(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn input_mode_label(selection: &ScenarioSelection) -> &'static str {
    match selection {
        ScenarioSelection::Csv(_) => "csv",
        ScenarioSelection::All | ScenarioSelection::Single(_) => "synthetic",
    }
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

fn collect_relative_files(dir: &Path) -> Result<Vec<String>> {
    let mut files = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path().display().to_string())
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn collect_pdf_text_artifacts(
    layout: &OutputLayout,
    manifest: &crate::engine::types::ReportManifest,
    markdown: &str,
    manifest_path: &Path,
) -> Result<Vec<PdfTextArtifact>> {
    let mut artifacts = Vec::new();
    artifacts.push(PdfTextArtifact {
        title: "Report Markdown Source".to_string(),
        artifact_path: manifest.report_markdown.clone(),
        artifact_kind: "markdown".to_string(),
        content: markdown.to_string(),
    });
    artifacts.push(PdfTextArtifact {
        title: "Run Manifest".to_string(),
        artifact_path: manifest_path.display().to_string(),
        artifact_kind: "json".to_string(),
        content: serde_json::to_string_pretty(manifest)?,
    });

    for path in &manifest.csv_paths {
        artifacts.push(PdfTextArtifact {
            title: format!("CSV Artifact: {}", file_name(path)),
            artifact_path: path.clone(),
            artifact_kind: "csv".to_string(),
            content: std::fs::read_to_string(path)
                .with_context(|| format!("failed to read {path}"))?,
        });
    }
    for path in &manifest.json_paths {
        artifacts.push(PdfTextArtifact {
            title: format!("JSON Artifact: {}", file_name(path)),
            artifact_path: path.clone(),
            artifact_kind: "json".to_string(),
            content: std::fs::read_to_string(path)
                .with_context(|| format!("failed to read {path}"))?,
        });
    }

    artifacts.push(PdfTextArtifact {
        title: "Archive Output Summary".to_string(),
        artifact_path: manifest.zip_archive.clone(),
        artifact_kind: "archive-summary".to_string(),
        content: format!(
            "Zip archive path: {}\nRun directory: {}\nFigures directory: {}\nCSV directory: {}\nJSON directory: {}\nReport directory: {}\n\nThe PDF report embeds the generated figure PNG artifacts and appends the text-based artifacts directly. The zip archive remains the machine-oriented bundle for direct file extraction.",
            manifest.zip_archive,
            manifest.run_dir,
            layout.figures_dir.display(),
            layout.csv_dir.display(),
            layout.json_dir.display(),
            layout.report_dir.display(),
        ),
    });

    Ok(artifacts)
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
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
    selected_heuristic_ids: String,
    resolution_basis: String,
    unknown_reason_class: String,
    unknown_reason_detail: String,
    candidate_labels: String,
    candidate_regimes: String,
    candidate_regime_explanations: String,
    candidate_admissibility: String,
    candidate_scope: String,
    candidate_applicability_notes: String,
    candidate_provenance_notes: String,
    candidate_rationales: String,
    compatibility_note: String,
    compatibility_reasons: String,
    conflict_notes: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct ReproducibilityCsvRow {
    scenario_id: String,
    first_hash: String,
    second_hash: String,
    identical: bool,
    materialized_components: String,
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
        selected_heuristic_ids: result.selected_heuristic_ids.join(" | "),
        resolution_basis: result.resolution_basis.clone(),
        unknown_reason_class: result.unknown_reason_class.clone().unwrap_or_default(),
        unknown_reason_detail: result.unknown_reason_detail.clone().unwrap_or_default(),
        candidate_labels: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.motif_label,
                    format_metric(candidate.score)
                )
            })
            .collect::<Vec<_>>()
            .join(" | "),
        candidate_regimes: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id,
                    if candidate.matched_regimes.is_empty() {
                        "none".to_string()
                    } else {
                        candidate.matched_regimes.join("|")
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(" | "),
        candidate_regime_explanations: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.regime_explanation
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_admissibility: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.admissibility_explanation
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_scope: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.scope_explanation
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_applicability_notes: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.entry.applicability_note
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_provenance_notes: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.entry.provenance.note
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_rationales: result
            .candidates
            .iter()
            .map(|candidate| format!("{}:{}", candidate.entry.heuristic_id, candidate.rationale))
            .collect::<Vec<_>>()
            .join(" || "),
        compatibility_note: result.compatibility_note.clone(),
        compatibility_reasons: result.compatibility_reasons.join(" | "),
        conflict_notes: result.conflict_notes.join(" | "),
        note: result.note.clone(),
    }
}

fn reproducibility_csv_row(
    check: &crate::engine::types::ReproducibilityCheck,
) -> ReproducibilityCsvRow {
    ReproducibilityCsvRow {
        scenario_id: check.scenario_id.clone(),
        first_hash: check.first_hash.clone(),
        second_hash: check.second_hash.clone(),
        identical: check.identical,
        materialized_components: check.materialized_components.join(" | "),
        note: check.note.clone(),
    }
}

fn value_at(values: &[f64], index: usize) -> Option<f64> {
    values.get(index).copied()
}
