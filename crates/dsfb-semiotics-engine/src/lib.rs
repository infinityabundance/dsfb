pub mod cli;
pub mod engine;
pub mod figures;
pub mod io;
pub mod math;
pub mod report;
pub mod sim;

pub use cli::args::{CliArgs, ScenarioSelection};
pub use engine::pipeline::{
    export_artifacts, run_all_demos, run_scenario, EngineConfig, StructuralSemioticsEngine,
};
pub use engine::types::{
    AdmissibilityEnvelope, CoordinatedResidualStructure, DetectabilityResult, EngineOutputBundle,
    GrammarStatus, ObservedTrajectory, PredictedTrajectory, ReportManifest, ResidualTrajectory,
    SemanticMatchResult, SignTrajectory,
};
