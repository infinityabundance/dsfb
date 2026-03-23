use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::config::DemoConfig;
use crate::error::{Error, Result};
use crate::external::build_owned_inputs_from_sequence;
use crate::gpu::try_execute_host_minimum_kernel;
use crate::host::{default_host_realistic_profile, supervise_temporal_reuse};
use crate::report::EXPERIMENT_SENTENCE;
use crate::scene::{generate_sequence_for_definition, scenario_by_id, ScenarioId};
use crate::taa::run_fixed_alpha_baseline;

#[derive(Clone, Debug, Serialize)]
pub struct GpuExecutionEntry {
    pub label: String,
    pub scenario_id: String,
    pub width: usize,
    pub height: usize,
    pub frame_index: usize,
    pub gpu_path_available: bool,
    pub actual_gpu_timing_measured: bool,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub total_ms: Option<f64>,
    pub dispatch_ms: Option<f64>,
    pub readback_ms: Option<f64>,
    pub mean_abs_trust_delta_vs_cpu: Option<f32>,
    pub mean_abs_alpha_delta_vs_cpu: Option<f32>,
    pub mean_abs_intervention_delta_vs_cpu: Option<f32>,
    pub workgroup_size: [u32; 3],
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GpuExecutionMetrics {
    pub measurement_kind: String,
    pub actual_gpu_timing_measured: bool,
    pub shader_language: String,
    pub kernel_name: String,
    pub entries: Vec<GpuExecutionEntry>,
    pub notes: Vec<String>,
}

pub fn run_gpu_execution_study(config: &DemoConfig) -> Result<GpuExecutionMetrics> {
    let scenarios = [ScenarioId::RevealBand, ScenarioId::MotionBiasBand];
    let mut entries = Vec::new();
    let mut any_measured = false;

    for scenario_id in scenarios {
        let definition = scenario_by_id(&config.scene, scenario_id).ok_or_else(|| {
            Error::Message(format!(
                "GPU execution scenario {} was unavailable",
                scenario_id.as_str()
            ))
        })?;
        let sequence = generate_sequence_for_definition(&definition);
        let frame_index = definition
            .onset_frame
            .min(sequence.frames.len().saturating_sub(1))
            .max(1);
        let fixed_alpha = run_fixed_alpha_baseline(&sequence, config.baseline.fixed_alpha);
        let previous_history = fixed_alpha.taa.resolved_frames.get(frame_index - 1);
        let inputs = build_owned_inputs_from_sequence(&sequence, frame_index, previous_history)?;
        let profile =
            default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
        let cpu_outputs = supervise_temporal_reuse(&inputs.borrow(), &profile);

        let maybe_gpu = try_execute_host_minimum_kernel(&inputs, profile.parameters)?;
        match maybe_gpu {
            Some(gpu) => {
                any_measured = true;
                entries.push(GpuExecutionEntry {
                    label: format!("gpu_host_minimum_{}", scenario_id.as_str()),
                    scenario_id: scenario_id.as_str().to_string(),
                    width: inputs.width(),
                    height: inputs.height(),
                    frame_index,
                    gpu_path_available: true,
                    actual_gpu_timing_measured: true,
                    adapter_name: Some(gpu.adapter_name),
                    backend: Some(gpu.backend),
                    total_ms: Some(gpu.total_ms),
                    dispatch_ms: Some(gpu.dispatch_ms),
                    readback_ms: Some(gpu.readback_ms),
                    mean_abs_trust_delta_vs_cpu: Some(mean_abs_delta(
                        cpu_outputs.trust.values(),
                        &gpu.trust,
                    )),
                    mean_abs_alpha_delta_vs_cpu: Some(mean_abs_delta(
                        cpu_outputs.alpha.values(),
                        &gpu.alpha,
                    )),
                    mean_abs_intervention_delta_vs_cpu: Some(mean_abs_delta(
                        cpu_outputs.intervention.values(),
                        &gpu.intervention,
                    )),
                    workgroup_size: [gpu.workgroup_size.0, gpu.workgroup_size.1, gpu.workgroup_size.2],
                    notes: vec![
                        "Measured on the current environment because a usable wgpu adapter was available.".to_string(),
                        "The kernel implements the current minimum host-realistic path, which excludes motion disagreement by design.".to_string(),
                    ],
                });
            }
            None => {
                entries.push(GpuExecutionEntry {
                    label: format!("gpu_host_minimum_{}", scenario_id.as_str()),
                    scenario_id: scenario_id.as_str().to_string(),
                    width: inputs.width(),
                    height: inputs.height(),
                    frame_index,
                    gpu_path_available: true,
                    actual_gpu_timing_measured: false,
                    adapter_name: None,
                    backend: None,
                    total_ms: None,
                    dispatch_ms: None,
                    readback_ms: None,
                    mean_abs_trust_delta_vs_cpu: None,
                    mean_abs_alpha_delta_vs_cpu: None,
                    mean_abs_intervention_delta_vs_cpu: None,
                    workgroup_size: [8, 8, 1],
                    notes: vec![
                        "The wgpu compute path is compiled into the crate, but no usable GPU adapter was available in the current environment.".to_string(),
                        "Run `cargo run --release -- run-gpu-path --output <dir>` on a GPU host to measure this kernel without changing code.".to_string(),
                    ],
                });
            }
        }
    }

    Ok(GpuExecutionMetrics {
        measurement_kind: if any_measured {
            "actual_gpu_timing_measured".to_string()
        } else {
            "gpu_path_implemented_but_not_measured_in_current_environment".to_string()
        },
        actual_gpu_timing_measured: any_measured,
        shader_language: "wgsl".to_string(),
        kernel_name: "dsfb_host_minimum".to_string(),
        entries,
        notes: vec![
            "This path is intended to remove the 'CPU-only timing proxy' blocker by providing a real GPU-executable kernel and an honest measured-vs-unmeasured disclosure.".to_string(),
            "The current kernel covers the minimum host-realistic supervisory path. Motion disagreement remains an optional extension and is not part of the minimum kernel.".to_string(),
        ],
    })
}

pub fn write_gpu_execution_report(path: &Path, metrics: &GpuExecutionMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# GPU Execution Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Measurement classification: `{}`.",
        metrics.measurement_kind
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Actual GPU timing measured: `{}`.",
        metrics.actual_gpu_timing_measured
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Kernel: `{}` in `{}`.",
        metrics.kernel_name, metrics.shader_language
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Label | Scenario | Resolution | Measured | Adapter | Total ms | Dispatch ms | Readback ms | Trust delta vs CPU |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |"
    );
    for entry in &metrics.entries {
        let _ = writeln!(
            markdown,
            "| {} | {} | {}x{} | {} | {} | {} | {} | {} | {} |",
            entry.label,
            entry.scenario_id,
            entry.width,
            entry.height,
            entry.actual_gpu_timing_measured,
            entry.adapter_name.as_deref().unwrap_or("unavailable"),
            format_f64(entry.total_ms),
            format_f64(entry.dispatch_ms),
            format_f64(entry.readback_ms),
            format_f32(entry.mean_abs_trust_delta_vs_cpu),
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## GPU Path Status");
    let _ = writeln!(markdown);
    for note in &metrics.notes {
        let _ = writeln!(markdown, "- {note}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## How To Run On A GPU Host");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "```bash\ncargo run --release -- run-gpu-path --output generated/gpu_path\n```"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not imply measured GPU performance when `Actual GPU timing measured` is `false`."
    );
    let _ = writeln!(
        markdown,
        "- It does not replace real engine-side GPU profiling or cache/bandwidth measurement."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    if metrics.actual_gpu_timing_measured {
        let _ = writeln!(
            markdown,
            "- The kernel is measured, but broader engine-integrated GPU profiling still remains."
        );
    } else {
        let _ = writeln!(
            markdown,
            "- A GPU-executable path now exists, but the current environment still lacks measured GPU execution."
        );
    }
    let _ = writeln!(
        markdown,
        "- Real engine captures and imported external buffers still need GPU-side evaluation."
    );

    fs::write(path, markdown)?;
    Ok(())
}

fn mean_abs_delta(a: &[f32], b: &[f32]) -> f32 {
    let count = a.len().min(b.len()).max(1);
    a.iter()
        .zip(b.iter())
        .map(|(left, right)| (left - right).abs())
        .sum::<f32>()
        / count as f32
}

fn format_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_f32(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.6}"))
        .unwrap_or_else(|| "n/a".to_string())
}
