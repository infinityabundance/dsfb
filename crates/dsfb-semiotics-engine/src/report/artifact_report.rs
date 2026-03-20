use crate::engine::types::{
    EngineOutputBundle, FigureArtifact, GrammarState, ReportManifest, ScenarioOutput,
};
use crate::math::metrics::format_metric;

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
    lines.push(format!("Input mode: `{}`", bundle.run_metadata.input_mode));
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
    lines.push("- Sign projection used in Figure 03: deterministic projected sign coordinates `[||r(t)||, dot(r(t), d(t))/||r(t)||, ||s(t)||]`, reported as residual norm, signed radial drift, and slew norm, with zero radial drift reported at exact zero residual norm.".to_string());
    lines.push("- Syntax metrics include outward and inward drift fractions from residual-norm and margin evolution, radial-sign dominance, radial-sign persistence, drift-channel sign alignment, residual-norm path monotonicity, residual-norm trend alignment, mean squared slew norm, late slew-growth score, slew spike count and strength, boundary grazing episodes, boundary recovery count, and grouped aggregate breach fraction when coordinated structure is configured. Labels such as `weakly-structured-baseline-like` and `mixed-structured` remain conservative summaries rather than health judgments.".to_string());
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
    lines.push(format!("- Note: {}", bundle.reproducibility_summary.note));
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
    lines.push("- Heuristic semantic matches are constrained typed-bank retrieval outcomes only; they are allowed to remain explicit compatible sets, ambiguous, or unknown.".to_string());
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
        .map(format_metric)
        .unwrap_or_else(|| "none in sampled horizon".to_string());

    vec![
        format!("### {}", scenario.record.title),
        String::new(),
        format!("- Scenario ID: `{}`", scenario.record.id),
        format!("- Data origin: {}", scenario.record.data_origin),
        format!("- Purpose: {}", scenario.record.purpose),
        format!("- Alignment: {}", scenario.record.theorem_alignment),
        format!("- Claim class: {}", scenario.record.claim_class),
        format!("- Violations observed: {}", violation_count),
        format!("- First exit time: {}", first_exit),
        format!(
            "- Syntax metrics: outward={}, inward={}, residual_norm_path_monotonicity={}, residual_norm_trend_alignment={}, radial_sign_persistence={}, radial_sign_dominance={}, drift_channel_sign_alignment={}, mean_squared_slew_norm={}, late_slew_growth_score={}, slew_spikes={}, spike_strength={}, grazing_episodes={}, boundary_recoveries={}, coordinated_group_breach_fraction={}",
            format_metric(scenario.syntax.outward_drift_fraction),
            format_metric(scenario.syntax.inward_drift_fraction),
            format_metric(scenario.syntax.residual_norm_path_monotonicity),
            format_metric(scenario.syntax.residual_norm_trend_alignment),
            format_metric(scenario.syntax.radial_sign_persistence),
            format_metric(scenario.syntax.radial_sign_dominance),
            format_metric(scenario.syntax.drift_channel_sign_alignment),
            format_metric(scenario.syntax.mean_squared_slew_norm),
            format_metric(scenario.syntax.late_slew_growth_score),
            scenario.syntax.slew_spike_count,
            format_metric(scenario.syntax.slew_spike_strength),
            scenario.syntax.boundary_grazing_episode_count,
            scenario.syntax.boundary_recovery_count,
            format_metric(scenario.syntax.coordinated_group_breach_fraction),
        ),
        format!(
            "- Syntax label: `{}`",
            scenario.syntax.trajectory_label
        ),
        format!("- Syntax note: {}", syntax_note(&scenario)),
        format!(
            "- Semantic disposition: `{:?}`",
            scenario.semantics.disposition
        ),
        format!(
            "- Selected heuristics: `{}`",
            if scenario.semantics.selected_heuristic_ids.is_empty() {
                "none".to_string()
            } else {
                scenario.semantics.selected_heuristic_ids.join("`, `")
            }
        ),
        format!(
            "- Semantic resolution basis: {}",
            scenario.semantics.resolution_basis
        ),
        format!(
            "- Semantic unknown reason class: {}",
            scenario
                .semantics
                .unknown_reason_class
                .clone()
                .unwrap_or_else(|| "n/a".to_string())
        ),
        format!(
            "- Semantic compatibility note: {}",
            scenario.semantics.compatibility_note
        ),
        format!("- Semantic note: {}", scenario.semantics.note),
        format!("- Limitation note: {}", scenario.record.limitations),
        String::new(),
    ]
    .into_iter()
    .chain(
        scenario
            .semantics
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "- Candidate `{}` (`{}`): score={}, regimes={}, regime_check={}, admissibility={}, scope={}, applicability={}, provenance={}, rationale={}",
                    candidate.entry.heuristic_id,
                    candidate.entry.motif_label,
                    format_metric(candidate.score),
                    if candidate.matched_regimes.is_empty() {
                        "none".to_string()
                    } else {
                        candidate.matched_regimes.join("|")
                    },
                    candidate.regime_explanation,
                    candidate.admissibility_explanation,
                    candidate.scope_explanation,
                    candidate.entry.applicability_note,
                    candidate.entry.provenance.note,
                    candidate.rationale
                )
            }),
    )
    .chain(std::iter::once(String::new()))
    .collect()
}

fn syntax_note(scenario: &ScenarioOutput) -> String {
    match scenario.syntax.trajectory_label.as_str() {
        "weakly-structured-baseline-like" => {
            "This syntax label is a low-commitment baseline-compatible summary relative to the configured prediction and envelope only. It is not a health or certification label.".to_string()
        }
        "mixed-structured" => {
            "This syntax label is conservative non-commitment: the exported deterministic metrics did not support a narrower syntax summary under the current rule set.".to_string()
        }
        "coordinated-outward-rise" => {
            "Coordination is surfaced at the syntax layer because grouped residual structure and aggregate breach evidence were explicitly configured and observed.".to_string()
        }
        _ => "This syntax label is a deterministic summary of the exported syntax metrics, not an inferred latent mechanism.".to_string(),
    }
}
