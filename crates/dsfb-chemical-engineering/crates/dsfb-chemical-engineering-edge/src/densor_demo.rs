//! P101 — an OPTIONAL, feature-gated demonstration that an edge residual-episode summary can be carried
//! through the `dsfb-densor-runtime` substrate **without moving any chemical logic into it**.
//!
//! The runtime crate is a domain-agnostic mechanism (load manifest → validate authority hashes → execute
//! stages → seal → emit receipt; "no claim without an authority hash"). This demo proves it is *usable* by
//! a real chemical pipeline while keeping that separation strict: **every stage implementation and all
//! chemical meaning live here, in the edge crate**; the runtime only gates authorities, records per-stage
//! hashes, and seals them into a [`RuntimeReceiptV1`]. The meaning of the run lives entirely in the
//! authorities it cites — here the frozen `atlas_hash_v1` (the detector/heuristic authority) and a frozen
//! per-episode `seal_policy_v1`.
//!
//! Pipeline (mirrors the runtime's worked example): for each fused episode of a synthetic dataset,
//! **stage A** `FusedEpisode → EpisodeSummary` (bound to `atlas_hash_v1`) then **stage B**
//! `EpisodeSummary → sealed line` (bound to `seal_policy_v1`); then the run is sealed and self-verified.
//!
//! It is gated behind the `densor-runtime-demo` feature and is **off the default build + replay path**, so
//! it adds no dependency to a normal `cargo build` and cannot affect any frozen evidence hash. It carries
//! **no new claim**: it is an integration illustration, not a chemical or cross-domain assertion.

use std::path::Path;

use dsfb_densor_runtime::seal::{sha256, CanonicalHasher};
use dsfb_densor_runtime::{
    AuthorityHash, DensorEntry, DensorKind, DensorManifest, Runtime, RuntimeError, RuntimeIndex,
    RuntimeReceiptV1, RuntimeStage, StageReceipt,
};

use crate::cli::synthetic_suite;
use crate::fusion::FusedEpisode;
use crate::pipeline::{analyze, PipelineConfig};

/// The typed per-episode summary that stage A produces and stage B seals. It is a *projection* of the
/// edge [`FusedEpisode`] — no new evidence, just the citable shape carried between runtime stages.
#[derive(Clone)]
struct EpisodeSummary {
    dataset: String,
    start: usize,
    end: usize,
    motif: String,
    n_families: usize,
    consensus: f64,
    entropy: f64,
}

/// Decode a 64-char hex digest (e.g. `atlas_hash_v1()`) into the raw 32 bytes an [`AuthorityHash`] carries,
/// so the runtime authority IS the frozen atlas digest (not a hash of its hex form).
fn hex_to_32(hex: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        if let Some(byte) = hex
            .get(2 * i..2 * i + 2)
            .and_then(|s| u8::from_str_radix(s, 16).ok())
        {
            *b = byte;
        }
    }
    out
}

/// CanonicalHasher (the runtime's, not edge's) over a fused episode's citable fields — the stage input hash.
fn hash_episode(dataset: &str, e: &FusedEpisode) -> [u8; 32] {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"edge.fused_episode.v1");
    h.field("dataset", dataset.as_bytes());
    h.u64("start", e.start_index as u64);
    h.u64("end", e.end_index as u64);
    h.field("motif", e.dominant_motif.token().as_bytes());
    for fam in &e.families {
        h.field("family", fam.as_bytes());
    }
    h.field(
        "consensus",
        format!("{:.6}", e.consensus_strength).as_bytes(),
    );
    h.field(
        "entropy",
        format!("{:.6}", e.disagreement_entropy).as_bytes(),
    );
    h.finalize()
}

/// CanonicalHasher over the projected summary — the stage output hash (and stage B's input hash).
fn hash_summary(s: &EpisodeSummary) -> [u8; 32] {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"edge.episode_summary.v1");
    h.field("dataset", s.dataset.as_bytes());
    h.u64("start", s.start as u64);
    h.u64("end", s.end as u64);
    h.field("motif", s.motif.as_bytes());
    h.u64("n_families", s.n_families as u64);
    h.field("consensus", format!("{:.6}", s.consensus).as_bytes());
    h.field("entropy", format!("{:.6}", s.entropy).as_bytes());
    h.finalize()
}

/// Stage A: project a fused episode to a typed summary, bound to the atlas authority.
struct EpisodeToSummaryStage {
    dataset: String,
    authorities: Vec<AuthorityHash>,
}
impl RuntimeStage<FusedEpisode, EpisodeSummary> for EpisodeToSummaryStage {
    fn stage_id(&self) -> &str {
        "episode_to_summary"
    }
    fn authority_hashes(&self) -> &[AuthorityHash] {
        &self.authorities
    }
    fn execute(&self, e: FusedEpisode) -> Result<StageReceipt<EpisodeSummary>, RuntimeError> {
        let input_hash = hash_episode(&self.dataset, &e);
        let output = EpisodeSummary {
            dataset: self.dataset.clone(),
            start: e.start_index,
            end: e.end_index,
            motif: e.dominant_motif.token().to_string(),
            n_families: e.families.len(),
            consensus: e.consensus_strength,
            entropy: e.disagreement_entropy,
        };
        let output_hash = hash_summary(&output);
        Ok(StageReceipt {
            stage_id: self.stage_id().to_string(),
            input_hash,
            output_hash,
            authority_hashes: self.authorities.clone(),
            output,
        })
    }
}

