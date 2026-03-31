use crate::config::PipelineConfig;
use crate::dataset::phm2018::Phm2018SupportStatus;
use crate::dataset::secom::SecomArchiveLayout;
use crate::error::Result;
use crate::heuristics::HeuristicEntry;
use crate::metrics::{BenchmarkMetrics, MotifMetric};
use crate::plots::FigureManifest;
use crate::precursor::PspEvaluation;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
struct ArtifactInventoryEntry {
    path: String,
    role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportArtifacts {
    pub markdown_path: PathBuf,
    pub tex_path: PathBuf,
    pub pdf_path: Option<PathBuf>,
    pub pdf_error: Option<String>,
}

pub fn write_reports(
    run_dir: &Path,
    config: &PipelineConfig,
    metrics: &BenchmarkMetrics,
    psp: &PspEvaluation,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
    secom_layout: &SecomArchiveLayout,
) -> Result<ReportArtifacts> {
    let markdown_path = run_dir.join("engineering_report.md");
    let tex_path = run_dir.join("engineering_report.tex");
    fs::write(
        &markdown_path,
        markdown_report(
            config,
            metrics,
            psp,
            figures,
            heuristics,
            phm_status,
            secom_layout,
        ),
    )?;
    fs::write(
        &tex_path,
        latex_report(
            config,
            metrics,
            psp,
            figures,
            heuristics,
            phm_status,
            secom_layout,
        ),
    )?;

    let (pdf_path, pdf_error) = compile_pdf(&tex_path, run_dir);

    Ok(ReportArtifacts {
        markdown_path,
        tex_path,
        pdf_path,
        pdf_error,
    })
}

fn markdown_report(
    config: &PipelineConfig,
    metrics: &BenchmarkMetrics,
    psp: &PspEvaluation,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
    secom_layout: &SecomArchiveLayout,
) -> String {
    let mut out = String::new();
    let artifact_inventory = artifact_inventory(figures);
    out.push_str("# DSFB Semiconductor Engineering Report\n\n");
    out.push_str("## Dataset\n\n");
    out.push_str("- Dataset: SECOM (UCI Machine Learning Repository)\n");
    out.push_str(
        "- Evidence class: Stage II public-benchmark evidence on real semiconductor data\n",
    );
    out.push_str("- Non-claim: this run does not establish SEMI compliance, production readiness, or physical root-cause attribution\n\n");

    out.push_str("## Archive Layout Note\n\n");
    out.push_str(&format!(
        "- Numeric columns parsed from `secom.data`: {}\n- Metadata attribute count claimed in `secom.names`: {}\n- Label rows parsed from `secom_labels.data`: {}\n- Label file includes timestamps: {}\n\n{}\n\n",
        secom_layout.data_file_numeric_column_count,
        secom_layout
            .metadata_attribute_count_claim
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into()),
        secom_layout.label_row_count,
        secom_layout.label_file_includes_timestamp,
        secom_layout.note,
    ));

    out.push_str("## Preprocessing Summary\n\n");
    out.push_str(&format!(
        "- Runs: {}\n- Features used by crate: {}\n- Passing runs: {}\n- Failure runs: {}\n- Dataset missing fraction: {:.4}\n- Healthy passing runs requested/found: {}/{}\n\n",
        metrics.summary.dataset_summary.run_count,
        metrics.summary.dataset_summary.feature_count,
        metrics.summary.dataset_summary.pass_count,
        metrics.summary.dataset_summary.fail_count,
        metrics.summary.dataset_summary.dataset_missing_fraction,
        metrics.summary.dataset_summary.healthy_pass_runs_requested,
        metrics.summary.dataset_summary.healthy_pass_runs_found,
    ));
    out.push_str("Missing values are kept as explicit missing entries during dataset loading and then imputed feature-wise with the healthy-window nominal mean before residual construction. This keeps the residual transform deterministic and auditable, but it does not make missingness semantically neutral.\n\n");

    out.push_str("## DSFB Instantiation\n\n");
    out.push_str(&format!(
        "- Nominal reference: healthy-window mean over first {} passing runs\n- Residual: x(k) - x_hat\n- Envelope radius rho: {:.1} * healthy-window residual std\n- Drift window W: {}\n- Boundary condition: |r| > {:.1} * rho and drift > {:.1} * healthy drift std\n- Slew threshold: {:.1} * healthy slew std\n- Recurrent-boundary grazing: {} hits in a {}-run window\n- Hysteresis confirmations: {}\n- Persistent-state minimum length: {}\n- Density window: {}\n- Baseline comparators: univariate 3-sigma residual threshold, plus univariate EWMA on residual norms with alpha = {:.2} and threshold mean + {:.1} * healthy EWMA std\n\n",
        config.healthy_pass_runs,
        config.envelope_sigma,
        config.drift_window,
        config.boundary_fraction_of_rho,
        config.drift_sigma_multiplier,
        config.slew_sigma_multiplier,
        config.grazing_min_hits,
        config.grazing_window,
        config.state_confirmation_steps,
        config.persistent_state_steps,
        config.density_window,
        config.ewma_alpha,
        config.ewma_sigma_multiplier,
    ));
    out.push_str(&format!(
        "Grammar logic in this crate is intentionally simple and deterministic: raw admissible, boundary, and violation states are derived from the envelope radius, outward drift, abrupt slew, and recurrent boundary grazing rules encoded in the saved parameter manifest. A hysteresis-confirmed state trace and persistence masks are then derived deterministically from those raw states.\n\nIn this crate, `DSFB Violation` means instantaneous hard envelope exit (`|r| > rho`). `Persistent Structural Precursor (PSP)` is a separate persistence-gated structural accumulation built from rolling boundary density, drift persistence with consistent residual sign, slew clustering, normalized EWMA occupancy, and heuristics-bank motif recurrence. The current PSP configuration uses `W = {}`, `K = {}`, `tau = {:.2}`, and fixed unit weights across the five scored PSP terms.\n\n",
        config.psp.window,
        config.psp.persistence_runs,
        config.psp.alert_tau,
    ));

