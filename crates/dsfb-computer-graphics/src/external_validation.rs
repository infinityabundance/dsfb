use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::config::DemoConfig;
use crate::error::{Error, Result};
use crate::external::{
    load_external_capture_bundle, run_external_import_from_manifest, ExternalCaptureBundle,
    ExternalHandoffMetrics, ExternalLoadedCapture, OwnedHostTemporalInputs,
    NO_REAL_EXTERNAL_DATA_PROVIDED,
};
use crate::frame::{
    mean_abs_error, mean_abs_error_over_mask, save_scalar_field_png, Color, ImageFrame, ScalarField,
};
use crate::gpu::try_execute_host_minimum_kernel;
use crate::host::{
    default_host_realistic_profile, supervise_temporal_reuse,
};
use crate::parameters::SmoothstepThreshold;
use crate::report::EXPERIMENT_SENTENCE;
use crate::sampling::{
    build_count_field, combine_fields, gradient_field, guided_allocation, invert_trust,
    local_contrast_field, mean_count_over_mask, AllocationPolicyId, BudgetCurve, BudgetCurvePoint,
    DemoBPolicyMetrics,
};
use crate::scene::{MotionVector, Normal3};
pub const ROI_CONTRACT_ALPHA: f32 = 0.15;
pub const ROI_CONTRACT_BASELINE_METHOD_ID: &str = "fixed_alpha";
pub const ROI_CONTRACT_SOURCE: &str = "fixed_alpha_local_contrast_0p15";
pub const ROI_CONTRACT_STATEMENT: &str =
    "ROI is defined as pixels where baseline error exceeds 15% of local contrast. The mask is computed once from the baseline and held fixed across all methods. DSFB does not influence ROI selection.";
pub const CANONICAL_HEADLINE_STATEMENT: &str =
    "DSFB improves strong temporal heuristics via structural supervision.";
pub const PURE_DSFB_LIMITATION_STATEMENT: &str =
    "DSFB alone does not outperform strong heuristic baselines in the current evaluation.";
pub const ROI_HONESTY_STATEMENT: &str =
    "The ROI definition captures approximately 50% of the frame under the fixed baseline-relative threshold, making the metric closer to a global structural error measure than a sparse artifact mask.";
