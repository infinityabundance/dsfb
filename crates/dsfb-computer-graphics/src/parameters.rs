use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SmoothstepThreshold {
    pub low: f32,
    pub high: f32,
}

impl SmoothstepThreshold {
    pub const fn new(low: f32, high: f32) -> Self {
        Self { low, high }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AlphaRange {
    pub min: f32,
    pub max: f32,
}

impl AlphaRange {
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BaselineParameters {
    pub fixed_alpha: f32,
    pub residual_alpha_range: AlphaRange,
    pub residual_threshold: SmoothstepThreshold,
    pub clamp_distance: SmoothstepThreshold,
    pub depth_disagreement: SmoothstepThreshold,
    pub normal_disagreement: SmoothstepThreshold,
    pub neighborhood_distance: SmoothstepThreshold,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HazardWeights {
    pub residual: f32,
    pub visibility: f32,
    pub depth: f32,
    pub normal: f32,
    pub motion: f32,
    pub neighborhood: f32,
    pub thin: f32,
    pub history_instability: f32,
    pub grammar: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HostProxyThresholds {
    pub residual: SmoothstepThreshold,
    pub depth: SmoothstepThreshold,
    pub normal: SmoothstepThreshold,
    pub motion: SmoothstepThreshold,
    pub neighborhood: SmoothstepThreshold,
    pub local_contrast: SmoothstepThreshold,
    pub hazard_curve: SmoothstepThreshold,
    pub thin_hint_mix: f32,
    pub thin_local_contrast_mix: f32,
    pub history_instability_residual_mix: f32,
    pub history_instability_neighborhood_mix: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct StructuralThresholds {
    pub disocclusion_like: f32,
    pub unstable_residual: f32,
    pub unstable_neighborhood: f32,
    pub motion_edge: f32,
    pub thin_edge: f32,
    pub thin_residual: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HazardMergeMode {
    MaxGate,
    WeightedAdd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustBehavior {
    GateLike,
    Graded,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HostSupervisionParameters {
    pub alpha_range: AlphaRange,
    pub weights: HazardWeights,
    pub thresholds: HostProxyThresholds,
    pub structural: StructuralThresholds,
    pub hazard_merge_mode: HazardMergeMode,
    pub trust_behavior: TrustBehavior,
}

pub fn baseline_parameters() -> BaselineParameters {
    BaselineParameters {
        fixed_alpha: 0.12,
        residual_alpha_range: AlphaRange::new(0.12, 0.72),
        residual_threshold: SmoothstepThreshold::new(0.08, 0.18),
        clamp_distance: SmoothstepThreshold::new(0.008, 0.10),
        depth_disagreement: SmoothstepThreshold::new(0.01, 0.08),
        normal_disagreement: SmoothstepThreshold::new(0.01, 0.16),
        neighborhood_distance: SmoothstepThreshold::new(0.01, 0.14),
    }
}

pub fn host_realistic_parameters() -> HostSupervisionParameters {
    HostSupervisionParameters {
        alpha_range: AlphaRange::new(0.08, 0.96),
        weights: HazardWeights {
            residual: 0.24,
            visibility: 0.0,
            depth: 0.16,
            normal: 0.11,
            motion: 0.0,
            neighborhood: 0.16,
            thin: 0.09,
            history_instability: 0.12,
            grammar: 0.18,
        },
        thresholds: HostProxyThresholds {
            residual: SmoothstepThreshold::new(0.015, 0.22),
            depth: SmoothstepThreshold::new(0.01, 0.08),
            normal: SmoothstepThreshold::new(0.01, 0.16),
            motion: SmoothstepThreshold::new(0.35, 1.90),
            neighborhood: SmoothstepThreshold::new(0.01, 0.14),
            local_contrast: SmoothstepThreshold::new(0.02, 0.18),
            hazard_curve: SmoothstepThreshold::new(0.03, 0.86),
            thin_hint_mix: 0.45,
            thin_local_contrast_mix: 0.55,
            history_instability_residual_mix: 0.58,
            history_instability_neighborhood_mix: 0.42,
        },
        structural: StructuralThresholds {
            disocclusion_like: 0.68,
            unstable_residual: 0.34,
            unstable_neighborhood: 0.20,
            motion_edge: 0.34,
            thin_edge: 0.34,
            thin_residual: 0.14,
        },
        hazard_merge_mode: HazardMergeMode::WeightedAdd,
        trust_behavior: TrustBehavior::Graded,
    }
}

pub fn motion_augmented_parameters() -> HostSupervisionParameters {
    let mut parameters = host_realistic_parameters();
    parameters.weights.motion = 0.14;
    parameters.structural.motion_edge = 0.28;
    parameters
}

pub fn gated_reference_parameters() -> HostSupervisionParameters {
    HostSupervisionParameters {
        alpha_range: AlphaRange::new(0.08, 0.96),
        weights: HazardWeights {
            residual: 0.26,
            visibility: 0.0,
            depth: 0.18,
            normal: 0.12,
            motion: 0.12,
            neighborhood: 0.14,
            thin: 0.08,
            history_instability: 0.10,
            grammar: 1.0,
        },
        thresholds: HostProxyThresholds {
            residual: SmoothstepThreshold::new(0.015, 0.22),
            depth: SmoothstepThreshold::new(0.01, 0.08),
            normal: SmoothstepThreshold::new(0.01, 0.16),
            motion: SmoothstepThreshold::new(0.5, 3.0),
            neighborhood: SmoothstepThreshold::new(0.01, 0.14),
            local_contrast: SmoothstepThreshold::new(0.02, 0.18),
            hazard_curve: SmoothstepThreshold::new(0.0, 1.0),
            thin_hint_mix: 0.45,
            thin_local_contrast_mix: 0.55,
            history_instability_residual_mix: 0.62,
            history_instability_neighborhood_mix: 0.38,
        },
        structural: StructuralThresholds {
            disocclusion_like: 0.72,
            unstable_residual: 0.38,
            unstable_neighborhood: 0.22,
            motion_edge: 0.45,
            thin_edge: 0.40,
            thin_residual: 0.18,
        },
        hazard_merge_mode: HazardMergeMode::MaxGate,
        trust_behavior: TrustBehavior::GateLike,
    }
}

pub fn synthetic_visibility_parameters() -> HostSupervisionParameters {
    let mut parameters = motion_augmented_parameters();
    parameters.weights.visibility = 0.22;
    parameters.weights.residual = 0.18;
    parameters.weights.depth = 0.12;
    parameters.weights.normal = 0.08;
    parameters.weights.neighborhood = 0.10;
    parameters.weights.thin = 0.08;
    parameters.weights.history_instability = 0.08;
    parameters.weights.grammar = 0.14;
    parameters
}
