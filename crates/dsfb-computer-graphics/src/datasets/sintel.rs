use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::datasets::{
    crate_root, download_if_missing, load_image_frame, manifest_path, relative_path,
    report_md_path, summary_json_path, unzip_if_needed, write_image_frame, write_json_file,
    write_mask_grid, write_scalar_grid, write_summary_and_refresh, write_vec2_grid,
    write_vec3_grid, BufferFieldSummary, DatasetCaptureSummary, DatasetMappingSummary,
    FieldQuality,
};
use crate::error::{Error, Result};
use crate::external::{
    BufferReference, ExternalBufferSet, ExternalCaptureEntry, ExternalCaptureManifest,
    ExternalCaptureMetadata, ExternalCaptureSource, ExternalNormalization,
    EXTERNAL_CAPTURE_FORMAT_VERSION,
};
use crate::frame::ImageFrame;
use crate::scene::{MotionVector, Normal3};

const SINTEL_COMPLETE_URL: &str = "http://files.is.tue.mpg.de/sintel/MPI-Sintel-complete.zip";
const SINTEL_DEPTH_URL: &str =
    "http://files.is.tue.mpg.de/jwulff/sintel/MPI-Sintel-depth-training-20150305.zip";
const PREFERRED_SEQUENCES: [&str; 4] = ["ambush_5", "market_6", "alley_2", "cave_4"];

pub(crate) fn prepare(output_dir: &Path) -> Result<DatasetMappingSummary> {
    let root = crate_root();
    let output_dir = if output_dir.is_absolute() {
        output_dir.to_path_buf()
    } else {
        root.join(output_dir)
    };

    let summary = match prepare_inner(&output_dir) {
        Ok(summary) => summary,
        Err(error) => {
            let blocker_summary = blocked_summary(&output_dir, error.to_string());
            write_summary_and_refresh(&blocker_summary)?;
            return Err(error);
        }
    };
    Ok(summary)
}

