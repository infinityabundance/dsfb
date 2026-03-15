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
    residual_value: f64,
    fixed_point_flag: bool,
    stabilization_iteration: usize,
    trust_gap: f64,
    state_id: String,
    next_state_id: String,
    next_trust_value: f64,
    trust_neutral_flag: bool,
    trust_increase_attempt_flag: bool,
    trust_gap_satisfied_flag: bool,
    monotonicity_satisfied_flag: bool,
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
    let accelerating_increase = custom_orbit("accelerating_increase", &[0, 2, 4]);
    let bounce_up = custom_orbit("bounce_up", &[1, 2, 1]);
    let gap_failure = custom_orbit("gap_failure", &[3, 3, 2, 1]);

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
        )
        .into_iter()
        .chain(orbit_rows(
            spec,
            &gap_failure,
            CaseClass::Violating,
            false,
            "Intentional violating witness: the orbit contains a trust-neutral step, so the theorem's strictly positive trust-gap premise does not hold.",
        ))
        .collect(),
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
            rows.extend(orbit_rows(
                spec,
                &accelerating_increase,
                CaseClass::Violating,
                false,
                "Intentional violating witness: repeated trust increases make the orbit non-monotone and non-admissible for TMTR.",
            ));
            rows.extend(orbit_rows(
                spec,
                &bounce_up,
                CaseClass::Violating,
                false,
                "Intentional violating witness: the first step increases trust before descending, so the update fails TMTR monotonicity at the boundary.",
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
            let trust_gap_satisfied_flag = trust_gap > 0.0;
            let monotonicity_satisfied_flag = !trust_increase_attempt_flag;
            let residual_value = stability_residual(step.next_state_value);
            let pass = if assumption_satisfied {
                monotonicity_satisfied_flag
            } else {
                false
            };
            let expected_outcome = if assumption_satisfied {
                String::from("TMTR witnesses should be monotone non-increasing in trust and stabilize at a fixed point or neutral plateau.")
            } else if trust_increase_attempt_flag {
                String::from("A trust-increasing proposal should be emitted as an intentional TMTR assumption-violating witness.")
            } else if !trust_gap_satisfied_flag {
                String::from("A theorem requiring a strictly positive trust gap should flag zero-gap or neutral-step witnesses as non-admissible.")
            } else {
                String::from("An assumption-violating TMTR witness should be emitted explicitly as a failing row.")
            };
            let observed_outcome = format!(
                "orbit={} iteration={} trust={} next_trust={} gap={} residual={} monotone={} gap_satisfied={}",
                orbit.orbit_id,
                step.iteration,
                step.trust_value,
                step.next_trust_value,
                trust_gap,
                residual_value,
                monotonicity_satisfied_flag,
                trust_gap_satisfied_flag
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
                residual_value,
                fixed_point_flag: step.fixed_point_flag,
                stabilization_iteration: orbit.stabilization_iteration,
                trust_gap,
                state_id: step.state_id.clone(),
                next_state_id: step.next_state_id.clone(),
                next_trust_value: step.next_trust_value,
                trust_neutral_flag: trust_gap.abs() < f64::EPSILON,
                trust_increase_attempt_flag,
                trust_gap_satisfied_flag,
                monotonicity_satisfied_flag,
            }
        })
        .collect()
}

fn stability_residual(next_state_value: i32) -> f64 {
    next_state_value.max(0) as f64
}
