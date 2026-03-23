use std::fs;
use std::io::Read;
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
pub const NO_REAL_EXTERNAL_DATA_PROVIDED: &str = "NO REAL EXTERNAL DATA PROVIDED";

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
    #[serde(default)]
    pub width: Option<usize>,
    #[serde(default)]
    pub height: Option<usize>,
    #[serde(default)]
    pub channels: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalBufferSet {
    pub current_color: BufferReference,
    #[serde(rename = "history_color", alias = "reprojected_history")]
    pub reprojected_history: BufferReference,
    pub motion_vectors: BufferReference,
    pub current_depth: BufferReference,
    #[serde(rename = "history_depth", alias = "reprojected_depth")]
    pub reprojected_depth: BufferReference,
    pub current_normals: BufferReference,
    #[serde(rename = "history_normals", alias = "reprojected_normals")]
    pub reprojected_normals: BufferReference,
    pub metadata: BufferReference,
    pub optional_mask: Option<BufferReference>,
    #[serde(default)]
    pub optional_reference: Option<BufferReference>,
    #[serde(default)]
    pub optional_ground_truth: Option<BufferReference>,
    #[serde(default)]
    pub optional_variance: Option<BufferReference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalCaptureEntry {
    pub label: String,
    pub buffers: ExternalBufferSet,
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
    #[serde(default)]
    pub buffers: Option<ExternalBufferSet>,
    #[serde(default)]
    pub captures: Vec<ExternalCaptureEntry>,
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
    #[serde(default)]
    pub real_external_data: bool,
    #[serde(default)]
    pub data_description: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalHandoffMetrics {
    pub measurement_kind: String,
    pub external_capable: bool,
    pub externally_validated: bool,
    pub real_external_data_provided: bool,
    pub no_real_external_data_provided: bool,
    pub source_kind: String,
    pub scenario_id: Option<String>,
    pub frame_index: usize,
    pub history_frame_index: usize,
    pub width: usize,
    pub height: usize,
    pub capture_count: usize,
    pub imported_formats: Vec<String>,
    pub required_buffers: Vec<String>,
    pub normalization_notes: Vec<String>,
    pub roi_source: String,
    pub ground_truth_available: bool,
    pub variance_available: bool,
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

#[derive(Clone, Debug)]
pub struct ExternalLoadedCapture {
    pub label: String,
    pub inputs: OwnedHostTemporalInputs,
    pub metadata: ExternalCaptureMetadata,
    pub mask: Option<Vec<bool>>,
    pub reference: Option<ImageFrame>,
    pub variance: Option<ScalarField>,
}

#[derive(Clone, Debug)]
pub struct ExternalCaptureBundle {
    pub manifest: ExternalCaptureManifest,
    pub captures: Vec<ExternalLoadedCapture>,
    pub real_external_data_provided: bool,
    pub no_real_external_data_provided: bool,
}

pub fn example_manifest() -> ExternalCaptureManifest {
    ExternalCaptureManifest {
        format_version: EXTERNAL_CAPTURE_FORMAT_VERSION.to_string(),
        description: "Synthetic compatibility example that exports one frame pair into the stable external buffer schema and re-imports it through the same file-based path.".to_string(),
        source: ExternalCaptureSource::SyntheticCompat {
            scenario_id: "motion_bias_band".to_string(),
            frame_index: None,
        },
        buffers: Some(ExternalBufferSet {
            current_color: BufferReference {
                path: "external_capture/current_color.png".to_string(),
                format: "png_rgb8".to_string(),
                semantic: "current color, normalized [0,1] RGB".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            reprojected_history: BufferReference {
                path: "external_capture/reprojected_history.png".to_string(),
                format: "png_rgb8".to_string(),
                semantic: "reprojected history color, normalized [0,1] RGB".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            motion_vectors: BufferReference {
                path: "external_capture/motion_vectors.json".to_string(),
                format: "json_vec2_f32".to_string(),
                semantic: "per-pixel motion vector to previous frame in pixel units".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            current_depth: BufferReference {
                path: "external_capture/current_depth.json".to_string(),
                format: "json_scalar_f32".to_string(),
                semantic: "current frame depth".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            reprojected_depth: BufferReference {
                path: "external_capture/reprojected_depth.json".to_string(),
                format: "json_scalar_f32".to_string(),
                semantic: "reprojected depth from previous frame".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            current_normals: BufferReference {
                path: "external_capture/current_normals.json".to_string(),
                format: "json_vec3_f32".to_string(),
                semantic: "current frame normals in view space, unit length".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            reprojected_normals: BufferReference {
                path: "external_capture/reprojected_normals.json".to_string(),
                format: "json_vec3_f32".to_string(),
                semantic: "reprojected normals from previous frame".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            metadata: BufferReference {
                path: "external_capture/metadata.json".to_string(),
                format: "json_metadata".to_string(),
                semantic: "capture metadata and provenance".to_string(),
                width: None,
                height: None,
                channels: None,
            },
            optional_mask: Some(BufferReference {
                path: "external_capture/optional_mask.json".to_string(),
                format: "json_mask_bool".to_string(),
                semantic: "optional ROI-like disclosure or debug mask".to_string(),
                width: None,
                height: None,
                channels: None,
            }),
            optional_reference: Some(BufferReference {
                path: "external_capture/optional_reference.png".to_string(),
                format: "png_rgb8".to_string(),
                semantic: "optional reference frame for evaluator-side error checks; synthetic in the bundled example".to_string(),
                width: None,
                height: None,
                channels: None,
            }),
            optional_ground_truth: None,
            optional_variance: None,
        }),
        captures: Vec::new(),
        normalization: ExternalNormalization {
            color: "linear RGB in [0,1]".to_string(),
            motion_vectors: "pixel offsets to the previous frame; positive x samples from a pixel further right in history".to_string(),
            depth: "monotonic depth with larger disagreement indicating less trust".to_string(),
            normals: "unit vectors in a consistent view-space basis".to_string(),
        },
        notes: vec![
            "Switch source.kind from synthetic_compat to files when real engine exports are available.".to_string(),
            "The example capture is external-capable but not externally validated.".to_string(),
            NO_REAL_EXTERNAL_DATA_PROVIDED.to_string(),
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
    let bundle = load_external_capture_bundle(config, manifest_path, output_dir)?;
    let first_capture = bundle
        .captures
        .first()
        .ok_or_else(|| Error::Message("external capture bundle had no captures".to_string()))?;
    let first_buffers = first_capture_buffer_set(&bundle.manifest)?;
    let resolved_manifest_path = output_dir.join("resolved_external_capture_manifest.json");
    fs::write(
        &resolved_manifest_path,
        serde_json::to_string_pretty(&bundle.manifest)?,
    )?;

    let profile =
        default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
    let outputs = supervise_temporal_reuse(&first_capture.inputs.borrow(), &profile);
    write_external_outputs(output_dir, &first_capture.inputs, &outputs)?;

    let no_real_external_data_provided =
        bundle.no_real_external_data_provided || !bundle.real_external_data_provided;
    let mut notes = first_capture.metadata.notes.clone();
    if no_real_external_data_provided {
        notes.push(NO_REAL_EXTERNAL_DATA_PROVIDED.to_string());
    }

    let metrics = ExternalHandoffMetrics {
        measurement_kind: if bundle.real_external_data_provided {
            "external_buffer_import_real".to_string()
        } else {
            "external_buffer_import_external_ready".to_string()
        },
        external_capable: true,
        externally_validated: first_capture.metadata.externally_validated,
        real_external_data_provided: bundle.real_external_data_provided,
        no_real_external_data_provided,
        source_kind: first_capture.metadata.source_kind.clone(),
        scenario_id: first_capture.metadata.scenario_id.clone(),
        frame_index: first_capture.metadata.frame_index,
        history_frame_index: first_capture.metadata.history_frame_index,
        width: first_capture.metadata.width,
        height: first_capture.metadata.height,
        capture_count: bundle.captures.len(),
        imported_formats: vec![
            first_buffers.current_color.format.clone(),
            first_buffers.reprojected_history.format.clone(),
            first_buffers.motion_vectors.format.clone(),
            first_buffers.current_depth.format.clone(),
            first_buffers.current_normals.format.clone(),
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
            bundle.manifest.normalization.color.clone(),
            bundle.manifest.normalization.motion_vectors.clone(),
            bundle.manifest.normalization.depth.clone(),
            bundle.manifest.normalization.normals.clone(),
        ],
        roi_source: if first_capture.mask.is_some() {
            "manifest_mask".to_string()
        } else {
            "derived_proxy_mask".to_string()
        },
        ground_truth_available: first_capture.reference.is_some(),
        variance_available: first_capture.variance.is_some(),
        mean_trust: outputs.trust.mean(),
        mean_alpha: outputs.alpha.mean(),
        intervention_rate: outputs.intervention.mean(),
        notes,
    };

    let report_path = output_dir.join("external_replay_report.md");
    let handoff_alias_path = output_dir.join("external_handoff_report.md");
    write_external_replay_report(&report_path, &metrics, &bundle.manifest)?;
    fs::copy(&report_path, &handoff_alias_path)?;

    Ok(ExternalImportArtifacts {
        report_path,
        metrics,
        resolved_manifest_path,
    })
}

pub fn load_external_capture_bundle(
    config: &DemoConfig,
    manifest_path: &Path,
    output_dir: &Path,
) -> Result<ExternalCaptureBundle> {
    let manifest_text = fs::read_to_string(manifest_path)?;
    let manifest: ExternalCaptureManifest = serde_json::from_str(&manifest_text)?;
    if manifest.format_version != EXTERNAL_CAPTURE_FORMAT_VERSION {
        return Err(Error::Message(format!(
            "unsupported external capture format version {}",
            manifest.format_version
        )));
    }

    let resolved_manifest = match &manifest.source {
        ExternalCaptureSource::Files => manifest.clone(),
        ExternalCaptureSource::SyntheticCompat {
            scenario_id,
            frame_index,
        } => {
            let scenario_id = parse_scenario_id(scenario_id)?;
            let definition = scenario_by_id(&config.scene, scenario_id).ok_or_else(|| {
                Error::Message(format!(
                    "synthetic compat scenario {scenario_id:?} not found"
                ))
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
            let metadata = ExternalCaptureMetadata {
                scenario_id: Some(sequence.scenario_id.as_str().to_string()),
                frame_index: export_frame_index,
                history_frame_index: export_frame_index.saturating_sub(1),
                width: inputs.width(),
                height: inputs.height(),
                source_kind: "synthetic_compat".to_string(),
                externally_validated: false,
                real_external_data: false,
                data_description: Some(
                    "Deterministic synthetic compatibility export generated inside the crate"
                        .to_string(),
                ),
                notes: vec![
                    "The example external capture was generated from the crate's deterministic synthetic suite.".to_string(),
                    "Replace source.kind with files and point the buffer paths at real engine exports to move beyond synthetic compatibility.".to_string(),
                    NO_REAL_EXTERNAL_DATA_PROVIDED.to_string(),
                ],
            };
            let capture_mask =
                compute_external_compatible_mask(&sequence.frames[export_frame_index]);
            materialize_capture_bundle(
                &manifest,
                output_dir,
                &inputs,
                &metadata,
                Some(&capture_mask),
                Some(&inputs.current_color),
            )?
        }
    };

    let captures = load_capture_entries(&resolved_manifest, manifest_path, output_dir)?;
    let real_external_data_provided = captures
        .iter()
        .any(|capture| capture.metadata.real_external_data);
    Ok(ExternalCaptureBundle {
        manifest: resolved_manifest,
        captures,
        real_external_data_provided,
        no_real_external_data_provided: !real_external_data_provided,
    })
}

fn materialize_capture_bundle(
    manifest: &ExternalCaptureManifest,
    base_dir: &Path,
    inputs: &OwnedHostTemporalInputs,
    metadata: &ExternalCaptureMetadata,
    optional_mask: Option<&[bool]>,
    optional_reference: Option<&ImageFrame>,
) -> Result<ExternalCaptureManifest> {
    let buffer_set = manifest.buffers.clone().ok_or_else(|| {
        Error::Message("synthetic compat manifest requires a top-level buffers block".to_string())
    })?;
    write_color_buffer(base_dir, &buffer_set.current_color, &inputs.current_color)?;
    write_color_buffer(
        base_dir,
        &buffer_set.reprojected_history,
        &inputs.reprojected_history,
    )?;
    write_vec2_buffer(
        base_dir,
        &buffer_set.motion_vectors,
        &inputs.motion_vectors,
        inputs.width(),
        inputs.height(),
    )?;
    write_scalar_buffer(
        base_dir,
        &buffer_set.current_depth,
        &inputs.current_depth,
        inputs.width(),
        inputs.height(),
    )?;
    write_scalar_buffer(
        base_dir,
        &buffer_set.reprojected_depth,
        &inputs.reprojected_depth,
        inputs.width(),
        inputs.height(),
    )?;
    write_vec3_buffer(
        base_dir,
        &buffer_set.current_normals,
        &inputs.current_normals,
        inputs.width(),
        inputs.height(),
    )?;
    write_vec3_buffer(
        base_dir,
        &buffer_set.reprojected_normals,
        &inputs.reprojected_normals,
        inputs.width(),
        inputs.height(),
    )?;
    if let Some(mask_ref) = &buffer_set.optional_mask {
        let fallback_mask;
        let mask_values = if let Some(values) = optional_mask {
            values
        } else {
            fallback_mask = vec![false; inputs.width() * inputs.height()];
            &fallback_mask
        };
        write_bool_buffer(
            base_dir,
            mask_ref,
            mask_values,
            inputs.width(),
            inputs.height(),
        )?;
    }
    if let Some(reference_ref) = &buffer_set
        .optional_ground_truth
        .as_ref()
        .or(buffer_set.optional_reference.as_ref())
    {
        if let Some(reference_frame) = optional_reference {
            write_color_buffer(base_dir, reference_ref, reference_frame)?;
        }
    }
    let metadata_path = base_dir.join(&buffer_set.metadata.path);
    if let Some(parent) = metadata_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&metadata_path, serde_json::to_string_pretty(metadata)?)?;
    let mut resolved = manifest.clone();
    resolved.buffers = Some(buffer_set);
    Ok(resolved)
}

fn load_capture_entries(
    manifest: &ExternalCaptureManifest,
    manifest_path: &Path,
    synthetic_base_dir: &Path,
) -> Result<Vec<ExternalLoadedCapture>> {
    let entries = capture_entries(manifest)?;
    match &manifest.source {
        ExternalCaptureSource::Files => {
            let base_dir = manifest_path.parent().ok_or_else(|| {
                Error::Message("manifest path had no parent directory".to_string())
            })?;
            entries
                .iter()
                .map(|(label, buffers)| {
                    load_single_capture(base_dir, label, buffers, &manifest.normalization)
                })
                .collect()
        }
        ExternalCaptureSource::SyntheticCompat { .. } => entries
            .iter()
            .map(|(label, buffers)| {
                load_single_capture(synthetic_base_dir, label, buffers, &manifest.normalization)
            })
            .collect(),
    }
}

fn capture_entries(manifest: &ExternalCaptureManifest) -> Result<Vec<(String, ExternalBufferSet)>> {
    if !manifest.captures.is_empty() {
        return Ok(manifest
            .captures
            .iter()
            .map(|entry| (entry.label.clone(), entry.buffers.clone()))
            .collect());
    }
    if let Some(buffers) = &manifest.buffers {
        return Ok(vec![("capture_0".to_string(), buffers.clone())]);
    }
    Err(Error::Message(
        "external capture manifest must provide either `buffers` for one frame pair or `captures` for a short sequence".to_string(),
    ))
}

fn first_capture_buffer_set(manifest: &ExternalCaptureManifest) -> Result<ExternalBufferSet> {
    capture_entries(manifest)?
        .into_iter()
        .next()
        .map(|(_, buffers)| buffers)
        .ok_or_else(|| Error::Message("external manifest had no buffer set".to_string()))
}

fn load_single_capture(
    base_dir: &Path,
    label: &str,
    buffers: &ExternalBufferSet,
    normalization: &ExternalNormalization,
) -> Result<ExternalLoadedCapture> {
    let metadata = load_metadata(base_dir, &buffers.metadata, false)?;
    let inputs = load_owned_inputs(buffers, base_dir, metadata.width, metadata.height)?;
    if metadata.width != inputs.width() || metadata.height != inputs.height() {
        return Err(Error::Message(format!(
            "metadata extent {}x{} did not match imported buffers {}x{}",
            metadata.width,
            metadata.height,
            inputs.width(),
            inputs.height()
        )));
    }
    validate_normalization(label, &inputs, normalization)?;
    let mask = buffers
        .optional_mask
        .as_ref()
        .map(|reference| load_bool_buffer(base_dir, reference, metadata.width, metadata.height))
        .transpose()?
        .map(|file| file.data);
    let reference = buffers
        .optional_ground_truth
        .as_ref()
        .or(buffers.optional_reference.as_ref())
        .map(|reference| load_color_buffer(base_dir, reference, metadata.width, metadata.height))
        .transpose()?;
    let variance = buffers
        .optional_variance
        .as_ref()
        .map(|reference| {
            load_scalar_buffer(base_dir, reference, metadata.width, metadata.height)
                .map(|file| ScalarField::from_values(metadata.width, metadata.height, file.data))
        })
        .transpose()?;

    Ok(ExternalLoadedCapture {
        label: label.to_string(),
        inputs,
        metadata,
        mask,
        reference,
        variance,
    })
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
    save_scalar_field_png(
        &outputs.trust,
        &output_dir.join("external_trust.png"),
        |value| {
            let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            [v, v, 255, 255]
        },
    )?;
    save_scalar_field_png(
        &outputs.alpha,
        &output_dir.join("external_alpha.png"),
        |value| {
            let v = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            [255, v, 0, 255]
        },
    )?;
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

fn write_external_replay_report(
    path: &Path,
    metrics: &ExternalHandoffMetrics,
    manifest: &ExternalCaptureManifest,
) -> Result<()> {
    let mut markdown = String::new();
    markdown.push_str("# External Replay Report\n\n");
    markdown.push_str(EXPERIMENT_SENTENCE);
    markdown.push_str("\n\n");
    markdown.push_str("This report covers the file-based external buffer replay path. It demonstrates that the crate is external-capable, not externally validated.\n\n");
    markdown.push_str(&format!(
        "Source kind: `{}`. Externally validated: `{}`. Real external data provided: `{}`.\n\n",
        metrics.source_kind, metrics.externally_validated, metrics.real_external_data_provided
    ));
    if metrics.no_real_external_data_provided {
        markdown.push_str(NO_REAL_EXTERNAL_DATA_PROVIDED);
        markdown.push_str("\n\n");
    }
    markdown.push_str("## Required Buffers\n\n");
    for buffer in &metrics.required_buffers {
        markdown.push_str(&format!("- `{buffer}`\n"));
    }
    markdown.push_str("\n## Accepted Formats\n\n");
    markdown.push_str("- `png_rgb8`\n");
    markdown.push_str("- `json_rgb_f32`\n");
    markdown.push_str("- `exr_rgb32f`\n");
    markdown.push_str("- `json_scalar_f32`\n");
    markdown.push_str("- `exr_r32f`\n");
    markdown.push_str("- `raw_r32f` with inline width/height/channels = 1\n");
    markdown.push_str("- `json_vec2_f32`\n");
    markdown.push_str("- `exr_rg32f`\n");
    markdown.push_str("- `raw_rg32f` with inline width/height/channels >= 2\n");
    markdown.push_str("- `json_vec3_f32`\n");
    markdown.push_str("- `raw_rgb32f` with inline width/height/channels >= 3\n");
    markdown.push_str("- `json_mask_bool`\n");
    markdown.push_str("- `raw_mask_u8` with inline width/height/channels = 1\n");
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
    markdown.push_str(
        "- Set `source.kind` to `files` and point the buffer paths at the exported assets.\n",
    );
    markdown.push_str("- Run `cargo run --release -- run-external-replay --manifest <manifest> --output <dir>`.\n");
    markdown.push_str(
        "- Alias: `cargo run --release -- replay-external --manifest <manifest> --output <dir>`.\n",
    );
    markdown.push_str("- Inspect `external_trust.png`, `external_alpha.png`, and `external_intervention.png` plus the generated report.\n\n");
    markdown.push_str("## What Is Not Proven\n\n");
    markdown.push_str("- This report does not claim any real engine capture has been validated unless the metadata says so.\n");
    markdown.push_str("- The example manifest included in the crate is synthetic compatibility data, not field data.\n\n");
    markdown.push_str("## Remaining Blockers\n\n");
    markdown.push_str("- A real renderer still needs to export buffers into this schema.\n");
    markdown.push_str("- Real production captures and engine motion vectors are still required for external validation.\n");
    markdown.push_str("- If the GPU external report is unmeasured on the evaluator machine, imported-capture GPU timing still remains future work there.\n\n");
    markdown.push_str("## Manifest Notes\n\n");
    for note in &manifest.notes {
        markdown.push_str(&format!("- {note}\n"));
    }
    fs::write(path, markdown)?;
    Ok(())
}

fn load_owned_inputs(
    buffers: &ExternalBufferSet,
    base_dir: &Path,
    expected_width: usize,
    expected_height: usize,
) -> Result<OwnedHostTemporalInputs> {
    let current_color = load_color_buffer(
        base_dir,
        &buffers.current_color,
        expected_width,
        expected_height,
    )?;
    let reprojected_history = load_color_buffer(
        base_dir,
        &buffers.reprojected_history,
        expected_width,
        expected_height,
    )?;
    let width = current_color.width();
    let height = current_color.height();
    let motion_vectors = load_vec2_buffer(
        base_dir,
        &buffers.motion_vectors,
        expected_width,
        expected_height,
    )?;
    validate_buffer_extent(
        "motion_vectors",
        motion_vectors.width,
        motion_vectors.height,
        width,
        height,
    )?;
    let current_depth = load_scalar_buffer(
        base_dir,
        &buffers.current_depth,
        expected_width,
        expected_height,
    )?;
    validate_buffer_extent(
        "current_depth",
        current_depth.width,
        current_depth.height,
        width,
        height,
    )?;
    let reprojected_depth = load_scalar_buffer(
        base_dir,
        &buffers.reprojected_depth,
        expected_width,
        expected_height,
    )?;
    validate_buffer_extent(
        "reprojected_depth",
        reprojected_depth.width,
        reprojected_depth.height,
        width,
        height,
    )?;
    let current_normals = load_vec3_buffer(
        base_dir,
        &buffers.current_normals,
        expected_width,
        expected_height,
    )?;
    validate_buffer_extent(
        "current_normals",
        current_normals.width,
        current_normals.height,
        width,
        height,
    )?;
    let reprojected_normals = load_vec3_buffer(
        base_dir,
        &buffers.reprojected_normals,
        expected_width,
        expected_height,
    )?;
    validate_buffer_extent(
        "reprojected_normals",
        reprojected_normals.width,
        reprojected_normals.height,
        width,
        height,
    )?;
    let optional_mask = buffers
        .optional_mask
        .as_ref()
        .map(|reference| load_bool_buffer(base_dir, reference, expected_width, expected_height))
        .transpose()?;
    if let Some(mask) = &optional_mask {
        validate_buffer_extent("optional_mask", mask.width, mask.height, width, height)?;
    }
    if let Some(reference) = buffers
        .optional_ground_truth
        .as_ref()
        .or(buffers.optional_reference.as_ref())
    {
        let optional_reference =
            load_color_buffer(base_dir, reference, expected_width, expected_height)?;
        validate_buffer_extent(
            "optional_reference",
            optional_reference.width(),
            optional_reference.height(),
            width,
            height,
        )?;
    }
    Ok(OwnedHostTemporalInputs {
        current_color,
        reprojected_history,
        motion_vectors: motion_vectors
            .data
            .into_iter()
            .map(|value| MotionVector {
                to_prev_x: value[0],
                to_prev_y: value[1],
            })
            .collect(),
        current_depth: current_depth.data,
        reprojected_depth: reprojected_depth.data,
        current_normals: current_normals
            .data
            .into_iter()
            .map(|value| Normal3::new(value[0], value[1], value[2]).normalized())
            .collect(),
        reprojected_normals: reprojected_normals
            .data
            .into_iter()
            .map(|value| Normal3::new(value[0], value[1], value[2]).normalized())
            .collect(),
        visibility_hint: optional_mask.map(|mask| mask.data),
        thin_hint: None,
    })
}

fn load_metadata(
    base_dir: &Path,
    reference: &BufferReference,
    externally_validated: bool,
) -> Result<ExternalCaptureMetadata> {
    if reference.format != "json_metadata" {
        return Err(Error::Message(format!(
            "metadata buffer {} must use json_metadata format",
            reference.path
        )));
    }
    let path = base_dir.join(&reference.path);
    let text = fs::read_to_string(path)?;
    let mut metadata: ExternalCaptureMetadata = serde_json::from_str(&text)?;
    metadata.externally_validated = externally_validated || metadata.externally_validated;
    if metadata.width == 0 || metadata.height == 0 {
        return Err(Error::Message(format!(
            "metadata {} declared zero-sized capture {}x{}",
            reference.path, metadata.width, metadata.height
        )));
    }
    if metadata.history_frame_index > metadata.frame_index {
        return Err(Error::Message(format!(
            "metadata {} had history_frame_index {} after frame_index {}",
            reference.path, metadata.history_frame_index, metadata.frame_index
        )));
    }
    Ok(metadata)
}

fn load_color_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<ImageFrame> {
    let path = base_dir.join(&reference.path);
    let frame = match reference.format.as_str() {
        "png_rgb8" => ImageFrame::load_png(&path)?,
        "exr_rgb32f" => load_exr_color(&path)?,
        "raw_rgb32f" => load_raw_color(&path, reference, expected_width, expected_height)?,
        "json_rgb_f32" => {
            let file: ColorBufferFile = serde_json::from_str(&fs::read_to_string(path)?)?;
            ImageFrame::from_pixels(
                file.width,
                file.height,
                file.data
                    .into_iter()
                    .map(|rgb| Color::rgb(rgb[0], rgb[1], rgb[2]))
                    .collect(),
            )
        }
        other => {
            return Err(Error::Message(format!(
                "unsupported color buffer format {other}"
            )))
        }
    };
    validate_buffer_extent(
        "color_buffer",
        frame.width(),
        frame.height(),
        expected_width,
        expected_height,
    )?;
    Ok(frame)
}

fn load_scalar_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<ScalarBufferFile> {
    let path = base_dir.join(&reference.path);
    let file = match reference.format.as_str() {
        "json_scalar_f32" => serde_json::from_str(&fs::read_to_string(path)?)?,
        "exr_r32f" => load_exr_scalar(&path)?,
        "raw_r32f" => load_raw_scalar(&path, reference, expected_width, expected_height)?,
        other => {
            return Err(Error::Message(format!(
                "unsupported scalar buffer format {other}"
            )))
        }
    };
    validate_element_count("scalar_buffer", file.width, file.height, file.data.len())?;
    validate_buffer_extent(
        "scalar_buffer",
        file.width,
        file.height,
        expected_width,
        expected_height,
    )?;
    Ok(file)
}

fn load_vec2_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<Vec2BufferFile> {
    let path = base_dir.join(&reference.path);
    let file = match reference.format.as_str() {
        "json_vec2_f32" => serde_json::from_str(&fs::read_to_string(path)?)?,
        "exr_rg32f" => load_exr_vec2(&path)?,
        "raw_rg32f" => load_raw_vec2(&path, reference, expected_width, expected_height)?,
        other => {
            return Err(Error::Message(format!(
                "unsupported vec2 buffer format {other}"
            )))
        }
    };
    validate_element_count("motion_vectors", file.width, file.height, file.data.len())?;
    validate_buffer_extent(
        "motion_vectors",
        file.width,
        file.height,
        expected_width,
        expected_height,
    )?;
    Ok(file)
}

fn load_vec3_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<Vec3BufferFile> {
    let path = base_dir.join(&reference.path);
    let file = match reference.format.as_str() {
        "json_vec3_f32" => serde_json::from_str(&fs::read_to_string(path)?)?,
        "exr_rgb32f" => load_exr_vec3(&path)?,
        "raw_rgb32f" => load_raw_vec3(&path, reference, expected_width, expected_height)?,
        other => {
            return Err(Error::Message(format!(
                "unsupported vec3 buffer format {other}"
            )))
        }
    };
    validate_element_count("normal_buffer", file.width, file.height, file.data.len())?;
    validate_buffer_extent(
        "normal_buffer",
        file.width,
        file.height,
        expected_width,
        expected_height,
    )?;
    Ok(file)
}

fn load_bool_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<BoolBufferFile> {
    let path = base_dir.join(&reference.path);
    let file = match reference.format.as_str() {
        "json_mask_bool" => serde_json::from_str(&fs::read_to_string(path)?)?,
        "raw_mask_u8" => load_raw_mask(&path, reference, expected_width, expected_height)?,
        other => {
            return Err(Error::Message(format!(
                "unsupported mask buffer format {other}"
            )))
        }
    };
    validate_element_count("optional_mask", file.width, file.height, file.data.len())?;
    validate_buffer_extent(
        "optional_mask",
        file.width,
        file.height,
        expected_width,
        expected_height,
    )?;
    Ok(file)
}

fn validate_buffer_extent(
    label: &str,
    width: usize,
    height: usize,
    expected_width: usize,
    expected_height: usize,
) -> Result<()> {
    if width != expected_width || height != expected_height {
        return Err(Error::Message(format!(
            "{label} extent {width}x{height} did not match expected {expected_width}x{expected_height}"
        )));
    }
    Ok(())
}

fn validate_element_count(label: &str, width: usize, height: usize, count: usize) -> Result<()> {
    let expected = width.saturating_mul(height);
    if count != expected {
        return Err(Error::Message(format!(
            "{label} had {count} elements but expected {expected} for {width}x{height}"
        )));
    }
    Ok(())
}

fn validate_normalization(
    label: &str,
    inputs: &OwnedHostTemporalInputs,
    normalization: &ExternalNormalization,
) -> Result<()> {
    if normalization.color.contains("[0,1]") || normalization.color.contains("linear RGB") {
        for (index, pixel) in inputs.current_color.pixels().iter().enumerate() {
            for channel in [pixel.r, pixel.g, pixel.b] {
                if !(-0.01..=1.01).contains(&channel) {
                    return Err(Error::Message(format!(
                        "{label} current_color pixel {index} violated normalized color expectations"
                    )));
                }
            }
        }
    }

    for (index, depth) in inputs.current_depth.iter().enumerate() {
        if !depth.is_finite() {
            return Err(Error::Message(format!(
                "{label} current_depth[{index}] was non-finite"
            )));
        }
    }
    for (index, depth) in inputs.reprojected_depth.iter().enumerate() {
        if !depth.is_finite() {
            return Err(Error::Message(format!(
                "{label} reprojected_depth[{index}] was non-finite"
            )));
        }
    }
    for (index, motion) in inputs.motion_vectors.iter().enumerate() {
        if !motion.to_prev_x.is_finite()
            || !motion.to_prev_y.is_finite()
            || motion.to_prev_x.abs() > inputs.width() as f32 * 4.0
            || motion.to_prev_y.abs() > inputs.height() as f32 * 4.0
        {
            return Err(Error::Message(format!(
                "{label} motion_vectors[{index}] violated finite/range validation"
            )));
        }
    }
    if normalization.normals.contains("unit") {
        for (index, normal) in inputs.current_normals.iter().enumerate() {
            let norm = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
            if !norm.is_finite() || (norm - 1.0).abs() > 0.05 {
                return Err(Error::Message(format!(
                    "{label} current_normals[{index}] violated unit-normal validation"
                )));
            }
        }
    }
    Ok(())
}

fn load_exr_color(path: &Path) -> Result<ImageFrame> {
    let image = image::open(path)?.to_rgb32f();
    let width = image.width() as usize;
    let height = image.height() as usize;
    let pixels = image
        .pixels()
        .map(|pixel| Color::rgb(pixel[0], pixel[1], pixel[2]))
        .collect();
    Ok(ImageFrame::from_pixels(width, height, pixels))
}

fn load_exr_scalar(path: &Path) -> Result<ScalarBufferFile> {
    let image = image::open(path)?.to_rgba32f();
    let width = image.width() as usize;
    let height = image.height() as usize;
    let data = image.pixels().map(|pixel| pixel[0]).collect();
    Ok(ScalarBufferFile {
        width,
        height,
        data,
    })
}

fn load_exr_vec2(path: &Path) -> Result<Vec2BufferFile> {
    let image = image::open(path)?.to_rgba32f();
    let width = image.width() as usize;
    let height = image.height() as usize;
    let data = image.pixels().map(|pixel| [pixel[0], pixel[1]]).collect();
    Ok(Vec2BufferFile {
        width,
        height,
        data,
    })
}

fn load_exr_vec3(path: &Path) -> Result<Vec3BufferFile> {
    let image = image::open(path)?.to_rgb32f();
    let width = image.width() as usize;
    let height = image.height() as usize;
    let data = image
        .pixels()
        .map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .collect();
    Ok(Vec3BufferFile {
        width,
        height,
        data,
    })
}

fn load_raw_color(
    path: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<ImageFrame> {
    let (width, height) = extent_from_reference(reference, expected_width, expected_height)?;
    let channels = channel_count(reference, 3)?;
    if channels < 3 {
        return Err(Error::Message(format!(
            "raw_rgb32f buffer {} must provide at least 3 channels",
            reference.path
        )));
    }
    let values = read_raw_f32_values(path)?;
    validate_raw_value_count("raw_rgb32f", width, height, channels, values.len())?;
    let mut pixels = Vec::with_capacity(width * height);
    for chunk in values.chunks_exact(channels) {
        pixels.push(Color::rgb(chunk[0], chunk[1], chunk[2]));
    }
    Ok(ImageFrame::from_pixels(width, height, pixels))
}

fn load_raw_scalar(
    path: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<ScalarBufferFile> {
    let (width, height) = extent_from_reference(reference, expected_width, expected_height)?;
    let channels = channel_count(reference, 1)?;
    if channels != 1 {
        return Err(Error::Message(format!(
            "raw_r32f buffer {} must declare channels = 1",
            reference.path
        )));
    }
    let values = read_raw_f32_values(path)?;
    validate_raw_value_count("raw_r32f", width, height, channels, values.len())?;
    Ok(ScalarBufferFile {
        width,
        height,
        data: values,
    })
}

fn load_raw_vec2(
    path: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<Vec2BufferFile> {
    let (width, height) = extent_from_reference(reference, expected_width, expected_height)?;
    let channels = channel_count(reference, 2)?;
    if channels < 2 {
        return Err(Error::Message(format!(
            "raw_rg32f buffer {} must provide at least 2 channels",
            reference.path
        )));
    }
    let values = read_raw_f32_values(path)?;
    validate_raw_value_count("raw_rg32f", width, height, channels, values.len())?;
    let data = values
        .chunks_exact(channels)
        .map(|chunk| [chunk[0], chunk[1]])
        .collect();
    Ok(Vec2BufferFile {
        width,
        height,
        data,
    })
}

fn load_raw_vec3(
    path: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<Vec3BufferFile> {
    let (width, height) = extent_from_reference(reference, expected_width, expected_height)?;
    let channels = channel_count(reference, 3)?;
    if channels < 3 {
        return Err(Error::Message(format!(
            "raw_rgb32f buffer {} must provide at least 3 channels",
            reference.path
        )));
    }
    let values = read_raw_f32_values(path)?;
    validate_raw_value_count("raw_rgb32f", width, height, channels, values.len())?;
    let data = values
        .chunks_exact(channels)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();
    Ok(Vec3BufferFile {
        width,
        height,
        data,
    })
}

fn load_raw_mask(
    path: &Path,
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<BoolBufferFile> {
    let (width, height) = extent_from_reference(reference, expected_width, expected_height)?;
    let channels = channel_count(reference, 1)?;
    if channels != 1 {
        return Err(Error::Message(format!(
            "raw_mask_u8 buffer {} must declare channels = 1",
            reference.path
        )));
    }
    let bytes = fs::read(path)?;
    let expected = width.saturating_mul(height);
    if bytes.len() != expected {
        return Err(Error::Message(format!(
            "raw_mask_u8 buffer {} had {} bytes but expected {} for {}x{}",
            reference.path,
            bytes.len(),
            expected,
            width,
            height
        )));
    }
    Ok(BoolBufferFile {
        width,
        height,
        data: bytes.into_iter().map(|value| value != 0).collect(),
    })
}

fn extent_from_reference(
    reference: &BufferReference,
    expected_width: usize,
    expected_height: usize,
) -> Result<(usize, usize)> {
    let width = reference.width.unwrap_or(expected_width);
    let height = reference.height.unwrap_or(expected_height);
    if width == 0 || height == 0 {
        return Err(Error::Message(format!(
            "buffer {} must declare positive width/height either in metadata or inline",
            reference.path
        )));
    }
    Ok((width, height))
}

fn channel_count(reference: &BufferReference, default_channels: usize) -> Result<usize> {
    let channels = reference.channels.unwrap_or(default_channels);
    if channels == 0 {
        return Err(Error::Message(format!(
            "buffer {} declared zero channels",
            reference.path
        )));
    }
    Ok(channels)
}

fn validate_raw_value_count(
    label: &str,
    width: usize,
    height: usize,
    channels: usize,
    value_count: usize,
) -> Result<()> {
    let expected = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(channels))
        .ok_or_else(|| Error::Message(format!("{label} extent overflowed")))?;
    if value_count != expected {
        return Err(Error::Message(format!(
            "{label} had {value_count} float values but expected {expected} for {width}x{height}x{channels}"
        )));
    }
    Ok(())
}

fn read_raw_f32_values(path: &Path) -> Result<Vec<f32>> {
    let mut file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    if bytes.len() % 4 != 0 {
        return Err(Error::Message(format!(
            "raw float buffer {} had {} bytes, which is not divisible by 4",
            path.display(),
            bytes.len()
        )));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

fn write_color_buffer(
    base_dir: &Path,
    reference: &BufferReference,
    frame: &ImageFrame,
) -> Result<()> {
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
        data: values
            .iter()
            .map(|value| [value.x, value.y, value.z])
            .collect(),
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
        .map(|(layer, disoccluded)| disoccluded && !matches!(*layer, SurfaceTag::ForegroundObject))
        .collect()
}
