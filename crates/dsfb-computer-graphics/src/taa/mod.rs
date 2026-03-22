use crate::frame::ImageFrame;
use crate::scene::SceneSequence;

#[derive(Clone, Debug)]
pub struct TaaRun {
    pub resolved_frames: Vec<ImageFrame>,
    pub reprojected_history_frames: Vec<ImageFrame>,
}

pub fn run_fixed_alpha(sequence: &SceneSequence, alpha: f32) -> TaaRun {
    let mut resolved_frames = Vec::with_capacity(sequence.frames.len());
    let mut reprojected_history_frames = Vec::with_capacity(sequence.frames.len());

    for (frame_index, scene_frame) in sequence.frames.iter().enumerate() {
        if frame_index == 0 {
            resolved_frames.push(scene_frame.ground_truth.clone());
            reprojected_history_frames.push(scene_frame.ground_truth.clone());
            continue;
        }

        let previous_resolved = &resolved_frames[frame_index - 1];
        let mut reprojected = ImageFrame::new(
            scene_frame.ground_truth.width(),
            scene_frame.ground_truth.height(),
        );
        let mut resolved = ImageFrame::new(
            scene_frame.ground_truth.width(),
            scene_frame.ground_truth.height(),
        );

        for y in 0..scene_frame.ground_truth.height() {
            for x in 0..scene_frame.ground_truth.width() {
                let motion = scene_frame.motion[y * scene_frame.ground_truth.width() + x];
                let history = previous_resolved
                    .sample_clamped(x as i32 + motion.to_prev_x, y as i32 + motion.to_prev_y);
                let current = scene_frame.ground_truth.get(x, y);
                reprojected.set(x, y, history);
                resolved.set(x, y, history.lerp(current, alpha));
            }
        }

        reprojected_history_frames.push(reprojected);
        resolved_frames.push(resolved);
    }

    TaaRun {
        resolved_frames,
        reprojected_history_frames,
    }
}
