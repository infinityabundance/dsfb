use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{CommandFactory, Parser, ValueEnum, ValueHint};

use crate::engine::config::{BankRunConfig, BankSourceConfig};
use crate::engine::types::EnvelopeMode;
use crate::evaluation::sweeps::{SweepConfig, SweepFamily};
use crate::math::envelope::EnvelopeSpec;

/// User-facing run selection resolved from CLI arguments.
#[derive(Clone, Debug)]
pub enum ScenarioSelection {
    All,
    Single(String),
    Csv(CsvInputConfig),
    Sweep(SweepConfig),
}

/// Validated CSV ingestion settings used by the library and CLI.
#[derive(Clone, Debug)]
pub struct CsvInputConfig {
    pub observed_csv: PathBuf,
    pub predicted_csv: PathBuf,
    pub scenario_id: String,
    pub channel_names: Option<Vec<String>>,
    pub time_column: Option<String>,
    pub dt_fallback: f64,
    pub envelope_mode: EnvelopeMode,
    pub envelope_base: f64,
    pub envelope_slope: f64,
    pub envelope_switch_step: Option<usize>,
    pub envelope_secondary_slope: Option<f64>,
    pub envelope_secondary_base: Option<f64>,
    pub envelope_name: String,
}

impl CsvInputConfig {
    /// Validates the CSV ingestion request and returns a typed envelope spec for downstream use.
    pub fn envelope_spec(&self) -> Result<EnvelopeSpec> {
        self.validate()?;
        let spec = EnvelopeSpec {
            name: self.envelope_name.clone(),
            mode: self.envelope_mode,
            base_radius: self.envelope_base,
            slope: self.envelope_slope,
            switch_step: self.envelope_switch_step,
            secondary_slope: self.envelope_secondary_slope,
            secondary_base: self.envelope_secondary_base,
        };
        spec.validate()?;
        Ok(spec)
    }

    /// Validates the CSV ingestion request without touching the filesystem.
    pub fn validate(&self) -> Result<()> {
        if self.scenario_id.trim().is_empty() {
            return Err(anyhow!(
                "CSV ingestion requires a non-empty scenario identifier"
            ));
        }
        if !self.dt_fallback.is_finite() || self.dt_fallback <= 0.0 {
            return Err(anyhow!(
                "CSV ingestion requires a positive finite dt fallback; got {}",
                self.dt_fallback
            ));
        }
        if let Some(channel_names) = &self.channel_names {
            if channel_names.is_empty() {
                return Err(anyhow!(
                    "CSV channel-name override must not be empty when supplied"
                ));
            }
            if channel_names.iter().any(|name| name.trim().is_empty()) {
                return Err(anyhow!(
                    "CSV channel-name override must not contain empty names"
                ));
            }
        }
        if matches!(self.envelope_mode, EnvelopeMode::RegimeSwitched)
            && (self.envelope_switch_step.is_none()
                || self.envelope_secondary_slope.is_none()
                || self.envelope_secondary_base.is_none())
        {
            return Err(anyhow!(
                "CSV regime-switched envelopes require --envelope-switch-step, --envelope-secondary-slope, and --envelope-secondary-base"
            ));
        }
        Ok(())
    }
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

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum InputModeArg {
    Synthetic,
    Csv,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum BankSourceArg {
    Builtin,
    External,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum SweepFamilyArg {
    GradualDriftSlope,
    CurvatureOnsetTiming,
    SpikeMagnitudeDuration,
    OscillationAmplitudeFrequency,
    CoordinatedRiseStrength,
    EnvelopeTightness,
}

impl From<SweepFamilyArg> for SweepFamily {
    fn from(value: SweepFamilyArg) -> Self {
        match value {
            SweepFamilyArg::GradualDriftSlope => SweepFamily::GradualDriftSlope,
            SweepFamilyArg::CurvatureOnsetTiming => SweepFamily::CurvatureOnsetTiming,
            SweepFamilyArg::SpikeMagnitudeDuration => SweepFamily::SpikeMagnitudeDuration,
            SweepFamilyArg::OscillationAmplitudeFrequency => {
                SweepFamily::OscillationAmplitudeFrequency
            }
            SweepFamilyArg::CoordinatedRiseStrength => SweepFamily::CoordinatedRiseStrength,
            SweepFamilyArg::EnvelopeTightness => SweepFamily::EnvelopeTightness,
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
        value_enum,
        help = "Optional input mode selector. Use `csv` to require external CSV ingestion flags, or `synthetic` to reject them explicitly."
    )]
    pub input_mode: Option<InputModeArg>,

    #[arg(
        long,
        value_enum,
        default_value_t = BankSourceArg::Builtin,
        help = "Heuristic-bank source selection. Use `external` together with --bank-path to load a validated bank artifact instead of the compiled builtin bank."
    )]
    pub bank_source: BankSourceArg,

    #[arg(
        long,
        value_hint = ValueHint::FilePath,
        help = "Path to an external heuristic-bank JSON artifact. Only valid together with --bank-source external."
    )]
    pub bank_path: Option<PathBuf>,

    #[arg(
        long,
        help = "Require symmetric compatibility and incompatibility graph links during heuristic-bank validation."
    )]
    pub strict_bank_validation: bool,

    #[arg(
        long,
        value_enum,
        help = "Run a deterministic synthetic parameter sweep instead of the default scenario catalog"
    )]
    pub sweep_family: Option<SweepFamilyArg>,

    #[arg(
        long,
        default_value_t = 0,
        help = "Optional sweep point count override. Zero uses the deterministic engine default."
    )]
    pub sweep_points: usize,

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
        long = "scenario-id",
        alias = "run-id",
        alias = "input-id",
        default_value = "csv_ingest",
        help = "Scenario identifier used when running the CSV ingestion path"
    )]
    pub scenario_id: String,

    #[arg(
        long,
        help = "Optional comma-separated channel names to override CSV headers"
    )]
    pub channel_names: Option<String>,

    #[arg(
        long,
        help = "Optional time column name for CSV ingestion. If omitted, the loader uses a `time` column when present and otherwise synthesizes sample times from row order and --dt."
    )]
    pub time_column: Option<String>,

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

    #[arg(
        long,
        default_value_t = 240,
        help = "Number of steps per synthetic scenario"
    )]
    pub steps: usize,

    #[arg(
        long,
        default_value_t = 1.0,
        help = "Sample interval for synthetic scenarios and CSV fallback timing when no explicit time column is supplied"
    )]
    pub dt: f64,
}

