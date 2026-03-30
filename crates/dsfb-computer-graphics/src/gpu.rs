use std::sync::mpsc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::error::{Error, Result};
use crate::external::OwnedHostTemporalInputs;
use crate::frame::ScalarField;
use crate::parameters::HostSupervisionParameters;

#[derive(Clone, Debug)]
pub struct GpuKernelResult {
    pub adapter_name: String,
    pub backend: String,
    pub trust: Vec<f32>,
    pub alpha: Vec<f32>,
    pub intervention: Vec<f32>,
    pub total_ms: f64,
    pub dispatch_ms: f64,
    pub readback_ms: f64,
    pub workgroup_size: (u32, u32, u32),
}

#[derive(Clone, Debug)]
struct ChunkExecutionResult {
    trust: Vec<f32>,
    alpha: Vec<f32>,
    intervention: Vec<f32>,
    total_ms: f64,
    dispatch_ms: f64,
    readback_ms: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuParams {
    size: [u32; 4],
    alpha_range: [f32; 4],
    residual_threshold: [f32; 4],
    depth_threshold: [f32; 4],
    normal_threshold: [f32; 4],
    neighborhood_threshold: [f32; 4],
    local_contrast_threshold: [f32; 4],
    hazard_curve_threshold: [f32; 4],
    weights_a: [f32; 4],
    weights_b: [f32; 4],
    history_instability_mix: [f32; 4],
    structural_a: [f32; 4],
    structural_b: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuColor {
    value: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuDepthPair {
    value: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuNormalPair {
    current: [f32; 4],
    history: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVec4 {
    value: [f32; 4],
}

const SHADER_SOURCE: &str = r#"
struct Params {
    size: vec4<u32>,
    alpha_range: vec4<f32>,
    residual_threshold: vec4<f32>,
    depth_threshold: vec4<f32>,
    normal_threshold: vec4<f32>,
    neighborhood_threshold: vec4<f32>,
    local_contrast_threshold: vec4<f32>,
    hazard_curve_threshold: vec4<f32>,
    weights_a: vec4<f32>,
    weights_b: vec4<f32>,
    history_instability_mix: vec4<f32>,
    structural_a: vec4<f32>,
    structural_b: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> current_color: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> reprojected_history: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> depth_pairs: array<vec2<f32>>;

struct NormalPair {
    current: vec4<f32>,
    history: vec4<f32>,
}

@group(0) @binding(3) var<storage, read> normal_pairs: array<NormalPair>;
@group(0) @binding(4) var<uniform> params: Params;
@group(0) @binding(5) var<storage, read_write> trust_out: array<f32>;
@group(0) @binding(6) var<storage, read_write> alpha_out: array<f32>;
@group(0) @binding(7) var<storage, read_write> intervention_out: array<f32>;

fn index_of(x: u32, y: u32) -> u32 {
    return y * params.size.x + x;
}

fn luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn smoothstep_threshold(low: f32, high: f32, value: f32) -> f32 {
    let edge_span = max(high - low, 1e-6);
    let t = clamp((value - low) / edge_span, 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn color_at(x: i32, y: i32) -> vec3<f32> {
    let width = i32(params.size.x);
    let height = i32(params.size.y);
    let clamped_x = clamp(x, 0, width - 1);
    let clamped_y = clamp(y, 0, height - 1);
    let idx = index_of(u32(clamped_x), u32(clamped_y));
    return current_color[idx].xyz;
}

fn local_contrast_gate(x: i32, y: i32) -> f32 {
    let center = luma(color_at(x, y));
    var strongest = 0.0;
    for (var oy: i32 = -1; oy <= 1; oy = oy + 1) {
        for (var ox: i32 = -1; ox <= 1; ox = ox + 1) {
            if (ox == 0 && oy == 0) {
                continue;
            }
            strongest = max(strongest, abs(center - luma(color_at(x + ox, y + oy))));
        }
    }
    return smoothstep_threshold(
        params.local_contrast_threshold.x,
        params.local_contrast_threshold.y,
        strongest
    );
}

fn neighborhood_gate(x: i32, y: i32, history_luma: f32) -> f32 {
    var min_luma = 1e9;
    var max_luma = -1e9;
    for (var oy: i32 = -1; oy <= 1; oy = oy + 1) {
        for (var ox: i32 = -1; ox <= 1; ox = ox + 1) {
            let sample = luma(color_at(x + ox, y + oy));
            min_luma = min(min_luma, sample);
            max_luma = max(max_luma, sample);
        }
    }
    var distance = 0.0;
    if (history_luma < min_luma) {
        distance = min_luma - history_luma;
    } else if (history_luma > max_luma) {
        distance = history_luma - max_luma;
    }
    return smoothstep_threshold(
        params.neighborhood_threshold.x,
        params.neighborhood_threshold.y,
        distance
    );
}

@compute @workgroup_size(1, 1, 1)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    if (gid.x >= params.size.x || gid.y >= params.size.y) {
        return;
    }
    let idx = index_of(gid.x, gid.y);
    let pixel_x = i32(gid.x);
    let pixel_y = i32(gid.y);
    let current = current_color[idx].xyz;
    let history = reprojected_history[idx].xyz;
    let residual = (abs(current.x - history.x) + abs(current.y - history.y) + abs(current.z - history.z)) / 3.0;
    let residual_gate = smoothstep_threshold(params.residual_threshold.x, params.residual_threshold.y, residual);
    let depth_pair = depth_pairs[idx];
    let depth_gate = smoothstep_threshold(
        params.depth_threshold.x,
        params.depth_threshold.y,
        abs(depth_pair.x - depth_pair.y)
    );
    let normal_pair = normal_pairs[idx];
    let n0 = normalize(normal_pair.current.xyz);
    let n1 = normalize(normal_pair.history.xyz);
    let normal_gate = smoothstep_threshold(
        params.normal_threshold.x,
        params.normal_threshold.y,
        1.0 - clamp(dot(n0, n1), -1.0, 1.0)
    );
    let history_luma = luma(history);
    let neighbor_gate = neighborhood_gate(pixel_x, pixel_y, history_luma);
    let thin_gate = local_contrast_gate(pixel_x, pixel_y);
    let history_instability = clamp(
        params.history_instability_mix.x * residual_gate +
        params.history_instability_mix.y * neighbor_gate,
        0.0,
        1.0
    );
    let structural_disagreement = max(depth_gate, normal_gate);
    var grammar_component = 0.0;
    if (structural_disagreement >= params.structural_a.x) {
        grammar_component = 0.88;
    } else if (residual_gate >= params.structural_a.y && neighbor_gate >= params.structural_a.z) {
        grammar_component = 0.62;
    } else if (thin_gate >= params.structural_b.x && residual_gate >= params.structural_b.y) {
        grammar_component = 0.32;
    }
    let hazard_raw =
        params.weights_a.x * residual_gate +
        params.weights_a.y * depth_gate +
        params.weights_a.z * normal_gate +
        params.weights_a.w * neighbor_gate +
        params.weights_b.x * thin_gate +
        params.weights_b.y * history_instability +
        params.weights_b.z * grammar_component;
    let hazard = smoothstep_threshold(
        params.hazard_curve_threshold.x,
        params.hazard_curve_threshold.y,
        clamp(hazard_raw, 0.0, 1.0)
    );
    trust_out[idx] = 1.0 - hazard;
    alpha_out[idx] = params.alpha_range.x + (params.alpha_range.y - params.alpha_range.x) * hazard;
    intervention_out[idx] = hazard;
}
"#;

pub fn try_execute_host_minimum_kernel(
    inputs: &OwnedHostTemporalInputs,
    parameters: HostSupervisionParameters,
) -> Result<Option<GpuKernelResult>> {
    pollster::block_on(try_execute_host_minimum_kernel_async(inputs, parameters))
}

async fn try_execute_host_minimum_kernel_async(
    inputs: &OwnedHostTemporalInputs,
    parameters: HostSupervisionParameters,
) -> Result<Option<GpuKernelResult>> {
    let instance = wgpu::Instance::default();
    let adapter = match instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
    {
        Some(adapter) => adapter,
        None => return Ok(None),
    };

    let adapter_info = adapter.get_info();
    let adapter_limits = adapter.limits();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("dsfb-computer-graphics-gpu-path"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size,
                    max_buffer_size: adapter_limits.max_buffer_size,
                    ..wgpu::Limits::default()
                },
            },
            None,
        )
        .await
        .map_err(|error| Error::Message(format!("failed to request wgpu device: {error}")))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("dsfb-host-minimum-wgsl"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("dsfb-host-minimum-layout"),
        entries: &[
            storage_layout_entry(0, true),
            storage_layout_entry(1, true),
            storage_layout_entry(2, true),
            storage_layout_entry(3, true),
            uniform_layout_entry(4),
            storage_layout_entry(5, false),
            storage_layout_entry(6, false),
            storage_layout_entry(7, false),
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("dsfb-host-minimum-pipeline-layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("dsfb-host-minimum-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });
    let max_binding_size = device.limits().max_storage_buffer_binding_size as usize;
    let chunk = if requires_tiled_dispatch(inputs, max_binding_size) {
        execute_host_minimum_tiled(
            &device,
            &queue,
            &pipeline,
            &bind_group_layout,
            inputs,
            parameters,
            max_binding_size,
        )?
    } else {
        execute_host_minimum_chunk(
            &device,
            &queue,
            &pipeline,
            &bind_group_layout,
            inputs,
            parameters,
        )?
    };

    Ok(Some(GpuKernelResult {
        adapter_name: adapter_info.name,
        backend: format!("{:?}", adapter_info.backend),
        trust: chunk.trust,
        alpha: chunk.alpha,
        intervention: chunk.intervention,
        total_ms: chunk.total_ms,
        dispatch_ms: chunk.dispatch_ms,
        readback_ms: chunk.readback_ms,
        workgroup_size: (1, 1, 1),
    }))
}

fn requires_tiled_dispatch(inputs: &OwnedHostTemporalInputs, max_binding_size: usize) -> bool {
    let pixel_count = inputs.width().saturating_mul(inputs.height());
    let largest_binding_bytes = pixel_count.saturating_mul(std::mem::size_of::<GpuNormalPair>());
    largest_binding_bytes > max_binding_size
}

fn execute_host_minimum_tiled(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
    bind_group_layout: &wgpu::BindGroupLayout,
    inputs: &OwnedHostTemporalInputs,
    parameters: HostSupervisionParameters,
    max_binding_size: usize,
) -> Result<ChunkExecutionResult> {
    let width = inputs.width();
    let height = inputs.height();
    let bytes_per_row = width
        .saturating_mul(std::mem::size_of::<GpuNormalPair>())
        .max(1);
    let max_rows_with_padding = max_binding_size / bytes_per_row;
    let stripe_rows = max_rows_with_padding.saturating_sub(2).max(1);
    if stripe_rows == 0 {
        return Err(Error::Message(
            "GPU tiled dispatch could not derive a non-zero stripe height".to_string(),
        ));
    }

    let pixel_count = width * height;
    let mut trust = Vec::with_capacity(pixel_count);
    let mut alpha = Vec::with_capacity(pixel_count);
    let mut intervention = Vec::with_capacity(pixel_count);
    let mut total_ms = 0.0;
    let mut dispatch_ms = 0.0;
    let mut readback_ms = 0.0;
    let mut output_row_start = 0usize;

    while output_row_start < height {
        let output_rows = stripe_rows.min(height - output_row_start);
        let pad_top = usize::from(output_row_start > 0);
        let pad_bottom = usize::from(output_row_start + output_rows < height);
        let sub_start = output_row_start.saturating_sub(pad_top);
        let sub_end = (output_row_start + output_rows + pad_bottom).min(height);
        let sub_inputs = slice_inputs_rows(inputs, sub_start, sub_end);
        let sub_result = execute_host_minimum_chunk(
            device,
            queue,
            pipeline,
            bind_group_layout,
            &sub_inputs,
            parameters,
        )?;
        let row_stride = width;
        let kept_start = pad_top * row_stride;
        let kept_len = output_rows * row_stride;
        let kept_end = kept_start + kept_len;
        trust.extend_from_slice(&sub_result.trust[kept_start..kept_end]);
        alpha.extend_from_slice(&sub_result.alpha[kept_start..kept_end]);
        intervention.extend_from_slice(&sub_result.intervention[kept_start..kept_end]);
        total_ms += sub_result.total_ms;
        dispatch_ms += sub_result.dispatch_ms;
        readback_ms += sub_result.readback_ms;
        output_row_start += output_rows;
    }

    Ok(ChunkExecutionResult {
        trust,
        alpha,
        intervention,
        total_ms,
        dispatch_ms,
        readback_ms,
    })
}

fn execute_host_minimum_chunk(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
    bind_group_layout: &wgpu::BindGroupLayout,
    inputs: &OwnedHostTemporalInputs,
    parameters: HostSupervisionParameters,
) -> Result<ChunkExecutionResult> {
    let pixel_count = inputs.width() * inputs.height();
    let color_current = pack_colors(&inputs.current_color);
    let color_history = pack_colors(&inputs.reprojected_history);
    let depth_pairs = pack_depth_pairs(&inputs.current_depth, &inputs.reprojected_depth);
    let normal_pairs = pack_normal_pairs(&inputs.current_normals, &inputs.reprojected_normals);
    let params = pack_params(inputs.width(), inputs.height(), parameters);

    let current_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("current-color"),
        contents: bytemuck::cast_slice(&color_current),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let history_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("reprojected-history"),
        contents: bytemuck::cast_slice(&color_history),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let depth_pairs_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("depth-pairs"),
        contents: bytemuck::cast_slice(&depth_pairs),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let normal_pairs_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("normal-pairs"),
        contents: bytemuck::cast_slice(&normal_pairs),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let output_size = (pixel_count * std::mem::size_of::<f32>()) as u64;
    let trust_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("trust-output"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let alpha_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("alpha-output"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let intervention_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("intervention-output"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let trust_staging = create_staging_buffer(device, output_size, "trust-staging");
    let alpha_staging = create_staging_buffer(device, output_size, "alpha-staging");
    let intervention_staging = create_staging_buffer(device, output_size, "intervention-staging");
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("dsfb-host-minimum-bind-group"),
        layout: bind_group_layout,
        entries: &[
            storage_binding(0, &current_buffer),
            storage_binding(1, &history_buffer),
            storage_binding(2, &depth_pairs_buffer),
            storage_binding(3, &normal_pairs_buffer),
            uniform_binding(4, &params_buffer),
            storage_binding(5, &trust_buffer),
            storage_binding(6, &alpha_buffer),
            storage_binding(7, &intervention_buffer),
        ],
    });

    let total_start = Instant::now();
    let dispatch_start = Instant::now();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("dsfb-host-minimum-encoder"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("dsfb-host-minimum-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let groups_x = inputs.width() as u32;
        let groups_y = inputs.height() as u32;
        pass.dispatch_workgroups(groups_x, groups_y, 1);
    }
    encoder.copy_buffer_to_buffer(&trust_buffer, 0, &trust_staging, 0, output_size);
    encoder.copy_buffer_to_buffer(&alpha_buffer, 0, &alpha_staging, 0, output_size);
    encoder.copy_buffer_to_buffer(
        &intervention_buffer,
        0,
        &intervention_staging,
        0,
        output_size,
    );
    queue.submit(Some(encoder.finish()));
    device.poll(wgpu::Maintain::Wait);
    let dispatch_ms = dispatch_start.elapsed().as_secs_f64() * 1000.0;

    let readback_start = Instant::now();
    let trust = read_f32_buffer(device, &trust_staging, pixel_count)?;
    let alpha = read_f32_buffer(device, &alpha_staging, pixel_count)?;
    let intervention = read_f32_buffer(device, &intervention_staging, pixel_count)?;
    let readback_ms = readback_start.elapsed().as_secs_f64() * 1000.0;

    Ok(ChunkExecutionResult {
        trust,
        alpha,
        intervention,
        total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
        dispatch_ms,
        readback_ms,
    })
}

fn slice_inputs_rows(
    inputs: &OwnedHostTemporalInputs,
    row_start: usize,
    row_end: usize,
) -> OwnedHostTemporalInputs {
    let height = row_end.saturating_sub(row_start);
    let width = inputs.width();
    OwnedHostTemporalInputs {
        current_color: slice_frame_rows(&inputs.current_color, row_start, row_end),
        reprojected_history: slice_frame_rows(&inputs.reprojected_history, row_start, row_end),
        motion_vectors: slice_rows(&inputs.motion_vectors, width, row_start, row_end),
        current_depth: slice_rows(&inputs.current_depth, width, row_start, row_end),
        reprojected_depth: slice_rows(&inputs.reprojected_depth, width, row_start, row_end),
        current_normals: slice_rows(&inputs.current_normals, width, row_start, row_end),
        reprojected_normals: slice_rows(&inputs.reprojected_normals, width, row_start, row_end),
        visibility_hint: inputs
            .visibility_hint
            .as_ref()
            .map(|mask| slice_rows(mask, width, row_start, row_end)),
        thin_hint: inputs
            .thin_hint
            .as_ref()
            .map(|field| ScalarField::from_values(width, height, slice_rows(field.values(), width, row_start, row_end))),
    }
}

fn slice_frame_rows(frame: &crate::frame::ImageFrame, row_start: usize, row_end: usize) -> crate::frame::ImageFrame {
    let width = frame.width();
    let height = row_end.saturating_sub(row_start);
    let mut pixels = Vec::with_capacity(width * height);
    for y in row_start..row_end {
        for x in 0..width {
            pixels.push(frame.get(x, y));
        }
    }
    crate::frame::ImageFrame::from_pixels(width, height, pixels)
}

fn slice_rows<T: Copy>(values: &[T], width: usize, row_start: usize, row_end: usize) -> Vec<T> {
    let start = row_start * width;
    let end = row_end * width;
    values[start..end].to_vec()
}

fn create_staging_buffer(device: &wgpu::Device, size: u64, label: &str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn read_f32_buffer(device: &wgpu::Device, buffer: &wgpu::Buffer, count: usize) -> Result<Vec<f32>> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device.poll(wgpu::Maintain::Wait);
    receiver
        .recv()
        .map_err(|_| Error::Message("failed to receive GPU map_async status".to_string()))?
        .map_err(|error| Error::Message(format!("failed to map GPU staging buffer: {error}")))?;
    let mapped = slice.get_mapped_range();
    let values = bytemuck::cast_slice::<u8, f32>(&mapped).to_vec();
    drop(mapped);
    buffer.unmap();
    if values.len() != count {
        return Err(Error::Message(format!(
            "GPU readback size mismatch: expected {count} floats, got {}",
            values.len()
        )));
    }
    Ok(values)
}

fn pack_colors(frame: &crate::frame::ImageFrame) -> Vec<GpuColor> {
    frame
        .pixels()
        .iter()
        .map(|pixel| GpuColor {
            value: [pixel.r, pixel.g, pixel.b, 1.0],
        })
        .collect()
}

fn pack_depth_pairs(current: &[f32], history: &[f32]) -> Vec<GpuDepthPair> {
    current
        .iter()
        .zip(history.iter())
        .map(|(current, history)| GpuDepthPair {
            value: [*current, *history],
        })
        .collect()
}

fn pack_normal_pairs(
    current: &[crate::scene::Normal3],
    history: &[crate::scene::Normal3],
) -> Vec<GpuNormalPair> {
    current
        .iter()
        .zip(history.iter())
        .map(|(current, history)| GpuNormalPair {
            current: [current.x, current.y, current.z, 0.0],
            history: [history.x, history.y, history.z, 0.0],
        })
        .collect()
}

fn pack_params(width: usize, height: usize, parameters: HostSupervisionParameters) -> GpuParams {
    GpuParams {
        size: [width as u32, height as u32, 0, 0],
        alpha_range: [
            parameters.alpha_range.min,
            parameters.alpha_range.max,
            0.0,
            0.0,
        ],
        residual_threshold: [
            parameters.thresholds.residual.low,
            parameters.thresholds.residual.high,
            0.0,
            0.0,
        ],
        depth_threshold: [
            parameters.thresholds.depth.low,
            parameters.thresholds.depth.high,
            0.0,
            0.0,
        ],
        normal_threshold: [
            parameters.thresholds.normal.low,
            parameters.thresholds.normal.high,
            0.0,
            0.0,
        ],
        neighborhood_threshold: [
            parameters.thresholds.neighborhood.low,
            parameters.thresholds.neighborhood.high,
            0.0,
            0.0,
        ],
        local_contrast_threshold: [
            parameters.thresholds.local_contrast.low,
            parameters.thresholds.local_contrast.high,
            0.0,
            0.0,
        ],
        hazard_curve_threshold: [
            parameters.thresholds.hazard_curve.low,
            parameters.thresholds.hazard_curve.high,
            0.0,
            0.0,
        ],
        weights_a: [
            parameters.weights.residual,
            parameters.weights.depth,
            parameters.weights.normal,
            parameters.weights.neighborhood,
        ],
        weights_b: [
            parameters.weights.thin,
            parameters.weights.history_instability,
            parameters.weights.grammar,
            0.0,
        ],
        history_instability_mix: [
            parameters.thresholds.history_instability_residual_mix,
            parameters.thresholds.history_instability_neighborhood_mix,
            0.0,
            0.0,
        ],
        structural_a: [
            parameters.structural.disocclusion_like,
            parameters.structural.unstable_residual,
            parameters.structural.unstable_neighborhood,
            0.0,
        ],
        structural_b: [
            parameters.structural.thin_edge,
            parameters.structural.thin_residual,
            0.0,
            0.0,
        ],
    }
}

fn storage_layout_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_binding<'a>(binding: u32, buffer: &'a wgpu::Buffer) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn uniform_binding<'a>(binding: u32, buffer: &'a wgpu::Buffer) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}
