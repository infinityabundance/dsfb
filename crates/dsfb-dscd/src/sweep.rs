use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use csv::Writer;
use dsfb::sim::SimConfig;
use dsfb_add::SimulationConfig;

use crate::config::{DscdSweepConfig, OutputPaths};
use crate::graph::{add_trust_gated_edge, reachable_from, DscdEdge, DscdGraph, Event, EventId};
use crate::integrations::{
    compute_structural_growth_for_dscd, generate_dscd_events_from_dsfb, DscdObserverSample,
};

#[derive(Debug, Clone)]
pub struct ThresholdRecord {
    pub tau: f64,
    pub expansion_ratio: f64,
    pub reachable_size: usize,
    pub s_infty: Option<f64>,
}

pub fn build_graph_from_samples(
    events: &[Event],
    samples: &[DscdObserverSample],
    trust_threshold: f64,
) -> DscdGraph {
    let mut graph = DscdGraph::default();
    for event in events {
        graph.add_event(event.clone());
    }

    let mut by_observer: BTreeMap<u32, Vec<&DscdObserverSample>> = BTreeMap::new();
    for sample in samples {
        by_observer
            .entry(sample.observer_id)
            .or_default()
            .push(sample);
    }

    for (observer_id, samples_for_observer) in &mut by_observer {
        samples_for_observer.sort_by_key(|sample| sample.event_id);
        for pair in samples_for_observer.windows(2) {
            let from = EventId(pair[0].event_id);
            let to = EventId(pair[1].event_id);
            add_trust_gated_edge(
                &mut graph,
                from,
                to,
                *observer_id,
                pair[1].trust,
                trust_threshold,
            );
        }
    }

    graph
}

pub fn run_trust_threshold_sweep(
    cfg: &DscdSweepConfig,
    dsfb_cfg: &SimConfig,
    add_cfg: &SimulationConfig,
    output_paths: &OutputPaths,
) -> Result<Vec<ThresholdRecord>> {
    cfg.validate()?;

    report_progress(2, "generating DSCD events from DSFB");
    let event_batch = generate_dscd_events_from_dsfb(dsfb_cfg, cfg.dsfb_params, cfg.num_events)?;
    report_progress(15, format!("generated {} events", event_batch.events.len()));
    report_progress(20, "computing structural growth baseline");
    let growth = compute_structural_growth_for_dscd(add_cfg)?;
    let tau_grid = cfg.tau_grid();
    let tau_total = tau_grid.len();
    report_progress(25, format!("prepared tau grid with {tau_total} steps"));
    let start = event_batch.events.first().map(|event| event.id);
    let first_tau = tau_grid[0];

    let base_graph = build_graph_from_samples(
        &event_batch.events,
        &event_batch.observer_samples,
        first_tau,
    );
    report_progress(28, "writing graph event and edge snapshots");
    write_graph_events_csv(
        &output_paths.run_dir.join("graph_events.csv"),
        &event_batch.events,
    )?;
    write_graph_edges_csv(
        &output_paths.run_dir.join("graph_edges.csv"),
        first_tau,
        &base_graph.edges,
    )?;

    let mut records = Vec::with_capacity(tau_grid.len());
    let mut last_reported = 29_usize;
    report_progress(30, "running trust-threshold sweep");
    for (idx, tau) in tau_grid.into_iter().enumerate() {
        let graph =
            build_graph_from_samples(&event_batch.events, &event_batch.observer_samples, tau);
        let reachable_size = start
            .map(|start_event| reachable_from(&graph, start_event, cfg.max_depth).len())
            .unwrap_or(0);
        let expansion_ratio = if event_batch.events.is_empty() {
            0.0
        } else {
            reachable_size as f64 / event_batch.events.len() as f64
        };

        records.push(ThresholdRecord {
            tau,
            expansion_ratio,
            reachable_size,
            s_infty: Some(growth.s_infty),
        });

        let progress = 30 + ((idx + 1) * 65) / tau_total;
        if progress > last_reported {
            report_progress(progress, format!("tau step {}/{}", idx + 1, tau_total));
            last_reported = progress;
        }
    }

    report_progress(97, "writing threshold_sweep.csv");
    write_threshold_sweep_csv(&output_paths.run_dir.join("threshold_sweep.csv"), &records)?;
    report_progress(100, "DSCD sweep complete");

    Ok(records)
}

fn report_progress(percent: usize, message: impl AsRef<str>) {
    let pct = percent.min(100);
    eprintln!("[{pct:>3}%] {}", message.as_ref());
}

fn write_graph_events_csv(path: &Path, events: &[Event]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(["event_id", "timestamp", "structural_tag"])?;

    for event in events {
        writer.write_record([
            event.id.0.to_string(),
            event
                .timestamp
                .map(|value| value.to_string())
                .unwrap_or_default(),
            event
                .structural_tag
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn write_graph_edges_csv(path: &Path, tau_threshold: f64, edges: &[DscdEdge]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "from_event_id",
        "to_event_id",
        "observer_id",
        "trust_value",
        "tau_threshold",
    ])?;

    for edge in edges {
        writer.write_record([
            edge.from.0.to_string(),
            edge.to.0.to_string(),
            edge.observer_id.to_string(),
            edge.trust_value.to_string(),
            tau_threshold.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn write_threshold_sweep_csv(path: &Path, records: &[ThresholdRecord]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(["tau", "expansion_ratio", "reachable_size", "s_infty"])?;

    for record in records {
        writer.write_record([
            record.tau.to_string(),
            record.expansion_ratio.to_string(),
            record.reachable_size.to_string(),
            record
                .s_infty
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toy_events() -> Vec<Event> {
        (0..5_u64)
            .map(|raw_id| Event {
                id: EventId(raw_id),
                timestamp: Some(raw_id as f64),
                structural_tag: Some(raw_id as f64 * 0.1),
            })
            .collect()
    }

    fn toy_samples() -> Vec<DscdObserverSample> {
        vec![
            DscdObserverSample {
                event_id: 0,
                observer_id: 0,
                trust: 0.1,
                residual_summary: 0.0,
            },
            DscdObserverSample {
                event_id: 1,
                observer_id: 0,
                trust: 0.2,
                residual_summary: 0.0,
            },
            DscdObserverSample {
                event_id: 2,
                observer_id: 0,
                trust: 0.4,
                residual_summary: 0.0,
            },
            DscdObserverSample {
                event_id: 3,
                observer_id: 0,
                trust: 0.7,
                residual_summary: 0.0,
            },
            DscdObserverSample {
                event_id: 4,
                observer_id: 0,
                trust: 0.9,
                residual_summary: 0.0,
            },
        ]
    }

    #[test]
    fn stricter_thresholds_do_not_increase_reachability() {
        let events = toy_events();
        let samples = toy_samples();
        let taus = [0.0, 0.25, 0.5, 0.75];

        let ratios: Vec<f64> = taus
            .iter()
            .map(|tau| {
                let graph = build_graph_from_samples(&events, &samples, *tau);
                reachable_from(&graph, EventId(0), None).len() as f64 / events.len() as f64
            })
            .collect();

        assert!(ratios.windows(2).all(|pair| pair[0] + 1.0e-12 >= pair[1]));
    }
}
