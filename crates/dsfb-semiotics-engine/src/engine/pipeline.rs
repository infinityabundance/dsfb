use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::cli::args::{CsvInputConfig, ScenarioSelection};
use crate::engine::bank::{
    HeuristicBankRegistry, HeuristicBankValidationReport, LoadedBankDescriptor,
};
use crate::engine::config::{
    BankSourceConfig, CommonRunConfig, CsvRunConfig, SyntheticRunConfig, SyntheticSelection,
};
use crate::engine::grammar_layer::{evaluate_detectability, evaluate_grammar_layer};
use crate::engine::residual_layer::extract_residuals;
use crate::engine::semantics_layer::retrieve_semantics_with_registry;
use crate::engine::settings::EngineSettings;
use crate::engine::sign_layer::construct_signs;
use crate::engine::syntax_layer::characterize_syntax_with_coordination_configured;
use crate::engine::types::{
    CoordinatedResidualStructure, DetectabilityBoundInputs, EngineOutputBundle, FigureArtifact,
    GroupDefinition, GroupResidualPoint, ObservedTrajectory, PredictedTrajectory,
    ReproducibilityCheck, ReproducibilitySummary, ResidualTrajectory, RunMetadata, ScenarioOutput,
    ScenarioRecord,
};
use crate::evaluation::evaluate_bundle;
use crate::evaluation::sweeps::{generate_sweep_members, SweepConfig, SweepMemberDefinition};
use crate::evaluation::types::{ArtifactCompletenessCheck, FigureIntegrityCheck};
use crate::figures::plots::render_all_figures;
use crate::figures::source::{
    baseline_comparator_source_rows, detectability_source_rows,
    prepare_publication_figure_source_tables, semantic_retrieval_source_rows,
    sweep_summary_source_rows, FigureSourceTable,
};
use crate::io::csv::write_rows;
use crate::io::input::load_csv_trajectories;
use crate::io::json::write_pretty;
use crate::io::output::{create_output_layout, prepare_clean_export_layout, OutputLayout};
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::io::zip::zip_directory;
use crate::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use crate::math::envelope::{build_envelope, EnvelopeSpec};
use crate::math::metrics::{format_metric, hash_serializable_hex, max_abs, pairwise_abs_mean};
use crate::report::artifact_report::build_markdown_report;
use crate::report::pdf::{write_artifact_pdf, PdfTextArtifact};
use crate::sim::generators::synthesize;
use crate::sim::scenarios::{all_scenarios, ScenarioDefinition};

/// Deterministic runtime configuration shared by synthetic and CSV-driven engine runs.
#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub seed: u64,
    pub steps: usize,
    pub dt: f64,
    pub output_root: Option<PathBuf>,
    pub bank: crate::engine::config::BankRunConfig,
    pub scenario_selection: ScenarioSelection,
}

impl EngineConfig {
    /// Builds a configuration for all synthetic scenarios.
    #[must_use]
    pub fn synthetic_all(common: CommonRunConfig) -> Self {
        SyntheticRunConfig::all(common).into()
    }

    /// Builds a configuration for one named synthetic scenario.
    #[must_use]
    pub fn synthetic_single(common: CommonRunConfig, scenario_id: impl Into<String>) -> Self {
        SyntheticRunConfig::single(common, scenario_id).into()
    }

    /// Builds a configuration for a CSV-driven run.
    #[must_use]
    pub fn csv(common: CommonRunConfig, input: CsvInputConfig) -> Self {
        CsvRunConfig::new(common, input).into()
    }

    /// Builds a configuration for a deterministic synthetic sweep.
    #[must_use]
    pub fn sweep(common: CommonRunConfig, sweep: SweepConfig) -> Self {
        Self {
            seed: common.seed,
            steps: common.steps,
            dt: common.dt,
            output_root: common.output_root,
            bank: common.bank,
            scenario_selection: ScenarioSelection::Sweep(sweep),
        }
    }

    /// Validates the deterministic run request before execution.
    pub fn validate(&self) -> Result<()> {
        match &self.scenario_selection {
            ScenarioSelection::All => SyntheticRunConfig {
                common: self.common(),
                selection: SyntheticSelection::All,
            }
            .validate(),
            ScenarioSelection::Single(id) => SyntheticRunConfig {
                common: self.common(),
                selection: SyntheticSelection::Single(id.clone()),
            }
            .validate(),
            ScenarioSelection::Csv(input) => {
                CsvRunConfig::new(self.common(), input.clone()).validate()
            }
            ScenarioSelection::Sweep(_) => self.common().validate(),
        }
    }

    fn common(&self) -> CommonRunConfig {
        CommonRunConfig {
            seed: self.seed,
            steps: self.steps,
            dt: self.dt,
            output_root: self.output_root.clone(),
            bank: self.bank.clone(),
        }
    }
}

