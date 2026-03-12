use std::f64::consts::PI;

use crate::config::SimulationConfig;
use crate::event::{RegimeLabel, StructuralEvent};
use crate::trust::{compute_trust, update_envelope};

pub fn generate_events(config: &SimulationConfig) -> Vec<StructuralEvent> {
    let mut envelopes = vec![0.0; config.n_channels];
    let mut events = Vec::with_capacity(config.n_events);

    for event_id in 0..config.n_events {
        let channel_id = event_id % config.n_channels;
        let time_index = event_id;
        let time = event_id as f64;
        let regime_label = regime_for_event(event_id, config);
        let latent_state = latent_state(time, channel_id, config.n_channels);
        let predicted_value = predicted_value(latent_state, time, channel_id);
        let observed_value = observed_value(
            predicted_value,
            time,
            channel_id,
            event_id,
            config,
            regime_label,
        );
        let residual = (observed_value - predicted_value).abs();
        let envelope = update_envelope(envelopes[channel_id], residual, config.envelope_decay);
        envelopes[channel_id] = envelope;
        let trust = compute_trust(envelope, config.beta);

        events.push(StructuralEvent {
            event_id,
            time_index,
            channel_id,
            latent_state,
            predicted_value,
            observed_value,
            residual,
            envelope,
            trust,
            regime_label,
        });
    }

    events
}

pub fn regime_for_event(event_id: usize, config: &SimulationConfig) -> RegimeLabel {
    let degradation_start = config.shock_start.saturating_sub(pre_shock_span(config));
    let recovery_end = config
        .shock_end
        .saturating_add(recovery_span(config))
        .min(config.n_events);

    if event_id < degradation_start {
        RegimeLabel::Baseline
    } else if event_id < config.shock_start {
        RegimeLabel::Degradation
    } else if event_id < config.shock_end {
        RegimeLabel::Shock
    } else if event_id < recovery_end {
        RegimeLabel::Recovery
    } else {
        RegimeLabel::Baseline
    }
}

fn pre_shock_span(config: &SimulationConfig) -> usize {
    (config.causal_window * 2)
        .max(32)
        .min(config.n_events.saturating_sub(1))
}

fn recovery_span(config: &SimulationConfig) -> usize {
    (config.causal_window * 4).max(64)
}

fn latent_state(time: f64, channel_id: usize, n_channels: usize) -> f64 {
    let channel = channel_id as f64;
    let centered_channel = channel - (n_channels.saturating_sub(1) as f64 / 2.0);
    (0.047 * time + 0.61 * channel).sin()
        + 0.35 * (0.011 * time + 0.23 * channel).sin()
        + 0.18 * centered_channel
        + 0.0009 * time
}

fn predicted_value(latent_state: f64, time: f64, channel_id: usize) -> f64 {
    let channel = channel_id as f64;
    latent_state + 0.08 * (0.19 * latent_state + 0.07 * time + 0.31 * channel).cos()
}

fn observed_value(
    predicted_value: f64,
    time: f64,
    channel_id: usize,
    event_id: usize,
    config: &SimulationConfig,
    regime_label: RegimeLabel,
) -> f64 {
    let channel = channel_id as f64;
    let sign = if (event_id + channel_id) % 2 == 0 {
        1.0
    } else {
        -1.0
    };
    let baseline_distortion = 0.012
        * ((0.137 * time + 0.43 * channel).sin() + 0.5 * (0.071 * time - 0.29 * channel).cos());
    let oscillation = 0.55
        + 0.25 * (0.09 * time + 0.21 * channel).sin().abs()
        + 0.20 * (0.041 * time + 0.37 * channel).cos().abs();

    let regime_distortion = match regime_label {
        RegimeLabel::Baseline => 0.0,
        RegimeLabel::Degradation => {
            let start = config.shock_start.saturating_sub(pre_shock_span(config));
            let span = config.shock_start.saturating_sub(start).max(1) as f64;
            let progress = event_id.saturating_sub(start) as f64 / span;
            0.10 * progress * oscillation
        }
        RegimeLabel::Shock => {
            let span = config.shock_end.saturating_sub(config.shock_start).max(1) as f64;
            let progress = event_id.saturating_sub(config.shock_start) as f64 / span;
            let crest = 0.75 + 0.25 * (2.0 * PI * progress).sin().abs();
            0.36 * oscillation * crest
        }
        RegimeLabel::Recovery => {
            let elapsed = event_id.saturating_sub(config.shock_end) as f64;
            let decay_span = recovery_span(config) as f64 / 2.5;
            let decay = (-elapsed / decay_span).exp();
            0.14 * oscillation * decay
        }
    };

    predicted_value + baseline_distortion + sign * regime_distortion
}
