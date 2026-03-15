use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::common::{CaseClass, CaseMetadata};
use crate::runners::{write_component_rows, RunnerContext};
use crate::sim::trust_dynamics::{custom_orbit, descending_orbit, neutral_cycle_orbit, TrustOrbit};

#[derive(Debug, Clone, Serialize)]
struct TmtrRow {
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
    orbit_id: String,
    iteration: usize,
    trust_value: f64,
    fixed_point_flag: bool,
    stabilization_iteration: usize,
    trust_gap: f64,
    state_id: String,
    next_state_id: String,
    next_trust_value: f64,
    trust_neutral_flag: bool,
    trust_increase_attempt_flag: bool,
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

fn build_rows(spec: &TheoremSpec) -> Vec<TmtrRow> {
    let descending = descending_orbit("descending", 5, 2);
    let fixed = custom_orbit("fixed", &[0, 0, 0, 0]);
    let identity = custom_orbit("identity", &[2, 2, 2, 2]);
    let neutral = neutral_cycle_orbit("neutral_cycle", 5);
    let increase_attempt = custom_orbit("increase_attempt", &[1, 3]);

    match spec.ordinal {
        1 | 2 | 3 | 4 | 6 | 10 | 11 | 12 | 17 | 18 | 20 => orbit_rows(
            spec,
            &descending,
            CaseClass::Passing,
            true,
            "Trust values descend monotonically until the orbit stabilizes.",
        ),
        5 => orbit_rows(
            spec,
            &fixed,
            CaseClass::Passing,
            true,
            "Fixed points preserve trust exactly.",
        ),
        7 => orbit_rows(
            spec,
            &descending,
            CaseClass::Passing,
            true,
            "Strict descent implies the current state is not a fixed point.",
        ),
        8 => orbit_rows(
            spec,
            &neutral,
            CaseClass::Boundary,
            true,
            "Periodic orbit keeps trust constant along the cycle.",
        ),
        9 => orbit_rows(
            spec,
            &descending,
            CaseClass::Passing,
            true,
            "Positive trust gap forces periodic behavior to collapse to fixed points.",
        ),
        13 => orbit_rows(
            spec,
            &fixed,
            CaseClass::Passing,
            true,
            "Trust-minimal states remain trust-minimal under the update.",
        ),
        14 => orbit_rows(
            spec,
            &custom_orbit("composed", &[4, 2, 1, 0, 0]),
            CaseClass::Passing,
            true,
            "Composition of trust-monotone updates remains trust-monotone.",
        ),
        15 => orbit_rows(
            spec,
            &identity,
            CaseClass::Boundary,
            true,
            "Identity update is trust-monotone with equality at every step.",
        ),
        16 => {
            let mut rows = orbit_rows(
                spec,
                &custom_orbit("constant_map", &[5, 0, 0, 0]),
                CaseClass::Passing,
                true,
                "Constant map is TMTR when it lands at a state of no greater trust.",
            );
            rows.extend(orbit_rows(
                spec,
                &increase_attempt,
                CaseClass::Violating,
                false,
                "Intentional violating witness: the proposed update increases trust and should be flagged as non-TMTR.",
            ));
            rows
        }
        19 => {
            let mut rows = orbit_rows(
                spec,
                &identity,
                CaseClass::Boundary,
                true,
                "Equal trust on consecutive iterates marks a trust-neutral update.",
            );
            rows.extend(orbit_rows(
                spec,
                &neutral,
                CaseClass::Boundary,
                true,
                "Neutral cycle provides repeated trust-equality witnesses.",
            ));
            rows
        }
        _ => unreachable!("unexpected TMTR theorem ordinal"),
    }
}

fn orbit_rows(
    spec: &TheoremSpec,
    orbit: &TrustOrbit,
    case_class: CaseClass,
    assumption_satisfied: bool,
    notes: &str,
) -> Vec<TmtrRow> {
    orbit
        .steps
        .iter()
        .map(|step| {
            let trust_gap = step.trust_value - step.next_trust_value;
            let trust_increase_attempt_flag = step.next_trust_value > step.trust_value;
            let pass = if assumption_satisfied {
                !trust_increase_attempt_flag
            } else {
                false
            };
            let expected_outcome = if assumption_satisfied {
                String::from("TMTR witnesses should be monotone non-increasing in trust and stabilize at a fixed point or neutral plateau.")
            } else {
                String::from("A trust-increasing proposal should violate TMTR monotonicity and appear as a failing witness.")
            };
            let observed_outcome = format!(
                "orbit={} iteration={} trust={} next_trust={} gap={}",
                orbit.orbit_id, step.iteration, step.trust_value, step.next_trust_value, trust_gap
            );

            let case = CaseMetadata::new(
                spec,
                "tmtr",
                format!("{}_t{}", orbit.orbit_id, step.iteration),
                case_class,
                assumption_satisfied,
                expected_outcome,
                observed_outcome,
                pass,
                notes,
            );

            TmtrRow {
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
                orbit_id: orbit.orbit_id.clone(),
                iteration: step.iteration,
                trust_value: step.trust_value,
                fixed_point_flag: step.fixed_point_flag,
                stabilization_iteration: orbit.stabilization_iteration,
                trust_gap,
                state_id: step.state_id.clone(),
                next_state_id: step.next_state_id.clone(),
                next_trust_value: step.next_trust_value,
                trust_neutral_flag: trust_gap.abs() < f64::EPSILON,
                trust_increase_attempt_flag,
            }
        })
        .collect()
}
