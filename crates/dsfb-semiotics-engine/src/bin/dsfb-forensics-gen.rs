#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use clap::{Parser, ValueEnum, ValueHint};
use dsfb_semiotics_engine::cli::args::{
    BankSourceArg, BankValidationModeArg, CsvInputConfig, EnvelopeModeArg,
};
use dsfb_semiotics_engine::engine::config::{BankRunConfig, BankValidationMode, CommonRunConfig};
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::settings::{EngineSettings, SmoothingMode};
use dsfb_semiotics_engine::engine::types::EnvelopeMode;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ForensicsSmoothingArg {
    Disabled,
    Ema,
    SafetyFirst,
}

impl ForensicsSmoothingArg {
    fn apply(self, settings: &mut EngineSettings, alpha: f64) {
        match self {
            Self::Disabled => settings.smoothing.mode = SmoothingMode::Disabled,
            Self::Ema => {
                settings.smoothing.mode = SmoothingMode::ExponentialMovingAverage;
                settings.smoothing.exponential_alpha = alpha;
            }
            Self::SafetyFirst => {
                settings.smoothing =
                    dsfb_semiotics_engine::engine::settings::SmoothingSettings::safety_first();
            }
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Generate a deterministic DSFB forensic bundle from an observed/predicted CSV pair"
)]
struct ForensicsArgs {
    #[arg(long, value_hint = ValueHint::FilePath)]
    observed_csv: PathBuf,

    #[arg(long, value_hint = ValueHint::FilePath)]
    predicted_csv: PathBuf,

    #[arg(long, default_value = "forensics_csv_case")]
    scenario_id: String,

    #[arg(long)]
    time_column: Option<String>,

    #[arg(long, default_value_t = 1.0)]
    dt: f64,

    #[arg(long, value_enum, default_value_t = EnvelopeModeArg::Fixed)]
    envelope_mode: EnvelopeModeArg,

    #[arg(long, default_value_t = 1.0)]
    envelope_base: f64,

    #[arg(long, default_value_t = 0.0)]
    envelope_slope: f64,

    #[arg(long)]
    envelope_switch_step: Option<usize>,

    #[arg(long)]
    envelope_secondary_slope: Option<f64>,

    #[arg(long)]
    envelope_secondary_base: Option<f64>,

    #[arg(long, default_value = "forensics_envelope")]
    envelope_name: String,

    #[arg(long, value_hint = ValueHint::DirPath)]
    output_dir: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = BankSourceArg::Builtin)]
    bank_mode: BankSourceArg,

    #[arg(long, value_hint = ValueHint::FilePath)]
    bank_path: Option<PathBuf>,

    #[arg(long)]
    strict_bank_validation: bool,

    #[arg(long, value_enum, default_value_t = BankValidationModeArg::Strict)]
    bank_validation_mode: BankValidationModeArg,

    #[arg(long, value_enum, default_value_t = ForensicsSmoothingArg::Disabled)]
    smoothing: ForensicsSmoothingArg,

    #[arg(long, default_value_t = 0.35)]
    smoothing_alpha: f64,

    #[arg(
        long,
        help = "Attempt to open the generated PDF report when the host supports it"
    )]
    open: bool,
}

fn main() -> Result<()> {
    let args = ForensicsArgs::parse();
    let common = CommonRunConfig {
        dt: args.dt,
        output_root: args.output_dir.clone(),
        bank: bank_config(&args),
        ..Default::default()
    };
    let input = CsvInputConfig {
        observed_csv: args.observed_csv.clone(),
        predicted_csv: args.predicted_csv.clone(),
        scenario_id: args.scenario_id.clone(),
        channel_names: None,
        time_column: args.time_column.clone(),
        dt_fallback: args.dt,
        envelope_mode: EnvelopeMode::from(args.envelope_mode),
        envelope_base: args.envelope_base,
        envelope_slope: args.envelope_slope,
        envelope_switch_step: args.envelope_switch_step,
        envelope_secondary_slope: args.envelope_secondary_slope,
        envelope_secondary_base: args.envelope_secondary_base,
        envelope_name: args.envelope_name.clone(),
    };
    let mut settings = EngineSettings::default();
    args.smoothing.apply(&mut settings, args.smoothing_alpha);

    let engine =
        StructuralSemioticsEngine::with_settings(EngineConfig::csv(common, input), settings)?;
    let bundle = engine.run_selected()?;
    let exported = export_artifacts(&bundle)?;

    println!("forensics_run_dir={}", exported.run_dir.display());
    println!("manifest={}", exported.manifest_path.display());
    println!("report_pdf={}", exported.report_pdf.display());
    println!("zip_archive={}", exported.zip_path.display());
    println!(
        "bank_source={}",
        bundle.run_metadata.bank.source_kind.as_label()
    );
    println!(
        "validation_mode={}",
        bundle.run_metadata.bank.validation_mode
    );
    println!(
        "smoothing_mode={}",
        bundle
            .run_metadata
            .engine_settings
            .smoothing
            .mode
            .as_label()
    );

    if args.open {
        println!(
            "open_status={}",
            try_open_report(exported.report_pdf.as_path())
        );
    }

    Ok(())
}

fn bank_config(args: &ForensicsArgs) -> BankRunConfig {
    let validation_mode = if args.strict_bank_validation {
        BankValidationMode::Strict
    } else {
        match args.bank_validation_mode {
            BankValidationModeArg::Strict => BankValidationMode::Strict,
            BankValidationModeArg::Permissive => BankValidationMode::Permissive,
        }
    };
    match args.bank_mode {
        BankSourceArg::Builtin => BankRunConfig::builtin_with_mode(validation_mode),
        BankSourceArg::External => BankRunConfig::external_with_mode(
            args.bank_path.clone().expect("external bank path"),
            validation_mode,
        ),
    }
}

fn try_open_report(report_pdf: &Path) -> String {
    let candidates: &[(&str, &[&str])] = if cfg!(target_os = "macos") {
        &[("open", &[])]
    } else if cfg!(target_os = "windows") {
        &[("cmd", &["/C", "start"])]
    } else {
        &[("xdg-open", &[])]
    };

    for (program, prefix) in candidates {
        let status = Command::new(program).args(*prefix).arg(report_pdf).status();
        match status {
            Ok(status) if status.success() => return format!("launched:{program}"),
            Ok(status) => {
                return format!("unsupported:{program}:exit={}", status.code().unwrap_or(-1))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return format!("unsupported:{program}:{error}"),
        }
    }

    "unsupported:no_platform_opener".to_string()
}