    out.push_str("## Quantitative Summary\n\n");
    out.push_str(&format!(
        "- Analyzable features: {}\n- Threshold alarm points: {}\n- EWMA alarm points: {}\n- DSFB raw boundary points: {}\n- DSFB persistent boundary points: {}\n- DSFB raw violation points: {}\n- DSFB persistent violation points: {}\n- PSP alert points: {}\n- PSP alert runs: {}\n- Failure runs with preceding PSP signal ({}-run lookback): {}\n- Failure runs with preceding persistent DSFB boundary signal ({}-run lookback): {}\n- Failure runs with preceding persistent DSFB violation signal ({}-run lookback): {}\n- Failure runs with preceding EWMA signal ({}-run lookback): {}\n- Failure runs with preceding threshold signal ({}-run lookback): {}\n\n",
        metrics.summary.analyzable_feature_count,
        metrics.summary.threshold_alarm_points,
        metrics.summary.ewma_alarm_points,
        metrics.summary.dsfb_raw_boundary_points,
        metrics.summary.dsfb_persistent_boundary_points,
        metrics.summary.dsfb_raw_violation_points,
        metrics.summary.dsfb_persistent_violation_points,
        psp.summary.alert_point_count,
        psp.summary.alert_run_count,
        config.pre_failure_lookback_runs,
        psp.summary.failure_run_recall,
        config.pre_failure_lookback_runs,
        metrics
            .summary
            .failure_runs_with_preceding_dsfb_persistent_boundary_signal,
        config.pre_failure_lookback_runs,
        metrics
            .summary
            .failure_runs_with_preceding_dsfb_persistent_violation_signal,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_ewma_signal,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_threshold_signal,
    ));
    out.push_str(
        "In the current implementation, the raw DSFB `Violation` state and the raw threshold share the same envelope-exit condition `|r| > rho`, so raw violation/threshold coincidences are expected before persistence filtering. PSP is evaluated separately and should be interpreted as a persistent multi-signal structural overlay, not as a redefinition of the violation state.\n\n",
    );

