//! Minimal Inline Deployment (Fast-Path Proxy)
//!
//! This module implements a reduced per-pixel proxy derived from residual evolution.
//! It is NOT the full DSFB supervisory system. It is a constrained proxy
//! intended to assess deployment feasibility under strict real-time budgets.
//!
//! The proxy computes per pixel:
//!   r_t = L1(C_t - H_t) / 3          residual magnitude
//!   d_t = r_t - r_{t-1}               drift
//!   s_t = d_t - d_{t-1}               slew
//!   u_t = |d_t| + lambda * |s_t|      scalar proxy
//!   T_t = saturate(1 - k * u_t)       trust
//!
//! Optionally, u_t is averaged over a 3×3 neighborhood before trust computation.
//!
//! State: two scalar per-pixel history buffers (residual, drift).
//! Output: one scalar per-pixel trust buffer.
//!
//! No heap allocation occurs in the inner loop of the CPU reference path.

use std::sync::mpsc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use serde::Serialize;
use wgpu::util::DeviceExt;

use crate::error::{Error, Result};

// ─── Constants ────────────────────────────────────────────────────────────────

/// Default slew weight λ.  Applied to |s_t| in the proxy formula.
pub const FAST_PATH_LAMBDA: f32 = 0.5;

/// Default trust slope k.  Scales the proxy before saturation.
pub const FAST_PATH_K: f32 = 2.0;

// ─── CPU reference ────────────────────────────────────────────────────────────

/// Output from one frame of the CPU reference fast-path computation.
pub struct FastPathCpuOutput {
    /// Trust values, one f32 per pixel, in \[0, 1\].
    pub trust: Vec<f32>,
    /// Updated residual history (r_t), to be fed as `residual_history_in` next frame.
    pub residual_history_out: Vec<f32>,
    /// Updated drift history (d_t), to be fed as `drift_history_in` next frame.
    pub drift_history_out: Vec<f32>,
}

/// CPU reference implementation of the minimal inline fast-path proxy.
///
/// Inputs are flat slices of per-pixel \[R, G, B\] triples.
/// All output `Vec`s are pre-allocated before the inner loop; no allocations
/// occur inside the per-pixel computation.
///
/// This function is deterministic: identical inputs produce identical outputs.
pub fn run_fast_path_cpu(
    current: &[[f32; 3]],
    history: &[[f32; 3]],
    residual_history_in: &[f32],
    drift_history_in: &[f32],
    width: usize,
    height: usize,
    lambda: f32,
    k: f32,
    local_aggregation: bool,
) -> FastPathCpuOutput {
    let pixel_count = width * height;
    assert_eq!(current.len(), pixel_count);
    assert_eq!(history.len(), pixel_count);
    assert_eq!(residual_history_in.len(), pixel_count);
    assert_eq!(drift_history_in.len(), pixel_count);

    // Pre-allocate output buffers before the inner computation loop.
    // No allocation occurs inside the per-pixel loops below.
    let mut trust = vec![0.0f32; pixel_count];
    let mut residual_history_out = vec![0.0f32; pixel_count];
    let mut drift_history_out = vec![0.0f32; pixel_count];
    let mut u_field = vec![0.0f32; pixel_count];

    // First pass: compute r_t, d_t, s_t, u_t for all pixels.
    // History outputs (r_t, d_t) are always written regardless of aggregation mode.
    for i in 0..pixel_count {
        let c = current[i];
        let h = history[i];
        let r_t = ((c[0] - h[0]).abs() + (c[1] - h[1]).abs() + (c[2] - h[2]).abs()) / 3.0;
        let d_t = r_t - residual_history_in[i];
        let s_t = d_t - drift_history_in[i];
        let u_t = d_t.abs() + lambda * s_t.abs();
        u_field[i] = u_t;
        residual_history_out[i] = r_t;
        drift_history_out[i] = d_t;
    }

    if local_aggregation {
        // Optional 3×3 mean of u_t before trust computation.
        // Uses only stack-local accumulators; no additional heap allocation inside the loop.
        for y in 0..height {
            for x in 0..width {
                let mut u_sum = 0.0f32;
                for oy in -1i32..=1 {
                    for ox in -1i32..=1 {
                        let nx = (x as i32 + ox).clamp(0, width as i32 - 1) as usize;
                        let ny = (y as i32 + oy).clamp(0, height as i32 - 1) as usize;
                        u_sum += u_field[ny * width + nx];
                    }
                }
                let u_prime = u_sum / 9.0;
                trust[y * width + x] = (1.0 - k * u_prime).clamp(0.0, 1.0);
            }
        }
    } else {
        for i in 0..pixel_count {
            trust[i] = (1.0 - k * u_field[i]).clamp(0.0, 1.0);
        }
    }

    FastPathCpuOutput {
        trust,
        residual_history_out,
        drift_history_out,
    }
}

