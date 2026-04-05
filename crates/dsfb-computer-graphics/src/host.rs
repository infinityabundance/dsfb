use serde::Serialize;

use crate::dsfb::{StateField, StructuralState};
use crate::frame::{Color, ImageFrame, ScalarField};
use crate::parameters::{
    gated_reference_parameters, host_realistic_parameters, motion_augmented_parameters,
    synthetic_visibility_parameters, HazardMergeMode, HostSupervisionParameters,
    SmoothstepThreshold, TrustBehavior,
};
use crate::scene::{MotionVector, Normal3};

#[derive(Clone, Debug)]
pub struct HostTemporalInputs<'a> {
    pub current_color: &'a ImageFrame,
    pub reprojected_history: &'a ImageFrame,
    pub motion_vectors: &'a [MotionVector],
    pub current_depth: &'a [f32],
    pub reprojected_depth: &'a [f32],
    pub current_normals: &'a [Normal3],
    pub reprojected_normals: &'a [Normal3],
    pub visibility_hint: Option<&'a [bool]>,
    pub thin_hint: Option<&'a ScalarField>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HostSupervisionProfile {
    pub id: String,
    pub label: String,
    pub description: String,
    pub modulate_alpha: bool,
    pub use_visibility_hint: bool,
    pub use_depth_proxy: bool,
    pub use_normal_proxy: bool,
    pub use_motion_proxy: bool,
    pub use_neighborhood_proxy: bool,
    pub use_thin_proxy: bool,
    pub use_history_instability: bool,
    pub use_grammar: bool,
    pub parameters: HostSupervisionParameters,
}

#[derive(Clone, Debug)]
pub struct HostProxyFields {
    pub residual_proxy: ScalarField,
    pub visibility_proxy: ScalarField,
    pub depth_proxy: ScalarField,
    pub normal_proxy: ScalarField,
    pub motion_proxy: ScalarField,
    pub neighborhood_proxy: ScalarField,
    pub thin_proxy: ScalarField,
    pub history_instability_proxy: ScalarField,
}

#[derive(Clone, Debug)]
pub struct HostSupervisionOutputs {
    pub residual: ScalarField,
    pub trust: ScalarField,
    pub alpha: ScalarField,
    pub intervention: ScalarField,
    pub proxies: HostProxyFields,
    pub state: StateField,
}

