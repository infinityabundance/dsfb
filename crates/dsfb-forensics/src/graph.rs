//! Deterministic causal graph construction and fragmentation metrics.
//!
//! References: `CORE-04` for trust-threshold graph gating, `DSCD-01` for
//! topological ordering, `DSCD-05` and `DSCD-07` for admissible pruning, and
//! `DSCD-09` for finite propagation over a DAG.

use std::cmp::Ordering;
use std::collections::VecDeque;

use dsfb::DsfbState;
use serde::Serialize;

/// Channel-local audit measurements used to build the causal graph.
#[derive(Clone, Debug)]
pub struct ChannelAuditInput {
    /// Channel index.
    pub index: usize,
    /// Measured scalar observation.
    pub measurement: f64,
    /// DSFB residual for the channel.
    pub residual: f64,
    /// Raw DSFB trust weight.
    pub raw_trust_weight: f64,
    /// Monotone forensic trust score.
    pub trust_score: f64,
    /// Measured second-difference slew.
    pub measurement_slew: f64,
    /// Deterministic envelope used by the auditor.
    pub deterministic_envelope: f64,
}

/// Graph-wide metrics used by the report and JSON trace.
#[derive(Clone, Debug, Serialize)]
pub struct GraphMetrics {
    /// Vertex count including the fused root.
    pub vertex_count: usize,
    /// Directed edge count.
    pub edge_count: usize,
    /// Weakly connected component count.
    pub connected_components: usize,
    /// Size of the largest weakly connected component.
    pub largest_component: usize,
    /// Longest reachable causal depth from the root.
    pub max_causal_depth: usize,
    /// Whether the graph is fragmented.
    pub fragmented: bool,
}

/// Deterministic causal graph and its derived metrics.
#[derive(Clone, Debug)]
pub struct CausalGraph {
    causal_depths: Vec<usize>,
    metrics: GraphMetrics,
}

impl CausalGraph {
    /// References: `DSCD-01` and `DSCD-09`.
    pub fn causal_depth(&self, vertex: usize) -> usize {
        self.causal_depths[vertex]
    }

    /// References: `CORE-04` and `DSCD-05`.
    pub fn metrics(&self) -> &GraphMetrics {
        &self.metrics
    }
}

/// Build the trust-gated causal graph for one audit step.
///
/// References: `CORE-04`, `DSCD-01`, `DSCD-05`, and `DSCD-07`.
pub fn build_causal_graph(
    state: DsfbState,
    channels: &[ChannelAuditInput],
    dt: f64,
    trust_alpha: f64,
) -> CausalGraph {
    let vertex_count = channels.len() + 1;
    let mut adjacency = vec![Vec::new(); vertex_count];
    let mut ranking: Vec<usize> = (0..channels.len()).collect();
    ranking.sort_by(|left, right| compare_channels(&channels[*left], &channels[*right]));

    for &channel_index in &ranking {
        if channel_is_admissible(&channels[channel_index], state, dt, trust_alpha) {
            adjacency[0].push(channel_index + 1);
        }
    }

    for (position, &source_index) in ranking.iter().enumerate() {
        if !channel_is_admissible(&channels[source_index], state, dt, trust_alpha) {
            continue;
        }
        for &target_index in ranking.iter().skip(position + 1) {
            if !channel_is_admissible(&channels[target_index], state, dt, trust_alpha) {
                continue;
            }
            if pair_is_admissible(state, &channels[source_index], &channels[target_index], dt) {
                adjacency[source_index + 1].push(target_index + 1);
            }
        }
    }

    let causal_depths = compute_causal_depths(&adjacency);
    let (connected_components, largest_component) = weak_component_metrics(&adjacency);
    let edge_count = adjacency.iter().map(Vec::len).sum();
    let max_causal_depth = *causal_depths.iter().max().unwrap_or(&0);
    let metrics = GraphMetrics {
        vertex_count,
        edge_count,
        connected_components,
        largest_component,
        max_causal_depth,
        fragmented: connected_components > 1,
    };

    CausalGraph {
        causal_depths,
        metrics,
    }
}

