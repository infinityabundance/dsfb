use std::cmp::Ordering;
use std::path::PathBuf;

use crate::config::{compute_run_id, SimulationConfig, CRATE_NAME, CRATE_VERSION};
use crate::export::{
    prepare_output_dir, write_bundle, DynError, ExportBundle, GraphSnapshotRow, RunManifestRow,
    ThresholdSweepRow, TimeLocalMetricsRow, TransitionSharpnessRow,
};
use crate::graph::{
    build_candidate_graph, collect_active_edges, compute_graph_stats, compute_graph_stats_in_range,
    CandidateGraph,
};
use crate::metrics::{discrete_derivative, low_high_thresholds, window_ranges, window_regime};
use crate::signal::generate_events;

const FINITE_SIZE_SWEEP: [usize; 4] = [250, 500, 1_000, 2_000];

#[derive(Clone, Debug)]
pub struct GeneratedRun {
    pub run_id: String,
    pub config_hash: String,
    pub timestamp: String,
    pub output_dir: PathBuf,
}

pub fn run_simulation(config: SimulationConfig) -> Result<GeneratedRun, DynError> {
    config
        .validate()
        .map_err(|message| -> DynError { message.into() })?;

    let run_id = compute_run_id(&config);
    let config_hash = config.config_hash();
    let primary_events = generate_events(&config);
    let primary_graph = build_candidate_graph(&primary_events, &config);
    let thresholds = config.tau_thresholds();
    let (tau_low, tau_high) = low_high_thresholds(&thresholds);

    let mut sweep_sizes = FINITE_SIZE_SWEEP.to_vec();
    if !sweep_sizes.contains(&config.n_events) {
        sweep_sizes.push(config.n_events);
        sweep_sizes.sort_unstable();
    }

    let mut threshold_sweep = Vec::new();
    let mut transition_sharpness = Vec::new();
    let mut primary_critical_threshold = thresholds[thresholds.len() / 2];

    for &n_events in &sweep_sizes {
        let sized_config = if n_events == config.n_events {
            config.clone()
        } else {
            config.scaled_for_n_events(n_events)
        };
        sized_config
            .validate()
            .map_err(|message| -> DynError { message.into() })?;

        let (events, graph) = if n_events == config.n_events {
            (primary_events.clone(), primary_graph.clone())
        } else {
            let events = generate_events(&sized_config);
            let graph = build_candidate_graph(&events, &sized_config);
            (events, graph)
        };

        let sweep_rows = build_threshold_sweep_rows(&run_id, &thresholds, &events, &graph);
        let transition_rows = build_transition_rows(&run_id, n_events, &sweep_rows);

        if n_events == config.n_events {
            primary_critical_threshold =
                select_critical_threshold(&transition_rows, primary_critical_threshold);
        }

        threshold_sweep.extend(sweep_rows);
        transition_sharpness.extend(transition_rows);
    }

    let time_local_metrics = build_time_local_rows(
        &run_id,
        &config,
        &primary_events,
        &primary_graph,
        &[tau_low, primary_critical_threshold, tau_high],
    );
    let graph_snapshot_low =
        build_graph_snapshot_rows(&run_id, tau_low, &primary_events, &primary_graph);
    let graph_snapshot_critical = build_graph_snapshot_rows(
        &run_id,
        primary_critical_threshold,
        &primary_events,
        &primary_graph,
    );
    let graph_snapshot_high =
        build_graph_snapshot_rows(&run_id, tau_high, &primary_events, &primary_graph);

    let output = prepare_output_dir(&SimulationConfig::repo_root())?;
    let manifest = RunManifestRow {
        run_id: run_id.clone(),
        timestamp: output.timestamp.clone(),
        config_hash: config_hash.clone(),
        crate_name: CRATE_NAME.to_string(),
        crate_version: CRATE_VERSION.to_string(),
        n_events: config.n_events,
        n_channels: config.n_channels,
        causal_window: config.causal_window,
        tau_steps: config.tau_steps,
        shock_start: config.shock_start,
        shock_end: config.shock_end,
        beta: config.beta,
        envelope_decay: config.envelope_decay,
    };

    let bundle = ExportBundle {
        manifest,
        events: primary_events,
        threshold_sweep,
        transition_sharpness,
        time_local_metrics,
        graph_snapshot_low,
        graph_snapshot_critical,
        graph_snapshot_high,
    };

    write_bundle(&bundle, &output.run_dir)?;

    Ok(GeneratedRun {
        run_id,
        config_hash,
        timestamp: output.timestamp,
        output_dir: output.run_dir,
    })
}

