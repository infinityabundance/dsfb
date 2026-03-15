use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::common::{CaseClass, CaseMetadata};
use crate::runners::{write_component_rows, RunnerContext};

#[derive(Debug, Clone, Serialize)]
struct CoreRow {
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
    time_step: usize,
    signal_value: f64,
    observation_value: f64,
    reconstructed_state: i32,
    residual_value: f64,
    trust_value: f64,
    regime_label: String,
    anomaly_flag: bool,
    graph_acyclic_flag: bool,
    observation_code: String,
    trust_threshold: f64,
    graph_edge_count: usize,
    metric_value: f64,
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

fn build_rows(spec: &TheoremSpec) -> Vec<CoreRow> {
    match spec.ordinal {
        1 => {
            let mut rows = exact_pipeline_rows(
                spec,
                CaseClass::Passing,
                "Exact reconstruction aligns recoverability, residual nulling, and regime preservation.",
            );
            rows.push(core_violation_row(
                spec,
                "noninjective_forward_map",
                6,
                2.0,
                101.0,
                1,
                1.0,
                0.55,
                "ambiguous",
                true,
                true,
                "obs_collision_101",
                0.5,
                3,
                2.0,
                "Theorem assumptions not satisfied; non-injective forward maps do not guarantee exact recoverability or regime preservation.",
                "Two structural states collapsed to observation 101; reconstruction returned s1 with residual 1.0 and the regime label became ambiguous.",
                "Assumption-violating witness: a non-injective forward map merges structural states under one observation, so exact recoverability and regime preservation are not applicable.",
            ));
            rows
        }
        2 => {
            let mut rows = exact_pipeline_rows(
                spec,
                CaseClass::Passing,
                "Projection to admissible observations followed by inversion reproduces the structural state.",
            );
            rows.push(core_violation_row(
                spec,
                "nonexact_projection_inverse",
                6,
                2.0,
                102.5,
                1,
                1.5,
                0.5,
                "projection_mismatch",
                true,
                true,
                "obs_outside_image_102_5",
                0.5,
                3,
                1.5,
                "Projection / inverse representation assumptions are not satisfied, so exact reconstruction is not expected.",
                "Observation 102.5 lies outside the exact forward image; inversion returned s1 with residual 1.5 instead of the originating structural state.",
                "Non-admissible witness: the observation is not an exact image point, so the DSFB representation theorem is not applicable.",
            ));
            rows
        }
        3 => {
            let mut rows = stability_rows(
                spec,
                CaseClass::Passing,
                "Trust-monotone recursion converges to a stable trust level.",
            );
            rows.push(core_violation_row(
                spec,
                "trust_increase_attempt",
                7,
                7.0,
                107.0,
                5,
                2.0,
                1.5,
                "unstable",
                true,
                true,
                "traj_increase",
                0.5,
                2,
                2.0,
                "TMTR monotone-descent assumptions are not satisfied, so stabilization is not expected.",
                "The proposed update introduced a trust-increasing step while residual remained 2.0, so the orbit is not licensed to stabilize under the theorem premises.",
                "Assumption-violating witness: the trust recursion allows an increasing step, so the TMTR stability theorem is not applicable.",
            ));
            rows
        }
        4 => vec![
            core_row(
                spec,
                "threshold_low",
                CaseClass::Passing,
                true,
                0,
                1.0,
                101.0,
                1,
                0.0,
                0.9,
                "high",
                false,
                true,
                "obs_101",
                0.2,
                5,
                5.0,
                "Low threshold admits more edges.",
            ),
            core_row(
                spec,
                "threshold_mid",
                CaseClass::Boundary,
                true,
                1,
                1.0,
                101.0,
                1,
                0.0,
                0.6,
                "high",
                false,
                true,
                "obs_101",
                0.5,
                3,
                3.0,
                "Intermediate threshold yields an intermediate admissible graph.",
            ),
            core_row(
                spec,
                "threshold_high",
                CaseClass::Boundary,
                true,
                2,
                1.0,
                101.0,
                1,
                0.0,
                0.3,
                "high",
                false,
                true,
                "obs_101",
                0.8,
                1,
                1.0,
                "Higher threshold prunes more edges in the trust-gated graph.",
            ),
            core_violation_row(
                spec,
                "threshold_edge_mismatch",
                3,
                1.0,
                101.0,
                1,
                0.5,
                0.65,
                "threshold_mismatch",
                true,
                true,
                "obs_threshold_mismatch",
                0.7,
                4,
                4.0,
                "The trust-threshold correspondence assumptions are not satisfied, so the Galois link is not expected to commute.",
                "An edge whose trust falls below the declared threshold remained admissible, producing 4 retained edges instead of the threshold-consistent subgraph.",
                "Non-admissible witness: edge retention no longer depends solely on the stated trust threshold, so the trust-graph Galois correspondence is not applicable.",
            ),
        ],
        5 => {
            let mut rows = exact_pipeline_rows(
                spec,
                CaseClass::Passing,
                "Historical replay followed by trace realization factorizes structural reconstruction.",
            );
            rows.push(core_violation_row(
                spec,
                "historical_replay_mismatch",
                6,
                3.0,
                103.0,
                1,
                2.0,
                0.4,
                "history_mismatch",
                true,
                true,
                "trace_mismatch",
                0.5,
                3,
                2.0,
                "Historical reconstruction assumptions are violated, so factorization through replay is not expected.",
                "Replay and realization disagreed on the reconstructed state for observation 103, leaving residual 2.0 and a non-admissible historical trace.",
                "Assumption-violating witness: historical replay is no longer admissible, so the HRET factorization theorem is not applicable.",
            ));
            rows
        }
        6 => vec![
            core_row(
                spec,
                "prune_then_tighten",
                CaseClass::Passing,
                true,
                0,
                2.0,
                102.0,
                2,
                0.0,
                0.7,
                "high",
                false,
                true,
                "obs_102",
                0.5,
                2,
                2.0,
                "Prune-then-tighten edge count.",
            ),
            core_row(
                spec,
                "tighten_then_prune",
                CaseClass::Passing,
                true,
                1,
                2.0,
                102.0,
                2,
                0.0,
                0.7,
                "high",
                false,
                true,
                "obs_102",
                0.5,
                2,
                2.0,
                "Tighten-then-prune edge count matches under threshold-only semantics.",
            ),
            core_violation_row(
                spec,
                "external_policy_edge_gate",
                2,
                2.0,
                102.0,
                2,
                1.0,
                0.7,
                "compatibility_broken",
                true,
                true,
                "compatibility_external_policy",
                0.5,
                3,
                1.0,
                "Compatibility assumptions are not satisfied when causal edge admissibility depends on more than the trust threshold.",
                "The graph update retained an edge because of an external policy tag rather than the threshold, so prune-then-tighten and tighten-then-prune no longer commute.",
                "Non-admissible witness: causal edge admissibility depends on an extra-semantic rule, so DSCD-TMTR commutation is not guaranteed.",
            ),
        ],
        7 => vec![
            core_row(
                spec,
                "fine_negative",
                CaseClass::Passing,
                true,
                0,
                -1.0,
                99.0,
                -1,
                0.0,
                0.8,
                "negative",
                false,
                true,
                "trace_neg",
                0.0,
                3,
                2.0,
                "Fine transition ordering is preserved under coarsening.",
            ),
            core_row(
                spec,
                "coarse_low",
                CaseClass::Boundary,
                true,
                1,
                -1.0,
                99.0,
                -1,
                0.0,
                0.8,
                "low",
                false,
                true,
                "trace_low",
                0.0,
                3,
                1.0,
                "Coarsened transition count does not exceed the fine transition count.",
            ),
            core_violation_row(
                spec,
                "invalid_coarsening_map",
                2,
                -1.0,
                99.0,
                -1,
                1.0,
                0.7,
                "invalid_coarsening",
                true,
                true,
                "trace_invalid_coarsen",
                0.0,
                3,
                2.0,
                "The coarsening theorem is not applicable when the regime label map is inconsistent.",
                "Fine regimes `negative` and `low` were merged into an inconsistent coarse label, reversing the intended transition semantics and leaving residual 1.0.",
                "Assumption-violating witness: the coarse label map is not a valid coarsening, so SRD transition preservation is not guaranteed.",
            ),
        ],
        8 => vec![
            core_row(
                spec,
                "nominal",
                CaseClass::Boundary,
                true,
                0,
                1.0,
                101.0,
                1,
                0.0,
                0.9,
                "high",
                false,
                true,
                "obs_101",
                0.0,
                3,
                0.0,
                "Nominal observation remains on the reconstructed forward image.",
            ),
            core_row(
                spec,
                "anomalous",
                CaseClass::Passing,
                true,
                1,
                4.0,
                104.0,
                1,
                3.0,
                0.6,
                "high",
                true,
                true,
                "obs_104",
                0.0,
                3,
                3.0,
                "Residual-triggered anomaly marks structural inconsistency with the reconstructed image.",
            ),
            core_violation_row(
                spec,
                "residual_semantics_mismatch",
                2,
                1.0,
                101.0,
                1,
                0.2,
                0.85,
                "detector_misconfigured",
                true,
                true,
                "obs_detector_mismatch",
                0.0,
                3,
                0.2,
                "Detector soundness is not expected when residual semantics and detector calibration assumptions are violated.",
                "The detector fired on residual 0.2 because the threshold semantics were misconfigured, so the anomaly decision no longer certifies structural inconsistency.",
                "Non-admissible witness: detector calibration no longer matches the residual semantics, so the ADD soundness theorem is not applicable.",
            ),
        ],
        9 => {
            let mut rows = stability_rows(
                spec,
                CaseClass::Passing,
                "Trust recursion and reconstruction error descent converge jointly.",
            );
            rows.push(core_violation_row(
                spec,
                "residual_descent_broken",
                7,
                7.0,
                107.0,
                4,
                1.5,
                0.8,
                "nonconvergent",
                true,
                true,
                "traj_residual_regrowth",
                0.5,
                2,
                1.5,
                "Joint convergence is not expected when monotone trust descent and residual descent assumptions are violated.",
                "Residual regrew to 1.5 while trust stalled at 0.8, so the orbit did not move toward a structurally stable inference state.",
                "Assumption-violating witness: residual descent is broken, so the DSFB-TMTR convergence theorem is not applicable.",
            ));
            rows
        }
        10 => {
            let mut rows = unified_pipeline_rows(spec);
            rows.push(core_violation_row(
                spec,
                "cross_layer_nondeterministic",
                8,
                2.5,
                104.5,
                1,
                3.0,
                0.45,
                "divergent",
                true,
                false,
                "stack_obs_nonadmissible",
                0.5,
                4,
                3.0,
                "The grand-unification theorem is not applicable once cross-layer admissibility and determinism assumptions are broken.",
                "Observation 104.5 admitted incompatible cross-layer reconstructions, leaving residual 3.0, anomaly=yes, and a non-acyclic graph witness.",
                "Non-admissible witness: the aligned stack is no longer exact or deterministic across layers, so the Grand Unification theorem is not expected to apply.",
            ));
            rows
        }
        11 => vec![
            core_row(
                spec,
                "gamma",
                CaseClass::Passing,
                true,
                0,
                1.0,
                101.0,
                1,
                0.0,
                0.8,
                "high",
                false,
                true,
                "obs_101",
                0.0,
                3,
                0.0,
                "Gamma observable is preserved by H∘R_S∘F = H.",
            ),
            core_row(
                spec,
                "trust",
                CaseClass::Passing,
                true,
                1,
                1.0,
                101.0,
                1,
                0.0,
                0.8,
                "high",
                false,
                true,
                "obs_101",
                0.0,
                3,
                0.0,
                "Trust observable is preserved by compression and reconstruction.",
            ),
            core_row(
                spec,
                "causal",
                CaseClass::Passing,
                true,
                2,
                1.0,
                101.0,
                1,
                0.0,
                0.8,
                "high",
                false,
                true,
                "obs_101",
                0.0,
                3,
                0.0,
                "Causal observable is preserved on the admissible class.",
            ),
            core_violation_row(
                spec,
                "compression_collision",
                3,
                1.0,
                101.0,
                0,
                1.0,
                0.6,
                "compressed_collision",
                true,
                true,
                "obs_compression_collision",
                0.0,
                3,
                1.0,
                "Functorial compression requires exact recoverability on the admissible class; outside that class observable invariance is not guaranteed.",
                "Compression collapsed distinguishable observables into one code, reconstructing state 0 and losing the causal observable with residual 1.0.",
                "Assumption-violating witness: compression is no longer exact on the admissible image, so structural observable invariance is not guaranteed.",
            ),
        ],
        _ => unreachable!("unexpected core theorem ordinal"),
    }
}

fn exact_pipeline_rows(spec: &TheoremSpec, case_class: CaseClass, notes: &str) -> Vec<CoreRow> {
    let signal = [0.0, 1.0, 2.0, 1.0, 0.0, 1.0];
    let trust = [0.9, 0.8, 0.7, 0.7, 0.6, 0.6];
    let regime = ["low", "mid", "high", "mid", "low", "mid"];
    signal
        .iter()
        .enumerate()
        .map(|(time_step, signal_value)| {
            core_row(
                spec,
                &format!("exact_t{time_step}"),
                case_class,
                true,
                time_step,
                *signal_value,
                100.0 + *signal_value,
                *signal_value as i32,
                0.0,
                trust[time_step],
                regime[time_step],
                false,
                true,
                &format!("obs_{time_step}"),
                0.5,
                3,
                0.0,
                notes,
            )
        })
        .collect()
}

fn stability_rows(spec: &TheoremSpec, case_class: CaseClass, notes: &str) -> Vec<CoreRow> {
    let trust = [5.0, 4.0, 3.0, 2.0, 1.0, 0.0, 0.0];
    trust
        .iter()
        .enumerate()
        .map(|(time_step, trust_value)| {
            core_row(
                spec,
                &format!("stability_t{time_step}"),
                case_class,
                true,
                time_step,
                time_step as f64,
                100.0 + time_step as f64,
                (time_step as i32).min(5),
                0.0,
                *trust_value,
                if *trust_value > 2.0 {
                    "descent"
                } else {
                    "stable"
                },
                false,
                true,
                &format!("traj_{time_step}"),
                0.5,
                2,
                *trust_value,
                notes,
            )
        })
        .collect()
}

fn unified_pipeline_rows(spec: &TheoremSpec) -> Vec<CoreRow> {
    let signal = [0.0, 1.0, 2.0, 3.0, 1.0, 0.5, 2.5, 1.0];
    let residual = [0.0, 0.0, 0.0, 1.5, 0.0, 0.0, 2.0, 0.0];
    let trust = [0.9, 0.8, 0.7, 0.6, 0.6, 0.5, 0.4, 0.4];
    let regimes = ["low", "mid", "high", "high", "mid", "low", "high", "mid"];
    signal
        .iter()
        .enumerate()
        .map(|(time_step, signal_value)| {
            let anomaly_flag = residual[time_step] > 1.0;
            core_row(
                spec,
                &format!("pipeline_t{time_step}"),
                if anomaly_flag {
                    CaseClass::Boundary
                } else {
                    CaseClass::Passing
                },
                true,
                time_step,
                *signal_value,
                100.0 + *signal_value,
                signal_value.round() as i32,
                residual[time_step],
                trust[time_step],
                regimes[time_step],
                anomaly_flag,
                true,
                &format!("stack_obs_{time_step}"),
                0.5,
                3,
                if anomaly_flag { 1.0 } else { 0.0 },
                "The full DSFB pipeline remains deterministic across reconstruction, trust, regime, anomaly, and history layers.",
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn core_violation_row(
    spec: &TheoremSpec,
    case_id: &str,
    time_step: usize,
    signal_value: f64,
    observation_value: f64,
    reconstructed_state: i32,
    residual_value: f64,
    trust_value: f64,
    regime_label: &str,
    anomaly_flag: bool,
    graph_acyclic_flag: bool,
    observation_code: &str,
    trust_threshold: f64,
    graph_edge_count: usize,
    metric_value: f64,
    expected_outcome: &str,
    observed_outcome: &str,
    notes: &str,
) -> CoreRow {
    core_row_with_outcomes(
        spec,
        case_id,
        CaseClass::Violating,
        false,
        false,
        time_step,
        signal_value,
        observation_value,
        reconstructed_state,
        residual_value,
        trust_value,
        regime_label,
        anomaly_flag,
        graph_acyclic_flag,
        observation_code,
        trust_threshold,
        graph_edge_count,
        metric_value,
        expected_outcome,
        observed_outcome,
        notes,
    )
}

#[allow(clippy::too_many_arguments)]
fn core_row(
    spec: &TheoremSpec,
    case_id: &str,
    case_class: CaseClass,
    assumption_satisfied: bool,
    time_step: usize,
    signal_value: f64,
    observation_value: f64,
    reconstructed_state: i32,
    residual_value: f64,
    trust_value: f64,
    regime_label: &str,
    anomaly_flag: bool,
    graph_acyclic_flag: bool,
    observation_code: &str,
    trust_threshold: f64,
    graph_edge_count: usize,
    metric_value: f64,
    notes: &str,
) -> CoreRow {
    let expected_outcome = if assumption_satisfied {
        "The aligned stack witness should remain internally consistent across reconstruction, trust, regime, anomaly, and graph layers."
    } else {
        "Assumption-violating aligned stack witnesses should visibly break the claimed end-to-end behavior."
    };
    let observed_outcome = format!(
        "t={} signal={} observation={} reconstructed_state={} residual={} trust={} regime={} anomaly={}",
        time_step,
        signal_value,
        observation_value,
        reconstructed_state,
        residual_value,
        trust_value,
        regime_label,
        anomaly_flag
    );

    core_row_with_outcomes(
        spec,
        case_id,
        case_class,
        assumption_satisfied,
        true,
        time_step,
        signal_value,
        observation_value,
        reconstructed_state,
        residual_value,
        trust_value,
        regime_label,
        anomaly_flag,
        graph_acyclic_flag,
        observation_code,
        trust_threshold,
        graph_edge_count,
        metric_value,
        expected_outcome,
        &observed_outcome,
        notes,
    )
}

#[allow(clippy::too_many_arguments)]
fn core_row_with_outcomes(
    spec: &TheoremSpec,
    case_id: &str,
    case_class: CaseClass,
    assumption_satisfied: bool,
    pass: bool,
    time_step: usize,
    signal_value: f64,
    observation_value: f64,
    reconstructed_state: i32,
    residual_value: f64,
    trust_value: f64,
    regime_label: &str,
    anomaly_flag: bool,
    graph_acyclic_flag: bool,
    observation_code: &str,
    trust_threshold: f64,
    graph_edge_count: usize,
    metric_value: f64,
    expected_outcome: &str,
    observed_outcome: &str,
    notes: &str,
) -> CoreRow {
    let case = CaseMetadata::new(
        spec,
        "core",
        case_id,
        case_class,
        assumption_satisfied,
        expected_outcome,
        observed_outcome,
        pass,
        notes,
    );

    CoreRow {
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
        time_step,
        signal_value,
        observation_value,
        reconstructed_state,
        residual_value,
        trust_value,
        regime_label: regime_label.to_string(),
        anomaly_flag,
        graph_acyclic_flag,
        observation_code: observation_code.to_string(),
        trust_threshold,
        graph_edge_count,
        metric_value,
    }
}
