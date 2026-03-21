use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::engine::bank::LoadedBankDescriptor;
use crate::engine::settings::EngineSettings;
use crate::evaluation::types::RunEvaluationBundle;

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GrammarReasonCode {
    Admissible,
    Boundary,
    RecurrentBoundaryGrazing,
    SustainedOutwardDrift,
    AbruptSlewViolation,
    EnvelopeViolation,
}

/// Deterministic trust scalar derived from grammar severity.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct TrustScalar(pub f64);

impl TrustScalar {
    /// Creates a bounded trust scalar in the unit interval.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Returns the scalar value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrammarStatus {
    pub scenario_id: String,
    pub step: usize,
    pub time: f64,
    pub state: GrammarState,
    pub reason_code: GrammarReasonCode,
    pub rule_category: String,
    pub reason_text: String,
    pub supporting_metric_summary: String,
    pub margin: f64,
    pub radius: f64,
    pub residual_norm: f64,
    pub trust_scalar: TrustScalar,
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
    /// Optional minimum grouped aggregate breach fraction required when grouped structure exists.
    pub min_coordinated_group_breach_fraction: Option<f64>,
    /// Optional maximum grouped aggregate breach fraction allowed for this motif.
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
    #[serde(default)]
    pub directional_incompatibility_exceptions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicCandidate {
    pub entry: HeuristicBankEntry,
    pub score: f64,
    /// Short list of the most relevant exported syntax metrics for this candidate.
    pub metric_highlights: Vec<String>,
    /// Explanation of how the grammar-state requirement for this candidate was satisfied.
    pub admissibility_explanation: String,
    /// Explanation of which regime tags were available and why the candidate was allowed.
    pub regime_explanation: String,
    /// Explanation of which syntax metrics satisfied the candidate's scope conditions.
    pub scope_explanation: String,
    /// Combined explanation used in reports and CSV exports.
    pub rationale: String,
    pub matched_regimes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticRetrievalAudit {
    /// Total typed bank entries considered before semantic filtering.
    pub heuristic_bank_entry_count: usize,
    /// Number of entries remaining after admissibility filtering.
    pub heuristic_candidates_post_admissibility: usize,
    /// Number of entries remaining after regime filtering.
    pub heuristic_candidates_post_regime: usize,
    /// Compatibility alias retained for outward export clarity; equals
    /// `heuristic_candidates_post_regime`.
    pub heuristic_candidates_pre_scope: usize,
    /// Number of entries remaining after scope filtering.
    pub heuristic_candidates_post_scope: usize,
    /// Number of entries rejected by admissibility requirements.
    pub heuristics_rejected_by_admissibility: usize,
    /// Number of entries rejected by regime requirements after admissibility passed.
    pub heuristics_rejected_by_regime: usize,
    /// Number of entries rejected by scope conditions after admissibility and regime passed.
    pub heuristics_rejected_by_scope: usize,
    /// Final number of selected heuristics carried in the semantic result.
    pub heuristics_selected_final: usize,
    /// Retrieval path used for this scenario (`linear` or `indexed`).
    pub retrieval_path: String,
    /// Number of candidates produced by the index prefilter before exact typed validation.
    pub prefilter_candidate_count: usize,
    /// Explicit IDs returned by the deterministic prefilter.
    pub prefilter_candidate_ids: Vec<String>,
    /// Number of index buckets consulted before exact typed validation.
    pub index_buckets_considered: usize,
    /// Explicit IDs that passed admissibility filtering.
    pub candidate_ids_post_admissibility: Vec<String>,
    /// Explicit IDs that passed regime filtering.
    pub candidate_ids_post_regime: Vec<String>,
    /// Explicit IDs that passed scope filtering.
    pub candidate_ids_post_scope: Vec<String>,
    /// Explicit IDs rejected at the admissibility stage.
    pub rejected_by_admissibility_ids: Vec<String>,
    /// Explicit IDs rejected at the regime stage.
    pub rejected_by_regime_ids: Vec<String>,
    /// Explicit IDs rejected at the scope stage.
    pub rejected_by_scope_ids: Vec<String>,
    /// Brief explanation of the filter-order semantics.
    pub note: String,
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
    /// Explicit deterministic retrieval counts and stage-wise filter membership.
    pub retrieval_audit: SemanticRetrievalAudit,
    pub candidates: Vec<HeuristicCandidate>,
    pub selected_labels: Vec<String>,
    pub selected_heuristic_ids: Vec<String>,
    pub resolution_basis: String,
    /// Coarse reason for an `Unknown` result, when one is returned.
    pub unknown_reason_class: Option<String>,
    /// More detailed explanation for `Unknown`, kept separate so reports can distinguish weak
    /// evidence from bank noncoverage explicitly.
    pub unknown_reason_detail: Option<String>,
    pub compatibility_note: String,
    /// Explicit pairwise compatibility notes when a `CompatibleSet` is returned.
    pub compatibility_reasons: Vec<String>,
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
    pub sweep_family: Option<String>,
    pub sweep_parameter_name: Option<String>,
    pub sweep_parameter_value: Option<f64>,
    pub sweep_secondary_parameter_name: Option<String>,
    pub sweep_secondary_parameter_value: Option<f64>,
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
    /// Additive machine-readable schema marker for exported run metadata.
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub run_configuration_hash: String,
    pub crate_name: String,
    pub crate_version: String,
    pub rust_version: Option<String>,
    pub git_commit: Option<String>,
    pub timestamp: String,
    pub input_mode: String,
    pub seed: u64,
    pub steps: usize,
    pub dt: f64,
    /// Deterministic engine settings captured with the run for future audit and replay.
    pub engine_settings: EngineSettings,
    /// Resolved heuristic-bank provenance for this run, including source and content hash.
    pub bank: LoadedBankDescriptor,
    /// Deterministic bounded online-history capacity used by the live/deployment-oriented path.
    pub online_history_buffer_capacity: usize,
    /// Numeric mode used by the deployment-oriented bounded-history path.
    pub numeric_mode: String,
    pub cli_args: Vec<String>,
    pub os: String,
    pub arch: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportManifest {
    /// Additive machine-readable schema marker for exported artifact manifests.
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub run_configuration_hash: String,
    pub crate_name: String,
    pub crate_version: String,
    pub timestamp: String,
    pub input_mode: String,
    pub online_history_buffer_capacity: usize,
    pub numeric_mode: String,
    pub bank: LoadedBankDescriptor,
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
    /// Evaluation summaries are kept separate from the engine-layer outputs.
    pub evaluation: RunEvaluationBundle,
    pub figure_artifacts: Vec<FigureArtifact>,
    pub reproducibility_check: ReproducibilityCheck,
    pub reproducibility_checks: Vec<ReproducibilityCheck>,
    pub reproducibility_summary: ReproducibilitySummary,
    pub report_manifest: Option<ReportManifest>,
    pub tabular_inventory: BTreeMap<String, Vec<String>>,
}