/// Stage B: seal the summary to a citable one-line record, bound to the seal-policy authority.
struct SummaryToSealStage {
    authorities: Vec<AuthorityHash>,
}
impl RuntimeStage<EpisodeSummary, String> for SummaryToSealStage {
    fn stage_id(&self) -> &str {
        "summary_to_sealed"
    }
    fn authority_hashes(&self) -> &[AuthorityHash] {
        &self.authorities
    }
    fn execute(&self, s: EpisodeSummary) -> Result<StageReceipt<String>, RuntimeError> {
        let input_hash = hash_summary(&s);
        let sealed = format!(
            "{}:{}-{}:{} ({} families)",
            s.dataset, s.start, s.end, s.motif, s.n_families
        );
        let mut h = CanonicalHasher::new();
        h.field("schema", b"edge.sealed_episode.v1");
        h.field("line", sealed.as_bytes());
        let output_hash = h.finalize();
        Ok(StageReceipt {
            stage_id: self.stage_id().to_string(),
            input_hash,
            output_hash,
            authority_hashes: self.authorities.clone(),
            output: sealed,
        })
    }
}

/// Build the manifest + run every fused episode of a representative synthetic dataset through the two
/// stages, then seal. Returns the sealed receipt, the manifest it was sealed against, and the episode
/// count. Deterministic + self-contained (synthetic data; no I/O). Reused by the CLI + the test.
fn build_and_seal() -> Result<(RuntimeReceiptV1, DensorManifest, usize), RuntimeError> {
    // A 30-variable step-drift synthetic from the golden suite — known to produce fused episodes.
    let ds = synthetic_suite()
        .into_iter()
        .find(|d| d.name == "synth_wide_step")
        .expect("synth_wide_step is in the synthetic suite");
    let res = analyze(
        &ds.name,
        &ds.kind,
        &ds.matrix,
        ds.n_base,
        PipelineConfig::default(),
    );

    // Frozen authorities: the atlas detector/heuristic authority (its real digest) + a per-episode seal
    // policy. A stage may only cite authorities the manifest froze (the runtime's "no claim without an
    // authority hash" gate); the chemical meaning lives in these, not in the runtime.
    let atlas_auth = AuthorityHash::new(
        "atlas_hash_v1",
        hex_to_32(&dsfb_chemical_engineering_atlas::hashes::atlas_hash_v1()),
    );
    let seal_auth = AuthorityHash::new(
        "seal_policy_v1",
        sha256(b"dsfb-chemical-engineering:per-episode-seal-policy-v1"),
    );

    let manifest = DensorManifest {
        pipeline_id: format!("edge_episode_demo:{}", ds.name),
        densors: vec![DensorEntry {
            id: ds.name.clone(),
            kind: DensorKind::Residual,
            evidence_hash: sha256(res.replay_hash.as_bytes()),
        }],
        authorities: vec![atlas_auth.clone(), seal_auth.clone()],
    };

    let mut rt = Runtime::start(&manifest)?;
    let stage_a = EpisodeToSummaryStage {
        dataset: ds.name.clone(),
        authorities: vec![atlas_auth],
    };
    let stage_b = SummaryToSealStage {
        authorities: vec![seal_auth],
    };
    let n_episodes = res.fused_episodes.len();
    for e in &res.fused_episodes {
        let summary = rt.run_stage(&stage_a, e.clone())?;
        let _sealed = rt.run_stage(&stage_b, summary)?;
    }
    Ok((rt.seal(), manifest, n_episodes))
}

/// CLI entry (`densor-runtime-demo`): run the demo and print the sealed run receipt + a runtime index line.
pub fn run_densor_runtime_demo(_crate_dir: &Path) -> i32 {
    match build_and_seal() {
        Ok((receipt, manifest, n_episodes)) => {
            println!("dsfb-densor-runtime demo — edge episodes carried through the substrate (no chemical logic moved in)");
            println!("  pipeline_id   : {}", receipt.pipeline_id);
            println!("  episodes      : {n_episodes} (each → stage A `episode_to_summary` → stage B `summary_to_sealed`)");
            println!("  stage records : {}", receipt.stages.len());
            println!("  receipt_hash  : {}", receipt.receipt_hash);
            println!("  verifies      : {}", receipt.verify(&manifest));
            let idx = RuntimeIndex::of(&receipt);
            println!("  {}", idx.summary_line());
            println!("  non_claim     : {}", receipt.non_claim);
            if receipt.verify(&manifest) {
                0
            } else {
                eprintln!("densor-runtime-demo: receipt did NOT verify");
                1
            }
        }
        Err(e) => {
            eprintln!("densor-runtime-demo failed: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_seals_a_verifiable_receipt_over_two_stages_per_episode() {
        let (receipt, manifest, n_episodes) = build_and_seal().expect("demo runs");
        assert!(
            n_episodes >= 1,
            "the synthetic dataset must produce at least one fused episode"
        );
        // Two stages per episode were recorded.
        assert_eq!(receipt.stages.len(), 2 * n_episodes);
        // The sealed run self-verifies against its manifest, and is deterministic.
        assert!(
            receipt.verify(&manifest),
            "the runtime receipt must verify against its manifest"
        );
        assert_eq!(receipt.receipt_hash.len(), 64);
        let (again, _, _) = build_and_seal().expect("demo re-runs");
        assert_eq!(
            receipt.receipt_hash, again.receipt_hash,
            "the demo must be deterministic"
        );
        // The runtime index reports both stage ids + both authorities (sorted, de-duped).
        let idx = RuntimeIndex::of(&receipt);
        assert!(idx.authorities.iter().any(|a| a == "atlas_hash_v1"));
        assert!(idx.authorities.iter().any(|a| a == "seal_policy_v1"));
    }
}
