//! Deterministic domain-agnostic benchmark generation for latent structural drift.
//!
//! References: `CORE-08` for generic anomaly soundness, `CORE-10` for
//! deterministic compositional replay, `DSFB-06` for trace reproducibility, and
//! `TMTR-01` for trust-monotone structural concern under progressive mismatch.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::input::{TraceDocument, TraceStep, TruthState};

/// Built-in benchmark scenarios for the forensic layer.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum BenchmarkScenario {
    /// Preserve the legacy replay-only path.
    None,
    /// Healthy coherent reference channels with no latent degradation.
    HealthyReference,
    /// Progressive latent signature drift that remains within simple QA limits for an interval.
    LatentSignatureDrift,
    /// A stronger degradation ramp that fragments quickly and breaches simple QA earlier.
    ChannelFragmentationRamp,
}

impl BenchmarkScenario {
    /// References: `CORE-10` and `DSFB-06`.
    pub fn enabled(self) -> bool {
        !matches!(self, Self::None)
    }

    /// References: `CORE-10` and `DSFB-06`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::HealthyReference => "healthy-reference",
            Self::LatentSignatureDrift => "latent-signature-drift",
            Self::ChannelFragmentationRamp => "channel-fragmentation-ramp",
        }
    }
}

/// Toggle for whether benchmark traces should be written into the run directory.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum BenchmarkWriteTrace {
    /// Write the generated benchmark trace artifact.
    On,
    /// Skip writing the generated benchmark trace artifact.
    Off,
}

impl BenchmarkWriteTrace {
    /// References: `CORE-10` and `DSFB-06`.
    pub fn enabled(self) -> bool {
        matches!(self, Self::On)
    }
}

/// Benchmark configuration for the synthetic latent degradation scenarios.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BenchmarkConfig {
    /// Selected built-in scenario.
    pub scenario: BenchmarkScenario,
    /// Number of simulation steps.
    pub step_count: usize,
    /// Time delta in seconds.
    pub dt: f64,
    /// Total channel count.
    pub channel_count: usize,
    /// Step where latent degradation begins.
    pub drift_start_step: usize,
    /// Linear drift ramp rate applied after degradation begins.
    pub drift_ramp_rate: f64,
    /// Maximum degradation amplitude before saturation.
    pub drift_amplitude_ceiling: f64,
    /// Simple scalar QA threshold used for conventional comparator metrics.
    pub conventional_qa_threshold: f64,
    /// Deterministic pseudo-jitter magnitude for all channels.
    pub jitter_level: f64,
    /// Indices of channels that undergo degradation.
    pub anomaly_channels: Vec<usize>,
    /// Optional recovery step that creates asymmetric snapback behavior.
    pub recovery_step: Option<usize>,
    /// Consecutive-step threshold for the sustained DSFB alert condition.
    pub alert_consecutive_steps: usize,
    /// Whether the generated trace should be written into the run directory.
    pub write_trace: BenchmarkWriteTrace,
}

/// Compact benchmark metadata stored with the audit outputs.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BenchmarkMetadata {
    /// Scenario identifier.
    pub scenario: String,
    /// Drift start step.
    pub drift_start_step: usize,
    /// Degraded channel indices.
    pub anomaly_channels: Vec<usize>,
    /// Conventional scalar QA threshold.
    pub conventional_qa_threshold: f64,
    /// Sustained alert horizon.
    pub alert_consecutive_steps: usize,
    /// Optional recovery step.
    pub recovery_step: Option<usize>,
}

