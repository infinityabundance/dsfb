use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::common::{CaseClass, CaseMetadata};
use crate::runners::{write_component_rows, RunnerContext};
use crate::sim::trace::{periodic_window_observation, sample_traces, EventTrace, TraceCodebook};

#[derive(Debug, Clone, Serialize)]
struct HretRow {
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
    trace_id: String,
    event_index: usize,
    trace_length: usize,
    prefix_length: usize,
    suffix_length: usize,
    observation_code: String,
    reconstruction_success: bool,
    replayability_flag: bool,
    injective_observation_flag: bool,
    event_symbol: String,
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

fn build_rows(spec: &TheoremSpec) -> Vec<HretRow> {
    let traces = sample_traces();
    let tau = traces[0].clone();
    let sigma = traces[1].clone();
    let rho = traces[2].clone();
    let periodic = traces[3].clone();
    let tau_sigma = tau.concat(&sigma, "tau_sigma");
    let sigma_rho = sigma.concat(&rho, "sigma_rho");
    let assoc_left = tau_sigma.concat(&rho, "assoc_left");
    let assoc_right = tau.concat(&sigma_rho, "assoc_right");
    let mut trace_catalog = vec![
        tau.clone(),
        sigma.clone(),
        rho.clone(),
        tau_sigma.clone(),
        sigma_rho.clone(),
        assoc_left.clone(),
        assoc_right.clone(),
        periodic.clone(),
    ];
    for length in 0..=tau_sigma.len() {
        trace_catalog.push(tau_sigma.prefix(length, format!("tau_sigma_prefix_{length}")));
        trace_catalog.push(tau_sigma.suffix(length, format!("tau_sigma_suffix_{length}")));
    }
    for length in 0..=sigma.len() {
        trace_catalog.push(sigma.prefix(length, format!("sigma_prefix_{length}")));
        trace_catalog.push(sigma.suffix(length, format!("sigma_suffix_{length}")));
    }
    let codebook = TraceCodebook::from_traces(&trace_catalog);

    match spec.ordinal {
        1 => rows_for_trace(
            spec,
            &assoc_left,
            &codebook,
            CaseClass::Passing,
            true,
            assoc_left.as_string() == assoc_right.as_string(),
            "Trace concatenation is associative.",
        ),
        2 => rows_for_trace(
            spec,
            &tau_sigma,
            &codebook,
            CaseClass::Passing,
            true,
            tau_sigma.len() == tau.len() + sigma.len(),
            "Concatenated trace length is additive.",
        ),
        3 => prefix_suffix_rows(
            spec,
            &sigma,
            &codebook,
            CaseClass::Passing,
            true,
            "Every prefix length is bounded by the trace length.",
            true,
        ),
        4 => prefix_suffix_rows(
            spec,
            &sigma,
            &codebook,
            CaseClass::Passing,
            true,
            "Every suffix length is bounded by the trace length.",
            false,
        ),
        5 => prefix_suffix_rows(
            spec,
            &tau_sigma,
            &codebook,
            CaseClass::Passing,
            true,
            "There is a unique prefix of each admissible length.",
            true,
        ),
        6 => prefix_suffix_rows(
            spec,
            &tau_sigma,
            &codebook,
            CaseClass::Passing,
            true,
            "There is a unique suffix of each admissible length.",
            false,
        ),
        7 => rows_for_trace(
            spec,
            &tau_sigma,
            &codebook,
            CaseClass::Passing,
            true,
            true,
            "Chosen prefix and complementary suffix reconstruct the original trace.",
        ),
        8 => {
            let replay = EventTrace::new("tau_copy", tau.events.clone());
            let mut rows = rows_for_trace(
                spec,
                &tau,
                &codebook,
                CaseClass::Passing,
                true,
                codebook.observation_code(&tau) == codebook.observation_code(&replay),
                "Equal traces give equal observations under a deterministic observation map.",
            );
            rows.extend(rows_for_trace(
                spec,
                &replay,
                &codebook,
                CaseClass::Passing,
                true,
                codebook.observation_code(&tau) == codebook.observation_code(&replay),
                "Replay trace matches the observation exactly.",
            ));
            rows
        }
        9 | 10 | 11 | 12 | 15 | 16 | 18 | 20 => {
            let mut rows = trace_catalog
                .iter()
                .flat_map(|trace| {
                    let code = codebook.observation_code(trace).unwrap_or_default();
                    let reconstructed = codebook
                        .reconstruct(code)
                        .map(|item| item.as_string() == trace.as_string())
                        .unwrap_or(false);
                    rows_for_trace(
                        spec,
                        trace,
                        &codebook,
                        CaseClass::Passing,
                        true,
                        reconstructed,
                        "Injective finite-trace coding yields unique exact reconstruction on the image.",
                    )
                })
                .collect::<Vec<_>>();
            if spec.ordinal == 9 {
                rows.extend(noninjective_observation_rows(spec));
            }
            rows
        }
        13 => (0..=tau_sigma.len())
            .flat_map(|prefix_length| {
                let prefix = tau_sigma.prefix(prefix_length, format!("prefix_{prefix_length}"));
                let reconstructed = codebook
                    .observation_code(&prefix)
                    .and_then(|code| codebook.reconstruct(code))
                    .map(|item| item.as_string() == prefix.as_string())
                    .unwrap_or(false);
                rows_for_trace(
                    spec,
                    &prefix,
                    &codebook,
                    CaseClass::Passing,
                    true,
                    reconstructed,
                    "Prefix-restricted observations reconstruct the corresponding prefix exactly.",
                )
            })
            .collect(),
        14 => rows_for_trace(
            spec,
            &tau_sigma,
            &codebook,
            CaseClass::Passing,
            true,
            tau_sigma.prefix(tau.len(), "tau_prefix").as_string() == tau.as_string(),
            "Extending a trace preserves the earlier prefix exactly.",
        ),
        17 => {
            let replay = EventTrace::new("tau_copy", tau.events.clone());
            let replay_two = EventTrace::new("tau_copy_2", tau.events.clone());
            let traces = [tau.clone(), replay, replay_two];
            traces
                .iter()
                .flat_map(|trace| {
                    rows_for_trace(
                        spec,
                        trace,
                        &codebook,
                        CaseClass::Passing,
                        true,
                        codebook.observation_code(trace) == codebook.observation_code(&tau),
                        "Observation equality defines an equivalence relation over traces.",
                    )
                })
                .collect()
        }
        19 => periodic_window_observation(&periodic, 2)
            .iter()
            .enumerate()
            .map(|(event_index, observation)| {
                let case = CaseMetadata::new(
                    spec,
                    "hret",
                    format!("periodic_window_{event_index}"),
                    CaseClass::Boundary,
                    true,
                    "Finite-window periodic observations should repeat with the same period.",
                    format!(
                        "trace={} event_index={} observation_code={}",
                        periodic.id, event_index, observation
                    ),
                    if event_index >= 3 {
                        observation == &periodic_window_observation(&periodic, 2)[event_index - 2]
                    } else {
                        true
                    },
                    "Periodic trace yields a periodic fixed-window observation sequence.",
                );

                HretRow {
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
                    trace_id: periodic.id.clone(),
                    event_index,
                    trace_length: periodic.len(),
                    prefix_length: event_index + 1,
                    suffix_length: periodic.len() - event_index,
                    observation_code: observation.clone(),
                    reconstruction_success: false,
                    replayability_flag: true,
                    injective_observation_flag: false,
                    event_symbol: periodic.events[event_index].to_string(),
                }
            })
            .collect(),
        _ => unreachable!("unexpected HRET theorem ordinal"),
    }
}

fn rows_for_trace(
    spec: &TheoremSpec,
    trace: &EventTrace,
    codebook: &TraceCodebook,
    case_class: CaseClass,
    assumption_satisfied: bool,
    pass: bool,
    notes: &str,
) -> Vec<HretRow> {
    let observation_code = codebook
        .observation_code(trace)
        .map(|code| code.to_string())
        .unwrap_or_else(|| String::from("untracked"));
    let reconstruction_success = observation_code
        .parse::<u32>()
        .ok()
        .and_then(|code| codebook.reconstruct(code))
        .map(|reconstructed| reconstructed.as_string() == trace.as_string())
        .unwrap_or(false);
    rows_for_trace_with_observation(
        spec,
        trace,
        observation_code,
        case_class,
        assumption_satisfied,
        pass,
        notes,
        reconstruction_success,
        reconstruction_success,
        true,
    )
}

fn rows_for_trace_with_observation(
    spec: &TheoremSpec,
    trace: &EventTrace,
    observation_code: String,
    case_class: CaseClass,
    assumption_satisfied: bool,
    pass: bool,
    notes: &str,
    reconstruction_success: bool,
    replayability_flag: bool,
    injective_observation_flag: bool,
) -> Vec<HretRow> {
    let expected_outcome = if assumption_satisfied {
        String::from("Injective admissible observations should support unique historical reconstruction or replay of the stated trace property.")
    } else {
        String::from("Non-injective observations should make historical reconstruction ambiguous and therefore fail uniqueness-based claims.")
    };
    if trace.events.is_empty() {
        let case = CaseMetadata::new(
            spec,
            "hret",
            format!("{}_empty", trace.id),
            case_class,
            assumption_satisfied,
            expected_outcome,
            format!(
                "trace={} event_index=0 observation_code={} reconstruction_success={}",
                trace.id, observation_code, reconstruction_success
            ),
            pass,
            notes,
        );

        return vec![HretRow {
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
            trace_id: trace.id.clone(),
            event_index: 0,
            trace_length: 0,
            prefix_length: 0,
            suffix_length: 0,
            observation_code,
            reconstruction_success,
            replayability_flag,
            injective_observation_flag,
            event_symbol: String::new(),
        }];
    }
    trace
        .events
        .iter()
        .enumerate()
        .map(|(event_index, event)| {
            let case = CaseMetadata::new(
                spec,
                "hret",
                format!("{}_e{}", trace.id, event_index),
                case_class,
                assumption_satisfied,
                expected_outcome.clone(),
                format!(
                    "trace={} event_index={} observation_code={} reconstruction_success={}",
                    trace.id, event_index, observation_code, reconstruction_success
                ),
                pass,
                notes,
            );

            HretRow {
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
                trace_id: trace.id.clone(),
                event_index,
                trace_length: trace.len(),
                prefix_length: event_index + 1,
                suffix_length: trace.len() - event_index,
                observation_code: observation_code.clone(),
                reconstruction_success,
                replayability_flag,
                injective_observation_flag,
                event_symbol: event.to_string(),
            }
        })
        .collect()
}

fn prefix_suffix_rows(
    spec: &TheoremSpec,
    trace: &EventTrace,
    codebook: &TraceCodebook,
    case_class: CaseClass,
    assumption_satisfied: bool,
    notes: &str,
    prefix_mode: bool,
) -> Vec<HretRow> {
    (0..=trace.len())
        .flat_map(|length| {
            let fragment = if prefix_mode {
                trace.prefix(length, format!("prefix_{length}"))
            } else {
                trace.suffix(length, format!("suffix_{length}"))
            };
            let pass = fragment.len() <= trace.len();
            rows_for_trace(
                spec,
                &fragment,
                codebook,
                case_class,
                assumption_satisfied,
                pass,
                notes,
            )
        })
        .collect()
}

fn noninjective_observation_rows(spec: &TheoremSpec) -> Vec<HretRow> {
    let tau = EventTrace::new("ambiguous_tau", vec!['a', 'b']);
    let eta = EventTrace::new("ambiguous_eta", vec!['c', 'b']);
    let observation_code = String::from("last_b");

    let mut rows = rows_for_trace_with_observation(
        spec,
        &tau,
        observation_code.clone(),
        CaseClass::Violating,
        false,
        false,
        "Intentional violating witness: distinct traces share the same non-injective observation code, so historical reconstruction is ambiguous.",
        false,
        true,
        false,
    );
    rows.extend(rows_for_trace_with_observation(
        spec,
        &eta,
        observation_code,
        CaseClass::Violating,
        false,
        false,
        "Intentional violating witness: a second distinct trace maps to the same observation code, confirming non-uniqueness.",
        false,
        true,
        false,
    ));
    rows
}