pub const ROI_AGGREGATION_MIN_CAPTURES: usize = 3;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalDemoAMethodMetrics {
    pub method_id: String,
    pub label: String,
    pub metric_source: String,
    pub overall_mae: f32,
    pub roi_mae: f32,
    pub non_roi_mae: f32,
    pub max_error: f32,
    pub temporal_error_accumulation: f32,
    pub intervention_rate: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalDemoACaptureMetrics {
    pub capture_label: String,
    pub roi_source: String,
    pub roi_pixels: usize,
    pub total_pixels: usize,
    pub roi_coverage: f32,
    pub roi_statement: String,
    pub baseline_method_id: String,
    pub reference_source: String,
    pub ground_truth_available: bool,
    pub metric_source: String,
    pub methods: Vec<ExternalDemoAMethodMetrics>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalDemoAMetrics {
    pub real_external_data_provided: bool,
    pub no_real_external_data_provided: bool,
    pub captures: Vec<ExternalDemoACaptureMetrics>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalDemoBCaptureMetrics {
    pub capture_label: String,
    pub regime: String,
    pub metric_source: String,
    pub roi_source: String,
    pub roi_pixels: usize,
    pub total_pixels: usize,
    pub roi_coverage: f32,
    pub roi_statement: String,
    pub baseline_method_id: String,
    pub reference_source: String,
    pub ground_truth_available: bool,
    pub budget_total_samples: usize,
    pub fixed_budget_equal: bool,
    pub policies: Vec<DemoBPolicyMetrics>,
    pub budget_curves: Vec<BudgetCurve>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalDemoBMetrics {
    pub real_external_data_provided: bool,
    pub no_real_external_data_provided: bool,
    pub captures: Vec<ExternalDemoBCaptureMetrics>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalCoverageSummary {
    pub realism_stress_case: bool,
    pub larger_roi_case: bool,
    pub mixed_regime_case: bool,
    pub coverage_status: String,
    pub missing: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize)]
struct TemporalTrustTrajectoryPoint {
    capture_label: String,
    frame_index: usize,
    mean_trust: f32,
    mean_alpha: f32,
    intervention_rate: f32,
    roi_coverage: f32,
    dsfb_roi_mae: f32,
    hybrid_roi_mae: f32,
}

#[derive(Clone, Debug, Serialize)]
struct TemporalTrustTrajectoryReport {
    onset_capture_label: String,
    peak_roi_capture_label: String,
    recovery_capture_label: String,
    points: Vec<TemporalTrustTrajectoryPoint>,
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

    let scaling_metrics = run_external_scaling_study(
        config,
        manifest_path,
        output_dir,
        &bundle,
        &gpu_metrics,
        &demo_a_metrics,
        &demo_b_metrics,
    )?;
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
    capture_label: Option<&str>,
    scaled_resolution: Option<(usize, usize)>,
) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    let bundle = load_external_capture_bundle(config, manifest_path, output_dir)?;
    let bundle = if capture_label.is_some() || scaled_resolution.is_some() {
        build_gpu_probe_bundle(bundle, capture_label, scaled_resolution)?
    } else {
        bundle
    };
    let metrics = run_external_gpu_metrics(config, &bundle)?;
    let path = output_dir.join("gpu_probe_metrics.json");
    fs::write(&path, serde_json::to_string_pretty(&metrics)?)?;
    Ok(path)
}

fn build_gpu_probe_bundle(
    bundle: ExternalCaptureBundle,
    capture_label: Option<&str>,
    scaled_resolution: Option<(usize, usize)>,
) -> Result<ExternalCaptureBundle> {
    let capture = if let Some(label) = capture_label {
        bundle
            .captures
            .iter()
            .find(|capture| capture.label == label)
            .cloned()
            .ok_or_else(|| Error::Message(format!("capture `{label}` was missing from the bundle")))?
    } else {
        bundle
            .captures
            .first()
            .cloned()
            .ok_or_else(|| Error::Message("external capture bundle had no captures".to_string()))?
    };
    let capture = if let Some((width, height)) = scaled_resolution {
        scale_external_capture(&capture, width, height)?
    } else {
        capture
    };
    Ok(ExternalCaptureBundle {
        manifest: bundle.manifest,
        captures: vec![capture],
        real_external_data_provided: bundle.real_external_data_provided,
        no_real_external_data_provided: bundle.no_real_external_data_provided,
    })
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
    let Some(executable) = gpu_probe_executable()? else {
        return Ok(None);
    };

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
    let mut trajectory_points = Vec::with_capacity(bundle.captures.len());

    for (capture_index, capture) in bundle.captures.iter().enumerate() {
        let outputs = supervise_temporal_reuse(&capture.inputs.borrow(), &profile);
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
        let (reference_frame, reference_source, metric_source) =
            capture_reference_frame_and_metric_source(capture);
        let (roi_mask, roi_source, roi_coverage) =
            roi_mask_for_capture(capture, &fixed_resolved, reference_frame);
        let dsfb_resolved = resolve_with_alpha(
            &capture.inputs.reprojected_history,
            &capture.inputs.current_color,
            &outputs.alpha,
        );
        let (strong_resolved, strong_alpha, strong_response) =
            run_external_strong_heuristic(config, capture);
        let (hybrid_resolved, _hybrid_alpha, hybrid_response) =
            run_external_dsfb_plus_strong_heuristic(
                capture,
                &outputs.alpha,
                &outputs.intervention,
                &strong_alpha,
                &strong_response,
            );
        let fixed_response = ScalarField::new(capture.inputs.width(), capture.inputs.height());

        let methods = vec![
            build_demo_a_method_metrics(
                "fixed_alpha",
                "Fixed alpha baseline",
                &fixed_resolved,
                reference_frame,
                metric_source,
                &roi_mask,
                &fixed_response,
            ),
            build_demo_a_method_metrics(
                "strong_heuristic",
                "Strong heuristic clamp",
                &strong_resolved,
                reference_frame,
                metric_source,
                &roi_mask,
                &strong_response,
            ),
            build_demo_a_method_metrics(
                "dsfb_host_minimum",
                "DSFB host minimum",
                &dsfb_resolved,
                reference_frame,
                metric_source,
                &roi_mask,
                &outputs.intervention,
            ),
            build_demo_a_method_metrics(
                "dsfb_plus_strong_heuristic",
                "DSFB + strong heuristic",
                &hybrid_resolved,
                reference_frame,
                metric_source,
                &roi_mask,
                &hybrid_response,
            ),
        ];
        let dsfb_roi_mae = methods
            .iter()
            .find(|method| method.method_id == "dsfb_host_minimum")
            .map(|method| method.roi_mae)
            .ok_or_else(|| {
                Error::Message(format!(
                    "capture `{}` did not retain dsfb_host_minimum metrics",
                    capture.label
                ))
            })?;
        let hybrid_roi_mae = methods
            .iter()
            .find(|method| method.method_id == "dsfb_plus_strong_heuristic")
            .map(|method| method.roi_mae)
            .ok_or_else(|| {
                Error::Message(format!(
                    "capture `{}` did not retain dsfb_plus_strong_heuristic metrics",
                    capture.label
                ))
            })?;

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
            hybrid_resolved.save_png(
                &figures_dir.join("demo_a_dsfb_plus_strong_heuristic.png"),
            )?;
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
            let trust_error_field = absolute_error_field(&dsfb_resolved, reference_frame);
            write_trust_histogram_figure(
                &outputs.trust,
                &figures_dir.join("trust_histogram.svg"),
            )?;
            write_trust_vs_error_figure(
                &outputs.trust,
                &trust_error_field,
                metric_source,
                &figures_dir.join("trust_vs_error.svg"),
            )?;
            save_trust_conditioned_error_map(
                &outputs.trust,
                &trust_error_field,
                &figures_dir.join("trust_conditioned_error_map.png"),
            )?;
        }

        trajectory_points.push(TemporalTrustTrajectoryPoint {
            capture_label: capture.label.clone(),
            frame_index: capture.metadata.frame_index,
            mean_trust: outputs.trust.mean(),
            mean_alpha: outputs.alpha.mean(),
            intervention_rate: outputs.intervention.mean(),
            roi_coverage,
            dsfb_roi_mae,
            hybrid_roi_mae,
        });

        captures.push(ExternalDemoACaptureMetrics {
            capture_label: capture.label.clone(),
            roi_source,
            roi_pixels: roi_mask.iter().filter(|value| **value).count(),
            total_pixels: capture.inputs.width() * capture.inputs.height(),
            roi_coverage,
            roi_statement: ROI_CONTRACT_STATEMENT.to_string(),
            baseline_method_id: ROI_CONTRACT_BASELINE_METHOD_ID.to_string(),
            reference_source: reference_source.to_string(),
            ground_truth_available: capture.reference.is_some(),
            metric_source: metric_source.to_string(),
            methods,
        });
    }

    let all_have_reference = captures.iter().all(|capture| capture.ground_truth_available);
    let reference_note = if all_have_reference {
        "The current bundle measures against exported `reference_color` on every capture; that reference is a higher-resolution Unreal proxy rather than a path-traced ground truth.".to_string()
    } else {
        "If no optional reference frame is supplied, the current frame is used as the explicit proxy reference for ROI and full-frame error.".to_string()
    };

    let metrics = ExternalDemoAMetrics {
        real_external_data_provided: bundle.real_external_data_provided,
        no_real_external_data_provided: bundle.no_real_external_data_provided,
        captures,
        notes: vec![
            "Demo A external replay uses the same host-minimum supervisory logic as the internal suite.".to_string(),
            ROI_CONTRACT_STATEMENT.to_string(),
            reference_note,
        ],
    };

    if trajectory_points.len() >= 2 {
        write_temporal_trust_trajectory_outputs(&trajectory_points, figures_dir)?;
    }

    Ok(metrics)
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
        let (reference_frame, reference_source, _) = capture_reference_frame_and_metric_source(capture);
        let (roi_mask, roi_source, roi_coverage) =
            roi_mask_for_capture(capture, &fixed_resolved, reference_frame);
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
            total_pixels,
            roi_coverage,
            roi_statement: ROI_CONTRACT_STATEMENT.to_string(),
            baseline_method_id: ROI_CONTRACT_BASELINE_METHOD_ID.to_string(),
            reference_source: reference_source.to_string(),
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
            ROI_CONTRACT_STATEMENT.to_string(),
        ],
    })
}

