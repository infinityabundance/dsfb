use crate::metrics::{BenchmarkMetrics, MotifMetric};
use serde::{Deserialize, Serialize};

pub const PRE_FAILURE_SLOW_DRIFT: &str = "pre_failure_slow_drift";
pub const TRANSIENT_EXCURSION: &str = "transient_excursion";
pub const TRANSITION_EXCURSION: &str = "transition_excursion";
pub const RECURRENT_BOUNDARY_APPROACH: &str = "recurrent_boundary_approach";
pub const PERSISTENT_INSTABILITY_CLUSTER: &str = "persistent_instability_cluster";
pub const TRANSITION_CLUSTER_SUPPORT: &str = "transition_cluster_support";
pub const WATCH_ONLY_BOUNDARY_GRAZING: &str = "watch_only_boundary_grazing";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeuristicAlertClass {
    Silent,
    Watch,
    Review,
    Escalate,
}

impl HeuristicAlertClass {
    pub fn as_lowercase(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::Watch => "watch",
            Self::Review => "review",
            Self::Escalate => "escalate",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HeuristicPolicyDefinition {
    pub motif_name: &'static str,
    pub signature_definition: &'static str,
    pub grammar_constraints: &'static str,
    pub regime_conditions: &'static str,
    pub applicability_rules: &'static str,
    pub interpretation: &'static str,
    pub alert_class_default: HeuristicAlertClass,
    pub requires_persistence: bool,
    pub requires_corroboration: bool,
    pub minimum_window: usize,
    pub minimum_hits: usize,
    pub recommended_action: &'static str,
    pub escalation_policy: &'static str,
    pub non_unique_warning: &'static str,
    pub known_limitations: &'static str,
    pub contributes_to_dsa: bool,
    pub suppresses_alert: bool,
    pub promotes_alert: bool,
}

impl HeuristicPolicyDefinition {
    pub fn maximum_allowed_fragmentation(self) -> f64 {
        1.0 / self.minimum_hits.max(1) as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeaturePolicyOverride {
    pub feature_index: usize,
    pub feature_name: String,
    pub alert_class_override: Option<HeuristicAlertClass>,
    pub requires_persistence_override: Option<bool>,
    pub requires_corroboration_override: Option<bool>,
    pub minimum_window_override: Option<usize>,
    pub minimum_hits_override: Option<usize>,
    pub maximum_allowed_fragmentation_override: Option<f64>,
    pub rescue_eligible: bool,
    pub rescue_priority: usize,
    pub allow_watch_only: Option<bool>,
    pub allow_review_without_escalate: Option<bool>,
    pub suppress_if_isolated: Option<bool>,
    pub override_reason: String,
}

const POLICY_DEFINITIONS: &[HeuristicPolicyDefinition] = &[
    HeuristicPolicyDefinition {
        motif_name: PRE_FAILURE_SLOW_DRIFT,
        signature_definition:
            "Residual norm exceeds 0.5*rho with drift above the healthy-window drift threshold.",
        grammar_constraints:
            "grammar_state=Boundary and grammar_reason=SustainedOutwardDrift",
        regime_conditions:
            "Outward drift remains thresholded and curvature stays below the abrupt-slew regime.",
        applicability_rules:
            "Apply only after grammar filtering confirms boundary proximity without direct envelope exit.",
        interpretation:
            "Candidate DSA-compatible drift motif that supports closer monitoring or maintenance review.",
        alert_class_default: HeuristicAlertClass::Review,
        requires_persistence: true,
        requires_corroboration: false,
        minimum_window: 5,
        minimum_hits: 2,
        recommended_action:
            "Increase review cadence, inspect neighboring channels, and corroborate with process context before intervention.",
        escalation_policy:
            "Escalate when the motif persists across repeated runs or is corroborated by scalar alarms and engineering context.",
        non_unique_warning:
            "This motif is not mechanism-specific and may reflect multiple latent causes.",
        known_limitations:
            "SECOM is anonymized and instance-level, so this motif does not support chamber-level attribution on its own.",
        contributes_to_dsa: true,
        suppresses_alert: false,
        promotes_alert: true,
    },
    HeuristicPolicyDefinition {
        motif_name: TRANSIENT_EXCURSION,
        signature_definition:
            "Residual norm enters the boundary zone with slew above the healthy-window slew threshold.",
        grammar_constraints:
            "grammar_state in {Boundary, Violation} and grammar_reason=AbruptSlewViolation",
        regime_conditions:
            "Curvature dominates the local sign tuple during a non-admissible excursion.",
        applicability_rules:
            "Apply only after grammar filtering confirms abrupt boundary interaction.",
        interpretation:
            "Compatible with transient upset or abrupt regime change, but not uniquely diagnostic.",
        alert_class_default: HeuristicAlertClass::Silent,
        requires_persistence: true,
        requires_corroboration: true,
        minimum_window: 5,
        minimum_hits: 2,
        recommended_action:
            "Check for corroborating context, inspect neighboring channels, and prefer confirmation over immediate intervention.",
        escalation_policy:
            "Escalate only when repeated, clustered with other motifs, or accompanied by direct envelope violations.",
        non_unique_warning:
            "A transient excursion can reflect measurement noise, regime switch, or genuine degradation.",
        known_limitations:
            "A single abrupt excursion does not identify physical cause and may not persist long enough for confident attribution.",
        contributes_to_dsa: true,
        suppresses_alert: true,
        promotes_alert: true,
    },
    HeuristicPolicyDefinition {
        motif_name: RECURRENT_BOUNDARY_APPROACH,
        signature_definition:
            "Residual norm revisits the boundary zone repeatedly without a confirmed envelope exit.",
        grammar_constraints:
            "grammar_state=Boundary and grammar_reason=RecurrentBoundaryGrazing",
        regime_conditions:
            "Boundary revisitation persists without direct envelope exit and without stable violation.",
        applicability_rules:
            "Apply only after grammar filtering confirms repeated boundary approach under the local envelope.",
        interpretation:
            "Ambiguous DSA motif that warrants continued observation rather than decisive attribution.",
        alert_class_default: HeuristicAlertClass::Watch,
        requires_persistence: true,
        requires_corroboration: true,
        minimum_window: 10,
        minimum_hits: 3,
        recommended_action:
            "Track persistence, compare against the scalar baselines, and prioritize manual review over automatic maintenance action.",
        escalation_policy:
            "Escalate when recurrent grazing concentrates in pre-failure windows or transitions into direct violations.",
        non_unique_warning:
            "Repeated boundary grazing can arise from nuisance variation as well as meaningful DSA structure.",
        known_limitations:
            "This motif is especially sensitive to envelope and drift thresholds, so calibration materially affects its prevalence.",
        contributes_to_dsa: true,
        suppresses_alert: true,
        promotes_alert: true,
    },
];

const EXPANDED_POLICY_DEFINITIONS: &[HeuristicPolicyDefinition] = &[
    HeuristicPolicyDefinition {
        motif_name: TRANSITION_EXCURSION,
        signature_definition:
            "Grammar-qualified transition motif with elevated slew, non-admissible envelope interaction, and abrupt state change.",
        grammar_constraints:
            "grammar_state in {TransientViolation, PersistentViolation} and grammar_reason=AbruptSlewViolation",
        regime_conditions:
            "Curvature dominates the sign tuple while the trajectory departs admissibility.",
        applicability_rules:
            "Apply only after grammar filtering confirms abrupt transition pressure at the envelope.",
        interpretation:
            "Candidate transition-instability event with elevated structural salience but ambiguous physical cause.",
        alert_class_default: HeuristicAlertClass::Review,
        requires_persistence: false,
        requires_corroboration: false,
        minimum_window: 3,
        minimum_hits: 1,
        recommended_action:
            "Inspect adjacent channels and grouped corroborators before promoting beyond Review.",
        escalation_policy:
            "Escalate only when the transition persists, repeats, or aligns with grouped corroboration.",
        non_unique_warning:
            "A transition excursion is not a unique fault signature and may reflect multiple process changes.",
        known_limitations:
            "SECOM does not expose mechanism labels, so this motif remains interpretive rather than causal.",
        contributes_to_dsa: false,
        suppresses_alert: false,
        promotes_alert: true,
    },
    HeuristicPolicyDefinition {
        motif_name: PERSISTENT_INSTABILITY_CLUSTER,
        signature_definition:
            "Repeated or sustained outward grammar pressure that is not reducible to isolated spikes.",
        grammar_constraints:
            "grammar_state in {SustainedOutwardDrift, PersistentViolation}",
        regime_conditions:
            "Non-admissible pressure recurs across neighboring runs with bounded fragmentation.",
        applicability_rules:
            "Apply only after grammar filtering confirms sustained pressure rather than single-point excursions.",
        interpretation:
            "Candidate persistent instability regime with potential operator significance if corroborated.",
        alert_class_default: HeuristicAlertClass::Review,
        requires_persistence: true,
        requires_corroboration: true,
        minimum_window: 5,
        minimum_hits: 2,
        recommended_action:
            "Review grouped corroborators, inspect adjacent precursor channels, and preserve ambiguity explicitly.",
        escalation_policy:
            "Escalate only when grouped corroboration or persistent violation confirms sustained structure.",
        non_unique_warning:
            "Persistent instability remains semantically ambiguous and does not identify a unique root cause.",
        known_limitations:
            "This motif is sensitive to persistence choices and grouped corroboration windows.",
        contributes_to_dsa: false,
        suppresses_alert: false,
        promotes_alert: true,
    },
    HeuristicPolicyDefinition {
        motif_name: TRANSITION_CLUSTER_SUPPORT,
        signature_definition:
            "Corroborating burst or boundary-pressure feature that aligns temporally with a primary precursor feature.",
        grammar_constraints:
            "grammar_state in {BoundaryGrazing, SustainedOutwardDrift, TransientViolation}",
        regime_conditions:
            "Supportive structure is temporally aligned with a grouped primary feature rather than isolated.",
        applicability_rules:
            "Apply only after grammar filtering and grouped temporal alignment confirm corroboration.",
        interpretation:
            "Supportive corroborator motif that increases confidence in another feature but is not decisive alone.",
        alert_class_default: HeuristicAlertClass::Watch,
        requires_persistence: false,
        requires_corroboration: true,
        minimum_window: 3,
        minimum_hits: 1,
        recommended_action:
            "Use as corroboration support; do not escalate on this motif alone.",
        escalation_policy:
            "Never escalate solely from support motifs without a primary precursor feature.",
        non_unique_warning:
            "Corroboration support indicates temporal alignment, not causal identity.",
        known_limitations:
            "Grouped alignment is deterministic but remains a limited surrogate for true mechanism coupling.",
        contributes_to_dsa: false,
        suppresses_alert: true,
        promotes_alert: true,
    },
    HeuristicPolicyDefinition {
        motif_name: WATCH_ONLY_BOUNDARY_GRAZING,
        signature_definition:
            "Boundary proximity without sufficient persistence or corroboration for Review promotion.",
        grammar_constraints: "grammar_state=BoundaryGrazing",
        regime_conditions:
            "Admissibility pressure is visible but remains weak, isolated, or sentinel-like.",
        applicability_rules:
            "Apply only after grammar filtering confirms envelope grazing without sustained outward drift.",
        interpretation:
            "Low-amplitude sentinel signal appropriate for Watch-only handling.",
        alert_class_default: HeuristicAlertClass::Watch,
        requires_persistence: false,
        requires_corroboration: true,
        minimum_window: 3,
        minimum_hits: 1,
        recommended_action:
            "Retain as Watch-only and wait for corroboration before manual investigation.",
        escalation_policy:
            "Do not escalate directly from boundary grazing without stronger semantic support.",
        non_unique_warning:
            "Boundary grazing alone is structurally ambiguous and often nuisance-dominated.",
        known_limitations:
            "This motif deliberately favors burden suppression over coverage recovery when isolated.",
        contributes_to_dsa: false,
        suppresses_alert: true,
        promotes_alert: false,
    },
];

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicEntry {
    pub motif_name: String,
    pub signature_definition: String,
    pub grammar_constraints: String,
    pub regime_conditions: String,
    pub applicability_rules: String,
    pub applicable_dataset: String,
    pub provenance_status: String,
    pub interpretation: String,
    pub severity: String,
    pub confidence: String,
    pub alert_class_default: HeuristicAlertClass,
    pub requires_persistence: bool,
    pub requires_corroboration: bool,
    pub minimum_window: usize,
    pub minimum_hits: usize,
    pub maximum_allowed_fragmentation: f64,
    pub recommended_action: String,
    pub escalation_policy: String,
    pub non_unique_warning: String,
    pub known_limitations: String,
    pub contributes_to_dsa_scoring: bool,
    pub contributes_to_dsa: bool,
    pub suppresses_alert: bool,
    pub promotes_alert: bool,
    pub observed_point_hits: usize,
    pub observed_run_hits: usize,
    pub pre_failure_window_run_hits: usize,
    pub pre_failure_window_precision_proxy: Option<f64>,
    pub status_note: String,
}

pub fn dsa_contributing_motif_names() -> &'static [&'static str] {
    &[
        PRE_FAILURE_SLOW_DRIFT,
        TRANSIENT_EXCURSION,
        RECURRENT_BOUNDARY_APPROACH,
    ]
}

pub fn heuristic_policy_definitions() -> &'static [HeuristicPolicyDefinition] {
    POLICY_DEFINITIONS
}

pub fn heuristic_policy_definition(motif_name: &str) -> Option<HeuristicPolicyDefinition> {
    POLICY_DEFINITIONS
        .iter()
        .copied()
        .chain(EXPANDED_POLICY_DEFINITIONS.iter().copied())
        .find(|definition| definition.motif_name == motif_name)
}

pub fn expanded_semantic_policy_definitions() -> Vec<HeuristicPolicyDefinition> {
    POLICY_DEFINITIONS
        .iter()
        .copied()
        .chain(EXPANDED_POLICY_DEFINITIONS.iter().copied())
        .collect()
}

pub fn build_heuristics_bank(
    metrics: &BenchmarkMetrics,
    dataset_name: &str,
) -> Vec<HeuristicEntry> {
    POLICY_DEFINITIONS
        .iter()
        .map(|definition| {
            let metric = motif(metrics, definition.motif_name);
            HeuristicEntry {
                motif_name: definition.motif_name.into(),
                signature_definition: definition.signature_definition.into(),
                grammar_constraints: definition.grammar_constraints.into(),
                regime_conditions: definition.regime_conditions.into(),
                applicability_rules: definition.applicability_rules.into(),
                applicable_dataset: dataset_name.into(),
                provenance_status: observed_status(metric),
                interpretation: definition.interpretation.into(),
                severity: definition.alert_class_default.as_lowercase().into(),
                confidence: confidence_note(metric),
                alert_class_default: definition.alert_class_default,
                requires_persistence: definition.requires_persistence,
                requires_corroboration: definition.requires_corroboration,
                minimum_window: definition.minimum_window,
                minimum_hits: definition.minimum_hits,
                maximum_allowed_fragmentation: definition.maximum_allowed_fragmentation(),
                recommended_action: definition.recommended_action.into(),
                escalation_policy: definition.escalation_policy.into(),
                non_unique_warning: definition.non_unique_warning.into(),
                known_limitations: definition.known_limitations.into(),
                contributes_to_dsa_scoring: definition.contributes_to_dsa,
                contributes_to_dsa: definition.contributes_to_dsa,
                suppresses_alert: definition.suppresses_alert,
                promotes_alert: definition.promotes_alert,
                observed_point_hits: metric.point_hits,
                observed_run_hits: metric.run_hits,
                pre_failure_window_run_hits: metric.pre_failure_window_run_hits,
                pre_failure_window_precision_proxy: metric.pre_failure_window_precision_proxy,
                status_note: format!(
                    "Observed {} points and {} run hits; {} of those run hits fall inside the configured pre-failure windows. Default alert class is {} with minimum_window={}, minimum_hits={}, and maximum_allowed_fragmentation={:.4}.",
                    metric.point_hits,
                    metric.run_hits,
                    metric.pre_failure_window_run_hits,
                    definition.alert_class_default.as_lowercase(),
                    definition.minimum_window,
                    definition.minimum_hits,
                    definition.maximum_allowed_fragmentation(),
                ),
            }
        })
        .collect()
}

fn motif<'a>(metrics: &'a BenchmarkMetrics, motif_name: &str) -> &'a MotifMetric {
    metrics
        .motif_metrics
        .iter()
        .find(|metric| metric.motif_name == motif_name)
        .unwrap_or_else(|| panic!("missing motif metric for {motif_name}"))
}

