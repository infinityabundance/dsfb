use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorSample {
    pub step: usize,
    pub time: f64,
    pub values: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservedTrajectory {
    pub scenario_id: String,
    pub channel_names: Vec<String>,
    pub samples: Vec<VectorSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictedTrajectory {
    pub scenario_id: String,
    pub channel_names: Vec<String>,
    pub samples: Vec<VectorSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResidualSample {
    pub step: usize,
    pub time: f64,
    pub values: Vec<f64>,
    pub norm: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DriftSample {
    pub step: usize,
    pub time: f64,
    pub values: Vec<f64>,
    pub norm: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlewSample {
    pub step: usize,
    pub time: f64,
    pub values: Vec<f64>,
    pub norm: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResidualTrajectory {
    pub scenario_id: String,
    pub channel_names: Vec<String>,
    pub samples: Vec<ResidualSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DriftTrajectory {
    pub scenario_id: String,
    pub channel_names: Vec<String>,
    pub samples: Vec<DriftSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlewTrajectory {
    pub scenario_id: String,
    pub channel_names: Vec<String>,
    pub samples: Vec<SlewSample>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EnvelopeMode {
    Fixed,
    Widening,
    Tightening,
    RegimeSwitched,
    Aggregate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvelopeSample {
    pub step: usize,
    pub time: f64,
    pub radius: f64,
    pub derivative_bound: f64,
    pub regime: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdmissibilityEnvelope {
    pub scenario_id: String,
    pub name: String,
    pub mode: EnvelopeMode,
    pub samples: Vec<EnvelopeSample>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrammarState {
    Admissible,
    Boundary,
    Violation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrammarStatus {
    pub scenario_id: String,
    pub step: usize,
    pub time: f64,
    pub state: GrammarState,
    pub margin: f64,
    pub radius: f64,
    pub residual_norm: f64,
    pub regime: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignSample {
    pub step: usize,
    pub time: f64,
    pub residual: Vec<f64>,
    pub drift: Vec<f64>,
    pub slew: Vec<f64>,
    pub residual_norm: f64,
    pub drift_norm: f64,
    pub slew_norm: f64,
    pub projection: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SignProjectionMethod {
    AggregateNormSignedRadialDrift,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignProjectionMetadata {
    pub method: SignProjectionMethod,
    pub axis_labels: [String; 3],
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignTrajectory {
    pub scenario_id: String,
    pub channel_names: Vec<String>,
    pub projection_metadata: SignProjectionMetadata,
    pub samples: Vec<SignSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyntaxCharacterization {
    pub scenario_id: String,
    pub outward_drift_fraction: f64,
    pub inward_drift_fraction: f64,
    pub sign_consistency: f64,
    pub directional_persistence: f64,
    pub channel_coherence: f64,
    pub aggregate_monotonicity: f64,
    pub monotone_drift_fraction: f64,
    pub curvature_energy: f64,
    pub mean_radial_drift: f64,
    pub min_margin: f64,
    pub mean_margin_delta: f64,
    pub max_slew_norm: f64,
    pub slew_spike_count: usize,
    pub boundary_grazing_episode_count: usize,
    pub repeated_grazing_count: usize,
    pub trajectory_label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectabilityBoundInputs {
    pub t0: f64,
    pub alpha: f64,
    pub kappa: f64,
    pub delta0: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectabilityResult {
    pub scenario_id: String,
    pub observed_crossing_step: Option<usize>,
    pub observed_crossing_time: Option<f64>,
    pub predicted_upper_bound: Option<f64>,
    pub bound_satisfied: Option<bool>,
    pub separation_at_exit: Option<f64>,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicScopeConditions {
    pub min_outward_drift_fraction: Option<f64>,
    pub max_outward_drift_fraction: Option<f64>,
    pub min_inward_drift_fraction: Option<f64>,
    pub max_curvature_energy: Option<f64>,
    pub min_curvature_energy: Option<f64>,
    pub min_directional_persistence: Option<f64>,
    pub min_sign_consistency: Option<f64>,
    pub min_channel_coherence: Option<f64>,
    pub min_aggregate_monotonicity: Option<f64>,
    pub min_slew_spike_count: Option<usize>,
    pub min_boundary_grazing_episodes: Option<usize>,
    pub require_group_breach: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AdmissibilityRequirement {
    Any,
    BoundaryInteraction,
    ViolationRequired,
    NoViolation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicProvenance {
    pub source: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicBankEntry {
    pub heuristic_id: String,
    pub motif_label: String,
    pub scope_conditions: HeuristicScopeConditions,
    pub admissibility_requirements: AdmissibilityRequirement,
    pub regime_tags: Vec<String>,
    pub provenance: HeuristicProvenance,
    pub applicability_note: String,
    pub retrieval_priority: u32,
    pub compatible_with: Vec<String>,
    pub incompatible_with: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicCandidate {
    pub entry: HeuristicBankEntry,
    pub score: f64,
    pub rationale: String,
    pub matched_regimes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SemanticDisposition {
    Match,
    Ambiguous,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticMatchResult {
    pub scenario_id: String,
    pub disposition: SemanticDisposition,
    pub motif_summary: String,
    pub candidates: Vec<HeuristicCandidate>,
    pub selected_labels: Vec<String>,
    pub compatibility_note: String,
    pub conflict_notes: Vec<String>,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupDefinition {
    pub group_id: String,
    pub member_indices: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupResidualPoint {
    pub scenario_id: String,
    pub group_id: String,
    pub step: usize,
    pub time: f64,
    pub aggregate_abs_mean: f64,
    pub local_max_abs: f64,
    pub aggregate_radius: f64,
    pub aggregate_margin: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoordinatedResidualStructure {
    pub scenario_id: String,
    pub groups: Vec<GroupDefinition>,
    pub points: Vec<GroupResidualPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioRecord {
    pub id: String,
    pub title: String,
    pub purpose: String,
    pub theorem_alignment: String,
    pub claim_class: String,
    pub limitations: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioOutput {
    pub record: ScenarioRecord,
    pub observed: ObservedTrajectory,
    pub predicted: PredictedTrajectory,
    pub residual: ResidualTrajectory,
    pub drift: DriftTrajectory,
    pub slew: SlewTrajectory,
    pub sign: SignTrajectory,
    pub envelope: AdmissibilityEnvelope,
    pub grammar: Vec<GrammarStatus>,
    pub syntax: SyntaxCharacterization,
    pub detectability: DetectabilityResult,
    pub semantics: SemanticMatchResult,
    pub coordinated: Option<CoordinatedResidualStructure>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FigureArtifact {
    pub figure_id: String,
    pub caption: String,
    pub png_path: String,
    pub svg_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReproducibilityCheck {
    pub scenario_id: String,
    pub first_hash: String,
    pub second_hash: String,
    pub identical: bool,
    pub materialized_components: Vec<String>,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReproducibilitySummary {
    pub scenario_count: usize,
    pub identical_count: usize,
    pub all_identical: bool,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunMetadata {
    pub crate_name: String,
    pub crate_version: String,
    pub rust_version: Option<String>,
    pub git_commit: Option<String>,
    pub timestamp: String,
    pub seed: u64,
    pub steps: usize,
    pub dt: f64,
    pub cli_args: Vec<String>,
    pub os: String,
    pub arch: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportManifest {
    pub crate_name: String,
    pub crate_version: String,
    pub timestamp: String,
    pub run_dir: String,
    pub report_markdown: String,
    pub report_pdf: String,
    pub zip_archive: String,
    pub figure_paths: Vec<String>,
    pub csv_paths: Vec<String>,
    pub json_paths: Vec<String>,
    pub scenario_ids: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineOutputBundle {
    pub run_metadata: RunMetadata,
    pub run_dir: PathBuf,
    pub scenario_outputs: Vec<ScenarioOutput>,
    pub figure_artifacts: Vec<FigureArtifact>,
    pub reproducibility_check: ReproducibilityCheck,
    pub reproducibility_checks: Vec<ReproducibilityCheck>,
    pub reproducibility_summary: ReproducibilitySummary,
    pub report_manifest: Option<ReportManifest>,
    pub tabular_inventory: BTreeMap<String, Vec<String>>,
}
