#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use anyhow::Result;
use clap::{Parser, ValueHint};
use serde::Serialize;

use dsfb_semiotics_engine::engine::settings::{EngineSettings, SmoothingSettings};
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use dsfb_semiotics_engine::live::{numeric_mode_label, to_real, OnlineStructuralEngine};
use dsfb_semiotics_engine::math::envelope::EnvelopeSpec;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Generate a target-facing constrained-profile timing demo for the bounded live path"
)]
struct Args {
    #[arg(
        long,
        value_hint = ValueHint::FilePath,
        default_value = "docs/generated/target_facing_timing_demo.json"
    )]
    output_json: PathBuf,

    #[arg(long, default_value_t = 160)]
    iterations: usize,

    #[arg(long, default_value_t = 24)]
    warmup: usize,
}

#[derive(Clone, Debug, Serialize)]
struct TimingSummary {
    name: String,
    iterations: usize,
    mean_ns: u128,
    median_ns: u128,
    p95_ns: u128,
    p99_ns: u128,
    max_ns: u128,
    jitter_ns: u128,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct TargetFacingTimingDemo {
    schema_version: String,
    platform: String,
    rust_version: String,
    numeric_mode: String,
    profile_name: String,
    build_expectation: String,
    iterations: usize,
    warmup: usize,
    assumptions: Vec<String>,
    metrics: Vec<TimingSummary>,
    note: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let report = build_report(args.iterations, args.warmup)?;
    if let Some(parent) = args.output_json.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.output_json, serde_json::to_vec_pretty(&report)?)?;
    println!("target_facing_timing_demo={}", args.output_json.display());
    Ok(())
}

fn build_report(iterations: usize, warmup: usize) -> Result<TargetFacingTimingDemo> {
    Ok(TargetFacingTimingDemo {
        schema_version: "dsfb-semiotics-target-facing-timing-demo/v1".to_string(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        rust_version: rust_version(),
        numeric_mode: numeric_mode_label().to_string(),
        profile_name: "constrained_host_profile".to_string(),
        build_expectation: "run with cargo run --release for the documented transition demo numbers"
            .to_string(),
        iterations,
        warmup,
        assumptions: vec![
            "bounded live path only; no dashboard, PDF generation, or offline artifact work".to_string(),
            "release-profile measurement on the current host, used as a target-facing constrained-profile demonstration rather than a certified target measurement".to_string(),
            "history_buffer_capacity=16, offline_history_enabled=false, builtin bank only".to_string(),
            "safety_first smoothing enabled to reflect the conservative bounded-lag deployment profile".to_string(),
            "3-axis batch path models an IMU-style multi-axis advisory/monitor ingress surface".to_string(),
        ],
        metrics: vec![
            summarize(
                "single_axis_monitor_step",
                measure_single_axis_monitor_step(iterations, warmup)?,
                "One-channel bounded advisory step under the constrained profile.",
            ),
            summarize(
                "imu_like_batch_step",
                measure_imu_like_batch_step(iterations, warmup)?,
                "Four-sample, 3-axis batch ingestion under the constrained profile.",
            ),
            summarize(
                "stress_violation_batch_step",
                measure_stress_violation_batch_step(iterations, warmup)?,
                "3-axis batch ingestion with violation-driving residual amplitudes under the constrained profile.",
            ),
        ],
        note: "These are observed bounds from a target-facing constrained-profile demonstration. They are distinct from the broader host timing report and do not claim certified WCET or target qualification.".to_string(),
    })
}

fn constrained_settings() -> EngineSettings {
    let mut settings = EngineSettings::default();
    settings.online.history_buffer_capacity = 16;
    settings.online.offline_history_enabled = false;
    settings.smoothing = SmoothingSettings::safety_first();
    settings
}

fn fixed_envelope(base_radius: f64) -> EnvelopeSpec {
    EnvelopeSpec {
        name: "target_demo".to_string(),
        mode: EnvelopeMode::Fixed,
        base_radius,
        slope: 0.0,
        switch_step: None,
        secondary_slope: None,
        secondary_base: None,
    }
}

fn measure_single_axis_monitor_step(iterations: usize, warmup: usize) -> Result<Vec<u128>> {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "target_single_axis",
        vec!["residual".to_string()],
        0.02,
        fixed_envelope(0.8),
        constrained_settings(),
    )?;
    let mut samples = Vec::with_capacity(iterations);
    for step in 0..(warmup + iterations) {
        let time = step as f64 * 0.02;
        let started = Instant::now();
        let _ = engine.push_residual_sample(time, &[to_real(0.12 + step as f64 * 0.0015)])?;
        let elapsed = started.elapsed().as_nanos();
        if step >= warmup {
            samples.push(elapsed);
        }
    }
    Ok(samples)
}

