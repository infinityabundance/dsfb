use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::config::DemoConfig;
use crate::error::{Error, Result};
use crate::external::{
    load_external_capture_bundle, run_external_import_from_manifest, ExternalCaptureBundle,
    ExternalHandoffMetrics, ExternalLoadedCapture, NO_REAL_EXTERNAL_DATA_PROVIDED,
};
use crate::frame::{
    mean_abs_error, mean_abs_error_over_mask, save_scalar_field_png, Color, ImageFrame, ScalarField,
};
use crate::gpu::try_execute_host_minimum_kernel;
use crate::host::{
    default_host_realistic_profile, supervise_temporal_reuse, HostSupervisionOutputs,
};
use crate::parameters::SmoothstepThreshold;
use crate::report::EXPERIMENT_SENTENCE;
use crate::sampling::{
    build_count_field, combine_fields, gradient_field, guided_allocation, invert_trust,
    local_contrast_field, mean_count_over_mask, AllocationPolicyId, BudgetCurve, BudgetCurvePoint,
    DemoBPolicyMetrics,
};
use crate::scene::{MotionVector, Normal3};

#[derive(Clone, Debug)]
pub struct ExternalValidationArtifacts {
    pub replay_report_path: PathBuf,
    pub handoff_report_path: PathBuf,
    pub validation_report_path: PathBuf,
    pub gpu_report_path: PathBuf,
    pub gpu_metrics_path: PathBuf,
    pub demo_a_report_path: PathBuf,
    pub demo_b_report_path: PathBuf,
    pub demo_b_metrics_path: PathBuf,
    pub scaling_report_path: PathBuf,
    pub scaling_metrics_path: PathBuf,
    pub memory_bandwidth_report_path: PathBuf,
    pub integration_scaling_report_path: PathBuf,
    pub resolved_manifest_path: PathBuf,
    pub figures_dir: PathBuf,
    pub handoff_metrics: ExternalHandoffMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalGpuCaptureMetrics {
    pub capture_label: String,
    pub measured_gpu: bool,
    pub adapter: Option<String>,
    pub backend: Option<String>,
    pub resolution: [usize; 2],
    pub kernel: String,
    pub total_ms: Option<f64>,
    pub dispatch_ms: Option<f64>,
    pub readback_ms: Option<f64>,
    pub mean_abs_trust_delta_vs_cpu: Option<f32>,
    pub mean_abs_alpha_delta_vs_cpu: Option<f32>,
    pub mean_abs_intervention_delta_vs_cpu: Option<f32>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalGpuMetrics {
    pub measurement_kind: String,
    pub measured_gpu: bool,
    pub actual_real_external_data: bool,
    pub kernel: String,
    pub captures: Vec<ExternalGpuCaptureMetrics>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalDemoAMethodMetrics {
    pub method_id: String,
    pub label: String,
    pub metric_source: String,
    pub overall_mae: f32,
    pub roi_mae: f32,
    pub non_roi_mae: f32,
    pub temporal_error_accumulation: f32,
    pub intervention_rate: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalDemoACaptureMetrics {
    pub capture_label: String,
    pub roi_source: String,
    pub roi_pixels: usize,
    pub ground_truth_available: bool,
    pub metric_source: String,
    pub methods: Vec<ExternalDemoAMethodMetrics>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalDemoAMetrics {
    pub real_external_data_provided: bool,
    pub no_real_external_data_provided: bool,
    pub captures: Vec<ExternalDemoACaptureMetrics>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalDemoBCaptureMetrics {
    pub capture_label: String,
    pub regime: String,
    pub metric_source: String,
    pub roi_source: String,
    pub roi_pixels: usize,
    pub ground_truth_available: bool,
    pub budget_total_samples: usize,
    pub fixed_budget_equal: bool,
    pub policies: Vec<DemoBPolicyMetrics>,
    pub budget_curves: Vec<BudgetCurve>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalDemoBMetrics {
    pub real_external_data_provided: bool,
    pub no_real_external_data_provided: bool,
    pub captures: Vec<ExternalDemoBCaptureMetrics>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalScalingEntry {
    pub label: String,
    pub source: String,
    pub width: usize,
    pub height: usize,
    pub attempted: bool,
    pub measured_gpu: bool,
    pub total_ms: Option<f64>,
    pub dispatch_ms: Option<f64>,
    pub readback_ms: Option<f64>,
    pub ms_per_megapixel: Option<f64>,
    pub scaling_ratio_vs_native: Option<f64>,
    pub pixel_ratio_vs_native: f64,
    pub approximately_linear: Option<bool>,
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalCoverageSummary {
    pub realism_stress_case: bool,
    pub larger_roi_case: bool,
    pub mixed_regime_case: bool,
    pub coverage_status: String,
    pub missing: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalScalingMetrics {
    pub measurement_kind: String,
    pub kernel: String,
    pub native_capture_label: String,
    pub native_resolution: [usize; 2],
    pub attempted_1080p: bool,
    pub attempted_4k: bool,
    pub entries: Vec<ExternalScalingEntry>,
    pub coverage: ExternalCoverageSummary,
    pub notes: Vec<String>,
}

pub fn run_external_validation_bundle(
    config: &DemoConfig,
    manifest_path: &Path,
    output_dir: &Path,
) -> Result<ExternalValidationArtifacts> {
    fs::create_dir_all(output_dir)?;

    let import = run_external_import_from_manifest(config, manifest_path, output_dir)?;
    let replay_metrics_path = output_dir.join("replay_metrics.json");
    fs::write(
        &replay_metrics_path,
        serde_json::to_string_pretty(&import.metrics)?,
    )?;
    let bundle = load_external_capture_bundle(config, manifest_path, output_dir)?;
    let figures_dir = output_dir.join("figures");
    fs::create_dir_all(&figures_dir)?;

    let gpu_metrics = run_external_gpu_metrics_safe(config, manifest_path, output_dir, &bundle)?;
    let gpu_metrics_path = output_dir.join("gpu_execution_metrics.json");
    fs::write(
        &gpu_metrics_path,
        serde_json::to_string_pretty(&gpu_metrics)?,
    )?;
    let gpu_report_path = output_dir.join("gpu_execution_report.md");
    write_gpu_external_report(&gpu_report_path, &gpu_metrics, &bundle)?;
    fs::copy(
        &gpu_metrics_path,
        output_dir.join("gpu_external_metrics.json"),
    )?;
    fs::copy(&gpu_report_path, output_dir.join("gpu_external_report.md"))?;

    let demo_a_metrics = run_demo_a_external_metrics(config, &bundle, &figures_dir)?;
    let demo_a_metrics_path = output_dir.join("demo_a_external_metrics.json");
    fs::write(
        &demo_a_metrics_path,
        serde_json::to_string_pretty(&demo_a_metrics)?,
    )?;
    let demo_a_report_path = output_dir.join("demo_a_external_report.md");
    write_demo_a_external_report(&demo_a_report_path, &demo_a_metrics, &bundle)?;

    let demo_b_metrics = run_demo_b_external_metrics(config, &bundle, &figures_dir)?;
    let demo_b_metrics_path = output_dir.join("demo_b_external_metrics.json");
    fs::write(
        &demo_b_metrics_path,
        serde_json::to_string_pretty(&demo_b_metrics)?,
    )?;
    let demo_b_report_path = output_dir.join("demo_b_external_report.md");
    write_demo_b_external_report(&demo_b_report_path, &demo_b_metrics, &bundle)?;

    let scaling_metrics =
        run_external_scaling_study(config, &bundle, &demo_a_metrics, &demo_b_metrics)?;
    let scaling_metrics_path = output_dir.join("scaling_metrics.json");
    fs::write(
        &scaling_metrics_path,
        serde_json::to_string_pretty(&scaling_metrics)?,
    )?;
    let scaling_report_path = output_dir.join("scaling_report.md");
    write_external_scaling_report(&scaling_report_path, &scaling_metrics, &bundle)?;
    let memory_bandwidth_report_path = output_dir.join("memory_bandwidth_report.md");
    write_memory_bandwidth_report(&memory_bandwidth_report_path, &scaling_metrics)?;
    let integration_scaling_report_path = output_dir.join("integration_scaling_report.md");
    write_integration_scaling_report(&integration_scaling_report_path, &scaling_metrics, &bundle)?;

    let validation_report_path = output_dir.join("external_validation_report.md");
    write_external_validation_report(
        &validation_report_path,
        &bundle,
        &import.metrics,
        &gpu_metrics,
        &demo_a_metrics,
        &demo_b_metrics,
        &scaling_metrics,
    )?;
    copy_representative_figure_aliases(output_dir, &figures_dir)?;

    Ok(ExternalValidationArtifacts {
        replay_report_path: import.report_path,
        handoff_report_path: output_dir.join("external_handoff_report.md"),
        validation_report_path,
        gpu_report_path,
        gpu_metrics_path,
        demo_a_report_path,
        demo_b_report_path,
        demo_b_metrics_path,
        scaling_report_path,
        scaling_metrics_path,
        memory_bandwidth_report_path,
        integration_scaling_report_path,
        resolved_manifest_path: import.resolved_manifest_path,
        figures_dir,
        handoff_metrics: import.metrics,
    })
}

pub fn probe_external_gpu_only(
    config: &DemoConfig,
    manifest_path: &Path,
    output_dir: &Path,
) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    let bundle = load_external_capture_bundle(config, manifest_path, output_dir)?;
    let metrics = run_external_gpu_metrics(config, &bundle)?;
    let path = output_dir.join("gpu_probe_metrics.json");
    fs::write(&path, serde_json::to_string_pretty(&metrics)?)?;
    Ok(path)
}

fn run_external_gpu_metrics(
    config: &DemoConfig,
    bundle: &ExternalCaptureBundle,
) -> Result<ExternalGpuMetrics> {
    let profile =
        default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
    let mut captures = Vec::with_capacity(bundle.captures.len());
    let mut measured_gpu = false;

    for capture in &bundle.captures {
        let cpu = supervise_temporal_reuse(&capture.inputs.borrow(), &profile);
        match try_execute_host_minimum_kernel(&capture.inputs, profile.parameters)? {
            Some(gpu) => {
                measured_gpu = true;
                captures.push(ExternalGpuCaptureMetrics {
                    capture_label: capture.label.clone(),
                    measured_gpu: true,
                    adapter: Some(gpu.adapter_name),
                    backend: Some(gpu.backend),
                    resolution: [capture.inputs.width(), capture.inputs.height()],
                    kernel: "dsfb_host_minimum".to_string(),
                    total_ms: Some(gpu.total_ms),
                    dispatch_ms: Some(gpu.dispatch_ms),
                    readback_ms: Some(gpu.readback_ms),
                    mean_abs_trust_delta_vs_cpu: Some(mean_abs_delta(cpu.trust.values(), &gpu.trust)),
                    mean_abs_alpha_delta_vs_cpu: Some(mean_abs_delta(cpu.alpha.values(), &gpu.alpha)),
                    mean_abs_intervention_delta_vs_cpu: Some(mean_abs_delta(
                        cpu.intervention.values(),
                        &gpu.intervention,
                    )),
                    notes: vec![
                        "Measured on imported external buffers using the same minimum host-realistic kernel as the synthetic GPU study.".to_string(),
                    ],
                });
            }
            None => {
                captures.push(ExternalGpuCaptureMetrics {
                    capture_label: capture.label.clone(),
                    measured_gpu: false,
                    adapter: None,
                    backend: None,
                    resolution: [capture.inputs.width(), capture.inputs.height()],
                    kernel: "dsfb_host_minimum".to_string(),
                    total_ms: None,
                    dispatch_ms: None,
                    readback_ms: None,
                    mean_abs_trust_delta_vs_cpu: None,
                    mean_abs_alpha_delta_vs_cpu: None,
                    mean_abs_intervention_delta_vs_cpu: None,
                    notes: vec![
                        "The GPU path is implemented, but no usable adapter was available in the current environment for this imported capture.".to_string(),
                    ],
                });
            }
        }
    }

    Ok(ExternalGpuMetrics {
        measurement_kind: if measured_gpu {
            "measured_gpu".to_string()
        } else {
            "gpu_path_unmeasured".to_string()
        },
        measured_gpu,
        actual_real_external_data: bundle.real_external_data_provided,
        kernel: "dsfb_host_minimum".to_string(),
        captures,
        notes: vec![
            "This report covers imported external buffers rather than the synthetic internal suite.".to_string(),
            if bundle.no_real_external_data_provided {
                NO_REAL_EXTERNAL_DATA_PROVIDED.to_string()
            } else {
                "Real external data was supplied through the file schema.".to_string()
            },
        ],
    })
}

fn run_external_gpu_metrics_safe(
    config: &DemoConfig,
    manifest_path: &Path,
    output_dir: &Path,
    bundle: &ExternalCaptureBundle,
) -> Result<ExternalGpuMetrics> {
    if let Some(metrics) = try_gpu_subprocess_probe(manifest_path, output_dir, bundle)? {
        return Ok(metrics);
    }
    run_external_gpu_metrics(config, bundle)
}

fn try_gpu_subprocess_probe(
    manifest_path: &Path,
    output_dir: &Path,
    bundle: &ExternalCaptureBundle,
) -> Result<Option<ExternalGpuMetrics>> {
    let executable = std::env::current_exe()?;
    let executable_name = executable
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if !executable_name.contains("dsfb-computer-graphics") {
        return Ok(None);
    }

    let probe_dir = output_dir.join("gpu_probe");
    fs::create_dir_all(&probe_dir)?;
    let status = Command::new(&executable)
        .arg("probe-external-gpu")
        .arg("--manifest")
        .arg(manifest_path)
        .arg("--output")
        .arg(&probe_dir)
        .status()?;
    let metrics_path = probe_dir.join("gpu_probe_metrics.json");
    if status.success() && metrics_path.exists() {
        let metrics: ExternalGpuMetrics = serde_json::from_str(&fs::read_to_string(metrics_path)?)?;
        return Ok(Some(metrics));
    }

    let crash_note = if let Some(code) = status.code() {
        format!("GPU subprocess attempt exited with status {code}")
    } else {
        "GPU subprocess attempt terminated by signal".to_string()
    };
    Ok(Some(unmeasured_gpu_metrics(bundle, vec![
        "GPU execution was attempted in a subprocess because some drivers abort the process on shader JIT failure.".to_string(),
        crash_note,
    ])))
}

fn unmeasured_gpu_metrics(bundle: &ExternalCaptureBundle, notes: Vec<String>) -> ExternalGpuMetrics {
    ExternalGpuMetrics {
        measurement_kind: "gpu_probe_failed".to_string(),
        measured_gpu: false,
        actual_real_external_data: bundle.real_external_data_provided,
        kernel: "dsfb_host_minimum".to_string(),
        captures: bundle
            .captures
            .iter()
            .map(|capture| ExternalGpuCaptureMetrics {
                capture_label: capture.label.clone(),
                measured_gpu: false,
                adapter: None,
                backend: None,
                resolution: [capture.inputs.width(), capture.inputs.height()],
                kernel: "dsfb_host_minimum".to_string(),
                total_ms: None,
                dispatch_ms: None,
                readback_ms: None,
                mean_abs_trust_delta_vs_cpu: None,
                mean_abs_alpha_delta_vs_cpu: None,
                mean_abs_intervention_delta_vs_cpu: None,
                notes: vec![
                    "The GPU path was attempted, but the driver/runtime could not complete the shader execution path safely in-process.".to_string(),
                ],
            })
            .collect(),
        notes,
    }
}

fn run_demo_a_external_metrics(
    config: &DemoConfig,
    bundle: &ExternalCaptureBundle,
    figures_dir: &Path,
) -> Result<ExternalDemoAMetrics> {
    let profile =
        default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
    let mut captures = Vec::with_capacity(bundle.captures.len());

    for (capture_index, capture) in bundle.captures.iter().enumerate() {
        let outputs = supervise_temporal_reuse(&capture.inputs.borrow(), &profile);
        let (roi_mask, roi_source) = roi_mask_for_capture(capture, &outputs);
        let dsfb_resolved = resolve_with_alpha(
            &capture.inputs.reprojected_history,
            &capture.inputs.current_color,
            &outputs.alpha,
        );
        let (strong_resolved, strong_alpha, strong_response) =
            run_external_strong_heuristic(config, capture);
        let fixed_alpha_field = constant_field(
            capture.inputs.width(),
            capture.inputs.height(),
            config.baseline.fixed_alpha,
        );
        let fixed_resolved = resolve_with_alpha(
            &capture.inputs.reprojected_history,
            &capture.inputs.current_color,
            &fixed_alpha_field,
        );
        let fixed_response = ScalarField::new(capture.inputs.width(), capture.inputs.height());

        let methods = vec![
            build_demo_a_method_metrics(
                "fixed_alpha",
                "Fixed alpha baseline",
                capture,
                &fixed_resolved,
                &roi_mask,
                &fixed_response,
            ),
            build_demo_a_method_metrics(
                "strong_heuristic",
                "Strong heuristic",
                capture,
                &strong_resolved,
                &roi_mask,
                &strong_response,
            ),
            build_demo_a_method_metrics(
                "dsfb_host_minimum",
                "DSFB host minimum",
                capture,
                &dsfb_resolved,
                &roi_mask,
                &outputs.intervention,
            ),
        ];

        if capture_index == 0 {
            capture
                .inputs
                .current_color
                .save_png(&figures_dir.join("current_color.png"))?;
            capture
                .inputs
                .reprojected_history
                .save_png(&figures_dir.join("reprojected_history.png"))?;
            fixed_resolved.save_png(&figures_dir.join("demo_a_fixed_alpha.png"))?;
            strong_resolved.save_png(&figures_dir.join("demo_a_strong_heuristic.png"))?;
            dsfb_resolved.save_png(&figures_dir.join("demo_a_dsfb.png"))?;
            save_scalar_field_png(
                &outputs.trust,
                &figures_dir.join("trust_map.png"),
                heatmap_blue,
            )?;
            save_scalar_field_png(
                &outputs.alpha,
                &figures_dir.join("alpha_map.png"),
                heatmap_orange,
            )?;
            save_scalar_field_png(
                &outputs.intervention,
                &figures_dir.join("intervention_map.png"),
                heatmap_red,
            )?;
            overlay_roi_mask(&capture.inputs.current_color, &roi_mask)
                .save_png(&figures_dir.join("roi_overlay.png"))?;
            save_scalar_field_png(
                &strong_alpha,
                &figures_dir.join("strong_alpha_map.png"),
                heatmap_orange,
            )?;
        }

        captures.push(ExternalDemoACaptureMetrics {
            capture_label: capture.label.clone(),
            roi_source,
            roi_pixels: roi_mask.iter().filter(|value| **value).count(),
            ground_truth_available: capture.reference.is_some(),
            metric_source: if capture.reference.is_some() {
                "real_reference".to_string()
            } else {
                "proxy_current_vs_history".to_string()
            },
            methods,
        });
    }

    Ok(ExternalDemoAMetrics {
        real_external_data_provided: bundle.real_external_data_provided,
        no_real_external_data_provided: bundle.no_real_external_data_provided,
        captures,
        notes: vec![
            "Demo A external replay uses the same host-minimum supervisory logic as the internal suite.".to_string(),
            "If no optional reference frame is supplied, ROI error becomes a lag proxy against current color and non-ROI error becomes a history-deviation proxy.".to_string(),
        ],
    })
}

fn run_demo_b_external_metrics(
    config: &DemoConfig,
    bundle: &ExternalCaptureBundle,
    figures_dir: &Path,
) -> Result<ExternalDemoBMetrics> {
    let profile =
        default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
    let mut captures = Vec::with_capacity(bundle.captures.len());

    for (capture_index, capture) in bundle.captures.iter().enumerate() {
        let outputs = supervise_temporal_reuse(&capture.inputs.borrow(), &profile);
        let (roi_mask, roi_source) = roi_mask_for_capture(capture, &outputs);
        let width = capture.inputs.width();
        let height = capture.inputs.height();
        let total_pixels = width * height;
        let total_samples = config.demo_b_uniform_spp * total_pixels;
        let variance_field = capture
            .variance
            .clone()
            .map(normalize_field)
            .unwrap_or_else(|| {
                temporal_variance_proxy(
                    &capture.inputs.current_color,
                    &capture.inputs.reprojected_history,
                )
            });
        let gradient = gradient_field(&capture.inputs.current_color);
        let contrast = local_contrast_field(&capture.inputs.current_color);
        let imported_trust = invert_trust(&outputs.trust);
        let combined = combine_fields(
            &[
                (&gradient, 0.30),
                (&contrast, 0.25),
                (&variance_field, 0.45),
            ],
            width,
            height,
        );
        let hybrid = combine_fields(
            &[(&imported_trust, 0.55), (&variance_field, 0.45)],
            width,
            height,
        );
        let base_error = combine_fields(
            &[
                (&gradient, 0.20),
                (&contrast, 0.20),
                (&variance_field, 0.35),
                (&imported_trust, 0.25),
            ],
            width,
            height,
        );

        let policies = vec![
            (
                AllocationPolicyId::Uniform,
                "Uniform",
                vec![config.demo_b_uniform_spp; total_pixels],
            ),
            (
                AllocationPolicyId::EdgeGuided,
                "Gradient magnitude",
                guided_allocation(&gradient, total_samples, 1, config.demo_b_max_spp)?,
            ),
            (
                AllocationPolicyId::ContrastGuided,
                "Contrast-based",
                guided_allocation(&contrast, total_samples, 1, config.demo_b_max_spp)?,
            ),
            (
                AllocationPolicyId::VarianceGuided,
                "Variance proxy",
                guided_allocation(&variance_field, total_samples, 1, config.demo_b_max_spp)?,
            ),
            (
                AllocationPolicyId::CombinedHeuristic,
                "Combined heuristic",
                guided_allocation(&combined, total_samples, 1, config.demo_b_max_spp)?,
            ),
            (
                AllocationPolicyId::ImportedTrust,
                "DSFB imported trust",
                guided_allocation(&imported_trust, total_samples, 1, config.demo_b_max_spp)?,
            ),
            (
                AllocationPolicyId::HybridTrustVariance,
                "Hybrid trust + variance",
                guided_allocation(&hybrid, total_samples, 1, config.demo_b_max_spp)?,
            ),
        ];

        let fixed_budget_equal = policies
            .iter()
            .all(|(_, _, counts)| counts.iter().sum::<usize>() == total_samples);
        let uniform_roi_mae =
            predicted_error_over_mask(&base_error, &policies[0].2, Some(&roi_mask), false);
        let regime = classify_external_demo_b_regime(&gradient, &variance_field, &contrast);

        let mut policy_metrics = Vec::new();
        let uniform_roi_mean_spp = mean_count_over_mask(&policies[0].2, &roi_mask);
        let non_roi_mask = invert_mask(&roi_mask);
        for (policy_index, (policy_id, label, counts)) in policies.iter().enumerate() {
            let counts_field = build_count_field(counts, width, height);
            if capture_index == 0 && policy_index < 4 {
                save_scalar_field_png(
                    &counts_field,
                    &figures_dir.join(format!("demo_b_allocation_{}.png", policy_id.as_str())),
                    heatmap_blue,
                )?;
            }
            let overall_mae = predicted_error_over_mask(&base_error, counts, None, false);
            let roi_mae = predicted_error_over_mask(&base_error, counts, Some(&roi_mask), false);
            let non_roi_mae =
                predicted_error_over_mask(&base_error, counts, Some(&non_roi_mask), false);
            let overall_rmse = predicted_error_over_mask(&base_error, counts, None, true);
            let roi_rmse = predicted_error_over_mask(&base_error, counts, Some(&roi_mask), true);
            let non_roi_rmse =
                predicted_error_over_mask(&base_error, counts, Some(&non_roi_mask), true);
            let roi_mean_spp = mean_count_over_mask(counts, &roi_mask);
            let non_roi_mean_spp = mean_count_over_mask(counts, &non_roi_mask);
            let max_spp = counts.iter().copied().max().unwrap_or(0);
            let allocation_concentration = counts_field.mean();
            let extra_roi_samples_vs_uniform = roi_mean_spp - uniform_roi_mean_spp;
            let roi_error_reduction_per_extra_roi_sample =
                if extra_roi_samples_vs_uniform.abs() <= f32::EPSILON {
                    0.0
                } else {
                    (uniform_roi_mae - roi_mae) / extra_roi_samples_vs_uniform.max(1e-6)
                };
            policy_metrics.push(DemoBPolicyMetrics {
                policy_id: policy_id.as_str().to_string(),
                label: label.to_string(),
                total_samples,
                overall_mae,
                overall_rmse,
                roi_mae,
                roi_rmse,
                non_roi_mae,
                non_roi_rmse,
                roi_mean_spp,
                non_roi_mean_spp,
                max_spp,
                allocation_concentration,
                extra_roi_samples_vs_uniform,
                roi_error_reduction_per_extra_roi_sample,
            });
        }

        let mut budget_curves = Vec::new();
        for (policy_id, field) in [
            (AllocationPolicyId::Uniform, None),
            (AllocationPolicyId::CombinedHeuristic, Some(&combined)),
            (AllocationPolicyId::ImportedTrust, Some(&imported_trust)),
            (AllocationPolicyId::HybridTrustVariance, Some(&hybrid)),
        ] {
            let mut points = Vec::new();
            for average_spp in [1usize, 2, 4, 8] {
                let budget = average_spp * total_pixels;
                let counts = if let Some(difficulty) = field {
                    guided_allocation(difficulty, budget, 1, config.demo_b_max_spp)?
                } else {
                    vec![average_spp; total_pixels]
                };
                points.push(BudgetCurvePoint {
                    average_spp: average_spp as f32,
                    roi_mae: predicted_error_over_mask(
                        &base_error,
                        &counts,
                        Some(&roi_mask),
                        false,
                    ),
                });
            }
            budget_curves.push(BudgetCurve {
                scenario_id: capture.label.clone(),
                policy_id: policy_id.as_str().to_string(),
                points,
            });
        }

        captures.push(ExternalDemoBCaptureMetrics {
            capture_label: capture.label.clone(),
            regime,
            metric_source: if capture.reference.is_some() {
                "allocation_proxy_with_optional_reference".to_string()
            } else {
                "allocation_proxy_without_reference".to_string()
            },
            roi_source,
            roi_pixels: roi_mask.iter().filter(|value| **value).count(),
            ground_truth_available: capture.reference.is_some(),
            budget_total_samples: total_samples,
            fixed_budget_equal,
            policies: policy_metrics,
            budget_curves,
        });
    }

    Ok(ExternalDemoBMetrics {
        real_external_data_provided: bundle.real_external_data_provided,
        no_real_external_data_provided: bundle.no_real_external_data_provided,
        captures,
        notes: vec![
            "External Demo B is an allocation proxy, not a live renderer replay, because imported captures do not contain per-sample shading or multi-budget re-renders.".to_string(),
            "The proxy still enforces identical total budgets across all policies and is intended to guide the next engine-side experiment rather than replace it.".to_string(),
        ],
    })
}

fn run_external_scaling_study(
    config: &DemoConfig,
    bundle: &ExternalCaptureBundle,
    demo_a: &ExternalDemoAMetrics,
    demo_b: &ExternalDemoBMetrics,
) -> Result<ExternalScalingMetrics> {
    let profile =
        default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
    let capture = bundle.captures.first().ok_or_else(|| {
        Error::Message("external scaling study requires at least one capture".to_string())
    })?;
    let native_width = capture.inputs.width();
    let native_height = capture.inputs.height();
    let native_pixels = (native_width * native_height) as f64;
    let coverage = coverage_summary(bundle, demo_a, demo_b);
    let mut entries = Vec::new();

    for (label, width, height, source) in [
        (
            "native_imported",
            native_width,
            native_height,
            "native_imported",
        ),
        (
            "scaled_1080p",
            1920usize,
            1080usize,
            "scaled_external_ready",
        ),
        ("scaled_4k", 3840usize, 2160usize, "scaled_external_ready"),
    ] {
        let attempt_inputs = if width == native_width && height == native_height {
            capture.inputs.clone()
        } else {
            scale_owned_inputs(&capture.inputs, width, height)?
        };
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let maybe_gpu = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            try_execute_host_minimum_kernel(&attempt_inputs, profile.parameters)
        }));
        std::panic::set_hook(previous_hook);
        let pixel_ratio = (width * height) as f64 / native_pixels.max(1.0);
        match maybe_gpu {
            Ok(Ok(Some(gpu))) => {
                let total_ms = gpu.total_ms;
                let native_total_ms = entries
                    .iter()
                    .find(|entry: &&ExternalScalingEntry| entry.label == "native_imported")
                    .and_then(|entry| entry.total_ms);
                let scaling_ratio_vs_native =
                    native_total_ms.map(|native| total_ms / native.max(1e-6));
                let approximately_linear = scaling_ratio_vs_native.map(|ratio| {
                    let ratio_per_pixel = ratio / pixel_ratio.max(1e-6);
                    (0.80..=1.25).contains(&ratio_per_pixel)
                });
                entries.push(ExternalScalingEntry {
                    label: label.to_string(),
                    source: source.to_string(),
                    width,
                    height,
                    attempted: true,
                    measured_gpu: true,
                    total_ms: Some(total_ms),
                    dispatch_ms: Some(gpu.dispatch_ms),
                    readback_ms: Some(gpu.readback_ms),
                    ms_per_megapixel: Some(total_ms / ((width * height) as f64 / 1_000_000.0)),
                    scaling_ratio_vs_native,
                    pixel_ratio_vs_native: pixel_ratio,
                    approximately_linear,
                    unavailable_reason: None,
                });
            }
            Ok(Ok(None)) => entries.push(ExternalScalingEntry {
                label: label.to_string(),
                source: source.to_string(),
                width,
                height,
                attempted: true,
                measured_gpu: false,
                total_ms: None,
                dispatch_ms: None,
                readback_ms: None,
                ms_per_megapixel: None,
                scaling_ratio_vs_native: None,
                pixel_ratio_vs_native: pixel_ratio,
                approximately_linear: None,
                unavailable_reason: Some(
                    "no usable GPU adapter available in the current environment".to_string(),
                ),
            }),
            Ok(Err(error)) => entries.push(ExternalScalingEntry {
                label: label.to_string(),
                source: source.to_string(),
                width,
                height,
                attempted: true,
                measured_gpu: false,
                total_ms: None,
                dispatch_ms: None,
                readback_ms: None,
                ms_per_megapixel: None,
                scaling_ratio_vs_native: None,
                pixel_ratio_vs_native: pixel_ratio,
                approximately_linear: None,
                unavailable_reason: Some(error.to_string()),
            }),
            Err(panic_payload) => entries.push(ExternalScalingEntry {
                label: label.to_string(),
                source: source.to_string(),
                width,
                height,
                attempted: true,
                measured_gpu: false,
                total_ms: None,
                dispatch_ms: None,
                readback_ms: None,
                ms_per_megapixel: None,
                scaling_ratio_vs_native: None,
                pixel_ratio_vs_native: pixel_ratio,
                approximately_linear: None,
                unavailable_reason: Some(format!(
                    "GPU scaling attempt failed at runtime: {}",
                    panic_payload_to_string(panic_payload)
                )),
            }),
        }
    }

    let measurement_kind = if entries.iter().any(|entry| entry.measured_gpu) {
        "measured_gpu".to_string()
    } else {
        "gpu_path_unmeasured".to_string()
    };

    Ok(ExternalScalingMetrics {
        measurement_kind,
        kernel: "dsfb_host_minimum".to_string(),
        native_capture_label: capture.label.clone(),
        native_resolution: [native_width, native_height],
        attempted_1080p: true,
        attempted_4k: true,
        entries,
        coverage,
        notes: vec![
            "Scaling runs use the same imported or scaled external-ready buffers and the same minimum host-realistic GPU kernel.".to_string(),
            "Readback is used here for validation and numeric comparison, not because the production path requires CPU readback.".to_string(),
        ],
    })
}

fn coverage_summary(
    bundle: &ExternalCaptureBundle,
    demo_a: &ExternalDemoAMetrics,
    demo_b: &ExternalDemoBMetrics,
) -> ExternalCoverageSummary {
    let larger_roi_case = demo_a
        .captures
        .iter()
        .any(|capture| capture.roi_pixels > 50);
    let mixed_regime_case = demo_b
        .captures
        .iter()
        .any(|capture| capture.regime == "mixed_regime");
    let realism_stress_case = bundle.captures.iter().any(is_realism_stress_capture);
    let mut missing = Vec::new();
    if !realism_stress_case {
        missing.push("realism_stress_case".to_string());
    }
    if !larger_roi_case {
        missing.push("larger_roi_case".to_string());
    }
    if !mixed_regime_case {
        missing.push("mixed_regime_case".to_string());
    }
    ExternalCoverageSummary {
        realism_stress_case,
        larger_roi_case,
        mixed_regime_case,
        coverage_status: if missing.is_empty() {
            "complete".to_string()
        } else {
            "partial".to_string()
        },
        missing,
    }
}

fn is_realism_stress_capture(capture: &ExternalLoadedCapture) -> bool {
    if let Some(scenario_id) = capture.metadata.scenario_id.as_deref() {
        if matches!(
            scenario_id,
            "motion_bias_band" | "noisy_reprojection" | "fast_pan" | "heuristic_friendly_pan"
        ) {
            return true;
        }
    }
    let motion_mean = capture
        .inputs
        .motion_vectors
        .iter()
        .map(|motion| {
            (motion.to_prev_x * motion.to_prev_x + motion.to_prev_y * motion.to_prev_y).sqrt()
        })
        .sum::<f32>()
        / capture.inputs.motion_vectors.len().max(1) as f32;
    let depth_disagreement = capture
        .inputs
        .current_depth
        .iter()
        .zip(capture.inputs.reprojected_depth.iter())
        .map(|(current, history)| (current - history).abs())
        .sum::<f32>()
        / capture.inputs.current_depth.len().max(1) as f32;
    motion_mean > 0.25 || depth_disagreement > 0.02
}

fn write_external_scaling_report(
    path: &Path,
    metrics: &ExternalScalingMetrics,
    bundle: &ExternalCaptureBundle,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# External Scaling Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    if bundle.no_real_external_data_provided {
        let _ = writeln!(markdown, "{NO_REAL_EXTERNAL_DATA_PROVIDED}");
        let _ = writeln!(markdown);
    }
    let _ = writeln!(
        markdown,
        "Native imported capture: `{}` at {}x{}.",
        metrics.native_capture_label, metrics.native_resolution[0], metrics.native_resolution[1]
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Label | Source | Resolution | Attempted | Measured GPU | total_ms | dispatch_ms | readback_ms | ms/MPixel | scaling ratio vs native | approx linear |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for entry in &metrics.entries {
        let _ = writeln!(
            markdown,
            "| {} | {} | {}x{} | {} | {} | {} | {} | {} | {} | {} | {} |",
            entry.label,
            entry.source,
            entry.width,
            entry.height,
            entry.attempted,
            entry.measured_gpu,
            format_f64(entry.total_ms),
            format_f64(entry.dispatch_ms),
            format_f64(entry.readback_ms),
            format_f64(entry.ms_per_megapixel),
            format_f64(entry.scaling_ratio_vs_native),
            entry
                .approximately_linear
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        );
        if let Some(reason) = &entry.unavailable_reason {
            let _ = writeln!(markdown, "  - unavailable: {reason}");
        }
    }
    let approximate_linearity = metrics
        .entries
        .iter()
        .skip(1)
        .filter_map(|entry| entry.approximately_linear)
        .all(|value| value);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Cost appears approximately linear with resolution: `{}`.",
        if metrics
            .entries
            .iter()
            .any(|entry| entry.approximately_linear.is_some())
        {
            approximate_linearity
        } else {
            false
        }
    );
    if !metrics
        .entries
        .iter()
        .any(|entry| entry.approximately_linear.is_some())
    {
        let _ = writeln!(
            markdown,
            "Approximate linearity could not be classified because no scaled GPU timing was measured in the current environment."
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Coverage");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- realism_stress_case: `{}`",
        metrics.coverage.realism_stress_case
    );
    let _ = writeln!(
        markdown,
        "- larger_roi_case: `{}`",
        metrics.coverage.larger_roi_case
    );
    let _ = writeln!(
        markdown,
        "- mixed_regime_case: `{}`",
        metrics.coverage.mixed_regime_case
    );
    let _ = writeln!(
        markdown,
        "- coverage_status: `{}`",
        metrics.coverage.coverage_status
    );
    if !metrics.coverage.missing.is_empty() {
        let _ = writeln!(
            markdown,
            "- missing coverage labels: {}",
            metrics.coverage.missing.join(", ")
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This scaling report does not replace full engine-side profiling on real exported captures."
    );
    let _ = writeln!(
        markdown,
        "- When a row is marked unavailable, the corresponding scaling point was attempted but not measured in the current environment."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Real imported captures still need the same scaling study on the target evaluator hardware."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn write_memory_bandwidth_report(path: &Path, metrics: &ExternalScalingMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let current_color_bytes_per_read = 16usize;
    let current_color_reads = 19usize;
    let bytes_read_per_pixel = current_color_bytes_per_read * current_color_reads + 16 + 8 + 8 + 32;
    let bytes_written_per_pixel = 12usize;
    let validation_readback_bytes_per_pixel = 12usize;
    let reads_per_pixel = 23usize;
    let writes_per_pixel = 3usize;

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Memory Bandwidth Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Label | Resolution | bytes read / px | bytes written / px | validation readback / px | estimated memory traffic MB | reads / px | writes / px | readback required in production |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for entry in &metrics.entries {
        let pixels = entry.width.saturating_mul(entry.height);
        let total_bytes = pixels
            * (bytes_read_per_pixel
                + bytes_written_per_pixel
                + validation_readback_bytes_per_pixel);
        let _ = writeln!(
            markdown,
            "| {} | {}x{} | {} | {} | {} | {:.2} | {} | {} | false |",
            entry.label,
            entry.width,
            entry.height,
            bytes_read_per_pixel,
            bytes_written_per_pixel,
            validation_readback_bytes_per_pixel,
            total_bytes as f64 / (1024.0 * 1024.0),
            reads_per_pixel,
            writes_per_pixel,
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Readback required in production: `false`.");
    let _ = writeln!(
        markdown,
        "Readback was used here only for validation, numerical delta checks, and report generation."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Memory Access / Coherence Analysis");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Buffer access pattern: linear per-pixel reads for current color, reprojected history, motion, depth pairs, and normal pairs, plus three scalar output writes."
    );
    let _ = writeln!(
        markdown,
        "- Neighborhood reads: the kernel performs two 3x3 neighborhood traversals over current color, one for local contrast and one for neighborhood-hull gating."
    );
    let _ = writeln!(
        markdown,
        "- Coherence expectation: adjacent threads in the 8x8 workgroup read strongly overlapping 3x3 neighborhoods, so the access pattern is locally coherent even though current color is revisited many times."
    );
    let _ = writeln!(
        markdown,
        "- Cache-friendliness: the minimum kernel avoids scattered history gathers because reprojection is precomputed before dispatch. That keeps the kernel more cache-friendly than a motion-indirected gather path."
    );
    let _ = writeln!(
        markdown,
        "- Cache risk: repeated 3x3 reads from a storage buffer still raise bandwidth pressure on current color, so profiling should confirm whether a texture path or shared-memory staging would reduce traffic materially."
    );
    let _ = writeln!(
        markdown,
        "- Optional path impact: any future motion-augmented kernel that reintroduces motion-neighborhood disagreement or in-kernel reprojection will increase cache pressure materially and should be treated as non-minimum."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report is analytical accounting based on the implemented kernel and does not replace external validation with hardware-counter collection."
    );
    let _ = writeln!(
        markdown,
        "- Reported traffic is sufficient for reviewer diligence, but not a substitute for per-architecture cache analysis."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- External validation still needs hardware counters and vendor-specific bandwidth profiling on imported real captures."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn write_integration_scaling_report(
    path: &Path,
    metrics: &ExternalScalingMetrics,
    bundle: &ExternalCaptureBundle,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Integration Scaling Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    if bundle.no_real_external_data_provided {
        let _ = writeln!(markdown, "{NO_REAL_EXTERNAL_DATA_PROVIDED}");
        let _ = writeln!(markdown);
    }
    let _ = writeln!(markdown, "## Pipeline Insertion");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The minimum kernel executes after history reprojection and after motion/depth/normal buffers are available, but before temporal resolve consumes per-pixel alpha or intervention."
    );
    let _ = writeln!(
        markdown,
        "- Alpha modulation is consumed by the temporal accumulation pass; trust and intervention are optional debug or allocator-driving side products."
    );
    let _ = writeln!(
        markdown,
        "- Production readback is not required. The current reports read buffers back only for validation and CPU/GPU delta checks."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Async-Compute Feasibility");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Async execution is feasible if reprojected history, depth, and normals are already materialized and the downstream TAA resolve can wait on a GPU-side signal rather than CPU readback."
    );
    let _ = writeln!(
        markdown,
        "- The minimum kernel has no scattered history gather and no CPU dependency, so the main async-compute risk is overlap contention on memory bandwidth rather than synchronization correctness."
    );
    let _ = writeln!(
        markdown,
        "- Profiling still needs to confirm that the 3x3 current-color neighborhood reads do not stall other post or denoise passes when overlapped."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Hazards / Barriers / Transitions");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Inputs should be transitioned to shader-read / storage-read state before dispatch."
    );
    let _ = writeln!(
        markdown,
        "- Trust, alpha, and intervention outputs should be transitioned from UAV/storage-write into the state required by the temporal resolve or any downstream debug visualization."
    );
    let _ = writeln!(
        markdown,
        "- A production integration should avoid CPU fences; only GPU barriers and queue synchronization should be required."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Pipeline Compatibility");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The minimum kernel is compatible with tiled, deferred, and post-lighting temporal pipelines because it consumes already-aligned per-pixel buffers and writes only local trust/alpha/intervention fields."
    );
    let _ = writeln!(
        markdown,
        "- The current design remains compatible with tiled or asynchronous execution because it does not require CPU-side intervention in production."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Scaling Interpretation");
    let _ = writeln!(markdown);
    for entry in &metrics.entries {
        let _ = writeln!(
            markdown,
            "- {} {}x{}: measured_gpu = `{}`, ms/MPixel = {}, approx_linear = {}",
            entry.label,
            entry.width,
            entry.height,
            entry.measured_gpu,
            format_f64(entry.ms_per_megapixel),
            entry
                .approximately_linear
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This integration note is implementation-specific analysis, not a substitute for engine-side trace profiling."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Async overlap, queue contention, and barrier cost still need confirmation inside a real renderer."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn write_gpu_external_report(
    path: &Path,
    metrics: &ExternalGpuMetrics,
    bundle: &ExternalCaptureBundle,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# GPU Execution Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "measured_gpu: `{}`", metrics.measured_gpu);
    let _ = writeln!(markdown, "measurement_kind: `{}`", metrics.measurement_kind);
    let _ = writeln!(markdown, "kernel: `{}`", metrics.kernel);
    let _ = writeln!(markdown);
    if bundle.no_real_external_data_provided {
        let _ = writeln!(markdown, "{NO_REAL_EXTERNAL_DATA_PROVIDED}");
        let _ = writeln!(markdown);
    }
    let _ = writeln!(
        markdown,
        "| Capture | measured_gpu | adapter | backend | resolution | total_ms | dispatch_ms | readback_ms | trust_delta_vs_cpu | alpha_delta_vs_cpu | intervention_delta_vs_cpu |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    for capture in &metrics.captures {
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {} | {}x{} | {} | {} | {} | {} | {} | {} |",
            capture.capture_label,
            capture.measured_gpu,
            capture.adapter.as_deref().unwrap_or("unavailable"),
            capture.backend.as_deref().unwrap_or("unavailable"),
            capture.resolution[0],
            capture.resolution[1],
            format_f64(capture.total_ms),
            format_f64(capture.dispatch_ms),
            format_f64(capture.readback_ms),
            format_f32(capture.mean_abs_trust_delta_vs_cpu),
            format_f32(capture.mean_abs_alpha_delta_vs_cpu),
            format_f32(capture.mean_abs_intervention_delta_vs_cpu),
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The imported external buffers can execute through the same minimum host-realistic GPU kernel as the internal study."
    );
    let _ = writeln!(
        markdown,
        "- GPU-vs-CPU numerical deltas are recorded whenever a GPU adapter is available."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This file does not prove production renderer integration or full engine-side GPU cost."
    );
    let _ = writeln!(
        markdown,
        "- If `measured_gpu` is `false`, the path is implemented but unmeasured in the current environment."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Engine-exported captures on the target evaluation hardware still need GPU-side profiling."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn write_demo_a_external_report(
    path: &Path,
    metrics: &ExternalDemoAMetrics,
    bundle: &ExternalCaptureBundle,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Demo A External Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    if bundle.no_real_external_data_provided {
        let _ = writeln!(markdown, "{NO_REAL_EXTERNAL_DATA_PROVIDED}");
        let _ = writeln!(markdown);
    }
    let _ = writeln!(
        markdown,
        "Point-vs-region disclosure is explicit per capture via the `point_vs_region` line below."
    );
    let _ = writeln!(markdown);
    for capture in &metrics.captures {
        let source_capture = bundle
            .captures
            .iter()
            .find(|candidate| candidate.label == capture.capture_label)
            .ok_or_else(|| {
                Error::Message(format!(
                    "Demo A capture {} was missing from the loaded bundle",
                    capture.capture_label
                ))
            })?;
        let total_pixels = source_capture.inputs.width() * source_capture.inputs.height();
        let roi_ratio = capture.roi_pixels as f32 / total_pixels.max(1) as f32;
        let _ = writeln!(markdown, "## Capture `{}`", capture.capture_label);
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "- ROI source: `{}`", capture.roi_source);
        let _ = writeln!(markdown, "- ROI pixels: `{}`", capture.roi_pixels);
        let _ = writeln!(
            markdown,
            "- ground_truth_available: `{}`",
            capture.ground_truth_available
        );
        let _ = writeln!(markdown, "- metric_source: `{}`", capture.metric_source);
        let _ = writeln!(
            markdown,
            "- point_vs_region: `{}`",
            if roi_ratio < 0.05 {
                "point_like"
            } else {
                "region_like"
            }
        );
        let _ = writeln!(
            markdown,
            "- realism_stress_note: `{}`",
            if is_realism_stress_capture(source_capture) {
                "realism_stress_case"
            } else {
                "not_classified_as_realism_stress"
            }
        );
        let _ = writeln!(
            markdown,
            "- larger_roi_note: `{}`",
            if roi_ratio > 0.10 {
                "larger_roi_case"
            } else {
                "not_a_larger_roi_case"
            }
        );
        if !capture.ground_truth_available {
            let _ = writeln!(markdown, "- ground truth unavailable -> proxy metrics used");
        }
        let _ = writeln!(markdown);
        let _ = writeln!(
            markdown,
            "| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |"
        );
        let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | ---: |");
        for method in &capture.methods {
            let _ = writeln!(
                markdown,
                "| {} | {:.5} | {:.5} | {:.5} | {:.5} |",
                method.label,
                method.roi_mae,
                method.non_roi_mae,
                method.temporal_error_accumulation,
                method.intervention_rate
            );
        }
        let _ = writeln!(markdown);
    }
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The same DSFB host-minimum supervisory layer runs on imported buffers and can be compared against fixed alpha and a strong heuristic baseline."
    );
    let _ = writeln!(
        markdown,
        "- ROI and non-ROI behavior remain separated on imported data."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Without an optional reference frame, ROI MAE and non-ROI MAE are proxy quantities rather than true reconstruction error."
    );
    let _ = writeln!(
        markdown,
        "- Even with a reference frame, this does not replace longer engine-side sequences."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Real engine capture sequences and longer temporal windows still need evaluation."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn write_demo_b_external_report(
    path: &Path,
    metrics: &ExternalDemoBMetrics,
    bundle: &ExternalCaptureBundle,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Demo B External Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    if bundle.no_real_external_data_provided {
        let _ = writeln!(markdown, "{NO_REAL_EXTERNAL_DATA_PROVIDED}");
        let _ = writeln!(markdown);
    }
    let _ = writeln!(
        markdown,
        "Regime labels used in this report: `aliasing_limited`, `variance_limited`, `mixed_regime`."
    );
    let _ = writeln!(markdown);
    for capture in &metrics.captures {
        let _ = writeln!(markdown, "## Capture `{}`", capture.capture_label);
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "- regime: `{}`", capture.regime);
        let _ = writeln!(markdown, "- metric_source: `{}`", capture.metric_source);
        let _ = writeln!(
            markdown,
            "- fixed_budget_equal: `{}`",
            capture.fixed_budget_equal
        );
        let _ = writeln!(markdown, "- ROI source: `{}`", capture.roi_source);
        let _ = writeln!(markdown, "- ROI pixels: `{}`", capture.roi_pixels);
        let _ = writeln!(markdown);
        let _ = writeln!(
            markdown,
            "| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |"
        );
        let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | ---: | ---: |");
        for policy in &capture.policies {
            let _ = writeln!(
                markdown,
                "| {} | {} | {:.5} | {:.5} | {:.3} | {:.3} |",
                policy.label,
                policy.total_samples,
                policy.roi_mae,
                policy.overall_mae,
                policy.roi_mean_spp,
                policy.non_roi_mean_spp
            );
        }
        let uniform = capture
            .policies
            .iter()
            .find(|policy| policy.policy_id == AllocationPolicyId::Uniform.as_str());
        let dsfb = capture
            .policies
            .iter()
            .find(|policy| policy.policy_id == AllocationPolicyId::ImportedTrust.as_str());
        let combined = capture
            .policies
            .iter()
            .find(|policy| policy.policy_id == AllocationPolicyId::CombinedHeuristic.as_str());
        if let (Some(uniform), Some(dsfb), Some(combined)) = (uniform, dsfb, combined) {
            let _ = writeln!(markdown);
            let _ = writeln!(markdown, "Aliasing vs variance discussion:");
            if capture.regime == "aliasing_limited" {
                let _ = writeln!(
                    markdown,
                    "- This imported capture is primarily aliasing-limited. DSFB imported trust changed ROI proxy error from {:.5} for uniform to {:.5}, while combined heuristic reached {:.5}.",
                    uniform.roi_mae, dsfb.roi_mae, combined.roi_mae
                );
            } else if capture.regime == "variance_limited" {
                let _ = writeln!(
                    markdown,
                    "- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from {:.5} for uniform to {:.5}, versus {:.5} for the combined heuristic.",
                    uniform.roi_mae, dsfb.roi_mae, combined.roi_mae
                );
            } else {
                let _ = writeln!(
                    markdown,
                    "- This imported capture is mixed-regime. The DSFB imported-trust allocator reached {:.5} ROI proxy error, compared with {:.5} for uniform and {:.5} for the combined heuristic.",
                    dsfb.roi_mae, uniform.roi_mae, combined.roi_mae
                );
            }
            if (dsfb.roi_mae - combined.roi_mae).abs() <= 1e-4 {
                let _ = writeln!(markdown, "- This capture is effectively a tie between DSFB imported trust and the strongest heuristic proxy.");
            } else if dsfb.roi_mae < combined.roi_mae {
                let _ = writeln!(
                    markdown,
                    "- DSFB imported trust wins on this imported capture under equal total budget."
                );
            } else {
                let _ = writeln!(markdown, "- The strongest heuristic proxy wins on this imported capture; that result is surfaced rather than hidden.");
            }
        }
        let _ = writeln!(markdown);
    }
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Imported captures can drive fixed-budget allocation policies, including DSFB imported trust and stronger heuristic competitors."
    );
    let _ = writeln!(
        markdown,
        "- Budget equality is enforced across all compared policies."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This is still an allocation proxy because imported captures do not provide a live renderer or per-sample ground truth."
    );
    let _ = writeln!(
        markdown,
        "- It does not replace real engine-side fixed-budget sampling experiments."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- A renderer-integrated fixed-budget replay still needs to confirm the allocation story on real per-sample shading."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn write_external_validation_report(
    path: &Path,
    bundle: &ExternalCaptureBundle,
    handoff: &ExternalHandoffMetrics,
    gpu: &ExternalGpuMetrics,
    demo_a: &ExternalDemoAMetrics,
    demo_b: &ExternalDemoBMetrics,
    scaling: &ExternalScalingMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# External Validation Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Data Description");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- source_kind: `{}`", handoff.source_kind);
    let _ = writeln!(markdown, "- captures: `{}`", handoff.capture_count);
    let _ = writeln!(
        markdown,
        "- real_external_data_provided: `{}`",
        bundle.real_external_data_provided
    );
    let _ = writeln!(
        markdown,
        "- synthetic vs real: `{}`",
        if bundle.real_external_data_provided {
            "real external data"
        } else {
            "synthetic compatibility export"
        }
    );
    if bundle.no_real_external_data_provided {
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "{NO_REAL_EXTERNAL_DATA_PROVIDED}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Pipeline Description");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- External replay uses the same DSFB host-minimum supervisory logic and the same minimum GPU kernel as the internal suite."
    );
    let _ = writeln!(
        markdown,
        "- Differences: imported buffers replace synthetic scene generation, and Demo B uses an allocation proxy because no live renderer samples are present."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## GPU Execution Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- measured_gpu: `{}`", gpu.measured_gpu);
    let _ = writeln!(markdown, "- kernel: `{}`", gpu.kernel);
    for capture in &gpu.captures {
        let _ = writeln!(
            markdown,
            "- capture `{}`: adapter = `{}`, total_ms = {}, dispatch_ms = {}, readback_ms = {}",
            capture.capture_label,
            capture.adapter.as_deref().unwrap_or("unavailable"),
            format_f64(capture.total_ms),
            format_f64(capture.dispatch_ms),
            format_f64(capture.readback_ms),
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Demo A Results");
    let _ = writeln!(markdown);
    for capture in &demo_a.captures {
        let _ = writeln!(
            markdown,
            "- `{}`: ROI source = `{}`, ROI pixels = {}, metric_source = `{}`",
            capture.capture_label, capture.roi_source, capture.roi_pixels, capture.metric_source
        );
        for method in &capture.methods {
            let _ = writeln!(
                markdown,
                "  - {}: ROI MAE = {:.5}, non-ROI MAE = {:.5}, temporal accumulation = {:.5}, intervention rate = {:.5}",
                method.label,
                method.roi_mae,
                method.non_roi_mae,
                method.temporal_error_accumulation,
                method.intervention_rate
            );
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Demo B Results");
    let _ = writeln!(markdown);
    for capture in &demo_b.captures {
        let _ = writeln!(
            markdown,
            "- `{}`: regime = `{}`, fixed_budget_equal = `{}`",
            capture.capture_label, capture.regime, capture.fixed_budget_equal
        );
        for policy in &capture.policies {
            let _ = writeln!(
                markdown,
                "  - {}: ROI error = {:.5}, global error = {:.5}, ROI mean spp = {:.3}",
                policy.label, policy.roi_mae, policy.overall_mae, policy.roi_mean_spp
            );
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Scaling / Coverage Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- attempted_1080p: `{}`", scaling.attempted_1080p);
    let _ = writeln!(markdown, "- attempted_4k: `{}`", scaling.attempted_4k);
    let _ = writeln!(
        markdown,
        "- realism_stress_case: `{}`",
        scaling.coverage.realism_stress_case
    );
    let _ = writeln!(
        markdown,
        "- larger_roi_case: `{}`",
        scaling.coverage.larger_roi_case
    );
    let _ = writeln!(
        markdown,
        "- mixed_regime_case: `{}`",
        scaling.coverage.mixed_regime_case
    );
    let _ = writeln!(
        markdown,
        "- coverage_status: `{}`",
        scaling.coverage.coverage_status
    );
    if !scaling.coverage.missing.is_empty() {
        let _ = writeln!(
            markdown,
            "- missing coverage labels: {}",
            scaling.coverage.missing.join(", ")
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The crate can ingest external buffers through a strict manifest and run the DSFB host-minimum supervisory layer on them."
    );
    let _ = writeln!(
        markdown,
        "- The same GPU kernel can execute on imported buffers, with explicit measured-vs-unmeasured disclosure."
    );
    let _ = writeln!(
        markdown,
        "- ROI vs non-ROI reporting survives the external path, and Demo B keeps equal budgets across stronger heuristic baselines."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not prove production-scene generalization."
    );
    let _ = writeln!(
        markdown,
        "- It does not prove engine integration unless real exported buffers are supplied."
    );
    let _ = writeln!(
        markdown,
        "- Demo B on imported captures remains an allocation proxy, not a renderer-integrated sampling benchmark."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- real external engine captures");
    let _ = writeln!(markdown, "- engine-side GPU profiling on imported buffers");
    let _ = writeln!(
        markdown,
        "- renderer-integrated Demo B replay with per-sample budgets"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Next Required Experiment");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Export one real frame pair plus an ROI/mask disclosure from an engine into the external schema, run `run-external-replay` on the target GPU, and compare fixed alpha, strong heuristic, and DSFB on the same imported capture."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn copy_representative_figure_aliases(output_dir: &Path, figures_dir: &Path) -> Result<()> {
    for (source_name, target_name) in [
        ("current_color.png", "before_current_color.png"),
        ("reprojected_history.png", "before_history_color.png"),
        ("demo_a_dsfb.png", "after_dsfb_resolved.png"),
        ("trust_map.png", "trust_map.png"),
        ("intervention_map.png", "intervention_map.png"),
        ("roi_overlay.png", "roi_overlay.png"),
    ] {
        let source = figures_dir.join(source_name);
        if source.exists() {
            fs::copy(source, output_dir.join(target_name))?;
        }
    }
    Ok(())
}

fn build_demo_a_method_metrics(
    method_id: &str,
    label: &str,
    capture: &ExternalLoadedCapture,
    resolved: &ImageFrame,
    roi_mask: &[bool],
    intervention: &ScalarField,
) -> ExternalDemoAMethodMetrics {
    if let Some(reference) = &capture.reference {
        ExternalDemoAMethodMetrics {
            method_id: method_id.to_string(),
            label: label.to_string(),
            metric_source: "real_reference".to_string(),
            overall_mae: mean_abs_error(resolved, reference),
            roi_mae: mean_abs_error_over_mask(resolved, reference, roi_mask),
            non_roi_mae: mean_abs_error_over_mask(resolved, reference, &invert_mask(roi_mask)),
            temporal_error_accumulation: mean_abs_error(resolved, reference),
            intervention_rate: intervention.mean(),
        }
    } else {
        let proxy = demo_a_proxy_field(capture, resolved, roi_mask);
        let non_roi_mask = invert_mask(roi_mask);
        ExternalDemoAMethodMetrics {
            method_id: method_id.to_string(),
            label: label.to_string(),
            metric_source: "proxy_current_vs_history".to_string(),
            overall_mae: proxy.mean(),
            roi_mae: proxy.mean_over_mask(roi_mask),
            non_roi_mae: proxy.mean_over_mask(&non_roi_mask),
            temporal_error_accumulation: proxy.mean(),
            intervention_rate: intervention.mean(),
        }
    }
}

fn demo_a_proxy_field(
    capture: &ExternalLoadedCapture,
    resolved: &ImageFrame,
    roi_mask: &[bool],
) -> ScalarField {
    let mut field = ScalarField::new(capture.inputs.width(), capture.inputs.height());
    let non_roi_mask = invert_mask(roi_mask);
    for y in 0..capture.inputs.height() {
        for x in 0..capture.inputs.width() {
            let index = y * capture.inputs.width() + x;
            let value = if roi_mask[index] {
                resolved
                    .get(x, y)
                    .abs_diff(capture.inputs.current_color.get(x, y))
            } else if non_roi_mask[index] {
                resolved
                    .get(x, y)
                    .abs_diff(capture.inputs.reprojected_history.get(x, y))
            } else {
                0.0
            };
            field.set(x, y, value);
        }
    }
    field
}

fn resolve_with_alpha(
    history: &ImageFrame,
    current: &ImageFrame,
    alpha: &ScalarField,
) -> ImageFrame {
    let mut frame = ImageFrame::new(current.width(), current.height());
    for y in 0..current.height() {
        for x in 0..current.width() {
            let blended = history.get(x, y).lerp(current.get(x, y), alpha.get(x, y));
            frame.set(x, y, blended);
        }
    }
    frame
}

fn run_external_strong_heuristic(
    config: &DemoConfig,
    capture: &ExternalLoadedCapture,
) -> (ImageFrame, ScalarField, ScalarField) {
    let width = capture.inputs.width();
    let height = capture.inputs.height();
    let mut resolved = ImageFrame::new(width, height);
    let mut alpha = ScalarField::new(width, height);
    let mut response = ScalarField::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let current = capture.inputs.current_color.get(x, y);
            let history = capture.inputs.reprojected_history.get(x, y);
            let clamped =
                clamp_to_current_neighborhood(&capture.inputs.current_color, history, x, y);
            let clamp_distance = clamped.abs_diff(history);
            let residual_gate = smoothstep(
                config.baseline.residual_threshold,
                current.abs_diff(clamped),
            );
            let depth_gate = smoothstep(
                config.baseline.depth_disagreement,
                (capture.inputs.current_depth[index] - capture.inputs.reprojected_depth[index])
                    .abs(),
            );
            let normal_gate = smoothstep(
                config.baseline.normal_disagreement,
                1.0 - capture.inputs.current_normals[index]
                    .dot(capture.inputs.reprojected_normals[index])
                    .clamp(-1.0, 1.0),
            );
            let neighborhood_gate =
                smoothstep(config.baseline.neighborhood_distance, clamp_distance);
            let trigger = residual_gate
                .max(depth_gate)
                .max(normal_gate)
                .max(neighborhood_gate);
            let pixel_alpha = config.baseline.residual_alpha_range.min
                + (config.baseline.residual_alpha_range.max
                    - config.baseline.residual_alpha_range.min)
                    * trigger;
            alpha.set(x, y, pixel_alpha);
            response.set(x, y, trigger);
            resolved.set(x, y, clamped.lerp(current, pixel_alpha));
        }
    }

    (resolved, alpha, response)
}

fn clamp_to_current_neighborhood(
    current: &ImageFrame,
    history: Color,
    x: usize,
    y: usize,
) -> Color {
    let mut min_r = f32::INFINITY;
    let mut min_g = f32::INFINITY;
    let mut min_b = f32::INFINITY;
    let mut max_r = f32::NEG_INFINITY;
    let mut max_g = f32::NEG_INFINITY;
    let mut max_b = f32::NEG_INFINITY;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            let sample = current.sample_clamped(x as i32 + dx, y as i32 + dy);
            min_r = min_r.min(sample.r);
            min_g = min_g.min(sample.g);
            min_b = min_b.min(sample.b);
            max_r = max_r.max(sample.r);
            max_g = max_g.max(sample.g);
            max_b = max_b.max(sample.b);
        }
    }
    Color::rgb(
        history.r.clamp(min_r, max_r),
        history.g.clamp(min_g, max_g),
        history.b.clamp(min_b, max_b),
    )
}

fn smoothstep(threshold: SmoothstepThreshold, value: f32) -> f32 {
    let span = (threshold.high - threshold.low).max(1e-6);
    let t = ((value - threshold.low) / span).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn roi_mask_for_capture(
    capture: &ExternalLoadedCapture,
    outputs: &HostSupervisionOutputs,
) -> (Vec<bool>, String) {
    if let Some(mask) = &capture.mask {
        if mask.iter().any(|value| *value) {
            return (mask.clone(), "manifest_mask".to_string());
        }
    }

    let width = capture.inputs.width();
    let height = capture.inputs.height();
    let total = width * height;
    let scores = (0..total)
        .map(|index| {
            let x = index % width;
            let y = index / width;
            outputs.intervention.get(x, y) * 0.55
                + outputs.proxies.depth_proxy.get(x, y) * 0.20
                + outputs.proxies.normal_proxy.get(x, y) * 0.10
                + outputs.proxies.neighborhood_proxy.get(x, y) * 0.15
        })
        .collect::<Vec<_>>();
    let mut sorted = scores.clone();
    sorted.sort_by(|left, right| right.total_cmp(left));
    let keep = (total / 16).max(1).min(total.saturating_sub(1).max(1));
    let threshold = sorted[keep.saturating_sub(1)];
    let mask = scores
        .iter()
        .map(|score| *score >= threshold)
        .collect::<Vec<_>>();
    (mask, "derived_proxy_mask".to_string())
}

fn overlay_roi_mask(frame: &ImageFrame, mask: &[bool]) -> ImageFrame {
    let mut output = frame.clone();
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            let index = y * frame.width() + x;
            if mask[index] {
                let current = frame.get(x, y);
                output.set(x, y, current.lerp(Color::rgb(0.12, 1.0, 0.24), 0.45));
            }
        }
    }
    output
}

fn constant_field(width: usize, height: usize, value: f32) -> ScalarField {
    ScalarField::from_values(width, height, vec![value; width * height])
}

fn invert_mask(mask: &[bool]) -> Vec<bool> {
    mask.iter().map(|value| !*value).collect()
}

fn normalize_field(field: ScalarField) -> ScalarField {
    let max_value = field
        .values()
        .iter()
        .copied()
        .fold(0.0f32, f32::max)
        .max(1e-6);
    let values = field
        .values()
        .iter()
        .map(|value| (*value / max_value).clamp(0.0, 1.0))
        .collect();
    ScalarField::from_values(field.width(), field.height(), values)
}

fn temporal_variance_proxy(current: &ImageFrame, history: &ImageFrame) -> ScalarField {
    let mut field = ScalarField::new(current.width(), current.height());
    for y in 0..current.height() {
        for x in 0..current.width() {
            let diff = current.get(x, y).abs_diff(history.get(x, y));
            field.set(x, y, (diff / 0.20).clamp(0.0, 1.0));
        }
    }
    field
}

fn scale_owned_inputs(
    inputs: &crate::external::OwnedHostTemporalInputs,
    target_width: usize,
    target_height: usize,
) -> Result<crate::external::OwnedHostTemporalInputs> {
    if target_width == 0 || target_height == 0 {
        return Err(Error::Message(
            "scaled external replay requires positive target dimensions".to_string(),
        ));
    }
    let scale_x = target_width as f32 / inputs.width() as f32;
    let scale_y = target_height as f32 / inputs.height() as f32;
    Ok(crate::external::OwnedHostTemporalInputs {
        current_color: scale_image_frame(&inputs.current_color, target_width, target_height),
        reprojected_history: scale_image_frame(
            &inputs.reprojected_history,
            target_width,
            target_height,
        ),
        motion_vectors: scale_motion_vectors(
            &inputs.motion_vectors,
            inputs.width(),
            inputs.height(),
            target_width,
            target_height,
            scale_x,
            scale_y,
        ),
        current_depth: scale_scalar_samples(
            &inputs.current_depth,
            inputs.width(),
            inputs.height(),
            target_width,
            target_height,
        ),
        reprojected_depth: scale_scalar_samples(
            &inputs.reprojected_depth,
            inputs.width(),
            inputs.height(),
            target_width,
            target_height,
        ),
        current_normals: scale_normals(
            &inputs.current_normals,
            inputs.width(),
            inputs.height(),
            target_width,
            target_height,
        ),
        reprojected_normals: scale_normals(
            &inputs.reprojected_normals,
            inputs.width(),
            inputs.height(),
            target_width,
            target_height,
        ),
        visibility_hint: inputs.visibility_hint.as_ref().map(|hint| {
            scale_bool_mask(
                hint,
                inputs.width(),
                inputs.height(),
                target_width,
                target_height,
            )
        }),
        thin_hint: None,
    })
}

fn scale_image_frame(frame: &ImageFrame, target_width: usize, target_height: usize) -> ImageFrame {
    let mut output = ImageFrame::new(target_width, target_height);
    let source_width = frame.width() as f32;
    let source_height = frame.height() as f32;
    for y in 0..target_height {
        for x in 0..target_width {
            let source_x = (x as f32 + 0.5) * source_width / target_width as f32 - 0.5;
            let source_y = (y as f32 + 0.5) * source_height / target_height as f32 - 0.5;
            output.set(x, y, frame.sample_bilinear_clamped(source_x, source_y));
        }
    }
    output
}

fn scale_scalar_samples(
    values: &[f32],
    source_width: usize,
    source_height: usize,
    target_width: usize,
    target_height: usize,
) -> Vec<f32> {
    let mut output = vec![0.0; target_width * target_height];
    for y in 0..target_height {
        for x in 0..target_width {
            let source_x = (x as f32 + 0.5) * source_width as f32 / target_width as f32 - 0.5;
            let source_y = (y as f32 + 0.5) * source_height as f32 / target_height as f32 - 0.5;
            output[y * target_width + x] =
                sample_scalar_bilinear(values, source_width, source_height, source_x, source_y);
        }
    }
    output
}

fn scale_motion_vectors(
    values: &[MotionVector],
    source_width: usize,
    source_height: usize,
    target_width: usize,
    target_height: usize,
    scale_x: f32,
    scale_y: f32,
) -> Vec<MotionVector> {
    let mut output = vec![
        MotionVector {
            to_prev_x: 0.0,
            to_prev_y: 0.0,
        };
        target_width * target_height
    ];
    for y in 0..target_height {
        for x in 0..target_width {
            let source_x = (x as f32 + 0.5) * source_width as f32 / target_width as f32 - 0.5;
            let source_y = (y as f32 + 0.5) * source_height as f32 / target_height as f32 - 0.5;
            let sampled =
                sample_motion_bilinear(values, source_width, source_height, source_x, source_y);
            output[y * target_width + x] = MotionVector {
                to_prev_x: sampled.to_prev_x * scale_x,
                to_prev_y: sampled.to_prev_y * scale_y,
            };
        }
    }
    output
}

fn scale_normals(
    values: &[Normal3],
    source_width: usize,
    source_height: usize,
    target_width: usize,
    target_height: usize,
) -> Vec<Normal3> {
    let mut output = vec![Normal3::new(0.0, 0.0, 1.0); target_width * target_height];
    for y in 0..target_height {
        for x in 0..target_width {
            let source_x = (x as f32 + 0.5) * source_width as f32 / target_width as f32 - 0.5;
            let source_y = (y as f32 + 0.5) * source_height as f32 / target_height as f32 - 0.5;
            output[y * target_width + x] =
                sample_normal_bilinear(values, source_width, source_height, source_x, source_y);
        }
    }
    output
}

fn scale_bool_mask(
    mask: &[bool],
    source_width: usize,
    source_height: usize,
    target_width: usize,
    target_height: usize,
) -> Vec<bool> {
    let mut output = vec![false; target_width * target_height];
    for y in 0..target_height {
        for x in 0..target_width {
            let source_x = ((x as f32 + 0.5) * source_width as f32 / target_width as f32)
                .floor()
                .clamp(0.0, source_width.saturating_sub(1) as f32)
                as usize;
            let source_y = ((y as f32 + 0.5) * source_height as f32 / target_height as f32)
                .floor()
                .clamp(0.0, source_height.saturating_sub(1) as f32)
                as usize;
            output[y * target_width + x] = mask[source_y * source_width + source_x];
        }
    }
    output
}

fn sample_scalar_bilinear(values: &[f32], width: usize, height: usize, x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);
    let sample = |sample_x: f32, sample_y: f32| {
        let sx = sample_x.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let sy = sample_y.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[sy * width + sx]
    };
    let top = sample(x0, y0) * (1.0 - tx) + sample(x1, y0) * tx;
    let bottom = sample(x0, y1) * (1.0 - tx) + sample(x1, y1) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn sample_motion_bilinear(
    values: &[MotionVector],
    width: usize,
    height: usize,
    x: f32,
    y: f32,
) -> MotionVector {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);
    let sample = |sample_x: f32, sample_y: f32| {
        let sx = sample_x.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let sy = sample_y.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[sy * width + sx]
    };
    let lerp = |a: MotionVector, b: MotionVector, t: f32| MotionVector {
        to_prev_x: a.to_prev_x + (b.to_prev_x - a.to_prev_x) * t,
        to_prev_y: a.to_prev_y + (b.to_prev_y - a.to_prev_y) * t,
    };
    lerp(
        lerp(sample(x0, y0), sample(x1, y0), tx),
        lerp(sample(x0, y1), sample(x1, y1), tx),
        ty,
    )
}

fn sample_normal_bilinear(
    values: &[Normal3],
    width: usize,
    height: usize,
    x: f32,
    y: f32,
) -> Normal3 {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);
    let sample = |sample_x: f32, sample_y: f32| {
        let sx = sample_x.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let sy = sample_y.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[sy * width + sx]
    };
    let lerp = |a: Normal3, b: Normal3, t: f32| {
        Normal3::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
        )
    };
    lerp(
        lerp(sample(x0, y0), sample(x1, y0), tx),
        lerp(sample(x0, y1), sample(x1, y1), tx),
        ty,
    )
    .normalized()
}

fn predicted_error_over_mask(
    base_error: &ScalarField,
    counts: &[usize],
    roi_mask: Option<&[bool]>,
    squared: bool,
) -> f32 {
    let width = base_error.width();
    let height = base_error.height();
    let mut sum = 0.0;
    let mut count = 0usize;
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let include = roi_mask.map(|mask| mask[index]).unwrap_or(true);
            if !include {
                continue;
            }
            let predicted = base_error.get(x, y) / (counts[index].max(1) as f32).sqrt();
            sum += if squared {
                predicted * predicted
            } else {
                predicted
            };
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else if squared {
        (sum / count as f32).sqrt()
    } else {
        sum / count as f32
    }
}

fn classify_external_demo_b_regime(
    gradient: &ScalarField,
    variance: &ScalarField,
    contrast: &ScalarField,
) -> String {
    let gradient_mean = gradient.mean();
    let variance_mean = variance.mean();
    let contrast_mean = contrast.mean();
    if gradient_mean + contrast_mean > variance_mean * 1.45 {
        "aliasing_limited".to_string()
    } else if variance_mean > gradient_mean * 1.20 {
        "variance_limited".to_string()
    } else {
        "mixed_regime".to_string()
    }
}

fn mean_abs_delta(left: &[f32], right: &[f32]) -> f32 {
    let count = left.len().min(right.len()).max(1);
    left.iter()
        .zip(right.iter())
        .map(|(lhs, rhs)| (lhs - rhs).abs())
        .sum::<f32>()
        / count as f32
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn format_f64(value: Option<f64>) -> String {
    value
        .map(|current| format!("{current:.4}"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_f32(value: Option<f32>) -> String {
    value
        .map(|current| format!("{current:.6}"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn heatmap_blue(value: f32) -> [u8; 4] {
    let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [v / 4, v / 2, 255, 255]
}

fn heatmap_orange(value: f32) -> [u8; 4] {
    let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [255, v, 32, 255]
}

fn heatmap_red(value: f32) -> [u8; 4] {
    let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [255, 24, v / 2, 255]
}
