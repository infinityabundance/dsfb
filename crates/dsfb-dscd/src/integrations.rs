use anyhow::{ensure, Result};
use dsfb::sim::{run_simulation_trace, SimConfig};
use dsfb::DsfbParams;
use dsfb_add::aet::run_aet_sweep;
use dsfb_add::analysis::structural_law::fit_with_ci;
use dsfb_add::iwlt::run_iwlt_sweep;
use dsfb_add::SimulationConfig;

use crate::graph::{Event, EventId};

#[derive(Debug, Clone)]
pub struct DscdObserverSample {
    pub event_id: u64,
    pub observer_id: u32,
    pub trust: f64,
    pub residual_summary: f64,
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
            observer_samples.push(DscdObserverSample {
                event_id: step.step as u64,
                observer_id: observer_id as u32,
                trust: stats.weight,
                residual_summary: stats.residual_ema,
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
