//! The deterministic execution spine: load manifest → validate authority hashes → execute stages → seal → emit.
//!
//! The runtime is intentionally thin. Its only jobs are (1) to validate the frozen manifest, (2) to *gate* each
//! stage — refusing any stage that declares no authority, or whose declared authority is not in the manifest's
//! frozen allow-list — and (3) to seal the ordered stage receipts into a tamper-evident [`RuntimeReceiptV1`].
//! It never mutates authority, never performs I/O, and admits no claim without an authority anchor.
//!
//! Because [`RuntimeStage`] is generic over `I`/`O`, a heterogeneous multi-stage pipeline is built by the caller
//! (each stage's `O` feeding the next stage's `I`); the runtime's contribution is the per-stage *gate* +
//! `record`, and the final `seal`. The chemical crate is one such caller; this crate carries no chemical logic.

use crate::authority::AuthorityHash;
use crate::errors::RuntimeError;
use crate::manifest::DensorManifest;
use crate::receipt::RuntimeReceiptV1;
use crate::stage::{RuntimeStage, StageReceipt, StageReceiptSummary};

/// Accumulates the gated, validated stage summaries of one run, then seals them.
pub struct Runtime<'m> {
    manifest: &'m DensorManifest,
    stages: Vec<StageReceiptSummary>,
}

impl<'m> Runtime<'m> {
    /// Start a run against a validated manifest. Validates the manifest up front (fails closed).
    pub fn start(manifest: &'m DensorManifest) -> Result<Self, RuntimeError> {
        manifest.validate()?;
        Ok(Runtime {
            manifest,
            stages: Vec::new(),
        })
    }

    /// Gate a stage's declared authorities against the manifest's frozen allow-list. A stage with no authorities,
    /// or one citing an authority the manifest does not pin, is refused (no claim without an authority anchor).
    fn gate(&self, stage_id: &str, authorities: &[AuthorityHash]) -> Result<(), RuntimeError> {
        if authorities.is_empty() {
            return Err(RuntimeError::MissingAuthority {
                stage: stage_id.to_string(),
            });
        }
        for a in authorities {
            if !self.manifest.permits_authority(a) {
                return Err(RuntimeError::AuthorityMismatch {
                    stage: stage_id.to_string(),
                    authority: a.name.clone(),
                });
            }
        }
        Ok(())
    }

    /// Execute one stage on `input`, gating its authorities first, then recording its receipt summary. Returns
    /// the stage's typed output so the caller can feed it into the next stage. Deterministic given a deterministic
    /// stage (the runtime adds no nondeterminism).
    pub fn run_stage<I, O, S: RuntimeStage<I, O>>(
        &mut self,
        stage: &S,
        input: I,
    ) -> Result<O, RuntimeError> {
        self.gate(stage.stage_id(), stage.authority_hashes())?;
        let receipt: StageReceipt<O> = stage.execute(input)?;
        // Defence in depth: the stage's own receipt authorities must also be gated (a stage cannot smuggle a
        // different authority into its receipt than the one it advertised).
        self.gate(&receipt.stage_id, &receipt.authority_hashes)?;
        self.stages.push(receipt.summary());
        Ok(receipt.output)
    }

    /// Seal the run: build the tamper-evident [`RuntimeReceiptV1`] over the ordered stage summaries.
    pub fn seal(self) -> RuntimeReceiptV1 {
        RuntimeReceiptV1::build(self.manifest, self.stages)
    }
}
