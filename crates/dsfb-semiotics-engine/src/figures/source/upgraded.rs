//! Run-specific figure source assembly for the upgraded paper/demo figures.

use super::*;

pub(super) fn prepare_figure_09(bundle: &EngineOutputBundle) -> FigureSourceTable {
    if is_primary_bearings_bundle(bundle) {
        if let Ok(table) = prepare_figure_09_bearings(bundle) {
            return table;
        }
    }
    if is_primary_milling_bundle(bundle) {
        if let Ok(table) = prepare_figure_09_milling(bundle) {
            return table;
        }
    }
    if is_synthetic_bundle(bundle) {
        if let Ok(table) = prepare_figure_09_synthetic(bundle) {
            return table;
        }
    }
    let rows = detectability_source_rows(bundle);
    let use_multi_case_comparison = rows
        .iter()
        .filter(|row| row.predicted_upper_bound.is_some())
        .count()
        > 1;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_09_detectability_bound_comparison",
        if use_multi_case_comparison {
            "Run-Specific Detectability Timing Summary"
        } else {
            "Run-Specific Detectability Context"
        },
        if use_multi_case_comparison {
            &["detectability_bound", "detectability_gap"]
        } else {
            &["detectability_context", "detectability_window_ratio"]
        },
        &[],
    );
    if use_multi_case_comparison {
        for (index, row) in rows.into_iter().enumerate() {
            if let Some(predicted) = row.predicted_upper_bound {
                push_bar_row(
                    &mut table,
                    "detectability_bound",
                    "Predicted vs Observed Detectability Times",
                    "scenario index",
                    "time to first exit",
                    "predicted_upper_bound",
                    "predicted upper bound",
                    "blue",
                    &row.scenario_id,
                    index * 2,
                    index as f64 + 0.18,
                    index as f64 + 0.40,
                    predicted,
                    &compact_scenario_tick_label(&row.scenario_id, index + 1),
                    "Predicted detectability upper bound for one configured theorem-aligned case.",
                );
            }
            if let Some(observed) = row.observed_crossing_time {
                push_bar_row(
                    &mut table,
                    "detectability_bound",
                    "Predicted vs Observed Detectability Times",
                    "scenario index",
                    "time to first exit",
                    "observed_crossing_time",
                    "observed crossing time",
                    "red",
                    &row.scenario_id,
                    index * 2 + 1,
                    index as f64 + 0.44,
                    index as f64 + 0.66,
                    observed,
                    &compact_scenario_tick_label(&row.scenario_id, index + 1),
                    "Observed detectability crossing time for one configured theorem-aligned case.",
                );
            }
            if let (Some(predicted), Some(observed)) =
                (row.predicted_upper_bound, row.observed_crossing_time)
            {
                push_bar_row(
                    &mut table,
                    "detectability_gap",
                    "Observed Minus Predicted Detectability Gap",
                    "scenario index",
                    "absolute timing gap",
                    "detectability_gap",
                    "absolute timing gap",
                    if observed <= predicted { "green" } else { "red" },
                    &row.scenario_id,
                    index,
                    index as f64 + 0.18,
                    index as f64 + 0.64,
                    (observed - predicted).abs(),
                    &compact_scenario_tick_label(&row.scenario_id, index + 1),
                    "Absolute gap between the observed detectability time and the configured predicted upper bound.",
                );
            }
        }
        return table;
    }

    if let Some(scenario) = detectability_summary_scenario(bundle) {
        let scenario_id = scenario.record.id.clone();
        let y_upper = scenario
            .residual
            .samples
            .iter()
            .map(|sample| sample.norm)
            .chain(scenario.envelope.samples.iter().map(|sample| sample.radius))
            .fold(0.0_f64, f64::max)
            .max(0.1);
        push_scalar_series(
            &mut table,
            "detectability_context",
            "Residual Norm vs Envelope Radius",
            "time",
            "magnitude",
            "residual_norm",
            "residual norm",
            "line",
            "red",
            &scenario_id,
            scenario
                .residual
                .samples
                .iter()
                .enumerate()
                .map(|(index, sample)| (index, sample.time, sample.norm)),
            "Residual norm trajectory for the selected run-specific detectability context view.",
        );
        push_scalar_series(
            &mut table,
            "detectability_context",
            "Residual Norm vs Envelope Radius",
            "time",
            "magnitude",
            "envelope_radius",
            "envelope radius",
            "line",
            "slate",
            &scenario_id,
            scenario
                .envelope
                .samples
                .iter()
                .enumerate()
                .map(|(index, sample)| (index, sample.time, sample.radius)),
            "Envelope radius trajectory for the selected run-specific detectability context view.",
        );
        if let Some(time) = scenario.detectability.predicted_upper_bound {
            let point_order = table.rows.len();
            push_segment_row(
                &mut table,
                "detectability_context",
                "Residual Norm vs Envelope Radius",
                "time",
                "magnitude",
                "predicted_upper_bound",
                "predicted bound",
                "blue",
                point_order,
                time,
                0.0,
                time,
                y_upper,
                &scenario_id,
                "Vertical marker for the configured detectability upper-bound time.",
            );
        }
        if let Some(time) = first_non_admissible_time(scenario) {
            let point_order = table.rows.len();
            push_segment_row(
                &mut table,
                "detectability_context",
                "Residual Norm vs Envelope Radius",
                "time",
                "magnitude",
                "first_boundary_time",
                "first boundary",
                "gold",
                point_order,
                time,
                0.0,
                time,
                y_upper,
                &scenario_id,
                "Vertical marker for the first non-admissible grammar interaction time.",
            );
        }
        if let Some(time) = first_violation_time(scenario) {
            let point_order = table.rows.len();
            push_segment_row(
                &mut table,
                "detectability_context",
                "Residual Norm vs Envelope Radius",
                "time",
                "magnitude",
                "first_violation_time",
                "first violation",
                "red",
                point_order,
                time,
                0.0,
                time,
                y_upper,
                &scenario_id,
                "Vertical marker for the first grammar violation time.",
            );
        }

        for (window_index, (ratio, label)) in detectability_window_max_ratio(scenario, 6)
            .into_iter()
            .enumerate()
        {
            push_bar_row(
                &mut table,
                "detectability_window_ratio",
                "Window Maximum Residual/Envelope Ratio",
                "window",
                "max residual/envelope ratio",
                "window_max_ratio",
                "window max ratio",
                if ratio > 1.0 { "red" } else { "teal" },
                &scenario_id,
                window_index,
                window_index as f64 + 0.18,
                window_index as f64 + 0.72,
                ratio,
                &label,
                "Maximum residual-norm to envelope-radius ratio within one deterministic run window.",
            );
        }
        return table;
    }

    if rows.is_empty() {
        if let Some(scenario) = detectability_summary_scenario(bundle) {
            let mut point_order = 0;
            if let Some(boundary_time) = first_non_admissible_time(scenario) {
                push_bar_row(
                    &mut table,
                    "detectability_bound",
                    "Predicted vs Observed Detectability Times",
                    "observed event",
                    "time to first event",
                    "observed_boundary_time",
                    "observed boundary",
                    "gold",
                    &scenario.record.id,
                    point_order,
                    0.18,
                    0.44,
                    boundary_time,
                    "boundary",
                    "Observed first non-admissible grammar interaction time rendered when no theorem-aligned predicted bound is available.",
                );
                point_order += 1;
            }
            if let Some(violation_time) = first_violation_time(scenario) {
                push_bar_row(
                    &mut table,
                    "detectability_bound",
                    "Predicted vs Observed Detectability Times",
                    "observed event",
                    "time to first event",
                    "observed_violation_time",
                    "observed violation",
                    "red",
                    &scenario.record.id,
                    point_order,
                    0.56,
                    0.82,
                    violation_time,
                    "violation",
                    "Observed first grammar-violation time rendered when no theorem-aligned predicted bound is available.",
                );
            }
            push_annotation_point(
                &mut table,
                "detectability_bound",
                "Predicted vs Observed Detectability Times",
                "observed event",
                "time to first event",
                "detectability_notice",
                "detectability notice",
                "annotation",
                "slate",
                &scenario.record.id,
                point_order + 1,
                0.5,
                0.0,
                "",
                &format!(
                    "scenario={} | no theorem-aligned predicted upper bound was configured for this run; showing observed grammar event times instead.",
                    scenario.record.id
                ),
                "Fallback note rendered when the selected run exposes observed grammar events but no predicted detectability upper bound.",
            );
            if table.rows.iter().all(|row| row.series_kind == "annotation") {
                push_annotation_point(
                    &mut table,
                    "detectability_bound",
                    "Predicted vs Observed Detectability Times",
                    "observed event",
                    "time to first event",
                    "detectability_notice_empty",
                    "detectability notice",
                    "annotation",
                    "slate",
                    &scenario.record.id,
                    point_order + 2,
                    0.5,
                    0.5,
                    "",
                    "No non-admissible grammar event was observed in this run, so no detectability-summary bars were available.",
                    "Fallback annotation rendered when the selected run exposes neither theorem-aligned detectability bounds nor observed non-admissible grammar events.",
                );
            }
        } else {
            push_annotation_point(
                &mut table,
                "detectability_bound",
                "Predicted vs Observed Detectability Times",
                "scenario index",
                "time to first exit",
                "detectability_notice",
                "detectability notice",
                "annotation",
                "slate",
                "",
                0,
                0.5,
                0.5,
                "",
                "No detectability-bound comparison rows were available for this run selection.",
                "Fallback annotation rendered when no scenarios in the selected bundle expose a detectability-bound comparison.",
            );
        }
    }
    table
}

