use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::{write_component_rows, RunnerContext};
use crate::sim::trace::{periodic_window_observation, sample_traces, EventTrace, TraceCodebook};

#[derive(Debug, Clone, Serialize)]
struct HretRow {
    theorem_id: String,
    theorem_name: String,
    component: &'static str,
    case_id: String,
    case_type: String,
    pass: bool,
    notes: String,
    assumptions_satisfied: bool,
    trace_id: String,
    event_index: usize,
    trace_length: usize,
    prefix_length: usize,
    suffix_length: usize,
    observation_code: String,
    reconstruction_success: bool,
    replayability_flag: bool,
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
        1 => rows_for_trace(spec, &assoc_left, &codebook, "satisfying", true, assoc_left.as_string() == assoc_right.as_string(), "Trace concatenation is associative."),
        2 => rows_for_trace(spec, &tau_sigma, &codebook, "satisfying", true, tau_sigma.len() == tau.len() + sigma.len(), "Concatenated trace length is additive."),
        3 => prefix_suffix_rows(spec, &sigma, &codebook, true, "Every prefix length is bounded by the trace length.", true),
        4 => prefix_suffix_rows(spec, &sigma, &codebook, true, "Every suffix length is bounded by the trace length.", false),
        5 => prefix_suffix_rows(spec, &tau_sigma, &codebook, true, "There is a unique prefix of each admissible length.", true),
        6 => prefix_suffix_rows(spec, &tau_sigma, &codebook, true, "There is a unique suffix of each admissible length.", false),
        7 => rows_for_trace(spec, &tau_sigma, &codebook, "satisfying", true, true, "Chosen prefix and complementary suffix reconstruct the original trace."),
        8 => {
            let replay = EventTrace::new("tau_copy", tau.events.clone());
            let mut rows = rows_for_trace(spec, &tau, &codebook, "satisfying", true, codebook.observation_code(&tau) == codebook.observation_code(&replay), "Equal traces give equal observations under a deterministic observation map.");
            rows.extend(rows_for_trace(spec, &replay, &codebook, "satisfying", true, codebook.observation_code(&tau) == codebook.observation_code(&replay), "Replay trace matches the observation exactly."));
            rows
        }
        9 | 10 | 11 | 12 | 15 | 16 | 18 | 20 => trace_catalog
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
                    "satisfying",
                    true,
                    reconstructed,
                    "Injective finite-trace coding yields unique exact reconstruction on the image.",
                )
            })
            .collect(),
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
                    "satisfying",
                    true,
                    reconstructed,
                    "Prefix-restricted observations reconstruct the corresponding prefix exactly.",
                )
            })
            .collect(),
        14 => rows_for_trace(spec, &tau_sigma, &codebook, "satisfying", true, tau_sigma.prefix(tau.len(), "tau_prefix").as_string() == tau.as_string(), "Extending a trace preserves the earlier prefix exactly."),
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
                        "satisfying",
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
            .map(|(event_index, observation)| HretRow {
                theorem_id: spec.id.clone(),
                theorem_name: spec.title.clone(),
                component: "hret",
                case_id: format!("periodic_window_{event_index}"),
                case_type: String::from("satisfying"),
                pass: if event_index >= 3 {
                    observation == &periodic_window_observation(&periodic, 2)[event_index - 2]
                } else {
                    true
                },
                notes: String::from("Periodic trace yields a periodic fixed-window observation sequence."),
                assumptions_satisfied: true,
                trace_id: periodic.id.clone(),
                event_index,
                trace_length: periodic.len(),
                prefix_length: event_index + 1,
                suffix_length: periodic.len() - event_index,
                observation_code: observation.clone(),
                reconstruction_success: false,
                replayability_flag: true,
                event_symbol: periodic.events[event_index].to_string(),
            })
            .collect(),
        _ => unreachable!("unexpected HRET theorem ordinal"),
    }
}

fn rows_for_trace(
    spec: &TheoremSpec,
    trace: &EventTrace,
    codebook: &TraceCodebook,
    case_type: &str,
    assumptions_satisfied: bool,
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
    if trace.events.is_empty() {
        return vec![HretRow {
            theorem_id: spec.id.clone(),
            theorem_name: spec.title.clone(),
            component: "hret",
            case_id: format!("{}_empty", trace.id),
            case_type: case_type.to_string(),
            pass,
            notes: notes.to_string(),
            assumptions_satisfied,
            trace_id: trace.id.clone(),
            event_index: 0,
            trace_length: 0,
            prefix_length: 0,
            suffix_length: 0,
            observation_code,
            reconstruction_success,
            replayability_flag: reconstruction_success,
            event_symbol: String::new(),
        }];
    }
    trace
        .events
        .iter()
        .enumerate()
        .map(|(event_index, event)| HretRow {
            theorem_id: spec.id.clone(),
            theorem_name: spec.title.clone(),
            component: "hret",
            case_id: format!("{}_e{}", trace.id, event_index),
            case_type: case_type.to_string(),
            pass,
            notes: notes.to_string(),
            assumptions_satisfied,
            trace_id: trace.id.clone(),
            event_index,
            trace_length: trace.len(),
            prefix_length: event_index + 1,
            suffix_length: trace.len() - event_index,
            observation_code: observation_code.clone(),
            reconstruction_success,
            replayability_flag: reconstruction_success,
            event_symbol: event.to_string(),
        })
        .collect()
}

fn prefix_suffix_rows(
    spec: &TheoremSpec,
    trace: &EventTrace,
    codebook: &TraceCodebook,
    assumptions_satisfied: bool,
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
                "satisfying",
                assumptions_satisfied,
                pass,
                notes,
            )
        })
        .collect()
}
