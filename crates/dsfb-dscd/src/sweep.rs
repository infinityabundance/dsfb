use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use csv::{StringRecord, Writer};
use dsfb::sim::SimConfig;
use dsfb_add::SimulationConfig;

use crate::config::{DscdScalingConfig, DscdSweepConfig, OutputPaths};
use crate::graph::{
    add_trust_gated_edge_with_provenance, reachable_from, DscdEdge, DscdGraph, Event, EventId,
};
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

#[derive(Debug, Clone, Copy)]
struct ScalingSummaryRow {
    n: usize,
    tau_star: f64,
    width_0_1_to_0_9: f64,
    max_derivative: f64,
}

#[derive(Debug, Clone, Copy, Default)]
struct ThresholdEvalOptions {
    start: Option<EventId>,
    max_depth: Option<usize>,
    s_infty: Option<f64>,
    progress_start: Option<usize>,
    progress_end: Option<usize>,
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
            add_trust_gated_edge_with_provenance(
                &mut graph,
                from,
                to,
                *observer_id,
                pair[1].trust,
                trust_threshold,
                pair[0].rewrite_rule_id,
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

    report_progress(28, "writing graph event and edge snapshots");
    let base_graph = build_graph_from_samples(
        &event_batch.events,
        &event_batch.observer_samples,
        first_tau,
    );
    write_graph_events_csv(
        &output_paths.run_dir.join("graph_events.csv"),
        &event_batch.events,
        &event_batch.observer_samples,
    )?;
    write_graph_edges_csv(
        &output_paths.run_dir.join("graph_edges.csv"),
        first_tau,
        &base_graph.edges,
    )?;

    if let Some(edge) = base_graph.edges.first() {
        let _ = write_edge_provenance_csv(
            &output_paths.run_dir,
            edge.edge_id,
            &base_graph.edges,
            &event_batch.observer_samples,
        )?;
    }

    report_progress(30, "running trust-threshold sweep");
    let records = compute_threshold_records(
        &event_batch.events,
        &event_batch.observer_samples,
        &tau_grid,
        ThresholdEvalOptions {
            start,
            max_depth: cfg.max_depth,
            s_infty: Some(growth.s_infty),
            progress_start: Some(30),
            progress_end: Some(95),
        },
    );

    report_progress(97, "writing threshold_sweep.csv");
    write_threshold_sweep_csv(&output_paths.run_dir.join("threshold_sweep.csv"), &records)?;

    report_progress(98, "writing finite_size_scaling.csv");
    let transition_width = compute_width_0_1_to_0_9(&records);
    let max_derivative = compute_max_derivative(&records).unwrap_or(0.0);
    write_finite_size_scaling_csv(
        &output_paths.run_dir.join("finite_size_scaling.csv"),
        cfg.num_events,
        transition_width,
        max_derivative,
    )?;

    report_progress(100, "DSCD sweep complete");
    Ok(records)
}

/// Run deterministic finite-size scaling for the DSCD threshold transition.
///
/// This routine is fully deterministic (no randomness) and is used to build the
/// scaling outputs that support DSCD threshold-sharpening analysis (Theorem 4).
pub fn run_threshold_scaling(config: &DscdScalingConfig, output_dir: &Path) -> Result<()> {
    config.validate()?;
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create scaling output dir {}",
            output_dir.display()
        )
    })?;

    let mut summary_rows = Vec::with_capacity(config.event_counts.len());
    let representative_n = config.event_counts.last().copied().unwrap_or(0);
    for (idx, &n) in config.event_counts.iter().enumerate() {
        report_progress(
            5 + ((idx + 1) * 60) / config.event_counts.len(),
            format!(
                "scaling sweep for N={n} ({}/{})",
                idx + 1,
                config.event_counts.len()
            ),
        );

        let dsfb_cfg = SimConfig {
            steps: n,
            ..SimConfig::default()
        };
        let event_batch = generate_dscd_events_from_dsfb(&dsfb_cfg, config.dsfb_params, n)?;
        let base_graph = build_graph_from_samples(
            &event_batch.events,
            &event_batch.observer_samples,
            config.tau_grid[0],
        );

        let records = compute_threshold_records(
            &event_batch.events,
            &event_batch.observer_samples,
            &config.tau_grid,
            ThresholdEvalOptions {
                start: Some(config.initial_event),
                max_depth: Some(config.max_path_length),
                s_infty: None,
                progress_start: None,
                progress_end: None,
            },
        );

        write_threshold_curve_csv(
            &output_dir.join(format!("threshold_curve_N_{n}.csv")),
            &records,
        )?;

        let row = ScalingSummaryRow {
            n,
            tau_star: compute_tau_star(&records, config.critical_fraction),
            width_0_1_to_0_9: compute_width_0_1_to_0_9(&records),
            max_derivative: compute_max_derivative(&records).unwrap_or(0.0),
        };
        summary_rows.push(row);

        if n == representative_n {
            write_graph_events_csv(
                &output_dir.join("graph_events.csv"),
                &event_batch.events,
                &event_batch.observer_samples,
            )?;
            write_graph_edges_csv(
                &output_dir.join("graph_edges.csv"),
                config.tau_grid[0],
                &base_graph.edges,
            )?;
            if let Some(edge) = base_graph.edges.first() {
                let _ = write_edge_provenance_csv(
                    output_dir,
                    edge.edge_id,
                    &base_graph.edges,
                    &event_batch.observer_samples,
                )?;
            }
        }
    }

    report_progress(92, "writing threshold_scaling_summary.csv");
    write_threshold_scaling_summary_csv(
        &output_dir.join("threshold_scaling_summary.csv"),
        &summary_rows,
    )?;
    write_finite_size_scaling_series_csv(
        &output_dir.join("finite_size_scaling.csv"),
        &summary_rows,
    )?;
    report_progress(100, "threshold scaling complete");

    Ok(())
}

