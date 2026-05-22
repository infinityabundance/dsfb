# DSFB-GPU-Debug — Claim Boundary Matrix

For each disclosed architecture element, this matrix records six
columns: **Claim**, **What is disclosed**, **Where enabled (code)**,
**Where tested**, **What is NOT claimed**, and **Artifact hash /
commit**. The "Not claimed" column is the **stricter** version of
the per-section paper non-claims; it is the operative boundary an
external reader should rely on.

The matrix is structurally paired with
[`PRIOR_ART_MAP.md`](PRIOR_ART_MAP.md) (same 17 element numbering)
and [`TIMESTAMP_RECEIPT.md`](TIMESTAMP_RECEIPT.md).

**Disclaimer.** This is not legal advice. The matrix records what
is disclosed in this repository; it does not assert patentability
or any prior-art ruling. Counsel evaluates those properties against
the public-accessibility record.

---

### Element 1 — Endoductive evidence court

- **Claim:** DSFB-GPU realises a deterministic inference mode named
  *endoduction* (internal evidence-field relations adjudicated under
  a declared evidence contract into a replayable structural verdict),
  distinct from induction / deduction / abduction.
- **Disclosed:** the inference-mode definition + the doctrine that
  endoduction is admitted in a hash-anchored densor-projection /
  witness-family / fusion / bank-admission chain.
- **Enabled:** archived prior-art manuscript §2.1
  (`sec:endoduction`); pipeline implementation across elements 3–8.
- **Tested:** hostile-reviewer scope guard at PAPER.1e seal
  (exactly 7 endoduction mentions: 1 abstract, 5 §2.1, 1 conclusion).
- **Not claimed:** endoduction is NOT claimed as a philosophical
  novelty; the four named modes are NOT claimed as exhaustive;
  endoduction does NOT claim causal explanation; endoduction admits
  replayable internal structure under a declared evidence contract,
  not external truth.
- **Artifact hash / commit:** PAPER.1e at `117c237`;
  archived prior-art PDF SHA-256 pinned in
  `ARTIFACT_MANIFEST.v1.toml`.

### Element 2 — Densor / tekmeric evidence model

- **Claim:** Densors are typed, hashable evidence objects; tekmeric
  inference is the deterministic adjudication of declared witnesses
  through provenance, contraindications, challenge records,
  activation decisions, coverage holes, and hash-linked court
  artifacts.
- **Disclosed:** the vocabulary + the contract that a densor is
  produced by deterministic projection / detector / fusion rules
  under a declared numeric, indexing, reduction, and tie-break law.
- **Enabled:** `crates/dsfb-gpu-debug-core/src/{event,residual,sign,
  detector,consensus,candidate}.rs`.
- **Tested:** `crates/dsfb-gpu-debug-core/tests/property.rs`,
  `crates/dsfb-gpu-debug-core/tests/breach.rs`.
- **Not claimed:** densors are NOT learned latent tensors; densorial
  inference does NOT replace neural inference; the v0 densor
  vocabulary is bounded to fixed-point Q16.16 evidence on a
  deterministic GPU path.
- **Artifact hash / commit:** doctrine landed across the corpus +
  paper arc; PAPER.1e at `117c237` reaffirms the vocabulary.

### Element 3 — CUDA evidence factory / CPU court split

- **Claim:** A GPU-accelerated deterministic evidence factory feeds a
  CPU-side semantic-authority court via a single cross-boundary path
  (`BankAdmissionToken`).
- **Disclosed:** the architectural split (Fig 1
  `fig:evidence-court-pipeline`), the cross-boundary token contract,
  and the five GPU evidence stages.
- **Enabled:** `crates/dsfb-gpu-debug-cuda/src/{dispatch,ffi}.rs`,
  `cuda/kernels.cu`, `crates/dsfb-gpu-debug-core/src/{bank,episode}.rs`.
- **Tested:** `crates/dsfb-gpu-debug-demo/tests/cross_stage_chain.rs`,
  `crates/dsfb-gpu-debug-demo/tests/cli_smoke.rs`.
- **Not claimed:** the GPU does NOT emit semantic episodes directly;
  the GPU does NOT replace the bank; the GPU does NOT prove root
  cause.
- **Artifact hash / commit:** front-door identity at `6dab121`;
  Fig 1 sealed at PAPER.1d (`72f5b31`).

