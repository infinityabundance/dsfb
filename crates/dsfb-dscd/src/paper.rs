//! DSCD paper-facing deterministic simulation and export pipeline.
//!
//! This module layers Deterministic Structural Causal Dynamics (DSCD) on top
//! of the existing DSFB and ADD crates. It keeps all computation deterministic
//! (no random sampling in the main path) and exports reproducible CSV artifacts
//! used by DSCD paper figures, including finite-size threshold scaling.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use csv::Writer;
use dsfb::sim::SimConfig;
use dsfb::DsfbParams;
use dsfb_add::aet::run_aet_sweep;
use dsfb_add::iwlt::run_iwlt_sweep;
use dsfb_add::SimulationConfig as AddSimulationConfig;
use rayon::prelude::*;
use serde::Serialize;

use crate::integrations::{
    generate_dscd_events_from_dsfb, DscdObserverSample, ResidualState, TrustProfile,
};

/// DSCD event node exported for causal graph and scaling analyses.
///
/// The values are derived deterministically from DSFB traces and ADD structural
/// sweeps (no stochastic sampling), supporting reproducible DSCD paper figures.
#[derive(Debug, Clone, Serialize)]
pub struct DscdEvent {
    pub id: u64,
    pub time_index: u64,
    pub module_id: u32,
    pub residual_state: String,
    pub trust: f64,
    pub echo_slope: f64,
    pub entropy_density: f64,
}

/// DSCD directed edge candidate with trust-gated activation metadata.
///
/// `rewrite_rule_id` provides local provenance from deterministic DSFB-derived
/// update rules for traceability figures.
#[derive(Debug, Clone, Serialize)]
pub struct DscdEdge {
    pub src: u64,
    pub dst: u64,
    pub trust_at_creation: f64,
    pub module_id: u32,
    pub rewrite_rule_id: Option<String>,
}

/// Trust-gated DSCD graph at a fixed threshold.
#[derive(Debug, Clone, Default)]
pub struct DscdGraph {
    pub events: Vec<DscdEvent>,
    pub edges: Vec<DscdEdge>,
}

/// Simulation configuration for deterministic DSCD scaling and exports.
///
/// This config drives finite-size transition measurements (Theorem 4 support)
/// and all paper CSV artifacts from a single reproducible run.
#[derive(Debug, Clone)]
pub struct DscdConfig {
    pub num_events: usize,
    pub taus: Vec<f64>,
    pub root_event_id: u64,
    pub output_dir: PathBuf,
    pub scaling_ns: Vec<usize>,
    pub quick_mode: bool,
}

impl Default for DscdConfig {
    fn default() -> Self {
        Self {
            num_events: 10_000,
            taus: linspace(0.0, 1.0, 201),
            root_event_id: 0,
            output_dir: PathBuf::from("output-dsfb-dscd"),
            scaling_ns: vec![2_000, 5_000, 10_000],
            quick_mode: true,
        }
    }
}

/// DSCD error type alias.
pub type DscdError = anyhow::Error;

#[derive(Debug, Clone)]
struct CurvePoint {
    tau: f64,
    expansion_ratio: f64,
}

#[derive(Debug, Clone)]
struct ScalingMetrics {
    num_events: usize,
    tau_star: f64,
    transition_width: f64,
    max_derivative: f64,
}

#[derive(Debug, Clone)]
struct EventAux {
    event: DscdEvent,
    rewrite_rule_id: String,
    observer_trust: HashMap<u32, f64>,
}

#[derive(Debug, Clone)]
struct Dataset {
    events: Vec<DscdEvent>,
    candidate_edges: Vec<DscdEdge>,
}

/// Build a trust-gated DAG for a threshold `tau`.
///
/// Edges are kept iff `trust_at_creation >= tau` and temporal ordering is
/// respected (`src.time_index < dst.time_index`).
pub fn build_graph_for_tau(
    events: &[DscdEvent],
    candidate_edges: &[DscdEdge],
    tau: f64,
) -> DscdGraph {
    let time_by_id: HashMap<u64, u64> = events
        .iter()
        .map(|event| (event.id, event.time_index))
        .collect();

    let mut edges = Vec::new();
    for edge in candidate_edges {
        if edge.trust_at_creation < tau {
            continue;
        }

        let Some(&src_time) = time_by_id.get(&edge.src) else {
            continue;
        };
        let Some(&dst_time) = time_by_id.get(&edge.dst) else {
            continue;
        };

        if src_time < dst_time {
            edges.push(edge.clone());
        }
    }

    DscdGraph {
        events: events.to_vec(),
        edges,
    }
}