fn prepare_inner(output_dir: &Path) -> Result<DatasetMappingSummary> {
    let downloads_dir = output_dir.join("downloads");
    let complete_archive = downloads_dir.join("MPI-Sintel-complete.zip");
    let depth_archive = downloads_dir.join("MPI-Sintel-depth-training-20150305.zip");
    download_if_missing(SINTEL_COMPLETE_URL, &complete_archive)?;
    download_if_missing(SINTEL_DEPTH_URL, &depth_archive)?;

    let source_dir = output_dir.join("source");
    unzip_if_needed(&complete_archive, &source_dir)?;
    unzip_if_needed(&depth_archive, &source_dir)?;

    let final_root =
        find_path_ending_with(&source_dir, Path::new("training").join("final").as_path())?;
    let clean_root =
        find_path_ending_with(&source_dir, Path::new("training").join("clean").as_path())?;
    let flow_root =
        find_path_ending_with(&source_dir, Path::new("training").join("flow").as_path())?;
    let depth_root =
        find_path_ending_with(&source_dir, Path::new("training").join("depth").as_path())?;

    let available_sequences = list_sequence_dirs(&final_root)?;
    if available_sequences.is_empty() {
        return Err(Error::Message(
            "Sintel extraction succeeded, but no training/final sequences were found".to_string(),
        ));
    }
    let selected_sequences = choose_sequences(&available_sequences);
    let manifest_dir = manifest_path("sintel")
        .parent()
        .ok_or_else(|| Error::Message("Sintel manifest path had no parent".to_string()))?
        .to_path_buf();

    let mut captures = Vec::new();
    let mut capture_summaries = Vec::new();
    for (sequence_index, sequence_id) in selected_sequences.iter().enumerate() {
        let prepared = prepare_sequence_variants(
            output_dir,
            &manifest_dir,
            sequence_id,
            &final_root.join(sequence_id),
            &clean_root.join(sequence_id),
            &flow_root.join(sequence_id),
            &depth_root.join(sequence_id),
            sequence_index == 0,
        )?;
        capture_summaries.extend(prepared.iter().map(|item| item.summary.clone()));
        captures.extend(prepared.into_iter().map(|item| item.entry));
    }

    let manifest = ExternalCaptureManifest {
        format_version: EXTERNAL_CAPTURE_FORMAT_VERSION.to_string(),
        description: "Official MPI Sintel captures mapped into the DSFB external replay schema with native flow and depth plus derived current-grid history buffers.".to_string(),
        source: ExternalCaptureSource::Files,
        buffers: None,
        captures,
        normalization: ExternalNormalization {
            color: "official Sintel PNG frames converted to normalized RGB in [0,1]".to_string(),
            motion_vectors: "current-grid backward motion vectors derived by inverting official forward flow; derived-high-confidence".to_string(),
            depth: "official Sintel depth for the current frame, with previous depth reprojected onto the current frame".to_string(),
            normals: "derived from official depth; derived-high-confidence".to_string(),
        },
        notes: vec![
            "Sintel provides native renderer-origin color, native forward optical flow, and native depth when the official depth archive is available.".to_string(),
            "Current-grid backward motion vectors are derived by splatting / inverting the official forward flow.".to_string(),
            "The clean pass is used only as an explicit proxy reference when final-pass inputs are evaluated.".to_string(),
        ],
    };
    write_json_file(&manifest_path("sintel"), &manifest)?;

    Ok(DatasetMappingSummary {
        dataset_id: "sintel".to_string(),
        dataset_name: "MPI Sintel".to_string(),
        why_chosen: "MPI Sintel is a standard, instantly recognizable renderer-origin motion benchmark with optical flow and official depth. It grounds the external validation package in motion-rich, renderer-like data serious graphics reviewers already know.".to_string(),
        prepared_output_dir: output_dir.display().to_string(),
        manifest_path: manifest_path("sintel").display().to_string(),
        dsfb_mode: "host_minimum_with_native_final_pass_color_native_depth_and_flow_derived_current_grid_history".to_string(),
        demo_a_metric_mode: "clean_vs_final_proxy_when_reference_pass_is_present".to_string(),
        demo_b_mode: "fixed_budget_allocation_proxy_with_derived_motion_boundary_roi".to_string(),
        reference_strategy: "current clean pass as explicit proxy reference for final-pass inputs".to_string(),
        official_urls: vec![SINTEL_COMPLETE_URL.to_string(), SINTEL_DEPTH_URL.to_string()],
        native_buffers: vec![
            "current_color".to_string(),
            "optical_flow_forward".to_string(),
            "current_depth".to_string(),
        ],
        derived_buffers: vec![
            "history_color".to_string(),
            "motion_vectors".to_string(),
            "history_depth".to_string(),
            "current_normals".to_string(),
            "history_normals".to_string(),
            "roi_mask".to_string(),
            "reference_proxy".to_string(),
        ],
        unsupported_buffers: vec!["renderer_ground_truth".to_string()],
        fields: vec![
            field("current_color", FieldQuality::Native, "official Sintel final-pass frame", "native renderer-origin input frame"),
            field("history_color", FieldQuality::DerivedHighConfidence, "previous final-pass frame reprojected onto the current frame", "derived from native adjacent frames plus inverted official flow"),
            field("motion_vectors", FieldQuality::DerivedHighConfidence, "current-grid backward flow derived from official forward flow", "native flow exists, but the current-grid backward field is derived for this schema"),
            field("current_depth", FieldQuality::Native, "official Sintel depth archive", "native depth"),
            field("history_depth", FieldQuality::DerivedHighConfidence, "previous native depth reprojected onto the current frame", "derived from native depth plus inverted official flow"),
            field("current_normals", FieldQuality::DerivedHighConfidence, "depth-gradient normals from native depth", "derived from the official depth field"),
            field("history_normals", FieldQuality::DerivedHighConfidence, "previous depth-gradient normals reprojected onto the current frame", "derived from native depth plus inverted official flow"),
            field("roi_mask", FieldQuality::DerivedLowConfidence, "motion-boundary / depth-discontinuity support mask", "no native ROI exists in Sintel, so ROI support is explicit derived logic"),
            field("reference", FieldQuality::DerivedLowConfidence, "current clean pass used as a proxy reference for final-pass input", "clean-vs-final is a renderer-like proxy, not temporal ground truth"),
        ],
        captures: capture_summaries,
        blockers: Vec::new(),
        notes: vec![
            format!("Sintel mapping report: {}", report_md_path("sintel").display()),
            format!("Sintel summary JSON: {}", summary_json_path("sintel").display()),
            "Depth is required here; if the official depth archive cannot be downloaded, preparation fails loudly.".to_string(),
        ],
    })
}

