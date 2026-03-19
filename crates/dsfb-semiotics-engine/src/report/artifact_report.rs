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
    lines.push("- Sign projection used in Figure 03: deterministic aggregate coordinates `[||r(t)||, dot(r(t), d(t))/||r(t)||, ||s(t)||]` when `||r(t)|| > 0`, with zero radial drift at exact zero residual norm.".to_string());
    lines.push("- Syntax metrics include outward and inward drift fractions from margin evolution, directional persistence, sign consistency, channel coherence, aggregate monotonicity, curvature energy, slew spike count, and boundary grazing episode count.".to_string());
    lines.push(
        "- Grammar: admissibility checked pointwise against `||r(t)|| <= rho(t)`.".to_string(),
    );
    lines.push("- Semantics: constrained retrieval over a typed heuristic bank with scope conditions, admissibility requirements, regime tags, provenance notes, and compatibility rules.".to_string());
    lines.push("- Detectability bound: `t* - t0 <= Delta0 / (alpha - kappa)` when configured assumptions hold.".to_string());
    lines.push(String::new());
    lines.push("## Reproducibility Summary".to_string());
    lines.push(String::new());
    lines.push(format!(
        "- Scenario count checked: {}",
        bundle.reproducibility_summary.scenario_count
    ));
    lines.push(format!(
        "- Identical materializations: {}",
        bundle.reproducibility_summary.identical_count
    ));
    lines.push(format!(
        "- All identical: `{}`",
        bundle.reproducibility_summary.all_identical
    ));
    lines.push(format!(
        "- Note: {}",
        bundle.reproducibility_summary.note
    ));
    lines.push(String::new());
    for check in &bundle.reproducibility_checks {
        lines.push(format!(
            "- `{}`: identical=`{}`, hash1=`{}`, hash2=`{}`",
            check.scenario_id, check.identical, check.first_hash, check.second_hash
        ));
    }
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
    lines.push("- Synthetic scenarios in this run are deterministic constructions intended to illustrate theorem-aligned behavior and auditable pipeline structure. CSV-ingested runs reuse the same pipeline without adding external validation claims.".to_string());
    lines.push("- CSV ingestion mode, when used, applies the same deterministic layers to user-supplied trajectories but does not add validation claims beyond the supplied inputs and configured envelope.".to_string());
    lines.push("- Envelope exits demonstrate detectable departure from the configured admissibility grammar, not unique identification of latent physical cause.".to_string());
    lines.push("- Heuristic semantic matches are constrained typed-bank retrieval outcomes only; they are allowed to remain compatible shortlists, ambiguous, or unknown.".to_string());
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
    lines.push("- PDF companion content: rendered markdown report, embedded figure artifacts, full artifact inventory, and appended text-based CSV/JSON/manifest/report sources.".to_string());
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
            "- Syntax metrics: outward={:.3}, inward={:.3}, monotonicity={:.3}, persistence={:.3}, curvature={:.3}, grazing_episodes={}",
            scenario.syntax.outward_drift_fraction,
            scenario.syntax.inward_drift_fraction,
            scenario.syntax.aggregate_monotonicity,
            scenario.syntax.directional_persistence,
            scenario.syntax.curvature_energy,
            scenario.syntax.boundary_grazing_episode_count
        ),
        format!(
            "- Semantic disposition: `{:?}`",
            scenario.semantics.disposition
        ),
        format!(
            "- Semantic compatibility note: {}",
            scenario.semantics.compatibility_note
        ),
        format!("- Limitation note: {}", scenario.record.limitations),
        String::new(),
    ]
}