/// Compute reachable component size from `root_id` using deterministic BFS.
pub fn compute_reachable_component_size(graph: &DscdGraph, root_id: u64) -> usize {
    if graph.events.is_empty() || !graph.events.iter().any(|event| event.id == root_id) {
        return 0;
    }

    let mut adjacency: HashMap<u64, Vec<u64>> = HashMap::new();
    for edge in &graph.edges {
        adjacency.entry(edge.src).or_default().push(edge.dst);
    }
    for neighbors in adjacency.values_mut() {
        neighbors.sort_unstable();
    }

    let mut queue = VecDeque::from([root_id]);
    let mut visited = HashSet::from([root_id]);
    while let Some(current) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(&current) {
            for next in neighbors {
                if visited.insert(*next) {
                    queue.push_back(*next);
                }
            }
        }
    }

    visited.len()
}

/// Reachable fraction for the root component in the trust-gated graph.
pub fn expansion_ratio(graph: &DscdGraph, root_id: u64) -> f64 {
    if graph.events.is_empty() {
        return 0.0;
    }
    compute_reachable_component_size(graph, root_id) as f64 / graph.events.len() as f64
}

/// Run full deterministic DSCD simulation, scaling, and CSV export pipeline.
///
/// Outputs are written inside `cfg.output_dir`, which should point to a
/// timestamped run folder under `output-dsfb-dscd/<YYYYMMDD_HHMMSS>/`.
pub fn run_dscd_simulation(cfg: &DscdConfig) -> std::result::Result<(), DscdError> {
    validate_config(cfg)?;
    fs::create_dir_all(&cfg.output_dir).with_context(|| {
        format!(
            "failed to create DSCD output directory {}",
            cfg.output_dir.display()
        )
    })?;

    let mut run_ns = cfg.scaling_ns.clone();
    run_ns.push(cfg.num_events);
    run_ns.sort_unstable();
    run_ns.dedup();

    let max_n = run_ns
        .iter()
        .copied()
        .max()
        .ok_or_else(|| anyhow!("no event counts configured"))?;
    let dataset = generate_dataset(max_n, cfg.quick_mode)?;

    let mut summary_rows = Vec::new();
    let mut curve_by_n: HashMap<usize, Vec<CurvePoint>> = HashMap::new();
    let mut main_events = Vec::new();
    let mut main_edges = Vec::new();

    for n in run_ns {
        let events = dataset.events[..n].to_vec();
        let candidate_edges: Vec<DscdEdge> = dataset
            .candidate_edges
            .iter()
            .filter(|edge| edge.src < n as u64 && edge.dst < n as u64)
            .cloned()
            .collect();

        let curve =
            compute_threshold_curve(&events, &candidate_edges, &cfg.taus, cfg.root_event_id);
        write_threshold_curve_csv(
            &cfg.output_dir.join(format!("threshold_curve_N_{n}.csv")),
            &curve,
        )?;

        let metrics = compute_scaling_metrics(n, &curve);
        summary_rows.push(metrics.clone());
        curve_by_n.insert(n, curve);

        if n == cfg.num_events {
            main_events = events;
            main_edges = candidate_edges;
        }
    }

    write_threshold_scaling_summary_csv(
        &cfg.output_dir.join("threshold_scaling_summary.csv"),
        &summary_rows,
    )?;

    let Some(main_curve) = curve_by_n.get(&cfg.num_events) else {
        bail!(
            "missing threshold curve for main num_events={}",
            cfg.num_events
        );
    };
    let main_metrics = compute_scaling_metrics(cfg.num_events, main_curve);
    let main_tau = main_metrics.tau_star;

    write_graph_events_csv(&cfg.output_dir.join("graph_events.csv"), &main_events)?;
    let final_graph = build_graph_for_tau(&main_events, &main_edges, main_tau);
    write_graph_edges_csv(&cfg.output_dir.join("graph_edges.csv"), &final_graph.edges)?;
    write_degree_distribution_csv(
        &cfg.output_dir.join("degree_distribution.csv"),
        &final_graph,
    )?;
    write_interval_sizes_csv(
        &cfg.output_dir.join("interval_sizes.csv"),
        &final_graph,
        1_024,
    )?;
    write_path_lengths_csv(
        &cfg.output_dir.join("path_lengths.csv"),
        &final_graph,
        cfg.root_event_id,
    )?;
    write_edge_provenance_csv(
        &cfg.output_dir.join("edge_provenance.csv"),
        &final_graph,
        100,
    )?;

    Ok(())
}

