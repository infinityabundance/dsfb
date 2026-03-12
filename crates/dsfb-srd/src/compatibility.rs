use crate::event::StructuralEvent;

pub fn channel_compatible(
    source: &StructuralEvent,
    target: &StructuralEvent,
    n_channels: usize,
) -> bool {
    let channel_delta = source.channel_id.abs_diff(target.channel_id);
    channel_delta == 0 || channel_delta == 1 || channel_delta + 1 == n_channels
}

pub fn structurally_similar(source: &StructuralEvent, target: &StructuralEvent) -> bool {
    let latent_gap = (source.latent_state - target.latent_state).abs();
    let observed_gap = (source.observed_value - target.observed_value).abs();
    let trust_gap = (source.trust - target.trust).abs();
    let residual_gap = (source.residual - target.residual).abs();
    let regime_bonus = if source.regime_label == target.regime_label {
        0.20
    } else {
        0.05
    };

    let similarity_score =
        1.35 - 0.45 * latent_gap - 0.30 * observed_gap - 0.25 * trust_gap - 0.35 * residual_gap
            + regime_bonus;

    similarity_score >= 0.65
}

pub fn compatible(source: &StructuralEvent, target: &StructuralEvent, n_channels: usize) -> bool {
    channel_compatible(source, target, n_channels) && structurally_similar(source, target)
}
