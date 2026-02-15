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

    // Smooth pulse envelope sampled at bin centers so a 1-step window
    // still receives full corruption amplitude.
    let phase = std::f64::consts::PI * ((local + 0.5) / duration);
    let envelope = phase.sin().abs();

    let group = cfg.corruption_group;
    let channel = cfg.corruption_channel;
    frame.y_groups[group][channel] += cfg.corruption_amplitude * envelope;

    true
}
