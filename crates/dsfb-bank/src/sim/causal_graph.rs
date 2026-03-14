use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub id: String,
    pub node_count: usize,
    pub edges: BTreeSet<(usize, usize)>,
}

impl Graph {
    pub fn new(id: &str, node_count: usize, edges: &[(usize, usize)]) -> Self {
        Self {
            id: id.to_string(),
            node_count,
            edges: edges.iter().copied().collect(),
        }
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn add_edge(&self, id: &str, edge: (usize, usize)) -> Self {
        let mut edges = self.edges.clone();
        edges.insert(edge);
        Self {
            id: id.to_string(),
            node_count: self.node_count,
            edges,
        }
    }

    pub fn remove_edges(&self, id: &str, removed: &[(usize, usize)]) -> Self {
        let mut edges = self.edges.clone();
        for edge in removed {
            edges.remove(edge);
        }
        Self {
            id: id.to_string(),
            node_count: self.node_count,
            edges,
        }
    }

    pub fn union(&self, id: &str, other: &Self) -> Self {
        let mut edges = self.edges.clone();
        edges.extend(other.edges.iter().copied());
        Self {
            id: id.to_string(),
            node_count: self.node_count.max(other.node_count),
            edges,
        }
    }

    pub fn adjacency(&self) -> Vec<Vec<usize>> {
        let mut adjacency = vec![Vec::new(); self.node_count];
        for &(from, to) in &self.edges {
            adjacency[from].push(to);
        }
        for neighbors in &mut adjacency {
            neighbors.sort_unstable();
        }
        adjacency
    }

    pub fn indegrees(&self) -> Vec<usize> {
        let mut indegrees = vec![0; self.node_count];
        for &(_, to) in &self.edges {
            indegrees[to] += 1;
        }
        indegrees
    }

    pub fn topo_order(&self) -> Option<Vec<usize>> {
        let adjacency = self.adjacency();
        let mut indegrees = self.indegrees();
        let mut queue = VecDeque::new();
        for (node, indegree) in indegrees.iter().enumerate() {
            if *indegree == 0 {
                queue.push_back(node);
            }
        }

        let mut order = Vec::with_capacity(self.node_count);
        while let Some(node) = queue.pop_front() {
            order.push(node);
            for &neighbor in &adjacency[node] {
                indegrees[neighbor] -= 1;
                if indegrees[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        if order.len() == self.node_count {
            Some(order)
        } else {
            None
        }
    }

    pub fn is_acyclic(&self) -> bool {
        self.topo_order().is_some()
    }

    pub fn reachability(&self) -> BTreeSet<(usize, usize)> {
        let adjacency = self.adjacency();
        let mut reachable = BTreeSet::new();
        for start in 0..self.node_count {
            let mut queue = VecDeque::from([start]);
            let mut visited = vec![false; self.node_count];
            while let Some(node) = queue.pop_front() {
                for &neighbor in &adjacency[node] {
                    if !visited[neighbor] {
                        visited[neighbor] = true;
                        reachable.insert((start, neighbor));
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        reachable
    }

    pub fn reachability_count(&self) -> usize {
        self.reachability().len()
    }

    pub fn longest_path_length(&self) -> usize {
        let order = match self.topo_order() {
            Some(order) => order,
            None => return 0,
        };
        let adjacency = self.adjacency();
        let mut best = vec![0usize; self.node_count];
        for node in order {
            let current = best[node];
            for &neighbor in &adjacency[node] {
                best[neighbor] = best[neighbor].max(current + 1);
            }
        }
        best.into_iter().max().unwrap_or(0)
    }

    pub fn sources(&self) -> Vec<usize> {
        self.indegrees()
            .into_iter()
            .enumerate()
            .filter_map(|(node, indegree)| (indegree == 0).then_some(node))
            .collect()
    }

    pub fn sinks(&self) -> Vec<usize> {
        let adjacency = self.adjacency();
        adjacency
            .into_iter()
            .enumerate()
            .filter_map(|(node, neighbors)| neighbors.is_empty().then_some(node))
            .collect()
    }

    pub fn topo_rank(&self) -> BTreeMap<usize, usize> {
        self.topo_order()
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(rank, node)| (node, rank))
            .collect()
    }

    pub fn transitive_reduction(&self, id: &str) -> Self {
        if !self.is_acyclic() {
            return self.clone();
        }
        let mut kept = self.edges.clone();
        for &(from, to) in &self.edges {
            let without_edge = self.remove_edges("tmp", &[(from, to)]);
            if without_edge.reachability().contains(&(from, to)) {
                kept.remove(&(from, to));
            }
        }
        Self {
            id: id.to_string(),
            node_count: self.node_count,
            edges: kept,
        }
    }

    pub fn repair_by_order(&self, id: &str, order: &[usize]) -> Self {
        let rank = order
            .iter()
            .enumerate()
            .map(|(index, node)| (*node, index))
            .collect::<BTreeMap<usize, usize>>();
        let edges = self
            .edges
            .iter()
            .copied()
            .filter(|(from, to)| rank.get(from).unwrap_or(&usize::MAX) < rank.get(to).unwrap_or(&0))
            .collect();
        Self {
            id: id.to_string(),
            node_count: self.node_count,
            edges,
        }
    }
}

pub fn sample_dags() -> Vec<Graph> {
    vec![
        Graph::new("chain4", 4, &[(0, 1), (1, 2), (2, 3)]),
        Graph::new("fork4", 4, &[(0, 1), (0, 2), (2, 3)]),
        Graph::new("diamond4", 4, &[(0, 1), (0, 2), (1, 3), (2, 3)]),
        Graph::new("ladder5", 5, &[(0, 1), (1, 2), (0, 3), (3, 4), (2, 4)]),
    ]
}

pub fn safe_edge_addition(base: &Graph) -> Graph {
    base.add_edge("safe_plus_shortcut", (0, base.node_count - 1))
}

pub fn unsafe_edge_addition(base: &Graph) -> Graph {
    base.add_edge("unsafe_back_edge", (base.node_count - 1, 0))
}

pub fn periodic_update_sequence() -> Vec<Graph> {
    let base = Graph::new("periodic_base", 4, &[(0, 1), (1, 2), (2, 3)]);
    let expanded = base.add_edge("periodic_expanded", (0, 2));
    vec![base.clone(), expanded.clone(), base.clone(), expanded, base]
}
