pub fn nominal_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    match channel {
        0 => 0.04 * (0.08 * t).sin(),
        1 => 0.05 * (0.05 * t + 0.4).cos(),
        _ => 0.03 * (0.04 * t + 1.2).sin(),
    }
}

pub fn abrupt_event_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    let pulse = 0.55 * (-((t - 120.0) / 6.0).powi(2)).exp();
    match channel {
        0 => 0.03 * (0.03 * t).sin() + pulse,
        1 => 0.02 * (0.04 * t + 0.7).cos() + 0.4 * pulse,
        _ => 0.02 * (0.05 * t + 0.2).sin() - 0.2 * pulse,
    }
}

pub fn oscillatory_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    match channel {
        0 => 0.16 * (0.18 * t).sin(),
        1 => 0.14 * (0.22 * t + 0.3).sin(),
        _ => 0.12 * (0.16 * t + 1.1).cos(),
    }
}

pub fn deterministic_noise(step: usize, channel: usize, seed: u64) -> f64 {
    let a = 12.9898 + channel as f64 * 1.347;
    let b = 78.233 + seed as f64 * 0.001;
    let phase = ((step as f64 + 1.0) * a + b).sin();
    0.018 * (43758.5453 * phase).sin()
}
