pub mod detectability;
pub mod io;
pub mod lattice;
pub mod perturbation;
pub mod report;
pub mod residuals;
pub mod spectra;
pub mod utils;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::detectability::{
    build_envelope, evaluate_signal, DetectabilitySummary, Envelope, EnvelopeProvenance,
};
use crate::io::{create_timestamped_run_directory, write_csv_rows, write_json_pretty, zip_directory};
use crate::lattice::Lattice;
use crate::perturbation::{distributed_strain, global_softening, grouped_cluster, point_defect, PointDefectSpec};
use crate::report::{write_pdf_report, write_reports};
use crate::residuals::{
    add_observation_noise, build_time_series, covariance_matrix, simulate_response,
    SimulationConfig, TimeSeriesBundle,
};
use crate::spectra::{analyze_symmetric, compare_spectra, SpectralComparison, SpectrumAnalysis};
use crate::utils::{covariance_trace, offdiag_energy, path_string};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExampleSelection {
    All,
    Baseline,
    PointDefect,
    Strain,
    GroupMode,
    Softening,
}

impl ExampleSelection {
    fn includes(self, other: Self) -> bool {
        self == Self::All || self == other
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Baseline => "baseline",
            Self::PointDefect => "point-defect",
            Self::Strain => "strain",
            Self::GroupMode => "group-mode",
            Self::Softening => "softening",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DemoConfig {
    pub output_root: PathBuf,
    pub example: ExampleSelection,
    pub sites: usize,
    pub steps: usize,
    pub dt: f64,
    pub damping: f64,
    pub observed_modes: usize,
    pub baseline_runs: usize,
    pub envelope_sigma: f64,
    pub envelope_floor: f64,
    pub consecutive_crossings: usize,
    pub normalization_epsilon: f64,
    pub pressure_test: PressureTestSettings,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            output_root: default_output_root(),
            example: ExampleSelection::All,
            sites: 12,
            steps: 320,
            dt: 0.04,
            damping: 0.06,
            observed_modes: 4,
            baseline_runs: 4,
            envelope_sigma: 3.0,
            envelope_floor: 0.003,
            consecutive_crossings: 3,
            normalization_epsilon: 1.0e-6,
            pressure_test: PressureTestSettings::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PressureTestSettings {
    pub enabled: bool,
    pub observation_noise_std: f64,
    pub predictor_spring_scale: f64,
    pub rng_seed: u64,
}

impl Default for PressureTestSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            observation_noise_std: 0.018,
            predictor_spring_scale: 0.97,
            rng_seed: 20_260_318,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExperimentResult {
    pub name: String,
    pub description: String,
    pub lattice: Lattice,
    pub spectrum: SpectrumAnalysis,
    pub comparison: SpectralComparison,
    pub simulation: TimeSeriesBundle,
    pub covariance: DMatrix<f64>,
}

#[derive(Clone, Debug)]
pub struct SofteningSweepResult {
    pub scales: Vec<f64>,
    pub smallest_eigenvalues: Vec<f64>,
    pub max_residual_norms: Vec<f64>,
    pub max_normalized_residual_norms: Vec<f64>,
    pub max_drift_norms: Vec<f64>,
    pub max_slew_norms: Vec<f64>,
    pub residual_energy_ratios: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct PressureTestCaseResult {
    pub case_name: String,
    pub description: String,
    pub additive_noise_std: f64,
    pub predictor_spring_scale: f64,
    pub rng_seed: u64,
    pub baseline_reference: TimeSeriesBundle,
    pub envelope: Envelope,
    pub signal_bundle: TimeSeriesBundle,
    pub detectability: DetectabilitySummary,
}

#[derive(Clone, Debug)]
pub struct PressureTestResult {
    pub cases: Vec<PressureTestCaseResult>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NormalizationMetadata {
    pub method: String,
    pub denominator: String,
    pub epsilon: f64,
    pub note: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExperimentSummary {
    pub name: String,
    pub description: String,
    pub delta_norm_2: f64,
    pub max_abs_shift: f64,
    pub max_shift_ratio: f64,
    pub bound_satisfied: bool,
    pub max_residual_norm: f64,
    pub max_normalized_residual_norm: f64,
    pub max_drift_norm: f64,
    pub max_slew_norm: f64,
    pub residual_energy_ratio: f64,
    pub covariance_trace: f64,
    pub covariance_offdiag_energy: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SofteningSummary {
    pub softest_scale: f64,
    pub softest_smallest_eigenvalue: f64,
    pub softest_max_residual_norm: f64,
    pub softest_max_normalized_residual_norm: f64,
    pub softest_max_drift_norm: f64,
    pub softest_max_slew_norm: f64,
    pub softest_residual_energy_ratio: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PressureTestCaseSummary {
    pub case_name: String,
    pub description: String,
    pub additive_noise_std: f64,
    pub predictor_spring_scale: f64,
    pub rng_seed: u64,
    pub detected: bool,
    pub first_crossing_time: Option<f64>,
    pub first_crossing_step: Option<usize>,
    pub crossing_margin: Option<f64>,
    pub normalized_crossing_margin: Option<f64>,
    pub max_raw_residual: f64,
    pub max_normalized_residual: f64,
    pub residual_energy_ratio: f64,
    pub envelope_provenance: EnvelopeProvenance,
}

#[derive(Clone, Debug, Serialize)]
pub struct PressureTestSummary {
    pub description: String,
    pub cases: Vec<PressureTestCaseSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunSummary {
    pub run_dir: String,
    pub selected_example: String,
    pub nominal_sites: usize,
    pub nominal_smallest_eigenvalue: f64,
    pub nominal_largest_eigenvalue: f64,
    pub normalization: NormalizationMetadata,
    pub envelope_provenance: EnvelopeProvenance,
    pub experiments: Vec<ExperimentSummary>,
    pub detectability: Option<DetectabilitySummary>,
    pub pressure_test: Option<PressureTestSummary>,
    pub softening: Option<SofteningSummary>,
    pub limitations: Vec<String>,
    pub figures: Vec<String>,
    pub report_markdown: String,
    pub report_pdf: String,
    pub zip_archive: String,
}

#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub run_dir: PathBuf,
    pub summary_json: PathBuf,
    pub report_pdf: PathBuf,
    pub zip_path: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
struct EigenvalueRow {
    mode: usize,
    eigenvalue: f64,
    frequency: f64,
}

#[derive(Clone, Debug, Serialize)]
struct PerturbedEigenvalueRow {
    experiment: String,
    mode: usize,
    eigenvalue: f64,
    frequency: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ObservationRow {
    step: usize,
    time: f64,
    channel: usize,
    value: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ResidualRow {
    experiment: String,
    step: usize,
    time: f64,
    channel: usize,
    predicted: f64,
    measured: f64,
    residual: f64,
}

#[derive(Clone, Debug, Serialize)]
struct SignalRow {
    experiment: String,
    step: usize,
    time: f64,
    channel: usize,
    value: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ResidualNormRow {
    experiment: String,
    case: String,
    step: usize,
    time: f64,
    predicted_norm: f64,
    measured_norm: f64,
    raw_residual_norm: f64,
    normalized_residual_norm: f64,
}

#[derive(Clone, Debug, Serialize)]
struct CovarianceRow {
    experiment: String,
    row: usize,
    column: usize,
    value: f64,
}

#[derive(Clone, Debug, Serialize)]
struct MetricRow {
    experiment: String,
    metric: String,
    value: f64,
    units: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct EnvelopeRow {
    step: usize,
    time: f64,
    mean: f64,
    std: f64,
    max_baseline: f64,
    upper: f64,
    baseline_reference: f64,
    baseline_reference_normalized: f64,
    point_defect: Option<f64>,
    point_defect_normalized: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct SofteningRow {
    spring_scale: f64,
    smallest_eigenvalue: f64,
    max_residual_norm: f64,
    max_normalized_residual_norm: f64,
    max_drift_norm: f64,
    max_slew_norm: f64,
    residual_energy_ratio: f64,
}

#[derive(Clone, Debug, Serialize)]
struct PressureTestRow {
    case_name: String,
    additive_noise_std: f64,
    predictor_spring_scale: f64,
    rng_seed: u64,
    detected: bool,
    first_crossing_step: Option<usize>,
    first_crossing_time: Option<f64>,
    crossing_margin: Option<f64>,
    normalized_crossing_margin: Option<f64>,
    max_raw_residual: f64,
    max_normalized_residual: f64,
    residual_energy_ratio: f64,
}

pub fn default_output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output-dsfb-lattice")
}

pub fn run_demo(config: DemoConfig) -> Result<RunOutcome> {
    validate_config(&config)?;

    let (timestamp, run_dir) = create_timestamped_run_directory(&config.output_root)?;
    let nominal_lattice = Lattice::monatomic_fixed_chain(config.sites, 1.0, 1.0)?;
    let nominal_dynamical = nominal_lattice.dynamical_matrix()?;
    let nominal_spectrum = analyze_symmetric(&nominal_dynamical)?;
    let simulation = SimulationConfig {
        steps: config.steps,
        dt: config.dt,
        damping: config.damping,
        observed_modes: config.observed_modes.min(config.sites),
    };
    let nominal_simulation = simulate_response(
        &nominal_dynamical,
        &nominal_spectrum.eigenvectors,
        &simulation,
        0,
    );
    let normalization = normalization_metadata(config.normalization_epsilon);

    let baseline_runs: Vec<TimeSeriesBundle> = (1..=config.baseline_runs)
        .map(|variant| {
            let baseline = simulate_response(
                &nominal_dynamical,
                &nominal_spectrum.eigenvectors,
                &simulation,
                variant,
            );
            build_time_series(
                &nominal_simulation.observations,
                &baseline.observations,
                config.normalization_epsilon,
            )
        })
        .collect();
    let baseline_norms: Vec<Vec<f64>> = baseline_runs
        .iter()
        .map(|bundle| bundle.residual_norms.clone())
        .collect();
    let baseline_reference = baseline_runs
        .first()
        .cloned()
        .context("baseline ensemble unexpectedly empty")?;
    let envelope = build_envelope(
        &baseline_norms,
        config.envelope_sigma,
        config.envelope_floor,
        "clean_nominal_baseline",
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

    let point_defect_result = if config.example.includes(ExampleSelection::PointDefect) {
        Some(run_experiment(
            "point_defect",
            "Single-site mass and spring perturbation used to illustrate selective spectral motion, residual growth, and finite-time envelope crossing in a controlled harmonic chain.",
            &nominal_lattice,
            &nominal_spectrum,
            &nominal_simulation.observations,
            &simulation,
            config.normalization_epsilon,
            point_defect(
                &nominal_lattice,
                &PointDefectSpec {
                    site: config.sites / 2,
                    mass_scale: 1.40,
                    spring_index: config.sites / 2 + 1,
                    spring_scale: 0.75,
                },
            ),
        )?)
    } else {
        None
    };

    let strain_result = if config.example.includes(ExampleSelection::Strain) {
        Some(run_experiment(
            "distributed_strain",
            "Smooth spring-gradient perturbation used as a toy strain-like deformation. The goal is to show coherent spectral drift and residual structure rather than material-specific strain calibration.",
            &nominal_lattice,
            &nominal_spectrum,
            &nominal_simulation.observations,
            &simulation,
            config.normalization_epsilon,
            distributed_strain(&nominal_lattice, 0.18),
        )?)
    } else {
        None
    };

    let group_mode_result = if config.example.includes(ExampleSelection::GroupMode) {
        Some(run_experiment(
            "group_mode_cluster",
            "Clustered multi-site perturbation with correlated spring softening and mild mass loading, used to compare residual covariance against the more localized point-defect case.",
            &nominal_lattice,
            &nominal_spectrum,
            &nominal_simulation.observations,
            &simulation,
            config.normalization_epsilon,
            grouped_cluster(&nominal_lattice, config.sites / 2, 1.8, 0.38),
        )?)
    } else {
        None
    };

    let softening_result = if config.example.includes(ExampleSelection::Softening) {
        Some(run_softening_sweep(
            &nominal_lattice,
            &nominal_spectrum,
            &nominal_simulation.observations,
            &simulation,
            config.normalization_epsilon,
        )?)
    } else {
        None
    };

    let detectability = point_defect_result.as_ref().map(|point_defect| {
        evaluate_signal(
            &point_defect.simulation.residual_norms,
            &envelope,
            config.consecutive_crossings,
            config.dt,
            config.normalization_epsilon,
        )
    });

    let pressure_test_result = if config.pressure_test.enabled {
        point_defect_result
            .as_ref()
            .map(|point_defect| {
                run_pressure_test(
                    &nominal_lattice,
                    &nominal_spectrum,
                    &point_defect.lattice,
                    &simulation,
                    config.baseline_runs,
                    config.envelope_sigma,
                    config.envelope_floor,
                    config.consecutive_crossings,
                    config.normalization_epsilon,
                    &config.pressure_test,
                )
            })
            .transpose()?
    } else {
        None
    };
    let pressure_test_summary = pressure_test_result
        .as_ref()
        .map(summarize_pressure_test);

    let mut experiments = Vec::new();
    if let Some(point_defect) = &point_defect_result {
        experiments.push(point_defect);
    }
    if let Some(strain) = &strain_result {
        experiments.push(strain);
    }
    if let Some(group_mode) = &group_mode_result {
        experiments.push(group_mode);
    }

    write_config_json(
        &run_dir,
        &timestamp,
        &config,
        &nominal_lattice,
        &simulation,
        &normalization,
        &envelope.provenance,
    )?;
    write_primary_csvs(
        &run_dir,
        &nominal_spectrum,
        &nominal_simulation.observations,
        &baseline_reference,
        &envelope,
        point_defect_result.as_ref(),
        detectability.as_ref(),
        &softening_result,
        pressure_test_result.as_ref(),
        experiments.as_slice(),
        config.dt,
    )?;

    let zip_path = config.output_root.join(format!("{timestamp}.zip"));
    let placeholder_summary = RunSummary {
        run_dir: path_string(&run_dir),
        selected_example: config.example.as_str().to_string(),
        nominal_sites: nominal_lattice.sites,
        nominal_smallest_eigenvalue: *nominal_spectrum.eigenvalues.first().unwrap_or(&0.0),
        nominal_largest_eigenvalue: *nominal_spectrum.eigenvalues.last().unwrap_or(&0.0),
        normalization: normalization.clone(),
        envelope_provenance: envelope.provenance.clone(),
        experiments: experiments.iter().map(|experiment| summarize_experiment(experiment)).collect(),
        detectability: detectability.clone(),
        pressure_test: pressure_test_summary.clone(),
        softening: softening_result.as_ref().map(summarize_softening),
        limitations: limitations(),
        figures: Vec::new(),
        report_markdown: path_string(&run_dir.join("report.md")),
        report_pdf: path_string(&run_dir.join("report.pdf")),
        zip_archive: path_string(&zip_path),
    };

    let report_artifacts = write_reports(
        &run_dir,
        &nominal_spectrum,
        point_defect_result.as_ref(),
        strain_result.as_ref(),
        group_mode_result.as_ref(),
        &envelope,
        &baseline_reference.residual_norms,
        detectability.as_ref(),
        softening_result.as_ref(),
        pressure_test_result.as_ref(),
        &placeholder_summary,
        config.dt,
    )?;

    let summary = RunSummary {
        figures: report_artifacts
            .figure_paths
            .iter()
            .map(|path| path_string(path))
            .collect(),
        report_markdown: path_string(&report_artifacts.markdown_path),
        report_pdf: path_string(&report_artifacts.pdf_path),
        ..placeholder_summary
    };

    let summary_path = run_dir.join("summary.json");
    write_json_pretty(&summary_path, &summary)?;
    if let Some(pressure_test_summary) = &summary.pressure_test {
        write_json_pretty(
            &run_dir.join("pressure_test_summary.json"),
            pressure_test_summary,
        )?;
    }
    write_pdf_report(
        &report_artifacts.pdf_path,
        &run_dir,
        &summary,
        &report_artifacts.figure_paths,
    )?;
    zip_directory(&run_dir, &zip_path)?;

    Ok(RunOutcome {
        run_dir,
        summary_json: summary_path,
        report_pdf: report_artifacts.pdf_path,
        zip_path,
    })
}

fn validate_config(config: &DemoConfig) -> Result<()> {
    if config.sites < 4 {
        bail!("at least four sites are required");
    }
    if config.steps < 32 {
        bail!("at least 32 steps are required");
    }
    if config.dt <= 0.0 || config.damping < 0.0 {
        bail!("dt must be positive and damping must be non-negative");
    }
    if config.observed_modes == 0 || config.observed_modes > config.sites {
        bail!("observed_modes must be between 1 and sites");
    }
    if config.baseline_runs == 0 {
        bail!("baseline_runs must be positive");
    }
    if config.consecutive_crossings == 0 {
        bail!("consecutive_crossings must be positive");
    }
    if config.normalization_epsilon <= 0.0 {
        bail!("normalization_epsilon must be positive");
    }
    if config.pressure_test.observation_noise_std < 0.0 {
        bail!("pressure_test observation_noise_std must be non-negative");
    }
    if config.pressure_test.predictor_spring_scale <= 0.0 {
        bail!("pressure_test predictor_spring_scale must be positive");
    }
    Ok(())
}

fn run_experiment(
    name: &str,
    description: &str,
    nominal_lattice: &Lattice,
    nominal_spectrum: &SpectrumAnalysis,
    nominal_observations: &[nalgebra::DVector<f64>],
    simulation: &SimulationConfig,
    normalization_epsilon: f64,
    perturbed_lattice: Lattice,
) -> Result<ExperimentResult> {
    let nominal_dynamical = nominal_lattice.dynamical_matrix()?;
    let perturbed_dynamical = perturbed_lattice.dynamical_matrix()?;
    let delta = &perturbed_dynamical - &nominal_dynamical;
    let spectrum = analyze_symmetric(&perturbed_dynamical)?;
    let comparison = compare_spectra(name, nominal_spectrum, &spectrum, &delta)?;
    let perturbed_simulation = simulate_response(
        &perturbed_dynamical,
        &nominal_spectrum.eigenvectors,
        simulation,
        0,
    );
    let simulation = build_time_series(
        nominal_observations,
        &perturbed_simulation.observations,
        normalization_epsilon,
    );
    let covariance = covariance_matrix(&simulation.residuals);

    Ok(ExperimentResult {
        name: name.to_string(),
        description: description.to_string(),
        lattice: perturbed_lattice,
        spectrum,
        comparison,
        simulation,
        covariance,
    })
}

fn run_softening_sweep(
    nominal_lattice: &Lattice,
    nominal_spectrum: &SpectrumAnalysis,
    nominal_observations: &[nalgebra::DVector<f64>],
    simulation: &SimulationConfig,
    normalization_epsilon: f64,
) -> Result<SofteningSweepResult> {
    let scales = vec![1.00, 0.92, 0.84, 0.76, 0.68, 0.60, 0.52, 0.44, 0.36, 0.28, 0.20, 0.14];
    let mut smallest_eigenvalues = Vec::with_capacity(scales.len());
    let mut max_residual_norms = Vec::with_capacity(scales.len());
    let mut max_normalized_residual_norms = Vec::with_capacity(scales.len());
    let mut max_drift_norms = Vec::with_capacity(scales.len());
    let mut max_slew_norms = Vec::with_capacity(scales.len());
    let mut residual_energy_ratios = Vec::with_capacity(scales.len());

    for scale in &scales {
        let lattice = global_softening(nominal_lattice, *scale);
        let dynamical = lattice.dynamical_matrix()?;
        let spectrum = analyze_symmetric(&dynamical)?;
        let response = simulate_response(&dynamical, &nominal_spectrum.eigenvectors, simulation, 0);
        let bundle = build_time_series(
            nominal_observations,
            &response.observations,
            normalization_epsilon,
        );
        smallest_eigenvalues.push(*spectrum.eigenvalues.first().unwrap_or(&0.0));
        max_residual_norms.push(bundle.residual_norms.iter().copied().fold(0.0_f64, f64::max));
        max_normalized_residual_norms.push(
            bundle
                .normalized_residual_norms
                .iter()
                .copied()
                .fold(0.0_f64, f64::max),
        );
        max_drift_norms.push(bundle.drift_norms.iter().copied().fold(0.0_f64, f64::max));
        max_slew_norms.push(bundle.slew_norms.iter().copied().fold(0.0_f64, f64::max));
        residual_energy_ratios.push(bundle.residual_energy_ratio);
    }

    Ok(SofteningSweepResult {
        scales,
        smallest_eigenvalues,
        max_residual_norms,
        max_normalized_residual_norms,
        max_drift_norms,
        max_slew_norms,
        residual_energy_ratios,
    })
}

fn run_pressure_test(
    nominal_lattice: &Lattice,
    nominal_spectrum: &SpectrumAnalysis,
    point_defect_lattice: &Lattice,
    simulation: &SimulationConfig,
    baseline_runs: usize,
    envelope_sigma: f64,
    envelope_floor: f64,
    consecutive_crossings: usize,
    normalization_epsilon: f64,
    settings: &PressureTestSettings,
) -> Result<PressureTestResult> {
    let nominal_dynamical = nominal_lattice.dynamical_matrix()?;
    let point_defect_dynamical = point_defect_lattice.dynamical_matrix()?;
    let case_specs = [
        (
            "clean",
            "Reference case with no additive observation noise and no predictor mismatch.",
            0.0,
            1.0,
            settings.rng_seed,
        ),
        (
            "noise_only",
            "Controlled synthetic pressure test with additive observation noise only.",
            settings.observation_noise_std,
            1.0,
            settings.rng_seed.wrapping_add(10_000),
        ),
        (
            "mismatch_only",
            "Controlled synthetic pressure test with predictor spring-scale mismatch only.",
            0.0,
            settings.predictor_spring_scale,
            settings.rng_seed.wrapping_add(20_000),
        ),
        (
            "noise_plus_mismatch",
            "Controlled synthetic pressure test with both additive observation noise and predictor mismatch.",
            settings.observation_noise_std,
            settings.predictor_spring_scale,
            settings.rng_seed.wrapping_add(30_000),
        ),
    ];
    let mut cases = Vec::with_capacity(case_specs.len());

    for (case_name, description, additive_noise_std, predictor_spring_scale, case_seed) in case_specs {
        let predictor_observations = simulate_predictor_observations(
            nominal_lattice,
            nominal_spectrum,
            simulation,
            predictor_spring_scale,
        )?;
        let baseline_runs = (1..=baseline_runs)
            .map(|variant| {
                let baseline = simulate_response(
                    &nominal_dynamical,
                    &nominal_spectrum.eigenvectors,
                    simulation,
                    variant,
                );
                let measured = add_observation_noise(
                    &baseline.observations,
                    additive_noise_std,
                    case_seed.wrapping_add(variant as u64),
                );
                build_time_series(
                    &predictor_observations,
                    &measured,
                    normalization_epsilon,
                )
            })
            .collect::<Vec<_>>();
        let baseline_reference = baseline_runs
            .first()
            .cloned()
            .context("pressure-test baseline ensemble unexpectedly empty")?;
        let baseline_norms = baseline_runs
            .iter()
            .map(|bundle| bundle.residual_norms.clone())
            .collect::<Vec<_>>();
        let envelope = build_envelope(
            &baseline_norms,
            envelope_sigma,
            envelope_floor,
            case_name,
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
        let point_defect_signal = simulate_response(
            &point_defect_dynamical,
            &nominal_spectrum.eigenvectors,
            simulation,
            0,
        );
        let measured = add_observation_noise(
            &point_defect_signal.observations,
            additive_noise_std,
            case_seed.wrapping_add(50_000),
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
        );
        cases.push(PressureTestCaseResult {
            case_name: case_name.to_string(),
            description: description.to_string(),
            additive_noise_std,
            predictor_spring_scale,
            rng_seed: case_seed,
            baseline_reference,
            envelope,
            signal_bundle,
            detectability,
        });
    }

    Ok(PressureTestResult { cases })
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

fn normalization_metadata(normalization_epsilon: f64) -> NormalizationMetadata {
    NormalizationMetadata {
        method: "residual_norm_relative_to_predicted_signal".to_string(),
        denominator: "||y_pred(t)||_2 + epsilon".to_string(),
        epsilon: normalization_epsilon,
        note: "The normalized residual norm is defined pointwise as ||r(t)||_2 / (||y_pred(t)||_2 + epsilon). The residual energy ratio is sum_t ||r(t)||_2^2 / (sum_t ||y_pred(t)||_2^2 + epsilon).".to_string(),
    }
}

fn summarize_experiment(experiment: &ExperimentResult) -> ExperimentSummary {
    ExperimentSummary {
        name: experiment.name.clone(),
        description: experiment.description.clone(),
        delta_norm_2: experiment.comparison.delta_norm_2,
        max_abs_shift: experiment.comparison.max_abs_shift,
        max_shift_ratio: experiment.comparison.max_shift_ratio,
        bound_satisfied: experiment.comparison.bound_satisfied,
        max_residual_norm: experiment
            .simulation
            .residual_norms
            .iter()
            .copied()
            .fold(0.0_f64, f64::max),
        max_normalized_residual_norm: experiment
            .simulation
            .normalized_residual_norms
            .iter()
            .copied()
            .fold(0.0_f64, f64::max),
        max_drift_norm: experiment
            .simulation
            .drift_norms
            .iter()
            .copied()
            .fold(0.0_f64, f64::max),
        max_slew_norm: experiment
            .simulation
            .slew_norms
            .iter()
            .copied()
            .fold(0.0_f64, f64::max),
        residual_energy_ratio: experiment.simulation.residual_energy_ratio,
        covariance_trace: covariance_trace(&experiment.covariance),
        covariance_offdiag_energy: offdiag_energy(&experiment.covariance),
    }
}

fn summarize_softening(softening: &SofteningSweepResult) -> SofteningSummary {
    let index = softening.scales.len() - 1;
    SofteningSummary {
        softest_scale: softening.scales[index],
        softest_smallest_eigenvalue: softening.smallest_eigenvalues[index],
        softest_max_residual_norm: softening.max_residual_norms[index],
        softest_max_normalized_residual_norm: softening.max_normalized_residual_norms[index],
        softest_max_drift_norm: softening.max_drift_norms[index],
        softest_max_slew_norm: softening.max_slew_norms[index],
        softest_residual_energy_ratio: softening.residual_energy_ratios[index],
    }
}

fn summarize_pressure_test(result: &PressureTestResult) -> PressureTestSummary {
    PressureTestSummary {
        description: "Controlled synthetic pressure test comparing clean, additive-noise, predictor-mismatch, and combined cases. Each case uses its own baseline-derived envelope under the same configuration, so the comparison remains regime-specific rather than universal.".to_string(),
        cases: result
            .cases
            .iter()
            .map(|case| PressureTestCaseSummary {
                case_name: case.case_name.clone(),
                description: case.description.clone(),
                additive_noise_std: case.additive_noise_std,
                predictor_spring_scale: case.predictor_spring_scale,
                rng_seed: case.rng_seed,
                detected: case.detectability.first_crossing_step.is_some(),
                first_crossing_time: case.detectability.first_crossing_time,
                first_crossing_step: case.detectability.first_crossing_step,
                crossing_margin: case.detectability.crossing_margin,
                normalized_crossing_margin: case.detectability.normalized_crossing_margin,
                max_raw_residual: case
                    .signal_bundle
                    .residual_norms
                    .iter()
                    .copied()
                    .fold(0.0_f64, f64::max),
                max_normalized_residual: case
                    .signal_bundle
                    .normalized_residual_norms
                    .iter()
                    .copied()
                    .fold(0.0_f64, f64::max),
                residual_energy_ratio: case.signal_bundle.residual_energy_ratio,
                envelope_provenance: case.envelope.provenance.clone(),
            })
            .collect(),
    }
}

fn write_config_json(
    run_dir: &std::path::Path,
    timestamp: &str,
    config: &DemoConfig,
    nominal_lattice: &Lattice,
    simulation: &SimulationConfig,
    normalization: &NormalizationMetadata,
    envelope_provenance: &EnvelopeProvenance,
) -> Result<()> {
    let config_json = json!({
        "timestamp": timestamp,
        "selected_example": config.example.as_str(),
        "output_root": path_string(&config.output_root),
        "simulation": simulation,
        "normalization": normalization,
        "envelope": envelope_provenance,
        "pressure_test": config.pressure_test,
        "nominal_lattice": nominal_lattice,
        "perturbation_specs": {
            "point_defect": {
                "site": config.sites / 2,
                "mass_scale": 1.40,
                "spring_index": config.sites / 2 + 1,
                "spring_scale": 0.75
            },
            "distributed_strain": {
                "strength": 0.18
            },
            "group_mode_cluster": {
                "center": config.sites / 2,
                "width": 1.8,
                "strength": 0.38
            },
            "softening_scales": [1.00, 0.92, 0.84, 0.76, 0.68, 0.60, 0.52, 0.44, 0.36, 0.28, 0.20, 0.14]
        }
    });
    write_json_pretty(&run_dir.join("config.json"), &config_json)
}

fn write_primary_csvs(
    run_dir: &std::path::Path,
    nominal_spectrum: &SpectrumAnalysis,
    nominal_observations: &[nalgebra::DVector<f64>],
    baseline_reference: &TimeSeriesBundle,
    envelope: &Envelope,
    point_defect: Option<&ExperimentResult>,
    detectability: Option<&DetectabilitySummary>,
    softening: &Option<SofteningSweepResult>,
    pressure_test: Option<&PressureTestResult>,
    experiments: &[&ExperimentResult],
    dt: f64,
) -> Result<()> {
    let eigenvalues_nominal: Vec<EigenvalueRow> = nominal_spectrum
        .eigenvalues
        .iter()
        .copied()
        .zip(nominal_spectrum.frequencies.iter().copied())
        .enumerate()
        .map(|(index, (eigenvalue, frequency))| EigenvalueRow {
            mode: index + 1,
            eigenvalue,
            frequency,
        })
        .collect();
    write_csv_rows(&run_dir.join("eigenvalues_nominal.csv"), &eigenvalues_nominal)?;

    let mut perturbed_rows = Vec::new();
    for experiment in experiments {
        for (index, (eigenvalue, frequency)) in experiment
            .spectrum
            .eigenvalues
            .iter()
            .copied()
            .zip(experiment.spectrum.frequencies.iter().copied())
            .enumerate()
        {
            perturbed_rows.push(PerturbedEigenvalueRow {
                experiment: experiment.name.clone(),
                mode: index + 1,
                eigenvalue,
                frequency,
            });
        }
    }
    write_csv_rows(&run_dir.join("eigenvalues_perturbed.csv"), &perturbed_rows)?;

    let nominal_rows = nominal_observations
        .iter()
        .enumerate()
        .flat_map(|(step, observation)| {
            observation.iter().enumerate().map(move |(channel, value)| ObservationRow {
                step,
                time: step as f64 * dt,
                channel: channel + 1,
                value: *value,
            })
        })
        .collect::<Vec<_>>();
    write_csv_rows(&run_dir.join("nominal_observations.csv"), &nominal_rows)?;

    let mut residual_rows = Vec::new();
    let mut residual_norm_rows = Vec::new();
    let mut drift_rows = Vec::new();
    let mut slew_rows = Vec::new();
    append_signal_rows(
        &mut residual_rows,
        &mut residual_norm_rows,
        &mut drift_rows,
        &mut slew_rows,
        "baseline_tolerance",
        "main",
        baseline_reference,
        dt,
    );
    for experiment in experiments {
        append_signal_rows(
            &mut residual_rows,
            &mut residual_norm_rows,
            &mut drift_rows,
            &mut slew_rows,
            &experiment.name,
            "main",
            &experiment.simulation,
            dt,
        );
    }
    write_csv_rows(&run_dir.join("residual_timeseries.csv"), &residual_rows)?;
    write_csv_rows(
        &run_dir.join("normalized_residual_norm_timeseries.csv"),
        &residual_norm_rows,
    )?;
    write_csv_rows(&run_dir.join("drift_timeseries.csv"), &drift_rows)?;
    write_csv_rows(&run_dir.join("slew_timeseries.csv"), &slew_rows)?;

    let mut covariance_rows = Vec::new();
    for experiment in experiments {
        for row in 0..experiment.covariance.nrows() {
            for column in 0..experiment.covariance.ncols() {
                covariance_rows.push(CovarianceRow {
                    experiment: experiment.name.clone(),
                    row: row + 1,
                    column: column + 1,
                    value: experiment.covariance[(row, column)],
                });
            }
        }
    }
    write_csv_rows(&run_dir.join("covariance.csv"), &covariance_rows)?;

    let metrics = build_metric_rows(
        envelope,
        baseline_reference,
        point_defect,
        detectability,
        softening,
        pressure_test,
        experiments,
        dt,
    );
    write_csv_rows(&run_dir.join("metrics.csv"), &metrics)?;

    let envelope_rows: Vec<EnvelopeRow> = envelope
        .upper
        .iter()
        .enumerate()
        .map(|(step, upper)| EnvelopeRow {
            step,
            time: step as f64 * dt,
            mean: envelope.mean[step],
            std: envelope.std[step],
            max_baseline: envelope.max_baseline[step],
            upper: *upper,
            baseline_reference: baseline_reference.residual_norms[step],
            baseline_reference_normalized: baseline_reference.normalized_residual_norms[step],
            point_defect: point_defect.map(|experiment| experiment.simulation.residual_norms[step]),
            point_defect_normalized: point_defect
                .map(|experiment| experiment.simulation.normalized_residual_norms[step]),
        })
        .collect();
    write_csv_rows(&run_dir.join("envelope_timeseries.csv"), &envelope_rows)?;

    if let Some(pressure_test) = pressure_test {
        let rows = pressure_test
            .cases
            .iter()
            .map(|case| PressureTestRow {
                case_name: case.case_name.clone(),
                additive_noise_std: case.additive_noise_std,
                predictor_spring_scale: case.predictor_spring_scale,
                rng_seed: case.rng_seed,
                detected: case.detectability.first_crossing_step.is_some(),
                first_crossing_step: case.detectability.first_crossing_step,
                first_crossing_time: case.detectability.first_crossing_time,
                crossing_margin: case.detectability.crossing_margin,
                normalized_crossing_margin: case.detectability.normalized_crossing_margin,
                max_raw_residual: case
                    .signal_bundle
                    .residual_norms
                    .iter()
                    .copied()
                    .fold(0.0_f64, f64::max),
                max_normalized_residual: case
                    .signal_bundle
                    .normalized_residual_norms
                    .iter()
                    .copied()
                    .fold(0.0_f64, f64::max),
                residual_energy_ratio: case.signal_bundle.residual_energy_ratio,
            })
            .collect::<Vec<_>>();
        write_csv_rows(&run_dir.join("pressure_test_summary.csv"), &rows)?;
        for case in &pressure_test.cases {
            append_residual_norm_rows(
                &mut residual_norm_rows,
                "point_defect",
                &case.case_name,
                &case.signal_bundle,
                dt,
            );
        }
        write_csv_rows(
            &run_dir.join("normalized_residual_norm_timeseries.csv"),
            &residual_norm_rows,
        )?;
    }

    if let Some(softening) = softening {
        let rows = softening
            .scales
            .iter()
            .copied()
            .zip(softening.smallest_eigenvalues.iter().copied())
            .zip(softening.max_residual_norms.iter().copied())
            .zip(softening.max_normalized_residual_norms.iter().copied())
            .zip(softening.max_drift_norms.iter().copied())
            .zip(softening.max_slew_norms.iter().copied())
            .zip(softening.residual_energy_ratios.iter().copied())
            .map(
                |((((((spring_scale, smallest_eigenvalue), max_residual_norm), max_normalized_residual_norm), max_drift_norm), max_slew_norm), residual_energy_ratio)| SofteningRow {
                    spring_scale,
                    smallest_eigenvalue,
                    max_residual_norm,
                    max_normalized_residual_norm,
                    max_drift_norm,
                    max_slew_norm,
                    residual_energy_ratio,
                },
            )
            .collect::<Vec<_>>();
        write_csv_rows(&run_dir.join("softening_sweep.csv"), &rows)?;
    }

    Ok(())
}

fn append_signal_rows(
    residual_rows: &mut Vec<ResidualRow>,
    residual_norm_rows: &mut Vec<ResidualNormRow>,
    drift_rows: &mut Vec<SignalRow>,
    slew_rows: &mut Vec<SignalRow>,
    experiment: &str,
    case: &str,
    bundle: &TimeSeriesBundle,
    dt: f64,
) {
    append_residual_norm_rows(residual_norm_rows, experiment, case, bundle, dt);
    for step in 0..bundle.residuals.len() {
        for channel in 0..bundle.residuals[step].len() {
            residual_rows.push(ResidualRow {
                experiment: experiment.to_string(),
                step,
                time: step as f64 * dt,
                channel: channel + 1,
                predicted: bundle.predicted[step][channel],
                measured: bundle.measured[step][channel],
                residual: bundle.residuals[step][channel],
            });
            drift_rows.push(SignalRow {
                experiment: experiment.to_string(),
                step,
                time: step as f64 * dt,
                channel: channel + 1,
                value: bundle.drifts[step][channel],
            });
            slew_rows.push(SignalRow {
                experiment: experiment.to_string(),
                step,
                time: step as f64 * dt,
                channel: channel + 1,
                value: bundle.slews[step][channel],
            });
        }
    }
}

fn append_residual_norm_rows(
    residual_norm_rows: &mut Vec<ResidualNormRow>,
    experiment: &str,
    case: &str,
    bundle: &TimeSeriesBundle,
    dt: f64,
) {
    for step in 0..bundle.residual_norms.len() {
        residual_norm_rows.push(ResidualNormRow {
            experiment: experiment.to_string(),
            case: case.to_string(),
            step,
            time: step as f64 * dt,
            predicted_norm: bundle.predicted_norms[step],
            measured_norm: bundle.measured_norms[step],
            raw_residual_norm: bundle.residual_norms[step],
            normalized_residual_norm: bundle.normalized_residual_norms[step],
        });
    }
}

fn build_metric_rows(
    envelope: &Envelope,
    baseline_reference: &TimeSeriesBundle,
    point_defect: Option<&ExperimentResult>,
    detectability: Option<&DetectabilitySummary>,
    softening: &Option<SofteningSweepResult>,
    pressure_test: Option<&PressureTestResult>,
    experiments: &[&ExperimentResult],
    dt: f64,
) -> Vec<MetricRow> {
    let mut rows = Vec::new();
    let (global_envelope_peak, global_envelope_peak_time) = envelope
        .upper
        .iter()
        .copied()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap())
        .map(|(step, value)| (value, step as f64))
        .unwrap_or((0.0, 0.0));

    rows.push(MetricRow {
        experiment: "baseline".to_string(),
        metric: "global_envelope_peak".to_string(),
        value: global_envelope_peak,
        units: "residual_norm".to_string(),
        note: "Global peak of the baseline-derived envelope over the whole trajectory; this peak alone does not determine detectability.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "baseline".to_string(),
        metric: "global_envelope_peak_time".to_string(),
        value: global_envelope_peak_time * dt,
        units: "time".to_string(),
        note: "Time at which the global envelope peak occurs.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "baseline".to_string(),
        metric: "baseline_reference_peak".to_string(),
        value: baseline_reference
            .residual_norms
            .iter()
            .copied()
            .fold(0.0_f64, f64::max),
        units: "residual_norm".to_string(),
        note: "Representative nominal baseline residual peak relative to the clean predictor.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "baseline".to_string(),
        metric: "baseline_reference_normalized_peak".to_string(),
        value: baseline_reference
            .normalized_residual_norms
            .iter()
            .copied()
            .fold(0.0_f64, f64::max),
        units: "ratio".to_string(),
        note: "Largest normalized residual norm among the baseline-derived reference runs.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "baseline".to_string(),
        metric: "baseline_reference_signal_peak".to_string(),
        value: envelope.provenance.baseline_reference_signal_peak,
        units: "observation_norm".to_string(),
        note: "Peak predicted signal norm used in the denominator of the normalized residual metric.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "baseline".to_string(),
        metric: "baseline_reference_signal_energy".to_string(),
        value: envelope.provenance.baseline_reference_signal_energy,
        units: "observation_energy".to_string(),
        note: "Predicted signal energy over the observation window for the baseline reference case.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "envelope".to_string(),
        metric: "baseline_runs".to_string(),
        value: envelope.provenance.parameters.baseline_runs as f64,
        units: "count".to_string(),
        note: "Number of baseline runs used to derive the regime-specific envelope.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "envelope".to_string(),
        metric: "sigma_multiplier".to_string(),
        value: envelope.provenance.parameters.sigma_multiplier,
        units: "sigma".to_string(),
        note: "Multiplier applied to the baseline residual-norm standard deviation in the envelope formula.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "envelope".to_string(),
        metric: "additive_floor".to_string(),
        value: envelope.provenance.parameters.additive_floor,
        units: "residual_norm".to_string(),
        note: "Additive floor applied after the mean-plus-sigma envelope estimate.".to_string(),
    });
    rows.push(MetricRow {
        experiment: "envelope".to_string(),
        metric: "baseline_ensemble_peak".to_string(),
        value: envelope.provenance.baseline_ensemble_peak,
        units: "residual_norm".to_string(),
        note: "Largest baseline residual norm observed across the baseline ensemble.".to_string(),
    });

    for experiment in experiments {
        let summary = summarize_experiment(experiment);
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "delta_norm_2".to_string(),
            value: summary.delta_norm_2,
            units: "eigenvalue".to_string(),
            note: "Spectral norm of Delta = D' - D.".to_string(),
        });
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "max_abs_shift".to_string(),
            value: summary.max_abs_shift,
            units: "eigenvalue".to_string(),
            note: "Largest absolute eigenvalue shift after sorting the symmetric spectrum.".to_string(),
        });
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "bound_satisfied_numeric".to_string(),
            value: if summary.bound_satisfied { 1.0 } else { 0.0 },
            units: "boolean".to_string(),
            note: "Numeric illustration of |lambda_i' - lambda_i| <= ||Delta||_2 on this toy example.".to_string(),
        });
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "max_residual_norm".to_string(),
            value: summary.max_residual_norm,
            units: "residual_norm".to_string(),
            note: "Largest observation residual norm over time.".to_string(),
        });
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "max_normalized_residual_norm".to_string(),
            value: summary.max_normalized_residual_norm,
            units: "ratio".to_string(),
            note: "Largest normalized residual norm using ||r(t)||_2 / (||y_pred(t)||_2 + epsilon).".to_string(),
        });
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "residual_energy_ratio".to_string(),
            value: summary.residual_energy_ratio,
            units: "ratio".to_string(),
            note: "Observation-window residual energy divided by predicted signal energy.".to_string(),
        });
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "max_drift_norm".to_string(),
            value: summary.max_drift_norm,
            units: "drift_norm".to_string(),
            note: "Largest drift norm over time.".to_string(),
        });
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "max_slew_norm".to_string(),
            value: summary.max_slew_norm,
            units: "slew_norm".to_string(),
            note: "Largest slew norm over time.".to_string(),
        });
        rows.push(MetricRow {
            experiment: experiment.name.clone(),
            metric: "covariance_offdiag_energy".to_string(),
            value: summary.covariance_offdiag_energy,
            units: "covariance".to_string(),
            note: "Residual covariance off-diagonal energy used as a compact multi-channel correlation summary.".to_string(),
        });
    }

    if let Some(point_defect) = point_defect {
        rows.push(MetricRow {
            experiment: "point_defect".to_string(),
            metric: "max_shift_ratio".to_string(),
            value: point_defect.comparison.max_shift_ratio,
            units: "ratio".to_string(),
            note: "max_i |lambda_i' - lambda_i| / ||Delta||_2".to_string(),
        });
    }

    if let Some(detectability) = detectability {
        rows.push(MetricRow {
            experiment: "detectability".to_string(),
            metric: "global_signal_peak".to_string(),
            value: detectability.global_signal_peak,
            units: "residual_norm".to_string(),
            note: "Global peak of the point-defect residual norm over the whole trajectory.".to_string(),
        });
        rows.push(MetricRow {
            experiment: "detectability".to_string(),
            metric: "global_signal_peak_time".to_string(),
            value: detectability.global_signal_peak_time,
            units: "time".to_string(),
            note: "Time at which the global signal peak occurs.".to_string(),
        });
        rows.push(MetricRow {
            experiment: "detectability".to_string(),
            metric: "global_envelope_peak".to_string(),
            value: detectability.global_envelope_peak,
            units: "residual_norm".to_string(),
            note: "Global peak of the detectability envelope over the whole trajectory.".to_string(),
        });
        rows.push(MetricRow {
            experiment: "detectability".to_string(),
            metric: "global_envelope_peak_time".to_string(),
            value: detectability.global_envelope_peak_time,
            units: "time".to_string(),
            note: "Time at which the global envelope peak occurs.".to_string(),
        });

        if let Some(step) = detectability.first_crossing_step {
            rows.push(MetricRow {
                experiment: "detectability".to_string(),
                metric: "first_crossing_step".to_string(),
                value: step as f64,
                units: "step".to_string(),
                note: "First time index where the pointwise condition signal(t) > envelope(t) is satisfied.".to_string(),
            });
        }
        if let Some(time) = detectability.first_crossing_time {
            rows.push(MetricRow {
                experiment: "detectability".to_string(),
                metric: "first_crossing_time".to_string(),
                value: time,
                units: "time".to_string(),
                note: "Physical time at the first pointwise detectability crossing.".to_string(),
            });
        }
        if let Some(value) = detectability.signal_at_first_crossing {
            rows.push(MetricRow {
                experiment: "detectability".to_string(),
                metric: "signal_at_first_crossing".to_string(),
                value,
                units: "residual_norm".to_string(),
                note: "Residual norm evaluated at the first crossing time.".to_string(),
            });
        }
        if let Some(value) = detectability.envelope_at_first_crossing {
            rows.push(MetricRow {
                experiment: "detectability".to_string(),
                metric: "envelope_at_first_crossing".to_string(),
                value,
                units: "residual_norm".to_string(),
                note: "Envelope value evaluated at the first crossing time.".to_string(),
            });
        }
        if let Some(value) = detectability.crossing_margin {
            rows.push(MetricRow {
                experiment: "detectability".to_string(),
                metric: "crossing_margin".to_string(),
                value,
                units: "residual_norm".to_string(),
                note: "Pointwise margin signal_at_first_crossing - envelope_at_first_crossing.".to_string(),
            });
        }
        if let Some(value) = detectability.normalized_crossing_margin {
            rows.push(MetricRow {
                experiment: "detectability".to_string(),
                metric: "normalized_crossing_margin".to_string(),
                value,
                units: "ratio".to_string(),
                note: "Pointwise crossing margin divided by the envelope value at the first crossing.".to_string(),
            });
        }
    }

    if let Some(softening) = softening {
        let last = softening.scales.len() - 1;
        rows.push(MetricRow {
            experiment: "softening".to_string(),
            metric: "softest_scale".to_string(),
            value: softening.scales[last],
            units: "scale".to_string(),
            note: "Smallest global spring scale included in the sweep.".to_string(),
        });
        rows.push(MetricRow {
            experiment: "softening".to_string(),
            metric: "softest_smallest_eigenvalue".to_string(),
            value: softening.smallest_eigenvalues[last],
            units: "eigenvalue".to_string(),
            note: "Smallest eigenvalue at the softest global spring scale.".to_string(),
        });
        rows.push(MetricRow {
            experiment: "softening".to_string(),
            metric: "softest_max_residual_norm".to_string(),
            value: softening.max_residual_norms[last],
            units: "residual_norm".to_string(),
            note: "Largest residual norm at the softest global spring scale.".to_string(),
        });
        rows.push(MetricRow {
            experiment: "softening".to_string(),
            metric: "softest_max_normalized_residual_norm".to_string(),
            value: softening.max_normalized_residual_norms[last],
            units: "ratio".to_string(),
            note: "Largest normalized residual norm at the softest global spring scale.".to_string(),
        });
        rows.push(MetricRow {
            experiment: "softening".to_string(),
            metric: "softest_residual_energy_ratio".to_string(),
            value: softening.residual_energy_ratios[last],
            units: "ratio".to_string(),
            note: "Residual energy ratio at the softest global spring scale.".to_string(),
        });
    }

    if let Some(pressure_test) = pressure_test {
        for case in &pressure_test.cases {
            let label = format!("pressure_test/{}", case.case_name);
            rows.push(MetricRow {
                experiment: label.clone(),
                metric: "additive_noise_std".to_string(),
                value: case.additive_noise_std,
                units: "observation_noise".to_string(),
                note: "Configured additive observation-noise standard deviation for this synthetic pressure-test case.".to_string(),
            });
            rows.push(MetricRow {
                experiment: label.clone(),
                metric: "predictor_spring_scale".to_string(),
                value: case.predictor_spring_scale,
                units: "scale".to_string(),
                note: "Global spring scale applied to the predictor model in this synthetic mismatch case.".to_string(),
            });
            rows.push(MetricRow {
                experiment: label.clone(),
                metric: "detected".to_string(),
                value: if case.detectability.first_crossing_step.is_some() { 1.0 } else { 0.0 },
                units: "boolean".to_string(),
                note: "Whether the pointwise detectability condition was met for this pressure-test case.".to_string(),
            });
            rows.push(MetricRow {
                experiment: label.clone(),
                metric: "max_raw_residual".to_string(),
                value: case
                    .signal_bundle
                    .residual_norms
                    .iter()
                    .copied()
                    .fold(0.0_f64, f64::max),
                units: "residual_norm".to_string(),
                note: "Maximum raw residual norm for this pressure-test case.".to_string(),
            });
            rows.push(MetricRow {
                experiment: label.clone(),
                metric: "max_normalized_residual".to_string(),
                value: case
                    .signal_bundle
                    .normalized_residual_norms
                    .iter()
                    .copied()
                    .fold(0.0_f64, f64::max),
                units: "ratio".to_string(),
                note: "Maximum normalized residual norm for this pressure-test case.".to_string(),
            });
            rows.push(MetricRow {
                experiment: label.clone(),
                metric: "residual_energy_ratio".to_string(),
                value: case.signal_bundle.residual_energy_ratio,
                units: "ratio".to_string(),
                note: "Residual energy divided by predicted signal energy for this pressure-test case.".to_string(),
            });
            if let Some(time) = case.detectability.first_crossing_time {
                rows.push(MetricRow {
                    experiment: label.clone(),
                    metric: "first_crossing_time".to_string(),
                    value: time,
                    units: "time".to_string(),
                    note: "First pointwise detectability crossing time for this pressure-test case.".to_string(),
                });
            }
            if let Some(value) = case.detectability.crossing_margin {
                rows.push(MetricRow {
                    experiment: label.clone(),
                    metric: "crossing_margin".to_string(),
                    value,
                    units: "residual_norm".to_string(),
                    note: "Raw pointwise crossing margin at the first crossing for this pressure-test case.".to_string(),
                });
            }
            if let Some(value) = case.detectability.normalized_crossing_margin {
                rows.push(MetricRow {
                    experiment: label,
                    metric: "normalized_crossing_margin".to_string(),
                    value,
                    units: "ratio".to_string(),
                    note: "Crossing margin normalized by the envelope value at the first crossing for this pressure-test case.".to_string(),
                });
            }
        }
    }

    rows
}

fn limitations() -> Vec<String> {
    vec![
        "The lattice is a deterministic fixed-end harmonic toy model rather than a material-calibrated crystal simulator.".to_string(),
        "The observation model uses nominal modal coordinates under deterministic forcing. Any added noise or predictor mismatch settings are controlled synthetic pressure tests rather than calibrated sensor-noise, identification, or uncertainty models.".to_string(),
        "The spectral inequality is illustrated numerically on finite matrices and should not be read as an empirical proof of the full theoretical framework.".to_string(),
        "Detectability results depend on the baseline envelope construction used here and therefore do not establish universal thresholds or universal defect identifiability.".to_string(),
        "Normalized residual metrics improve comparability inside this crate, but they remain tied to the chosen observation scaling and denominator definition used here.".to_string(),
        "The softening sweep is a toy precursor study consistent with the paper's interpretation of approaching instability, not a claim of general phase-transition forecasting.".to_string(),
    ]
}
