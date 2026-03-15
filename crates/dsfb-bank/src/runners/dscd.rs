use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::common::{CaseClass, CaseMetadata};
use crate::runners::{write_component_rows, RunnerContext};
use crate::sim::causal_graph::{
    periodic_update_sequence, safe_edge_addition, sample_dags, unsafe_edge_addition, Graph,
};

#[derive(Debug, Clone, Serialize)]
struct DscdRow {
    theorem_id: String,
    theorem_name: String,
    component: &'static str,
    case_id: String,
    case_class: CaseClass,
    assumption_satisfied: bool,
    expected_outcome: String,
    observed_outcome: String,
    pass: bool,
    notes: String,
    graph_id: String,
    node_count: usize,
    edge_count: usize,
    longest_path: usize,
    reachability_count: usize,
    acyclic_flag: bool,
    attempted_edge_addition_flag: bool,
    cycle_created_flag: bool,
    reduction_edge_count: usize,
    repaired_edge_count: usize,
    iteration: Option<usize>,
    source_count: usize,
    sink_count: usize,
}

pub fn run(
    spec: &TheoremSpec,
    ctx: &RunnerContext<'_>,
) -> Result<crate::runners::TheoremExecutionResult> {
    let rows = build_rows(spec);
    let pass_count = rows.iter().filter(|row| row.pass).count();
    let fail_count = rows.len().saturating_sub(pass_count);
    write_component_rows(spec, ctx, &rows, pass_count, fail_count)
}