#[derive(Clone)]
struct PreparedCapture {
    entry: ExternalCaptureEntry,
    summary: DatasetCaptureSummary,
}

fn prepare_sequence_variants(
    output_dir: &Path,
    manifest_dir: &Path,
    sequence_id: &str,
    final_dir: &Path,
    clean_dir: &Path,
    flow_dir: &Path,
    depth_dir: &Path,
    emit_point_and_region_pair: bool,
) -> Result<Vec<PreparedCapture>> {
    let frame_paths = sorted_files(final_dir, "png")?;
    let clean_paths = sorted_files(clean_dir, "png")?;
    let flow_paths = sorted_files(flow_dir, "flo")?;
    let depth_paths = sorted_files(depth_dir, "dpt")?;
    if frame_paths.len() < 2
        || clean_paths.is_empty()
        || flow_paths.is_empty()
        || depth_paths.len() < 2
    {
        return Err(Error::Message(format!(
            "Sintel sequence {} does not have enough frames / flow / depth for mapping",
            sequence_id
        )));
    }

    let best_prev_index = select_best_flow_index(&flow_paths)?;
    let current_index = (best_prev_index + 1).min(frame_paths.len().saturating_sub(1));
    let previous_final = load_image_frame(&frame_paths[best_prev_index])?;
    let current_final = load_image_frame(&frame_paths[current_index])?;
    let current_clean =
        load_image_frame(&clean_paths[current_index.min(clean_paths.len().saturating_sub(1))])?;
    let flow_forward = load_flo(&flow_paths[best_prev_index])?;
    let current_depth =
        load_dpt(&depth_paths[current_index.min(depth_paths.len().saturating_sub(1))])?;
    let previous_depth =
        load_dpt(&depth_paths[best_prev_index.min(depth_paths.len().saturating_sub(1))])?;

    let width = current_final.width();
    let height = current_final.height();
    if flow_forward.len() != width * height || current_depth.len() != width * height {
        return Err(Error::Message(format!(
            "Sintel sequence {} produced mismatched flow/depth extent",
            sequence_id
        )));
    }

    let motion = invert_forward_flow(&flow_forward, width, height);
    let history_color = warp_image(&previous_final, &motion, width, height);
    let history_depth = warp_scalar(&previous_depth, &motion, width, height);
    let current_normals = derive_normals(&current_depth, width, height);
    let previous_normals = derive_normals(&previous_depth, width, height);
    let history_normals = warp_normals(&previous_normals, &motion, width, height);
    let roi_score = build_roi_score(
        &motion,
        &current_depth,
        &current_final,
        &current_clean,
        width,
        height,
    );

    let mut captures = Vec::new();
    let mut variants = vec![("mixed", 0.10f32, "derived_mixed_roi".to_string())];
    if emit_point_and_region_pair {
        variants.push(("point", 0.03, "derived_point_roi".to_string()));
        variants.push(("region", 0.18, "derived_region_roi".to_string()));
    } else {
        variants.push(("region", 0.15, "derived_region_roi".to_string()));
    }

    for (variant_id, percentile, roi_kind) in variants {
        let mask = percentile_mask(&roi_score, percentile);
        let capture_label = format!("{sequence_id}_{variant_id}_frame_{current_index:04}");
        let capture_dir = output_dir.join("captures").join(&capture_label);
        fs::create_dir_all(&capture_dir)?;

        let current_color_path = capture_dir.join("current_color.png");
        let history_color_path = capture_dir.join("history_color.png");
        let motion_path = capture_dir.join("motion_vectors.json");
        let current_depth_path = capture_dir.join("current_depth.json");
        let history_depth_path = capture_dir.join("history_depth.json");
        let current_normals_path = capture_dir.join("current_normals.json");
        let history_normals_path = capture_dir.join("history_normals.json");
        let mask_path = capture_dir.join("roi_mask.json");
        let reference_path = capture_dir.join("reference_proxy.png");
        let variance_path = capture_dir.join("variance_proxy.json");
        let metadata_path = capture_dir.join("metadata.json");

        write_image_frame(&current_color_path, &current_final)?;
        write_image_frame(&history_color_path, &history_color)?;
        write_vec2_grid(&motion_path, &motion, width, height)?;
        write_scalar_grid(&current_depth_path, &current_depth, width, height)?;
        write_scalar_grid(&history_depth_path, &history_depth, width, height)?;
        write_vec3_grid(&current_normals_path, &current_normals, width, height)?;
        write_vec3_grid(&history_normals_path, &history_normals, width, height)?;
        write_mask_grid(&mask_path, &mask, width, height)?;
        write_image_frame(&reference_path, &current_clean)?;
        write_scalar_grid(
            &variance_path,
            &temporal_difference_proxy(&current_final, &current_clean),
            width,
            height,
        )?;

        let metadata = ExternalCaptureMetadata {
            scenario_id: None,
            frame_index: current_index,
            history_frame_index: best_prev_index,
            width,
            height,
            source_kind: "mpi_sintel_final_pass".to_string(),
            externally_validated: true,
            real_external_data: true,
            data_description: Some(
                "Official MPI Sintel frame pair mapped into the DSFB external schema".to_string(),
            ),
            notes: vec![
                format!("sequence_id={sequence_id}"),
                "Current color is native Sintel final pass.".to_string(),
                "Motion vectors are derived by inverting official forward flow onto the current pixel grid.".to_string(),
                "Depth is native Sintel depth; normals are derived from depth.".to_string(),
                "ROI support is derived from motion-boundary and depth-discontinuity structure because Sintel provides no native ROI.".to_string(),
                "The clean pass is written as a proxy reference only and is not claimed as renderer ground truth.".to_string(),
            ],
        };
        write_json_file(&metadata_path, &metadata)?;

        let entry = ExternalCaptureEntry {
            label: capture_label.clone(),
            buffers: ExternalBufferSet {
                current_color: buffer_ref(
                    manifest_dir,
                    &current_color_path,
                    "png_rgb8",
                    "native Sintel final-pass frame",
                ),
                reprojected_history: buffer_ref(
                    manifest_dir,
                    &history_color_path,
                    "png_rgb8",
                    "previous Sintel frame reprojected to the current frame",
                ),
                motion_vectors: buffer_ref(
                    manifest_dir,
                    &motion_path,
                    "json_vec2_f32",
                    "current-grid backward motion derived from official forward flow",
                ),
                current_depth: buffer_ref(
                    manifest_dir,
                    &current_depth_path,
                    "json_scalar_f32",
                    "native Sintel depth",
                ),
                reprojected_depth: buffer_ref(
                    manifest_dir,
                    &history_depth_path,
                    "json_scalar_f32",
                    "previous Sintel depth reprojected to the current frame",
                ),
                current_normals: buffer_ref(
                    manifest_dir,
                    &current_normals_path,
                    "json_vec3_f32",
                    "normals derived from native Sintel depth",
                ),
                reprojected_normals: buffer_ref(
                    manifest_dir,
                    &history_normals_path,
                    "json_vec3_f32",
                    "previous normals reprojected to the current frame",
                ),
                metadata: buffer_ref(
                    manifest_dir,
                    &metadata_path,
                    "json_metadata",
                    "capture metadata",
                ),
                optional_mask: Some(buffer_ref(
                    manifest_dir,
                    &mask_path,
                    "json_mask_bool",
                    "derived motion-boundary ROI",
                )),
                optional_reference: Some(buffer_ref(
                    manifest_dir,
                    &reference_path,
                    "png_rgb8",
                    "clean-pass proxy reference",
                )),
                optional_ground_truth: None,
                optional_variance: Some(buffer_ref(
                    manifest_dir,
                    &variance_path,
                    "json_scalar_f32",
                    "clean-vs-final proxy variance field",
                )),
            },
        };

        let roi_pixels = mask.iter().filter(|value| **value).count();
        let roi_ratio = roi_pixels as f32 / (width * height).max(1) as f32;
        let flow_mean = motion
            .iter()
            .map(|value| {
                (value.to_prev_x * value.to_prev_x + value.to_prev_y * value.to_prev_y).sqrt()
            })
            .sum::<f32>()
            / motion.len().max(1) as f32;
        let mut case_tags = vec!["realism_stress_case".to_string()];
        if roi_ratio > 0.12 {
            case_tags.push("larger_roi_case".to_string());
        }
        if variant_id == "mixed" || (0.05..=0.14).contains(&roi_ratio) {
            case_tags.push("mixed_regime_candidate".to_string());
        }
        if variant_id == "point" {
            case_tags.push("point_roi_case".to_string());
        } else {
            case_tags.push("region_roi_case".to_string());
        }
        if flow_mean > 3.0 {
            case_tags.push("high_motion_case".to_string());
        }

        captures.push(PreparedCapture {
            entry,
            summary: DatasetCaptureSummary {
                label: capture_label,
                sequence_id: sequence_id.to_string(),
                frame_index: current_index,
                roi_kind,
                case_tags,
            },
        });
    }

    Ok(captures)
}

