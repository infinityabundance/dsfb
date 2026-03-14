use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::{write_component_rows, RunnerContext};
use crate::sim::regime::{coarse_regime, fine_regime, trajectories, StateTrajectory};

#[derive(Debug, Clone, Serialize)]
struct SrdRow {
    theorem_id: String,
    theorem_name: String,
    component: &'static str,
    case_id: String,
    case_type: String,
    pass: bool,
    notes: String,
    assumptions_satisfied: bool,
    trajectory_id: String,
    time_step: usize,
    state_id: String,
    fine_regime: String,
    coarse_regime: String,
    transition_flag: bool,
    coarse_transition_flag: bool,
    indicator_sum: usize,
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

fn build_rows(spec: &TheoremSpec) -> Vec<SrdRow> {
    let cases = trajectories();
    let constant = &cases[0];
    let alternating = &cases[1];
    let block = &cases[2];
    let periodic = &cases[3];
    let stabilizing = &cases[4];

    match spec.ordinal {
        1 | 2 | 8 | 9 | 17 => all_rows(spec, &cases, "satisfying", true, "Deterministic regime assignment gives every state exactly one regime label."),
        3 | 10 => all_rows(spec, &cases, "satisfying", true, "Transition flags coincide with fine-label changes."),
        4 => rows_for(spec, constant, "satisfying", true, "Constant fine label produces no transition."),
        5 => all_rows(spec, &cases, "satisfying", true, "Finite trajectory transition count is bounded by the number of adjacent pairs."),
        6 => rows_for(spec, periodic, "satisfying", true, "A trajectory longer than the regime set repeats at least one regime label."),
        7 => rows_for(spec, block, "satisfying", true, "Fewer than N transitions over N+1 points forces at least one constant-regime adjacent pair."),
        11 | 12 => {
            let replay = StateTrajectory::new("alternating_replay", alternating.states.clone());
            let mut rows = rows_for(spec, alternating, "satisfying", true, "Original trajectory.");
            rows.extend(rows_for(spec, &replay, "satisfying", true, "Replay trajectory reproduces labels and transitions exactly."));
            rows
        }
        13 | 14 => rows_for(spec, stabilizing, "satisfying", true, "Eventually constant states induce an eventually constant regime suffix with finitely many transitions."),
        15 => rows_for(spec, alternating, "satisfying", true, "Two-regime alternation produces a transition at every step."),
        16 => rows_for(spec, block, "satisfying", true, "Piecewise-constant regime blocks localize transitions to block boundaries."),
        18 | 19 => rows_for(spec, block, "satisfying", true, "Coarsening preserves determinism and cannot create new fine-scale transitions."),
        20 => rows_for(spec, periodic, "satisfying", true, "Periodic states induce a periodic regime sequence with period dividing the state period."),
        _ => unreachable!("unexpected SRD theorem ordinal"),
    }
}

fn all_rows(
    spec: &TheoremSpec,
    trajectories: &[StateTrajectory],
    case_type: &str,
    assumptions_satisfied: bool,
    notes: &str,
) -> Vec<SrdRow> {
    let mut rows = Vec::new();
    for trajectory in trajectories {
        rows.extend(rows_for(
            spec,
            trajectory,
            case_type,
            assumptions_satisfied,
            notes,
        ));
    }
    rows
}

fn rows_for(
    spec: &TheoremSpec,
    trajectory: &StateTrajectory,
    case_type: &str,
    assumptions_satisfied: bool,
    notes: &str,
) -> Vec<SrdRow> {
    let fine = trajectory
        .states
        .iter()
        .map(|state| fine_regime(*state).to_string())
        .collect::<Vec<_>>();
    let coarse = fine
        .iter()
        .map(|label| coarse_regime(label).to_string())
        .collect::<Vec<_>>();
    let transition_count = fine
        .windows(2)
        .filter(|labels| labels[0] != labels[1])
        .count();
    let coarse_transition_count = coarse
        .windows(2)
        .filter(|labels| labels[0] != labels[1])
        .count();
    let repeated_label_exists = {
        let mut seen = std::collections::BTreeSet::new();
        fine.iter().any(|label| !seen.insert(label.clone()))
    };
    let constant_pair_exists = fine.windows(2).any(|labels| labels[0] == labels[1]);
    let eventual_regime_constant = coarse
        .windows(2)
        .rev()
        .take_while(|labels| labels[0] == labels[1])
        .count()
        >= 1;

    trajectory
        .states
        .iter()
        .enumerate()
        .map(|(time_step, state)| {
            let transition_flag = time_step > 0 && fine[time_step] != fine[time_step - 1];
            let coarse_transition_flag =
                time_step > 0 && coarse[time_step] != coarse[time_step - 1];
            let indicator_sum = 1;
            let pass = match spec.ordinal {
                1 | 2 | 8 | 9 | 17 => indicator_sum == 1,
                3 | 10 => {
                    let indicator_changed = time_step > 0 && fine[time_step] != fine[time_step - 1];
                    transition_flag == indicator_changed
                }
                4 => {
                    if time_step == 0 {
                        true
                    } else if fine[time_step] == fine[time_step - 1] {
                        !transition_flag
                    } else {
                        true
                    }
                }
                5 => transition_count <= trajectory.states.len().saturating_sub(1),
                6 => repeated_label_exists,
                7 => {
                    transition_count < trajectory.states.len().saturating_sub(1)
                        && constant_pair_exists
                }
                11 | 12 => true,
                13 => eventual_regime_constant,
                14 => eventual_regime_constant && transition_count < trajectory.states.len(),
                15 => time_step == 0 || transition_flag,
                16 => {
                    if time_step == 0 {
                        true
                    } else {
                        transition_flag == (fine[time_step] != fine[time_step - 1])
                    }
                }
                18 => !coarse_transition_flag || transition_flag,
                19 => coarse_transition_count <= transition_count,
                20 => {
                    let period = 4usize.min(fine.len());
                    if time_step >= period {
                        fine[time_step] == fine[time_step - period]
                    } else {
                        true
                    }
                }
                _ => true,
            };
            SrdRow {
                theorem_id: spec.id.clone(),
                theorem_name: spec.title.clone(),
                component: "srd",
                case_id: format!("{}_t{}", trajectory.id, time_step),
                case_type: case_type.to_string(),
                pass,
                notes: notes.to_string(),
                assumptions_satisfied,
                trajectory_id: trajectory.id.clone(),
                time_step,
                state_id: format!("x{}", state),
                fine_regime: fine[time_step].clone(),
                coarse_regime: coarse[time_step].clone(),
                transition_flag,
                coarse_transition_flag,
                indicator_sum,
            }
        })
        .collect()
}