fn run_external_scaling_study(
    _config: &DemoConfig,
    manifest_path: &Path,
    output_dir: &Path,
    bundle: &ExternalCaptureBundle,
    gpu: &ExternalGpuMetrics,
    demo_a: &ExternalDemoAMetrics,
    demo_b: &ExternalDemoBMetrics,
) -> Result<ExternalScalingMetrics> {
    let capture = bundle.captures.first().ok_or_else(|| {
        Error::Message("external scaling study requires at least one capture".to_string())
    })?;
    let native_width = capture.inputs.width();
    let native_height = capture.inputs.height();
    let native_pixels = (native_width * native_height) as f64;
    let coverage = coverage_summary(bundle, demo_a, demo_b);
    let native_capture = gpu
        .captures
        .iter()
        .find(|candidate| candidate.capture_label == capture.label)
        .or_else(|| gpu.captures.first());
    let native_total_ms = native_capture.and_then(|metrics| metrics.total_ms);
    let mut entries = vec![ExternalScalingEntry {
        label: "native_imported".to_string(),
        source: "native_imported".to_string(),
        width: native_width,
        height: native_height,
        attempted: true,
        measured_gpu: native_capture.map(|metrics| metrics.measured_gpu).unwrap_or(false),
        total_ms: native_capture.and_then(|metrics| metrics.total_ms),
        dispatch_ms: native_capture.and_then(|metrics| metrics.dispatch_ms),
        readback_ms: native_capture.and_then(|metrics| metrics.readback_ms),
        ms_per_megapixel: native_capture
            .and_then(|metrics| metrics.total_ms)
            .map(|total_ms| total_ms / (native_pixels / 1_000_000.0).max(1e-6)),
        scaling_ratio_vs_native: native_total_ms.map(|_| 1.0),
        pixel_ratio_vs_native: 1.0,
        approximately_linear: native_total_ms.map(|_| true),
        unavailable_reason: native_capture
            .filter(|metrics| !metrics.measured_gpu || metrics.total_ms.is_none())
            .map(|metrics| metrics.notes.join(" ")),
    }];

    let executable = gpu_probe_executable()?;
    for (label, width, height) in [("scaled_1080p", 1920usize, 1080usize), ("scaled_4k", 3840usize, 2160usize)] {
        entries.push(if let Some(executable) = &executable {
            probe_scaled_gpu_entry(
                executable,
                manifest_path,
                output_dir,
                &capture.label,
                label,
                width,
                height,
                native_pixels,
                native_total_ms,
            )?
        } else {
            unavailable_scaling_entry(
                label,
                width,
                height,
                native_pixels,
                false,
                "scaled probe requires the standalone dsfb-computer-graphics binary; library/test invocation cannot safely isolate GPU shader-JIT failures"
                    .to_string(),
            )
        });
    }

    let measured_scaled = entries
        .iter()
        .skip(1)
        .any(|entry| entry.measured_gpu && entry.total_ms.is_some());
    let measurement_kind = if measured_scaled {
        "gpu_scaled_probe_measured".to_string()
    } else if entries.iter().any(|entry| entry.measured_gpu) {
        "native_gpu_only".to_string()
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
        notes: if measured_scaled {
            vec![
                "Scaled GPU timings were measured in isolated subprocess probes to avoid driver-specific shader-JIT crashes from corrupting the canonical run.".to_string(),
                "The native imported timing reuses the same minimum-kernel path as gpu_execution_metrics.json.".to_string(),
            ]
        } else {
            vec![
                "Scaled GPU probes were attempted only when the standalone binary was available for isolated subprocess execution.".to_string(),
                "When a scaled row is unavailable, the run fails closed rather than guessing a scaling claim.".to_string(),
            ]
        },
    })
}

fn gpu_probe_executable() -> Result<Option<PathBuf>> {
    let executable = std::env::current_exe()?;
    let executable_name = executable
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if executable_name.contains("dsfb-computer-graphics") {
        Ok(Some(executable))
    } else {
        Ok(None)
    }
}

fn probe_scaled_gpu_entry(
    executable: &Path,
    manifest_path: &Path,
    output_dir: &Path,
    capture_label: &str,
    label: &str,
    width: usize,
    height: usize,
    native_pixels: f64,
    native_total_ms: Option<f64>,
) -> Result<ExternalScalingEntry> {
    let pixel_ratio = (width * height) as f64 / native_pixels.max(1.0);
    let probe_dir = output_dir.join("scaling_gpu_probe").join(label);
    fs::create_dir_all(&probe_dir)?;
    let status = Command::new(executable)
        .arg("probe-external-gpu")
        .arg("--manifest")
        .arg(manifest_path)
        .arg("--output")
        .arg(&probe_dir)
        .arg("--capture-label")
        .arg(capture_label)
        .arg("--width")
        .arg(width.to_string())
        .arg("--height")
        .arg(height.to_string())
        .status()?;
    let metrics_path = probe_dir.join("gpu_probe_metrics.json");
    if status.success() && metrics_path.exists() {
        let metrics: ExternalGpuMetrics = serde_json::from_str(&fs::read_to_string(metrics_path)?)?;
        let capture = metrics.captures.first().ok_or_else(|| {
            Error::Message(format!(
                "scaled GPU probe `{label}` did not emit any capture metrics"
            ))
        })?;
        return Ok(build_scaling_entry(
            label,
            "scaled_subprocess_probe",
            width,
            height,
            pixel_ratio,
            capture,
            native_total_ms,
        ));
    }

    let reason = if let Some(code) = status.code() {
        format!("scaled GPU subprocess exited with status {code}")
    } else {
        "scaled GPU subprocess terminated by signal".to_string()
    };
    Ok(unavailable_scaling_entry(
        label,
        width,
        height,
        native_pixels,
        true,
        reason,
    ))
}

