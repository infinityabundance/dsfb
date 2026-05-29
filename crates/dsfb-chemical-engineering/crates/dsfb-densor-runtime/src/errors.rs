//! Error types for the densor runtime. Every failure mode is explicit and inspectable — the runtime never
//! panics on bad input, it returns one of these.

use std::fmt;

/// A failure verifying a single densor object's seal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DensorError {
    /// `densor_id` was empty — a densor must be identifiable.
    EmptyId,
    /// The recomputed evidence hash did not match the stored one (tamper / corruption).
    EvidenceMismatch,
}

impl fmt::Display for DensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DensorError::EmptyId => write!(f, "densor has an empty densor_id"),
            DensorError::EvidenceMismatch => {
                write!(f, "densor evidence_hash does not match its recomputed seal")
            }
        }
    }
}
impl std::error::Error for DensorError {}

/// A failure during pipeline execution. The runtime is disciplined: it refuses to execute a stage that does not
/// declare the authority hashes it was built against, and it refuses to admit a stage whose declared authorities
/// do not match the manifest's frozen set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// A stage declared no authority hashes — no claim may be executed without an authority anchor.
    MissingAuthority { stage: String },
    /// A stage's declared authority hash is not in the manifest's frozen authority set (or differs).
    AuthorityMismatch { stage: String, authority: String },
    /// The manifest itself is malformed (empty, duplicate ids, …).
    ManifestInvalid(String),
    /// The stage's `execute` reported a domain failure.
    StageFailed { stage: String, reason: String },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::MissingAuthority { stage } => {
                write!(f, "stage '{stage}' declared no authority hashes (no claim without an authority anchor)")
            }
            RuntimeError::AuthorityMismatch { stage, authority } => {
                write!(f, "stage '{stage}' authority '{authority}' is not in the manifest's frozen authority set")
            }
            RuntimeError::ManifestInvalid(why) => write!(f, "densor manifest invalid: {why}"),
            RuntimeError::StageFailed { stage, reason } => {
                write!(f, "stage '{stage}' failed: {reason}")
            }
        }
    }
}
impl std::error::Error for RuntimeError {}
