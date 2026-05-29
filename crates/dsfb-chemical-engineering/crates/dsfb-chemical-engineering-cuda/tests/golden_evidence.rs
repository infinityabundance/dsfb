//! Frozen golden gate for the CPU-reference evidence contract.
//!
//! The inline tests in `evidence.rs` prove the lane evidence is *deterministic* (two calls agree) but
//! not that it reproduces a *reference* value: a semantics change to the quantiser, the drift window,
//! the per-sample record layout, or the evidence-root field order would change both calls together and
//! still pass. This test pins the `lane_evidence_cpu` digest and the assembled `evidence_root` of a
//! fixed synthetic input to frozen constants, so any such drift fails loudly **even without a GPU**.
//! It is the CUDA-side analogue of `edge/tests/golden_replay.rs` and the frozen `atlas_hash_v1` gate.
//!
//! The input is built from **exact dyadic rationals** (`k/8 - 4`), which have identical IEEE-754 bit
//! patterns on every platform — so the digest (which hashes raw float bits) is portable across
//! machines and compilers, not tied to a platform `libm`. If a contract change is intentional, mint
//! the new hex on the pinned toolchain (see `rust-toolchain.toml`) and re-freeze the constants below.

use dsfb_chemical_engineering_cuda::court::{evidence_root, CaseFile};
use dsfb_chemical_engineering_cuda::evidence::{lane_evidence_cpu, merkle_root, LaneEvidence};

const N_LANES: u32 = 3;
const N_SAMPLES: usize = 96;

/// Deterministic, bit-portable lane: every value is an exact multiple of 1/8 in [-4, +3.875].
fn lane_values(lane: u32) -> Vec<f64> {
    (0..N_SAMPLES)
        .map(|i| (((i * 7 + lane as usize * 13) % 64) as f64) / 8.0 - 4.0)
        .collect()
}

fn build_lanes() -> Vec<LaneEvidence> {
    (0..N_LANES)
        .map(|l| lane_evidence_cpu(l, &lane_values(l)))
        .collect()
}

/// Frozen lane-0 digest (hex SHA-256 of the 40-bytes/sample canonical record). Re-freeze deliberately.
const GOLDEN_LANE0_DIGEST: &str =
    "1888ba3c8d573dd91c68420e1fc298cd27dc365dbbea0404258d6ebb95e67bcd";
/// Frozen Merkle root over the three lane digests, in lane order.
const GOLDEN_MERKLE_ROOT: &str = "249815f8926c5b9e33f4dc9e339ec89595ba8783fd4d29d430ead7e06aa7aee1";
/// Frozen evidence root over passport + lanes + merkle (the court's single ground-truth hash).
const GOLDEN_EVIDENCE_ROOT: &str =
    "b88aeaf358bcbb42f2307a8d726e083f5a582b049809512d90b0eee9bd90607c";

#[test]
fn cpu_evidence_matches_frozen_golden() {
    let lanes = build_lanes();
    assert_eq!(lanes.len(), N_LANES as usize);

    // Sanity: each lane processed all samples and the digest is a 64-char lowercase hex string.
    for l in &lanes {
        assert_eq!(l.n_samples, N_SAMPLES as u32);
        assert_eq!(l.digest_hex.len(), 64);
    }

    assert_eq!(
        lanes[0].digest_hex, GOLDEN_LANE0_DIGEST,
        "lane-0 digest drift — evidence-contract regression OR intended change (if intended, re-freeze)"
    );

    let merkle = merkle_root(&lanes);
    assert_eq!(
        merkle, GOLDEN_MERKLE_ROOT,
        "merkle-root drift — re-freeze if the change is intended"
    );

    // Assemble a case file the same way the court does, and pin the evidence root.
    let cf = CaseFile::assemble(
        "golden",
        "00".repeat(32),
        (N_LANES as u64) * (N_SAMPLES as u64),
        lanes.clone(),
        None,
    );
    assert_eq!(cf.merkle_root, merkle);
    assert_eq!(
        cf.evidence_root, GOLDEN_EVIDENCE_ROOT,
        "evidence-root drift — contract/field-order regression OR intended change (if intended, re-freeze)"
    );

    // And it must replay byte-exact against a fresh recomputation (the court's core guarantee).
    let recomputed = build_lanes();
    let root = evidence_root(&cf.passport, &recomputed, &merkle_root(&recomputed));
    assert_eq!(
        root, cf.evidence_root,
        "recomputed evidence root must equal the sealed root"
    );
}

// ── Two-block SHA padding residue class (n_samples ≡ 3 mod 8) ──────────────────────────────────────
// N_SAMPLES=43 makes each lane's hashed stream 40*43 = 1720 bytes, and 1720 mod 64 = 56 — exactly the
// two-block padding case that the device-SHA bug (P43) mishandled. The CPU reference uses the `sha2`
// crate (correct by definition), so this pins a frozen value confirming the golden *suite itself*
// exercises the residue class (the N=96 case above is ≡0 mod 8 and dodges it). On a CUDA host the
// `gpu_cpu_parity.rs` gate then proves the device kernel matches this CPU reference on the same class.
const N_SAMPLES_2BLK: usize = 43; // 43 ≡ 3 (mod 8); 40*43 ≡ 56 (mod 64) → two-block lane digest

fn lane_values_2blk(lane: u32) -> Vec<f64> {
    (0..N_SAMPLES_2BLK)
        .map(|i| (((i * 7 + lane as usize * 13) % 64) as f64) / 8.0 - 4.0)
        .collect()
}

/// Frozen lane-0 digest for the two-block (N=43) stream. Re-freeze deliberately if the contract changes.
const GOLDEN_2BLK_LANE0_DIGEST: &str =
    "8faa926b9ebe7a09813347da79726640161d17907a661323baaab9f00aa7e398";
/// Frozen evidence root for the N=43, 3-lane case.
const GOLDEN_2BLK_EVIDENCE_ROOT: &str =
    "ca5d1c672d48b300d7089fa7711b60057f07df11fe22ed6c1f2362fbb01ffb9a";

#[test]
fn cpu_evidence_two_block_residue_matches_frozen_golden() {
    let lanes: Vec<LaneEvidence> = (0..N_LANES)
        .map(|l| lane_evidence_cpu(l, &lane_values_2blk(l)))
        .collect();
    for l in &lanes {
        assert_eq!(l.n_samples, N_SAMPLES_2BLK as u32);
        // 40 bytes/sample * 43 = 1720 bytes; 1720 % 64 == 56 → the lane digest is a two-block SHA pad.
        assert_eq!(
            (40 * N_SAMPLES_2BLK) % 64,
            56,
            "this case must exercise the two-block padding residue"
        );
    }
    assert_eq!(
        lanes[0].digest_hex, GOLDEN_2BLK_LANE0_DIGEST,
        "two-block lane-0 digest drift — re-freeze if the change is intended"
    );
    let cf = CaseFile::assemble(
        "golden2blk",
        "11".repeat(32),
        (N_LANES as u64) * (N_SAMPLES_2BLK as u64),
        lanes.clone(),
        None,
    );
    assert_eq!(
        cf.evidence_root, GOLDEN_2BLK_EVIDENCE_ROOT,
        "two-block evidence-root drift — re-freeze if the change is intended"
    );
}
