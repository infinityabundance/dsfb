use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::engine::settings::EvaluationSettings;
use crate::engine::types::{
    DetectabilityBoundInputs, EnvelopeMode, GroupDefinition, ObservedTrajectory,
    PredictedTrajectory, ScenarioRecord, VectorSample,
};
use crate::math::envelope::EnvelopeSpec;

/// Deterministic synthetic sweep families supported by the crate.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SweepFamily {
    GradualDriftSlope,
    CurvatureOnsetTiming,
    SpikeMagnitudeDuration,
    OscillationAmplitudeFrequency,
    CoordinatedRiseStrength,
    EnvelopeTightness,
}

impl SweepFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GradualDriftSlope => "gradual_drift_slope",
            Self::CurvatureOnsetTiming => "curvature_onset_timing",
            Self::SpikeMagnitudeDuration => "spike_magnitude_duration",
            Self::OscillationAmplitudeFrequency => "oscillation_amplitude_frequency",
            Self::CoordinatedRiseStrength => "coordinated_rise_strength",
            Self::EnvelopeTightness => "envelope_tightness",
        }
    }
}

/// Deterministic sweep request used by the CLI and library.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweepConfig {
    pub family: SweepFamily,
    pub points: usize,
}

/// Sweep member definition prepared for the core engine pipeline.
#[derive(Clone, Debug)]
pub struct SweepMemberDefinition {
    pub family: SweepFamily,
    pub parameter_name: String,
    pub parameter_value: f64,
    pub secondary_parameter_name: Option<String>,
    pub secondary_parameter_value: Option<f64>,
    pub record: ScenarioRecord,
    pub observed: ObservedTrajectory,
    pub predicted: PredictedTrajectory,
    pub envelope_spec: EnvelopeSpec,
    pub detectability_inputs: Option<DetectabilityBoundInputs>,
    pub groups: Vec<GroupDefinition>,
    pub aggregate_envelope_spec: Option<EnvelopeSpec>,
}

impl SweepConfig {
    pub fn normalized(&self, settings: &EvaluationSettings) -> Self {
        Self {
            family: self.family,
            points: if self.points == 0 {
                settings.default_sweep_points
            } else {
                self.points
            }
            .min(12),
        }
    }
}

