//! `DigestEquivalenceHarnessV1` — the law every CUDA evidence-kernel change must pass (P70).
//!
//! # Why this exists
//! Nsight Compute counters (measured on the RTX 4080 SUPER — see
//! `reports/NSIGHT_SUMMARY.md` and `docs/cuda_evidence_kernel_v2_design.md`) show the V1
//! `evidence_kernel` runs at ~8.3% warp occupancy / <11% SM throughput on a single run: it is
//! **grid-starved**, not memory-bound. That measurement motivates a faster V2 kernel. But the GPU
//! here is an *acceleration + attestation* layer, never a second source of truth: correctness is
//! defined by byte-exact agreement with the CPU reference. So **any** kernel that claims to replace
//! or augment V1 must first prove it changes *only timing*, never a sealed byte.
//!
//! # The law
//! For identical input, a candidate evidence producer MUST reproduce, byte-for-byte, the CPU
//! reference's
//!   1. per-lane [`LaneEvidence`] (digest + summary fields),
//!   2. Merkle root over the lane digests,
//!   3. [`crate::court::evidence_root`], and
//!   4. replay verdict (`verify_replay` of the candidate lanes against the reference case file).
//!
//! Only wall-clock time may differ. A producer that changes any of (1)–(4) is **not** a V2 of this
//! kernel — it is a new, separately-versioned evidence *format* (see the design doc's V2-B).
//!
//! # The battery
//! [`battery`] builds adversarial input shapes that have historically broken byte-exactness, so the
//! gate catches regressions the 20-dataset replay set might miss:
//! - **SHA-padding boundaries.** Each sample feeds 40 bytes into SHA-256, whose compression runs on
//!   64-byte blocks; the per-lane message length crosses a block boundary on a period tied to
//!   `n_samples (mod 8)`. `n_samples ≡ 3 (mod 8)` is the exact class that hid the P43 two-block
//!   padding bug, so the battery covers that residue and its neighbours explicitly.
//! - **Warp / block edges.** Lane counts one short of, exactly at, and one past the 128-thread block
//!   (and a warp) catch off-by-one grid/guard errors.
//! - **Out-of-band path.** A case salted with `NaN`/`±inf` exercises the non-finite handling
//!   (sealed as `q = 0`, counted in `oob_count`) on both backends.
//! - **Degenerate all-zero residuals.** Every detector quiet, but the digest is still well-defined.
//!
//! Built with `--features cuda` and a device present, the candidate is the GPU `evidence_kernel`, so
//! this is a real CPU↔GPU gate over the battery. Built without CUDA (or with no device), the
//! candidate is the CPU path again — a determinism self-check that still validates the harness and
//! the Merkle/root/replay chain. A future V2 kernel is wired in as an additional candidate here.

use crate::court::CaseFile;
use crate::dispatch::{evidence_cpu, Lanes};
use crate::evidence::LaneEvidence;
use sha2::{Digest, Sha256};

/// Outcome of one equivalence check over a single input shape. Every boolean must be `true`.
#[derive(Debug, Clone)]
pub struct EquivalenceReport {
    /// Human-readable battery case label (e.g. `"rand 96x11 (ns mod8=3)"`).
    pub case: String,
    /// Which candidate backend produced the comparison (`"cuda:<device>"` or a CPU label).
    pub backend: String,
    pub n_lanes: u32,
    pub n_samples: u32,
    /// Candidate per-lane evidence equals the CPU reference (digest + every summary field).
    pub lanes_match: bool,
    /// Candidate Merkle root over lane digests equals the reference Merkle root.
    pub merkle_match: bool,
    /// Candidate `evidence_root` equals the reference `evidence_root`.
    pub root_match: bool,
    /// Candidate lanes replay byte-exact against the reference case file.
    pub replay_match: bool,
    pub reference_root: String,
    pub candidate_root: String,
}

