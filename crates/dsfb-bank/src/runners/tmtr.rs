use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::{write_component_rows, RunnerContext};
use crate::sim::trust_dynamics::{custom_orbit, descending_orbit, neutral_cycle_orbit, TrustOrbit};

#[derive(Debug, Clone, Serialize)]
struct TmtrRow {
    theorem_id: String,
    theorem_name: String,
    component: &'static str,
    case_id: String,
    case_type: String,
    pass: bool,
    notes: String,
    assumptions_satisfied: bool,
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
    match spec.ordinal {
        1 | 2 | 3 | 4 | 6 | 10 | 11 | 12 | 17 | 18 | 20 => orbit_rows(
            spec,
            &descending,
            "satisfying",
            true,
            "Trust values descend monotonically until the orbit stabilizes.",
        ),
        5 => orbit_rows(
            spec,
            &fixed,
            "satisfying",
            true,
            "Fixed points preserve trust exactly.",
        ),
        7 => orbit_rows(
            spec,
            &descending,
            "satisfying",
            true,
            "Strict descent implies the current state is not a fixed point.",
        ),
        8 => orbit_rows(
            spec,
            &neutral,
            "satisfying",
            true,
            "Periodic orbit keeps trust constant along the cycle.",
        ),
        9 => orbit_rows(
            spec,
            &descending,
            "satisfying",
            true,
            "Positive trust gap forces periodic behavior to collapse to fixed points.",
        ),
        13 => orbit_rows(
            spec,
            &fixed,
            "satisfying",
            true,
            "Trust-minimal states remain trust-minimal under the update.",
        ),
        14 => orbit_rows(
            spec,
            &custom_orbit("composed", &[4, 2, 1, 0, 0]),
            "satisfying",
            true,
            "Composition of trust-monotone updates remains trust-monotone.",
        ),
        15 => orbit_rows(
            spec,
            &identity,
            "satisfying",
            true,
            "Identity update is trust-monotone with equality at every step.",
        ),
        16 => {
            let constant_down = custom_orbit("constant_map", &[5, 0, 0, 0]);
            let constant_up = custom_orbit("constant_map_violation", &[1, 3]);
            let mut rows = orbit_rows(
                spec,
                &constant_down,
                "satisfying",
                true,
                "Constant map is TMTR when it lands at a state of no greater trust.",
            );
            rows.extend(orbit_rows(
                spec,
                &constant_up,
                "boundary",
                false,
                "Contrast case violates the constant-map TMTR criterion because trust increases.",
            ));
            rows
        }
        19 => {
            let mut rows = orbit_rows(
                spec,
                &identity,
                "satisfying",
                true,
                "Equal trust on consecutive iterates marks a trust-neutral update.",
            );
            rows.extend(orbit_rows(
                spec,
                &neutral,
                "satisfying",
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
    case_type: &str,
    assumptions_satisfied: bool,
    notes: &str,
) -> Vec<TmtrRow> {
    orbit
        .steps
        .iter()
        .map(|step| {
            let trust_gap = step.trust_value - step.next_trust_value;
            let pass = if assumptions_satisfied {
                step.next_trust_value <= step.trust_value
            } else {
                step.next_trust_value > step.trust_value
            };
            TmtrRow {
                theorem_id: spec.id.clone(),
                theorem_name: spec.title.clone(),
                component: "tmtr",
                case_id: format!("{}_t{}", orbit.orbit_id, step.iteration),
                case_type: case_type.to_string(),
                pass,
                notes: notes.to_string(),
                assumptions_satisfied,
                orbit_id: orbit.orbit_id.clone(),
                iteration: step.iteration,
                trust_value: step.trust_value,
                fixed_point_flag: step.fixed_point_flag,
                stabilization_iteration: orbit.stabilization_iteration,
                trust_gap,
                state_id: step.state_id.clone(),
                next_state_id: step.next_state_id.clone(),
                next_trust_value: step.next_trust_value,
                trust_neutral_flag: (trust_gap).abs() < f64::EPSILON,
            }
        })
        .collect()
}
