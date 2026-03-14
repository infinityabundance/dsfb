use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BankSelection {
    Dsfb,
    Dscd,
    Tmtr,
    Add,
    Srd,
    Hret,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunSelection {
    All,
    Core,
    Bank(BankSelection),
}

#[derive(Debug, Parser, Clone)]
#[command(
    name = "dsfb-bank",
    version,
    about = "Executable empirical companion for the DSFB theorem banks"
)]
pub struct Cli {
    /// Run every theorem bank, the core theorem layer, and all realization exports.
    #[arg(long)]
    pub all: bool,
    /// Run only the core theorem layer.
    #[arg(long)]
    pub core: bool,
    /// Run a single theorem bank.
    #[arg(long)]
    pub bank: Option<BankSelection>,
    /// List available theorem demos and realization outputs.
    #[arg(long)]
    pub list: bool,
    /// Override the output root directory.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Optional deterministic seed used for witness construction.
    #[arg(long)]
    pub seed: Option<u64>,
}

impl Cli {
    pub fn selection(&self) -> Result<RunSelection> {
        let mut count = 0;
        if self.all {
            count += 1;
        }
        if self.core {
            count += 1;
        }
        if self.bank.is_some() {
            count += 1;
        }
        if self.list {
            count += 1;
        }
        if count == 0 {
            bail!("select one of --all, --core, --bank <component>, or --list");
        }
        if count > 1 {
            bail!("use exactly one of --all, --core, --bank <component>, or --list");
        }

        Ok(if self.all {
            RunSelection::All
        } else if self.core {
            RunSelection::Core
        } else if let Some(bank) = self.bank {
            RunSelection::Bank(bank)
        } else {
            bail!("--list does not produce a run selection")
        })
    }
}
