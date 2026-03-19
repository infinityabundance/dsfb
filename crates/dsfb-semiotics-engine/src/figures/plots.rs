use std::path::Path;

use anyhow::{Context, Result};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};

use crate::engine::types::{EngineOutputBundle, FigureArtifact, GrammarState, ScenarioOutput};
use crate::figures::export::figure_paths;
use crate::figures::styles::{BLUE, GOLD, GREEN, RED, SLATE, TEAL, WHITE_BG};

pub fn render_all_figures(
    bundle: &EngineOutputBundle,
    figures_dir: &Path,
) -> Result<Vec<FigureArtifact>> {
    let mut figures = Vec::new();
    figures.push(render_01(bundle, figures_dir)?);
    figures.push(render_02(bundle, figures_dir)?);
    figures.push(render_03(bundle, figures_dir)?);
    figures.push(render_04(bundle, figures_dir)?);
    figures.push(render_05(bundle, figures_dir)?);
    figures.push(render_06(bundle, figures_dir)?);
    figures.push(render_07(bundle, figures_dir)?);
    figures.push(render_08(bundle, figures_dir)?);
    figures.push(render_09(bundle, figures_dir)?);
    figures.push(render_10(figures_dir)?);
    figures.push(render_11(bundle, figures_dir)?);
    figures.push(render_12(bundle, figures_dir)?);
    Ok(figures)
}

fn figure_observation_overview<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    scenario: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let areas = root.split_evenly((2, 1));
    let upper = &areas[0];
    let lower = &areas[1];
    let times = times(scenario);
    let observed = series_channel(&scenario.observed.samples, 0);
    let predicted = series_channel(&scenario.predicted.samples, 0);
    let residual_norm = scenario
        .residual
        .samples
        .iter()
        .map(|s| s.norm)
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&times);
    let (y_min, y_max) = combined_bounds(&[observed.clone(), predicted.clone()]);

    let mut chart = ChartBuilder::on(upper)
        .caption("Observation and Prediction", ("sans-serif", 28))
        .margin(16)
        .x_label_area_size(36)
        .y_label_area_size(56)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_desc("time")
        .y_desc("channel 1 trajectory")
        .draw()?;
    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(observed.iter().copied()),
            &BLUE,
        ))?
        .label("observed")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], BLUE.stroke_width(3)));
    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(predicted.iter().copied()),
            &GREEN,
        ))?
        .label("predicted")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], GREEN.stroke_width(3)));
    chart.configure_series_labels().border_style(BLACK).draw()?;

    let (_, residual_max) = bounds(&residual_norm);
    let mut residual_chart = ChartBuilder::on(lower)
        .caption("Residual Norm", ("sans-serif", 28))
        .margin(16)
        .x_label_area_size(36)
        .y_label_area_size(56)
        .build_cartesian_2d(x_min..x_max, 0.0..(residual_max * 1.15).max(0.1))?;
    residual_chart
        .configure_mesh()
        .x_desc("time")
        .y_desc("||r(t)||")
        .draw()?;
    residual_chart.draw_series(LineSeries::new(
        times.iter().copied().zip(residual_norm.into_iter()),
        &RED,
    ))?;
    root.present()?;
    Ok(())
}

fn figure_drift_slew<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    scenario: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let areas = root.split_evenly((3, 1));
    let times = times(scenario);
    let residual = scenario
        .residual
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();
    let drift = scenario
        .sign
        .samples
        .iter()
        .map(|sample| sample.projection[1])
        .collect::<Vec<_>>();
    let slew = scenario
        .slew
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();

    draw_single_series(
        &areas[0],
        "Residual Norm",
        &times,
        &residual,
        "||r(t)||",
        &BLUE,
    )?;
    draw_single_series(
        &areas[1],
        "Signed Radial Drift",
        &times,
        &drift,
        "dot(r,d)/||r||",
        &GREEN,
    )?;
    draw_single_series(&areas[2], "Slew Norm", &times, &slew, "||s(t)||", &RED)?;
    root.present()?;
    Ok(())
}

