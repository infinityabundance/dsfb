//! `dsfb-densor-runtime` — a thin, deterministic execution-substrate skeleton for DSFB densor pipelines.
//!
//! Its only job is to standardise how DSFB evidence objects ("densors") are **loaded, validated against frozen
//! authority hashes, executed stage by stage, sealed, and reported** — so future DSFB work shares one disciplined
//! spine instead of re-implementing it. The flow is exactly:
//!
//! ```text
//! load manifest  ->  validate authority hashes  ->  execute stages  ->  seal evidence  ->  emit receipt
//! ```
//!
//! **Discipline (the substrate is deliberately small and strict):**
//! - no network, no hidden downloads, no filesystem side effects in the core;
//! - no probabilistic runtime behaviour — every output is a pure function of the inputs;
//! - no authority mutation during execution (authorities are a frozen allow-list);
//! - **no claim without an authority hash** — a stage that declares no authority is refused.
//!
//! **Non-claim.** This crate is a *mechanism*, not an authority. It carries **no chemical-engineering content and
//! makes no cross-domain claims** — the meaning of any pipeline lives entirely in the authorities a run cites and
//! in the domain crate that defines the stages (e.g. `dsfb-chemical-engineering-edge`). The runtime only attests
//! *which stages ran over which densors against which frozen authorities, sealing to which digest*.
//!
//! See [`runtime::Runtime`] for the spine, [`receipt::RuntimeReceiptV1`] for the sealed output, and the worked
//! `tests` below for a complete two-stage example (with determinism + tamper + authority-gate coverage).

#![forbid(unsafe_code)]

pub mod authority;
pub mod densor;
pub mod errors;
pub mod index;
pub mod manifest;
pub mod receipt;
pub mod runtime;
pub mod seal;
pub mod stage;