pub(super) fn prepare_figure_12(bundle: &EngineOutputBundle) -> FigureSourceTable {
    if is_primary_bearings_bundle(bundle) {
        if let Ok(table) = prepare_figure_12_bearings(bundle) {
            return table;
        }
    }
    if is_primary_milling_bundle(bundle) {
        if let Ok(table) = prepare_figure_12_milling(bundle) {
            return table;
        }
    }
    if is_synthetic_bundle(bundle) {
        if let Ok(table) = prepare_figure_12_synthetic(bundle) {
            return table;
        }
    }
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_12_semantic_retrieval_heuristics_bank",
        "Run-Specific Constrained Retrieval Process",
        &[
            "post_regime_candidate_scores",
            "retrieval_filter_funnel",
            "retrieval_stage_rejections",
        ],
        &["retrieval_filter_funnel", "retrieval_stage_rejections"],
    );
    for (index, row) in semantic_retrieval_source_rows(bundle)
        .into_iter()
        .enumerate()
    {
        let label = compact_scenario_tick_label(&row.scenario_id, row.representative_rank);
        let score_bars = representative_candidate_score_bars(bundle, &row.scenario_id);
        if score_bars.is_empty() {
            let x_left = index as f64 * 1.2 + 0.18;
            push_bar_row(
                &mut table,
                "post_regime_candidate_scores",
                "Top Post-Regime Candidate Scores",
                "candidate preview",
                "candidate score",
                "no_post_regime_candidate",
                "no candidate",
                "slate",
                &row.scenario_id,
                index,
                x_left,
                x_left + 0.46,
                0.0,
                &format!("{label} none"),
                "Zero-height placeholder rendered when the current run produced no post-regime candidates.",
            );
        } else {
            for (candidate_index, (candidate_label, score)) in score_bars.into_iter().enumerate() {
                let x_base = index as f64 * 1.35 + candidate_index as f64 * 0.22;
                push_bar_row(
                    &mut table,
                    "post_regime_candidate_scores",
                    "Top Post-Regime Candidate Scores",
                    "candidate preview",
                    "candidate score",
                    &format!("candidate_score_{candidate_index}"),
                    &candidate_label,
                    ["blue", "red", "teal", "gold"][candidate_index % 4],
                    &row.scenario_id,
                    index * 10 + candidate_index,
                    x_base + 0.18,
                    x_base + 0.36,
                    score,
                    &format!("{label} {candidate_label}"),
                    "Ranked post-regime candidate score preview rendered from the exported retrieval audit.",
                );
            }
        }

        for (stage_index, (stage_id, stage_label, value, color_key)) in [
            (
                "prefilter_count",
                "pref",
                row.prefilter_candidate_count as f64,
                "slate",
            ),
            (
                "post_admissibility_count",
                "adm",
                row.heuristic_candidates_post_admissibility as f64,
                "gold",
            ),
            (
                "post_regime_count",
                "reg",
                row.heuristic_candidates_post_regime as f64,
                "teal",
            ),
            (
                "post_scope_count",
                "scope",
                row.heuristic_candidates_post_scope as f64,
                "green",
            ),
            (
                "selected_count",
                "sel",
                row.heuristics_selected_final as f64,
                "blue",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let x_base = index as f64 * 1.35 + stage_index as f64 * 0.22;
            push_bar_row(
                &mut table,
                "retrieval_filter_funnel",
                "Retrieval Filter Funnel Counts",
                "retrieval stage",
                "candidate count",
                stage_id,
                stage_label,
                color_key,
                &row.scenario_id,
                index * 10 + stage_index,
                x_base + 0.18,
                x_base + 0.36,
                value,
                &format!("{label} {stage_label}"),
                "Stage-wise candidate-count bar rendered from the exported retrieval audit.",
            );
        }

        for (stage_index, (stage_id, stage_label, value, color_key)) in [
            (
                "rejected_by_admissibility",
                "rej adm",
                row.heuristics_rejected_by_admissibility as f64,
                "gold",
            ),
            (
                "rejected_by_regime",
                "rej reg",
                row.heuristics_rejected_by_regime as f64,
                "teal",
            ),
            (
                "rejected_by_scope",
                "rej scope",
                row.heuristics_rejected_by_scope as f64,
                "red",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let x_base = index as f64 * 0.95 + stage_index as f64 * 0.24;
            push_bar_row(
                &mut table,
                "retrieval_stage_rejections",
                "Retrieval Rejections By Stage",
                "rejection stage",
                "candidate count",
                stage_id,
                stage_label,
                color_key,
                &row.scenario_id,
                index * 10 + stage_index,
                x_base + 0.18,
                x_base + 0.40,
                value,
                &format!("{label} {stage_label}"),
                "Stage-specific rejected-candidate count rendered from the exported retrieval audit.",
            );
        }
    }
    table
}

pub(super) fn prepare_figure_13(bundle: &EngineOutputBundle) -> FigureSourceTable {
    if is_primary_bearings_bundle(bundle) {
        if let Ok(table) = prepare_figure_13_bearings(bundle) {
            return table;
        }
    }
    if is_primary_milling_bundle(bundle) {
        if let Ok(table) = prepare_figure_13_milling(bundle) {
            return table;
        }
    }
    if is_synthetic_bundle(bundle) {
        if let Ok(table) = prepare_figure_13_synthetic(bundle) {
            return table;
        }
    }
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_13_internal_baseline_comparators",
        "Run-Specific Internal Comparator Activity",
        &[
            "comparator_first_trigger_time",
            "comparator_onset_rank",
            "comparator_trigger_counts",
        ],
        &["comparator_onset_rank", "comparator_trigger_counts"],
    );
    for (index, row) in baseline_comparator_source_rows(bundle)
        .into_iter()
        .enumerate()
    {
        push_bar_row(
            &mut table,
            "comparator_first_trigger_time",
            "Median First Trigger Time By Comparator",
            "comparator",
            "first trigger time",
            &format!("{}_first_trigger_time", row.comparator_id),
            &row.comparator_label,
            ["blue", "gold", "teal", "red", "green", "slate"][index % 6],
            &row.comparator_id,
            index,
            index as f64 + 0.18,
            index as f64 + 0.82,
            row.median_first_trigger_time.unwrap_or(0.0),
            short_comparator_tick_label(&row.comparator_id),
            "Comparator first-trigger timing bar. Zero indicates that no trigger was observed in the selected run.",
        );
        push_bar_row(
            &mut table,
            "comparator_onset_rank",
            "Comparator Onset Order",
            "comparator",
            "rank (0 = no trigger)",
            &format!("{}_onset_rank", row.comparator_id),
            &row.comparator_label,
            ["blue", "gold", "teal", "red", "green", "slate"][index % 6],
            &row.comparator_id,
            index,
            index as f64 + 0.18,
            index as f64 + 0.82,
            row.onset_rank.unwrap_or(0) as f64,
            short_comparator_tick_label(&row.comparator_id),
            "Comparator onset-order bar derived from the median first-trigger time ordering within the current run.",
        );
        push_bar_row(
            &mut table,
            "comparator_trigger_counts",
            "Triggered Scenario Count By Comparator",
            "comparator",
            "triggered scenarios",
            &row.comparator_id,
            &row.comparator_label,
            ["blue", "gold", "teal", "red", "green", "slate"][index % 6],
            &row.comparator_id,
            index,
            index as f64 + 0.18,
            index as f64 + 0.82,
            row.triggered_scenario_count as f64,
            short_comparator_tick_label(&row.comparator_id),
            "Comparator trigger-count bar.",
        );
    }
    table
}

