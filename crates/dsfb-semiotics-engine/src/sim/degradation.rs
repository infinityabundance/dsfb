pub fn gradual_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    match channel {
        0 => 0.04 + 0.00155 * t + 0.015 * (0.05 * t).sin(),
        1 => 0.03 + 0.00105 * t + 0.012 * (0.04 * t + 0.6).sin(),
        _ => 0.02 + 0.0006 * t + 0.009 * (0.06 * t + 1.2).cos(),
    }
}

pub fn curvature_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    let onset = (t - 90.0).max(0.0);
    let base = 0.02 + 0.0006 * t + 0.00004 * onset * onset;
    match channel {
        0 => base,
        1 => 0.5 * base + 0.01 * (0.07 * t).sin(),
        _ => 0.3 * base + 0.012 * (0.04 * t + 0.8).cos(),
    }
}

pub fn inward_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    let base = 0.31 - 0.00085 * t + 0.012 * (0.06 * t).sin();
    match channel {
        0 => base,
        1 => 0.62 * base + 0.01 * (0.03 * t + 0.8).sin(),
        _ => 0.5 * base + 0.008 * (0.05 * t + 1.1).cos(),
    }
}

pub fn regime_switched_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    let nominal = 0.08 + 0.0012 * t;
    let shifted = 0.18 + 0.0033 * (t - 90.0).max(0.0);
    let base = if step < 90 { nominal } else { shifted };
    match channel {
        0 => base,
        1 => 0.55 * base + 0.01 * (0.08 * t).sin(),
        _ => 0.35 * base + 0.009 * (0.05 * t + 1.4).cos(),
    }
}
