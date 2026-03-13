use std::collections::{HashMap, VecDeque};

use serde::Serialize;

use crate::observer::ObserverSeries;
use crate::tmtr::CorrectionEvent;

#[derive(Debug, Clone, Serialize)]
pub struct CausalNode {
    pub id: String,
    pub time: f64,
    pub level: usize,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CausalEdge {
    pub scenario: String,
    pub mode: String,
    pub edge_type: String,
    pub source_node: String,
    pub source_time: f64,
    pub source_level: usize,
    pub target_node: String,
    pub target_time: f64,
    pub target_level: usize,
    pub trust_weight: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CausalGraph {
    pub nodes: Vec<CausalNode>,
    pub edges: Vec<CausalEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CausalMetricsSummary {
    pub edge_count: usize,
    pub backward_edge_count: usize,
    pub cycle_count: usize,
    pub reachable_nodes_from_anchor: usize,
    pub local_window_edge_density: f64,
    pub max_in_degree: usize,
    pub max_out_degree: usize,
    pub max_path_length: usize,
    pub mean_path_length: f64,
}

pub fn build_causal_graph(
    scenario: &str,
    mode: &str,
    observers: &[ObserverSeries],
    correction_events: &[CorrectionEvent],
    min_trust_gap: f64,
) -> CausalGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for observer in observers {
        for step in 0..observer.estimate.len() {
            nodes.push(CausalNode {
                id: state_node_id(observer.level, step),
                time: step as f64,
                level: observer.level,
                kind: "state".to_string(),
            });
            if step > 0 {
                edges.push(CausalEdge {
                    scenario: scenario.to_string(),
                    mode: mode.to_string(),
                    edge_type: "state_propagation".to_string(),
                    source_node: state_node_id(observer.level, step - 1),
                    source_time: (step - 1) as f64,
                    source_level: observer.level,
                    target_node: state_node_id(observer.level, step),
                    target_time: step as f64,
                    target_level: observer.level,
                    trust_weight: observer.trust[step],
                });
            }
        }
    }

    for step in 0..observers[0].estimate.len().saturating_sub(1) {
        for source_index in (1..observers.len()).rev() {
            let target_index = source_index - 1;
            let source = &observers[source_index];
            let target = &observers[target_index];
            let trust = source.trust[step];
            if trust > target.trust[step] + min_trust_gap {
                edges.push(CausalEdge {
                    scenario: scenario.to_string(),
                    mode: mode.to_string(),
                    edge_type: "trust_gate".to_string(),
                    source_node: state_node_id(source.level, step),
                    source_time: step as f64,
                    source_level: source.level,
                    target_node: state_node_id(target.level, step + 1),
                    target_time: (step + 1) as f64,
                    target_level: target.level,
                    trust_weight: trust,
                });
            }
        }
    }

    for (index, event) in correction_events.iter().enumerate() {
        let correction_time = event.anchor_time as f64 + 0.1;
        let correction_node = format!("corr:{index}");
        let commit_time = correction_time + 0.1;
        let commit_node = format!("commit:{index}");
        nodes.push(CausalNode {
            id: correction_node.clone(),
            time: correction_time,
            level: event.target_level,
            kind: "correction".to_string(),
        });
        nodes.push(CausalNode {
            id: commit_node.clone(),
            time: commit_time,
            level: event.target_level,
            kind: "commit".to_string(),
        });
        edges.push(CausalEdge {
            scenario: scenario.to_string(),
            mode: mode.to_string(),
            edge_type: "correction_source".to_string(),
            source_node: state_node_id(event.source_level, event.anchor_time),
            source_time: event.anchor_time as f64,
            source_level: event.source_level,
            target_node: correction_node.clone(),
            target_time: correction_time,
            target_level: event.target_level,
            trust_weight: event.trust_weight,
        });
        edges.push(CausalEdge {
            scenario: scenario.to_string(),
            mode: mode.to_string(),
            edge_type: "correction_context".to_string(),
            source_node: state_node_id(event.target_level, event.corrected_time),
            source_time: event.corrected_time as f64,
            source_level: event.target_level,
            target_node: correction_node.clone(),
            target_time: correction_time,
            target_level: event.target_level,
            trust_weight: event.trust_weight,
        });
        edges.push(CausalEdge {
            scenario: scenario.to_string(),
            mode: mode.to_string(),
            edge_type: "correction_commit".to_string(),
            source_node: correction_node,
            source_time: correction_time,
            source_level: event.target_level,
            target_node: commit_node,
            target_time: commit_time,
            target_level: event.target_level,
            trust_weight: event.trust_weight,
        });
    }

    CausalGraph { nodes, edges }
}

pub fn summarize_causal_graph(graph: &CausalGraph, delta: usize) -> CausalMetricsSummary {
    let edge_count = graph.edges.len();
    let backward_edge_count = graph
        .edges
        .iter()
        .filter(|edge| edge.target_time < edge.source_time)
        .count();
    let cycle_count = if has_cycle(&graph.nodes, &graph.edges) {
        1
    } else {
        0
    };

    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut indegree: HashMap<&str, usize> = HashMap::new();
    let mut outdegree: HashMap<&str, usize> = HashMap::new();
    let node_times: HashMap<&str, f64> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.time))
        .collect();
    for node in &graph.nodes {
        indegree.entry(node.id.as_str()).or_insert(0);
        outdegree.entry(node.id.as_str()).or_insert(0);
    }
    for edge in &graph.edges {
        adjacency
            .entry(edge.source_node.as_str())
            .or_default()
            .push(edge.target_node.as_str());
        *indegree.entry(edge.target_node.as_str()).or_insert(0) += 1;
        *outdegree.entry(edge.source_node.as_str()).or_insert(0) += 1;
    }