fn choose_sequences(available: &[String]) -> Vec<String> {
    let mut selected = Vec::new();
    for preferred in PREFERRED_SEQUENCES {
        if available.iter().any(|sequence| sequence == preferred) {
            selected.push(preferred.to_string());
        }
    }
    if selected.is_empty() {
        selected.extend(available.iter().take(2).cloned());
    }
    selected.truncate(2);
    selected
}

fn list_sequence_dirs(root: &Path) -> Result<Vec<String>> {
    let mut sequences = fs::read_dir(root)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    sequences.sort();
    Ok(sequences)
}

fn sorted_files(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn select_best_flow_index(flow_paths: &[PathBuf]) -> Result<usize> {
    let mut best: Option<(f32, usize)> = None;
    for (index, path) in flow_paths.iter().enumerate() {
        let flow = load_flo(path)?;
        let mean = flow
            .iter()
            .map(|value| (value.0 * value.0 + value.1 * value.1).sqrt())
            .sum::<f32>()
            / flow.len().max(1) as f32;
        if best.map(|(best_mean, _)| mean > best_mean).unwrap_or(true) {
            best = Some((mean, index));
        }
    }
    best.map(|(_, index)| index)
        .ok_or_else(|| Error::Message("failed to select a Sintel flow frame".to_string()))
}

fn load_flo(path: &Path) -> Result<Vec<(f32, f32)>> {
    let mut file = fs::File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    if &magic != b"PIEH" {
        return Err(Error::Message(format!(
            "invalid Sintel .flo magic in {}",
            path.display()
        )));
    }
    let width = read_i32(&mut file)? as usize;
    let height = read_i32(&mut file)? as usize;
    let mut values = Vec::with_capacity(width * height);
    for _ in 0..width * height {
        values.push((read_f32(&mut file)?, read_f32(&mut file)?));
    }
    Ok(values)
}

fn load_dpt(path: &Path) -> Result<Vec<f32>> {
    let mut file = fs::File::open(path)?;
    let _magic = read_i32(&mut file)?;
    let width = read_i32(&mut file)? as usize;
    let height = read_i32(&mut file)? as usize;
    let mut values = Vec::with_capacity(width * height);
    for _ in 0..width * height {
        values.push(read_f32(&mut file)?);
    }
    Ok(values)
}

fn read_i32(file: &mut fs::File) -> Result<i32> {
    let mut bytes = [0u8; 4];
    file.read_exact(&mut bytes)?;
    Ok(i32::from_le_bytes(bytes))
}

fn read_f32(file: &mut fs::File) -> Result<f32> {
    let mut bytes = [0u8; 4];
    file.read_exact(&mut bytes)?;
    Ok(f32::from_le_bytes(bytes))
}

fn invert_forward_flow(
    flow_forward: &[(f32, f32)],
    width: usize,
    height: usize,
) -> Vec<MotionVector> {
    let mut accum_x = vec![0.0f32; width * height];
    let mut accum_y = vec![0.0f32; width * height];
    let mut weight = vec![0.0f32; width * height];
    for y in 0..height {
        for x in 0..width {
            let (fx, fy) = flow_forward[y * width + x];
            let target_x = x as f32 + fx;
            let target_y = y as f32 + fy;
            if !(0.0..=(width.saturating_sub(1) as f32)).contains(&target_x)
                || !(0.0..=(height.saturating_sub(1) as f32)).contains(&target_y)
            {
                continue;
            }
            let x0 = target_x.floor() as usize;
            let y0 = target_y.floor() as usize;
            let x1 = (x0 + 1).min(width.saturating_sub(1));
            let y1 = (y0 + 1).min(height.saturating_sub(1));
            let tx = (target_x - x0 as f32).clamp(0.0, 1.0);
            let ty = (target_y - y0 as f32).clamp(0.0, 1.0);
            for (px, py, w) in [
                (x0, y0, (1.0 - tx) * (1.0 - ty)),
                (x1, y0, tx * (1.0 - ty)),
                (x0, y1, (1.0 - tx) * ty),
                (x1, y1, tx * ty),
            ] {
                let index = py * width + px;
                accum_x[index] += -fx * w;
                accum_y[index] += -fy * w;
                weight[index] += w;
            }
        }
    }

    let mut motion = vec![
        MotionVector {
            to_prev_x: 0.0,
            to_prev_y: 0.0,
        };
        width * height
    ];
    for index in 0..motion.len() {
        if weight[index] > 1e-6 {
            motion[index] = MotionVector {
                to_prev_x: accum_x[index] / weight[index],
                to_prev_y: accum_y[index] / weight[index],
            };
        }
    }
    fill_motion_holes(&mut motion, &weight, width, height);
    motion
}

fn fill_motion_holes(motion: &mut [MotionVector], weight: &[f32], width: usize, height: usize) {
    for _ in 0..3 {
        let previous = motion.to_vec();
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                if weight[index] > 1e-6 {
                    continue;
                }
                let mut sum_x = 0.0;
                let mut sum_y = 0.0;
                let mut count = 0.0;
                for ny in y.saturating_sub(1)..=(y + 1).min(height.saturating_sub(1)) {
                    for nx in x.saturating_sub(1)..=(x + 1).min(width.saturating_sub(1)) {
                        if nx == x && ny == y {
                            continue;
                        }
                        let neighbor = previous[ny * width + nx];
                        if neighbor.to_prev_x != 0.0 || neighbor.to_prev_y != 0.0 {
                            sum_x += neighbor.to_prev_x;
                            sum_y += neighbor.to_prev_y;
                            count += 1.0;
                        }
                    }
                }
                if count > 0.0 {
                    motion[index] = MotionVector {
                        to_prev_x: sum_x / count,
                        to_prev_y: sum_y / count,
                    };
                }
            }
        }
    }
}

