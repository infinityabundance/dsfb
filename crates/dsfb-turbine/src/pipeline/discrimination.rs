//! P1: FD003 multi-fault discrimination analysis.
//!
//! FD003 contains two fault modes: HPC degradation and fan degradation.
//! This module analyzes whether DSFB grammar trajectories and reason
//! codes differ between the two fault populations.
//!
//! Since C-MAPSS does not label which engines have which fault mode,
//! we use the known discriminative sensors:
//! - Fan degradation primarily affects fan speed (s8/Nf), bypass ratio (s15/BPR)
//! - HPC degradation primarily affects HPC outlet temp (s3/T30), HPC pressure (s7/P30)
//!
//! We cluster engines by which channels trigger Boundary first, and check
//! whether the resulting clusters show different grammar-state trajectory
//! patterns.

use crate::pipeline::engine_eval::EngineEvalResult;
use crate::core::channels::ChannelId;
use crate::core::grammar::GrammarState;
use std::fmt::Write;

/// Discrimination result for FD003.
#[derive(Debug)]
pub struct DiscriminationResult {
    /// Total engines.
    pub total_engines: usize,
    /// Engines where HPC-related channels triggered Boundary first.
    pub hpc_primary_count: usize,
    /// Engines where fan-related channels triggered Boundary first.
    pub fan_primary_count: usize,
    /// Engines where both triggered simultaneously or ambiguously.
    pub ambiguous_count: usize,
    /// HPC-primary engines whose aggregate trajectory reaches Violation.
    pub hpc_violation_count: usize,
    /// Fan-primary engines whose aggregate trajectory reaches Violation.
    pub fan_violation_count: usize,
    /// Ambiguous engines whose aggregate trajectory reaches Violation.
    pub ambiguous_violation_count: usize,
    /// Mean lead time for HPC-primary engines.
    pub hpc_mean_lead: f64,
    /// Mean lead time for fan-primary engines.
    pub fan_mean_lead: f64,
    /// Per-engine classification.
    pub classifications: Vec<(u16, FaultCluster)>,
}

/// Which fault mode appears dominant based on channel triggering order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultCluster {
    /// HPC-related channels triggered first.
    HpcPrimary,
    /// Fan-related channels triggered first.
    FanPrimary,
    /// Ambiguous or simultaneous.
    Ambiguous,
}

/// HPC-indicative channels.
const HPC_CHANNELS: &[ChannelId] = &[
    ChannelId::TempHpcOutlet,      // T30
    ChannelId::PressureHpcOutlet,  // P30
    ChannelId::StaticPressureHpc,  // Ps30
    ChannelId::FuelFlowRatio,      // phi
];

/// Fan-indicative channels.
const FAN_CHANNELS: &[ChannelId] = &[
    ChannelId::FanSpeed,           // Nf
    ChannelId::CorrectedFanSpeed,  // NRf
    ChannelId::BypassRatio,        // BPR
];

/// Classifies one engine by which channel group triggered Boundary first.
fn classify_engine(result: &EngineEvalResult) -> FaultCluster {
    let mut earliest_hpc: Option<u32> = None;
    let mut earliest_fan: Option<u32> = None;

    for (ch, fb) in &result.channel_boundary_cycles {
        if let Some(cycle) = fb {
            let is_hpc = HPC_CHANNELS.contains(ch);
            let is_fan = FAN_CHANNELS.contains(ch);

            if is_hpc {
                earliest_hpc = Some(earliest_hpc.map_or(*cycle, |c: u32| c.min(*cycle)));
            }
            if is_fan {
                earliest_fan = Some(earliest_fan.map_or(*cycle, |c: u32| c.min(*cycle)));
            }
        }
    }

    match (earliest_hpc, earliest_fan) {
        (Some(h), Some(f)) => {
            if h < f { FaultCluster::HpcPrimary }
            else if f < h { FaultCluster::FanPrimary }
            else { FaultCluster::Ambiguous }
        }
        (Some(_), None) => FaultCluster::HpcPrimary,
        (None, Some(_)) => FaultCluster::FanPrimary,
        (None, None) => FaultCluster::Ambiguous,
    }
}

/// Returns the most severe aggregate grammar state reached by an engine.
fn peak_grammar_state(result: &EngineEvalResult) -> GrammarState {
    let mut peak = GrammarState::Admissible;
    for state in &result.grammar_trajectory {
        if state.severity() > peak.severity() {
            peak = *state;
        }
    }
    peak
}