fn figure_sign_space<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    scenario: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let points = scenario
        .sign
        .samples
        .iter()
        .map(|sample| (sample.projection[0], sample.projection[1]))
        .collect::<Vec<_>>();
    let x_values = points.iter().map(|(x, _)| *x).collect::<Vec<_>>();
    let y_values = points.iter().map(|(_, y)| *y).collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&x_values);
    let (y_min, y_max) = bounds(&y_values);

    let mut chart = ChartBuilder::on(&root)
        .caption("Projected Sign-Space Trajectory", ("sans-serif", 30))
        .margin(24)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_desc(format!(
            "projection coordinate 1: {}",
            scenario.sign.projection_metadata.axis_labels[0]
        ))
        .y_desc(format!(
            "projection coordinate 2: {}",
            scenario.sign.projection_metadata.axis_labels[1]
        ))
        .draw()?;
    chart.draw_series(LineSeries::new(points.iter().copied(), &TEAL))?;
    chart.draw_series(points.iter().enumerate().step_by(12).map(|(index, point)| {
        let color = if index == 0 { GREEN } else { BLUE };
        Circle::new(*point, 4, color.filled())
    }))?;
    if let Some(first) = points.first() {
        chart.draw_series(std::iter::once(Text::new(
            "start",
            *first,
            ("sans-serif", 18).into_font().color(&GREEN),
        )))?;
    }
    if let Some(last) = points.last() {
        chart.draw_series(std::iter::once(Text::new(
            "end",
            *last,
            ("sans-serif", 18).into_font().color(&RED),
        )))?;
    }
    chart.draw_series(std::iter::once(Text::new(
        scenario.sign.projection_metadata.note.clone(),
        (
            x_min + (x_max - x_min) * 0.03,
            y_max - (y_max - y_min) * 0.08,
        ),
        ("sans-serif", 14).into_font().color(&SLATE),
    )))?;
    root.present()?;
    Ok(())
}

fn figure_syntax_comparison<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    monotone: &ScenarioOutput,
    curvature: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let times = times(monotone);
    let monotone_series = monotone
        .residual
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();
    let curvature_series = curvature
        .residual
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&times);
    let (y_min, y_max) = combined_bounds(&[monotone_series.clone(), curvature_series.clone()]);
    let mut chart = ChartBuilder::on(&root)
        .caption("Syntax Comparison", ("sans-serif", 30))
        .margin(24)
        .x_label_area_size(40)
        .y_label_area_size(56)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_desc("time")
        .y_desc("residual norm")
        .draw()?;
    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(monotone_series.iter().copied()),
            &BLUE,
        ))?
        .label("monotone drift")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], BLUE.stroke_width(3)));
    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(curvature_series.iter().copied()),
            &RED,
        ))?
        .label("curvature dominated")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], RED.stroke_width(3)));
    chart.configure_series_labels().border_style(BLACK).draw()?;
    root.present()?;
    Ok(())
}

fn figure_envelope_exit<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    scenario: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    draw_norm_vs_envelope(
        &root,
        scenario,
        "Envelope Exit Under Sustained Outward Drift",
    )?;
    root.present()?;
    Ok(())
}

fn figure_envelope_invariance<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    scenario: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    draw_norm_vs_envelope(
        &root,
        scenario,
        "Envelope Invariance Under Inward-Compatible Drift",
    )?;
    root.present()?;
    Ok(())
}

fn figure_exit_invariance_pair<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    exit_case: &ScenarioOutput,
    invariance_case: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let times = times(exit_case);
    let envelope = exit_case
        .envelope
        .samples
        .iter()
        .map(|sample| sample.radius)
        .collect::<Vec<_>>();
    let exit_norm = exit_case
        .residual
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();
    let invariance_norm = invariance_case
        .residual
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&times);
    let (_, y_max) =
        combined_bounds(&[envelope.clone(), exit_norm.clone(), invariance_norm.clone()]);
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Exit-Invariance Pair on Shared Envelope",
            ("sans-serif", 30),
        )
        .margin(24)
        .x_label_area_size(40)
        .y_label_area_size(56)
        .build_cartesian_2d(x_min..x_max, 0.0..(y_max * 1.15))?;
    chart
        .configure_mesh()
        .x_desc("time")
        .y_desc("norm / envelope radius")
        .draw()?;
    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(envelope.iter().copied()),
            &SLATE,
        ))?
        .label("shared envelope")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], SLATE.stroke_width(3)));
    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(exit_norm.iter().copied()),
            &RED,
        ))?
        .label("outward drift")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], RED.stroke_width(3)));
    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(invariance_norm.iter().copied()),
            &GREEN,
        ))?
        .label("inward compatible")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], GREEN.stroke_width(3)));
    chart.configure_series_labels().border_style(BLACK).draw()?;
    root.present()?;
    Ok(())
}

