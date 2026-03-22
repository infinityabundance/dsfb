pub mod cli;
pub mod config;
pub mod dsfb;
pub mod error;
pub mod frame;
pub mod metrics;
pub mod pipeline;
pub mod plots;
pub mod report;
pub mod scene;
pub mod taa;

pub use config::{DemoConfig, SceneConfig};
pub use error::{Error, Result};
pub use pipeline::{generate_scene_artifacts, run_demo_a, DemoAArtifacts};