    out.push_str("## Lead-Time and Nuisance Proxies\n\n");
    out.push_str(&format!(
        "- Mean PSP lead (runs): {}\n- Median PSP lead (runs): {}\n- Mean raw DSFB boundary lead (runs): {}\n- Mean persistent DSFB boundary lead (runs): {}\n- Mean raw DSFB violation lead (runs): {}\n- Mean persistent DSFB violation lead (runs): {}\n- Mean EWMA lead (runs): {}\n- Mean threshold lead (runs): {}\n- Mean PSP minus EWMA lead delta (runs): {}\n- Mean PSP minus threshold lead delta (runs): {}\n- Mean persistent DSFB boundary minus EWMA lead delta (runs): {}\n- Mean persistent DSFB boundary minus threshold lead delta (runs): {}\n- Pass-run nuisance proxy, PSP: {:.4}\n- Pass-run nuisance proxy, persistent DSFB boundary: {:.4}\n- Pass-run nuisance proxy, persistent DSFB violation: {:.4}\n- Pass-run nuisance proxy, EWMA: {:.4}\n- Pass-run nuisance proxy, threshold: {:.4}\n- Persistent boundary episodes: {}\n- Mean persistent boundary episode length: {}\n- Mean persistent non-escalating boundary episode fraction: {}\n\n",
        format_option_f64(psp.summary.mean_lead_time_runs),
        format_option_f64(psp.summary.median_lead_time_runs),
        format_option_f64(metrics.lead_time_summary.mean_raw_boundary_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_persistent_boundary_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_raw_violation_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_persistent_violation_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_ewma_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_threshold_lead_runs),
        format_option_f64(psp.summary.mean_lead_delta_vs_ewma_runs),
        format_option_f64(psp.summary.mean_lead_delta_vs_threshold_runs),
        format_option_f64(
            metrics
                .lead_time_summary
                .mean_persistent_boundary_minus_ewma_delta_runs,
        ),
        format_option_f64(
            metrics
                .lead_time_summary
                .mean_persistent_boundary_minus_threshold_delta_runs,
        ),
        psp.summary.pass_run_nuisance_proxy,
        metrics
            .summary
            .pass_run_dsfb_persistent_boundary_nuisance_rate,
        metrics
            .summary
            .pass_run_dsfb_persistent_violation_nuisance_rate,
        metrics.summary.pass_run_ewma_nuisance_rate,
        metrics.summary.pass_run_threshold_nuisance_rate,
        metrics.boundary_episode_summary.persistent_episode_count,
        format_option_f64(metrics.boundary_episode_summary.mean_persistent_episode_length),
        format_option_f64(
            metrics
                .boundary_episode_summary
                .persistent_non_escalating_episode_fraction,
        ),
    ));
    out.push_str(
        "The nuisance numbers above are pass-run proxies on SECOM labels, not fab-level false-alarm-rate certification metrics.\n\n",
    );
    out.push_str("## Persistent Structural Precursor (PSP)\n\n");
    out.push_str(&format!(
        "- DSFB Violation: instantaneous hard envelope exit\n- PSP: persistent multi-signal structural accumulation\n- Failure-run recall, PSP: {}/{}\n- Failure-run recall, threshold: {}/{}\n- Failure-run recall, EWMA: {}/{}\n- Mean lead time, PSP: {}\n- Median lead time, PSP: {}\n- Pass-run nuisance proxy, PSP: {:.4}\n- Mean lead delta vs threshold (runs): {}\n- Mean lead delta vs EWMA (runs): {}\n\n{}\n\n",
        psp.comparison_summary.psp.failure_run_recall,
        psp.comparison_summary.psp.failure_runs,
        psp.comparison_summary.threshold.failure_run_recall,
        psp.comparison_summary.threshold.failure_runs,
        psp.comparison_summary.ewma.failure_run_recall,
        psp.comparison_summary.ewma.failure_runs,
        format_option_f64(psp.comparison_summary.psp.mean_lead_time_runs),
        format_option_f64(psp.comparison_summary.psp.median_lead_time_runs),
        psp.comparison_summary.psp.pass_run_nuisance_proxy,
        format_option_f64(
            psp
                .comparison_summary
                .psp
                .mean_lead_delta_vs_threshold_runs,
        ),
        format_option_f64(
            psp
                .comparison_summary
                .psp
                .mean_lead_delta_vs_ewma_runs,
        ),
        psp.comparison_summary.conclusion,
    ));
    out.push_str(&format!(
        "## Density Summary\n\n- Density window: {} runs\n- Mean persistent boundary density, failure-labeled runs: {:.4}\n- Mean persistent boundary density, pass-labeled runs: {:.4}\n- Mean persistent violation density, failure-labeled runs: {:.4}\n- Mean persistent violation density, pass-labeled runs: {:.4}\n- Mean threshold density, failure-labeled runs: {:.4}\n- Mean threshold density, pass-labeled runs: {:.4}\n- Mean EWMA density, failure-labeled runs: {:.4}\n- Mean EWMA density, pass-labeled runs: {:.4}\n\n",
        metrics.density_summary.density_window,
        metrics.density_summary.mean_persistent_boundary_density_failure,
        metrics.density_summary.mean_persistent_boundary_density_pass,
        metrics.density_summary.mean_persistent_violation_density_failure,
        metrics.density_summary.mean_persistent_violation_density_pass,
        metrics.density_summary.mean_threshold_density_failure,
        metrics.density_summary.mean_threshold_density_pass,
        metrics.density_summary.mean_ewma_density_failure,
        metrics.density_summary.mean_ewma_density_pass,
    ));
    out.push_str(&format!("{}\n\n", psp_statement(psp)));

    if let Some(drsc) = &figures.drsc {
        out.push_str("## Deterministic Residual Stateflow Chart (DRSC)\n\n");
        out.push_str(&format!(
            "The crate now emits an operator-facing DRSC figure and aligned trace CSV for the top persistent-boundary feature in the current run (`{}`). The chart is cropped to the selected failure window ending at run {}. The top layer plots normalized residual, drift, and slew; the middle layer uses persistent deterministic states; and the bottom layer shows normalized envelope occupancy together with normalized EWMA occupancy. The figure annotates the first persistent boundary, the first persistent violation when present, and the failure-labeled run. This implementation does not have a trust scalar, so the lower layer is an admissibility overlay rather than a trust plot.\n\n- Figure: figures/{}\n- Trace CSV: {}\n\n",
            drsc.feature_name,
            drsc.failure_run_index,
            drsc.figure_file,
            drsc.trace_csv,
        ));
    }

