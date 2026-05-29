//! Runtime stages and their per-stage receipts.
//!
//! A `RuntimeStage<I, O>` is one deterministic transform in a densor pipeline. It declares the authority hashes it
//! was built against, and on `execute` returns a [`StageReceipt`] binding the input hash, the output hash, and
//! those authorities to the produced output. Discipline (enforced by the runtime, see `runtime.rs`): a stage must
//! be deterministic, must declare ≥1 authority hash, must not mutate authority during execution, and must not
//! perform I/O or network access.

use crate::authority::AuthorityHash;
use crate::errors::RuntimeError;
use crate::seal::to_hex;

/// The evidence a stage emits: the input/output hashes, the authorities consulted, and the output value itself.
pub struct StageReceipt<O> {
    pub stage_id: String,
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub authority_hashes: Vec<AuthorityHash>,
    pub output: O,
}

impl<O> StageReceipt<O> {
    /// A compact, sealable summary of this receipt (drops the typed output, keeps the hashes) — what the run-level
    /// [`crate::receipt::RuntimeReceiptV1`] records and seals.
    pub fn summary(&self) -> StageReceiptSummary {
        StageReceiptSummary {
            stage_id: self.stage_id.clone(),
            input_hash: self.input_hash,
            output_hash: self.output_hash,
            authority_hashes: self.authority_hashes.clone(),
        }
    }
}

/// The output-erased summary of a stage receipt (hashes + authorities), used in the sealed run record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StageReceiptSummary {
    pub stage_id: String,
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub authority_hashes: Vec<AuthorityHash>,
}

impl StageReceiptSummary {
    pub fn input_hex(&self) -> String {
        to_hex(&self.input_hash)
    }
    pub fn output_hex(&self) -> String {
        to_hex(&self.output_hash)
    }
}

/// One deterministic transform in a densor pipeline.
pub trait RuntimeStage<I, O> {
    /// A stable, non-empty stage identifier.
    fn stage_id(&self) -> &str;
    /// The frozen authorities this stage was built against (must be non-empty — see the runtime gate).
    fn authority_hashes(&self) -> &[AuthorityHash];
    /// Execute deterministically, returning a receipt binding input/output hashes + authorities to the output.
    /// Implementors compute `input_hash`/`output_hash` via the crate's [`crate::seal`] helpers.
    fn execute(&self, input: I) -> Result<StageReceipt<O>, RuntimeError>;
}
