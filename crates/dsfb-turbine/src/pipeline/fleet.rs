//! Fleet-level evaluation orchestrator.
//!
//! Runs DSFB pipeline across all engines in a dataset and
//! computes aggregate metrics.

use crate::dataset::cmapss::CmapssDataset;
use crate::pipeline::engine_eval::{evaluate_engine, EngineEvalResult};
use crate::pipeline::metrics::{compute_fleet_metrics, FleetMetrics};
use crate::core::config::DsfbConfig;
use crate::core::channels::{ChannelId, INFORMATIVE_CHANNELS_FD001};

/// Runs the complete fleet evaluation on a C-MAPSS dataset.
pub fn evaluate_fleet(
    dataset: &CmapssDataset,
    config: &DsfbConfig,
    channels: &[ChannelId],
) -> (Vec<EngineEvalResult>, FleetMetrics) {
    let units = dataset.units();
    let mut results = Vec::with_capacity(units.len());

    for &unit in &units {
        // Extract channel data for this unit
        let channel_data: Vec<(ChannelId, Vec<f64>)> = channels.iter()
            .map(|&ch| {
                let vals = dataset.channel_for_unit(unit, ch.cmapss_sensor_index());
                (ch, vals)
            })
            .filter(|(_, vals)| !vals.is_empty())
            .collect();

        if channel_data.is_empty() {
            continue;
        }

        let result = evaluate_engine(unit, &channel_data, config);
        results.push(result);
    }

    let metrics = compute_fleet_metrics(&results);
    (results, metrics)
}

/// Runs the default FD001 evaluation with default configuration.
pub fn evaluate_fd001(dataset: &CmapssDataset) -> (Vec<EngineEvalResult>, FleetMetrics) {
    let config = DsfbConfig::cmapss_fd001_default();
    evaluate_fleet(dataset, &config, INFORMATIVE_CHANNELS_FD001)
}
