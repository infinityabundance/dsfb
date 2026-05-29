//! Gate for `DigestEquivalenceHarnessV1`: every candidate evidence producer must match the CPU
//! reference byte-for-byte across the adversarial battery (lane evidence, Merkle root, evidence_root,
//! replay). Built `--features cuda` this is a CPU↔GPU gate on the real device; otherwise it is a CPU
//! determinism self-check. See `src/digest_equivalence.rs` for the law and the battery rationale.

use dsfb_chemical_engineering_cuda::digest_equivalence as de;

#[test]
fn evidence_backends_are_digest_equivalent_across_the_battery() {
    let reports = de::run_harness();
    assert!(!reports.is_empty(), "battery must not be empty");

    let mut failures = Vec::new();
    for r in &reports {
        if !r.passed() {
            failures.push(format!(
                "  {} [{}]: lanes={} merkle={} root={} replay={}  ref={}… cand={}…",
                r.case,
                r.backend,
                r.lanes_match,
                r.merkle_match,
                r.root_match,
                r.replay_match,
                &r.reference_root[..16.min(r.reference_root.len())],
                &r.candidate_root[..16.min(r.candidate_root.len())],
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "digest-equivalence FAILED for {} of {} case(s):\n{}",
        failures.len(),
        reports.len(),
        failures.join("\n")
    );
}

/// V2-A gate (GPU only): the batched kernel, run over ALL battery cases in one launch, must be
/// digest-identical to the CPU reference per lane. Skips gracefully when no CUDA device is present.
#[cfg(feature = "cuda")]
#[test]
fn v2a_batched_kernel_is_digest_identical_to_reference() {
    let Some(reports) = de::run_batched_harness() else {
        eprintln!("no CUDA device — skipping V2-A batched gate");
        return;
    };
    assert!(!reports.is_empty(), "batched battery must not be empty");
    let mut failures = Vec::new();
    for r in &reports {
        if !r.passed() {
            failures.push(format!(
                "  {} [{}]: lanes={} merkle={} root={} replay={}",
                r.case, r.backend, r.lanes_match, r.merkle_match, r.root_match, r.replay_match
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "V2-A batched digest-equivalence FAILED for {} of {} case(s):\n{}",
        failures.len(),
        reports.len(),
        failures.join("\n")
    );
}

/// V2-B gate (GPU only): the segment-parallel kernel must reproduce the CPU `lane_evidence_v2_cpu`
/// Merkle-segment reference byte-for-byte, at two segment sizes — a small one (`seg=4 < DRIFT_WINDOW`,
/// stressing partial + full halo warm-up and many segments) and the production `SEGMENT_SIZE`.
#[cfg(feature = "cuda")]
#[test]
fn v2b_segmented_kernel_matches_cpu_merkle_reference() {
    use dsfb_chemical_engineering_cuda::evidence::SEGMENT_SIZE;
    for seg in [4usize, SEGMENT_SIZE] {
        let Some(reports) = de::run_v2_segmented_harness(seg) else {
            eprintln!("no CUDA device — skipping V2-B gate");
            return;
        };
        assert!(!reports.is_empty(), "V2-B battery must not be empty");
        let mut failures = Vec::new();
        for r in &reports {
            if !r.passed() {
                failures.push(format!(
                    "  {} [{}]: lanes={} merkle={} root={} replay={}",
                    r.case, r.backend, r.lanes_match, r.merkle_match, r.root_match, r.replay_match
                ));
            }
        }
        assert!(
            failures.is_empty(),
            "V2-B segmented (seg={seg}) FAILED for {} of {} case(s):\n{}",
            failures.len(),
            reports.len(),
            failures.join("\n")
        );
    }
}