/// Library entrypoint for deterministic structural-semiotics runs.
#[derive(Clone, Debug)]
pub struct StructuralSemioticsEngine {
    config: EngineConfig,
    settings: EngineSettings,
    bank_registry: HeuristicBankRegistry,
    loaded_bank: LoadedBankDescriptor,
    bank_validation: HeuristicBankValidationReport,
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

#[derive(Clone, Debug, Default)]
struct TabularArtifactsSummary {
    figure_integrity_checks: Vec<FigureIntegrityCheck>,
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

/// Runs the full synthetic scenario catalog under one deterministic configuration.
pub fn run_all_demos(config: EngineConfig) -> Result<EngineOutputBundle> {
    StructuralSemioticsEngine::new(config).run_all()
}

/// Runs one named scenario under the provided deterministic configuration.
pub fn run_scenario(config: EngineConfig, scenario_id: &str) -> Result<EngineOutputBundle> {
    StructuralSemioticsEngine::new(config).run_single(scenario_id)
}

/// Writes figures, CSV, JSON, markdown, PDF, and zip artifacts for one completed bundle.
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

    let figure_source_tables = prepare_publication_figure_source_tables(bundle)?;
    let figure_artifacts = render_all_figures(&figure_source_tables, &layout.figures_dir)?;
    let tabular_summary = write_tabular_artifacts(
        bundle,
        &figure_source_tables,
        &figure_artifacts,
        &layout,
        bundle
            .run_metadata
            .engine_settings
            .plotting
            .count_like_integer_tolerance,
    )?;

    let manifest_path = layout.run_dir.join("manifest.json");
    let report_markdown_path = layout.report_dir.join("dsfb_semiotics_engine_report.md");
    let report_pdf_path = layout.report_dir.join("dsfb_semiotics_engine_report.pdf");
    let zip_path = layout.run_dir.join(format!(
        "dsfb-semiotics-engine-{}.zip",
        bundle.run_metadata.timestamp
    ));

    let initial_manifest = build_report_manifest(
        bundle,
        &figure_artifacts,
        &layout,
        &report_markdown_path,
        &report_pdf_path,
        &zip_path,
        None,
    )?;
    let initial_markdown = build_markdown_report(
        bundle,
        &figure_artifacts,
        &initial_manifest,
        None,
        Some(&tabular_summary.figure_integrity_checks),
    );
    std::fs::write(&report_markdown_path, &initial_markdown)
        .with_context(|| format!("failed to write {}", report_markdown_path.display()))?;
    write_pretty(&manifest_path, &initial_manifest)?;
    let text_artifacts = collect_pdf_text_artifacts(
        &layout,
        &initial_manifest,
        &initial_markdown,
        &manifest_path,
    )?;
    write_artifact_pdf(
        &report_pdf_path,
        "dsfb-semiotics-engine report",
        &initial_markdown
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>(),
        &figure_artifacts,
        &initial_manifest,
        &text_artifacts,
    )?;
    zip_directory(&layout.run_dir, &zip_path)?;
    let completeness = build_artifact_completeness(
        bundle,
        &layout,
        &figure_artifacts,
        &report_markdown_path,
        &report_pdf_path,
        &zip_path,
        &manifest_path,
    )?;
    write_rows(
        layout.csv_dir.join("artifact_completeness.csv").as_path(),
        std::iter::once(completeness.clone()),
    )?;
    write_pretty(
        layout.json_dir.join("artifact_completeness.json").as_path(),
        &completeness,
    )?;
    let report_manifest = build_report_manifest(
        bundle,
        &figure_artifacts,
        &layout,
        &report_markdown_path,
        &report_pdf_path,
        &zip_path,
        Some(&completeness),
    )?;
    let final_markdown = build_markdown_report(
        bundle,
        &figure_artifacts,
        &report_manifest,
        Some(&completeness),
        Some(&tabular_summary.figure_integrity_checks),
    );
    std::fs::write(&report_markdown_path, &final_markdown)
        .with_context(|| format!("failed to write {}", report_markdown_path.display()))?;
    write_pretty(&manifest_path, &report_manifest)?;
    let final_text_artifacts =
        collect_pdf_text_artifacts(&layout, &report_manifest, &final_markdown, &manifest_path)?;
    write_artifact_pdf(
        &report_pdf_path,
        "dsfb-semiotics-engine report",
        &final_markdown
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>(),
        &figure_artifacts,
        &report_manifest,
        &final_text_artifacts,
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
        Self::with_settings(config, EngineSettings::default())
            .expect("deterministic engine configuration and heuristic bank should validate")
    }

    /// Creates an engine with explicit deterministic settings.
    pub fn with_settings(config: EngineConfig, settings: EngineSettings) -> Result<Self> {
        config.validate()?;
        let (bank_registry, loaded_bank, bank_validation) = match &config.bank.source {
            BankSourceConfig::Builtin => {
                HeuristicBankRegistry::load_builtin(config.bank.strict_validation)?
            }
            BankSourceConfig::External(path) => {
                HeuristicBankRegistry::load_external_json(path, config.bank.strict_validation)?
            }
        };
        Ok(Self {
            config,
            settings,
            bank_registry,
            loaded_bank,
            bank_validation,
        })
    }

