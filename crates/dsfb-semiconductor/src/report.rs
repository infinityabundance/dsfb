use crate::config::PipelineConfig;
use crate::dataset::phm2018::Phm2018SupportStatus;
use crate::error::Result;
use crate::heuristics::HeuristicEntry;
use crate::metrics::BenchmarkMetrics;
use crate::plots::FigureManifest;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
) -> Result<ReportArtifacts> {
    let markdown_path = run_dir.join("engineering_report.md");
    let tex_path = run_dir.join("engineering_report.tex");
    fs::write(
        &markdown_path,
        markdown_report(config, metrics, figures, heuristics, phm_status),
    )?;
    fs::write(
        &tex_path,
        latex_report(config, metrics, figures, heuristics, phm_status),
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
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
) -> String {
    let mut out = String::new();
    out.push_str("# DSFB Semiconductor Engineering Report\n\n");
    out.push_str("## Dataset\n\n");
    out.push_str("- Dataset: SECOM (UCI Machine Learning Repository)\n");
    out.push_str(
        "- Evidence class: Stage II public-benchmark evidence on real semiconductor data\n",
    );
    out.push_str("- Non-claim: this run does not establish SEMI compliance, production readiness, or physical root-cause attribution\n\n");
    out.push_str("## Preprocessing Summary\n\n");
    out.push_str(&format!(
        "- Runs: {}\n- Features in archive: {}\n- Passing runs: {}\n- Failure runs: {}\n- Dataset missing fraction: {:.4}\n- Healthy passing runs requested/found: {}/{}\n\n",
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
        "- Nominal reference: healthy-window mean over first {} passing runs\n- Residual: x(k) - x_hat\n- Envelope radius rho: {:.1} * healthy-window residual std\n- Drift window W: {}\n- Boundary condition: |r| > {:.1} * rho and drift > {:.1} * healthy drift std\n- Slew threshold: {:.1} * healthy slew std\n- Recurrent-boundary grazing: {} hits in a {}-run window\n- Baseline comparators: univariate 3-sigma residual threshold, plus univariate EWMA on residual norms with alpha = {:.2} and threshold mean + {:.1} * healthy EWMA std\n\n",
        config.healthy_pass_runs,
        config.envelope_sigma,
        config.drift_window,
        config.boundary_fraction_of_rho,
        config.drift_sigma_multiplier,
        config.slew_sigma_multiplier,
        config.grazing_min_hits,
        config.grazing_window,
        config.ewma_alpha,
        config.ewma_sigma_multiplier,
    ));
    out.push_str("Grammar logic in this crate is intentionally simple and deterministic: admissible, boundary, and violation states are derived from the envelope radius, outward drift, abrupt slew, and recurrent boundary grazing rules encoded in the saved parameter manifest.\n\n");
    out.push_str("## Quantitative Summary\n\n");
    out.push_str(&format!(
        "- Analyzable features: {}\n- Threshold alarm points: {}\n- EWMA alarm points: {}\n- DSFB boundary points: {}\n- DSFB violation points: {}\n- Failure runs with preceding DSFB signal ({}-run lookback): {}\n- Failure runs with preceding EWMA signal ({}-run lookback): {}\n- Failure runs with preceding threshold signal ({}-run lookback): {}\n\n",
        metrics.summary.analyzable_feature_count,
        metrics.summary.threshold_alarm_points,
        metrics.summary.ewma_alarm_points,
        metrics.summary.dsfb_boundary_points,
        metrics.summary.dsfb_violation_points,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_dsfb_signal,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_ewma_signal,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_threshold_signal,
    ));
    out.push_str("## Heuristics Bank\n\n");
    out.push_str("| Motif | Provenance | Interpretation |\n|---|---|---|\n");
    for entry in heuristics {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            entry.motif_name, entry.provenance_status, entry.interpretation
        ));
    }
    out.push_str("\n## Figures\n\n");
    for file in &figures.files {
        out.push_str(&format!("- figures/{}\n", file));
    }
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
    out.push_str("- PHM 2018 support is not claimed beyond the manual-placement contract and archive probe until the real archive is present and verified.\n");
    out.push_str("- PDF generation depends on a local `pdflatex` installation.\n\n");
    out.push_str("## Explicit Non-Claims\n\n");
    out.push_str("- No universal superiority claim over SPC, EWMA, FDC, or ML baselines\n");
    out.push_str("- No standards-compliance or completed qualification claim\n");
    out.push_str("- No chamber-mechanism or physical root-cause attribution from SECOM alone\n");
    out.push_str("- No claim that PHM 2018 ingestion is complete unless the archive is actually present and verified\n");
    out
}