/// How a CUDA kernel optimisation is admitted relative to the V1 sealed reference. Every optimisation must fall
/// into exactly one of these — so a performance change can never *silently* mutate forensic identity (the panel's
/// "no silent mutation, ever"). The load-bearing invariant is the per-lane evidence (`lanes_match`): if the data
/// each lane sealed is byte-identical, a differing root can only be a deliberate re-sealing *format*, not a data
/// regression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaOptimizationStatus {
    /// Byte-exact: the candidate reproduces the entire V1 sealed chain (lanes + Merkle + `evidence_root` + replay).
    DigestIdenticalOptimization,
    /// Per-lane evidence is byte-identical but the root/Merkle *construction* differs by design (e.g. the V2-B
    /// segmented `evidence_root_v2`) — admitted as a declared new format version, not a regression.
    NewEvidenceFormatVersion,
    /// The per-lane evidence itself diverged — the optimisation mutated forensic data; rejected, must not ship.
    RejectedPerformanceRegression,
}

impl CudaOptimizationStatus {
    /// Stable tag for reports / logs.
    pub fn tag(self) -> &'static str {
        match self {
            CudaOptimizationStatus::DigestIdenticalOptimization => "digest-identical-optimization",
            CudaOptimizationStatus::NewEvidenceFormatVersion => "new-evidence-format-version",
            CudaOptimizationStatus::RejectedPerformanceRegression => {
                "rejected-performance-regression"
            }
        }
    }
}

impl EquivalenceReport {
    /// The law holds for this case iff the full sealed chain matches.
    pub fn passed(&self) -> bool {
        self.lanes_match && self.merkle_match && self.root_match && self.replay_match
    }

    /// Classify this report against the V1 sealed reference (see [`CudaOptimizationStatus`]). An optimisation is
    /// admitted ONLY as digest-identical (full chain matches) or as a declared new evidence format (per-lane
    /// evidence preserved, root construction changed); anything that perturbs the per-lane evidence is a rejected
    /// regression. `lanes_match` is decisive because it is the check that the *data* each lane sealed is unchanged.
    pub fn optimization_status(&self) -> CudaOptimizationStatus {
        if self.passed() {
            CudaOptimizationStatus::DigestIdenticalOptimization
        } else if self.lanes_match {
            CudaOptimizationStatus::NewEvidenceFormatVersion
        } else {
            CudaOptimizationStatus::RejectedPerformanceRegression
        }
    }
}

/// Build a synthetic sample-major [`Lanes`] of the given shape, each element filled by `f(sample,lane)`.
///
/// `input_sha256` is computed exactly as [`crate::dispatch::build_lanes`] does (SHA-256 over the raw
/// IEEE-754 bits of the buffer, in order), so the passport — and therefore `evidence_root` — is
/// derived the same way a real dataset's would be.
fn make_lanes(n_lanes: u32, n_samples: u32, f: impl Fn(u32, u32) -> f64) -> Lanes {
    let nl = n_lanes as usize;
    let ns = n_samples as usize;
    let mut data = vec![0.0f64; nl * ns];
    for i in 0..ns {
        for l in 0..nl {
            data[i * nl + l] = f(i as u32, l as u32);
        }
    }
    let mut h = Sha256::new();
    for &x in &data {
        h.update(x.to_bits().to_le_bytes());
    }
    let input_sha256 = h.finalize().iter().map(|b| format!("{:02x}", b)).collect();
    Lanes {
        data,
        n_lanes,
        n_samples,
        input_sha256,
    }
}