    /// Runs whatever scenario selection was encoded in the engine configuration.
    pub fn run_selected(&self) -> Result<EngineOutputBundle> {
        match &self.config.scenario_selection {
            ScenarioSelection::All => self.run_all(),
            ScenarioSelection::Single(scenario_id) => self.run_single(scenario_id),
            ScenarioSelection::Csv(input) => self.run_csv(input),
            ScenarioSelection::Sweep(config) => self.run_sweep(config),
        }
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

    pub fn run_sweep(&self, config: &SweepConfig) -> Result<EngineOutputBundle> {
        let prepared = generate_sweep_members(
            &config.normalized(&self.settings.evaluation),
            self.config.steps,
            self.config.dt,
        )?
        .into_iter()
        .map(|member| self.prepare_sweep_member(member))
        .collect::<Vec<_>>();
        self.execute_prepared(&prepared)
    }

    fn execute_prepared(&self, prepared: &[PreparedScenario]) -> Result<EngineOutputBundle> {
        self.config.validate()?;
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
        let run_metadata = run_metadata(
            &layout.timestamp,
            input_mode_label(&self.config.scenario_selection),
            self.config.seed,
            self.config.steps,
            self.config.dt,
            &self.settings,
            &self.loaded_bank,
        );
        let provisional_bundle = EngineOutputBundle {
            run_metadata: run_metadata.clone(),
            run_dir: layout.run_dir.clone(),
            scenario_outputs: scenario_outputs.clone(),
            evaluation: crate::evaluation::types::RunEvaluationBundle {
                summary: crate::evaluation::types::RunEvaluationSummary {
                    schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                    engine_version: run_metadata.crate_version.clone(),
                    bank_version: run_metadata.bank.bank_version.clone(),
                    evaluation_version: "evaluation/v1".to_string(),
                    input_mode: input_mode_label(&self.config.scenario_selection).to_string(),
                    scenario_count: 0,
                    semantic_disposition_counts: BTreeMap::new(),
                    syntax_label_counts: BTreeMap::new(),
                    boundary_interaction_count: 0,
                    violation_count: 0,
                    comparator_trigger_counts: BTreeMap::new(),
                    reproducible_scenario_count: 0,
                    all_reproducible: false,
                    note: String::new(),
                },
                scenario_evaluations: Vec::new(),
                baseline_results: Vec::new(),
                bank_validation: self.bank_validation.clone(),
                artifact_completeness: None,
                sweep_results: Vec::new(),
                sweep_summary: None,
            },
            figure_artifacts: Vec::<FigureArtifact>::new(),
            reproducibility_check: reproducibility_check.clone(),
            reproducibility_checks: reproducibility_checks.clone(),
            reproducibility_summary: reproducibility_summary.clone(),
            report_manifest: None,
            tabular_inventory: BTreeMap::new(),
        };
        let evaluation = evaluate_bundle(
            &provisional_bundle,
            &self.settings.evaluation,
            &self.bank_validation,
            None,
        );

        Ok(EngineOutputBundle {
            run_metadata,
            run_dir: layout.run_dir,
            scenario_outputs,
            evaluation,
            figure_artifacts: Vec::<FigureArtifact>::new(),
            reproducibility_check,
            reproducibility_checks,
            reproducibility_summary,
            report_manifest: None,
            tabular_inventory: BTreeMap::new(),
        })
    }

    fn prepare_sweep_member(&self, member: SweepMemberDefinition) -> PreparedScenario {
        PreparedScenario {
            record: member.record,
            observed: member.observed,
            predicted: member.predicted,
            envelope_spec: member.envelope_spec,
            detectability_inputs: member.detectability_inputs,
            groups: member.groups,
            aggregate_envelope_spec: member.aggregate_envelope_spec,
        }
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
        input.validate()?;
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
                sweep_family: None,
                sweep_parameter_name: None,
                sweep_parameter_value: None,
                sweep_secondary_parameter_name: None,
                sweep_secondary_parameter_value: None,
            },
            observed,
            predicted,
            envelope_spec: input.envelope_spec()?,
            detectability_inputs: None,
            groups: Vec::new(),
            aggregate_envelope_spec: None,
        })
    }

    fn build_partial_output(&self, prepared: &PreparedScenario) -> Result<PartialScenarioOutput> {
        prepared.envelope_spec.validate()?;
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
        let syntax = characterize_syntax_with_coordination_configured(
            &sign,
            &grammar,
            coordinated.as_ref(),
            &self.settings.syntax,
        );

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
        let semantics = retrieve_semantics_with_registry(
            &partial.record.id,
            &partial.syntax,
            &partial.grammar,
            partial.coordinated.as_ref(),
            &self.bank_registry,
            &self.settings.semantics,
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

fn write_tabular_artifacts(
    bundle: &EngineOutputBundle,
    figure_source_tables: &[FigureSourceTable],
    figure_artifacts: &[FigureArtifact],
    layout: &OutputLayout,
    count_like_integer_tolerance: f64,
) -> Result<TabularArtifactsSummary> {
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
            .map(|check| reproducibility_csv_row(bundle, check)),
    )?;
    write_rows(
        layout.csv_dir.join("reproducibility_summary.csv").as_path(),
        std::iter::once(bundle.reproducibility_summary.clone()),
    )?;
    write_rows(
        layout.csv_dir.join("evaluation_summary.csv").as_path(),
        std::iter::once(evaluation_summary_csv_row(&bundle.evaluation.summary)),
    )?;
    write_rows(
        layout.csv_dir.join("scenario_evaluations.csv").as_path(),
        bundle
            .evaluation
            .scenario_evaluations
            .iter()
            .map(scenario_evaluation_csv_row),
    )?;
    write_rows(
        layout.csv_dir.join("baseline_comparators.csv").as_path(),
        bundle.evaluation.baseline_results.clone(),
    )?;
    write_rows(
        layout
            .csv_dir
            .join("heuristic_bank_validation.csv")
            .as_path(),
        std::iter::once(bank_validation_csv_row(&bundle.evaluation.bank_validation)),
    )?;
    if !bundle.evaluation.sweep_results.is_empty() {
        write_rows(
            layout.csv_dir.join("sweep_results.csv").as_path(),
            bundle
                .evaluation
                .sweep_results
                .iter()
                .map(sweep_point_csv_row),
        )?;
    }
    if let Some(summary) = &bundle.evaluation.sweep_summary {
        write_rows(
            layout.csv_dir.join("sweep_summary.csv").as_path(),
            std::iter::once(sweep_summary_csv_row(summary)),
        )?;
    }

    let figure_integrity_checks = write_summary_figure_source_tables(
        bundle,
        figure_source_tables,
        figure_artifacts,
        layout,
        count_like_integer_tolerance,
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
        layout
            .json_dir
            .join("loaded_heuristic_bank_descriptor.json")
            .as_path(),
        &bundle.run_metadata.bank,
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
        layout.json_dir.join("evaluation_summary.json").as_path(),
        &bundle.evaluation.summary,
    )?;
    write_pretty(
        layout.json_dir.join("scenario_evaluations.json").as_path(),
        &bundle.evaluation.scenario_evaluations,
    )?;
    write_pretty(
        layout.json_dir.join("baseline_comparators.json").as_path(),
        &bundle.evaluation.baseline_results,
    )?;
    write_pretty(
        layout.json_dir.join("semantic_matches.json").as_path(),
        &bundle
            .scenario_outputs
            .iter()
            .map(|scenario| scenario.semantics.clone())
            .collect::<Vec<_>>(),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("heuristic_bank_validation.json")
            .as_path(),
        &bundle.evaluation.bank_validation,
    )?;
    if !bundle.evaluation.sweep_results.is_empty() {
        write_pretty(
            layout.json_dir.join("sweep_results.json").as_path(),
            &bundle.evaluation.sweep_results,
        )?;
    }
    if let Some(summary) = &bundle.evaluation.sweep_summary {
        write_pretty(
            layout.json_dir.join("sweep_summary.json").as_path(),
            summary,
        )?;
    }
    write_pretty(
        layout.json_dir.join("scenario_outputs.json").as_path(),
        &bundle.scenario_outputs,
    )?;

    Ok(TabularArtifactsSummary {
        figure_integrity_checks,
    })
}

fn write_summary_figure_source_tables(
    bundle: &EngineOutputBundle,
    figure_source_tables: &[FigureSourceTable],
    figure_artifacts: &[FigureArtifact],
    layout: &OutputLayout,
    count_like_integer_tolerance: f64,
) -> Result<Vec<FigureIntegrityCheck>> {
    let mut checks = Vec::new();
    let figure_lookup = figure_artifacts
        .iter()
        .map(|artifact| (artifact.figure_id.clone(), artifact))
        .collect::<BTreeMap<_, _>>();

    for table in figure_source_tables {
        let source_csv = layout
            .csv_dir
            .join(format!("{}_source.csv", table.figure_id))
            .display()
            .to_string();
        let source_json = layout
            .json_dir
            .join(format!("{}_source.json", table.figure_id))
            .display()
            .to_string();
        write_rows(Path::new(&source_csv), table.rows.clone())?;
        write_pretty(Path::new(&source_json), table)?;

        let panel_order = ordered_panel_ids(table);
        let panel_labels = panel_order
            .iter()
            .map(|panel_id| panel_title(table, panel_id))
            .collect::<Vec<_>>();
        let series_lengths = panel_order
            .iter()
            .map(|panel_id| {
                table
                    .rows
                    .iter()
                    .filter(|row| row.panel_id == *panel_id)
                    .count()
            })
            .collect::<Vec<_>>();
        let source_row_count = table.rows.len();
        let nonempty_series =
            !table.rows.is_empty() && series_lengths.iter().all(|length| *length > 0);
        let nonzero_values_present = table.rows.iter().any(|row| {
            row.y_value.abs() > 1.0e-12
                || row
                    .secondary_y_value
                    .map(|value| value.abs() > 1.0e-12)
                    .unwrap_or(false)
        });
        let count_like_panels_integerlike = table.count_like_panel_ids.iter().all(|panel_id| {
            table
                .rows
                .iter()
                .filter(|row| row.panel_id == *panel_id)
                .all(|row| {
                    (row.y_value - row.y_value.round()).abs() <= count_like_integer_tolerance
                })
        });
        let artifact = figure_lookup.get(&table.figure_id);
        let png_path = artifact
            .map(|artifact| artifact.png_path.clone())
            .unwrap_or_default();
        let svg_path = artifact
            .map(|artifact| artifact.svg_path.clone())
            .unwrap_or_default();
        let png_present = !png_path.is_empty() && Path::new(&png_path).is_file();
        let svg_present = !svg_path.is_empty() && Path::new(&svg_path).is_file();
        let observed_panel_count = panel_order.len();
        checks.push(FigureIntegrityCheck {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: bundle.run_metadata.crate_version.clone(),
            bank_version: bundle.run_metadata.bank.bank_version.clone(),
            figure_id: table.figure_id.clone(),
            expected_panel_count: table.expected_panel_count,
            observed_panel_count,
            panel_labels,
            series_lengths,
            source_row_count,
            nonempty_series,
            nonzero_values_present,
            count_like_panels_integerlike,
            consistent_with_source: observed_panel_count == table.expected_panel_count
                && nonempty_series
                && count_like_panels_integerlike
                && png_present
                && svg_present,
            source_csv,
            source_json,
            png_path,
            svg_path,
            png_present,
            svg_present,
            note: "Figure rendered from the exported figure-source table; integrity check covers panel count, source rows, count-like panels, and emitted PNG/SVG presence."
                .to_string(),
        });
    }

    write_legacy_summary_figure_sources(bundle, layout)?;

    write_rows(
        layout.csv_dir.join("figure_integrity_checks.csv").as_path(),
        checks.iter().map(figure_integrity_csv_row),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_integrity_checks.json")
            .as_path(),
        &checks,
    )?;

    Ok(checks)
}

fn write_legacy_summary_figure_sources(
    bundle: &EngineOutputBundle,
    layout: &OutputLayout,
) -> Result<()> {
    let detectability_rows = detectability_source_rows(bundle);
    write_rows(
        layout
            .csv_dir
            .join("figure_09_detectability_source.csv")
            .as_path(),
        detectability_rows.clone(),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_09_detectability_source.json")
            .as_path(),
        &detectability_rows,
    )?;

    let semantic_rows = semantic_retrieval_source_rows(bundle);
    write_rows(
        layout
            .csv_dir
            .join("figure_12_semantic_retrieval_source.csv")
            .as_path(),
        semantic_rows.clone(),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_12_semantic_retrieval_source.json")
            .as_path(),
        &semantic_rows,
    )?;

    let baseline_rows = baseline_comparator_source_rows(bundle);
    write_rows(
        layout
            .csv_dir
            .join("figure_13_internal_baseline_comparators_source.csv")
            .as_path(),
        baseline_rows.clone(),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_13_internal_baseline_comparators_source.json")
            .as_path(),
        &baseline_rows,
    )?;

    if !bundle.evaluation.sweep_results.is_empty() {
        let sweep_rows = sweep_summary_source_rows(bundle);
        #[derive(Serialize)]
        struct SweepLegacySourceCsvRow {
            schema_version: String,
            figure_id: String,
            sweep_family: String,
            scenario_id: String,
            parameter_name: String,
            parameter_value: f64,
            semantic_disposition: String,
            disposition_code: i32,
            selected_heuristic_ids: String,
            note: String,
        }
        write_rows(
            layout
                .csv_dir
                .join("figure_14_sweep_stability_source.csv")
                .as_path(),
            sweep_rows.iter().map(|row| SweepLegacySourceCsvRow {
                schema_version: row.schema_version.clone(),
                figure_id: row.figure_id.clone(),
                sweep_family: row.sweep_family.clone(),
                scenario_id: row.scenario_id.clone(),
                parameter_name: row.parameter_name.clone(),
                parameter_value: row.parameter_value,
                semantic_disposition: row.semantic_disposition.clone(),
                disposition_code: row.disposition_code,
                selected_heuristic_ids: row.selected_heuristic_ids.join(" | "),
                note: row.note.clone(),
            }),
        )?;
        write_pretty(
            layout
                .json_dir
                .join("figure_14_sweep_stability_source.json")
                .as_path(),
            &sweep_rows,
        )?;
    }

    Ok(())
}

fn ordered_panel_ids(table: &FigureSourceTable) -> Vec<String> {
    let mut panel_ids = Vec::new();
    for row in &table.rows {
        if !panel_ids.iter().any(|panel_id| panel_id == &row.panel_id) {
            panel_ids.push(row.panel_id.clone());
        }
    }
    panel_ids
}

fn panel_title(table: &FigureSourceTable, panel_id: &str) -> String {
    table
        .rows
        .iter()
        .find(|row| row.panel_id == panel_id)
        .map(|row| row.panel_title.clone())
        .unwrap_or_else(|| panel_id.to_string())
}

fn run_metadata(
    timestamp: &str,
    input_mode: &str,
    seed: u64,
    steps: usize,
    dt: f64,
    settings: &EngineSettings,
    bank: &LoadedBankDescriptor,
) -> RunMetadata {
    RunMetadata {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        bank_version: bank.bank_version.clone(),
        crate_name: "dsfb-semiotics-engine".to_string(),
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_version: command_stdout("rustc", &["--version"]),
        git_commit: command_stdout("git", &["rev-parse", "HEAD"]),
        timestamp: timestamp.to_string(),
        input_mode: input_mode.to_string(),
        seed,
        steps,
        dt,
        engine_settings: settings.clone(),
        bank: bank.clone(),
        cli_args: std::env::args().collect(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn input_mode_label(selection: &ScenarioSelection) -> &'static str {
    match selection {
        ScenarioSelection::Csv(_) => "csv",
        ScenarioSelection::Sweep(_) => "synthetic-sweep",
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

fn build_report_manifest(
    bundle: &EngineOutputBundle,
    figure_artifacts: &[FigureArtifact],
    layout: &OutputLayout,
    report_markdown_path: &Path,
    report_pdf_path: &Path,
    zip_path: &Path,
    completeness: Option<&ArtifactCompletenessCheck>,
) -> Result<crate::engine::types::ReportManifest> {
    let mut notes = vec![
        "Synthetic and CSV-driven runs share the same deterministic engine layers.".to_string(),
        "Semantic outputs are constrained heuristic retrieval results, not unique-cause claims.".to_string(),
        "Evaluation outputs summarize the deterministic engine with internal deterministic comparators only.".to_string(),
        format!(
            "Heuristic bank source=`{}`, version=`{}`, hash=`{}`.",
            bundle.run_metadata.bank.source_kind.as_label(),
            bundle.run_metadata.bank.bank_version,
            bundle.run_metadata.bank.content_hash
        ),
    ];
    if let Some(completeness) = completeness {
        notes.push(format!(
            "Artifact completeness: complete=`{}` with {} figures, {} CSV files, and {} JSON files.",
            completeness.complete,
            completeness.figure_count,
            completeness.csv_count,
            completeness.json_count
        ));
    }
    Ok(crate::engine::types::ReportManifest {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        crate_name: bundle.run_metadata.crate_name.clone(),
        crate_version: bundle.run_metadata.crate_version.clone(),
        timestamp: bundle.run_metadata.timestamp.clone(),
        input_mode: bundle.run_metadata.input_mode.clone(),
        bank: bundle.run_metadata.bank.clone(),
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
        notes,
    })
}

fn build_artifact_completeness(
    bundle: &EngineOutputBundle,
    layout: &OutputLayout,
    figure_artifacts: &[FigureArtifact],
    report_markdown_path: &Path,
    report_pdf_path: &Path,
    zip_path: &Path,
    manifest_path: &Path,
) -> Result<ArtifactCompletenessCheck> {
    let csv_count = collect_relative_files(&layout.csv_dir)?.len() + 1;
    let json_count = collect_relative_files(&layout.json_dir)?.len() + 1;
    let report_markdown_present = report_markdown_path.is_file();
    let report_pdf_present = report_pdf_path.is_file();
    let zip_present = zip_path.is_file();
    let manifest_present = manifest_path.is_file();
    let complete = report_markdown_present
        && report_pdf_present
        && zip_present
        && manifest_present
        && !figure_artifacts.is_empty();
    Ok(ArtifactCompletenessCheck {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        figure_count: figure_artifacts.len() * 2,
        csv_count,
        json_count,
        report_markdown_present,
        report_pdf_present,
        zip_present,
        manifest_present,
        complete,
        note: "Artifact completeness is evaluated after the deterministic export pipeline has emitted figures, tables, report files, manifest, and zip archive.".to_string(),
    })
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
    heuristic_bank_entry_count: usize,
    heuristic_candidates_post_admissibility: usize,
    heuristic_candidates_post_regime: usize,
    heuristic_candidates_pre_scope: usize,
    heuristic_candidates_post_scope: usize,
    heuristics_rejected_by_admissibility: usize,
    heuristics_rejected_by_regime: usize,
    heuristics_rejected_by_scope: usize,
    heuristics_selected_final: usize,
    candidate_ids_post_admissibility: String,
    candidate_ids_post_regime: String,
    candidate_ids_post_scope: String,
    rejected_by_admissibility_ids: String,
    rejected_by_regime_ids: String,
    rejected_by_scope_ids: String,
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
    candidate_metric_highlights: String,
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
    schema_version: String,
    engine_version: String,
    bank_version: String,
    scenario_id: String,
    first_hash: String,
    second_hash: String,
    identical: bool,
    materialized_components: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct EvaluationSummaryCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    evaluation_version: String,
    input_mode: String,
    scenario_count: usize,
    semantic_disposition_counts: String,
    syntax_label_counts: String,
    boundary_interaction_count: usize,
    violation_count: usize,
    comparator_trigger_counts: String,
    reproducible_scenario_count: usize,
    all_reproducible: bool,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct ScenarioEvaluationCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    scenario_id: String,
    input_mode: String,
    syntax_label: String,
    semantic_disposition: String,
    selected_heuristic_ids: String,
    heuristic_bank_entry_count: usize,
    heuristic_candidates_post_admissibility: usize,
    heuristic_candidates_post_regime: usize,
    heuristic_candidates_pre_scope: usize,
    heuristic_candidates_post_scope: usize,
    heuristics_rejected_by_admissibility: usize,
    heuristics_rejected_by_regime: usize,
    heuristics_rejected_by_scope: usize,
    heuristics_selected_final: usize,
    boundary_sample_count: usize,
    violation_sample_count: usize,
    first_boundary_time: Option<f64>,
    first_violation_time: Option<f64>,
    reproducible: bool,
    triggered_baseline_count: usize,
    unknown_reason_class: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct BankValidationCsvRow {
    schema_version: String,
    engine_version: String,
    bank_schema_version: String,
    bank_version: String,
    bank_source_kind: String,
    bank_source_path: String,
    bank_content_hash: String,
    strict_validation: bool,
    entry_count: usize,
    valid: bool,
    duplicate_ids: String,
    self_link_notes: String,
    compatibility_conflicts: String,
    missing_compatibility_links: String,
    missing_incompatibility_links: String,
    strict_validation_errors: String,
    unknown_link_targets: String,
    provenance_gaps: String,
    scope_sanity_notes: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct SweepPointCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    sweep_family: String,
    scenario_id: String,
    parameter_name: String,
    parameter_value: f64,
    secondary_parameter_name: String,
    secondary_parameter_value: Option<f64>,
    syntax_label: String,
    semantic_disposition: String,
    selected_heuristic_ids: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct SweepSummaryCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    sweep_family: String,
    member_count: usize,
    unique_syntax_labels: String,
    unique_semantic_dispositions: String,
    unknown_count: usize,
    ambiguous_count: usize,
    disposition_flip_count: usize,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct FigureIntegrityCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    figure_id: String,
    expected_panel_count: usize,
    observed_panel_count: usize,
    panel_labels: String,
    series_lengths: String,
    source_row_count: usize,
    nonempty_series: bool,
    nonzero_values_present: bool,
    count_like_panels_integerlike: bool,
    consistent_with_source: bool,
    source_csv: String,
    source_json: String,
    png_path: String,
    svg_path: String,
    png_present: bool,
    svg_present: bool,
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
        heuristic_bank_entry_count: result.retrieval_audit.heuristic_bank_entry_count,
        heuristic_candidates_post_admissibility: result
            .retrieval_audit
            .heuristic_candidates_post_admissibility,
        heuristic_candidates_post_regime: result.retrieval_audit.heuristic_candidates_post_regime,
        heuristic_candidates_pre_scope: result.retrieval_audit.heuristic_candidates_pre_scope,
        heuristic_candidates_post_scope: result.retrieval_audit.heuristic_candidates_post_scope,
        heuristics_rejected_by_admissibility: result
            .retrieval_audit
            .heuristics_rejected_by_admissibility,
        heuristics_rejected_by_regime: result.retrieval_audit.heuristics_rejected_by_regime,
        heuristics_rejected_by_scope: result.retrieval_audit.heuristics_rejected_by_scope,
        heuristics_selected_final: result.retrieval_audit.heuristics_selected_final,
        candidate_ids_post_admissibility: result
            .retrieval_audit
            .candidate_ids_post_admissibility
            .join(" | "),
        candidate_ids_post_regime: result.retrieval_audit.candidate_ids_post_regime.join(" | "),
        candidate_ids_post_scope: result.retrieval_audit.candidate_ids_post_scope.join(" | "),
        rejected_by_admissibility_ids: result
            .retrieval_audit
            .rejected_by_admissibility_ids
            .join(" | "),
        rejected_by_regime_ids: result.retrieval_audit.rejected_by_regime_ids.join(" | "),
        rejected_by_scope_ids: result.retrieval_audit.rejected_by_scope_ids.join(" | "),
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
        candidate_metric_highlights: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id,
                    candidate.metric_highlights.join("; ")
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
    bundle: &EngineOutputBundle,
    check: &crate::engine::types::ReproducibilityCheck,
) -> ReproducibilityCsvRow {
    ReproducibilityCsvRow {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        scenario_id: check.scenario_id.clone(),
        first_hash: check.first_hash.clone(),
        second_hash: check.second_hash.clone(),
        identical: check.identical,
        materialized_components: check.materialized_components.join(" | "),
        note: check.note.clone(),
    }
}

fn evaluation_summary_csv_row(
    summary: &crate::evaluation::types::RunEvaluationSummary,
) -> EvaluationSummaryCsvRow {
    EvaluationSummaryCsvRow {
        schema_version: summary.schema_version.clone(),
        engine_version: summary.engine_version.clone(),
        bank_version: summary.bank_version.clone(),
        evaluation_version: summary.evaluation_version.clone(),
        input_mode: summary.input_mode.clone(),
        scenario_count: summary.scenario_count,
        semantic_disposition_counts: join_count_map(&summary.semantic_disposition_counts),
        syntax_label_counts: join_count_map(&summary.syntax_label_counts),
        boundary_interaction_count: summary.boundary_interaction_count,
        violation_count: summary.violation_count,
        comparator_trigger_counts: join_count_map(&summary.comparator_trigger_counts),
        reproducible_scenario_count: summary.reproducible_scenario_count,
        all_reproducible: summary.all_reproducible,
        note: summary.note.clone(),
    }
}

fn scenario_evaluation_csv_row(
    summary: &crate::evaluation::types::ScenarioEvaluationSummary,
) -> ScenarioEvaluationCsvRow {
    ScenarioEvaluationCsvRow {
        schema_version: summary.schema_version.clone(),
        engine_version: summary.engine_version.clone(),
        bank_version: summary.bank_version.clone(),
        scenario_id: summary.scenario_id.clone(),
        input_mode: summary.input_mode.clone(),
        syntax_label: summary.syntax_label.clone(),
        semantic_disposition: summary.semantic_disposition.clone(),
        selected_heuristic_ids: summary.selected_heuristic_ids.join(" | "),
        heuristic_bank_entry_count: summary.heuristic_bank_entry_count,
        heuristic_candidates_post_admissibility: summary.heuristic_candidates_post_admissibility,
        heuristic_candidates_post_regime: summary.heuristic_candidates_post_regime,
        heuristic_candidates_pre_scope: summary.heuristic_candidates_pre_scope,
        heuristic_candidates_post_scope: summary.heuristic_candidates_post_scope,
        heuristics_rejected_by_admissibility: summary.heuristics_rejected_by_admissibility,
        heuristics_rejected_by_regime: summary.heuristics_rejected_by_regime,
        heuristics_rejected_by_scope: summary.heuristics_rejected_by_scope,
        heuristics_selected_final: summary.heuristics_selected_final,
        boundary_sample_count: summary.boundary_sample_count,
        violation_sample_count: summary.violation_sample_count,
        first_boundary_time: summary.first_boundary_time,
        first_violation_time: summary.first_violation_time,
        reproducible: summary.reproducible,
        triggered_baseline_count: summary.triggered_baseline_count,
        unknown_reason_class: summary.unknown_reason_class.clone().unwrap_or_default(),
        note: summary.note.clone(),
    }
}

fn bank_validation_csv_row(
    report: &crate::engine::bank::HeuristicBankValidationReport,
) -> BankValidationCsvRow {
    BankValidationCsvRow {
        schema_version: report.schema_version.clone(),
        engine_version: report.engine_version.clone(),
        bank_schema_version: report.bank_schema_version.clone(),
        bank_version: report.bank_version.clone(),
        bank_source_kind: report.bank_source_kind.as_label().to_string(),
        bank_source_path: report.bank_source_path.clone().unwrap_or_default(),
        bank_content_hash: report.bank_content_hash.clone(),
        strict_validation: report.strict_validation,
        entry_count: report.entry_count,
        valid: report.valid,
        duplicate_ids: report.duplicate_ids.join(" | "),
        self_link_notes: report.self_link_notes.join(" | "),
        compatibility_conflicts: report.compatibility_conflicts.join(" | "),
        missing_compatibility_links: report.missing_compatibility_links.join(" | "),
        missing_incompatibility_links: report.missing_incompatibility_links.join(" | "),
        strict_validation_errors: report.strict_validation_errors.join(" | "),
        unknown_link_targets: report.unknown_link_targets.join(" | "),
        provenance_gaps: report.provenance_gaps.join(" | "),
        scope_sanity_notes: report.scope_sanity_notes.join(" | "),
        note: report.note.clone(),
    }
}

fn sweep_point_csv_row(point: &crate::evaluation::types::SweepPointResult) -> SweepPointCsvRow {
    SweepPointCsvRow {
        schema_version: point.schema_version.clone(),
        engine_version: point.engine_version.clone(),
        bank_version: point.bank_version.clone(),
        sweep_family: point.sweep_family.clone(),
        scenario_id: point.scenario_id.clone(),
        parameter_name: point.parameter_name.clone(),
        parameter_value: point.parameter_value,
        secondary_parameter_name: point.secondary_parameter_name.clone().unwrap_or_default(),
        secondary_parameter_value: point.secondary_parameter_value,
        syntax_label: point.syntax_label.clone(),
        semantic_disposition: point.semantic_disposition.clone(),
        selected_heuristic_ids: point.selected_heuristic_ids.join(" | "),
        note: point.note.clone(),
    }
}

fn sweep_summary_csv_row(
    summary: &crate::evaluation::types::SweepRunSummary,
) -> SweepSummaryCsvRow {
    SweepSummaryCsvRow {
        schema_version: summary.schema_version.clone(),
        engine_version: summary.engine_version.clone(),
        bank_version: summary.bank_version.clone(),
        sweep_family: summary.sweep_family.clone(),
        member_count: summary.member_count,
        unique_syntax_labels: summary.unique_syntax_labels.join(" | "),
        unique_semantic_dispositions: summary.unique_semantic_dispositions.join(" | "),
        unknown_count: summary.unknown_count,
        ambiguous_count: summary.ambiguous_count,
        disposition_flip_count: summary.disposition_flip_count,
        note: summary.note.clone(),
    }
}

fn figure_integrity_csv_row(check: &FigureIntegrityCheck) -> FigureIntegrityCsvRow {
    FigureIntegrityCsvRow {
        schema_version: check.schema_version.clone(),
        engine_version: check.engine_version.clone(),
        bank_version: check.bank_version.clone(),
        figure_id: check.figure_id.clone(),
        expected_panel_count: check.expected_panel_count,
        observed_panel_count: check.observed_panel_count,
        panel_labels: check.panel_labels.join(" | "),
        series_lengths: check
            .series_lengths
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(" | "),
        source_row_count: check.source_row_count,
        nonempty_series: check.nonempty_series,
        nonzero_values_present: check.nonzero_values_present,
        count_like_panels_integerlike: check.count_like_panels_integerlike,
        consistent_with_source: check.consistent_with_source,
        source_csv: check.source_csv.clone(),
        source_json: check.source_json.clone(),
        png_path: check.png_path.clone(),
        svg_path: check.svg_path.clone(),
        png_present: check.png_present,
        svg_present: check.svg_present,
        note: check.note.clone(),
    }
}

fn join_count_map(map: &BTreeMap<String, usize>) -> String {
    map.iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn value_at(values: &[f64], index: usize) -> Option<f64> {
    values.get(index).copied()
}