### Element 4 — Semantic Non-Bypass Axiom

- **Claim:** No GPU-side path produces an admitted `Episode`; only
  the CPU bank's private `BankAdmissionToken` constructor can mint
  an admitted episode, and the case-file emitter rejects any
  episode lacking the token (`SemanticBypassRejected`).
- **Disclosed:** the axiom + the type-level enforcement.
- **Enabled:** `crates/dsfb-gpu-debug-core/src/episode.rs`,
  `crates/dsfb-gpu-debug-core/src/bank.rs`,
  `crates/dsfb-gpu-debug-core/src/casefile.rs`.
- **Tested:** `crates/dsfb-gpu-debug-core/tests/breach.rs::semantic_bypass_rejected`.
- **Not claimed:** the axiom is module-scoped Rust visibility, NOT
  hardware-enforced memory isolation; NOT a formal verification
  proof.
- **Artifact hash / commit:** v0 architecture seal.

### Element 5 — BankAdmissionToken private-constructor enforcement

- **Claim:** Module-private constructor on `BankAdmissionToken`
  prevents any caller outside `bank.rs` from constructing an
  admitted `Episode`.
- **Disclosed:** the type definition + visibility + the consumer
  contract.
- **Enabled:** `crates/dsfb-gpu-debug-core/src/bank.rs` (constructor),
  `crates/dsfb-gpu-debug-core/src/episode.rs` (consumer).
- **Tested:** `crates/dsfb-gpu-debug-core/tests/breach.rs`.
- **Not claimed:** the access control is Rust module visibility,
  NOT a cryptographic or hardware-enforced capability.
- **Artifact hash / commit:** v0; unchanged across the arc.

### Element 6 — Q16.16 fixed-point deterministic numeric contract

- **Claim:** Same-device byte-exact CPU↔GPU equivalence under
  Q16.16 fixed-point arithmetic, with no FMA, no fast-math, no
  atomics for accumulation, no warp shuffles for reduction.
- **Disclosed:** the numeric contract bytes + the locked nvcc flag
  set + the banker's-round-to-even rule.
- **Enabled:** `crates/dsfb-gpu-debug-core/src/fixed.rs`,
  `cuda/common.cuh`, `cuda/kernels.cu`, `contract.toml`.
- **Tested:** `crates/dsfb-gpu-debug-core/tests/property.rs`
  (Theorem-9 analog over 64 LCG seeds).
- **Not claimed:** byte-exact CPU↔GPU equivalence is asserted ONLY
  for the recorded toolchain (same device, same binary, same driver,
  same CUDA version); cross-driver / cross-CUDA-version / cross-
  hardware byte-identity is NOT claimed.
- **Artifact hash / commit:** v0 prior-art proof.

### Element 7 — Locked CUDA kernel sequence

- **Claim:** The five evidence kernels run in a fixed sequence
  (residual_field → drift_slew_sign → detector_motif →
  consensus_grid → candidate_collapse) pinned by the execution
  contract. Reordering yields `KernelSequenceMismatch`.
- **Disclosed:** the sequence + the contract field that pins it.
- **Enabled:** `cuda/kernels.cu`,
  `crates/dsfb-gpu-debug-cuda/src/dispatch.rs`, `contract.toml`
  (`[kernels].sequence`).
- **Tested:** `crates/dsfb-gpu-debug-core/tests/breach.rs::kernel_sequence_mismatch`.
- **Not claimed:** Throughput-mode fused variants (post-S-PERF.x)
  preserve the same per-stage hashes via plan-locked byte-identity
  gates but are NOT claimed to be a different logical sequence;
  fused kernels are an optimization variant, not a contract change.
- **Artifact hash / commit:** v0; fused variants sealed across the
  S-PERF arc.

### Element 8 — Stage hash chain / verdict case file

- **Claim:** Twelve-link Merkle-style hash chain from input through
  every intermediate stage to the final verdict case file binds
  every artifact byte to the input catalog. Same input + same
  court state ⇒ same verdict.
- **Disclosed:** the chain layout (Fig `fig:hashchain`), the
  canonical-byte serialization, and the SHA-256 link rule.
- **Enabled:** `crates/dsfb-gpu-debug-core/src/{hash,casefile,serialize}.rs`.
- **Tested:** `crates/dsfb-gpu-debug-core/tests/replay.rs`,
  `crates/dsfb-gpu-debug-demo/tests/cross_stage_chain.rs`,
  `crates/dsfb-gpu-debug-demo/tests/s_real_3_bundle_integrity.rs`.
