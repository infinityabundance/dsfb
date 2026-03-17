use std::path::Path;

use anyhow::{Context, Result};
use plotters::prelude::*;

use crate::report::manifest::{BenchmarkRow, HeroBenchmarkRow};
use crate::sim::runner::{ScenarioRun, TopologySnapshot};

const COLORS: [RGBColor; 6] = [RED, BLUE, GREEN, MAGENTA, CYAN, BLACK];

pub fn render_figures(
    figures_dir: &Path,
    scenarios: &[ScenarioRun],
    benchmark_rows: &[BenchmarkRow],
    hero_rows: &[HeroBenchmarkRow],
) -> Result<()> {
    std::fs::create_dir_all(figures_dir)
        .with_context(|| format!("failed to create {}", figures_dir.display()))?;
    render_lambda2_timeseries(&figures_dir.join("lambda2_timeseries.png"), scenarios)?;
    render_residual_timeseries(&figures_dir.join("residual_timeseries.png"), scenarios)?;
    render_drift_slew(&figures_dir.join("drift_slew.png"), scenarios)?;
    render_trust_evolution(&figures_dir.join("trust_evolution.png"), scenarios)?;
    render_baseline_comparison(&figures_dir.join("baseline_comparison.png"), benchmark_rows)?;
    render_scaling_curves(&figures_dir.join("scaling_curves.png"), benchmark_rows)?;
    render_noise_stress_curves(&figures_dir.join("noise_stress_curves.png"), benchmark_rows)?;
    render_multimode_comparison(&figures_dir.join("multimode_comparison.png"), benchmark_rows)?;
    render_topology_snapshots(&figures_dir.join("topology_snapshots.png"), scenarios)?;
    render_hero_leadtime_comparison(&figures_dir.join("hero_leadtime_comparison.png"), hero_rows)?;
    render_hero_benchmark_table(&figures_dir.join("hero_benchmark_table.png"), hero_rows)?;
    Ok(())
}

fn render_lambda2_timeseries(path: &Path, scenarios: &[ScenarioRun]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let max_time = scenarios
        .iter()
        .flat_map(|scenario| scenario.time_series.iter().map(|row| row.time))
        .fold(1.0_f64, f64::max);
    let max_lambda2 = scenarios
        .iter()
        .flat_map(|scenario| scenario.time_series.iter().map(|row| row.lambda2))
        .fold(1.0_f64, f64::max)
        .max(0.2);

    let mut chart = ChartBuilder::on(&root)
        .caption("Algebraic connectivity lambda_2(t) under structural scenarios", ("sans-serif", 28))
        .margin(16)
        .x_label_area_size(48)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_time, 0.0..(max_lambda2 * 1.05))?;
    chart.configure_mesh().x_desc("time").y_desc("lambda_2(t)").draw()?;

    for (index, scenario) in scenarios.iter().enumerate() {
        let color = COLORS[index % COLORS.len()];
        chart
            .draw_series(LineSeries::new(
                scenario.time_series.iter().map(|row| (row.time, row.lambda2)),
                &color,
            ))?
            .label(scenario.definition.name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 24, y)], color));
    }
    chart.configure_series_labels().border_style(BLACK).draw()?;
    Ok(())
}

fn render_residual_timeseries(path: &Path, scenarios: &[ScenarioRun]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((2, 1));
    let focus = focus_scenario(scenarios);
    let upper = &areas[0];
    let lower = &areas[1];
    let max_time = focus.time_series.last().map(|row| row.time).unwrap_or(1.0);
    let max_y = focus
        .time_series
        .iter()
        .map(|row| row.lambda2.max(row.predicted_lambda2))
        .fold(1.0_f64, f64::max)
        .max(0.2);

    let mut upper_chart = ChartBuilder::on(upper)
        .caption(
            format!("Observed vs predicted lambda_2(t): {}", focus.definition.name),
            ("sans-serif", 24),
        )
        .margin(12)
        .x_label_area_size(32)
        .y_label_area_size(56)
        .build_cartesian_2d(0.0..max_time, 0.0..(max_y * 1.05))?;
    upper_chart.configure_mesh().x_desc("time").y_desc("lambda_2").draw()?;
    upper_chart.draw_series(LineSeries::new(
        focus.time_series.iter().map(|row| (row.time, row.lambda2)),
        &BLUE,
    ))?;
    upper_chart.draw_series(LineSeries::new(
        focus.time_series.iter().map(|row| (row.time, row.predicted_lambda2)),
        &RED,
    ))?;

    let residual_limit = focus
        .time_series
        .iter()
        .map(|row| row.scalar_signal.max(row.scalar_signal_limit))
        .fold(0.2_f64, f64::max);
    let mut lower_chart = ChartBuilder::on(lower)
        .caption("Negative residual detector signal and calibrated limit", ("sans-serif", 24))
        .margin(12)
        .x_label_area_size(42)
        .y_label_area_size(56)
        .build_cartesian_2d(0.0..max_time, 0.0..(residual_limit * 1.05))?;
    lower_chart
        .configure_mesh()
        .x_desc("time")
        .y_desc("max(-r_lambda2(t), 0)")
        .draw()?;
    lower_chart.draw_series(LineSeries::new(
        focus.time_series.iter().map(|row| (row.time, row.scalar_signal)),
        &MAGENTA,
    ))?;
    lower_chart.draw_series(LineSeries::new(
        focus.time_series
            .iter()
            .map(|row| (row.time, row.scalar_signal_limit)),
        &BLACK,
    ))?;
    Ok(())
}

