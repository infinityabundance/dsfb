use serde::Serialize;

use crate::config::{ScenarioSelection, SimulationConfig};

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioDefinition {
    pub name: String,
    pub title: String,
    pub description: String,
    pub n_steps: usize,
    pub degraded_start: usize,
    pub degraded_end: usize,
    pub refinement_end: usize,
    pub delta: usize,
    pub prediction_horizon: usize,
    pub disturbance_amplitude: f64,
    pub disturbance_center: usize,
    pub disturbance_width: usize,
    pub l3_period: usize,
    pub l3_recovery_period: usize,
    pub l3_resume_start: usize,
    pub level_gains: [f64; 3],
    pub trust_ceilings: [f64; 3],
    pub trust_betas: [f64; 3],
    pub envelope_decay: [f64; 3],
    pub drift_scales: [f64; 3],
    pub measurement_bias_scales: [f64; 3],
    pub availability_penalties: [f64; 3],
    pub eta: [f64; 2],
    pub tube_eval_start: usize,
    pub tube_eval_end: usize,
    pub tube_stride: usize,
    pub resonance_threshold: f64,
}

impl ScenarioDefinition {
    pub fn degraded_interval(&self) -> (usize, usize) {
        (self.degraded_start, self.degraded_end)
    }

    pub fn degradation_factor(&self, step: usize) -> f64 {
        if step < self.degraded_start || step > self.degraded_end {
            return 0.0;
        }
        let width = (self.degraded_end - self.degraded_start).max(1) as f64;
        let phase = (step - self.degraded_start) as f64 / width;
        let taper = (std::f64::consts::PI * phase).sin().powi(2);
        0.45 + 0.55 * taper
    }

    pub fn disturbance_component(&self, step: usize) -> f64 {
        let distance = (step as isize - self.disturbance_center as isize).unsigned_abs() as f64;
        let sigma = self.disturbance_width.max(1) as f64;
        self.disturbance_amplitude * (-0.5 * (distance / sigma).powi(2)).exp()
    }

    pub fn truth_series(&self) -> Vec<f64> {
        (0..self.n_steps)
            .map(|step| {
                let u = step as f64;
                0.016 * u
                    + 0.85 * (0.026 * u).sin()
                    + 0.22 * (0.091 * u).cos()
                    + 0.12 * (0.007 * u * u / self.n_steps as f64).sin()
                    + self.disturbance_component(step)
            })
            .collect()
    }

    pub fn level_name(level: usize) -> &'static str {
        match level {
            1 => "local_continuous",
            2 => "aggregate_smoother",
            3 => "intermittent_high_trust",
            _ => "observer",
        }
    }

    pub fn l3_available(&self, step: usize) -> bool {
        if step >= self.degraded_start.saturating_sub(self.l3_period) && step < self.l3_resume_start
        {
            return false;
        }
        if step >= self.l3_resume_start {
            return step % self.l3_recovery_period == 0;
        }
        step % self.l3_period == 0
    }
}

pub fn scenario_suite(config: &SimulationConfig) -> Vec<ScenarioDefinition> {
    let n = config.n_steps.max(96);
    match config.scenario {
        ScenarioSelection::All => vec![
            disturbance_recovery(n, config),
            forward_prediction(n, config),
            hierarchy_consistency(n, config),
        ],
        ScenarioSelection::DisturbanceRecovery => vec![disturbance_recovery(n, config)],
        ScenarioSelection::ForwardPrediction => vec![forward_prediction(n, config)],
        ScenarioSelection::HierarchyConsistency => vec![hierarchy_consistency(n, config)],
        ScenarioSelection::AerospaceNavigation => vec![aerospace_navigation(n, config)],
        ScenarioSelection::RoboticsSensorOcclusion => vec![robotics_sensor_occlusion(n, config)],
        ScenarioSelection::IndustrialFaultRefinement => {
            vec![industrial_fault_refinement(n, config)]
        }
        ScenarioSelection::NeuralMultimodalDelay => vec![neural_multimodal_delay(n, config)],
    }
}

fn disturbance_recovery(n: usize, config: &SimulationConfig) -> ScenarioDefinition {
    base_definition(
        "disturbance_recovery",
        "Disturbance and Recovery",
        "Retroactive refinement after a degraded sensing interval with delayed high-trust recovery.",
        n,
        config,
        0.28,
        0.44,
        0.58,
        0.38,
        0.06,
        0.82,
        18,
        4,
        [0.38, 0.52, 0.68],
        [0.88, 0.95, 0.995],
        [1.8, 1.5, 1.25],
        [0.80, 0.84, 0.88],
        [0.14, 0.08, 0.05],
        [0.10, 0.05, 0.015],
        [0.0, 0.0, 0.015],
        [0.062, 0.078],
        0.24,
        0.76,
        12,
        0.24,
    )
}

fn forward_prediction(n: usize, config: &SimulationConfig) -> ScenarioDefinition {
    base_definition(
        "forward_prediction",
        "Forward Prediction Tubes",
        "Prediction tube construction over a bounded horizon before and after refinement.",
        n,
        config,
        0.20,
        0.31,
        0.43,
        0.29,
        0.05,
        0.56,
        14,
        3,
        [0.41, 0.56, 0.72],
        [0.87, 0.95, 0.995],
        [1.9, 1.55, 1.2],
        [0.79, 0.85, 0.90],
        [0.13, 0.08, 0.045],
        [0.085, 0.045, 0.012],
        [0.0, 0.0, 0.012],
        [0.058, 0.074],
        0.22,
        0.70,
        10,
        0.22,
    )
}