// ─── GPU shader ───────────────────────────────────────────────────────────────

const FAST_PATH_SHADER: &str = r#"
struct Params {
    // [width, height, has_local_agg (0=off, 1=on), _padding]
    size: vec4<u32>,
    // [lambda, k, _, _]
    coefficients: vec4<f32>,
}

@group(0) @binding(0) var<storage, read>       current_color:        array<vec4<f32>>;
@group(0) @binding(1) var<storage, read>       reprojected_history:  array<vec4<f32>>;
@group(0) @binding(2) var<storage, read>       residual_history_in:  array<f32>;
@group(0) @binding(3) var<storage, read>       drift_history_in:     array<f32>;
@group(0) @binding(4) var<uniform>             params:               Params;
@group(0) @binding(5) var<storage, read_write> trust_out:            array<f32>;
@group(0) @binding(6) var<storage, read_write> residual_history_out: array<f32>;
@group(0) @binding(7) var<storage, read_write> drift_history_out:    array<f32>;

fn l1_residual(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return (abs(a.x - b.x) + abs(a.y - b.y) + abs(a.z - b.z)) / 3.0;
}

/// Compute the proxy scalar u for an arbitrary (clamped) pixel coordinate.
/// Used by the optional 3×3 local aggregation path.
fn proxy_u_at(xi: i32, yi: i32) -> f32 {
    let w = i32(params.size.x);
    let h = i32(params.size.y);
    let xc = clamp(xi, 0, w - 1);
    let yc = clamp(yi, 0, h - 1);
    let j = u32(yc) * params.size.x + u32(xc);
    let cur  = current_color[j].xyz;
    let hist = reprojected_history[j].xyz;
    let r_prev = residual_history_in[j];
    let d_prev = drift_history_in[j];
    let r_j = l1_residual(cur, hist);
    let d_j = r_j - r_prev;
    let s_j = d_j - d_prev;
    return abs(d_j) + params.coefficients.x * abs(s_j);
}

/// Single-pass fast-path proxy kernel.
///
/// Computes r_t, d_t, s_t, u_t per pixel.
/// Writes trust (with optional 3×3 local mean of u), r_t, and d_t.
@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = params.size.x;
    let h = params.size.y;
    if (gid.x >= w || gid.y >= h) { return; }

    let i = gid.y * w + gid.x;
    let lambda = params.coefficients.x;
    let k      = params.coefficients.y;

    let cur  = current_color[i].xyz;
    let hist = reprojected_history[i].xyz;
    let r_prev = residual_history_in[i];
    let d_prev = drift_history_in[i];

    let r_t = l1_residual(cur, hist);
    let d_t = r_t - r_prev;
    let s_t = d_t - d_prev;

    var trust: f32;
    if (params.size.z == 1u) {
        // Optional 3×3 local mean of u before trust.
        var u_sum = 0.0;
        for (var oy: i32 = -1; oy <= 1; oy = oy + 1) {
            for (var ox: i32 = -1; ox <= 1; ox = ox + 1) {
                u_sum = u_sum + proxy_u_at(i32(gid.x) + ox, i32(gid.y) + oy);
            }
        }
        trust = clamp(1.0 - k * (u_sum / 9.0), 0.0, 1.0);
    } else {
        let u_t = abs(d_t) + lambda * abs(s_t);
        trust = clamp(1.0 - k * u_t, 0.0, 1.0);
    }

    trust_out[i]            = trust;
    residual_history_out[i] = r_t;
    drift_history_out[i]    = d_t;
}
"#;

// ─── GPU types ────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FpGpuParams {
    size: [u32; 4],
    coefficients: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FpColor4 {
    value: [f32; 4],
}

// ─── GPU result ───────────────────────────────────────────────────────────────

