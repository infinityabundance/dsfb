use serde::Serialize;

use crate::config::SceneConfig;
use crate::frame::{Color, ImageFrame};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum ScenarioId {
    ThinReveal,
    FastPan,
    DiagonalReveal,
    ContrastPulse,
    StabilityHoldout,
}

impl ScenarioId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ThinReveal => "thin_reveal",
            Self::FastPan => "fast_pan",
            Self::DiagonalReveal => "diagonal_reveal",
            Self::ContrastPulse => "contrast_pulse",
            Self::StabilityHoldout => "stability_holdout",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::ThinReveal => "Thin-Structure Reveal",
            Self::FastPan => "Fast Lateral Reveal",
            Self::DiagonalReveal => "Diagonal Subpixel Reveal",
            Self::ContrastPulse => "Contrast Pulse Stress",
            Self::StabilityHoldout => "Stability Holdout",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ScenarioExpectation {
    BenefitExpected,
    NeutralExpected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SurfaceTag {
    Background,
    ThinStructure,
    ForegroundObject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MotionVector {
    pub to_prev_x: i32,
    pub to_prev_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Normal3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Normal3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn normalized(self) -> Self {
        let norm = (self.x * self.x + self.y * self.y + self.z * self.z)
            .sqrt()
            .max(f32::EPSILON);
        Self::new(self.x / norm, self.y / norm, self.z / norm)
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

#[derive(Clone, Debug)]
pub struct SceneFrame {
    pub index: usize,
    pub ground_truth: ImageFrame,
    pub layers: Vec<SurfaceTag>,
    pub motion: Vec<MotionVector>,
    pub disocclusion_mask: Vec<bool>,
    pub object_rect: Rect,
    pub depth: Vec<f32>,
    pub normals: Vec<Normal3>,
}

#[derive(Clone, Debug)]
pub struct SceneSequence {
    pub config: SceneConfig,
    pub scenario_id: ScenarioId,
    pub scenario_title: String,
    pub scenario_description: String,
    pub expectation: ScenarioExpectation,
    pub onset_frame: usize,
    pub target_label: String,
    pub target_mask: Vec<bool>,
    pub frames: Vec<SceneFrame>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SceneManifest {
    pub scenario_id: String,
    pub scenario_title: String,
    pub scenario_description: String,
    pub expectation: ScenarioExpectation,
    pub target_label: String,
    pub config: SceneConfig,
    pub frame_count: usize,
    pub onset_frame: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioDefinition {
    pub id: ScenarioId,
    pub title: &'static str,
    pub description: &'static str,
    pub expectation: ScenarioExpectation,
    pub target_label: &'static str,
    pub scene: SceneConfig,
    pub onset_frame: usize,
}

#[derive(Clone, Copy, Debug)]
enum BackgroundStyle {
    Default,
    Textured,
    HighContrast,
    Calm,
}

#[derive(Clone, Copy, Debug)]
enum ThinStyle {
    VerticalAndDiagonal,
    DiagonalOnly,
    None,
}

#[derive(Clone, Copy, Debug)]
enum MotionProfile {
    EaseOut,
    FastPan,
    Static,
}

#[derive(Clone, Copy, Debug)]
struct PulseSpec {
    rect: Rect,
    start_frame: usize,
    intensity: f32,
}

#[derive(Clone, Debug)]
struct InternalScenarioSpec {
    id: ScenarioId,
    title: &'static str,
    description: &'static str,
    expectation: ScenarioExpectation,
    target_label: &'static str,
    scene: SceneConfig,
    onset_frame: usize,
    background_style: BackgroundStyle,
    thin_style: ThinStyle,
    motion_profile: MotionProfile,
    pulse: Option<PulseSpec>,
}

pub fn canonical_scenario(config: &SceneConfig) -> ScenarioDefinition {
    let spec = internal_canonical_spec(config);
    ScenarioDefinition {
        id: spec.id,
        title: spec.title,
        description: spec.description,
        expectation: spec.expectation,
        target_label: spec.target_label,
        scene: spec.scene,
        onset_frame: spec.onset_frame,
    }
}

pub fn scenario_suite(config: &SceneConfig) -> Vec<ScenarioDefinition> {
    internal_scenario_suite(config)
        .into_iter()
        .map(|spec| ScenarioDefinition {
            id: spec.id,
            title: spec.title,
            description: spec.description,
            expectation: spec.expectation,
            target_label: spec.target_label,
            scene: spec.scene,
            onset_frame: spec.onset_frame,
        })
        .collect()
}

pub fn scenario_by_id(config: &SceneConfig, scenario_id: ScenarioId) -> Option<ScenarioDefinition> {
    internal_scenario_suite(config)
        .into_iter()
        .find(|spec| spec.id == scenario_id)
        .map(|spec| ScenarioDefinition {
            id: spec.id,
            title: spec.title,
            description: spec.description,
            expectation: spec.expectation,
            target_label: spec.target_label,
            scene: spec.scene,
            onset_frame: spec.onset_frame,
        })
}

pub fn generate_sequence(config: &SceneConfig) -> SceneSequence {
    generate_sequence_for_scenario(&internal_canonical_spec(config))
}

pub fn generate_sequence_for_definition(definition: &ScenarioDefinition) -> SceneSequence {
    let spec = internal_scenario_suite(&definition.scene)
        .into_iter()
        .find(|candidate| candidate.id == definition.id)
        .unwrap_or_else(|| InternalScenarioSpec {
            id: definition.id,
            title: definition.title,
            description: definition.description,
            expectation: definition.expectation,
            target_label: definition.target_label,
            scene: definition.scene.clone(),
            onset_frame: definition.onset_frame,
            background_style: BackgroundStyle::Default,
            thin_style: ThinStyle::VerticalAndDiagonal,
            motion_profile: MotionProfile::EaseOut,
            pulse: None,
        });
    generate_sequence_for_scenario(&spec)
}

pub fn build_manifest(sequence: &SceneSequence) -> SceneManifest {
    SceneManifest {
        scenario_id: sequence.scenario_id.as_str().to_string(),
        scenario_title: sequence.scenario_title.clone(),
        scenario_description: sequence.scenario_description.clone(),
        expectation: sequence.expectation,
        target_label: sequence.target_label.clone(),
        config: sequence.config.clone(),
        frame_count: sequence.frames.len(),
        onset_frame: sequence.onset_frame,
    }
}

fn generate_sequence_for_scenario(spec: &InternalScenarioSpec) -> SceneSequence {
    let object_positions = build_object_positions(&spec.scene, spec.motion_profile);
    let mut frames: Vec<SceneFrame> = Vec::with_capacity(spec.scene.frame_count);

    for frame_index in 0..spec.scene.frame_count {
        let object_rect = Rect {
            x: object_positions[frame_index],
            y: spec.scene.object_top_y,
            width: spec.scene.object_width as i32,
            height: spec.scene.object_height as i32,
        };
        let previous_object_x = if frame_index == 0 {
            object_rect.x
        } else {
            object_positions[frame_index - 1]
        };
        let object_dx = object_rect.x - previous_object_x;

        let mut ground_truth = ImageFrame::new(spec.scene.width, spec.scene.height);
        let mut layers = vec![SurfaceTag::Background; spec.scene.width * spec.scene.height];
        let mut motion = vec![
            MotionVector {
                to_prev_x: 0,
                to_prev_y: 0,
            };
            spec.scene.width * spec.scene.height
        ];
        let mut depth = vec![0.0f32; spec.scene.width * spec.scene.height];
        let mut normals = vec![Normal3::new(0.0, 0.0, 1.0); spec.scene.width * spec.scene.height];

        for y in 0..spec.scene.height {
            for x in 0..spec.scene.width {
                let x_i = x as i32;
                let y_i = y as i32;
                let pixel_index = y * spec.scene.width + x;
                let background_base = background_color(x, y, &spec.scene, spec.background_style);
                let mut color = apply_pulse(background_base, frame_index, x_i, y_i, spec.pulse);
                let mut layer = SurfaceTag::Background;
                let mut depth_value = background_depth(x, y, &spec.scene, spec.background_style);
                let mut normal_value = background_normal(x, y, &spec.scene, spec.background_style);

                if is_thin_structure(x_i, y_i, &spec.scene, spec.thin_style) {
                    color = apply_pulse(
                        thin_structure_color(x_i, y_i, &spec.scene, spec.thin_style),
                        frame_index,
                        x_i,
                        y_i,
                        spec.pulse,
                    );
                    layer = SurfaceTag::ThinStructure;
                    depth_value = thin_structure_depth(x_i, y_i, &spec.scene, spec.thin_style);
                    normal_value = thin_structure_normal(x_i, y_i, &spec.scene, spec.thin_style);
                }

                if !matches!(spec.motion_profile, MotionProfile::Static)
                    && object_rect.contains(x_i, y_i)
                {
                    color = object_color(x_i, y_i, object_rect);
                    layer = SurfaceTag::ForegroundObject;
                    depth_value = object_depth(x_i, y_i, object_rect);
                    normal_value = object_normal(x_i, y_i, object_rect);
                }

                ground_truth.set(x, y, color);
                layers[pixel_index] = layer;
                depth[pixel_index] = depth_value;
                normals[pixel_index] = normal_value.normalized();
                if matches!(layer, SurfaceTag::ForegroundObject) {
                    motion[pixel_index] = MotionVector {
                        to_prev_x: -object_dx,
                        to_prev_y: 0,
                    };
                }
            }
        }

        let disocclusion_mask = if frame_index == 0 {
            vec![false; spec.scene.width * spec.scene.height]
        } else {
            let previous_layers = &frames[frame_index - 1].layers;
            let mut mask = vec![false; spec.scene.width * spec.scene.height];
            for y in 0..spec.scene.height {
                for x in 0..spec.scene.width {
                    let index = y * spec.scene.width + x;
                    let motion_vector = motion[index];
                    let prev_x = (x as i32 + motion_vector.to_prev_x)
                        .clamp(0, spec.scene.width as i32 - 1)
                        as usize;
                    let prev_y = (y as i32 + motion_vector.to_prev_y)
                        .clamp(0, spec.scene.height as i32 - 1)
                        as usize;
                    let previous_layer = previous_layers[prev_y * spec.scene.width + prev_x];
                    mask[index] = previous_layer != layers[index]
                        && !matches!(layers[index], SurfaceTag::ForegroundObject);
                }
            }
            mask
        };

        frames.push(SceneFrame {
            index: frame_index,
            ground_truth,
            layers,
            motion,
            disocclusion_mask,
            object_rect,
            depth,
            normals,
        });
    }

    let target_mask = build_target_mask(spec, &frames);

    SceneSequence {
        config: spec.scene.clone(),
        scenario_id: spec.id,
        scenario_title: spec.title.to_string(),
        scenario_description: spec.description.to_string(),
        expectation: spec.expectation,
        onset_frame: spec.onset_frame,
        target_label: spec.target_label.to_string(),
        target_mask,
        frames,
    }
}

fn build_target_mask(spec: &InternalScenarioSpec, frames: &[SceneFrame]) -> Vec<bool> {
    let width = spec.scene.width;
    let height = spec.scene.height;
    let frame = &frames[spec.onset_frame.min(frames.len().saturating_sub(1))];

    match spec.id {
        ScenarioId::ThinReveal | ScenarioId::FastPan | ScenarioId::DiagonalReveal => frame
            .layers
            .iter()
            .zip(frame.disocclusion_mask.iter().copied())
            .map(|(layer, disoccluded)| disoccluded && matches!(*layer, SurfaceTag::ThinStructure))
            .collect(),
        ScenarioId::ContrastPulse => {
            let pulse = spec
                .pulse
                .expect("contrast-pulse scenarios require a pulse region");
            let mut mask = vec![false; width * height];
            for y in 0..height {
                for x in 0..width {
                    let x_i = x as i32;
                    let y_i = y as i32;
                    if pulse.rect.contains(x_i, y_i)
                        && !matches!(frame.layers[y * width + x], SurfaceTag::ForegroundObject)
                    {
                        mask[y * width + x] = true;
                    }
                }
            }
            mask
        }
        ScenarioId::StabilityHoldout => {
            let mut mask = vec![false; width * height];
            let band = Rect {
                x: (width as i32 / 2) - 18,
                y: (height as i32 / 2) - 14,
                width: 36,
                height: 28,
            };
            for y in 0..height {
                for x in 0..width {
                    let x_i = x as i32;
                    let y_i = y as i32;
                    if band.contains(x_i, y_i)
                        && !matches!(frame.layers[y * width + x], SurfaceTag::ForegroundObject)
                    {
                        mask[y * width + x] = true;
                    }
                }
            }
            mask
        }
    }
}

fn internal_canonical_spec(config: &SceneConfig) -> InternalScenarioSpec {
    InternalScenarioSpec {
        id: ScenarioId::ThinReveal,
        title: ScenarioId::ThinReveal.title(),
        description: "Moving occluder reveals thin vertical and diagonal structure on a deterministic patterned background.",
        expectation: ScenarioExpectation::BenefitExpected,
        target_label: "revealed thin structure",
        scene: config.clone(),
        onset_frame: config.move_frames.min(config.frame_count.saturating_sub(2)),
        background_style: BackgroundStyle::Default,
        thin_style: ThinStyle::VerticalAndDiagonal,
        motion_profile: MotionProfile::EaseOut,
        pulse: None,
    }
}

fn internal_scenario_suite(config: &SceneConfig) -> Vec<InternalScenarioSpec> {
    let base_onset = config.move_frames.min(config.frame_count.saturating_sub(2));

    let mut fast_pan_scene = config.clone();
    fast_pan_scene.object_width = 26;
    fast_pan_scene.object_height = 46;
    fast_pan_scene.object_start_x = 14;
    fast_pan_scene.object_stop_x = 86;
    fast_pan_scene.move_frames = 4;

    let mut diagonal_scene = config.clone();
    diagonal_scene.object_width = 24;
    diagonal_scene.object_height = 42;
    diagonal_scene.object_start_x = 44;
    diagonal_scene.object_stop_x = 70;
    diagonal_scene.move_frames = 5;
    diagonal_scene.thin_vertical_x = 70;

    let mut contrast_scene = config.clone();
    contrast_scene.object_start_x = 20;
    contrast_scene.object_stop_x = 20;
    contrast_scene.move_frames = 0;

    let mut holdout_scene = config.clone();
    holdout_scene.object_start_x = 8;
    holdout_scene.object_stop_x = 8;
    holdout_scene.move_frames = 0;

    vec![
        internal_canonical_spec(config),
        InternalScenarioSpec {
            id: ScenarioId::FastPan,
            title: ScenarioId::FastPan.title(),
            description: "Faster occluder motion over a textured backdrop stresses motion disagreement, depth rejection, and neighborhood stability.",
            expectation: ScenarioExpectation::BenefitExpected,
            target_label: "fast-pan reveal region",
            scene: fast_pan_scene.clone(),
            onset_frame: fast_pan_scene.move_frames.min(fast_pan_scene.frame_count.saturating_sub(2)),
            background_style: BackgroundStyle::Textured,
            thin_style: ThinStyle::VerticalAndDiagonal,
            motion_profile: MotionProfile::FastPan,
            pulse: None,
        },
        InternalScenarioSpec {
            id: ScenarioId::DiagonalReveal,
            title: ScenarioId::DiagonalReveal.title(),
            description: "Diagonal subpixel structure on a high-contrast background stresses neighborhood clamping and thin-structure proxies.",
            expectation: ScenarioExpectation::BenefitExpected,
            target_label: "diagonal thin reveal",
            scene: diagonal_scene.clone(),
            onset_frame: diagonal_scene.move_frames.min(diagonal_scene.frame_count.saturating_sub(2)),
            background_style: BackgroundStyle::HighContrast,
            thin_style: ThinStyle::DiagonalOnly,
            motion_profile: MotionProfile::EaseOut,
            pulse: None,
        },
        InternalScenarioSpec {
            id: ScenarioId::ContrastPulse,
            title: ScenarioId::ContrastPulse.title(),
            description: "A bounded lighting change with no geometry reveal stresses false positives and is intended as a low-benefit honesty case rather than a DSFB win scenario.",
            expectation: ScenarioExpectation::NeutralExpected,
            target_label: "pulse region",
            scene: contrast_scene.clone(),
            onset_frame: base_onset,
            background_style: BackgroundStyle::Calm,
            thin_style: ThinStyle::None,
            motion_profile: MotionProfile::Static,
            pulse: Some(PulseSpec {
                rect: Rect {
                    x: (contrast_scene.width as i32 / 2) - 18,
                    y: (contrast_scene.height as i32 / 2) - 18,
                    width: 52,
                    height: 36,
                },
                start_frame: base_onset,
                intensity: 1.22,
            }),
        },
        InternalScenarioSpec {
            id: ScenarioId::StabilityHoldout,
            title: ScenarioId::StabilityHoldout.title(),
            description: "Static holdout case with no reveal event. Useful for verifying low false-positive intervention and bounded neutral behavior.",
            expectation: ScenarioExpectation::NeutralExpected,
            target_label: "holdout background patch",
            scene: holdout_scene,
            onset_frame: base_onset,
            background_style: BackgroundStyle::Default,
            thin_style: ThinStyle::VerticalAndDiagonal,
            motion_profile: MotionProfile::Static,
            pulse: None,
        },
    ]
}

fn build_object_positions(config: &SceneConfig, profile: MotionProfile) -> Vec<i32> {
    let mut positions = Vec::with_capacity(config.frame_count);
    for frame_index in 0..config.frame_count {
        let position = match profile {
            MotionProfile::Static => config.object_start_x as f32,
            MotionProfile::EaseOut => {
                if frame_index < config.move_frames.max(1) {
                    let t = frame_index as f32 / config.move_frames.max(1) as f32;
                    let eased = 1.0 - (1.0 - t).powi(2);
                    config.object_start_x as f32
                        + (config.object_stop_x - config.object_start_x) as f32 * eased
                } else {
                    config.object_stop_x as f32
                }
            }
            MotionProfile::FastPan => {
                if frame_index < config.move_frames.max(1) {
                    let t = frame_index as f32 / config.move_frames.max(1) as f32;
                    let eased = t.powf(0.75);
                    config.object_start_x as f32
                        + (config.object_stop_x - config.object_start_x) as f32 * eased
                } else {
                    config.object_stop_x as f32
                }
            }
        };
        positions.push(position.round() as i32);
    }
    positions
}

fn background_color(x: usize, y: usize, config: &SceneConfig, style: BackgroundStyle) -> Color {
    let xf = x as f32 / (config.width.saturating_sub(1).max(1)) as f32;
    let yf = y as f32 / (config.height.saturating_sub(1).max(1)) as f32;
    let checker = if ((x / 12) + (y / 12)) % 2 == 0 {
        1.0
    } else {
        0.0
    };
    let diagonal = if (x + 2 * y) % 22 < 6 { 1.0 } else { 0.0 };
    let stripes = if (3 * x + y) % 17 < 5 { 1.0 } else { 0.0 };
    let vignette_x = (xf - 0.5).abs();
    let vignette_y = (yf - 0.5).abs();
    let vignette = 1.0 - (vignette_x * 0.35 + vignette_y * 0.4);

    match style {
        BackgroundStyle::Default => Color::rgb(
            (0.12 + 0.16 * xf + 0.05 * checker + 0.03 * diagonal) * vignette,
            (0.15 + 0.11 * yf + 0.04 * diagonal) * vignette,
            (0.22 + 0.18 * (1.0 - xf) + 0.03 * checker) * vignette,
        ),
        BackgroundStyle::Textured => Color::rgb(
            (0.10 + 0.18 * xf + 0.08 * checker + 0.05 * stripes) * vignette,
            (0.11 + 0.15 * yf + 0.10 * diagonal + 0.04 * stripes) * vignette,
            (0.18 + 0.20 * (1.0 - xf) + 0.06 * checker) * vignette,
        ),
        BackgroundStyle::HighContrast => Color::rgb(
            (0.08 + 0.24 * checker + 0.20 * diagonal + 0.05 * xf) * vignette,
            (0.08 + 0.18 * stripes + 0.07 * yf) * vignette,
            (0.12 + 0.25 * (1.0 - checker) + 0.04 * xf) * vignette,
        ),
        BackgroundStyle::Calm => {
            Color::rgb(0.18 + 0.06 * xf, 0.18 + 0.05 * yf, 0.24 + 0.06 * (1.0 - xf))
        }
    }
}

fn background_depth(x: usize, y: usize, config: &SceneConfig, style: BackgroundStyle) -> f32 {
    let xf = x as f32 / config.width.max(1) as f32;
    let yf = y as f32 / config.height.max(1) as f32;
    let base = 0.78 + 0.06 * xf + 0.04 * yf;
    match style {
        BackgroundStyle::Default | BackgroundStyle::Calm => base,
        BackgroundStyle::Textured => base + 0.01 * ((x / 8 + y / 7) % 3) as f32,
        BackgroundStyle::HighContrast => base + 0.015 * ((x / 6 + y / 5) % 2) as f32,
    }
}

fn background_normal(x: usize, y: usize, config: &SceneConfig, style: BackgroundStyle) -> Normal3 {
    let xf = x as f32 / config.width.max(1) as f32;
    let yf = y as f32 / config.height.max(1) as f32;
    let tilt = match style {
        BackgroundStyle::Default => 0.03,
        BackgroundStyle::Textured => 0.08,
        BackgroundStyle::HighContrast => 0.10,
        BackgroundStyle::Calm => 0.01,
    };
    Normal3::new((xf - 0.5) * tilt, (0.5 - yf) * tilt, 1.0).normalized()
}

fn is_thin_structure(x: i32, y: i32, config: &SceneConfig, style: ThinStyle) -> bool {
    let vertical = x == config.thin_vertical_x && y >= 14 && y <= config.height as i32 - 14;
    let diagonal_line = {
        let diagonal = 0.58 * x as f32 + 10.0;
        (y as f32 - diagonal).abs() <= 0.55 && (28..=118).contains(&x)
    };
    match style {
        ThinStyle::VerticalAndDiagonal => vertical || diagonal_line,
        ThinStyle::DiagonalOnly => diagonal_line,
        ThinStyle::None => false,
    }
}

fn thin_structure_color(x: i32, y: i32, config: &SceneConfig, style: ThinStyle) -> Color {
    match style {
        ThinStyle::VerticalAndDiagonal if x == config.thin_vertical_x => {
            let pulse = if y % 6 < 3 { 1.0 } else { 0.82 };
            Color::rgb(0.95 * pulse, 0.96 * pulse, 0.98)
        }
        ThinStyle::DiagonalOnly => Color::rgb(0.24, 0.29, 0.35),
        ThinStyle::VerticalAndDiagonal => Color::rgb(0.64, 0.90, 0.96),
        ThinStyle::None => Color::rgb(0.0, 0.0, 0.0),
    }
}

fn thin_structure_depth(x: i32, _y: i32, config: &SceneConfig, style: ThinStyle) -> f32 {
    match style {
        ThinStyle::VerticalAndDiagonal if x == config.thin_vertical_x => 0.70,
        ThinStyle::DiagonalOnly => 0.68,
        ThinStyle::VerticalAndDiagonal => 0.72,
        ThinStyle::None => 0.80,
    }
}

fn thin_structure_normal(x: i32, _y: i32, config: &SceneConfig, style: ThinStyle) -> Normal3 {
    match style {
        ThinStyle::VerticalAndDiagonal if x == config.thin_vertical_x => {
            Normal3::new(0.05, 0.0, 0.998).normalized()
        }
        ThinStyle::DiagonalOnly => Normal3::new(0.24, -0.08, 0.96).normalized(),
        ThinStyle::VerticalAndDiagonal => Normal3::new(0.16, -0.06, 0.98).normalized(),
        ThinStyle::None => Normal3::new(0.0, 0.0, 1.0),
    }
}

fn object_color(x: i32, y: i32, rect: Rect) -> Color {
    let local_x = (x - rect.x) as f32 / rect.width.max(1) as f32;
    let local_y = (y - rect.y) as f32 / rect.height.max(1) as f32;
    let stripe = if local_x > 0.36 && local_x < 0.46 {
        0.55
    } else {
        1.0
    };
    let rim = if !(2..=(rect.width - 3)).contains(&(x - rect.x))
        || !(2..=(rect.height - 3)).contains(&(y - rect.y))
    {
        1.12
    } else {
        1.0
    };

    Color::rgb(
        (0.82 + 0.10 * local_y) * stripe * rim,
        (0.35 + 0.12 * (1.0 - local_y)) * stripe * rim,
        (0.20 + 0.08 * local_x) * stripe * rim,
    )
    .clamp01()
}

fn object_depth(x: i32, y: i32, rect: Rect) -> f32 {
    let local_x = (x - rect.x) as f32 / rect.width.max(1) as f32;
    let local_y = (y - rect.y) as f32 / rect.height.max(1) as f32;
    0.30 + 0.05 * local_x + 0.03 * local_y
}

fn object_normal(x: i32, y: i32, rect: Rect) -> Normal3 {
    let local_x = (x - rect.x) as f32 / rect.width.max(1) as f32 - 0.5;
    let local_y = (y - rect.y) as f32 / rect.height.max(1) as f32 - 0.5;
    Normal3::new(local_x * 0.24, -local_y * 0.12, 1.0).normalized()
}

fn apply_pulse(
    color: Color,
    frame_index: usize,
    x: i32,
    y: i32,
    pulse: Option<PulseSpec>,
) -> Color {
    let Some(pulse) = pulse else {
        return color;
    };
    if frame_index < pulse.start_frame || !pulse.rect.contains(x, y) {
        return color;
    }
    Color::rgb(
        color.r * pulse.intensity,
        color.g * pulse.intensity,
        color.b * pulse.intensity,
    )
    .clamp01()
}