impl CliArgs {
    /// Parses and cross-validates the CLI surface before any engine work begins.
    pub fn parse_args() -> Self {
        let args = Self::parse();
        let csv_mode = args.observed_csv.is_some() || args.predicted_csv.is_some();
        let explicit_csv_mode = matches!(args.input_mode, Some(InputModeArg::Csv));
        let explicit_synthetic_mode = matches!(args.input_mode, Some(InputModeArg::Synthetic));
        let sweep_mode = args.sweep_family.is_some();

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
        if sweep_mode && args.all {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--sweep-family and --all are mutually exclusive",
                )
                .exit();
        }
        if sweep_mode && args.scenario.is_some() {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--sweep-family and --scenario are mutually exclusive",
                )
                .exit();
        }
        if sweep_mode && csv_mode {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--sweep-family cannot be combined with CSV ingestion flags",
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
        if explicit_csv_mode && !csv_mode {
            Self::command()
                .error(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "--input-mode csv requires both --observed-csv and --predicted-csv",
                )
                .exit();
        }
        if explicit_synthetic_mode && csv_mode {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--input-mode synthetic cannot be combined with CSV ingestion flags",
                )
                .exit();
        }
        if args.time_column.is_some() && !csv_mode {
            Self::command()
                .error(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "--time-column is only valid together with --observed-csv and --predicted-csv",
                )
                .exit();
        }
        if args.sweep_points > 0 && !sweep_mode {
            Self::command()
                .error(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "--sweep-points requires --sweep-family",
                )
                .exit();
        }
        if sweep_mode && explicit_csv_mode {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--input-mode csv cannot be combined with --sweep-family",
                )
                .exit();
        }
        if args.bank_source == BankSourceArg::External && args.bank_path.is_none() {
            Self::command()
                .error(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "--bank-source external requires --bank-path",
                )
                .exit();
        }
        if args.bank_source == BankSourceArg::Builtin && args.bank_path.is_some() {
            Self::command()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--bank-path is only valid together with --bank-source external",
                )
                .exit();
        }

        args
    }

    /// Converts validated CLI arguments into the typed scenario selection consumed by the engine.
    pub fn selection(&self) -> ScenarioSelection {
        if let Some(family) = self.sweep_family {
            ScenarioSelection::Sweep(SweepConfig {
                family: family.into(),
                points: self.sweep_points,
            })
        } else if let (Some(observed_csv), Some(predicted_csv)) =
            (&self.observed_csv, &self.predicted_csv)
        {
            ScenarioSelection::Csv(CsvInputConfig {
                observed_csv: observed_csv.clone(),
                predicted_csv: predicted_csv.clone(),
                scenario_id: self.scenario_id.clone(),
                channel_names: self.channel_names.as_deref().map(parse_channel_names),
                time_column: self.time_column.clone(),
                dt_fallback: self.dt,
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

    /// Converts the validated CLI bank flags into a typed deterministic bank-loading policy.
    #[must_use]
    pub fn bank_config(&self) -> BankRunConfig {
        match self.bank_source {
            BankSourceArg::Builtin => BankRunConfig::builtin(),
            BankSourceArg::External => BankRunConfig {
                source: BankSourceConfig::External(
                    self.bank_path
                        .clone()
                        .expect("validated external bank path"),
                ),
                strict_validation: self.strict_bank_validation,
            },
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