fn warp_image(
    previous: &ImageFrame,
    motion: &[MotionVector],
    width: usize,
    height: usize,
) -> ImageFrame {
    let mut frame = ImageFrame::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let vector = motion[y * width + x];
            frame.set(
                x,
                y,
                previous.sample_bilinear_clamped(
                    x as f32 + vector.to_prev_x,
                    y as f32 + vector.to_prev_y,
                ),
            );
        }
    }
    frame
}

fn warp_scalar(values: &[f32], motion: &[MotionVector], width: usize, height: usize) -> Vec<f32> {
    let mut warped = vec![0.0; width * height];
    for y in 0..height {
        for x in 0..width {
            let vector = motion[y * width + x];
            warped[y * width + x] = sample_scalar(
                values,
                width,
                height,
                x as f32 + vector.to_prev_x,
                y as f32 + vector.to_prev_y,
            );
        }
    }
    warped
}

fn warp_normals(
    values: &[Normal3],
    motion: &[MotionVector],
    width: usize,
    height: usize,
) -> Vec<Normal3> {
    let mut warped = vec![Normal3::new(0.0, 0.0, 1.0); width * height];
    for y in 0..height {
        for x in 0..width {
            let vector = motion[y * width + x];
            warped[y * width + x] = sample_normal(
                values,
                width,
                height,
                x as f32 + vector.to_prev_x,
                y as f32 + vector.to_prev_y,
            );
        }
    }
    warped
}