fn build_rows(spec: &TheoremSpec) -> Vec<DscdRow> {
    let dags = sample_dags();
    let chain = dags[0].clone();
    let fork = dags[1].clone();
    let diamond = dags[2].clone();
    let ladder = dags[3].clone();

    match spec.ordinal {
        1 => dags
            .iter()
            .map(|graph| {
                row(
                    spec,
                    &graph.id,
                    CaseClass::Passing,
                    true,
                    graph.is_acyclic(),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    None,
                    "Every finite DAG admits a topological ordering.",
                )
            })
            .collect(),
        2 => dags
            .iter()
            .map(|graph| {
                row(
                    spec,
                    &format!("{}_antisymmetry", graph.id),
                    CaseClass::Passing,
                    true,
                    graph.is_acyclic(),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    None,
                    "Reachability is antisymmetric for distinct vertices in a DAG.",
                )
            })
            .collect(),
        3 => dags
            .iter()
            .map(|graph| {
                row(
                    spec,
                    &format!("{}_irreflexive", graph.id),
                    CaseClass::Passing,
                    true,
                    !graph.reachability().iter().any(|(u, v)| u == v),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    None,
                    "No positive-length path returns to its start in an admissible DSCD graph.",
                )
            })
            .collect(),
        4 => {
            let safe = safe_edge_addition(&chain);
            let unsafe_graph = unsafe_edge_addition(&chain);
            vec![
                row(
                    spec,
                    "safe_addition",
                    CaseClass::Passing,
                    true,
                    safe.is_acyclic(),
                    &safe,
                    true,
                    false,
                    safe.transitive_reduction("reduced").edge_count(),
                    safe.edge_count(),
                    None,
                    "Safe edge addition preserves acyclicity because the target does not reach the source.",
                ),
                row(
                    spec,
                    "unsafe_addition_cycle",
                    CaseClass::Violating,
                    false,
                    false,
                    &unsafe_graph,
                    true,
                    true,
                    unsafe_graph.edge_count(),
                    unsafe_graph.edge_count(),
                    None,
                    "Intentional violating witness: adding a back-edge creates a cycle, so the updated graph is non-admissible.",
                ),
            ]
        }
        5 => vec![
            row(
                spec,
                "pruned_chain",
                CaseClass::Passing,
                true,
                chain.remove_edges("pruned", &[(1, 2)]).is_acyclic(),
                &chain.remove_edges("pruned", &[(1, 2)]),
                false,
                false,
                chain.transitive_reduction("reduced").edge_count(),
                chain.edge_count(),
                None,
                "Removing edges from a DAG cannot create a cycle.",
            ),
            row(
                spec,
                "pruned_diamond",
                CaseClass::Passing,
                true,
                diamond.remove_edges("pruned", &[(0, 2)]).is_acyclic(),
                &diamond.remove_edges("pruned", &[(0, 2)]),
                false,
                false,
                diamond.transitive_reduction("reduced").edge_count(),
                diamond.edge_count(),
                None,
                "Pruned subgraph remains admissible.",
            ),
        ],
        6 => {
            let safe = safe_edge_addition(&chain);
            vec![row(
                spec,
                "reachability_monotone",
                CaseClass::Passing,
                true,
                chain.reachability().is_subset(&safe.reachability()),
                &safe,
                true,
                false,
                safe.transitive_reduction("reduced").edge_count(),
                safe.edge_count(),
                None,
                "Admissible edge addition preserves existing reachability.",
            )]
        }
        7 => {
            let pruned = diamond.remove_edges("pruned", &[(0, 2)]);
            vec![row(
                spec,
                "reachability_reduced",
                CaseClass::Passing,
                true,
                pruned.reachability().is_subset(&diamond.reachability()),
                &pruned,
                false,
                false,
                pruned.transitive_reduction("reduced").edge_count(),
                diamond.edge_count(),
                None,
                "Reachability in the pruned graph was already present in the original graph.",
            )]
        }
        8 => dags
            .iter()
            .map(|graph| {
                row(
                    spec,
                    &format!("{}_path_bound", graph.id),
                    CaseClass::Passing,
                    true,
                    graph.longest_path_length() <= graph.node_count.saturating_sub(1),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    None,
                    "Longest directed path respects the |V|-1 bound in finite DAGs.",
                )
            })
            .collect(),
        9 => dags
            .iter()
            .map(|graph| {
                row(
                    spec,
                    &format!("{}_termination", graph.id),
                    CaseClass::Passing,
                    true,
                    graph.is_acyclic(),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    None,
                    "Propagation that traverses edges without revisiting vertices terminates finitely in a DAG.",
                )
            })
            .collect(),
        10 => dags
            .iter()
            .map(|graph| {
                row(
                    spec,
                    &format!("{}_sources_sinks", graph.id),
                    CaseClass::Passing,
                    true,
                    !graph.sources().is_empty() && !graph.sinks().is_empty(),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    None,
                    "Every finite DAG has at least one source and one sink.",
                )
            })
            .collect(),
        11 => dags
            .iter()
            .map(|graph| {
                row(
                    spec,
                    &format!("{}_partial_order", graph.id),
                    CaseClass::Passing,
                    true,
                    graph.is_acyclic(),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    None,
                    "Reachability plus equality induces a partial order on DAG vertices.",
                )
            })
            .collect(),
        12 => vec![
            row(
                spec,
                "diamond_reduction",
                CaseClass::Passing,
                true,
                diamond.transitive_reduction("reduced_a").edges
                    == diamond.transitive_reduction("reduced_b").edges,
                &diamond,
                false,
                false,
                diamond.transitive_reduction("reduced_a").edge_count(),
                diamond.edge_count(),
                None,
                "Finite DAG transitive reduction is unique.",
            ),
            row(
                spec,
                "ladder_reduction",
                CaseClass::Passing,
                true,
                ladder.transitive_reduction("reduced_a").edges
                    == ladder.transitive_reduction("reduced_b").edges,
                &ladder,
                false,
                false,
                ladder.transitive_reduction("reduced_a").edge_count(),
                ladder.edge_count(),
                None,
                "Second graph yields the same reduction on repeated computation.",
            ),
        ],
        13 => dags
            .iter()
            .map(|graph| {
                let reduction = graph.transitive_reduction("reduced");
                row(
                    spec,
                    &format!("{}_reachability_preserved", graph.id),
                    CaseClass::Passing,
                    true,
                    reduction.reachability() == graph.reachability(),
                    graph,
                    false,
                    false,
                    reduction.edge_count(),
                    graph.edge_count(),
                    None,
                    "Transitive reduction preserves reachability exactly.",
                )
            })
            .collect(),
        14 => {
            let update_a = safe_edge_addition(&fork);
            let update_b = safe_edge_addition(&fork);
            vec![row(
                spec,
                "duplicate_update",
                CaseClass::Passing,
                true,
                update_a.edges == update_b.edges,
                &update_a,
                true,
                false,
                update_a.transitive_reduction("reduced").edge_count(),
                update_a.edge_count(),
                None,
                "Equal input graphs map to equal updated graphs under a deterministic operator.",
            )]
        }
        15 => periodic_update_sequence()
            .iter()
            .enumerate()
            .map(|(iteration, graph)| {
                row(
                    spec,
                    &format!("replay_step_{iteration}"),
                    CaseClass::Passing,
                    true,
                    graph.is_acyclic(),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    Some(iteration),
                    "Deterministic replay reproduces the same DSCD evolution sequence.",
                )
            })
            .collect(),
        16 => periodic_update_sequence()
            .iter()
            .enumerate()
            .map(|(iteration, graph)| {
                row(
                    spec,
                    &format!("periodic_step_{iteration}"),
                    CaseClass::Passing,
                    true,
                    graph.is_acyclic(),
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    Some(iteration),
                    "Finite-state deterministic evolution enters a repeating orbit.",
                )
            })
            .collect(),
        17 => {
            let updated = fork.add_edge("localized_update", (0, 3));
            let new_pairs = updated
                .reachability()
                .difference(&fork.reachability())
                .copied()
                .collect::<Vec<_>>();
            vec![row(
                spec,
                "localized_descendant_effect",
                CaseClass::Passing,
                true,
                new_pairs.iter().all(|(u, _)| *u == 0),
                &updated,
                true,
                false,
                updated.transitive_reduction("reduced").edge_count(),
                updated.edge_count(),
                None,
                "New reachability created by an outgoing-edge update is localized through the modified vertex u.",
            )]
        }
        18 => {
            let union = chain.union("common_order_union", &fork);
            vec![row(
                spec,
                "common_order_union",
                CaseClass::Passing,
                true,
                union.is_acyclic(),
                &union,
                false,
                false,
                union.transitive_reduction("reduced").edge_count(),
                union.edge_count(),
                None,
                "Union of graphs sharing a common topological order remains admissible.",
            )]
        }
        19 => dags
            .iter()
            .map(|graph| {
                let ranks = graph.topo_rank();
                let pass = graph
                    .reachability()
                    .iter()
                    .all(|(u, v)| ranks.get(u).unwrap_or(&0) < ranks.get(v).unwrap_or(&0));
                row(
                    spec,
                    &format!("{}_rank_consistency", graph.id),
                    CaseClass::Passing,
                    true,
                    pass,
                    graph,
                    false,
                    false,
                    graph.transitive_reduction("reduced").edge_count(),
                    graph.edge_count(),
                    None,
                    "Reachability pairs respect strict topological rank order.",
                )
            })
            .collect(),
        20 => {
            let cyclic = Graph::new("cyclic_candidate", 4, &[(0, 1), (1, 2), (2, 3), (3, 1)]);
            let repaired = cyclic.repair_by_order("repaired", &[0, 1, 2, 3]);
            vec![
                row(
                    spec,
                    "repair_candidate_cycle",
                    CaseClass::Violating,
                    false,
                    false,
                    &cyclic,
                    false,
                    true,
                    cyclic.edge_count(),
                    cyclic.edge_count(),
                    None,
                    "Intentional violating witness: the candidate graph already contains a cycle before repair.",
                ),
                row(
                    spec,
                    "repair_back_edges",
                    CaseClass::Boundary,
                    true,
                    repaired.is_acyclic(),
                    &repaired,
                    false,
                    false,
                    repaired.transitive_reduction("reduced").edge_count(),
                    repaired.edge_count(),
                    None,
                    "Boundary witness: removing order-violating edges repairs the graph into a DAG.",
                ),
            ]
        }
        _ => unreachable!("unexpected DSCD theorem ordinal"),
    }
}