fn render_drift_slew(path: &Path, scenarios: &[ScenarioRun]) -> Result<()> {
    let focus = focus_scenario(scenarios);
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((2, 1));
    let max_time = focus.time_series.last().map(|row| row.time).unwrap_or(1.0);
    let drift_limit = focus
        .time_series
        .iter()
        .map(|row| (-row.scalar_drift).max(0.0).max(row.scalar_drift_limit))
        .fold(0.2_f64, f64::max);
    let slew_limit = focus
        .time_series
        .iter()
        .map(|row| row.scalar_slew.abs())
        .fold(0.2_f64, f64::max);

    let mut drift_chart = ChartBuilder::on(&areas[0])
        .caption(
            format!("Residual drift diagnostic: {}", focus.definition.name),
            ("sans-serif", 24),
        )
        .margin(12)
        .x_label_area_size(32)
        .y_label_area_size(56)
        .build_cartesian_2d(0.0..max_time, 0.0..(drift_limit * 1.05))?;
    drift_chart
        .configure_mesh()
        .x_desc("time")
        .y_desc("max(-dot r_lambda2(t), 0)")
        .draw()?;
    drift_chart.draw_series(LineSeries::new(
        focus.time_series
            .iter()
            .map(|row| (row.time, (-row.scalar_drift).max(0.0))),
        &BLUE,
    ))?;
    drift_chart.draw_series(LineSeries::new(
        focus.time_series
            .iter()
            .map(|row| (row.time, row.scalar_drift_limit)),
        &BLACK,
    ))?;

    let mut slew_chart = ChartBuilder::on(&areas[1])
        .caption("Residual slew diagnostic", ("sans-serif", 24))
        .margin(12)
        .x_label_area_size(42)
        .y_label_area_size(56)
        .build_cartesian_2d(0.0..max_time, -slew_limit..slew_limit)?;
    slew_chart.configure_mesh().x_desc("time").y_desc("ddot r_lambda2(t)").draw()?;
    slew_chart.draw_series(LineSeries::new(
        focus.time_series.iter().map(|row| (row.time, row.scalar_slew)),
        &RED,
    ))?;
    Ok(())
}

fn render_trust_evolution(path: &Path, scenarios: &[ScenarioRun]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let max_time = scenarios
        .iter()
        .flat_map(|scenario| scenario.time_series.iter().map(|row| row.time))
        .fold(1.0_f64, f64::max);
    let mut chart = ChartBuilder::on(&root)
        .caption("Trust-gated attenuation of effective interactions", ("sans-serif", 28))
        .margin(16)
        .x_label_area_size(48)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_time, 0.0..1.05)?;
    chart.configure_mesh().x_desc("time").y_desc("affected-set trust").draw()?;
    for (index, scenario) in scenarios.iter().enumerate() {
        let color = COLORS[index % COLORS.len()];
        chart
            .draw_series(LineSeries::new(
                scenario
                    .time_series
                    .iter()
                    .map(|row| (row.time, row.affected_mean_trust)),
                &color,
            ))?
            .label(scenario.definition.name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 24, y)], color));
    }
    chart.configure_series_labels().border_style(BLACK).draw()?;
    Ok(())
}

