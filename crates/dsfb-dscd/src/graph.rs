use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventId(pub u64);

#[derive(Debug, Clone)]
pub struct Event {
    pub id: EventId,
    pub timestamp: Option<f64>,
    pub structural_tag: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct DscdEdge {
    pub edge_id: u64,
    pub from: EventId,
    pub to: EventId,
    pub observer_id: u32,
    pub trust_value: f64,
    pub trust_at_creation: f64,
    pub rewrite_rule_at_source: u32,
}

#[derive(Debug, Default, Clone)]
pub struct DscdGraph {
    pub events: Vec<Event>,
    pub edges: Vec<DscdEdge>,
}

impl DscdGraph {
    pub fn add_event(&mut self, event: Event) {
        if let Some(existing) = self
            .events
            .iter_mut()
            .find(|candidate| candidate.id == event.id)
        {
            *existing = event;
            return;
        }

        self.events.push(event);
        self.events.sort_by_key(|candidate| candidate.id);
    }

    pub fn has_event(&self, event_id: EventId) -> bool {
        self.events.iter().any(|event| event.id == event_id)
    }
}

pub fn add_trust_gated_edge(
    graph: &mut DscdGraph,
    from: EventId,
    to: EventId,
    observer_id: u32,
    trust_value: f64,
    trust_threshold: f64,
) {
    add_trust_gated_edge_with_provenance(
        graph,
        from,
        to,
        observer_id,
        trust_value,
        trust_threshold,
        0,
    );
}

pub fn add_trust_gated_edge_with_provenance(
    graph: &mut DscdGraph,
    from: EventId,
    to: EventId,
    observer_id: u32,
    trust_value: f64,
    trust_threshold: f64,
    rewrite_rule_at_source: u32,
) {
    if from >= to || trust_value < trust_threshold {
        return;
    }

    if !(graph.has_event(from) && graph.has_event(to)) {
        return;
    }

    if graph
        .edges
        .iter()
        .any(|edge| edge.from == from && edge.to == to && edge.observer_id == observer_id)
    {
        return;
    }

    if reachable_from(graph, to, None).contains(&from) {
        return;
    }

    let edge_id = graph.edges.len() as u64;
    graph.edges.push(DscdEdge {
        edge_id,
        from,
        to,
        observer_id,
        trust_value,
        trust_at_creation: trust_value,
        rewrite_rule_at_source,
    });
}

pub fn reachable_from(graph: &DscdGraph, start: EventId, max_depth: Option<usize>) -> Vec<EventId> {
    if !graph.has_event(start) {
        return Vec::new();
    }

    let mut adjacency: HashMap<EventId, Vec<EventId>> = HashMap::new();
    for edge in &graph.edges {
        adjacency.entry(edge.from).or_default().push(edge.to);
    }
    for neighbors in adjacency.values_mut() {
        neighbors.sort_unstable();
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([(start, 0_usize)]);
    visited.insert(start);

    while let Some((node, depth)) = queue.pop_front() {
        if max_depth.is_some_and(|limit| depth >= limit) {
            continue;
        }

        if let Some(neighbors) = adjacency.get(&node) {
            for &next in neighbors {
                if visited.insert(next) {
                    queue.push_back((next, depth + 1));
                }
            }
        }
    }

    let mut out: Vec<_> = visited.into_iter().collect();
    out.sort_unstable();
    out
}

pub fn expansion_ratio(graph: &DscdGraph, start: EventId) -> f64 {
    if graph.events.is_empty() {
        return 0.0;
    }

    reachable_from(graph, start, None).len() as f64 / graph.events.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toy_graph() -> DscdGraph {
        let mut graph = DscdGraph::default();
        for raw_id in 0..4_u64 {
            graph.add_event(Event {
                id: EventId(raw_id),
                timestamp: Some(raw_id as f64),
                structural_tag: None,
            });
        }
        graph
    }

    #[test]
    fn trust_gated_edges_remain_acyclic() {
        let mut graph = toy_graph();
        add_trust_gated_edge(&mut graph, EventId(0), EventId(1), 0, 0.9, 0.5);
        add_trust_gated_edge(&mut graph, EventId(1), EventId(2), 0, 0.9, 0.5);
        add_trust_gated_edge(&mut graph, EventId(2), EventId(0), 0, 0.9, 0.5);
        add_trust_gated_edge(&mut graph, EventId(2), EventId(2), 0, 0.9, 0.5);

        assert_eq!(graph.edges.len(), 2);
        assert!(graph.edges.iter().all(|edge| edge.from < edge.to));
    }

    #[test]
    fn reachable_nodes_are_sorted_and_include_start() {
        let mut graph = toy_graph();
        add_trust_gated_edge(&mut graph, EventId(0), EventId(1), 0, 0.9, 0.5);
        add_trust_gated_edge(&mut graph, EventId(1), EventId(3), 0, 0.9, 0.5);

        assert_eq!(
            reachable_from(&graph, EventId(0), None),
            vec![EventId(0), EventId(1), EventId(3)]
        );
    }
}