fn prepare_figure_09_bearings(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = bearings_primary_scenario(bundle)?;
    let selection = select_matched_windows(scenario)
        .with_context(|| "unable to select comparable NASA Bearings windows for figure 09")?;
    let channel_label = scenario_channel_label(scenario, selection.channel_index);
    let note = format!(
        "Windows were matched within the NASA Bearings run by minimizing absolute primary residual-magnitude difference on `{}` while retaining materially different meta-slew and grammar outcomes. stable_steps={}..{}, departure_steps={}..{}, stable_mean_abs_primary={:.6}, departure_mean_abs_primary={:.6}, stable_mean_abs_meta={:.6}, departure_mean_abs_meta={:.6}. Primary residual magnitude alone does not separate the cases; meta-residual structure does.",
        channel_label,
        selection.stable_window.start,
        selection.stable_window.end.saturating_sub(1),
        selection.departure_window.start,
        selection.departure_window.end.saturating_sub(1),
        selection.stable_primary_mean,
        selection.departure_primary_mean,
        selection.stable_meta_mean,
        selection.departure_meta_mean
    );
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_09_detectability_bound_comparison",
        "NASA Bearings: Similar Primary Magnitude, Divergent Meta-Residual, Divergent Outcome",
        &[
            "primary_magnitude_similarity",
            "meta_residual_divergence",
            "outcome_consequence",
        ],
        &[],
    );
    push_scalar_series(
        &mut table,
        "primary_magnitude_similarity",
        "Panel A: Primary residual magnitude remains closely matched",
        "window-local sample index",
        &format!("|{} residual|", channel_label),
        "stable_primary_window",
        "stable window",
        "line",
        "blue",
        &scenario.record.id,
        scenario.residual.samples[selection.stable_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, sample)| {
                (
                    local_index,
                    local_index as f64,
                    sample
                        .values
                        .get(selection.channel_index)
                        .copied()
                        .unwrap_or_default()
                        .abs(),
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "primary_magnitude_similarity",
        "Panel A: Primary residual magnitude remains closely matched",
        "window-local sample index",
        &format!("|{} residual|", channel_label),
        "departure_primary_window",
        "departure window",
        "line",
        "red",
        &scenario.record.id,
        scenario.residual.samples[selection.departure_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, sample)| {
                (
                    local_index,
                    local_index as f64,
                    sample
                        .values
                        .get(selection.channel_index)
                        .copied()
                        .unwrap_or_default()
                        .abs(),
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "meta_residual_divergence",
        "Panel B: Meta-residual slew diverges despite similar primary magnitude",
        "window-local sample index",
        &format!("|{} slew|", channel_label),
        "stable_meta_window",
        "stable meta-window",
        "line",
        "blue",
        &scenario.record.id,
        scenario.slew.samples[selection.stable_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, sample)| {
                (
                    local_index,
                    local_index as f64,
                    sample
                        .values
                        .get(selection.channel_index)
                        .copied()
                        .unwrap_or_default()
                        .abs(),
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "meta_residual_divergence",
        "Panel B: Meta-residual slew diverges despite similar primary magnitude",
        "window-local sample index",
        &format!("|{} slew|", channel_label),
        "departure_meta_window",
        "departure meta-window",
        "line",
        "red",
        &scenario.record.id,
        scenario.slew.samples[selection.departure_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, sample)| {
                (
                    local_index,
                    local_index as f64,
                    sample
                        .values
                        .get(selection.channel_index)
                        .copied()
                        .unwrap_or_default()
                        .abs(),
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "outcome_consequence",
        "Panel C: Grammar outcome differs after the meta-residual divergence",
        "window-local sample index",
        "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
        "stable_outcome_window",
        "stable outcome",
        "line",
        "blue",
        &scenario.record.id,
        scenario.grammar[selection.stable_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, status)| {
                (
                    local_index,
                    local_index as f64,
                    grammar_state_code(status.state) as f64,
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "outcome_consequence",
        "Panel C: Grammar outcome differs after the meta-residual divergence",
        "window-local sample index",
        "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
        "departure_outcome_window",
        "departure outcome",
        "line",
        "red",
        &scenario.record.id,
        scenario.grammar[selection.departure_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, status)| {
                (
                    local_index,
                    local_index as f64,
                    grammar_state_code(status.state) as f64,
                )
            }),
        &note,
    );
    Ok(table)
}

fn prepare_figure_12_bearings(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = bearings_primary_scenario(bundle)?;
    let timeline = build_prefix_semantic_timeline(bundle, scenario)?;
    let note = "NASA Bearings semantic-timeline source derived from prefix-by-prefix retrieval over the executed run. The figure shows semantic evolution rather than a single final candidate snapshot.".to_string();
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_12_semantic_retrieval_heuristics_bank",
        "NASA Bearings: Semantic Evolution, Ambiguity Narrowing, and Disposition Timeline",
        &[
            "semantic_score_timeline",
            "semantic_candidate_count_timeline",
            "semantic_disposition_timeline",
        ],
        &[],
    );
    push_scalar_series(
        &mut table,
        "semantic_score_timeline",
        "Panel A: Top-candidate score and margin evolve through time",
        "time",
        "candidate score / score margin",
        "top_candidate_score",
        "top candidate score",
        "line",
        "blue",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.top_score)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "semantic_score_timeline",
        "Panel A: Top-candidate score and margin evolve through time",
        "time",
        "candidate score / score margin",
        "top_score_margin",
        "top score margin",
        "line",
        "red",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.top_score_margin)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "semantic_candidate_count_timeline",
        "Panel B: Candidate-set narrowing through admissibility and scope",
        "time",
        "candidate count",
        "post_regime_count",
        "post-regime count",
        "line",
        "gold",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.post_regime_candidate_count as f64)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "semantic_candidate_count_timeline",
        "Panel B: Candidate-set narrowing through admissibility and scope",
        "time",
        "candidate count",
        "post_scope_count",
        "post-scope count",
        "line",
        "teal",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.post_scope_candidate_count as f64)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "semantic_disposition_timeline",
        "Panel C: Semantic disposition evolves from ambiguity toward constrained interpretation",
        "time",
        "semantic code (0=Unknown, 1=Ambiguous, 2=CompatibleSet, 3=Match)",
        "semantic_disposition_code",
        "semantic disposition",
        "line",
        "green",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.semantic_disposition_code as f64)),
        &note,
    );
    Ok(table)
}

fn prepare_figure_13_bearings(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = bearings_primary_scenario(bundle)?;
    let semantic_timeline = build_prefix_semantic_timeline(bundle, scenario)?;
    let event_timeline = build_scenario_event_timeline(bundle, scenario)?;
    let note = "NASA Bearings interpretability-delta source rows derived from the executed comparator results, grammar trajectory, and prefix semantic timeline. The figure is framed as baseline alarm view versus DSFB structural interpretation, not as a benchmark superiority claim.".to_string();
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_13_internal_baseline_comparators",
        "NASA Bearings: Baseline Alarm View Versus DSFB Structural Interpretation",
        &[
            "baseline_alarm_timing",
            "dsfb_grammar_timeline",
            "dsfb_semantic_timeline",
        ],
        &["baseline_alarm_timing"],
    );
    let triggered_results = bundle
        .evaluation
        .baseline_results
        .iter()
        .filter(|result| result.scenario_id == scenario.record.id && result.triggered)
        .collect::<Vec<_>>();
    for (index, result) in triggered_results.iter().enumerate() {
        if let Some(first_trigger_time) = result.first_trigger_time {
            push_bar_row(
                &mut table,
                "baseline_alarm_timing",
                "Panel A: Baseline comparator first-alarm timing",
                "comparator",
                "first alarm time",
                &format!("{}_first_alarm", result.comparator_id),
                &result.comparator_label,
                ["blue", "gold", "teal", "red", "green", "slate"][index % 6],
                &scenario.record.id,
                index,
                index as f64 + 0.18,
                index as f64 + 0.82,
                first_trigger_time,
                short_comparator_tick_label(&result.comparator_id),
                &note,
            );
        }
    }
    push_scalar_series(
        &mut table,
        "dsfb_grammar_timeline",
        "Panel B: DSFB syntax and grammar add structured temporal context",
        "time",
        "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
        "grammar_state_code",
        "grammar state",
        "line",
        "red",
        &scenario.record.id,
        scenario
            .grammar
            .iter()
            .enumerate()
            .map(|(index, status)| (index, status.time, grammar_state_code(status.state) as f64)),
        &note,
    );
    for (index, event) in event_timeline
        .iter()
        .filter(|event| event.layer == "syntax")
        .enumerate()
    {
        push_segment_row(
            &mut table,
            "dsfb_grammar_timeline",
            "Panel B: DSFB syntax and grammar add structured temporal context",
            "time",
            "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
            &format!("syntax_transition_{}", index + 1),
            &event.event_label,
            "slate",
            index,
            event.time,
            0.0,
            event.time,
            2.1,
            &scenario.record.id,
            &note,
        );
    }
    push_scalar_series(
        &mut table,
        "dsfb_semantic_timeline",
        "Panel C: DSFB semantic layer shows interpretation evolution beyond alarms",
        "time",
        "semantic code (0=Unknown, 1=Ambiguous, 2=CompatibleSet, 3=Match)",
        "semantic_disposition_code",
        "semantic disposition",
        "line",
        "green",
        &scenario.record.id,
        semantic_timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.semantic_disposition_code as f64)),
        &note,
    );
    Ok(table)
}
fn prepare_figure_09_milling(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = milling_primary_scenario(bundle)?;
    let selection = select_process_windows(scenario)
        .with_context(|| "unable to select comparable NASA Milling windows for figure 09")?;
    let channel_label = scenario_channel_label(scenario, selection.channel_index);
    let note = format!(
        "Windows were matched within the NASA Milling run by minimizing absolute primary residual-magnitude difference on `{}` while retaining materially different higher-order slew and grammar outcomes. stable_steps={}..{}, departure_steps={}..{}, stable_mean_abs_primary={:.6}, departure_mean_abs_primary={:.6}, stable_mean_abs_meta={:.6}, departure_mean_abs_meta={:.6}, stable_mean_outcome={:.6}, departure_mean_outcome={:.6}. Similar primary behavior alone does not separate the milling process windows; first-order behavior alone is insufficient, and higher-order structure plus grammar outcome do.",
        channel_label,
        selection.stable_window.start,
        selection.stable_window.end.saturating_sub(1),
        selection.departure_window.start,
        selection.departure_window.end.saturating_sub(1),
        selection.stable_primary_mean,
        selection.departure_primary_mean,
        selection.stable_meta_mean,
        selection.departure_meta_mean,
        selection.stable_outcome_mean,
        selection.departure_outcome_mean
    );
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_09_detectability_bound_comparison",
        "NASA Milling: Similar Primary Behavior, Divergent Higher-Order Structure, Divergent Outcome",
        &[
            "primary_magnitude_similarity",
            "meta_residual_divergence",
            "outcome_consequence",
        ],
        &[],
    );
    push_scalar_series(
        &mut table,
        "primary_magnitude_similarity",
        "Panel A: Similar primary residual behavior across milling process windows",
        "window-local sample index",
        &format!("|{} residual|", channel_label),
        "stable_primary_window",
        "stable process window",
        "line",
        "blue",
        &scenario.record.id,
        scenario.residual.samples[selection.stable_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, sample)| {
                (
                    local_index,
                    local_index as f64,
                    sample
                        .values
                        .get(selection.channel_index)
                        .copied()
                        .unwrap_or_default()
                        .abs(),
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "primary_magnitude_similarity",
        "Panel A: Similar primary residual behavior across milling process windows",
        "window-local sample index",
        &format!("|{} residual|", channel_label),
        "departure_primary_window",
        "departure process window",
        "line",
        "red",
        &scenario.record.id,
        scenario.residual.samples[selection.departure_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, sample)| {
                (
                    local_index,
                    local_index as f64,
                    sample
                        .values
                        .get(selection.channel_index)
                        .copied()
                        .unwrap_or_default()
                        .abs(),
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "meta_residual_divergence",
        "Panel B: Higher-order slew diverges within the milling run",
        "window-local sample index",
        &format!("|{} slew|", channel_label),
        "stable_meta_window",
        "stable meta-window",
        "line",
        "blue",
        &scenario.record.id,
        scenario.slew.samples[selection.stable_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, sample)| {
                (
                    local_index,
                    local_index as f64,
                    sample
                        .values
                        .get(selection.channel_index)
                        .copied()
                        .unwrap_or_default()
                        .abs(),
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "meta_residual_divergence",
        "Panel B: Higher-order slew diverges within the milling run",
        "window-local sample index",
        &format!("|{} slew|", channel_label),
        "departure_meta_window",
        "departure meta-window",
        "line",
        "red",
        &scenario.record.id,
        scenario.slew.samples[selection.departure_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, sample)| {
                (
                    local_index,
                    local_index as f64,
                    sample
                        .values
                        .get(selection.channel_index)
                        .copied()
                        .unwrap_or_default()
                        .abs(),
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "outcome_consequence",
        "Panel C: Grammar outcome diverges across the matched milling windows",
        "window-local sample index",
        "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
        "stable_outcome_window",
        "stable outcome",
        "line",
        "blue",
        &scenario.record.id,
        scenario.grammar[selection.stable_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, status)| {
                (
                    local_index,
                    local_index as f64,
                    grammar_state_code(status.state) as f64,
                )
            }),
        &note,
    );
    push_scalar_series(
        &mut table,
        "outcome_consequence",
        "Panel C: Grammar outcome diverges across the matched milling windows",
        "window-local sample index",
        "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
        "departure_outcome_window",
        "departure outcome",
        "line",
        "red",
        &scenario.record.id,
        scenario.grammar[selection.departure_window.clone()]
            .iter()
            .enumerate()
            .map(|(local_index, status)| {
                (
                    local_index,
                    local_index as f64,
                    grammar_state_code(status.state) as f64,
                )
            }),
        &note,
    );
    Ok(table)
}

fn prepare_figure_09_synthetic(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let selection = select_synthetic_structural_pair(bundle)
        .with_context(|| "unable to select comparable synthetic cases for figure 09")?;
    let admissible = selection.admissible_case;
    let detectable = selection.detectable_case;
    let note = format!(
        "Synthetic figure 09 uses the executed synthetic pair `{}` and `{}` chosen by minimizing primary residual-magnitude difference while preserving materially different higher-order slew and grammar outcomes. admissible_mean_primary={:.6}, detectable_mean_primary={:.6}, admissible_mean_meta={:.6}, detectable_mean_meta={:.6}, admissible_mean_outcome={:.6}, detectable_mean_outcome={:.6}. The cases keep apparent primary residual magnitude similar while higher-order structure and grammar outcome diverge, so primary behavior alone is insufficient.",
        admissible.record.id,
        detectable.record.id,
        selection.admissible_primary_mean,
        selection.detectable_primary_mean,
        selection.admissible_meta_mean,
        selection.detectable_meta_mean,
        selection.admissible_outcome_mean,
        selection.detectable_outcome_mean
    );
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_09_detectability_bound_comparison",
        "Synthetic: Similar Primary Magnitude, Divergent Higher-Order Structure, Divergent Outcome",
        &[
            "primary_magnitude_similarity",
            "meta_residual_divergence",
            "outcome_consequence",
        ],
        &[],
    );
    push_scalar_series(
        &mut table,
        "primary_magnitude_similarity",
        "Panel A: Primary residual magnitude stays closely matched by construction",
        "time",
        "residual norm",
        "admissible_primary_case",
        "magnitude-matched admissible",
        "line",
        "blue",
        &admissible.record.id,
        admissible
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "primary_magnitude_similarity",
        "Panel A: Primary residual magnitude stays closely matched by construction",
        "time",
        "residual norm",
        "detectable_primary_case",
        "magnitude-matched detectable",
        "line",
        "red",
        &detectable.record.id,
        detectable
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "meta_residual_divergence",
        "Panel B: Higher-order slew separates the controlled synthetic cases",
        "time",
        "slew norm",
        "admissible_meta_case",
        "admissible higher-order structure",
        "line",
        "blue",
        &admissible.record.id,
        admissible
            .slew
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "meta_residual_divergence",
        "Panel B: Higher-order slew separates the controlled synthetic cases",
        "time",
        "slew norm",
        "detectable_meta_case",
        "detectable higher-order structure",
        "line",
        "red",
        &detectable.record.id,
        detectable
            .slew
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "outcome_consequence",
        "Panel C: Grammar outcome diverges under similar first-order behavior",
        "time",
        "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
        "admissible_outcome_case",
        "admissible outcome",
        "line",
        "blue",
        &admissible.record.id,
        admissible
            .grammar
            .iter()
            .enumerate()
            .map(|(index, status)| (index, status.time, grammar_state_code(status.state) as f64)),
        &note,
    );
    push_scalar_series(
        &mut table,
        "outcome_consequence",
        "Panel C: Grammar outcome diverges under similar first-order behavior",
        "time",
        "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
        "detectable_outcome_case",
        "detectable outcome",
        "line",
        "red",
        &detectable.record.id,
        detectable
            .grammar
            .iter()
            .enumerate()
            .map(|(index, status)| (index, status.time, grammar_state_code(status.state) as f64)),
        &note,
    );
    Ok(table)
}

fn prepare_figure_12_milling(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = milling_primary_scenario(bundle)?;
    let timeline = build_prefix_semantic_timeline(bundle, scenario)?;
    let note = "NASA Milling semantic-timeline source derived from prefix-by-prefix retrieval over the executed milling run. The figure shows semantic process through the milling progression rather than a single final candidate snapshot.".to_string();
    build_semantic_timeline_table(
        bundle,
        scenario,
        &timeline,
        "NASA Milling: Semantic Evolution Through Process Windows",
        &note,
    )
}

fn prepare_figure_12_synthetic(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = synthetic_transition_scenario(bundle)?;
    let timeline = build_prefix_semantic_timeline(bundle, scenario)?;
    let note = format!(
        "Synthetic semantic-timeline source derived from prefix-by-prefix retrieval over `{}`. The figure shows semantic evolution through the controlled synthetic transition rather than a static label snapshot.",
        scenario.record.id
    );
    build_semantic_timeline_table(
        bundle,
        scenario,
        &timeline,
        "Synthetic: Semantic Evolution, Candidate Narrowing, and Disposition Timeline",
        &note,
    )
}

fn prepare_figure_13_milling(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = milling_primary_scenario(bundle)?;
    let semantic_timeline = build_prefix_semantic_timeline(bundle, scenario)?;
    let event_timeline = build_scenario_event_timeline(bundle, scenario)?;
    let note = "NASA Milling interpretability-delta source rows derived from the executed comparator results, grammar trajectory, and prefix semantic timeline. The figure is framed as baseline comparator view versus DSFB structural interpretation in the milling context, not as a performance benchmark.".to_string();
    build_interpretability_delta_table(
        bundle,
        scenario,
        &semantic_timeline,
        &event_timeline,
        "NASA Milling: Baseline Comparator View Versus DSFB Structural Interpretation",
        &note,
    )
}

fn prepare_figure_13_synthetic(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = synthetic_transition_scenario(bundle)?;
    let semantic_timeline = build_prefix_semantic_timeline(bundle, scenario)?;
    let event_timeline = build_scenario_event_timeline(bundle, scenario)?;
    let note = format!(
        "Synthetic interpretability-delta source rows derived from the executed comparator results, grammar trajectory, and prefix semantic timeline for `{}`. The figure shows what the internal deterministic comparators see first and what DSFB adds structurally, without claiming performance superiority.",
        scenario.record.id
    );
    build_interpretability_delta_table(
        bundle,
        scenario,
        &semantic_timeline,
        &event_timeline,
        "Synthetic: Baseline Comparator View Versus DSFB Structural Interpretation",
        &note,
    )
}

fn build_semantic_timeline_table(
    bundle: &EngineOutputBundle,
    scenario: &ScenarioOutput,
    timeline: &[crate::engine::event_timeline::PrefixSemanticTimelinePoint],
    plot_title: &str,
    note: &str,
) -> Result<FigureSourceTable> {
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_12_semantic_retrieval_heuristics_bank",
        plot_title,
        &[
            "semantic_score_timeline",
            "semantic_candidate_count_timeline",
            "semantic_disposition_timeline",
        ],
        &[],
    );
    push_scalar_series(
        &mut table,
        "semantic_score_timeline",
        "Panel A: Top-candidate score and score margin evolve through time",
        "time",
        "candidate score / score margin",
        "top_candidate_score",
        "top candidate score",
        "line",
        "blue",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.top_score)),
        note,
    );
    push_scalar_series(
        &mut table,
        "semantic_score_timeline",
        "Panel A: Top-candidate score and score margin evolve through time",
        "time",
        "candidate score / score margin",
        "top_score_margin",
        "top score margin",
        "line",
        "red",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.top_score_margin)),
        note,
    );
    push_scalar_series(
        &mut table,
        "semantic_candidate_count_timeline",
        "Panel B: Candidate-set narrowing through admissibility and scope",
        "time",
        "candidate count",
        "post_regime_count",
        "post-regime count",
        "line",
        "gold",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.post_regime_candidate_count as f64)),
        note,
    );
    push_scalar_series(
        &mut table,
        "semantic_candidate_count_timeline",
        "Panel B: Candidate-set narrowing through admissibility and scope",
        "time",
        "candidate count",
        "post_scope_count",
        "post-scope count",
        "line",
        "teal",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.post_scope_candidate_count as f64)),
        note,
    );
    push_scalar_series(
        &mut table,
        "semantic_disposition_timeline",
        "Panel C: Semantic disposition evolves through the current run",
        "time",
        "semantic code (0=Unknown, 1=Ambiguous, 2=CompatibleSet, 3=Match)",
        "semantic_disposition_code",
        "semantic disposition",
        "line",
        "green",
        &scenario.record.id,
        timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.semantic_disposition_code as f64)),
        note,
    );
    Ok(table)
}