fn figure_residual_separation<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    admissible: &ScenarioOutput,
    detectable: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let times = times(admissible);
    let admissible_norm = admissible
        .residual
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();
    let detectable_norm = detectable
        .residual
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();
    let envelope = detectable
        .envelope
        .samples
        .iter()
        .map(|sample| sample.radius)
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&times);
    let (_, y_max) = combined_bounds(&[
        admissible_norm.clone(),
        detectable_norm.clone(),
        envelope.clone(),
    ]);
    let mut chart = ChartBuilder::on(&root)
        .caption("Residual Trajectory Separation", ("sans-serif", 30))
        .margin(24)
        .x_label_area_size(40)
        .y_label_area_size(56)
        .build_cartesian_2d(x_min..x_max, 0.0..(y_max * 1.15))?;
    chart
        .configure_mesh()
        .x_desc("time")
        .y_desc("||r(t)||")
        .draw()?;
    chart.draw_series(LineSeries::new(
        times.iter().copied().zip(envelope.iter().copied()),
        &SLATE,
    ))?;
    chart.draw_series(LineSeries::new(
        times.iter().copied().zip(admissible_norm.iter().copied()),
        &BLUE,
    ))?;
    chart.draw_series(LineSeries::new(
        times.iter().copied().zip(detectable_norm.iter().copied()),
        &RED,
    ))?;
    root.present()?;
    Ok(())
}

fn figure_detectability_bounds<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    bundle: &EngineOutputBundle,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let cases = [
        "outward_exit_case_a",
        "outward_exit_case_b",
        "outward_exit_case_c",
        "magnitude_matched_detectable",
    ]
    .iter()
    .filter_map(|id| {
        bundle
            .scenario_outputs
            .iter()
            .find(|scenario| &scenario.record.id == id)
    })
    .collect::<Vec<_>>();
    let cases = if cases.is_empty() {
        bundle
            .scenario_outputs
            .iter()
            .filter(|scenario| scenario.detectability.predicted_upper_bound.is_some())
            .collect::<Vec<_>>()
    } else {
        cases
    };
    let y_max = cases
        .iter()
        .flat_map(|scenario| {
            [
                scenario.detectability.predicted_upper_bound.unwrap_or(0.0),
                scenario.detectability.observed_crossing_time.unwrap_or(0.0),
            ]
        })
        .fold(0.0, f64::max)
        .max(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Predicted vs Observed Detectability Times",
            ("sans-serif", 30),
        )
        .margin(24)
        .x_label_area_size(60)
        .y_label_area_size(64)
        .build_cartesian_2d(0.0_f64..cases.len().max(1) as f64, 0.0_f64..(y_max * 1.15))?;
    chart
        .configure_mesh()
        .x_desc("scenario index")
        .y_desc("time to first exit")
        .draw()?;

    for (index, scenario) in cases.iter().enumerate() {
        let x = index as f64 + 0.25;
        if let Some(predicted) = scenario.detectability.predicted_upper_bound {
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x, 0.0), (x + 0.18, predicted)],
                BLUE.filled(),
            )))?;
        }
        if let Some(observed) = scenario.detectability.observed_crossing_time {
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x + 0.22, 0.0), (x + 0.40, observed)],
                RED.filled(),
            )))?;
        }
        chart.draw_series(std::iter::once(Text::new(
            scenario.record.id.clone(),
            (index as f64 + 0.02, 4.0),
            ("sans-serif", 14).into_font().color(&BLACK),
        )))?;
    }
    root.present()?;
    Ok(())
}

