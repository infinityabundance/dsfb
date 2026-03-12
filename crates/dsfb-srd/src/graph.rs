use std::collections::VecDeque;

use crate::compatibility::compatible;
use crate::config::SimulationConfig;
use crate::event::StructuralEvent;
use crate::metrics::component_entropy;

#[derive(Clone, Debug)]
pub struct CandidateEdge {
    pub src: usize,
    pub dst: usize,
    pub compatible: bool,
}

#[derive(Clone, Debug)]
pub struct CandidateGraph {
    pub n_events: usize,
    pub outgoing: Vec<Vec<usize>>,
    pub candidate_edges: Vec<CandidateEdge>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DirectedGraphStats {
    pub reachable_count: usize,
    pub edge_count: usize,
    pub mean_out_degree: f64,
    pub largest_component_fraction: f64,
    pub component_entropy: f64,
}

pub fn build_candidate_graph(
    events: &[StructuralEvent],
    config: &SimulationConfig,
) -> CandidateGraph {
    let mut outgoing = vec![Vec::new(); events.len()];
    let mut candidate_edges = Vec::new();

    for src in 0..events.len() {
        let upper = (src + config.causal_window + 1).min(events.len());
        for dst in (src + 1)..upper {
            if compatible(&events[src], &events[dst], config.n_channels) {
                let edge_index = candidate_edges.len();
                candidate_edges.push(CandidateEdge {
                    src,
                    dst,
                    compatible: true,
                });
                outgoing[src].push(edge_index);
            }
        }
    }

    CandidateGraph {
        n_events: events.len(),
        outgoing,
        candidate_edges,
    }
}

pub fn compute_graph_stats(
    candidate_graph: &CandidateGraph,
    events: &[StructuralEvent],
    tau_threshold: f64,
    anchor: usize,
) -> DirectedGraphStats {
    compute_graph_stats_in_range(
        candidate_graph,
        events,
        tau_threshold,
        anchor,
        (0, candidate_graph.n_events),
    )
}

pub fn compute_graph_stats_in_range(
    candidate_graph: &CandidateGraph,
    events: &[StructuralEvent],
    tau_threshold: f64,
    anchor: usize,
    range: (usize, usize),
) -> DirectedGraphStats {
    let (start, end) = range;
    if start >= end || end > candidate_graph.n_events || anchor < start || anchor >= end {
        return DirectedGraphStats::default();
    }

    let window_len = end - start;
    let mut reachable = vec![false; candidate_graph.n_events];
    let mut queue = VecDeque::new();
    reachable[anchor] = true;
    queue.push_back(anchor);

    while let Some(node) = queue.pop_front() {
        if !source_is_active(events, node, tau_threshold) {
            continue;
        }

        for &edge_index in &candidate_graph.outgoing[node] {
            let edge = &candidate_graph.candidate_edges[edge_index];
            if edge.dst < start || edge.dst >= end {
                continue;
            }
            if !reachable[edge.dst] {
                reachable[edge.dst] = true;
                queue.push_back(edge.dst);
            }
        }
    }

    let reachable_count = (start..end).filter(|&event_id| reachable[event_id]).count();
    let (undirected, edge_count) =
        thresholded_weak_adjacency(candidate_graph, events, tau_threshold, (start, end));
    let component_sizes = weak_component_sizes(&undirected);

    DirectedGraphStats {
        reachable_count,
        edge_count,
        mean_out_degree: edge_count as f64 / window_len as f64,
        largest_component_fraction: largest_component_fraction(&component_sizes, window_len),
        component_entropy: component_entropy(&component_sizes, window_len),
    }
}

pub fn collect_active_edges(
    candidate_graph: &CandidateGraph,
    events: &[StructuralEvent],
    tau_threshold: f64,
    range: Option<(usize, usize)>,
) -> Vec<CandidateEdge> {
    let (start, end) = range.unwrap_or((0, candidate_graph.n_events));
    let mut active_edges = Vec::new();

    for src in start..end {
        if !source_is_active(events, src, tau_threshold) {
            continue;
        }
        for &edge_index in &candidate_graph.outgoing[src] {
            let edge = &candidate_graph.candidate_edges[edge_index];
            if edge.dst < start || edge.dst >= end {
                continue;
            }
            active_edges.push(edge.clone());
        }
    }

    active_edges
}

pub fn weak_component_sizes(undirected: &[Vec<usize>]) -> Vec<usize> {
    if undirected.is_empty() {
        return Vec::new();
    }

    let mut visited = vec![false; undirected.len()];
    let mut component_sizes = Vec::new();

    for node in 0..undirected.len() {
        if visited[node] {
            continue;
        }

        let mut queue = VecDeque::new();
        queue.push_back(node);
        visited[node] = true;
        let mut size = 0usize;

        while let Some(current) = queue.pop_front() {
            size += 1;
            for &next in &undirected[current] {
                if !visited[next] {
                    visited[next] = true;
                    queue.push_back(next);
                }
            }
        }

        component_sizes.push(size);
    }

    component_sizes
}

fn thresholded_weak_adjacency(
    candidate_graph: &CandidateGraph,
    events: &[StructuralEvent],
    tau_threshold: f64,
    range: (usize, usize),
) -> (Vec<Vec<usize>>, usize) {
    let (start, end) = range;
    let mut edge_count = 0usize;
    let mut undirected = vec![Vec::new(); end.saturating_sub(start)];

    for src in start..end {
        if !source_is_active(events, src, tau_threshold) {
            continue;
        }

        for &edge_index in &candidate_graph.outgoing[src] {
            let edge = &candidate_graph.candidate_edges[edge_index];
            if edge.dst < start || edge.dst >= end {
                continue;
            }

            edge_count += 1;
            let src_local = src - start;
            let dst_local = edge.dst - start;
            undirected[src_local].push(dst_local);
            undirected[dst_local].push(src_local);
        }
    }

    (undirected, edge_count)
}

fn largest_component_fraction(component_sizes: &[usize], total_nodes: usize) -> f64 {
    if total_nodes == 0 {
        return 0.0;
    }

    component_sizes.iter().copied().max().unwrap_or(0) as f64 / total_nodes as f64
}

fn source_is_active(events: &[StructuralEvent], event_id: usize, tau_threshold: f64) -> bool {
    events[event_id].trust >= tau_threshold
}

#[cfg(test)]
mod tests {
    use super::weak_component_sizes;

    #[test]
    fn weak_component_sizes_include_isolates() {
        let undirected = vec![vec![1], vec![0], vec![], vec![4], vec![3]];
        let mut sizes = weak_component_sizes(&undirected);
        sizes.sort_unstable();
        assert_eq!(sizes, vec![1, 2, 2]);
    }
}
