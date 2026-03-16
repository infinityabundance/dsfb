//! Forensic audit engine for the DSFB stack.
//!
//! References: `CORE-04`, `CORE-08`, and `CORE-10` for graph gating, anomaly
//! soundness, and deterministic stack composition; `DSFB-07` and `DSFB-08` for
//! residual semantics; `DSCD-05` and `DSCD-07` for admissible edge pruning; and
//! `TMTR-01`, `TMTR-04`, and `TMTR-10` for monotone trust descent and
//! stabilization.

use anyhow::{bail, Result};
use chrono::Utc;
use dsfb::{DsfbObserver, DsfbParams, DsfbState};
use serde::Serialize;

use crate::cli::BaselineComparison;
use crate::complexity::{classify_step_complexity, StepComplexity};
use crate::ekf::{BaselineEkf, EkfStepResult};
use crate::graph::{build_causal_graph, CausalGraph, ChannelAuditInput, GraphMetrics};
use crate::report::{award_seal, SealLevel};
use crate::input::{TraceDocument, TraceStep};

/// Runtime configuration for the forensic auditor.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ForensicConfig {
    /// Slew threshold used by the deterministic envelope.
    pub slew_threshold: f64,
    /// Trust floor below which an update is treated as pruned or down-weighted.
    pub trust_alpha: f64,
    /// Whether the EKF baseline is enabled.
    pub baseline_comparison: BaselineComparison,
}

/// DSFB-compatible state snapshot serialized into the output trace.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct StateSnapshot {
    /// Phase / position.
    pub phi: f64,
    /// Drift / velocity.
    pub omega: f64,
    /// Slew / acceleration.
    pub alpha: f64,
}

/// One channel-local provenance update written into `causal_trace.json`.
#[derive(Clone, Debug, Serialize)]
pub struct ChannelUpdate {
    /// Stable channel label.
    pub channel: String,
    /// The rule or theorem ID that dominated the update.
    pub rule_id: String,
    /// Monotone forensic trust score.
    pub trust_score: f64,
    /// Causal depth from the fused root.
    pub causal_depth: usize,
    /// Raw DSFB trust weight.
    pub raw_trust_weight: f64,
    /// Measurement value for the step.
    pub measurement: f64,
    /// DSFB residual.
    pub residual: f64,
    /// Residual EMA exposed by the DSFB observer.
    pub residual_ema: f64,
    /// Measured second-difference slew.
    pub measurement_slew: f64,
    /// Deterministic envelope for the update.
    pub deterministic_envelope: f64,
    /// Whether the DSFB stack pruned or down-weighted the update.
    pub dsfb_pruned: bool,
    /// Whether the EKF baseline accepted the measurement.
    pub ekf_accepted: bool,
    /// Whether the EKF acceptance constitutes a silent failure.
    pub silent_failure: bool,
    /// Whether the channel participated in a shatter event at this step.
    pub shatter_event: bool,
}

/// One per-step forensic record written into `causal_trace.json`.
#[derive(Clone, Debug, Serialize)]
pub struct StepAuditRecord {
    /// Step index in the input trace.
    pub step_index: usize,
    /// Time step in seconds.
    pub dt: f64,
    /// Dominant theorem or rule ID for the step.
    pub rule_id: String,
    /// Minimum trust score observed across all channel updates.
    pub trust_score: f64,
    /// Maximum causal depth observed in the graph.
    pub causal_depth: usize,
    /// Whether the step created a causal-topology shatter event.
    pub shatter_event: bool,
    /// Number of silent failures in the step.
    pub silent_failures: usize,
    /// Graph metrics for the step.
    pub graph: GraphMetrics,
    /// Complexity log for the step.
    pub complexity: StepComplexity,
    /// DSFB state after the update.
    pub dsfb_state: StateSnapshot,
    /// Optional EKF baseline state after the update.
    pub ekf_state: Option<StateSnapshot>,
    /// Channel-local provenance updates.
    pub updates: Vec<ChannelUpdate>,
}

