//! Auxiliary T.10 helper: prints the canonical `corpus_hash_v1`
//! and the `CaseFileV2Header` hashes used in the T.10 receipts.
//! Not a behavioural test — runs only when invoked with
//! `--nocapture` so the values land in the receipt.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_debug_core::casefile_v2::{
    casefile_v2_header_hash, AtlasAlgebraStatus, CaseFileV2Header, CaseFileV2Schema, CorpusStage,
};
use dsfb_gpu_debug_core::motif::DetectorProfile;

fn hex32(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[test]
fn print_corpus_and_header_hashes() {
    let corpus = compute_corpus_hash_v1();
    println!("corpus_hash_v1 = {}", corpus.to_hex());

    for profile in &[
        DetectorProfile::D16,
        DetectorProfile::D64,
        DetectorProfile::D128,
        DetectorProfile::D205,
    ] {
        let header = CaseFileV2Header {
            schema: CaseFileV2Schema::HeaderOnlyT10,
            corpus_hash_v1: corpus.bytes,
            corpus_stage: CorpusStage::FrozenT10,
            detector_profile: *profile,
            detector_registry_hash: profile.registry_hash(),
            atlas_algebra_status: AtlasAlgebraStatus::S1_1TypeSurfaceOnly,
            semantic_non_bypass: true,
        };
        let h = casefile_v2_header_hash(&header);
        println!(
            "casefile_v2_header_hash ({}) = {}",
            profile.name(),
            hex32(&h)
        );
    }
}