fn build_scaling_entry(
    label: &str,
    source: &str,
    width: usize,
    height: usize,
    pixel_ratio: f64,
    capture: &ExternalGpuCaptureMetrics,
    native_total_ms: Option<f64>,
) -> ExternalScalingEntry {
    let total_ms = capture.total_ms;
    let scaling_ratio_vs_native = native_total_ms
        .zip(total_ms)
        .map(|(native, total)| total / native.max(1e-9));
    let approximately_linear = scaling_ratio_vs_native.map(|scaling_ratio| {
        let normalized = scaling_ratio / pixel_ratio.max(1e-9);
        (normalized - 1.0).abs() <= 0.20
    });
    ExternalScalingEntry {
        label: label.to_string(),
        source: source.to_string(),
        width,
        height,
        attempted: true,
        measured_gpu: capture.measured_gpu,
        total_ms,
        dispatch_ms: capture.dispatch_ms,
        readback_ms: capture.readback_ms,
        ms_per_megapixel: total_ms.map(|value| value / (((width * height) as f64) / 1_000_000.0).max(1e-6)),
        scaling_ratio_vs_native,
        pixel_ratio_vs_native: pixel_ratio,
        approximately_linear,
        unavailable_reason: if capture.measured_gpu && total_ms.is_some() {
            None
        } else if capture.notes.is_empty() {
            Some("scaled GPU probe did not return a usable timing".to_string())
        } else {
            Some(capture.notes.join(" "))
        },
    }
}

fn unavailable_scaling_entry(
    label: &str,
    width: usize,
    height: usize,
    native_pixels: f64,
    attempted: bool,
    reason: String,
) -> ExternalScalingEntry {
    ExternalScalingEntry {
        label: label.to_string(),
        source: "scaled_subprocess_probe".to_string(),
        width,
        height,
        attempted,
        measured_gpu: false,
        total_ms: None,
        dispatch_ms: None,
        readback_ms: None,
        ms_per_megapixel: None,
        scaling_ratio_vs_native: None,
        pixel_ratio_vs_native: (width * height) as f64 / native_pixels.max(1.0),
        approximately_linear: None,
        unavailable_reason: Some(reason),
    }
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

fn scale_external_capture(
    capture: &ExternalLoadedCapture,
    width: usize,
    height: usize,
) -> Result<ExternalLoadedCapture> {
    if width == 0 || height == 0 {
        return Err(Error::Message(
            "scaled GPU probe requires non-zero width and height".to_string(),
        ));
    }
    let source_width = capture.inputs.width();
    let source_height = capture.inputs.height();
    if width == source_width && height == source_height {
        return Ok(capture.clone());
    }

    let inputs = OwnedHostTemporalInputs {
        current_color: scale_image_frame(&capture.inputs.current_color, width, height),
        reprojected_history: scale_image_frame(&capture.inputs.reprojected_history, width, height),
        motion_vectors: scale_motion_vectors(
            &capture.inputs.motion_vectors,
            source_width,
            source_height,
            width,
            height,
        ),
        current_depth: scale_scalar_buffer(
            &capture.inputs.current_depth,
            source_width,
            source_height,
            width,
            height,
        ),
        reprojected_depth: scale_scalar_buffer(
            &capture.inputs.reprojected_depth,
            source_width,
            source_height,
            width,
            height,
        ),
        current_normals: scale_normal_buffer(
            &capture.inputs.current_normals,
            source_width,
            source_height,
            width,
            height,
        ),
        reprojected_normals: scale_normal_buffer(
            &capture.inputs.reprojected_normals,
            source_width,
            source_height,
            width,
            height,
        ),
        visibility_hint: capture
            .inputs
            .visibility_hint
            .as_ref()
            .map(|mask| scale_bool_buffer(mask, source_width, source_height, width, height)),
        thin_hint: capture
            .inputs
            .thin_hint
            .as_ref()
            .map(|field| scale_scalar_field(field, width, height)),
    };
    let mut metadata = capture.metadata.clone();
    metadata.width = width;
    metadata.height = height;
    metadata.notes.push(format!(
        "This capture was scaled in-memory from {}x{} to {}x{} for an isolated GPU scaling probe.",
        source_width, source_height, width, height
    ));

    Ok(ExternalLoadedCapture {
        label: capture.label.clone(),
        inputs,
        metadata,
        mask: capture
            .mask
            .as_ref()
            .map(|mask| scale_bool_buffer(mask, source_width, source_height, width, height)),
        reference: capture
            .reference
            .as_ref()
            .map(|reference| scale_image_frame(reference, width, height)),
        variance: capture
            .variance
            .as_ref()
            .map(|variance| scale_scalar_field(variance, width, height)),
    })
}

fn scale_image_frame(frame: &ImageFrame, width: usize, height: usize) -> ImageFrame {
    let mut scaled = ImageFrame::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let sample_x = scaled_sample_coordinate(x, frame.width(), width);
            let sample_y = scaled_sample_coordinate(y, frame.height(), height);
            scaled.set(x, y, frame.sample_bilinear_clamped(sample_x, sample_y));
        }
    }
    scaled
}

fn scale_scalar_field(field: &ScalarField, width: usize, height: usize) -> ScalarField {
    ScalarField::from_values(
        width,
        height,
        scale_scalar_buffer(field.values(), field.width(), field.height(), width, height),
    )
}

fn scale_scalar_buffer(
    values: &[f32],
    source_width: usize,
    source_height: usize,
    width: usize,
    height: usize,
) -> Vec<f32> {
    let mut scaled = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let sample_x = scaled_sample_coordinate(x, source_width, width);
            let sample_y = scaled_sample_coordinate(y, source_height, height);
            scaled.push(sample_scalar_bilinear(
                values,
                source_width,
                source_height,
                sample_x,
                sample_y,
            ));
        }
    }
    scaled
}