fn figure_pipeline_flow<DB: DrawingBackend>(root: DrawingArea<DB, Shift>) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    root.draw(&Text::new(
        "Deterministic Structural Semiotics Engine",
        (800, 110),
        TextStyle::from(("sans-serif", 36).into_font())
            .color(&BLACK)
            .pos(Pos::new(HPos::Center, VPos::Center)),
    ))?;
    root.draw(&Text::new(
        "Fixed layered maps from residual extraction to constrained semantic retrieval",
        (800, 150),
        TextStyle::from(("sans-serif", 22).into_font())
            .color(&SLATE)
            .pos(Pos::new(HPos::Center, VPos::Center)),
    ))?;

    let boxes = [
        (
            (70, 250),
            (310, 420),
            "1",
            "Residual Layer",
            "r(t) = y(t) - y_hat(t)",
        ),
        (
            (370, 250),
            (610, 420),
            "2",
            "Sign Layer",
            "sigma(t) = (r(t), d(t), s(t))",
        ),
        (
            (670, 250),
            (910, 420),
            "3",
            "Syntax Layer",
            "drift / slew structure",
        ),
        (
            (970, 250),
            (1210, 420),
            "4",
            "Grammar Layer",
            "||r(t)|| <= rho(t)",
        ),
        (
            (1270, 250),
            (1510, 420),
            "5",
            "Semantics Layer",
            "heuristics bank retrieval",
        ),
    ];
    for (index, (start, end, ordinal, title, subtitle)) in boxes.iter().enumerate() {
        let color = [BLUE, TEAL, GREEN, GOLD, RED][index];
        root.draw(&Rectangle::new([*start, *end], color.mix(0.18).filled()))?;
        root.draw(&Rectangle::new([*start, *end], color.stroke_width(3)))?;
        let center_x = (start.0 + end.0) / 2;
        root.draw(&Text::new(
            *ordinal,
            (center_x, start.1 + 28),
            TextStyle::from(("sans-serif", 22).into_font())
                .color(&color)
                .pos(Pos::new(HPos::Center, VPos::Center)),
        ))?;
        root.draw(&Text::new(
            *title,
            (center_x, start.1 + 78),
            TextStyle::from(("sans-serif", 28).into_font())
                .color(&BLACK)
                .pos(Pos::new(HPos::Center, VPos::Center)),
        ))?;
        root.draw(&Text::new(
            *subtitle,
            (center_x, start.1 + 128),
            TextStyle::from(("sans-serif", 20).into_font())
                .color(&SLATE)
                .pos(Pos::new(HPos::Center, VPos::Center)),
        ))?;
        if index + 1 < boxes.len() {
            let arrow_y = (start.1 + end.1) / 2;
            let arrow_x0 = end.0 + 18;
            let arrow_x1 = boxes[index + 1].0 .0 - 18;
            root.draw(&PathElement::new(
                vec![(arrow_x0, arrow_y), (arrow_x1, arrow_y)],
                BLACK.stroke_width(3),
            ))?;
            root.draw(&PathElement::new(
                vec![
                    (arrow_x1 - 16, arrow_y - 10),
                    (arrow_x1, arrow_y),
                    (arrow_x1 - 16, arrow_y + 10),
                ],
                BLACK.stroke_width(3),
            ))?;
        }
    }
    root.draw(&Text::new(
        "Each layer is deterministic and auditable.",
        (800, 560),
        TextStyle::from(("sans-serif", 28).into_font())
            .color(&SLATE)
            .pos(Pos::new(HPos::Center, VPos::Center)),
    ))?;
    root.draw(&Text::new(
        "Identical inputs yield identical intermediate objects, grammar decisions, and semantic outputs.",
        (800, 605),
        TextStyle::from(("sans-serif", 24).into_font())
            .color(&SLATE)
            .pos(Pos::new(HPos::Center, VPos::Center)),
    ))?;
    root.present()?;
    Ok(())
}

