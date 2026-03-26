use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::DemoConfig;
use crate::error::{Error, Result};
use crate::external::{
    load_bool_buffer_from_reference, load_color_buffer_from_reference,
    load_scalar_buffer_from_reference, load_vec2_buffer_from_reference,
    load_vec3_buffer_from_reference, BufferReference, ExternalBufferSet, ExternalCaptureEntry,
    ExternalCaptureManifest, ExternalCaptureMetadata, ExternalCaptureSource, ExternalNormalization,
    EXTERNAL_CAPTURE_FORMAT_VERSION,
};
use crate::external_validation::{
    run_external_validation_bundle, ExternalDemoAMetrics, ExternalDemoBMetrics, ExternalGpuMetrics,
    ExternalScalingMetrics,
};
use crate::frame::{
    mean_abs_error_over_mask, save_scalar_field_png, Color, ImageFrame, ScalarField,
};
use crate::host::{
    default_host_realistic_profile, supervise_temporal_reuse, HostSupervisionOutputs,
};
use crate::scene::{MotionVector, Normal3};

pub const UNREAL_NATIVE_SCHEMA_VERSION: &str = "dsfb_unreal_native_v1";
pub const UNREAL_NATIVE_DATASET_KIND: &str = "unreal_native";
pub const UNREAL_NATIVE_PROVENANCE_LABEL: &str = "unreal_native";
pub const UNREAL_NATIVE_PDF_FILE_NAME: &str = "artifacts_bundle.pdf";
pub const UNREAL_NATIVE_ZIP_FILE_NAME: &str = "artifacts_bundle.zip";
pub const UNREAL_NATIVE_EXECUTIVE_SHEET_FILE_NAME: &str = "executive_evidence_sheet.png";
pub const UNREAL_NATIVE_EVIDENCE_MANIFEST_FILE_NAME: &str = "evidence_bundle_manifest.json";

