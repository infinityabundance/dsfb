use serde::{Deserialize, Serialize};

use crate::parameters::{
    baseline_parameters, host_realistic_parameters, AlphaRange, BaselineParameters,
};

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
    pub baseline: BaselineParameters,
    pub dsfb_alpha_range: AlphaRange,
    pub trust_map_frame_offset: usize,
    pub comparison_frame_offset: usize,
    pub demo_b_reference_spp: usize,
    pub demo_b_uniform_spp: usize,
    pub demo_b_min_spp: usize,
    pub demo_b_max_spp: usize,
}

impl Default for DemoConfig {
    fn default() -> Self {
        let host = host_realistic_parameters();
        Self {
            scene: SceneConfig::default(),
            baseline: baseline_parameters(),
            dsfb_alpha_range: host.alpha_range,
            trust_map_frame_offset: 0,
            comparison_frame_offset: 2,
            demo_b_reference_spp: 64,
            demo_b_uniform_spp: 2,
            demo_b_min_spp: 1,
            demo_b_max_spp: 12,
        }
    }
}