fn figure_coordinated_group<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    scenario: &ScenarioOutput,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let Some(coordinated) = scenario.coordinated.as_ref() else {
        root.draw(&Text::new(
            "Coordinated / grouped structure not configured for this run",
            (640, 220),
            TextStyle::from(("sans-serif", 30).into_font())
                .color(&BLACK)
                .pos(Pos::new(HPos::Center, VPos::Center)),
        ))?;
        root.draw(&Text::new(
            "Figure 11 is populated with local-vs-aggregate envelopes only when a grouped residual scenario is present.",
            (640, 290),
            TextStyle::from(("sans-serif", 20).into_font())
                .color(&SLATE)
                .pos(Pos::new(HPos::Center, VPos::Center)),
        ))?;
        root.present()?;
        return Ok(());
    };
    let areas = root.split_evenly((2, 1));
    let times = times(scenario);

    let local0 = series_channel(&scenario.residual.samples, 0)
        .into_iter()
        .map(f64::abs)
        .collect::<Vec<_>>();
    let local1 = series_channel(&scenario.residual.samples, 1)
        .into_iter()
        .map(f64::abs)
        .collect::<Vec<_>>();
    let local2 = series_channel(&scenario.residual.samples, 2)
        .into_iter()
        .map(f64::abs)
        .collect::<Vec<_>>();
    let local_env = scenario
        .envelope
        .samples
        .iter()
        .map(|sample| sample.radius)
        .collect::<Vec<_>>();
    let aggregate = coordinated
        .points
        .iter()
        .map(|point| point.aggregate_abs_mean)
        .collect::<Vec<_>>();
    let aggregate_env = coordinated
        .points
        .iter()
        .map(|point| point.aggregate_radius)
        .collect::<Vec<_>>();

    draw_multi_series(
        &areas[0],
        "Local Channel Absolute Residuals",
        &times,
        &[
            ("ch1", local0, BLUE),
            ("ch2", local1, GREEN),
            ("ch3", local2, TEAL),
            ("local envelope", local_env, SLATE),
        ],
        "local |r_i(t)|",
    )?;
    draw_multi_series(
        &areas[1],
        "Aggregate Group Residual and Envelope",
        &times,
        &[
            ("aggregate abs mean", aggregate, RED),
            ("aggregate envelope", aggregate_env, SLATE),
        ],
        "aggregate metric",
    )?;
    root.present()?;
    Ok(())
}

fn figure_semantic_retrieval<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    bundle: &EngineOutputBundle,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let areas = root.split_evenly((3, 1));
    let representatives = representative_scenarios(
        bundle,
        &["gradual_degradation", "abrupt_event", "nominal_stable"],
        3,
    );
    let first = representatives
        .first()
        .copied()
        .context("missing representative scenario")?;
    let second = representatives.get(1).copied().unwrap_or(first);
    let third = representatives.get(2).copied().unwrap_or(second);

    let scores = vec![
        (
            first.record.id.as_str(),
            first
                .semantics
                .candidates
                .first()
                .map(|c| c.score)
                .unwrap_or(0.0),
            BLUE,
        ),
        (
            second.record.id.as_str(),
            second
                .semantics
                .candidates
                .first()
                .map(|c| c.score)
                .unwrap_or(0.0),
            RED,
        ),
        (
            third.record.id.as_str(),
            third
                .semantics
                .candidates
                .first()
                .map(|c| c.score)
                .unwrap_or(0.0),
            SLATE,
        ),
    ];
    draw_score_bars(&areas[0], "Observed Motif Score", &scores)?;

    let grammar_counts = vec![
        (
            first.record.id.as_str(),
            boundary_or_violation_count(first) as f64,
            GOLD,
        ),
        (
            second.record.id.as_str(),
            boundary_or_violation_count(second) as f64,
            GOLD,
        ),
        (
            third.record.id.as_str(),
            boundary_or_violation_count(third) as f64,
            GOLD,
        ),
    ];
    draw_score_bars(&areas[1], "Admissibility Filter Count", &grammar_counts)?;

    let disposition_values = vec![
        (
            first.record.id.as_str(),
            disposition_value(&first.semantics),
            BLUE,
        ),
        (
            second.record.id.as_str(),
            disposition_value(&second.semantics),
            RED,
        ),
        (
            third.record.id.as_str(),
            disposition_value(&third.semantics),
            SLATE,
        ),
    ];
    draw_score_bars(&areas[2], "Retrieval Outcome Score", &disposition_values)?;
    root.present()?;
    Ok(())
}

fn draw_norm_vs_envelope<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    scenario: &ScenarioOutput,
    title: &str,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let times = times(scenario);
    let residual_norm = scenario
        .residual
        .samples
        .iter()
        .map(|sample| sample.norm)
        .collect::<Vec<_>>();
    let envelope = scenario
        .envelope
        .samples
        .iter()
        .map(|sample| sample.radius)
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&times);
    let (_, y_max) = combined_bounds(&[residual_norm.clone(), envelope.clone()]);
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 30))
        .margin(24)
        .x_label_area_size(40)
        .y_label_area_size(56)
        .build_cartesian_2d(x_min..x_max, 0.0..(y_max * 1.15))?;
    chart
        .configure_mesh()
        .x_desc("time")
        .y_desc("norm / envelope radius")
        .draw()?;
    chart.draw_series(LineSeries::new(
        times.iter().copied().zip(envelope.iter().copied()),
        &SLATE,
    ))?;
    chart.draw_series(LineSeries::new(
        times.iter().copied().zip(residual_norm.iter().copied()),
        &RED,
    ))?;
    if let Some(exit_time) = scenario.detectability.observed_crossing_time {
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(exit_time, 0.0), (exit_time, y_max * 1.10)],
            BLACK.mix(0.6).stroke_width(2),
        )))?;
    }
    Ok(())
}