    out.push_str("## Motif Calibration Summary\n\n");
    out.push_str("| Motif | Point hits | Run hits | Pre-failure window run hits | Precision proxy |\n|---|---:|---:|---:|---:|\n");
    for metric in &metrics.motif_metrics {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            metric.motif_name,
            metric.point_hits,
            metric.run_hits,
            metric.pre_failure_window_run_hits,
            format_option_f64(metric.pre_failure_window_precision_proxy),
        ));
    }

    out.push_str("\n## Heuristics Bank\n\n");
    out.push_str("| Motif | Provenance | Severity | Recommended action | Escalation policy | Known limitations |\n|---|---|---|---|---|---|\n");
    for entry in heuristics {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            entry.motif_name,
            entry.provenance_status,
            entry.severity,
            entry.recommended_action,
            entry.escalation_policy,
            entry.known_limitations,
        ));
    }

    out.push_str("\n## Figures\n\n");
    for file in &figures.files {
        out.push_str(&format!("- figures/{}\n", file));
    }

    out.push_str("\n## Artifact Inventory\n\n");
    out.push_str("| Path | Role |\n|---|---|\n");
    for entry in &artifact_inventory {
        out.push_str(&format!("| {} | {} |\n", entry.path, entry.role));
    }
    out.push('\n');

    out.push_str("\n## PHM 2018 Status\n\n");
    out.push_str(&format!(
        "- Official page: {}\n- Manual archive path: {}\n- Implemented now: {}\n- Blocker: {}\n\n",
        phm_status.official_page,
        phm_status.manual_placement_path.display(),
        phm_status.fully_implemented,
        phm_status.blocker,
    ));

    out.push_str("## Limitations of This Run\n\n");
    out.push_str("- SECOM is a real semiconductor dataset, but it is anonymized and instance-level; this run does not validate chamber-mechanism attribution or run-to-failure prognostics.\n");
    out.push_str("- The current comparator set remains narrow: a univariate residual-magnitude threshold plus a simple univariate EWMA residual-norm comparator. Stronger multivariate FDC baselines such as PCA/T-squared/SPE and lightweight ML anomaly detectors are not yet implemented in this crate.\n");
    out.push_str("- The lead-time and nuisance numbers are bounded proxy metrics derived from SECOM pass/fail labels and fixed lookback windows, not fab-qualified economic or false-alarm metrics.\n");
    out.push_str("- PHM 2018 support is not claimed beyond the manual-placement contract and archive probe until the real archive is present and verified.\n");
    out.push_str("- PDF generation depends on a local `pdflatex` installation.\n\n");

    out.push_str("## Explicit Non-Claims\n\n");
    out.push_str("- No universal superiority claim over SPC, EWMA, FDC, or ML baselines\n");
    out.push_str("- No standards-compliance or completed qualification claim\n");
    out.push_str("- No SEMI E125 compatibility claim\n");
    out.push_str("- No chamber-mechanism or physical root-cause attribution from SECOM alone\n");
    out.push_str("- No Kani verification claim for this crate\n");
    out.push_str("- No no_alloc, SIMD, rayon, or parallel-acceleration claim for this crate\n");
    out.push_str("- No claim that PHM 2018 ingestion is complete unless the archive is actually present and verified\n");
    out
}

