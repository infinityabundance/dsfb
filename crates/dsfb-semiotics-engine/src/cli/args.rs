use std::path::PathBuf;

use clap::{CommandFactory, Parser, ValueHint};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScenarioSelection {
    All,
    Single(String),
}

#[derive(Clone, Debug, Parser)]
#[command(
    author,
    version,
    about = "Deterministic structural semiotics engine with reproducible figures, reports, and archive outputs"
)]
pub struct CliArgs {
    #[arg(long, help = "Run all paper-aligned synthetic demonstrations")]
    pub all: bool,

    #[arg(
        long,
        value_name = "SCENARIO_ID",
        help = "Run one named scenario or experiment case"
    )]
    pub scenario: Option<String>,

    #[arg(
        long,
        value_hint = ValueHint::DirPath,
        help = "Override the output root directory; a fresh timestamped folder is still created beneath it"
    )]
    pub output_dir: Option<PathBuf>,

    #[arg(long, default_value_t = 123, help = "Deterministic scenario seed")]
    pub seed: u64,

    #[arg(long, default_value_t = 240, help = "Number of steps per scenario")]
    pub steps: usize,

    #[arg(long, default_value_t = 1.0, help = "Sample interval")]
    pub dt: f64,
}

impl CliArgs {
    pub fn parse_args() -> Self {
        let args = Self::parse();
        if args.all && args.scenario.is_some() {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--all and --scenario are mutually exclusive",
                )
                .exit();
        }
        args
    }

    pub fn selection(&self) -> ScenarioSelection {
        if let Some(scenario) = &self.scenario {
            ScenarioSelection::Single(scenario.clone())
        } else {
            ScenarioSelection::All
        }
    }
}

impl CliArgs {
    pub fn selected_scenario(&self) -> Option<&str> {
        self.scenario.as_deref()
    }
}