#[derive(Clone, Debug)]
pub struct UnrealNativeArtifacts {
    pub run_dir: PathBuf,
    pub materialized_manifest_path: PathBuf,
    pub summary_path: PathBuf,
    pub metrics_csv_path: PathBuf,
    pub metrics_summary_path: PathBuf,
    pub comparison_summary_path: PathBuf,
    pub provenance_path: PathBuf,
    pub failure_modes_path: PathBuf,
    pub notebook_manifest_path: PathBuf,
    pub executive_sheet_path: PathBuf,
    pub pdf_path: PathBuf,
    pub zip_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealNativeManifest {
    pub schema_version: String,
    pub dataset_kind: String,
    pub provenance_label: String,
    pub dataset_id: String,
    pub description: String,
    pub engine: UnrealEngineInfo,
    pub contract: UnrealCaptureContract,
    pub frames: Vec<UnrealFrameEntry>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealEngineInfo {
    pub engine_name: String,
    pub engine_version: String,
    pub capture_tool: String,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub capture_script: Option<String>,
    #[serde(default)]
    pub real_engine_capture: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealCaptureContract {
    pub color_space: String,
    pub tonemap: String,
    pub depth_convention: String,
    pub normal_space: String,
    pub motion_vector_convention: String,
    pub coordinate_space: String,
    #[serde(default)]
    pub history_source: String,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealFrameEntry {
    pub label: String,
    pub frame_index: usize,
    pub history_frame_index: usize,
    pub buffers: UnrealFrameBuffers,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealFrameBuffers {
    pub current_color: BufferReference,
    pub previous_color: BufferReference,
    #[serde(default)]
    pub history_color: Option<BufferReference>,
    pub motion_vectors: BufferReference,
    pub current_depth: BufferReference,
    pub previous_depth: BufferReference,
    #[serde(default)]
    pub history_depth: Option<BufferReference>,
    pub current_normals: BufferReference,
    pub previous_normals: BufferReference,
    #[serde(default)]
    pub history_normals: Option<BufferReference>,
    pub metadata: BufferReference,
    #[serde(default)]
    pub host_output: Option<BufferReference>,
    #[serde(default)]
    pub reference_color: Option<BufferReference>,
    #[serde(default)]
    pub roi_mask: Option<BufferReference>,
    #[serde(default)]
    pub disocclusion_mask: Option<BufferReference>,
    #[serde(default)]
    pub reactive_mask: Option<BufferReference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealFrameMetadata {
    pub frame_index: usize,
    pub history_frame_index: usize,
    pub width: usize,
    pub height: usize,
    pub source_kind: String,
    #[serde(default)]
    pub externally_validated: bool,
    #[serde(default)]
    pub real_external_data: bool,
    #[serde(default)]
    pub data_description: Option<String>,
    #[serde(default)]
    pub provenance_label: Option<String>,
    #[serde(default)]
    pub scene_name: Option<String>,
    #[serde(default)]
    pub shot_name: Option<String>,
    #[serde(default)]
    pub exposure: Option<String>,
    #[serde(default)]
    pub tonemap: Option<String>,
    #[serde(default)]
    pub camera: Option<UnrealCameraMetadata>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealCameraMetadata {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub position: Option<[f32; 3]>,
    #[serde(default)]
    pub forward: Option<[f32; 3]>,
    #[serde(default)]
    pub fov_degrees: Option<f32>,
    #[serde(default)]
    pub jitter_pixels: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize)]
struct ProvenanceRecord {
    schema_version: String,
    dataset_kind: String,
    provenance_label: String,
    dataset_id: String,
    manifest_path: String,
    materialized_manifest_path: String,
    run_name: String,
    run_dir: String,
    timestamp_epoch_seconds: u64,
    git_commit: Option<String>,
    cli_args: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct SummaryRecord {
    schema_version: String,
    dataset_kind: String,
    provenance_label: String,
    dataset_id: String,
    run_name: String,
    run_dir: String,
    capture_count: usize,
    classification_counts: ClassificationCounts,
    executive_capture_label: String,
    pdf_file_name: String,
    zip_file_name: String,
    notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct MetricsSummaryRecord {
    dataset_id: String,
    provenance_label: String,
    capture_count: usize,
    classification_counts: ClassificationCounts,
    captures: Vec<CaptureSummaryRecord>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ClassificationCounts {
    dsfb_helpful: usize,
    dsfb_neutral: usize,
    heuristic_favorable: usize,
    richer_cues_required: usize,
}

#[derive(Clone, Debug, Serialize)]
struct CaptureSummaryRecord {
    capture_label: String,
    scene_name: String,
    shot_name: String,
    frame_index: usize,
    classification: String,
    metric_source: String,
    dsfb_roi_mae: f32,
    strong_heuristic_roi_mae: f32,
    fixed_alpha_roi_mae: f32,
    dsfb_mean_trust: f32,
    dsfb_mean_alpha: f32,
    dsfb_intervention_rate: f32,
    roi_residual_mean: f32,
    instability_fraction: f32,
    gpu_total_ms: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct EvidenceBundleManifest {
    dataset_id: String,
    provenance_label: String,
    run_name: String,
    pdf_file_name: String,
    zip_file_name: String,
    executive_sheet_file_name: String,
    summary_file_name: String,
    metrics_summary_file_name: String,
    comparison_summary_file_name: String,
    failure_modes_file_name: String,
    frames: Vec<EvidenceFrameManifest>,
}

#[derive(Clone, Debug, Serialize)]
struct EvidenceFrameManifest {
    label: String,
    scene_name: String,
    shot_name: String,
    frame_index: usize,
    classification: String,
    explanation: EvidenceExplanation,
    key_metrics: Vec<KeyMetric>,
    current_frame_path: String,
    baseline_frame_path: String,
    trust_map_path: String,
    alpha_map_path: String,
    intervention_map_path: String,
    residual_map_path: String,
    instability_overlay_path: String,
    roi_overlay_path: String,
    output_panel_path: String,
}

#[derive(Clone, Debug, Serialize)]
struct EvidenceExplanation {
    what_went_wrong: String,
    what_dsfb_detected: String,
    what_dsfb_changed: String,
    overhead_and_caveat: String,
}

#[derive(Clone, Debug, Serialize)]
struct KeyMetric {
    label: String,
    value: String,
}

#[derive(Clone, Debug, Serialize)]
struct NotebookManifest {
    dataset_id: String,
    provenance_label: String,
    run_name: String,
    run_dir_name: String,
    executive_sheet_file_name: String,
    pdf_bundle_file_name: String,
    zip_bundle_file_name: String,
    comparison_summary_file_name: String,
    primary_panel_file_name: String,
}

#[derive(Clone, Debug)]
struct MaterializedRun {
    materialized_manifest_path: PathBuf,
    captures: Vec<MaterializedCapture>,
}

#[derive(Clone, Debug)]
struct MaterializedCapture {
    label: String,
    frame_index: usize,
    scene_name: String,
    shot_name: String,
    baseline_source: String,
    host_output: Option<ImageFrame>,
    roi_mask: Option<Vec<bool>>,
    disocclusion_mask: Option<Vec<bool>>,
}

#[derive(Clone, Debug)]
struct FrameArtifacts {
    label: String,
    scene_name: String,
    shot_name: String,
    frame_index: usize,
    classification: String,
    current_frame_path: PathBuf,
    baseline_frame_path: PathBuf,
    trust_map_path: PathBuf,
    alpha_map_path: PathBuf,
    intervention_map_path: PathBuf,
    residual_map_path: PathBuf,
    instability_overlay_path: PathBuf,
    roi_overlay_path: PathBuf,
    output_panel_path: PathBuf,
    metric_source: String,
    dsfb_roi_mae: f32,
    strong_heuristic_roi_mae: f32,
    fixed_alpha_roi_mae: f32,
    dsfb_mean_trust: f32,
    dsfb_mean_alpha: f32,
    dsfb_intervention_rate: f32,
    roi_residual_mean: f32,
    instability_fraction: f32,
    gpu_total_ms: Option<f64>,
    explanation: EvidenceExplanation,
}

#[derive(Clone, Debug)]
struct DemoAMethodSelection<'a> {
    metric_source: &'a str,
    fixed_alpha: &'a crate::external_validation::ExternalDemoAMethodMetrics,
    strong_heuristic: &'a crate::external_validation::ExternalDemoAMethodMetrics,
    dsfb: &'a crate::external_validation::ExternalDemoAMethodMetrics,
}

pub fn run_unreal_native(
    config: &DemoConfig,
    manifest_path: &Path,
    output_root: &Path,
    run_name_override: Option<&str>,
    cli_args: &[String],
) -> Result<UnrealNativeArtifacts> {
    let manifest = load_and_validate_manifest(manifest_path)?;
    let run_name = run_name_override
        .map(|value| value.to_string())
        .unwrap_or_else(default_run_name);
    let run_dir = create_run_dir(output_root, &run_name)?;

    let materialized = materialize_unreal_manifest(&manifest, manifest_path, &run_dir)?;
    run_external_validation_bundle(config, &materialized.materialized_manifest_path, &run_dir)?;

    let demo_a: ExternalDemoAMetrics =
        read_json(&run_dir.join("demo_a_external_metrics.json"))?;
    let demo_b: ExternalDemoBMetrics =
        read_json(&run_dir.join("demo_b_external_metrics.json"))?;
    let gpu: ExternalGpuMetrics = read_json(&run_dir.join("gpu_execution_metrics.json"))?;
    let scaling: ExternalScalingMetrics = read_json(&run_dir.join("scaling_metrics.json"))?;

    let per_frame_dir = run_dir.join("per_frame");
    fs::create_dir_all(&per_frame_dir)?;
    let frame_artifacts = generate_per_frame_artifacts(
        config,
        &materialized.materialized_manifest_path,
        &materialized.captures,
        &demo_a,
        &gpu,
        &per_frame_dir,
    )?;

    let comparison_summary_path = run_dir.join("comparison_summary.md");
    let metrics_csv_path = run_dir.join("metrics.csv");
    let metrics_summary_path = run_dir.join("metrics_summary.json");
    let failure_modes_path = run_dir.join("failure_modes.md");
    let provenance_path = run_dir.join("provenance.json");
    let run_manifest_path = run_dir.join("run_manifest.json");
    let summary_path = run_dir.join("summary.json");
    let evidence_manifest_path = run_dir.join(UNREAL_NATIVE_EVIDENCE_MANIFEST_FILE_NAME);
    let notebook_manifest_path = run_dir.join("notebook_manifest.json");

    write_run_manifest(&run_manifest_path, manifest_path, &manifest, &materialized)?;

    let counts = classification_counts(&frame_artifacts);
    let capture_summaries = frame_artifacts
        .iter()
        .map(|frame| CaptureSummaryRecord {
            capture_label: frame.label.clone(),
            scene_name: frame.scene_name.clone(),
            shot_name: frame.shot_name.clone(),
            frame_index: frame.frame_index,
            classification: frame.classification.clone(),
            metric_source: frame.metric_source.clone(),
            dsfb_roi_mae: frame.dsfb_roi_mae,
            strong_heuristic_roi_mae: frame.strong_heuristic_roi_mae,
            fixed_alpha_roi_mae: frame.fixed_alpha_roi_mae,
            dsfb_mean_trust: frame.dsfb_mean_trust,
            dsfb_mean_alpha: frame.dsfb_mean_alpha,
            dsfb_intervention_rate: frame.dsfb_intervention_rate,
            roi_residual_mean: frame.roi_residual_mean,
            instability_fraction: frame.instability_fraction,
            gpu_total_ms: frame.gpu_total_ms,
        })
        .collect::<Vec<_>>();

    write_metrics_csv(&metrics_csv_path, &capture_summaries, &demo_b)?;
    let metrics_summary = MetricsSummaryRecord {
        dataset_id: manifest.dataset_id.clone(),
        provenance_label: manifest.provenance_label.clone(),
        capture_count: capture_summaries.len(),
        classification_counts: counts.clone(),
        captures: capture_summaries.clone(),
    };
    fs::write(
        &metrics_summary_path,
        serde_json::to_string_pretty(&metrics_summary)?,
    )?;

    write_comparison_summary(&comparison_summary_path, &manifest, &frame_artifacts, &demo_b)?;
    write_failure_modes(&failure_modes_path, &manifest, &materialized.captures, &scaling)?;

    let timestamp_epoch_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let provenance = ProvenanceRecord {
        schema_version: manifest.schema_version.clone(),
        dataset_kind: manifest.dataset_kind.clone(),
        provenance_label: manifest.provenance_label.clone(),
        dataset_id: manifest.dataset_id.clone(),
        manifest_path: manifest_path.display().to_string(),
        materialized_manifest_path: materialized.materialized_manifest_path.display().to_string(),
        run_name: run_name.clone(),
        run_dir: run_dir.display().to_string(),
        timestamp_epoch_seconds,
        git_commit: git_commit_hash(),
        cli_args: cli_args.to_vec(),
    };
    fs::write(&provenance_path, serde_json::to_string_pretty(&provenance)?)?;

    let executive_frame = select_executive_frame(&frame_artifacts)?;
    let evidence_manifest = EvidenceBundleManifest {
        dataset_id: manifest.dataset_id.clone(),
        provenance_label: manifest.provenance_label.clone(),
        run_name: run_name.clone(),
        pdf_file_name: UNREAL_NATIVE_PDF_FILE_NAME.to_string(),
        zip_file_name: UNREAL_NATIVE_ZIP_FILE_NAME.to_string(),
        executive_sheet_file_name: UNREAL_NATIVE_EXECUTIVE_SHEET_FILE_NAME.to_string(),
        summary_file_name: "summary.json".to_string(),
        metrics_summary_file_name: "metrics_summary.json".to_string(),
        comparison_summary_file_name: "comparison_summary.md".to_string(),
        failure_modes_file_name: "failure_modes.md".to_string(),
        frames: frame_artifacts
            .iter()
            .map(|frame| EvidenceFrameManifest {
                label: frame.label.clone(),
                scene_name: frame.scene_name.clone(),
                shot_name: frame.shot_name.clone(),
                frame_index: frame.frame_index,
                classification: frame.classification.clone(),
                explanation: frame.explanation.clone(),
                key_metrics: vec![
                    KeyMetric {
                        label: "DSFB ROI MAE".to_string(),
                        value: format!("{:.5}", frame.dsfb_roi_mae),
                    },
                    KeyMetric {
                        label: "Strong heuristic ROI MAE".to_string(),
                        value: format!("{:.5}", frame.strong_heuristic_roi_mae),
                    },
                    KeyMetric {
                        label: "Mean trust".to_string(),
                        value: format!("{:.4}", frame.dsfb_mean_trust),
                    },
                    KeyMetric {
                        label: "Intervention rate".to_string(),
                        value: format!("{:.4}", frame.dsfb_intervention_rate),
                    },
                    KeyMetric {
                        label: "GPU total ms".to_string(),
                        value: frame
                            .gpu_total_ms
                            .map(|value| format!("{value:.4}"))
                            .unwrap_or_else(|| "n/a".to_string()),
                    },
                ],
                current_frame_path: relative_path_string(&frame.current_frame_path, &run_dir),
                baseline_frame_path: relative_path_string(&frame.baseline_frame_path, &run_dir),
                trust_map_path: relative_path_string(&frame.trust_map_path, &run_dir),
                alpha_map_path: relative_path_string(&frame.alpha_map_path, &run_dir),
                intervention_map_path: relative_path_string(&frame.intervention_map_path, &run_dir),
                residual_map_path: relative_path_string(&frame.residual_map_path, &run_dir),
                instability_overlay_path: relative_path_string(
                    &frame.instability_overlay_path,
                    &run_dir,
                ),
                roi_overlay_path: relative_path_string(&frame.roi_overlay_path, &run_dir),
                output_panel_path: relative_path_string(&frame.output_panel_path, &run_dir),
            })
            .collect(),
    };
    fs::write(
        &evidence_manifest_path,
        serde_json::to_string_pretty(&evidence_manifest)?,
    )?;

    let summary = SummaryRecord {
        schema_version: manifest.schema_version.clone(),
        dataset_kind: manifest.dataset_kind.clone(),
        provenance_label: manifest.provenance_label.clone(),
        dataset_id: manifest.dataset_id.clone(),
        run_name: run_name.clone(),
        run_dir: run_dir.display().to_string(),
        capture_count: frame_artifacts.len(),
        classification_counts: counts,
        executive_capture_label: executive_frame.label.clone(),
        pdf_file_name: UNREAL_NATIVE_PDF_FILE_NAME.to_string(),
        zip_file_name: UNREAL_NATIVE_ZIP_FILE_NAME.to_string(),
        notes: vec![
            "Engine-native empirical replay executed on a strict Unreal-native manifest.".to_string(),
            "No synthetic fallback is available in this mode.".to_string(),
            "Any missing required Unreal-native buffer is a hard failure.".to_string(),
        ],
    };
    fs::write(&summary_path, serde_json::to_string_pretty(&summary)?)?;

    run_bundle_builder(&run_dir)?;

    let notebook_manifest = NotebookManifest {
        dataset_id: manifest.dataset_id.clone(),
        provenance_label: manifest.provenance_label.clone(),
        run_name: run_name.clone(),
        run_dir_name: run_name.clone(),
        executive_sheet_file_name: UNREAL_NATIVE_EXECUTIVE_SHEET_FILE_NAME.to_string(),
        pdf_bundle_file_name: UNREAL_NATIVE_PDF_FILE_NAME.to_string(),
        zip_bundle_file_name: UNREAL_NATIVE_ZIP_FILE_NAME.to_string(),
        comparison_summary_file_name: "comparison_summary.md".to_string(),
        primary_panel_file_name: executive_frame
            .output_panel_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("boardroom_panel.png")
            .to_string(),
    };
    fs::write(
        &notebook_manifest_path,
        serde_json::to_string_pretty(&notebook_manifest)?,
    )?;

    let executive_sheet_path = run_dir.join(UNREAL_NATIVE_EXECUTIVE_SHEET_FILE_NAME);
    let pdf_path = run_dir.join(UNREAL_NATIVE_PDF_FILE_NAME);
    let zip_path = run_dir.join(UNREAL_NATIVE_ZIP_FILE_NAME);

    Ok(UnrealNativeArtifacts {
        run_dir,
        materialized_manifest_path: materialized.materialized_manifest_path,
        summary_path,
        metrics_csv_path,
        metrics_summary_path,
        comparison_summary_path,
        provenance_path,
        failure_modes_path,
        notebook_manifest_path,
        executive_sheet_path,
        pdf_path,
        zip_path,
    })
}

fn load_and_validate_manifest(path: &Path) -> Result<UnrealNativeManifest> {
    let manifest: UnrealNativeManifest = read_json(path)?;
    if manifest.schema_version != UNREAL_NATIVE_SCHEMA_VERSION {
        return Err(Error::Message(format!(
            "unreal-native manifest {} used schema_version `{}` but `{}` is required",
            path.display(),
            manifest.schema_version,
            UNREAL_NATIVE_SCHEMA_VERSION
        )));
    }
    if manifest.dataset_kind != UNREAL_NATIVE_DATASET_KIND {
        return Err(Error::Message(format!(
            "unreal-native manifest {} used dataset_kind `{}` but `{}` is required",
            path.display(),
            manifest.dataset_kind,
            UNREAL_NATIVE_DATASET_KIND
        )));
    }
    if manifest.provenance_label != UNREAL_NATIVE_PROVENANCE_LABEL {
        return Err(Error::Message(format!(
            "unreal-native manifest {} used provenance_label `{}` but `{}` is required",
            path.display(),
            manifest.provenance_label,
            UNREAL_NATIVE_PROVENANCE_LABEL
        )));
    }
    if manifest.engine.engine_name != "unreal_engine" {
        return Err(Error::Message(format!(
            "unreal-native manifest {} declared engine_name `{}`; only `unreal_engine` is accepted",
            path.display(),
            manifest.engine.engine_name
        )));
    }
    if !manifest.engine.real_engine_capture {
        return Err(Error::Message(format!(
            "unreal-native manifest {} is not marked as a real Unreal capture; this mode refuses pending, proxy, or synthetic provenance",
            path.display()
        )));
    }
    if manifest.frames.is_empty() {
        return Err(Error::Message(format!(
            "unreal-native manifest {} contained no frames",
            path.display()
        )));
    }
    validate_contract(&manifest.contract)?;
    validate_unique_frames(&manifest.frames)?;
    Ok(manifest)
}

fn validate_contract(contract: &UnrealCaptureContract) -> Result<()> {
    if !contract.color_space.contains("linear") {
        return Err(Error::Message(
            "unreal-native contract must declare a linear color space".to_string(),
        ));
    }
    if !matches!(
        contract.tonemap.as_str(),
        "disabled" | "pre_tonemap_capture" | "scene_capture_png_linearized"
    ) {
        return Err(Error::Message(format!(
            "unreal-native contract tonemap `{}` is unsupported; use `disabled`, `pre_tonemap_capture`, or `scene_capture_png_linearized`",
            contract.tonemap
        )));
    }
    if !matches!(contract.normal_space.as_str(), "view_space_unit" | "world_space_unit") {
        return Err(Error::Message(format!(
            "unreal-native contract normal_space `{}` is unsupported; use `view_space_unit` or `world_space_unit`",
            contract.normal_space
        )));
    }
    if !matches!(
        contract.depth_convention.as_str(),
        "monotonic_linear_depth" | "monotonic_visualized_depth"
    ) {
        return Err(Error::Message(format!(
            "unreal-native contract depth_convention `{}` is unsupported; use `monotonic_linear_depth` or `monotonic_visualized_depth`",
            contract.depth_convention
        )));
    }
    if !matches!(
        contract.motion_vector_convention.as_str(),
        "pixel_offset_to_prev" | "ndc_to_prev"
    ) {
        return Err(Error::Message(format!(
            "unreal-native contract motion_vector_convention `{}` is unsupported",
            contract.motion_vector_convention
        )));
    }
    Ok(())
}

fn validate_unique_frames(frames: &[UnrealFrameEntry]) -> Result<()> {
    let mut labels = BTreeSet::new();
    let mut indices = BTreeSet::new();
    for frame in frames {
        if !labels.insert(frame.label.clone()) {
            return Err(Error::Message(format!(
                "duplicate unreal-native capture label `{}`",
                frame.label
            )));
        }
        if !indices.insert(frame.frame_index) {
            return Err(Error::Message(format!(
                "duplicate unreal-native frame_index `{}`",
                frame.frame_index
            )));
        }
        if frame.history_frame_index >= frame.frame_index {
            return Err(Error::Message(format!(
                "capture `{}` must provide history_frame_index < frame_index",
                frame.label
            )));
        }
    }
    Ok(())
}

fn create_run_dir(output_root: &Path, run_name: &str) -> Result<PathBuf> {
    fs::create_dir_all(output_root)?;
    let run_dir = output_root.join(run_name);
    if run_dir.exists() {
        return Err(Error::Message(format!(
            "refusing to overwrite existing unreal-native run directory {}",
            run_dir.display()
        )));
    }
    fs::create_dir_all(&run_dir)?;
    Ok(run_dir)
}

fn default_run_name() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unreal_native_{seconds}")
}

fn materialize_unreal_manifest(
    manifest: &UnrealNativeManifest,
    manifest_path: &Path,
    run_dir: &Path,
) -> Result<MaterializedRun> {
    let base_dir = manifest_path.parent().ok_or_else(|| {
        Error::Message(format!(
            "unreal-native manifest {} had no parent directory",
            manifest_path.display()
        ))
    })?;
    let materialized_dir = run_dir.join("materialized_external");
    fs::create_dir_all(&materialized_dir)?;

    let mut sorted_frames = manifest.frames.clone();
    sorted_frames.sort_by_key(|frame| frame.frame_index);

    let mut external_entries = Vec::with_capacity(sorted_frames.len());
    let mut captures = Vec::with_capacity(sorted_frames.len());

    for frame in &sorted_frames {
        let metadata = load_unreal_frame_metadata(base_dir, &frame.buffers.metadata)?;
        validate_frame_metadata(frame, &metadata)?;

        let current_color = load_color_buffer_from_reference(
            base_dir,
            &frame.buffers.current_color,
            metadata.width,
            metadata.height,
        )?;
        let previous_color = load_color_buffer_from_reference(
            base_dir,
            &frame.buffers.previous_color,
            metadata.width,
            metadata.height,
        )?;
        let (_, _, mut motion_data) = load_vec2_buffer_from_reference(
            base_dir,
            &frame.buffers.motion_vectors,
            metadata.width,
            metadata.height,
        )?;
        normalize_motion_vectors(
            &mut motion_data,
            metadata.width,
            metadata.height,
            &manifest.contract.motion_vector_convention,
        )?;
        let motion_vectors = motion_data
            .iter()
            .map(|value| MotionVector {
                to_prev_x: value[0],
                to_prev_y: value[1],
            })
            .collect::<Vec<_>>();

        let (_, _, current_depth) = load_scalar_buffer_from_reference(
            base_dir,
            &frame.buffers.current_depth,
            metadata.width,
            metadata.height,
        )?;
        let (_, _, previous_depth) = load_scalar_buffer_from_reference(
            base_dir,
            &frame.buffers.previous_depth,
            metadata.width,
            metadata.height,
        )?;
        let (_, _, current_normal_data) = load_vec3_buffer_from_reference(
            base_dir,
            &frame.buffers.current_normals,
            metadata.width,
            metadata.height,
        )?;
        let (_, _, previous_normal_data) = load_vec3_buffer_from_reference(
            base_dir,
            &frame.buffers.previous_normals,
            metadata.width,
            metadata.height,
        )?;
        let current_normals = current_normal_data
            .into_iter()
            .map(|value| Normal3::new(value[0], value[1], value[2]).normalized())
            .collect::<Vec<_>>();
        let previous_normals = previous_normal_data
            .into_iter()
            .map(|value| Normal3::new(value[0], value[1], value[2]).normalized())
            .collect::<Vec<_>>();

        let history_color = match &frame.buffers.history_color {
            Some(reference) => load_color_buffer_from_reference(
                base_dir,
                reference,
                metadata.width,
                metadata.height,
            )?,
            None => reproject_image(&previous_color, &motion_vectors),
        };
        let history_depth = match &frame.buffers.history_depth {
            Some(reference) => load_scalar_buffer_from_reference(
                base_dir,
                reference,
                metadata.width,
                metadata.height,
            )?
            .2,
            None => reproject_scalar(&previous_depth, metadata.width, metadata.height, &motion_vectors),
        };
        let history_normals = match &frame.buffers.history_normals {
            Some(reference) => load_vec3_buffer_from_reference(
                base_dir,
                reference,
                metadata.width,
                metadata.height,
            )?
            .2
            .into_iter()
            .map(|value| Normal3::new(value[0], value[1], value[2]).normalized())
            .collect(),
            None => reproject_normals(
                &previous_normals,
                metadata.width,
                metadata.height,
                &motion_vectors,
            ),
        };

        let host_output = frame
            .buffers
            .host_output
            .as_ref()
            .map(|reference| {
                load_color_buffer_from_reference(base_dir, reference, metadata.width, metadata.height)
            })
            .transpose()?;
        let reference_color = frame
            .buffers
            .reference_color
            .as_ref()
            .map(|reference| {
                load_color_buffer_from_reference(base_dir, reference, metadata.width, metadata.height)
            })
            .transpose()?;
        let roi_mask = frame
            .buffers
            .roi_mask
            .as_ref()
            .map(|reference| load_mask_any(base_dir, reference, metadata.width, metadata.height))
            .transpose()?;
        let disocclusion_mask = frame
            .buffers
            .disocclusion_mask
            .as_ref()
            .map(|reference| load_mask_any(base_dir, reference, metadata.width, metadata.height))
            .transpose()?;

        let capture_dir = materialized_dir.join(&frame.label);
        fs::create_dir_all(&capture_dir)?;

        let current_color_path = capture_dir.join("current_color.png");
        let history_color_path = capture_dir.join("reprojected_history.png");
        let motion_vectors_path = capture_dir.join("motion_vectors.json");
        let current_depth_path = capture_dir.join("current_depth.json");
        let history_depth_path = capture_dir.join("reprojected_depth.json");
        let current_normals_path = capture_dir.join("current_normals.json");
        let history_normals_path = capture_dir.join("reprojected_normals.json");
        let metadata_path = capture_dir.join("metadata.json");
        let roi_mask_path = capture_dir.join("roi_mask.json");
        let reference_path = capture_dir.join("reference_color.png");

        current_color.save_png(&current_color_path)?;
        history_color.save_png(&history_color_path)?;
        write_json(&motion_vectors_path, &Vec2Json {
            width: metadata.width,
            height: metadata.height,
            data: motion_vectors
                .iter()
                .map(|motion| [motion.to_prev_x, motion.to_prev_y])
                .collect(),
        })?;
        write_json(&current_depth_path, &ScalarJson {
            width: metadata.width,
            height: metadata.height,
            data: current_depth.clone(),
        })?;
        write_json(&history_depth_path, &ScalarJson {
            width: metadata.width,
            height: metadata.height,
            data: history_depth.clone(),
        })?;
        write_json(&current_normals_path, &Vec3Json {
            width: metadata.width,
            height: metadata.height,
            data: current_normals.iter().map(|normal| [normal.x, normal.y, normal.z]).collect(),
        })?;
        write_json(&history_normals_path, &Vec3Json {
            width: metadata.width,
            height: metadata.height,
            data: history_normals.iter().map(|normal| [normal.x, normal.y, normal.z]).collect(),
        })?;

        let external_metadata = ExternalCaptureMetadata {
            scenario_id: metadata.scene_name.clone(),
            frame_index: metadata.frame_index,
            history_frame_index: metadata.history_frame_index,
            width: metadata.width,
            height: metadata.height,
            source_kind: UNREAL_NATIVE_DATASET_KIND.to_string(),
            externally_validated: true,
            real_external_data: true,
            data_description: Some(
                "Unreal Engine exported frame pair materialized into the DSFB external replay contract"
                    .to_string(),
            ),
            notes: metadata.notes.clone(),
        };
        write_json(&metadata_path, &external_metadata)?;

        let optional_mask = if let Some(mask) = &roi_mask {
            write_json(&roi_mask_path, &BoolJson {
                width: metadata.width,
                height: metadata.height,
                data: mask.clone(),
            })?;
            Some(BufferReference {
                path: relative_path_string(&roi_mask_path, run_dir),
                format: "json_mask_bool".to_string(),
                semantic: "roi_mask".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(1),
            })
        } else {
            None
        };

        let optional_reference = if let Some(reference) = &reference_color {
            reference.save_png(&reference_path)?;
            Some(BufferReference {
                path: relative_path_string(&reference_path, run_dir),
                format: "png_rgb8".to_string(),
                semantic: "reference_color".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(3),
            })
        } else {
            None
        };

        let buffers = ExternalBufferSet {
            current_color: BufferReference {
                path: relative_path_string(&current_color_path, run_dir),
                format: "png_rgb8".to_string(),
                semantic: "current_color".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(3),
            },
            reprojected_history: BufferReference {
                path: relative_path_string(&history_color_path, run_dir),
                format: "png_rgb8".to_string(),
                semantic: "reprojected_history".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(3),
            },
            motion_vectors: BufferReference {
                path: relative_path_string(&motion_vectors_path, run_dir),
                format: "json_vec2_f32".to_string(),
                semantic: "motion_vectors".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(2),
            },
            current_depth: BufferReference {
                path: relative_path_string(&current_depth_path, run_dir),
                format: "json_scalar_f32".to_string(),
                semantic: "current_depth".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(1),
            },
            reprojected_depth: BufferReference {
                path: relative_path_string(&history_depth_path, run_dir),
                format: "json_scalar_f32".to_string(),
                semantic: "reprojected_depth".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(1),
            },
            current_normals: BufferReference {
                path: relative_path_string(&current_normals_path, run_dir),
                format: "json_vec3_f32".to_string(),
                semantic: "current_normals".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(3),
            },
            reprojected_normals: BufferReference {
                path: relative_path_string(&history_normals_path, run_dir),
                format: "json_vec3_f32".to_string(),
                semantic: "reprojected_normals".to_string(),
                width: Some(metadata.width),
                height: Some(metadata.height),
                channels: Some(3),
            },
            metadata: BufferReference {
                path: relative_path_string(&metadata_path, run_dir),
                format: "json_metadata".to_string(),
                semantic: "metadata".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            optional_mask,
            optional_reference,
            optional_ground_truth: None,
            optional_variance: None,
        };

        external_entries.push(ExternalCaptureEntry {
            label: frame.label.clone(),
            buffers,
        });

        captures.push(MaterializedCapture {
            label: frame.label.clone(),
            frame_index: frame.frame_index,
            scene_name: metadata
                .scene_name
                .clone()
                .unwrap_or_else(|| manifest.dataset_id.clone()),
            shot_name: metadata
                .shot_name
                .clone()
                .unwrap_or_else(|| "shot_000".to_string()),
            baseline_source: if host_output.is_some() {
                "exported_host_output".to_string()
            } else {
                "strong_heuristic_baseline".to_string()
            },
            host_output,
            roi_mask,
            disocclusion_mask,
        });
    }

    let external_manifest = ExternalCaptureManifest {
        format_version: EXTERNAL_CAPTURE_FORMAT_VERSION.to_string(),
        description: format!(
            "Materialized external replay manifest generated from the strict Unreal-native dataset `{}`",
            manifest.dataset_id
        ),
        source: ExternalCaptureSource::EngineNative {
            engine_type: "unreal".to_string(),
            engine_version: Some(manifest.engine.engine_version.clone()),
            capture_tool: Some(manifest.engine.capture_tool.clone()),
            capture_note: Some(
                "real Unreal capture materialized into reprojected replay inputs with no synthetic fallback"
                    .to_string(),
            ),
        },
        buffers: None,
        captures: external_entries,
        normalization: ExternalNormalization {
            color: manifest.contract.color_space.clone(),
            motion_vectors: format!(
                "{}; normalized into pixel offsets to the previous frame",
                manifest.contract.motion_vector_convention
            ),
            depth: manifest.contract.depth_convention.clone(),
            normals: manifest.contract.normal_space.clone(),
        },
        notes: vec![
            "provenance_label=unreal_native".to_string(),
            "real_engine_capture=true".to_string(),
            "no synthetic fallback is implemented in this path".to_string(),
        ],
    };
    let materialized_manifest_path = run_dir.join("materialized_unreal_external_manifest.json");
    fs::write(
        &materialized_manifest_path,
        serde_json::to_string_pretty(&external_manifest)?,
    )?;

    Ok(MaterializedRun {
        materialized_manifest_path,
        captures,
    })
}

fn generate_per_frame_artifacts(
    config: &DemoConfig,
    materialized_manifest_path: &Path,
    materialized_captures: &[MaterializedCapture],
    demo_a: &ExternalDemoAMetrics,
    gpu: &ExternalGpuMetrics,
    per_frame_dir: &Path,
) -> Result<Vec<FrameArtifacts>> {
    let bundle = crate::external::load_external_capture_bundle(
        config,
        materialized_manifest_path,
        per_frame_dir,
    )?;
    let profile =
        default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
    let gpu_by_label = gpu
        .captures
        .iter()
        .map(|capture| (capture.capture_label.clone(), capture.total_ms))
        .collect::<BTreeMap<_, _>>();

    let mut frames = Vec::with_capacity(bundle.captures.len());
    for capture in &bundle.captures {
        let materialized = materialized_captures
            .iter()
            .find(|candidate| candidate.label == capture.label)
            .ok_or_else(|| {
                Error::Message(format!(
                    "materialized capture `{}` was missing during per-frame artifact generation",
                    capture.label
                ))
            })?;
        let methods = find_demo_a_methods(demo_a, &capture.label)?;
        let outputs = supervise_temporal_reuse(&capture.inputs.borrow(), &profile);
        let dsfb_resolved = resolve_with_alpha(
            &capture.inputs.reprojected_history,
            &capture.inputs.current_color,
            &outputs.alpha,
        );
        let (strong_resolved, _, strong_response) = run_strong_heuristic(config, capture);
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
        let roi_mask = materialized
            .roi_mask
            .clone()
            .unwrap_or_else(|| derive_roi_mask(&outputs));
        let instability_mask = materialized
            .disocclusion_mask
            .clone()
            .unwrap_or_else(|| derive_instability_mask(&outputs, &capture.inputs.current_color, &capture.inputs.reprojected_history));
        let baseline = materialized
            .host_output
            .clone()
            .unwrap_or_else(|| strong_resolved.clone());

        let capture_dir = per_frame_dir.join(&capture.label);
        fs::create_dir_all(&capture_dir)?;
        let current_frame_path = capture_dir.join("current_frame.png");
        let baseline_frame_path = capture_dir.join("baseline_or_host_output.png");
        let trust_map_path = capture_dir.join("trust_map.png");
        let alpha_map_path = capture_dir.join("alpha_map.png");
        let intervention_map_path = capture_dir.join("intervention_map.png");
        let residual_map_path = capture_dir.join("residual_map.png");
        let instability_overlay_path = capture_dir.join("instability_overlay.png");
        let roi_overlay_path = capture_dir.join("roi_overlay.png");
        let output_panel_path = capture_dir.join(format!("boardroom_panel_{}.png", capture.label));

        capture.inputs.current_color.save_png(&current_frame_path)?;
        baseline.save_png(&baseline_frame_path)?;
        save_scalar_field_png(&outputs.trust, &trust_map_path, heatmap_blue)?;
        save_scalar_field_png(&outputs.alpha, &alpha_map_path, heatmap_orange)?;
        save_scalar_field_png(
            &outputs.intervention,
            &intervention_map_path,
            heatmap_red,
        )?;
        let residual_field =
            residual_field(&capture.inputs.current_color, &capture.inputs.reprojected_history);
        save_scalar_field_png(&residual_field, &residual_map_path, heatmap_residual)?;
        overlay_mask(
            &capture.inputs.current_color,
            &instability_mask,
            Color::rgb(1.0, 0.1, 0.1),
            0.45,
        )
        .save_png(&instability_overlay_path)?;
        overlay_mask(
            &capture.inputs.current_color,
            &roi_mask,
            Color::rgb(0.12, 1.0, 0.24),
            0.45,
        )
        .save_png(&roi_overlay_path)?;

        let classification = classify_capture(&methods);
        let roi_residual_mean = residual_field.mean_over_mask(&roi_mask);
        let instability_fraction = instability_mask
            .iter()
            .filter(|value| **value)
            .count() as f32
            / instability_mask.len().max(1) as f32;
        let explanation = build_explanation(
            &classification,
            materialized,
            &methods,
            roi_residual_mean,
            instability_fraction,
        );

        let baseline_gap = mean_abs_error_over_mask(&baseline, &dsfb_resolved, &roi_mask);
        let _ = baseline_gap;
        let _ = strong_response;
        let _ = fixed_resolved;

        frames.push(FrameArtifacts {
            label: capture.label.clone(),
            scene_name: materialized.scene_name.clone(),
            shot_name: materialized.shot_name.clone(),
            frame_index: materialized.frame_index,
            classification,
            current_frame_path,
            baseline_frame_path,
            trust_map_path,
            alpha_map_path,
            intervention_map_path,
            residual_map_path,
            instability_overlay_path,
            roi_overlay_path,
            output_panel_path,
            metric_source: methods.metric_source.to_string(),
            dsfb_roi_mae: methods.dsfb.roi_mae,
            strong_heuristic_roi_mae: methods.strong_heuristic.roi_mae,
            fixed_alpha_roi_mae: methods.fixed_alpha.roi_mae,
            dsfb_mean_trust: outputs.trust.mean(),
            dsfb_mean_alpha: outputs.alpha.mean(),
            dsfb_intervention_rate: outputs.intervention.mean(),
            roi_residual_mean,
            instability_fraction,
            gpu_total_ms: gpu_by_label.get(&capture.label).copied().flatten(),
            explanation,
        });
    }

    Ok(frames)
}

fn write_run_manifest(
    path: &Path,
    manifest_path: &Path,
    manifest: &UnrealNativeManifest,
    materialized: &MaterializedRun,
) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": manifest.schema_version,
        "dataset_kind": manifest.dataset_kind,
        "provenance_label": manifest.provenance_label,
        "dataset_id": manifest.dataset_id,
        "manifest_path": manifest_path.display().to_string(),
        "materialized_manifest_path": materialized.materialized_manifest_path.display().to_string(),
        "capture_count": materialized.captures.len(),
        "engine": manifest.engine,
        "contract": manifest.contract,
        "notes": manifest.notes,
    });
    fs::write(path, serde_json::to_string_pretty(&payload)?)?;
    Ok(())
}

fn classification_counts(frames: &[FrameArtifacts]) -> ClassificationCounts {
    let mut counts = ClassificationCounts::default();
    for frame in frames {
        match frame.classification.as_str() {
            "dsfb_helpful" => counts.dsfb_helpful += 1,
            "dsfb_neutral" => counts.dsfb_neutral += 1,
            "heuristic_favorable" => counts.heuristic_favorable += 1,
            _ => counts.richer_cues_required += 1,
        }
    }
    counts
}

fn write_metrics_csv(
    path: &Path,
    capture_summaries: &[CaptureSummaryRecord],
    demo_b: &ExternalDemoBMetrics,
) -> Result<()> {
    let mut csv = String::new();
    let _ = writeln!(
        csv,
        "record_type,capture_label,scene_name,shot_name,frame_index,classification,metric_source,dsfb_roi_mae,strong_heuristic_roi_mae,fixed_alpha_roi_mae,dsfb_mean_trust,dsfb_mean_alpha,dsfb_intervention_rate,roi_residual_mean,instability_fraction,gpu_total_ms"
    );
    for capture in capture_summaries {
        let _ = writeln!(
            csv,
            "demo_a,{},{},{},{},{},{},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{}",
            capture.capture_label,
            capture.scene_name,
            capture.shot_name,
            capture.frame_index,
            capture.classification,
            capture.metric_source,
            capture.dsfb_roi_mae,
            capture.strong_heuristic_roi_mae,
            capture.fixed_alpha_roi_mae,
            capture.dsfb_mean_trust,
            capture.dsfb_mean_alpha,
            capture.dsfb_intervention_rate,
            capture.roi_residual_mean,
            capture.instability_fraction,
            capture
                .gpu_total_ms
                .map(|value| format!("{value:.5}"))
                .unwrap_or_else(|| "n/a".to_string()),
        );
    }
    for capture in &demo_b.captures {
        for policy in &capture.policies {
            let _ = writeln!(
                csv,
                "demo_b,{capture_label},,,,{metric_source},{policy_id},{roi_mae:.5},,,,{roi_mean_spp:.5},{non_roi_mean_spp:.5},{overall_mae:.5},,",
                capture_label = capture.capture_label,
                metric_source = capture.metric_source,
                policy_id = policy.policy_id,
                roi_mae = policy.roi_mae,
                roi_mean_spp = policy.roi_mean_spp,
                non_roi_mean_spp = policy.non_roi_mean_spp,
                overall_mae = policy.overall_mae,
            );
        }
    }
    fs::write(path, csv)?;
    Ok(())
}

fn write_comparison_summary(
    path: &Path,
    manifest: &UnrealNativeManifest,
    frames: &[FrameArtifacts],
    demo_b: &ExternalDemoBMetrics,
) -> Result<()> {
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Unreal-Native Comparison Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Dataset `{}` is labeled `{}` and was executed through the strict Unreal-native replay path.",
        manifest.dataset_id, manifest.provenance_label
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Capture Classification");
    let _ = writeln!(markdown);
    for frame in frames {
        let _ = writeln!(
            markdown,
            "- `{}` ({}/{} frame {}): `{}`. DSFB ROI MAE = {:.5}, strong heuristic ROI MAE = {:.5}, fixed alpha ROI MAE = {:.5}.",
            frame.label,
            frame.scene_name,
            frame.shot_name,
            frame.frame_index,
            frame.classification,
            frame.dsfb_roi_mae,
            frame.strong_heuristic_roi_mae,
            frame.fixed_alpha_roi_mae
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Demo B Policy Posture");
    let _ = writeln!(markdown);
    for capture in &demo_b.captures {
        let imported = capture
            .policies
            .iter()
            .find(|policy| policy.policy_id == "imported_trust");
        let combined = capture
            .policies
            .iter()
            .find(|policy| policy.policy_id == "combined_heuristic");
        let uniform = capture
            .policies
            .iter()
            .find(|policy| policy.policy_id == "uniform");
        if let (Some(imported), Some(combined), Some(uniform)) = (imported, combined, uniform) {
            let winner = if imported.roi_mae + 1e-4 < combined.roi_mae {
                "DSFB-helpful allocation case"
            } else if (imported.roi_mae - combined.roi_mae).abs() <= 1e-4 {
                "DSFB-neutral allocation case"
            } else {
                "heuristic-favorable allocation case"
            };
            let _ = writeln!(
                markdown,
                "- `{}`: {}. Imported trust ROI error = {:.5}, combined heuristic ROI error = {:.5}, uniform ROI error = {:.5}.",
                capture.capture_label,
                winner,
                imported.roi_mae,
                combined.roi_mae,
                uniform.roi_mae
            );
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Boundaries");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This is evidence consistent with reduced temporal artifact risk in bounded cases, not a claim of universal outperformance."
    );
    let _ = writeln!(
        markdown,
        "- Demo B remains an advisory allocation proxy unless a live renderer budget path is exported."
    );
    let _ = writeln!(
        markdown,
        "- The crate is acting as a supervisory trust / admissibility / intervention layer, not a renderer replacement."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn write_failure_modes(
    path: &Path,
    manifest: &UnrealNativeManifest,
    captures: &[MaterializedCapture],
    scaling: &ExternalScalingMetrics,
) -> Result<()> {
    let missing_optional = captures
        .iter()
        .filter(|capture| capture.roi_mask.is_none() || capture.disocclusion_mask.is_none())
        .map(|capture| capture.label.clone())
        .collect::<Vec<_>>();
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Unreal-Native Failure Modes");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This file is first-class evidence for where the Unreal-native replay path should remain bounded or advisory."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Structural Limits");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Residual-only evidence weakens when the host output already tracks the current frame closely."
    );
    let _ = writeln!(
        markdown,
        "- Missing ROI or disocclusion masks force the run to derive overlays from the DSFB response, which is useful for triage but not a substitute for exported engine annotations."
    );
    let _ = writeln!(
        markdown,
        "- Transparency, particles, UI, post effects, and specular-only motion can violate the view-space normal and monotonic-depth assumptions."
    );
    let _ = writeln!(
        markdown,
        "- If motion vectors are noisy or encoded in a convention that does not match the manifest, the run fails rather than silently downgrading."
    );
    let _ = writeln!(
        markdown,
        "- Where a host heuristic already performs strongly, DSFB should be interpreted as a bounded monitor or advisory layer."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Export-Specific Notes");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Dataset `{}` uses motion_vector_convention = `{}` and history_source = `{}`.",
        manifest.dataset_id,
        manifest.contract.motion_vector_convention,
        manifest.contract.history_source
    );
    if !missing_optional.is_empty() {
        let _ = writeln!(
            markdown,
            "- Optional overlays were missing for: {}.",
            missing_optional.join(", ")
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Scaling Limits");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Scaling measurement kind: `{}`. Coverage status: `{}`.",
        scaling.measurement_kind, scaling.coverage.coverage_status
    );
    for entry in &scaling.entries {
        if let Some(reason) = &entry.unavailable_reason {
            let _ = writeln!(
                markdown,
                "- `{}` {}x{} unavailable: {}",
                entry.label, entry.width, entry.height, reason
            );
        }
    }
    fs::write(path, markdown)?;
    Ok(())
}

fn select_executive_frame<'a>(frames: &'a [FrameArtifacts]) -> Result<&'a FrameArtifacts> {
    frames
        .iter()
        .max_by(|left, right| {
            let left_gain = left.strong_heuristic_roi_mae - left.dsfb_roi_mae;
            let right_gain = right.strong_heuristic_roi_mae - right.dsfb_roi_mae;
            left_gain.total_cmp(&right_gain)
        })
        .ok_or_else(|| Error::Message("no per-frame artifacts were generated".to_string()))
}

fn run_bundle_builder(run_dir: &Path) -> Result<()> {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("colab")
        .join("build_unreal_native_bundle.py");
    let status = Command::new("python3")
        .arg(&script)
        .arg("--run-dir")
        .arg(run_dir)
        .status()
        .map_err(|error| {
            Error::Message(format!(
                "failed to launch unreal-native bundle builder {}: {error}",
                script.display()
            ))
        })?;
    if !status.success() {
        return Err(Error::Message(format!(
            "unreal-native bundle builder {} exited with status {}",
            script.display(),
            status
        )));
    }
    Ok(())
}

fn find_demo_a_methods<'a>(
    demo_a: &'a ExternalDemoAMetrics,
    capture_label: &str,
) -> Result<DemoAMethodSelection<'a>> {
    let capture = demo_a
        .captures
        .iter()
        .find(|capture| capture.capture_label == capture_label)
        .ok_or_else(|| {
            Error::Message(format!(
                "Demo A metrics did not contain capture `{capture_label}`"
            ))
        })?;
    let fixed_alpha = capture
        .methods
        .iter()
        .find(|method| method.method_id == "fixed_alpha")
        .ok_or_else(|| Error::Message(format!("capture `{capture_label}` missing fixed_alpha")))?;
    let strong_heuristic = capture
        .methods
        .iter()
        .find(|method| method.method_id == "strong_heuristic")
        .ok_or_else(|| {
            Error::Message(format!(
                "capture `{capture_label}` missing strong_heuristic"
            ))
        })?;
    let dsfb = capture
        .methods
        .iter()
        .find(|method| method.method_id == "dsfb_host_minimum")
        .ok_or_else(|| {
            Error::Message(format!(
                "capture `{capture_label}` missing dsfb_host_minimum"
            ))
        })?;
    Ok(DemoAMethodSelection {
        metric_source: &capture.metric_source,
        fixed_alpha,
        strong_heuristic,
        dsfb,
    })
}

fn classify_capture(methods: &DemoAMethodSelection<'_>) -> String {
    let epsilon = 1e-4;
    if methods.dsfb.roi_mae + epsilon
        < methods
            .strong_heuristic
            .roi_mae
            .min(methods.fixed_alpha.roi_mae)
    {
        "dsfb_helpful".to_string()
    } else if (methods.dsfb.roi_mae - methods.strong_heuristic.roi_mae).abs() <= epsilon {
        "dsfb_neutral".to_string()
    } else if methods.strong_heuristic.roi_mae + epsilon < methods.dsfb.roi_mae {
        "heuristic_favorable".to_string()
    } else {
        "richer_cues_required".to_string()
    }
}

fn build_explanation(
    classification: &str,
    materialized: &MaterializedCapture,
    methods: &DemoAMethodSelection<'_>,
    roi_residual_mean: f32,
    instability_fraction: f32,
) -> EvidenceExplanation {
    let what_went_wrong = format!(
        "Temporal reuse risk was concentrated in scene `{}` / shot `{}` frame {} with ROI residual {:.5} and instability coverage {:.3}.",
        materialized.scene_name, materialized.shot_name, materialized.frame_index, roi_residual_mean, instability_fraction
    );
    let what_dsfb_detected = format!(
        "DSFB concentrated low trust and intervention in the exported Unreal-native ROI, with ROI MAE {:.5} against strong heuristic ROI MAE {:.5}.",
        methods.dsfb.roi_mae, methods.strong_heuristic.roi_mae
    );
    let what_dsfb_changed = match classification {
        "dsfb_helpful" => "The supervisory layer would route this region toward higher alpha / intervention and away from blind temporal reuse.".to_string(),
        "dsfb_neutral" => "The supervisory layer agreed with the strongest host heuristic closely enough that this should be treated as a bounded monitor result, not a large behavioral delta.".to_string(),
        "heuristic_favorable" => "The strongest host heuristic outperformed DSFB on this frame; the evidence is surfaced directly rather than hidden.".to_string(),
        _ => "The current observability is not rich enough to claim a strong DSFB advantage on this frame.".to_string(),
    };
    let overhead_and_caveat = format!(
        "GPU measurement is advisory and environment-dependent; baseline source for this frame was `{}`. This is not a renderer replacement claim.",
        materialized.baseline_source
    );
    EvidenceExplanation {
        what_went_wrong,
        what_dsfb_detected,
        what_dsfb_changed,
        overhead_and_caveat,
    }
}

fn load_unreal_frame_metadata(base_dir: &Path, reference: &BufferReference) -> Result<UnrealFrameMetadata> {
    if reference.format != "json_metadata" {
        return Err(Error::Message(format!(
            "unreal-native metadata {} must use json_metadata format",
            reference.path
        )));
    }
    let path = resolve_path(base_dir, &reference.path);
    read_json(&path)
}

fn validate_frame_metadata(frame: &UnrealFrameEntry, metadata: &UnrealFrameMetadata) -> Result<()> {
    if metadata.width == 0 || metadata.height == 0 {
        return Err(Error::Message(format!(
            "capture `{}` declared zero-sized extent {}x{}",
            frame.label, metadata.width, metadata.height
        )));
    }
    if metadata.frame_index != frame.frame_index {
        return Err(Error::Message(format!(
            "capture `{}` manifest frame_index {} did not match metadata frame_index {}",
            frame.label, frame.frame_index, metadata.frame_index
        )));
    }
    if metadata.history_frame_index != frame.history_frame_index {
        return Err(Error::Message(format!(
            "capture `{}` manifest history_frame_index {} did not match metadata history_frame_index {}",
            frame.label, frame.history_frame_index, metadata.history_frame_index
        )));
    }
    if metadata.source_kind != UNREAL_NATIVE_DATASET_KIND {
        return Err(Error::Message(format!(
            "capture `{}` metadata source_kind `{}` is invalid; `{}` is required",
            frame.label, metadata.source_kind, UNREAL_NATIVE_DATASET_KIND
        )));
    }
    if metadata.provenance_label.as_deref() != Some(UNREAL_NATIVE_PROVENANCE_LABEL) {
        return Err(Error::Message(format!(
            "capture `{}` metadata provenance_label must be `{}`",
            frame.label, UNREAL_NATIVE_PROVENANCE_LABEL
        )));
    }
    if !metadata.real_external_data {
        return Err(Error::Message(format!(
            "capture `{}` metadata real_external_data=false; unreal-native mode refuses synthetic or proxy provenance",
            frame.label
        )));
    }
    Ok(())
}

fn normalize_motion_vectors(
    values: &mut [[f32; 2]],
    width: usize,
    height: usize,
    convention: &str,
) -> Result<()> {
    match convention {
        "pixel_offset_to_prev" => {}
        "ndc_to_prev" => {
            let scale_x = width as f32 / 2.0;
            let scale_y = height as f32 / 2.0;
            for value in values {
                value[0] *= scale_x;
                value[1] *= scale_y;
            }
        }
        other => {
            return Err(Error::Message(format!(
                "unsupported unreal-native motion vector convention `{other}`"
            )))
        }
    }
    Ok(())
}

fn reproject_image(previous: &ImageFrame, motion_vectors: &[MotionVector]) -> ImageFrame {
    let mut output = ImageFrame::new(previous.width(), previous.height());
    for y in 0..previous.height() {
        for x in 0..previous.width() {
            let motion = motion_vectors[y * previous.width() + x];
            output.set(
                x,
                y,
                previous.sample_bilinear_clamped(x as f32 + motion.to_prev_x, y as f32 + motion.to_prev_y),
            );
        }
    }
    output
}

fn reproject_scalar(
    previous: &[f32],
    width: usize,
    height: usize,
    motion_vectors: &[MotionVector],
) -> Vec<f32> {
    let mut output = vec![0.0; width * height];
    for y in 0..height {
        for x in 0..width {
            let motion = motion_vectors[y * width + x];
            output[y * width + x] = sample_scalar(previous, width, height, x as f32 + motion.to_prev_x, y as f32 + motion.to_prev_y);
        }
    }
    output
}

fn reproject_normals(
    previous: &[Normal3],
    width: usize,
    height: usize,
    motion_vectors: &[MotionVector],
) -> Vec<Normal3> {
    let mut output = vec![Normal3::new(0.0, 0.0, 1.0); width * height];
    for y in 0..height {
        for x in 0..width {
            let motion = motion_vectors[y * width + x];
            output[y * width + x] = sample_normal(previous, width, height, x as f32 + motion.to_prev_x, y as f32 + motion.to_prev_y);
        }
    }
    output
}

fn sample_scalar(values: &[f32], width: usize, height: usize, x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);
    let sample = |sx: f32, sy: f32| -> f32 {
        let ix = sx.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let iy = sy.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[iy * width + ix]
    };
    let top = sample(x0, y0) * (1.0 - tx) + sample(x1, y0) * tx;
    let bottom = sample(x0, y1) * (1.0 - tx) + sample(x1, y1) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn sample_normal(values: &[Normal3], width: usize, height: usize, x: f32, y: f32) -> Normal3 {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);
    let sample = |sx: f32, sy: f32| -> Normal3 {
        let ix = sx.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let iy = sy.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[iy * width + ix]
    };
    let lerp = |a: Normal3, b: Normal3, t: f32| {
        Normal3::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
        )
    };
    lerp(lerp(sample(x0, y0), sample(x1, y0), tx), lerp(sample(x0, y1), sample(x1, y1), tx), ty).normalized()
}

fn load_mask_any(
    base_dir: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<Vec<bool>> {
    match reference.format.as_str() {
        "json_mask_bool" | "raw_mask_u8" => {
            Ok(load_bool_buffer_from_reference(base_dir, reference, expected_width, expected_height)?.2)
        }
        "json_scalar_f32" | "raw_r32f" | "exr_r32f" => {
            let values =
                load_scalar_buffer_from_reference(base_dir, reference, expected_width, expected_height)?.2;
            Ok(values.into_iter().map(|value| value >= 0.5).collect())
        }
        other => Err(Error::Message(format!(
            "unsupported unreal-native mask format `{other}` for {}",
            reference.path
        ))),
    }
}

fn resolve_with_alpha(
    history: &ImageFrame,
    current: &ImageFrame,
    alpha: &ScalarField,
) -> ImageFrame {
    let mut output = ImageFrame::new(current.width(), current.height());
    for y in 0..current.height() {
        for x in 0..current.width() {
            output.set(
                x,
                y,
                history.get(x, y).lerp(current.get(x, y), alpha.get(x, y)),
            );
        }
    }
    output
}

fn run_strong_heuristic(
    config: &DemoConfig,
    capture: &crate::external::ExternalLoadedCapture,
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
            let clamped = clamp_to_current_neighborhood(&capture.inputs.current_color, history, x, y);
            let clamp_distance = clamped.abs_diff(history);
            let residual_gate = smoothstep(
                config.baseline.residual_threshold.low,
                config.baseline.residual_threshold.high,
                current.abs_diff(clamped),
            );
            let depth_gate = smoothstep(
                config.baseline.depth_disagreement.low,
                config.baseline.depth_disagreement.high,
                (capture.inputs.current_depth[index] - capture.inputs.reprojected_depth[index]).abs(),
            );
            let normal_gate = smoothstep(
                config.baseline.normal_disagreement.low,
                config.baseline.normal_disagreement.high,
                1.0 - capture.inputs.current_normals[index]
                    .dot(capture.inputs.reprojected_normals[index])
                    .clamp(-1.0, 1.0),
            );
            let neighborhood_gate = smoothstep(
                config.baseline.neighborhood_distance.low,
                config.baseline.neighborhood_distance.high,
                clamp_distance,
            );
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

fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let span = (high - low).max(1e-6);
    let t = ((value - low) / span).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn residual_field(current: &ImageFrame, history: &ImageFrame) -> ScalarField {
    let mut field = ScalarField::new(current.width(), current.height());
    for y in 0..current.height() {
        for x in 0..current.width() {
            field.set(x, y, (current.get(x, y).abs_diff(history.get(x, y)) / 0.25).clamp(0.0, 1.0));
        }
    }
    field
}

fn derive_roi_mask(outputs: &HostSupervisionOutputs) -> Vec<bool> {
    let width = outputs.trust.width();
    let height = outputs.trust.height();
    let total = width * height;
    let mut scores = (0..total)
        .map(|index| {
            let x = index % width;
            let y = index / width;
            outputs.intervention.get(x, y) * 0.60
                + outputs.proxies.depth_proxy.get(x, y) * 0.20
                + outputs.proxies.normal_proxy.get(x, y) * 0.10
                + outputs.proxies.neighborhood_proxy.get(x, y) * 0.10
        })
        .collect::<Vec<_>>();
    let mut sorted = scores.clone();
    sorted.sort_by(|left, right| right.total_cmp(left));
    let keep = (total / 12).max(1).min(total.max(1));
    let threshold = sorted[keep.saturating_sub(1)];
    scores.drain(..).map(|score| score >= threshold).collect()
}

fn derive_instability_mask(
    outputs: &HostSupervisionOutputs,
    current: &ImageFrame,
    history: &ImageFrame,
) -> Vec<bool> {
    let residual = residual_field(current, history);
    let mut mask = vec![false; current.width() * current.height()];
    for y in 0..current.height() {
        for x in 0..current.width() {
            let index = y * current.width() + x;
            mask[index] = outputs.intervention.get(x, y) > 0.45
                || outputs.trust.get(x, y) < 0.35
                || residual.get(x, y) > 0.35;
        }
    }
    mask
}

fn overlay_mask(frame: &ImageFrame, mask: &[bool], overlay: Color, strength: f32) -> ImageFrame {
    let mut output = frame.clone();
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            let index = y * frame.width() + x;
            if mask[index] {
                output.set(x, y, frame.get(x, y).lerp(overlay, strength));
            }
        }
    }
    output
}

fn constant_field(width: usize, height: usize, value: f32) -> ScalarField {
    ScalarField::from_values(width, height, vec![value; width * height])
}

fn resolve_path(base_dir: &Path, relative_or_absolute: &str) -> PathBuf {
    let candidate = Path::new(relative_or_absolute);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        base_dir.join(candidate)
    }
}

fn relative_path_string(path: &Path, base_dir: &Path) -> String {
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn git_commit_hash() -> Option<String> {
    Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
struct ScalarJson {
    width: usize,
    height: usize,
    data: Vec<f32>,
}

#[derive(Clone, Debug, Serialize)]
struct Vec2Json {
    width: usize,
    height: usize,
    data: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize)]
struct Vec3Json {
    width: usize,
    height: usize,
    data: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, Serialize)]
struct BoolJson {
    width: usize,
    height: usize,
    data: Vec<bool>,
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

fn heatmap_residual(value: f32) -> [u8; 4] {
    let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [255, v / 3, 16, 255]
}