fn render_baseline_comparison(path: &Path, benchmark_rows: &[BenchmarkRow]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((1, 2));
    let hero_scenarios = ["gradual_edge_degradation", "communication_loss"];
    let max_lead = benchmark_rows
        .iter()
        .filter(|row| hero_scenarios.contains(&row.scenario.as_str()))
        .flat_map(|row| {
            [
                row.scalar_detection_lead_time.unwrap_or(0.0),
                row.multimode_detection_lead_time.unwrap_or(0.0),
                row.best_baseline_lead_time.unwrap_or(0.0),
            ]
        })
        .fold(0.5_f64, f64::max)
        * 1.18;
    let mut lead_chart = ChartBuilder::on(&areas[0])
        .caption("DSFB lead time versus best baseline", ("sans-serif", 24))
        .margin(16)
        .x_label_area_size(64)
        .y_label_area_size(60)
        .build_cartesian_2d(0..(hero_scenarios.len() * 3).max(3), -0.25..max_lead)?;
    lead_chart
        .configure_mesh()
        .y_desc("lead time (s)")
        .x_labels(hero_scenarios.len() * 3)
        .x_label_formatter(&|index| {
            let scenario_index = index / 3;
            let detector = match index % 3 {
                0 => "baseline",
                1 => "scalar",
                _ => "multi",
            };
            format!(
                "{}-{}",
                hero_scenarios.get(scenario_index).copied().unwrap_or("n/a"),
                detector
            )
        })
        .draw()?;
    for (scenario_index, scenario) in hero_scenarios.iter().enumerate() {
        let rows = benchmark_rows
            .iter()
            .filter(|row| row.scenario == *scenario)
            .collect::<Vec<_>>();
        let baseline = average_option(rows.iter().filter_map(|row| row.best_baseline_lead_time));
        let scalar = average_option(rows.iter().filter_map(|row| row.scalar_detection_lead_time));
        let multi = average_option(rows.iter().filter_map(|row| row.multimode_detection_lead_time));
        let base = scenario_index * 3;
        lead_chart.draw_series([
            Rectangle::new([(base, 0.0), (base + 1, baseline)], GREEN.filled()),
            Rectangle::new([(base + 1, 0.0), (base + 2, scalar)], BLUE.filled()),
            Rectangle::new([(base + 2, 0.0), (base + 3, multi)], RED.filled()),
        ])?;
    }

    let max_gain = benchmark_rows
        .iter()
        .filter(|row| hero_scenarios.contains(&row.scenario.as_str()))
        .filter_map(|row| row.lead_time_gain_vs_best_baseline)
        .map(f64::abs)
        .fold(0.4_f64, f64::max)
        * 1.18;
    let mut gain_chart = ChartBuilder::on(&areas[1])
        .caption("Average DSFB gain over best baseline", ("sans-serif", 24))
        .margin(16)
        .x_label_area_size(56)
        .y_label_area_size(60)
        .build_cartesian_2d(0..hero_scenarios.len().max(1), -max_gain..max_gain)?;
    gain_chart
        .configure_mesh()
        .y_desc("lead-time gain (s)")
        .x_labels(hero_scenarios.len().max(1))
        .x_label_formatter(&|index| hero_scenarios.get(*index).copied().unwrap_or("").to_string())
        .draw()?;
    gain_chart.draw_series(hero_scenarios.iter().enumerate().map(|(index, scenario)| {
        let rows = benchmark_rows.iter().filter(|row| row.scenario == *scenario);
        let value = average_option(rows.filter_map(|row| row.lead_time_gain_vs_best_baseline));
        Rectangle::new(
            [(index, 0.0), (index + 1, value)],
            if value >= 0.0 { MAGENTA.filled() } else { BLACK.filled() },
        )
    }))?;
    Ok(())
}

fn render_scaling_curves(path: &Path, benchmark_rows: &[BenchmarkRow]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let max_agents = benchmark_rows.iter().map(|row| row.agents as f64).fold(1.0_f64, f64::max);
    let max_runtime = benchmark_rows.iter().map(|row| row.runtime_ms).fold(1.0_f64, f64::max);
    let mut chart = ChartBuilder::on(&root)
        .caption("Scaling cost versus swarm size", ("sans-serif", 28))
        .margin(16)
        .x_label_area_size(48)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_agents, 0.0..(max_runtime * 1.1).max(1.0))?;
    chart.configure_mesh().x_desc("agents").y_desc("runtime (ms)").draw()?;
    let scenario_names = unique_scenario_names(benchmark_rows);
    for (index, name) in scenario_names.iter().enumerate() {
        let color = COLORS[index % COLORS.len()];
        let series = benchmark_rows
            .iter()
            .filter(|row| row.scenario == *name)
            .map(|row| (row.agents as f64, row.runtime_ms));
        chart
            .draw_series(LineSeries::new(series, &color))?
            .label(name.clone())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 24, y)], color));
    }
    chart.configure_series_labels().border_style(BLACK).draw()?;
    Ok(())
}

