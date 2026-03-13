use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Local;
use csv::Writer;
use serde::Serialize;
use serde_json::json;

use crate::simulation::SimulationRun;

#[derive(Debug, Clone)]
pub struct RunDirectory {
    pub output_root: PathBuf,
    pub timestamp: String,
    pub run_dir: PathBuf,
}

pub fn create_run_directory(output_root: &Path) -> Result<RunDirectory> {
    fs::create_dir_all(output_root)
        .with_context(|| format!("failed to create output root {}", output_root.display()))?;
    loop {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let run_dir = output_root.join(&timestamp);
        if !run_dir.exists() {
            fs::create_dir_all(&run_dir)
                .with_context(|| format!("failed to create run directory {}", run_dir.display()))?;
            return Ok(RunDirectory {
                output_root: output_root.to_path_buf(),
                timestamp,
                run_dir,
            });
        }
        thread::sleep(Duration::from_millis(1100));
    }
}

pub fn write_run_outputs(run: &SimulationRun, run_dir: &RunDirectory) -> Result<()> {
    write_json(run_dir.run_dir.join("config.json"), &run.config)?;
    let scenario_names = run
        .scenarios
        .iter()
        .map(|scenario| scenario.definition.name.clone())
        .collect::<Vec<_>>();
    write_json(
        run_dir.run_dir.join("run_manifest.json"),
        &json!({
            "crate": "dsfb-tmtr",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": run_dir.timestamp,
            "output_root": run_dir.output_root,
            "run_dir": run_dir.run_dir,
            "config_hash": run.config_hash,
            "scenario_count": run.scenarios.len(),
            "scenarios": scenario_names,
            "artifacts": [
                "config.json",
                "run_manifest.json",
                "scenario_summary.csv",
                "trajectories.csv",
                "trust_timeseries.csv",
                "residuals.csv",
                "correction_events.csv",
                "prediction_tubes.csv",
                "causal_edges.csv",
                "causal_metrics.csv",
                "notebook_ready_summary.json"
            ]
        }),
    )?;

    write_scenario_summary_csv(run, &run_dir.run_dir.join("scenario_summary.csv"))?;
    write_trajectories_csv(run, &run_dir.run_dir.join("trajectories.csv"))?;
    write_trust_csv(run, &run_dir.run_dir.join("trust_timeseries.csv"))?;
    write_residuals_csv(run, &run_dir.run_dir.join("residuals.csv"))?;
    write_corrections_csv(run, &run_dir.run_dir.join("correction_events.csv"))?;
    write_prediction_tubes_csv(run, &run_dir.run_dir.join("prediction_tubes.csv"))?;
    write_causal_edges_csv(run, &run_dir.run_dir.join("causal_edges.csv"))?;
    write_causal_metrics_csv(run, &run_dir.run_dir.join("causal_metrics.csv"))?;
    write_json(
        run_dir.run_dir.join("notebook_ready_summary.json"),
        &run.notebook_summary,
    )?;
    Ok(())
}

