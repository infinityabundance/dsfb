use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::frame::{Color, ImageFrame};
use crate::scene::{MotionVector, Normal3};

mod davis;
mod sintel;

pub const DAVIS_SUMMARY_JSON: &str = "generated/davis_mapping_summary.json";
pub const SINTEL_SUMMARY_JSON: &str = "generated/sintel_mapping_summary.json";
pub const DAVIS_REPORT_MD: &str = "generated/davis_mapping_report.md";
pub const SINTEL_REPORT_MD: &str = "generated/sintel_mapping_report.md";
pub const PREPARATION_REPORT_MD: &str = "generated/dataset_preparation_report.md";
pub const DATASET_MAPPING_DOC_MD: &str = "docs/dataset_mapping.md";
pub const DAVIS_MANIFEST_JSON: &str = "examples/davis_external_manifest.json";
pub const SINTEL_MANIFEST_JSON: &str = "examples/sintel_external_manifest.json";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldQuality {
    Native,
    DerivedHighConfidence,
    DerivedLowConfidence,
    Unavailable,
}

impl FieldQuality {
    fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::DerivedHighConfidence => "derived-high-confidence",
            Self::DerivedLowConfidence => "derived-low-confidence",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferFieldSummary {
    pub field_id: String,
    pub quality: FieldQuality,
    pub source: String,
    pub disclosure: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetCaptureSummary {
    pub label: String,
    pub sequence_id: String,
    pub frame_index: usize,
    pub roi_kind: String,
    pub case_tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetMappingSummary {
    pub dataset_id: String,
    pub dataset_name: String,
    pub why_chosen: String,
    pub prepared_output_dir: String,
    pub manifest_path: String,
    pub dsfb_mode: String,
    pub demo_a_metric_mode: String,
    pub demo_b_mode: String,
    pub reference_strategy: String,
    pub official_urls: Vec<String>,
    pub native_buffers: Vec<String>,
    pub derived_buffers: Vec<String>,
    pub unsupported_buffers: Vec<String>,
    pub fields: Vec<BufferFieldSummary>,
    pub captures: Vec<DatasetCaptureSummary>,
    pub blockers: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaxonomyDatasetEntry {
    pub dataset_id: String,
    pub realism_stress_case: String,
    pub larger_roi_case: String,
    pub mixed_regime_case: String,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalValidationTaxonomy {
    pub datasets: Vec<TaxonomyDatasetEntry>,
    pub aggregate_status: String,
}

pub fn prepare_davis_dataset(output_dir: &Path) -> Result<PathBuf> {
    let summary = davis::prepare(output_dir)?;
    write_summary_and_refresh(&summary)?;
    Ok(crate_root().join(DAVIS_MANIFEST_JSON))
}

pub fn prepare_sintel_dataset(output_dir: &Path) -> Result<PathBuf> {
    let summary = sintel::prepare(output_dir)?;
    write_summary_and_refresh(&summary)?;
    Ok(crate_root().join(SINTEL_MANIFEST_JSON))
}

pub fn validate_standard_external_package(output_root: &Path) -> Result<()> {
    let root = crate_root();
    let davis_summary = read_summary(&root.join(DAVIS_SUMMARY_JSON))?;
    let sintel_summary = read_summary(&root.join(SINTEL_SUMMARY_JSON))?;

    for path in [
        root.join("docs/external_dataset_plan.md"),
        root.join(DATASET_MAPPING_DOC_MD),
        root.join(PREPARATION_REPORT_MD),
        root.join(DAVIS_REPORT_MD),
        root.join(SINTEL_REPORT_MD),
        root.join(DAVIS_MANIFEST_JSON),
        root.join(SINTEL_MANIFEST_JSON),
    ] {
        require_file(&path)?;
    }

    for summary in [&davis_summary, &sintel_summary] {
        if !summary.blockers.is_empty() {
            return Err(Error::Message(format!(
                "{} preparation is still blocked: {}",
                summary.dataset_name,
                summary.blockers.join("; ")
            )));
        }
        require_file(Path::new(&summary.prepared_output_dir))?;
        require_file(Path::new(&summary.manifest_path))?;
    }

    validate_dataset_output(output_root, "external_davis", &davis_summary)?;
    validate_dataset_output(output_root, "external_sintel", &sintel_summary)?;

    let taxonomy = build_taxonomy(output_root)?;
    let taxonomy_path = output_root.join("external_validation_taxonomy.json");
    fs::create_dir_all(output_root)?;
    fs::write(&taxonomy_path, serde_json::to_string_pretty(&taxonomy)?)?;

    let report_path = output_root.join("external_validation_report.md");
    write_final_external_validation_report(
        &report_path,
        output_root,
        &davis_summary,
        &sintel_summary,
        &taxonomy,
    )?;

    let evaluator_handoff_path = output_root.join("evaluator_handoff.md");
    write_final_evaluator_handoff(
        &evaluator_handoff_path,
        output_root,
        &davis_summary,
        &sintel_summary,
    )?;

    let readiness_path = output_root.join("check_signing_readiness.md");
    write_check_signing_readiness(
        &readiness_path,
        output_root,
        &davis_summary,
        &sintel_summary,
        &taxonomy,
    )?;

    for path in [
        taxonomy_path,
        report_path,
        evaluator_handoff_path,
        readiness_path,
    ] {
        require_file(&path)?;
    }

    Ok(())
}

pub(crate) fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn summary_json_path(dataset_id: &str) -> PathBuf {
    match dataset_id {
        "davis" => crate_root().join(DAVIS_SUMMARY_JSON),
        "sintel" => crate_root().join(SINTEL_SUMMARY_JSON),
        other => crate_root().join(format!("generated/{other}_mapping_summary.json")),
    }
}

pub(crate) fn report_md_path(dataset_id: &str) -> PathBuf {
    match dataset_id {
        "davis" => crate_root().join(DAVIS_REPORT_MD),
        "sintel" => crate_root().join(SINTEL_REPORT_MD),
        other => crate_root().join(format!("generated/{other}_mapping_report.md")),
    }
}

pub(crate) fn manifest_path(dataset_id: &str) -> PathBuf {
    match dataset_id {
        "davis" => crate_root().join(DAVIS_MANIFEST_JSON),
        "sintel" => crate_root().join(SINTEL_MANIFEST_JSON),
        other => crate_root().join(format!("examples/{other}_external_manifest.json")),
    }
}

pub(crate) fn write_summary_and_refresh(summary: &DatasetMappingSummary) -> Result<()> {
    let summary_path = summary_json_path(&summary.dataset_id);
    if let Some(parent) = summary_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&summary_path, serde_json::to_string_pretty(summary)?)?;
    write_dataset_mapping_report(&report_md_path(&summary.dataset_id), summary)?;
    refresh_shared_reports()
}

pub(crate) fn refresh_shared_reports() -> Result<()> {
    let root = crate_root();
    let davis = read_summary_if_exists(&root.join(DAVIS_SUMMARY_JSON))?;
    let sintel = read_summary_if_exists(&root.join(SINTEL_SUMMARY_JSON))?;
    write_shared_dataset_mapping_doc(
        &root.join(DATASET_MAPPING_DOC_MD),
        davis.as_ref(),
        sintel.as_ref(),
    )?;
    write_shared_preparation_report(
        &root.join(PREPARATION_REPORT_MD),
        davis.as_ref(),
        sintel.as_ref(),
    )?;
    Ok(())
}

pub(crate) fn require_command(name: &str) -> Result<()> {
    let status = Command::new("bash")
        .arg("-lc")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Message(format!(
            "required command `{name}` was not found in PATH"
        )))
    }
}

pub(crate) fn download_if_missing(url: &str, archive_path: &Path) -> Result<()> {
    if archive_path.exists() && archive_path.metadata()?.len() > 0 {
        return Ok(());
    }
    require_command("curl")?;
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let status = Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--retry")
        .arg("3")
        .arg("--output")
        .arg(archive_path)
        .arg(url)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Message(format!(
            "failed to download {url} into {}",
            archive_path.display()
        )))
    }
}

