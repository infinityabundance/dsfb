use crate::sim::diagnostics::MeasurementFrame;
use crate::sim::state::BenchConfig;

pub fn apply_impulse_corruption(
    cfg: &BenchConfig,
    frame: &mut MeasurementFrame,
    step: usize,
) -> bool {
    let start = cfg.corruption_start;
    let end = cfg.corruption_start + cfg.corruption_duration;
    if step < start || step >= end {
        return false;
    }

    let local = (step - start) as f64;
    let duration = cfg.corruption_duration as f64;

    // Smooth pulse envelope in [0, 1], peaking at the middle of the window.
    let phase = std::f64::consts::PI * (local / duration);
    let envelope = phase.sin().abs();

    let group = cfg.corruption_group;
    let channel = cfg.corruption_channel;
    frame.y_groups[group][channel] += cfg.corruption_amplitude * envelope;

    true
}
