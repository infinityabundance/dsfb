use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneConfig {
    pub width: usize,
    pub height: usize,
    pub frame_count: usize,
    pub object_width: usize,
    pub object_height: usize,
    pub object_start_x: i32,
    pub object_stop_x: i32,
    pub object_top_y: i32,
    pub move_frames: usize,
    pub thin_vertical_x: i32,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            width: 160,
            height: 96,
            frame_count: 18,
            object_width: 38,
            object_height: 44,
            object_start_x: 24,
            object_stop_x: 58,
            object_top_y: 26,
            move_frames: 6,
            thin_vertical_x: 54,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DemoConfig {
    pub scene: SceneConfig,
    pub baseline_alpha: f32,
    pub dsfb_alpha_min: f32,
    pub dsfb_alpha_max: f32,
    pub trust_map_frame_offset: usize,
    pub comparison_frame_offset: usize,
    pub demo_b_reference_spp: usize,
    pub demo_b_uniform_spp: usize,
    pub demo_b_min_spp: usize,
    pub demo_b_max_spp: usize,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            scene: SceneConfig::default(),
            baseline_alpha: 0.12,
            dsfb_alpha_min: 0.08,
            dsfb_alpha_max: 0.96,
            trust_map_frame_offset: 0,
            comparison_frame_offset: 2,
            demo_b_reference_spp: 64,
            demo_b_uniform_spp: 2,
            demo_b_min_spp: 1,
            demo_b_max_spp: 12,
        }
    }
}
