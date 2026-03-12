use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};

use crate::event::StructuralEvent;

pub type DynError = Box<dyn Error>;

#[derive(Clone, Debug)]
pub struct RunManifestRow {
    pub run_id: String,
    pub timestamp: String,
    pub config_hash: String,
    pub crate_name: String,
    pub crate_version: String,
    pub n_events: usize,
    pub n_channels: usize,
    pub causal_window: usize,
    pub tau_steps: usize,
    pub shock_start: usize,
    pub shock_end: usize,
    pub beta: f64,
    pub envelope_decay: f64,
}

#[derive(Clone, Debug)]
pub struct ThresholdSweepRow {
    pub run_id: String,
    pub n_events: usize,
    pub tau_threshold: f64,
    pub reachable_count: usize,
    pub reachable_fraction: f64,
    pub edge_count: usize,
    pub mean_out_degree: f64,
    pub largest_component_fraction: f64,
    pub component_entropy: f64,
}

#[derive(Clone, Debug)]
pub struct TransitionSharpnessRow {
    pub run_id: String,
    pub n_events: usize,
    pub tau_midpoint: f64,
    pub drho_dtau: f64,
    pub abs_drho_dtau: f64,
}

#[derive(Clone, Debug)]
pub struct TimeLocalMetricsRow {
    pub run_id: String,
    pub tau_threshold: f64,
    pub window_start: usize,
    pub window_end: usize,
    pub anchor_event: usize,
    pub reachable_fraction: f64,
    pub active_edge_count: usize,
    pub mean_out_degree: f64,
    pub regime_label: String,
}

#[derive(Clone, Debug)]
pub struct GraphSnapshotRow {
    pub run_id: String,
    pub tau_threshold: f64,
    pub src: usize,
    pub dst: usize,
    pub src_trust: f64,
    pub dst_trust: f64,
    pub compatible: bool,
}

#[derive(Clone, Debug)]
pub struct ExportBundle {
    pub manifest: RunManifestRow,
    pub events: Vec<StructuralEvent>,
    pub threshold_sweep: Vec<ThresholdSweepRow>,
    pub transition_sharpness: Vec<TransitionSharpnessRow>,
    pub time_local_metrics: Vec<TimeLocalMetricsRow>,
    pub graph_snapshot_low: Vec<GraphSnapshotRow>,
    pub graph_snapshot_critical: Vec<GraphSnapshotRow>,
    pub graph_snapshot_high: Vec<GraphSnapshotRow>,
    pub graph_snapshot_tau_020: Vec<GraphSnapshotRow>,
    pub graph_snapshot_tau_030: Vec<GraphSnapshotRow>,
    pub graph_snapshot_tau_040: Vec<GraphSnapshotRow>,
}

#[derive(Clone, Debug)]
pub struct ExportOutcome {
    pub timestamp: String,
    pub run_dir: PathBuf,
}

pub fn prepare_output_dir(repo_root: &Path) -> Result<ExportOutcome, DynError> {
    let output_root = repo_root.join("output-dsfb-srd");
    fs::create_dir_all(&output_root)?;

    let base = Utc::now();
    for seconds in 0..120 {
        let timestamp = (base + Duration::seconds(seconds))
            .format("%Y%m%d-%H%M%S")
            .to_string();
        let run_dir = output_root.join(&timestamp);
        if !run_dir.exists() {
            fs::create_dir_all(&run_dir)?;
            return Ok(ExportOutcome { timestamp, run_dir });
        }
    }

    Err("unable to allocate a unique timestamped output directory".into())
}

pub fn write_bundle(bundle: &ExportBundle, run_dir: &Path) -> Result<(), DynError> {
    write_run_manifest(&run_dir.join("run_manifest.csv"), &bundle.manifest)?;
    write_events(
        &run_dir.join("events.csv"),
        &bundle.manifest.run_id,
        &bundle.events,
    )?;
    write_threshold_sweep(
        &run_dir.join("threshold_sweep.csv"),
        &bundle.threshold_sweep,
    )?;
    write_transition_sharpness(
        &run_dir.join("transition_sharpness.csv"),
        &bundle.transition_sharpness,
    )?;
    write_time_local_metrics(
        &run_dir.join("time_local_metrics.csv"),
        &bundle.time_local_metrics,
    )?;
    write_graph_snapshot(
        &run_dir.join("graph_snapshot_low.csv"),
        &bundle.graph_snapshot_low,
    )?;
    write_graph_snapshot(
        &run_dir.join("graph_snapshot_critical.csv"),
        &bundle.graph_snapshot_critical,
    )?;
    write_graph_snapshot(
        &run_dir.join("graph_snapshot_high.csv"),
        &bundle.graph_snapshot_high,
    )?;
    write_graph_snapshot(
        &run_dir.join("graph_snapshot_tau_020.csv"),
        &bundle.graph_snapshot_tau_020,
    )?;
    write_graph_snapshot(
        &run_dir.join("graph_snapshot_tau_030.csv"),
        &bundle.graph_snapshot_tau_030,
    )?;
    write_graph_snapshot(
        &run_dir.join("graph_snapshot_tau_040.csv"),
        &bundle.graph_snapshot_tau_040,
    )?;

    Ok(())
}