fn hierarchy_consistency(n: usize, config: &SimulationConfig) -> ScenarioDefinition {
    base_definition(
        "hierarchy_consistency",
        "Multi-Level Hierarchy",
        "Three-level observer hierarchy demonstrating trust-gated propagation and bounded recursion depth.",
        n,
        config,
        0.18,
        0.30,
        0.46,
        0.27,
        0.045,
        0.64,
        16,
        4,
        [0.36, 0.50, 0.70],
        [0.86, 0.95, 0.995],
        [1.95, 1.6, 1.15],
        [0.81, 0.86, 0.91],
        [0.12, 0.07, 0.04],
        [0.08, 0.04, 0.010],
        [0.0, 0.0, 0.010],
        [0.055, 0.070],
        0.18,
        0.64,
        10,
        0.20,
    )
}

fn aerospace_navigation(n: usize, config: &SimulationConfig) -> ScenarioDefinition {
    let mut def = disturbance_recovery(n, config);
    def.name = "aerospace_navigation".to_string();
    def.title = "Aerospace Navigation Preset".to_string();
    def.description =
        "Longer blackout-style degradation with sparse high-trust recovery updates.".to_string();
    def.disturbance_amplitude = 0.94;
    def.l3_period = 22;
    def.l3_recovery_period = 5;
    def
}

fn robotics_sensor_occlusion(n: usize, config: &SimulationConfig) -> ScenarioDefinition {
    let mut def = disturbance_recovery(n, config);
    def.name = "robotics_sensor_occlusion".to_string();
    def.title = "Robotics Sensor Occlusion Preset".to_string();
    def.description = "Shorter occlusion interval with rapid high-trust relocking.".to_string();
    def.disturbance_amplitude = 0.52;
    def.l3_period = 10;
    def.l3_recovery_period = 2;
    def
}

fn industrial_fault_refinement(n: usize, config: &SimulationConfig) -> ScenarioDefinition {
    let mut def = hierarchy_consistency(n, config);
    def.name = "industrial_fault_refinement".to_string();
    def.title = "Industrial Fault Refinement Preset".to_string();
    def.description =
        "Fault-driven residual growth followed by bounded corrective recursion.".to_string();
    def.disturbance_amplitude = 0.72;
    def
}

fn neural_multimodal_delay(n: usize, config: &SimulationConfig) -> ScenarioDefinition {
    let mut def = forward_prediction(n, config);
    def.name = "neural_multimodal_delay".to_string();
    def.title = "Neural Multimodal Delay Preset".to_string();
    def.description =
        "Delayed high-trust modality arrival with bounded forward prediction tightening."
            .to_string();
    def.l3_period = 24;
    def.l3_recovery_period = 6;
    def.disturbance_amplitude = 0.48;
    def
}

#[allow(clippy::too_many_arguments)]
fn base_definition(
    name: &str,
    title: &str,
    description: &str,
    n: usize,
    config: &SimulationConfig,
    degraded_start_ratio: f64,
    degraded_end_ratio: f64,
    refinement_end_ratio: f64,
    disturbance_center_ratio: f64,
    disturbance_width_ratio: f64,
    disturbance_amplitude: f64,
    l3_period: usize,
    l3_recovery_period: usize,
    level_gains: [f64; 3],
    trust_ceilings: [f64; 3],
    trust_betas: [f64; 3],
    envelope_decay: [f64; 3],
    drift_scales: [f64; 3],
    measurement_bias_scales: [f64; 3],
    availability_penalties: [f64; 3],
    eta: [f64; 2],
    tube_eval_start_ratio: f64,
    tube_eval_end_ratio: f64,
    tube_stride: usize,
    resonance_threshold: f64,
) -> ScenarioDefinition {
    let degraded_start = (n as f64 * degraded_start_ratio).round() as usize;
    let degraded_end = (n as f64 * degraded_end_ratio).round() as usize;
    let refinement_end = (n as f64 * refinement_end_ratio).round() as usize;
    let disturbance_center = (n as f64 * disturbance_center_ratio).round() as usize;
    let disturbance_width = (n as f64 * disturbance_width_ratio).round() as usize;
    let tube_eval_start = (n as f64 * tube_eval_start_ratio).round() as usize;
    let tube_eval_end = (n as f64 * tube_eval_end_ratio).round() as usize;
    let delta = config.delta.min((n / 8).max(10));
    let prediction_horizon = config.prediction_horizon.min((n / 6).max(8));

    ScenarioDefinition {
        name: name.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        n_steps: n,
        degraded_start,
        degraded_end,
        refinement_end,
        delta,
        prediction_horizon,
        disturbance_amplitude,
        disturbance_center,
        disturbance_width,
        l3_period,
        l3_recovery_period,
        l3_resume_start: degraded_end.saturating_add(1),
        level_gains,
        trust_ceilings,
        trust_betas,
        envelope_decay,
        drift_scales,
        measurement_bias_scales,
        availability_penalties,
        eta,
        tube_eval_start,
        tube_eval_end,
        tube_stride,
        resonance_threshold,
    }
}
