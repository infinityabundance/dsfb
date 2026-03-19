pub fn grouped_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    let coordinated = 0.03 + 0.00165 * t + 0.012 * (0.05 * t).sin();
    match channel {
        0 => coordinated,
        1 => coordinated * 0.92 + 0.008 * (0.07 * t + 0.4).sin(),
        2 => coordinated * 0.85 + 0.009 * (0.05 * t + 0.8).cos(),
        _ => 0.05 + 0.0004 * t + 0.014 * (0.09 * t + 0.2).sin(),
    }
}
