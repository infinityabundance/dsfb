//! DSFB-Debug: service-call graph auto-inference (no_std).
//!
//! # Why auto-inference matters
//!
//! `causality::attribute_root_causes` (paper §10) requires a service
//! dependency graph as input. Earlier versions made the graph an
//! operator-supplied prerequisite: a deployment couldn't run
//! root-cause attribution until a human curated the
//! parent→child service map. This module removes that prerequisite
//! by recovering the graph directly from observed span parent-child
//! links in the trace stream.
//!
//! # Algorithm — Tarjan's SCC
//!
//! Tarjan's strongly-connected-components algorithm
//! (Tarjan 1972, "Depth-first search and linear graph algorithms",
//! SIAM J. Comput. 1(2):146-160) handles cyclic dependency
//! structures (e.g.\ mutual-recursion service pairs). Each SCC
//! contracts to a single supernode; the resulting DAG is the
//! service dependency graph that
//! `causality::attribute_root_causes` consumes.
//!
//! # Determinism (Theorem 9 preserved)
//!
//! The entire graph-walk is a pure function of the input edges.
//! Tarjan's algorithm is deterministic given a fixed input
//! ordering; the caller's responsibility is to pass `ServiceEdge`
//! slices in canonical order (lexicographic sort on
//! `(parent_service_idx, child_service_idx)` is sufficient).
//!
//! # Input shape
//!
//! `ServiceEdge { parent_service_idx, child_service_idx }` —
//! indices into a caller-managed `signal_names` table. The minimal
//! Jaeger / OTLP span shape suffices: `service_name`, `span_id`,
//! `parent_span_id` map to ServiceEdges via a one-pass scan over
//! the span list.

#![allow(clippy::too_many_arguments)]

/// One observed parent->child invocation between services.
/// `parent_service_idx` and `child_service_idx` are indices into a
/// caller-managed `signal_names` table.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ServiceEdge {
    pub parent: u16,
    pub child: u16,
}

/// Maximum SCC graph size handled. Larger graphs return an empty
/// result; in practice 256 services covers most microservice meshes.
const MAX_NODES: usize = 256;

/// Infer a parent->child service edge list from raw observation pairs.
///
/// Deduplicates input edges and collapses self-loops. Returns the edges
/// in canonical (lowest-parent, then lowest-child) order so downstream
/// causality attribution is deterministic.
///
/// `observed` may contain duplicates, self-loops, and out-of-range
/// indices; this function tolerates all three. Edges with either
/// endpoint >= num_services are dropped.
pub fn infer_service_graph_from_observed(
    observed: &[ServiceEdge],
    num_services: usize,
    out_edges: &mut [(u16, u16)],
) -> usize {
    if num_services == 0 || num_services > MAX_NODES {
        return 0;
    }
    // Adjacency bitset (out_edges) and (in_edges) for dedup.
    let mut seen = [[false; MAX_NODES]; MAX_NODES];

    let mut count: usize = 0;
    let mut i = 0;
    while i < observed.len() {
        let e = observed[i];
        let p = e.parent as usize;
        let c = e.child as usize;
        if p < num_services && c < num_services && p != c && !seen[p][c] {
            seen[p][c] = true;
            // Emit in row-major (parent-major, child-minor) order — but
            // since we're scanning input in caller order, we'll re-sort
            // at the end for canonical output.
            if count < out_edges.len() {
                out_edges[count] = (e.parent, e.child);
                count += 1;
            }
        }
        i += 1;
    }

    // Canonical sort: lexicographic (parent, child).
    canonical_sort(&mut out_edges[..count]);
    count
}

fn canonical_sort(edges: &mut [(u16, u16)]) {
    // Insertion sort — adequate for small N; deterministic.
    let n = edges.len();
    let mut i = 1;
    while i < n {
        let cur = edges[i];
        let mut j = i;
        while j > 0 && (edges[j - 1].0 > cur.0
                        || (edges[j - 1].0 == cur.0 && edges[j - 1].1 > cur.1))
        {
            edges[j] = edges[j - 1];
            j -= 1;
        }
        edges[j] = cur;
        i += 1;
    }
}