/// Result from a single-frame GPU fast-path kernel execution.
#[derive(Clone, Debug, Serialize)]
pub struct FastPathGpuOutput {
    pub trust: Vec<f32>,
    pub residual_history_out: Vec<f32>,
    pub drift_history_out: Vec<f32>,
    /// Wall-clock time including dispatch + readback (ms).
    pub total_ms: f64,
    /// Wall-clock time for dispatch + poll only (ms).
    pub dispatch_ms: f64,
    pub adapter_name: String,
    pub backend: String,
}

/// Execute one frame of the GPU fast-path proxy.
///
/// Returns `Ok(None)` if no wgpu adapter is available.
pub fn try_run_fast_path_gpu(
    current: &[[f32; 3]],
    history: &[[f32; 3]],
    residual_history_in: &[f32],
    drift_history_in: &[f32],
    width: usize,
    height: usize,
    lambda: f32,
    k: f32,
    local_aggregation: bool,
) -> Result<Option<FastPathGpuOutput>> {
    pollster::block_on(try_run_fast_path_gpu_async(
        current,
        history,
        residual_history_in,
        drift_history_in,
        width,
        height,
        lambda,
        k,
        local_aggregation,
    ))
}

async fn try_run_fast_path_gpu_async(
    current: &[[f32; 3]],
    history: &[[f32; 3]],
    residual_history_in: &[f32],
    drift_history_in: &[f32],
    width: usize,
    height: usize,
    lambda: f32,
    k: f32,
    local_aggregation: bool,
) -> Result<Option<FastPathGpuOutput>> {
    let instance = wgpu::Instance::default();
    let adapter = match instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
    {
        Some(a) => a,
        None => return Ok(None),
    };
    let adapter_info = adapter.get_info();
    let adapter_limits = adapter.limits();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("dsfb-fast-path"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_storage_buffer_binding_size: adapter_limits
                        .max_storage_buffer_binding_size,
                    max_buffer_size: adapter_limits.max_buffer_size,
                    ..wgpu::Limits::default()
                },
            },
            None,
        )
        .await
        .map_err(|e| Error::Message(format!("wgpu device request failed: {e}")))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("dsfb-fast-path-wgsl"),
        source: wgpu::ShaderSource::Wgsl(FAST_PATH_SHADER.into()),
    });
    let bind_group_layout = make_bind_group_layout(&device);
    let pipeline = make_pipeline(&device, &shader, &bind_group_layout);

    let pixel_count = width * height;
    let current_packed = pack_colors(current);
    let history_packed = pack_colors(history);
    let params_val = FpGpuParams {
        size: [width as u32, height as u32, u32::from(local_aggregation), 0],
        coefficients: [lambda, k, 0.0, 0.0],
    };

    let out_bytes = (pixel_count * std::mem::size_of::<f32>()) as u64;
    let (
        cur_buf,
        hist_buf,
        res_in_buf,
        drift_in_buf,
        params_buf,
        trust_buf,
        res_out_buf,
        drift_out_buf,
    ) = upload_gpu_buffers(
        &device,
        &current_packed,
        &history_packed,
        residual_history_in,
        drift_history_in,
        &params_val,
        pixel_count,
    );
    let trust_staging = make_staging(&device, out_bytes, "fp-trust-stg");
    let res_staging = make_staging(&device, out_bytes, "fp-res-stg");
    let drift_staging = make_staging(&device, out_bytes, "fp-drift-stg");

    let bind_group = make_bind_group(
        &device,
        &bind_group_layout,
        &cur_buf,
        &hist_buf,
        &res_in_buf,
        &drift_in_buf,
        &params_buf,
        &trust_buf,
        &res_out_buf,
        &drift_out_buf,
    );

    let total_start = Instant::now();
    let dispatch_start = Instant::now();
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fp-enc") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("fp-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(width as u32, height as u32, 1);
    }
    encoder.copy_buffer_to_buffer(&trust_buf, 0, &trust_staging, 0, out_bytes);
    encoder.copy_buffer_to_buffer(&res_out_buf, 0, &res_staging, 0, out_bytes);
    encoder.copy_buffer_to_buffer(&drift_out_buf, 0, &drift_staging, 0, out_bytes);
    queue.submit(Some(encoder.finish()));
    device.poll(wgpu::Maintain::Wait);
    let dispatch_ms = dispatch_start.elapsed().as_secs_f64() * 1000.0;

    let trust = read_f32_buf(&device, &trust_staging, pixel_count)?;
    let residual_out = read_f32_buf(&device, &res_staging, pixel_count)?;
    let drift_out = read_f32_buf(&device, &drift_staging, pixel_count)?;
    let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

    Ok(Some(FastPathGpuOutput {
        trust,
        residual_history_out: residual_out,
        drift_history_out: drift_out,
        total_ms,
        dispatch_ms,
        adapter_name: adapter_info.name,
        backend: format!("{:?}", adapter_info.backend),
    }))
}