fn derive_normals(depth: &[f32], width: usize, height: usize) -> Vec<Normal3> {
    let mut normals = vec![Normal3::new(0.0, 0.0, 1.0); width * height];
    for y in 0..height {
        for x in 0..width {
            let x0 = x.saturating_sub(1);
            let x1 = (x + 1).min(width.saturating_sub(1));
            let y0 = y.saturating_sub(1);
            let y1 = (y + 1).min(height.saturating_sub(1));
            let dzdx = depth[y * width + x1] - depth[y * width + x0];
            let dzdy = depth[y1 * width + x] - depth[y0 * width + x];
            normals[y * width + x] = Normal3::new(-dzdx, -dzdy, 1.0).normalized();
        }
    }
    normals
}

fn build_roi_score(
    motion: &[MotionVector],
    depth: &[f32],
    final_frame: &ImageFrame,
    clean_frame: &ImageFrame,
    width: usize,
    height: usize,
) -> Vec<f32> {
    let motion_mag = motion
        .iter()
        .map(|value| (value.to_prev_x * value.to_prev_x + value.to_prev_y * value.to_prev_y).sqrt())
        .collect::<Vec<_>>();
    let mut score = vec![0.0; width * height];
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let local_motion = local_gradient(&motion_mag, width, height, x, y);
            let depth_edge = local_gradient(depth, width, height, x, y);
            let clean_final_gap = final_frame.get(x, y).abs_diff(clean_frame.get(x, y));
            score[index] = motion_mag[index] * 0.45
                + local_motion * 0.30
                + depth_edge * 0.15
                + clean_final_gap * 0.10;
        }
    }
    normalize_values(score)
}

