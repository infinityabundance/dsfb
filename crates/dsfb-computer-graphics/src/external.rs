use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::DemoConfig;
use crate::error::{Error, Result};
use crate::frame::{save_scalar_field_png, Color, ImageFrame, ScalarField};
use crate::host::{
    default_host_realistic_profile, supervise_temporal_reuse, HostSupervisionOutputs,
    HostTemporalInputs,
};
use crate::report::EXPERIMENT_SENTENCE;
use crate::scene::{
    generate_sequence_for_definition, scenario_by_id, MotionVector, Normal3, ScenarioId,
    SceneFrame, SceneSequence, SurfaceTag,
};
use crate::taa::run_fixed_alpha_baseline;

pub const EXTERNAL_CAPTURE_FORMAT_VERSION: &str = "dsfb_external_capture_v1";

#[derive(Clone, Debug)]
pub struct OwnedHostTemporalInputs {
    pub current_color: ImageFrame,
    pub reprojected_history: ImageFrame,
    pub motion_vectors: Vec<MotionVector>,
    pub current_depth: Vec<f32>,
    pub reprojected_depth: Vec<f32>,
    pub current_normals: Vec<Normal3>,
    pub reprojected_normals: Vec<Normal3>,
    pub visibility_hint: Option<Vec<bool>>,
    pub thin_hint: Option<ScalarField>,
}

impl OwnedHostTemporalInputs {
    pub fn width(&self) -> usize {
        self.current_color.width()
    }

    pub fn height(&self) -> usize {
        self.current_color.height()
    }

