use serde::Serialize;

use crate::causal::{
    build_causal_graph, summarize_causal_graph, CausalGraph, CausalMetricsSummary,
};
use crate::config::SimulationConfig;
use crate::metrics::{
    build_prediction_tubes, notebook_ready_summary, summarize_scenario, NotebookReadySummary,
    PredictionTubePoint, ScenarioSummaryRow,
};
use crate::observer::{simulate_observers, ObserverSeries};
use crate::scenario::{scenario_suite, ScenarioDefinition};
use crate::tmtr::{apply_tmtr, CorrectionEvent, RecursionStats};

#[derive(Debug, Clone, Serialize)]
pub struct ModeArtifacts {
    pub mode: String,
    pub observers: Vec<ObserverSeries>,
    pub correction_events: Vec<CorrectionEvent>,
    pub prediction_tubes: Vec<PredictionTubePoint>,
    pub causal_graph: CausalGraph,
    pub causal_metrics: CausalMetricsSummary,
    pub recursion_stats: RecursionStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioArtifacts {
    pub definition: ScenarioDefinition,
    pub truth: Vec<f64>,
    pub baseline: ModeArtifacts,
    pub tmtr: ModeArtifacts,
    pub summary: ScenarioSummaryRow,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationRun {
    pub config: SimulationConfig,
    pub config_hash: String,
    pub scenarios: Vec<ScenarioArtifacts>,
    pub notebook_summary: NotebookReadySummary,
}

impl SimulationRun {
    pub fn stable_signature(&self) -> anyhow::Result<String> {
        #[derive(Serialize)]
        struct StableView<'a> {
            config: &'a SimulationConfig,
            config_hash: &'a str,
            scenarios: &'a [ScenarioArtifacts],
        }
        Ok(serde_json::to_string(&StableView {
            config: &self.config,
            config_hash: &self.config_hash,
            scenarios: &self.scenarios,
        })?)
    }
}

pub fn run_simulation(config: &SimulationConfig) -> anyhow::Result<SimulationRun> {
    let scenarios = scenario_suite(config)
        .into_iter()
        .map(|definition| run_single_scenario(config, definition))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let notebook_summary =
        notebook_ready_summary(&config.output_root, &collect_summaries(&scenarios));
    Ok(SimulationRun {
        config: config.clone(),
        config_hash: config.stable_hash()?,
        scenarios,
        notebook_summary,
    })
}

fn run_single_scenario(
    config: &SimulationConfig,
    definition: ScenarioDefinition,
) -> anyhow::Result<ScenarioArtifacts> {
    let truth = definition.truth_series();
    let (specs, baseline_observers) = simulate_observers(&definition, &truth);
    let baseline_primary = baseline_observers[0].clone();
    let baseline_tubes = build_prediction_tubes(&definition, "baseline", &baseline_primary, &truth);
    let baseline_graph = build_causal_graph(
        &definition.name,
        "baseline",
        &baseline_observers,
        &[],
        config.min_trust_gap,
    );
    let baseline_causal = summarize_causal_graph(&baseline_graph, definition.delta);

    let tmtr_result = apply_tmtr(&definition, config, &specs, &baseline_observers, &truth);
    let tmtr_primary = tmtr_result.observers[0].clone();
    let tmtr_tubes = build_prediction_tubes(&definition, "tmtr", &tmtr_primary, &truth);
    let tmtr_graph = build_causal_graph(
        &definition.name,
        "tmtr",
        &tmtr_result.observers,
        &tmtr_result.correction_events,
        config.min_trust_gap,
    );
    let tmtr_causal = summarize_causal_graph(&tmtr_graph, definition.delta);

    let summary = summarize_scenario(
        &definition,
        &baseline_primary,
        &tmtr_primary,
        &baseline_tubes,
        &tmtr_tubes,
        &baseline_causal,
        &tmtr_causal,
        &tmtr_result.recursion_stats,
    );

    Ok(ScenarioArtifacts {
        definition,
        truth,
        baseline: ModeArtifacts {
            mode: "baseline".to_string(),
            observers: baseline_observers,
            correction_events: Vec::new(),
            prediction_tubes: baseline_tubes,
            causal_graph: baseline_graph,
            causal_metrics: baseline_causal,
            recursion_stats: RecursionStats {
                total_correction_events: 0,
                max_recursion_depth: 0,
                mean_recursion_depth: 0.0,
                convergence_iterations: 0,
                average_correction_magnitude: 0.0,
                average_correction_trust_weight: 0.0,
                monotonicity_violations: 0,
            },
        },
        tmtr: ModeArtifacts {
            mode: "tmtr".to_string(),
            observers: tmtr_result.observers,
            correction_events: tmtr_result.correction_events,
            prediction_tubes: tmtr_tubes,
            causal_graph: tmtr_graph,
            causal_metrics: tmtr_causal,
            recursion_stats: tmtr_result.recursion_stats,
        },
        summary,
    })
}

fn collect_summaries(scenarios: &[ScenarioArtifacts]) -> Vec<ScenarioSummaryRow> {
    scenarios
        .iter()
        .map(|scenario| scenario.summary.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::config::SimulationConfig;
    use crate::simulation::run_simulation;

    #[test]
    fn deterministic_signature_is_stable() {
        let config = SimulationConfig {
            n_steps: 240,
            ..SimulationConfig::default()
        };
        let first = run_simulation(&config).expect("first simulation");
        let second = run_simulation(&config).expect("second simulation");
        assert_eq!(first.config_hash, second.config_hash);
        assert_eq!(
            first.stable_signature().expect("first signature"),
            second.stable_signature().expect("second signature")
        );
    }
}