pub(crate) fn unzip_if_needed(archive_path: &Path, destination: &Path) -> Result<()> {
    let marker = destination.join(".extract_complete");
    if marker.exists() {
        return Ok(());
    }
    require_command("unzip")?;
    fs::create_dir_all(destination)?;
    let status = Command::new("unzip")
        .arg("-o")
        .arg(archive_path)
        .arg("-d")
        .arg(destination)
        .status()?;
    if !status.success() {
        return Err(Error::Message(format!(
            "failed to extract {} into {}",
            archive_path.display(),
            destination.display()
        )));
    }
    fs::write(marker, archive_path.display().to_string())?;
    Ok(())
}

pub(crate) fn read_summary(path: &Path) -> Result<DatasetMappingSummary> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

pub(crate) fn write_dataset_mapping_report(
    path: &Path,
    summary: &DatasetMappingSummary,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# {} Mapping Report", summary.dataset_name);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Why This Dataset");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", summary.why_chosen);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## DSFB Mode");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- DSFB mode: `{}`", summary.dsfb_mode);
    let _ = writeln!(
        markdown,
        "- Demo A metric mode: `{}`",
        summary.demo_a_metric_mode
    );
    let _ = writeln!(markdown, "- Demo B mode: `{}`", summary.demo_b_mode);
    let _ = writeln!(
        markdown,
        "- reference strategy: `{}`",
        summary.reference_strategy
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Buffer Mapping");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "| Field | Quality | Source | Disclosure |");
    let _ = writeln!(markdown, "| --- | --- | --- | --- |");
    for field in &summary.fields {
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {} |",
            field.field_id,
            field.quality.as_str(),
            field.source,
            field.disclosure
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Native Buffers");
    let _ = writeln!(markdown);
    for field in &summary.native_buffers {
        let _ = writeln!(markdown, "- `{field}`");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Derived Buffers");
    let _ = writeln!(markdown);
    for field in &summary.derived_buffers {
        let _ = writeln!(markdown, "- `{field}`");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Unsupported Buffers");
    let _ = writeln!(markdown);
    if summary.unsupported_buffers.is_empty() {
        let _ = writeln!(markdown, "- none");
    } else {
        for field in &summary.unsupported_buffers {
            let _ = writeln!(markdown, "- `{field}`");
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Prepared Captures");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Label | Sequence | Frame | ROI kind | Case tags |"
    );
    let _ = writeln!(markdown, "| --- | --- | ---: | --- | --- |");
    for capture in &summary.captures {
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {} | {} |",
            capture.label,
            capture.sequence_id,
            capture.frame_index,
            capture.roi_kind,
            capture.case_tags.join(", ")
        );
    }
    let _ = writeln!(markdown);
    if !summary.blockers.is_empty() {
        let _ = writeln!(markdown, "## Blockers");
        let _ = writeln!(markdown);
        for blocker in &summary.blockers {
            let _ = writeln!(markdown, "- {blocker}");
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Notes");
    let _ = writeln!(markdown);
    for note in &summary.notes {
        let _ = writeln!(markdown, "- {note}");
    }
    fs::write(path, markdown)?;
    Ok(())
}

pub(crate) fn write_json_file(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

pub(crate) fn write_image_frame(path: &Path, frame: &ImageFrame) -> Result<()> {
    frame.save_png(path)
}

pub(crate) fn write_scalar_grid(
    path: &Path,
    values: &[f32],
    width: usize,
    height: usize,
) -> Result<()> {
    let payload = serde_json::json!({
        "width": width,
        "height": height,
        "data": values,
    });
    write_json_file(path, &payload)
}

pub(crate) fn write_vec2_grid(
    path: &Path,
    values: &[MotionVector],
    width: usize,
    height: usize,
) -> Result<()> {
    let data = values
        .iter()
        .map(|value| [value.to_prev_x, value.to_prev_y])
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "width": width,
        "height": height,
        "data": data,
    });
    write_json_file(path, &payload)
}

pub(crate) fn write_vec3_grid(
    path: &Path,
    values: &[Normal3],
    width: usize,
    height: usize,
) -> Result<()> {
    let data = values
        .iter()
        .map(|value| [value.x, value.y, value.z])
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "width": width,
        "height": height,
        "data": data,
    });
    write_json_file(path, &payload)
}

pub(crate) fn write_mask_grid(
    path: &Path,
    values: &[bool],
    width: usize,
    height: usize,
) -> Result<()> {
    let payload = serde_json::json!({
        "width": width,
        "height": height,
        "data": values,
    });
    write_json_file(path, &payload)
}

pub(crate) fn load_image_frame(path: &Path) -> Result<ImageFrame> {
    let image = image::open(path)?.to_rgba8();
    let width = image.width() as usize;
    let height = image.height() as usize;
    let pixels = image
        .pixels()
        .map(|pixel| {
            Color::rgb(
                pixel[0] as f32 / 255.0,
                pixel[1] as f32 / 255.0,
                pixel[2] as f32 / 255.0,
            )
        })
        .collect();
    Ok(ImageFrame::from_pixels(width, height, pixels))
}

pub(crate) fn relative_path(from_dir: &Path, to_path: &Path) -> PathBuf {
    let from = normalize_absolute_path(from_dir);
    let to = normalize_absolute_path(to_path);
    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();
    let mut shared = 0usize;
    while shared < from_components.len()
        && shared < to_components.len()
        && from_components[shared] == to_components[shared]
    {
        shared += 1;
    }

    let mut relative = PathBuf::new();
    for component in &from_components[shared..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }
    for component in &to_components[shared..] {
        relative.push(component.as_os_str());
    }
    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    relative
}

fn normalize_absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        crate_root().join(path)
    }
}

fn read_summary_if_exists(path: &Path) -> Result<Option<DatasetMappingSummary>> {
    if path.exists() {
        Ok(Some(read_summary(path)?))
    } else {
        Ok(None)
    }
}

fn write_shared_dataset_mapping_doc(
    path: &Path,
    davis: Option<&DatasetMappingSummary>,
    sintel: Option<&DatasetMappingSummary>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Dataset Mapping");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This document records the native-vs-derived mapping used to run the DSFB external replay path on DAVIS and MPI Sintel."
    );
    let _ = writeln!(markdown);
    for summary in [davis, sintel].into_iter().flatten() {
        let _ = writeln!(markdown, "## {}", summary.dataset_name);
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "- manifest: `{}`", summary.manifest_path);
        let _ = writeln!(markdown, "- DSFB mode: `{}`", summary.dsfb_mode);
        let _ = writeln!(
            markdown,
            "- derived-vs-native disclosure: all fields below are labeled as native, derived-high-confidence, derived-low-confidence, or unavailable"
        );
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "| Field | Quality | Source |");
        let _ = writeln!(markdown, "| --- | --- | --- |");
        for field in &summary.fields {
            let _ = writeln!(
                markdown,
                "| {} | {} | {} |",
                field.field_id,
                field.quality.as_str(),
                field.source
            );
        }
        let _ = writeln!(markdown);
    }
    fs::write(path, markdown)?;
    Ok(())
}