- **Not claimed:** the chain pins the artifact bytes that produced
  a given verdict; it does NOT claim that upstream input bytes are
  ground-truth labels.
- **Artifact hash / commit:** S-REAL.3 seal at `a8aaa04`;
  60-row chain at `reports/s_real_3/bundle_hash_chain.txt`
  (SHA-256 pinned in `ARTIFACT_MANIFEST.v1.toml`).

### Element 9 — Device Traffic Receipt / measurement law (S-PERF.1)

- **Claim:** Byte-accounting receipt + 8-line bandwidth-claim policy
  + 8 plan-required negatives; every CUDA bandwidth claim must
  cite the receipt to be admissible.
- **Disclosed:** the receipt schema, the 8-line policy, and the
  saturation threshold (80 % of declared theoretical peak).
- **Enabled:** `crates/dsfb-gpu-atlas-corpus/src/s_perf_1_device_traffic_receipt.rs`.
- **Tested:** `crates/dsfb-gpu-atlas-corpus/tests/s_perf_1_device_traffic_receipt_invariants.rs`.
- **Not claimed:** the receipt does NOT assert any measured
  bandwidth at baseline; it asserts the LAW that future claims
  must obey. NOT a saturation claim by itself.
- **Artifact hash / commit:** S-PERF.1 at `9575ce3`.

### Element 10 — Layer-A resident densor pipeline (S-PERF.2)

- **Claim:** A plan-locked five-stage device-resident evidence
  pipeline (EvidenceDensorProjection / WitnessDensorEvaluation /
  FusionDensorReduction / CandidateDensorCollapse /
  StageDigestEmission) with five forbidden-host-activity flags.
- **Disclosed:** the pipeline shape + the device-residency invariants.
- **Enabled:** `crates/dsfb-gpu-atlas-corpus/src/s_perf_2_layer_a_resident_pipeline.rs`.
- **Tested:** `crates/dsfb-gpu-atlas-corpus/tests/s_perf_2_layer_a_resident_pipeline_invariants.rs`.
- **Not claimed:** S-PERF.2 declares the pipeline shape; it does
  NOT benchmark throughput on it.
- **Artifact hash / commit:** S-PERF.2 at `1c78ca4`.

### Element 11 — Family compaction / detector-count-not-kernel-count

- **Claim:** 152 active detectors are compacted into 14 GPU-family
  lanes (S-PERF.4); detector count is NOT kernel count.
- **Disclosed:** the compaction schema + the plan-locked rule that
  active witnesses must be family-compacted before performance
  claims are made.
- **Enabled:** `crates/dsfb-gpu-atlas-corpus/src/s_perf_4_active_family_compaction.rs`.
- **Tested:** `crates/dsfb-gpu-atlas-corpus/tests/s_perf_4_active_family_compaction_invariants.rs`.
- **Not claimed:** active-detector compaction is a benchmark-shape
  rule, NOT a claim that any specific detector family is sufficient
  for any domain task.
- **Artifact hash / commit:** S-PERF.4 at `0a1ab3b`.

### Element 12 — Digest preservation contract (S-PERF.10)

- **Claim:** Plan-locked digest-preservation laws that any future
  digest-compaction kernel rewrite must satisfy (same-mode digest
  root law, canonical fragment-merge order, digest-mode non-aliasing,
  case-file chain preservation).
- **Disclosed:** the four laws + the plan-locked declared modes.
- **Enabled:** `crates/dsfb-gpu-atlas-corpus/src/s_perf_10_digest_lane_plan.rs`.
- **Tested:** `crates/dsfb-gpu-atlas-corpus/tests/s_perf_10_digest_lane_plan_invariants.rs`.
- **Not claimed:** the contract preserves digest roots WITHIN a
  declared digest mode; cross-mode root equality is NOT required.
- **Artifact hash / commit:** S-PERF.10 at `14bdc18`.

### Element 13 — A6.1 structural fusion optimisation

- **Claim:** Structural window-walk fusion cut
  `detector_motif_fused_d64_kernel` 1,324,608 → 910,283 ns
  (−31.3 %, 1.46× faster) and L1 LOAD 2,339,556,672 →
  1,139,949,568 bit-exact across 5 captures (−51.3 %, 2.05× cut)
  on RTX 4080 SUPER / CUDA 13.2.
