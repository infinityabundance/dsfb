use std::fs;
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

const DAVIS_URL: &str =
    "https://data.vision.ee.ethz.ch/csergi/share/davis/DAVIS-2017-trainval-480p.zip";
const PREFERRED_SEQUENCES: [&str; 4] = ["dance-twirl", "soapbox", "camel", "dog"];

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
    let archive_path = downloads_dir.join("DAVIS-2017-trainval-480p.zip");
    download_if_missing(DAVIS_URL, &archive_path)?;

    let source_dir = output_dir.join("source");
    unzip_if_needed(&archive_path, &source_dir)?;

    let images_root =
        find_path_ending_with(&source_dir, Path::new("JPEGImages").join("480p").as_path())?;
    let masks_root =
        find_path_ending_with(&source_dir, Path::new("Annotations").join("480p").as_path())?;

    let available_sequences = list_sequence_dirs(&images_root)?;
    if available_sequences.is_empty() {
        return Err(Error::Message(
            "DAVIS extraction succeeded, but no 480p sequences were found".to_string(),
        ));
    }
    let selected_sequences = choose_sequences(&available_sequences);
    let manifest_dir = manifest_path("davis")
        .parent()
        .ok_or_else(|| Error::Message("DAVIS manifest path had no parent".to_string()))?
        .to_path_buf();

    let mut captures = Vec::new();
    let mut capture_summaries = Vec::new();
    for sequence_id in selected_sequences {
        let sequence_image_dir = images_root.join(&sequence_id);
        let sequence_mask_dir = masks_root.join(&sequence_id);
        let prepared = prepare_capture(
            output_dir,
            &manifest_dir,
            &sequence_id,
            &sequence_image_dir,
            &sequence_mask_dir,
        )?;
        capture_summaries.push(prepared.summary);
        captures.push(prepared.entry);
    }

    let manifest = ExternalCaptureManifest {
        format_version: EXTERNAL_CAPTURE_FORMAT_VERSION.to_string(),
        description: "Official DAVIS 2017 trainval 480p captures mapped into the DSFB external replay schema.".to_string(),
        source: ExternalCaptureSource::Files,
        buffers: None,
        captures,
        normalization: ExternalNormalization {
            color: "sRGB frames converted to normalized RGB in [0,1]".to_string(),
            motion_vectors: "deterministic block-matching proxy in pixel offsets to the previous frame; derived-low-confidence".to_string(),
            depth: "segmentation-guided relative-depth proxy in [0,1]; derived-low-confidence and explicitly non-metric".to_string(),
            normals: "normals derived from the relative-depth proxy; derived-low-confidence".to_string(),
        },
        notes: vec![
            "DAVIS provides native real-video color and native segmentation masks.".to_string(),
            "Motion vectors, depth, and normals are explicit derived proxies rather than native buffers.".to_string(),
            "Demo A metrics remain proxy-only because the mapped path has no renderer-quality ground truth.".to_string(),
        ],
    };
    write_json_file(&manifest_path("davis"), &manifest)?;

    Ok(DatasetMappingSummary {
        dataset_id: "davis".to_string(),
        dataset_name: "DAVIS 2017".to_string(),
        why_chosen: "DAVIS is a standard, immediately recognizable real-video benchmark with dense segmentation masks. It anchors the external replay path in real image content instead of only synthetic scenes.".to_string(),
        prepared_output_dir: output_dir.display().to_string(),
        manifest_path: manifest_path("davis").display().to_string(),
        dsfb_mode: "host_minimum_with_native_color_and_roi_plus_derived_motion_depth_normal_proxies".to_string(),
        demo_a_metric_mode: "proxy_only_without_renderer_ground_truth".to_string(),
        demo_b_mode: "fixed_budget_allocation_proxy_with_native_roi_masks".to_string(),
        reference_strategy: "no reference frame; current-vs-history proxy metrics only".to_string(),
        official_urls: vec![DAVIS_URL.to_string()],
        native_buffers: vec![
            "current_color".to_string(),
            "history_color".to_string(),
            "roi_mask".to_string(),
        ],
        derived_buffers: vec![
            "motion_vectors".to_string(),
            "current_depth".to_string(),
            "history_depth".to_string(),
            "current_normals".to_string(),
            "history_normals".to_string(),
        ],
        unsupported_buffers: vec!["ground_truth_reference".to_string()],
        fields: vec![
            field("current_color", FieldQuality::Native, "official DAVIS RGB frame", "native DAVIS image"),
            field("history_color", FieldQuality::Native, "adjacent DAVIS RGB frame reprojected to the current frame using the derived motion field", "native frames; reprojection uses the derived motion proxy"),
            field("motion_vectors", FieldQuality::DerivedLowConfidence, "deterministic block-matching optical-flow proxy", "derived from adjacent DAVIS frames; not native optical flow"),
            field("current_depth", FieldQuality::DerivedLowConfidence, "segmentation-guided relative-depth proxy", "foreground/background relative-depth proxy only; not metric depth"),
            field("history_depth", FieldQuality::DerivedLowConfidence, "previous-frame relative depth warped into current frame", "derived from previous DAVIS mask/image and the motion proxy"),
            field("current_normals", FieldQuality::DerivedLowConfidence, "depth-gradient normals", "derived from the relative-depth proxy"),
            field("history_normals", FieldQuality::DerivedLowConfidence, "previous depth-gradient normals warped into current frame", "derived from the previous relative-depth proxy"),
            field("roi_mask", FieldQuality::Native, "official DAVIS segmentation annotation", "native binary ROI support after unioning non-zero objects"),
            field("reference", FieldQuality::Unavailable, "not provided by DAVIS in this mapping", "no renderer-quality temporal ground truth is available in the mapped path"),
        ],
        captures: capture_summaries,
        blockers: Vec::new(),
        notes: vec![
            format!("DAVIS mapping report: {}", report_md_path("davis").display()),
            format!("DAVIS summary JSON: {}", summary_json_path("davis").display()),
            "Derived fields are labeled derived-low-confidence in both the reports and the manifests.".to_string(),
            "No derived-high-confidence fields exist for this dataset: DAVIS does not provide native optical flow, metric depth, or renderer-origin outputs, so all derived buffers are explicitly labeled derived-low-confidence rather than derived-high-confidence.".to_string(),
        ],
    })
}