pub fn generate_sweep_members(
    config: &SweepConfig,
    steps: usize,
    dt: f64,
) -> Result<Vec<SweepMemberDefinition>> {
    let config = if config.points == 0 {
        SweepConfig {
            family: config.family,
            points: 5,
        }
    } else {
        config.clone()
    };
    Ok(match config.family {
        SweepFamily::GradualDriftSlope => {
            let values = linspace(0.0008, 0.0032, config.points);
            values
                .into_iter()
                .enumerate()
                .map(|(index, slope)| {
                    single_parameter_member(
                        config.family,
                        "drift_slope",
                        slope,
                        steps,
                        dt,
                        &format!("sweep_gradual_drift_{index:02}"),
                        "Gradual Drift Slope Sweep",
                        3,
                        EnvelopeSpec {
                            name: format!("sweep_gradual_drift_{index:02}_envelope"),
                            mode: EnvelopeMode::Widening,
                            base_radius: 0.22,
                            slope: 0.0006,
                            switch_step: None,
                            secondary_slope: None,
                            secondary_base: None,
                        },
                        |step, channel| {
                            let t = step as f64;
                            match channel {
                                0 => 0.04 + slope * t + 0.015 * (0.05 * t).sin(),
                                1 => 0.03 + 0.72 * slope * t + 0.010 * (0.04 * t + 0.6).sin(),
                                _ => 0.02 + 0.45 * slope * t + 0.008 * (0.06 * t + 1.2).cos(),
                            }
                        },
                    )
                })
                .collect()
        }
        SweepFamily::CurvatureOnsetTiming => {
            let values = linspace(40.0, 160.0, config.points);
            values
                .into_iter()
                .enumerate()
                .map(|(index, onset)| {
                    single_parameter_member(
                        config.family,
                        "onset_step",
                        onset,
                        steps,
                        dt,
                        &format!("sweep_curvature_onset_{index:02}"),
                        "Curvature Onset Timing Sweep",
                        3,
                        EnvelopeSpec {
                            name: format!("sweep_curvature_onset_{index:02}_envelope"),
                            mode: EnvelopeMode::Fixed,
                            base_radius: 0.56,
                            slope: 0.0,
                            switch_step: None,
                            secondary_slope: None,
                            secondary_base: None,
                        },
                        |step, channel| {
                            let t = step as f64;
                            let onset_term = (t - onset).max(0.0);
                            let base = 0.02 + 0.0006 * t + 0.00004 * onset_term * onset_term;
                            match channel {
                                0 => base,
                                1 => 0.5 * base + 0.01 * (0.07 * t).sin(),
                                _ => 0.3 * base + 0.012 * (0.04 * t + 0.8).cos(),
                            }
                        },
                    )
                })
                .collect()
        }
        SweepFamily::SpikeMagnitudeDuration => {
            let magnitudes = linspace(0.18, 0.60, config.points);
            magnitudes
                .into_iter()
                .enumerate()
                .map(|(index, magnitude)| {
                    let duration = 4.0 + index as f64;
                    dual_parameter_member(
                        config.family,
                        "spike_magnitude",
                        magnitude,
                        "spike_duration",
                        duration,
                        steps,
                        dt,
                        &format!("sweep_spike_{index:02}"),
                        "Spike Magnitude/Duration Sweep",
                        3,
                        EnvelopeSpec {
                            name: format!("sweep_spike_{index:02}_envelope"),
                            mode: EnvelopeMode::Fixed,
                            base_radius: 0.68,
                            slope: 0.0,
                            switch_step: None,
                            secondary_slope: None,
                            secondary_base: None,
                        },
                        move |step, channel| {
                            let t = step as f64;
                            let pulse = magnitude * (-((t - 120.0) / duration).powi(2)).exp();
                            match channel {
                                0 => 0.03 * (0.03 * t).sin() + pulse,
                                1 => 0.02 * (0.04 * t + 0.7).cos() + 0.45 * pulse,
                                _ => 0.02 * (0.05 * t + 0.2).sin() - 0.18 * pulse,
                            }
                        },
                    )
                })
                .collect()
        }
        SweepFamily::OscillationAmplitudeFrequency => {
            let amplitudes = linspace(0.08, 0.20, config.points);
            amplitudes
                .into_iter()
                .enumerate()
                .map(|(index, amplitude)| {
                    let frequency = 0.10 + 0.02 * index as f64;
                    dual_parameter_member(
                        config.family,
                        "oscillation_amplitude",
                        amplitude,
                        "oscillation_frequency",
                        frequency,
                        steps,
                        dt,
                        &format!("sweep_oscillation_{index:02}"),
                        "Oscillation Amplitude/Frequency Sweep",
                        3,
                        EnvelopeSpec {
                            name: format!("sweep_oscillation_{index:02}_envelope"),
                            mode: EnvelopeMode::Fixed,
                            base_radius: 0.42,
                            slope: 0.0,
                            switch_step: None,
                            secondary_slope: None,
                            secondary_base: None,
                        },
                        move |step, channel| {
                            let t = step as f64;
                            match channel {
                                0 => amplitude * (frequency * t).sin(),
                                1 => (amplitude * 0.88) * ((frequency * 1.18) * t + 0.3).sin(),
                                _ => (amplitude * 0.75) * ((frequency * 0.92) * t + 1.1).cos(),
                            }
                        },
                    )
                })
                .collect()
        }
        SweepFamily::CoordinatedRiseStrength => {
            let values = linspace(0.0007, 0.0024, config.points);
            values
                .into_iter()
                .enumerate()
                .map(|(index, slope)| coordinated_member(config.family, slope, steps, dt, index))
                .collect()
        }
        SweepFamily::EnvelopeTightness => {
            let values = linspace(0.22, 0.44, config.points);
            values
                .into_iter()
                .enumerate()
                .map(|(index, envelope_base)| {
                    single_parameter_member(
                        config.family,
                        "envelope_base",
                        envelope_base,
                        steps,
                        dt,
                        &format!("sweep_envelope_tightness_{index:02}"),
                        "Envelope Tightness Sweep",
                        3,
                        EnvelopeSpec {
                            name: format!("sweep_envelope_tightness_{index:02}_envelope"),
                            mode: EnvelopeMode::Fixed,
                            base_radius: envelope_base,
                            slope: 0.0,
                            switch_step: None,
                            secondary_slope: None,
                            secondary_base: None,
                        },
                        |step, channel| {
                            let t = step as f64;
                            match channel {
                                0 => 0.16 + 0.0024 * t,
                                1 => 0.03 + 0.012 * (0.04 * t + 0.4).sin(),
                                _ => 0.02 + 0.010 * (0.05 * t + 0.8).cos(),
                            }
                        },
                    )
                })
                .collect()
        }
    })
}

