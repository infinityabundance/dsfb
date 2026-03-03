//! Deterministic Structural Causal Dynamics (DSCD) layered on DSFB + ADD.
//!
//! This crate provides two layers:
//! - legacy sweep utilities used by existing notebooks/binaries in this repo,
//! - paper-facing deterministic simulation APIs (`DscdConfig`,
//!   `run_dscd_simulation`) for threshold scaling, provenance, and figure CSVs.
//!
//! All DSCD pipelines are deterministic and reproducible.

pub mod config;
pub mod graph;
pub mod integrations;
pub mod paper;
pub mod sweep;

pub use config::{
    create_timestamped_output_dir, create_timestamped_output_dir_in, workspace_root_dir,
    DscdScalingConfig, DscdSweepConfig, OutputPaths,
};
pub use graph::{
    add_trust_gated_edge, add_trust_gated_edge_with_provenance,
    expansion_ratio as legacy_expansion_ratio, reachable_from, DscdEdge as LegacyDscdEdge,
    DscdGraph as LegacyDscdGraph, Event, EventId,
};
pub use integrations::{
    compute_structural_growth_for_dscd, generate_dscd_events_from_dsfb, DscdEventBatch,
    DscdObserverSample, ResidualState, RewriteRule, StructuralGrowthSummary, TrustProfile,
};
pub use paper::{
    build_graph_for_tau, compute_reachable_component_size, create_dscd_run_dir, expansion_ratio,
    linspace, run_dscd_simulation, DscdConfig, DscdEdge, DscdError, DscdEvent, DscdGraph,
};
pub use sweep::{
    build_graph_from_samples, export_edge_provenance_by_edge_id,
    export_edge_provenance_by_endpoints, run_threshold_scaling, run_trust_threshold_sweep,
    ThresholdRecord,
};
