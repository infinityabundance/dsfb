use anyhow::{ensure, Result};
use dsfb::sim::{run_simulation_trace, SimConfig};
use dsfb::DsfbParams;
use dsfb_add::aet::run_aet_sweep;
use dsfb_add::analysis::structural_law::fit_with_ci;
use dsfb_add::iwlt::run_iwlt_sweep;
use dsfb_add::SimulationConfig;

use crate::graph::{Event, EventId};

/// Deterministic per-observer trust profile used to induce non-uniform
/// trust threshold crossings without randomness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustProfile {
    Tight,
    Medium,
    Loose,
}

impl TrustProfile {
    pub fn from_observer_index(observer_index: u32) -> Self {
        match observer_index % 3 {
            0 => Self::Tight,
            1 => Self::Medium,
            _ => Self::Loose,
        }
    }

    fn trust_floor(self) -> f64 {
        match self {
            Self::Tight => 0.05,
            Self::Medium => 0.08,
            Self::Loose => 0.10,
        }
    }

    fn trust_ceiling(self) -> f64 {
        match self {
            Self::Tight => 0.70,
            Self::Medium => 0.85,
            Self::Loose => 0.95,
        }
    }

    fn growth_gain(self) -> f64 {
        match self {
            Self::Tight => 0.020,
            Self::Medium => 0.030,
            Self::Loose => 0.042,
        }
    }

    fn decay_factor(self) -> f64 {
        match self {
            Self::Tight => 0.965,
            Self::Medium => 0.975,
            Self::Loose => 0.985,
        }
    }

    fn envelope_limit(self) -> f64 {
        match self {
            Self::Tight => 0.10,
            Self::Medium => 0.20,
            Self::Loose => 0.30,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tight => "tight",
            Self::Medium => "medium",
            Self::Loose => "loose",
        }
    }
}

/// Deterministic residual state bucket used for event/edge provenance exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidualState {
    Low,
    Medium,
    High,
}

impl ResidualState {
    pub fn from_residual(residual_ema: f64) -> Self {
        if residual_ema <= 0.10 {
            Self::Low
        } else if residual_ema <= 0.30 {
            Self::Medium
        } else {
            Self::High
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "L",
            Self::Medium => "M",
            Self::High => "H",
        }
    }
}

/// Symbolic deterministic local rewrite rule identifier used for traceability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewriteRule {
    StableEnvelope,
    ModerateEnvelope,
    HighResidualRecovery,
    EnvelopeDecay,
}

impl RewriteRule {
    pub fn from_residual_state(state: ResidualState, envelope_ok: bool) -> Self {
        if !envelope_ok {
            return Self::EnvelopeDecay;
        }

        match state {
            ResidualState::Low => Self::StableEnvelope,
            ResidualState::Medium => Self::ModerateEnvelope,
            ResidualState::High => Self::HighResidualRecovery,
        }
    }

    pub fn id(self) -> u32 {
        match self {
            Self::StableEnvelope => 0,
            Self::ModerateEnvelope => 1,
            Self::HighResidualRecovery => 2,
            Self::EnvelopeDecay => 3,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::StableEnvelope => "stable_envelope",
            Self::ModerateEnvelope => "moderate_envelope",
            Self::HighResidualRecovery => "high_residual_recovery",
            Self::EnvelopeDecay => "envelope_decay",
        }
    }
}

fn initialize_profiled_trust(profile: TrustProfile, baseline_weight: f64) -> f64 {
    let blended = 0.5 * baseline_weight + 0.5 * profile.trust_floor();
    blended.clamp(profile.trust_floor(), profile.trust_ceiling())
}

/// Deterministic trust update that keeps the DSCD monotonicity condition:
/// trust only increases when the residual envelope is satisfied.
fn update_profiled_trust(
    profile: TrustProfile,
    current: f64,
    residual_ema: f64,
    envelope_ok: bool,
) -> f64 {
    let floor = profile.trust_floor();
    let ceiling = profile.trust_ceiling();

    if envelope_ok {
        let residual_factor = (1.0 / (1.0 + residual_ema.max(0.0))).clamp(0.0, 1.0);
        let proposal = current + profile.growth_gain() * residual_factor;
        proposal.max(current).clamp(floor, ceiling)
    } else {
        (current * profile.decay_factor()).clamp(floor, ceiling)
    }
}