/// Runs FD003 multi-fault discrimination analysis.
pub fn analyze_discrimination(results: &[EngineEvalResult]) -> DiscriminationResult {
    let mut classifications = Vec::with_capacity(results.len());
    let mut hpc_count = 0usize;
    let mut fan_count = 0usize;
    let mut ambiguous_count = 0usize;
    let mut hpc_violation_count = 0usize;
    let mut fan_violation_count = 0usize;
    let mut ambiguous_violation_count = 0usize;
    let mut hpc_lead_sum = 0.0f64;
    let mut hpc_lead_n = 0usize;
    let mut fan_lead_sum = 0.0f64;
    let mut fan_lead_n = 0usize;

    for result in results {
        let cluster = classify_engine(result);
        let peak_state = peak_grammar_state(result);
        classifications.push((result.unit, cluster));

        match cluster {
            FaultCluster::HpcPrimary => {
                hpc_count += 1;
                if peak_state == GrammarState::Violation {
                    hpc_violation_count += 1;
                }
                if let Some(lt) = result.structural_lead_time {
                    hpc_lead_sum += lt as f64;
                    hpc_lead_n += 1;
                }
            }
            FaultCluster::FanPrimary => {
                fan_count += 1;
                if peak_state == GrammarState::Violation {
                    fan_violation_count += 1;
                }
                if let Some(lt) = result.structural_lead_time {
                    fan_lead_sum += lt as f64;
                    fan_lead_n += 1;
                }
            }
            FaultCluster::Ambiguous => {
                ambiguous_count += 1;
                if peak_state == GrammarState::Violation {
                    ambiguous_violation_count += 1;
                }
            }
        }
    }

    DiscriminationResult {
        total_engines: results.len(),
        hpc_primary_count: hpc_count,
        fan_primary_count: fan_count,
        ambiguous_count,
        hpc_violation_count,
        fan_violation_count,
        ambiguous_violation_count,
        hpc_mean_lead: if hpc_lead_n > 0 { hpc_lead_sum / hpc_lead_n as f64 } else { 0.0 },
        fan_mean_lead: if fan_lead_n > 0 { fan_lead_sum / fan_lead_n as f64 } else { 0.0 },
        classifications,
    }
}

/// Formats the discrimination result as text.
pub fn discrimination_report(dr: &DiscriminationResult) -> String {
    let mut out = String::with_capacity(4096);
    let _ = writeln!(out, "── P1: FD003 Multi-Fault Discrimination Analysis ─────────────");
    let _ = writeln!(out, "  FD003 contains TWO fault modes: HPC degradation + Fan degradation.");
    let _ = writeln!(out, "  DSFB classifies engines by which channel group triggers Boundary first:");
    let _ = writeln!(out, "    HPC indicators: T30, P30, Ps30, phi");
    let _ = writeln!(out, "    Fan indicators: Nf, NRf, BPR");
    let _ = writeln!(out);
    let _ = writeln!(out, "  Total engines:      {}", dr.total_engines);
    let _ = writeln!(out, "  HPC-primary:        {} ({:.1}%)", dr.hpc_primary_count,
        100.0 * dr.hpc_primary_count as f64 / dr.total_engines.max(1) as f64);
    let _ = writeln!(out, "  Fan-primary:        {} ({:.1}%)", dr.fan_primary_count,
        100.0 * dr.fan_primary_count as f64 / dr.total_engines.max(1) as f64);
    let _ = writeln!(out, "  Ambiguous:          {} ({:.1}%)", dr.ambiguous_count,
        100.0 * dr.ambiguous_count as f64 / dr.total_engines.max(1) as f64);
    let _ = writeln!(out);
    let _ = writeln!(out, "  Mean lead time (HPC-primary): {:.1} cycles", dr.hpc_mean_lead);
    let _ = writeln!(out, "  Mean lead time (Fan-primary): {:.1} cycles", dr.fan_mean_lead);
    let _ = writeln!(out);
    let _ = writeln!(out, "  Aggregate trajectory severity summary:");
    let _ = writeln!(out, "    HPC-primary reaching Violation: {}/{}",
        dr.hpc_violation_count, dr.hpc_primary_count);
    let _ = writeln!(out, "    Fan-primary reaching Violation: {}/{}",
        dr.fan_violation_count, dr.fan_primary_count);
    let _ = writeln!(out, "    Ambiguous reaching Violation:   {}/{}",
        dr.ambiguous_violation_count, dr.ambiguous_count);
    let _ = writeln!(out);
    let _ = writeln!(out, "  Interpretation:");
    let _ = writeln!(out, "    DSFB does NOT diagnose fault mode. It observes which residual");
    let _ = writeln!(out, "    channels show structural deviation first. The fact that different");
    let _ = writeln!(out, "    engines trigger different channel groups first is consistent with");
    let _ = writeln!(out, "    the presence of two distinct degradation mechanisms in FD003.");
    let _ = writeln!(out, "    This structural discrimination is information that scalar");
    let _ = writeln!(out, "    RUL-threshold alarms do not provide.");
    let _ = writeln!(out);
    let _ = writeln!(out, "  Non-claim: This analysis does not prove fault-mode identification.");
    let _ = writeln!(out, "  It shows that DSFB grammar trajectories carry structural information");
    let _ = writeln!(out, "  that is consistent with the known fault-mode diversity of FD003.");

    // Per-engine listing (first 20)
    let _ = writeln!(out);
    let _ = writeln!(out, "  Per-engine classifications (first 20):");
    for (unit, cluster) in dr.classifications.iter().take(20) {
        let label = match cluster {
            FaultCluster::HpcPrimary => "HPC-primary",
            FaultCluster::FanPrimary => "Fan-primary",
            FaultCluster::Ambiguous => "Ambiguous",
        };
        let _ = writeln!(out, "    Unit {:3}: {}", unit, label);
    }

    out
}