    let max_in_degree = indegree.values().copied().max().unwrap_or(0);
    let max_out_degree = outdegree.values().copied().max().unwrap_or(0);

    let anchor_node = graph
        .nodes
        .iter()
        .find(|node| node.kind == "correction")
        .or_else(|| graph.nodes.first());
    let reachable_nodes_from_anchor = anchor_node
        .map(|node| reachable_count(node.id.as_str(), &adjacency))
        .unwrap_or(0);

    let local_window_edges = graph
        .edges
        .iter()
        .filter(|edge| edge.target_time - edge.source_time <= delta as f64 + 1.0)
        .count();
    let node_count = graph.nodes.len().max(1);
    let possible_local_edges = node_count * delta.max(1);
    let local_window_edge_density = local_window_edges as f64 / possible_local_edges as f64;

    let longest_paths = longest_path_lengths(&graph.nodes, &graph.edges, &node_times);
    let max_path_length = longest_paths.values().copied().max().unwrap_or(0);
    let mean_path_length = if longest_paths.is_empty() {
        0.0
    } else {
        longest_paths
            .values()
            .copied()
            .map(|value| value as f64)
            .sum::<f64>()
            / longest_paths.len() as f64
    };

    CausalMetricsSummary {
        edge_count,
        backward_edge_count,
        cycle_count,
        reachable_nodes_from_anchor,
        local_window_edge_density,
        max_in_degree,
        max_out_degree,
        max_path_length,
        mean_path_length,
    }
}

fn state_node_id(level: usize, step: usize) -> String {
    format!("state:L{level}:t{step}")
}

fn reachable_count<'a>(anchor: &'a str, adjacency: &HashMap<&'a str, Vec<&'a str>>) -> usize {
    let mut seen = HashMap::<&str, bool>::new();
    let mut queue = VecDeque::new();
    queue.push_back(anchor);
    while let Some(node) = queue.pop_front() {
        if seen.insert(node, true).is_some() {
            continue;
        }
        if let Some(children) = adjacency.get(node) {
            for child in children {
                queue.push_back(child);
            }
        }
    }
    seen.len()
}