fn local_gradient(values: &[f32], width: usize, height: usize, x: usize, y: usize) -> f32 {
    let center = values[y * width + x];
    let mut strongest: f32 = 0.0;
    for ny in y.saturating_sub(1)..=(y + 1).min(height.saturating_sub(1)) {
        for nx in x.saturating_sub(1)..=(x + 1).min(width.saturating_sub(1)) {
            if nx == x && ny == y {
                continue;
            }
            strongest = strongest.max((center - values[ny * width + nx]).abs());
        }
    }
    strongest
}

fn normalize_values(mut values: Vec<f32>) -> Vec<f32> {
    let max_value = values.iter().copied().fold(0.0f32, f32::max).max(1e-6);
    for value in &mut values {
        *value = (*value / max_value).clamp(0.0, 1.0);
    }
    values
}

fn percentile_mask(score: &[f32], percentile: f32) -> Vec<bool> {
    let mut sorted = score.to_vec();
    sorted.sort_by(|left, right| right.total_cmp(left));
    let keep = ((score.len() as f32 * percentile).round() as usize)
        .clamp(1, score.len().saturating_sub(1).max(1));
    let threshold = sorted[keep.saturating_sub(1)];
    score.iter().map(|value| *value >= threshold).collect()
}

fn temporal_difference_proxy(final_frame: &ImageFrame, clean_frame: &ImageFrame) -> Vec<f32> {
    final_frame
        .pixels()
        .iter()
        .zip(clean_frame.pixels())
        .map(|(final_pixel, clean_pixel)| final_pixel.abs_diff(*clean_pixel))
        .collect()
}

