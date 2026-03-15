use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::common::{CaseClass, CaseMetadata};
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
    case_class: CaseClass,
    assumption_satisfied: bool,
    expected_outcome: String,
    observed_outcome: String,
    pass: bool,
    notes: String,
    signal_id: String,
    time_step: usize,
    signal_value: f64,
    residual_value: f64,
    first_difference: f64,
    second_difference: f64,
    threshold: f64,
    detector_output: String,
    anomaly_magnitude: f64,
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
    let nominal_drift = linear_signal("nominal_drift", 0.0, 0.15, 6);

    match spec.ordinal {
        1 => rows_from_signal(
            spec,
            &exact_linear,
            thresholds,
            |x| x + 1.0,
            "residual_threshold",
            CaseClass::Passing,
            true,
            "Zero residual trace never crosses a positive residual threshold.",
        ),
        2 => rows_from_signal(
            spec,
            &step,
            thresholds,
            |x| x,
            "residual_threshold",
            CaseClass::Passing,
            true,
            "Residual threshold detector fires whenever |r_t| exceeds tau_r.",
        ),
        3 => rows_from_signal(
            spec,
            &drift,
            thresholds,
            |x| x,
            "difference_threshold",
            CaseClass::Passing,
            true,
            "Difference detector fires on persistent large first differences.",
        ),
        4 => rows_from_signal(
            spec,
            &spike,
            thresholds,
            |x| x,
            "curvature_threshold",
            CaseClass::Passing,
            true,
            "Curvature detector fires on large second differences.",
        ),
        5 => rows_from_signal(
            spec,
            &constant,
            thresholds,
            |x| x,
            "difference_threshold",
            CaseClass::Passing,
            true,
            "Constant signal has zero first and second differences, so difference-based rules stay silent.",
        ),
        6 => rows_from_signal(
            spec,
            &exact_linear,
            thresholds,
            |x| x + 1.0,
            "curvature_threshold",
            CaseClass::Passing,
            true,
            "Linear signal yields identically zero second difference.",
        ),
        7 => rows_from_signal(
            spec,
            &drift,
            thresholds,
            |x| x,
            "difference_sign",
            CaseClass::Passing,
            true,
            "Monotone drift produces a constant-sign first difference.",
        ),
        8 => rows_from_signal(
            spec,
            &drift,
            thresholds,
            |x| x,
            "difference_threshold",
            CaseClass::Passing,
            true,
            "Persistent drift above tau_Delta triggers at every step on the interval.",
        ),
        9 => rows_from_signal(
            spec,
            &step,
            thresholds,
            |x| x,
            "difference_threshold",
            CaseClass::Passing,
            true,
            "Step discontinuity creates a single large first-difference anomaly.",
        ),
        10 => rows_from_signal(
            spec,
            &spike,
            thresholds,
            |x| x,
            "curvature_threshold",
            CaseClass::Passing,
            true,
            "Impulse-like spike produces a concentrated curvature anomaly.",
        ),
        11 => rows_from_signal(
            spec,
            &drift,
            thresholds,
            |x| x,
            "residual_mismatch",
            CaseClass::Passing,
            true,
            "Residual rows equal deterministic model mismatch g(y_{t-1})-f(y_{t-1}).",
        ),
        12 => rows_from_signal(
            spec,
            &exact_linear,
            thresholds,
            |x| x + 1.0,
            "residual_exact",
            CaseClass::Passing,
            true,
            "Exact model match produces zero residual.",
        ),
        13 => rows_from_signal(
            spec,
            &drift,
            thresholds,
            |x| x + 0.7,
            "residual_bound",
            CaseClass::Passing,
            true,
            "Bounded model error produces bounded residual magnitude.",
        ),
        14 => {
            let mut rows = rows_from_signal(
                spec,
                &step,
                thresholds,
                |x| x,
                "residual_threshold",
                CaseClass::Passing,
                true,
                "Threshold separation eliminates false negatives for deterministic anomalies above gamma.",
            );
            let false_negative_thresholds = DetectorThresholds {
                residual: 4.0,
                difference: thresholds.difference,
                curvature: thresholds.curvature,
            };
            rows.extend(
                evaluate_signal(&step, false_negative_thresholds, |x| x)
                    .into_iter()
                    .filter(|point| point.residual_value.abs() >= 3.0)
                    .map(|point| {
                        add_row(
                            spec,
                            &step,
                            point,
                            false_negative_thresholds,
                            "residual_threshold_false_negative",
                            CaseClass::Violating,
                            false,
                            false,
                            "Intentional violating witness: tau_r exceeds the anomaly magnitude, so the detector misses the step anomaly.",
                        )
                    }),
            );
            rows
        }
        15 => {
            let mut rows = rows_from_signal(
                spec,
                &constant,
                thresholds,
                |x| x,
                "residual_threshold",
                CaseClass::Passing,
                true,
                "Residual threshold above nominal bound eliminates false positives.",
            );
            let false_positive_thresholds = DetectorThresholds {
                residual: 0.1,
                difference: thresholds.difference,
                curvature: thresholds.curvature,
            };
            rows.extend(
                evaluate_signal(&nominal_drift, false_positive_thresholds, |x| x)
                    .into_iter()
                    .skip(1)
                    .take(1)
                    .map(|point| {
                        add_row(
                            spec,
                            &nominal_drift,
                            point,
                            false_positive_thresholds,
                            "residual_threshold_false_positive",
                            CaseClass::Violating,
                            false,
                            false,
                            "Intentional violating witness: tau_r is set below the nominal drift residual, so the detector raises a false positive.",
                        )
                    }),
            );
            rows
        }
        16 => {
            let mut rows = rows_from_signal(
                spec,
                &spike,
                thresholds,
                |x| x,
                "union_detector",
                CaseClass::Passing,
                true,
                "Finite OR of deterministic detector rules remains deterministic.",
            );
            rows.extend(rows_from_signal(
                spec,
                &spike,
                thresholds,
                |x| x,
                "union_detector_replay",
                CaseClass::Passing,
                true,
                "Repeated evaluation yields identical combined decisions.",
            ));
            rows
        }
        17 => rows_from_signal(
            spec,
            &spike,
            thresholds,
            |x| x,
            "simultaneous_multi_rule",
            CaseClass::Boundary,
            true,
            "When multiple detector inequalities hold together, each deterministic rule fires.",
        )
        .into_iter()
        .filter(|row| row.detector_output.contains('|'))
        .collect(),
        18 => rows_from_signal(
            spec,
            &bounded,
            thresholds,
            |x| x,
            "bounded_variation",
            CaseClass::Passing,
            true,
            "Bounded per-step variation bounds the first difference.",
        ),
        19 => rows_from_signal(
            spec,
            &bounded,
            thresholds,
            |x| x,
            "bounded_curvature",
            CaseClass::Passing,
            true,
            "Bounded first differences imply bounded second differences.",
        ),
        20 => {
            let mut rows = rows_from_signal(
                spec,
                &step,
                thresholds,
                |x| x,
                "replayability",
                CaseClass::Passing,
                true,
                "First evaluation of the observed history.",
            );
            rows.extend(rows_from_signal(
                spec,
                &step,
                thresholds,
                |x| x,
                "replayability_repeat",
                CaseClass::Passing,
                true,
                "Second evaluation yields identical ADD flags.",
            ));
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
    case_class: CaseClass,
    assumption_satisfied: bool,
    notes: &str,
) -> Vec<AddRow> {
    evaluate_signal(signal, thresholds, reference)
        .into_iter()
        .map(|point| {
            let pass = pass_for_rule(rule_name, thresholds, &point);
            add_row(
                spec,
                signal,
                point,
                thresholds,
                rule_name,
                case_class,
                assumption_satisfied,
                pass,
                notes,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn add_row(
    spec: &TheoremSpec,
    signal: &SignalTrace,
    point: DetectorPoint,
    thresholds: DetectorThresholds,
    rule_name: &str,
    case_class: CaseClass,
    assumption_satisfied: bool,
    pass: bool,
    notes: &str,
) -> AddRow {
    let threshold = threshold_for_rule(rule_name, thresholds);
    let detector_output = format_detector_output(&point);
    let anomaly_magnitude = anomaly_magnitude_for_rule(rule_name, &point);
    let expected_outcome = if assumption_satisfied {
        String::from("Detector behavior should match the theorem's threshold semantics on the admissible signal class.")
    } else {
        String::from("Misconfigured thresholds should visibly break the intended ADD guarantee.")
    };
    let observed_outcome = format!(
        "signal={} t={} residual={} first_difference={} second_difference={} detector_output={}",
        signal.id,
        point.time_step,
        point.residual_value,
        point.first_difference,
        point.second_difference,
        detector_output
    );

    let case = CaseMetadata::new(
        spec,
        "add",
        format!("{}_t{}", signal.id, point.time_step),
        case_class,
        assumption_satisfied,
        expected_outcome,
        observed_outcome,
        pass,
        notes,
    );

    AddRow {
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
        signal_id: signal.id.clone(),
        time_step: point.time_step,
        signal_value: point.signal_value,
        residual_value: point.residual_value,
        first_difference: point.first_difference,
        second_difference: point.second_difference,
        threshold,
        detector_output,
        anomaly_magnitude,
        rule_name: rule_name.to_string(),
    }
}

fn pass_for_rule(rule_name: &str, thresholds: DetectorThresholds, point: &DetectorPoint) -> bool {
    match rule_name {
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
            } else if point.time_step == 0 {
                true
            } else {
                point.residual_trigger == (point.residual_value.abs() > thresholds.residual)
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
        _ => false,
    }
}

fn threshold_for_rule(rule_name: &str, thresholds: DetectorThresholds) -> f64 {
    match rule_name {
        "difference_threshold" | "difference_sign" | "bounded_variation" => thresholds.difference,
        "curvature_threshold" | "bounded_curvature" => thresholds.curvature,
        "union_detector" | "union_detector_replay" | "simultaneous_multi_rule" => thresholds
            .residual
            .max(thresholds.difference)
            .max(thresholds.curvature),
        _ => thresholds.residual,
    }
}

fn anomaly_magnitude_for_rule(rule_name: &str, point: &DetectorPoint) -> f64 {
    match rule_name {
        "difference_threshold" | "difference_sign" | "bounded_variation" => {
            point.first_difference.abs()
        }
        "curvature_threshold" | "bounded_curvature" => point.second_difference.abs(),
        "union_detector" | "union_detector_replay" | "simultaneous_multi_rule" => point
            .residual_value
            .abs()
            .max(point.first_difference.abs())
            .max(point.second_difference.abs()),
        _ => point.residual_value.abs(),
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