fn measure_imu_like_batch_step(iterations: usize, warmup: usize) -> Result<Vec<u128>> {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "target_batch",
        vec![
            "imu_x".to_string(),
            "imu_y".to_string(),
            "imu_z".to_string(),
        ],
        0.01,
        fixed_envelope(0.9),
        constrained_settings(),
    )?;
    let mut samples = Vec::with_capacity(iterations);
    for batch in 0..(warmup + iterations) {
        let t0 = batch as f64 * 0.04;
        let times = [t0, t0 + 0.01, t0 + 0.02, t0 + 0.03];
        let base = batch as f64 * 0.0006;
        let values = [
            to_real(0.10 + base),
            to_real(0.02),
            to_real(-0.02),
            to_real(0.11 + base),
            to_real(0.025),
            to_real(-0.015),
            to_real(0.12 + base),
            to_real(0.03),
            to_real(-0.01),
            to_real(0.13 + base),
            to_real(0.032),
            to_real(-0.008),
        ];
        let started = Instant::now();
        let _ = engine.push_residual_sample_batch(&times, &values)?;
        let elapsed = started.elapsed().as_nanos();
        if batch >= warmup {
            samples.push(elapsed);
        }
    }
    Ok(samples)
}

fn measure_stress_violation_batch_step(iterations: usize, warmup: usize) -> Result<Vec<u128>> {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "target_stress",
        vec![
            "imu_x".to_string(),
            "imu_y".to_string(),
            "imu_z".to_string(),
        ],
        0.01,
        fixed_envelope(0.45),
        constrained_settings(),
    )?;
    let mut samples = Vec::with_capacity(iterations);
    for batch in 0..(warmup + iterations) {
        let t0 = batch as f64 * 0.03;
        let times = [t0, t0 + 0.01, t0 + 0.02];
        let base = 0.24 + batch as f64 * 0.0012;
        let values = [
            to_real(base),
            to_real(0.10),
            to_real(0.08),
            to_real(base + 0.05),
            to_real(0.12),
            to_real(0.09),
            to_real(base + 0.10),
            to_real(0.13),
            to_real(0.10),
        ];
        let started = Instant::now();
        let _ = engine.push_residual_sample_batch(&times, &values)?;
        let elapsed = started.elapsed().as_nanos();
        if batch >= warmup {
            samples.push(elapsed);
        }
    }
    Ok(samples)
}

fn summarize(name: &str, mut samples: Vec<u128>, note: &str) -> TimingSummary {
    samples.sort_unstable();
    let mean_ns = if samples.is_empty() {
        0
    } else {
        samples.iter().sum::<u128>() / samples.len() as u128
    };
    let median_ns = percentile(&samples, 0.50);
    let p95_ns = percentile(&samples, 0.95);
    let p99_ns = percentile(&samples, 0.99);
    let max_ns = samples.last().copied().unwrap_or_default();
    TimingSummary {
        name: name.to_string(),
        iterations: samples.len(),
        mean_ns,
        median_ns,
        p95_ns,
        p99_ns,
        max_ns,
        jitter_ns: max_ns.saturating_sub(median_ns),
        note: note.to_string(),
    }
}

fn percentile(sorted: &[u128], quantile: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() - 1) as f64 * quantile).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn rust_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "rustc version unavailable".to_string())
}
