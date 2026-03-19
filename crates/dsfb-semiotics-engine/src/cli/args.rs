use std::path::PathBuf;

use clap::{CommandFactory, Parser, ValueEnum, ValueHint};

use crate::engine::types::EnvelopeMode;

#[derive(Clone, Debug)]
pub enum ScenarioSelection {
    All,
    Single(String),
    Csv(CsvInputConfig),
}

#[derive(Clone, Debug)]
pub struct CsvInputConfig {
    pub observed_csv: PathBuf,
    pub predicted_csv: PathBuf,
    pub scenario_id: String,
    pub channel_names: Option<Vec<String>>,
    pub envelope_mode: EnvelopeMode,
    pub envelope_base: f64,
    pub envelope_slope: f64,
    pub envelope_switch_step: Option<usize>,
    pub envelope_secondary_slope: Option<f64>,
    pub envelope_secondary_base: Option<f64>,
    pub envelope_name: String,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum EnvelopeModeArg {
    Fixed,
    Widening,
    Tightening,
    RegimeSwitched,
}

impl From<EnvelopeModeArg> for EnvelopeMode {
    fn from(value: EnvelopeModeArg) -> Self {
        match value {
            EnvelopeModeArg::Fixed => EnvelopeMode::Fixed,
            EnvelopeModeArg::Widening => EnvelopeMode::Widening,
            EnvelopeModeArg::Tightening => EnvelopeMode::Tightening,
            EnvelopeModeArg::RegimeSwitched => EnvelopeMode::RegimeSwitched,
        }
    }
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
        value_hint = ValueHint::FilePath,
        help = "CSV ingestion mode: observed trajectory CSV with headers including time and channel columns"
    )]
    pub observed_csv: Option<PathBuf>,

    #[arg(
        long,
        value_hint = ValueHint::FilePath,
        help = "CSV ingestion mode: predicted trajectory CSV with headers including time and channel columns"
    )]
    pub predicted_csv: Option<PathBuf>,

    #[arg(
        long,
        default_value = "csv_ingest",
        help = "Scenario identifier used when running the CSV ingestion path"
    )]
    pub input_id: String,

    #[arg(
        long,
        help = "Optional comma-separated channel names to override CSV headers"
    )]
    pub channel_names: Option<String>,

    #[arg(
        long,
        default_value_t = EnvelopeModeArg::Fixed,
        value_enum,
        help = "Envelope mode for CSV ingestion"
    )]
    pub envelope_mode: EnvelopeModeArg,

    #[arg(
        long,
        default_value_t = 1.0,
        help = "Base envelope radius for CSV ingestion mode"
    )]
    pub envelope_base: f64,

    #[arg(
        long,
        default_value_t = 0.0,
        help = "Primary envelope slope for CSV ingestion mode"
    )]
    pub envelope_slope: f64,

    #[arg(
        long,
        help = "Optional switch step for CSV ingestion with regime-switched envelopes"
    )]
    pub envelope_switch_step: Option<usize>,

    #[arg(
        long,
        help = "Optional secondary slope for CSV ingestion with regime-switched envelopes"
    )]
    pub envelope_secondary_slope: Option<f64>,

    #[arg(
        long,
        help = "Optional secondary base for CSV ingestion with regime-switched envelopes"
    )]
    pub envelope_secondary_base: Option<f64>,

    #[arg(
        long,
        default_value = "csv_ingest_envelope",
        help = "Envelope name used in CSV ingestion mode"
    )]
    pub envelope_name: String,

    #[arg(
        long,
        value_hint = ValueHint::DirPath,
        help = "Override the output root directory; a fresh timestamped folder is still created beneath it"
    )]
    pub output_dir: Option<PathBuf>,

    #[arg(long, default_value_t = 123, help = "Deterministic scenario seed")]
    pub seed: u64,

    #[arg(long, default_value_t = 240, help = "Number of steps per synthetic scenario")]
    pub steps: usize,

    #[arg(long, default_value_t = 1.0, help = "Sample interval for synthetic scenarios")]
    pub dt: f64,
}

impl CliArgs {
    pub fn parse_args() -> Self {
        let args = Self::parse();
        let csv_mode = args.observed_csv.is_some() || args.predicted_csv.is_some();

        if args.all && args.scenario.is_some() {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--all and --scenario are mutually exclusive",
                )
                .exit();
        }
        if csv_mode && args.all {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--all and CSV ingestion flags are mutually exclusive",
                )
                .exit();
        }
        if csv_mode && args.scenario.is_some() {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--scenario and CSV ingestion flags are mutually exclusive",
                )
                .exit();
        }
        if args.observed_csv.is_some() ^ args.predicted_csv.is_some() {
            Self::command()
                .error(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "--observed-csv and --predicted-csv must be provided together",
                )
                .exit();
        }

        args
    }

    pub fn selection(&self) -> ScenarioSelection {
        if let (Some(observed_csv), Some(predicted_csv)) =
            (&self.observed_csv, &self.predicted_csv)
        {
            ScenarioSelection::Csv(CsvInputConfig {
                observed_csv: observed_csv.clone(),
                predicted_csv: predicted_csv.clone(),
                scenario_id: self.input_id.clone(),
                channel_names: self.channel_names.as_deref().map(parse_channel_names),
                envelope_mode: self.envelope_mode.into(),
                envelope_base: self.envelope_base,
                envelope_slope: self.envelope_slope,
                envelope_switch_step: self.envelope_switch_step,
                envelope_secondary_slope: self.envelope_secondary_slope,
                envelope_secondary_base: self.envelope_secondary_base,
                envelope_name: self.envelope_name.clone(),
            })
        } else if let Some(scenario) = &self.scenario {
            ScenarioSelection::Single(scenario.clone())
        } else {
            ScenarioSelection::All
        }
    }
}

fn parse_channel_names(spec: &str) -> Vec<String> {
    spec.split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