#[derive(Debug, Clone)]
pub struct DscdObserverSample {
    pub event_id: u64,
    pub time_index: usize,
    pub observer_id: u32,
    pub trust: f64,
    pub residual_summary: f64,
    pub residual_state: ResidualState,
    pub rewrite_rule_id: u32,
    pub rewrite_rule_label: &'static str,
    pub trust_profile: TrustProfile,
    pub envelope_ok: bool,
}

#[derive(Debug, Clone)]
pub struct DscdEventBatch {
    pub events: Vec<Event>,
    pub observer_samples: Vec<DscdObserverSample>,
}

#[derive(Debug, Clone, Copy)]
pub struct StructuralGrowthSummary {
    pub s_infty: f64,
    pub law_slope: f64,
}

pub fn generate_dscd_events_from_dsfb(
    scenario: &SimConfig,
    dsfb_params: DsfbParams,
    num_events: usize,
) -> Result<DscdEventBatch> {
    ensure!(num_events > 0, "num_events must be greater than zero");

    let mut run_cfg = scenario.clone();
    run_cfg.steps = num_events;

    let trace = run_simulation_trace(run_cfg, dsfb_params);
    let mut events = Vec::with_capacity(trace.len());
    let mut observer_samples = Vec::new();
    let channels = trace
        .first()
        .map(|step| step.trust_stats.len())
        .unwrap_or(0);
    let mut trust_state = vec![0.0; channels];

    if let Some(first_step) = trace.first() {
        for (observer_index, trust_slot) in trust_state.iter_mut().enumerate().take(channels) {
            let baseline = first_step
                .trust_stats
                .get(observer_index)
                .map(|stats| stats.weight)
                .unwrap_or(1.0 / channels.max(1) as f64);
            let profile = TrustProfile::from_observer_index(observer_index as u32);
            *trust_slot = initialize_profiled_trust(profile, baseline);
        }
    }

    for step in trace {
        let structural_tag = if step.trust_stats.is_empty() {
            None
        } else {
            Some(
                step.trust_stats
                    .iter()
                    .map(|stats| stats.residual_ema)
                    .sum::<f64>()
                    / step.trust_stats.len() as f64,
            )
        };

        events.push(Event {
            id: EventId(step.step as u64),
            timestamp: Some(step.t),
            structural_tag,
        });

        for (observer_id, stats) in step.trust_stats.iter().enumerate() {
            let observer_id_u32 = observer_id as u32;
            let profile = TrustProfile::from_observer_index(observer_id_u32);
            let residual_state = ResidualState::from_residual(stats.residual_ema);
            let envelope_ok = stats.residual_ema <= profile.envelope_limit();
            let next_trust = update_profiled_trust(
                profile,
                trust_state[observer_id],
                stats.residual_ema,
                envelope_ok,
            );
            trust_state[observer_id] = next_trust;
            let rewrite_rule = RewriteRule::from_residual_state(residual_state, envelope_ok);

            observer_samples.push(DscdObserverSample {
                event_id: step.step as u64,
                time_index: step.step,
                observer_id: observer_id_u32,
                trust: next_trust,
                residual_summary: stats.residual_ema,
                residual_state,
                rewrite_rule_id: rewrite_rule.id(),
                rewrite_rule_label: rewrite_rule.as_str(),
                trust_profile: profile,
                envelope_ok,
            });
        }
    }

    Ok(DscdEventBatch {
        events,
        observer_samples,
    })
}

pub fn compute_structural_growth_for_dscd(
    add_cfg: &SimulationConfig,
) -> Result<StructuralGrowthSummary> {
    add_cfg.validate()?;

    let lambda_grid = add_cfg.lambda_grid();
    let aet = run_aet_sweep(add_cfg, &lambda_grid)?;
    let iwlt = run_iwlt_sweep(add_cfg, &lambda_grid)?;
    let fit = fit_with_ci(&aet.echo_slope, &iwlt.entropy_density)?;

    let s_infty = iwlt
        .entropy_density
        .iter()
        .copied()
        .reduce(f64::max)
        .unwrap_or(0.0);

    Ok(StructuralGrowthSummary {
        s_infty,
        law_slope: fit.slope,
    })
}