/// Export full provenance for an edge identified by `edge_id`.
///
/// The output is written to `edge_provenance_<edge_id>.csv` in `run_dir`.
pub fn export_edge_provenance_by_edge_id(run_dir: &Path, edge_id: u64) -> Result<PathBuf> {
    let edge_row = load_edge_row_by_id(run_dir, edge_id)?;
    let source_event_row =
        load_source_event_row(run_dir, edge_row.source_event_id, edge_row.observer_id)?;
    write_edge_provenance_row(run_dir, edge_row, source_event_row)
}

/// Export full provenance for an edge identified by `(source_event_id, target_event_id)`.
pub fn export_edge_provenance_by_endpoints(
    run_dir: &Path,
    source_event_id: EventId,
    target_event_id: EventId,
) -> Result<PathBuf> {
    let edge_row = load_edge_row_by_endpoints(run_dir, source_event_id.0, target_event_id.0)?;
    let source_event_row =
        load_source_event_row(run_dir, edge_row.source_event_id, edge_row.observer_id)?;
    write_edge_provenance_row(run_dir, edge_row, source_event_row)
}

fn compute_threshold_records(
    events: &[Event],
    samples: &[DscdObserverSample],
    tau_grid: &[f64],
    options: ThresholdEvalOptions,
) -> Vec<ThresholdRecord> {
    let mut records = Vec::with_capacity(tau_grid.len());
    let mut last_reported = options.progress_start.unwrap_or(0).saturating_sub(1);

    for (idx, tau) in tau_grid.iter().copied().enumerate() {
        let graph = build_graph_from_samples(events, samples, tau);
        let reachable_size = options
            .start
            .map(|start_event| reachable_from(&graph, start_event, options.max_depth).len())
            .unwrap_or(0);
        let expansion_ratio = if events.is_empty() {
            0.0
        } else {
            reachable_size as f64 / events.len() as f64
        };

        records.push(ThresholdRecord {
            tau,
            expansion_ratio,
            reachable_size,
            s_infty: options.s_infty,
        });

        if let (Some(start_pct), Some(end_pct)) = (options.progress_start, options.progress_end) {
            let span = end_pct.saturating_sub(start_pct);
            let progress = start_pct + ((idx + 1) * span) / tau_grid.len().max(1);
            if progress > last_reported {
                report_progress(progress, format!("tau step {}/{}", idx + 1, tau_grid.len()));
                last_reported = progress;
            }
        }
    }

    records
}