fn sample_scalar(values: &[f32], width: usize, height: usize, x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);
    let sample = |sx: f32, sy: f32| {
        let px = sx.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let py = sy.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[py * width + px]
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
    let sample = |sx: f32, sy: f32| {
        let px = sx.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let py = sy.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[py * width + px]
    };
    let lerp = |a: Normal3, b: Normal3, t: f32| {
        Normal3::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
        )
        .normalized()
    };
    lerp(
        lerp(sample(x0, y0), sample(x1, y0), tx),
        lerp(sample(x0, y1), sample(x1, y1), tx),
        ty,
    )
}

fn find_path_ending_with(root: &Path, suffix: &Path) -> Result<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.ends_with(suffix) {
            return Ok(dir);
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                stack.push(entry.path());
            }
        }
    }
    Err(Error::Message(format!(
        "failed to find path ending with {} inside {}",
        suffix.display(),
        root.display()
    )))
}

fn buffer_ref(
    manifest_dir: &Path,
    actual_path: &Path,
    format: &str,
    semantic: &str,
) -> BufferReference {
    BufferReference {
        path: relative_path(manifest_dir, actual_path)
            .display()
            .to_string(),
        format: format.to_string(),
        semantic: semantic.to_string(),
        width: None,
        height: None,
        channels: None,
    }
}

fn field(
    field_id: &str,
    quality: FieldQuality,
    source: &str,
    disclosure: &str,
) -> BufferFieldSummary {
    BufferFieldSummary {
        field_id: field_id.to_string(),
        quality,
        source: source.to_string(),
        disclosure: disclosure.to_string(),
    }
}

fn blocked_summary(output_dir: &Path, blocker: String) -> DatasetMappingSummary {
    DatasetMappingSummary {
        dataset_id: "sintel".to_string(),
        dataset_name: "MPI Sintel".to_string(),
        why_chosen: "MPI Sintel is the primary renderer-like / motion-aware external dataset for this crate.".to_string(),
        prepared_output_dir: output_dir.display().to_string(),
        manifest_path: manifest_path("sintel").display().to_string(),
        dsfb_mode: "blocked".to_string(),
        demo_a_metric_mode: "blocked".to_string(),
        demo_b_mode: "blocked".to_string(),
        reference_strategy: "blocked".to_string(),
        official_urls: vec![SINTEL_COMPLETE_URL.to_string(), SINTEL_DEPTH_URL.to_string()],
        native_buffers: Vec::new(),
        derived_buffers: Vec::new(),
        unsupported_buffers: Vec::new(),
        fields: Vec::new(),
        captures: Vec::new(),
        blockers: vec![blocker],
        notes: vec!["Preparation failed before a runnable manifest could be emitted.".to_string()],
    }
}
