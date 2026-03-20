use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::cli::args::{CsvInputConfig, ScenarioSelection};
use crate::engine::pipeline::EngineConfig;

/// Default deterministic seed used by the crate when no override is supplied.
pub const DEFAULT_SEED: u64 = 123;
/// Default synthetic horizon length used by the crate when no override is supplied.
pub const DEFAULT_STEPS: usize = 240;
/// Default sample interval used by the crate when no override is supplied.
pub const DEFAULT_DT: f64 = 1.0;

/// Typed bank source selection used by deterministic runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BankSourceConfig {
    Builtin,
    External(PathBuf),
}

/// Typed heuristic-bank governance mode for deterministic runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BankValidationMode {
    Strict,
    Permissive,
}

impl BankValidationMode {
    /// Returns whether the bank must satisfy full strict governance checks.
    #[must_use]
    pub const fn is_strict(self) -> bool {
        matches!(self, Self::Strict)
    }

    /// Returns the machine-readable validation-mode label.
    #[must_use]
    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Permissive => "permissive",
        }
    }
}

/// Deterministic heuristic-bank loading policy shared by synthetic and CSV-driven runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BankRunConfig {
    pub source: BankSourceConfig,
    pub validation_mode: BankValidationMode,
}

/// Common engine settings shared by synthetic and CSV-driven runs.
#[derive(Clone, Debug)]
pub struct CommonRunConfig {
    pub seed: u64,
    pub steps: usize,
    pub dt: f64,
    pub output_root: Option<PathBuf>,
    pub bank: BankRunConfig,
}

/// Explicit synthetic selection for library consumers who do not want to manipulate CLI enums.
#[derive(Clone, Debug)]
pub enum SyntheticSelection {
    All,
    Single(String),
}

/// Typed synthetic run request with shared deterministic settings.
#[derive(Clone, Debug)]
pub struct SyntheticRunConfig {
    pub common: CommonRunConfig,
    pub selection: SyntheticSelection,
}

/// Typed CSV-driven run request with shared deterministic settings.
#[derive(Clone, Debug)]
pub struct CsvRunConfig {
    pub common: CommonRunConfig,
    pub input: CsvInputConfig,
}

impl Default for CommonRunConfig {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SEED,
            steps: DEFAULT_STEPS,
            dt: DEFAULT_DT,
            output_root: None,
            bank: BankRunConfig::default(),
        }
    }
}

impl CommonRunConfig {
    /// Validates the common deterministic runtime settings.
    pub fn validate(&self) -> Result<()> {
        if self.steps == 0 {
            return Err(anyhow!("engine configuration requires steps > 0"));
        }
        if !self.dt.is_finite() || self.dt <= 0.0 {
            return Err(anyhow!(
                "engine configuration requires a positive finite dt; got {}",
                self.dt
            ));
        }
        self.bank.validate()?;
        Ok(())
    }
}

impl Default for BankRunConfig {
    fn default() -> Self {
        Self {
            source: BankSourceConfig::Builtin,
            validation_mode: BankValidationMode::Strict,
        }
    }
}

impl BankRunConfig {
    /// Returns a deterministic built-in bank selection.
    #[must_use]
    pub fn builtin() -> Self {
        Self::default()
    }

    /// Returns a deterministic built-in bank selection with an explicit governance mode.
    #[must_use]
    pub fn builtin_with_mode(validation_mode: BankValidationMode) -> Self {
        Self {
            source: BankSourceConfig::Builtin,
            validation_mode,
        }
    }

    /// Returns a deterministic external-bank selection.
    #[must_use]
    pub fn external(path: PathBuf, strict_validation: bool) -> Self {
        Self {
            source: BankSourceConfig::External(path),
            validation_mode: if strict_validation {
                BankValidationMode::Strict
            } else {
                BankValidationMode::Permissive
            },
        }
    }

    /// Returns a deterministic external-bank selection with an explicit governance mode.
    #[must_use]
    pub fn external_with_mode(path: PathBuf, validation_mode: BankValidationMode) -> Self {
        Self {
            source: BankSourceConfig::External(path),
            validation_mode,
        }
    }

    /// Returns whether this run uses strict bank validation.
    #[must_use]
    pub const fn is_strict(&self) -> bool {
        self.validation_mode.is_strict()
    }

    /// Validates the bank-loading request without touching the filesystem.
    pub fn validate(&self) -> Result<()> {
        match &self.source {
            BankSourceConfig::Builtin => Ok(()),
            BankSourceConfig::External(path) => {
                if path.as_os_str().is_empty() {
                    Err(anyhow!(
                        "external bank loading requires a non-empty bank path"
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl SyntheticRunConfig {
    /// Creates a typed request for the full synthetic scenario suite.
    #[must_use]
    pub fn all(common: CommonRunConfig) -> Self {
        Self {
            common,
            selection: SyntheticSelection::All,
        }
    }

    /// Creates a typed request for one named synthetic scenario.
    #[must_use]
    pub fn single(common: CommonRunConfig, scenario_id: impl Into<String>) -> Self {
        Self {
            common,
            selection: SyntheticSelection::Single(scenario_id.into()),
        }
    }

    /// Validates the shared settings and the synthetic selection.
    pub fn validate(&self) -> Result<()> {
        self.common.validate()?;
        if let SyntheticSelection::Single(id) = &self.selection {
            if id.trim().is_empty() {
                return Err(anyhow!(
                    "synthetic single-scenario selection requires a non-empty scenario id"
                ));
            }
        }
        Ok(())
    }
}

impl CsvRunConfig {
    /// Creates a typed request for a CSV-driven run.
    #[must_use]
    pub fn new(common: CommonRunConfig, input: CsvInputConfig) -> Self {
        Self { common, input }
    }

    /// Validates the shared settings and the CSV-specific request.
    pub fn validate(&self) -> Result<()> {
        self.common.validate()?;
        self.input.validate()
    }
}

impl From<SyntheticRunConfig> for EngineConfig {
    fn from(value: SyntheticRunConfig) -> Self {
        let scenario_selection = match value.selection {
            SyntheticSelection::All => ScenarioSelection::All,
            SyntheticSelection::Single(id) => ScenarioSelection::Single(id),
        };
        Self {
            seed: value.common.seed,
            steps: value.common.steps,
            dt: value.common.dt,
            output_root: value.common.output_root,
            bank: value.common.bank,
            scenario_selection,
        }
    }
}

impl From<CsvRunConfig> for EngineConfig {
    fn from(value: CsvRunConfig) -> Self {
        Self {
            seed: value.common.seed,
            steps: value.common.steps,
            dt: value.common.dt,
            output_root: value.common.output_root,
            bank: value.common.bank,
            scenario_selection: ScenarioSelection::Csv(value.input),
        }
    }
}