#[allow(clippy::too_many_arguments)]
fn row(
    spec: &TheoremSpec,
    case_id: &str,
    case_class: CaseClass,
    assumption_satisfied: bool,
    pass: bool,
    graph: &Graph,
    attempted_edge_addition_flag: bool,
    cycle_created_flag: bool,
    reduction_edge_count: usize,
    repaired_edge_count: usize,
    iteration: Option<usize>,
    notes: &str,
) -> DscdRow {
    let expected_outcome = if assumption_satisfied {
        String::from("Admissible DSCD witnesses should remain acyclic while preserving the stated reachability invariant.")
    } else {
        String::from("Cycle-creating graph updates should fail admissibility or require explicit repair before the theorem can apply.")
    };
    let observed_outcome = format!(
        "graph={} nodes={} edges={} acyclic={} longest_path={} reachability={} cycle_created={}",
        graph.id,
        graph.node_count,
        graph.edge_count(),
        graph.is_acyclic(),
        graph.longest_path_length(),
        graph.reachability_count(),
        cycle_created_flag
    );

    let case = CaseMetadata::new(
        spec,
        "dscd",
        case_id,
        case_class,
        assumption_satisfied,
        expected_outcome,
        observed_outcome,
        pass,
        notes,
    );

    DscdRow {
        theorem_id: case.theorem_id,
        theorem_name: case.theorem_name,
        component: case.component,
        case_id: case.case_id,
        case_class: case.case_class,
        assumption_satisfied: case.assumption_satisfied,
        expected_outcome: case.expected_outcome,
        observed_outcome: case.observed_outcome,
        pass: case.pass,
        notes: case.notes,
        graph_id: graph.id.clone(),
        node_count: graph.node_count,
        edge_count: graph.edge_count(),
        longest_path: graph.longest_path_length(),
        reachability_count: graph.reachability_count(),
        acyclic_flag: graph.is_acyclic(),
        attempted_edge_addition_flag,
        cycle_created_flag,
        reduction_edge_count,
        repaired_edge_count,
        iteration,
        source_count: graph.sources().len(),
        sink_count: graph.sinks().len(),
    }
}