/// Build evenly spaced deterministic threshold grid.
pub fn linspace(start: f64, end: f64, count: usize) -> Vec<f64> {
    if count <= 1 {
        return vec![start];
    }
    let step = (end - start) / (count.saturating_sub(1) as f64);
    (0..count).map(|idx| start + step * idx as f64).collect()
}

fn validate_config(cfg: &DscdConfig) -> Result<()> {
    if cfg.num_events == 0 {
        bail!("num_events must be greater than zero");
    }
    if cfg.taus.is_empty() {
        bail!("taus must contain at least one threshold");
    }
    if cfg.taus.iter().any(|tau| !tau.is_finite()) {
        bail!("taus must be finite values");
    }
    if cfg.taus.windows(2).any(|pair| pair[1] < pair[0]) {
        bail!("taus must be sorted in nondecreasing order");
    }
    if cfg.scaling_ns.contains(&0) {
        bail!("scaling_ns values must be greater than zero");
    }
    Ok(())
}

fn generate_dataset(max_events: usize, quick_mode: bool) -> Result<Dataset> {
    let dsfb_cfg = SimConfig {
        steps: max_events,
        ..SimConfig::default()
    };
    let event_batch = generate_dscd_events_from_dsfb(&dsfb_cfg, DsfbParams::default(), max_events)?;

    let mut add_cfg = AddSimulationConfig::default();
    let add_steps = if quick_mode {
        max_events.min(1_024)
    } else {
        max_events.min(4_096)
    }
    .max(128);
    add_cfg.steps_per_run = add_steps;
    add_cfg.multi_steps_per_run = vec![add_steps];
    let lambda_grid = add_cfg.lambda_grid();
    let aet = run_aet_sweep(&add_cfg, &lambda_grid)
        .map_err(|error| anyhow!("failed to compute AET sweep for DSCD: {error}"))?;
    let iwlt = run_iwlt_sweep(&add_cfg, &lambda_grid)
        .map_err(|error| anyhow!("failed to compute IWLT sweep for DSCD: {error}"))?;
    let echo_len = aet.echo_slope.len().max(1);
    let entropy_len = iwlt.entropy_density.len().max(1);

    let mut samples_by_event: BTreeMap<u64, Vec<&DscdObserverSample>> = BTreeMap::new();
    for sample in &event_batch.observer_samples {
        samples_by_event
            .entry(sample.event_id)
            .or_default()
            .push(sample);
    }

    let mut aux_events = Vec::with_capacity(event_batch.events.len());
    for (idx, event) in event_batch.events.iter().enumerate() {
        let event_id = event.id.0;
        let samples = samples_by_event.get(&event_id).cloned().unwrap_or_default();

        let mut trust_sum = 0.0;
        let mut residual_sum = 0.0;
        let mut observer_trust = HashMap::new();
        let mut best_observer = 0_u32;
        let mut best_trust = f64::NEG_INFINITY;
        let mut rewrite_rule_id = String::from("stable_envelope");

        for sample in &samples {
            trust_sum += sample.trust;
            residual_sum += sample.residual_summary;
            observer_trust.insert(sample.observer_id, sample.trust);
            if sample.trust > best_trust {
                best_trust = sample.trust;
                best_observer = sample.observer_id;
                rewrite_rule_id = sample.rewrite_rule_label.to_string();
            }
        }

        let sample_count = samples.len().max(1) as f64;
        let avg_residual = residual_sum / sample_count;
        let avg_trust = (trust_sum / sample_count).clamp(0.0, 1.0);
        let residual_state = ResidualState::from_residual(avg_residual)
            .as_str()
            .to_string();

        let event = DscdEvent {
            id: event_id,
            time_index: idx as u64,
            module_id: best_observer,
            residual_state,
            trust: avg_trust,
            echo_slope: aet.echo_slope[idx % echo_len],
            entropy_density: iwlt.entropy_density[idx % entropy_len],
        };

        aux_events.push(EventAux {
            event,
            rewrite_rule_id,
            observer_trust,
        });
    }

    let candidate_edges = build_candidate_edges(&aux_events, quick_mode);
    let events = aux_events.into_iter().map(|entry| entry.event).collect();
    Ok(Dataset {
        events,
        candidate_edges,
    })
}

