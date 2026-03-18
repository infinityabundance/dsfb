use serde::{Deserialize, Serialize};

use crate::canonical::CanonicalCaseMetrics;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeuristicSettings {
    pub enabled: bool,
    pub ambiguity_tolerance: f64,
    pub low_noise_threshold: f64,
    pub similarity_metric: String,
}

impl Default for HeuristicSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            ambiguity_tolerance: 0.18,
            low_noise_threshold: 0.01,
            similarity_metric: "weighted_l1".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicDescriptor {
    pub delta_norm_2: f64,
    pub mean_abs_eigenvalue_shift: f64,
    pub max_normalized_residual_norm: f64,
    pub residual_energy_ratio: f64,
    pub max_drift_norm: f64,
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
    pub ambiguity_flag: bool,
    pub ambiguity_gap: Option<f64>,
    pub ambiguity_tolerance: f64,
    pub ambiguity_note: Option<String>,
    pub ranked_candidates: Vec<HeuristicCandidate>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicBankSummary {
    pub description: String,
    pub similarity_metric: String,
    pub ambiguity_tolerance: f64,
    pub admissibility_note: String,
    pub entries: Vec<HeuristicEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeuristicRankingRow {
    pub observed_subject: String,
    pub observed_case: String,
    pub observed_perturbation_class: String,
    pub top_match: Option<String>,
    pub top_distance: Option<f64>,
    pub ambiguity_flag: bool,
    pub ambiguity_gap: Option<f64>,
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
        admissibility_note: "Candidates can be filtered by simple admissibility tags such as harmonic_only, low_noise_only, and grouped_mode_case. Filtered entries remain visible in the ranking artifact with exclusion notes.".to_string(),
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
            Some(weighted_l1_distance(&descriptor, &entry.descriptor))
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
    let second_distance = ranked_candidates
        .iter()
        .filter_map(|candidate| candidate.distance)
        .nth(1);
    let ambiguity_gap = top_distance.zip(second_distance).map(|(top, second)| second - top);
    let ambiguity_flag = ambiguity_gap
        .map(|gap| gap <= settings.ambiguity_tolerance)
        .unwrap_or(false);
    let ambiguity_note = if ambiguity_flag {
        Some(
            "Ambiguous ranking: the top candidate and runner-up are near-tied within the configured descriptor-space tolerance."
                .to_string(),
        )
    } else {
        None
    };

    if admissible_candidates.is_empty() {
        return HeuristicRanking {
            observed_subject: observed.subject.clone(),
            observed_case: observed.case.clone(),
            observed_perturbation_class: observed.perturbation_class.clone(),
            case_tags: case_tags.to_vec(),
            descriptor,
            top_match: None,
            top_distance: None,
            ambiguity_flag: false,
            ambiguity_gap: None,
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
        ambiguity_flag,
        ambiguity_gap,
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
                ambiguity_flag: ranking.ambiguity_flag,
                ambiguity_gap: ranking.ambiguity_gap,
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
        mean_abs_eigenvalue_shift: metrics.spectral.mean_abs_eigenvalue_shift,
        max_normalized_residual_norm: metrics.residual.max_normalized_residual_norm,
        residual_energy_ratio: metrics.residual.residual_energy_ratio,
        max_drift_norm: metrics.temporal.max_drift_norm,
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

fn weighted_l1_distance(left: &HeuristicDescriptor, right: &HeuristicDescriptor) -> f64 {
    0.9 * (left.delta_norm_2 - right.delta_norm_2).abs()
        + 1.2 * (left.mean_abs_eigenvalue_shift - right.mean_abs_eigenvalue_shift).abs()
        + 1.0 * (left.max_normalized_residual_norm - right.max_normalized_residual_norm).abs()
        + 8.0 * (left.residual_energy_ratio - right.residual_energy_ratio).abs()
        + 1.0 * (left.max_drift_norm - right.max_drift_norm).abs()
        + 0.8 * (left.covariance_offdiag_energy - right.covariance_offdiag_energy).abs()
        + 0.5 * (left.covariance_rank_estimate - right.covariance_rank_estimate).abs()
        + 0.4 * (left.detected_flag - right.detected_flag).abs()
        + 0.3 * (left.normalized_first_crossing_time - right.normalized_first_crossing_time).abs()
}