// ─── Timing study ─────────────────────────────────────────────────────────────

/// One row in the fast-path timing study output.
#[derive(Clone, Debug, Serialize)]
pub struct FastPathTimingEntry {
    pub resolution_label: String,
    pub width: usize,
    pub height: usize,
    pub pixel_count: usize,
    pub warmup_runs: usize,
    pub measured_runs: usize,
    pub local_aggregation: bool,
    /// Mean wall-clock dispatch+poll time across measured runs (ms).
    pub mean_dispatch_ms: f64,
    /// Mean wall-clock total (dispatch+readback) time across measured runs (ms).
    pub mean_total_ms: f64,
    pub min_dispatch_ms: f64,
    pub max_dispatch_ms: f64,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub actual_gpu_timing: bool,
    pub notes: Vec<String>,
}

/// Full result from `run_fast_path_timing_study`.
#[derive(Clone, Debug, Serialize)]
pub struct FastPathTimingStudy {
    pub measurement_kind: String,
    pub actual_gpu_timing: bool,
    pub lambda: f32,
    pub k: f32,
    pub entries: Vec<FastPathTimingEntry>,
    pub notes: Vec<String>,
}

/// Run the GPU timing study for the fast-path proxy at 1080p and 4K.
///
/// Uses synthetic uniform-colour inputs at each resolution.
/// Runs `warmup_runs` iterations before measuring, then averages `measured_runs`.
///
/// Returns measured timings if a wgpu adapter is available; otherwise returns a
/// study with `actual_gpu_timing = false` and no entries.
pub fn run_fast_path_timing_study() -> Result<FastPathTimingStudy> {
    let resolutions: &[(&str, usize, usize)] = if cfg!(debug_assertions) {
        // Reduced sizes for debug/test builds.
        &[("854x480_debug", 854, 480), ("1280x720_debug", 1280, 720)]
    } else {
        &[("1920x1080", 1920, 1080), ("3840x2160", 3840, 2160)]
    };

    let warmup_runs = 3usize;
    let measured_runs = 10usize;

    // Acquire GPU device once; reuse across all resolution tests.
    let gpu = pollster::block_on(acquire_device());
    let (device, queue, adapter_name, backend) = match gpu {
        Some(g) => g,
        None => {
            return Ok(FastPathTimingStudy {
                measurement_kind: "gpu_unavailable_no_measurement".to_string(),
                actual_gpu_timing: false,
                lambda: FAST_PATH_LAMBDA,
                k: FAST_PATH_K,
                entries: vec![],
                notes: vec![
                    "No wgpu adapter was available. GPU timing could not be measured.".to_string(),
                ],
            });
        }
    };

    // Build shader + pipeline once; shared across all resolutions.
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fp-timing-shader"),
        source: wgpu::ShaderSource::Wgsl(FAST_PATH_SHADER.into()),
    });
    let bgl = make_bind_group_layout(&device);
    let pipeline = make_pipeline(&device, &shader, &bgl);

    let mut entries = Vec::new();

    for &(label, width, height) in resolutions {
        let pixel_count = width * height;
        // Synthetic inputs: uniform mid-grey current, slightly darker history, zero histories.
        let current_rgb: Vec<[f32; 3]> = vec![[0.5f32, 0.5, 0.5]; pixel_count];
        let history_rgb: Vec<[f32; 3]> = vec![[0.48f32, 0.48, 0.48]; pixel_count];
        let res_in = vec![0.0f32; pixel_count];
        let drift_in = vec![0.0f32; pixel_count];

        let current_packed = pack_colors(&current_rgb);
        let history_packed = pack_colors(&history_rgb);
        let params_val = FpGpuParams {
            size: [width as u32, height as u32, 0, 0],
            coefficients: [FAST_PATH_LAMBDA, FAST_PATH_K, 0.0, 0.0],
        };
        let out_bytes = (pixel_count * std::mem::size_of::<f32>()) as u64;

        let (
            cur_buf,
            hist_buf,
            res_in_buf,
            drift_in_buf,
            params_buf,
            trust_buf,
            res_out_buf,
            drift_out_buf,
        ) = upload_gpu_buffers(
            &device,
            &current_packed,
            &history_packed,
            &res_in,
            &drift_in,
            &params_val,
            pixel_count,
        );
        let trust_staging = make_staging(&device, out_bytes, "fp-timing-stg");
        let bind_group = make_bind_group(
            &device,
            &bgl,
            &cur_buf,
            &hist_buf,
            &res_in_buf,
            &drift_in_buf,
            &params_buf,
            &trust_buf,
            &res_out_buf,
            &drift_out_buf,
        );

        // Warmup.
        for _ in 0..warmup_runs {
            let mut enc = device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(width as u32, height as u32, 1);
            }
            queue.submit(Some(enc.finish()));
            device.poll(wgpu::Maintain::Wait);
        }

        // Measured runs.
        let mut dispatch_times = Vec::with_capacity(measured_runs);
        let mut total_times = Vec::with_capacity(measured_runs);
        for _ in 0..measured_runs {
            let t0 = Instant::now();
            let d0 = Instant::now();
            let mut enc = device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(width as u32, height as u32, 1);
            }
            // Include one buffer copy (trust only) to bound actual transfer cost.
            enc.copy_buffer_to_buffer(&trust_buf, 0, &trust_staging, 0, out_bytes);
            queue.submit(Some(enc.finish()));
            device.poll(wgpu::Maintain::Wait);
            dispatch_times.push(d0.elapsed().as_secs_f64() * 1000.0);
            total_times.push(t0.elapsed().as_secs_f64() * 1000.0);
        }

        let n = measured_runs as f64;
        let mean_dispatch = dispatch_times.iter().sum::<f64>() / n;
        let mean_total = total_times.iter().sum::<f64>() / n;
        let min_d = dispatch_times.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_d = dispatch_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        entries.push(FastPathTimingEntry {
            resolution_label: label.to_string(),
            width,
            height,
            pixel_count,
            warmup_runs,
            measured_runs,
            local_aggregation: false,
            mean_dispatch_ms: mean_dispatch,
            mean_total_ms: mean_total,
            min_dispatch_ms: min_d,
            max_dispatch_ms: max_d,
            adapter_name: Some(adapter_name.clone()),
            backend: Some(backend.clone()),
            actual_gpu_timing: true,
            notes: vec![
                "Timing uses wgpu dispatch + Maintain::Wait (CPU-side wall clock).".to_string(),
                "Inputs are synthetic uniform-colour buffers; no real capture data required.".to_string(),
                "This reflects the reduced proxy only, not the full DSFB supervisory system.".to_string(),
            ],
        });
    }

    Ok(FastPathTimingStudy {
        measurement_kind: "gpu_fast_path_proxy_cpu_wall_clock".to_string(),
        actual_gpu_timing: true,
        lambda: FAST_PATH_LAMBDA,
        k: FAST_PATH_K,
        entries,
        notes: vec![
            "These timings reflect the minimal inline deployment proxy.".to_string(),
            "They must not be interpreted as the cost of the full DSFB supervisory system.".to_string(),
        ],
    })
}

