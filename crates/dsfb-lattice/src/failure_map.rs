use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::canonical::{build_canonical_case_metrics, CanonicalCaseMetrics};
use crate::detectability::{
    build_envelope, crossing_regime_label, evaluate_signal,
    DetectabilityInterpretationSettings, SemanticStatus,
};
use crate::heuristics::{
    case_tags_for_case, rank_case_against_bank, AmbiguityTier, HeuristicBankSummary,
    HeuristicRanking, HeuristicSettings,
};
use crate::lattice::Lattice;
use crate::perturbation::{distributed_strain, global_softening, point_defect, PointDefectSpec};
use crate::residuals::{
    add_observation_noise, build_time_series, covariance_matrix, simulate_response,
    SimulationConfig,
};
use crate::semantic::assess_semantic_status;
use crate::spectra::{analyze_symmetric, compare_spectra, SpectrumAnalysis};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FailureMapScenarioSpec {
    pub scenario_name: String,
    pub description: String,
    pub perturbation_class: String,
    pub point_mass_scale: f64,
    pub point_spring_scale: f64,
    pub strain_strength: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FailureMapSettings {
    pub enabled: bool,
    pub rng_seed: u64,
    pub noise_levels: Vec<f64>,
    pub predictor_spring_scales: Vec<f64>,
    pub scenarios: Vec<FailureMapScenarioSpec>,
}

impl Default for FailureMapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            rng_seed: 20_260_319,
            noise_levels: vec![0.0, 0.01, 0.02, 0.04, 0.07],
            predictor_spring_scales: vec![1.0, 0.99, 0.97, 0.94, 0.90],
            scenarios: vec![
                FailureMapScenarioSpec {
                    scenario_name: "weak_point_defect".to_string(),
                    description: "Weak localized defect used to show where detectability degrades and eventually fails under controlled stress.".to_string(),
                    perturbation_class: "point_defect".to_string(),
                    point_mass_scale: 1.08,
                    point_spring_scale: 0.95,
                    strain_strength: 0.0,
                },
                FailureMapScenarioSpec {
                    scenario_name: "ambiguous_mixed_signature".to_string(),
                    description: "Mixed weak point-defect and smooth-strain case used to expose descriptor-space ambiguity under controlled stress.".to_string(),
                    perturbation_class: "ambiguous_point_defect_vs_strain".to_string(),
                    point_mass_scale: 1.08,
                    point_spring_scale: 0.96,
                    strain_strength: 0.14,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FailureMapPoint {
    pub scenario_name: String,
    pub scenario_description: String,
    pub perturbation_class: String,
    pub noise_std: f64,
    pub predictor_spring_scale: f64,
    pub rng_seed: u64,
    pub canonical_metrics: CanonicalCaseMetrics,
    pub heuristic_ranking: HeuristicRanking,
    pub semantic_status: SemanticStatus,
    pub semantic_reason: String,
    pub status_label: SemanticStatus,
}

#[derive(Clone, Debug, Serialize)]
pub struct FailureMapResult {
    pub description: String,
    pub settings: FailureMapSettings,
    pub points: Vec<FailureMapPoint>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FailureMapRow {
    pub scenario_name: String,
    pub scenario_description: String,
    pub perturbation_class: String,
    pub noise_std: f64,
    pub predictor_spring_scale: f64,
    pub rng_seed: u64,
    pub detected: bool,
    pub crossing_regime_label: crate::detectability::CrossingRegimeLabel,
    pub detectability_interpretation_class: crate::detectability::DetectabilityInterpretationClass,
    pub detection_strength_band: crate::detectability::DetectionStrengthBand,
    pub boundary_proximate_crossing: bool,
    pub first_crossing_time: Option<f64>,
    pub first_crossing_step: Option<usize>,
    pub crossing_margin: Option<f64>,
    pub normalized_crossing_margin: Option<f64>,
    pub post_crossing_persistence_duration: Option<f64>,
    pub post_crossing_fraction: Option<f64>,
    pub peak_margin_after_crossing: Option<f64>,
    pub max_raw_residual_norm: f64,
    pub max_normalized_residual_norm: f64,
    pub max_drift_norm: f64,
    pub delta_norm_2: f64,
    pub max_abs_eigenvalue_shift: f64,
    pub mean_abs_eigenvalue_shift: f64,
    pub covariance_trace: f64,
    pub covariance_offdiag_energy: f64,
    pub covariance_rank_estimate: usize,
    pub envelope_mode: String,
    pub envelope_basis: String,
    pub envelope_sigma_multiplier: f64,
    pub envelope_additive_floor: f64,
    pub envelope_baseline_runs: usize,
    pub envelope_universal: bool,
    pub top_match: Option<String>,
    pub top_distance: Option<f64>,
    pub runner_up_match: Option<String>,
    pub runner_up_distance: Option<f64>,
    pub ambiguity_tier: AmbiguityTier,
    pub ambiguity_flag: bool,
    pub ambiguity_gap: Option<f64>,
    pub heuristic_relative_gap: Option<f64>,
    pub heuristic_distance_ratio: Option<f64>,
    pub semantic_status: SemanticStatus,
    pub semantic_reason: String,
    pub status_label: SemanticStatus,
    pub degradation_label: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FailureMapSummaryPoint {
    pub scenario_name: String,
    pub perturbation_class: String,
    pub noise_std: f64,
    pub predictor_spring_scale: f64,
    pub detected: bool,
    pub crossing_regime_label: crate::detectability::CrossingRegimeLabel,
    pub detectability_interpretation_class: crate::detectability::DetectabilityInterpretationClass,
    pub detection_strength_band: crate::detectability::DetectionStrengthBand,
    pub boundary_proximate_crossing: bool,
    pub crossing_margin: Option<f64>,
    pub normalized_crossing_margin: Option<f64>,
    pub post_crossing_fraction: Option<f64>,
    pub top_match: Option<String>,
    pub runner_up_match: Option<String>,
    pub ambiguity_tier: AmbiguityTier,
    pub ambiguity_flag: bool,
    pub ambiguity_gap: Option<f64>,
    pub heuristic_relative_gap: Option<f64>,
    pub heuristic_distance_ratio: Option<f64>,
    pub semantic_status: SemanticStatus,
    pub semantic_reason: String,
    pub status_label: SemanticStatus,
    pub degradation_label: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FailureMapScenarioAggregate {
    pub scenario_name: String,
    pub perturbation_class: String,
    pub clear_structural_detection_count: usize,
    pub marginal_structural_detection_count: usize,
    pub degraded_count: usize,
    pub ambiguous_count: usize,
    pub degraded_ambiguous_count: usize,
    pub not_detected_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct FailureMapSummary {
    pub description: String,
    pub settings: FailureMapSettings,
    pub scenarios: Vec<FailureMapScenarioAggregate>,
    pub points: Vec<FailureMapSummaryPoint>,
}

pub fn run_failure_map(
    nominal_lattice: &Lattice,
    nominal_spectrum: &SpectrumAnalysis,
    simulation: &SimulationConfig,
    baseline_runs: usize,
    envelope_sigma: f64,
    envelope_floor: f64,
    consecutive_crossings: usize,
    normalization_epsilon: f64,
    detectability_interpretation: &DetectabilityInterpretationSettings,
    heuristic_bank: &HeuristicBankSummary,
    heuristic_settings: &HeuristicSettings,
    settings: &FailureMapSettings,
) -> Result<FailureMapResult> {
    let nominal_dynamical = nominal_lattice.dynamical_matrix()?;
    let mut points = Vec::new();

    for (scenario_index, scenario) in settings.scenarios.iter().enumerate() {
        let perturbed_lattice = scenario_lattice(nominal_lattice, scenario);
        let perturbed_dynamical = perturbed_lattice.dynamical_matrix()?;
        let delta = &perturbed_dynamical - &nominal_dynamical;
        let perturbed_spectrum = analyze_symmetric(&perturbed_dynamical)?;
        let comparison = compare_spectra(
            &scenario.scenario_name,
            nominal_spectrum,
            &perturbed_spectrum,
            &delta,
        )?;
        let true_signal =
            simulate_response(&perturbed_dynamical, &nominal_spectrum.eigenvectors, simulation, 0);

        for (noise_index, noise_std) in settings.noise_levels.iter().copied().enumerate() {
            for (scale_index, predictor_spring_scale) in
                settings.predictor_spring_scales.iter().copied().enumerate()
            {
                let cell_seed = settings
                    .rng_seed
                    .wrapping_add((scenario_index as u64) * 100_000)
                    .wrapping_add((noise_index as u64) * 1_000)
                    .wrapping_add((scale_index as u64) * 10);

                let predictor_observations = simulate_predictor_observations(
                    nominal_lattice,
                    nominal_spectrum,
                    simulation,
                    predictor_spring_scale,
                )?;
                let baseline_ensemble = (1..=baseline_runs)
                    .map(|variant| {
                        let baseline = simulate_response(
                            &nominal_dynamical,
                            &nominal_spectrum.eigenvectors,
                            simulation,
                            variant,
                        );
                        let measured = add_observation_noise(
                            &baseline.observations,
                            noise_std,
                            cell_seed.wrapping_add(variant as u64),
                        );
                        build_time_series(
                            &predictor_observations,
                            &measured,
                            normalization_epsilon,
                        )
                    })
                    .collect::<Vec<_>>();
                let baseline_reference = baseline_ensemble[0].clone();
                let baseline_norms = baseline_ensemble
                    .iter()
                    .map(|bundle| bundle.residual_norms.clone())
                    .collect::<Vec<_>>();
                let regime_label = format!(
                    "{}_noise_{noise_std:.3}_predictor_{predictor_spring_scale:.3}",
                    scenario.scenario_name
                );
                let envelope = build_envelope(
                    &baseline_norms,
                    envelope_sigma,
                    envelope_floor,
                    &regime_label,
                    baseline_reference
                        .residual_norms
                        .iter()
                        .copied()
                        .fold(0.0_f64, f64::max),
                    baseline_reference
                        .predicted_norms
                        .iter()
                        .copied()
                        .fold(0.0_f64, f64::max),
                    baseline_reference
                        .predicted_norms
                        .iter()
                        .map(|value| value.powi(2))
                        .sum(),
                );
                let measured = add_observation_noise(
                    &true_signal.observations,
                    noise_std,
                    cell_seed.wrapping_add(50_000),
                );
                let signal_bundle = build_time_series(
                    &predictor_observations,
                    &measured,
                    normalization_epsilon,
                );
                let detectability = evaluate_signal(
                    &signal_bundle.residual_norms,
                    &envelope,
                    consecutive_crossings,
                    simulation.dt,
                    normalization_epsilon,
                    crossing_regime_label(noise_std, predictor_spring_scale),
                    detectability_interpretation,
                );
                let covariance = covariance_matrix(&signal_bundle.residuals);
                let canonical_metrics = build_canonical_case_metrics(
                    "failure_map",
                    &regime_label,
                    &scenario.perturbation_class,
                    &comparison,
                    &signal_bundle,
                    Some(&detectability),
                    &covariance,
                    &envelope.provenance,
                    simulation.dt,
                );
                let case_tags =
                    case_tags_for_case(noise_std, &scenario.perturbation_class, heuristic_settings);
                let ranking = rank_case_against_bank(
                    &canonical_metrics,
                    heuristic_bank,
                    heuristic_settings,
                    &case_tags,
                );
                let semantic_assessment = assess_semantic_status(&detectability, Some(&ranking));
                points.push(FailureMapPoint {
                    scenario_name: scenario.scenario_name.clone(),
                    scenario_description: scenario.description.clone(),
                    perturbation_class: scenario.perturbation_class.clone(),
                    noise_std,
                    predictor_spring_scale,
                    rng_seed: cell_seed,
                    canonical_metrics,
                    heuristic_ranking: ranking,
                    semantic_status: semantic_assessment.semantic_status,
                    semantic_reason: semantic_assessment.semantic_reason,
                    status_label: semantic_assessment.semantic_status,
                });
            }
        }
    }

    Ok(FailureMapResult {
        description: "Controlled synthetic failure map over noise and predictor mismatch. The map is intended to make explicit where pointwise crossing remains clearly structurally legible, where clean detections are only marginally above the boundary, where stressed early or low-margin crossings become degraded, where descriptor-space ambiguity rises, and where detection fails under this bounded toy setup. In particular, it makes visible that detectability is not monotone in raw residual size alone.".to_string(),
        settings: settings.clone(),
        points,
    })
}

pub fn summarize_failure_map(result: &FailureMapResult) -> FailureMapSummary {
    let scenarios = result
        .settings
        .scenarios
        .iter()
        .map(|scenario| {
            let points = result
                .points
                .iter()
                .filter(|point| point.scenario_name == scenario.scenario_name);
            let mut clear_structural_detection_count = 0usize;
            let mut marginal_structural_detection_count = 0usize;
            let mut degraded_count = 0usize;
            let mut ambiguous_count = 0usize;
            let mut degraded_ambiguous_count = 0usize;
            let mut not_detected_count = 0usize;
            for point in points {
                match point.semantic_status {
                    SemanticStatus::ClearStructuralDetection => {
                        clear_structural_detection_count += 1
                    }
                    SemanticStatus::MarginalStructuralDetection => {
                        marginal_structural_detection_count += 1
                    }
                    SemanticStatus::Degraded => degraded_count += 1,
                    SemanticStatus::Ambiguous => ambiguous_count += 1,
                    SemanticStatus::DegradedAmbiguous => degraded_ambiguous_count += 1,
                    SemanticStatus::NotDetected => not_detected_count += 1,
                }
            }
            FailureMapScenarioAggregate {
                scenario_name: scenario.scenario_name.clone(),
                perturbation_class: scenario.perturbation_class.clone(),
                clear_structural_detection_count,
                marginal_structural_detection_count,
                degraded_count,
                ambiguous_count,
                degraded_ambiguous_count,
                not_detected_count,
            }
        })
        .collect::<Vec<_>>();

    let points = result
        .points
        .iter()
        .map(|point| FailureMapSummaryPoint {
            scenario_name: point.scenario_name.clone(),
            perturbation_class: point.perturbation_class.clone(),
            noise_std: point.noise_std,
            predictor_spring_scale: point.predictor_spring_scale,
            detected: point.canonical_metrics.detectability.detected,
            crossing_regime_label: point.canonical_metrics.detectability.crossing_regime_label,
            detectability_interpretation_class: point
                .canonical_metrics
                .detectability
                .interpretation_class,
            detection_strength_band: point
                .canonical_metrics
                .detectability
                .detection_strength_band,
            boundary_proximate_crossing: point
                .canonical_metrics
                .detectability
                .boundary_proximate_crossing,
            crossing_margin: point.canonical_metrics.detectability.crossing_margin,
            normalized_crossing_margin: point.canonical_metrics.detectability.normalized_crossing_margin,
            post_crossing_fraction: point.canonical_metrics.detectability.post_crossing_fraction,
            top_match: point.heuristic_ranking.top_match.clone(),
            runner_up_match: point.heuristic_ranking.runner_up_match.clone(),
            ambiguity_tier: point.heuristic_ranking.ambiguity_tier,
            ambiguity_flag: point.heuristic_ranking.ambiguity_flag,
            ambiguity_gap: point.heuristic_ranking.ambiguity_gap,
            heuristic_relative_gap: point.heuristic_ranking.relative_gap,
            heuristic_distance_ratio: point.heuristic_ranking.distance_ratio,
            semantic_status: point.semantic_status,
            semantic_reason: point.semantic_reason.clone(),
            status_label: point.status_label,
            degradation_label: point.semantic_status.as_str().to_string(),
        })
        .collect::<Vec<_>>();

    FailureMapSummary {
        description: result.description.clone(),
        settings: result.settings.clone(),
        scenarios,
        points,
    }
}

pub fn flatten_failure_map_points(result: &FailureMapResult) -> Vec<FailureMapRow> {
    result
        .points
        .iter()
        .map(|point| FailureMapRow {
            scenario_name: point.scenario_name.clone(),
            scenario_description: point.scenario_description.clone(),
            perturbation_class: point.perturbation_class.clone(),
            noise_std: point.noise_std,
            predictor_spring_scale: point.predictor_spring_scale,
            rng_seed: point.rng_seed,
            detected: point.canonical_metrics.detectability.detected,
            crossing_regime_label: point.canonical_metrics.detectability.crossing_regime_label,
            detectability_interpretation_class: point
                .canonical_metrics
                .detectability
                .interpretation_class,
            detection_strength_band: point
                .canonical_metrics
                .detectability
                .detection_strength_band,
            boundary_proximate_crossing: point
                .canonical_metrics
                .detectability
                .boundary_proximate_crossing,
            first_crossing_time: point.canonical_metrics.detectability.first_crossing_time,
            first_crossing_step: point.canonical_metrics.detectability.first_crossing_step,
            crossing_margin: point.canonical_metrics.detectability.crossing_margin,
            normalized_crossing_margin: point
                .canonical_metrics
                .detectability
                .normalized_crossing_margin,
            post_crossing_persistence_duration: point
                .canonical_metrics
                .detectability
                .post_crossing_persistence_duration,
            post_crossing_fraction: point.canonical_metrics.detectability.post_crossing_fraction,
            peak_margin_after_crossing: point
                .canonical_metrics
                .detectability
                .peak_margin_after_crossing,
            max_raw_residual_norm: point.canonical_metrics.residual.max_raw_residual_norm,
            max_normalized_residual_norm: point
                .canonical_metrics
                .residual
                .max_normalized_residual_norm,
            max_drift_norm: point.canonical_metrics.temporal.max_drift_norm,
            delta_norm_2: point.canonical_metrics.spectral.delta_norm_2,
            max_abs_eigenvalue_shift: point
                .canonical_metrics
                .spectral
                .max_abs_eigenvalue_shift,
            mean_abs_eigenvalue_shift: point
                .canonical_metrics
                .spectral
                .mean_abs_eigenvalue_shift,
            covariance_trace: point.canonical_metrics.correlation.covariance_trace,
            covariance_offdiag_energy: point
                .canonical_metrics
                .correlation
                .covariance_offdiag_energy,
            covariance_rank_estimate: point
                .canonical_metrics
                .correlation
                .covariance_rank_estimate,
            envelope_mode: point.canonical_metrics.envelope.envelope_mode.clone(),
            envelope_basis: point.canonical_metrics.envelope.envelope_basis.clone(),
            envelope_sigma_multiplier: point
                .canonical_metrics
                .envelope
                .envelope_sigma_multiplier,
            envelope_additive_floor: point
                .canonical_metrics
                .envelope
                .envelope_additive_floor,
            envelope_baseline_runs: point.canonical_metrics.envelope.envelope_baseline_runs,
            envelope_universal: point.canonical_metrics.envelope.envelope_universal,
            top_match: point.heuristic_ranking.top_match.clone(),
            top_distance: point.heuristic_ranking.top_distance,
            runner_up_match: point.heuristic_ranking.runner_up_match.clone(),
            runner_up_distance: point.heuristic_ranking.runner_up_distance,
            ambiguity_tier: point.heuristic_ranking.ambiguity_tier,
            ambiguity_flag: point.heuristic_ranking.ambiguity_flag,
            ambiguity_gap: point.heuristic_ranking.ambiguity_gap,
            heuristic_relative_gap: point.heuristic_ranking.relative_gap,
            heuristic_distance_ratio: point.heuristic_ranking.distance_ratio,
            semantic_status: point.semantic_status,
            semantic_reason: point.semantic_reason.clone(),
            status_label: point.status_label,
            degradation_label: point.semantic_status.as_str().to_string(),
        })
        .collect()
}

fn scenario_lattice(nominal_lattice: &Lattice, scenario: &FailureMapScenarioSpec) -> Lattice {
    let base = point_defect(
        nominal_lattice,
        &PointDefectSpec {
            site: nominal_lattice.sites / 2,
            mass_scale: scenario.point_mass_scale,
            spring_index: nominal_lattice.sites / 2 + 1,
            spring_scale: scenario.point_spring_scale,
        },
    );
    if scenario.strain_strength > 0.0 {
        let mut lattice = distributed_strain(&base, scenario.strain_strength);
        lattice.label = scenario.perturbation_class.clone();
        lattice
    } else {
        let mut lattice = base;
        lattice.label = scenario.perturbation_class.clone();
        lattice
    }
}

fn simulate_predictor_observations(
    nominal_lattice: &Lattice,
    nominal_spectrum: &SpectrumAnalysis,
    simulation: &SimulationConfig,
    predictor_spring_scale: f64,
) -> Result<Vec<nalgebra::DVector<f64>>> {
    let predictor_lattice = if (predictor_spring_scale - 1.0).abs() < 1.0e-12 {
        nominal_lattice.clone()
    } else {
        global_softening(nominal_lattice, predictor_spring_scale)
    };
    let predictor_dynamical = predictor_lattice.dynamical_matrix()?;
    Ok(
        simulate_response(
            &predictor_dynamical,
            &nominal_spectrum.eigenvectors,
            simulation,
            0,
        )
        .observations,
    )
}