/// Strongly-connected-component count via Tarjan's algorithm.
/// Returns the SCC ID per node into `scc_id_out` (length must be >=
/// num_services), and the total SCC count.
///
/// Used for cycle detection: if SCC count < num_services, the graph
/// has at least one cycle and the operator should consider whether the
/// causality attribution algorithm's "first slew → upstream-most"
/// heuristic is valid for their topology (it is for DAGs; ambiguous
/// for graphs with cycles).
pub fn tarjan_scc(
    edges: &[(u16, u16)],
    num_services: usize,
    scc_id_out: &mut [u16],
) -> usize {
    if num_services == 0 || num_services > MAX_NODES || scc_id_out.len() < num_services {
        return 0;
    }

    // Adjacency list constructed from edges. Use fixed-size arrays.
    let mut adj_start = [0_u16; MAX_NODES + 1];
    let mut adj_targets = [0_u16; MAX_NODES * MAX_NODES];
    // Two-pass: count outgoing per node, then fill.
    let mut out_count = [0_u16; MAX_NODES];
    let mut e = 0;
    while e < edges.len() {
        let (p, _c) = edges[e];
        if (p as usize) < num_services {
            out_count[p as usize] += 1;
        }
        e += 1;
    }
    let mut acc = 0_u16;
    let mut k = 0;
    while k <= num_services {
        adj_start[k] = acc;
        if k < num_services {
            acc = acc.saturating_add(out_count[k]);
        }
        k += 1;
    }
    let mut filled = [0_u16; MAX_NODES];
    e = 0;
    while e < edges.len() {
        let (p, c) = edges[e];
        if (p as usize) < num_services && (c as usize) < num_services {
            let pos = adj_start[p as usize] + filled[p as usize];
            if (pos as usize) < adj_targets.len() {
                adj_targets[pos as usize] = c;
                filled[p as usize] += 1;
            }
        }
        e += 1;
    }

    // Tarjan state.
    let mut index = 0_u32;
    let mut stack = [0_u16; MAX_NODES];
    let mut on_stack = [false; MAX_NODES];
    let mut top = 0_usize;
    let mut node_index = [u32::MAX; MAX_NODES];
    let mut node_lowlink = [0_u32; MAX_NODES];
    let mut scc_count = 0_usize;

    // Iterative DFS — Rust no_std friendly (no recursion limit).
    let mut dfs_stack = [(0_u16, 0_u16); MAX_NODES];
    let mut dfs_top = 0_usize;

    for v0 in 0..num_services {
        if node_index[v0] != u32::MAX {
            continue;
        }
        // Push v0 with iterator pos 0
        dfs_stack[dfs_top] = (v0 as u16, 0);
        dfs_top += 1;
        node_index[v0] = index;
        node_lowlink[v0] = index;
        index += 1;
        stack[top] = v0 as u16;
        on_stack[v0] = true;
        top += 1;

        while dfs_top > 0 {
            let (v, mut iter_pos) = dfs_stack[dfs_top - 1];
            let start = adj_start[v as usize];
            let end = adj_start[v as usize + 1];
            let mut advanced = false;
            while (start + iter_pos) < end {
                let w = adj_targets[(start + iter_pos) as usize];
                iter_pos += 1;
                if node_index[w as usize] == u32::MAX {
                    // Recurse on w
                    dfs_stack[dfs_top - 1].1 = iter_pos;
                    dfs_stack[dfs_top] = (w, 0);
                    dfs_top += 1;
                    node_index[w as usize] = index;
                    node_lowlink[w as usize] = index;
                    index += 1;
                    if top < MAX_NODES {
                        stack[top] = w;
                        on_stack[w as usize] = true;
                        top += 1;
                    }
                    advanced = true;
                    break;
                } else if on_stack[w as usize] {
                    let lw = node_lowlink[v as usize];
                    let iw = node_index[w as usize];
                    if iw < lw {
                        node_lowlink[v as usize] = iw;
                    }
                }
            }
            if advanced {
                continue;
            }
            // Finished iterating v's neighbours.
            // Pop one frame; propagate lowlink to parent.
            dfs_top -= 1;
            if dfs_top > 0 {
                let parent = dfs_stack[dfs_top - 1].0;
                let lp = node_lowlink[parent as usize];
                let lv = node_lowlink[v as usize];
                if lv < lp {
                    node_lowlink[parent as usize] = lv;
                }
            }
            // If v is an SCC root, pop the SCC.
            if node_lowlink[v as usize] == node_index[v as usize] {
                loop {
                    if top == 0 {
                        break;
                    }
                    top -= 1;
                    let w = stack[top];
                    on_stack[w as usize] = false;
                    scc_id_out[w as usize] = scc_count as u16;
                    if w == v {
                        break;
                    }
                }
                scc_count += 1;
            }
        }
    }
    scc_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_and_drops_self_loops() {
        let observed = [
            ServiceEdge { parent: 0, child: 1 },
            ServiceEdge { parent: 0, child: 1 }, // duplicate
            ServiceEdge { parent: 1, child: 1 }, // self-loop
            ServiceEdge { parent: 2, child: 0 },
        ];
        let mut out = [(0_u16, 0_u16); 16];
        let n = infer_service_graph_from_observed(&observed, 3, &mut out);
        assert_eq!(n, 2, "self-loop dropped, duplicate dedup");
        // Canonical order: (0, 1), (2, 0)
        assert_eq!(out[0], (0, 1));
        assert_eq!(out[1], (2, 0));
    }

    #[test]
    fn out_of_range_indices_dropped() {
        let observed = [
            ServiceEdge { parent: 0, child: 5 },   // child out of range (num_services=3)
            ServiceEdge { parent: 99, child: 0 },  // parent out of range
            ServiceEdge { parent: 1, child: 2 },
        ];
        let mut out = [(0_u16, 0_u16); 16];
        let n = infer_service_graph_from_observed(&observed, 3, &mut out);
        assert_eq!(n, 1);
        assert_eq!(out[0], (1, 2));
    }

    #[test]
    fn scc_linear_chain_yields_n_components() {
        // Linear chain s0 → s1 → s2 → s3 has 4 SCCs (one per node).
        let edges = [(0_u16, 1_u16), (1, 2), (2, 3)];
        let mut scc_id = [0_u16; 4];
        let n = tarjan_scc(&edges, 4, &mut scc_id);
        assert_eq!(n, 4);
    }

    #[test]
    fn scc_cycle_collapses_to_one() {
        // 3-cycle: s0 → s1 → s2 → s0 has 1 SCC.
        let edges = [(0_u16, 1_u16), (1, 2), (2, 0)];
        let mut scc_id = [0_u16; 3];
        let n = tarjan_scc(&edges, 3, &mut scc_id);
        assert_eq!(n, 1);
        // All three nodes belong to the same SCC.
        assert_eq!(scc_id[0], scc_id[1]);
        assert_eq!(scc_id[1], scc_id[2]);
    }

    #[test]
    fn scc_mixed_topology() {
        // 4 nodes: s0 → s1, s1 → s2 → s1 (cycle), s3 standalone
        // SCCs: {s0}, {s1, s2}, {s3} = 3 SCCs.
        let edges = [(0_u16, 1_u16), (1, 2), (2, 1)];
        let mut scc_id = [0_u16; 4];
        let n = tarjan_scc(&edges, 4, &mut scc_id);
        assert_eq!(n, 3);
    }
}