pub fn supervise_temporal_reuse(
    inputs: &HostTemporalInputs<'_>,
    profile: &HostSupervisionProfile,
) -> HostSupervisionOutputs {
    let width = inputs.current_color.width();
    let height = inputs.current_color.height();
    let parameters = profile.parameters;

    let mut residual = ScalarField::new(width, height);
    let mut trust = ScalarField::new(width, height);
    let mut alpha = ScalarField::new(width, height);
    let mut intervention = ScalarField::new(width, height);
    let mut residual_proxy = ScalarField::new(width, height);
    let mut visibility_proxy = ScalarField::new(width, height);
    let mut depth_proxy = ScalarField::new(width, height);
    let mut normal_proxy = ScalarField::new(width, height);
    let mut motion_proxy = ScalarField::new(width, height);
    let mut neighborhood_proxy = ScalarField::new(width, height);
    let mut thin_proxy = ScalarField::new(width, height);
    let mut history_instability_proxy = ScalarField::new(width, height);
    let mut state = StateField::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let current = inputs.current_color.get(x, y);
            let history = inputs.reprojected_history.get(x, y);
            let residual_value = current.abs_diff(history);
            let residual_gate =
                smoothstep_threshold(parameters.thresholds.residual, residual_value);
            let depth_gate = if profile.use_depth_proxy {
                smoothstep_threshold(
                    parameters.thresholds.depth,
                    (inputs.current_depth[index] - inputs.reprojected_depth[index]).abs(),
                )
            } else {
                0.0
            };
            let normal_gate = if profile.use_normal_proxy {
                let dot = inputs.current_normals[index]
                    .dot(inputs.reprojected_normals[index])
                    .clamp(-1.0, 1.0);
                smoothstep_threshold(parameters.thresholds.normal, 1.0 - dot)
            } else {
                0.0
            };
            let motion_gate = if profile.use_motion_proxy {
                motion_disagreement_proxy(
                    inputs.motion_vectors,
                    width,
                    height,
                    x,
                    y,
                    parameters.thresholds.motion,
                )
            } else {
                0.0
            };
            let neighborhood_gate = if profile.use_neighborhood_proxy {
                neighborhood_inconsistency_proxy(
                    inputs.current_color,
                    history,
                    x,
                    y,
                    parameters.thresholds.neighborhood,
                )
            } else {
                0.0
            };
            let thin_gate = if profile.use_thin_proxy {
                if let Some(thin_hint) = inputs.thin_hint {
                    (parameters.thresholds.thin_hint_mix * thin_hint.get(x, y)
                        + parameters.thresholds.thin_local_contrast_mix
                            * local_contrast_proxy(
                                inputs.current_color,
                                x,
                                y,
                                parameters.thresholds.local_contrast,
                            ))
                    .clamp(0.0, 1.0)
                } else {
                    local_contrast_proxy(
                        inputs.current_color,
                        x,
                        y,
                        parameters.thresholds.local_contrast,
                    )
                }
            } else {
                0.0
            };
            let visibility_gate: f32 = if profile.use_visibility_hint {
                inputs
                    .visibility_hint
                    .map(|hint| if hint[index] { 1.0 } else { 0.0 })
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            let history_instability_gate = if profile.use_history_instability {
                (parameters.thresholds.history_instability_residual_mix * residual_gate
                    + parameters.thresholds.history_instability_neighborhood_mix
                        * neighborhood_gate)
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };

            let state_value = classify_state(
                residual_gate,
                visibility_gate.max(depth_gate).max(normal_gate),
                motion_gate,
                thin_gate,
                neighborhood_gate,
                parameters,
            );
            let grammar_component = if profile.use_grammar {
                grammar_hazard(state_value)
            } else {
                0.0
            };

            let weighted = parameters.weights.residual * residual_gate
                + parameters.weights.visibility * visibility_gate
                + parameters.weights.depth * depth_gate
                + parameters.weights.normal * normal_gate
                + parameters.weights.motion * motion_gate
                + parameters.weights.neighborhood * neighborhood_gate
                + parameters.weights.thin * thin_gate
                + parameters.weights.history_instability * history_instability_gate;
            let grammar_gate = parameters.weights.grammar * grammar_component;
            let hazard_raw = match parameters.hazard_merge_mode {
                HazardMergeMode::MaxGate => weighted.max(grammar_gate),
                HazardMergeMode::WeightedAdd => weighted + grammar_gate,
            };
            let hazard = match parameters.trust_behavior {
                TrustBehavior::GateLike => hazard_raw.clamp(0.0, 1.0),
                TrustBehavior::Graded => smoothstep_threshold(
                    parameters.thresholds.hazard_curve,
                    hazard_raw.clamp(0.0, 1.0),
                ),
            };
            let trust_value = 1.0 - hazard;
            let alpha_value = if profile.modulate_alpha {
                parameters.alpha_range.min
                    + (parameters.alpha_range.max - parameters.alpha_range.min) * hazard
            } else {
                parameters.alpha_range.min
            };

            residual.set(x, y, residual_value);
            residual_proxy.set(x, y, residual_gate);
            visibility_proxy.set(x, y, visibility_gate);
            depth_proxy.set(x, y, depth_gate);
            normal_proxy.set(x, y, normal_gate);
            motion_proxy.set(x, y, motion_gate);
            neighborhood_proxy.set(x, y, neighborhood_gate);
            thin_proxy.set(x, y, thin_gate);
            history_instability_proxy.set(x, y, history_instability_gate);
            trust.set(x, y, trust_value);
            alpha.set(x, y, alpha_value);
            intervention.set(x, y, hazard);
            state.set(x, y, state_value);
        }
    }

    HostSupervisionOutputs {
        residual,
        trust,
        alpha,
        intervention,
        proxies: HostProxyFields {
            residual_proxy,
            visibility_proxy,
            depth_proxy,
            normal_proxy,
            motion_proxy,
            neighborhood_proxy,
            thin_proxy,
            history_instability_proxy,
        },
        state,
    }
}

pub fn default_host_realistic_profile(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut parameters = host_realistic_parameters();
    parameters.alpha_range.min = alpha_min;
    parameters.alpha_range.max = alpha_max;
    HostSupervisionProfile {
        id: "dsfb_host_realistic".to_string(),
        label: "DSFB host-realistic minimum".to_string(),
        description: "Minimum decision-facing path: residual, depth, normal, neighborhood, thin proxy, and grammar supervision without privileged visibility or motion disagreement.".to_string(),
        modulate_alpha: true,
        use_visibility_hint: false,
        use_depth_proxy: true,
        use_normal_proxy: true,
        use_motion_proxy: false,
        use_neighborhood_proxy: true,
        use_thin_proxy: true,
        use_history_instability: true,
        use_grammar: true,
        parameters,
    }
}

