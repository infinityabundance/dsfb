# DSFB-Chemical-Engineering

**Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in
Chemical Engineering.**

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>


*Riaan de Beer — Invariant Forge LLC — ORCID [0009-0006-1155-027X](https://orcid.org/0009-0006-1155-027X)*

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![assurance: dsfb-gray](https://img.shields.io/badge/dsfb--gray-50%E2%80%9379%25%20per%20crate-orange)](audit/dsfb-gray/SUMMARY.md)
[![cargo-audit](https://img.shields.io/badge/cargo--audit-0%20advisories-brightgreen)](audit/cargo-audit/README.md)
[![unsafe](https://img.shields.io/badge/unsafe-forbid%20in%204%2F6%20crates-brightgreen)](audit/cargo-geiger/README.md)
[![Miri](https://img.shields.io/badge/Miri-no%20UB-brightgreen)](audit/miri/README.md)
[![Kani](https://img.shields.io/badge/Kani-6%2F6%20harnesses-brightgreen)](audit/kani/README.md)
[![cargo-fuzz](https://img.shields.io/badge/cargo--fuzz-225.2M%20execs%2C%200%20crashes-brightgreen)](audit/cargo-fuzz/README.md)
[![tests](https://img.shields.io/badge/tests-295%20passing-brightgreen)](reports/verification_report.md)

> **Audit suite.** dsfb-gray assurance (per-crate) + a real Rust security/supply-chain suite
> (cargo-audit / geiger / auditable / vet / crev / scan) + Miri (no UB) + Kani (6/6 bounded grammar-soundness
> proofs) + **cargo-fuzz** (225.2M executions, 0 crashes — empirical companion to Kani) + a static panic-surface
> analysis — all reported **honestly** in [`audit/`](audit/); open [`audit/index.html`](audit/index.html) for the
> dashboard. The extended tools were **installed and run in-sandbox**: **cargo-fuzz** (225.2M execs, 0 crashes),
> **Flux** (checks the `core` crate clean), **valgrind** (**CLEAN — 0 memory errors** on the static-musl pipeline with
> musl-allocator suppressions; the glibc route is blocked by this host's compiled-in AVX-512, proven since even
> `/usr/bin/true` SIGILLs under valgrind), **hax** (extracted the `no_std` core to a real **716-line F\* model**), and
> **Creusot** (built its `creusot-rustc` driver and **translated our `classify_axis` to Coma verification IR**; the SMT
> prove step needs creusot's hermetic forked why3), and **crux-mir** (built from source via GHC/crucible/mir-json and
> **proved the core invariants `Valid`, 4/4 goals**, on a second symbolic engine — corroborating Kani). **loom** is N/A
> by design (no shared-state concurrency). Every folder records the real result or the precise remaining step + a
> verified command — no verdict fabricated. These are review-readiness / evidence artifacts, **not** a compliance
> certification.

---

> **Status:** complete and verified. All six workspace crates build and test (edge execution + cuda court +
> the `atlas` and `corpus` authority crates + the `no_std` embedded `core`, which also runs on an emulated
> Cortex-M3, + the `dsfb-densor-runtime` execution substrate, which carries no chemical claims), plus two standalone
> crates excluded from the host workspace (the `py` pyo3 bindings and the `wasm` "what-if" Chemical Court simulator); the edge demo runs all 20 datasets deterministically
> (byte-exact replay); the CUDA forensic court produces evidence roots that are
> **byte-for-byte identical to a CPU reference on all 20 datasets**. This crate snapshot does **not** bundle
> the optional paper PDF or rendered figure gallery; the artifact index records that absence explicitly
> instead of inventing a PDF hash or figure manifest. The Colab notebook runs end-to-end. This README is the living source of truth and is
> kept in sync with the code. See [`reports/verification_report.md`](reports/verification_report.md) for the
> verbatim, re-runnable verification (capture host, commands, test counts, frozen hashes), and
> `PROJECT_PLAN.md` for design + per-phase detail.
>
> **Verified (local, NVIDIA RTX 4080 SUPER):** edge replay 20/20 · CUDA replay + cross-backend
> verify 20/20 · dataset SHA-256 provenance gate 20/20 · Tennessee Eastman IDV(1) detection delay
> = 1 sample (onset @ 160), IDV(4)/IDV(6) = 0, baseline false-positive 3–9% · memory roofline
> ~636 GB/s (~88% of peak). Numbers on non-stationary batch / heterogeneous / tabular datasets are
> reported honestly, including where the framework struggles.
>
> **Confidential evaluation (no data egress):** DSFB does not require plant data to leave the operator's
> control — the operator runs the read-only court locally and shares only a **redacted, hash-linked evidence
> bundle** (`ConfidentialEvaluationBundleV1` / `PartnerDataEscrowProtocolV1`); real tag names are hashed, raw
> values stripped, and a weak heuristic that fails its evidence burden emits `unknown`, never a fabricated
> label. See [`docs/real_data_dropin.md`](docs/real_data_dropin.md) + [`docs/sbir_operator_data_request.md`](docs/sbir_operator_data_request.md).
>
> The CUDA evidence kernel (one thread per lane, sequential over samples) is the **baseline deterministic
> sealing** contract — chosen for auditable determinism, **not** a final optimized-throughput claim. Any
> GPU optimization must pass a **digest-equivalence law**: identical lane digests / Merkle root / evidence
> root / replay, only timing may change — so a speedup can never mutate the court record. Two such
> optimizations are **built, gated, and measured** behind that law (see `docs/cuda_evidence_kernel_v2_design.md`):
> **V2-A lane-batching** (digest-identical; occupancy 8%→52%) and **V2-B segment-parallel** (an opt-in
> `evidence_root_v2` Merkle-segment format; deep case ~18× kernel, ~8× end-to-end with pinned transfer) —
> plus a documented *negative* result (a `cudaMemcpyAsync` overlap that measured slower and was reverted).
>
> **Navigation:** [`breadth_surface.toml`](breadth_surface.toml) is the top-level index of every major
> verifiable claim → its governing artifact → the one command that reproduces it → its evidence tier. It is
> kept honest by a self-check court (`tests/breadth_surface_court.rs`: tiers resolve, reproduction commands
> are real subcommands, artifacts exist, counts match `atlas::validate()` + the manifests) and hash-sealed.

> **Start here — one-click index.** [`reports/index.html`](reports/index.html) is a deterministic, self-contained
> map of the shipped artifact graph (the eight crates — seven chemical + the `dsfb-densor-runtime` substrate ·
> 20 evidence bundles with their roots · the explicit zero-figure
> manifest for this snapshot · the governance courts · the controlled-access policy · the SBIR/operator docs), regenerated by
> `dsfb-chem-edge generate-index` and sealed with a re-runnable `index_root`. Its companion court
> **`ArtifactIndexCourtV1`** (`dsfb-chem-edge verify-index`) re-derives that `index_root` and cross-checks every
> linked artifact — paper absence/PDF hash when present, the 20 bundle roots, figure manifest entries, doc paths, the crate list, the
> controlled-access policy flags, and the embedded court hashes (9 checks, 0 fail). Open it first.
>
> **By role:** SBIR / industrial evaluation → [`docs/sbir_phase_i_one_page_eval.md`](docs/sbir_phase_i_one_page_eval.md)
> (objective · what DSFB reads · what it *never* writes · non-claims · licensing boundary); embedded engineers →
> [`docs/embedded_memory_budget.md`](docs/embedded_memory_budget.md) (heap 0, `#![forbid(unsafe_code)]`,
> ≈168 B/channel at N=8); private plant evaluation → `cargo run -q -p dsfb-chemical-engineering-edge -- confidential-demo`
> (no data egress); the honest audit dashboard → [`audit/index.html`](audit/index.html).
>
> **Review ZIP vs public-release archive — they are different artifacts.** A *review* bundle handed to a
> reviewer may legitimately include internal working context (e.g. session notes) by design. A *public-release
> archive* must instead be produced with **`git archive`** (never a raw zip), which `export-ignore`s those files,
> and must pass **`release-scrub --archive-dir`** before upload. The verifiable commit→archive→deposit provenance
> chain is in [`docs/public_archive_proof.md`](docs/public_archive_proof.md); the gate is
> [`docs/release_checklist.md`](docs/release_checklist.md) §C.

## What this is

Modern chemical process monitoring already has strong tools: PCA, PLS, MSPC, DPCA, KPCA, ICA, CVA,
contribution plots, EWMA/CUSUM, and machine-learning anomaly methods. **DSFB-Chemical-Engineering
does not try to beat them.** It is a deterministic, **read-only residual-interpretation layer**:
the residuals, scores, distances, reconstruction errors, contribution traces, threshold crossings,
and detector disagreements those methods already emit — and usually discard — are treated as the
**primary evidence object**. DSFB then applies **Drift–Slew Fusion Bootstrap** (first-order residual
drift, second-order residual slew, admissibility envelopes, detector consensus, and a chemical
heuristics bank) to convert residual streams into auditable, human-readable **structural episodes**
and a **replayable evidence case file**.

The framing, stated plainly:

> Existing chemometrics detectors are the **witness bank**. DSFB produces the **court record** —
> a deterministic, replayable case file over the evidence they testify.

DSFB-Chemical-Engineering **is not** a new controller, plant model, estimator, or a replacement for
PCA/PLS/MSPC or ML fault detection. It writes to no setpoint, alarm limit, historian tag, or control
variable. Removing it restores the pre-deployment baseline exactly.

**Evidence-strength hierarchy.** Evidence is *typed*, not pooled: each `EvidenceKind` maps deterministically to a
physical-witness rung (where it is a graded residual witness) and a claim-strength tier — one source of truth in
`evidence_kind.rs` / `witness_strength.rs` / `claim_strength.rs`, test-pinned, and rendered in the paper
(Table: evidence-strength hierarchy). A rung is *evidence strength*, never a causal claim; the tier bounds how
strongly a derived statement may be read. A fourth tier, **NonClaim**, is reserved for root-cause / causality /
accuracy-superiority / compliance — statements DSFB never makes.

| `EvidenceKind` | Physical-witness rung | Claim-strength tier |
|---|---|---|
| `PhysicalBalance` | BalanceClosure (6, physical) | Tier 2 · EvidenceInterpretation |
| `ProcessTopology` | TopologyResidenceAligned (5, physical) | Tier 3 · SpeculativeImplication |
| `ControllerContext` | ControlActionConsistent (4) | Tier 2 · EvidenceInterpretation |
| `ChemometricDetector` | DetectorFamilyQuorum (3) | Tier 2 · EvidenceInterpretation |
| `HeuristicPattern` | HeuristicPatternOnly (2) | Tier 2 · EvidenceInterpretation |
| `PrecedentSimilarity` | PrecedentSimilarityOnly (1) | Tier 3 · SpeculativeImplication |
| `HistorianImport` | — (sealed file digest) | Tier 1 · SealedFact |
| `FirstPrinciplesEquation`, `InstrumentationHealth`, `DatasetQuality`, `OperatorAnnotation`, `NarrativeSummary` | — | Tier 2 · EvidenceInterpretation |

This is what Q94 means by *structure, not substrate*: a moving average smooths a scalar; DSFB types each residual
witness, ranks it, bounds the claim it can support, and seals the result.

### Deterministic inference, not "deterministic-upon-probability"

This is the most common point of confusion, so it is worth stating directly: **"deterministic
inference" here does not mean Bayesian, and it does not mean a deterministic computation of a
statistical estimate.** Data-driven *soft sensors* infer a hard-to-measure quantity from cheap sensors
and are dominated by probabilistic estimators (PLS, neural nets, SVR/GPR, Bayesian latent-variable
models). Even the methods usually *called* deterministic — OLS, PLS — are deterministic only in
**computation**: least squares is identically the Gaussian maximum-likelihood estimator (squared-error
loss is the Gaussian negative log-likelihood), optimal only **under** noise assumptions (Gauss–Markov).
That probabilistic foundation is what makes them noise-fragile — which is exactly why the field moved
to *more* explicitly probabilistic methods, not fewer.

DSFB is deterministic in a stronger sense: **no probability model, no likelihood, no expected-loss
objective, and no distributional assumption anywhere in the inference path** — a fixed finite-state
grammar over residual triples, whose only statistics are *distribution-free* calibration thresholds
(nearest-rank quantiles, median/MAD). Pure deterministic inference (*densorial / tekmeric*):

> residual **densor → deterministic witness court → replayable case file**
> *(vs neural/Bayesian: tensor → learned weights or posterior → probabilistic output)*

The differentiator is therefore **determinism, assumption-freedom, auditability, and reading
usually-discarded noise as structural signature** — *not* predictive accuracy. A probabilistic soft
sensor returns a value with a variance; DSFB returns a byte-exact, replayable evidence trail.

## Repository layout

```
dsfb-chemical-engineering/
  README.md                 ← this file (source of truth #2)
  PROJECT_PLAN.md           ← in-repo design + status (source of truth #1)
  Cargo.toml                ← workspace (edge + cuda + atlas + corpus + core + densor-runtime)
  crates/
    dsfb-chemical-engineering-edge/   ← CPU execution; residual grammar + detectors + fusion + witnesses
    dsfb-chemical-engineering-cuda/   ← CUDA-accelerated; Nsight-measured byte-exact forensic court
    dsfb-chemical-engineering-atlas/  ← authority: detector + H1–H6 heuristic + F1–F12 fault-signature records
    dsfb-chemical-engineering-corpus/ ← authority: public soft-sensor dataset catalogue (sourced; no bytes vendored)
    dsfb-chemical-engineering-core/   ← no_std, no-heap, fixed-point embedded core (+ QEMU Cortex-M smoke harness)
    dsfb-densor-runtime/               ← deterministic execution-substrate skeleton (no chemical/cross-domain claims)
    dsfb-chemical-engineering-py/      ← pyo3 Python bindings (standalone; built with maturin)
    dsfb-chemical-engineering-wasm/    ← WASM "what-if" Chemical Court simulator (standalone; browser tool)
  paper/figures/             ← explicit zero-figure manifest for this crate snapshot
  notebooks/                ← Colab reproducibility notebook (does not compile the paper)
  research/                 ← read-only research inputs (gitignored)
```

## The crates

DSFB-Chemical-Engineering separates **execution** from **authority** (+ an embedded core, a runtime substrate, and Python bindings):

| Crate | Role |
|---|---|
| `dsfb-chemical-engineering-edge` | **Execution (CPU).** Residual pipeline, drift/slew/envelope grammar, detector execution, deterministic quorum fusion, heuristics bank, reports/figures, mass/energy-balance witnesses, and the **Chemical Court Record v1** bundle. **Unsafe-forbidden (`#![forbid(unsafe_code)]`), dependency-light edge Rust; std-only as shipped (no GPU required).** The `no_std`/no-heap/fixed-point embedded profile (`docs/edge_core_profile.md`) is now **realised as the `dsfb-chemical-engineering-core` crate** (below). |
| `dsfb-chemical-engineering-cuda` | **Acceleration + forensic court (GPU).** Same detector/fusion semantics; evidence production with **fixed-point determinism + on-GPU SHA-256**, a hash-linked **forensic court**, byte-exact CPU/GPU replay verification, and **Nsight + GB/s** benches. |
| `dsfb-chemical-engineering-atlas` | **Authority: what evidence is *allowed to mean*.** Curated `&'static` records — the 18 executed chemometric detectors, the H1–H6 process heuristics, and the **F1–F12 process-fault signature bank** (cheap-sensor residual fingerprints) — with validation gates and a frozen `atlas_hash_v1`. `no_std`; depends on nothing. |
| `dsfb-chemical-engineering-corpus` | **Authority: soft-sensor data catalogue.** A provenance-bound, deduplicated catalogue of public soft-sensor datasets (cheap sensors → hard-to-measure target), every record sourced with URL + licence + access flag; **no dataset bytes vendored**. `corpus_hash_v1`. *(PubChem-scale molecular densors are a future companion corpus, **not** part of this artifact — this crate is a soft-sensor dataset authority catalogue only; the companion is designed in [`docs/molecular_corpus_companion.md`](docs/molecular_corpus_companion.md).)* |
| `dsfb-chemical-engineering-core` | **Embedded core (`no_std`, no-heap, fixed-point).** The residual triple + ring buffer + admissibility envelope + grammar state machine in scaled integers (no `std`, no allocation, `#![forbid(unsafe_code)]`, zero dependencies). **Builds for `thumbv7m-none-eabi` and runs on an emulated Cortex-M3** (the `qemu-smoke` harness). Not claimed bit-identical to the edge float pipeline — the same grammar, calibrated independently. |
| `dsfb-densor-runtime` | **Execution substrate (mechanism, not chemistry).** A thin, deterministic `load → validate authority → execute stages → seal → emit receipt` spine (traits `Densor` / `RuntimeStage` / `StageReceipt`, a per-stage *no-claim-without-an-authority-hash* gate, and a sealed `RuntimeReceiptV1`). `#![forbid(unsafe_code)]`, Miri-clean. It **carries no chemical and no cross-domain claims** — the chemical crates remain the sole domain authority; this is a reusable runtime skeleton only. |
| `dsfb-chemical-engineering-py` | **Python bindings (pyo3, standalone).** A thin wheel (built with maturin) exposing the file-free read-only courts (`version`, `classify_unit_pair`, `grade_readiness`) to Python. Excluded from the host workspace; publishing is USER-ONLY. |
| `dsfb-chemical-engineering-wasm` | **Browser "what-if" Chemical Court simulator (standalone).** Compiles the dependency-free `core` grammar to `wasm32-unknown-unknown` (raw `extern "C"` exports, no wasm-bindgen) behind a static HTML/JS shell: an operator drags the admissibility envelope (`k` / grazing band / drift window) and watches the *same* immutable residual stream re-classify. A HAZOP/training what-if tool — sandboxed, advisory, not a controller; excluded from the host workspace. |

> **Embedded memory budget (`…-core`).** Per-channel `DsfbCore<N>` = **8·N + 104 bytes** (ring buffer + previous
> sample + envelope + classifier), heap = **0**, `#![forbid(unsafe_code)]`, `panic = "abort"`. At the default
> window `N=8` that is **168 B/channel**; a 64-channel unit fits in **≈10.5 KiB (16% of a 64 KiB Cortex-M3 RAM)**.
> Exercised on QEMU `lm3s6965evb` via the `qemu-smoke` harness. Full breakdown: [`docs/embedded_memory_budget.md`](docs/embedded_memory_budget.md).
>
> | window `N` | bytes / channel | 64 channels |
> |---|---|---|
> | 8 | 168 B | ≈10.5 KiB |
> | 16 | 232 B | ≈14.5 KiB |
> | 32 | 360 B | ≈22.5 KiB |

> **Detector counts, stated exactly:** the edge detector corpus contains **57** chemometric detector
> records, of which **18 are executed** by the demonstration pipeline (the remaining 39 are catalogued
> prior art). The authority `atlas` crate **freezes the 18 executed detector records, the H1–H6 process
> heuristics, and the F1–F12 fault-signature bank** behind `atlas_hash_v1` — it does not drop the 39
> catalogued detectors; those remain in the edge corpus TOML. Of the **12 fault signatures, 7 are
> executed** (F6 leak + F7 sensor-drift balance/isolation witnesses; F1 stiction, F2 pump cavitation,
> F3 heat-transfer fouling, F8 controller-masking, F9 valve-hunting synthetic demonstrators, gated by
> edge `tests/fault_demonstrators.rs`); the other 5 stay catalogued.

Doctrine (inherited from the DSFB-GPU lineage): **execution computes residual episodes; the atlas and
corpus define what detectors, heuristics, fault signatures, and datasets are allowed to mean; the GPU
produces evidence and the court decides.**

## Quick start

```bash
# Edge crate — full demo over all datasets (CPU, no GPU required)
cargo run --release -p dsfb-chemical-engineering-edge -- demo

# Edge — other commands (all CPU):
#   analyze <dataset>        one dataset, metrics to stdout
#   casefile <dataset>       write the Chemical Court Record v1 evidence bundle (see below)
#   atlas                    detector-atlas + atlas authority summary (atlas_hash_v1)
#   corpus                   soft-sensor data-corpus authority (needs --features soft-sensor-corpus)
#   verify-replay            run the synthetic suite twice; confirm identical replay hashes
#   completeness-court       machine-checked artifact-graph completeness + consistency gate
#   release-scrub            public-release hygiene gate (placeholder DOI / private-file leak / controlled rows)
#   unit-consistency         unit/dimension court over every documented balance (°C↔K, bar↔Pa, …)
#   data-readiness <csv>     grade a real historian CSV before analysis (Ready / Caveats / NotReady)
#   figures                  optional local figure campaign + verbose log + deterministic ZIP (needs python3/matplotlib)
#   regime-eval              regime-conditioned admissibility-envelope evaluation
#   historian <csv>          replay a plant-historian CSV in batch-record mode
#   balance-witness <name>   mass/energy-balance witness on an instrumented dataset
#   control-action <name>    control-action context for a dataset
#   generate-index           deterministic reports/index.{html,json} map of the whole artifact graph
#   verify-index             ArtifactIndexCourtV1 — re-derives index_root, cross-checks every live artifact
#   narration-context <ds>   emit a Court Record's citable-anchor narration context (also auto-emitted by casefile)
#   confidential-demo        one-command redacted partner-evaluation bundle (no raw plant-data egress)
#   densor-runtime-demo      carry edge episodes through the runtime substrate (needs --features densor-runtime-demo)
cargo run --release -p dsfb-chemical-engineering-edge -- casefile tennessee_eastman_idv01

# CUDA crate — build kernels, run audit, emit forensic case files, verify byte-exact replay
PATH="/opt/cuda/bin:$PATH" bash crates/dsfb-chemical-engineering-cuda/scripts/build_cuda.sh
cargo run --release -p dsfb-chemical-engineering-cuda -- demo

# CUDA Nsight + GB/s benches (run multiple times)
bash crates/dsfb-chemical-engineering-cuda/scripts/run_bench.sh
bash crates/dsfb-chemical-engineering-cuda/scripts/run_nsight.sh

# No paper PDF is bundled in this crate snapshot; generated paper artifacts should be added only
# when their source, PDF, and verification logs are committed together.
```

Each demo writes a timestamped `output-dsfb-chemical-engineering/<stamp>/` containing
`manifest.json`, `detector_outputs.csv`, `residual_streams.csv`, `dsfb_episodes.csv`,
`heuristic_labels.csv`, `replay_hashes.json`, `figures/*.png`, `report.md`, and
`artifact_bundle.zip`.

## The Chemical Court Record v1

> **DSFB-Chemical-Engineering does not emit an alarm. It emits a court record of why an
> alarm-like structure was or was not admitted.**

`casefile <dataset>` (and every per-dataset slot of `demo`) writes a canonical, versioned,
hash-rooted evidence bundle — `dsfb_chemical_engineering_casefile_v1/` — containing **exactly**
these files:

| File | What it holds |
|---|---|
| `casefile.json` | Manifest: format id + version, `evidence_root`, per-episode **claim-boundary badges**, counts, per-file SHA-256, and a single `bundle_root`. |
| `admitted_episodes.csv` | The episodes quorum fusion **admitted**, each with its claim-boundary badge, motif, NE 107 status, and evidence grade. |
| `detector_witnesses.csv` | Per episode × detector: which witnesses **fired** vs stayed **silent** (the silent set is as informative as the firing set). |
| `rejected_candidates.csv` | Near-episodes fusion **examined and refused**, with a `rejection_reason` and the raw mechanism preserved. |
| `unknown_taxonomy.csv` | Each **UNKNOWN** episode placed in one of five deterministic classes, with a suggested operator action. |
| `residual_provenance.csv` | The residual-provenance ledger (which residuals fed which detector). |
| `ne107_status_trace.csv` | Per-sample NAMUR **NE 107** plant-status trace (presentation mapping; no compliance claim). |
| `alarm_rationalization.csv` | ISA-18.2 flood → fused-episode rationalisation (evidence, never alarm *suppression*). |
| `operator_report.html` | Static, operator-readable case file. |
| `evidence_root.txt` | One line: the byte-exact replay/evidence root hash. |
| `non_claims.md` | The bounded non-claims statement + this run's badge summary; `ROOT_CAUSE_NOT_ADMITTED` applies to the whole file. |

**Claim-boundary badges** make every output self-bounding: `STRUCTURE_ONLY`, `CANDIDATE_FAULT`,
`NEAR_MISS`, `SENSOR_QUALITY`, `CONTROL_CONTEXT_REQUIRED`, `PHYSICS_WITNESS_REQUIRED`
(per episode) plus bundle-level `ROOT_CAUSE_NOT_ADMITTED` and `REPLAY_VERIFIED`.

**Inspect a case file in 3 minutes:** open `casefile.json` → read `evidence_root` and `bundle_root`
(the two hashes that pin the case); scan `counts` (admitted / rejected / unknown / near-miss) and the
per-episode `badge`; open `admitted_episodes.csv` for the episodes and `rejected_candidates.csv` for
what was refused and why; read `non_claims.md` for the bounded claim. Re-run `casefile <dataset>` and
confirm `bundle_root` is byte-identical to verify replay.

**Narration context (auto-emitted).** Alongside each bundle, `casefile` also writes `narration_context.{md,json}`
(or run `narration-context <dataset>`): the complete vocabulary of **citable evidence anchors** — one per episode,
each with the **claim tier** it may be asserted at — plus the binding contract. A constrained external narrator may
re-present the record only by citing these anchors; an unanchored or over-tier sentence is rejected by
`NoNarrativeHallucinationGateV1`. It is a separate artifact (not one of the ten hashed files), so it never moves a
`bundle_root`. See [`docs/constrained_narration_extension.md`](docs/constrained_narration_extension.md).

**What an operator should do with an UNKNOWN** (`unknown_taxonomy.csv`):

| Class | Meaning | Action |
|---|---|---|
| `UNKNOWN_SHORT_TRANSIENT` | brief transient (≤3 steps) | monitor; no action unless it recurs |
| `UNKNOWN_OUT_OF_BANK_DOMAIN` | evidence outside the heuristic vocabulary | extend the bank / consult a domain expert |
| `UNKNOWN_DETECTOR_CONFLICT` | detectors disagree (high entropy) | investigate the disagreement; check control/regime context |
| `UNKNOWN_WEAK_QUORUM` | quorum met but thin support | watch |
| `UNKNOWN_STRUCTURAL_UNMAPPED` | sustained, consistent, unmatched structure | escalate for diagnosis (the genuine "new pattern" bucket) |

## Noise-floor preservation (evidence, not detection)

The CUDA evidence contract seals the **raw IEEE-754 bits** of every residual sample into the per-lane
SHA-256 digest, alongside the fixed-point `q,e,d,s` quantities. So the sub-threshold noise floor that
conventional monitoring discards — anything below the alarm limit — is retained in the sealed record and
re-emerges byte-for-byte on replay. The lane digest changes even when two physically different residuals
round to the same fixed-point integer.

This is a **preserved-evidence** property, **not** a detection capability, and the line is mechanical:

- The inference path **never reads** the raw bits to raise, grade, or admit an episode — the DSFB
  grammar consumes only the residual triple; the raw bits go into the digest and nowhere else.
- Two runs whose residuals differ only **below the quantisation grid** produce **identical** episodes
  and badges; only their digests differ. No sensitivity or sub-threshold-detection claim is made.
- The guarantee is one-directional: the noise floor is **recoverable from the record**, not that
  anything diagnostic was learned from it.

The value is forensic: a post-incident review can ask *"what did the instrument report, bit for bit, in
the minutes before the event?"* and a hash-linked record that sealed the raw bits can answer it — and
prove it was not altered after the fact. Full treatment: paper §"Noise-floor preservation", and
[docs/noise_floor_preservation.md](docs/noise_floor_preservation.md).

## Worked forensic incident (fictional, 3-minute read)

For a self-contained end-to-end example — a *fictional, fully-synthetic* plant incident (no real entity,
no agencies) run through one `historian` command into a Chemical Court Record — see
[docs/forensic_incident_walkthrough.md](docs/forensic_incident_walkthrough.md). A surge-tank
level-transmitter spoof is caught by the **balance witness** (closure 0 → 8; NE 107 flips to `Failure` at
onset) while the statistical detector bank's sub-quorum candidate is **recorded as a rejection** rather
than forced into an alarm — the whole doctrine on one screen. The paper formalises the surrounding
**milestone-gated evaluation protocol** (M0–M3 with replay-checkable go/no-go gates) in §"Operator
evaluation protocol".

## The chemometric detector atlas

A literature **detector corpus** (modeled on `dsfb-gpu-atlas-corpus`) instantiated for chemometrics
and organized into four families:

- **A. Classical MSPC** — PCA-T², PCA-SPE/Q, PLS score/prediction residual, SIMCA distance,
  contribution plots, Hotelling T².
- **B. Dynamic / Temporal** — DPCA, moving-window / recursive PCA, EWMA, CUSUM, MEWMA, Page-Hinkley,
  Mann-Kendall, Pettitt / SNHT / MOSUM / Buishand change-points, lagged-residual autocorrelation.
- **C. Nonlinear / Distributional** — KPCA / autoencoder reconstruction error, ICA residual,
  one-class SVM / kNN / LOF distance, distribution distances (KS, KL, JS, MMD, Wasserstein, energy,
  Hellinger, TV, PSI), spectral entropy, wavelet energy.
- **D. Process-structure** — variable-group co-drift, unit-operation block, mass/energy-balance
  residual, control-action mismatch, actuator-lag, sensor-stiction / valve-hunting, batch-phase
  residual.

Every detector emits a common schema (`detector_id, time_index, variable_scope, unit_scope,
raw_score, normalized_score, threshold, signed_margin, breach_state, drift, slew, confidence_hint,
provenance`). DSFB fuses them by deterministic quorum (not black-box voting) into typed episodes,
and a chemical **heuristics bank** maps residual motifs to operator-readable labels.

## Datasets

20 public chemical-engineering / chemometric datasets. Only small **processed slices** are vendored,
each with a provenance entry (source URL, license, citation, retrieval date, SHA-256) in
`crates/dsfb-chemical-engineering-edge/data/MANIFEST.toml`. Datasets requiring a data-use agreement
(e.g. iTrust SWaT/WADI) are **not** redistributed — only loaders and a clearly-labelled synthetic
stand-in are shipped; the notebook fetches the full data after the user accepts the upstream EULA.
Each dataset is labelled **[M] measured** or **[S] public simulation benchmark** for honesty. See
`PROJECT_PLAN.md` for the full table.

**Controlled-access dataset policy** (mirrors `ControlledAccessDatasetPolicy::STANDING` in
[`release_scrub.rs`](crates/dsfb-chemical-engineering-edge/src/release_scrub.rs); enforced by the `release-scrub`
court, P82/P87). For iTrust-gated and similarly controlled testbeds (SWaT / WADI / BATADAL): raw bytes **✗**,
processed/instrumented rows **✗**, attack lists **✗**, reconstructable windows **✗** — but aggregate metrics **✓**
and the reproducible scripts **✓**. The controlled rows live in the untracked `research/` quarantine; only
metadata-only `*_instrumented.roles.json` sidecars ship, each declaring four no-rows provenance flags
(`release_status` / `contains_controlled_access_rows` / `reconstructable_from_committed_bytes` /
`redistribution_policy`). Naming a dataset (filename or citation) is **required credit, not redistribution**.

### Dataset provenance classification (corpus authority, P53)

The `dsfb-chemical-engineering-corpus` authority crate classifies every one of its 20 records on four
**honest disclosure** axes, sealed into `corpus_hash_v1` and counted by `census()` (a gate asserts each
axis partitions all 20 records). These are confidence / policy statements derived from the cited licence
+ URL + venue — **not** legal opinions or quality judgements — so a downstream user can see, reproducibly,
how verifiable each dataset's terms are. The crate vendors **no dataset bytes** regardless of any tier.
Run `cargo run -p … --features soft-sensor-corpus -- corpus` to print the live census.

| Axis | Tier breakdown (of 20) |
|---|---|
| **Licence confidence** | explicit-open (CC0/CC BY) **8** · copyleft (GPL) **1** · stated-needs-verification **5** · research-use-customary **4** · agreement-governed **2** |
| **Access confidence** | open-confirmed (curated repo / DOI) **12** · open-mirror-unverified **4** · account-required (Kaggle) **1** · code-generates-data **1** · agreement-required **2** |
| **Redistribution policy** *(what a downstream user must respect — this crate ships nothing)* | upstream-permits-attribution **8** · copyleft-share-alike **1** · verify-before-redistribution **9** · prohibited-by-agreement **2** |
| **Source authority** *(provenance robustness, not data quality)* | DOI archive **3** · curated ML repository (UCI/OpenML) **8** · package distribution (CRAN) **1** · simulator codebase **1** · governed testbed (iTrust) **2** · community upload (Kaggle) **1** · author/vendor host **4** |

So **8 of 20** carry an explicit open licence and **12 of 20** are open-confirmed access; the remaining
records are disclosed exactly — 5 licences and 4 access routes are flagged *requires verification*, and
the 2 iTrust testbed datasets (SWaT/WADI) are honestly marked agreement-governed / redistribution-prohibited.

## Reproducibility

Fixed Rust/crate versions, frozen thresholds, deterministic seeds, dataset SHA-256 gates, artifact
and replay hashes, exact CLI commands, and a Colab notebook. The CUDA path adds byte-exact replay
verification of the forensic case file. The notebook shows code feedback and regenerates the paper
figures but **does not** compile the paper.

**Byte-verify the balance-witness results without redistributing data.**
`scripts/verify_reproducibility.py` regenerates each balance-witness trace and checks a canonical,
platform-portable SHA-256 of it against committed digests
([data/instrumented/EXPECTED_DIGESTS.toml](crates/dsfb-chemical-engineering-edge/data/instrumented/EXPECTED_DIGESTS.toml)).
The four **synthetic** demonstrators (three-tank, quadruple-tank, CSTR, CSTH) are reproducible by anyone;
the two **gated** ones (SWaT T101, BATADAL T1) verify only for a holder of the licensed data (the verifier
skips them otherwise) — so the real-data results are independently checkable **without** shipping a byte.
This converts "trust the recipe" into "byte-confirm your run".

## What this project does NOT claim

- It does **not** claim higher accuracy or faster detection than established chemometrics. The
  framing is augmentation, not competition.
- It does **not** prove physical root cause; residual motifs suggest structural candidates only.
- Public benchmarks (TEP, BSM1/2, IndPenSim, CSTR, …) are valuable but are **not** substitutes for
  proprietary plant historian data.
- It carries **no safety/control authority**; it is advisory unless separately certified and
  integrated by plant operators.

The correct failure mode is to emit *"unknown structural episode with preserved evidence"* rather
than to force a confident diagnosis.

## Change log (P-phase highlights)

- **Figure campaign path — optional, not bundled in this snapshot.** The renderer package (`scripts/figures/`,
  graphviz-first/networkx-fallback, colourblind-safe, on-figure disclaimers, SHA-256 figure-provenance manifest)
  remains a local artifact path, but the committed crate currently carries a zero-figure manifest and no paper PDF.
  This is deliberate release hygiene: generated paper/figure artifacts should ship only when their source, outputs,
  and verification logs are committed together.
- **P71 — final QA / legendary pass (correctness + consistency only).** Reconciled every count
  the P57–P69 build-out moved: edge lib **62**, default `cargo test --workspace` = **103**, cuda **13** with
  `--features cuda` (GPU-run) — the stale "55 workspace" / "edge 14 lib" / "cuda 7" claims were corrected
  here, in PROJECT_PLAN, and in the verification report. `ArtifactCompletenessCourtV1` re-run → COMPLETE
  (9/0), replay 6/6 byte-identical, frozen authority hashes intact. No code or paper-source change — only
  doc reconciliation. This completes the approved P52–P71 program.
- **P69 — SBIRTransitionPackV1 (machine + human Phase-I-style readiness pack; generic).** A hash-sealed,
  deterministic transition pack that reuses the P41 milestone-gate protocol: milestone gates (M0–M3, each
  with a replay-checkable go/no-go condition anchored to a sealed artifact hash and a `Met`/`Pending`/
  `OutOfScope` status), readiness claims (each with an explicit boundary of what is *not* claimed), a risk
  register, reproduction steps, and an up-front non-claims block. It renders both as a sealed machine record
  (`verify()`) and a human one-pager (`to_markdown()`), and **names no agency, program, or vendor**
  (test-enforced). Replay-inert.
- **P68 — context overlays + append-only human-review chains.** Real operations layer context and human
  review on top of immutable evidence without mutating it. Two read-only overlays mark windows where
  residuals are *expected* — `RecipeTransitionGuardV1` (near a batch-recipe phase change) and
  `MaintenanceEventOverlayV1` (inside a maintenance outage) — so an episode in such a window is
  contextualised, not blindly alarmed. Two append-only hash-chained logs make review tamper-evident:
  `OperatorAnnotationLedgerV1` (operator notes, each chained to the prior entry) and
  `EvidenceAmendmentChainV1` (amendments anchored to the immutable original evidence hash — the original
  stands, corrections are appended). All hash-sealed + self-verifying. Replay-inert.
- **P67 — governed authority drift (MigrationReceipt / RegistryCompatibility / SemanticDiff).** When the
  authority changes (e.g. an `atlas_hash_v1` re-freeze), three hash-sealed objects record exactly what
  moved: a semantic diff (added/removed/changed/unchanged) over record snapshots, a detector-registry
  compatibility verdict (backward-compatible iff nothing was removed), and a per-heuristic migration
  receipt. Replay-inert.
- **P66 — ProcessNarrativeCompilerV1 + NoNarrativeHallucinationGateV1 (anti-hallucination).** Operator
  prose with no hallucination path: the compiler is **not an LLM** — every sentence is a fixed template
  filled from structured fields and carries the hash of the specific sealed evidence object it came from;
  the gate then mechanically proves every sentence is anchored to a real evidence object (any un-anchored
  sentence fails). A passing narrative cannot assert anything not backed by sealed evidence. Replay-inert.
- **P65 — structural-similarity / case-law objects (advisory-only).** Episodes get a structural
  fingerprint (`EpisodeShapeHashV1`); from it, nearest-motif retrieval, cross-run recurrence detection, a
  shape-keyed case-law precedent index ("this resembles case X"), and cross-plant fleet comparison — every
  result carrying a sealed advisory that resemblance is a retrieval hint, **not** an identity or causal
  claim. Replay-inert.
- **P64 — multi-unit topology + propagation objects (explicitly non-causal).** A feed→reactor→separator
  demonstrator with declared residence times backs four new objects: `ProcessTopologyGraphV1` (unit/flow
  graph), `ResidenceTimeAlignmentV1` (at-lag correlation between an upstream and downstream unit — advisory),
  `FaultPropagationWitnessV1` (observed-vs-declared lag consistency, with a mandatory non-causal disclaimer),
  and `CausalNonClaimGraphV1` — the anti-overclaim object whose every rendering states *no causal claim is
  made* (the disclaimer is sealed into the hash). Replay-inert.
- **P63 — SensitivitySweepReceiptV1 + AblationCourtV1.** A deterministic threshold-grid sweep records the
  headline metric at every grid point and reports its range (how sensitive the result is to threshold
  choice), and an ablation court measures each component's contribution by disabling it and recording the
  delta vs the full pipeline (naming the most load-bearing component). Both hash-sealed + replay-inert.
- **P62 — ControllerMaskingHeuristicV2 + ValveStictionWitnessV1 + AlarmFloodCompressionReportV1 + OperatorIncidentHTMLV1.**
  Controller-masking is sharpened from a single H6 condition to a four-signal conjunction (stable PV ∧
  drifting MV ∧ rising effort ∧ rising residual energy); valve stiction gets a formal four-signature
  witness. The alarm-flood compression report makes the `lost_evidence = 0` / `recoverable = true`
  invariants explicit (compression is a view, raw alarms preserved) and emits HTML, alongside a sealed
  nine-question operator incident one-pager. Replay-inert.
- **P61 — BalanceWitnessV1 (+ stoichiometric/yield/selectivity residuals) + SoftSensorWitnessV1.** The
  mass/energy balance-closure residual becomes a hash-sealed `BalanceWitnessV1` record, and three
  reaction-chemistry residual kinds are added (stoichiometric, yield, selectivity). `SoftSensorWitnessV1`
  makes a soft sensor's output first-class evidence — measured / prediction / residual / interval, with the
  model family and a `training_scope_hash` disclosed — so its error stream is court-admissible. Replay-inert.
- **P60 — UnknownTaxonomyV1 (7-class authority) + ConfuserDocketV1 (per-episode).** A curated, hash-sealed
  7-class taxonomy of *why* an episode is left unknown (short transient, detector conflict, out-of-regime,
  residual degeneracy, missing process context, non-stationary baseline, insufficient witness diversity) is
  added to the atlas authority and folded into `atlas_hash_v1` (deliberately re-frozen; CUDA evidence roots
  + edge bundle roots unaffected). A per-episode `ConfuserDocketV1` promotes the atlas's static confuser
  list to an emitted, hash-sealed docket recording, for each matched fault, the confusers ruled out and the
  discriminating signature. Replay-inert.
- **P59 — DetectorDisagreementForensicsV1 + NegativeWitnessV1.** Promotes the *silent* half of the
  evidence to first-class objects: a `NegativeWitnessV1` records a silent detector (which one, why it was
  silent, and what its silence rules out), and `DetectorDisagreementForensicsV1` is the full per-episode
  report — firing (participating), silent (negative witnesses), contradicting, a witness-diversity score,
  and the carried disagreement entropy — sealed by a self-verifying `forensics_hash`. Additive, replay-inert.
- **P58 — ChemometricPassportV1 (per-detector) + ResidualProvenanceGraphV1 (DOT/JSON).** A per-detector
  passport pins, by SHA-256, the baseline window the detector fit on, the input matrix it scored, and the
  output stream it emitted, plus the disclosed threshold/normalization/missingness policy — all sealed
  under a self-verifying `passport_hash`. The residual-provenance ledger is promoted to a hash-sealed
  **graph** (`raw → residual → detector → episode → label → court root`) emittable as Graphviz DOT and
  JSON, so "where did this label come from?" is a walkable, verifiable artifact. Additive, replay-inert.
- **P57 — RegimeEnvelopeV1 + ChemicalAuthoritySeparationLawV1 (named, hash-sealed authority objects).**
  `RegimeEnvelopeV1` formalizes the per-regime admissibility envelope into a self-verifying object with a
  `provenance_hash` sealing exactly how it was calibrated (and a `relaxes_default()` invariant so a
  tightened envelope is caught structurally). `ChemicalAuthoritySeparationLawV1` states the five
  execution↔authority separation rules as an executable doctrine — three re-checked at runtime (executed ⊆
  catalogued; deterministic single-source authority hash; `&'static const` records), two enforced at
  compile time (authority is `no_std`/dependency-free; the dependency arrow points one way). Additive +
  replay-inert (6/6).
- **P54 — verification report + `ArtifactCompletenessCourtV1`.** A new `completeness-court` edge command +
  gate asserts the committed **artifact graph** is complete and mutually consistent: MANIFEST count = 20
  blocks, every dataset has a 64-hex SHA-256, every frozen Court Record entry has a 64-hex
  `bundle_root`+`evidence_root`, the manifest and bundle-root tables name the **same** 20 datasets with
  consistent provenance kinds, the atlas/corpus authorities validate with well-formed hashes, the corpus
  classification census **partitions all 20 records** on each axis, and the metrics protocols are present.
  It emits a hash-sealed pass/fail report — current verdict **COMPLETE (9/0)**. A companion
  `reports/verification_report.md` records the capture host, verbatim re-runnable commands, test totals
  (103 workspace + 13 `--features cuda`), the frozen hashes, replay 6/6, and the court verdict. Scoped
  honestly to the machine-checkable graph (PDF/prose-count reconciliation stays the manual QA sweep).
- **P70 — CUDA evidence-kernel V2 (GPU-measured, digest-equivalence-gated; pulled forward).** Built
  `DigestEquivalenceHarnessV1` (the law: any kernel must reproduce the CPU reference's lane digests /
  Merkle root / `evidence_root` / replay byte-for-byte) and two optimizations behind it, each measured on
  an RTX 4080 SUPER via Nsight: **V2-A** lane-batching (digest-identical; achieved occupancy 8%→52%) and
  **V2-B** segment-parallel (opt-in `evidence_root_v2` Merkle-segment format; deep 1024×8192 case
  **29.75→1.65 ms ≈ 18×** kernel, ~8× end-to-end with pinned H2D), plus digest-preserving SHA micro-opts.
  A `cudaMemcpyAsync` stream-overlap was built, **measured 2.7× slower**, and reverted — disclosed as a
  negative result (occupancy collapses when chunked). All gated byte-exact; V1's frozen roots unchanged.
  See `docs/cuda_evidence_kernel_v2_design.md` + `crates/…-cuda/reports/NSIGHT_SUMMARY.md`. (CUDA was
  pulled ahead of the sequenced P54–P69 plan at the maintainer's direction; those resume next.)
- **P53 — dataset provenance-classification tiers (disclosure accuracy; `corpus_hash_v1` re-frozen).**
  The `corpus` authority crate now classifies all 20 records on four orthogonal, hash-sealed *honest
  disclosure* axes — **licence confidence**, **access confidence**, **redistribution policy** (what a
  downstream user must respect; this crate ships no bytes), and **source-authority kind** — each derived
  from the cited licence + URL + venue (confidence/policy statements, not legal opinions). A `census()`
  counts every tier and a gate asserts each axis **partitions all 20 records**; the agreement-gated
  iTrust datasets (SWaT/WADI) are invariant-checked as redistribution-prohibited. See the new "Dataset
  provenance classification" table above. Folding the tiers into the canonical preimage re-froze
  `corpus_hash_v1` (deliberate); both execution backends print the new hash + census. Corpus tests 5→7;
  edge/CUDA evidence roots unaffected (corpus is off the replay path); replay 6/6 byte-identical.
- **P52 — public-release wording + consistency (disclosure accuracy; replay-inert).** A release-readiness
  pass over public-facing wording: neutralized stale public-facing notes, fixed a stale
  `corpus` doc-comment ("molecular corpus"→"soft-sensor data corpus"), added an explicit clarifier that
  **PubChem-scale molecular densors are a future companion corpus, not part of this artifact** (the
  `corpus` crate is a soft-sensor dataset catalogue only), reframed the paper front-matter to the
  **four-crate** architecture (crates.io edge/cuda · authority atlas/corpus), corrected a stale "0 overfull"
  claim to the honest `.log`-audited wording, and framed the CUDA evidence kernel as **baseline
  deterministic sealing, not a final-throughput claim** — any future optimization must pass a
  **digest-equivalence law** (identical lane digests / Merkle root / evidence root / replay; only timing
  changes). Docs/comments/paper only; workspace tests green, `verify-replay` 6/6 byte-identical.
- **P48–P51 — second panel re-review, additive hardening (breadth preserved; current state: 42-page paper).**
  A second read-only 5-discipline panel re-reviewed the post-P47 state (composite ≈9.1–9.3, up from ≈8.6;
  the SHA-256 fix was independently C-port-proven across all message lengths). It drove four fixes:
  **P48** caught a genuine honesty slip — the prior "0 overfull" check was a `build_paper.sh` grep-filter
  artifact; the real `.log` had 1 overfull (the P41 milestone table), now fixed, and the build script now
  audits the `.log` directly. Also corrected the energy-balance noise-floor budget (`ρc_pV·σ_T/Δt ≈ 5×10³`
  J/min — thousands, a lower bound; the ~2.5×10⁵ baseline is model-form-dominated) and added a closure-gate
  **clause (iii) model-fidelity** + small fixes (SWaT 36/35, slew wording, Johansson citation).
  **P49** added a **host-side SHA-256 parity test** (gates the `n≡3 mod 8` fix on every machine, no GPU)
  + a two-block `golden_evidence` case. **P50** extended byte-verification from the witnesses to the
  **full Court Record across all 20 datasets** (`EXPECTED_BUNDLE_ROOTS.toml` + `--bundles`), added a
  fired-after-onset disposition guard, and committed `METRICS_DEFINITIONS.toml` (the rate denominators).
  **P51** promoted **F3 (fouling)** + **F8 (controller-masking)** Catalogued→Executed via gated
  demonstrators (now **6 of 12** executed; `atlas_hash_v1` re-frozen) + minor Rust polish. All replay-inert.
- **P42–P47 — final QA + first panel-driven additive hardening batch (breadth preserved).**
  **P42** final paper QA pass (0 undefined/0 overfull, 0 `[?]`/`[??]`, every count reconciled, full
  traceability). Then a 5-specialist panel review (composite ≈8.6/10) drove five additive fixes:
  **P43** fixed a device-SHA-256 two-block padding bug (`n_samples ≡ 3 mod 8` inflated the length by
  512 bits → GPU digest diverged from the host; no wrong evidence was ever sealed — the court fails
  closed to the CPU reference — but the "GPU≡CPU on all 20" claim had held only by sample-count luck);
  added a feature-gated GPU↔CPU parity gate spanning the residue classes.
  **P44** tiered the balance-closure disclosure (mass closes ≈0; energy carries a structural offset)
  and added the differentiation-noise-floor applicability bound `≈ ρc_pV·σ_T/Δt`.
  **P45** defined the title construct **residual semiotics**, fixed the "second-order"→"first-order
  rate-of-change" slew mislabel, defined the neologisms, and added a construct-glossary appendix.
  **P46** signed-zero-canonical CSV formatting + a frozen `bundle_root` golden gate; `MannKendall`
  overflow guard; honest `out()` threshold floor; ragged/tiny-CSV resilience (no panic).
  **P47** `verify_reproducibility.py` + committed `EXPECTED_DIGESTS.toml` — byte-confirm the
  balance-witness results (4 synthetic + 2 gated) against canonical digests **without redistributing
  data**. All replay-inert: `verify-replay` 6/6, CUDA `evidence_root` 20/20, clippy clean.
- **P34–P41 — prior-art strengthening batch (additive; breadth preserved).**
  **P34** provenance accuracy — the nine simulations now read `simulation/slice` everywhere at runtime
  (was hardcoded `measured/slice`), with a provenance-consistency gate.
  **P35** promoted the balance-witness applicability criterion to its own paper section + abstract line +
  [docs/balance_witness_criterion.md](docs/balance_witness_criterion.md).
  **P36** SWaT scope-stratified recall vs the official iTrust attack list — within-scope recall 5/5 = 100%,
  73% out-of-scope specificity, 4.4% FP (recipe `scripts/swat_scope_recall.py`; list not redistributed).
  **P37** incumbent head-to-head on TEP IDV(1) (`scripts/head_to_head_tep_idv1.py`,
  [docs/head_to_head_tep_idv1.md](docs/head_to_head_tep_idv1.md)) — 10,041 breach-steps → 6 episodes.
  **P38** correctness/determinism hardening — `analyze()` degenerate-matrix guard, cuda
  `gpu_cross_verified` flag, checked GPU downcast, a frozen cuda evidence golden gate, `RejectionReason`
  reserved-vocabulary grouping. **P39** noise-floor preservation as its own bounded disclosure
  ([docs/noise_floor_preservation.md](docs/noise_floor_preservation.md)) + a causal-claim softening.
  **P40** F1 (stiction) + F9 (valve-hunting) **catalogued → executed** via synthetic control-loop
  demonstrators (gated by `tests/fault_demonstrators.rs`); `atlas_hash_v1` re-frozen, CUDA `evidence_root`
  unchanged. **P41** a fictional, fully-synthetic forensic incident walkthrough
  ([docs/forensic_incident_walkthrough.md](docs/forensic_incident_walkthrough.md)) + a milestone-gated
  (M0–M3) evaluation protocol in the paper. Paper → 40 pages; all gates green; replay byte-identical.
- **P33 — hardening.** `rust-toolchain.toml` pin; **frozen golden replay-hash gate**; cuda
  `#![deny(unsafe_code)]`; NaN-safe robust statistics + detector sorts; a **Rust-side MANIFEST SHA-256
  provenance gate** in `demo`; `balance.rs` witness tests; the cuda case-file `Passport` now seals the
  `atlas_hash` (binds each case to its authority, evidence root unchanged).
- **P32 — historian evaluation.** The `historian <csv>` loader accepts the richer long-format schema
  (`unit`/`controller_mode`/`setpoint`/`manipulated_variable` → derived per-tag witness columns; bare
  `timestamp,tag,value` unchanged); a historian run now emits the Court Record bundle and, when a
  `<name>.roles.json` declares a balance, a balance witness. Ships a synthetic tank-historian fixture.
  Also fixed a NaN-sort panic in the robust median/MAD (quality-gated samples no longer poison or crash
  the statistic).
- **P31 — operator evaluation protocol + paper honesty fixes.** New paper §; SWaT dual-role reconciled;
  balance-witness false-positive rates reported (SWaT 4.4%, BATADAL 0.1%); negative TEP delays owned;
  "no distributional assumption" softened to its defensible core. Paper → 36 pages.
- **P29 — Chemical Court Record v1.** Canonical `dsfb_chemical_engineering_casefile_v1/` bundle +
  per-episode claim-boundary badges + `non_claims.md`; `casefile <dataset>` command (see above).
- **P28 — authority CLI.** `edge corpus` and `cuda atlas` / `cuda corpus` print the frozen
  `atlas_hash_v1` / `corpus_hash_v1` from **both** execution backends (one shared authority).
- **P22–P26 — four-crate split + soft-sensor pivot.** Added the `atlas` (detector + H1–H6 + F1–F12
  authority) and `corpus` (20 sourced public soft-sensor datasets) crates; the corpus pivoted from a
  PubChem surface to **deterministic soft sensing**.
- **P16–P21 — balance witnesses.** Five mass/energy-balance arms; real-data wins on **BATADAL** and
  the **SWaT** testbed; the closure-gate applicability criterion (and honest rejections).
- **P9–P15 — operator + court depth.** Regime-conditioned envelopes, 5-class unknown taxonomy,
  evidence grades, NE 107 + alarm-rationalisation exports, operator HTML, residual-provenance ledger,
  counterfactual non-admission, contribution traces, per-rule challenge docket, plant-historian replay.

## Acknowledgments

Above all, we thank the countless chemical engineers and process scientists whose accumulated work this builds
upon — we see as far as we do only by standing on the shoulders of giants, and without them this work would not be
possible. Full dataset credits and provider links are in [`ACKNOWLEDGMENTS.md`](ACKNOWLEDGMENTS.md).

## License

Apache-2.0 for the reference implementation; commercial deployment requires a separate written
license. See `LICENSE` and `NOTICE`. Bundled dataset slices retain their upstream licenses.

## Citation

See `CITATION.cff`. Dataset credits + provider links: [`ACKNOWLEDGMENTS.md`](ACKNOWLEDGMENTS.md) (per-dataset
provenance in `crates/dsfb-chemical-engineering-edge/data/MANIFEST.toml`).