fn draw_single_series<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    title: &str,
    times: &[f64],
    values: &[f64],
    y_label: &str,
    color: &RGBColor,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let (x_min, x_max) = bounds(times);
    let (y_min, y_max) = bounds(values);
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 26))
        .margin(18)
        .x_label_area_size(34)
        .y_label_area_size(54)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_desc("time")
        .y_desc(y_label)
        .draw()?;
    chart.draw_series(LineSeries::new(
        times.iter().copied().zip(values.iter().copied()),
        color,
    ))?;
    Ok(())
}

fn draw_multi_series<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    title: &str,
    times: &[f64],
    series: &[(&str, Vec<f64>, RGBColor)],
    y_label: &str,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let (x_min, x_max) = bounds(times);
    let (_, y_max) = combined_bounds(
        &series
            .iter()
            .map(|(_, values, _)| values.clone())
            .collect::<Vec<_>>(),
    );
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 26))
        .margin(18)
        .x_label_area_size(34)
        .y_label_area_size(54)
        .build_cartesian_2d(x_min..x_max, 0.0..(y_max * 1.15).max(0.1))?;
    chart
        .configure_mesh()
        .x_desc("time")
        .y_desc(y_label)
        .draw()?;
    for (label, values, color) in series {
        chart
            .draw_series(LineSeries::new(
                times.iter().copied().zip(values.iter().copied()),
                color,
            ))?
            .label(*label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], color.stroke_width(3)));
    }
    chart.configure_series_labels().border_style(BLACK).draw()?;
    Ok(())
}

fn draw_score_bars<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    title: &str,
    values: &[(&str, f64, RGBColor)],
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 24))
        .margin(18)
        .x_label_area_size(44)
        .y_label_area_size(56)
        .build_cartesian_2d(
            0.0_f64..values.len().max(1) as f64,
            0.0_f64
                ..(values
                    .iter()
                    .map(|(_, value, _)| *value)
                    .fold(0.0, f64::max)
                    * 1.15)
                    .max(1.0),
        )?;
    chart
        .configure_mesh()
        .x_desc("representative scenario")
        .y_desc("score")
        .draw()?;
    for (index, (label, value, color)) in values.iter().enumerate() {
        let x0 = index as f64 + 0.18;
        let x1 = index as f64 + 0.64;
        chart.draw_series(std::iter::once(Rectangle::new(
            [(x0, 0.0), (x1, *value)],
            color.filled(),
        )))?;
        chart.draw_series(std::iter::once(Text::new(
            (*label).to_string(),
            (index as f64 + 0.10, 0.8),
            ("sans-serif", 14).into_font().color(&BLACK),
        )))?;
    }
    Ok(())
}