/// Top-level machine-readable trace document.
#[derive(Clone, Debug, Serialize)]
pub struct CausalTraceDocument {
    /// Input trace path or label.
    pub input_trace: String,
    /// UTC timestamp of the audit run.
    pub generated_at_utc: String,
    /// Active forensic configuration.
    pub config: ForensicConfig,
    /// Stable channel names.
    pub channel_names: Vec<String>,
    /// One deterministic record per input step.
    pub steps: Vec<StepAuditRecord>,
}

/// Human-readable summary fields shared by the markdown and JSON reports.
#[derive(Clone, Debug, Serialize)]
pub struct ForensicRunSummary {
    /// Input trace path or label.
    pub input_trace: String,
    /// Total number of steps processed.
    pub total_steps: usize,
    /// Total channel count.
    pub channel_count: usize,
    /// Total update count.
    pub total_updates: usize,
    /// Total shatter events.
    pub shatter_events: usize,
    /// Total silent failures.
    pub silent_failures: usize,
    /// Total DSFB-pruned or down-weighted updates.
    pub pruned_updates: usize,
    /// Total EKF-accepted updates.
    pub baseline_accepted_updates: usize,
    /// Mean forensic trust score across all updates.
    pub mean_trust_score: f64,
    /// Minimum forensic trust score across all updates.
    pub min_trust_score: f64,
    /// Maximum causal depth across the run.
    pub max_causal_depth: usize,
    /// Maximum weakly connected component count across the run.
    pub max_components: usize,
    /// Reasoning-consistency score in `[0,1]`.
    pub reasoning_consistency: f64,
    /// Per-step symbolic complexity bound.
    pub complexity_bound: String,
    /// Maximum primitive operations observed.
    pub max_total_ops: usize,
    /// Maximum transient memory words observed.
    pub max_memory_words: usize,
    /// Whether the EKF baseline was enabled.
    pub baseline_enabled: bool,
    /// Active slew threshold.
    pub slew_threshold: f64,
    /// Active trust alpha.
    pub trust_alpha: f64,
    /// Optional DSFB MAE against truth, if truth was present.
    pub dsfb_phi_mae: Option<f64>,
    /// Optional EKF MAE against truth, if truth was present and baseline was enabled.
    pub ekf_phi_mae: Option<f64>,
    /// Final DSFB seal of integrity.
    pub seal: SealLevel,
}

/// Final audit outputs.
#[derive(Clone, Debug, Serialize)]
pub struct AuditRun {
    /// Machine-readable causal trace.
    pub causal_trace: CausalTraceDocument,
    /// Run-level summary used by the markdown report.
    pub summary: ForensicRunSummary,
}

/// Stateful forensic auditor.
pub struct ForensicAuditor {
    config: ForensicConfig,
    dsfb: DsfbObserver,
    ekf: Option<BaselineEkf>,
    channel_names: Vec<String>,
    trust_penalties: Vec<f64>,
    measurement_history: Vec<[f64; 2]>,
    previous_components: usize,
    previous_fragmented: bool,
}

impl ForensicAuditor {
    /// Create a new forensic auditor.
    ///
    /// References: `CORE-10`, `TMTR-01`, and `TMTR-04`.
    pub fn new(config: ForensicConfig, channel_names: &[String], initial_state: DsfbState) -> Self {
        let mut dsfb = DsfbObserver::new(DsfbParams::default(), channel_names.len());
        dsfb.init(initial_state);
        let ekf = if config.baseline_comparison.enabled() {
            Some(BaselineEkf::new(initial_state))
        } else {
            None
        };
        Self {
            config,
            dsfb,
            ekf,
            channel_names: channel_names.to_vec(),
            trust_penalties: vec![0.0; channel_names.len()],
            measurement_history: vec![[0.0; 2]; channel_names.len()],
            previous_components: channel_names.len() + 1,
            previous_fragmented: false,
        }
    }

