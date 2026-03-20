#![forbid(unsafe_code)]
//! Deterministic structural-semiotics engine for residual-based meaning extraction.

pub mod cli;
pub mod engine;
pub mod evaluation;
pub mod figures;
pub mod io;
pub mod math;
pub mod report;
pub mod sim;

pub use cli::args::{CliArgs, ScenarioSelection};
pub use engine::bank::{
    BankSourceKind, HeuristicBankRegistry, HeuristicBankValidationReport, LoadedBankDescriptor,
};
pub use engine::config::{
    BankRunConfig, BankSourceConfig, CommonRunConfig, CsvRunConfig, SyntheticRunConfig,
    SyntheticSelection, DEFAULT_DT, DEFAULT_SEED, DEFAULT_STEPS,
};
pub use engine::pipeline::{
    export_artifacts, run_all_demos, run_scenario, EngineConfig, StructuralSemioticsEngine,
};
pub use engine::settings::{
    EngineSettings, EvaluationSettings, ReportingSettings, SemanticRetrievalSettings,
    SyntaxThresholds,
};
pub use engine::types::{
    AdmissibilityEnvelope, CoordinatedResidualStructure, DetectabilityResult, EngineOutputBundle,
    GrammarStatus, ObservedTrajectory, PredictedTrajectory, ReportManifest, ResidualTrajectory,
    SemanticMatchResult, SignTrajectory,
};
pub use io::schema::{ARTIFACT_SCHEMA_VERSION, HEURISTIC_BANK_SCHEMA_VERSION};
