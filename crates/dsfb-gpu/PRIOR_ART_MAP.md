# DSFB-GPU-Debug — Prior-Art Map

This document maps every disclosed architecture element of
**DSFB-GPU-Debug** to its location in code, tests, receipts, and
sealed commit history. It is the examiner / reviewer / attorney /
competitor map for the prior-art evidence package
(see [`CLAIM_BOUNDARY_MATRIX.md`](CLAIM_BOUNDARY_MATRIX.md) and
[`TIMESTAMP_RECEIPT.md`](TIMESTAMP_RECEIPT.md)).

**Disclaimer.** This is not legal advice. The map records where the
disclosed architecture lives; it does not assert patentability,
priority, or any prior-art ruling. Counsel evaluates those properties
against the public-accessibility record. See the licence and IP notes
in [`LICENSE`](LICENSE), [`NOTICE`](NOTICE), and the paper's IP-notice
tcolorbox.

Repository: [`https://github.com/infinityabundance/dsfb`](https://github.com/infinityabundance/dsfb).
Zenodo DOI (v1.0, sealed 2026-05-22): [`10.5281/zenodo.20338027`](https://doi.org/10.5281/zenodo.20338027).
SWHID assignment is pending an operator-side Software Heritage
save-code-now request; the Zenodo deposit is the primary public
prior-art timestamp anchor.

Sealed commit at PA.1 seal: `2281cc8`. Every per-element
"Commit" field below is the sealed commit of the campaign that
landed the element (e.g. PAPER.1e at `117c237` for Element 1
*Endoductive evidence court*); the artifact paths and contents
have been byte-stable since those commits.

---

## Element 1 — Endoductive evidence court

- **Name:** Endoduction as the fourth mode of inference (deterministic
  adjudication of internal evidence-field relations into a replayable
  structural verdict).
- **Paper section:** §2.1 (`sec:endoduction`); threaded into the
  abstract and conclusion.
- **Source files:** doctrinal vocabulary only; the inference mode is
  implemented across the whole pipeline (elements 3–8 below).
- **Tests:** the hostile-reviewer scope guard run at PAPER.1e seal
  confirms "endoduct" appears in exactly the abstract, §2.1 body,
  and conclusion (7 total hits, no bleed-through).
- **Receipts:** the archived prior-art PDF §2.1.
- **Hashes:** the archived prior-art PDF SHA-256 pinned in
  [`ARTIFACT_MANIFEST.v1.toml`](ARTIFACT_MANIFEST.v1.toml).
- **Commit:** PAPER.1e at `117c237`.
- **Non-claim:** Endoduction is the plan-locked label for what
  DSFB-GPU already does; it is not claimed as a philosophical
  novelty, and the four modes named (deduction / induction /
  abduction / endoduction) are not claimed as exhaustive.

## Element 2 — Densor / tekmeric evidence model

- **Name:** Densorial inference (deterministic inference over
  *densors* — typed, hashable evidence objects) + tekmeric inference
  (evidence-based deterministic adjudication of witnesses).
- **Paper section:** §2 *Densorial and Tekmeric Inference*
  (`sec:densorial`).
- **Source files:** `crates/dsfb-gpu-debug-core/src/event.rs`
  (TraceEvent / residual-cell densors),
  `crates/dsfb-gpu-debug-core/src/residual.rs`,
  `crates/dsfb-gpu-debug-core/src/sign.rs`,
  `crates/dsfb-gpu-debug-core/src/detector.rs`,
  `crates/dsfb-gpu-debug-core/src/consensus.rs`,
  `crates/dsfb-gpu-debug-core/src/candidate.rs`.
- **Tests:** `crates/dsfb-gpu-debug-core/tests/property.rs`,
  `crates/dsfb-gpu-debug-core/tests/breach.rs`.
- **Receipts:** the archived prior-art PDF §2.
- **Hashes:** pinned in `ARTIFACT_MANIFEST.v1.toml`.
- **Commit:** doctrine landed across the corpus and paper arc; the
  vocabulary is plan-locked.
- **Non-claim:** Densorial inference does not replace neural
  inference; the v0 implementation is bounded to fixed-point
  Q16.16 evidence on a deterministic GPU path.

## Element 3 — CUDA evidence factory / CPU court split

- **Name:** GPU-accelerated deterministic evidence production with
  CPU-side semantic authority (the *evidence-court split*).
- **Paper section:** §3 *CUDA Evidence Factory*
  (`sec:cuda-evidence-factory`); Fig 1 *Evidence-court pipeline*
  (`fig:evidence-court-pipeline`); §A.6 *DSFB-GPU-Debug Architecture*
  with the detailed pipeline figure (`fig:architecture`).
- **Source files:** `crates/dsfb-gpu-debug-cuda/src/dispatch.rs`
  (CUDA dispatch path), `crates/dsfb-gpu-debug-cuda/src/ffi.rs`
  (FFI surface), `cuda/kernels.cu` (the five evidence kernels and the
  fused throughput variants), `crates/dsfb-gpu-debug-core/src/bank.rs`
  + `crates/dsfb-gpu-debug-core/src/episode.rs` (CPU court).
- **Tests:** `crates/dsfb-gpu-debug-demo/tests/cross_stage_chain.rs`,
  `crates/dsfb-gpu-debug-demo/tests/cli_smoke.rs`.
- **Receipts:** the archived prior-art PDF §3 + Fig 1.
- **Hashes:** sealed receipts under `reports/s_real_<tier>/<id>/`
  carry the per-dataset case-file hashes that anchor the
  evidence-factory output.
- **Commit:** doctrinal seal at the densorial / tekmeric front-door
  identity commit (`6dab121`); Fig 1 sealed at PAPER.1d (`72f5b31`).
- **Non-claim:** the GPU never admits episodes; the CPU bank is the
  only path that can mint a `BankAdmissionToken`.

## Element 4 — Semantic Non-Bypass Axiom

- **Name:** Axiomatic prohibition on the GPU minting admitted
  episodes; the CPU bank holds semantic authority.
- **Paper section:** Propositions (Semantic Non-Bypass); §A.6
  architecture; Fig 1 *Evidence-court pipeline* dashed-boundary
  visualisation.
- **Source files:** `crates/dsfb-gpu-debug-core/src/episode.rs` (the
  `Episode` type carries `Option<BankAdmissionToken>`),
  `crates/dsfb-gpu-debug-core/src/bank.rs` (private constructor),
  `crates/dsfb-gpu-debug-core/src/casefile.rs` (verdict
  `SemanticBypassRejected` if an episode lacks the admission token).
- **Tests:** `crates/dsfb-gpu-debug-core/tests/breach.rs::semantic_bypass_rejected`.
- **Receipts:** the archived prior-art PDF Propositions section.
- **Hashes:** the per-dataset case-file hashes pinned in
  `reports/s_real_3/bundle_manifest.toml` are the empirical witness
  that no GPU short-circuit produced any of the 316 admitted episodes.
- **Commit:** v0 architecture seal (the axiom is load-bearing from
  the earliest commits).
- **Non-claim:** the axiom is a v0 type-level enforcement, not a
  formal verification proof.

## Element 5 — BankAdmissionToken private-constructor enforcement

- **Name:** Token-based access control that prevents any caller
  outside `bank.rs` from constructing an admitted `Episode`.
- **Paper section:** §A.6 *Architecture*; Fig 1 (the cross-boundary
  arrow labelled `private BankAdmissionToken`).
- **Source files:** `crates/dsfb-gpu-debug-core/src/bank.rs`
  (the `BankAdmissionToken` type with its `pub(super)` /
  module-private constructor); consumers in
  `crates/dsfb-gpu-debug-core/src/episode.rs`.
- **Tests:** `crates/dsfb-gpu-debug-core/tests/breach.rs`.
- **Receipts:** the archived prior-art PDF §A.6.
- **Commit:** v0; unchanged across the prior-art arc.
- **Non-claim:** access control is module-scoped Rust visibility, not
  hardware-enforced memory isolation.

## Element 6 — Q16.16 fixed-point deterministic numeric contract

- **Name:** Q16.16 fixed-point arithmetic with locked rounding,
  banker's-round-to-even at bit 15, no FMA, no fast-math, no atomics
  for accumulation, no warp shuffles for reduction.
- **Paper section:** §A.9 *Fixed-Point Determinism*; §A.7
  *Deterministic Execution Contract*.
- **Source files:** `crates/dsfb-gpu-debug-core/src/fixed.rs` (Rust
  reference); `cuda/common.cuh` (CUDA mirror); `cuda/kernels.cu`
  (consumers); `contract.toml`
  (`[numeric] mode = "fixed_q16"`).
- **Tests:** `crates/dsfb-gpu-debug-core/tests/property.rs`
  (Theorem-9 analog over 64 LCG seeds; same-device byte-exact replay).
- **Receipts:** the archived prior-art PDF §A.9.
- **Commit:** v0 prior-art proof.
- **Non-claim:** byte-exact CPU↔GPU equivalence is asserted only on
  the recorded toolchain (same device, same binary, same driver,
  same CUDA version); cross-driver / cross-CUDA-version / cross-
  hardware byte-identity is NOT claimed.

## Element 7 — Locked CUDA kernel sequence

- **Name:** Fixed five-kernel evidence sequence (residual_field →
  drift_slew_sign → detector_motif → consensus_grid →
  candidate_collapse) whose order is pinned by the execution
  contract; reordering triggers `KernelSequenceMismatch`.
- **Paper section:** §A.8 *CUDA Kernel Mapping*; Fig
  `fig:kernels` *kernel_sequence*.
- **Source files:** `cuda/kernels.cu` (kernel bodies);
  `crates/dsfb-gpu-debug-cuda/src/dispatch.rs` (dispatch order);
  `contract.toml` (`[kernels].sequence`).
- **Tests:** `crates/dsfb-gpu-debug-core/tests/breach.rs::kernel_sequence_mismatch`.
- **Receipts:** the archived prior-art PDF §A.8.
- **Commit:** v0.
- **Non-claim:** the sequence is the v0 evidence-production order;
  Throughput-mode fused variants (sealed in the S-PERF / S-REAL
  arc) preserve the same per-stage hashes via plan-locked
  byte-identity gates but do NOT alter the sequence's logical
  meaning.

## Element 8 — Stage hash chain / verdict case file

- **Name:** Twelve-link Merkle-style hash chain from input through
  every intermediate stage to the final verdict case file. Each link
  hashes its stage bytes plus the previous link's hash.
- **Paper section:** §A.11 *Verdict Case File*; Fig `fig:hashchain`
  *hash_chain*.
- **Source files:** `crates/dsfb-gpu-debug-core/src/hash.rs` (Q16.16
  scalar SHA-256); `crates/dsfb-gpu-debug-core/src/casefile.rs`
  (chain construction + verdict emission);
  `crates/dsfb-gpu-debug-core/src/serialize.rs` (canonical bytes).
- **Tests:** `crates/dsfb-gpu-debug-core/tests/replay.rs`,
  `crates/dsfb-gpu-debug-demo/tests/cross_stage_chain.rs`.
- **Receipts:** `reports/s_real_3/bundle_hash_chain.txt` (60 rows,
  20 datasets × 3 chain links each), pinned by SHA-256 in
  `ARTIFACT_MANIFEST.v1.toml`.
- **Commit:** S-REAL.3 audit-gauntlet seal at `a8aaa04`.
- **Non-claim:** the chain pins the artifact bytes that produced a
  given verdict; it does not claim that the upstream input bytes
  are themselves ground-truth labels.

## Element 9 — Device Traffic Receipt / measurement law (S-PERF.1)

- **Name:** Plan-locked byte-accounting receipt + 8-line
  bandwidth-claim policy + 8 plan-required negatives; sealed
  measurement law that every subsequent CUDA bandwidth claim must
  cite.
- **Paper section:** §5 *Device Traffic Receipt (Measurement Law)*
  (`sec:device-traffic-receipt`).
- **Source files:** `crates/dsfb-gpu-atlas-corpus/src/s_perf_1_device_traffic_receipt.rs`.
- **Tests:** `crates/dsfb-gpu-atlas-corpus/tests/s_perf_1_device_traffic_receipt_invariants.rs`.
- **Receipts:** the archived prior-art PDF §5; corpus receipts under
  `reports/s_perf_1_*.txt`.
- **Commit:** S-PERF.1 at `9575ce3`.
- **Non-claim:** the receipt does not assert any measured bandwidth
  at baseline; it asserts the law that future claims must obey.

## Element 10 — Layer-A resident densor pipeline (S-PERF.2)

- **Name:** Plan-locked five-stage device-resident evidence pipeline
  (EvidenceDensorProjection / WitnessDensorEvaluation /
  FusionDensorReduction / CandidateDensorCollapse /
  StageDigestEmission) with five forbidden-host-activity flags.
- **Paper section:** §6 *Layer-A Resident Densor Pipeline*
  (`sec:layer-a-resident-pipeline`).
- **Source files:** `crates/dsfb-gpu-atlas-corpus/src/s_perf_2_layer_a_resident_pipeline.rs`.
- **Tests:** `crates/dsfb-gpu-atlas-corpus/tests/s_perf_2_layer_a_resident_pipeline_invariants.rs`.
- **Receipts:** the archived prior-art PDF §6.
- **Commit:** S-PERF.2 at `1c78ca4`.
- **Non-claim:** S-PERF.2 declares the pipeline shape; it does not
  benchmark throughput on it (the saturation evidence lives in
  S-REAL.3.1 and S-PERF.16.a).

## Element 11 — Family compaction / detector-count-not-kernel-count

- **Name:** Active-detector family-compaction schema (152 active
  detectors compacted into 14 GPU-family lanes per S-PERF.4) and the
  plan-locked rule that detector count is NOT kernel count.
- **Paper section:** §13 *Active-Detector Family Compaction*
  (`sec:active-family-compaction`).
- **Source files:** `crates/dsfb-gpu-atlas-corpus/src/s_perf_4_active_family_compaction.rs`.
- **Tests:** `crates/dsfb-gpu-atlas-corpus/tests/s_perf_4_active_family_compaction_invariants.rs`.
- **Receipts:** the archived prior-art PDF §13.
- **Commit:** S-PERF.4 at `0a1ab3b`.
- **Non-claim:** active-detector compaction is a benchmark-shape
  rule, not a claim that any specific detector family is sufficient
  for any domain task.

## Element 12 — Digest preservation contract (S-PERF.10)

- **Name:** Plan-locked digest-preservation laws (same-mode digest
  root law, canonical fragment-merge order, digest-mode non-aliasing,
  case-file chain preservation) that any future digest-compaction
  kernel rewrite must satisfy.
- **Paper section:** §18 *DigestLanePlanV1 / Digest-Cost Audit*
  (`sec:digest-lane-plan`).
- **Source files:** `crates/dsfb-gpu-atlas-corpus/src/s_perf_10_digest_lane_plan.rs`.
- **Tests:** `crates/dsfb-gpu-atlas-corpus/tests/s_perf_10_digest_lane_plan_invariants.rs`.
- **Receipts:** the archived prior-art PDF §18.
- **Commit:** S-PERF.10 at `14bdc18`.
- **Non-claim:** the contract preserves digest roots WITHIN a
  declared digest mode; cross-mode root equality is NOT required.

## Element 13 — A6.1 structural fusion optimisation

- **Name:** Structural window-walk fusion of
  `detector_motif_fused_d64_kernel` cutting per-kernel duration
  1,324,608 → 910,283 ns (−31.3 %, 1.46× faster) and L1 LOAD
  2,339,556,672 → 1,139,949,568 bit-exact across 5 captures
  (−51.3 %, 2.05× cut). The A5 v1/v2/v2.1 source-level caching
  attempts hit the same L1 LOAD floor bit-exactly across three
  independent rewrites (structural amplification, not source-level
  cacheability).
- **Paper section:** §11.3 *A6.1 — Null-to-Structural-Fusion
  Optimisation* (`sec:case-a6-1`); Fig `fig:a6-1-before-after`.
- **Source files:** `cuda/kernels.cu` (the post-A6.1
  `detector_motif_fused_d64_kernel` body); the A5 archive is
  preserved in the git history under the A5 series of commits.
- **Tests:** `crates/dsfb-gpu-debug-demo/tests/s_perf_16_a6_1_post.rs`
  (load-bearing byte-identity pins on the post-A6.1 path).
- **Receipts:** `reports/s_perf_16_a6_1_post.txt`, pinned by SHA-256
  in `ARTIFACT_MANIFEST.v1.toml`.
- **Commit:** A6.1 sealed at `3e84e05`.
- **Non-claim:** A6.1 is a local kernel win bound by Tier 1 sealed-
  artifact byte-counter evidence on RTX 4080 SUPER / CUDA 13.2; it
  is NOT a system-level bandwidth claim, and bench BW is the noise
  filter while ncu is the court witness.

## Element 14 — S-REAL 20-dataset replay audit

- **Name:** The 20-dataset S-REAL audit gauntlet — 5 source-class
  families (DebuggingSoftwareTelemetry / ObservabilityTraces /
  TimeSeriesAnomaly / ReliabilityIndustrial / SoftwareDefects), 316
  admitted episodes, byte-identical within-run replay across two
  dispatches per dataset, CI-guarded by the 60-row hash chain.
- **Paper section:** §8 *S-REAL Audit Gauntlet (20-Dataset Sealed
  Bundle)* (`sec:s-real-audit-gauntlet`); Fig `fig:sperf-sreal-ladder`.
- **Source files:** `crates/dsfb-gpu-debug-demo/src/cli/s_real_audit.rs`,
  `crates/dsfb-gpu-debug-demo/src/cli/ingest.rs`,
  `crates/dsfb-gpu-debug-demo/src/cli/audit_report.rs`.
- **Tests:** `crates/dsfb-gpu-debug-demo/tests/s_real_3_bundle_integrity.rs`
  (5-test integrity suite, plan-locked),
  `crates/dsfb-gpu-debug-demo/tests/s_real_1_replay_byte_identity.rs`,
  the plan-required `s_real_audit` invariants in
  `s_real_audit.rs::tests` (post-S-REAL.3.1.2 split).
- **Receipts:** `reports/s_real_3/bundle_manifest.toml`,
  `reports/s_real_3/bundle_hash_chain.txt`,
  `reports/s_real_<tier>/<id>/{run_receipt,casefile,episodes,...}` for
  each of the 20 datasets.
- **Commit:** S-REAL.3 at `a8aaa04`; S-REAL.3.1.2 hygiene close-out
  at `6843a40`.
- **Non-claim:** S-REAL.3 admits replayable structural episodes; it
  does NOT claim ground-truth incident identification, RUL
  prediction, fault diagnosis, or any domain-truth result.

## Element 15 — 30-fixture saturation-regime classifier

- **Name:** The S-REAL.3.1 saturation sweep over all 30 fixtures
  (20 audit + 10 saturation-class), classifying each into
  saturation-class / transition / launch-bound regimes under
  plan-locked thresholds derived from the S-PERF.16.a anchor.
- **Paper section:** §9 *S-REAL Saturation Sweep (30-Fixture
  Classification)* (`sec:s-real-saturation-sweep`); Fig
  `fig:saturation-scatter`.
- **Source files:** `scripts/s_real_saturation_sweep.sh`,
  `crates/dsfb-gpu-debug-demo/tests/s_real_saturation_bench.rs`.
- **Tests:** `s_real_saturation_bench` plus the plan-required
  partition tests in `s_real_audit.rs::tests` post-S-REAL.3.1.2
  (audit-vs-saturation surface split).
- **Receipts:** `reports/s_real_saturation_sweep.txt`, pinned by
  SHA-256 in `ARTIFACT_MANIFEST.v1.toml`.
- **Commit:** S-REAL.3.1 at `fde8a99`; surface split sealed at
  S-REAL.3.1.2 (`6843a40`).
- **Non-claim:** wide GB/s is logical `DetectorCellWide`-arena
  throughput (264 B per cell), NOT physical HBM bandwidth; regime
  behaviour is governed by cell-count and dispatcher workload
  shape, NOT domain label.

## Element 16 — Colab public replay gate (COLAB.S-REAL.1)

- **Name:** The public Colab replay surface: a notebook that
  rebuilds the CUDA path from source on a free Colab GPU runtime,
  re-runs `s-real-audit --dataset all` on the 20 vendored audit
  datasets, and produces a downloadable ZIP bundle with the
  plan-locked A–F verdict classification (Build / Dataset SHA /
  Audit run / Per-run replay / Bundle integrity / Cross-hardware
  byte-identity).
- **Paper section:** §A.16 *Atlas Continuation* (COLAB.S-REAL.1
  paragraph); paper §Future Work paragraph naming the public
  replay surface.
- **Source files:** `notebooks/dsfb_gpu_debug_colab.ipynb`,
  `notebooks/README.md`,
  `scripts/pack_for_colab.sh`,
  `scripts/package_s_real_colab_outputs.sh`.
- **Tests:** the operator-facing replay protocol is the test (an
  operator runs the notebook and verifies the emitted bundle's
  hash chain). The notebook's pre-commit PENDING-guard (sealed in
  S-REAL.3.1.2) enforces that no operator can download a ZIP with
  unpopulated A–F gates.
- **Receipts:** `notebooks/dsfb_gpu_debug_colab.ipynb` (SHA-256
  pinned in `ARTIFACT_MANIFEST.v1.toml`), `notebooks/README.md`.
- **Commit:** COLAB.S-REAL.1 at `3548366`; PENDING-guard at
  `6843a40`.
- **Non-claim:** COLAB.S-REAL.1 is a reproducibility tool, NOT a
  benchmarking platform; Colab thermal variance makes any per-run
  GB/s value a courtesy snapshot. Cross-hardware byte-identity is
  reported honestly (PASS or divergence with the exact differing
  artifact path).

## Element 17 — DPU architectural implication boundary + DPU Conceptual Architecture Specification (Appendix F)

- **Name:** Plan-locked Tier 3 architectural read of four
  hardware shapes the measured S-PERF / S-REAL wall motivates
  (window-fact units, motif-mask lanes, fixed-order reducers,
  digest / receipt lanes), extended at DPU.1 into the full
  conceptual architecture specification in Appendix F. NOT a
  hardware claim.
- **Paper section:** §12 *Hardware Implications: Densorial / DPU
  Accelerator Architectural Read* (`sec:hardware-implications`)
  declares the Tier 3 architectural-read band and the four
  hardware shapes. Fig `fig:dpu-implication-block` (Fig 5)
  sketches the block topology. Appendix F
  (`sec:appendix-f-dpu-conceptual-architecture`) elaborates the
  full DPU conceptual architecture specification: per-primitive
  I/O contracts, determinism contracts, architectural-read
  throughput targets, hash-chain topology (TreeSha256V1 +
  CompactDensorDigestV1 stage-adaptive selector), cross-
  architecture posture, related-work positioning, future-work
  enumeration (DPU.2–DPU.5), and the plan-locked non-claims
  band. Appendix F's plan-locked thesis names the contribution
  as architectural prior art at the conceptual level, not a
  silicon claim.
- **Source files:** doctrinal section; references the empirical
  evidence in elements 9–15 above.
- **Tests:** the hostile-reviewer wording-audit at PAPER.1d /
  PAPER.1e seal confirms the Tier 3 claim band is preserved
  everywhere the DPU shapes are named. DPU.1's Appendix F
  carries the Tier 3 framing through every primitive
  specification, throughput target, and related-work
  paragraph; the plan-locked non-claims band closes the
  appendix with the verbatim "no silicon, no benchmark, no
  patent, no certification" disclosure.
- **Receipts:** the archived prior-art PDF §12 (Tier 3 band +
  Fig 5) and Appendix F (DPU Conceptual Architecture
  Specification).
- **Hashes:** the archived prior-art PDF SHA-256 pinned in
  `ARTIFACT_MANIFEST.v1.toml`.
- **Commit:** PAPER.1c at `eed6fe2` (§12 sealed); Fig 5 sealed
  at PAPER.1d (`72f5b31`); DPU.1 Appendix F sealed at
  `4046668`.
- **Non-claim:** §12 and Appendix F are conceptual
  architectural implications only; no DPU implementation, no
  hardware speedup, no silicon claim, no benchmark, no
  head-to-head against any GPU / TPU / NPU / IPU / WSE / RDU
  vendor, and no patentability or regulatory-fitness claim is
  made. The S-PERF / S-REAL evidence motivates the shape; it
  does not prove a DPU exists. Within-DPU-architecture
  determinism is declared by construction (Q16.16 fixed-point,
  fixed reduction order, scalar SHA-256, no atomics, no warp
  shuffles); cross-DPU-architecture and cross-vendor
  byte-identity are explicitly deferred to DPU.2–DPU.5 future
  work and remain NOT-claimed at PUBLISH.1.

---

## Element 18 — Commercial-clean subset partition (PENDING)

The commercial-clean subset partition over the sealed 20-dataset
S-REAL bundle is plan-deferred to **S-REAL.4** (post-RELEASE.1
per the plan-locked sequence). Once S-REAL.4 seals, this
section will gain its full row mapping the partition receipt, the
`s_real_commercial_bundle_hash_v1`, the operator instructions, and
the plan-locked non-claim.

---

## Element 19 — Deterministic Witness Family Gap Audit (T.13.GAP)

- **Name:** Audit-not-claim corpus campaign that walks seven
  plan-locked major survey taxonomies (Classic outlier /
  Time-series anomaly / SPC / Streaming sketch / Graph topology /
  Robust statistics / Deterministic ML-adjacent) against the
  ratified DSFB-GPU-Atlas witness corpus (SEED 1..=54 plus
  T.12.a..T.12.p expansion entries 5001..=6699 ratified by
  T.12.consolidate at `corpus_hash_v2`) and classifies every
  surveyed method record into one of twelve disposition buckets
  under four own-namespace hashes.
- **Paper section:** Appendix G *Deterministic Witness Family Gap
  Audit (T.13.GAP)* (`sec:appendix-g-t13-gap`).
- **Source files:**
  `crates/dsfb-gpu-atlas-corpus/src/t13_gap_witness_family_audit.rs`
  (corpus module; ~1430 lines);
  `crates/dsfb-gpu-atlas-corpus/tests/t13_gap_invariants.rs`
  (acceptance suite; 29 external tests).
- **Tests:** 33 acceptance tests total (29 external + 4 inline
  in the module's `inline_tests` block), including the
  CAMPAIGN IDENTITY case-insensitive completeness-claim
  scanner (`t13_gap_rejects_completeness_claim`), the nine
  other plan-required load-bearing negatives, four
  structural defect rules, hash determinism and
  pairwise-distinctness pins, renderer byte-stability, and
  the seed admission check.
- **Receipts:** the archived prior-art PDF Appendix G; the
  corpus module's deterministic audit report consumed via
  four own-namespace hashes minted at T.13.GAP seal commit
  `7d7729f`.
- **Hashes:**
  - `survey_taxonomy_index_hash_v1` under
    `DSFB-GPU-ATLAS:T13-GAP-SURVEY-TAXONOMY-INDEX:v1\0`.
  - `deterministic_gap_candidate_index_hash_v1` under
    `DSFB-GPU-ATLAS:T13-GAP-DETERMINISTIC-CANDIDATE-INDEX:v1\0`.
  - `gap_disposition_report_hash_v1` under
    `DSFB-GPU-ATLAS:T13-GAP-DISPOSITION-REPORT:v1\0`.
  - `taxonomy_gap_audit_hash_v1` (top-level META) under
    `DSFB-GPU-ATLAS:T13-GAP-TAXONOMY-AUDIT-REPORT:v1\0`. The
    top-level hash binds `corpus_hash_v1` + `SEED.len()` + the
    three component hashes plus the verbatim plan-locked
    thesis string, so any drift surfaces as a top-level hash
    change. All four hashes are deterministic across two
    consecutive builds and pairwise distinct from every prior
    court anchor.
- **Commit:** T.13.GAP corpus module sealed LOCALLY at
  `7d7729f`; T.13.GAP.PAPER paper-side appendix sealed LOCALLY
  at `f7e6d9e`.
- **Non-claim:** the audit RECORDS deterministic-witness-family
  classifications; it does NOT claim the Atlas covers every
  known anomaly method; it does NOT promote any of the four
  recorded deterministic gap candidates (Matrix Profile / SAX /
  persistent-homology summary / Minimum Covariance Determinant)
  to canonical detector entries — downstream T.x amendment
  proposal under T.12.0 intake court adjudicates promotion;
  the seven survey panels are bounded to plan-named taxonomies
  plus their explicit citation lists; the audit does NOT
  mutate `corpus_hash_v1` / `corpus_hash_v2` or any prior court
  hash anchor.

---

## Cross-references

- [`CLAIM_BOUNDARY_MATRIX.md`](CLAIM_BOUNDARY_MATRIX.md) — for each
  element, what is disclosed vs what is NOT claimed.
- [`TIMESTAMP_RECEIPT.md`](TIMESTAMP_RECEIPT.md) — public-
  accessibility receipt; SHA-256 + commit hash + Zenodo DOI
  `10.5281/zenodo.20338027` (v1.0, sealed 2026-05-22) + SWHID slot.
- [`ARTIFACT_MANIFEST.v1.toml`](ARTIFACT_MANIFEST.v1.toml) —
  machine-readable artifact index with SHA-256 pins.
- [`CITATION.cff`](CITATION.cff) — citation metadata (DOI
  `10.5281/zenodo.20338027`).
- [`codemeta.json`](codemeta.json) — CodeMeta JSON-LD software
  metadata (DOI-identified).
- [`.zenodo.json`](.zenodo.json) — Zenodo deposit metadata for the
  v1.0 deposit at DOI `10.5281/zenodo.20338027`.
- [`sbom.spdx.json`](sbom.spdx.json) — SPDX SBOM.
- [`LICENSE`](LICENSE) — Apache-2.0.
- [`NOTICE`](NOTICE) — Background-IP notice.