fn build_candidate_edges(events: &[EventAux], quick_mode: bool) -> Vec<DscdEdge> {
    let neighborhood = if quick_mode { 16 } else { 24 };
    let mut edges = Vec::new();

    for dst_idx in 1..events.len() {
        let dst = &events[dst_idx];
        let start_idx = dst_idx.saturating_sub(neighborhood);
        for src in events.iter().take(dst_idx).skip(start_idx) {
            if src.event.time_index >= dst.event.time_index {
                continue;
            }

            let resonance = (src.event.echo_slope - dst.event.echo_slope).abs()
                + (src.event.entropy_density - dst.event.entropy_density).abs();
            let gap = dst.event.time_index.saturating_sub(src.event.time_index);
            let profile_mismatch = profile_spread(src.event.module_id, dst.event.module_id);
            let max_resonance = 0.12 + 0.018 * gap as f64 + profile_mismatch;
            if resonance > max_resonance {
                continue;
            }

            let trust_at_creation = dst
                .observer_trust
                .get(&dst.event.module_id)
                .copied()
                .unwrap_or(dst.event.trust);

            edges.push(DscdEdge {
                src: src.event.id,
                dst: dst.event.id,
                trust_at_creation,
                module_id: dst.event.module_id,
                rewrite_rule_id: Some(src.rewrite_rule_id.clone()),
            });
        }
    }

    edges
}

fn profile_spread(src_observer: u32, dst_observer: u32) -> f64 {
    let src_profile = TrustProfile::from_observer_index(src_observer);
    let dst_profile = TrustProfile::from_observer_index(dst_observer);
    match (src_profile, dst_profile) {
        (TrustProfile::Tight, TrustProfile::Loose) | (TrustProfile::Loose, TrustProfile::Tight) => {
            0.020
        }
        (TrustProfile::Medium, TrustProfile::Medium) => 0.0,
        _ => 0.010,
    }
}

fn compute_threshold_curve(
    events: &[DscdEvent],
    candidate_edges: &[DscdEdge],
    taus: &[f64],
    root_event_id: u64,
) -> Vec<CurvePoint> {
    taus.par_iter()
        .map(|tau| {
            let graph = build_graph_for_tau(events, candidate_edges, *tau);
            CurvePoint {
                tau: *tau,
                expansion_ratio: expansion_ratio(&graph, root_event_id),
            }
        })
        .collect()
}

fn compute_scaling_metrics(num_events: usize, curve: &[CurvePoint]) -> ScalingMetrics {
    let tau_star = find_tau_star(curve, 0.5)
        .unwrap_or_else(|| curve.last().map(|point| point.tau).unwrap_or_default());
    let tau_0_9 = find_tau_star(curve, 0.9)
        .unwrap_or_else(|| curve.first().map(|point| point.tau).unwrap_or_default());
    let tau_0_1 = find_tau_star(curve, 0.1)
        .unwrap_or_else(|| curve.last().map(|point| point.tau).unwrap_or(tau_0_9));

    let max_derivative = curve
        .windows(2)
        .filter_map(|pair| {
            let dt = pair[1].tau - pair[0].tau;
            if dt.abs() <= f64::EPSILON {
                return None;
            }
            Some(((pair[1].expansion_ratio - pair[0].expansion_ratio) / dt).abs())
        })
        .fold(0.0_f64, f64::max);

    ScalingMetrics {
        num_events,
        tau_star,
        transition_width: (tau_0_1 - tau_0_9).max(0.0),
        max_derivative,
    }
}

