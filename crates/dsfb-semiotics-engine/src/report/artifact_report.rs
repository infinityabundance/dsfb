use crate::engine::types::{
    EngineOutputBundle, FigureArtifact, GrammarState, ReportManifest, ScenarioOutput,
};

pub fn build_markdown_report(
    bundle: &EngineOutputBundle,
    figures: &[FigureArtifact],
    manifest: &ReportManifest,
) -> String {
    let mut lines = Vec::new();
    lines.push("# DSFB Structural Semiotics Engine Artifact Report".to_string());
    lines.push(String::new());
    lines.push(format!("Timestamp: `{}`", bundle.run_metadata.timestamp));
    lines.push(format!(
        "Crate: `{}` v`{}`",
        bundle.run_metadata.crate_name, bundle.run_metadata.crate_version
    ));
    if let Some(git_commit) = &bundle.run_metadata.git_commit {
        lines.push(format!("Git commit: `{git_commit}`"));
    }
    if let Some(rust_version) = &bundle.run_metadata.rust_version {
        lines.push(format!("Rust: `{rust_version}`"));
    }
    lines.push(String::new());
    lines.push("## Definitions Used".to_string());
    lines.push(String::new());
    lines.push("- Residual: `r(t) = y(t) - y_hat(t)`".to_string());
    lines.push("- Drift: `d(t) = dr/dt` via deterministic finite differences.".to_string());
    lines.push("- Slew: `s(t) = d^2r/dt^2` via deterministic second differences.".to_string());
    lines.push("- Sign tuple: `sigma(t) = (r(t), d(t), s(t))`.".to_string());
    lines.push(
        "- Grammar: admissibility checked pointwise against `||r(t)|| <= rho(t)`.".to_string(),
    );
    lines.push("- Detectability bound: `t* - t0 <= Delta0 / (alpha - kappa)` when configured assumptions hold.".to_string());
    lines.push(String::new());
    lines.push("## Scenario Summary".to_string());
    lines.push(String::new());
    for scenario in &bundle.scenario_outputs {
        lines.extend(render_scenario_summary(scenario));
    }
    lines.push("## Figure Captions".to_string());
    lines.push(String::new());
    for figure in figures {
        lines.push(format!("- `{}`: {}", figure.figure_id, figure.caption));
    }
    lines.push(String::new());
    lines.push("## Limitations and Non-Claims".to_string());
    lines.push(String::new());
    lines.push("- All demonstrations in this run are synthetic, deterministic constructions intended to illustrate theorem-aligned behavior and auditable pipeline structure.".to_string());
    lines.push("- Envelope exits demonstrate detectable departure from the configured admissibility grammar, not unique identification of latent physical cause.".to_string());
    lines.push("- Heuristic semantic matches are constrained motif retrieval outcomes only; they are allowed to remain ambiguous or unknown.".to_string());
    lines.push("- No certification claim is made. The artifact is aligned with deterministic and auditable engineering evaluation logic only.".to_string());
    lines.push(String::new());
    lines.push("## Manifest Summary".to_string());
    lines.push(String::new());
    lines.push(format!("- Run directory: `{}`", manifest.run_dir));
    lines.push(format!("- Figure files: {}", manifest.figure_paths.len()));
    lines.push(format!("- CSV files: {}", manifest.csv_paths.len()));
    lines.push(format!("- JSON files: {}", manifest.json_paths.len()));
    lines.push(format!("- PDF report: `{}`", manifest.report_pdf));
    lines.push(format!("- Zip archive: `{}`", manifest.zip_archive));
    lines.push(String::new());
    lines.join("\n")
}

fn render_scenario_summary(scenario: &ScenarioOutput) -> Vec<String> {
    let violation_count = scenario
        .grammar
        .iter()
        .filter(|status| matches!(status.state, GrammarState::Violation))
        .count();
    let first_exit = scenario
        .detectability
        .observed_crossing_time
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "none in sampled horizon".to_string());

    vec![
        format!("### {}", scenario.record.title),
        String::new(),
        format!("- Scenario ID: `{}`", scenario.record.id),
        format!("- Purpose: {}", scenario.record.purpose),
        format!("- Alignment: {}", scenario.record.theorem_alignment),
        format!("- Claim class: {}", scenario.record.claim_class),
        format!("- Violations observed: {}", violation_count),
        format!("- First exit time: {}", first_exit),
        format!(
            "- Semantic disposition: `{:?}`",
            scenario.semantics.disposition
        ),
        format!("- Limitation note: {}", scenario.record.limitations),
        String::new(),
    ]
}
