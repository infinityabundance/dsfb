//! IMU-style synthetic residual constructions for A-PNT-facing examples.

/// Synthetic three-axis IMU residual story with a GPS-denied blackout window, slow thermal drift
/// on one axis, background noise on the others, and one abrupt mode-switch event.
pub fn imu_thermal_drift_gps_denied_residual(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    let blackout_start = 60.0;
    let drift_onset = 75.0;
    let mode_switch = 120.0;
    let blackout_scale = if t < blackout_start { 0.35 } else { 1.0 };
    let thermal_bias = if t < drift_onset {
        0.0
    } else {
        let tau = t - drift_onset;
        0.00042 * tau + 0.000006 * tau * tau
    };
    let switch_pulse = 0.018 * (-((t - mode_switch) / 3.0).powi(2)).exp();

    match channel {
        // X axis accumulates the slow thermal drift plus the mode-switch pulse.
        0 => blackout_scale * (0.0008 * (0.12 * t).sin()) + thermal_bias + switch_pulse,
        // Y axis stays near bounded jitter with a much smaller switch coupling.
        1 => blackout_scale * (0.0012 * (0.15 * t + 0.5).cos()) + 0.25 * switch_pulse,
        // Z axis stays mostly noise-like with opposite-signed switch coupling.
        _ => blackout_scale * (0.0010 * (0.10 * t + 1.0).sin()) - 0.15 * switch_pulse,
    }
}