fn report_progress(percent: usize, message: impl AsRef<str>) {
    let pct = percent.min(100);
    eprintln!("[{pct:>3}%] {}", message.as_ref());
}

fn write_graph_events_csv(
    path: &Path,
    events: &[Event],
    samples: &[DscdObserverSample],
) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "event_id",
        "time_index",
        "observer_id",
        "residual_state",
        "rewrite_rule_id",
        "trust_value",
        "residual_summary",
        "rewrite_rule_label",
        "trust_profile",
        "envelope_ok",
        "timestamp",
        "structural_tag",
    ])?;

    let event_by_id: HashMap<u64, &Event> =
        events.iter().map(|event| (event.id.0, event)).collect();

    for sample in samples {
        let event_meta = event_by_id.get(&sample.event_id).copied();
        writer.write_record([
            sample.event_id.to_string(),
            sample.time_index.to_string(),
            sample.observer_id.to_string(),
            sample.residual_state.as_str().to_string(),
            sample.rewrite_rule_id.to_string(),
            sample.trust.to_string(),
            sample.residual_summary.to_string(),
            sample.rewrite_rule_label.to_string(),
            sample.trust_profile.as_str().to_string(),
            sample.envelope_ok.to_string(),
            event_meta
                .and_then(|event| event.timestamp)
                .map(|value| value.to_string())
                .unwrap_or_default(),
            event_meta
                .and_then(|event| event.structural_tag)
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
        "edge_id",
        "source_event_id",
        "target_event_id",
        "from_event_id",
        "to_event_id",
        "observer_id",
        "trust_at_creation",
        "trust_value",
        "rewrite_rule_at_source",
        "tau_threshold",
    ])?;

    for edge in edges {
        writer.write_record([
            edge.edge_id.to_string(),
            edge.from.0.to_string(),
            edge.to.0.to_string(),
            edge.from.0.to_string(),
            edge.to.0.to_string(),
            edge.observer_id.to_string(),
            edge.trust_at_creation.to_string(),
            edge.trust_value.to_string(),
            edge.rewrite_rule_at_source.to_string(),
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

fn write_threshold_curve_csv(path: &Path, records: &[ThresholdRecord]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(["tau", "expansion_ratio"])?;
    for record in records {
        writer.write_record([record.tau.to_string(), record.expansion_ratio.to_string()])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_threshold_scaling_summary_csv(path: &Path, rows: &[ScalingSummaryRow]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(["N", "tau_star", "width_0_1_to_0_9", "max_derivative"])?;
    for row in rows {
        writer.write_record([
            row.n.to_string(),
            row.tau_star.to_string(),
            row.width_0_1_to_0_9.to_string(),
            row.max_derivative.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn compute_tau_star(records: &[ThresholdRecord], critical_fraction: f64) -> f64 {
    interpolate_tau_at_or_below(records, critical_fraction)
        .or_else(|| records.last().map(|record| record.tau))
        .unwrap_or(0.0)
}

fn compute_width_0_1_to_0_9(records: &[ThresholdRecord]) -> f64 {
    let tau_0_9 = interpolate_tau_at_or_below(records, 0.9)
        .or_else(|| records.first().map(|record| record.tau))
        .unwrap_or(0.0);
    let tau_0_1 = interpolate_tau_at_or_below(records, 0.1)
        .or_else(|| records.last().map(|record| record.tau))
        .unwrap_or(tau_0_9);
    (tau_0_1 - tau_0_9).max(0.0)
}

fn interpolate_tau_at_or_below(records: &[ThresholdRecord], threshold: f64) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    if records[0].expansion_ratio <= threshold {
        return Some(records[0].tau);
    }

    for pair in records.windows(2) {
        let prev = &pair[0];
        let curr = &pair[1];
        if curr.expansion_ratio <= threshold {
            let drho = curr.expansion_ratio - prev.expansion_ratio;
            if drho.abs() <= f64::EPSILON {
                return Some(curr.tau);
            }
            let alpha = (threshold - prev.expansion_ratio) / drho;
            return Some(prev.tau + alpha * (curr.tau - prev.tau));
        }
    }

    None
}

fn compute_max_derivative(records: &[ThresholdRecord]) -> Option<f64> {
    let mut max_derivative: Option<f64> = None;
    for pair in records.windows(2) {
        let dtau = pair[1].tau - pair[0].tau;
        if dtau.abs() <= f64::EPSILON {
            continue;
        }

        let derivative = ((pair[1].expansion_ratio - pair[0].expansion_ratio) / dtau).abs();
        if !derivative.is_finite() {
            continue;
        }

        max_derivative = Some(match max_derivative {
            Some(current) => current.max(derivative),
            None => derivative,
        });
    }

    max_derivative
}

fn write_finite_size_scaling_csv(
    path: &Path,
    num_events: usize,
    transition_width: f64,
    max_derivative: f64,
) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "num_events",
        "transition_width",
        "width_0_1_to_0_9",
        "max_derivative",
    ])?;
    writer.write_record([
        num_events.to_string(),
        transition_width.to_string(),
        transition_width.to_string(),
        max_derivative.to_string(),
    ])?;
    writer.flush()?;
    Ok(())
}

fn write_finite_size_scaling_series_csv(path: &Path, rows: &[ScalingSummaryRow]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "num_events",
        "transition_width",
        "width_0_1_to_0_9",
        "max_derivative",
    ])?;
    for row in rows {
        writer.write_record([
            row.n.to_string(),
            row.width_0_1_to_0_9.to_string(),
            row.width_0_1_to_0_9.to_string(),
            row.max_derivative.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct EdgeCsvRow {
    edge_id: u64,
    source_event_id: u64,
    target_event_id: u64,
    observer_id: u32,
    trust_at_creation: f64,
    rewrite_rule_at_source: u32,
}

#[derive(Debug, Clone)]
struct EventCsvRow {
    time_index: String,
    residual_state: String,
    rewrite_rule_id: String,
    residual_summary: String,
    trust_value: String,
}

fn write_edge_provenance_csv(
    run_dir: &Path,
    edge_id: u64,
    edges: &[DscdEdge],
    samples: &[DscdObserverSample],
) -> Result<PathBuf> {
    let edge = edges
        .iter()
        .find(|edge| edge.edge_id == edge_id)
        .ok_or_else(|| anyhow!("edge_id {} not found", edge_id))?;

    let source_sample = samples
        .iter()
        .find(|sample| sample.event_id == edge.from.0 && sample.observer_id == edge.observer_id);

    let path = run_dir.join(format!("edge_provenance_{}.csv", edge_id));
    let mut writer = Writer::from_path(&path)?;
    writer.write_record([
        "edge_id",
        "source_event_id",
        "target_event_id",
        "observer_id",
        "trust_at_creation",
        "rewrite_rule_at_source",
        "time_index",
        "residual_state",
        "rewrite_rule_id",
        "residual_summary",
        "trust_value",
    ])?;

    writer.write_record([
        edge.edge_id.to_string(),
        edge.from.0.to_string(),
        edge.to.0.to_string(),
        edge.observer_id.to_string(),
        edge.trust_at_creation.to_string(),
        edge.rewrite_rule_at_source.to_string(),
        source_sample
            .map(|sample| sample.time_index.to_string())
            .unwrap_or_default(),
        source_sample
            .map(|sample| sample.residual_state.as_str().to_string())
            .unwrap_or_default(),
        source_sample
            .map(|sample| sample.rewrite_rule_id.to_string())
            .unwrap_or_default(),
        source_sample
            .map(|sample| sample.residual_summary.to_string())
            .unwrap_or_default(),
        source_sample
            .map(|sample| sample.trust.to_string())
            .unwrap_or_default(),
    ])?;

    writer.flush()?;
    Ok(path)
}

fn load_edge_row_by_id(run_dir: &Path, edge_id: u64) -> Result<EdgeCsvRow> {
    let path = run_dir.join("graph_edges.csv");
    let mut reader = csv::Reader::from_path(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let headers = reader.headers()?.clone();

    let edge_id_idx = header_index(&headers, "edge_id")?;
    let source_idx = header_index(&headers, "source_event_id")?;
    let target_idx = header_index(&headers, "target_event_id")?;
    let observer_idx = header_index(&headers, "observer_id")?;
    let trust_idx = header_index(&headers, "trust_at_creation")?;
    let rewrite_idx = header_index(&headers, "rewrite_rule_at_source")?;

    for row in reader.records() {
        let row = row?;
        if parse_u64(&row, edge_id_idx)? == edge_id {
            return Ok(EdgeCsvRow {
                edge_id,
                source_event_id: parse_u64(&row, source_idx)?,
                target_event_id: parse_u64(&row, target_idx)?,
                observer_id: parse_u32(&row, observer_idx)?,
                trust_at_creation: parse_f64(&row, trust_idx)?,
                rewrite_rule_at_source: parse_u32(&row, rewrite_idx)?,
            });
        }
    }

    Err(anyhow!(
        "edge_id {} not found in {}",
        edge_id,
        path.display()
    ))
}

fn load_edge_row_by_endpoints(
    run_dir: &Path,
    source_event_id: u64,
    target_event_id: u64,
) -> Result<EdgeCsvRow> {
    let path = run_dir.join("graph_edges.csv");
    let mut reader = csv::Reader::from_path(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let headers = reader.headers()?.clone();

    let edge_id_idx = header_index(&headers, "edge_id")?;
    let source_idx = header_index(&headers, "source_event_id")?;
    let target_idx = header_index(&headers, "target_event_id")?;
    let observer_idx = header_index(&headers, "observer_id")?;
    let trust_idx = header_index(&headers, "trust_at_creation")?;
    let rewrite_idx = header_index(&headers, "rewrite_rule_at_source")?;

    for row in reader.records() {
        let row = row?;
        if parse_u64(&row, source_idx)? == source_event_id
            && parse_u64(&row, target_idx)? == target_event_id
        {
            return Ok(EdgeCsvRow {
                edge_id: parse_u64(&row, edge_id_idx)?,
                source_event_id,
                target_event_id,
                observer_id: parse_u32(&row, observer_idx)?,
                trust_at_creation: parse_f64(&row, trust_idx)?,
                rewrite_rule_at_source: parse_u32(&row, rewrite_idx)?,
            });
        }
    }

    Err(anyhow!(
        "edge ({}, {}) not found in {}",
        source_event_id,
        target_event_id,
        path.display()
    ))
}

fn load_source_event_row(
    run_dir: &Path,
    source_event_id: u64,
    observer_id: u32,
) -> Result<Option<EventCsvRow>> {
    let path = run_dir.join("graph_events.csv");
    let mut reader = csv::Reader::from_path(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let headers = reader.headers()?.clone();

    let event_id_idx = header_index(&headers, "event_id")?;
    let observer_idx = header_index(&headers, "observer_id")?;
    let time_idx = header_index(&headers, "time_index")?;
    let state_idx = header_index(&headers, "residual_state")?;
    let rewrite_idx = header_index(&headers, "rewrite_rule_id")?;
    let residual_idx = header_index(&headers, "residual_summary")?;
    let trust_idx = header_index(&headers, "trust_value")?;

    for row in reader.records() {
        let row = row?;
        if parse_u64(&row, event_id_idx)? == source_event_id
            && parse_u32(&row, observer_idx)? == observer_id
        {
            return Ok(Some(EventCsvRow {
                time_index: row.get(time_idx).unwrap_or_default().to_string(),
                residual_state: row.get(state_idx).unwrap_or_default().to_string(),
                rewrite_rule_id: row.get(rewrite_idx).unwrap_or_default().to_string(),
                residual_summary: row.get(residual_idx).unwrap_or_default().to_string(),
                trust_value: row.get(trust_idx).unwrap_or_default().to_string(),
            }));
        }
    }

    Ok(None)
}

fn write_edge_provenance_row(
    run_dir: &Path,
    edge_row: EdgeCsvRow,
    event_row: Option<EventCsvRow>,
) -> Result<PathBuf> {
    let path = run_dir.join(format!("edge_provenance_{}.csv", edge_row.edge_id));
    let mut writer = Writer::from_path(&path)?;
    writer.write_record([
        "edge_id",
        "source_event_id",
        "target_event_id",
        "observer_id",
        "trust_at_creation",
        "rewrite_rule_at_source",
        "time_index",
        "residual_state",
        "rewrite_rule_id",
        "residual_summary",
        "trust_value",
    ])?;

    writer.write_record([
        edge_row.edge_id.to_string(),
        edge_row.source_event_id.to_string(),
        edge_row.target_event_id.to_string(),
        edge_row.observer_id.to_string(),
        edge_row.trust_at_creation.to_string(),
        edge_row.rewrite_rule_at_source.to_string(),
        event_row
            .as_ref()
            .map(|row| row.time_index.clone())
            .unwrap_or_default(),
        event_row
            .as_ref()
            .map(|row| row.residual_state.clone())
            .unwrap_or_default(),
        event_row
            .as_ref()
            .map(|row| row.rewrite_rule_id.clone())
            .unwrap_or_default(),
        event_row
            .as_ref()
            .map(|row| row.residual_summary.clone())
            .unwrap_or_default(),
        event_row
            .as_ref()
            .map(|row| row.trust_value.clone())
            .unwrap_or_default(),
    ])?;

    writer.flush()?;
    Ok(path)
}

fn header_index(headers: &StringRecord, name: &str) -> Result<usize> {
    headers
        .iter()
        .position(|column| column == name)
        .ok_or_else(|| anyhow!("missing column '{}'", name))
}

fn parse_u64(record: &StringRecord, idx: usize) -> Result<u64> {
    record
        .get(idx)
        .ok_or_else(|| anyhow!("missing field index {}", idx))?
        .parse::<u64>()
        .with_context(|| format!("failed to parse u64 at index {}", idx))
}

fn parse_u32(record: &StringRecord, idx: usize) -> Result<u32> {
    record
        .get(idx)
        .ok_or_else(|| anyhow!("missing field index {}", idx))?
        .parse::<u32>()
        .with_context(|| format!("failed to parse u32 at index {}", idx))
}

fn parse_f64(record: &StringRecord, idx: usize) -> Result<f64> {
    record
        .get(idx)
        .ok_or_else(|| anyhow!("missing field index {}", idx))?
        .parse::<f64>()
        .with_context(|| format!("failed to parse f64 at index {}", idx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::integrations::{ResidualState, TrustProfile};

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
                time_index: 0,
                observer_id: 0,
                trust: 1.0,
                residual_summary: 0.0,
                residual_state: ResidualState::Low,
                rewrite_rule_id: 0,
                rewrite_rule_label: "stable_envelope",
                trust_profile: TrustProfile::Medium,
                envelope_ok: true,
            },
            DscdObserverSample {
                event_id: 1,
                time_index: 1,
                observer_id: 0,
                trust: 0.8,
                residual_summary: 0.1,
                residual_state: ResidualState::Low,
                rewrite_rule_id: 0,
                rewrite_rule_label: "stable_envelope",
                trust_profile: TrustProfile::Medium,
                envelope_ok: true,
            },
            DscdObserverSample {
                event_id: 2,
                time_index: 2,
                observer_id: 0,
                trust: 0.6,
                residual_summary: 0.2,
                residual_state: ResidualState::Medium,
                rewrite_rule_id: 1,
                rewrite_rule_label: "moderate_envelope",
                trust_profile: TrustProfile::Medium,
                envelope_ok: true,
            },
            DscdObserverSample {
                event_id: 3,
                time_index: 3,
                observer_id: 0,
                trust: 0.3,
                residual_summary: 0.3,
                residual_state: ResidualState::High,
                rewrite_rule_id: 2,
                rewrite_rule_label: "high_residual_recovery",
                trust_profile: TrustProfile::Medium,
                envelope_ok: true,
            },
            DscdObserverSample {
                event_id: 4,
                time_index: 4,
                observer_id: 0,
                trust: 0.1,
                residual_summary: 0.4,
                residual_state: ResidualState::High,
                rewrite_rule_id: 3,
                rewrite_rule_label: "envelope_decay",
                trust_profile: TrustProfile::Medium,
                envelope_ok: false,
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

    #[test]
    fn finite_size_metrics_are_computed_from_threshold_records() {
        let records = vec![
            ThresholdRecord {
                tau: 0.0,
                expansion_ratio: 1.0,
                reachable_size: 0,
                s_infty: None,
            },
            ThresholdRecord {
                tau: 0.2,
                expansion_ratio: 0.8,
                reachable_size: 0,
                s_infty: None,
            },
            ThresholdRecord {
                tau: 0.4,
                expansion_ratio: 0.2,
                reachable_size: 0,
                s_infty: None,
            },
            ThresholdRecord {
                tau: 0.6,
                expansion_ratio: 0.0,
                reachable_size: 0,
                s_infty: None,
            },
        ];

        let width = compute_width_0_1_to_0_9(&records);
        let max_derivative = compute_max_derivative(&records).expect("max derivative");

        assert!((width - 0.4).abs() < 1.0e-12);
        assert!((max_derivative - 3.0).abs() < 1.0e-12);
    }

    #[test]
    fn scaling_writes_summary_and_curves() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let out_dir = std::env::temp_dir().join(format!("dsfb-dscd-scaling-test-{}", now));

        let cfg = DscdScalingConfig {
            event_counts: vec![64, 128],
            tau_grid: vec![0.0, 0.5, 1.0],
            initial_event: EventId(0),
            max_path_length: usize::MAX,
            critical_fraction: 0.5,
            ..DscdScalingConfig::default()
        };

        run_threshold_scaling(&cfg, &out_dir).expect("scaling should run");

        let summary_path = out_dir.join("threshold_scaling_summary.csv");
        assert!(summary_path.exists());
        assert!(out_dir.join("threshold_curve_N_64.csv").exists());
        assert!(out_dir.join("threshold_curve_N_128.csv").exists());

        let mut reader = csv::Reader::from_path(&summary_path).expect("open summary");
        let rows: Vec<_> = reader
            .records()
            .collect::<std::result::Result<_, _>>()
            .expect("rows");
        assert_eq!(rows.len(), 2);

        let tau_star: f64 = rows[0][1].parse().expect("tau_star parse");
        let width: f64 = rows[0][2].parse().expect("width parse");
        assert!((0.0..=1.0).contains(&tau_star));
        assert!(width >= 0.0);

        let _ = fs::remove_dir_all(&out_dir);
    }
}