struct PreparedCapture {
    entry: ExternalCaptureEntry,
    summary: DatasetCaptureSummary,
}

fn prepare_capture(
    output_dir: &Path,
    manifest_dir: &Path,
    sequence_id: &str,
    image_dir: &Path,
    mask_dir: &Path,
) -> Result<PreparedCapture> {
    let (current_index, prev_path, current_path, prev_mask_path, current_mask_path) =
        select_best_pair(image_dir, mask_dir)?;
    let previous = load_image_frame(&prev_path)?;
    let current = load_image_frame(&current_path)?;
    if previous.width() != current.width() || previous.height() != current.height() {
        return Err(Error::Message(format!(
            "DAVIS sequence {} produced mismatched frame extents",
            sequence_id
        )));
    }
    let width = current.width();
    let height = current.height();
    let prev_mask = load_mask(&prev_mask_path)?;
    let current_mask = load_mask(&current_mask_path)?;
    let motion = derive_block_matching_motion(&previous, &current, &prev_mask, &current_mask);
    let history_color = warp_image(&previous, &motion, width, height);
    let current_depth = derive_relative_depth(&current, &current_mask);
    let previous_depth = derive_relative_depth(&previous, &prev_mask);
    let history_depth = warp_scalar(&previous_depth, &motion, width, height);
    let current_normals = derive_normals(&current_depth, width, height);
    let previous_normals = derive_normals(&previous_depth, width, height);
    let history_normals = warp_normals(&previous_normals, &motion, width, height);

    let capture_label = format!("{sequence_id}_frame_{current_index:04}");
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
    let metadata_path = capture_dir.join("metadata.json");

    write_image_frame(&current_color_path, &current)?;
    write_image_frame(&history_color_path, &history_color)?;
    write_vec2_grid(&motion_path, &motion, width, height)?;
    write_scalar_grid(&current_depth_path, &current_depth, width, height)?;
    write_scalar_grid(&history_depth_path, &history_depth, width, height)?;
    write_vec3_grid(&current_normals_path, &current_normals, width, height)?;
    write_vec3_grid(&history_normals_path, &history_normals, width, height)?;
    write_mask_grid(&mask_path, &current_mask, width, height)?;

    let metadata = ExternalCaptureMetadata {
        scenario_id: None,
        frame_index: current_index,
        history_frame_index: current_index.saturating_sub(1),
        width,
        height,
        source_kind: "davis_2017_real_video".to_string(),
        externally_validated: true,
        real_external_data: true,
        data_description: Some(
            "Official DAVIS 2017 trainval 480p real-video frame pair mapped into the DSFB external schema".to_string(),
        ),
        notes: vec![
            format!("sequence_id={sequence_id}"),
            "ROI comes from the official DAVIS annotation after unioning non-zero object ids.".to_string(),
            "Motion vectors use deterministic block matching on adjacent frames and remain derived-low-confidence.".to_string(),
            "Depth is a segmentation-guided relative-depth proxy, not metric depth.".to_string(),
            "Normals are derived from the relative-depth proxy.".to_string(),
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
                "current DAVIS RGB frame",
            ),
            reprojected_history: buffer_ref(
                manifest_dir,
                &history_color_path,
                "png_rgb8",
                "previous DAVIS frame reprojected to the current frame",
            ),
            motion_vectors: buffer_ref(
                manifest_dir,
                &motion_path,
                "json_vec2_f32",
                "derived block-matching motion proxy",
            ),
            current_depth: buffer_ref(
                manifest_dir,
                &current_depth_path,
                "json_scalar_f32",
                "derived DAVIS relative-depth proxy",
            ),
            reprojected_depth: buffer_ref(
                manifest_dir,
                &history_depth_path,
                "json_scalar_f32",
                "previous DAVIS relative depth warped to the current frame",
            ),
            current_normals: buffer_ref(
                manifest_dir,
                &current_normals_path,
                "json_vec3_f32",
                "normals derived from the current relative-depth proxy",
            ),
            reprojected_normals: buffer_ref(
                manifest_dir,
                &history_normals_path,
                "json_vec3_f32",
                "previous normals warped to the current frame",
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
                "native DAVIS ROI mask",
            )),
            optional_reference: None,
            optional_ground_truth: None,
            optional_variance: None,
        },
    };

    let roi_pixels = current_mask.iter().filter(|value| **value).count();
    let roi_ratio = roi_pixels as f32 / (width * height).max(1) as f32;
    let motion_mean = motion
        .iter()
        .map(|value| (value.to_prev_x * value.to_prev_x + value.to_prev_y * value.to_prev_y).sqrt())
        .sum::<f32>()
        / motion.len().max(1) as f32;
    let mut case_tags = vec!["realism_stress_case".to_string()];
    if roi_ratio > 0.10 {
        case_tags.push("larger_roi_case".to_string());
    }
    if (0.04..=0.10).contains(&roi_ratio) || motion_mean > 2.5 {
        case_tags.push("mixed_regime_candidate".to_string());
    }

    Ok(PreparedCapture {
        entry,
        summary: DatasetCaptureSummary {
            label: capture_label,
            sequence_id: sequence_id.to_string(),
            frame_index: current_index,
            roi_kind: if roi_ratio > 0.08 {
                "native_region_roi".to_string()
            } else {
                "native_compact_roi".to_string()
            },
            case_tags,
        },
    })
}