fn latex_report(
    config: &PipelineConfig,
    metrics: &BenchmarkMetrics,
    psp: &PspEvaluation,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
    secom_layout: &SecomArchiveLayout,
) -> String {
    let artifact_inventory = artifact_inventory(figures);
    let figure_blocks = figures
        .files
        .iter()
        .map(|file| {
            let caption = if figures
                .drsc
                .as_ref()
                .map(|drsc| drsc.figure_file == *file)
                .unwrap_or(false)
            {
                format!(
                    "Deterministic Residual Stateflow Chart (DRSC) for the top persistent-boundary feature in the current run: synchronized residual/drift/slew structure, persistent deterministic state band, and admissibility/EWMA occupancy."
                )
            } else {
                format!("Generated artifact: {}", file)
            };
            format!(
                "\\begin{{figure}}[htbp]\n\\centering\n\\includegraphics[width=0.92\\linewidth]{{figures/{}}}\n\\caption{{{}}}\n\\end{{figure}}\n",
                latex_escape(file),
                latex_escape(&caption)
            )
        })
        .collect::<String>();

    let heuristic_rows = heuristics
        .iter()
        .map(|entry| {
            format!(
                "{} & {} & {} & {} \\\\\n",
                latex_escape(&entry.motif_name),
                latex_escape(&entry.provenance_status),
                latex_escape(&entry.severity),
                latex_escape(&entry.recommended_action)
            )
        })
        .collect::<String>();

    let motif_rows = metrics
        .motif_metrics
        .iter()
        .map(|metric| motif_row(metric))
        .collect::<String>();
    let artifact_rows = artifact_inventory
        .iter()
        .map(|entry| {
            format!(
                "{} & {} \\\\\n",
                latex_escape(&entry.path),
                latex_escape(&entry.role)
            )
        })
        .collect::<String>();

    format!(
        "\\documentclass[11pt]{{article}}
\\usepackage[margin=1in]{{geometry}}
\\usepackage{{booktabs}}
\\usepackage{{graphicx}}
\\usepackage{{longtable}}
\\usepackage{{hyperref}}
\\begin{{document}}
\\title{{DSFB Semiconductor Engineering Report}}
\\author{{Automatically generated by dsfb-semiconductor}}
\\date{{}}
\\maketitle

\\section*{{Dataset}}
This report documents a real-data DSFB run on the SECOM dataset from the UCI Machine Learning Repository. It is a Stage II public-benchmark artifact, not a deployment or standards-compliance report.

\\section*{{Archive layout note}}
The current distributed archive parses as {} numeric columns in \\texttt{{secom.data}}. The \\texttt{{secom.names}} metadata text claims {} attributes. The crate uses the numeric columns actually present in \\texttt{{secom.data}} and reads labels and timestamps separately from \\texttt{{secom\\_labels.data}}.

\\section*{{Preprocessing summary}}
\\begin{{tabular}}{{lr}}
\\toprule
Runs & {} \\\\
Features used by crate & {} \\\\
Passing runs & {} \\\\
Failure runs & {} \\\\
Dataset missing fraction & {:.4} \\\\
Healthy passing runs requested & {} \\\\
Healthy passing runs found & {} \\\\
\\bottomrule
\\end{{tabular}}

\\section*{{DSFB instantiation}}
The nominal reference is the healthy-window mean over the first {} passing runs. Residuals are defined as $x(k) - \\hat{{x}}(k)$. The admissibility envelope radius is {:.1}$\\sigma$ on the healthy residual distribution. The drift window is $W = {}$. The boundary rule in this implementation is $|r| > {:.1}\\rho$ with drift above {:.1}$\\sigma_{{\\mathrm{{healthy\\ drift}}}}$. Abrupt slew tags use {:.1}$\\sigma_{{\\mathrm{{healthy\\ slew}}}}$. Hysteresis-confirmed state changes require {} confirmations, persistent-state alarms require {} consecutive confirmed steps, and density metrics use a {}-run sliding window. The scalar comparator set contains a raw residual threshold and a univariate EWMA on residual norms with $\\alpha = {:.2}$ and a threshold at the healthy-window EWMA mean plus {:.1}$\\sigma$. In this crate, \\texttt{{DSFB Violation}} remains the instantaneous hard envelope exit state, while \\texttt{{PSP}} is a separate persistent multi-signal structural accumulation with $W = {}$, $K = {}$, $\\tau = {:.2}$, and fixed unit weights across the five scored PSP terms.

\\section*{{Quantitative summary}}
\\begin{{tabular}}{{lr}}
\\toprule
Analyzable features & {} \\\\
Threshold alarm points & {} \\\\
EWMA alarm points & {} \\\\
DSFB raw boundary points & {} \\\\
DSFB persistent boundary points & {} \\\\
DSFB raw violation points & {} \\\\
DSFB persistent violation points & {} \\\\
PSP alert points & {} \\\\
PSP alert runs & {} \\\\
Failure runs with preceding PSP signal & {} \\\\
Failure runs with preceding persistent DSFB boundary signal & {} \\\\
Failure runs with preceding persistent DSFB violation signal & {} \\\\
Failure runs with preceding EWMA signal & {} \\\\
Failure runs with preceding threshold signal & {} \\\\
\\bottomrule
\\end{{tabular}}

In the current implementation, the raw DSFB \\texttt{{Violation}} state and the raw threshold share the same envelope-exit condition $|r| > \\rho$, so raw violation/threshold coincidences are expected before persistence filtering. PSP is evaluated separately as a persistent multi-signal structural overlay and is not a redefinition of the violation state.

\\section*{{Lead-time and nuisance proxies}}
\\begin{{tabular}}{{lr}}
\\toprule
Mean PSP lead (runs) & {} \\\\
Median PSP lead (runs) & {} \\\\
Mean raw DSFB boundary lead (runs) & {} \\\\
Mean persistent DSFB boundary lead (runs) & {} \\\\
Mean raw DSFB violation lead (runs) & {} \\\\
Mean persistent DSFB violation lead (runs) & {} \\\\
Mean EWMA lead (runs) & {} \\\\
Mean threshold lead (runs) & {} \\\\
Mean PSP minus EWMA lead delta & {} \\\\
Mean PSP minus threshold lead delta & {} \\\\
Mean persistent DSFB boundary minus EWMA lead delta & {} \\\\
Mean persistent DSFB boundary minus threshold lead delta & {} \\\\
Pass-run nuisance proxy, PSP & {:.4} \\\\
Pass-run nuisance proxy, persistent DSFB boundary & {:.4} \\\\
Pass-run nuisance proxy, persistent DSFB violation & {:.4} \\\\
Pass-run nuisance proxy, EWMA & {:.4} \\\\
Pass-run nuisance proxy, threshold & {:.4} \\\\
Persistent boundary episodes & {} \\\\
Mean persistent boundary episode length & {} \\\\
Persistent non-escalating boundary episode fraction & {} \\\\
\\bottomrule
\\end{{tabular}}

These nuisance values are pass-run proxies on SECOM labels, not fab-qualified false-alarm-rate certification metrics.

\\section*{{Persistent Structural Precursor (PSP)}}
\\begin{{tabular}}{{lr}}
\\toprule
Failure-run recall, PSP & {}/{} \\\\
Failure-run recall, threshold & {}/{} \\\\
Failure-run recall, EWMA & {}/{} \\\\
Mean lead time, PSP & {} \\\\
Median lead time, PSP & {} \\\\
Pass-run nuisance proxy, PSP & {:.4} \\\\
Mean lead delta vs threshold & {} \\\\
Mean lead delta vs EWMA & {} \\\\
\\bottomrule
\\end{{tabular}}

{}

\\section*{{Density summary}}
\\begin{{tabular}}{{lr}}
\\toprule
Density window (runs) & {} \\\\
Mean persistent boundary density, failure-labeled runs & {:.4} \\\\
Mean persistent boundary density, pass-labeled runs & {:.4} \\\\
Mean persistent violation density, failure-labeled runs & {:.4} \\\\
Mean persistent violation density, pass-labeled runs & {:.4} \\\\
Mean threshold density, failure-labeled runs & {:.4} \\\\
Mean threshold density, pass-labeled runs & {:.4} \\\\
Mean EWMA density, failure-labeled runs & {:.4} \\\\
Mean EWMA density, pass-labeled runs & {:.4} \\\\
\\bottomrule
\\end{{tabular}}

{}

{}

\\section*{{Motif metrics}}
\\begin{{longtable}}{{p{{0.24\\linewidth}}rrrr}}
\\toprule
Motif & Point hits & Run hits & Pre-failure run hits & Precision proxy \\\\
\\midrule
{}
\\bottomrule
\\end{{longtable}}

\\section*{{Heuristics bank}}
\\begin{{longtable}}{{p{{0.18\\linewidth}}p{{0.15\\linewidth}}p{{0.12\\linewidth}}p{{0.42\\linewidth}}}}
\\toprule
Motif & Provenance & Severity & Recommended action \\\\
\\midrule
{}
\\bottomrule
\\end{{longtable}}

\\section*{{Artifact inventory}}
\\begin{{longtable}}{{p{{0.38\\linewidth}}p{{0.52\\linewidth}}}}
\\toprule
Path & Role \\\\
\\midrule
{}
\\bottomrule
\\end{{longtable}}

\\section*{{PHM 2018 status}}
The official PHM 2018 ion mill etch dataset path is \\url{{{}}}. The manual archive contract for this crate is \\texttt{{{}}}. Full PHM 2018 support is intentionally not claimed in this run. Blocker: {}.

\\section*{{Limitations of this run}}
\\begin{{itemize}}
\\item SECOM is a real semiconductor dataset, but it is anonymized and instance-level; this run does not validate chamber-mechanism attribution or run-to-failure prognostics.
\\item The current comparator set remains narrow: a univariate residual-magnitude threshold plus a simple univariate EWMA residual-norm comparator. Stronger multivariate FDC baselines such as PCA/Hotelling-$T^2$/SPE and lightweight ML anomaly detectors are not yet implemented in this crate.
\\item The lead-time and nuisance values are bounded proxy metrics derived from SECOM pass/fail labels and fixed lookback windows, not fab-qualified economic or false-alarm metrics.
\\item PHM 2018 support is not claimed beyond the manual-placement contract and archive probe unless the real archive is present and verified.
\\item PDF generation in this artifact path depends on a local \\texttt{{pdflatex}} installation.
\\end{{itemize}}

\\section*{{Explicit non-claims}}
\\begin{{itemize}}
\\item No universal superiority claim over SPC, EWMA, FDC, or ML baselines.
\\item No SEMI compliance or completed qualification claim.
\\item No SEMI E125 compatibility claim.
\\item No physical root-cause attribution from SECOM alone.
\\item No Kani verification claim for this crate.
\\item No \\texttt{{no\\_alloc}}, SIMD, rayon, or parallel-acceleration claim for this crate.
\\item No claim that PHM 2018 ingestion is complete unless the real archive is present and verified.
\\end{{itemize}}

{}
\\end{{document}}
",
        secom_layout.data_file_numeric_column_count,
        secom_layout
            .metadata_attribute_count_claim
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into()),
        metrics.summary.dataset_summary.run_count,
        metrics.summary.dataset_summary.feature_count,
        metrics.summary.dataset_summary.pass_count,
        metrics.summary.dataset_summary.fail_count,
        metrics.summary.dataset_summary.dataset_missing_fraction,
        metrics.summary.dataset_summary.healthy_pass_runs_requested,
        metrics.summary.dataset_summary.healthy_pass_runs_found,
        config.healthy_pass_runs,
        config.envelope_sigma,
        config.drift_window,
        config.boundary_fraction_of_rho,
        config.drift_sigma_multiplier,
        config.slew_sigma_multiplier,
        config.state_confirmation_steps,
        config.persistent_state_steps,
        config.density_window,
        config.ewma_alpha,
        config.ewma_sigma_multiplier,
        config.psp.window,
        config.psp.persistence_runs,
        config.psp.alert_tau,
        metrics.summary.analyzable_feature_count,
        metrics.summary.threshold_alarm_points,
        metrics.summary.ewma_alarm_points,
        metrics.summary.dsfb_raw_boundary_points,
        metrics.summary.dsfb_persistent_boundary_points,
        metrics.summary.dsfb_raw_violation_points,
        metrics.summary.dsfb_persistent_violation_points,
        psp.summary.alert_point_count,
        psp.summary.alert_run_count,
        psp.summary.failure_run_recall,
        metrics
            .summary
            .failure_runs_with_preceding_dsfb_persistent_boundary_signal,
        metrics
            .summary
            .failure_runs_with_preceding_dsfb_persistent_violation_signal,
        metrics.summary.failure_runs_with_preceding_ewma_signal,
        metrics.summary.failure_runs_with_preceding_threshold_signal,
        format_option_f64(psp.summary.mean_lead_time_runs),
        format_option_f64(psp.summary.median_lead_time_runs),
        format_option_f64(metrics.lead_time_summary.mean_raw_boundary_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_persistent_boundary_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_raw_violation_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_persistent_violation_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_ewma_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_threshold_lead_runs),
        format_option_f64(psp.summary.mean_lead_delta_vs_ewma_runs),
        format_option_f64(psp.summary.mean_lead_delta_vs_threshold_runs),
        format_option_f64(
            metrics
                .lead_time_summary
                .mean_persistent_boundary_minus_ewma_delta_runs,
        ),
        format_option_f64(
            metrics
                .lead_time_summary
                .mean_persistent_boundary_minus_threshold_delta_runs,
        ),
        psp.summary.pass_run_nuisance_proxy,
        metrics
            .summary
            .pass_run_dsfb_persistent_boundary_nuisance_rate,
        metrics
            .summary
            .pass_run_dsfb_persistent_violation_nuisance_rate,
        metrics.summary.pass_run_ewma_nuisance_rate,
        metrics.summary.pass_run_threshold_nuisance_rate,
        metrics.boundary_episode_summary.persistent_episode_count,
        format_option_f64(metrics.boundary_episode_summary.mean_persistent_episode_length),
        format_option_f64(
            metrics
                .boundary_episode_summary
                .persistent_non_escalating_episode_fraction,
        ),
        psp.comparison_summary.psp.failure_run_recall,
        psp.comparison_summary.psp.failure_runs,
        psp.comparison_summary.threshold.failure_run_recall,
        psp.comparison_summary.threshold.failure_runs,
        psp.comparison_summary.ewma.failure_run_recall,
        psp.comparison_summary.ewma.failure_runs,
        format_option_f64(psp.comparison_summary.psp.mean_lead_time_runs),
        format_option_f64(psp.comparison_summary.psp.median_lead_time_runs),
        psp.comparison_summary.psp.pass_run_nuisance_proxy,
        format_option_f64(
            psp
                .comparison_summary
                .psp
                .mean_lead_delta_vs_threshold_runs,
        ),
        format_option_f64(
            psp
                .comparison_summary
                .psp
                .mean_lead_delta_vs_ewma_runs,
        ),
        latex_escape(&psp.comparison_summary.conclusion),
        metrics.density_summary.density_window,
        metrics.density_summary.mean_persistent_boundary_density_failure,
        metrics.density_summary.mean_persistent_boundary_density_pass,
        metrics.density_summary.mean_persistent_violation_density_failure,
        metrics.density_summary.mean_persistent_violation_density_pass,
        metrics.density_summary.mean_threshold_density_failure,
        metrics.density_summary.mean_threshold_density_pass,
        metrics.density_summary.mean_ewma_density_failure,
        metrics.density_summary.mean_ewma_density_pass,
        latex_escape(&psp_statement(psp)),
        drsc_latex_section(figures),
        motif_rows,
        heuristic_rows,
        artifact_rows,
        phm_status.official_page,
        latex_escape(&phm_status.manual_placement_path.display().to_string()),
        latex_escape(phm_status.blocker),
        figure_blocks,
    )
}

