use serde::Serialize;

use crate::dsfb::{StateField, StructuralState};
use crate::frame::{Color, ImageFrame, ScalarField};
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
    pub alpha_min: f32,
    pub alpha_max: f32,
    pub modulate_alpha: bool,
    pub use_visibility_hint: bool,
    pub use_depth_proxy: bool,
    pub use_normal_proxy: bool,
    pub use_motion_proxy: bool,
    pub use_neighborhood_proxy: bool,
    pub use_thin_proxy: bool,
    pub use_history_instability: bool,
    pub use_grammar: bool,
    pub residual_weight: f32,
    pub visibility_weight: f32,
    pub depth_weight: f32,
    pub normal_weight: f32,
    pub motion_weight: f32,
    pub neighborhood_weight: f32,
    pub thin_weight: f32,
    pub history_instability_weight: f32,
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
            let residual_gate = smoothstep(0.015, 0.22, residual_value);
            let depth_gate = if profile.use_depth_proxy {
                smoothstep(
                    0.01,
                    0.08,
                    (inputs.current_depth[index] - inputs.reprojected_depth[index]).abs(),
                )
            } else {
                0.0
            };
            let normal_gate = if profile.use_normal_proxy {
                let dot = inputs.current_normals[index]
                    .dot(inputs.reprojected_normals[index])
                    .clamp(-1.0, 1.0);
                smoothstep(0.01, 0.16, 1.0 - dot)
            } else {
                0.0
            };
            let motion_gate = if profile.use_motion_proxy {
                motion_disagreement_proxy(inputs.motion_vectors, width, height, x, y)
            } else {
                0.0
            };
            let neighborhood_gate = if profile.use_neighborhood_proxy {
                neighborhood_inconsistency_proxy(inputs.current_color, history, x, y)
            } else {
                0.0
            };
            let thin_gate = if profile.use_thin_proxy {
                if let Some(thin_hint) = inputs.thin_hint {
                    (0.45 * thin_hint.get(x, y)
                        + 0.55 * local_contrast_proxy(inputs.current_color, x, y))
                    .clamp(0.0, 1.0)
                } else {
                    local_contrast_proxy(inputs.current_color, x, y)
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
                (0.62 * residual_gate + 0.38 * neighborhood_gate).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let state_value = classify_state(
                residual_gate,
                visibility_gate.max(depth_gate).max(normal_gate),
                motion_gate,
                thin_gate,
                neighborhood_gate,
            );
            let grammar_gate = if profile.use_grammar {
                grammar_hazard(state_value)
            } else {
                0.0
            };

            let weighted = profile.residual_weight * residual_gate
                + profile.visibility_weight * visibility_gate
                + profile.depth_weight * depth_gate
                + profile.normal_weight * normal_gate
                + profile.motion_weight * motion_gate
                + profile.neighborhood_weight * neighborhood_gate
                + profile.thin_weight * thin_gate
                + profile.history_instability_weight * history_instability_gate;
            let hazard = weighted.max(grammar_gate).clamp(0.0, 1.0);
            let trust_value = 1.0 - hazard;
            let alpha_value = if profile.modulate_alpha {
                profile.alpha_min + (profile.alpha_max - profile.alpha_min) * hazard
            } else {
                profile.alpha_min
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
    HostSupervisionProfile {
        id: "dsfb_host_realistic".to_string(),
        label: "DSFB host-realistic".to_string(),
        description: "Residual, depth, normal, motion, neighborhood, and local thin/contrast supervision without privileged visibility hints.".to_string(),
        alpha_min,
        alpha_max,
        modulate_alpha: true,
        use_visibility_hint: false,
        use_depth_proxy: true,
        use_normal_proxy: true,
        use_motion_proxy: true,
        use_neighborhood_proxy: true,
        use_thin_proxy: true,
        use_history_instability: true,
        use_grammar: true,
        residual_weight: 0.26,
        visibility_weight: 0.0,
        depth_weight: 0.18,
        normal_weight: 0.12,
        motion_weight: 0.12,
        neighborhood_weight: 0.14,
        thin_weight: 0.08,
        history_instability_weight: 0.10,
    }
}

pub fn synthetic_visibility_profile(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    HostSupervisionProfile {
        id: "dsfb_synthetic_visibility".to_string(),
        label: "DSFB visibility-assisted".to_string(),
        description: "Research/debug mode that augments host-realistic cues with a synthetic visibility hint.".to_string(),
        alpha_min,
        alpha_max,
        modulate_alpha: true,
        use_visibility_hint: true,
        use_depth_proxy: true,
        use_normal_proxy: true,
        use_motion_proxy: true,
        use_neighborhood_proxy: true,
        use_thin_proxy: true,
        use_history_instability: true,
        use_grammar: true,
        residual_weight: 0.20,
        visibility_weight: 0.22,
        depth_weight: 0.12,
        normal_weight: 0.08,
        motion_weight: 0.10,
        neighborhood_weight: 0.10,
        thin_weight: 0.08,
        history_instability_weight: 0.10,
    }
}

pub fn profile_without_visibility(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = synthetic_visibility_profile(alpha_min, alpha_max);
    profile.id = "dsfb_no_visibility".to_string();
    profile.label = "DSFB without visibility cue".to_string();
    profile.description = "Visibility-assisted DSFB ablation with the synthetic visibility cue disabled while keeping the rest of the supervisory structure intact.".to_string();
    profile.use_visibility_hint = false;
    profile.visibility_weight = 0.0;
    profile
}

pub fn profile_without_thin(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = default_host_realistic_profile(alpha_min, alpha_max);
    profile.id = "dsfb_no_thin".to_string();
    profile.label = "DSFB without thin proxy".to_string();
    profile.use_thin_proxy = false;
    profile.thin_weight = 0.0;
    profile
}

pub fn profile_without_motion(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = default_host_realistic_profile(alpha_min, alpha_max);
    profile.id = "dsfb_no_motion_edge".to_string();
    profile.label = "DSFB without motion disagreement".to_string();
    profile.use_motion_proxy = false;
    profile.motion_weight = 0.0;
    profile
}

pub fn profile_without_grammar(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = default_host_realistic_profile(alpha_min, alpha_max);
    profile.id = "dsfb_no_grammar".to_string();
    profile.label = "DSFB without grammar".to_string();
    profile.use_grammar = false;
    profile
}

pub fn profile_residual_only(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let conservative_alpha_max = alpha_min + 0.42 * (alpha_max - alpha_min);
    HostSupervisionProfile {
        id: "dsfb_residual_only".to_string(),
        label: "DSFB residual-only".to_string(),
        description: "Residual-only supervisory hazard without auxiliary structure cues. The alpha mapping is intentionally conservative so this remains a true single-cue ablation rather than a near-clone of the stronger residual-threshold baseline.".to_string(),
        alpha_min,
        alpha_max: conservative_alpha_max,
        modulate_alpha: true,
        use_visibility_hint: false,
        use_depth_proxy: false,
        use_normal_proxy: false,
        use_motion_proxy: false,
        use_neighborhood_proxy: false,
        use_thin_proxy: false,
        use_history_instability: false,
        use_grammar: false,
        residual_weight: 0.72,
        visibility_weight: 0.0,
        depth_weight: 0.0,
        normal_weight: 0.0,
        motion_weight: 0.0,
        neighborhood_weight: 0.0,
        thin_weight: 0.0,
        history_instability_weight: 0.0,
    }
}

pub fn profile_without_alpha_modulation(alpha_min: f32, alpha_max: f32) -> HostSupervisionProfile {
    let mut profile = default_host_realistic_profile(alpha_min, alpha_max);
    profile.id = "dsfb_trust_no_alpha".to_string();
    profile.label = "DSFB trust without alpha modulation".to_string();
    profile.modulate_alpha = false;
    profile
}

fn local_contrast_proxy(frame: &ImageFrame, x: usize, y: usize) -> f32 {
    let center = frame.get(x, y).luma();
    let mut strongest = 0.0f32;
    for (nx, ny) in neighbors(x, y, frame.width(), frame.height()) {
        strongest = strongest.max((center - frame.get(nx, ny).luma()).abs());
    }
    smoothstep(0.02, 0.18, strongest)
}

fn neighborhood_inconsistency_proxy(
    current_color: &ImageFrame,
    history: Color,
    x: usize,
    y: usize,
) -> f32 {
    let mut min_luma = f32::INFINITY;
    let mut max_luma = f32::NEG_INFINITY;
    for (nx, ny) in neighbors(x, y, current_color.width(), current_color.height()) {
        let luma = current_color.get(nx, ny).luma();
        min_luma = min_luma.min(luma);
        max_luma = max_luma.max(luma);
    }
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
    smoothstep(0.01, 0.14, distance)
}

fn motion_disagreement_proxy(
    motion_vectors: &[MotionVector],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
) -> f32 {
    let base = motion_vectors[y * width + x];
    let mut strongest = 0.0f32;
    for (nx, ny) in neighbors(x, y, width, height) {
        let neighbor = motion_vectors[ny * width + nx];
        let delta = (base.to_prev_x - neighbor.to_prev_x).abs()
            + (base.to_prev_y - neighbor.to_prev_y).abs();
        strongest = strongest.max(delta as f32);
    }
    smoothstep(0.5, 3.0, strongest)
}

fn classify_state(
    residual_gate: f32,
    structural_disagreement: f32,
    motion_gate: f32,
    thin_gate: f32,
    neighborhood_gate: f32,
) -> StructuralState {
    if structural_disagreement >= 0.72 {
        StructuralState::DisocclusionLike
    } else if residual_gate >= 0.38 && neighborhood_gate >= 0.22 {
        StructuralState::UnstableHistory
    } else if motion_gate >= 0.45 || (thin_gate >= 0.40 && residual_gate >= 0.18) {
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

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn neighbors(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut values = Vec::with_capacity(8);
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                values.push((nx as usize, ny as usize));
            }
        }
    }
    values
}