pub fn synthetic_visibility_profile(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut parameters = synthetic_visibility_parameters();
    parameters.alpha_range.min = alpha_min;
    parameters.alpha_range.max = alpha_max;
    HostSupervisionProfile {
        id: "dsfb_synthetic_visibility".to_string(),
        label: "DSFB visibility-assisted".to_string(),
        description: "Research/debug mode that augments host-realistic cues with a synthetic visibility hint.".to_string(),
        modulate_alpha: true,
        use_visibility_hint: true,
        use_depth_proxy: true,
        use_normal_proxy: true,
        use_motion_proxy: true,
        use_neighborhood_proxy: true,
        use_thin_proxy: true,
        use_history_instability: true,
        use_grammar: true,
        parameters,
    }
}

pub fn motion_augmented_profile(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut parameters = motion_augmented_parameters();
    parameters.alpha_range.min = alpha_min;
    parameters.alpha_range.max = alpha_max;
    HostSupervisionProfile {
        id: "dsfb_motion_augmented".to_string(),
        label: "DSFB motion-augmented".to_string(),
        description: "Optional extension that adds motion disagreement to the minimum host-realistic path. It is kept only if scenario evidence shows it matters.".to_string(),
        modulate_alpha: true,
        use_visibility_hint: false,
        use_depth_proxy: true,
        use_normal_proxy: true,
        use_motion_proxy: true,
        use_neighborhood_proxy: true,
        use_thin_proxy: true,
        use_history_instability: true,
        use_grammar: true,
        parameters,
    }
}

pub fn gated_reference_profile(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut parameters = gated_reference_parameters();
    parameters.alpha_range.min = alpha_min;
    parameters.alpha_range.max = alpha_max;
    HostSupervisionProfile {
        id: "dsfb_host_gated_reference".to_string(),
        label: "DSFB gated reference".to_string(),
        description: "Reference implementation of the earlier near-binary gate-like supervisory mode, retained for trust diagnostics and comparison.".to_string(),
        modulate_alpha: true,
        use_visibility_hint: false,
        use_depth_proxy: true,
        use_normal_proxy: true,
        use_motion_proxy: true,
        use_neighborhood_proxy: true,
        use_thin_proxy: true,
        use_history_instability: true,
        use_grammar: true,
        parameters,
    }
}

pub fn profile_without_visibility(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = synthetic_visibility_profile(alpha_min, alpha_max);
    profile.id = "dsfb_no_visibility".to_string();
    profile.label = "DSFB without visibility cue".to_string();
    profile.description = "Visibility-assisted DSFB ablation with the synthetic visibility cue disabled while keeping the rest of the supervisory structure intact.".to_string();
    profile.use_visibility_hint = false;
    profile.parameters.weights.visibility = 0.0;
    profile
}

pub fn profile_without_thin(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = default_host_realistic_profile(alpha_min, alpha_max);
    profile.id = "dsfb_no_thin".to_string();
    profile.label = "DSFB without thin proxy".to_string();
    profile.use_thin_proxy = false;
    profile.parameters.weights.thin = 0.0;
    profile
}

pub fn profile_without_motion(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = motion_augmented_profile(alpha_min, alpha_max);
    profile.id = "dsfb_no_motion_edge".to_string();
    profile.label = "DSFB without motion disagreement".to_string();
    profile.use_motion_proxy = false;
    profile.parameters.weights.motion = 0.0;
    profile
}

pub fn profile_without_grammar(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = default_host_realistic_profile(alpha_min, alpha_max);
    profile.id = "dsfb_no_grammar".to_string();
    profile.label = "DSFB without grammar".to_string();
    profile.use_grammar = false;
    profile.parameters.weights.grammar = 0.0;
    profile
}

pub fn profile_residual_only(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let conservative_alpha_max = alpha_min + 0.42 * (alpha_max - alpha_min);
    let mut parameters = host_realistic_parameters();
    parameters.alpha_range.min = alpha_min;
    parameters.alpha_range.max = conservative_alpha_max;
    parameters.weights.residual = 0.72;
    parameters.weights.visibility = 0.0;
    parameters.weights.depth = 0.0;
    parameters.weights.normal = 0.0;
    parameters.weights.motion = 0.0;
    parameters.weights.neighborhood = 0.0;
    parameters.weights.thin = 0.0;
    parameters.weights.history_instability = 0.0;
    parameters.weights.grammar = 0.0;
    HostSupervisionProfile {
        id: "dsfb_residual_only".to_string(),
        label: "DSFB residual-only".to_string(),
        description: "Residual-only supervisory hazard without auxiliary structure cues. The alpha mapping is intentionally conservative so this remains a true single-cue ablation rather than a near-clone of the stronger residual-threshold baseline.".to_string(),
        modulate_alpha: true,
        use_visibility_hint: false,
        use_depth_proxy: false,
        use_normal_proxy: false,
        use_motion_proxy: false,
        use_neighborhood_proxy: false,
        use_thin_proxy: false,
        use_history_instability: false,
        use_grammar: false,
        parameters,
    }
}