// ─── GPU helpers ──────────────────────────────────────────────────────────────

async fn acquire_device() -> Option<(wgpu::Device, wgpu::Queue, String, String)> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await?;
    let info = adapter.get_info();
    let adapter_limits = adapter.limits();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("dsfb-fast-path-timing"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_storage_buffer_binding_size: adapter_limits
                        .max_storage_buffer_binding_size,
                    max_buffer_size: adapter_limits.max_buffer_size,
                    ..wgpu::Limits::default()
                },
            },
            None,
        )
        .await
        .ok()?;
    Some((device, queue, info.name, format!("{:?}", info.backend)))
}

fn make_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fp-bgl"),
        entries: &[
            storage_entry(0, true),
            storage_entry(1, true),
            storage_entry(2, true),
            storage_entry(3, true),
            uniform_entry(4),
            storage_entry(5, false),
            storage_entry(6, false),
            storage_entry(7, false),
        ],
    })
}

fn make_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    bgl: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[bgl],
        push_constant_ranges: &[],
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("dsfb-fast-path-pipeline"),
        layout: Some(&layout),
        module: shader,
        entry_point: "main",
    })
}

#[allow(clippy::too_many_arguments)]
fn upload_gpu_buffers(
    device: &wgpu::Device,
    current_packed: &[FpColor4],
    history_packed: &[FpColor4],
    residual_in: &[f32],
    drift_in: &[f32],
    params: &FpGpuParams,
    pixel_count: usize,
) -> (
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
) {
    let out_size = (pixel_count * std::mem::size_of::<f32>()) as u64;
    let cur = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fp-cur"),
        contents: bytemuck::cast_slice(current_packed),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let hist = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fp-hist"),
        contents: bytemuck::cast_slice(history_packed),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let res_in = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fp-res-in"),
        contents: bytemuck::cast_slice(residual_in),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let drift_in_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fp-drift-in"),
        contents: bytemuck::cast_slice(drift_in),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fp-params"),
        contents: bytemuck::bytes_of(params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let trust_out = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fp-trust"),
        size: out_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let res_out = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fp-res-out"),
        size: out_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let drift_out = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fp-drift-out"),
        size: out_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    (cur, hist, res_in, drift_in_buf, params_buf, trust_out, res_out, drift_out)
}

#[allow(clippy::too_many_arguments)]
fn make_bind_group(
    device: &wgpu::Device,
    bgl: &wgpu::BindGroupLayout,
    cur: &wgpu::Buffer,
    hist: &wgpu::Buffer,
    res_in: &wgpu::Buffer,
    drift_in: &wgpu::Buffer,
    params: &wgpu::Buffer,
    trust_out: &wgpu::Buffer,
    res_out: &wgpu::Buffer,
    drift_out: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: bgl,
        entries: &[
            buf_entry(0, cur),
            buf_entry(1, hist),
            buf_entry(2, res_in),
            buf_entry(3, drift_in),
            buf_entry(4, params),
            buf_entry(5, trust_out),
            buf_entry(6, res_out),
            buf_entry(7, drift_out),
        ],
    })
}

