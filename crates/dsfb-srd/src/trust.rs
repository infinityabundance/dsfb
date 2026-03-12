pub fn update_envelope(previous_envelope: f64, residual: f64, decay: f64) -> f64 {
    (previous_envelope * decay).max(residual)
}

pub fn compute_trust(envelope: f64, beta: f64) -> f64 {
    (-beta * envelope).exp().clamp(0.0, 1.0)
}