    /// Audit a full trace and return machine-readable and human-readable summaries.
    ///
    /// References: `CORE-04`, `CORE-08`, `CORE-10`, `DSCD-05`, and `TMTR-10`.
    pub fn audit_trace(&mut self, trace: &TraceDocument, input_label: &str) -> Result<AuditRun> {
        if trace.channel_names.len() != self.channel_names.len() {
            bail!(
                "trace channel count {} does not match auditor channel count {}",
                trace.channel_names.len(),
                self.channel_names.len()
            );
        }

        let mut records = Vec::with_capacity(trace.steps.len());
        let mut trust_sum = 0.0;
        let mut trust_count = 0usize;
        let mut min_trust_score: f64 = 1.0;
        let mut pruned_updates = 0usize;
        let mut baseline_accepted_updates = 0usize;
        let mut shatter_events = 0usize;
        let mut silent_failures = 0usize;
        let mut max_causal_depth = 0usize;
        let mut max_components = 0usize;
        let mut max_total_ops = 0usize;
        let mut max_memory_words = 0usize;
        let mut dsfb_abs_error_sum = 0.0;
        let mut ekf_abs_error_sum = 0.0;
        let mut truth_count = 0usize;

        for step in &trace.steps {
            let record = self.audit_step(step);
            for update in &record.updates {
                trust_sum += update.trust_score;
                trust_count += 1;
                min_trust_score = min_trust_score.min(update.trust_score);
                if update.dsfb_pruned {
                    pruned_updates += 1;
                }
                if update.ekf_accepted {
                    baseline_accepted_updates += 1;
                }
                if update.silent_failure {
                    silent_failures += 1;
                }
            }
            if record.shatter_event {
                shatter_events += 1;
            }
            max_causal_depth = max_causal_depth.max(record.graph.max_causal_depth);
            max_components = max_components.max(record.graph.connected_components);
            max_total_ops = max_total_ops.max(record.complexity.total_ops);
            max_memory_words = max_memory_words.max(record.complexity.memory_words);
            if let Some(truth) = step.truth {
                truth_count += 1;
                dsfb_abs_error_sum += (record.dsfb_state.phi - truth.phi).abs();
                if let Some(ekf_state) = record.ekf_state {
                    ekf_abs_error_sum += (ekf_state.phi - truth.phi).abs();
                }
            }
            records.push(record);
        }

        let total_updates = trust_count;
        let mean_trust_score = if trust_count > 0 {
            trust_sum / trust_count as f64
        } else {
            0.0
        };
        let weighted_failures = shatter_events as f64 * 2.0
            + silent_failures as f64 * 3.0
            + pruned_updates as f64 * 0.25;
        let reasoning_consistency =
            (1.0 - weighted_failures / (total_updates.max(1) as f64)).clamp(0.0, 1.0);
        let baseline_enabled = self.config.baseline_comparison.enabled();
        let dsfb_phi_mae = if truth_count > 0 {
            Some(dsfb_abs_error_sum / truth_count as f64)
        } else {
            None
        };
        let ekf_phi_mae = if truth_count > 0 && baseline_enabled {
            Some(ekf_abs_error_sum / truth_count as f64)
        } else {
            None
        };

        let mut summary = ForensicRunSummary {
            input_trace: input_label.to_string(),
            total_steps: trace.steps.len(),
            channel_count: trace.channel_names.len(),
            total_updates,
            shatter_events,
            silent_failures,
            pruned_updates,
            baseline_accepted_updates,
            mean_trust_score,
            min_trust_score,
            max_causal_depth,
            max_components,
            reasoning_consistency,
            complexity_bound: "O(c^2)".to_string(),
            max_total_ops,
            max_memory_words,
            baseline_enabled,
            slew_threshold: self.config.slew_threshold,
            trust_alpha: self.config.trust_alpha,
            dsfb_phi_mae,
            ekf_phi_mae,
            seal: SealLevel::Level1,
        };
        summary.seal = award_seal(&summary);

        Ok(AuditRun {
            causal_trace: CausalTraceDocument {
                input_trace: input_label.to_string(),
                generated_at_utc: Utc::now().to_rfc3339(),
                config: self.config,
                channel_names: trace.channel_names.clone(),
                steps: records,
            },
            summary,
        })
    }

