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
    /// Deterministic projection method used for figure-oriented sign visualization.
    pub method: SignProjectionMethod,
    /// Human-readable labels for the three projected coordinates.
    pub axis_labels: [String; 3],
    /// Exact explanation of how the projected coordinates were constructed.
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
    /// Fraction of sampled times where margin evolution and radial drift indicate outward motion
    /// relative to the configured admissibility envelope.
    pub outward_drift_fraction: f64,
    /// Fraction of sampled times where margin evolution and radial drift indicate inward motion
    /// relative to the configured admissibility envelope.
    pub inward_drift_fraction: f64,
    /// Dominant nonzero sign share of the radial-drift sequence.
    /// Compatibility alias retained for existing artifacts; equals `radial_sign_dominance`.
    pub sign_consistency: f64,
    /// Adjacent-agreement fraction across nonzero radial-drift signs.
    /// Compatibility alias retained for existing artifacts; equals `radial_sign_persistence`.
    pub directional_persistence: f64,
    /// Mean within-sample sign alignment across drift channels.
    /// Compatibility alias retained for existing artifacts; equals
    /// `drift_channel_sign_alignment`.
    pub channel_coherence: f64,
    /// Net residual-norm change divided by total residual-norm variation.
    /// Compatibility alias retained for existing artifacts; equals
    /// `residual_norm_path_monotonicity`.
    pub aggregate_monotonicity: f64,
    /// Fraction of residual-norm increments aligned with the net residual-norm trend.
    /// Compatibility alias retained for existing artifacts; equals
    /// `residual_norm_trend_alignment`.
    pub monotone_drift_fraction: f64,
    /// Mean squared slew norm across the sampled trajectory.
    /// Compatibility alias retained for existing artifacts; equals `mean_squared_slew_norm`.
    pub curvature_energy: f64,
    /// Deterministic onset score derived from early-to-late slew-norm growth.
    /// Compatibility alias retained for existing artifacts; equals `late_slew_growth_score`.
    pub curvature_onset_score: f64,
    /// Dominant nonzero sign share of the radial-drift sequence.
    pub radial_sign_dominance: f64,
    /// Adjacent agreement fraction across the active nonzero radial-drift sign sequence.
    pub radial_sign_persistence: f64,
    /// Mean within-sample sign alignment across active drift channels.
    pub drift_channel_sign_alignment: f64,
    /// Net residual-norm change divided by total residual-norm path variation.
    pub residual_norm_path_monotonicity: f64,
    /// Fraction of nonzero residual-norm increments aligned with the net residual-norm trend.
    pub residual_norm_trend_alignment: f64,
    /// Mean squared slew norm across the sampled trajectory.
    pub mean_squared_slew_norm: f64,
    /// Deterministic score derived from early-to-late slew-norm growth.
    pub late_slew_growth_score: f64,
    /// Mean radial drift `dot(r,d)/||r||`, with zero reported at exact zero residual norm.
    pub mean_radial_drift: f64,
    /// Smallest residual-envelope margin observed along the trajectory.
    pub min_margin: f64,
    /// Mean derivative of the residual-envelope margin over sampled times.
    pub mean_margin_delta: f64,
    /// Maximum slew norm observed across the trajectory.
    pub max_slew_norm: f64,
    /// Count of slew-norm samples above the deterministic spike threshold.
    pub slew_spike_count: usize,
    /// Average positive excess of slew norm above the deterministic spike threshold.
    pub slew_spike_strength: f64,
    /// Number of distinct contiguous boundary episodes.
    pub boundary_grazing_episode_count: usize,
    /// Number of returns from non-admissible states back to admissible states.
    pub boundary_recovery_count: usize,
    /// Boundary episode count minus one; used as a compact repeated-grazing indicator.
    pub repeated_grazing_count: usize,
    /// Fraction of grouped aggregate residual points with negative aggregate margin.
    /// Reported as zero when no grouped structure is configured.
    pub coordinated_group_breach_fraction: f64,
    /// Compact deterministic summary label derived from the richer syntax metrics.
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
    pub max_inward_drift_fraction: Option<f64>,
    pub max_curvature_energy: Option<f64>,
    pub min_curvature_energy: Option<f64>,
    pub max_curvature_onset_score: Option<f64>,
    pub min_curvature_onset_score: Option<f64>,
    pub min_directional_persistence: Option<f64>,
    pub min_sign_consistency: Option<f64>,
    pub min_channel_coherence: Option<f64>,
    pub min_aggregate_monotonicity: Option<f64>,
    pub max_aggregate_monotonicity: Option<f64>,
    pub min_slew_spike_count: Option<usize>,
    pub max_slew_spike_count: Option<usize>,
    pub min_slew_spike_strength: Option<f64>,
    pub max_slew_spike_strength: Option<f64>,
    pub min_boundary_grazing_episodes: Option<usize>,
    pub max_boundary_grazing_episodes: Option<usize>,
    pub min_boundary_recovery_count: Option<usize>,
    pub max_coordinated_group_breach_fraction: Option<f64>,
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
    pub short_label: String,
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
    pub admissibility_explanation: String,
    pub regime_explanation: String,
    pub scope_explanation: String,
    pub rationale: String,
    pub matched_regimes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SemanticDisposition {
    Match,
    CompatibleSet,
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
    pub selected_heuristic_ids: Vec<String>,
    pub resolution_basis: String,
    pub unknown_reason_class: Option<String>,
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
    pub data_origin: String,
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
    pub input_mode: String,
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
    pub input_mode: String,
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