fn make_staging(device: &wgpu::Device, size: u64, label: &str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn read_f32_buf(device: &wgpu::Device, buf: &wgpu::Buffer, count: usize) -> Result<Vec<f32>> {
    let slice = buf.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|_| Error::Message("GPU map_async channel closed".to_string()))?
        .map_err(|e| Error::Message(format!("GPU map_async failed: {e}")))?;
    let mapped = slice.get_mapped_range();
    let values: Vec<f32> = bytemuck::cast_slice(&mapped).to_vec();
    drop(mapped);
    buf.unmap();
    if values.len() != count {
        return Err(Error::Message(format!(
            "GPU readback: expected {count} f32 values, got {}",
            values.len()
        )));
    }
    Ok(values)
}

fn pack_colors(pixels: &[[f32; 3]]) -> Vec<FpColor4> {
    pixels
        .iter()
        .map(|&[r, g, b]| FpColor4 { value: [r, g, b, 1.0] })
        .collect()
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
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

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
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

fn buf_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

// ─── SVG trust visualisation ──────────────────────────────────────────────────

/// Render a trust field as a simple SVG heat-map strip (one rect per pixel column).
///
/// Used only for artifact visualisation; not part of the core computation.
pub fn render_trust_strip_svg(trust: &[f32], width: usize, height: usize) -> String {
    let sample_width = width.min(256);
    let step = width / sample_width.max(1);
    let bar_h = 40usize;
    let total_w = sample_width * 2;
    let total_h = bar_h + 20;
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}">"#
    );
    // Sample the middle row.
    let mid_y = height / 2;
    for col in 0..sample_width {
        let src_x = (col * step).min(width.saturating_sub(1));
        let i = mid_y * width + src_x;
        let t = trust.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0);
        let g = (t * 220.0) as u8;
        let r = ((1.0 - t) * 220.0) as u8;
        svg.push_str(&format!(
            r#"<rect x="{}" y="0" width="2" height="{bar_h}" fill="rgb({r},{g},80)"/>"#,
            col * 2
        ));
    }
    svg.push_str(&format!(
        "<text x=\"4\" y=\"{}\" font-size=\"10\" fill=\"#444\">trust (mid-row, {}x{})</text>",
        bar_h + 14,
        width,
        height
    ));
    svg.push_str("</svg>");
    svg
}