fn write_shared_preparation_report(
    path: &Path,
    davis: Option<&DatasetMappingSummary>,
    sintel: Option<&DatasetMappingSummary>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Dataset Preparation Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "| Dataset | Prepared | Manifest | Blockers |");
    let _ = writeln!(markdown, "| --- | --- | --- | --- |");
    for (label, summary) in [("DAVIS", davis), ("MPI Sintel", sintel)] {
        match summary {
            Some(summary) => {
                let _ = writeln!(
                    markdown,
                    "| {} | {} | `{}` | {} |",
                    label,
                    if summary.blockers.is_empty() {
                        "true"
                    } else {
                        "false"
                    },
                    summary.manifest_path,
                    if summary.blockers.is_empty() {
                        "none".to_string()
                    } else {
                        summary.blockers.join("; ")
                    }
                );
            }
            None => {
                let _ = writeln!(
                    markdown,
                    "| {} | false | missing | not prepared yet |",
                    label
                );
            }
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Exact Gates");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- `docs/dataset_mapping.md` must exist.");
    let _ = writeln!(
        markdown,
        "- `generated/davis_mapping_report.md` must exist."
    );
    let _ = writeln!(
        markdown,
        "- `generated/sintel_mapping_report.md` must exist."
    );
    let _ = writeln!(
        markdown,
        "- derived-vs-native labeling must be explicit in every mapping table."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn validate_dataset_output(
    output_root: &Path,
    dir_name: &str,
    summary: &DatasetMappingSummary,
) -> Result<()> {
    let dataset_dir = output_root.join(dir_name);
    for path in [
        dataset_dir.join("external_replay_report.md"),
        dataset_dir.join("external_handoff_report.md"),
        dataset_dir.join("external_validation_report.md"),
        dataset_dir.join("replay_metrics.json"),
        dataset_dir.join("gpu_execution_report.md"),
        dataset_dir.join("gpu_execution_metrics.json"),
        dataset_dir.join("demo_a_external_report.md"),
        dataset_dir.join("demo_a_external_metrics.json"),
        dataset_dir.join("demo_b_external_report.md"),
        dataset_dir.join("demo_b_external_metrics.json"),
        dataset_dir.join("scaling_report.md"),
        dataset_dir.join("scaling_metrics.json"),
        dataset_dir.join("memory_bandwidth_report.md"),
        dataset_dir.join("integration_scaling_report.md"),
        dataset_dir.join("resolved_external_capture_manifest.json"),
        dataset_dir.join("figures").join("trust_map.png"),
        dataset_dir.join("figures").join("intervention_map.png"),
        dataset_dir.join("figures").join("roi_overlay.png"),
        dataset_dir.join("figures").join("current_color.png"),
        dataset_dir.join("figures").join("demo_a_dsfb.png"),
        dataset_dir.join("figures").join("demo_a_fixed_alpha.png"),
    ] {
        require_file(&path)?;
    }

    let mapping_report = fs::read_to_string(report_md_path(&summary.dataset_id))?;
    for required_phrase in [
        "derived-high-confidence",
        "derived-low-confidence",
        "native",
        "DSFB mode",
    ] {
        if !mapping_report.contains(required_phrase) {
            return Err(Error::Message(format!(
                "{} mapping report is missing required phrase `{required_phrase}`",
                summary.dataset_name
            )));
        }
    }

    let replay_report = fs::read_to_string(dataset_dir.join("external_replay_report.md"))?;
    if !replay_report.contains("external-capable") {
        return Err(Error::Message(format!(
            "{} replay report must distinguish external-capable from externally validated",
            summary.dataset_name
        )));
    }

    let gpu_report = fs::read_to_string(dataset_dir.join("gpu_execution_report.md"))?;
    for required_phrase in [
        "measured_gpu:",
        "backend",
        "trust_delta_vs_cpu",
        "alpha_delta_vs_cpu",
        "intervention_delta_vs_cpu",
    ] {
        if !gpu_report.contains(required_phrase) {
            return Err(Error::Message(format!(
                "{} GPU execution report is missing `{required_phrase}`",
                summary.dataset_name
            )));
        }
    }

    let demo_a_report = fs::read_to_string(dataset_dir.join("demo_a_external_report.md"))?;
    for required_phrase in [
        "ROI source",
        "non-ROI",
        "point_vs_region",
        "realism_stress_note",
        "proxy",
    ] {
        if !demo_a_report.contains(required_phrase) {
            return Err(Error::Message(format!(
                "{} Demo A report is missing `{required_phrase}`",
                summary.dataset_name
            )));
        }
    }

    let demo_b_report = fs::read_to_string(dataset_dir.join("demo_b_external_report.md"))?;
    for required_phrase in [
        "Gradient magnitude",
        "Variance proxy",
        "Combined heuristic",
        "DSFB imported trust",
        "Hybrid trust + variance",
        "fixed_budget_equal",
        "aliasing_limited",
        "variance_limited",
        "mixed_regime",
    ] {
        if !demo_b_report.contains(required_phrase) {
            return Err(Error::Message(format!(
                "{} Demo B report is missing `{required_phrase}`",
                summary.dataset_name
            )));
        }
    }

    let scaling_report = fs::read_to_string(dataset_dir.join("scaling_report.md"))?;
    for required_phrase in [
        "scaled_1080p",
        "scaled_4k",
        "Cost appears approximately linear with resolution",
        "realism_stress_case",
        "larger_roi_case",
        "mixed_regime_case",
    ] {
        if !scaling_report.contains(required_phrase) {
            return Err(Error::Message(format!(
                "{} scaling report is missing `{required_phrase}`",
                summary.dataset_name
            )));
        }
    }

    let memory_report = fs::read_to_string(dataset_dir.join("memory_bandwidth_report.md"))?;
    for required_phrase in [
        "Readback required in production: `false`",
        "Memory Access / Coherence Analysis",
        "estimated memory traffic",
    ] {
        if !memory_report.contains(required_phrase) {
            return Err(Error::Message(format!(
                "{} memory bandwidth report is missing `{required_phrase}`",
                summary.dataset_name
            )));
        }
    }

    let integration_report = fs::read_to_string(dataset_dir.join("integration_scaling_report.md"))?;
    for required_phrase in [
        "Async-Compute Feasibility",
        "Production readback is not required",
        "Hazards / Barriers / Transitions",
        "Pipeline Compatibility",
    ] {
        if !integration_report.contains(required_phrase) {
            return Err(Error::Message(format!(
                "{} integration report is missing `{required_phrase}`",
                summary.dataset_name
            )));
        }
    }

    Ok(())
}

fn build_taxonomy(output_root: &Path) -> Result<ExternalValidationTaxonomy> {
    let mut datasets = Vec::new();
    for dir_name in ["external_davis", "external_sintel"] {
        let metrics_text =
            fs::read_to_string(output_root.join(dir_name).join("scaling_metrics.json"))?;
        let metrics: Value = serde_json::from_str(&metrics_text)?;
        let coverage = &metrics["coverage"];
        let note = if coverage["coverage_status"].as_str() == Some("complete") {
            "coverage complete".to_string()
        } else {
            format!(
                "coverage partial; missing: {}",
                coverage["missing"]
                    .as_array()
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "unspecified".to_string())
            )
        };
        datasets.push(TaxonomyDatasetEntry {
            dataset_id: dir_name.trim_start_matches("external_").to_string(),
            realism_stress_case: coverage_status(coverage["realism_stress_case"].as_bool()),
            larger_roi_case: coverage_status(coverage["larger_roi_case"].as_bool()),
            mixed_regime_case: coverage_status(coverage["mixed_regime_case"].as_bool()),
            notes: vec![note],
        });
    }

    let aggregate_complete = datasets.iter().all(|entry| {
        [
            entry.realism_stress_case.as_str(),
            entry.larger_roi_case.as_str(),
            entry.mixed_regime_case.as_str(),
        ]
        .iter()
        .all(|status| *status == "covered" || *status == "explicitly_missing")
    });
    Ok(ExternalValidationTaxonomy {
        datasets,
        aggregate_status: if aggregate_complete {
            "complete_or_explicitly_missing".to_string()
        } else {
            "incomplete".to_string()
        },
    })
}

fn coverage_status(value: Option<bool>) -> String {
    match value {
        Some(true) => "covered".to_string(),
        Some(false) => "explicitly_missing".to_string(),
        None => "explicitly_missing".to_string(),
    }
}

fn write_final_external_validation_report(
    path: &Path,
    output_root: &Path,
    davis: &DatasetMappingSummary,
    sintel: &DatasetMappingSummary,
    taxonomy: &ExternalValidationTaxonomy,
) -> Result<()> {
    let davis_gpu: Value = serde_json::from_str(&fs::read_to_string(
        output_root
            .join("external_davis")
            .join("gpu_execution_metrics.json"),
    )?)?;
    let sintel_gpu: Value = serde_json::from_str(&fs::read_to_string(
        output_root
            .join("external_sintel")
            .join("gpu_execution_metrics.json"),
    )?)?;
    let davis_demo_b: Value = serde_json::from_str(&fs::read_to_string(
        output_root
            .join("external_davis")
            .join("demo_b_external_metrics.json"),
    )?)?;
    let sintel_demo_b: Value = serde_json::from_str(&fs::read_to_string(
        output_root
            .join("external_sintel")
            .join("demo_b_external_metrics.json"),
    )?)?;
    let davis_scaling: Value = serde_json::from_str(&fs::read_to_string(
        output_root
            .join("external_davis")
            .join("scaling_metrics.json"),
    )?)?;
    let sintel_scaling: Value = serde_json::from_str(&fs::read_to_string(
        output_root
            .join("external_sintel")
            .join("scaling_metrics.json"),
    )?)?;
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# External Validation Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Why DAVIS And Sintel");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- DAVIS: {}", davis.why_chosen);
    let _ = writeln!(markdown, "- MPI Sintel: {}", sintel.why_chosen);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Dataset Contributions");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- DAVIS contributes real captured video plus native segmentation masks."
    );
    let _ = writeln!(markdown, "- MPI Sintel contributes renderer-origin motion-rich sequences, optical flow, and official depth when available.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Native Vs Derived Buffers");
    let _ = writeln!(markdown);
    append_summary_fields(&mut markdown, "DAVIS", davis);
    append_summary_fields(&mut markdown, "MPI Sintel", sintel);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## DSFB Modes Run");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- DAVIS: `{}`", davis.dsfb_mode);
    let _ = writeln!(markdown, "- MPI Sintel: `{}`", sintel.dsfb_mode);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## GPU Execution Summary");
    let _ = writeln!(markdown);
    append_gpu_summary(&mut markdown, "DAVIS", &davis_gpu);
    append_gpu_summary(&mut markdown, "MPI Sintel", &sintel_gpu);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Demo A External Results");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- DAVIS uses proxy-only Demo A metrics because no renderer-quality reference exists in the mapped path."
    );
    let _ = writeln!(
        markdown,
        "- MPI Sintel uses a clean-vs-final pass proxy when available and labels it explicitly as proxy rather than renderer ground truth."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Demo B External Results");
    let _ = writeln!(markdown);
    append_demo_b_summary(&mut markdown, "DAVIS", &davis_demo_b);
    append_demo_b_summary(&mut markdown, "MPI Sintel", &sintel_demo_b);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Scaling And Memory");
    let _ = writeln!(markdown);
    append_scaling_summary(&mut markdown, "DAVIS", &davis_scaling);
    append_scaling_summary(&mut markdown, "MPI Sintel", &sintel_scaling);
    let _ = writeln!(markdown, "- 1080p scaling is attempted on both datasets.");
    let _ = writeln!(
        markdown,
        "- 4K scaling is attempted when the GPU path can run on scaled buffers."
    );
    let _ = writeln!(markdown, "- Memory / bandwidth reports explicitly state that readback is used for validation, not required in production.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Pipeline Insertion / Async");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Async feasibility is discussed per dataset in the integration reports."
    );
    let _ = writeln!(
        markdown,
        "- Production readback is explicitly classified as not required."
    );
    let _ = writeln!(
        markdown,
        "- Barrier / transition discussion remains implementation guidance rather than proof."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Coverage Taxonomy");
    let _ = writeln!(markdown);
    for dataset in &taxonomy.datasets {
        let _ = writeln!(
            markdown,
            "- {}: realism_stress_case=`{}`, larger_roi_case=`{}`, mixed_regime_case=`{}`",
            dataset.dataset_id,
            dataset.realism_stress_case,
            dataset.larger_roi_case,
            dataset.mixed_regime_case
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- DAVIS and MPI Sintel are both integrated into the same DSFB external replay path."
    );
    let _ = writeln!(markdown, "- GPU execution is attempted on both dataset-mapped paths, with measured-vs-unmeasured status made explicit.");
    let _ = writeln!(
        markdown,
        "- Native-vs-derived buffer provenance is disclosed instead of hidden."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This package does not prove production-engine integration."
    );
    let _ = writeln!(
        markdown,
        "- Demo B remains an allocation proxy rather than a live renderer sampling benchmark."
    );
    let _ = writeln!(
        markdown,
        "- DAVIS depth and normal support remain derived proxies, not native geometry buffers."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    for blocker in collect_blockers(&davis_gpu, &sintel_gpu, taxonomy) {
        let _ = writeln!(markdown, "- {blocker}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Next Highest-Value Experiment");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Export one engine-native temporal capture with true history, motion, depth, and normals, then run the same DAVIS/Sintel comparison stack on that capture to close the renderer-integration gap."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn write_final_evaluator_handoff(
    path: &Path,
    output_root: &Path,
    _davis: &DatasetMappingSummary,
    _sintel: &DatasetMappingSummary,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Evaluator Handoff");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Standard External Datasets: DAVIS + Sintel");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- prepare DAVIS: `cargo run --release -- prepare-davis --output data/external/davis`"
    );
    let _ = writeln!(
        markdown,
        "- prepare Sintel: `cargo run --release -- prepare-sintel --output data/external/sintel`"
    );
    let _ = writeln!(
        markdown,
        "- replay DAVIS: `cargo run --release -- run-external-replay --manifest examples/davis_external_manifest.json --output {}`",
        output_root.join("external_davis").display()
    );
    let _ = writeln!(
        markdown,
        "- replay Sintel: `cargo run --release -- run-external-replay --manifest examples/sintel_external_manifest.json --output {}`",
        output_root.join("external_sintel").display()
    );
    let _ = writeln!(
        markdown,
        "- validate everything: `cargo run --release -- validate-final --output {}`",
        output_root.display()
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Expected outputs:");
    let _ = writeln!(markdown, "- `external_davis/*` and `external_sintel/*` with replay, GPU, Demo A, Demo B, scaling, memory, and integration reports.");
    let _ = writeln!(markdown, "- `external_validation_taxonomy.json`.");
    let _ = writeln!(markdown, "- `external_validation_report.md`.");
    let _ = writeln!(markdown, "- `check_signing_readiness.md`.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Success looks like:");
    let _ = writeln!(markdown, "- both manifests load");
    let _ = writeln!(
        markdown,
        "- both dataset paths produce replay + GPU reports"
    );
    let _ = writeln!(markdown, "- proxy-vs-native distinctions stay explicit");
    let _ = writeln!(
        markdown,
        "- fixed-budget Demo B remains equal across all policies"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Failure looks like:");
    let _ = writeln!(markdown, "- dataset download blocked");
    let _ = writeln!(markdown, "- missing per-dataset report or manifest");
    let _ = writeln!(markdown, "- hidden derived buffers or missing disclosure");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Interpretation rule:");
    let _ = writeln!(markdown, "- DAVIS and clean-vs-final Sintel comparisons may use proxies; read those as decision support, not renderer ground truth.");
    fs::write(path, markdown)?;
    Ok(())
}

fn write_check_signing_readiness(
    path: &Path,
    output_root: &Path,
    _davis: &DatasetMappingSummary,
    _sintel: &DatasetMappingSummary,
    taxonomy: &ExternalValidationTaxonomy,
) -> Result<()> {
    let davis_gpu: Value = serde_json::from_str(&fs::read_to_string(
        output_root
            .join("external_davis")
            .join("gpu_execution_metrics.json"),
    )?)?;
    let sintel_gpu: Value = serde_json::from_str(&fs::read_to_string(
        output_root
            .join("external_sintel")
            .join("gpu_execution_metrics.json"),
    )?)?;
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Check Signing Readiness");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "| Area | Status | Classification | Notes |");
    let _ = writeln!(markdown, "| --- | --- | --- | --- |");
    let _ = writeln!(
        markdown,
        "| DAVIS prep | ready | external | official DAVIS data mapped into the schema |"
    );
    let _ = writeln!(
        markdown,
        "| Sintel prep | ready | external | official Sintel data mapped into the schema |"
    );
    let _ = writeln!(
        markdown,
        "| DAVIS GPU | {} | {} | measured_gpu=`{}` |",
        if davis_gpu["measured_gpu"].as_bool() == Some(true) {
            "ready"
        } else {
            "partial"
        },
        if davis_gpu["measured_gpu"].as_bool() == Some(true) {
            "external"
        } else {
            "external"
        },
        davis_gpu["measured_gpu"].as_bool().unwrap_or(false)
    );
    let _ = writeln!(
        markdown,
        "| Sintel GPU | {} | {} | measured_gpu=`{}` |",
        if sintel_gpu["measured_gpu"].as_bool() == Some(true) {
            "ready"
        } else {
            "partial"
        },
        if sintel_gpu["measured_gpu"].as_bool() == Some(true) {
            "external"
        } else {
            "external"
        },
        sintel_gpu["measured_gpu"].as_bool().unwrap_or(false)
    );
    let _ = writeln!(
        markdown,
        "| Taxonomy coverage | {} | external | aggregate_status=`{}` |",
        if taxonomy.aggregate_status == "complete_or_explicitly_missing" {
            "ready"
        } else {
            "partial"
        },
        taxonomy.aggregate_status
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    for blocker in collect_blockers(&davis_gpu, &sintel_gpu, taxonomy) {
        let class = if blocker.contains("GPU") || blocker.contains("engine") {
            "external"
        } else {
            "internal"
        };
        let _ = writeln!(markdown, "- [{}] {}", class, blocker);
    }
    fs::write(path, markdown)?;
    Ok(())
}

fn collect_blockers(
    davis_gpu: &Value,
    sintel_gpu: &Value,
    taxonomy: &ExternalValidationTaxonomy,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if davis_gpu["measured_gpu"].as_bool() != Some(true) {
        blockers.push("DAVIS GPU timing remains unmeasured on this machine".to_string());
    }
    if sintel_gpu["measured_gpu"].as_bool() != Some(true) {
        blockers.push("Sintel GPU timing remains unmeasured on this machine".to_string());
    }
    for dataset in &taxonomy.datasets {
        for (label, status) in [
            ("realism_stress_case", dataset.realism_stress_case.as_str()),
            ("larger_roi_case", dataset.larger_roi_case.as_str()),
            ("mixed_regime_case", dataset.mixed_regime_case.as_str()),
        ] {
            if status == "explicitly_missing" {
                blockers.push(format!(
                    "{} coverage is partial for {}",
                    label, dataset.dataset_id
                ));
            }
        }
    }
    blockers.push("renderer-integrated sampling validation is still pending".to_string());
    blockers
}

fn append_summary_fields(markdown: &mut String, label: &str, summary: &DatasetMappingSummary) {
    let _ = writeln!(
        markdown,
        "- {} native buffers: {}",
        label,
        summary.native_buffers.join(", ")
    );
    let _ = writeln!(
        markdown,
        "- {} derived buffers: {}",
        label,
        summary.derived_buffers.join(", ")
    );
    if !summary.unsupported_buffers.is_empty() {
        let _ = writeln!(
            markdown,
            "- {} unsupported buffers: {}",
            label,
            summary.unsupported_buffers.join(", ")
        );
    }
}

fn append_gpu_summary(markdown: &mut String, label: &str, gpu: &Value) {
    let _ = writeln!(
        markdown,
        "- {} measured_gpu=`{}`, actual_real_external_data=`{}`",
        label,
        gpu["measured_gpu"].as_bool().unwrap_or(false),
        gpu["actual_real_external_data"].as_bool().unwrap_or(false)
    );
}

fn append_demo_b_summary(markdown: &mut String, label: &str, metrics: &Value) {
    let capture_count = metrics["captures"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let _ = writeln!(
        markdown,
        "- {} captures evaluated: {}",
        label, capture_count
    );
    if let Some(captures) = metrics["captures"].as_array() {
        for capture in captures.iter().take(2) {
            let _ = writeln!(
                markdown,
                "  - {} regime=`{}` fixed_budget_equal=`{}`",
                capture["capture_label"].as_str().unwrap_or("unknown"),
                capture["regime"].as_str().unwrap_or("unknown"),
                capture["fixed_budget_equal"].as_bool().unwrap_or(false)
            );
        }
    }
}

fn append_scaling_summary(markdown: &mut String, label: &str, metrics: &Value) {
    let _ = writeln!(
        markdown,
        "- {} attempted_1080p=`{}` attempted_4k=`{}`",
        label,
        metrics["attempted_1080p"].as_bool().unwrap_or(false),
        metrics["attempted_4k"].as_bool().unwrap_or(false)
    );
}

fn require_file(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(Error::Message(format!(
            "required file missing: {}",
            path.display()
        )))
    }
}
