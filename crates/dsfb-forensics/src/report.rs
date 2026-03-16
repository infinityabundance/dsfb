//! Report generation and DSFB seal assignment.
//!
//! References: `CORE-08` for explicit anomaly accounting, `CORE-10` for
//! full-stack consistency, and `TMTR-04`/`TMTR-10` for stabilized trust traces.

use serde::Serialize;

use crate::auditor::ForensicRunSummary;

/// Integrity level assigned to the run.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum SealLevel {
    /// Severe causal fragmentation or silent-failure risk was observed.
    Level1,
    /// The run stayed mostly coherent but recorded bounded structural debt.
    Level2,
    /// The run stayed causally coherent with no silent failures.
    Level3,
}

/// Award a DSFB seal of integrity for the run.
///
/// References: `CORE-08`, `CORE-10`, `TMTR-04`, and `TMTR-10`.
pub fn award_seal(summary: &ForensicRunSummary) -> SealLevel {
    if summary.shatter_events == 0
        && summary.silent_failures == 0
        && summary.reasoning_consistency >= 0.92
    {
        SealLevel::Level3
    } else if summary.reasoning_consistency >= 0.75 && summary.silent_failures <= 2 {
        SealLevel::Level2
    } else {
        SealLevel::Level1
    }
}

/// Render the human-readable markdown report.
///
/// References: `CORE-08`, `CORE-10`, and `TMTR-04`.
pub fn render_markdown_report(summary: &ForensicRunSummary) -> String {
    let seal = match summary.seal {
        SealLevel::Level1 => "Level 1",
        SealLevel::Level2 => "Level 2",
        SealLevel::Level3 => "Level 3",
    };
    let mut body = String::new();
    body.push_str("# DSFB Forensics Report\n\n");
    body.push_str("## Verdict\n\n");
    body.push_str(&format!(
        "- DSFB Seal of Integrity: **{}**\n- Reasoning consistency: **{:.3}**\n- Shatter events: **{}**\n- Silent failures: **{}**\n\n",
        seal,
        summary.reasoning_consistency,
        summary.shatter_events,
        summary.silent_failures,
    ));
    body.push_str("## Run Summary\n\n");
    body.push_str(&format!(
        "- Input trace: `{}`\n- Steps: `{}`\n- Channels: `{}`\n- Slew threshold: `{:.3}`\n- Trust alpha: `{:.3}`\n- Baseline comparison: `{}`\n\n",
        summary.input_trace,
        summary.total_steps,
        summary.channel_count,
        summary.slew_threshold,
        summary.trust_alpha,
        if summary.baseline_enabled { "on" } else { "off" },
    ));
    body.push_str("## Structural Findings\n\n");
    body.push_str(&format!(
        "- Pruned or down-weighted updates: `{}`\n- EKF accepted updates: `{}`\n- Maximum causal depth: `{}`\n- Maximum weak components: `{}`\n- Mean trust score: `{:.3}`\n- Minimum trust score: `{:.3}`\n\n",
        summary.pruned_updates,
        summary.baseline_accepted_updates,
        summary.max_causal_depth,
        summary.max_components,
        summary.mean_trust_score,
        summary.min_trust_score,
    ));
    body.push_str("## Complexity Guarantee\n\n");
    body.push_str(&format!(
        "- Per-step bound: `{}`\n- Maximum primitive operations observed: `{}`\n- Maximum transient memory words observed: `{}`\n\n",
        summary.complexity_bound,
        summary.max_total_ops,
        summary.max_memory_words,
    ));
    if let Some(scenario) = &summary.benchmark_scenario {
        body.push_str("## Benchmark Early-Warning Analysis\n\n");
        body.push_str(&format!(
            "- Scenario: `{}`\n- Drift start step: `{}`\n- Anomaly channels: `{:?}`\n- Conventional QA fail step: `{}`\n- DSFB first alert step: `{}`\n- DSFB lead time (steps): `{}`\n- DSFB lead time (seconds): `{}`\n- Early detection: `{}`\n\n",
            scenario,
            summary.benchmark_drift_start_step.unwrap_or_default(),
            summary.benchmark_anomaly_channels,
            option_step_text(summary.conventional_qa_fail_step),
            option_step_text(summary.dsfb_first_alert_step),
            option_isize_text(summary.dsfb_lead_time_steps),
            option_f64_text(summary.dsfb_lead_time_seconds),
            summary.degradation_detected_early,
        ));
        body.push_str("Interpretation: ");
        body.push_str(&benchmark_interpretation(summary));
        body.push_str("\n\n");
    }
    if let Some(mae) = summary.dsfb_phi_mae {
        body.push_str("## Accuracy Context\n\n");
        body.push_str(&format!("- DSFB phi MAE: `{:.6}`\n", mae));
        if let Some(baseline_mae) = summary.ekf_phi_mae {
            body.push_str(&format!("- EKF phi MAE: `{:.6}`\n", baseline_mae));
        }
        body.push('\n');
    }
    body.push_str("## Rationale\n\n");
    match summary.seal {
        SealLevel::Level3 => body.push_str(
            "The causal topology remained connected, trust contraction stayed stable, and the EKF baseline did not accept measurements that the DSFB stack rejected as structurally inconsistent.\n",
        ),
        SealLevel::Level2 => body.push_str(
            "The run preserved most deterministic reasoning guarantees, but it accumulated bounded structural debt through limited shatter or silent-failure activity.\n",
        ),
        SealLevel::Level1 => body.push_str(
            "The run experienced enough fragmentation or silent-failure activity that the audit layer cannot certify strong reasoning integrity.\n",
        ),
    }
    body
}

fn benchmark_interpretation(summary: &ForensicRunSummary) -> &'static str {
    match (
        summary.dsfb_first_alert_step,
        summary.conventional_qa_fail_step,
        summary.degradation_detected_early,
    ) {
        (Some(_), Some(_), true) => {
            "DSFB raised a sustained structural warning before the conventional scalar QA threshold failed."
        }
        (Some(_), None, true) => {
            "DSFB raised a sustained structural warning even though the conventional scalar QA threshold never failed during the benchmark horizon."
        }
        (Some(_), Some(_), false) => {
            "DSFB did not provide lead time ahead of the conventional scalar QA threshold in this run."
        }
        _ => "The benchmark did not produce a sustained DSFB alert in this run.",
    }
}

fn option_step_text(value: Option<usize>) -> String {
    value
        .map(|step| step.to_string())
        .unwrap_or_else(|| "not reached".to_string())
}

fn option_isize_text(value: Option<isize>) -> String {
    value
        .map(|step| step.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn option_f64_text(value: Option<f64>) -> String {
    value
        .map(|seconds| format!("{seconds:.6}"))
        .unwrap_or_else(|| "n/a".to_string())
}