fn write_json(path: PathBuf, value: &impl Serialize) -> Result<()> {
    let file =
        File::create(&path).with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(file, value)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn csv_writer(path: &Path) -> Result<Writer<File>> {
    let file =
        File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(Writer::from_writer(file))
}

fn write_scenario_summary_csv(run: &SimulationRun, path: &Path) -> Result<()> {
    let mut writer = csv_writer(path)?;
    for scenario in &run.scenarios {
        writer.serialize(&scenario.summary)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_trajectories_csv(run: &SimulationRun, path: &Path) -> Result<()> {
    #[derive(Serialize)]
    struct Row<'a> {
        scenario: &'a str,
        mode: &'a str,
        observer_level: usize,
        observer_name: &'a str,
        time_index: usize,
        ground_truth: f64,
        prediction: f64,
        estimate: f64,
        measurement: Option<f64>,
        available: bool,
        degraded_interval: bool,
        refinement_interval: bool,
    }
    let mut writer = csv_writer(path)?;
    for scenario in &run.scenarios {
        for mode in [&scenario.baseline, &scenario.tmtr] {
            for observer in &mode.observers {
                for time_index in 0..observer.estimate.len() {
                    writer.serialize(Row {
                        scenario: &scenario.definition.name,
                        mode: &mode.mode,
                        observer_level: observer.level,
                        observer_name: &observer.name,
                        time_index,
                        ground_truth: scenario.truth[time_index],
                        prediction: observer.prediction[time_index],
                        estimate: observer.estimate[time_index],
                        measurement: observer.measurement[time_index],
                        available: observer.available[time_index],
                        degraded_interval: (scenario.definition.degraded_start
                            ..=scenario.definition.degraded_end)
                            .contains(&time_index),
                        refinement_interval: (scenario.definition.degraded_start
                            ..=scenario.definition.refinement_end)
                            .contains(&time_index),
                    })?;
                }
            }
        }
    }
    writer.flush()?;
    Ok(())
}

fn write_trust_csv(run: &SimulationRun, path: &Path) -> Result<()> {
    #[derive(Serialize)]
    struct Row<'a> {
        scenario: &'a str,
        mode: &'a str,
        observer_level: usize,
        observer_name: &'a str,
        time_index: usize,
        trust: f64,
        envelope: f64,
    }
    let mut writer = csv_writer(path)?;
    for scenario in &run.scenarios {
        for mode in [&scenario.baseline, &scenario.tmtr] {
            for observer in &mode.observers {
                for time_index in 0..observer.trust.len() {
                    writer.serialize(Row {
                        scenario: &scenario.definition.name,
                        mode: &mode.mode,
                        observer_level: observer.level,
                        observer_name: &observer.name,
                        time_index,
                        trust: observer.trust[time_index],
                        envelope: observer.envelope[time_index],
                    })?;
                }
            }
        }
    }
    writer.flush()?;
    Ok(())
}

fn write_residuals_csv(run: &SimulationRun, path: &Path) -> Result<()> {
    #[derive(Serialize)]
    struct Row<'a> {
        scenario: &'a str,
        mode: &'a str,
        observer_level: usize,
        observer_name: &'a str,
        time_index: usize,
        residual: f64,
        abs_residual: f64,
        innovation: f64,
    }
    let mut writer = csv_writer(path)?;
    for scenario in &run.scenarios {
        for mode in [&scenario.baseline, &scenario.tmtr] {
            for observer in &mode.observers {
                for time_index in 0..observer.residual.len() {
                    writer.serialize(Row {
                        scenario: &scenario.definition.name,
                        mode: &mode.mode,
                        observer_level: observer.level,
                        observer_name: &observer.name,
                        time_index,
                        residual: observer.residual[time_index],
                        abs_residual: observer.residual[time_index].abs(),
                        innovation: observer.innovation[time_index],
                    })?;
                }
            }
        }
    }
    writer.flush()?;
    Ok(())
}

fn write_corrections_csv(run: &SimulationRun, path: &Path) -> Result<()> {
    let mut writer = csv_writer(path)?;
    for scenario in &run.scenarios {
        for event in &scenario.tmtr.correction_events {
            writer.serialize(event)?;
        }
    }
    writer.flush()?;
    Ok(())
}

fn write_prediction_tubes_csv(run: &SimulationRun, path: &Path) -> Result<()> {
    let mut writer = csv_writer(path)?;
    for scenario in &run.scenarios {
        for tube in scenario
            .baseline
            .prediction_tubes
            .iter()
            .chain(scenario.tmtr.prediction_tubes.iter())
        {
            writer.serialize(tube)?;
        }
    }
    writer.flush()?;
    Ok(())
}

fn write_causal_edges_csv(run: &SimulationRun, path: &Path) -> Result<()> {
    let mut writer = csv_writer(path)?;
    for scenario in &run.scenarios {
        for edge in scenario
            .baseline
            .causal_graph
            .edges
            .iter()
            .chain(scenario.tmtr.causal_graph.edges.iter())
        {
            writer.serialize(edge)?;
        }
    }
    writer.flush()?;
    Ok(())
}

fn write_causal_metrics_csv(run: &SimulationRun, path: &Path) -> Result<()> {
    #[derive(Serialize)]
    struct Row<'a> {
        scenario: &'a str,
        mode: &'a str,
        metric: &'a str,
        value: f64,
    }
    let mut writer = csv_writer(path)?;
    for scenario in &run.scenarios {
        for (mode, metrics) in [
            ("baseline", &scenario.baseline.causal_metrics),
            ("tmtr", &scenario.tmtr.causal_metrics),
        ] {
            let rows = [
                ("edge_count", metrics.edge_count as f64),
                ("backward_edge_count", metrics.backward_edge_count as f64),
                ("cycle_count", metrics.cycle_count as f64),
                (
                    "reachable_nodes_from_anchor",
                    metrics.reachable_nodes_from_anchor as f64,
                ),
                (
                    "local_window_edge_density",
                    metrics.local_window_edge_density,
                ),
                ("max_in_degree", metrics.max_in_degree as f64),
                ("max_out_degree", metrics.max_out_degree as f64),
                ("max_path_length", metrics.max_path_length as f64),
                ("mean_path_length", metrics.mean_path_length),
            ];
            for (metric, value) in rows {
                writer.serialize(Row {
                    scenario: &scenario.definition.name,
                    mode,
                    metric,
                    value,
                })?;
            }
        }
    }
    writer.flush()?;
    Ok(())
}