    fn audit_step(&mut self, step: &TraceStep) -> StepAuditRecord {
        let dsfb_diag = self.dsfb.step_with_diagnostics(&step.measurements, step.dt);
        let ekf_result = self
            .ekf
            .as_mut()
            .map(|baseline| baseline.step(&step.measurements, step.dt));

        let channel_inputs = self.build_channel_inputs(step, &dsfb_diag, ekf_result.as_ref());
        let graph = build_causal_graph(dsfb_diag.state, &channel_inputs, step.dt, self.config.trust_alpha);
        let shatter_event = self.detect_shatter_event(&graph, &channel_inputs);
        let complexity = classify_step_complexity(
            step.step,
            self.channel_names.len(),
            graph.metrics().edge_count,
            self.config.baseline_comparison.enabled(),
        );
        let silent_failures = channel_inputs
            .iter()
            .enumerate()
            .filter(|(index, input)| {
                let accepted = ekf_result
                    .as_ref()
                    .and_then(|result| result.decisions.get(*index))
                    .map(|decision| decision.accepted)
                    .unwrap_or(false);
                accepted && is_pruned(*input, self.config.trust_alpha)
            })
            .count();

        let mut updates = Vec::with_capacity(self.channel_names.len());
        for (index, input) in channel_inputs.iter().enumerate() {
            let ekf_accepted = ekf_result
                .as_ref()
                .and_then(|result| result.decisions.get(index))
                .map(|decision| decision.accepted)
                .unwrap_or(false);
            let dsfb_pruned = is_pruned(input, self.config.trust_alpha);
            let silent_failure = ekf_accepted && dsfb_pruned;
            let rule_id = dominant_rule_id(dsfb_pruned, silent_failure, shatter_event).to_string();
            updates.push(ChannelUpdate {
                channel: self.channel_names[index].clone(),
                rule_id,
                trust_score: input.trust_score,
                causal_depth: graph.causal_depth(index + 1),
                raw_trust_weight: input.raw_trust_weight,
                measurement: input.measurement,
                residual: input.residual,
                residual_ema: self.dsfb.ema_residual(index),
                measurement_slew: input.measurement_slew,
                deterministic_envelope: input.deterministic_envelope,
                dsfb_pruned,
                ekf_accepted,
                silent_failure,
                shatter_event,
            });
        }

        let rule_id = updates
            .iter()
            .find(|update| update.silent_failure)
            .map(|update| update.rule_id.clone())
            .unwrap_or_else(|| {
                if shatter_event {
                    "CORE-04".to_string()
                } else {
                    "CORE-10".to_string()
                }
            });
        let trust_score = updates
            .iter()
            .map(|update| update.trust_score)
            .fold(1.0, f64::min);

        StepAuditRecord {
            step_index: step.step,
            dt: step.dt,
            rule_id,
            trust_score,
            causal_depth: graph.metrics().max_causal_depth,
            shatter_event,
            silent_failures,
            graph: graph.metrics().clone(),
            complexity,
            dsfb_state: state_snapshot(dsfb_diag.state),
            ekf_state: ekf_result.map(|result| state_snapshot(result.state)),
            updates,
        }
    }

    fn build_channel_inputs(
        &mut self,
        step: &TraceStep,
        dsfb_diag: &dsfb::DsfbStepDiagnostics,
        ekf_result: Option<&EkfStepResult>,
    ) -> Vec<ChannelAuditInput> {
        let mut inputs = Vec::with_capacity(step.measurements.len());
        for index in 0..step.measurements.len() {
            let measurement = step.measurements[index];
            let measurement_slew = second_difference(measurement, self.measurement_history[index], step.dt);
            let residual = dsfb_diag.residuals[index];
            let raw_trust_weight = dsfb_diag.trust_stats[index].weight;
            let deterministic_envelope = self.deterministic_envelope(dsfb_diag.state, step.dt);
            let trust_score = self.update_trust_score(
                index,
                residual,
                raw_trust_weight,
                measurement_slew,
                deterministic_envelope,
                ekf_result.and_then(|result| result.decisions.get(index)).map(|decision| decision.accepted).unwrap_or(false),
            );

            inputs.push(ChannelAuditInput {
                index,
                measurement,
                residual,
                raw_trust_weight,
                trust_score,
                measurement_slew,
                deterministic_envelope,
            });
            self.measurement_history[index] = [self.measurement_history[index][1], measurement];
        }
        inputs
    }