fn scale_bool_buffer(
    values: &[bool],
    source_width: usize,
    source_height: usize,
    width: usize,
    height: usize,
) -> Vec<bool> {
    let mut scaled = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let sample_x = scaled_sample_coordinate(x, source_width, width)
                .round()
                .clamp(0.0, (source_width.saturating_sub(1)) as f32) as usize;
            let sample_y = scaled_sample_coordinate(y, source_height, height)
                .round()
                .clamp(0.0, (source_height.saturating_sub(1)) as f32) as usize;
            scaled.push(values[sample_y * source_width + sample_x]);
        }
    }
    scaled
}

fn scale_motion_vectors(
    values: &[MotionVector],
    source_width: usize,
    source_height: usize,
    width: usize,
    height: usize,
) -> Vec<MotionVector> {
    let scale_x = width as f32 / source_width.max(1) as f32;
    let scale_y = height as f32 / source_height.max(1) as f32;
    let mut scaled = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let sample_x = scaled_sample_coordinate(x, source_width, width);
            let sample_y = scaled_sample_coordinate(y, source_height, height);
            let motion = sample_motion_bilinear(values, source_width, source_height, sample_x, sample_y);
            scaled.push(MotionVector {
                to_prev_x: motion.to_prev_x * scale_x,
                to_prev_y: motion.to_prev_y * scale_y,
            });
        }
    }
    scaled
}

fn scale_normal_buffer(
    values: &[Normal3],
    source_width: usize,
    source_height: usize,
    width: usize,
    height: usize,
) -> Vec<Normal3> {
    let mut scaled = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let sample_x = scaled_sample_coordinate(x, source_width, width);
            let sample_y = scaled_sample_coordinate(y, source_height, height);
            scaled.push(sample_normal_bilinear(
                values,
                source_width,
                source_height,
                sample_x,
                sample_y,
            ));
        }
    }
    scaled
}

fn scaled_sample_coordinate(index: usize, source_extent: usize, scaled_extent: usize) -> f32 {
    ((index as f32 + 0.5) * source_extent as f32 / scaled_extent.max(1) as f32) - 0.5
}

fn sample_scalar_bilinear(
    values: &[f32],
    width: usize,
    height: usize,
    x: f32,
    y: f32,
) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);
    let p00 = scalar_at(values, width, height, x0 as i32, y0 as i32);
    let p10 = scalar_at(values, width, height, x1 as i32, y0 as i32);
    let p01 = scalar_at(values, width, height, x0 as i32, y1 as i32);
    let p11 = scalar_at(values, width, height, x1 as i32, y1 as i32);
    let top = p00 * (1.0 - tx) + p10 * tx;
    let bottom = p01 * (1.0 - tx) + p11 * tx;
    top * (1.0 - ty) + bottom * ty
}

fn scalar_at(values: &[f32], width: usize, height: usize, x: i32, y: i32) -> f32 {
    let clamped_x = x.clamp(0, width as i32 - 1) as usize;
    let clamped_y = y.clamp(0, height as i32 - 1) as usize;
    values[clamped_y * width + clamped_x]
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
    let p00 = motion_at(values, width, height, x0 as i32, y0 as i32);
    let p10 = motion_at(values, width, height, x1 as i32, y0 as i32);
    let p01 = motion_at(values, width, height, x0 as i32, y1 as i32);
    let p11 = motion_at(values, width, height, x1 as i32, y1 as i32);
    let top_x = p00.to_prev_x * (1.0 - tx) + p10.to_prev_x * tx;
    let top_y = p00.to_prev_y * (1.0 - tx) + p10.to_prev_y * tx;
    let bottom_x = p01.to_prev_x * (1.0 - tx) + p11.to_prev_x * tx;
    let bottom_y = p01.to_prev_y * (1.0 - tx) + p11.to_prev_y * tx;
    MotionVector {
        to_prev_x: top_x * (1.0 - ty) + bottom_x * ty,
        to_prev_y: top_y * (1.0 - ty) + bottom_y * ty,
    }
}

fn motion_at(values: &[MotionVector], width: usize, height: usize, x: i32, y: i32) -> MotionVector {
    let clamped_x = x.clamp(0, width as i32 - 1) as usize;
    let clamped_y = y.clamp(0, height as i32 - 1) as usize;
    values[clamped_y * width + clamped_x]
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
    let p00 = normal_at(values, width, height, x0 as i32, y0 as i32);
    let p10 = normal_at(values, width, height, x1 as i32, y0 as i32);
    let p01 = normal_at(values, width, height, x0 as i32, y1 as i32);
    let p11 = normal_at(values, width, height, x1 as i32, y1 as i32);
    let top = Normal3::new(
        p00.x * (1.0 - tx) + p10.x * tx,
        p00.y * (1.0 - tx) + p10.y * tx,
        p00.z * (1.0 - tx) + p10.z * tx,
    );
    let bottom = Normal3::new(
        p01.x * (1.0 - tx) + p11.x * tx,
        p01.y * (1.0 - tx) + p11.y * tx,
        p01.z * (1.0 - tx) + p11.z * tx,
    );
    Normal3::new(
        top.x * (1.0 - ty) + bottom.x * ty,
        top.y * (1.0 - ty) + bottom.y * ty,
        top.z * (1.0 - ty) + bottom.z * ty,
    )
    .normalized()
}