- **Disclosed:** the byte-counter delta + the structural-fusion
  rewrite + the A5 v1/v2/v2.1 triple-null context.
- **Enabled:** `cuda/kernels.cu` (post-A6.1
  `detector_motif_fused_d64_kernel`).
- **Tested:** byte-identity pins in the post-A6.1 CUDA test suite.
- **Not claimed:** A6.1 is a LOCAL kernel win bound by Tier 1
  sealed-artifact byte-counter evidence on the recorded hardware;
  it is NOT a system-level bandwidth claim, and bench BW is the
  noise filter while ncu is the court witness.
- **Artifact hash / commit:** A6.1 at `3e84e05`;
  `reports/s_perf_16_a6_1_post.txt` SHA-256 pinned in
  `ARTIFACT_MANIFEST.v1.toml`.

### Element 14 — S-REAL 20-dataset replay audit

- **Claim:** The 20-dataset S-REAL audit gauntlet — 5 source-class
  families, 316 admitted episodes, byte-identical within-run replay
  across two dispatches per dataset, CI-guarded by the 60-row hash
  chain.
- **Disclosed:** the 20 datasets, the 9-artifact per-dataset receipt
  shape, the family classification, and the replay protocol.
- **Enabled:** `crates/dsfb-gpu-debug-demo/src/cli/s_real_audit.rs`,
  `crates/dsfb-gpu-debug-demo/src/cli/ingest.rs`,
  `crates/dsfb-gpu-debug-demo/src/cli/audit_report.rs`.
- **Tested:** `crates/dsfb-gpu-debug-demo/tests/s_real_3_bundle_integrity.rs`
  (5-test suite),
  `crates/dsfb-gpu-debug-demo/tests/s_real_1_replay_byte_identity.rs`,
  the plan-required `s_real_audit` invariants in
  `s_real_audit.rs::tests`.
- **Not claimed:** S-REAL.3 admits replayable structural episodes;
  it does NOT claim ground-truth incident identification, RUL
  prediction, fault diagnosis, or any domain-truth result.
- **Artifact hash / commit:** S-REAL.3 at `a8aaa04`;
  S-REAL.3.1.2 hygiene close-out at `6843a40`;
  bundle artifacts pinned in `ARTIFACT_MANIFEST.v1.toml`.

### Element 15 — 30-fixture saturation-regime classifier

- **Claim:** The 30-fixture saturation sweep classifies each fixture
  into saturation-class / transition / launch-bound regimes under
  plan-locked thresholds derived from the S-PERF.16.a anchor.
- **Disclosed:** the regime thresholds (50 % / 5 % of the
  S-PERF.16.a 22.74 GB/s anchor), the 30-fixture sweep, and the
  plan-locked wide-GB/s accounting non-claim.
- **Enabled:** `scripts/s_real_saturation_sweep.sh`,
  `crates/dsfb-gpu-debug-demo/tests/s_real_saturation_bench.rs`.
- **Tested:** `s_real_saturation_bench`; surface-split tests
  (`audit_all_excludes_large_saturation_fixtures`,
  `audit_dataset_table_no_duplicate_ids`).
- **Not claimed:** wide GB/s is LOGICAL `DetectorCellWide`-arena
  throughput (264 B per cell), NOT physical HBM bandwidth; regime
  behaviour is governed by cell-count and dispatcher workload shape,
  NOT domain label; the 10 saturation-class fixtures are NOT claimed
  to be benchmarks for their upstream domains; the 20 launch-bound
  fixtures are NOT claimed to be unsuitable for the audit.
- **Artifact hash / commit:** S-REAL.3.1 at `fde8a99`;
  surface split at S-REAL.3.1.2 (`6843a40`);
  `reports/s_real_saturation_sweep.txt` SHA-256 pinned in
  `ARTIFACT_MANIFEST.v1.toml`.

### Element 16 — Colab public replay gate (COLAB.S-REAL.1)

- **Claim:** A public Colab notebook rebuilds the CUDA path from
  source on a free Colab GPU runtime, re-runs the 20-dataset audit,
  and produces a downloadable ZIP bundle with a plan-locked A–F
  verdict (Build / Dataset SHA / Audit run / Per-run replay /
  Bundle integrity / Cross-hardware byte-identity).