fn find_tau_star(curve: &[CurvePoint], ratio_threshold: f64) -> Option<f64> {
    if curve.is_empty() {
        return None;
    }
    if curve[0].expansion_ratio <= ratio_threshold {
        return Some(curve[0].tau);
    }

    for pair in curve.windows(2) {
        let prev = &pair[0];
        let curr = &pair[1];
        if curr.expansion_ratio > ratio_threshold {
            continue;
        }
        let drho = curr.expansion_ratio - prev.expansion_ratio;
        if drho.abs() <= f64::EPSILON {
            return Some(curr.tau);
        }
        let alpha = (ratio_threshold - prev.expansion_ratio) / drho;
        return Some(prev.tau + alpha * (curr.tau - prev.tau));
    }

    None
}

fn write_threshold_curve_csv(path: &Path, curve: &[CurvePoint]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(["tau", "expansion_ratio"])?;
    for point in curve {
        writer.write_record([point.tau.to_string(), point.expansion_ratio.to_string()])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_threshold_scaling_summary_csv(path: &Path, rows: &[ScalingMetrics]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "num_events",
        "tau_star",
        "transition_width",
        "max_derivative",
    ])?;
    for row in rows {
        writer.write_record([
            row.num_events.to_string(),
            row.tau_star.to_string(),
            row.transition_width.to_string(),
            row.max_derivative.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_graph_events_csv(path: &Path, events: &[DscdEvent]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "id",
        "time_index",
        "module_id",
        "residual_state",
        "trust",
        "echo_slope",
        "entropy_density",
    ])?;
    for event in events {
        writer.write_record([
            event.id.to_string(),
            event.time_index.to_string(),
            event.module_id.to_string(),
            event.residual_state.clone(),
            event.trust.to_string(),
            event.echo_slope.to_string(),
            event.entropy_density.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_graph_edges_csv(path: &Path, edges: &[DscdEdge]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "src",
        "dst",
        "module_id",
        "trust_at_creation",
        "rewrite_rule_id",
    ])?;
    for edge in edges {
        writer.write_record([
            edge.src.to_string(),
            edge.dst.to_string(),
            edge.module_id.to_string(),
            edge.trust_at_creation.to_string(),
            edge.rewrite_rule_id.clone().unwrap_or_default(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_degree_distribution_csv(path: &Path, graph: &DscdGraph) -> Result<()> {
    let mut in_degree: HashMap<u64, usize> =
        graph.events.iter().map(|event| (event.id, 0)).collect();
    let mut out_degree: HashMap<u64, usize> =
        graph.events.iter().map(|event| (event.id, 0)).collect();

    for edge in &graph.edges {
        *out_degree.entry(edge.src).or_insert(0) += 1;
        *in_degree.entry(edge.dst).or_insert(0) += 1;
    }

    let mut writer = Writer::from_path(path)?;
    writer.write_record(["event_id", "in_degree", "out_degree"])?;
    for event in &graph.events {
        writer.write_record([
            event.id.to_string(),
            in_degree.get(&event.id).copied().unwrap_or(0).to_string(),
            out_degree.get(&event.id).copied().unwrap_or(0).to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_interval_sizes_csv(path: &Path, graph: &DscdGraph, sample_count: usize) -> Result<()> {
    let mut ids_by_time: Vec<(u64, u64)> = graph
        .events
        .iter()
        .map(|event| (event.time_index, event.id))
        .collect();
    ids_by_time.sort_unstable();
    let ordered_ids: Vec<u64> = ids_by_time.into_iter().map(|(_, id)| id).collect();

    let mut adjacency: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut reverse_adjacency: HashMap<u64, Vec<u64>> = HashMap::new();
    for edge in &graph.edges {
        adjacency.entry(edge.src).or_default().push(edge.dst);
        reverse_adjacency
            .entry(edge.dst)
            .or_default()
            .push(edge.src);
    }

    let pairs = deterministic_pairs(&ordered_ids, sample_count);
    let mut forward_cache: HashMap<u64, HashSet<u64>> = HashMap::new();
    let mut reverse_cache: HashMap<u64, HashSet<u64>> = HashMap::new();

    let mut writer = Writer::from_path(path)?;
    writer.write_record(["src", "dst", "interval_size"])?;
    for (src, dst) in pairs {
        let forward = reachability_set(src, &adjacency, &mut forward_cache);
        let interval_size = if !forward.contains(&dst) {
            0
        } else {
            let backward = reachability_set(dst, &reverse_adjacency, &mut reverse_cache);
            forward
                .intersection(&backward)
                .filter(|node| **node != src && **node != dst)
                .count()
        };

        writer.write_record([src.to_string(), dst.to_string(), interval_size.to_string()])?;
    }
    writer.flush()?;
    Ok(())
}

fn reachability_set(
    start: u64,
    adjacency: &HashMap<u64, Vec<u64>>,
    cache: &mut HashMap<u64, HashSet<u64>>,
) -> HashSet<u64> {
    if let Some(cached) = cache.get(&start) {
        return cached.clone();
    }

    let mut queue = VecDeque::from([start]);
    let mut visited = HashSet::from([start]);
    while let Some(current) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(&current) {
            for next in neighbors {
                if visited.insert(*next) {
                    queue.push_back(*next);
                }
            }
        }
    }

    cache.insert(start, visited.clone());
    visited
}

fn deterministic_pairs(ordered_ids: &[u64], sample_count: usize) -> Vec<(u64, u64)> {
    if ordered_ids.len() < 2 || sample_count == 0 {
        return Vec::new();
    }

    let n = ordered_ids.len();
    let max_pairs = n.saturating_mul(n.saturating_sub(1)) / 2;
    let target = sample_count.min(max_pairs);

    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut seen = BTreeSet::new();
    let mut pairs = Vec::with_capacity(target);
    let mut attempts = 0_usize;
    let max_attempts = target.saturating_mul(20).max(128);

    while pairs.len() < target && attempts < max_attempts {
        attempts += 1;
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let i = (state as usize) % (n - 1);
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let gap = ((state as usize) % (n - i - 1)) + 1;
        let j = i + gap;
        if seen.insert((i, j)) {
            pairs.push((ordered_ids[i], ordered_ids[j]));
        }
    }

    if pairs.len() < target {
        for i in 0..(n - 1) {
            for j in (i + 1)..n {
                if seen.insert((i, j)) {
                    pairs.push((ordered_ids[i], ordered_ids[j]));
                    if pairs.len() == target {
                        return pairs;
                    }
                }
            }
        }
    }

    pairs
}

fn write_path_lengths_csv(path: &Path, graph: &DscdGraph, root_event_id: u64) -> Result<()> {
    let mut events = graph.events.clone();
    events.sort_by_key(|event| event.time_index);

    let mut adjacency: HashMap<u64, Vec<u64>> = HashMap::new();
    for edge in &graph.edges {
        adjacency.entry(edge.src).or_default().push(edge.dst);
    }

    let mut distances: HashMap<u64, Option<usize>> =
        events.iter().map(|event| (event.id, None)).collect();
    if distances.contains_key(&root_event_id) {
        distances.insert(root_event_id, Some(0));
    }

    for event in &events {
        let Some(Some(dist)) = distances.get(&event.id).copied() else {
            continue;
        };

        if let Some(neighbors) = adjacency.get(&event.id) {
            for next in neighbors {
                let candidate = dist + 1;
                let current = distances.get(next).copied().flatten();
                if current.map_or(true, |value| candidate > value) {
                    distances.insert(*next, Some(candidate));
                }
            }
        }
    }

    let mut writer = Writer::from_path(path)?;
    writer.write_record(["event_id", "longest_path_length"])?;
    for event in &events {
        writer.write_record([
            event.id.to_string(),
            distances
                .get(&event.id)
                .copied()
                .flatten()
                .unwrap_or(0)
                .to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_edge_provenance_csv(path: &Path, graph: &DscdGraph, limit: usize) -> Result<()> {
    let event_by_id: HashMap<u64, &DscdEvent> =
        graph.events.iter().map(|event| (event.id, event)).collect();
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "src",
        "dst",
        "trust_at_creation",
        "module_id",
        "rewrite_rule_id",
        "residual_state_src",
        "residual_state_dst",
        "time_index_src",
        "time_index_dst",
    ])?;

    for edge in graph.edges.iter().take(limit) {
        let Some(src) = event_by_id.get(&edge.src) else {
            continue;
        };
        let Some(dst) = event_by_id.get(&edge.dst) else {
            continue;
        };
        writer.write_record([
            edge.src.to_string(),
            edge.dst.to_string(),
            edge.trust_at_creation.to_string(),
            edge.module_id.to_string(),
            edge.rewrite_rule_id.clone().unwrap_or_default(),
            src.residual_state.clone(),
            dst.residual_state.clone(),
            src.time_index.to_string(),
            dst.time_index.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

/// Create a timestamped DSCD run directory under an output root.
pub fn create_dscd_run_dir(output_root: &Path) -> Result<PathBuf> {
    fs::create_dir_all(output_root).with_context(|| {
        format!(
            "failed to create DSCD output root {}",
            output_root.display()
        )
    })?;

    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    let mut run_dir = output_root.join(&stamp);
    let mut suffix = 1_usize;
    while run_dir.exists() {
        run_dir = output_root.join(format!("{stamp}_{suffix:02}"));
        suffix += 1;
    }

    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;
    Ok(run_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn trust_gate_respects_tau_and_time_order() {
        let events = vec![
            DscdEvent {
                id: 0,
                time_index: 0,
                module_id: 0,
                residual_state: "L".to_string(),
                trust: 0.9,
                echo_slope: 0.1,
                entropy_density: 0.2,
            },
            DscdEvent {
                id: 1,
                time_index: 1,
                module_id: 0,
                residual_state: "M".to_string(),
                trust: 0.7,
                echo_slope: 0.2,
                entropy_density: 0.3,
            },
        ];
        let edges = vec![
            DscdEdge {
                src: 0,
                dst: 1,
                trust_at_creation: 0.8,
                module_id: 0,
                rewrite_rule_id: Some("rule_a".to_string()),
            },
            DscdEdge {
                src: 1,
                dst: 0,
                trust_at_creation: 0.9,
                module_id: 0,
                rewrite_rule_id: Some("rule_b".to_string()),
            },
        ];

        let graph = build_graph_for_tau(&events, &edges, 0.75);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].src, 0);
        assert_eq!(graph.edges[0].dst, 1);
    }

    #[test]
    fn tiny_scaling_run_writes_required_outputs() {
        let out_dir = std::env::temp_dir().join(format!(
            "dsfb-dscd-sim-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));

        let cfg = DscdConfig {
            num_events: 128,
            taus: vec![0.0, 0.5, 1.0],
            root_event_id: 0,
            output_dir: out_dir.clone(),
            scaling_ns: vec![64, 128],
            quick_mode: true,
        };
        run_dscd_simulation(&cfg).expect("simulation should run");

        assert!(out_dir.join("threshold_scaling_summary.csv").exists());
        assert!(out_dir.join("threshold_curve_N_64.csv").exists());
        assert!(out_dir.join("threshold_curve_N_128.csv").exists());
        assert!(out_dir.join("graph_events.csv").exists());
        assert!(out_dir.join("graph_edges.csv").exists());
        assert!(out_dir.join("degree_distribution.csv").exists());
        assert!(out_dir.join("interval_sizes.csv").exists());
        assert!(out_dir.join("path_lengths.csv").exists());
        assert!(out_dir.join("edge_provenance.csv").exists());

        let mut reader = csv::Reader::from_path(out_dir.join("threshold_scaling_summary.csv"))
            .expect("summary csv");
        for row in reader.records() {
            let row = row.expect("csv row");
            let tau_star: f64 = row.get(1).expect("tau_star").parse().expect("parse tau");
            let width: f64 = row.get(2).expect("width").parse().expect("parse width");
            assert!((0.0..=1.0).contains(&tau_star));
            assert!(width >= 0.0);
        }

        let _ = fs::remove_dir_all(out_dir);
    }
}