fn artifact_inventory(figures: &FigureManifest) -> Vec<ArtifactInventoryEntry> {
    let mut entries = vec![
        ArtifactInventoryEntry {
            path: "dataset_summary.json".into(),
            role: "Dataset summary and healthy-window counts.".into(),
        },
        ArtifactInventoryEntry {
            path: "parameter_manifest.json".into(),
            role: "Saved deterministic DSFB and PSP parameter values.".into(),
        },
        ArtifactInventoryEntry {
            path: "run_configuration.json".into(),
            role: "CLI/data-root/output-root run configuration.".into(),
        },
        ArtifactInventoryEntry {
            path: "benchmark_metrics.json".into(),
            role: "Top-level benchmark metrics, summaries, and feature metrics.".into(),
        },
        ArtifactInventoryEntry {
            path: "baseline_comparison_summary.json".into(),
            role: "DSFB state-layer comparison against threshold and EWMA baselines.".into(),
        },
        ArtifactInventoryEntry {
            path: "psp_vs_baselines.json".into(),
            role: "Saved PSP recall, lead-time, nuisance, and delta comparison summary.".into(),
        },
        ArtifactInventoryEntry {
            path: "feature_metrics.csv".into(),
            role: "Per-feature DSFB and baseline point counts.".into(),
        },
        ArtifactInventoryEntry {
            path: "psp_metrics.csv".into(),
            role: "Per-feature, per-run PSP structural features, scores, and alerts.".into(),
        },
        ArtifactInventoryEntry {
            path: "per_failure_run_signals.csv".into(),
            role: "Per-failure DSFB state-layer earliest-signal and lead-time records.".into(),
        },
        ArtifactInventoryEntry {
            path: "per_failure_run_psp_signals.csv".into(),
            role: "Per-failure PSP earliest-signal and lead-time records.".into(),
        },
        ArtifactInventoryEntry {
            path: "lead_time_metrics.csv".into(),
            role: "Flattened lead-time comparison table.".into(),
        },
        ArtifactInventoryEntry {
            path: "density_metrics.csv".into(),
            role: "Sliding-window density metrics per run.".into(),
        },
        ArtifactInventoryEntry {
            path: "residuals.csv".into(),
            role: "Residual trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "drifts.csv".into(),
            role: "Drift trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "slews.csv".into(),
            role: "Slew trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "ewma_baseline.csv".into(),
            role: "EWMA baseline trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "grammar_states.csv".into(),
            role: "Raw and confirmed DSFB grammar states per feature and run.".into(),
        },
        ArtifactInventoryEntry {
            path: "heuristics_bank.json".into(),
            role: "Provenance-aware heuristic guidance derived from observed motifs.".into(),
        },
        ArtifactInventoryEntry {
            path: "secom_archive_layout.json".into(),
            role: "Archive-layout inspection and metadata mismatch note.".into(),
        },
        ArtifactInventoryEntry {
            path: "phm2018_support_status.json".into(),
            role: "PHM 2018 manual-placement and support-status record.".into(),
        },
        ArtifactInventoryEntry {
            path: "engineering_report.md".into(),
            role: "Markdown engineering report with metrics, figures, and artifact inventory.".into(),
        },
        ArtifactInventoryEntry {
            path: "engineering_report.tex".into(),
            role: "LaTeX source for the report and figure bundle PDF.".into(),
        },
        ArtifactInventoryEntry {
            path: "engineering_report.pdf".into(),
            role: "PDF report artifact catalog and figure bundle, when pdflatex is available.".into(),
        },
        ArtifactInventoryEntry {
            path: "artifact_manifest.json".into(),
            role: "Machine-readable manifest of output artifact paths.".into(),
        },
        ArtifactInventoryEntry {
            path: "run_bundle.zip".into(),
            role: "ZIP archive containing the complete run directory.".into(),
        },
    ];

    if figures.drsc.is_some() {
        entries.push(ArtifactInventoryEntry {
            path: "drsc_top_feature.csv".into(),
            role: "Aligned DRSC trace export for the selected feature window.".into(),
        });
    }

    for file in &figures.files {
        entries.push(ArtifactInventoryEntry {
            path: format!("figures/{file}"),
            role: "Notebook-parity PNG figure emitted directly by the crate.".into(),
        });
    }

    entries
}

