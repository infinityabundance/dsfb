use std::path::Path;

use anyhow::{Context, Result};
use plotters::prelude::*;

use crate::report::manifest::BenchmarkRow;
use crate::sim::runner::{ScenarioRun, TopologySnapshot};

const COLORS: [RGBColor; 6] = [RED, BLUE, GREEN, MAGENTA, CYAN, BLACK];

pub fn render_figures(
    figures_dir: &Path,
    scenarios: &[ScenarioRun],
    benchmark_rows: &[BenchmarkRow],
) -> Result<()> {
    std::fs::create_dir_all(figures_dir)
        .with_context(|| format!("failed to create {}", figures_dir.display()))?;
    render_lambda2_timeseries(&figures_dir.join("lambda2_timeseries.png"), scenarios)?;
    render_residual_timeseries(&figures_dir.join("residual_timeseries.png"), scenarios)?;
    render_drift_slew(&figures_dir.join("drift_slew.png"), scenarios)?;
    render_trust_evolution(&figures_dir.join("trust_evolution.png"), scenarios)?;
    render_baseline_comparison(&figures_dir.join("baseline_comparison.png"), scenarios)?;
    render_scaling_curves(&figures_dir.join("scaling_curves.png"), benchmark_rows)?;
    render_noise_stress_curves(&figures_dir.join("noise_stress_curves.png"), benchmark_rows)?;
    render_multimode_comparison(&figures_dir.join("multimode_comparison.png"), benchmark_rows)?;
    render_topology_snapshots(&figures_dir.join("topology_snapshots.png"), scenarios)?;
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

fn render_baseline_comparison(path: &Path, scenarios: &[ScenarioRun]) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let labels = ["state-norm", "disagreement", "raw-lambda2", "scalar-residual", "multi-mode"];
    let values = average_detection_metrics(scenarios);
    let max_value = values.iter().copied().fold(1.0_f64, f64::max).max(0.1);

    let mut chart = ChartBuilder::on(&root)
        .caption("Average detection lead time against baselines", ("sans-serif", 28))
        .margin(16)
        .x_label_area_size(48)
        .y_label_area_size(60)
        .build_cartesian_2d(0..labels.len(), 0.0..(max_value * 1.1))?;
    chart
        .configure_mesh()
        .x_labels(labels.len())
        .x_label_formatter(&|index| labels.get(*index).copied().unwrap_or("").to_string())
        .y_desc("lead time (s)")
        .draw()?;
    chart.draw_series(values.iter().enumerate().map(|(index, value)| {
        Rectangle::new([(index, 0.0), (index + 1, *value)], COLORS[index % COLORS.len()].filled())
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

fn average_detection_metrics(scenarios: &[ScenarioRun]) -> [f64; 5] {
    let non_nominal = scenarios
        .iter()
        .filter(|scenario| scenario.definition.name != "nominal")
        .collect::<Vec<_>>();
    let scalar = average_option(non_nominal.iter().filter_map(|scenario| scenario.summary.scalar_detection_lead_time));
    let multi = average_option(non_nominal.iter().filter_map(|scenario| scenario.summary.multimode_detection_lead_time));
    let state = average_option(non_nominal.iter().filter_map(|scenario| scenario.summary.baseline_state_lead_time));
    let disagreement =
        average_option(non_nominal.iter().filter_map(|scenario| scenario.summary.baseline_disagreement_lead_time));
    let lambda2 = average_option(non_nominal.iter().filter_map(|scenario| scenario.summary.baseline_lambda2_lead_time));
    [state, disagreement, lambda2, scalar, multi]
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