    pub fn borrow(&self) -> HostTemporalInputs<'_> {
        HostTemporalInputs {
            current_color: &self.current_color,
            reprojected_history: &self.reprojected_history,
            motion_vectors: &self.motion_vectors,
            current_depth: &self.current_depth,
            reprojected_depth: &self.reprojected_depth,
            current_normals: &self.current_normals,
            reprojected_normals: &self.reprojected_normals,
            visibility_hint: self.visibility_hint.as_deref(),
            thin_hint: self.thin_hint.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferReference {
    pub path: String,
    pub format: String,
    pub semantic: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalBufferSet {
    pub current_color: BufferReference,
    pub reprojected_history: BufferReference,
    pub motion_vectors: BufferReference,
    pub current_depth: BufferReference,
    pub reprojected_depth: BufferReference,
    pub current_normals: BufferReference,
    pub reprojected_normals: BufferReference,
    pub metadata: BufferReference,
    pub optional_mask: Option<BufferReference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExternalCaptureSource {
    Files,
    SyntheticCompat {
        scenario_id: String,
        frame_index: Option<usize>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalNormalization {
    pub color: String,
    pub motion_vectors: String,
    pub depth: String,
    pub normals: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalCaptureManifest {
    pub format_version: String,
    pub description: String,
    pub source: ExternalCaptureSource,
    pub buffers: ExternalBufferSet,
    pub normalization: ExternalNormalization,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ScalarBufferFile {
    width: usize,
    height: usize,
    data: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Vec2BufferFile {
    width: usize,
    height: usize,
    data: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Vec3BufferFile {
    width: usize,
    height: usize,
    data: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BoolBufferFile {
    width: usize,
    height: usize,
    data: Vec<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ColorBufferFile {
    width: usize,
    height: usize,
    data: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalCaptureMetadata {
    pub scenario_id: Option<String>,
    pub frame_index: usize,
    pub history_frame_index: usize,
    pub width: usize,
    pub height: usize,
    pub source_kind: String,
    pub externally_validated: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalHandoffMetrics {
    pub measurement_kind: String,
    pub external_capable: bool,
    pub externally_validated: bool,
    pub source_kind: String,
    pub scenario_id: Option<String>,
    pub frame_index: usize,
    pub history_frame_index: usize,
    pub width: usize,
    pub height: usize,
    pub imported_formats: Vec<String>,
    pub required_buffers: Vec<String>,
    pub normalization_notes: Vec<String>,
    pub mean_trust: f32,
    pub mean_alpha: f32,
    pub intervention_rate: f32,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ExternalImportArtifacts {
    pub report_path: PathBuf,
    pub metrics: ExternalHandoffMetrics,
    pub resolved_manifest_path: PathBuf,
}

pub fn example_manifest() -> ExternalCaptureManifest {
    ExternalCaptureManifest {
        format_version: EXTERNAL_CAPTURE_FORMAT_VERSION.to_string(),
        description: "Synthetic compatibility example that exports one frame pair into the stable external buffer schema and re-imports it through the same file-based path.".to_string(),
        source: ExternalCaptureSource::SyntheticCompat {
            scenario_id: "motion_bias_band".to_string(),
            frame_index: None,
        },
        buffers: ExternalBufferSet {
            current_color: BufferReference {
                path: "external_capture/current_color.png".to_string(),
                format: "png_rgb8".to_string(),
                semantic: "current color, normalized [0,1] RGB".to_string(),
            },
            reprojected_history: BufferReference {
                path: "external_capture/reprojected_history.png".to_string(),
                format: "png_rgb8".to_string(),
                semantic: "reprojected history color, normalized [0,1] RGB".to_string(),
            },
            motion_vectors: BufferReference {
                path: "external_capture/motion_vectors.json".to_string(),
                format: "json_vec2_f32".to_string(),
                semantic: "per-pixel motion vector to previous frame in pixel units".to_string(),
            },
            current_depth: BufferReference {
                path: "external_capture/current_depth.json".to_string(),
                format: "json_scalar_f32".to_string(),
                semantic: "current frame depth".to_string(),
            },
            reprojected_depth: BufferReference {
                path: "external_capture/reprojected_depth.json".to_string(),
                format: "json_scalar_f32".to_string(),
                semantic: "reprojected depth from previous frame".to_string(),
            },
            current_normals: BufferReference {
                path: "external_capture/current_normals.json".to_string(),
                format: "json_vec3_f32".to_string(),
                semantic: "current frame normals in view space, unit length".to_string(),
            },
            reprojected_normals: BufferReference {
                path: "external_capture/reprojected_normals.json".to_string(),
                format: "json_vec3_f32".to_string(),
                semantic: "reprojected normals from previous frame".to_string(),
            },
            metadata: BufferReference {
                path: "external_capture/metadata.json".to_string(),
                format: "json_metadata".to_string(),
                semantic: "capture metadata and provenance".to_string(),
            },
            optional_mask: Some(BufferReference {
                path: "external_capture/optional_mask.json".to_string(),
                format: "json_mask_bool".to_string(),
                semantic: "optional ROI-like disclosure or debug mask".to_string(),
            }),
        },
        normalization: ExternalNormalization {
            color: "linear RGB in [0,1]".to_string(),
            motion_vectors: "pixel offsets to the previous frame; positive x samples from a pixel further right in history".to_string(),
            depth: "monotonic depth with larger disagreement indicating less trust".to_string(),
            normals: "unit vectors in a consistent view-space basis".to_string(),
        },
        notes: vec![
            "Switch source.kind from synthetic_compat to files when real engine exports are available.".to_string(),
            "The example capture is external-capable but not externally validated.".to_string(),
        ],
    }
}

pub fn write_example_manifest(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&example_manifest())?)?;
    Ok(())
}

pub fn build_owned_inputs_from_sequence(
    sequence: &SceneSequence,
    frame_index: usize,
    previous_history: Option<&ImageFrame>,
) -> Result<OwnedHostTemporalInputs> {
    let frame_index = frame_index.min(sequence.frames.len().saturating_sub(1));
    if frame_index == 0 {
        return Err(Error::Message(
            "external capture requires a frame index after the first frame".to_string(),
        ));
    }

    let scene_frame = &sequence.frames[frame_index];
    let previous_scene = &sequence.frames[frame_index - 1];
    let history_source = previous_history.unwrap_or(&previous_scene.ground_truth);
    let reprojected_history = reproject_frame(history_source, scene_frame);
    let reprojected_depth = reproject_depth(previous_scene, scene_frame);
    let reprojected_normals = reproject_normals(previous_scene, scene_frame);

    Ok(OwnedHostTemporalInputs {
        current_color: scene_frame.ground_truth.clone(),
        reprojected_history,
        motion_vectors: scene_frame.motion.clone(),
        current_depth: scene_frame.depth.clone(),
        reprojected_depth,
        current_normals: scene_frame.normals.clone(),
        reprojected_normals,
        visibility_hint: None,
        thin_hint: None,
    })
}

pub fn run_external_import_from_manifest(
    config: &DemoConfig,
    manifest_path: &Path,
    output_dir: &Path,
) -> Result<ExternalImportArtifacts> {
    fs::create_dir_all(output_dir)?;
    let manifest_text = fs::read_to_string(manifest_path)?;
    let manifest: ExternalCaptureManifest = serde_json::from_str(&manifest_text)?;
    if manifest.format_version != EXTERNAL_CAPTURE_FORMAT_VERSION {
        return Err(Error::Message(format!(
            "unsupported external capture format version {}",
            manifest.format_version
        )));
    }

    let (resolved_manifest, inputs, metadata) =
        resolve_manifest_and_load_inputs(config, manifest_path, output_dir, manifest)?;
    let resolved_manifest_path = output_dir.join("resolved_external_capture_manifest.json");
    fs::write(
        &resolved_manifest_path,
        serde_json::to_string_pretty(&resolved_manifest)?,
    )?;

    let profile =
        default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
    let outputs = supervise_temporal_reuse(&inputs.borrow(), &profile);
    write_external_outputs(output_dir, &inputs, &outputs)?;

    let metrics = ExternalHandoffMetrics {
        measurement_kind: "external_buffer_import".to_string(),
        external_capable: true,
        externally_validated: metadata.externally_validated,
        source_kind: metadata.source_kind.clone(),
        scenario_id: metadata.scenario_id.clone(),
        frame_index: metadata.frame_index,
        history_frame_index: metadata.history_frame_index,
        width: metadata.width,
        height: metadata.height,
        imported_formats: vec![
            resolved_manifest.buffers.current_color.format.clone(),
            resolved_manifest.buffers.reprojected_history.format.clone(),
            resolved_manifest.buffers.motion_vectors.format.clone(),
            resolved_manifest.buffers.current_depth.format.clone(),
            resolved_manifest.buffers.current_normals.format.clone(),
        ],
        required_buffers: vec![
            "current_color".to_string(),
            "reprojected_history".to_string(),
            "motion_vectors".to_string(),
            "current_depth".to_string(),
            "reprojected_depth".to_string(),
            "current_normals".to_string(),
            "reprojected_normals".to_string(),
        ],
        normalization_notes: vec![
            resolved_manifest.normalization.color.clone(),
            resolved_manifest.normalization.motion_vectors.clone(),
            resolved_manifest.normalization.depth.clone(),
            resolved_manifest.normalization.normals.clone(),
        ],
        mean_trust: outputs.trust.mean(),
        mean_alpha: outputs.alpha.mean(),
        intervention_rate: outputs.intervention.mean(),
        notes: metadata.notes.clone(),
    };

    let report_path = output_dir.join("external_handoff_report.md");
    write_external_handoff_report(&report_path, &metrics, &resolved_manifest)?;

    Ok(ExternalImportArtifacts {
        report_path,
        metrics,
        resolved_manifest_path,
    })
}

fn resolve_manifest_and_load_inputs(
    config: &DemoConfig,
    manifest_path: &Path,
    output_dir: &Path,
    manifest: ExternalCaptureManifest,
) -> Result<(ExternalCaptureManifest, OwnedHostTemporalInputs, ExternalCaptureMetadata)> {
    match &manifest.source {
        ExternalCaptureSource::Files => {
            let base_dir = manifest_path
                .parent()
                .ok_or_else(|| Error::Message("manifest path had no parent directory".to_string()))?;
            let inputs = load_owned_inputs(&manifest, base_dir)?;
            let metadata = load_metadata(base_dir, &manifest.buffers.metadata, false)?;
            Ok((manifest, inputs, metadata))
        }
        ExternalCaptureSource::SyntheticCompat {
            scenario_id,
            frame_index,
        } => {
            let scenario_id = parse_scenario_id(scenario_id)?;
            let definition = scenario_by_id(&config.scene, scenario_id).ok_or_else(|| {
                Error::Message(format!("synthetic compat scenario {scenario_id:?} not found"))
            })?;
            let sequence = generate_sequence_for_definition(&definition);
            let export_frame_index = frame_index.unwrap_or(
                definition
                    .onset_frame
                    .min(sequence.frames.len().saturating_sub(1)),
            );
            let fixed_alpha = run_fixed_alpha_baseline(&sequence, config.baseline.fixed_alpha);
            let previous_history = fixed_alpha.taa.resolved_frames.get(export_frame_index - 1);
            let inputs =
                build_owned_inputs_from_sequence(&sequence, export_frame_index, previous_history)?;
            let capture_dir = output_dir;
            let resolved_manifest = materialize_capture_bundle(
                &manifest,
                capture_dir,
                &inputs,
                &ExternalCaptureMetadata {
                    scenario_id: Some(sequence.scenario_id.as_str().to_string()),
                    frame_index: export_frame_index,
                    history_frame_index: export_frame_index.saturating_sub(1),
                    width: inputs.width(),
                    height: inputs.height(),
                    source_kind: "synthetic_compat".to_string(),
                    externally_validated: false,
                    notes: vec![
                        "The example external capture was generated from the crate's deterministic synthetic suite.".to_string(),
                        "Replace source.kind with files and point the buffer paths at real engine exports to move beyond synthetic compatibility.".to_string(),
                    ],
                },
            )?;
            let loaded = load_owned_inputs(&resolved_manifest, capture_dir)?;
            let metadata = load_metadata(capture_dir, &resolved_manifest.buffers.metadata, false)?;
            Ok((resolved_manifest, loaded, metadata))
        }
    }
}

fn materialize_capture_bundle(
    manifest: &ExternalCaptureManifest,
    base_dir: &Path,
    inputs: &OwnedHostTemporalInputs,
    metadata: &ExternalCaptureMetadata,
) -> Result<ExternalCaptureManifest> {
    write_color_buffer(
        base_dir,
        &manifest.buffers.current_color,
        &inputs.current_color,
    )?;
    write_color_buffer(
        base_dir,
        &manifest.buffers.reprojected_history,
        &inputs.reprojected_history,
    )?;
    write_vec2_buffer(
        base_dir,
        &manifest.buffers.motion_vectors,
        &inputs.motion_vectors,
        inputs.width(),
        inputs.height(),
    )?;
    write_scalar_buffer(
        base_dir,
        &manifest.buffers.current_depth,
        &inputs.current_depth,
        inputs.width(),
        inputs.height(),
    )?;
    write_scalar_buffer(
        base_dir,
        &manifest.buffers.reprojected_depth,
        &inputs.reprojected_depth,
        inputs.width(),
        inputs.height(),
    )?;
    write_vec3_buffer(
        base_dir,
        &manifest.buffers.current_normals,
        &inputs.current_normals,
        inputs.width(),
        inputs.height(),
    )?;
    write_vec3_buffer(
        base_dir,
        &manifest.buffers.reprojected_normals,
        &inputs.reprojected_normals,
        inputs.width(),
        inputs.height(),
    )?;
    if let Some(mask_ref) = &manifest.buffers.optional_mask {
        write_bool_buffer(
            base_dir,
            mask_ref,
            &vec![false; inputs.width() * inputs.height()],
            inputs.width(),
            inputs.height(),
        )?;
    }
    let metadata_path = base_dir.join(&manifest.buffers.metadata.path);
    if let Some(parent) = metadata_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&metadata_path, serde_json::to_string_pretty(metadata)?)?;
    Ok(manifest.clone())
}

fn write_external_outputs(
    output_dir: &Path,
    inputs: &OwnedHostTemporalInputs,
    outputs: &HostSupervisionOutputs,
) -> Result<()> {
    inputs
        .current_color
        .save_png(&output_dir.join("external_current_color.png"))?;
    inputs
        .reprojected_history
        .save_png(&output_dir.join("external_reprojected_history.png"))?;
    save_scalar_field_png(&outputs.trust, &output_dir.join("external_trust.png"), |value| {
        let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        [v, v, 255, 255]
    })?;
    save_scalar_field_png(&outputs.alpha, &output_dir.join("external_alpha.png"), |value| {
        let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        [255, v, 0, 255]
    })?;
    save_scalar_field_png(
        &outputs.intervention,
        &output_dir.join("external_intervention.png"),
        |value| {
            let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            [255, 32, v, 255]
        },
    )?;
    Ok(())
}

fn write_external_handoff_report(
    path: &Path,
    metrics: &ExternalHandoffMetrics,
    manifest: &ExternalCaptureManifest,
) -> Result<()> {
    let mut markdown = String::new();
    markdown.push_str("# External Handoff Report\n\n");
    markdown.push_str(EXPERIMENT_SENTENCE);
    markdown.push_str("\n\n");
    markdown.push_str("This report covers the file-based external buffer import path. It demonstrates that the crate is external-capable, not externally validated.\n\n");
    markdown.push_str(&format!(
        "Source kind: `{}`. Externally validated: `{}`.\n\n",
        metrics.source_kind, metrics.externally_validated
    ));
    markdown.push_str("## Required Buffers\n\n");
    for buffer in &metrics.required_buffers {
        markdown.push_str(&format!("- `{buffer}`\n"));
    }
    markdown.push_str("\n## Accepted Formats\n\n");
    markdown.push_str("- `png_rgb8`\n");
    markdown.push_str("- `json_rgb_f32`\n");
    markdown.push_str("- `json_scalar_f32`\n");
    markdown.push_str("- `json_vec2_f32`\n");
    markdown.push_str("- `json_vec3_f32`\n");
    markdown.push_str("- `json_mask_bool`\n");
    markdown.push_str("- `json_metadata`\n\n");
    markdown.push_str("## Normalization Conventions\n\n");
    for note in &metrics.normalization_notes {
        markdown.push_str(&format!("- {note}\n"));
    }
    markdown.push_str("\n## Imported Capture Summary\n\n");
    markdown.push_str(&format!(
        "- Resolution: {}x{}\n- Frame index: {}\n- History frame index: {}\n- Mean trust: {:.4}\n- Mean alpha: {:.4}\n- Mean intervention: {:.4}\n",
        metrics.width,
        metrics.height,
        metrics.frame_index,
        metrics.history_frame_index,
        metrics.mean_trust,
        metrics.mean_alpha,
        metrics.intervention_rate
    ));
    markdown.push_str("\n## How An Engine Team Would Use This\n\n");
    markdown.push_str("- Export one frame pair using the buffer names and normalization described in the manifest.\n");
    markdown.push_str("- Set `source.kind` to `files` and point the buffer paths at the exported assets.\n");
    markdown.push_str("- Run `cargo run --release -- import-external --manifest <manifest> --output <dir>`.\n");
    markdown.push_str("- Inspect `external_trust.png`, `external_alpha.png`, and `external_intervention.png` plus the generated report.\n\n");
    markdown.push_str("## What Is Not Proven\n\n");
    markdown.push_str("- This report does not claim any real engine capture has been validated unless the metadata says so.\n");
    markdown.push_str("- The example manifest included in the crate is synthetic compatibility data, not field data.\n\n");
    markdown.push_str("## Remaining Blockers\n\n");
    markdown.push_str("- A real renderer still needs to export buffers into this schema.\n");
    markdown.push_str("- Real production captures and engine motion vectors are still required for external validation.\n");
    markdown.push_str("- GPU measurements on imported captures remain future work.\n\n");
    markdown.push_str("## Manifest Notes\n\n");
    for note in &manifest.notes {
        markdown.push_str(&format!("- {note}\n"));
    }
    fs::write(path, markdown)?;
    Ok(())
}

fn load_owned_inputs(manifest: &ExternalCaptureManifest, base_dir: &Path) -> Result<OwnedHostTemporalInputs> {
    let current_color = load_color_buffer(base_dir, &manifest.buffers.current_color)?;
    let reprojected_history = load_color_buffer(base_dir, &manifest.buffers.reprojected_history)?;
    let motion_vectors = load_vec2_buffer(base_dir, &manifest.buffers.motion_vectors)?;
    let current_depth = load_scalar_buffer(base_dir, &manifest.buffers.current_depth)?;
    let reprojected_depth = load_scalar_buffer(base_dir, &manifest.buffers.reprojected_depth)?;
    let current_normals = load_vec3_buffer(base_dir, &manifest.buffers.current_normals)?;
    let reprojected_normals = load_vec3_buffer(base_dir, &manifest.buffers.reprojected_normals)?;
    let optional_mask = manifest
        .buffers
        .optional_mask
        .as_ref()
        .map(|reference| load_bool_buffer(base_dir, reference))
        .transpose()?;
    Ok(OwnedHostTemporalInputs {
        current_color,
        reprojected_history,
        motion_vectors,
        current_depth,
        reprojected_depth,
        current_normals,
        reprojected_normals,
        visibility_hint: optional_mask,
        thin_hint: None,
    })
}

fn load_metadata(
    base_dir: &Path,
    reference: &BufferReference,
    externally_validated: bool,
) -> Result<ExternalCaptureMetadata> {
    let path = base_dir.join(&reference.path);
    let text = fs::read_to_string(path)?;
    let mut metadata: ExternalCaptureMetadata = serde_json::from_str(&text)?;
    metadata.externally_validated = externally_validated || metadata.externally_validated;
    Ok(metadata)
}

fn load_color_buffer(base_dir: &Path, reference: &BufferReference) -> Result<ImageFrame> {
    let path = base_dir.join(&reference.path);
    match reference.format.as_str() {
        "png_rgb8" => ImageFrame::load_png(&path),
        "json_rgb_f32" => {
            let file: ColorBufferFile = serde_json::from_str(&fs::read_to_string(path)?)?;
            Ok(ImageFrame::from_pixels(
                file.width,
                file.height,
                file.data
                    .into_iter()
                    .map(|rgb| Color::rgb(rgb[0], rgb[1], rgb[2]))
                    .collect(),
            ))
        }
        other => Err(Error::Message(format!(
            "unsupported color buffer format {other}"
        ))),
    }
}

fn load_scalar_buffer(base_dir: &Path, reference: &BufferReference) -> Result<Vec<f32>> {
    let path = base_dir.join(&reference.path);
    let file: ScalarBufferFile = serde_json::from_str(&fs::read_to_string(path)?)?;
    Ok(file.data)
}

fn load_vec2_buffer(base_dir: &Path, reference: &BufferReference) -> Result<Vec<MotionVector>> {
    let path = base_dir.join(&reference.path);
    let file: Vec2BufferFile = serde_json::from_str(&fs::read_to_string(path)?)?;
    Ok(file
        .data
        .into_iter()
        .map(|value| MotionVector {
            to_prev_x: value[0],
            to_prev_y: value[1],
        })
        .collect())
}

fn load_vec3_buffer(base_dir: &Path, reference: &BufferReference) -> Result<Vec<Normal3>> {
    let path = base_dir.join(&reference.path);
    let file: Vec3BufferFile = serde_json::from_str(&fs::read_to_string(path)?)?;
    Ok(file
        .data
        .into_iter()
        .map(|value| Normal3::new(value[0], value[1], value[2]).normalized())
        .collect())
}

fn load_bool_buffer(base_dir: &Path, reference: &BufferReference) -> Result<Vec<bool>> {
    let path = base_dir.join(&reference.path);
    let file: BoolBufferFile = serde_json::from_str(&fs::read_to_string(path)?)?;
    Ok(file.data)
}

fn write_color_buffer(base_dir: &Path, reference: &BufferReference, frame: &ImageFrame) -> Result<()> {
    let path = base_dir.join(&reference.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match reference.format.as_str() {
        "png_rgb8" => frame.save_png(&path),
        "json_rgb_f32" => {
            let file = ColorBufferFile {
                width: frame.width(),
                height: frame.height(),
                data: frame
                    .pixels()
                    .iter()
                    .map(|pixel| [pixel.r, pixel.g, pixel.b])
                    .collect(),
            };
            fs::write(path, serde_json::to_string_pretty(&file)?)?;
            Ok(())
        }
        other => Err(Error::Message(format!(
            "unsupported color output format {other}"
        ))),
    }
}

fn write_scalar_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    values: &[f32],
    width: usize,
    height: usize,
) -> Result<()> {
    let path = base_dir.join(&reference.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = ScalarBufferFile {
        width,
        height,
        data: values.to_vec(),
    };
    fs::write(path, serde_json::to_string_pretty(&file)?)?;
    Ok(())
}

fn write_vec2_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    values: &[MotionVector],
    width: usize,
    height: usize,
) -> Result<()> {
    let path = base_dir.join(&reference.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = Vec2BufferFile {
        width,
        height,
        data: values
            .iter()
            .map(|value| [value.to_prev_x, value.to_prev_y])
            .collect(),
    };
    fs::write(path, serde_json::to_string_pretty(&file)?)?;
    Ok(())
}

fn write_vec3_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    values: &[Normal3],
    width: usize,
    height: usize,
) -> Result<()> {
    let path = base_dir.join(&reference.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = Vec3BufferFile {
        width,
        height,
        data: values.iter().map(|value| [value.x, value.y, value.z]).collect(),
    };
    fs::write(path, serde_json::to_string_pretty(&file)?)?;
    Ok(())
}

fn write_bool_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    values: &[bool],
    width: usize,
    height: usize,
) -> Result<()> {
    let path = base_dir.join(&reference.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = BoolBufferFile {
        width,
        height,
        data: values.to_vec(),
    };
    fs::write(path, serde_json::to_string_pretty(&file)?)?;
    Ok(())
}

pub fn parse_scenario_id(value: &str) -> Result<ScenarioId> {
    match value {
        "thin_reveal" => Ok(ScenarioId::ThinReveal),
        "fast_pan" => Ok(ScenarioId::FastPan),
        "diagonal_reveal" => Ok(ScenarioId::DiagonalReveal),
        "reveal_band" => Ok(ScenarioId::RevealBand),
        "motion_bias_band" => Ok(ScenarioId::MotionBiasBand),
        "layered_slats" => Ok(ScenarioId::LayeredSlats),
        "noisy_reprojection" => Ok(ScenarioId::NoisyReprojection),
        "heuristic_friendly_pan" => Ok(ScenarioId::HeuristicFriendlyPan),
        "contrast_pulse" => Ok(ScenarioId::ContrastPulse),
        "stability_holdout" => Ok(ScenarioId::StabilityHoldout),
        other => Err(Error::Message(format!("unknown scenario id {other}"))),
    }
}

pub fn build_example_external_capture(
    config: &DemoConfig,
    output_dir: &Path,
) -> Result<ExternalImportArtifacts> {
    let manifest_path = output_dir.join("example_external_capture_manifest.json");
    write_example_manifest(&manifest_path)?;
    run_external_import_from_manifest(config, &manifest_path, output_dir)
}

fn reproject_frame(previous_resolved: &ImageFrame, scene_frame: &SceneFrame) -> ImageFrame {
    let mut reprojected = ImageFrame::new(
        scene_frame.ground_truth.width(),
        scene_frame.ground_truth.height(),
    );
    for y in 0..scene_frame.ground_truth.height() {
        for x in 0..scene_frame.ground_truth.width() {
            let motion = scene_frame.motion[y * scene_frame.ground_truth.width() + x];
            reprojected.set(
                x,
                y,
                previous_resolved.sample_bilinear_clamped(
                    x as f32 + motion.to_prev_x,
                    y as f32 + motion.to_prev_y,
                ),
            );
        }
    }
    reprojected
}

fn reproject_depth(previous_scene_frame: &SceneFrame, scene_frame: &SceneFrame) -> Vec<f32> {
    reproject_scalar_buffer(
        &previous_scene_frame.depth,
        scene_frame.ground_truth.width(),
        scene_frame.ground_truth.height(),
        &scene_frame.motion,
    )
}

fn reproject_normals(previous_scene_frame: &SceneFrame, scene_frame: &SceneFrame) -> Vec<Normal3> {
    let width = scene_frame.ground_truth.width();
    let height = scene_frame.ground_truth.height();
    let mut reprojected = vec![Normal3::new(0.0, 0.0, 1.0); width * height];
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let motion = scene_frame.motion[index];
            reprojected[index] = sample_normal_bilinear_clamped(
                &previous_scene_frame.normals,
                width,
                height,
                x as f32 + motion.to_prev_x,
                y as f32 + motion.to_prev_y,
            );
        }
    }
    reprojected
}

fn reproject_scalar_buffer(
    previous_values: &[f32],
    width: usize,
    height: usize,
    motion: &[MotionVector],
) -> Vec<f32> {
    let mut reprojected = vec![0.0; width * height];
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let vector = motion[index];
            reprojected[index] = sample_scalar_bilinear_clamped(
                previous_values,
                width,
                height,
                x as f32 + vector.to_prev_x,
                y as f32 + vector.to_prev_y,
            );
        }
    }
    reprojected
}

fn sample_scalar_bilinear_clamped(
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
    let sample = |sample_x: f32, sample_y: f32| {
        let sx = sample_x.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let sy = sample_y.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[sy * width + sx]
    };
    let top = sample(x0, y0) * (1.0 - tx) + sample(x1, y0) * tx;
    let bottom = sample(x0, y1) * (1.0 - tx) + sample(x1, y1) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn sample_normal_bilinear_clamped(
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
    let mix = |a: Normal3, b: Normal3, t: f32| {
        Normal3::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
        )
    };
    mix(
        mix(sample(x0, y0), sample(x1, y0), tx),
        mix(sample(x0, y1), sample(x1, y1), tx),
        ty,
    )
    .normalized()
}

pub fn compute_external_compatible_mask(scene_frame: &SceneFrame) -> Vec<bool> {
    scene_frame
        .layers
        .iter()
        .zip(scene_frame.disocclusion_mask.iter().copied())
        .map(|(layer, disoccluded)| {
            disoccluded && !matches!(*layer, SurfaceTag::ForegroundObject)
        })
        .collect()
}