fn drsc_latex_section(figures: &FigureManifest) -> String {
    if let Some(drsc) = &figures.drsc {
        format!(
            "\\section*{{Deterministic Residual Stateflow Chart (DRSC)}}\nThe crate emits a deterministic operator-facing DRSC artifact for the top persistent-boundary feature in the current run (\\texttt{{{}}}). The chart is cropped to the selected failure window ending at run {}. The upper layer plots normalized residual, drift, and slew; the middle layer is the persistent deterministic state band; and the lower layer shows normalized admissibility-envelope occupancy together with normalized EWMA occupancy. The figure annotates the first persistent boundary, the first persistent violation when present, and the failure-labeled run. This version does not implement a trust scalar, so the lower layer is an admissibility overlay rather than a trust plot. The aligned trace CSV is \\texttt{{{}}}.\n\n",
            latex_escape(&drsc.feature_name),
            drsc.failure_run_index,
            latex_escape(&drsc.trace_csv),
        )
    } else {
        String::new()
    }
}

fn psp_statement(psp: &PspEvaluation) -> String {
    psp.comparison_summary.conclusion.clone()
}

fn motif_row(metric: &MotifMetric) -> String {
    format!(
        "{} & {} & {} & {} & {} \\\\\n",
        latex_escape(&metric.motif_name),
        metric.point_hits,
        metric.run_hits,
        metric.pre_failure_window_run_hits,
        latex_escape(&format_option_f64(
            metric.pre_failure_window_precision_proxy
        )),
    )
}

