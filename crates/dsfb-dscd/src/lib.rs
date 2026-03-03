pub mod config;
pub mod graph;
pub mod integrations;
pub mod sweep;

pub use config::{
    create_timestamped_output_dir, create_timestamped_output_dir_in, workspace_root_dir,
    DscdScalingConfig, DscdSweepConfig, OutputPaths,
};
pub use graph::{
    add_trust_gated_edge, add_trust_gated_edge_with_provenance, expansion_ratio, reachable_from,
    DscdEdge, DscdGraph, Event, EventId,
};
pub use integrations::{
    compute_structural_growth_for_dscd, generate_dscd_events_from_dsfb, DscdEventBatch,
    DscdObserverSample, ResidualState, RewriteRule, StructuralGrowthSummary, TrustProfile,
};
pub use sweep::{
    build_graph_from_samples, export_edge_provenance_by_edge_id,
    export_edge_provenance_by_endpoints, run_threshold_scaling, run_trust_threshold_sweep,
    ThresholdRecord,
};
