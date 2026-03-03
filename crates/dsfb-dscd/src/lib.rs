pub mod config;
pub mod graph;
pub mod integrations;
pub mod sweep;

pub use config::{create_timestamped_output_dir, workspace_root_dir, DscdSweepConfig, OutputPaths};
pub use graph::{
    add_trust_gated_edge, expansion_ratio, reachable_from, DscdEdge, DscdGraph, Event, EventId,
};
pub use integrations::{
    compute_structural_growth_for_dscd, generate_dscd_events_from_dsfb, DscdEventBatch,
    DscdObserverSample, StructuralGrowthSummary,
};
pub use sweep::{build_graph_from_samples, run_trust_threshold_sweep, ThresholdRecord};