fn observed_status(metric: &MotifMetric) -> String {
    if metric.point_hits > 0 {
        "SECOM-observed".into()
    } else {
        "framework-defined".into()
    }
}

fn confidence_note(metric: &MotifMetric) -> String {
    if metric.point_hits > 0 {
        "Stage-II observed on SECOM; interpretive and non-mechanistic.".into()
    } else {
        "Framework-defined only; not yet observed in the current run.".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{
        BenchmarkMetrics, BoundaryEpisodeSummary, DensitySummary, LeadTimeSummary,
    };
    use crate::preprocessing::DatasetSummary;

    fn sample_metrics() -> BenchmarkMetrics {
        BenchmarkMetrics {
            summary: crate::metrics::BenchmarkSummary {
                dataset_summary: DatasetSummary {
                    run_count: 10,
                    feature_count: 3,
                    pass_count: 8,
                    fail_count: 2,
                    dataset_missing_fraction: 0.0,
                    healthy_pass_runs_requested: 3,
                    healthy_pass_runs_found: 3,
                },
                analyzable_feature_count: 3,
                grammar_imputation_suppression_points: 0,
                threshold_alarm_points: 0,
                ewma_alarm_points: 0,
                cusum_alarm_points: 0,
                run_energy_alarm_points: 0,
                pca_fdc_alarm_points: 0,
                dsfb_raw_boundary_points: 0,
                dsfb_persistent_boundary_points: 0,
                dsfb_raw_violation_points: 0,
                dsfb_persistent_violation_points: 0,
                failure_runs: 2,
                failure_runs_with_preceding_dsfb_raw_signal: 0,
                failure_runs_with_preceding_dsfb_persistent_signal: 0,
                failure_runs_with_preceding_dsfb_raw_boundary_signal: 0,
                failure_runs_with_preceding_dsfb_persistent_boundary_signal: 0,
                failure_runs_with_preceding_dsfb_raw_violation_signal: 0,
                failure_runs_with_preceding_dsfb_persistent_violation_signal: 0,
                failure_runs_with_preceding_ewma_signal: 0,
                failure_runs_with_preceding_cusum_signal: 0,
                failure_runs_with_preceding_run_energy_signal: 0,
                failure_runs_with_preceding_pca_fdc_signal: 0,
                failure_runs_with_preceding_threshold_signal: 0,
                pass_runs: 8,
                pass_runs_with_dsfb_raw_boundary_signal: 0,
                pass_runs_with_dsfb_persistent_boundary_signal: 0,
                pass_runs_with_dsfb_raw_violation_signal: 0,
                pass_runs_with_dsfb_persistent_violation_signal: 0,
                pass_runs_with_ewma_signal: 0,
                pass_runs_with_cusum_signal: 0,
                pass_runs_with_run_energy_signal: 0,
                pass_runs_with_pca_fdc_signal: 0,
                pass_runs_with_threshold_signal: 0,
                pass_run_dsfb_raw_boundary_nuisance_rate: 0.0,
                pass_run_dsfb_persistent_boundary_nuisance_rate: 0.0,
                pass_run_dsfb_raw_violation_nuisance_rate: 0.0,
                pass_run_dsfb_persistent_violation_nuisance_rate: 0.0,
                pass_run_ewma_nuisance_rate: 0.0,
                pass_run_cusum_nuisance_rate: 0.0,
                pass_run_run_energy_nuisance_rate: 0.0,
                pass_run_pca_fdc_nuisance_rate: 0.0,
                pass_run_threshold_nuisance_rate: 0.0,
            },
            lead_time_summary: LeadTimeSummary {
                failure_runs_with_raw_boundary_lead: 0,
                failure_runs_with_persistent_boundary_lead: 0,
                failure_runs_with_raw_violation_lead: 0,
                failure_runs_with_persistent_violation_lead: 0,
                failure_runs_with_threshold_lead: 0,
                failure_runs_with_ewma_lead: 0,
                failure_runs_with_cusum_lead: 0,
                failure_runs_with_run_energy_lead: 0,
                failure_runs_with_pca_fdc_lead: 0,
                mean_raw_boundary_lead_runs: None,
                mean_persistent_boundary_lead_runs: None,
                mean_raw_violation_lead_runs: None,
                mean_persistent_violation_lead_runs: None,
                mean_threshold_lead_runs: None,
                mean_ewma_lead_runs: None,
                mean_cusum_lead_runs: None,
                mean_run_energy_lead_runs: None,
                mean_pca_fdc_lead_runs: None,
                mean_raw_boundary_minus_cusum_delta_runs: None,
                mean_raw_boundary_minus_run_energy_delta_runs: None,
                mean_raw_boundary_minus_pca_fdc_delta_runs: None,
                mean_raw_boundary_minus_threshold_delta_runs: None,
                mean_raw_boundary_minus_ewma_delta_runs: None,
                mean_persistent_boundary_minus_cusum_delta_runs: None,
                mean_persistent_boundary_minus_run_energy_delta_runs: None,
                mean_persistent_boundary_minus_pca_fdc_delta_runs: None,
                mean_persistent_boundary_minus_threshold_delta_runs: None,
                mean_persistent_boundary_minus_ewma_delta_runs: None,
                mean_raw_violation_minus_cusum_delta_runs: None,
                mean_raw_violation_minus_run_energy_delta_runs: None,
                mean_raw_violation_minus_pca_fdc_delta_runs: None,
                mean_raw_violation_minus_threshold_delta_runs: None,
                mean_raw_violation_minus_ewma_delta_runs: None,
                mean_persistent_violation_minus_cusum_delta_runs: None,
                mean_persistent_violation_minus_run_energy_delta_runs: None,
                mean_persistent_violation_minus_pca_fdc_delta_runs: None,
                mean_persistent_violation_minus_threshold_delta_runs: None,
                mean_persistent_violation_minus_ewma_delta_runs: None,
            },
            density_summary: DensitySummary {
                density_window: 3,
                mean_raw_boundary_density_failure: 0.0,
                mean_raw_boundary_density_pass: 0.0,
                mean_persistent_boundary_density_failure: 0.0,
                mean_persistent_boundary_density_pass: 0.0,
                mean_raw_violation_density_failure: 0.0,
                mean_raw_violation_density_pass: 0.0,
                mean_persistent_violation_density_failure: 0.0,
                mean_persistent_violation_density_pass: 0.0,
                mean_threshold_density_failure: 0.0,
                mean_threshold_density_pass: 0.0,
                mean_ewma_density_failure: 0.0,
                mean_ewma_density_pass: 0.0,
                mean_cusum_density_failure: 0.0,
                mean_cusum_density_pass: 0.0,
            },
            boundary_episode_summary: BoundaryEpisodeSummary {
                raw_episode_count: 0,
                persistent_episode_count: 0,
                mean_raw_episode_length: None,
                mean_persistent_episode_length: None,
                max_raw_episode_length: 0,
                max_persistent_episode_length: 0,
                raw_non_escalating_episode_fraction: None,
                persistent_non_escalating_episode_fraction: None,
            },
            dsa_summary: None,
            motif_metrics: vec![
                MotifMetric {
                    motif_name: PRE_FAILURE_SLOW_DRIFT.into(),
                    point_hits: 5,
                    run_hits: 4,
                    pre_failure_window_run_hits: 3,
                    pre_failure_window_precision_proxy: Some(0.75),
                },
                MotifMetric {
                    motif_name: TRANSIENT_EXCURSION.into(),
                    point_hits: 2,
                    run_hits: 2,
                    pre_failure_window_run_hits: 1,
                    pre_failure_window_precision_proxy: Some(0.5),
                },
                MotifMetric {
                    motif_name: RECURRENT_BOUNDARY_APPROACH.into(),
                    point_hits: 7,
                    run_hits: 5,
                    pre_failure_window_run_hits: 3,
                    pre_failure_window_precision_proxy: Some(0.6),
                },
            ],
            per_failure_run_signals: Vec::new(),
            density_metrics: Vec::new(),
            feature_metrics: Vec::new(),
            top_feature_indices: Vec::new(),
        }
    }

    #[test]
    fn heuristic_policy_mapping_is_deterministic() {
        let bank = build_heuristics_bank(&sample_metrics(), "SECOM");
        let transient = bank
            .iter()
            .find(|entry| entry.motif_name == TRANSIENT_EXCURSION)
            .unwrap();
        let recurrent = bank
            .iter()
            .find(|entry| entry.motif_name == RECURRENT_BOUNDARY_APPROACH)
            .unwrap();
        let drift = bank
            .iter()
            .find(|entry| entry.motif_name == PRE_FAILURE_SLOW_DRIFT)
            .unwrap();

        assert_eq!(transient.alert_class_default, HeuristicAlertClass::Silent);
        assert_eq!(recurrent.alert_class_default, HeuristicAlertClass::Watch);
        assert_eq!(drift.alert_class_default, HeuristicAlertClass::Review);
        assert!(transient.requires_corroboration);
        assert!(recurrent.requires_corroboration);
        assert!(!drift.requires_corroboration);
    }

    #[test]
    fn maximum_fragmentation_defaults_follow_minimum_hits() {
        let transient = heuristic_policy_definition(TRANSIENT_EXCURSION).unwrap();
        let recurrent = heuristic_policy_definition(RECURRENT_BOUNDARY_APPROACH).unwrap();

        assert!((transient.maximum_allowed_fragmentation() - 0.5).abs() < 1.0e-9);
        assert!((recurrent.maximum_allowed_fragmentation() - (1.0 / 3.0)).abs() < 1.0e-9);
    }
}