fn render_01(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let scenario = scenario_or_first(bundle, "gradual_degradation")?;
    let figure_id = "figure_01_residual_prediction_observation_overview";
    let caption = "Residual, observation, and prediction overview for the gradual degradation case. Synthetic deterministic demonstration only.";
    let size = (1280, 840);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_observation_overview(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        scenario,
    )?;
    figure_observation_overview(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        scenario,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_02(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let scenario = scenario_or_first(bundle, "abrupt_event")?;
    let figure_id = "figure_02_drift_and_slew_decomposition";
    let caption = "Residual norm, signed aggregate drift, and slew norm decomposition for a representative case. Synthetic deterministic demonstration only when the bundled scenario suite is used.";
    let size = (1280, 960);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_drift_slew(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        scenario,
    )?;
    figure_drift_slew(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        scenario,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_03(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let scenario = scenario_or_first(bundle, "curvature_onset")?;
    let figure_id = "figure_03_sign_space_projection";
    let caption = "Projected sign trajectory using the deterministic aggregate coordinates [||r||, signed aggregate drift, ||s||]. Synthetic deterministic demonstration only when the bundled scenario suite is used.";
    let size = (1280, 720);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_sign_space(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        scenario,
    )?;
    figure_sign_space(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        scenario,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_04(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let (monotone, curvature) =
        scenario_pair_or_first(bundle, "gradual_degradation", "curvature_onset")?;
    let figure_id = "figure_04_syntax_comparison";
    let caption = "Syntax comparison between monotone drift and curvature-dominated trajectories. Synthetic deterministic demonstration only.";
    let size = (1280, 720);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_syntax_comparison(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        monotone,
        curvature,
    )?;
    figure_syntax_comparison(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        monotone,
        curvature,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_05(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let scenario = scenario_or_first(bundle, "outward_exit_case_a")?;
    let figure_id = "figure_05_envelope_exit_under_sustained_outward_drift";
    let caption = "Residual norm and admissibility envelope for the sustained outward-drift exit case. Synthetic theorem-aligned demonstration only.";
    let size = (1280, 720);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_envelope_exit(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        scenario,
    )?;
    figure_envelope_exit(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        scenario,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_06(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let scenario = scenario_or_first(bundle, "inward_invariance")?;
    let figure_id = "figure_06_envelope_invariance_under_inward_drift";
    let caption = "Residual norm and admissibility envelope for the inward-compatible invariance case. Synthetic theorem-aligned demonstration only.";
    let size = (1280, 720);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_envelope_invariance(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        scenario,
    )?;
    figure_envelope_invariance(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        scenario,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_07(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let (exit_case, invariance_case) =
        scenario_pair_or_first(bundle, "outward_exit_case_a", "inward_invariance")?;
    let figure_id = "figure_07_exit_invariance_pair_common_envelope";
    let caption = "Exit-invariance pair under a common visualization envelope, contrasting outward drift with inward-compatible containment. Synthetic theorem-aligned demonstration only.";
    let size = (1280, 720);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_exit_invariance_pair(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        exit_case,
        invariance_case,
    )?;
    figure_exit_invariance_pair(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        exit_case,
        invariance_case,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_08(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let (admissible, detectable) = scenario_pair_or_first(
        bundle,
        "magnitude_matched_admissible",
        "magnitude_matched_detectable",
    )?;
    let figure_id = "figure_08_residual_trajectory_separation";
    let caption = "Residual trajectory separation between magnitude-matched admissible and detectable cases. Synthetic theorem-aligned demonstration only.";
    let size = (1280, 720);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_residual_separation(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        admissible,
        detectable,
    )?;
    figure_residual_separation(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        admissible,
        detectable,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_09(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_09_detectability_bound_comparison";
    let caption = "Predicted residual-envelope detectability bounds versus observed envelope-crossing times across multiple synthetic cases.";
    let size = (1280, 720);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_detectability_bounds(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        bundle,
    )?;
    figure_detectability_bounds(SVGBackend::new(&svg_path, size).into_drawing_area(), bundle)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_10(figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_10_deterministic_pipeline_flow";
    let caption = "Deterministic layered engine flow showing residual extraction, sign construction, syntax, grammar, and semantic retrieval as auditable maps.";
    let size = (1600, 900);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_pipeline_flow(BitMapBackend::new(&png_path, size).into_drawing_area())?;
    figure_pipeline_flow(SVGBackend::new(&svg_path, size).into_drawing_area())?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_11(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let scenario = scenario_or_first(bundle, "grouped_correlated")?;
    let figure_id = "figure_11_coordinated_group_semiotics";
    let caption = "Local versus aggregate envelopes for the grouped correlated case. Synthetic deterministic demonstration only.";
    let size = (1280, 840);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_coordinated_group(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        scenario,
    )?;
    figure_coordinated_group(
        SVGBackend::new(&svg_path, size).into_drawing_area(),
        scenario,
    )?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_12(bundle: &EngineOutputBundle, figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_12_semantic_retrieval_heuristics_bank";
    let caption = "Constrained semantic retrieval summary across representative motifs, including matched, ambiguous, and unknown outcomes. Synthetic deterministic demonstration only.";
    let size = (1280, 860);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_semantic_retrieval(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        bundle,
    )?;
    figure_semantic_retrieval(SVGBackend::new(&svg_path, size).into_drawing_area(), bundle)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn scenario_or_first<'a>(bundle: &'a EngineOutputBundle, id: &str) -> Result<&'a ScenarioOutput> {
    bundle
        .scenario_outputs
        .iter()
        .find(|scenario| scenario.record.id == id)
        .or_else(|| bundle.scenario_outputs.first())
        .context("missing scenario for figure rendering")
}

fn scenario_pair_or_first<'a>(
    bundle: &'a EngineOutputBundle,
    first_id: &str,
    second_id: &str,
) -> Result<(&'a ScenarioOutput, &'a ScenarioOutput)> {
    let first = scenario_or_first(bundle, first_id)?;
    let second = bundle
        .scenario_outputs
        .iter()
        .find(|scenario| scenario.record.id == second_id)
        .or_else(|| {
            bundle
                .scenario_outputs
                .iter()
                .find(|scenario| scenario.record.id != first.record.id)
        })
        .unwrap_or(first);
    Ok((first, second))
}

fn representative_scenarios<'a>(
    bundle: &'a EngineOutputBundle,
    preferred_ids: &[&str],
    count: usize,
) -> Vec<&'a ScenarioOutput> {
    let mut seen = std::collections::BTreeSet::new();
    let mut selected = Vec::new();
    for id in preferred_ids {
        if let Some(scenario) = bundle
            .scenario_outputs
            .iter()
            .find(|scenario| scenario.record.id == *id)
        {
            if seen.insert(scenario.record.id.clone()) {
                selected.push(scenario);
            }
        }
    }
    for scenario in &bundle.scenario_outputs {
        if selected.len() >= count {
            break;
        }
        if seen.insert(scenario.record.id.clone()) {
            selected.push(scenario);
        }
    }
    selected
}

fn times(scenario: &ScenarioOutput) -> Vec<f64> {
    scenario
        .observed
        .samples
        .iter()
        .map(|sample| sample.time)
        .collect()
}

fn series_channel<T>(samples: &[T], channel: usize) -> Vec<f64>
where
    T: SampleValues,
{
    samples
        .iter()
        .map(|sample| sample.values().get(channel).copied().unwrap_or_default())
        .collect()
}

trait SampleValues {
    fn values(&self) -> &[f64];
}

impl SampleValues for crate::engine::types::VectorSample {
    fn values(&self) -> &[f64] {
        &self.values
    }
}

impl SampleValues for crate::engine::types::ResidualSample {
    fn values(&self) -> &[f64] {
        &self.values
    }
}

impl SampleValues for crate::engine::types::DriftSample {
    fn values(&self) -> &[f64] {
        &self.values
    }
}

impl SampleValues for crate::engine::types::SlewSample {
    fn values(&self) -> &[f64] {
        &self.values
    }
}

fn bounds(values: &[f64]) -> (f64, f64) {
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !min.is_finite() || !max.is_finite() || (max - min).abs() < 1.0e-9 {
        (min.min(0.0) - 0.1, max.max(0.0) + 0.1)
    } else {
        let margin = (max - min) * 0.08;
        (min - margin, max + margin)
    }
}

fn combined_bounds(series: &[Vec<f64>]) -> (f64, f64) {
    let mut values = Vec::new();
    for sequence in series {
        values.extend(sequence.iter().copied());
    }
    bounds(&values)
}

fn boundary_or_violation_count(scenario: &ScenarioOutput) -> usize {
    scenario
        .grammar
        .iter()
        .filter(|status| !matches!(status.state, GrammarState::Admissible))
        .count()
}

fn disposition_value(result: &crate::engine::types::SemanticMatchResult) -> f64 {
    match result.disposition {
        crate::engine::types::SemanticDisposition::Match => 1.0,
        crate::engine::types::SemanticDisposition::CompatibleSet => 0.8,
        crate::engine::types::SemanticDisposition::Ambiguous => 0.6,
        crate::engine::types::SemanticDisposition::Unknown => 0.2,
    }
}

fn artifact(
    figure_id: &str,
    caption: &str,
    png_path: std::path::PathBuf,
    svg_path: std::path::PathBuf,
) -> FigureArtifact {
    FigureArtifact {
        figure_id: figure_id.to_string(),
        caption: caption.to_string(),
        png_path: png_path.display().to_string(),
        svg_path: svg_path.display().to_string(),
    }
}
