//! Deterministic pipeline core orchestration for scenario preparation, layered execution, and run metadata.

use std::collections::BTreeMap;
use std::path::PathBuf;
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
use crate::engine::pipeline_evaluation::{compare_outputs, summarize_reproducibility};
use crate::engine::residual_layer::extract_residuals;
use crate::engine::semantics_layer::retrieve_semantics_with_registry;
use crate::engine::settings::EngineSettings;
use crate::engine::sign_layer::construct_signs;
use crate::engine::syntax_layer::characterize_syntax_with_coordination_configured;
use crate::engine::types::{
    CoordinatedResidualStructure, DetectabilityBoundInputs, EngineOutputBundle, GroupDefinition,
    GroupResidualPoint, ObservedTrajectory, PredictedTrajectory, ResidualTrajectory, RunMetadata,
    ScenarioOutput, ScenarioRecord,
};
use crate::evaluation::evaluate_bundle;
use crate::evaluation::sweeps::{generate_sweep_members, SweepConfig, SweepMemberDefinition};
use crate::io::input::load_csv_trajectories;
use crate::io::output::create_output_layout;
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use crate::math::envelope::{build_envelope, EnvelopeSpec};
use crate::math::metrics::{format_metric, hash_serializable_hex, max_abs, pairwise_abs_mean};
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
            &self.config,
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
            figure_artifacts: Vec::new(),
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
            figure_artifacts: Vec::new(),
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

fn run_metadata(
    timestamp: &str,
    config: &EngineConfig,
    settings: &EngineSettings,
    bank: &LoadedBankDescriptor,
) -> RunMetadata {
    let input_mode = input_mode_label(&config.scenario_selection);
    let run_configuration_hash = run_configuration_hash(config, settings, bank);
    RunMetadata {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        bank_version: bank.bank_version.clone(),
        run_configuration_hash,
        crate_name: "dsfb-semiotics-engine".to_string(),
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_version: command_stdout("rustc", &["--version"]),
        git_commit: command_stdout("git", &["rev-parse", "HEAD"]),
        timestamp: timestamp.to_string(),
        input_mode: input_mode.to_string(),
        seed: config.seed,
        steps: config.steps,
        dt: config.dt,
        engine_settings: settings.clone(),
        bank: bank.clone(),
        online_history_buffer_capacity: settings.online.history_buffer_capacity,
        numeric_mode: settings.online.numeric_mode.clone(),
        cli_args: std::env::args().collect(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

#[derive(Serialize)]
struct RunConfigurationIdentity<'a> {
    input_mode: &'a str,
    selection_identity: String,
    seed: u64,
    steps: usize,
    dt: f64,
    engine_settings: &'a EngineSettings,
    bank_source_kind: &'a str,
    bank_source_path: &'a Option<String>,
    bank_version: &'a str,
    validation_mode: &'a str,
}

fn run_configuration_hash(
    config: &EngineConfig,
    settings: &EngineSettings,
    bank: &LoadedBankDescriptor,
) -> String {
    let identity = RunConfigurationIdentity {
        input_mode: input_mode_label(&config.scenario_selection),
        selection_identity: selection_identity(&config.scenario_selection),
        seed: config.seed,
        steps: config.steps,
        dt: config.dt,
        engine_settings: settings,
        bank_source_kind: bank.source_kind.as_label(),
        bank_source_path: &bank.source_path,
        bank_version: &bank.bank_version,
        validation_mode: &bank.validation_mode,
    };
    hash_serializable_hex("run_configuration", &identity)
        .map(|digest| digest.fnv1a_64_hex)
        .unwrap_or_else(|_| "hash-unavailable".to_string())
}

fn selection_identity(selection: &ScenarioSelection) -> String {
    match selection {
        ScenarioSelection::All => "synthetic:all".to_string(),
        ScenarioSelection::Single(id) => format!("synthetic:single:{id}"),
        ScenarioSelection::Sweep(config) => {
            format!(
                "synthetic:sweep:{}:{}",
                config.family.as_str(),
                config.points
            )
        }
        ScenarioSelection::Csv(config) => {
            let channel_identity = config
                .channel_names
                .as_ref()
                .map(|names| names.join("|"))
                .unwrap_or_else(|| "<headers>".to_string());
            let envelope_mode = format!("{:?}", config.envelope_mode);
            let envelope_base = format_metric(config.envelope_base);
            let envelope_slope = format_metric(config.envelope_slope);
            format!(
                "csv:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{:?}",
                config.scenario_id,
                config.observed_csv.display(),
                config.predicted_csv.display(),
                config.time_column.as_deref().unwrap_or("<none>"),
                channel_identity,
                config.envelope_name,
                envelope_mode,
                envelope_base,
                envelope_slope,
                config.envelope_switch_step,
                config.envelope_secondary_slope,
                config.envelope_secondary_base
            )
        }
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