fn build_interpretability_delta_table(
    bundle: &EngineOutputBundle,
    scenario: &ScenarioOutput,
    semantic_timeline: &[crate::engine::event_timeline::PrefixSemanticTimelinePoint],
    event_timeline: &[crate::engine::event_timeline::ScenarioEventTimelineRow],
    plot_title: &str,
    note: &str,
) -> Result<FigureSourceTable> {
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_13_internal_baseline_comparators",
        plot_title,
        &[
            "baseline_alarm_timing",
            "dsfb_grammar_timeline",
            "dsfb_semantic_timeline",
        ],
        &["baseline_alarm_timing"],
    );
    let triggered_results = bundle
        .evaluation
        .baseline_results
        .iter()
        .filter(|result| result.scenario_id == scenario.record.id && result.triggered)
        .collect::<Vec<_>>();
    for (index, result) in triggered_results.iter().enumerate() {
        if let Some(first_trigger_time) = result.first_trigger_time {
            push_bar_row(
                &mut table,
                "baseline_alarm_timing",
                "Panel A: Baseline comparator first-alarm timing",
                "comparator",
                "first alarm time",
                &format!("{}_first_alarm", result.comparator_id),
                &result.comparator_label,
                ["blue", "gold", "teal", "red", "green", "slate"][index % 6],
                &scenario.record.id,
                index,
                index as f64 + 0.18,
                index as f64 + 0.82,
                first_trigger_time,
                short_comparator_tick_label(&result.comparator_id),
                note,
            );
        }
    }
    push_scalar_series(
        &mut table,
        "dsfb_grammar_timeline",
        "Panel B: DSFB syntax and grammar add structured temporal context",
        "time",
        "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
        "grammar_state_code",
        "grammar state",
        "line",
        "red",
        &scenario.record.id,
        scenario
            .grammar
            .iter()
            .enumerate()
            .map(|(index, status)| (index, status.time, grammar_state_code(status.state) as f64)),
        note,
    );
    for (index, event) in event_timeline
        .iter()
        .filter(|event| event.layer == "syntax")
        .enumerate()
    {
        push_segment_row(
            &mut table,
            "dsfb_grammar_timeline",
            "Panel B: DSFB syntax and grammar add structured temporal context",
            "time",
            "grammar state code (0=Adm, 1=Bnd, 2=Viol)",
            &format!("syntax_transition_{}", index + 1),
            &event.event_label,
            "slate",
            index,
            event.time,
            0.0,
            event.time,
            2.1,
            &scenario.record.id,
            note,
        );
    }
    push_scalar_series(
        &mut table,
        "dsfb_semantic_timeline",
        "Panel C: DSFB semantic layer shows interpretation evolution beyond alarms",
        "time",
        "semantic code (0=Unknown, 1=Ambiguous, 2=CompatibleSet, 3=Match)",
        "semantic_disposition_code",
        "semantic disposition",
        "line",
        "green",
        &scenario.record.id,
        semantic_timeline
            .iter()
            .enumerate()
            .map(|(index, point)| (index, point.time, point.semantic_disposition_code as f64)),
        note,
    );
    Ok(table)
}