pub fn profile_without_alpha_modulation(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = default_host_realistic_profile(alpha_min, alpha_max);
    profile.id = "dsfb_trust_no_alpha".to_string();
    profile.label = "DSFB trust without alpha modulation".to_string();
    profile.modulate_alpha = false;
    profile
}

fn local_contrast_proxy(
    frame: &ImageFrame,
    x: usize,
    y: usize,
    threshold: SmoothstepThreshold,
) -> f32 {
    let center = frame.get(x, y).luma();
    let mut strongest = 0.0f32;
    for_each_neighbor(x, y, frame.width(), frame.height(), |nx, ny| {
        strongest = strongest.max((center - frame.get(nx, ny).luma()).abs());
    });
    smoothstep_threshold(threshold, strongest)
}

fn neighborhood_inconsistency_proxy(
    current_color: &ImageFrame,
    history: Color,
    x: usize,
    y: usize,
    threshold: SmoothstepThreshold,
) -> f32 {
    let mut min_luma = f32::INFINITY;
    let mut max_luma = f32::NEG_INFINITY;
    for_each_neighbor(x, y, current_color.width(), current_color.height(), |nx, ny| {
        let luma = current_color.get(nx, ny).luma();
        min_luma = min_luma.min(luma);
        max_luma = max_luma.max(luma);
    });
    let current_luma = current_color.get(x, y).luma();
    min_luma = min_luma.min(current_luma);
    max_luma = max_luma.max(current_luma);
    let history_luma = history.luma();
    let distance = if history_luma < min_luma {
        min_luma - history_luma
    } else if history_luma > max_luma {
        history_luma - max_luma
    } else {
        0.0
    };
    smoothstep_threshold(threshold, distance)
}

fn motion_disagreement_proxy(
    motion_vectors: &[MotionVector],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    threshold: SmoothstepThreshold,
) -> f32 {
    let base = motion_vectors[y * width + x];
    let mut strongest = 0.0f32;
    for_each_neighbor(x, y, width, height, |nx, ny| {
        let neighbor = motion_vectors[ny * width + nx];
        let delta_x = base.to_prev_x - neighbor.to_prev_x;
        let delta_y = base.to_prev_y - neighbor.to_prev_y;
        strongest = strongest.max((delta_x * delta_x + delta_y * delta_y).sqrt());
    });
    smoothstep_threshold(threshold, strongest)
}

fn classify_state(
    residual_gate: f32,
    structural_disagreement: f32,
    motion_gate: f32,
    thin_gate: f32,
    neighborhood_gate: f32,
    parameters: HostSupervisionParameters,
) -> StructuralState {
    if structural_disagreement >= parameters.structural.disocclusion_like {
        StructuralState::DisocclusionLike
    } else if residual_gate >= parameters.structural.unstable_residual
        && neighborhood_gate >= parameters.structural.unstable_neighborhood
    {
        StructuralState::UnstableHistory
    } else if motion_gate >= parameters.structural.motion_edge
        || (thin_gate >= parameters.structural.thin_edge
            && residual_gate >= parameters.structural.thin_residual)
    {
        StructuralState::MotionEdge
    } else {
        StructuralState::Nominal
    }
}

fn grammar_hazard(state: StructuralState) -> f32 {
    match state {
        StructuralState::Nominal => 0.0,
        StructuralState::MotionEdge => 0.32,
        StructuralState::UnstableHistory => 0.62,
        StructuralState::DisocclusionLike => 0.88,
    }
}

fn smoothstep_threshold(threshold: SmoothstepThreshold, value: f32) -> f32 {
    let edge_span = (threshold.high - threshold.low).max(f32::EPSILON);
    let t = ((value - threshold.low) / edge_span).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Calls `f` for each of the (up to 8) 8-connected neighbours of `(x, y)`.
/// Zero heap allocation. Inlined by the compiler in the per-pixel hot path.
#[inline(always)]
fn for_each_neighbor(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    mut f: impl FnMut(usize, usize),
) {
    let x = x as i32;
    let y = y as i32;
    let w = width as i32;
    let h = height as i32;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < w && ny >= 0 && ny < h {
                f(nx as usize, ny as usize);
            }
        }
    }
}