/// Deterministic pseudo-random residual (SplitMix64 hashed to a signed, spread-magnitude double), so
/// the battery is byte-reproducible across machines and runs.
fn prand(i: u32, l: u32) -> f64 {
    // Mix the (sample, lane) coordinates into a 64-bit seed, then run one SplitMix64 step.
    let mut z = ((i as u64) << 32 | l as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // Map to roughly [-50, 50) with sub-integer detail, so quantisation, drift, and slew all vary.
    ((z % 100_000) as f64) / 1000.0 - 50.0
}

/// The adversarial battery of synthetic input shapes (see module docs for the rationale of each).
pub fn battery() -> Vec<(String, Lanes)> {
    let mut v = Vec::new();
    // SHA-padding boundary sweep: cover n_samples mod 8 = {1,2,3,0,3,0,3,0,3} incl. the P43 class.
    for &ns in &[1u32, 2, 3, 8, 11, 16, 19, 64, 67] {
        v.push((
            format!("rand 96x{ns} (ns mod8={})", ns % 8),
            make_lanes(96, ns, prand),
        ));
    }
    // Warp / block (128-thread) edges.
    for &nl in &[1u32, 7, 31, 32, 33, 127, 128, 129, 256] {
        v.push((
            format!("rand {nl}x50 (lane/block edge)"),
            make_lanes(nl, 50, prand),
        ));
    }
    // Out-of-band path: NaN / +inf / -inf scattered through otherwise-random residuals.
    v.push((
        "non-finite (NaN/inf) oob path 64x40".into(),
        make_lanes(64, 40, |i, l| match (i + l) % 13 {
            0 => f64::NAN,
            5 => f64::INFINITY,
            9 => f64::NEG_INFINITY,
            _ => prand(i, l),
        }),
    ));
    // Degenerate all-zero residuals.
    v.push(("all-zero 64x40".into(), make_lanes(64, 40, |_, _| 0.0)));
    v
}

/// Reference evidence: the CPU path *is* the definition of correctness.
pub fn reference_evidence(lanes: &Lanes) -> Vec<LaneEvidence> {
    evidence_cpu(lanes)
}

/// Candidate evidence: the GPU `evidence_kernel` when built `--features cuda` with a device present,
/// else the CPU path again (a determinism self-check). Returns `(lanes, backend-label)`.
pub fn candidate_evidence(lanes: &Lanes) -> (Vec<LaneEvidence>, String) {
    #[cfg(feature = "cuda")]
    {
        match crate::ffi::run_evidence(&lanes.data, lanes.n_lanes, lanes.n_samples) {
            Ok(gpu) => (
                decode_gpu(lanes, &gpu),
                format!("cuda:{}", crate::ffi::device_name()),
            ),
            // No device / CUDA error: fall back to CPU so the harness still runs (and is honest the
            // GPU did not execute via the backend label).
            Err(_) => (evidence_cpu(lanes), "cpu(no-cuda-device-fallback)".into()),
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        (evidence_cpu(lanes), "cpu(determinism-self-check)".into())
    }
}

/// Decode the flat GPU output buffers into per-lane [`LaneEvidence`] (same layout as `dispatch::try_cuda`).
#[cfg(feature = "cuda")]
fn decode_gpu(lanes: &Lanes, gpu: &crate::ffi::GpuEvidence) -> Vec<LaneEvidence> {
    (0..lanes.n_lanes as usize)
        .map(|l| {
            let dig = &gpu.digests[l * 32..l * 32 + 32];
            let s = &gpu.summary[l * 5..l * 5 + 5];
            LaneEvidence {
                lane_id: l as u32,
                n_samples: lanes.n_samples,
                // Counts are bounded by n_samples, so they fit u32; a value that does not is a corrupt
                // GPU result and the equality check below will flag it.
                n_exceedances: u32::try_from(s[0]).unwrap_or(u32::MAX),
                oob_count: u32::try_from(s[1]).unwrap_or(u32::MAX),
                peak_exceedance: s[2],
                peak_abs_drift: s[3],
                peak_abs_slew: s[4],
                digest_hex: dig.iter().map(|b| format!("{:02x}", b)).collect(),
            }
        })
        .collect()
}

/// Compare candidate evidence to the reference over the full sealed chain for one input.
pub fn compare(
    case: &str,
    lanes: &Lanes,
    reference: &[LaneEvidence],
    candidate: &[LaneEvidence],
    backend: &str,
) -> EquivalenceReport {
    let ns_total = lanes.n_samples as u64;
    // Assemble a case file for each producer (same dataset id + input digest, so the passport is
    // identical and any root difference is purely a lane-evidence difference).
    let cf_ref = CaseFile::assemble(
        "equivalence-harness",
        lanes.input_sha256.clone(),
        ns_total,
        reference.to_vec(),
        None,
    );
    let cf_cand = CaseFile::assemble(
        "equivalence-harness",
        lanes.input_sha256.clone(),
        ns_total,
        candidate.to_vec(),
        None,
    );
    // Replay the candidate lanes against the *reference* case file: the strongest check, since it
    // re-derives the root from the candidate and asserts it equals the reference's sealed root.
    let replay = cf_ref.verify_replay(candidate);
    EquivalenceReport {
        case: case.into(),
        backend: backend.into(),
        n_lanes: lanes.n_lanes,
        n_samples: lanes.n_samples,
        lanes_match: reference == candidate,
        merkle_match: cf_ref.merkle_root == cf_cand.merkle_root,
        root_match: cf_ref.evidence_root == cf_cand.evidence_root,
        replay_match: replay.matched,
        reference_root: cf_ref.evidence_root,
        candidate_root: cf_cand.evidence_root,
    }
}

/// Run the whole battery; returns one [`EquivalenceReport`] per case. The gate passes iff every
/// report `passed()`.
pub fn run_harness() -> Vec<EquivalenceReport> {
    battery()
        .into_iter()
        .map(|(label, lanes)| {
            let reference = reference_evidence(&lanes);
            let (candidate, backend) = candidate_evidence(&lanes);
            compare(&label, &lanes, &reference, &candidate, &backend)
        })
        .collect()
}

/// V2-A gate: pack **every** battery case's lanes into ONE batched launch (lane-contiguous, jagged),
/// run `evidence_kernel_batched`, then check each case's lanes against the CPU reference.
///
/// This is the proof that V2-A is **digest-identical** to V1: a single launch spanning many
/// heterogeneous "datasets" (different sample counts) must reproduce, per lane, exactly the per-lane
/// evidence the CPU reference produces for that lane in isolation. It returns `None` when CUDA is not
/// compiled in or no device is present (the gate is a GPU-only check; the CPU determinism self-check
/// in [`run_harness`] still runs everywhere).
#[cfg(feature = "cuda")]
pub fn run_batched_harness() -> Option<Vec<EquivalenceReport>> {
    let cases = battery();
    // Concatenate all lanes into one lane-contiguous buffer with per-lane (offset, nsamples).
    let mut xcat: Vec<f64> = Vec::new();
    let mut offsets: Vec<u64> = Vec::new();
    let mut nsamples: Vec<u32> = Vec::new();
    // Per-case (global-lane-start, n_lanes), to slice the flat output back out per case.
    let mut layout: Vec<(usize, usize)> = Vec::new();
    for (_, lanes) in &cases {
        let nl = lanes.n_lanes as usize;
        let ns = lanes.n_samples as usize;
        layout.push((offsets.len(), nl));
        for l in 0..nl {
            offsets.push(xcat.len() as u64);
            nsamples.push(ns as u32);
            // De-interleave sample-major → lane-contiguous: lane l's samples in time order.
            for i in 0..ns {
                xcat.push(lanes.data[i * nl + l]);
            }
        }
    }
    let gpu = crate::ffi::run_evidence_batched(&xcat, &offsets, &nsamples).ok()?;
    let backend = format!("cuda-batched:{}", crate::ffi::device_name());
    let mut reports = Vec::new();
    for (ci, (label, lanes)) in cases.iter().enumerate() {
        let (gstart, nl) = layout[ci];
        // Decode this case's lanes from the flat batched output; local lane_id 0..nl to match the
        // CPU reference (lane_id is not part of the digest — the SHA is over samples only).
        let candidate: Vec<LaneEvidence> = (0..nl)
            .map(|l| {
                let g = gstart + l;
                let dig = &gpu.digests[g * 32..g * 32 + 32];
                let s = &gpu.summary[g * 5..g * 5 + 5];
                LaneEvidence {
                    lane_id: l as u32,
                    n_samples: lanes.n_samples,
                    n_exceedances: u32::try_from(s[0]).unwrap_or(u32::MAX),
                    oob_count: u32::try_from(s[1]).unwrap_or(u32::MAX),
                    peak_exceedance: s[2],
                    peak_abs_drift: s[3],
                    peak_abs_slew: s[4],
                    digest_hex: dig.iter().map(|b| format!("{:02x}", b)).collect(),
                }
            })
            .collect();
        let reference = reference_evidence(lanes);
        reports.push(compare(
            &format!("V2-A batched: {label}"),
            lanes,
            &reference,
            &candidate,
            &backend,
        ));
    }
    Some(reports)
}

/// V2-B gate: run the segment-parallel kernel over every battery case (segment size `seg`), combine
/// per lane (concat segment digests → SHA-256 = `lane_root_v2`; reduce partial summaries), and check
/// each lane against the CPU `lane_evidence_v2_cpu` reference for the same `seg`. This proves the GPU
/// V2-B kernel — with its halo warm-up and per-segment hashing — reproduces the authoritative
/// Merkle-segment format byte-for-byte. Returns `None` when CUDA is absent / no device.
#[cfg(feature = "cuda")]
pub fn run_v2_segmented_harness(seg: usize) -> Option<Vec<EquivalenceReport>> {
    use crate::evidence::{lane_evidence_v2_cpu, DRIFT_WINDOW};
    use sha2::{Digest, Sha256};
    let cases = battery();
    // Concatenate lanes (lane-contiguous) and build the per-segment descriptors.
    let mut xcat: Vec<f64> = Vec::new();
    let (mut seg_base, mut seg_start, mut seg_end, mut seg_warmup) = (
        Vec::<u64>::new(),
        Vec::<u32>::new(),
        Vec::<u32>::new(),
        Vec::<u32>::new(),
    );
    // One (case_index, lane_index, n_segments) per lane, in the order segments were emitted.
    let mut lane_segs: Vec<(usize, usize, usize)> = Vec::new();
    for (ci, (_, lanes)) in cases.iter().enumerate() {
        let nl = lanes.n_lanes as usize;
        let ns = lanes.n_samples as usize;
        for l in 0..nl {
            let off = xcat.len() as u64;
            for i in 0..ns {
                xcat.push(lanes.data[i * nl + l]); // de-interleave to lane-contiguous
            }
            let nseg = if ns == 0 { 0 } else { ns.div_ceil(seg) };
            for k in 0..nseg {
                let s = k * seg;
                let e = ((k + 1) * seg).min(ns);
                seg_base.push(off);
                seg_start.push(s as u32);
                seg_end.push(e as u32);
                seg_warmup.push(s.min(DRIFT_WINDOW) as u32);
            }
            lane_segs.push((ci, l, nseg));
        }
    }
    let gpu =
        crate::ffi::run_evidence_v2_segmented(&xcat, &seg_base, &seg_start, &seg_end, &seg_warmup)
            .ok()?;
    let backend = format!("cuda-v2seg:{}", crate::ffi::device_name());
    // Combine per lane (segments were emitted in lane_segs order; walk a global segment cursor).
    let mut per_case: std::collections::BTreeMap<usize, Vec<LaneEvidence>> = Default::default();
    let mut gseg = 0usize;
    for (ci, l, nseg) in &lane_segs {
        let mut concat = Vec::with_capacity(nseg * 32);
        let (mut n_exc, mut oob, mut pke, mut pkd, mut pks) = (0u32, 0u32, 0i64, 0i64, 0i64);
        for _ in 0..*nseg {
            concat.extend_from_slice(&gpu.seg_digests[gseg * 32..gseg * 32 + 32]);
            let s = &gpu.seg_summary[gseg * 5..gseg * 5 + 5];
            // Disjoint segments: counts are additive, peaks are the max across segments.
            n_exc += s[0] as u32;
            oob += s[1] as u32;
            pke = pke.max(s[2]);
            pkd = pkd.max(s[3]);
            pks = pks.max(s[4]);
            gseg += 1;
        }
        let mut h = Sha256::new();
        h.update(&concat);
        let root = h.finalize();
        let lanes = &cases[*ci].1;
        per_case.entry(*ci).or_default().push(LaneEvidence {
            lane_id: *l as u32,
            n_samples: lanes.n_samples,
            n_exceedances: n_exc,
            oob_count: oob,
            peak_exceedance: pke,
            peak_abs_drift: pkd,
            peak_abs_slew: pks,
            digest_hex: root.iter().map(|b| format!("{:02x}", b)).collect(),
        });
    }
    let mut reports = Vec::new();
    for (ci, (label, lanes)) in cases.iter().enumerate() {
        let nl = lanes.n_lanes as usize;
        let ns = lanes.n_samples as usize;
        let reference: Vec<LaneEvidence> = (0..nl)
            .map(|l| {
                let seq: Vec<f64> = (0..ns).map(|i| lanes.data[i * nl + l]).collect();
                lane_evidence_v2_cpu(l as u32, &seq, seg)
            })
            .collect();
        let candidate = per_case.remove(&ci).unwrap_or_default();
        reports.push(compare(
            &format!("V2-B seg={seg}: {label}"),
            lanes,
            &reference,
            &candidate,
            &backend,
        ));
    }
    Some(reports)
}

#[cfg(test)]
mod opt_status_tests {
    //! CPU-only classification tests for [`CudaOptimizationStatus`] (no GPU / `--features cuda` needed): the
    //! enum + `optimization_status()` derive purely from the report's booleans, so they are fully checkable here.
    use super::*;

    fn report(lanes: bool, merkle: bool, root: bool, replay: bool) -> EquivalenceReport {
        EquivalenceReport {
            case: "synthetic".into(),
            backend: "cpu-test".into(),
            n_lanes: 4,
            n_samples: 8,
            lanes_match: lanes,
            merkle_match: merkle,
            root_match: root,
            replay_match: replay,
            reference_root: "ref".into(),
            candidate_root: "cand".into(),
        }
    }

    #[test]
    fn full_chain_match_is_digest_identical() {
        assert_eq!(
            report(true, true, true, true).optimization_status(),
            CudaOptimizationStatus::DigestIdenticalOptimization
        );
    }

    #[test]
    fn lane_identical_but_different_root_is_new_format() {
        // The V2-B segmented case: per-lane evidence byte-identical, root/Merkle construction differs by design.
        assert_eq!(
            report(true, false, false, true).optimization_status(),
            CudaOptimizationStatus::NewEvidenceFormatVersion
        );
    }

    #[test]
    fn lane_evidence_divergence_is_rejected() {
        // The data each lane sealed differs ⇒ a real regression, never admitted.
        assert_eq!(
            report(false, false, false, true).optimization_status(),
            CudaOptimizationStatus::RejectedPerformanceRegression
        );
        assert_eq!(
            report(false, true, true, true).optimization_status(),
            CudaOptimizationStatus::RejectedPerformanceRegression
        );
    }

    #[test]
    fn status_tags_are_stable() {
        assert_eq!(
            CudaOptimizationStatus::DigestIdenticalOptimization.tag(),
            "digest-identical-optimization"
        );
        assert_eq!(
            CudaOptimizationStatus::NewEvidenceFormatVersion.tag(),
            "new-evidence-format-version"
        );
        assert_eq!(
            CudaOptimizationStatus::RejectedPerformanceRegression.tag(),
            "rejected-performance-regression"
        );
    }
}