fn compare_channels(left: &ChannelAuditInput, right: &ChannelAuditInput) -> Ordering {
    right
        .trust_score
        .partial_cmp(&left.trust_score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.index.cmp(&right.index))
}

fn channel_is_admissible(
    channel: &ChannelAuditInput,
    state: DsfbState,
    dt: f64,
    trust_alpha: f64,
) -> bool {
    channel.trust_score >= trust_alpha
        && channel.measurement_slew <= channel.deterministic_envelope
        && channel.residual.abs() <= residual_band(state, channel, dt)
}

fn pair_is_admissible(
    state: DsfbState,
    source: &ChannelAuditInput,
    target: &ChannelAuditInput,
    dt: f64,
) -> bool {
    let displacement_band = dt * (state.omega.abs() + 1.0)
        + 0.5 * dt * dt * (state.alpha.abs() + source.deterministic_envelope.max(target.deterministic_envelope));
    let trust_gate = source.trust_score >= target.trust_score;
    let slew_gate = source.measurement_slew <= source.deterministic_envelope
        && target.measurement_slew <= target.deterministic_envelope;
    trust_gate && slew_gate && (source.measurement - target.measurement).abs() <= displacement_band.max(0.05)
}

fn residual_band(state: DsfbState, channel: &ChannelAuditInput, dt: f64) -> f64 {
    let motion_band = dt * (state.omega.abs() + 1.0) + 0.5 * dt * dt * (state.alpha.abs() + channel.deterministic_envelope);
    let trust_band = (1.0 - channel.raw_trust_weight).max(0.0) * 0.5;
    motion_band + trust_band + 0.05
}

fn compute_causal_depths(adjacency: &[Vec<usize>]) -> Vec<usize> {
    let mut depths = vec![0usize; adjacency.len()];
    let mut queue = VecDeque::new();
    queue.push_back(0usize);
    while let Some(vertex) = queue.pop_front() {
        let next_depth = depths[vertex] + 1;
        for &neighbor in &adjacency[vertex] {
            if next_depth > depths[neighbor] {
                depths[neighbor] = next_depth;
                queue.push_back(neighbor);
            }
        }
    }
    depths
}

fn weak_component_metrics(adjacency: &[Vec<usize>]) -> (usize, usize) {
    let undirected = build_undirected(adjacency);
    let mut visited = vec![false; adjacency.len()];
    let mut components = 0usize;
    let mut largest = 0usize;
    for vertex in 0..adjacency.len() {
        if visited[vertex] {
            continue;
        }
        components += 1;
        let mut size = 0usize;
        let mut queue = VecDeque::new();
        queue.push_back(vertex);
        visited[vertex] = true;
        while let Some(current) = queue.pop_front() {
            size += 1;
            for &neighbor in &undirected[current] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        largest = largest.max(size);
    }
    (components, largest)
}

fn build_undirected(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut undirected = vec![Vec::new(); adjacency.len()];
    for (source, targets) in adjacency.iter().enumerate() {
        for &target in targets {
            undirected[source].push(target);
            undirected[target].push(source);
        }
    }
    undirected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragmented_graph_is_detected() {
        let state = DsfbState::new(0.0, 0.0, 0.0);
        let channels = vec![
            ChannelAuditInput {
                index: 0,
                measurement: 0.0,
                residual: 0.0,
                raw_trust_weight: 0.6,
                trust_score: 0.9,
                measurement_slew: 0.1,
                deterministic_envelope: 1.0,
            },
            ChannelAuditInput {
                index: 1,
                measurement: 5.0,
                residual: 4.5,
                raw_trust_weight: 0.1,
                trust_score: 0.05,
                measurement_slew: 9.0,
                deterministic_envelope: 1.0,
            },
        ];
        let graph = build_causal_graph(state, &channels, 0.1, 0.2);
        assert!(graph.metrics().fragmented);
    }
}