fn build_threshold_sweep_rows(
    run_id: &str,
    thresholds: &[f64],
    events: &[crate::event::StructuralEvent],
    graph: &CandidateGraph,
) -> Vec<ThresholdSweepRow> {
    thresholds
        .iter()
        .copied()
        .map(|tau_threshold| {
            let stats = compute_graph_stats(graph, events, tau_threshold, 0);
            ThresholdSweepRow {
                run_id: run_id.to_string(),
                n_events: events.len(),
                tau_threshold,
                reachable_count: stats.reachable_count,
                reachable_fraction: stats.reachable_count as f64 / events.len() as f64,
                edge_count: stats.edge_count,
                mean_out_degree: stats.mean_out_degree,
                largest_component_fraction: stats.largest_component_fraction,
            }
        })
        .collect()
}

fn build_transition_rows(
    run_id: &str,
    n_events: usize,
    sweep_rows: &[ThresholdSweepRow],
) -> Vec<TransitionSharpnessRow> {
    let points: Vec<(f64, f64)> = sweep_rows
        .iter()
        .map(|row| (row.tau_threshold, row.reachable_fraction))
        .collect();

    discrete_derivative(&points)
        .into_iter()
        .map(
            |(tau_midpoint, drho_dtau, abs_drho_dtau)| TransitionSharpnessRow {
                run_id: run_id.to_string(),
                n_events,
                tau_midpoint,
                drho_dtau,
                abs_drho_dtau,
            },
        )
        .collect()
}

fn select_critical_threshold(rows: &[TransitionSharpnessRow], fallback: f64) -> f64 {
    rows.iter()
        .max_by(|left, right| {
            left.abs_drho_dtau
                .partial_cmp(&right.abs_drho_dtau)
                .unwrap_or(Ordering::Equal)
        })
        .map(|row| row.tau_midpoint)
        .unwrap_or(fallback)
}

fn build_time_local_rows(
    run_id: &str,
    config: &SimulationConfig,
    events: &[crate::event::StructuralEvent],
    graph: &CandidateGraph,
    thresholds: &[f64],
) -> Vec<TimeLocalMetricsRow> {
    let mut rows = Vec::new();

    for &tau_threshold in thresholds {
        for (window_start, window_end_exclusive) in window_ranges(config) {
            let anchor_event = window_start;
            let stats = compute_graph_stats_in_range(
                graph,
                events,
                tau_threshold,
                anchor_event,
                (window_start, window_end_exclusive),
            );
            let window_len = window_end_exclusive - window_start;
            rows.push(TimeLocalMetricsRow {
                run_id: run_id.to_string(),
                tau_threshold,
                window_start,
                window_end: window_end_exclusive - 1,
                anchor_event,
                reachable_fraction: stats.reachable_count as f64 / window_len as f64,
                active_edge_count: stats.edge_count,
                mean_out_degree: stats.mean_out_degree,
                regime_label: window_regime(events, window_start, window_end_exclusive)
                    .as_str()
                    .to_string(),
            });
        }
    }

    rows
}

fn build_graph_snapshot_rows(
    run_id: &str,
    tau_threshold: f64,
    events: &[crate::event::StructuralEvent],
    graph: &CandidateGraph,
) -> Vec<GraphSnapshotRow> {
    collect_active_edges(graph, events, tau_threshold, None)
        .into_iter()
        .map(|edge| GraphSnapshotRow {
            run_id: run_id.to_string(),
            tau_threshold,
            src: edge.src,
            dst: edge.dst,
            src_trust: events[edge.src].trust,
            dst_trust: events[edge.dst].trust,
            compatible: edge.compatible,
        })
        .collect()
}