- **Disclosed:** the notebook, the A–F verdict semantics, and the
  PENDING-guard cell that prevents downloading a ZIP with
  unpopulated A–F gates.
- **Enabled:** `notebooks/dsfb_gpu_debug_colab.ipynb`,
  `notebooks/README.md`, `scripts/pack_for_colab.sh`,
  `scripts/package_s_real_colab_outputs.sh`.
- **Tested:** operator-side; the PENDING-guard cell is the
  load-bearing pre-download check.
- **Not claimed:** COLAB.S-REAL.1 is a REPRODUCIBILITY tool, NOT a
  benchmarking platform; Colab thermal variance makes per-run GB/s
  values courtesy snapshots; cross-hardware byte-identity is
  reported honestly (PASS or divergence with the exact differing
  artifact path), NOT claimed in advance.
- **Artifact hash / commit:** COLAB.S-REAL.1 at `3548366`;
  PENDING-guard at `6843a40`; notebook SHA-256 pinned in
  `ARTIFACT_MANIFEST.v1.toml`.

### Element 17 — DPU architectural implication boundary + DPU Conceptual Architecture Specification (Appendix F)

- **Claim:** The measured S-PERF / S-REAL wall motivates four
  hardware shapes (window-fact units, motif-mask lanes, fixed-order
  reducers, digest / receipt lanes) under the plan-locked Tier 3
  *architectural read* band (§12); DPU.1 (Appendix F) elaborates
  the four shapes into a full conceptual architecture
  specification — per-primitive I/O contracts, determinism
  contracts, architectural-read throughput targets, hash-chain
  topology (TreeSha256V1 + CompactDensorDigestV1 stage-adaptive
  selector), cross-architecture posture, related-work
  positioning, and DPU.2–DPU.5 future-work enumeration. Tier 3
  band carried verbatim through every primitive specification.
- **Disclosed:** §12 four shapes + their empirical evidence anchors
  in elements 11–15; Appendix F full per-primitive specification
  (WFU / MML / FOR / DRL), consolidated determinism contract, and
  architectural prior-art posture at the silicon-concept level.
- **Enabled:** doctrinal paper section + Fig 5
  `fig:dpu-implication-block` (§12); Appendix F
  (`sec:appendix-f-dpu-conceptual-architecture`) full conceptual
  architecture specification.
- **Tested:** the hostile-reviewer audit at PAPER.1e seal preserved
  the Tier 3 claim band everywhere DPU shapes are named.
  Appendix F's plan-locked non-claims band (F.11) closes the
  appendix with the verbatim "no silicon, no benchmark, no
  vendor head-to-head, no patentability, no regulatory
  fitness" disclosure; every primitive specification carries
  the Tier 3 framing in the architectural-read throughput
  target.
- **Not claimed:** §12 and Appendix F are conceptual
  architectural implications ONLY; no DPU implementation, no
  hardware speedup, no silicon claim, no fabrication, no
  benchmark, no head-to-head performance claim against any
  GPU / TPU / NPU / IPU / WSE / RDU vendor, no patentability,
  no deployment readiness, no regulatory-fitness certification,
  no cross-vendor or cross-architecture byte-identical
  determinism claim. Within-DPU-architecture determinism is
  declared by construction; cross-architecture byte-identity
  is explicit DPU.2–DPU.5 future work and remains NOT-claimed.
  The Semantic Non-Bypass Axiom is the architectural invariant
  preserved at silicon level: the DPU produces evidence; the
  CPU bank constructs the admission token.
- **Artifact hash / commit:** PAPER.1c at `eed6fe2` (§12 +
  Fig 5 sealed at PAPER.1d `72f5b31`); DPU.1 Appendix F sealed
  at `4046668`.

---

### Element 18 — Commercial-clean subset partition (PENDING)