fn latex_report(
    config: &PipelineConfig,
    metrics: &BenchmarkMetrics,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
) -> String {
    let figure_blocks = figures
        .files
        .iter()
        .map(|file| {
            format!(
                "\\begin{{figure}}[htbp]\n\\centering\n\\includegraphics[width=0.92\\linewidth]{{figures/{}}}\n\\caption{{Generated artifact: {}}}\n\\end{{figure}}\n",
                latex_escape(file),
                latex_escape(file)
            )
        })
        .collect::<String>();

    let heuristic_rows = heuristics
        .iter()
        .map(|entry| {
            format!(
                "{} & {} & {} \\\\\n",
                latex_escape(&entry.motif_name),
                latex_escape(&entry.provenance_status),
                latex_escape(&entry.interpretation)
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

\\section*{{Preprocessing summary}}
\\begin{{tabular}}{{lr}}
\\toprule
Runs & {} \\\\
Features in archive & {} \\\\
Passing runs & {} \\\\
Failure runs & {} \\\\
Dataset missing fraction & {:.4} \\\\
Healthy passing runs requested & {} \\\\
Healthy passing runs found & {} \\\\
\\bottomrule
\\end{{tabular}}

Missing values are preserved as explicit missing entries during parsing and then imputed feature-wise with the healthy-window nominal mean before residual construction. This choice is deterministic and reproducible, but it does not imply that missingness is operationally benign.

\\section*{{DSFB instantiation}}
The nominal reference is the healthy-window mean over the first {} passing runs. Residuals are defined as $x(k) - \\hat{{x}}(k)$. The admissibility envelope radius is {:.1}$\\sigma$ on the healthy residual distribution. The drift window is $W = {}$. The boundary rule in this implementation is $|r| > {:.1}\\rho$ with drift above {:.1}$\\sigma_{{\\mathrm{{healthy\\ drift}}}}$. Abrupt slew tags use {:.1}$\\sigma_{{\\mathrm{{healthy\\ slew}}}}$. The scalar comparator set contains a raw residual threshold and a univariate EWMA on residual norms with $\\alpha = {:.2}$ and a threshold at the healthy-window EWMA mean plus {:.1}$\\sigma$.

Grammar logic in this crate is intentionally simple and deterministic: admissible, boundary, and violation states are derived from the saved envelope, drift, slew, and recurrent-boundary rules.

\\section*{{Quantitative summary}}
\\begin{{tabular}}{{lr}}
\\toprule
Analyzable features & {} \\\\
Threshold alarm points & {} \\\\
EWMA alarm points & {} \\\\
DSFB boundary points & {} \\\\
DSFB violation points & {} \\\\
Failure runs with preceding DSFB signal & {} \\\\
Failure runs with preceding EWMA signal & {} \\\\
Failure runs with preceding threshold signal & {} \\\\
\\bottomrule
\\end{{tabular}}

\\section*{{Heuristics bank}}
\\begin{{longtable}}{{p{{0.22\\linewidth}}p{{0.18\\linewidth}}p{{0.5\\linewidth}}}}
\\toprule
Motif & Provenance & Interpretation \\\\
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
\\item PHM 2018 support is not claimed beyond the manual-placement contract and archive probe unless the real archive is present and verified.
\\item PDF generation in this artifact path depends on a local \\texttt{{pdflatex}} installation.
\\end{{itemize}}

\\section*{{Explicit non-claims}}
\\begin{{itemize}}
\\item No universal superiority claim over SPC, EWMA, FDC, or ML baselines.
\\item No SEMI compliance or completed qualification claim.
\\item No physical root-cause attribution from SECOM alone.
\\item No claim that PHM 2018 ingestion is complete unless the real archive is present and verified.
\\end{{itemize}}

{}
\\end{{document}}
",
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
        config.ewma_alpha,
        config.ewma_sigma_multiplier,
        metrics.summary.analyzable_feature_count,
        metrics.summary.threshold_alarm_points,
        metrics.summary.ewma_alarm_points,
        metrics.summary.dsfb_boundary_points,
        metrics.summary.dsfb_violation_points,
        metrics.summary.failure_runs_with_preceding_dsfb_signal,
        metrics.summary.failure_runs_with_preceding_ewma_signal,
        metrics.summary.failure_runs_with_preceding_threshold_signal,
        heuristic_rows,
        phm_status.official_page,
        latex_escape(&phm_status.manual_placement_path.display().to_string()),
        latex_escape(phm_status.blocker),
        figure_blocks,
    )
}

fn compile_pdf(tex_path: &Path, output_dir: &Path) -> (Option<PathBuf>, Option<String>) {
    let filename = tex_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "engineering_report.tex".into());
    let status = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg("-output-directory")
        .arg(output_dir)
        .arg(&filename)
        .current_dir(output_dir)
        .output();

    match status {
        Ok(output) if output.status.success() => {
            let pdf_path = output_dir.join(filename.replace(".tex", ".pdf"));
            (Some(pdf_path), None)
        }
        Ok(output) => (
            None,
            Some(
                String::from_utf8_lossy(&output.stderr).to_string()
                    + &String::from_utf8_lossy(&output.stdout),
            ),
        ),
        Err(err) => (None, Some(err.to_string())),
    }
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
