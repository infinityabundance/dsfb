pub fn update_envelope(previous: f64, residual: f64, availability_penalty: f64, decay: f64) -> f64 {
    let next = decay * previous + (1.0 - decay) * (residual.abs() + availability_penalty);
    next.max(0.0)
}

pub fn trust_from_envelope(envelope: f64, beta: f64, ceiling: f64) -> f64 {
    (ceiling * (-beta * envelope.max(0.0)).exp()).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{trust_from_envelope, update_envelope};

    #[test]
    fn trust_values_remain_bounded() {
        let mut envelope = 0.0;
        for step in 0..128 {
            envelope = update_envelope(envelope, step as f64 * 0.11, 0.03, 0.82);
            let trust = trust_from_envelope(envelope, 1.3, 0.97);
            assert!((0.0..=1.0).contains(&trust));
        }
    }
}