impl BenchmarkConfig {
    /// Validate and normalize the benchmark configuration.
    ///
    /// References: `DSFB-06` and `CORE-10`.
    pub fn validate(&self) -> Result<()> {
        if self.step_count == 0 {
            bail!("benchmark step count must be positive");
        }
        if self.channel_count == 0 {
            bail!("benchmark channel count must be positive");
        }
        if !(self.dt.is_finite() && self.dt > 0.0) {
            bail!("benchmark dt must be positive and finite");
        }
        if !(self.drift_ramp_rate.is_finite() && self.drift_ramp_rate >= 0.0) {
            bail!("benchmark drift ramp rate must be finite and non-negative");
        }
        if !(self.drift_amplitude_ceiling.is_finite() && self.drift_amplitude_ceiling >= 0.0) {
            bail!("benchmark drift amplitude ceiling must be finite and non-negative");
        }
        if !(self.conventional_qa_threshold.is_finite() && self.conventional_qa_threshold > 0.0) {
            bail!("benchmark QA threshold must be positive and finite");
        }
        if !(self.jitter_level.is_finite() && self.jitter_level >= 0.0) {
            bail!("benchmark jitter level must be finite and non-negative");
        }
        if self.alert_consecutive_steps == 0 {
            bail!("benchmark alert consecutive steps must be positive");
        }
        if self.drift_start_step >= self.step_count {
            bail!(
                "benchmark drift start step {} must be smaller than step count {}",
                self.drift_start_step,
                self.step_count
            );
        }
        if let Some(recovery_step) = self.recovery_step {
            if recovery_step >= self.step_count {
                bail!(
                    "benchmark recovery step {} must be smaller than step count {}",
                    recovery_step,
                    self.step_count
                );
            }
        }
        if self.anomaly_channels.iter().any(|&index| index >= self.channel_count) {
            bail!("benchmark anomaly channel index out of bounds for channel count {}", self.channel_count);
        }
        Ok(())
    }

    /// Produce a stable metadata view for output manifests and summaries.
    ///
    /// References: `CORE-10` and `DSFB-06`.
    pub fn metadata(&self) -> BenchmarkMetadata {
        BenchmarkMetadata {
            scenario: self.scenario.as_str().to_string(),
            drift_start_step: self.drift_start_step,
            anomaly_channels: self.normalized_anomaly_channels(),
            conventional_qa_threshold: self.conventional_qa_threshold,
            alert_consecutive_steps: self.alert_consecutive_steps,
            recovery_step: self.recovery_step,
        }
    }

    /// Return sorted, deduplicated anomaly channels with scenario-aware defaults.
    ///
    /// References: `CORE-10` and `DSFB-06`.
    pub fn normalized_anomaly_channels(&self) -> Vec<usize> {
        if matches!(self.scenario, BenchmarkScenario::HealthyReference | BenchmarkScenario::None) {
            return Vec::new();
        }
        let mut channels = BTreeSet::new();
        if self.anomaly_channels.is_empty() {
            channels.insert(self.channel_count.saturating_sub(1));
        } else {
            for &index in &self.anomaly_channels {
                channels.insert(index);
            }
        }
        channels.into_iter().collect()
    }
}

/// Generate a deterministic trace for the selected benchmark scenario.
///
/// References: `DSFB-06`, `CORE-10`, and `TMTR-01`.
pub fn generate_trace(config: &BenchmarkConfig) -> Result<TraceDocument> {
    config.validate()?;
    let anomaly_channels = config.normalized_anomaly_channels();
    let mut steps = Vec::with_capacity(config.step_count);
    let mut phi = 0.0;
    let mut omega = 0.42;
    let mut previous_omega = omega;
    let mut anomaly_memory = vec![0.0; config.channel_count];

    for step_index in 0..config.step_count {
        let u = step_index as f64;
        let latent_drive = 0.028 * (0.17 * u).sin() + 0.012 * (0.05 * u).cos();
        omega += latent_drive * config.dt;
        phi += omega * config.dt;
        let alpha = (omega - previous_omega) / config.dt;
        previous_omega = omega;

        let truth = TruthState { phi, omega, alpha };
        let mut measurements = Vec::with_capacity(config.channel_count);
        for channel_index in 0..config.channel_count {
            let channel_phase = channel_index as f64 * 0.73;
            let healthy_shape = config.jitter_level
                * (0.62 * (0.31 * u + channel_phase).sin()
                    + 0.38 * (0.19 * u + 0.5 * channel_phase).cos());
            let coherent_bias = config.jitter_level * 0.22 * (0.11 * u).sin();
            let mut measurement = phi + healthy_shape + coherent_bias;

            if anomaly_channels.contains(&channel_index) {
                let delta =
                    degradation_delta(config, step_index, channel_index, anomaly_memory[channel_index]);
                anomaly_memory[channel_index] = delta;
                measurement += delta;
            }

            measurements.push(measurement);
        }

        steps.push(TraceStep {
            step: step_index,
            dt: config.dt,
            measurements,
            truth: Some(truth),
        });
    }

    Ok(TraceDocument {
        channel_names: (0..config.channel_count)
            .map(|index| format!("measurement_{index}"))
            .collect(),
        steps,
    })
}

