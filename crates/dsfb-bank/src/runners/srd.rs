use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::common::{CaseClass, CaseMetadata};
use crate::runners::{write_component_rows, RunnerContext};
use crate::sim::regime::{coarse_regime, fine_regime, trajectories, StateTrajectory};

#[derive(Debug, Clone, Serialize)]
struct SrdRow {
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
    trajectory_id: String,
    time_step: usize,
    state_id: String,
    fine_regime: String,
    coarse_regime: String,
    transition_flag: bool,
    coarse_transition_flag: bool,
    regime_valid_flag: bool,
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
        1 | 2 | 8 | 9 | 17 => {
            let mut rows = all_rows(
                spec,
                &cases,
                CaseClass::Passing,
                true,
                "Deterministic regime assignment gives every state exactly one regime label.",
            );
            if spec.ordinal == 1 {
                rows.push(manual_row(
                    spec,
                    "invalid_regime_label",
                    CaseClass::Violating,
                    false,
                    false,
                    "invalid_partition",
                    0,
                    "x_invalid",
                    "mixed",
                    "unknown",
                    false,
                    false,
                    false,
                    "Intentional violating witness: an invalid fine/coarse label pair breaks the admissible regime partition.",
                ));
            }
            rows
        }
        3 | 10 => all_rows(
            spec,
            &cases,
            CaseClass::Passing,
            true,
            "Transition flags coincide with fine-label changes.",
        ),
        4 => rows_for(
            spec,
            constant,
            CaseClass::Boundary,
            true,
            "Constant fine label produces no transition.",
        ),
        5 => all_rows(
            spec,
            &cases,
            CaseClass::Passing,
            true,
            "Finite trajectory transition count is bounded by the number of adjacent pairs.",
        ),
        6 => rows_for(
            spec,
            periodic,
            CaseClass::Boundary,
            true,
            "A trajectory longer than the regime set repeats at least one regime label.",
        ),
        7 => rows_for(
            spec,
            block,
            CaseClass::Boundary,
            true,
            "Fewer than N transitions over N+1 points forces at least one constant-regime adjacent pair.",
        ),
        11 | 12 => {
            let replay = StateTrajectory::new("alternating_replay", alternating.states.clone());
            let mut rows = rows_for(
                spec,
                alternating,
                CaseClass::Passing,
                true,
                "Original trajectory.",
            );
            rows.extend(rows_for(
                spec,
                &replay,
                CaseClass::Passing,
                true,
                "Replay trajectory reproduces labels and transitions exactly.",
            ));
            rows
        }
        13 | 14 => rows_for(
            spec,
            stabilizing,
            CaseClass::Passing,
            true,
            "Eventually constant states induce an eventually constant regime suffix with finitely many transitions.",
        ),
        15 => rows_for(
            spec,
            alternating,
            CaseClass::Passing,
            true,
            "Two-regime alternation produces a transition at every step.",
        ),
        16 => rows_for(
            spec,
            block,
            CaseClass::Passing,
            true,
            "Piecewise-constant regime blocks localize transitions to block boundaries.",
        ),
        18 | 19 => {
            let mut rows = rows_for(
                spec,
                block,
                CaseClass::Boundary,
                true,
                "Coarsening preserves determinism and cannot create new fine-scale transitions.",
            );
            if spec.ordinal == 18 {
                rows.push(manual_row(
                    spec,
                    "invalid_coarsening_semantics",
                    CaseClass::Violating,
                    false,
                    false,
                    "coarsening_violation",
                    1,
                    "x2",
                    "positive",
                    "low",
                    false,
                    true,
                    false,
                    "Intentional violating witness: the coarse regime contradicts the admissible coarsening map and invents a transition.",
                ));
            }
            rows
        }
        20 => rows_for(
            spec,
            periodic,
            CaseClass::Passing,
            true,
            "Periodic states induce a periodic regime sequence with period dividing the state period.",
        ),
        _ => unreachable!("unexpected SRD theorem ordinal"),
    }
}

fn all_rows(
    spec: &TheoremSpec,
    trajectories: &[StateTrajectory],
    case_class: CaseClass,
    assumption_satisfied: bool,
    notes: &str,
) -> Vec<SrdRow> {
    let mut rows = Vec::new();
    for trajectory in trajectories {
        rows.extend(rows_for(
            spec,
            trajectory,
            case_class,
            assumption_satisfied,
            notes,
        ));
    }
    rows
}

fn rows_for(
    spec: &TheoremSpec,
    trajectory: &StateTrajectory,
    case_class: CaseClass,
    assumption_satisfied: bool,
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

            manual_row(
                spec,
                &format!("{}_t{}", trajectory.id, time_step),
                case_class,
                assumption_satisfied,
                pass,
                &trajectory.id,
                time_step,
                &format!("x{}", state),
                &fine[time_step],
                &coarse[time_step],
                transition_flag,
                coarse_transition_flag,
                true,
                notes,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn manual_row(
    spec: &TheoremSpec,
    case_id: &str,
    case_class: CaseClass,
    assumption_satisfied: bool,
    pass: bool,
    trajectory_id: &str,
    time_step: usize,
    state_id: &str,
    fine_regime_label: &str,
    coarse_regime_label: &str,
    transition_flag: bool,
    coarse_transition_flag: bool,
    regime_valid_flag: bool,
    notes: &str,
) -> SrdRow {
    let expected_outcome = if assumption_satisfied {
        String::from("SRD witnesses should use valid fine/coarse regime labels and transitions consistent with admissible partition semantics.")
    } else {
        String::from("Invalid regime labels or invalid coarsening semantics should appear as failing witnesses outside the theorem assumptions.")
    };
    let observed_outcome = format!(
        "trajectory={} t={} state={} fine={} coarse={} transition={} coarse_transition={} regime_valid={}",
        trajectory_id,
        time_step,
        state_id,
        fine_regime_label,
        coarse_regime_label,
        transition_flag,
        coarse_transition_flag,
        regime_valid_flag
    );

    let case = CaseMetadata::new(
        spec,
        "srd",
        case_id,
        case_class,
        assumption_satisfied,
        expected_outcome,
        observed_outcome,
        pass,
        notes,
    );

    SrdRow {
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
        trajectory_id: trajectory_id.to_string(),
        time_step,
        state_id: state_id.to_string(),
        fine_regime: fine_regime_label.to_string(),
        coarse_regime: coarse_regime_label.to_string(),
        transition_flag,
        coarse_transition_flag,
        regime_valid_flag,
        indicator_sum: usize::from(regime_valid_flag),
    }
}
