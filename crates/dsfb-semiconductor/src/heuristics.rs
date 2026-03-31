use crate::metrics::{BenchmarkMetrics, MotifMetric};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicEntry {
    pub motif_name: String,
    pub signature_definition: String,
    pub applicable_dataset: String,
    pub provenance_status: String,
    pub interpretation: String,
    pub severity: String,
    pub confidence: String,
    pub recommended_action: String,
    pub escalation_policy: String,
    pub non_unique_warning: String,
    pub known_limitations: String,
    pub observed_point_hits: usize,
    pub observed_run_hits: usize,
    pub pre_failure_window_run_hits: usize,
    pub pre_failure_window_precision_proxy: Option<f64>,
    pub status_note: String,
}

pub fn build_heuristics_bank(
    metrics: &BenchmarkMetrics,
    dataset_name: &str,
) -> Vec<HeuristicEntry> {
    let sustained = motif(metrics, "pre_failure_slow_drift");
    let abrupt = motif(metrics, "transient_excursion");
    let grazing = motif(metrics, "recurrent_boundary_approach");

    vec![
        HeuristicEntry {
            motif_name: "pre_failure_slow_drift".into(),
            signature_definition:
                "Residual norm exceeds 0.5*rho with drift above the healthy-window drift threshold."
                    .into(),
            applicable_dataset: dataset_name.into(),
            provenance_status: observed_status(sustained),
            interpretation:
                "Candidate early-warning drift motif that supports closer monitoring or maintenance review."
                    .into(),
            severity: "review".into(),
            confidence: confidence_note(sustained),
            recommended_action:
                "Increase review cadence, inspect neighboring channels, and corroborate with process context before intervention."
                    .into(),
            escalation_policy:
                "Escalate when the motif persists across repeated runs or is corroborated by scalar alarms and engineering context."
                    .into(),
            non_unique_warning:
                "This motif is not mechanism-specific and may reflect multiple latent causes."
                    .into(),
            known_limitations:
                "SECOM is anonymized and instance-level, so this motif does not support chamber-level attribution on its own."
                    .into(),
            observed_point_hits: sustained.point_hits,
            observed_run_hits: sustained.run_hits,
            pre_failure_window_run_hits: sustained.pre_failure_window_run_hits,
            pre_failure_window_precision_proxy: sustained.pre_failure_window_precision_proxy,
            status_note: format!(
                "Observed {} points and {} run hits; {} of those run hits fall inside the configured pre-failure windows.",
                sustained.point_hits, sustained.run_hits, sustained.pre_failure_window_run_hits
            ),
        },
        HeuristicEntry {
            motif_name: "transient_excursion".into(),
            signature_definition:
                "Residual norm enters the boundary zone with slew above the healthy-window slew threshold."
                    .into(),
            applicable_dataset: dataset_name.into(),
            provenance_status: observed_status(abrupt),
            interpretation:
                "Compatible with transient upset or abrupt regime change, but not uniquely diagnostic."
                    .into(),
            severity: "watch".into(),
            confidence: confidence_note(abrupt),
            recommended_action:
                "Check for corroborating context, inspect neighboring channels, and prefer confirmation over immediate intervention."
                    .into(),
            escalation_policy:
                "Escalate only when repeated, clustered with other motifs, or accompanied by direct envelope violations."
                    .into(),
            non_unique_warning:
                "A transient excursion can reflect measurement noise, regime switch, or genuine degradation."
                    .into(),
            known_limitations:
                "A single abrupt excursion does not identify physical cause and may not persist long enough for confident attribution."
                    .into(),
            observed_point_hits: abrupt.point_hits,
            observed_run_hits: abrupt.run_hits,
            pre_failure_window_run_hits: abrupt.pre_failure_window_run_hits,
            pre_failure_window_precision_proxy: abrupt.pre_failure_window_precision_proxy,
            status_note: format!(
                "Observed {} points and {} run hits; {} of those run hits fall inside the configured pre-failure windows.",
                abrupt.point_hits, abrupt.run_hits, abrupt.pre_failure_window_run_hits
            ),
        },
        HeuristicEntry {
            motif_name: "recurrent_boundary_approach".into(),
            signature_definition:
                "Residual norm revisits the boundary zone repeatedly without a confirmed envelope exit."
                    .into(),
            applicable_dataset: dataset_name.into(),
            provenance_status: observed_status(grazing),
            interpretation:
                "Ambiguous precursor motif that warrants continued observation rather than decisive attribution."
                    .into(),
            severity: "review".into(),
            confidence: confidence_note(grazing),
            recommended_action:
                "Track persistence, compare against the scalar baselines, and prioritize manual review over automatic maintenance action."
                    .into(),
            escalation_policy:
                "Escalate when recurrent grazing concentrates in pre-failure windows or transitions into direct violations."
                    .into(),
            non_unique_warning:
                "Repeated boundary grazing can arise from nuisance variation as well as meaningful precursor structure."
                    .into(),
            known_limitations:
                "This motif is especially sensitive to envelope and drift thresholds, so calibration materially affects its prevalence."
                    .into(),
            observed_point_hits: grazing.point_hits,
            observed_run_hits: grazing.run_hits,
            pre_failure_window_run_hits: grazing.pre_failure_window_run_hits,
            pre_failure_window_precision_proxy: grazing.pre_failure_window_precision_proxy,
            status_note: format!(
                "Observed {} points and {} run hits; {} of those run hits fall inside the configured pre-failure windows.",
                grazing.point_hits, grazing.run_hits, grazing.pre_failure_window_run_hits
            ),
        },
    ]
}

fn motif<'a>(metrics: &'a BenchmarkMetrics, motif_name: &str) -> &'a MotifMetric {
    metrics
        .motif_metrics
        .iter()
        .find(|metric| metric.motif_name == motif_name)
        .unwrap_or_else(|| panic!("missing motif metric for {motif_name}"))
}

fn observed_status(metric: &MotifMetric) -> String {
    if metric.point_hits > 0 {
        "SECOM-observed".into()
    } else {
        "framework-defined".into()
    }
}

fn confidence_note(metric: &MotifMetric) -> String {
    if metric.point_hits > 0 {
        "Stage-II observed on SECOM; interpretive and non-mechanistic.".into()
    } else {
        "Framework-defined only; not yet observed in the current run.".into()
    }
}