fn render_noise_stress_curves(path: &Path, benchmark_rows: &[BenchmarkRow]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((1, 2));
    let max_noise = benchmark_rows
        .iter()
        .map(|row| row.noise_level)
        .fold(0.2_f64, f64::max)
        .max(0.05);

    let mut left = ChartBuilder::on(&areas[0])
        .caption("TPR versus noise level", ("sans-serif", 24))
        .margin(12)
        .x_label_area_size(40)
        .y_label_area_size(56)
        .build_cartesian_2d(0.0..max_noise, 0.0..1.05)?;
    left.configure_mesh().x_desc("noise").y_desc("TPR").draw()?;
    left.draw_series(LineSeries::new(
        aggregate_by_noise(benchmark_rows, |row| row.multimode_true_positive_rate),
        &BLUE,
    ))?;
    left.draw_series(LineSeries::new(
        aggregate_by_noise(benchmark_rows, |row| row.scalar_true_positive_rate),
        &RED,
    ))?;

    let mut right = ChartBuilder::on(&areas[1])
        .caption("FPR versus noise level", ("sans-serif", 24))
        .margin(12)
        .x_label_area_size(40)
        .y_label_area_size(56)
        .build_cartesian_2d(0.0..max_noise, 0.0..1.05)?;
    right.configure_mesh().x_desc("noise").y_desc("FPR").draw()?;
    right.draw_series(LineSeries::new(
        aggregate_by_noise(benchmark_rows, |row| row.multimode_false_positive_rate),
        &BLUE,
    ))?;
    right.draw_series(LineSeries::new(
        aggregate_by_noise(benchmark_rows, |row| row.scalar_false_positive_rate),
        &RED,
    ))?;
    Ok(())
}

fn render_multimode_comparison(path: &Path, benchmark_rows: &[BenchmarkRow]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((1, 2));
    let scenario_names = unique_scenario_names(benchmark_rows);
    let max_lead = max_multimode_value(benchmark_rows);
    let mut lead_chart = ChartBuilder::on(&areas[0])
        .caption("Scalar vs multi-mode lead time", ("sans-serif", 24))
        .margin(16)
        .x_label_area_size(56)
        .y_label_area_size(60)
        .build_cartesian_2d(0..(scenario_names.len() * 2).max(2), 0.0..max_lead)?;
    lead_chart
        .configure_mesh()
        .y_desc("lead time (s)")
        .x_labels(scenario_names.len() * 2)
        .x_label_formatter(&|index| {
            let scenario_index = index / 2;
            let detector = if index % 2 == 0 { "scalar" } else { "multi" };
            format!("{}-{}", scenario_names.get(scenario_index).cloned().unwrap_or_default(), detector)
        })
        .draw()?;

    for (scenario_index, name) in scenario_names.iter().enumerate() {
        let rows = benchmark_rows.iter().filter(|row| row.scenario == *name).collect::<Vec<_>>();
        let scalar = average_option(rows.iter().filter_map(|row| row.scalar_detection_lead_time));
        let multi = average_option(rows.iter().filter_map(|row| row.multimode_detection_lead_time));
        lead_chart.draw_series([
            Rectangle::new(
                [(scenario_index * 2, 0.0), (scenario_index * 2 + 1, scalar)],
                BLUE.filled(),
            ),
            Rectangle::new(
                [(scenario_index * 2 + 1, 0.0), (scenario_index * 2 + 2, multi)],
                RED.filled(),
            ),
        ])?;
    }

    let delta_limit = benchmark_rows
        .iter()
        .filter_map(|row| row.multimode_minus_scalar_seconds)
        .map(f64::abs)
        .fold(0.5_f64, f64::max)
        * 1.1;
    let mut delta_chart = ChartBuilder::on(&areas[1])
        .caption("Multi-mode detection advantage", ("sans-serif", 24))
        .margin(16)
        .x_label_area_size(56)
        .y_label_area_size(60)
        .build_cartesian_2d(0..scenario_names.len().max(1), -delta_limit..delta_limit)?;
    delta_chart
        .configure_mesh()
        .y_desc("scalar_detection_step - multimode_detection_step (s)")
        .x_labels(scenario_names.len().max(1))
        .x_label_formatter(&|index| scenario_names.get(*index).cloned().unwrap_or_default())
        .draw()?;
    delta_chart.draw_series(scenario_names.iter().enumerate().map(|(index, name)| {
        let rows = benchmark_rows.iter().filter(|row| row.scenario == *name);
        let value = average_option(rows.filter_map(|row| row.multimode_minus_scalar_seconds));
        Rectangle::new(
            [(index, 0.0), (index + 1, value)],
            if value >= 0.0 { GREEN.filled() } else { RED.filled() },
        )
    }))?;
    Ok(())
}