fn coordinated_member(
    family: SweepFamily,
    slope: f64,
    steps: usize,
    dt: f64,
    index: usize,
) -> SweepMemberDefinition {
    let base = 0.03;
    let channels = channel_names(4);
    let scenario_id = format!("sweep_coordinated_rise_{index:02}");
    let (observed, predicted) = build_trajectories(
        &scenario_id,
        steps,
        dt,
        &channels,
        |time, channel| {
            0.6 * (0.035 * time + 0.45 * channel as f64).sin()
                + 0.22 * (0.011 * time + 0.3 * channel as f64).cos()
        },
        |step, channel| {
            let t = step as f64;
            let coordinated = base + slope * t + 0.012 * (0.05 * t).sin();
            match channel {
                0 => coordinated,
                1 => coordinated * 0.92 + 0.008 * (0.07 * t + 0.4).sin(),
                2 => coordinated * 0.85 + 0.009 * (0.05 * t + 0.8).cos(),
                _ => 0.05 + 0.0004 * t + 0.014 * (0.09 * t + 0.2).sin(),
            }
        },
    );
    SweepMemberDefinition {
        family,
        parameter_name: "group_rise_slope".to_string(),
        parameter_value: slope,
        secondary_parameter_name: None,
        secondary_parameter_value: None,
        record: ScenarioRecord {
            id: scenario_id,
            title: "Coordinated Rise Strength Sweep".to_string(),
            data_origin: "synthetic-sweep".to_string(),
            purpose: "Deterministically vary coordinated group rise strength to inspect syntax, semantics, and comparator stability.".to_string(),
            theorem_alignment: "Synthetic sweep member used for calibration-style inspection rather than theorem proof.".to_string(),
            claim_class: "synthetic sweep".to_string(),
            limitations: "This sweep member is a deterministic synthetic construction for internal calibration-style comparison only.".to_string(),
            sweep_family: Some(family.as_str().to_string()),
            sweep_parameter_name: Some("group_rise_slope".to_string()),
            sweep_parameter_value: Some(slope),
            sweep_secondary_parameter_name: None,
            sweep_secondary_parameter_value: None,
        },
        observed,
        predicted,
        envelope_spec: EnvelopeSpec {
            name: format!("sweep_coordinated_rise_{index:02}_envelope"),
            mode: EnvelopeMode::Widening,
            base_radius: 0.28,
            slope: 0.0005,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: vec![
            GroupDefinition {
                group_id: "front_cluster".to_string(),
                member_indices: vec![0, 1, 2],
            },
            GroupDefinition {
                group_id: "tail_channel".to_string(),
                member_indices: vec![3],
            },
        ],
        aggregate_envelope_spec: Some(EnvelopeSpec {
            name: format!("sweep_coordinated_rise_{index:02}_aggregate"),
            mode: EnvelopeMode::Aggregate,
            base_radius: 0.22,
            slope: 0.0003,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn single_parameter_member<F>(
    family: SweepFamily,
    parameter_name: &str,
    parameter_value: f64,
    steps: usize,
    dt: f64,
    scenario_id: &str,
    title: &str,
    channels: usize,
    envelope_spec: EnvelopeSpec,
    residual_signal: F,
) -> SweepMemberDefinition
where
    F: Fn(usize, usize) -> f64,
{
    let channel_names = channel_names(channels);
    let (observed, predicted) = build_trajectories(
        scenario_id,
        steps,
        dt,
        &channel_names,
        prediction_signal,
        residual_signal,
    );
    SweepMemberDefinition {
        family,
        parameter_name: parameter_name.to_string(),
        parameter_value,
        secondary_parameter_name: None,
        secondary_parameter_value: None,
        record: ScenarioRecord {
            id: scenario_id.to_string(),
            title: title.to_string(),
            data_origin: "synthetic-sweep".to_string(),
            purpose: format!(
                "Deterministically sweep {} for internal calibration-style evaluation.",
                parameter_name
            ),
            theorem_alignment: "Synthetic sweep member used to inspect stability and failure boundaries of the deterministic pipeline.".to_string(),
            claim_class: "synthetic sweep".to_string(),
            limitations: "This sweep member is a deterministic synthetic construction for internal comparator and stability analysis only.".to_string(),
            sweep_family: Some(family.as_str().to_string()),
            sweep_parameter_name: Some(parameter_name.to_string()),
            sweep_parameter_value: Some(parameter_value),
            sweep_secondary_parameter_name: None,
            sweep_secondary_parameter_value: None,
        },
        observed,
        predicted,
        envelope_spec,
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn dual_parameter_member<F>(
    family: SweepFamily,
    parameter_name: &str,
    parameter_value: f64,
    secondary_parameter_name: &str,
    secondary_parameter_value: f64,
    steps: usize,
    dt: f64,
    scenario_id: &str,
    title: &str,
    channels: usize,
    envelope_spec: EnvelopeSpec,
    residual_signal: F,
) -> SweepMemberDefinition
where
    F: Fn(usize, usize) -> f64,
{
    let channel_names = channel_names(channels);
    let (observed, predicted) = build_trajectories(
        scenario_id,
        steps,
        dt,
        &channel_names,
        prediction_signal,
        residual_signal,
    );
    SweepMemberDefinition {
        family,
        parameter_name: parameter_name.to_string(),
        parameter_value,
        secondary_parameter_name: Some(secondary_parameter_name.to_string()),
        secondary_parameter_value: Some(secondary_parameter_value),
        record: ScenarioRecord {
            id: scenario_id.to_string(),
            title: title.to_string(),
            data_origin: "synthetic-sweep".to_string(),
            purpose: format!(
                "Deterministically sweep {} and {} for internal calibration-style evaluation.",
                parameter_name, secondary_parameter_name
            ),
            theorem_alignment: "Synthetic sweep member used to inspect stability and conservative semantic transitions.".to_string(),
            claim_class: "synthetic sweep".to_string(),
            limitations: "This sweep member is a deterministic synthetic construction for internal comparator and stability analysis only.".to_string(),
            sweep_family: Some(family.as_str().to_string()),
            sweep_parameter_name: Some(parameter_name.to_string()),
            sweep_parameter_value: Some(parameter_value),
            sweep_secondary_parameter_name: Some(secondary_parameter_name.to_string()),
            sweep_secondary_parameter_value: Some(secondary_parameter_value),
        },
        observed,
        predicted,
        envelope_spec,
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn build_trajectories<F, G>(
    scenario_id: &str,
    steps: usize,
    dt: f64,
    channel_names: &[String],
    prediction_fn: F,
    residual_fn: G,
) -> (ObservedTrajectory, PredictedTrajectory)
where
    F: Fn(f64, usize) -> f64,
    G: Fn(usize, usize) -> f64,
{
    let samples = (0..steps)
        .map(|step| {
            let time = step as f64 * dt;
            let predicted = (0..channel_names.len())
                .map(|channel| prediction_fn(time, channel))
                .collect::<Vec<_>>();
            let residual = (0..channel_names.len())
                .map(|channel| residual_fn(step, channel))
                .collect::<Vec<_>>();
            let observed = predicted
                .iter()
                .zip(&residual)
                .map(|(base, residual)| base + residual)
                .collect::<Vec<_>>();
            (
                VectorSample {
                    step,
                    time,
                    values: observed,
                },
                VectorSample {
                    step,
                    time,
                    values: predicted,
                },
            )
        })
        .collect::<Vec<_>>();
    (
        ObservedTrajectory {
            scenario_id: scenario_id.to_string(),
            channel_names: channel_names.to_vec(),
            samples: samples
                .iter()
                .map(|(observed, _)| observed.clone())
                .collect(),
        },
        PredictedTrajectory {
            scenario_id: scenario_id.to_string(),
            channel_names: channel_names.to_vec(),
            samples: samples
                .iter()
                .map(|(_, predicted)| predicted.clone())
                .collect(),
        },
    )
}

fn prediction_signal(time: f64, channel: usize) -> f64 {
    0.6 * (0.035 * time + 0.45 * channel as f64).sin()
        + 0.22 * (0.011 * time + 0.3 * channel as f64).cos()
}

fn channel_names(count: usize) -> Vec<String> {
    (1..=count)
        .map(|index| format!("channel_{index}"))
        .collect()
}

fn linspace(start: f64, end: f64, count: usize) -> Vec<f64> {
    if count <= 1 {
        return vec![start];
    }
    let step = (end - start) / (count - 1) as f64;
    (0..count)
        .map(|index| start + step * index as f64)
        .collect()
}