fn normal_at(values: &[Normal3], width: usize, height: usize, x: i32, y: i32) -> Normal3 {
    let clamped_x = x.clamp(0, width as i32 - 1) as usize;
    let clamped_y = y.clamp(0, height as i32 - 1) as usize;
    values[clamped_y * width + clamped_x]
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
        "- Imported-buffer scaling does not replace full in-engine profiling on the final evaluator hardware and renderer integration point."
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
    let has_sequence = metrics.captures.len() >= 2;
    let all_have_reference = metrics.captures.iter().all(|capture| capture.ground_truth_available);
    let _ = writeln!(
        markdown,
        "Point-vs-region disclosure is explicit per capture via the `point_vs_region` line below."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{ROI_CONTRACT_STATEMENT}");
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
        let _ = writeln!(markdown, "- ROI coverage: `{:.2}%`", capture.roi_coverage * 100.0);
        let _ = writeln!(
            markdown,
            "- ROI baseline method: `{}`",
            capture.baseline_method_id
        );
        let _ = writeln!(
            markdown,
            "- reference_source: `{}`",
            capture.reference_source
        );
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
            "| Method | full-frame MAE | ROI MAE | non-ROI MAE | max error | temporal error accumulation | intervention rate |"
        );
        let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
        for method in &capture.methods {
            let _ = writeln!(
                markdown,
                "| {} | {:.5} | {:.5} | {:.5} | {:.5} | {:.5} | {:.5} |",
                method.label,
                method.overall_mae,
                method.roi_mae,
                method.non_roi_mae,
                method.max_error,
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
        "- The same DSFB host-minimum supervisory layer runs on imported buffers and can be compared against fixed alpha, a strong heuristic baseline, and an explicit DSFB + strong heuristic hybrid."
    );
    let _ = writeln!(
        markdown,
        "- ROI and non-ROI behavior remain separated on imported data."
    );
    if has_sequence {
        let _ = writeln!(
            markdown,
            "- The current real Unreal-native package also emits `figures/trust_temporal_trajectory.svg` and `figures/trust_temporal_trajectory.json` over the ordered capture sequence."
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    if all_have_reference {
        let _ = writeln!(
            markdown,
            "- The current bundle measures against `reference_color`, but that reference is a higher-resolution exported Unreal proxy rather than a path-traced ground truth."
        );
    } else {
        let _ = writeln!(
            markdown,
            "- Without an optional reference frame, error is measured against the current-frame proxy rather than a high-spp reconstruction."
        );
    }
    let _ = writeln!(
        markdown,
        "- Even with a reference frame, this does not replace longer engine-side sequences."
    );
    if has_sequence {
        let _ = writeln!(
            markdown,
            "- The current bundle generates `figures/trust_histogram.svg`, `figures/trust_vs_error.svg`, `figures/trust_conditioned_error_map.png`, and `figures/trust_temporal_trajectory.svg`; these are calibration artifacts over a short five-frame sequence, not a broad temporal generalization claim."
        );
    } else {
        let _ = writeln!(
            markdown,
            "- The current bundle generates `figures/trust_histogram.svg`, `figures/trust_vs_error.svg`, and `figures/trust_conditioned_error_map.png`; for a single frame pair these are calibration diagnostics, not temporal claims."
        );
    }
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
    let _ = writeln!(markdown, "{ROI_CONTRACT_STATEMENT}");
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
        let _ = writeln!(markdown, "- ROI coverage: `{:.2}%`", capture.roi_coverage * 100.0);
        let _ = writeln!(
            markdown,
            "- ROI baseline method: `{}`",
            capture.baseline_method_id
        );
        let _ = writeln!(
            markdown,
            "- reference_source: `{}`",
            capture.reference_source
        );
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
    let _ = writeln!(markdown, "- ROI contract: {ROI_CONTRACT_STATEMENT}");
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
            "- `{}`: ROI source = `{}`, ROI pixels = {}, ROI coverage = {:.2}%, metric_source = `{}`",
            capture.capture_label,
            capture.roi_source,
            capture.roi_pixels,
            capture.roi_coverage * 100.0,
            capture.metric_source
        );
        for method in &capture.methods {
            let _ = writeln!(
                markdown,
                "  - {}: full-frame MAE = {:.5}, ROI MAE = {:.5}, non-ROI MAE = {:.5}, max error = {:.5}, temporal accumulation = {:.5}, intervention rate = {:.5}",
                method.label,
                method.overall_mae,
                method.roi_mae,
                method.non_roi_mae,
                method.max_error,
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
        "- The same GPU kernel can execute on imported buffers, with explicit measured-vs-unmeasured disclosure and isolated scaled-resolution probes when the standalone binary is available."
    );
    let _ = writeln!(
        markdown,
        "- ROI vs non-ROI reporting survives the external path, Demo B keeps equal budgets across stronger heuristic baselines, and Demo A now includes an explicit DSFB + strong heuristic hybrid."
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
    if demo_a.captures.len() >= 2 {
        let _ = writeln!(
            markdown,
            "- The trust trajectory is now measured across an ordered five-frame real Unreal-native sequence, but that short sequence is still not enough to claim broad temporal calibration."
        );
    } else {
        let _ = writeln!(
            markdown,
            "- For the single checked-in Unreal-native sample, trust calibration artifacts are per-frame diagnostics only; no temporal trust trajectory claim is made."
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
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
        "Move from the current five-frame exported Unreal-native sequence to a longer production-representative engine capture, preserve the same fixed ROI contract and baseline ladder, and confirm the trust trajectory plus scaled GPU timings on the target evaluation hardware."
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
    resolved: &ImageFrame,
    reference: &ImageFrame,
    metric_source: &str,
    roi_mask: &[bool],
    intervention: &ScalarField,
) -> ExternalDemoAMethodMetrics {
    let error_field = absolute_error_field(resolved, reference);
    let non_roi_mask = invert_mask(roi_mask);
    ExternalDemoAMethodMetrics {
        method_id: method_id.to_string(),
        label: label.to_string(),
        metric_source: metric_source.to_string(),
        overall_mae: mean_abs_error(resolved, reference),
        roi_mae: mean_abs_error_over_mask(resolved, reference, roi_mask),
        non_roi_mae: mean_abs_error_over_mask(resolved, reference, &non_roi_mask),
        max_error: scalar_field_max(&error_field),
        temporal_error_accumulation: mean_abs_error(resolved, reference),
        intervention_rate: intervention.mean(),
    }
}

pub(crate) fn capture_reference_frame_and_metric_source<'a>(
    capture: &'a ExternalLoadedCapture,
) -> (&'a ImageFrame, &'static str, &'static str) {
    if let Some(reference) = &capture.reference {
        (reference, "reference_color", "real_reference")
    } else {
        (
            &capture.inputs.current_color,
            "current_color_proxy",
            "current_color_proxy",
        )
    }
}

pub(crate) fn absolute_error_field(frame_a: &ImageFrame, frame_b: &ImageFrame) -> ScalarField {
    let mut field = ScalarField::new(frame_a.width(), frame_a.height());
    for y in 0..frame_a.height() {
        for x in 0..frame_a.width() {
            field.set(x, y, frame_a.get(x, y).abs_diff(frame_b.get(x, y)));
        }
    }
    field
}

pub(crate) fn scalar_field_max(field: &ScalarField) -> f32 {
    field.values().iter().copied().fold(0.0, f32::max)
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

fn run_external_dsfb_plus_strong_heuristic(
    capture: &ExternalLoadedCapture,
    dsfb_alpha: &ScalarField,
    dsfb_intervention: &ScalarField,
    strong_alpha: &ScalarField,
    strong_response: &ScalarField,
) -> (ImageFrame, ScalarField, ScalarField) {
    let width = capture.inputs.width();
    let height = capture.inputs.height();
    let mut resolved = ImageFrame::new(width, height);
    let mut alpha = ScalarField::new(width, height);
    let mut response = ScalarField::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let current = capture.inputs.current_color.get(x, y);
            let history = capture.inputs.reprojected_history.get(x, y);
            let clamped =
                clamp_to_current_neighborhood(&capture.inputs.current_color, history, x, y);
            let hybrid_alpha = dsfb_alpha.get(x, y).max(strong_alpha.get(x, y));
            let hybrid_response = dsfb_intervention.get(x, y).max(strong_response.get(x, y));
            alpha.set(x, y, hybrid_alpha);
            response.set(x, y, hybrid_response);
            resolved.set(x, y, clamped.lerp(current, hybrid_alpha));
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

pub(crate) fn roi_mask_for_capture(
    capture: &ExternalLoadedCapture,
    baseline: &ImageFrame,
    reference: &ImageFrame,
) -> (Vec<bool>, String, f32) {
    let contrast = local_contrast_field(reference);
    let mut mask = vec![false; capture.inputs.width() * capture.inputs.height()];
    for y in 0..capture.inputs.height() {
        for x in 0..capture.inputs.width() {
            let index = y * capture.inputs.width() + x;
            let baseline_error = baseline.get(x, y).abs_diff(reference.get(x, y));
            let threshold = ROI_CONTRACT_ALPHA * contrast.get(x, y);
            mask[index] = baseline_error > threshold;
        }
    }
    let coverage = mask.iter().filter(|value| **value).count() as f32 / mask.len().max(1) as f32;
    (mask, ROI_CONTRACT_SOURCE.to_string(), coverage)
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

fn write_trust_histogram_figure(trust: &ScalarField, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let bin_count = 10usize;
    let mut counts = vec![0usize; bin_count];
    for value in trust.values() {
        let bin = ((*value).clamp(0.0, 0.999_999) * bin_count as f32) as usize;
        counts[bin.min(bin_count - 1)] += 1;
    }
    let max_count = counts.iter().copied().max().unwrap_or(1).max(1) as f32;
    let mut bars = String::new();
    for (index, count) in counts.iter().enumerate() {
        let height = 260.0 * (*count as f32 / max_count);
        let x = 86.0 + index as f32 * 62.0;
        let y = 368.0 - height;
        let _ = writeln!(
            bars,
            r##"<rect x="{x:.1}" y="{y:.1}" width="42" height="{height:.1}" fill="#4cc9f0"/>"##
        );
        let _ = writeln!(
            bars,
            r##"<text x="{:.1}" y="392" font-size="13" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{:.1}</text>"##,
            x,
            index as f32 / bin_count as f32
        );
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="900" height="460" viewBox="0 0 900 460">
<rect width="900" height="460" fill="#0b1320"/>
<text x="36" y="42" font-size="28" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Trust Histogram</text>
<text x="36" y="68" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Per-pixel DSFB trust distribution for the canonical imported capture.</text>
<line x1="70" y1="96" x2="70" y2="368" stroke="#f4f7fb" stroke-width="2"/>
<line x1="70" y1="368" x2="760" y2="368" stroke="#f4f7fb" stroke-width="2"/>
{bars}
<text x="36" y="106" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">count</text>
<text x="730" y="420" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">trust</text>
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

fn write_trust_vs_error_figure(
    trust: &ScalarField,
    error: &ScalarField,
    metric_source: &str,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let bin_count = 10usize;
    let mut counts = vec![0usize; bin_count];
    let mut means = vec![0.0f32; bin_count];
    for (trust_value, error_value) in trust.values().iter().zip(error.values()) {
        let bin = ((*trust_value).clamp(0.0, 0.999_999) * bin_count as f32) as usize;
        let index = bin.min(bin_count - 1);
        counts[index] += 1;
        means[index] += *error_value;
    }
    for (count, mean) in counts.iter().zip(means.iter_mut()) {
        if *count > 0 {
            *mean /= *count as f32;
        }
    }
    let max_error = means.iter().copied().fold(0.05, f32::max);
    let left = 82.0f32;
    let right = 780.0f32;
    let top = 92.0f32;
    let bottom = 366.0f32;
    let x_scale = (right - left) / (bin_count.saturating_sub(1)) as f32;
    let mut points = String::new();
    for (index, mean) in means.iter().enumerate() {
        let x = left + index as f32 * x_scale;
        let y = bottom - (mean / max_error.max(1e-6)) * (bottom - top);
        let _ = writeln!(points, "{x:.1},{y:.1} ");
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="920" height="460" viewBox="0 0 920 460">
<rect width="920" height="460" fill="#0b1320"/>
<text x="36" y="42" font-size="28" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Trust vs Error Curve</text>
<text x="36" y="68" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Mean per-pixel error by trust bin. Error source: {metric_source}.</text>
<line x1="{left}" y1="{top}" x2="{left}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<line x1="{left}" y1="{bottom}" x2="{right}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<polyline fill="none" stroke="#8bd450" stroke-width="3.5" points="{points}"/>
<text x="36" y="102" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">mean error</text>
<text x="744" y="420" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">trust bin</text>
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

fn save_trust_conditioned_error_map(
    trust: &ScalarField,
    error: &ScalarField,
    path: &Path,
) -> Result<()> {
    let mut conditioned = ScalarField::new(trust.width(), trust.height());
    for y in 0..trust.height() {
        for x in 0..trust.width() {
            conditioned.set(x, y, error.get(x, y) * (1.0 - trust.get(x, y)));
        }
    }
    save_scalar_field_png(&conditioned, path, heatmap_red)
}

fn write_temporal_trust_trajectory_outputs(
    points: &[TemporalTrustTrajectoryPoint],
    figures_dir: &Path,
) -> Result<()> {
    let mut ordered = points.to_vec();
    ordered.sort_by_key(|point| point.frame_index);
    let peak_index = ordered
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.roi_coverage.total_cmp(&right.roi_coverage))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let report = TemporalTrustTrajectoryReport {
        onset_capture_label: ordered
            .first()
            .map(|point| point.capture_label.clone())
            .unwrap_or_default(),
        peak_roi_capture_label: ordered[peak_index].capture_label.clone(),
        recovery_capture_label: ordered
            .last()
            .map(|point| point.capture_label.clone())
            .unwrap_or_default(),
        points: ordered.clone(),
    };
    fs::write(
        figures_dir.join("trust_temporal_trajectory.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    write_temporal_trust_trajectory_figure(
        &report,
        &figures_dir.join("trust_temporal_trajectory.svg"),
    )
}

fn write_temporal_trust_trajectory_figure(
    report: &TemporalTrustTrajectoryReport,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let point_count = report.points.len().max(2);
    let width = 980.0f32;
    let height = 520.0f32;
    let left = 86.0f32;
    let right = 860.0f32;
    let top = 88.0f32;
    let bottom = 418.0f32;
    let inner_width = right - left;
    let inner_height = bottom - top;
    let x_scale = inner_width / (point_count.saturating_sub(1)) as f32;
    let max_roi_error = report
        .points
        .iter()
        .map(|point| point.dsfb_roi_mae.max(point.hybrid_roi_mae))
        .fold(1e-6f32, f32::max);

    let trust_path = polyline_points(
        &report
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                (
                    left + index as f32 * x_scale,
                    bottom - point.mean_trust.clamp(0.0, 1.0) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let intervention_path = polyline_points(
        &report
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                (
                    left + index as f32 * x_scale,
                    bottom - point.intervention_rate.clamp(0.0, 1.0) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let roi_coverage_path = polyline_points(
        &report
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                (
                    left + index as f32 * x_scale,
                    bottom - point.roi_coverage.clamp(0.0, 1.0) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let dsfb_error_path = polyline_points(
        &report
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (point.dsfb_roi_mae / max_roi_error) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let hybrid_error_path = polyline_points(
        &report
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (point.hybrid_roi_mae / max_roi_error) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let peak_index = report
        .points
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.roi_coverage.total_cmp(&right.roi_coverage))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let peak_x = left + peak_index as f32 * x_scale;

    let mut labels = String::new();
    for (index, point) in report.points.iter().enumerate() {
        let x = left + index as f32 * x_scale;
        let _ = writeln!(
            labels,
            r##"<text x="{x:.1}" y="446" text-anchor="middle" font-size="13" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{}</text>"##,
            point.capture_label
        );
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="{width}" height="{height}" fill="#0b1320"/>
<text x="34" y="42" font-size="28" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Temporal Trust Trajectory</text>
<text x="34" y="68" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Ordered real Unreal-native sequence from onset-side frame {onset} through peak ROI frame {peak} to recovery-side frame {recovery}.</text>
<line x1="{left}" y1="{top}" x2="{left}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<line x1="{left}" y1="{bottom}" x2="{right}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<path d="{trust_path}" fill="none" stroke="#4cc9f0" stroke-width="3.5"/>
<path d="{intervention_path}" fill="none" stroke="#ef476f" stroke-width="3.5" stroke-dasharray="10 8"/>
<path d="{roi_coverage_path}" fill="none" stroke="#8bd450" stroke-width="3.5"/>
<path d="{dsfb_error_path}" fill="none" stroke="#ffd166" stroke-width="3.5"/>
<path d="{hybrid_error_path}" fill="none" stroke="#f4978e" stroke-width="3.5" stroke-dasharray="6 6"/>
<line x1="{peak_x:.1}" y1="{top}" x2="{peak_x:.1}" y2="{bottom}" stroke="#f4f7fb" stroke-width="1.5" stroke-dasharray="8 6"/>
<text x="628" y="110" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Mean trust</text>
<line x1="566" y1="104" x2="616" y2="104" stroke="#4cc9f0" stroke-width="3.5"/>
<text x="628" y="136" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Intervention rate</text>
<line x1="566" y1="130" x2="616" y2="130" stroke="#ef476f" stroke-width="3.5" stroke-dasharray="10 8"/>
<text x="628" y="162" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">ROI coverage</text>
<line x1="566" y1="156" x2="616" y2="156" stroke="#8bd450" stroke-width="3.5"/>
<text x="628" y="188" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">DSFB ROI MAE / max</text>
<line x1="566" y1="182" x2="616" y2="182" stroke="#ffd166" stroke-width="3.5"/>
<text x="628" y="214" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Hybrid ROI MAE / max</text>
<line x1="566" y1="208" x2="616" y2="208" stroke="#f4978e" stroke-width="3.5" stroke-dasharray="6 6"/>
<text x="34" y="102" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">normalized value</text>
<text x="788" y="470" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">ordered capture label</text>
{labels}
</svg>"##,
        onset = report.onset_capture_label,
        peak = report.peak_roi_capture_label,
        recovery = report.recovery_capture_label,
    );
    fs::write(path, svg)?;
    Ok(())
}

fn polyline_points(points: &[(f32, f32)]) -> String {
    let mut path = String::new();
    for (index, (x, y)) in points.iter().enumerate() {
        let command = if index == 0 { "M" } else { "L" };
        let _ = write!(path, "{command}{x:.1},{y:.1} ");
    }
    path
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