/// Write a generated benchmark trace to CSV using the existing input schema.
///
/// References: `DSFB-06`, `DSFB-07`, and `CORE-10`.
pub fn write_trace_csv(path: &Path, trace: &TraceDocument) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    let mut header = vec![
        "step".to_string(),
        "dt".to_string(),
        "truth_phi".to_string(),
        "truth_omega".to_string(),
        "truth_alpha".to_string(),
    ];
    header.extend(trace.channel_names.iter().cloned());
    writer
        .write_record(&header)
        .with_context(|| format!("failed to write header to {}", path.display()))?;

    for step in &trace.steps {
        let mut row = Vec::with_capacity(5 + step.measurements.len());
        row.push(step.step.to_string());
        row.push(step.dt.to_string());
        if let Some(truth) = step.truth {
            row.push(truth.phi.to_string());
            row.push(truth.omega.to_string());
            row.push(truth.alpha.to_string());
        } else {
            row.push(String::new());
            row.push(String::new());
            row.push(String::new());
        }
        row.extend(step.measurements.iter().map(ToString::to_string));
        writer
            .write_record(&row)
            .with_context(|| format!("failed to write row to {}", path.display()))?;
    }
    writer
        .flush()
        .with_context(|| format!("failed to flush {}", path.display()))
}

fn degradation_delta(
    config: &BenchmarkConfig,
    step_index: usize,
    channel_index: usize,
    previous_delta: f64,
) -> f64 {
    if !config.scenario.enabled() || step_index < config.drift_start_step {
        return 0.0;
    }

    let progress = (step_index - config.drift_start_step + 1) as f64;
    let base_ramp = (progress * config.drift_ramp_rate).min(config.drift_amplitude_ceiling);
    let scenario_gain = match config.scenario {
        BenchmarkScenario::HealthyReference | BenchmarkScenario::None => 0.0,
        BenchmarkScenario::LatentSignatureDrift => 1.0,
        BenchmarkScenario::ChannelFragmentationRamp => 1.55,
    };
    let sign = if channel_index % 2 == 0 { 1.0 } else { -1.0 };
    let phase = channel_index as f64 * 0.61;
    let shape_mismatch = scenario_gain
        * base_ramp
        * (0.58 + 0.26 * (0.47 * progress + phase).sin() + 0.12 * (0.23 * progress).cos());
    let slew_mismatch = scenario_gain * base_ramp * 0.24 * (0.93 * progress + 0.4 * phase).sin();
    let lag_memory = 0.32 * previous_delta;
    let mut delta = sign * (shape_mismatch + slew_mismatch) + lag_memory;

    if let Some(recovery_step) = config.recovery_step {
        if step_index >= recovery_step {
            let recovery_progress = (step_index - recovery_step + 1) as f64;
            let recovery_term = scenario_gain
                * base_ramp
                * (0.52 - 0.18 * (0.51 * recovery_progress + phase).cos());
            delta -= sign * recovery_term;
        }
    }

    let clamp = (config.drift_amplitude_ceiling * 1.6).max(config.jitter_level * 2.0 + 0.05);
    delta.clamp(-clamp, clamp)
}
