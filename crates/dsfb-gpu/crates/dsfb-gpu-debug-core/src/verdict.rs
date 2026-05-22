//! Final verdict enum and its lexical name table.
//!
//! Every value here corresponds to a specific exit path through the
//! pipeline. The verdict is the only end-state of a comparison or a
//! single-run case file, and the spec's breach-detection tests check
//! that the right one fires under each manipulation.

/// Final verdict attached to every case-file. Names are spec-defined.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum FinalVerdict {
    /// CPU and GPU paths produced byte-identical artifacts: the
    /// strongest possible outcome.
    ReplayAdmissible = 0,
    /// CPU-only path executed; no GPU comparison was attempted (host
    /// without CUDA, or `run-gpu` not invoked).
    CpuOnlyAdmissible = 1,
    /// GPU path executed and matches what a CPU run would have produced;
    /// the CPU side wasn't available for the comparison at this run.
    GpuReplayAdmissible = 2,
    /// Contract bytes themselves were rejected (numeric mode wrong,
    /// kernel sequence reordered, etc.).
    ContractBreach = 3,
    /// `bank_hash` in the contract does not match the bank actually
    /// loaded.
    BankMismatch = 4,
    /// `detector_registry_hash` in the contract does not match the
    /// detector registry.
    DetectorRegistryMismatch = 5,
    /// A per-stage hash diverged between the CPU and GPU artifacts.
    NumericMismatch = 6,
    /// The kernel-sequence list in the contract does not match what
    /// the run actually executed.
    KernelSequenceMismatch = 7,
    /// An episode was found in the case file that did not carry a bank
    /// admission token — the Semantic Non-Bypass guard tripped.
    SemanticBypassRejected = 8,
}

impl FinalVerdict {
    /// Stable lowercase-camel name used in the canonical case-file
    /// bytes. Renaming any of these breaks every prior case file.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ReplayAdmissible => "ReplayAdmissible",
            Self::CpuOnlyAdmissible => "CpuOnlyAdmissible",
            Self::GpuReplayAdmissible => "GpuReplayAdmissible",
            Self::ContractBreach => "ContractBreach",
            Self::BankMismatch => "BankMismatch",
            Self::DetectorRegistryMismatch => "DetectorRegistryMismatch",
            Self::NumericMismatch => "NumericMismatch",
            Self::KernelSequenceMismatch => "KernelSequenceMismatch",
            Self::SemanticBypassRejected => "SemanticBypassRejected",
        }
    }

    /// Exit code the CLI uses for this verdict. Mirrors the matrix in
    /// the README.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::ReplayAdmissible | Self::CpuOnlyAdmissible | Self::GpuReplayAdmissible => 0,
            Self::ContractBreach
            | Self::BankMismatch
            | Self::DetectorRegistryMismatch
            | Self::NumericMismatch
            | Self::KernelSequenceMismatch => 3,
            Self::SemanticBypassRejected => 4,
        }
    }
}
