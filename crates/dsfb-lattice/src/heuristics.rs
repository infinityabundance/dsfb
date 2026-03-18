use serde::{Deserialize, Serialize};

use crate::canonical::CanonicalCaseMetrics;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicWeights {
    pub delta_norm_2: f64,
    pub max_abs_eigenvalue_shift: f64,
    pub mean_abs_eigenvalue_shift: f64,
    pub max_normalized_residual_norm: f64,
    pub residual_energy_ratio: f64,
    pub max_drift_norm: f64,
    pub covariance_trace: f64,
    pub covariance_offdiag_energy: f64,
    pub covariance_rank_estimate: f64,
    pub detected_flag: f64,
    pub normalized_first_crossing_time: f64,
}

impl Default for HeuristicWeights {
    fn default() -> Self {
        Self {
            delta_norm_2: 0.9,
            max_abs_eigenvalue_shift: 0.9,
            mean_abs_eigenvalue_shift: 1.2,
            max_normalized_residual_norm: 1.0,
            residual_energy_ratio: 8.0,
            max_drift_norm: 1.0,
            covariance_trace: 0.5,
            covariance_offdiag_energy: 0.8,
            covariance_rank_estimate: 0.5,
            detected_flag: 0.4,
            normalized_first_crossing_time: 0.3,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicSettings {
    pub enabled: bool,
    pub ambiguity_tolerance: f64,
    pub near_tie_band: f64,
    pub near_tie_relative_gap_threshold: f64,
    pub near_tie_distance_ratio_threshold: f64,
    pub low_noise_threshold: f64,
    pub similarity_metric: String,
    pub weights: HeuristicWeights,
}

impl Default for HeuristicSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            ambiguity_tolerance: 0.18,
            near_tie_band: 0.04,
            near_tie_relative_gap_threshold: 0.15,
            near_tie_distance_ratio_threshold: 0.88,
            low_noise_threshold: 0.01,
            similarity_metric: "weighted_l1".to_string(),
            weights: HeuristicWeights::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AmbiguityTier {
    Unambiguous,
    NearTie,
    Ambiguous,
    Unavailable,
}

impl AmbiguityTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unambiguous => "unambiguous",
            Self::NearTie => "near_tie",
            Self::Ambiguous => "ambiguous",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicDescriptor {
    pub delta_norm_2: f64,
    pub max_abs_eigenvalue_shift: f64,
    pub mean_abs_eigenvalue_shift: f64,
    pub max_normalized_residual_norm: f64,
    pub residual_energy_ratio: f64,
    pub max_drift_norm: f64,
    pub covariance_trace: f64,
    pub covariance_offdiag_energy: f64,
    pub covariance_rank_estimate: f64,
    pub detected_flag: f64,
    pub normalized_first_crossing_time: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicEntry {
    pub id: String,
    pub perturbation_class: String,
    pub descriptor: HeuristicDescriptor,
    pub admissibility_tags: Vec<String>,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicCandidate {
    pub id: String,
    pub perturbation_class: String,
    pub distance: Option<f64>,
    pub admissible: bool,
    pub excluded_tags: Vec<String>,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicRanking {
    pub observed_subject: String,
    pub observed_case: String,
    pub observed_perturbation_class: String,
    pub case_tags: Vec<String>,
    pub descriptor: HeuristicDescriptor,
    pub top_match: Option<String>,
    pub top_distance: Option<f64>,
    pub runner_up_match: Option<String>,
    pub runner_up_distance: Option<f64>,
    pub ambiguity_tier: AmbiguityTier,
    pub ambiguity_flag: bool,
    pub ambiguity_gap: Option<f64>,
    pub relative_gap: Option<f64>,
    pub distance_ratio: Option<f64>,
    pub ambiguity_tolerance: f64,
    pub ambiguity_note: Option<String>,
    pub ranked_candidates: Vec<HeuristicCandidate>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicBankSummary {
    pub description: String,
    pub similarity_metric: String,
    pub ambiguity_tolerance: f64,
    pub near_tie_band: f64,
    pub near_tie_relative_gap_threshold: f64,
    pub near_tie_distance_ratio_threshold: f64,
    pub descriptor_fields: Vec<String>,
    pub weights: HeuristicWeights,
    pub admissibility_note: String,
    pub retrieval_note: String,
    pub entries: Vec<HeuristicEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicRankingRow {
    pub observed_subject: String,
    pub observed_case: String,
    pub observed_perturbation_class: String,
    pub top_match: Option<String>,
    pub top_distance: Option<f64>,
    pub runner_up_match: Option<String>,
    pub runner_up_distance: Option<f64>,
    pub ambiguity_tier: AmbiguityTier,
    pub ambiguity_flag: bool,
    pub ambiguity_gap: Option<f64>,
    pub relative_gap: Option<f64>,
    pub distance_ratio: Option<f64>,
    pub candidate_rank: usize,
    pub candidate_id: String,
    pub candidate_class: String,
    pub candidate_distance: Option<f64>,
    pub admissible: bool,
    pub excluded_tags: String,
}

pub fn build_heuristic_bank(
    references: &[CanonicalCaseMetrics],
    settings: &HeuristicSettings,
) -> HeuristicBankSummary {
    let mut entries = references
        .iter()
        .filter_map(|metrics| {
            let tags = default_tags_for_class(&metrics.perturbation_class);
            if tags.is_empty() && metrics.perturbation_class == "softening" {
                return None;
            }
            Some(HeuristicEntry {
                id: format!("{}_{}", metrics.subject, metrics.perturbation_class),
                perturbation_class: metrics.perturbation_class.clone(),
                descriptor: descriptor_from_canonical(metrics),
                admissibility_tags: tags,
                notes: default_notes_for_class(&metrics.perturbation_class),
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.id.cmp(&right.id));

    HeuristicBankSummary {
        description: "The heuristic bank is a transparent retrieval layer over compact canonical descriptors. It ranks candidate perturbation classes rather than forcing a unique classification.".to_string(),
        similarity_metric: settings.similarity_metric.clone(),
        ambiguity_tolerance: settings.ambiguity_tolerance,
        near_tie_band: settings.near_tie_band,
        near_tie_relative_gap_threshold: settings.near_tie_relative_gap_threshold,
        near_tie_distance_ratio_threshold: settings.near_tie_distance_ratio_threshold,
        descriptor_fields: descriptor_fields(),
        weights: settings.weights.clone(),
        admissibility_note: "Candidates can be filtered by simple admissibility tags such as harmonic_only, low_noise_only, and grouped_mode_case. Filtered entries remain visible in the ranking artifact with exclusion notes.".to_string(),
        retrieval_note: "Similarity is computed with an explicit weighted L1 distance over the exported descriptor fields. The weights and ambiguity thresholds are serialized with the bank so the ranking and its caution bands remain inspectable.".to_string(),
        entries,
    }
}

pub fn rank_case_against_bank(
    observed: &CanonicalCaseMetrics,
    bank: &HeuristicBankSummary,
    settings: &HeuristicSettings,
    case_tags: &[String],
) -> HeuristicRanking {
    let descriptor = descriptor_from_canonical(observed);
    let mut admissible_candidates = Vec::new();
    let mut ranked_candidates = Vec::new();

    for entry in &bank.entries {
        let excluded_tags = entry
            .admissibility_tags
            .iter()
            .filter(|tag| !case_tags.iter().any(|case_tag| case_tag == *tag))
            .cloned()
            .collect::<Vec<_>>();
        let admissible = excluded_tags.is_empty();
        let distance = if admissible {
            Some(weighted_l1_distance(
                &descriptor,
                &entry.descriptor,
                &settings.weights,
            ))
        } else {
            None
        };
        if let Some(distance) = distance {
            admissible_candidates.push((distance, entry));
        }
        ranked_candidates.push(HeuristicCandidate {
            id: entry.id.clone(),
            perturbation_class: entry.perturbation_class.clone(),
            distance,
            admissible,
            excluded_tags,
            notes: entry.notes.clone(),
        });
    }

    ranked_candidates.sort_by(|left, right| match (left.distance, right.distance) {
        (Some(left_distance), Some(right_distance)) => left_distance
            .partial_cmp(&right_distance)
            .unwrap(),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.id.cmp(&right.id),
    });

    let top_distance = ranked_candidates.iter().find_map(|candidate| candidate.distance);
    let top_match = ranked_candidates
        .iter()
        .find(|candidate| candidate.admissible)
        .map(|candidate| candidate.perturbation_class.clone());
    let runner_up = ranked_candidates
        .iter()
        .filter(|candidate| candidate.admissible)
        .nth(1);
    let runner_up_match = runner_up.map(|candidate| candidate.perturbation_class.clone());
    let runner_up_distance = runner_up.and_then(|candidate| candidate.distance);
    let second_distance = runner_up_distance;
    let ambiguity_gap = top_distance.zip(second_distance).map(|(top, second)| second - top);
    let relative_gap = top_distance.zip(second_distance).map(|(top, second)| {
        (second - top) / second.abs().max(1.0e-9)
    });
    let distance_ratio = top_distance.zip(second_distance).map(|(top, second)| {
        top / second.abs().max(1.0e-9)
    });
    let ambiguity_tier = classify_ambiguity(
        top_distance,
        runner_up_distance,
        ambiguity_gap,
        relative_gap,
        distance_ratio,
        settings,
    );
    let ambiguity_flag = ambiguity_tier == AmbiguityTier::Ambiguous;
    let ambiguity_note = ambiguity_note(ambiguity_tier);

    if admissible_candidates.is_empty() {
        return HeuristicRanking {
            observed_subject: observed.subject.clone(),
            observed_case: observed.case.clone(),
            observed_perturbation_class: observed.perturbation_class.clone(),
            case_tags: case_tags.to_vec(),
            descriptor,
            top_match: None,
            top_distance: None,
            runner_up_match: None,
            runner_up_distance: None,
            ambiguity_tier: AmbiguityTier::Unavailable,
            ambiguity_flag: false,
            ambiguity_gap: None,
            relative_gap: None,
            distance_ratio: None,
            ambiguity_tolerance: settings.ambiguity_tolerance,
            ambiguity_note: Some(
                "No heuristic candidates remained admissible for the current case tags.".to_string(),
            ),
            ranked_candidates,
        };
    }

    HeuristicRanking {
        observed_subject: observed.subject.clone(),
        observed_case: observed.case.clone(),
        observed_perturbation_class: observed.perturbation_class.clone(),
        case_tags: case_tags.to_vec(),
        descriptor,
        top_match,
        top_distance,
        runner_up_match,
        runner_up_distance,
        ambiguity_tier,
        ambiguity_flag,
        ambiguity_gap,
        relative_gap,
        distance_ratio,
        ambiguity_tolerance: settings.ambiguity_tolerance,
        ambiguity_note,
        ranked_candidates,
    }
}

pub fn flatten_rankings(rankings: &[HeuristicRanking]) -> Vec<HeuristicRankingRow> {
    let mut rows = Vec::new();
    for ranking in rankings {
        for (index, candidate) in ranking.ranked_candidates.iter().enumerate() {
            rows.push(HeuristicRankingRow {
                observed_subject: ranking.observed_subject.clone(),
                observed_case: ranking.observed_case.clone(),
                observed_perturbation_class: ranking.observed_perturbation_class.clone(),
                top_match: ranking.top_match.clone(),
                top_distance: ranking.top_distance,
                runner_up_match: ranking.runner_up_match.clone(),
                runner_up_distance: ranking.runner_up_distance,
                ambiguity_tier: ranking.ambiguity_tier,
                ambiguity_flag: ranking.ambiguity_flag,
                ambiguity_gap: ranking.ambiguity_gap,
                relative_gap: ranking.relative_gap,
                distance_ratio: ranking.distance_ratio,
                candidate_rank: index + 1,
                candidate_id: candidate.id.clone(),
                candidate_class: candidate.perturbation_class.clone(),
                candidate_distance: candidate.distance,
                admissible: candidate.admissible,
                excluded_tags: candidate.excluded_tags.join("|"),
            });
        }
    }
    rows
}

pub fn descriptor_from_canonical(metrics: &CanonicalCaseMetrics) -> HeuristicDescriptor {
    let crossing_time = metrics.detectability.first_crossing_time.unwrap_or(-1.0);
    let normalized_first_crossing_time = if crossing_time >= 0.0 {
        crossing_time / (metrics.residual.time_to_peak_residual.max(1.0e-6))
    } else {
        -1.0
    };

    HeuristicDescriptor {
        delta_norm_2: metrics.spectral.delta_norm_2,
        max_abs_eigenvalue_shift: metrics.spectral.max_abs_eigenvalue_shift,
        mean_abs_eigenvalue_shift: metrics.spectral.mean_abs_eigenvalue_shift,
        max_normalized_residual_norm: metrics.residual.max_normalized_residual_norm,
        residual_energy_ratio: metrics.residual.residual_energy_ratio,
        max_drift_norm: metrics.temporal.max_drift_norm,
        covariance_trace: metrics.correlation.covariance_trace,
        covariance_offdiag_energy: metrics.correlation.covariance_offdiag_energy,
        covariance_rank_estimate: metrics.correlation.covariance_rank_estimate as f64,
        detected_flag: if metrics.detectability.detected { 1.0 } else { 0.0 },
        normalized_first_crossing_time,
    }
}

pub fn case_tags_for_case(
    noise_std: f64,
    perturbation_class: &str,
    settings: &HeuristicSettings,
) -> Vec<String> {
    let mut tags = vec!["harmonic_only".to_string()];
    if noise_std <= settings.low_noise_threshold {
        tags.push("low_noise_only".to_string());
    }
    if perturbation_class.contains("group_mode") {
        tags.push("grouped_mode_case".to_string());
    }
    tags
}

fn default_tags_for_class(perturbation_class: &str) -> Vec<String> {
    match perturbation_class {
        "point_defect" => vec!["harmonic_only".to_string()],
        "distributed_strain" => vec!["harmonic_only".to_string(), "low_noise_only".to_string()],
        "group_mode_cluster" => vec!["harmonic_only".to_string(), "grouped_mode_case".to_string()],
        _ => vec!["harmonic_only".to_string()],
    }
}

fn default_notes_for_class(perturbation_class: &str) -> String {
    match perturbation_class {
        "point_defect" => "Localized mass-and-spring perturbation reference descriptor.".to_string(),
        "distributed_strain" => "Smooth spring-gradient perturbation reference descriptor.".to_string(),
        "group_mode_cluster" => "Clustered correlated perturbation reference descriptor.".to_string(),
        _ => "Reference descriptor for a bounded synthetic perturbation class.".to_string(),
    }
}

fn weighted_l1_distance(
    left: &HeuristicDescriptor,
    right: &HeuristicDescriptor,
    weights: &HeuristicWeights,
) -> f64 {
    weights.delta_norm_2 * (left.delta_norm_2 - right.delta_norm_2).abs()
        + weights.max_abs_eigenvalue_shift
            * (left.max_abs_eigenvalue_shift - right.max_abs_eigenvalue_shift).abs()
        + weights.mean_abs_eigenvalue_shift
            * (left.mean_abs_eigenvalue_shift - right.mean_abs_eigenvalue_shift).abs()
        + weights.max_normalized_residual_norm
            * (left.max_normalized_residual_norm - right.max_normalized_residual_norm).abs()
        + weights.residual_energy_ratio
            * (left.residual_energy_ratio - right.residual_energy_ratio).abs()
        + weights.max_drift_norm * (left.max_drift_norm - right.max_drift_norm).abs()
        + weights.covariance_trace * (left.covariance_trace - right.covariance_trace).abs()
        + weights.covariance_offdiag_energy
            * (left.covariance_offdiag_energy - right.covariance_offdiag_energy).abs()
        + weights.covariance_rank_estimate
            * (left.covariance_rank_estimate - right.covariance_rank_estimate).abs()
        + weights.detected_flag * (left.detected_flag - right.detected_flag).abs()
        + weights.normalized_first_crossing_time
            * (left.normalized_first_crossing_time - right.normalized_first_crossing_time).abs()
}

fn descriptor_fields() -> Vec<String> {
    vec![
        "delta_norm_2".to_string(),
        "max_abs_eigenvalue_shift".to_string(),
        "mean_abs_eigenvalue_shift".to_string(),
        "max_normalized_residual_norm".to_string(),
        "residual_energy_ratio".to_string(),
        "max_drift_norm".to_string(),
        "covariance_trace".to_string(),
        "covariance_offdiag_energy".to_string(),
        "covariance_rank_estimate".to_string(),
        "detected_flag".to_string(),
        "normalized_first_crossing_time".to_string(),
    ]
}

fn classify_ambiguity(
    top_distance: Option<f64>,
    runner_up_distance: Option<f64>,
    ambiguity_gap: Option<f64>,
    relative_gap: Option<f64>,
    distance_ratio: Option<f64>,
    settings: &HeuristicSettings,
) -> AmbiguityTier {
    if top_distance.is_none() {
        return AmbiguityTier::Unavailable;
    }
    if runner_up_distance.is_none() {
        return AmbiguityTier::Unambiguous;
    }

    let gap = ambiguity_gap.unwrap_or(f64::INFINITY);
    let relative = relative_gap.unwrap_or(f64::INFINITY);
    let ratio = distance_ratio.unwrap_or(0.0);

    if gap <= settings.ambiguity_tolerance {
        AmbiguityTier::Ambiguous
    } else if gap <= settings.ambiguity_tolerance + settings.near_tie_band
        || relative <= settings.near_tie_relative_gap_threshold
        || ratio >= settings.near_tie_distance_ratio_threshold
    {
        AmbiguityTier::NearTie
    } else {
        AmbiguityTier::Unambiguous
    }
}

fn ambiguity_note(tier: AmbiguityTier) -> Option<String> {
    match tier {
        AmbiguityTier::Ambiguous => Some(
            "Ambiguous ranking: the top candidate and runner-up are within the configured descriptor-space ambiguity tolerance."
                .to_string(),
        ),
        AmbiguityTier::NearTie => Some(
            "Near-tie ranking: the top candidate remains first, but the runner-up is close enough in descriptor space that the interpretation should be treated cautiously."
                .to_string(),
        ),
        AmbiguityTier::Unavailable => Some(
            "No heuristic candidates remained admissible for the current case tags.".to_string(),
        ),
        AmbiguityTier::Unambiguous => None,
    }
}
