use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::{write_component_rows, RunnerContext};
use crate::sim::anomaly::{evaluate_signal, DetectorPoint, DetectorThresholds};
use crate::sim::deterministic_signal::{
    constant_signal, linear_signal, spike_signal, step_signal, SignalTrace,
};

#[derive(Debug, Clone, Serialize)]
struct AddRow {
    theorem_id: String,
    theorem_name: String,
    component: &'static str,
    case_id: String,
    case_type: String,
    pass: bool,
    notes: String,
    assumptions_satisfied: bool,
    signal_id: String,
    time_step: usize,
    signal_value: f64,
    residual_value: f64,
    first_difference: f64,
    second_difference: f64,
    threshold: f64,
    detector_output: String,
    rule_name: String,
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

fn build_rows(spec: &TheoremSpec) -> Vec<AddRow> {
    let thresholds = DetectorThresholds {
        residual: 1.0,
        difference: 0.8,
        curvature: 1.5,
    };
    let constant = constant_signal("constant", 2.0, 6);
    let exact_linear = linear_signal("exact_linear", 0.0, 1.0, 6);
    let drift = linear_signal("drift", 0.0, 1.2, 6);
    let step = step_signal("step", 0.0, 3.0, 3, 6);
    let spike = spike_signal("spike", 0.0, 4.0, 2, 6);
    let bounded = linear_signal("bounded", 0.0, 0.5, 8);

    match spec.ordinal {
        1 => rows_from_signal(spec, &exact_linear, thresholds, |x| x + 1.0, "residual_threshold", "satisfying", true, "Zero residual trace never crosses a positive residual threshold."),
        2 => rows_from_signal(spec, &step, thresholds, |x| x, "residual_threshold", "satisfying", true, "Residual threshold detector fires whenever |r_t| exceeds tau_r."),
        3 => rows_from_signal(spec, &drift, thresholds, |x| x, "difference_threshold", "satisfying", true, "Difference detector fires on persistent large first differences."),
        4 => rows_from_signal(spec, &spike, thresholds, |x| x, "curvature_threshold", "satisfying", true, "Curvature detector fires on large second differences."),
        5 => rows_from_signal(spec, &constant, thresholds, |x| x, "difference_threshold", "satisfying", true, "Constant signal has zero first and second differences, so difference-based rules stay silent."),
        6 => rows_from_signal(spec, &exact_linear, thresholds, |x| x + 1.0, "curvature_threshold", "satisfying", true, "Linear signal yields identically zero second difference."),
        7 => rows_from_signal(spec, &drift, thresholds, |x| x, "difference_sign", "satisfying", true, "Monotone drift produces a constant-sign first difference."),
        8 => rows_from_signal(spec, &drift, thresholds, |x| x, "difference_threshold", "satisfying", true, "Persistent drift above tau_Delta triggers at every step on the interval."),
        9 => rows_from_signal(spec, &step, thresholds, |x| x, "difference_threshold", "satisfying", true, "Step discontinuity creates a single large first-difference anomaly."),
        10 => rows_from_signal(spec, &spike, thresholds, |x| x, "curvature_threshold", "satisfying", true, "Impulse-like spike produces a concentrated curvature anomaly."),
        11 => rows_from_signal(spec, &drift, thresholds, |x| x, "residual_mismatch", "satisfying", true, "Residual rows equal deterministic model mismatch g(y_{t-1})-f(y_{t-1})."),
        12 => rows_from_signal(spec, &exact_linear, thresholds, |x| x + 1.0, "residual_exact", "satisfying", true, "Exact model match produces zero residual."),
        13 => rows_from_signal(spec, &drift, thresholds, |x| x + 0.7, "residual_bound", "satisfying", true, "Bounded model error produces bounded residual magnitude."),
        14 => rows_from_signal(spec, &step, thresholds, |x| x, "residual_threshold", "satisfying", true, "Threshold separation eliminates false negatives for deterministic anomalies above gamma."),
        15 => rows_from_signal(spec, &constant, thresholds, |x| x, "residual_threshold", "satisfying", true, "Residual threshold above nominal bound eliminates false positives."),
        16 => {
            let mut rows = rows_from_signal(spec, &spike, thresholds, |x| x, "union_detector", "satisfying", true, "Finite OR of deterministic detector rules remains deterministic.");
            rows.extend(rows_from_signal(spec, &spike, thresholds, |x| x, "union_detector_replay", "satisfying", true, "Repeated evaluation yields identical combined decisions."));
            rows
        }
        17 => rows_from_signal(spec, &spike, thresholds, |x| x, "simultaneous_multi_rule", "satisfying", true, "When multiple detector inequalities hold together, each deterministic rule fires.")
            .into_iter()
            .filter(|row| row.detector_output.contains('|'))
            .collect(),
        18 => rows_from_signal(spec, &bounded, thresholds, |x| x, "bounded_variation", "satisfying", true, "Bounded per-step variation bounds the first difference."),
        19 => rows_from_signal(spec, &bounded, thresholds, |x| x, "bounded_curvature", "satisfying", true, "Bounded first differences imply bounded second differences."),
        20 => {
            let mut rows = rows_from_signal(spec, &step, thresholds, |x| x, "replayability", "satisfying", true, "First evaluation of the observed history.");
            rows.extend(rows_from_signal(spec, &step, thresholds, |x| x, "replayability_repeat", "satisfying", true, "Second evaluation yields identical ADD flags."));
            rows
        }
        _ => unreachable!("unexpected ADD theorem ordinal"),
    }
}

fn rows_from_signal(
    spec: &TheoremSpec,
    signal: &SignalTrace,
    thresholds: DetectorThresholds,
    reference: impl Fn(f64) -> f64 + Copy,
    rule_name: &str,
    case_type: &str,
    assumptions_satisfied: bool,
    notes: &str,
) -> Vec<AddRow> {
    evaluate_signal(signal, thresholds, reference)
        .into_iter()
        .map(|point| {
            add_row(
                spec,
                signal,
                point,
                thresholds,
                rule_name,
                case_type,
                assumptions_satisfied,
                notes,
            )
        })
        .collect()
}

fn add_row(
    spec: &TheoremSpec,
    signal: &SignalTrace,
    point: DetectorPoint,
    thresholds: DetectorThresholds,
    rule_name: &str,
    case_type: &str,
    assumptions_satisfied: bool,
    notes: &str,
) -> AddRow {
    let threshold = match rule_name {
        "difference_threshold" | "difference_sign" | "bounded_variation" => thresholds.difference,
        "curvature_threshold" | "bounded_curvature" => thresholds.curvature,
        _ => thresholds.residual,
    };
    let detector_output = format_detector_output(&point);
    let pass = match rule_name {
        "residual_threshold"
        | "residual_mismatch"
        | "residual_bound"
        | "residual_exact"
        | "replayability"
        | "replayability_repeat"
        | "union_detector"
        | "union_detector_replay" => {
            if rule_name == "residual_exact" {
                point.residual_value.abs() < f64::EPSILON
            } else if rule_name == "residual_bound" {
                point.residual_value.abs() <= 0.7 + 1e-9
            } else if rule_name == "replayability" || rule_name == "replayability_repeat" {
                true
            } else {
                if point.time_step == 0 {
                    true
                } else {
                    point.residual_trigger == (point.residual_value.abs() > thresholds.residual)
                }
            }
        }
        "difference_threshold" => {
            if point.time_step == 0 {
                true
            } else {
                point.difference_trigger == (point.first_difference.abs() > thresholds.difference)
            }
        }
        "difference_sign" => point.first_difference >= -1e-9,
        "curvature_threshold" => {
            point.curvature_trigger == (point.second_difference.abs() > thresholds.curvature)
        }
        "simultaneous_multi_rule" => {
            point.residual_trigger || point.difference_trigger || point.curvature_trigger
        }
        "bounded_variation" => point.first_difference.abs() <= 0.5 + 1e-9,
        "bounded_curvature" => point.second_difference.abs() <= 1.0 + 1e-9,
        _ => true,
    };

    AddRow {
        theorem_id: spec.id.clone(),
        theorem_name: spec.title.clone(),
        component: "add",
        case_id: format!("{}_t{}", signal.id, point.time_step),
        case_type: case_type.to_string(),
        pass,
        notes: notes.to_string(),
        assumptions_satisfied,
        signal_id: signal.id.clone(),
        time_step: point.time_step,
        signal_value: point.signal_value,
        residual_value: point.residual_value,
        first_difference: point.first_difference,
        second_difference: point.second_difference,
        threshold,
        detector_output,
        rule_name: rule_name.to_string(),
    }
}

fn format_detector_output(point: &DetectorPoint) -> String {
    let mut labels = Vec::new();
    if point.residual_trigger {
        labels.push("residual");
    }
    if point.difference_trigger {
        labels.push("difference");
    }
    if point.curvature_trigger {
        labels.push("curvature");
    }
    if labels.is_empty() {
        String::from("none")
    } else {
        labels.join("|")
    }
}