fn has_cycle(nodes: &[CausalNode], edges: &[CausalEdge]) -> bool {
    let mut indegree = HashMap::<&str, usize>::new();
    let mut adjacency = HashMap::<&str, Vec<&str>>::new();
    for node in nodes {
        indegree.insert(node.id.as_str(), 0);
    }
    for edge in edges {
        *indegree.entry(edge.target_node.as_str()).or_insert(0) += 1;
        adjacency
            .entry(edge.source_node.as_str())
            .or_default()
            .push(edge.target_node.as_str());
    }
    let mut queue = VecDeque::new();
    for (node, degree) in &indegree {
        if *degree == 0 {
            queue.push_back(*node);
        }
    }
    let mut visited = 0usize;
    while let Some(node) = queue.pop_front() {
        visited += 1;
        if let Some(children) = adjacency.get(node) {
            for child in children {
                if let Some(entry) = indegree.get_mut(child) {
                    *entry = entry.saturating_sub(1);
                    if *entry == 0 {
                        queue.push_back(child);
                    }
                }
            }
        }
    }
    visited != indegree.len()
}

fn longest_path_lengths<'a>(
    nodes: &'a [CausalNode],
    edges: &'a [CausalEdge],
    node_times: &HashMap<&'a str, f64>,
) -> HashMap<&'a str, usize> {
    let mut adjacency = HashMap::<&str, Vec<&str>>::new();
    let mut indegree = HashMap::<&str, usize>::new();
    for node in nodes {
        indegree.insert(node.id.as_str(), 0);
    }
    for edge in edges {
        adjacency
            .entry(edge.source_node.as_str())
            .or_default()
            .push(edge.target_node.as_str());
        *indegree.entry(edge.target_node.as_str()).or_insert(0) += 1;
    }
    let mut queue = VecDeque::new();
    let mut distance = HashMap::<&str, usize>::new();
    let mut ordered = nodes.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.time.total_cmp(&right.time));
    for node in &ordered {
        distance.insert(node.id.as_str(), 0);
        if indegree.get(node.id.as_str()).copied().unwrap_or(0) == 0 {
            queue.push_back(node.id.as_str());
        }
    }
    while let Some(node) = queue.pop_front() {
        let source_distance = distance.get(node).copied().unwrap_or(0);
        if let Some(children) = adjacency.get(node) {
            for child in children {
                let source_time = node_times.get(node).copied().unwrap_or_default();
                let target_time = node_times.get(child).copied().unwrap_or_default();
                if target_time >= source_time {
                    let candidate = source_distance + 1;
                    if candidate > distance.get(child).copied().unwrap_or(0) {
                        distance.insert(child, candidate);
                    }
                }
                if let Some(entry) = indegree.get_mut(child) {
                    *entry = entry.saturating_sub(1);
                    if *entry == 0 {
                        queue.push_back(child);
                    }
                }
            }
        }
    }
    distance
}

#[cfg(test)]
mod tests {
    use super::{summarize_causal_graph, CausalEdge, CausalGraph, CausalNode};

    #[test]
    fn forward_edges_do_not_trigger_backward_detection() {
        let graph = CausalGraph {
            nodes: vec![
                CausalNode {
                    id: "a".to_string(),
                    time: 0.0,
                    level: 1,
                    kind: "state".to_string(),
                },
                CausalNode {
                    id: "b".to_string(),
                    time: 1.0,
                    level: 1,
                    kind: "state".to_string(),
                },
            ],
            edges: vec![CausalEdge {
                scenario: "test".to_string(),
                mode: "tmtr".to_string(),
                edge_type: "state".to_string(),
                source_node: "a".to_string(),
                source_time: 0.0,
                source_level: 1,
                target_node: "b".to_string(),
                target_time: 1.0,
                target_level: 1,
                trust_weight: 1.0,
            }],
        };
        let summary = summarize_causal_graph(&graph, 4);
        assert_eq!(summary.backward_edge_count, 0);
        assert_eq!(summary.cycle_count, 0);
    }
}