pub use authority::AuthorityHash;
pub use densor::{Densor, DensorKind};
pub use errors::{DensorError, RuntimeError};
pub use index::RuntimeIndex;
pub use manifest::{DensorEntry, DensorManifest};
pub use receipt::RuntimeReceiptV1;
pub use runtime::Runtime;
pub use stage::{RuntimeStage, StageReceipt};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seal::{sha256, CanonicalHasher};

    // ── A worked two-stage example pipeline over a residual-like Vec<i64> ──────────────────────────────
    // Stage A scales the input by a fixed factor (a "scale policy"); Stage B counts entries beyond a band
    // (an "envelope"). Both are pure + deterministic, each bound to one frozen authority. This is illustrative
    // only — it carries no chemical meaning; it exercises the substrate's load/validate/execute/seal/emit spine.

    fn scale_authority() -> AuthorityHash {
        AuthorityHash::new("scale_policy_v1", sha256(b"scale=1000000"))
    }
    fn envelope_authority() -> AuthorityHash {
        AuthorityHash::new("envelope_v1", sha256(b"abs_band=3"))
    }

    fn hash_i64s(label: &str, xs: &[i64]) -> [u8; 32] {
        let mut h = CanonicalHasher::new();
        h.field("schema", label.as_bytes());
        for x in xs {
            h.u64("x", *x as u64);
        }
        h.finalize()
    }

    struct ScaleStage {
        factor: i64,
        authorities: Vec<AuthorityHash>,
    }
    impl RuntimeStage<Vec<i64>, Vec<i64>> for ScaleStage {
        fn stage_id(&self) -> &str {
            "scale"
        }
        fn authority_hashes(&self) -> &[AuthorityHash] {
            &self.authorities
        }
        fn execute(&self, input: Vec<i64>) -> Result<StageReceipt<Vec<i64>>, RuntimeError> {
            let input_hash = hash_i64s("scale.in", &input);
            let output: Vec<i64> = input
                .iter()
                .map(|x| x.saturating_mul(self.factor))
                .collect();
            let output_hash = hash_i64s("scale.out", &output);
            Ok(StageReceipt {
                stage_id: self.stage_id().to_string(),
                input_hash,
                output_hash,
                authority_hashes: self.authorities.clone(),
                output,
            })
        }
    }

    struct CountBeyondStage {
        band: i64,
        authorities: Vec<AuthorityHash>,
    }
    impl RuntimeStage<Vec<i64>, usize> for CountBeyondStage {
        fn stage_id(&self) -> &str {
            "count_beyond"
        }
        fn authority_hashes(&self) -> &[AuthorityHash] {
            &self.authorities
        }
        fn execute(&self, input: Vec<i64>) -> Result<StageReceipt<usize>, RuntimeError> {
            let input_hash = hash_i64s("count.in", &input);
            let output = input
                .iter()
                .filter(|x| x.unsigned_abs() as i64 > self.band)
                .count();
            let mut h = CanonicalHasher::new();
            h.u64("count", output as u64);
            Ok(StageReceipt {
                stage_id: self.stage_id().to_string(),
                input_hash,
                output_hash: h.finalize(),
                authority_hashes: self.authorities.clone(),
                output,
            })
        }
    }

    fn manifest() -> DensorManifest {
        DensorManifest {
            pipeline_id: "example_residual_pipeline".into(),
            densors: vec![DensorEntry {
                id: "residual_vec".into(),
                kind: DensorKind::Residual,
                evidence_hash: sha256(b"residual_vec"),
            }],
            authorities: vec![scale_authority(), envelope_authority()],
        }
    }

    fn run_example() -> RuntimeReceiptV1 {
        let m = manifest();
        let mut rt = Runtime::start(&m).unwrap();
        let scaled = rt
            .run_stage(
                &ScaleStage {
                    factor: 2,
                    authorities: vec![scale_authority()],
                },
                vec![1, -5, 2],
            )
            .unwrap();
        let _count = rt
            .run_stage(
                &CountBeyondStage {
                    band: 3,
                    authorities: vec![envelope_authority()],
                },
                scaled,
            )
            .unwrap();
        rt.seal()
    }

    #[test]
    fn pipeline_runs_seals_and_is_deterministic() {
        let a = run_example();
        let b = run_example();
        assert_eq!(
            a.receipt_hash, b.receipt_hash,
            "the sealed run receipt must be deterministic"
        );
        assert_eq!(a.receipt_hash.len(), 64);
        assert_eq!(a.stages.len(), 2);
        assert!(
            a.verify(&manifest()),
            "the receipt must self-verify against its manifest"
        );
    }

    #[test]
    fn tampering_a_stage_breaks_the_seal() {
        let mut r = run_example();
        // Forge a different output hash on the first stage → the seal must no longer verify.
        r.stages[0].output_hash = sha256(b"forged");
        assert!(
            !r.verify(&manifest()),
            "a tampered stage summary must fail verification"
        );
    }

    #[test]
    fn stage_without_authority_is_refused() {
        // No claim without an authority anchor: a stage declaring no authorities is rejected by the gate.
        let m = manifest();
        let mut rt = Runtime::start(&m).unwrap();
        let err = rt
            .run_stage(
                &ScaleStage {
                    factor: 2,
                    authorities: vec![],
                },
                vec![1, 2],
            )
            .unwrap_err();
        assert!(matches!(err, RuntimeError::MissingAuthority { .. }));
    }

    #[test]
    fn stage_citing_an_unpinned_authority_is_refused() {
        // A stage may only cite authorities the manifest froze; an unknown authority is rejected.
        let m = manifest();
        let mut rt = Runtime::start(&m).unwrap();
        let rogue = AuthorityHash::new("rogue_v1", sha256(b"not-in-manifest"));
        let err = rt
            .run_stage(
                &ScaleStage {
                    factor: 2,
                    authorities: vec![rogue],
                },
                vec![1, 2],
            )
            .unwrap_err();
        assert!(matches!(err, RuntimeError::AuthorityMismatch { .. }));
    }

    #[test]
    fn empty_pipeline_id_manifest_is_invalid() {
        let mut m = manifest();
        m.pipeline_id = "  ".into();
        assert!(matches!(
            Runtime::start(&m),
            Err(RuntimeError::ManifestInvalid(_))
        ));
    }

    #[test]
    fn runtime_index_summarises_the_run() {
        let r = run_example();
        let idx = RuntimeIndex::of(&r);
        assert_eq!(idx.stage_ids, vec!["scale", "count_beyond"]);
        assert_eq!(idx.authorities, vec!["envelope_v1", "scale_policy_v1"]); // sorted, de-duped
        assert!(idx.summary_line().contains("example_residual_pipeline"));
    }
}
