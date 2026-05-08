//! DSFB-Debug: error types — additive, Copy, no-panic.
//!
//! Every fallible operation in the crate returns `Result<T, DsfbError>`
//! rather than panicking. The crate's top-level lints
//! (`#![deny(clippy::unwrap_used)]`) enforce this at compile time.
//!
//! `DsfbError` is `Copy + Clone + Debug + PartialEq + Eq` — every
//! variant is a stack-only enum, no heap allocation. Variants are
//! additive across versions: existing variants are never removed or
//! renumbered, so cross-version `match` arms remain valid.
//!
//! # Variant taxonomy
//!
//! - **DimensionMismatch / BufferTooSmall** — caller-shape errors
//!   (operator passed wrong slice lengths or undersized output buffer)
//! - **MissingRealData / HashMismatch** — `paper-lock` integrity
//!   gates (sentinel fixture / SHA-256 drift)
//! - **ParseError** — adapter/fixture-format errors (invalid TSV,
//!   missing header, etc.)
//! - **InvalidConfig** — paper-lock config-construction errors
//!
//! Operators reading these errors at runtime can map each variant to
//! a specific corrective action; the crate documents the action in
//! each variant's per-variant doc-comment.

/// All errors the DSFB engine can produce.
/// No heap allocation — all variants are Copy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DsfbError {
    /// Input slice length does not match expected signal count
    DimensionMismatch { expected: usize, got: usize },
    /// Baseline has not been established (healthy window not computed)
    BaselineNotEstablished,
    /// Envelope radius is zero or negative for a signal
    InvalidEnvelopeRadius { signal_index: usize },
    /// Window index is out of range for history buffer
    WindowOutOfRange { index: u64 },
    /// Episode buffer is full — increase MAX_EPISODES
    EpisodeBufferFull,
    /// Signal buffer is full — increase MAX_SIGNALS
    SignalBufferFull,
    /// History buffer is full — increase MAX_WINDOWS
    HistoryBufferFull,
    /// Heuristics bank is full — increase MAX_MOTIFS
    HeuristicsBankFull,
    /// Configuration parameter is invalid
    InvalidConfig(&'static str),
    /// Dataset parse error at a specific record
    ParseError { record: u64, field: u16 },
    /// Insufficient healthy-window data for baseline
    InsufficientBaselineData { available: usize, required: usize },
    /// Internal flat-aggregation buffer too small for the requested
    /// `num_signals * num_windows`. Returned at `run_evaluation` entry.
    BufferTooSmall { needed: usize, available: usize },
    /// `paper-lock` real-data evaluation requested but the supplied fixture
    /// bytes are empty. Never falls back to synthetic data.
    MissingRealData,
    /// SHA-256 of supplied fixture bytes does not match the manifest's
    /// `fixture_sha256`. Indicates tampering or upstream drift; the
    /// engine refuses to proceed.
    HashMismatch,
}

impl core::fmt::Display for DsfbError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DimensionMismatch { expected, got } => {
                write!(f, "dimension mismatch: expected {}, got {}", expected, got)
            }
            Self::BaselineNotEstablished => write!(f, "baseline not established"),
            Self::InvalidEnvelopeRadius { signal_index } => {
                write!(f, "invalid envelope radius at signal {}", signal_index)
            }
            Self::WindowOutOfRange { index } => {
                write!(f, "window index {} out of range", index)
            }
            Self::EpisodeBufferFull => write!(f, "episode buffer full"),
            Self::SignalBufferFull => write!(f, "signal buffer full"),
            Self::HistoryBufferFull => write!(f, "history buffer full"),
            Self::HeuristicsBankFull => write!(f, "heuristics bank full"),
            Self::InvalidConfig(msg) => write!(f, "invalid config: {}", msg),
            Self::ParseError { record, field } => {
                write!(f, "parse error at record {}, field {}", record, field)
            }
            Self::InsufficientBaselineData { available, required } => {
                write!(f, "insufficient baseline data: {} of {} required", available, required)
            }
            Self::BufferTooSmall { needed, available } => {
                write!(f, "internal flat buffer too small: needed {}, available {}", needed, available)
            }
            Self::MissingRealData => write!(f, "real-data fixture missing or empty (paper-lock refuses to fall back to synthetic)"),
            Self::HashMismatch => write!(f, "fixture SHA-256 does not match manifest"),
        }
    }
}

pub type Result<T> = core::result::Result<T, DsfbError>;
