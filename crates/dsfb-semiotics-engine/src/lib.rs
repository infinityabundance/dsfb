#![forbid(unsafe_code)]
//! Deterministic structural-semiotics engine for residual-based meaning extraction.

pub mod cli;
pub mod dashboard;
pub mod demos;
pub mod engine;
pub mod evaluation;
pub mod figures;
pub mod io;
pub mod live;
pub mod math;
pub mod report;
pub mod sim;

pub use cli::args::{CliArgs, ScenarioSelection};
pub use dashboard::{
    DashboardReplay, DashboardReplayConfig, DashboardReplayEvent, DashboardReplayStream,
    DASHBOARD_EVENT_SCHEMA_VERSION,
};
pub use engine::bank::{
    BankSourceKind, HeuristicBankRegistry, HeuristicBankValidationReport, LoadedBankDescriptor,
};
pub use engine::config::{
    BankRunConfig, BankSourceConfig, BankValidationMode, CommonRunConfig, CsvRunConfig,
    SyntheticRunConfig, SyntheticSelection, DEFAULT_DT, DEFAULT_SEED, DEFAULT_STEPS,
};
pub use engine::pipeline::{
    export_artifacts, run_all_demos, run_scenario, EngineConfig, StructuralSemioticsEngine,
};
pub use engine::settings::{
    EngineSettings, EvaluationSettings, OnlineEngineSettings, PlottingSettings, ReportingSettings,
    RetrievalIndexSettings, SemanticRetrievalSettings, SmoothingMode, SmoothingSettings,
    SyntaxThresholds,
};
pub use engine::types::{
    AdmissibilityEnvelope, CoordinatedResidualStructure, DetectabilityResult, EngineOutputBundle,
    GrammarReasonCode, GrammarState, GrammarStatus, ObservedTrajectory, PredictedTrajectory,
    ReportManifest, ResidualTrajectory, SemanticMatchResult, SignTrajectory, TrustScalar,
};
pub use io::schema::{ARTIFACT_SCHEMA_VERSION, HEURISTIC_BANK_SCHEMA_VERSION};
pub use live::{
    numeric_mode_label, LiveEngineStatus, OnlineStructuralEngine, Real, RingBuffer,
    LIVE_ENGINE_STATUS_SCHEMA_VERSION,
};