fn write_run_manifest(path: &Path, row: &RunManifestRow) -> Result<(), DynError> {
    let line = format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{}",
        row.run_id,
        row.timestamp,
        row.config_hash,
        row.crate_name,
        row.crate_version,
        row.n_events,
        row.n_channels,
        row.causal_window,
        row.tau_steps,
        row.shock_start,
        row.shock_end,
        fmt_f64(row.beta),
        fmt_f64(row.envelope_decay),
    );

    write_lines(
        path,
        "run_id,timestamp,config_hash,crate_name,crate_version,n_events,n_channels,causal_window,tau_steps,shock_start,shock_end,beta,envelope_decay",
        std::iter::once(line),
    )
}

fn write_events(path: &Path, run_id: &str, events: &[StructuralEvent]) -> Result<(), DynError> {
    let lines = events.iter().map(|event| {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{}",
            run_id,
            event.event_id,
            event.time_index,
            event.channel_id,
            fmt_f64(event.latent_state),
            fmt_f64(event.predicted_value),
            fmt_f64(event.observed_value),
            fmt_f64(event.residual),
            fmt_f64(event.envelope),
            fmt_f64(event.trust),
            event.regime_label.as_str(),
        )
    });

    write_lines(
        path,
        "run_id,event_id,time_index,channel_id,latent_state,predicted_value,observed_value,residual,envelope,trust,regime_label",
        lines,
    )
}

fn write_threshold_sweep(path: &Path, rows: &[ThresholdSweepRow]) -> Result<(), DynError> {
    let lines = rows.iter().map(|row| {
        format!(
            "{},{},{},{},{},{},{},{},{}",
            row.run_id,
            row.n_events,
            fmt_f64(row.tau_threshold),
            row.reachable_count,
            fmt_f64(row.reachable_fraction),
            row.edge_count,
            fmt_f64(row.mean_out_degree),
            fmt_f64(row.largest_component_fraction),
            fmt_f64(row.component_entropy),
        )
    });

    write_lines(
        path,
        "run_id,n_events,tau_threshold,reachable_count,reachable_fraction,edge_count,mean_out_degree,largest_component_fraction,component_entropy",
        lines,
    )
}

fn write_transition_sharpness(
    path: &Path,
    rows: &[TransitionSharpnessRow],
) -> Result<(), DynError> {
    let lines = rows.iter().map(|row| {
        format!(
            "{},{},{},{},{}",
            row.run_id,
            row.n_events,
            fmt_f64(row.tau_midpoint),
            fmt_f64(row.drho_dtau),
            fmt_f64(row.abs_drho_dtau),
        )
    });

    write_lines(
        path,
        "run_id,n_events,tau_midpoint,drho_dtau,abs_drho_dtau",
        lines,
    )
}

fn write_time_local_metrics(path: &Path, rows: &[TimeLocalMetricsRow]) -> Result<(), DynError> {
    let lines = rows.iter().map(|row| {
        format!(
            "{},{},{},{},{},{},{},{},{}",
            row.run_id,
            fmt_f64(row.tau_threshold),
            row.window_start,
            row.window_end,
            row.anchor_event,
            fmt_f64(row.reachable_fraction),
            row.active_edge_count,
            fmt_f64(row.mean_out_degree),
            row.regime_label,
        )
    });

    write_lines(
        path,
        "run_id,tau_threshold,window_start,window_end,anchor_event,reachable_fraction,active_edge_count,mean_out_degree,regime_label",
        lines,
    )
}

fn write_graph_snapshot(path: &Path, rows: &[GraphSnapshotRow]) -> Result<(), DynError> {
    let lines = rows.iter().map(|row| {
        format!(
            "{},{},{},{},{},{},{}",
            row.run_id,
            fmt_f64(row.tau_threshold),
            row.src,
            row.dst,
            fmt_f64(row.src_trust),
            fmt_f64(row.dst_trust),
            row.compatible,
        )
    });

    write_lines(
        path,
        "run_id,tau_threshold,src,dst,src_trust,dst_trust,compatible",
        lines,
    )
}

fn write_lines<I>(path: &Path, header: &str, lines: I) -> Result<(), DynError>
where
    I: IntoIterator<Item = String>,
{
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "{header}")?;
    for line in lines {
        writeln!(writer, "{line}")?;
    }
    writer.flush()?;
    Ok(())
}

fn fmt_f64(value: f64) -> String {
    format!("{value:.8}")
}