    fn deterministic_envelope(&self, state: DsfbState, dt: f64) -> f64 {
        self.config.slew_threshold + state.alpha.abs() + 0.5 * dt * (state.omega.abs() + 1.0)
    }

    fn update_trust_score(
        &mut self,
        index: usize,
        residual: f64,
        raw_trust_weight: f64,
        measurement_slew: f64,
        deterministic_envelope: f64,
        ekf_accepted: bool,
    ) -> f64 {
        let slew_ratio = if deterministic_envelope > 0.0 {
            measurement_slew / deterministic_envelope
        } else {
            0.0
        };
        let residual_ratio = residual.abs() / (deterministic_envelope + 0.05);
        let raw_penalty = if raw_trust_weight < self.config.trust_alpha {
            (self.config.trust_alpha - raw_trust_weight) / self.config.trust_alpha.max(1e-6)
        } else {
            0.0
        };
        let baseline_penalty = if ekf_accepted && (slew_ratio > 1.0 || residual_ratio > 1.0) {
            0.35
        } else {
            0.0
        };
        let increment = (slew_ratio - 1.0).max(0.0) * 0.8
            + (residual_ratio - 1.0).max(0.0) * 0.5
            + raw_penalty * 0.3
            + baseline_penalty;
        self.trust_penalties[index] += increment;
        (-self.trust_penalties[index]).exp().clamp(0.0, 1.0)
    }

    fn detect_shatter_event(&mut self, graph: &CausalGraph, channels: &[ChannelAuditInput]) -> bool {
        let components = graph.metrics().connected_components;
        let fragmented = graph.metrics().fragmented;
        let exceeded_envelope = channels
            .iter()
            .any(|channel| channel.measurement_slew > channel.deterministic_envelope);
        let shatter_event =
            fragmented && exceeded_envelope && (!self.previous_fragmented || components > self.previous_components);
        self.previous_fragmented = fragmented;
        self.previous_components = components;
        shatter_event
    }
}

/// Infer an initial state from the input trace.
///
/// References: `DSFB-03`, `DSFB-06`, and `CORE-10`.
pub fn infer_initial_state(trace: &TraceDocument) -> Result<DsfbState> {
    let first = trace
        .steps
        .first()
        .ok_or_else(|| anyhow::anyhow!("trace must contain at least one step"))?;
    if let Some(truth) = first.truth {
        return Ok(DsfbState::new(truth.phi, truth.omega, truth.alpha));
    }
    let phi = first.measurements.iter().sum::<f64>() / first.measurements.len() as f64;
    Ok(DsfbState::new(phi, 0.0, 0.0))
}

fn second_difference(current: f64, history: [f64; 2], dt: f64) -> f64 {
    if history == [0.0, 0.0] {
        return 0.0;
    }
    let dt2 = dt * dt;
    if dt2 <= 0.0 {
        return 0.0;
    }
    (current - 2.0 * history[1] + history[0]).abs() / dt2
}

fn is_pruned(input: &ChannelAuditInput, trust_alpha: f64) -> bool {
    input.trust_score < trust_alpha || input.measurement_slew > input.deterministic_envelope
}

fn dominant_rule_id(dsfb_pruned: bool, silent_failure: bool, shatter_event: bool) -> &'static str {
    if silent_failure {
        "CORE-08"
    } else if shatter_event {
        "CORE-04"
    } else if dsfb_pruned {
        "TMTR-01"
    } else {
        "CORE-10"
    }
}

fn state_snapshot(state: DsfbState) -> StateSnapshot {
    StateSnapshot {
        phi: state.phi,
        omega: state.omega,
        alpha: state.alpha,
    }
}
