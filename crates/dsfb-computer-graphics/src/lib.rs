pub mod cli;
pub mod config;
pub mod cost;
pub mod datasets;
pub mod dsfb;
pub mod error;
pub mod external;
pub mod external_validation;
pub mod frame;
pub mod gpu;
pub mod gpu_execution;
pub mod host;
pub mod metrics;
pub mod outputs;
pub mod parameters;
pub mod pipeline;
pub mod plots;
pub mod report;
pub mod sampling;
pub mod scaling;
pub mod scene;
pub mod sensitivity;
pub mod taa;
pub mod timing;

pub use config::{DemoConfig, SceneConfig};
pub use error::{Error, Result};
pub use outputs::{
    artifact_manifest_path, create_named_run_dir, create_timestamped_run_dir,
    format_run_directory_name, format_zip_bundle_name, pdf_bundle_path, zip_bundle_path, RunLayout,
    ARTIFACT_MANIFEST_FILE_NAME, NOTEBOOK_OUTPUT_ROOT_NAME, PDF_BUNDLE_FILE_NAME,
};
pub use pipeline::{
    export_evaluator_handoff, export_minimal_report, generate_scene_artifacts,
    import_external_buffers, parse_scenario_id, run_all, run_all_filtered, run_demo_a,
    run_demo_a_filtered, run_demo_b, run_demo_b_efficiency_only, run_demo_b_filtered,
    run_external_replay_only, run_gpu_path_only, run_realism_bridge_only, run_realism_suite_only,
    run_resolution_scaling_only, run_sensitivity_only, run_timing_only,
    scenario_definitions_for_filter, validate_artifact_bundle, validate_final_bundle,
    DemoAArtifacts, DemoBArtifacts, RunAllArtifacts,
};
