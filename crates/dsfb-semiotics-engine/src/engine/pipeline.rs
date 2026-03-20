//! Public deterministic pipeline facade.
//!
//! This module intentionally stays small. Core run orchestration lives in
//! `crate::engine::pipeline_core`, artifact assembly/export lives in
//! `crate::engine::pipeline_artifacts`, and reproducibility aggregation lives in
//! `crate::engine::pipeline_evaluation`.

pub use crate::engine::pipeline_artifacts::{export_artifacts, ExportedArtifacts};
pub use crate::engine::pipeline_core::{
    run_all_demos, run_scenario, EngineConfig, StructuralSemioticsEngine,
};