fn render_topology_snapshots(path: &Path, scenarios: &[ScenarioRun]) -> Result<()> {
    let focus = focus_scenario_with_snapshots(scenarios);
    let root = BitMapBackend::new(path, (1440, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((1, 3));
    for (panel, snapshot) in areas.iter().zip(focus.topology_snapshots.iter().take(3)) {
        draw_snapshot(panel, snapshot)?;
    }
    Ok(())
}

fn render_hero_leadtime_comparison(path: &Path, hero_rows: &[HeroBenchmarkRow]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let max_value = hero_rows
        .iter()
        .flat_map(|row| {
            [
                row.scalar_lead_time.unwrap_or(0.0),
                row.multimode_lead_time.unwrap_or(0.0),
                row.best_baseline_lead_time.unwrap_or(0.0),
            ]
        })
        .fold(0.5_f64, f64::max)
        * 1.15;
    let mut chart = ChartBuilder::on(&root)
        .caption("Hero lead-time comparison across calibrated scenarios", ("sans-serif", 28))
        .margin(16)
        .x_label_area_size(64)
        .y_label_area_size(64)
        .build_cartesian_2d(0..(hero_rows.len() * 3).max(3), 0.0..max_value)?;
    chart
        .configure_mesh()
        .y_desc("lead time (s)")
        .x_labels((hero_rows.len() * 3).max(3))
        .x_label_formatter(&|index| {
            let scenario_index = index / 3;
            let label = match index % 3 {
                0 => "scalar",
                1 => "multi",
                _ => "baseline",
            };
            format!(
                "{}-{}",
                hero_rows
                    .get(scenario_index)
                    .map(|row| row.scenario.as_str())
                    .unwrap_or("n/a"),
                label
            )
        })
        .draw()?;

    for (scenario_index, row) in hero_rows.iter().enumerate() {
        let base = scenario_index * 3;
        chart.draw_series([
            Rectangle::new(
                [(base, 0.0), (base + 1, row.scalar_lead_time.unwrap_or(0.0))],
                BLUE.filled(),
            ),
            Rectangle::new(
                [(base + 1, 0.0), (base + 2, row.multimode_lead_time.unwrap_or(0.0))],
                RED.filled(),
            ),
            Rectangle::new(
                [(base + 2, 0.0), (base + 3, row.best_baseline_lead_time.unwrap_or(0.0))],
                GREEN.filled(),
            ),
        ])?;
        chart.draw_series(std::iter::once(Text::new(
            row.winner.clone(),
            (base + 1, max_value * 0.94),
            ("sans-serif", 18).into_font(),
        )))?;
    }
    Ok(())
}

fn render_hero_benchmark_table(path: &Path, hero_rows: &[HeroBenchmarkRow]) -> Result<()> {
    let root = BitMapBackend::new(path, (1700, 520)).into_drawing_area();
    root.fill(&WHITE)?;
    root.draw(&Text::new(
        "Hero benchmark summary",
        (40, 40),
        ("sans-serif", 30).into_font(),
    ))?;
    let columns = [
        ("scenario", 40),
        ("scalar LT", 260),
        ("multimode LT", 430),
        ("best baseline LT", 640),
        ("trust delay", 870),
        ("scalar TPR/FPR", 1050),
        ("multimode TPR/FPR", 1290),
        ("winner", 1540),
    ];
    for (label, x) in columns {
        root.draw(&Text::new(label, (x, 90), ("sans-serif", 20).into_font()))?;
    }
    for y in [105, 155, 215, 275, 335, 395, 455, 515] {
        root.draw(&PathElement::new(vec![(30, y), (1660, y)], BLACK.mix(0.2)))?;
    }
    for (index, row) in hero_rows.iter().enumerate() {
        let y = 140 + index as i32 * 60;
        let values = [
            row.scenario.clone(),
            display_option(row.scalar_lead_time),
            display_option(row.multimode_lead_time),
            display_option(row.best_baseline_lead_time),
            display_option(row.trust_suppression_delay),
            format!("{:.3}/{:.3}", row.scalar_true_positive_rate, row.scalar_false_positive_rate),
            format!(
                "{:.3}/{:.3}",
                row.multimode_true_positive_rate, row.multimode_false_positive_rate
            ),
            row.winner.clone(),
        ];
        let x_positions = [40, 260, 430, 640, 870, 1050, 1290, 1540];
        for (value, x) in values.iter().zip(x_positions.iter()) {
            root.draw(&Text::new(
                value.clone(),
                (*x, y),
                ("sans-serif", 18).into_font(),
            ))?;
        }
    }
    Ok(())
}

fn draw_snapshot(area: &DrawingArea<BitMapBackend<'_>, plotters::coord::Shift>, snapshot: &TopologySnapshot) -> Result<()> {
    let mut chart = ChartBuilder::on(area)
        .caption(
            format!("{}: {}", snapshot.scenario, snapshot.label),
            ("sans-serif", 20),
        )
        .margin(10)
        .x_label_area_size(24)
        .y_label_area_size(24)
        .build_cartesian_2d(-2.2..2.2, -1.8..1.8)?;
    chart.configure_mesh().disable_mesh().draw()?;
    chart.draw_series(snapshot.edges.iter().map(|edge| {
        let source = snapshot.agents[edge.source].position;
        let target = snapshot.agents[edge.target].position;
        PathElement::new(
            vec![(source.x, source.y), (target.x, target.y)],
            BLACK.mix(edge.weight.min(1.0)),
        )
    }))?;
    chart.draw_series(snapshot.agents.iter().map(|agent| {
        Circle::new((agent.position.x, agent.position.y), 4, BLUE.filled())
    }))?;
    Ok(())
}

fn focus_scenario(scenarios: &[ScenarioRun]) -> &ScenarioRun {
    scenarios
        .iter()
        .find(|scenario| scenario.definition.name == "communication_loss")
        .or_else(|| scenarios.iter().find(|scenario| scenario.definition.name == "gradual_edge_degradation"))
        .unwrap_or(&scenarios[0])
}

fn focus_scenario_with_snapshots(scenarios: &[ScenarioRun]) -> &ScenarioRun {
    scenarios
        .iter()
        .find(|scenario| scenario.topology_snapshots.len() >= 3)
        .unwrap_or(&scenarios[0])
}

fn unique_scenario_names(rows: &[BenchmarkRow]) -> Vec<String> {
    let mut names = rows.iter().map(|row| row.scenario.clone()).collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

fn aggregate_by_noise<F>(rows: &[BenchmarkRow], value: F) -> Vec<(f64, f64)>
where
    F: Fn(&BenchmarkRow) -> f64,
{
    let mut noises = rows.iter().map(|row| row.noise_level).collect::<Vec<_>>();
    noises.sort_by(f64::total_cmp);
    noises.dedup_by(|left, right| (*left - *right).abs() < 1.0e-9);
    noises
        .into_iter()
        .map(|noise| {
            let filtered = rows.iter().filter(|row| (row.noise_level - noise).abs() < 1.0e-9);
            let mean = average(filtered.map(&value));
            (noise, mean)
        })
        .collect()
}

fn average_option<I>(iter: I) -> f64
where
    I: Iterator<Item = f64>,
{
    let values = iter.collect::<Vec<_>>();
    average(values.into_iter())
}

fn average<I>(iter: I) -> f64
where
    I: Iterator<Item = f64>,
{
    let values = iter.collect::<Vec<_>>();
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn max_multimode_value(rows: &[BenchmarkRow]) -> f64 {
    rows.iter()
        .flat_map(|row| [row.scalar_detection_lead_time.unwrap_or(0.0), row.multimode_detection_lead_time.unwrap_or(0.0)])
        .fold(0.5_f64, f64::max)
        * 1.2
}

fn display_option(value: Option<f64>) -> String {
    value
        .map(|number| format!("{number:.3}"))
        .unwrap_or_else(|| "n/a".to_string())
}