The commercial-clean subset partition lands in a follow-on
bundle-partition campaign that ratifies the partition under a
separate own-namespace hash above the sealed S-REAL.3 bundle.
This row will be populated when that campaign seals. The v1.0
prior-art deposit at Zenodo DOI
[`10.5281/zenodo.20338027`](https://doi.org/10.5281/zenodo.20338027)
does not gate Element 18.

---

### Element 19 — Deterministic Witness Family Gap Audit (T.13.GAP)

- **Claim:** The DSFB-GPU-Atlas witness corpus already contains
  the major deterministic spine across seven plan-locked survey
  panels (Classic outlier / Time-series anomaly / SPC /
  Streaming sketch / Graph topology / Robust statistics /
  Deterministic ML-adjacent). Twelve disposition buckets
  classify every surveyed method into one of:
  ExistingCanonicalAuthorityResolution / ParameterizationOf /
  DomainTransferOf / CompositionOf / AliasOf /
  NewCanonicalCandidate / five rejection variants
  (NotDeterministic / LearnedBlackBox / ProbabilisticEstimator /
  RuntimeOnlyMetric / NotEvidenceBearing) /
  DeferredNeedsSourceContract. On the T.13.GAP v1 seed, the
  histogram is 23 ExistingCanonicalAuthorityResolution + 1
  CompositionOf + 4 NewCanonicalCandidate + 4 RejectedLearnedBlackBox
  + 1 each of RejectedNotDeterministic / RejectedProbabilisticEstimator /
  RejectedNotEvidenceBearing / DeferredNeedsSourceContract = 36
  method records total across the seven panels.
- **Disclosed:** seven survey panels × 36 method records;
  twelve disposition buckets; four own-namespace hashes minted
  deterministically at T.13.GAP seal `7d7729f`; ten plan-required
  load-bearing negatives plus four structural defect rules;
  four deterministic gap candidates RECORDED (Matrix Profile /
  STOMP discord, SAX symbolic residual, persistent-homology
  summary, Minimum Covariance Determinant); plan-locked GPU-
  family mapping closure over the 14 S-PERF.4 families; live
  `corpus_hash_v1` anchor in the top-level META hash.
- **Enabled:** Appendix G *Deterministic Witness Family Gap
  Audit (T.13.GAP)* (`sec:appendix-g-t13-gap`); corpus module
  at `crates/dsfb-gpu-atlas-corpus/src/t13_gap_witness_family_audit.rs`;
  acceptance suite at
  `crates/dsfb-gpu-atlas-corpus/tests/t13_gap_invariants.rs`.
- **Tested:** 33 acceptance tests (29 external + 4 inline);
  the CAMPAIGN IDENTITY case-insensitive completeness-claim
  scanner (`t13_gap_rejects_completeness_claim`) enforces the
  audit-not-claim discipline mechanically; the verifier admits
  the seed report clean; cross-anchor pairwise-distinctness
  pinned across the four T.13.GAP hashes plus every prior
  court anchor; renderer byte-stability pinned for text + JSON.
- **Not claimed:** the audit does NOT claim the Atlas covers
  every known anomaly method (only the seven plan-named
  taxonomies plus their explicit citation lists are surveyed);
  does NOT promote any of the four recorded gap candidates to
  canonical (downstream T.x amendment proposal adjudicates
  promotion under T.12.0 intake court); does NOT mutate
  `corpus_hash_v1` or `corpus_hash_v2`; does NOT mutate any
  prior court hash anchor; does NOT execute any GPU code;
  does NOT alter `SEED.len()` (stays 54); does NOT rebaseline
  R.12b episodes 13/89/1917; does NOT modify the corpus module
  sealed at `7d7729f` (the paper appendix CITES the module
  verbatim). The Element-19 prior-art surface rides on the v1.0
  Zenodo deposit at DOI
  [`10.5281/zenodo.20338027`](https://doi.org/10.5281/zenodo.20338027);
  Software Heritage SWHID is a separate later operator-side
  archival surface and does NOT gate Element 19.
- **Artifact hash / commit:** T.13.GAP corpus module sealed
  LOCALLY at `7d7729f` (4 own-namespace hashes minted +
  pinned at that commit); T.13.GAP.PAPER paper-side appendix
  + Element 19 rows sealed LOCALLY at `f7e6d9e`.

---

## Cross-references

- [`PRIOR_ART_MAP.md`](PRIOR_ART_MAP.md) — same element numbering;
  describes WHERE each element lives.
- [`TIMESTAMP_RECEIPT.md`](TIMESTAMP_RECEIPT.md) — public-
  accessibility receipt.
- [`ARTIFACT_MANIFEST.v1.toml`](ARTIFACT_MANIFEST.v1.toml) — SHA-256
  pins for paper PDF, sealed bundle artifacts, COLAB notebook.