fn choose_sequences(available: &[String]) -> Vec<String> {
    let mut selected = Vec::new();
    for preferred in PREFERRED_SEQUENCES {
        if available.iter().any(|sequence| sequence == preferred) {
            selected.push(preferred.to_string());
        }
    }
    if selected.is_empty() {
        selected.extend(available.iter().take(3).cloned());
    }
    selected.truncate(3);
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

fn select_best_pair(
    image_dir: &Path,
    mask_dir: &Path,
) -> Result<(usize, PathBuf, PathBuf, PathBuf, PathBuf)> {
    let frame_paths = sorted_files(image_dir, "jpg")?;
    let mask_paths = sorted_files(mask_dir, "png")?;
    if frame_paths.len() < 2 || mask_paths.len() < 2 {
        return Err(Error::Message(format!(
            "sequence {} does not have enough DAVIS frames for a temporal pair",
            image_dir.display()
        )));
    }

    let mut best: Option<(f32, usize)> = None;
    for index in 1..frame_paths.len().min(mask_paths.len()) {
        let current = load_image_frame(&frame_paths[index])?;
        let previous = load_image_frame(&frame_paths[index - 1])?;
        let current_mask = load_mask(&mask_paths[index])?;
        let roi_ratio =
            current_mask.iter().filter(|value| **value).count() as f32 / current_mask.len() as f32;
        let photometric_change = mean_abs_diff(&previous, &current);
        let score = photometric_change + roi_ratio * 0.5;
        if best
            .map(|(best_score, _)| score > best_score)
            .unwrap_or(true)
        {
            best = Some((score, index));
        }
    }

    let best_index = best
        .map(|(_, index)| index)
        .ok_or_else(|| Error::Message("failed to select a DAVIS frame pair".to_string()))?;
    Ok((
        best_index,
        frame_paths[best_index - 1].clone(),
        frame_paths[best_index].clone(),
        mask_paths[best_index - 1].clone(),
        mask_paths[best_index].clone(),
    ))
}

fn mean_abs_diff(previous: &ImageFrame, current: &ImageFrame) -> f32 {
    previous
        .pixels()
        .iter()
        .zip(current.pixels())
        .map(|(prev, curr)| prev.abs_diff(*curr))
        .sum::<f32>()
        / previous.len().max(1) as f32
}

fn derive_block_matching_motion(
    previous: &ImageFrame,
    current: &ImageFrame,
    previous_mask: &[bool],
    current_mask: &[bool],
) -> Vec<MotionVector> {
    let width = current.width();
    let height = current.height();
    let tile_size = 8usize;
    let patch_radius = 2i32;
    let search_radius = 4i32;
    let grid_w = (width + tile_size - 1) / tile_size;
    let grid_h = (height + tile_size - 1) / tile_size;
    let global = centroid_shift(previous_mask, current_mask, width, height);
    let previous_luma = previous
        .pixels()
        .iter()
        .map(|pixel| pixel.luma())
        .collect::<Vec<_>>();
    let current_luma = current
        .pixels()
        .iter()
        .map(|pixel| pixel.luma())
        .collect::<Vec<_>>();
    let mut grid = vec![
        MotionVector {
            to_prev_x: global.0 as f32,
            to_prev_y: global.1 as f32,
        };
        grid_w * grid_h
    ];

    for gy in 0..grid_h {
        for gx in 0..grid_w {
            let x = (gx * tile_size + tile_size / 2).min(width.saturating_sub(1));
            let y = (gy * tile_size + tile_size / 2).min(height.saturating_sub(1));
            let mut best_cost = f32::INFINITY;
            let mut best = global;
            for dy in (global.1 - search_radius)..=(global.1 + search_radius) {
                for dx in (global.0 - search_radius)..=(global.0 + search_radius) {
                    let mut cost = 0.0;
                    for py in -patch_radius..=patch_radius {
                        for px in -patch_radius..=patch_radius {
                            let cx = (x as i32 + px).clamp(0, width.saturating_sub(1) as i32);
                            let cy = (y as i32 + py).clamp(0, height.saturating_sub(1) as i32);
                            let px_prev =
                                (x as i32 + dx + px).clamp(0, width.saturating_sub(1) as i32);
                            let py_prev =
                                (y as i32 + dy + py).clamp(0, height.saturating_sub(1) as i32);
                            let current_index = cy as usize * width + cx as usize;
                            let previous_index = py_prev as usize * width + px_prev as usize;
                            cost +=
                                (current_luma[current_index] - previous_luma[previous_index]).abs();
                        }
                    }
                    if cost < best_cost
                        || ((cost - best_cost).abs() <= f32::EPSILON
                            && dx.abs() + dy.abs() < best.0.abs() + best.1.abs())
                    {
                        best_cost = cost;
                        best = (dx, dy);
                    }
                }
            }
            grid[gy * grid_w + gx] = MotionVector {
                to_prev_x: best.0 as f32,
                to_prev_y: best.1 as f32,
            };
        }
    }

    let mut per_pixel = vec![
        MotionVector {
            to_prev_x: 0.0,
            to_prev_y: 0.0,
        };
        width * height
    ];
    for y in 0..height {
        for x in 0..width {
            let gx0 = (x / tile_size).min(grid_w.saturating_sub(1));
            let gy0 = (y / tile_size).min(grid_h.saturating_sub(1));
            let gx1 = (gx0 + 1).min(grid_w.saturating_sub(1));
            let gy1 = (gy0 + 1).min(grid_h.saturating_sub(1));
            let tx = ((x % tile_size) as f32 / tile_size as f32).clamp(0.0, 1.0);
            let ty = ((y % tile_size) as f32 / tile_size as f32).clamp(0.0, 1.0);
            let v00 = grid[gy0 * grid_w + gx0];
            let v10 = grid[gy0 * grid_w + gx1];
            let v01 = grid[gy1 * grid_w + gx0];
            let v11 = grid[gy1 * grid_w + gx1];
            per_pixel[y * width + x] = bilerp_motion(v00, v10, v01, v11, tx, ty);
        }
    }
    per_pixel
}

fn bilerp_motion(
    v00: MotionVector,
    v10: MotionVector,
    v01: MotionVector,
    v11: MotionVector,
    tx: f32,
    ty: f32,
) -> MotionVector {
    let lerp = |a: MotionVector, b: MotionVector, t: f32| MotionVector {
        to_prev_x: a.to_prev_x + (b.to_prev_x - a.to_prev_x) * t,
        to_prev_y: a.to_prev_y + (b.to_prev_y - a.to_prev_y) * t,
    };
    lerp(lerp(v00, v10, tx), lerp(v01, v11, tx), ty)
}

fn centroid_shift(
    previous_mask: &[bool],
    current_mask: &[bool],
    width: usize,
    height: usize,
) -> (i32, i32) {
    let prev = mask_centroid(previous_mask, width, height);
    let curr = mask_centroid(current_mask, width, height);
    match (prev, curr) {
        (Some(prev), Some(curr)) => (
            (prev.0 - curr.0).round() as i32,
            (prev.1 - curr.1).round() as i32,
        ),
        _ => (0, 0),
    }
}

fn mask_centroid(mask: &[bool], width: usize, height: usize) -> Option<(f32, f32)> {
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut count = 0.0;
    for y in 0..height {
        for x in 0..width {
            if mask[y * width + x] {
                sum_x += x as f32;
                sum_y += y as f32;
                count += 1.0;
            }
        }
    }
    if count > 0.0 {
        Some((sum_x / count, sum_y / count))
    } else {
        None
    }
}

fn derive_relative_depth(current: &ImageFrame, mask: &[bool]) -> Vec<f32> {
    let mut depth = vec![0.0; current.len()];
    for y in 0..current.height() {
        for x in 0..current.width() {
            let index = y * current.width() + x;
            let base = if mask[index] { 0.35 } else { 0.75 };
            let luma = current.get(x, y).luma();
            depth[index] = (base + (0.5 - luma) * 0.16).clamp(0.05, 0.95);
        }
    }
    depth
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

fn load_mask(path: &Path) -> Result<Vec<bool>> {
    let image = image::open(path)?.to_luma8();
    Ok(image.pixels().map(|pixel| pixel[0] != 0).collect())
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
        dataset_id: "davis".to_string(),
        dataset_name: "DAVIS 2017".to_string(),
        why_chosen: "DAVIS is the primary real-video external dataset for this crate.".to_string(),
        prepared_output_dir: output_dir.display().to_string(),
        manifest_path: manifest_path("davis").display().to_string(),
        dsfb_mode: "blocked".to_string(),
        demo_a_metric_mode: "blocked".to_string(),
        demo_b_mode: "blocked".to_string(),
        reference_strategy: "blocked".to_string(),
        official_urls: vec![DAVIS_URL.to_string()],
        native_buffers: Vec::new(),
        derived_buffers: Vec::new(),
        unsupported_buffers: Vec::new(),
        fields: Vec::new(),
        captures: Vec::new(),
        blockers: vec![blocker],
        notes: vec!["Preparation failed before a runnable manifest could be emitted.".to_string()],
    }
}
