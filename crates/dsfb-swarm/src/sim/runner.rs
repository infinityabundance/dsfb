use std::time::Instant;

use anyhow::Result;
use nalgebra::DMatrix;
use serde::Serialize;

use crate::config::RunConfig;
use crate::math::baselines::{BaselineMonitor, BaselineRow};
use crate::math::envelopes::{AnomalyCertificate, EnvelopeMonitor};
use crate::math::laplacian::{delta_norm, laplacian};
use crate::math::metrics::{summarize, MetricsInput, ScenarioSummary};
use crate::math::predictor::PredictorState;
use crate::math::residuals::{compute_residual_stack, ResidualStack};
use crate::math::spectrum::compute_spectrum;
use crate::math::trust::{TrustModel, TrustSnapshot};
use crate::sim::agents::{initialize_agents, AgentState};
use crate::sim::dynamics::evolve_agents;
use crate::sim::graph::{build_nominal_graph, edge_records, pair_disagreement, EdgeRecord};
use crate::sim::scenarios::ScenarioDefinition;

#[derive(Debug, Clone, Serialize)]
pub struct TimeSeriesRow {
    pub scenario: String,
    pub step: usize,
    pub time: f64,
    pub lambda2: f64,
    pub predicted_lambda2: f64,
    pub scalar_residual: f64,
    pub scalar_drift: f64,
    pub scalar_slew: f64,
    pub scalar_residual_envelope: f64,
    pub combined_score: f64,
    pub combined_envelope: f64,
    pub laplacian_delta_norm: f64,
    pub mean_node_trust: f64,
    pub min_node_trust: f64,
    pub affected_mean_trust: f64,
    pub anomaly_scalar: bool,
    pub anomaly_multimode: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpectrumRow {
    pub scenario: String,
    pub step: usize,
    pub time: f64,
    pub mode: usize,
    pub eigenvalue: f64,
    pub predicted: f64,
    pub residual: f64,
    pub drift: f64,
    pub slew: f64,
    pub mode_shape_residual: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidualRow {
    pub scenario: String,
    pub step: usize,
    pub time: f64,
    pub mode: usize,
    pub residual: f64,
    pub abs_residual: f64,
    pub drift: f64,
    pub slew: f64,
    pub mode_shape_residual: f64,
    pub scalar_flag: bool,
    pub multimode_flag: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrustRow {
    pub scenario: String,
    pub step: usize,
    pub time: f64,
    pub node: usize,
    pub trust: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnomalyEvent {
    pub scenario: String,
    pub step: usize,
    pub time: f64,
    pub detector: String,
    pub certificate: AnomalyCertificate,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopologySnapshot {
    pub scenario: String,
    pub label: String,
    pub step: usize,
    pub time: f64,
    pub agents: Vec<AgentState>,
    pub edges: Vec<EdgeRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioRun {
    pub definition: ScenarioDefinition,
    pub summary: ScenarioSummary,
    pub time_series: Vec<TimeSeriesRow>,
    pub spectra: Vec<SpectrumRow>,
    pub residuals: Vec<ResidualRow>,
    pub trust: Vec<TrustRow>,
    pub baselines: Vec<BaselineRow>,
    pub anomalies: Vec<AnomalyEvent>,
    pub topology_snapshots: Vec<TopologySnapshot>,
}

pub fn run_scenario(config: &RunConfig, definition: ScenarioDefinition) -> Result<ScenarioRun> {
    let monitored_modes = config.monitored_modes.min(config.agents.saturating_sub(1).max(1));
    let scalar_modes = 1;
    let mut agents = initialize_agents(config.agents);
    let mut predictor_scalar = PredictorState::new(config.predictor);
    let mut predictor_multi = PredictorState::new(config.predictor);
    let mut scalar_monitor = EnvelopeMonitor::new(config.warmup_steps);
    let mut multimode_monitor = EnvelopeMonitor::new(config.warmup_steps);
    let mut trust_model = TrustModel::new(config.trust_mode, config.agents);
    let mut baseline_monitor = BaselineMonitor::new(config.warmup_steps);

    let mut previous_laplacian: Option<DMatrix<f64>> = None;
    let mut previous_scalar: Option<ResidualStack> = None;
    let mut previous_multi: Option<ResidualStack> = None;

    let mut time_series = Vec::with_capacity(config.steps);
    let mut spectra_rows = Vec::new();
    let mut residual_rows = Vec::new();
    let mut trust_rows = Vec::new();
    let mut baseline_rows = Vec::with_capacity(config.steps);
    let mut anomalies = Vec::new();
    let mut snapshots = Vec::new();

    let run_start = Instant::now();
    for step in 0..config.steps {
        let time = step as f64 * config.dt;
        let nominal = build_nominal_graph(
            &agents,
            config.interaction_radius,
            config.k_neighbors,
            config.base_gain,
        );
        let perturbed = definition.apply_edge_modifiers(step, &nominal.adjacency);
        let effective_adjacency = perturbed.component_mul(trust_model.current_edge_trust());
        let pair_disagreement_matrix = pair_disagreement(&agents, &effective_adjacency);
        let lap = laplacian(&effective_adjacency);
        let spectral = compute_spectrum(&lap);

        let scalar_observed = vec![spectral.lambda2];
        let scalar_predicted = predictor_scalar.predict_values(scalar_modes);
        let scalar_residual = compute_residual_stack(
            &scalar_observed,
            &scalar_predicted,
            previous_scalar.as_ref().map(|stack| stack.residuals.as_slice()),
            previous_scalar.as_ref().map(|stack| stack.drifts.as_slice()),
            &spectral.eigenvectors,
            predictor_scalar.previous_vectors(),
            config.dt,
        );
        let scalar_certificate = scalar_monitor.update(&scalar_residual);

        let multi_observed = spectral
            .eigenvalues
            .iter()
            .skip(1)
            .take(monitored_modes)
            .copied()
            .collect::<Vec<_>>();
        let multi_predicted = predictor_multi.predict_values(monitored_modes);
        let multi_residual = compute_residual_stack(
            &multi_observed,
            &multi_predicted,
            previous_multi.as_ref().map(|stack| stack.residuals.as_slice()),
            previous_multi.as_ref().map(|stack| stack.drifts.as_slice()),
            &spectral.eigenvectors,
            predictor_multi.previous_vectors(),
            config.dt,
        );
        let multimode_certificate = multimode_monitor.update(&multi_residual);

        let global_score = if config.multi_mode {
            multi_residual.combined_score
        } else {
            scalar_residual.combined_score
        };
        let trust_snapshot = trust_model.update(
            &perturbed,
            &pair_disagreement_matrix,
            global_score,
            &definition.affected_nodes(config.agents),
        );

        let baseline = baseline_monitor.update(&agents, &effective_adjacency, spectral.lambda2, time);
        baseline_rows.push(baseline.clone());
        record_spectra(
            &definition,
            step,
            time,
            &multi_residual,
            &mut spectra_rows,
            &mut residual_rows,
            scalar_certificate.flagged,
            multimode_certificate.flagged,
        );
        record_trust(&definition, step, time, &trust_snapshot, &mut trust_rows);

        if scalar_certificate.flagged {
            anomalies.push(AnomalyEvent {
                scenario: definition.name.to_string(),
                step,
                time,
                detector: "scalar_lambda2".to_string(),
                certificate: scalar_certificate.clone(),
            });
        }
        if multimode_certificate.flagged {
            anomalies.push(AnomalyEvent {
                scenario: definition.name.to_string(),
                step,
                time,
                detector: "multimode_stack".to_string(),
                certificate: multimode_certificate.clone(),
            });
        }

        let envelope = scalar_monitor.state().clone();
        time_series.push(TimeSeriesRow {
            scenario: definition.name.to_string(),
            step,
            time,
            lambda2: spectral.lambda2,
            predicted_lambda2: scalar_predicted.first().copied().unwrap_or(0.0),
            scalar_residual: scalar_residual.scalar_residual,
            scalar_drift: scalar_residual.scalar_drift,
            scalar_slew: scalar_residual.scalar_slew,
            scalar_residual_envelope: envelope.scalar_residual_envelope,
            combined_score: multi_residual.combined_score,
            combined_envelope: multimode_monitor.state().combined_envelope,
            laplacian_delta_norm: delta_norm(&lap, previous_laplacian.as_ref()),
            mean_node_trust: trust_snapshot.mean_node_trust,
            min_node_trust: trust_snapshot.min_node_trust,
            affected_mean_trust: trust_snapshot.affected_mean_trust,
            anomaly_scalar: scalar_certificate.flagged,
            anomaly_multimode: multimode_certificate.flagged,
            connected: spectral.lambda2 > 1.0e-5,
        });

        if should_capture_snapshot(step, config.steps, definition.onset_step) {
            snapshots.push(TopologySnapshot {
                scenario: definition.name.to_string(),
                label: snapshot_label(step, config.steps, definition.onset_step),
                step,
                time,
                agents: agents.clone(),
                edges: edge_records(&effective_adjacency),
            });
        }

        predictor_scalar.update(&scalar_observed, spectral.eigenvectors.clone());
        predictor_multi.update(&multi_observed, spectral.eigenvectors.clone());
        previous_laplacian = Some(lap);
        previous_scalar = Some(scalar_residual);
        previous_multi = Some(multi_residual);

        evolve_agents(&mut agents, &effective_adjacency, config, &definition, step);
    }

    let runtime_ms = run_start.elapsed().as_secs_f64() * 1_000.0;
    let summary = summarize(MetricsInput {
        scenario: definition.kind,
        scenario_name: definition.name,
        agents: config.agents,
        steps: config.steps,
        dt: config.dt,
        noise_level: config.noise_level,
        onset_step: definition.onset_step.min(config.steps.saturating_sub(1)),
        lambda2: &time_series.iter().map(|row| row.lambda2).collect::<Vec<_>>(),
        scalar_flags: &time_series.iter().map(|row| row.anomaly_scalar).collect::<Vec<_>>(),
        multimode_flags: &time_series.iter().map(|row| row.anomaly_multimode).collect::<Vec<_>>(),
        baseline_state_flags: &baseline_rows.iter().map(|row| row.state_norm_flag).collect::<Vec<_>>(),
        baseline_disagreement_flags: &baseline_rows
            .iter()
            .map(|row| row.disagreement_energy_flag)
            .collect::<Vec<_>>(),
        baseline_lambda2_flags: &baseline_rows.iter().map(|row| row.raw_lambda2_flag).collect::<Vec<_>>(),
        affected_trust: &time_series
            .iter()
            .map(|row| row.affected_mean_trust)
            .collect::<Vec<_>>(),
        scalar_residuals: &time_series
            .iter()
            .map(|row| row.scalar_residual)
            .collect::<Vec<_>>(),
        scalar_envelopes: &time_series
            .iter()
            .map(|row| row.scalar_residual_envelope)
            .collect::<Vec<_>>(),
        combined_scores: &time_series.iter().map(|row| row.combined_score).collect::<Vec<_>>(),
        laplacian_delta_norms: &time_series
            .iter()
            .map(|row| row.laplacian_delta_norm)
            .collect::<Vec<_>>(),
        runtime_ms,
    });

    Ok(ScenarioRun {
        definition,
        summary,
        time_series,
        spectra: spectra_rows,
        residuals: residual_rows,
        trust: trust_rows,
        baselines: baseline_rows,
        anomalies,
        topology_snapshots: snapshots,
    })
}

fn record_spectra(
    definition: &ScenarioDefinition,
    step: usize,
    time: f64,
    residuals: &ResidualStack,
    spectra_rows: &mut Vec<SpectrumRow>,
    residual_rows: &mut Vec<ResidualRow>,
    scalar_flag: bool,
    multimode_flag: bool,
) {
    for index in 0..residuals.observed.len() {
        let mode = index + 2;
        spectra_rows.push(SpectrumRow {
            scenario: definition.name.to_string(),
            step,
            time,
            mode,
            eigenvalue: residuals.observed[index],
            predicted: residuals.predicted[index],
            residual: residuals.residuals[index],
            drift: residuals.drifts[index],
            slew: residuals.slews[index],
            mode_shape_residual: residuals.mode_shape_residuals[index],
        });
        residual_rows.push(ResidualRow {
            scenario: definition.name.to_string(),
            step,
            time,
            mode,
            residual: residuals.residuals[index],
            abs_residual: residuals.residuals[index].abs(),
            drift: residuals.drifts[index],
            slew: residuals.slews[index],
            mode_shape_residual: residuals.mode_shape_residuals[index],
            scalar_flag,
            multimode_flag,
        });
    }
}

fn record_trust(
    definition: &ScenarioDefinition,
    step: usize,
    time: f64,
    snapshot: &TrustSnapshot,
    trust_rows: &mut Vec<TrustRow>,
) {
    for (node, trust) in snapshot.node_trust.iter().copied().enumerate() {
        trust_rows.push(TrustRow {
            scenario: definition.name.to_string(),
            step,
            time,
            node,
            trust,
        });
    }
}

fn should_capture_snapshot(step: usize, total_steps: usize, onset_step: usize) -> bool {
    step == 0 || step == onset_step.min(total_steps.saturating_sub(1)) || step == total_steps.saturating_sub(1)
}

fn snapshot_label(step: usize, total_steps: usize, onset_step: usize) -> String {
    if step == 0 {
        "pre_anomaly".to_string()
    } else if step == onset_step.min(total_steps.saturating_sub(1)) {
        "onset".to_string()
    } else {
        "late".to_string()
    }
}