fn compile_pdf(tex_path: &Path, output_dir: &Path) -> (Option<PathBuf>, Option<String>) {
    let filename = tex_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "engineering_report.tex".into());
    let pdf_path = output_dir.join(filename.replace(".tex", ".pdf"));
    let mut combined_output = String::new();
    let mut any_success = false;

    for _ in 0..2 {
        match Command::new("pdflatex")
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg("-output-directory")
            .arg(".")
            .arg(&filename)
            .current_dir(output_dir)
            .output()
        {
            Ok(output) => {
                combined_output.push_str(&String::from_utf8_lossy(&output.stderr));
                combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
                if output.status.success() {
                    any_success = true;
                }
            }
            Err(err) => {
                if pdf_path.exists() {
                    return (Some(pdf_path), Some(err.to_string()));
                }
                return (None, Some(err.to_string()));
            }
        }
    }

    if any_success && pdf_path.exists() {
        return (Some(pdf_path), None);
    }
    if pdf_path.exists() {
        return (
            Some(pdf_path),
            (!combined_output.trim().is_empty()).then_some(combined_output),
        );
    }

    (None, (!combined_output.trim().is_empty()).then_some(combined_output))
}

fn format_option_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "n/a".into())
}

fn latex_escape(input: &str) -> String {
    input
        .replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}
