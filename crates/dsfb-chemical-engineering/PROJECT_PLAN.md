# DSFB-Chemical-Engineering — Project Plan & Design (source of truth #1)

This is the project-facing design and status document. It records the architecture, the dataset
manifest, the detector atlas, the benchmarking methodology, the build/run commands, and the current
build status. It is kept in sync with the code at every milestone.

*Riaan de Beer — Invariant Forge LLC — ORCID 0009-0006-1155-027X.*

---

## 1. Thesis & scope

DSFB-Chemical-Engineering is a deterministic, **read-only residual-interpretation layer** for
chemical process monitoring. Established chemometrics detectors are the **witness bank**; DSFB
produces the **replayable court record** over the residuals they emit. It replaces no estimator,
controller, historian, or alarm system. Prior-art framing is deliberately **broad**.

**Hard rules.** Empirical honesty (every paper number traces to a committed artifact); no
overclaiming; explicit *measured* vs *simulation-benchmark* dataset labels; an honest *unknown rate*;
augmentation-not-competition posture; local git only (never push/publish); read-only public-dataset
handling with provenance + SHA-256.

## 2. Architecture

```
ResidualSample ─▶ ResidualProcessor ─▶ ResidualTriple(r, δ, σ)
                  (drift ring-buffer δ, slew first-difference σ)
                                     │
                       AdmissibilityEnvelope ─▶ evaluate ─▶ EnvelopeEval
                                     │
                            GrammarClassifier (automaton)
        precedence: Compound > EnvViolation > SlewSpike > DriftAccum
                   > BoundaryGrazing > Recovery > Nominal  (+ SensorFault OOB)
                                     │
        GrammarState + ReasonCode ─▶ Episodes (contiguous) ─▶ EpisodeSummary
```

On top of this DSFB core sits the **chemometric detector atlas** (a literature detector corpus of
families A–D), a **deterministic quorum fusion** layer (drift persistence, slew intensity,
detector-family diversity, variable-group coherence, process-phase admissibility, provenance), and a
**chemical heuristics bank** (motif → operator-readable label). The CUDA crate adds a GPU evidence
factory + a hash-linked forensic court + byte-exact replay + Nsight/GB-s benches.

### Crate split (eight crates — execution · authority · embedded · bindings · browser · runtime substrate)
> Originally four (execution vs authority); the post-P71 program added the embedded `core` (Wave 5), the
> `py` bindings (Wave 5), and the standalone `wasm` what-if simulator (panel batch); P97.3 added the
> `dsfb-densor-runtime` execution substrate (the sixth workspace member — a mechanism that carries **no chemical
> and no cross-domain claims**, so it does not change the chemical surface). `core` and `dsfb-densor-runtime` are
> workspace members; `py`/`wasm` are standalone workspaces excluded from the host build. See the post-P71 ledger at
> the end of this file for their commit detail.
- `dsfb-chemical-engineering-edge` — **execution**: the core library + CPU pipeline + CLI/demo +
  figures + mass/energy-balance witnesses + the **Chemical Court Record v1** bundle (`court_record`).
  Unsafe-forbidden (`#![forbid(unsafe_code)]`), dependency-light edge Rust; **std-only as shipped**
  (uses `std::{fs,path,time}`; no GPU required) — **not a `no_std` crate** (unlike `atlas`/`corpus`).
  The detector corpus holds **57** records, of which **18 are executed**; the `atlas` crate freezes
  those 18 executed records + H1–H6 + F1–F12 (the 39 catalogued detectors stay in the edge TOML;
  Phase-C promoted mewma/dpca/mosum/mmd Catalogued→Executed, 14→18; the 57-detector census is unchanged).
- `dsfb-chemical-engineering-cuda` — path-depends on `edge` for detector/fusion semantics; adds
  `cuda/` kernels (fixed-point + on-GPU SHA-256), FFI/dispatch/pinned workspace, `court/*`, `bench/*`.
- `dsfb-chemical-engineering-atlas` — **authority** (no_std, depends on nothing): curated `&'static`
  records for the 18 executed chemometric detectors, the H1–H6 process heuristics, and the F1–F12
  process-fault signature bank; validation gates + frozen `atlas_hash_v1`. `edge` consumes it and a
  subset-gate test proves every executed detector/heuristic is catalogued.
- `dsfb-chemical-engineering-corpus` — **authority** (no_std, depends on nothing): a provenance-bound,
  deduplicated catalogue of public soft-sensor datasets (cheap sensors → hard-to-measure target),
  every record sourced (URL + licence + access) and classified on four hash-sealed provenance tiers
  (licence/access confidence, downstream redistribution policy, source-authority kind; P53); no dataset
  bytes vendored; `corpus_hash_v1`. `edge` depends on it optionally under feature `soft-sensor-corpus`
  (off by default).
- `dsfb-chemical-engineering-core` *(Wave 5)* — **embedded** (`no_std`, no-heap, fixed-point, zero deps,
  `#![forbid(unsafe_code)]`): the residual triple + ring buffer + admissibility envelope + grammar state
  machine in scaled integers. Builds for `thumbv7m-none-eabi` **and** `wasm32-unknown-unknown`; the
  `qemu-smoke/` harness runs it on an emulated Cortex-M3. Not claimed bit-identical to the edge float pipeline.
- `dsfb-chemical-engineering-py` *(Wave 5)* — **bindings** (pyo3, standalone/excluded from the host workspace):
  a thin abi3 wheel exposing the file-free read-only courts (`version` / `classify_unit_pair` /
  `grade_readiness`); built with maturin; publishing is USER-ONLY.
- `dsfb-chemical-engineering-wasm` *(panel batch)* — **browser** (standalone/excluded; raw `extern "C"` exports,
  no wasm-bindgen): the interactive Chemical Court "what-if" simulator. Builds the dependency-free `core` grammar
  to `wasm32-unknown-unknown` + a static HTML/JS shell (`web/`) that replays a residual stream under an
  operator-amended admissibility envelope over immutable evidence (a HAZOP/training tool; not a controller).
- `dsfb-densor-runtime` *(P97.3)* — **runtime substrate** (workspace member, `std`, `#![forbid(unsafe_code)]`):
  a thin deterministic `load → validate authority → execute stages → seal → emit receipt` spine (traits
  `Densor` / `RuntimeStage` / `StageReceipt`, a per-stage *no-claim-without-an-authority-hash* gate, a sealed
  `RuntimeReceiptV1`). A reusable mechanism only — it carries **no chemical and no cross-domain claims**, so the
  chemical crates remain the sole domain authority (strictly-chemical posture preserved). 8 unit tests
  (determinism + tamper-evidence + both gate refusals); Miri-clean.

## 3. Chemometric detector atlas (families A–D)

Each detector keeps the corpus schema (`canonical_id, display_name, aliases, primitive_family,
mathematical_form, decision_functional, input_requirements, origin_domains, output_witness,
witness_role, negative_witness_kind, fusion_axes, confuser_profile, deterministic_status,
implementation_status, family, genealogy, source_refs`) and is build-time validated (SHA-256 dedup;
deterministic count gate).

- **A. Classical MSPC:** Shewhart, Hotelling T², PCA-T², PCA-SPE/Q, PLS score/prediction residual,
  SIMCA distance, contribution plots, Western-Electric/Nelson rules, Tukey fences, robust z (MAD).
- **B. Dynamic/Temporal:** EWMA, CUSUM, MEWMA, Page-Hinkley, Mann-Kendall, Pettitt, SNHT, MOSUM,
  Buishand, DPCA, moving-window/recursive PCA, lagged autocorrelation break.
- **C. Nonlinear/Distributional:** KPCA / autoencoder reconstruction error, ICA residual, one-class
  SVM / kNN / LOF distance, KS, KL, JS, MMD, Wasserstein, energy, Hellinger, total-variation, PSI,
  spectral entropy, wavelet energy.
- **D. Process-structure:** variable-group co-drift, unit-operation block, mass/energy-balance
  residual, control-action mismatch, actuator-lag, sensor-stiction, valve-hunting, batch-phase
  residual, missingness spike/coupling.

### Heuristics bank (chemical) — H1–H6
H1 sensor bias drift · H2 actuator stiction / valve stick-slip · H3 reactor thermal excursion
candidate · H4 feed-composition disturbance · H5 batch-phase misalignment · H6 controller-
compensation masking. Schema: `heuristic_id, name, applicable_process_type, required_variables,
detector_inputs, residual_pattern, drift_condition, slew_condition, admissibility_condition,
episode_label, operator_explanation, known_false_positive_modes, known_false_negative_modes,
severity_policy, engineering_basis`.

## 4. Datasets (20)

`[M]` measured real data · `[S]` public simulation benchmark. Only small processed slices are
vendored (openly-licensed sources); provenance + SHA-256 in `…/edge/data/MANIFEST.toml`.

This table lists the **actual committed slices** (the exact `data/slices/*.csv` set the demo runs and
the MANIFEST seals — 10 measured / 9 simulation / 1 gated stand-in = 20). Simulation slices are
generated deterministically; measured slices are processed from the cited public source.

| # | Committed dataset (slice id) | Type | DSFB role | Vendoring |
|---|---|---|---|---|
| 1–5 | Tennessee Eastman IDV01/04/06/13/14, Rieth 2017 (`tennessee_eastman_idv0{1,4,6}`, `…idv1{3,4}`) | [S] plantwide process | canonical fault-propagation; detection-delay study | 5 slices + SHA |
| 6 | Three-tank system (`three_tank`) | [S] actuator/leak | leak / actuator residual; mass-balance witness | slice + SHA |
| 7 | CSTR reactor (`cstr_reactor`) | [S] reactor | thermocouple-drift motif; energy-balance witness | slice + SHA |
| 8 | BSM1 wastewater, reduced ASM1 (`bsm1_wastewater`) | [S] bioprocess control | slow drift / disturbance residual | slice + SHA |
| 9 | Penicillin fed-batch, sim (`penicillin_fedbatch`) | [S] batch/PAT | batch-phase envelopes (regime-conditioned hard case) | slice + SHA |
| 10 | Gas Sensor Array Drift, UCI (`gas_sensor_array_drift`) | [M] e-nose | 36-month sensor drift (flagship hard case, ~74% FP) | slice + SHA |
| 11 | Air Quality multisensor, UCI (`air_quality_multisensor`) | [M] e-nose / env | cheap-sensor co-drift | slice + SHA |
| 12–13 | Wine Quality red + white, UCI (`wine_quality_red`, `…_white`) | [M] physicochemical | tabular residual / co-drift | 2 slices + SHA |
| 14 | Steel Plates Faults, UCI (`steel_plates_faults`) | [M] manufacturing | tabular fault residual | slice + SHA |
| 15 | SECOM semiconductor, UCI (`secom_semiconductor`) | [M] process monitoring | high-dimensional tabular residual | slice + SHA |
| 16 | Tecator NIR meat, OpenML (`tecator_nir_meat`) | [M] spectroscopy | fat/protein NIR residual (corpus flags `cheap_sensor=false`) | slice + SHA |
| 17–19 | Corn NIR m5/mp5/mp6, Eigenvector (`corn_nir_m5`, `…_mp5`, `…_mp6`) | [M] spectroscopy | NIR reconstruction residual, instrument standardisation | 3 slices + SHA |
| 20 | SWaT stand-in, iTrust SUTD — real data gated (`swat_water_treatment_standin`) | [gated] water-treatment ICS | plant-wide ICS residual; the balance-witness result on the **real** testbed is computed locally and **not** redistributed | synthetic stand-in slice + SHA |

## 5. CUDA + forensic court + Nsight methodology

- Kernels in `cuda/kernels.cu` use **fixed-point determinism** + **on-GPU SHA-256** (mirroring
  `dsfb-gpu`). The CPU **court** retains semantic authority.
- Court (`src/court/*`): hash-linked case files, passports, execution attestation, challenge docket,
  precedent, admissibility — built from residuals **and usually-discarded noise** → deterministic,
  replayable, byte-exact, auditable verdict. Replay re-derives identical hashes.
- **Benches (run multiple times):** `run_bench.sh` measures effective bandwidth (GB/s) over the
  20-dataset bundle, **≥5 runs** (min/median/max + variance committed); `run_nsight.sh` captures
  `nsys` + `ncu` metrics (achieved occupancy, DRAM throughput, L2 hit rate, warp efficiency),
  **≥5 runs / ≥2 variants**, all raw outputs committed under `reports/`. Paper tables cite them.

## 6. Paper

`paper/dsfb_chemical_engineering.tex` (+ `preamble.tex`, `bibliography.bib`), auto-compiled by
`paper/build_paper.sh` (latexmk, fallback 4-pass pdflatex+bibtex). Structure follows the project
outline: introduction → chemometrics/MSPC background → read-only residual semiotics → detector
ensemble → heuristics bank → formal definitions + propositions → CUDA evidence factory + forensic
court → experimental protocol → results (Tables 1–6) → effective-bandwidth + measured CUDA pipeline
→ case studies (incl. mandatory failure case) → **legendary limitations + 20-question critical
reviewer** → reproducibility + conclusion → appendices A–E. All numbers trace to committed artifacts.

## 7. Build / run commands

```bash
cargo build --release                                            # workspace
cargo test                                                       # all crates (edge, cuda, atlas, corpus)
cargo run --release -p dsfb-chemical-engineering-edge -- demo    # edge full demo
PATH="/opt/cuda/bin:$PATH" bash crates/dsfb-chemical-engineering-cuda/scripts/build_cuda.sh
cargo run --release -p dsfb-chemical-engineering-cuda -- demo    # cuda demo + court + replay
bash crates/dsfb-chemical-engineering-cuda/scripts/run_bench.sh  # GB/s, multiple runs
bash crates/dsfb-chemical-engineering-cuda/scripts/run_nsight.sh # nsys/ncu, multiple runs
bash paper/build_paper.sh                                        # paper PDF
jupyter nbconvert --to notebook --execute notebooks/dsfb_chemical_engineering_colab.ipynb
```

## 8. Build status

> **Historical phase log (P0 → present), most-relevant-last.** This is the running development record. For
> *current* verified status read the **Implementation audit** block near the end of this section (and
> [`reports/verification_report.md`](reports/verification_report.md)); the per-phase entries below are
> historical and may describe counts/claims that were true *at that phase* and have since been superseded
> (e.g. page counts and test totals before the P71 reconciliation). Entries are not rewritten retroactively.

- [x] **P0 — Scaffold & guardrails:** git, workspace `Cargo.toml`, `README.md`, `PROJECT_PLAN.md`,
      `LICENSE`/`NOTICE`/`CITATION.cff`, `.gitignore`. Reference code extracted (gitignored).
- [x] **P1 — Edge core + chemometric detector atlas + fusion + heuristics bank + reports + CLI/demo.**
      Implemented: DSFB engine (`dsfb_core`), NIPALS PCA + FFT (`linalg`,`fft`), 14 executed detectors
      across 4 families (`detectors`), 57-detector validated corpus (`atlas`,`corpus/`), one-sided
      exceedance grammar + deterministic quorum fusion (`fusion`), H1–H6 heuristics bank with honest
      "unknown" fallback (`heuristics`), CSV/JSON/MD reports + store-only ZIP bundler (`report`),
      synthetic generators + CSV slice loader (`datasets`), CLI `demo|analyze|atlas|verify-replay`
      (later phases add `corpus`, `regime-eval`, `historian`, `balance-witness`, `control-action`,
      `casefile`), Kani harnesses. Tests pass; demo is byte-exact reproducible. On the synthetic suite: detection
      delay 0–16 samples, baseline false-positive rate 0.7–6.3%, unknown rate 50–90% (honest).
- [x] **P2 — Datasets.** 20 slices committed (`data/slices/`, 6.8 MB) with provenance manifest
      (`data/MANIFEST.toml`, real SHA-256 + source + license + citation): **10 measured** (Wine
      red/white; SECOM @590; Steel Plates; Gas Sensor Array Drift; Air Quality; Tecator NIR; Corn
      NIR m5/mp5/mp6), **9 simulation** (TEP IDV1/4/6/13/14 @52 vars — Downs–Vogel simulator; CSTR
      Arrhenius, three-tank Torricelli, BSM1-style ASM1, fed-batch penicillin), **1 agreement-gated**
      stand-in (SWaT). `scripts/fetch_datasets.py` reproduces them; `datasets.rs` loads slices.
- [x] **P3 — Edge evaluation + figures.** Demo runs all 20 deterministically (replay OK). Key
      results: TEP IDV1 detection delay = 1 sample (onset @160), IDV4/IDV6 = 0, baseline FP 3–9%;
      CSTR delay 0 / FP 0.4%. High baseline FP on non-stationary batch (penicillin), heterogeneous
      (gas-drift) and high-dim tabular (SECOM) sets is reported honestly. `scripts/gen_figures.py`
      renders 9 publication figures to `paper/figures/`; `scripts/export_artifacts.sh` is the
      one-command pipeline. `metrics.csv` supplies the Tables 1–6 data.
- [x] **P4 — CUDA crate: evidence factory + forensic court + byte-exact replay.** Fixed-point
      evidence contract (`evidence.rs`) with on-GPU SHA-256 (`cuda/{kernels.cu,sha256.cuh,common.cuh}`)
      that reproduces the host `sha2` digest **byte-for-byte**; FFI (`ffi.rs`, unsafe confined),
      `build.rs` (nvcc discovery, locked `--fmad=false`, sm_75 PTX + sm_89 native), dispatch with
      CPU/GPU cross-verification, forensic court (`court.rs`: passport, hash-linked case file, Merkle
      + evidence root, challenge docket, precedent chain, execution attestation, replay). **Verified:
      all 20 datasets' evidence roots are identical on CPU and GPU (xverify OK), replay OK.** CLI:
      demo | bench | profile | verify-replay | device; scripts build_cuda/run_bench/run_nsight.
- [x] **P5 — Nsight + GB/s benches.** GB/s sweep run 3× (5 internal runs/size) — evidence factory
      2.1–9.6 GB/s, memory roofline 620–641 GB/s (~88% of the 4080 SUPER's ~736 GB/s peak),
      committed `reports/bench_*.json`. Nsight Systems campaign: 5 runs × 3 size variants
      (`reports/nsys_*.txt`); evidence_kernel medians 6.87 ms (g, 4096 lanes) / 13.8 ms (a) /
      27.7 ms (b, 1024 deep) — confirming the kernel is SHA-256-compute / parallelism-bound, the
      same conclusion as the CUDA-event bench. `reports/NSIGHT_SUMMARY.md` + `nsight_summary.json`
      distill it. **Nsight Compute (ncu) microarch counters were captured** (perf counters enabled):
      SM throughput 2.6–10.2%, DRAM 0.9–2.0%, achieved occupancy ≈8.3%, L2 sector-hit ≈97% —
      compute/parallelism-bound confirmed (`reports/NSIGHT_SUMMARY.md`; paper Table `cuda_ncu`).
- [x] **P6 — Paper.** `paper/dsfb_chemical_engineering.tex` (+ `preamble.tex`, `bibliography.bib`,
      `sections/{limitations,appendices}.tex`, `tables/*.tex` auto-generated by `gen_tables.py`)
      builds via `build_paper.sh` (latexmk, fallback 4-pass) to a clean **PDF (33 pages at P6; grown with
      later phases — see the rollup below for the current count), 0 undefined
      refs/cites, 0 overfull boxes at P6** (a single overfull was later introduced in P41 and caught/fixed
      in P48; the build script now audits the `.log` directly). Every results number traces to committed
      artifacts. Includes
      the legendary limitations section (12 structural limitations + the **20-question hostile-
      reviewer attack surface** across 5 personas) and the honest-ceiling statement.
- [x] **P7 — Colab notebook.** `notebooks/dsfb_chemical_engineering_colab.ipynb` (generated by
      `gen_notebook.py`): §1 env+source → §2 build the two executable crates → §3 dataset SHA-256 gate → §4 run the
      20-dataset edge audit + CUDA forensic court → §5 pack downloadable ZIP → §6 byte-exact replay +
      cross-backend verify → §7 figures + metrics → §8 non-claims. **Validated headless** (nbconvert,
      exit 0): all 20 datasets replay OK / xverify OK, ZIP built. **Does not compile the paper.**
      Also fixed a manifest bug (a prior `--only` fetch had clobbered `MANIFEST.toml` to 2 entries;
      `write_manifest` now merges, manifest restored to 20, slices byte-identical, paper table fixed).
- [x] **P8 — Final verification + doc sweep.** Workspace tests pass (edge 5, cuda 4, cuda+feature 4);
      release build OK with CUDA; edge demo 20/20 replay OK; CUDA court 20/20 replay + cross-backend
      verify OK; dataset SHA-256 gate 20/20; paper PDF clean (33 pages at P8; 35 now); notebook validated headless.
      Authorship: all commits authored solely by Riaan de Beer. Local commits only — never pushed or published.
- [x] **P9–P18 — 20-item innovation roadmap (all implemented, opt-in, replay-preserving).** Regime-
      conditioned admissibility envelopes (`calibrate_regime_envelope`, `run_regime_eval`; gas-sensor
      baseline FP ~74→? and penicillin ~54→39%); unknown taxonomy (`classify_unknown`); disagreement
      fingerprints (`disagreement_entropy`); evidence-quality grades (`evidence_grade`); NE 107 +
      alarm-rationalisation exports (`write_ne107`, `write_alarm_rationalization`); operator HTML
      (`write_operator_report_html`); Chemical Non-Interference Axiom + minimal integration contract
      (paper); residual provenance ledger (`write_residual_provenance`); counterfactual non-admission
      (`write_non_admission`); contribution-trace court (`write_contribution_traces`); per-rule
      challenge docket (`challenge_docket`); fault-signature thesis (paper + later the atlas bank);
      plant-historian replay + batch mode (`run_historian`, `load_historian_csv`); control-action
      context (`run_control_action`). Default-off everywhere → sealed Tier-1 replay hashes byte-identical.
- [x] **P16–P21 — mass/energy-balance witnesses (`balance.rs`, `run_balance_witness`).** Five balance
      types (`mass_three_tank`, `energy_cstr`, `mass_quad_tank`, `energy_csth`, `mass_tank_volume`) on
      instrumented + REAL data: synthetic three-/quadruple-tank leaks + CSTR/CSTH drift caught at onset;
      **BATADAL** C-Town T1 fires on exactly the PU2 attacks (real water-network SCADA); **SWaT** T101
      fires on the LIT101 700 mm-freeze spoofs (real physical testbed). Applicability criterion
      (closed fully-metered volume + a fault that makes a conserved quantity *appear* non-conserved)
      validated on 6 real datasets; TEP/PRONTO/UCI-WWTP/BattLeDIM/RP-1043/HAI honestly rejected by the
      closure gate. SWaT/BATADAL data NOT redistributed (git-ignored; recipe + provenance only).
- [x] **P22–P23 — `atlas` + `corpus` authority crates** (four-crate execution/authority split).
      `atlas`: 14 executed detector records + H1–H6 process-heuristic records + validation gates +
      `atlas_hash_v1`; `edge` subset-gate proves every executed detector/heuristic is catalogued.
      (`corpus` initially shipped a PubChem/molecular densor surface — superseded in P26.)
- [x] **P24 — atlas process-fault SIGNATURE bank** (`fault_signature.rs`): F1–F12 cheap-sensor residual
      fingerprints (stiction, cavitation, fouling, HX bypass, pump/bearing, leak, sensor drift,
      controller masking, valve hunting, blockage, imbalance, refrigerant), each grounded in named
      public datasets; executed/catalogued honest (leak + sensor-drift executed). Folded into
      `atlas_hash_v1`.
- [x] **P25 — positioning: "deterministic inference, not deterministic-upon-probability"** (paper
      §`sec:pure-determinism` + README). OLS ≡ Gaussian MLE / PLS statistical foundation → noise-fragile;
      DSFB is pure deterministic (no probability model/likelihood/loss; distribution-free calibration).
- [x] **P26 — corpus pivot: PubChem REMOVED → chemical-engineering soft-sensor data corpus.**
      `SoftSensorDatasetRecordV1` + 20 sourced public datasets (SRU, Debutanizer, TEP, mining flotation,
      CCPP, gas-turbine CO/NOx, steel, wastewater BSM1/N2O/UCI, pulp Kappa, PRONTO, SECOM, gas-drift,
      IndPenSim, Tecator/Corn NIR flagged non-cheap, gated SWaT/WADI); `SourceRef` per record;
      `redistributed=false`; `corpus_hash_v1`; `prep_softsensor.py` seals a local copy's provenance.
      No dataset bytes vendored; no ownership/bio/tox/hazard/performance claims.
- [x] **P27 — doc stale-seam sync + commit-message cleanup.** Paper architecture reconciled to four crates;
      stray trailer lines that had crept into P8–P26 commit messages removed; every commit message authored
      solely by the owner.
- [x] **P28 — authority CLI.** `edge corpus` + `cuda atlas`/`cuda corpus` commands; both execution
      backends print the identical frozen `atlas_hash_v1` / `corpus_hash_v1` (one shared authority).
- [x] **P29 — Chemical Court Record v1** (`court_record.rs`, `casefile` command). Canonical, versioned
      `dsfb_chemical_engineering_casefile_v1/` bundle (exactly 11 files), per-episode claim-boundary
      badges, `RejectionReason` vocabulary, `non_claims.md`, deterministic `bundle_root`; `demo` emits
      one per dataset. Replay-inert (not in `canonical_replay_hash`). 4 dedicated tests.
- [x] **P30 — documentation-accuracy pass.** Corrected the edge `no_std` mislabel (std-only, unsafe-
      forbidden, dependency-light); 57-vs-14 detector clarification; reconciled dataset Table 4 to the
      actual committed 20; corpus count 21→20; README Court Record / UNKNOWN-action sections + change log.
- [x] **P31 — paper: Operator Evaluation Protocol + honesty fixes.** New `sec:opeval`; SWaT synthetic-
      vs-real provenance reconciled; balance-witness false-positive rates added (SWaT-T101 4.4%,
      BATADAL-T1 0.1%); negative TEP delays IDV(13)/IDV(14) owned; "no distributional assumption"
      softened to its defensible core + Gauss–Markov/Gaussian-MLE conflation fixed. Paper → 36 pages.
- [x] **P32 — historian replay evaluation.** `load_historian_csv` accepts the richer long-format schema
      (`setpoint`/`manipulated_variable`/`controller_mode` → derived per-tag witness columns; `unit`
      provenance; bare schema unchanged); `run_historian` emits the Court Record + a balance witness when
      a roles sidecar exists; synthetic tank-historian fixture (`scripts/gen_historian_fixture.py`).
      Fixed a real NaN-sort panic in `linalg::median` (filters non-finite before sorting).
- [x] **P33 (pt1) — reproducibility + safety hardening.** `rust-toolchain.toml` pin (1.94.1); frozen
      golden pipeline-replay-hash gate (`edge/tests/golden_replay.rs`); `cuda #![deny(unsafe_code)]`
      (+`#[allow]` on `ffi`); `replay_deterministic` doc corrected.
- [x] **P33 (pt2) — NaN-safety + provenance gate + tests.** `hashing.rs` `f64q` out-of-range values hash
      raw bits (no large-magnitude collision; in-range encoding unchanged → frozen digests identical);
      `detectors.rs` KS/kNN/sensor-bias sorts filter non-finite (NaN no longer panics the total-order
      sort); `datasets.rs` `verify_manifest_sha256` — Rust-side dataset-provenance gate, run in `run_demo`
      (aborts on a slice/hash mismatch; "20 verified"); `balance.rs` witness unit tests; cuda `Passport`
      now carries `atlas_hash` (binds each case file to its authority; NOT in `evidence_root`, so roots
      stay byte-identical).
- [x] **P34 — provenance accuracy at the source.** `datasets::manifest_kinds()` + `data_kind_tag()`;
      `discover_datasets` stamps each slice's MANIFEST `kind` instead of a hardcoded `measured/slice`, so
      the nine simulations now read `simulation/slice` in `metrics.csv`, every `casefile.json` `data_kind`,
      the operator HTML, `manifest.json`, and `result.json` (the honest labels previously lived only in
      MANIFEST.toml + the paper). New `provenance_gate.rs` (2 tests): discovered kind ≡ MANIFEST kind.
      Replay-inert (`data_kind` ∉ `canonical_replay_hash`); `bundle_root` changes for the nine sims (correct).
- [x] **P35 — balance-witness applicability criterion → its own disclosure (additive).** New paper
      `\section{Balance-witness applicability criterion}` (`sec:balancecriterion`) + an abstract sentence;
      new `docs/balance_witness_criterion.md` (the closure-gate criterion, the witness math per `balance.rs`
      arm, the positive/negative evidence, a worked example). The residual-grammar apparatus is unchanged.
- [x] **P36 — SWaT scope-stratified recall (criterion confirmation).** `scripts/swat_scope_recall.py`
      classifies the 35 labelled windows against the **official iTrust attack list**: 5 attacks touch a
      T101 balance term → **within-scope recall 5/5 = 100%**, 73% out-of-scope specificity, 4.4% FP. The
      apparent "13/35" is the criterion confirmed, not a miss rate. Attack list iTrust-licensed, not
      redistributed (only the recipe + result are committed).
- [x] **P37 — incumbent head-to-head worked example (TEP IDV-1, additive).**
      `scripts/head_to_head_tep_idv1.py` derives, from the deterministic run, incumbent practice
      (**10,041** breach-steps / **102** ISA-18.2 activations / MSPC contribution-plot top variables) vs
      the Court Record (**6** episodes — 1674×/17× triage reduction — claim-boundary badges, **15** logged
      `QUORUM_NOT_MET` rejections, evidence/bundle roots). New paper `sec:headtohead` + `tab:headtohead`;
      `docs/head_to_head_tep_idv1.md`. A demonstration of the recorded artifact, not a superiority claim.
- [x] **P38 — correctness + determinism hardening (additive; all hashes unchanged).**
      (1) `pipeline::analyze`/`timelines_for` now guard `n_samples < 2` (the `clamp(2,n)` previously
      panicked on a 0/1-row matrix) and return a well-formed empty result; `DataMatrix::new`'s
      rectangularity contract documented; regression test added.
      (2) cuda `gpu_cross_verified` flag added to `EvidenceRun`/`ExecutionAttestation`/manifest: `true`
      only when a real GPU run matched the CPU reference, `false` on the CPU-only path — so
      `cross_backend_verified` (self-consistent) is no longer mistaken for genuine two-backend agreement.
      (3) the GPU-decode `as u32` downcast of the lane counts is now a checked `u32::try_from(..).ok()?`
      (a corrupt GPU value falls back to the CPU reference rather than truncating silently).
      (4) new frozen `tests/golden_evidence.rs` pins the CPU `lane_evidence_cpu` digest + `evidence_root`
      on a bit-portable dyadic input (catches a contract drift without a GPU).
      (5) the `RejectionReason` vocabulary is split into the one emitted reason + a documented
      `RESERVED` group (with the context that will emit each), and `map_rejection` is de-duplicated.
      `evidence_root`/`canonical_replay_hash`/`atlas`/`corpus` hashes all unchanged.
- [x] **P39 — paper accuracy + noise-floor preservation disclosure (additive).** (a) Softened the
      causal claim in `sec:pure-determinism`: the field's move to explicitly probabilistic methods is
      attributed to nonlinearity / uncertainty-quantification / small-data regularisation, with
      noise-distribution sensitivity named as *one thread, not the sole cause* (was "exactly why the
      field moved"). (b) Hoisted + expanded the noise-floor property into its own
      `\section{Noise-floor preservation: preserved evidence, not detection}` (`sec:noisefloor`): the
      raw-IEEE-754-bits mechanism + an explicit, mechanical **bound** (the inference path never reads the
      raw bits; identical episodes for sub-grid differences; recoverable-not-diagnostic). Matching README
      section + `docs/noise_floor_preservation.md`. Paper → 39 pages.
- [x] **P40 — F-signature reduction-to-practice: F1 + F9 catalogued→executed (additive breadth;
      re-freezes `atlas_hash_v1`).** Two new deterministic control-loop demonstrators
      (`scripts/gen_instrumented.py` → `data/instrumented/valve_{stiction,hunting}_instrumented.csv`,
      PV/OP only, no roles sidecar) exhibit the F1 stick-slip and F9 fixed-period limit-cycle motifs;
      the new gate `edge/tests/fault_demonstrators.rs` runs them through `pipeline::analyze` and asserts
      a single fused episode forms at the labelled onset (delay≈0, quiet baseline) and the signatures'
      named detectors (`spectral_entropy_spe`, `page_hinkley_spe`, `cusum_spe`) fire densely. F1 and F9
      promoted `Catalogued`→`Executed` in `fault_signature.rs` (now **4 executed** — F1, F6, F7, F9 — of
      12). `atlas_hash_v1` re-frozen `9932e8b9…`→`d5ab6a43…` (implementation_status is sealed into
      `fault_signatures_hash_v1`); the cuda Passport `atlas_hash` tracks it but the CUDA `evidence_root`
      is **unchanged** (20/20 replay OK, passport excluded from the root). Paper atlas §+ README updated.
- [x] **P41 — mock forensic incident + milestone-gated evaluation protocol (additive; fictional, no
      agencies).** New fully-synthetic, FICTIONAL incident fixture (`scripts/gen_incident_fixture.py` →
      `data/historian/northgate_r101_incident.csv` + roles): "Northgate Specialty Chemicals" R-101
      cooling-water surge-tank TK-110 with a mid-batch LIT-110 sensor spoof. Run through `historian`, the
      balance witness catches it (closure $0\!\to\!8$; NE~107 `Failure` on exactly the 36 post-onset
      samples) while the statistical bank's sub-quorum candidate `[54..89]` is recorded as a
      `QUORUM_NOT_MET` rejection — the doctrine on one screen. `docs/forensic_incident_walkthrough.md`
      is the 3-minute read (evidence/bundle roots, replay-verified). Paper `sec:opeval` gains a
      milestone-gated protocol (`tab:milestones`, M0–M3 with replay-checkable go/no-go gates anchored to
      reported metrics), a minimum-data spec, and a baseline arm. README pointer added. Paper → 40 pages.
- [x] **P42 — final paper QA / legendary pass (correctness/consistency only; breadth preserved).**
      Rebuilt + audited the PDF end-to-end: **0 undefined cites/refs, 0 overfull boxes** (this "0 overfull"
      was later found in P48 to be a `build_paper.sh` grep-filter artifact — 1 real overfull existed and was
      fixed in P48, which added a direct `.log` audit; underfull hboxes + enumitem warnings remain, are
      benign, and are **not** claimed); rendered-page inspection found **0 `[?]`/`[??]` markers**, no mojibake, no truncated
      table cells (the new `tab:headtohead` p21 and `tab:milestones` p24 render within margins). Stale-number
      sweep reconciled every count this batch moved — page count (40), the **4-of-12 executed** fault
      signatures, the head-to-head `1674×` (matches `tab:compression`), SWaT `4.4%`/`5/5`, and the
      10/9/1 dataset provenance (MANIFEST tally == paper claim). Every `\cref{sec:…}` resolves; no
      contradictions; results trace to committed artifacts. README change-log extended to cover P34–P41.
      No code or paper-source change needed beyond doc sync — the artifact was already clean.
- [x] **P43 — CUDA device-SHA-256 padding fix + GPU↔CPU parity gate (panel-driven; correctness).**
      A 5-specialist panel review surfaced one concrete defect: the device SHA-256 `final()`
      (`cuda/sha256.cuh`) inflated the encoded message length by 512 bits in the two-block padding case
      — pre-pad `buflen` 56..63, i.e. any lane whose `40·n_samples` stream is ≡ 56 (mod 64) ⇔
      `n_samples ≡ 3 (mod 8)` — because the padding bytes were fed through `update`, which adds 512 per
      completed block. On those inputs the GPU digest diverged from the host `sha2` crate (the court then
      fails closed to the CPU reference, so **no wrong evidence was ever sealed**, but the "GPU≡CPU on
      all 20" claim held only because the 20 datasets' sample counts dodge that residue class). Fixed by
      encoding a **pre-pad length snapshot**; verified at the algorithm level against `hashlib` over the
      empty string, FIPS "abc", the 119/120/121-byte edge cases, and `40·n` for n=1..40 (0 mismatches,
      incl. the previously-broken n≡3). New feature-gated `tests/gpu_cpu_parity.rs` exercises the residue
      classes (over-representing n≡3) on a CUDA host. Paper `sec:cuda` cross-backend paragraph updated:
      byte-exactness is now correct-by-construction for all message lengths and gate-enforced.
- [x] **P44 — energy-balance disclosure tiering + differentiation-noise-floor bound (panel-driven;
      disclosure accuracy).** The panel noted the energy-balance closures do not close to ≈0: they
      differentiate a measured temperature, and the gain `ρc_pV/Δt` amplifies thermocouple noise `σ_T`
      into a large baseline band (~250 kJ/min for the CSTR), so the "7.5×/251×" energy ratios are over a
      *noisy* baseline, while only the mass/volume balances close to ≈0. Tiered the claim in
      `docs/balance_witness_criterion.md` (new §1.1) and the paper `sec:operator` balance paragraph:
      mass closes ≈0 when fully metered; energy carries a structural offset; the witness keys on the
      sustained *relative* shift. Converted this into a **quantitative applicability bound** atop the
      qualitative gate — an energy witness resolves a fault only if its sustained shift exceeds the
      differentiation-noise floor `≈ ρc_pV·σ_T/Δt` — which also explains the CSTR slow-drift detection
      delay honestly. No code change; replay-inert.
- [x] **P45 — paper construct clarity (panel-driven; construct-definition + priority).** (a) Added a
      formal **Residual semiotics** definition as the lead of `sec:formal` (the title-level construct was
      previously used but never defined). (b) Fixed the **slew mislabel**: "second-order slew" →
      "first-order rate-of-change slew" in the abstract, the `sec:formal` definition, and `limitations.tex`
      — the formula `σ_k=(r_k−r_{k-1})/Δt` (`eq:slew`) is a first difference; drift and slew are both
      first-order operators on `r`. (c) Defined the previously-undefined neologisms inline
      (`sec:pure-determinism`): **densor** (the deterministic-evidence analogue of a tensor), **densorial**,
      **tekmeric** (Gr. *tekmērion*, evidence/proof). (d) New **construct glossary** appendix
      (`app:glossary`): every coined/load-bearing term defined once with a pointer to its defining
      section. Paper → 41 pages. No code change; replay-inert.

- [x] **P46 — Rust + `bundle_root` hardening (panel-driven; determinism + robustness).** (a)
      Signed-zero-canonical CSV formatting (`court_record::fmt6`, `report::fmt`, `figures.rs`): a value
      that renders `"-0.000000"` is emitted `"0.000000"`, so a near-zero `signed_margin` whose sign flips
      across platforms cannot change the bytes the `bundle_root` SHA-256s (the sealed `evidence_root`/
      `replay_hash` already used the quantised `f64q` path); every non-zero value is byte-unchanged.
      (b) New frozen **`bundle_root` golden** gate (`court_record.rs` test, `tennessee_eastman_idv01`) —
      the cross-run analogue of `golden_replay`/`atlas_hash`. (c) `MannKendall` variance now casts factors
      to f64 before multiplying (no `usize` overflow on a large window; bit-identical for in-range n).
      (d) `out()` threshold floor is now `thr.max(1e-12)` (genuinely ≥ 1e-12 incl. non-positive/NaN, not a
      magnitude clamp — matches the doc invariant). (e) `load_csv_slice` NaN-pads ragged rows and uses a
      `4.min(n)` baseline floor (no `clamp(4,n)` panic for n<4), so a malformed/tiny CSV degrades
      gracefully instead of panicking; regression test added. All replay-inert: verify-replay 6/6,
      golden_replay green, bundle_root golden pinned.
- [x] **P47 — reproducibility digests for the balance witnesses (panel-driven; enablement + priority).**
      `scripts/verify_reproducibility.py` regenerates each balance-witness trace via the committed CLI and
      hashes a **platform-portable canonical form** (per row `time_index:round(residual·1e6):grammar_state`,
      signed zero normalised — independent of float text formatting), checked against committed
      `data/instrumented/EXPECTED_DIGESTS.toml`. The four **synthetic** demonstrators are reproducible by
      anyone; the two **gated** ones (SWaT T101, BATADAL T1) verify only for a holder of the licensed data
      (skipped otherwise) — so the headline real-data results become independently byte-checkable **without
      redistributing a single byte** (only the hashes are committed). Converts "trust the recipe" into
      "byte-confirm your run". Wired into `app:repro` + the README reproducibility section. No code change.
- [x] **P48 — paper accuracy + honest overfull detection (2nd-panel re-review; correctness + honesty).**
      A 2nd 5-discipline read-only panel (composite ≈9.1–9.3, up from ≈8.6; the CUDA reviewer independently
      proved the P43 SHA fix correct across all 201 message lengths) surfaced one genuine honesty finding:
      the prior **"0 overfull"** claim was an artifact of `build_paper.sh` grep-filtering `Overfull` out of
      its own stdout — the real `.log` had **1 Overfull \hbox (9.51pt)** in `tab:milestones` (P41).
      Fixed: (a) `build_paper.sh` now audits the actual `.log` for Overfull/Underfull/undefined; (b) the
      `tab:milestones` Milestone column widened (0.085→0.11\linewidth) → genuinely **0 Overfull**;
      (c) the **energy-balance applicability budget corrected** — `ρc_pV·σ_T/Δt ≈ 5×10³` J/min (thousands,
      a valid *lower* bound), with the dominant `~2.5×10⁵` J/min model-form/discretization term named, and
      a **clause (iii) model-fidelity** added to the closure-gate criterion (paper `sec:operator` +
      `sec:balancecriterion` + `docs/balance_witness_criterion.md` §1.1/§2); (d) small fixes: three-tank
      "≈0"→"small Torricelli offset (3.4)", SWaT "36 vs 35" reconciled, "accelerating slew"→"sharp slew
      spike", Johansson (2000) quadruple-tank citation added. Paper → 42 pages. No code change; replay-inert.

- [x] **P49 — SHA-256 regression gate on every machine (panel-driven; determinism coverage).** The P43
      parity gate is `#[cfg(feature="cuda")]` + needs a GPU, so it was dormant on CPU-only CI. Added a
      **host-side** `cuda/tests/sha256_host_parity.rs`: a faithful Rust port of the fixed `sha256.cuh`
      `update`/`final` (pre-pad length snapshot) asserted equal to `sha2::Sha256` for the empty string,
      "abc", lengths {55,56,63,64,119,120,121,184}, and `40·n` for `n ≡ 3 (mod 8)` {3,11,27,43} — the
      previously-broken two-block class now fails on *any* box if the padding regresses. Also added a
      second `golden_evidence` case at **N_SAMPLES=43** (`40·43 ≡ 56 mod 64`) so the CPU golden suite
      itself exercises the two-block residue (the N=96 case is ≡0 mod 8 and dodged it). No source change;
      replay-inert.
- [x] **P50 — extend byte-verification to the whole Court Record + metric definitions (panel-driven;
      enablement + priority, highest reproducibility leverage).** Previously only the balance-witness
      traces, the 6 synthetic replay hashes, and the single idv01 `bundle_root` were committed-digest
      checkable. Added (1) `data/EXPECTED_BUNDLE_ROOTS.toml` — the frozen `bundle_root` + `evidence_root`
      for **all 20** public datasets (the bundle is timestamp/path-free → deterministic; verified casefile
      ≡ demo for both roots), plus a `verify_reproducibility.py --bundles` mode that regenerates (via
      `demo`) and byte-checks every one (all 20 OK) — so anyone with the committed slices can confirm the
      *entire operator artifact*, not just the witnesses; (2) an explicit **fired-after-onset disposition
      guard** in the witness verifier (asserts the *result*, not only trace-byte reproduction); (3)
      `data/METRICS_DEFINITIONS.toml` — the denominators/protocols behind every reported rate (SWaT 30 s
      blocks 119/2700, BATADAL hourly 5/3958, the 10,041 breach-steps / 102 ISA-18.2 activations defs), so
      a non-licensee can reconstruct the arithmetic and the measurement protocol is itself disclosed.
      Wired into `app:repro`. No Rust source change; replay-inert.
- [x] **P51 — F3+F8 reduction-to-practice + Rust polish (panel-driven; additive breadth + robustness).**
      Promoted **F3 heat-transfer fouling** + **F8 controller-compensation masking** Catalogued→Executed
      via two new synthetic demonstrators (`gen_instrumented.py` → `heat_fouling_instrumented`,
      `controller_masking_instrumented`) gated by `edge/tests/fault_demonstrators.rs`: F3's slow monotone
      fouling drift fires `ewma_spe` densely (delay≈23, honest for a gradual fault); F8's masked latent
      change (CV held, MV ramps) fires `pca_t2`+`ewma_spe` densely before any raw CV breach. Executed
      fault signatures now **6 of 12** (F1,F3,F6,F7,F8,F9); `atlas_hash_v1` re-frozen
      `d5ab6a43…`→`3c243779…` (CUDA `evidence_root` 20/20 and the edge `bundle_root`s unaffected — the
      edge court record does not embed `atlas_hash`, re-verified). Paper `sec:atlas` + README bumped to
      6-of-12. Rust polish (replay-inert): cuda mismatch-path attestation now reports the CPU backend it
      actually sealed (not a GPU throughput); `fuse` tie-break doc reconciled to the real `max_by_key`
      (largest-Ord-wins) semantics; `bench.rs` sort uses `unwrap_or(Equal)`.
- [x] **P52 — public-release wording + consistency (panel-driven; disclosure accuracy).** Release-readiness
      pass over public-facing wording, replay-inert (docs/comments/paper only; one Rust doc-comment).
      (a) Neutralized two stale wording lines in the P8/P27 entries above. (b) Fixed the stale
      `corpus/src/hashing.rs:1` doc-comment "molecular corpus"→"soft-sensor data corpus" (the crate is a
      soft-sensor dataset catalogue; the molecular corpus is a *future* companion, never this artifact).
      (c) Added the explicit "PubChem-scale molecular densors are a future companion corpus, not part of
      this artifact" clarifier to README (corpus row) and paper (`sec:` crate paragraph) so the soft-sensor
      scope is unambiguous. (d) Reframed the paper front-matter to the **four-crate** architecture
      (crates.io: edge/cuda · authority: atlas/corpus) without overflowing the title block (`.log`-audited:
      0 overfull). (e) Corrected the inaccurate P42 "0 overfull/underfull/0 warnings" claim → honest "0
      overfull, 0 undefined" with the P48 grep-filter-artifact note (underfull + enumitem warnings remain,
      benign, **not** claimed). (f) Framed the CUDA evidence kernel as **baseline deterministic sealing, not
      final throughput**, forward-referencing the digest-equivalence law (paper `sec:perf` + README), so
      the one-thread-per-lane mapping reads as an auditability choice, not a performance ceiling. Paper
      stays 42 pages, 0 undefined / 0 overfull (`.log`-audited); workspace tests green; `verify-replay` 6/6
      byte-identical (replay-inert as expected).
- [x] **P53 — DatasetLicenseConfidenceV1: full provenance-classification tiers (4th-panel; disclosure
      accuracy). `corpus_hash_v1` deliberately re-frozen.** Added four orthogonal, hash-sealed *honest
      disclosure* axes to every `SoftSensorDatasetRecordV1`: `LicenseConfidence` (ExplicitOpen /
      ExplicitCopyleft / StatedNeedsVerification / ResearchUseCustomary / AgreementGoverned),
      `AccessConfidence` (OpenConfirmed / OpenMirrorUnverified / AccountRequired / GeneratedByCode /
      AgreementRequired), `RedistributionPolicy` (UpstreamPermitsAttribution / UpstreamCopyleftShareAlike /
      UpstreamVerifyBeforeRedistribution / ProhibitedByAgreement — documents what a *downstream* user must
      respect; this crate still ships **no** bytes), and `SourceAuthorityKind` (DoiArchive /
      CuratedMlRepository / PackageDistribution / SimulatorCodebase / GovernedTestbed / CommunityUpload /
      AuthorOrVendorHost). All 20 records classified with an inline reasoning comment each, derived from the
      cited licence + URL host + venue (confidence/policy, **not** legal opinions or quality claims).
      Counts: licence **8** explicit-open / 1 copyleft / 5 stated-verify / 4 research-use / 2 agreement;
      access **12** open-confirmed / 4 open-mirror / 1 account / 1 code-gen / 2 agreement; redistribution
      8/1/9/2; authority 3 DOI / 8 curated-ML / 1 package / 1 simulator / 2 testbed / 1 community / 4
      vendor-host. New `census()` + `ClassificationCensus` (counts every tier, never fails); a gate asserts
      each axis **partitions all 20 records** (no gaps) + a SWaT/WADI agreement-prohibited invariant
      (corpus tests 5→7). Tiers folded into the canonical preimage → `corpus_hash_v1` re-frozen
      `1a5f4c31…`→`7ce33a2e…` (deliberate, documented; both execution backends print the new hash + the
      census). README "Dataset provenance classification" table + paper `Datasets` § disclose the counts.
      Edge `bundle_root`s / CUDA `evidence_root` unaffected (corpus is off the residual/replay path);
      `verify-replay` 6/6 byte-identical; paper 42pp 0/0.
- [x] **P70 — CUDA evidence-kernel V2 (GPU-measured; pulled forward out of sequence at maintainer's
      direction; P54–P69 resume next).** Built `DigestEquivalenceHarnessV1` (`cuda/src/digest_equivalence.rs`):
      the law that any kernel must reproduce the CPU reference's per-lane `LaneEvidence` + Merkle root +
      `evidence_root` + replay byte-for-byte, over an adversarial battery (SHA-padding `n≡3 mod 8`,
      warp/block edges, NaN/inf, all-zero). Two optimizations behind it, all measured on a real RTX 4080
      SUPER via Nsight (`ncu`/`nsys`; the dev sandbox builds+runs CUDA, only `ncu` counters are root-gated):
      **V2-A** lane-batching (`evidence_kernel_batched`, digest-IDENTICAL — shared `__device__` routine;
      occupancy 8%→52%), **V2-B** segment-parallel (`evidence_kernel_v2_segmented` + `lane_evidence_v2_cpu`,
      an opt-in `evidence_root_v2` Merkle-segment format with a `DRIFT_WINDOW` halo warm-up;
      `SEGMENT_SIZE=256`). Deep 1024×8192 case **29.75 → 1.65 ms ≈ 18×** kernel (SM 2.6%→49%) via V2-B +
      digest-preserving SHA micro-opts (`__funnelshift_r` rotate + unrolled rounds; chunked SHA buffering);
      **pinned H2D** (`cudaHostRegister`) ~13.5→26.7 GB/s, **≈8× end-to-end**. A `cudaMemcpyAsync`
      multi-stream overlap was built, **measured 2.7× slower** (chunking collapses occupancy), and reverted
      — disclosed as a negative result. All gated byte-exact (golden_evidence + gpu_cpu_parity unchanged;
      V1's frozen roots intact). Disclosed in paper `sec:perf` (V2 subsection), `docs/cuda_evidence_kernel_v2_design.md`,
      README, and `cuda/reports/NSIGHT_SUMMARY.md`. The V1 kernel remains the canonical reference.
- [x] **P54 — verification report + `ArtifactCompletenessCourtV1` (4th-panel; enablement + disclosure
      accuracy).** New std-only edge module `completeness.rs` + `completeness-court` CLI command + gate
      (`edge/tests/completeness_court.rs`): a deterministic court that the committed **artifact graph** is
      complete + mutually consistent — MANIFEST `dataset_count` = 20 blocks, every dataset has a 64-hex
      SHA-256, every `EXPECTED_BUNDLE_ROOTS` entry has 64-hex `bundle_root`+`evidence_root`, the two tables
      name the **same** 20 datasets, provenance `kind` tags agree (`data_kind_tag`), atlas + corpus
      `validate()` pass with well-formed hashes, the corpus census **partitions all 20 records** on each of
      the 4 P53 axes, and the metrics-protocol sections are present. Emits a hash-sealed pass/fail report
      (`report_hash` via `CanonicalHasher`); **verdict COMPLETE (9 pass, 0 fail)**, `report_hash 98821ad4…`.
      Scoped honestly: it asserts the machine-checkable graph from committed files, **not** PDF/prose-count
      reconciliation (that stays the manual P71 sweep) — disclosed in the module doc + the report. New
      `reports/verification_report.md` records the capture host (rustc 1.94.1, CachyOS, CUDA 13.2, RTX 4080
      SUPER), verbatim re-runnable commands, test totals (**55** workspace + **13** `--features cuda`),
      frozen `atlas_hash_v1`/`corpus_hash_v1`, replay 6/6, 20/20 bundle roots, paper 42pp 0/0, and the court
      verdict. Workspace green; completeness gate passes with and without the `soft-sensor-corpus` feature;
      replay-inert (no change to any sealed artifact).
- [x] **P55 — EdgeCoreProfileV1 design doc (`docs/edge_core_profile.md`; enablement disclosure, design
      only).** Documents how the DSFB core would run `no_std` / no-heap / fixed-point on an MCU
      (RP2040/Cortex-M/QEMU route), grounded honestly in the **existence proof** already in the repo: the
      CUDA `evidence.rs` contract is *already* pure integer fixed-point (`SCALE=1e6`, fixed `DRIFT_WINDOW=16`
      integer ring, integer drift/slew/exceedance, O(window) state) and gated byte-exact against the CPU
      reference — so the embedded profile is an *extraction* of a proven shape, not a new algorithm. Covers
      the core-vs-shell split (grammar/detectors/fusion/heuristics core vs CSV/JSON/figures shell), the
      `heapless` + `u16`-channel-index + `&'static` atlas changes, a single-digit-KB SRAM budget, the
      host-side equivalence gate (same `DigestEquivalenceHarnessV1` discipline, GPU as reference), and
      honest bounds (heavy multivariate detectors stay on the shell; the `no_std` core crate is **not**
      built). Docs-only; nothing else touched.
- [x] **P56 — PubChem molecular-corpus companion design note (`docs/molecular_corpus_companion.md`;
      boundary + breadth disclosure, design only).** Completes the P52 forward-reference: the shipped
      `corpus` is and remains the **soft-sensor dataset catalogue**; a future, separate
      `dsfb-chemical-engineering-molecular-corpus` (its own frozen `molecular_corpus_hash_v1`) is described
      — `CompoundDensorV1` (cheap molecular descriptors → hard-to-measure target, the soft-sensor framing
      lifted to chemistry), provenance-bound PubChem/ChEMBL shards classified on the **same four P53 tiers**,
      descriptor admissibility envelopes, fingerprint motifs, scaffold + **confuser dockets**, and H7–H16
      molecular heuristics extending H1–H6. Mirrors the existing discipline verbatim (no_std, `&'static`,
      mandatory `SourceRef`, **no compound bytes vendored**, deterministic replay, authority separation,
      no tox/hazard/accuracy claims). Explicit boundary: **NOT built, NOT part of this artifact, does NOT
      mutate the soft-sensor corpus.** README PubChem clarifier now points to the note. Docs-only.
- [x] **P57 — RegimeEnvelopeV1 + ChemicalAuthoritySeparationLawV1 (PART B begins: formalize precursors →
      named, hash-sealed V1 authority objects).** Two new edge modules, additive + replay-inert.
      **`regime_envelope.rs::RegimeEnvelopeV1`** formalizes the per-regime `calibrate_regime_envelope`
      output into a citable object: `regime_id`/`phase_id`/`var_group`/`family`/`bounds`
      (`AdmissibilityEnvelope`)/`transition_policy` (RelaxNeverTighten | GlobalDefaultFallback)/
      `calibration` provenance/**`provenance_hash`** (CanonicalHasher `f64q` over metadata+bounds+how-derived).
      Self-verifying (`verify()` re-derives the seal from its own fields), with a `relaxes_default()`
      structural invariant check (the bounds are a superset of the global default on every axis — a
      tightened, invalid envelope is caught independently of the hash). `calibrate_and_seal()` is the
      formalization entry point. **`authority_law.rs::ChemicalAuthoritySeparationLawV1`** names the five
      execution↔authority separation rules as an `&'static` doctrine + an executable `check_separation_law()`:
      R1 (authority pure/no_std) + R2 (one-way dep) are compile-time structural (enforced-by-construction);
      R3 (executed ⊆ catalogued — re-checked against `build_bank`/`canonical_bank` vs the atlas), R4
      (deterministic single-source authority hash), R5 (`&'static const` ⇒ stable hash) are re-checked at
      runtime; verdict **all 5 hold**. Added `PartialEq` to `AdmissibilityEnvelope` (derive-only,
      replay-inert). Tests: edge lib 14→18; workspace green; `verify-replay` 6/6 byte-identical; both
      modules clippy-clean.
- [x] **P58 — ChemometricPassportV1 (per-detector) + ResidualProvenanceGraphV1 (DOT/JSON).** Two new edge
      modules, additive + replay-inert. **`passport.rs::ChemometricPassportV1`** is the per-detector
      companion to the case-level CUDA passport: it pins, by SHA-256, the `baseline_window_hash` (rows the
      detector fit on), `input_matrix_hash` (matrix scored), and `output_hash` (the emitted
      `DetectorOutput` stream), records the disclosed `threshold_policy`/`normalization`/`missingness`,
      and seals all of it under `passport_hash` (self-verifying; `build()` computes the component hashes
      from real data). **`provenance_graph.rs::ResidualProvenanceGraphV1`** promotes the flat
      `residual_provenance.csv` ledger to an explicit hash-sealed **graph** (`raw → residual → detector →
      episode → label → court_root`) emittable as Graphviz **DOT** + **JSON**, via a `ProvenanceGraphBuilder`
      (`node`/`link`/`seal`), with a `graph_hash` and `verify()`. Tests: edge lib 18→24; workspace green;
      `verify-replay` 6/6 byte-identical; both modules clippy-clean.
- [x] **P59 — DetectorDisagreementForensicsV1 + NegativeWitnessV1 (`disagreement.rs`).** Promotes the
      silent half of the evidence to first-class objects. **`NegativeWitnessV1`** records a silent
      detector as evidence: `detector_id`/`family`/`why_silent` (NominalResidual | BelowThreshold |
      NotApplicable | NoInput)/`subspace_implication` (what the silence rules out — bounded, advisory).
      **`DetectorDisagreementForensicsV1`** is the full per-episode report: `participating` (firing),
      `silent` (negative witnesses = roster − firing), `contradicting` (fired but disagree with the
      dominant motif), `witness_diversity_score` (distinct firing families ÷ total families), the carried
      `disagreement_entropy`, sealed by `forensics_hash` (self-verifying `build()`/`verify()`). Additive,
      replay-inert. edge lib 24→26; workspace green; `verify-replay` 6/6; clippy-clean.
- [x] **P60 — UnknownTaxonomyV1 (7-class, authority) + ConfuserDocketV1 (per-episode, edge). `atlas_hash_v1`
      deliberately re-frozen.** **UnknownTaxonomyV1**: new atlas module `unknown_taxonomy.rs` — a curated,
      hash-sealed 7-class taxonomy of *why an episode is left unknown* (short_transient, detector_conflict,
      out_of_regime_envelope, residual_degeneracy, missing_process_context, non_stationary_baseline,
      insufficient_witness_diversity), each with a name/description/advisory disposition (the 5 reviewer
      classes added to 2 retained = the clean 7). Folded into `atlas_hash_v1` via a new
      `unknown_taxonomy_hash_v1()` component + a validation gate (exactly 7, well-formed, unique ids) →
      **`atlas_hash_v1` re-frozen `3c243779…`→`8f921f58…`** (deliberate; the atlas test pin + the
      verification report updated). CUDA `evidence_root` 20/20 + edge `bundle_root`s **unaffected**
      (passport excluded from the root; edge court record doesn't embed `atlas_hash`; golden_evidence +
      gpu_cpu_parity green). **ConfuserDocketV1**: new edge module `confuser.rs` — promotes the static atlas
      `confuser_faults` field to a per-episode emitted docket (`for_fault(episode_ref, fault_id)` reads the
      atlas signature, emits matched fault + discriminating signature + the confusers ruled out, sealed by
      `docket_hash`; cites only catalogued confusers, never invents). atlas tests green; edge lib 26→29;
      `verify-replay` 6/6; clippy-clean.
- [x] **P61 — BalanceWitnessV1 (+ stoichiometric/yield/selectivity residual kinds) + SoftSensorWitnessV1.**
      Two new edge modules, additive + replay-inert. **`balance_witness.rs`**: `BalanceWitnessV1` turns the
      `balance.rs` closure residual into a hash-sealed witness record (kind / balance equation / units /
      n_samples / peak |residual| / residual_hash / witness_hash; self-verifying), and adds three
      **reaction-chemistry residual kinds** the physical balances didn't cover — `Stoichiometric`
      (`product − ratio·reactant`), `Yield` (`theoretical − actual`), `Selectivity`
      (`target − desired/total`, zero-conversion-safe) — each a pure helper. **`softsensor.rs`**:
      `SoftSensorWitnessV1` makes a soft sensor's output a first-class witness — `measured` / `prediction`
      / `residual` (= measured−prediction, the DSFB-admissible error) / `interval_half_width` /
      `model_family` (disclosed, no probabilistic claim) / `training_scope_hash` (seals what it was fit
      on) — sealed by `witness_hash`. edge lib 29→34; workspace green; `verify-replay` 6/6; clippy-clean.
- [x] **P62 — ControllerMaskingHeuristicV2 + ValveStictionWitnessV1 + AlarmFloodCompressionReportV1 (HTML)
      + OperatorIncidentHTMLV1 (9-question).** Two new edge modules, additive + replay-inert.
      **`controller_masking.rs`**: `ControllerMaskingHeuristicV2` sharpens H6 to a **four-signal
      conjunction** (PV-stable ∧ MV-drift ∧ effort-rising ∧ residual-energy-rising — masking suspected
      only when all 4 hold, separating it from benign tuning), with a `from_streams()` deriver + sealed
      `verdict_hash`; `ValveStictionWitnessV1` formalizes F1's motif over four signatures (sawtooth /
      deadband / limit-cycle / PV-lag; suspected at ≥2), sealed. **`operator_reports.rs`**:
      `AlarmFloodCompressionReportV1` (ISA-18.2 before→after raw-alarms→episodes + ratio, with the
      **`lost_evidence=0` / `recoverable=true`** invariants made explicit, emits HTML) +
      `OperatorIncidentHTMLV1` (the nine-question one-page incident report — what/when/where/which
      detectors/candidate/ruled-out/severity/check/evidence-root — sealed + HTML). edge lib 34→39;
      workspace green; `verify-replay` 6/6; clippy-clean.
- [x] **P63 — SensitivitySweepReceiptV1 + AblationCourtV1 (PART C begins: new evidence objects).** Two new
      edge modules, additive + replay-inert. **`sweep.rs`**: `SensitivitySweepReceiptV1` runs a
      deterministic **Cartesian threshold-grid** over named axes (envelope k, quorum min_families, drift
      window, …), evaluates a pure metric at every point in fixed row-major order, and seals the grid +
      results; headline `metric_range` (max−min) discloses robustness-to-thresholds. **`ablation.rs`**:
      `AblationCourtV1` runs a component-ablation arm per disabled component vs the full-pipeline baseline,
      records each arm's `delta_vs_full`, identifies the `most_load_bearing` component, sealed by
      `court_hash`. Both closure-driven (pure evaluators → reproducible). edge lib 39→43; workspace green;
      `verify-replay` 6/6; clippy-clean.
- [x] **P64 — multi-unit demonstrator + ProcessTopologyGraphV1 + FaultPropagationWitnessV1 +
      ResidenceTimeAlignmentV1 + CausalNonClaimGraphV1.** Two new edge modules, additive + replay-inert;
      the feed→reactor→separator multi-unit demonstrator (declared residence times) is exercised in-test
      (synthetic — so the 20-dataset count / MANIFEST / completeness-court are untouched). **`topology.rs`**:
      `ProcessTopologyGraphV1` (units + residence-time-labelled flows; `is_upstream_of` reachability; DOT;
      sealed) + `ResidenceTimeAlignmentV1` (aligns upstream→downstream residuals by the declared
      residence lag, reports the at-lag Pearson correlation — **advisory, not causal**). **`propagation.rs`**:
      `FaultPropagationWitnessV1` (upstream/downstream onsets + observed-vs-declared lag consistency +
      a **mandatory non-causal disclaimer**) + `CausalNonClaimGraphV1` (the anti-overclaim object — edges
      carry precedence + topology, every DOT rendering prints `NO CAUSAL CLAIM` with dashed edges; the
      disclaimer is sealed into the hash so it cannot be stripped). A shared `NON_CAUSAL_DISCLAIMER`
      const is the single source of truth. edge lib 43→47; workspace green; `verify-replay` 6/6; clippy-clean.
- [x] **P65 — EpisodeShapeHashV1 + MotifNearestNeighborV1 + CrossRunEpisodeRecurrenceV1 +
      CaseLawPrecedentIndexV1 + FleetPlantComparisonV1 (`precedent.rs`; structural-similarity, advisory).**
      Five hash-sealed objects for comparing episodes structurally **without ever claiming identity**.
      `EpisodeShapeHashV1` = motif sequence + coarse feature vector + exact `shape_hash` + a Euclidean
      `distance`. `MotifNearestNeighborV1` = top-k nearest catalogued motifs (deterministic distance-then-id
      sort). `CrossRunEpisodeRecurrenceV1` = same shape recurring across runs within a tolerance.
      `CaseLawPrecedentIndexV1` = a shape-keyed index of past cases with `nearest_precedent` lookup.
      `FleetPlantComparisonV1` = dominant shapes shared across plants. Every result carries the shared
      `SIMILARITY_ADVISORY` ("retrieval hint, not identity, not causal"), sealed into the hash. edge lib
      47→51; workspace green; `verify-replay` 6/6; clippy-clean.
- [x] **P66 — ProcessNarrativeCompilerV1 + NoNarrativeHallucinationGateV1 (`narrative.rs`; flagship
      anti-hallucination).** A narrative compiler that is **not an LLM**: every sentence is a fixed
      deterministic template filled from structured fields, and every sentence carries the `anchor_hash`
      of the specific sealed evidence object it derived from (episode / label / witness / ruled-out /
      balance) — there is no free-text path, and the label template cannot emit a root-cause claim.
      `NoNarrativeHallucinationGateV1` mechanically proves the property: **every sentence's anchor is in
      the case's evidence-object hash set** — any un-anchored sentence (a hallucination) is surfaced and
      fails the gate. A passing narrative cannot contain a claim not backed by sealed evidence. edge lib
      51→53; workspace green; `verify-replay` 6/6; clippy-clean.
- [x] **P67 — HeuristicMigrationReceiptV1 + DetectorRegistryCompatibilityV1 + SemanticDiffReportV1
      (`migration.rs`; governed authority drift).** When the authority changes (a re-freeze like P60),
      these record exactly what moved. `SemanticDiffReportV1` diffs two record snapshots (id→content-hash)
      into added/removed/changed/unchanged. `DetectorRegistryCompatibilityV1` specialises that to the
      detector registry with a `backward_compatible` verdict (true iff nothing was *removed* — added/
      changed are allowed). `HeuristicMigrationReceiptV1` is a per-heuristic receipt (from/to version +
      hashes + disclosed changed fields; `changed = from_hash != to_hash`). All hash-sealed + self-
      verifying; work on any versioned record set. edge lib 53→56; workspace green; `verify-replay` 6/6;
      clippy-clean.
- [x] **P68 — RecipeTransitionGuardV1 + MaintenanceEventOverlayV1 + OperatorAnnotationLedgerV1 +
      EvidenceAmendmentChainV1 (`annotation.rs`; read-only context overlays + append-only human-review
      chains).** Real operations layer *context* and *human review* on top of immutable evidence without
      ever mutating it. The two overlays are read-only: `RecipeTransitionGuardV1.in_transition_window()`
      marks samples near a batch-recipe phase change, `MaintenanceEventOverlayV1.covers()` marks samples
      inside a maintenance outage — an episode in such a window is contextualised, never blindly alarmed;
      they annotate, they never change the evidence. The two logs are **append-only + hash-chained**:
      `OperatorAnnotationLedgerV1` chains each operator note to the prior entry's hash (tamper-evident —
      altering or removing a past entry breaks `verify_chain()`); `EvidenceAmendmentChainV1` anchors its
      genesis link to the *immutable* `original_evidence_hash` so amendments are appended while the
      original sealed evidence stands unmodified. All hash-sealed + self-verifying. edge lib 56→59;
      workspace green; `verify-replay` 6/6; clippy-clean.
- [x] **P69 — SBIRTransitionPackV1 (`transition_pack.rs`; machine + human Phase-I-style readiness pack;
      generic, no agencies).** A Phase-I-style effort closes by handing a reviewer a transition pack — what
      was attempted, which go/no-go gates were met, what is ready vs not, the residual risks, how to
      reproduce. `SBIRTransitionPackV1` packages exactly that, **deterministically and hash-sealed**,
      reusing the P41 milestone-gate protocol: `TransitionMilestone` carries id (M0–M3) / objective /
      replay-checkable go/no-go gate / `evidence_anchor` (SHA-256 of the sealed artifact) / `MilestoneStatus`
      (`Met` | `Pending` | `OutOfScope` — the last is an honest "not attempted this phase"). It is
      simultaneously machine-readable (sealed serde record + `pack_hash` + `verify()`) and human-readable
      (`to_markdown()` one-pager). Bounded honestly: it **names no agency, program, or vendor** (a test
      asserts the rendered pack is agency-free), every `ReadinessClaim` carries an explicit `boundary_note`
      of what is *not* claimed, and a `non_claims` block states the limits up front. Additive + off the
      replay path. edge lib 59→62; workspace green; `verify-replay` 6/6; clippy-clean.
- [x] **P71 — final QA / legendary pass (correctness + consistency only; breadth preserved).** Clean rebuild
      + honest `.log` audit of the PDF: **42 pages, 0 Overfull hbox, 0 undefined cite/ref** (20 benign
      underfull, not claimed). Re-measured every count the P57–P69 build-out moved and reconciled the docs:
      **edge lib 62**, default `cargo test --workspace` = **103** (edge 62 + 3 + 6 + 2 + 4 + 1 + 1; atlas 7;
      corpus 7; cuda CPU-path 10), cuda **13** with `--features cuda` (GPU-run: 6 lib-unit + 3
      `DigestEquivalenceHarnessV1` V1/V2-A/V2-B + 2 golden-evidence + 1 GPU↔CPU parity + 1 host-SHA) — the
      stale "55 workspace" / "edge 14 lib" / "cuda 7" claims in PROJECT_PLAN, README, and the verification
      report were corrected. Re-ran `ArtifactCompletenessCourtV1` → **COMPLETE (9 pass, 0 fail)**,
      `report_hash 98821ad4…` (unchanged). `verify-replay` **6/6 byte-identical**; CUDA `evidence_root`
      20/20 byte-exact GPU↔CPU; frozen `atlas_hash_v1 8f921f58…` / `corpus_hash_v1 7ce33a2e…` intact.
      Contradiction + traceability + no-overclaim sweep: the two new objects (P68 overlays/chains, P69
      transition pack) are read-only / append-only / bounded with explicit non-claims; no agency names leak;
      no stale "pending" phrasings remain. No code or paper-source change needed — only doc reconciliation.
- [x] **Figure campaign — 60-figure deterministic gallery + chem-eng practitioner grounding (additive;
      replay-inert).** Driven by a committed elite-panel review (`docs/legendary_panel_review.md`, ~9.2/10)
      whose recommendation ledger (R1–R9 + four "sharpen" items) is the backbone, and a committed chem-eng
      practitioner dossier (`docs/chemical_engineering_practitioner_dossier.md`) that puts the cited homework
      (ISA-18.2/IEC 62682, NAMUR NE107/NE131, IEC 61511 SIS boundary, MOC, historian/OPC-UA, conservation-law
      witnesses, residence-time topology) on the page. New deterministic renderer package
      `crates/…-edge/scripts/figures/` (SOURCE_DATE_EPOCH-pinned → re-render byte-identical; graphviz-first /
      networkx fallback; Wong colourblind-safe palette; on-figure honesty disclaimers; SHA-256
      figure-provenance manifest) renders **60 distinct figures** across nine groups (A core method · B atlas ·
      C performance/CUDA · D topology/provenance graphs · E disagreement forensics · F results · G physics
      witnesses · H regime · I practitioner). New Rust `figure_export.rs` builds byte-stable representative
      instances of the not-demo-exercised evidence objects (graphs via their own `to_dot()`) + dumps the atlas
      authority as JSON. **Two generation methods in lockstep:** the `dsfb-chem-edge figures` command
      (demo → export → render → verbose `figure_build_log.txt` → deterministic `figures_bundle.zip`) and the
      Colab notebook §7b gallery. Each figure was rendered, **read back as an image, and iterated until it
      cleared a legendary rubric** (e.g. moved the residual representative off the degenerate cstr SPE; showed
      the BATADAL out-of-scope band rather than fabricated per-attack heights; row-normalised the detector
      heatmap; showed the gas-sensor regime case as an honest *negative* result). The **17 strongest** figures
      are embedded in the paper (**42→51 pages**, 0 overfull / 0 undefined, `.log`-audited; all 22 figures
      embedded). edge lib unchanged (no new tests); `verify-replay` 6/6; clippy-clean on touched Rust; figures
      additive + off the replay path. Local commits only.

> **Implementation audit (P-docs vs code), at the P71 reconciliation** (frozen P71-era snapshot — counts
> below are the P71 baseline and are not rewritten retroactively; the **post-P71 program is logged in its own
> current-state section immediately after this block**): the 20-item innovation roadmap is fully
> implemented; the approved **P52–P71 plan is COMPLETE** (P71 final-QA done), with **P70 (CUDA
> evidence-kernel V2) pulled forward out of sequence** (P34–P42 prior-art strengthening batch;
> P43+ panel-driven additive hardening; P52 public-release wording + consistency; P53 corpus
> provenance-classification tiers; P54 verification report + ArtifactCompletenessCourtV1; P55 EdgeCoreProfileV1 design doc; P56 molecular-corpus companion note; P57 RegimeEnvelopeV1 + ChemicalAuthoritySeparationLawV1; P58 ChemometricPassportV1 + ResidualProvenanceGraphV1; P59 DetectorDisagreementForensicsV1 + NegativeWitnessV1; P60 UnknownTaxonomyV1 + ConfuserDocketV1 (atlas_hash 8f921f58…); P61 BalanceWitnessV1 + SoftSensorWitnessV1; P62 ControllerMaskingV2 + ValveStiction + AlarmFlood + OperatorIncident; P63 SensitivitySweepReceiptV1 + AblationCourtV1; P64 Topology + Propagation + ResidenceTime + CausalNonClaim; P65 EpisodeShapeHash + MotifNN + Recurrence + Precedent + FleetCompare; P66 NarrativeCompiler + NoHallucinationGate; P67 MigrationReceipt + RegistryCompat + SemanticDiff; P68 RecipeGuard + MaintenanceOverlay + AnnotationLedger + AmendmentChain; P69 SBIRTransitionPackV1; P70 CUDA V2; P71 final-QA reconciliation —
> digest-equivalence harness + V2-A/V2-B + measured ~18× kernel / ~8× end-to-end, all GPU-measured +
> gated). Artifact state: **four crates build + test** —
> edge **62** lib + **3** authority-gate + **6** court-record + **2** provenance-gate + **4**
> fault-demonstrator + **1** golden-replay + **1** completeness-court tests, atlas **7**, corpus **7**, cuda **13** with
> `--features cuda` (6 lib-unit incl. the **P70** V2 evidence-format tests + 3 `DigestEquivalenceHarnessV1`
> V1/V2-A/V2-B + 2 golden-evidence + 1 GPU↔CPU parity + 1 host-SHA-parity; 10 of these run in the CPU-only
> default build, the GPU↔CPU parity + 2 GPU digest-equivalence variants being feature-gated) — **103 tests
> total in the default `cargo test --workspace`** (CPU-only, GPU-measured); **paper 51-page PDF, 0 undefined cites/refs,
> 0 overfull (verified against the `.log`)** (42→51pp after the 60-figure campaign embedded 17 best figures);
> `verify-replay` 6/6 **byte-identical to the pre-change baseline across P21–P51** (every phase confirmed
> replay-inert; the golden gate now pins the synthetic replay hashes); CUDA `evidence_root` 20/20
> byte-identical across the P40 `atlas_hash` re-freezes; the Chemical Court Record v1 bundle + historian
> replay (incl. the P41 fictional incident) are demonstrated end-to-end. P9–P51 are local commits on
> `master`, not pushed.

---

## Post-P71 program — v2 frontier + mechanized breadth discipline (current state)

> A panel-driven additive program **after** the P52–P71 plan completed. Strategy unchanged: **prior art /
> defensive publication — breadth widens, never narrows.** Every object is additive, off the replay path,
> hash-sealed, self-verifying, test-gated, and carries explicit non-claims; no frozen hash moved (the
> Wave-1 governed re-freezes are noted in `EXPECTED_BUNDLE_ROOTS.toml`). Local commits on `master`, not pushed.

- [x] **Wave 1 — robustness + honesty (release-blocking).** A1 PCA degeneracy guard (`data.rs Baseline::fit`:
  near-constant baseline channel → raw deviation; CSTR SPE 1.4e35 → bounded; governed re-mint of the two
  affected datasets) + regression tests; A2 stop silent report writes (tracked tally, exit 3 on incomplete);
  A3 claim-boundary banner + claim-strength legend on `operator_report.html` (governed re-mint of all 20
  `bundle_root`s; `evidence_root`s unchanged); B behavioural tests + independent completeness oracle + 3 new
  Kani grammar-totality proofs (VERIFY SUCCESSFUL).
- [x] **Wave 2 — public-release hygiene + mechanized claim discipline.** H1 `.gitattributes` export-ignore +
  top-level `reports/verification_report.md`; **P72** `PublicReleaseScrubCourtV1` (`release-scrub`, caught a
  real placeholder DOI); **P73** `ClaimStrengthV1` (executable Tier-1/2/3); **P74** `PhysicalWitnessStrengthV1`
  ladder; **P76** whole-operator-report claim audit; **P79** `authority-diff` CLI; **P77** SBIR evaluator pack
  (`docs/sbir_*`).
- [x] **Phase C (catalogue breadth)** — `docs/catalogue_expansion_targets.md`: disclosed (catalogued, NOT
  executed) next detectors / fault signatures / benchmarks (DAMADICS, CWRU, TEP-extended, BattLeDIM) /
  balance-witness types. Executing any of them stays a governed `atlas_hash_v1` + replay re-freeze (deferred to
  a dedicated pass). [`2ede5d8`]
- [x] **Wave 3 — physics, balance & first-principles expansion.**
  `UnitConsistencyCourtV1` + `unit-consistency` CLI (°C↔K / bar↔Pa / mass↔mole-fraction; runs over all 6
  documented balances) [`111f8a0`]; `SpecLimitWitnessV1` (hard limit vs statistical anomaly; no SIS authority)
  [`defe509`]; `PermitBoundaryWitnessV1` (consent variables + tightest-approach headroom; not compliance
  certification) [`40dd2e0`]; `FirstPrinciplesWitnessAdapterV1` + `EquationResidualPassportV1` (Arrhenius /
  Antoine / Raoult / Henry / heat-transfer / pump-curve / valve-Cv → model–plant residual + sealed
  provenance/validity) [`5944eed`]; **P75** balance-witness pack (tank-inventory / splitter-mixer / HX-energy /
  element / separator-component / utility-loop) [`12228a5`]; `ResidualEnergyBudgetV1` (interpretable residual
  decomposition; magnitude split, not causal attribution) [`68e4815`]; doc stale-scan + post-P71 ledger
  [`b959704`].
- [x] **Wave 4 — industrial historian layer** (closes the public-data → real-plant gap).
  `IndustrialDataReadinessCourtV1` (grade an export Ready/ReadyWithCaveats/NotReady before analysis) [`e86c151`];
  `MultiRateAlignmentCourtV1` + `ManualSampleBridgeV1` (auditable ragged→grid resampling receipts; lab-sample
  bridge with custody hash) [`2fbb5dd`]; `SetpointResidualSeparationV1` + `ControllerModeGuardV1` +
  `ControlLoopInteractionMapV1` (PV/MV/SP context — kills setpoint-change false alarms) [`2a4a7d6`];
  `StartupShutdownEnvelopeV1` (per-transient-phase envelopes; transient-alarms-reclassified) [`60d079f`];
  context/QA witnesses `MaterialLotWitnessV1` + `CertificateOfAnalysisDensorV1` + `CleanInPlaceWitnessV1`
  [`34b1428`] and `SensorTrustDegradationLedgerV1` + `CalibrationEventWitnessV1` [`34b1428`] and
  `BatchGenealogyGraphV1` + `PlantTwinReplayV1` [`d50bde3`]; real-data drop-in: `data-readiness <csv>` CLI +
  `docs/real_data_dropin.md` (TRL-4→5 depends on a user-supplied ungated export) [`a692913`]; doc stale-scan
  [`47a6d50`].
- [x] **Wave 5 — edge core + CUDA timing + ecosystem.** New **`dsfb-chemical-engineering-core`** crate —
  `no_std`, no-heap, fixed-point (scaled-i64) residual triple + ring buffer + admissibility envelope + the full
  grammar state machine, zero dependencies, `#![forbid(unsafe_code)]`; **builds for `thumbv7m-none-eabi` and
  runs on an emulated Cortex-M3** (the standalone `qemu-smoke` harness, cortex-m-rt + semihosting) [`c7b4c11`].
  Deposit-readiness: `Dockerfile` (reproducible CPU-only local-run container) + `docs/release_checklist.md`
  (local gates + USER-ONLY outward steps); CITATION.cff already present [`85f5faf`]. **CPU-vs-GPU end-to-end
  timing**: measured the CPU reference in-sandbox, paired with the P5 GPU campaign (7.4×/15.9×/33.2× at
  32/128/512 MB, ~66× below the 637 GB/s roofline → R4 auditability framing) + a Nsight handoff doc [`ff81ee1`].
  **maturin/pyo3** Python binding skeleton (`dsfb-chemical-engineering-py`, standalone, abi3; builds the cdylib;
  publish USER-ONLY) [`65f554d`]; doc stale-scan [`3b68d57`].
- [x] **Wave 6 — confidential-evaluation chain** (the commercial unlock: the operator runs the court locally
  and shares only a redacted, hash-linked evidence bundle — raw data never leaves their control). 6a
  `PlantDataContractV1` + `HistorianImportReceiptV1` [`96ccb42`]; 6b data-quality (`DataQualityEpisodeV1` +
  `FrozenTagDetectorV1` + `ClockSkewWitnessV1`) [`02130a8`]; 6c observability (`InstrumentationCoverageMapV1` +
  `ObservabilityNonClaimReceiptV1` + `ResidualWitnessCoverageScoreV1`) [`04b232a`]; 6d event evidence
  (`ChemicalEventOntologyV1` + `EpisodeEvidenceLedgerV1` + `EvidenceMinimumsMatrixV1` + `WitnessBurdenOfProofV1`
  — fail the burden → `unknown`) [`126d35d`]; 6e adversarial (`AdversarialConfuserSuiteV1` +
  `FalseNarrativeRegressionTestV1`) [`e7cfa66`]; 6f export (`CaseFileRedactionMapV1` + `TamperEvidenceSealV1` +
  `AuditTrailExportV1` [`e6eb6e2`]; `ConfidentialEvaluationBundleV1` + `PartnerDataEscrowProtocolV1` +
  `ChemicalProcessPlaybookV1` [`b137866`]); doc stale-scan [`c6c4942`].
- [x] **Wave 7 — research-grade (in-sandbox subset + handoff).** Physics: `Interval` +
  `PhysicsInformedEnvelopeV1` (interval-arithmetic model–plant mismatch as first-class evidence) +
  `MultiPhysicsCrossWitnessV1` [`a479171`]. Semiotics: `HierarchicalMultiScaleFusionV1` +
  `SpectralGrammarTokenV1` [`b7c8f1c`]. Forensic/operator: `MerkleDagAmendmentChainV1` +
  `OperatorUncertaintyDashboardV1` + `SignatureDiscoveryAssistantV1` [`f38628c`]. Ecosystem/formalism:
  `DsfbBenchV1` + `SafetyCertificationDossierV1` + `ProofObligationLedgerV1` [`4ff22b9`]. The external-tool /
  infra items (Lean4/Coq, OPC-UA, heterogeneous/WGSL backend, open registry, Densor IR) are
  catalogued as handoff in `docs/wave7_research_roadmap.md` [`bf767f3`], honouring **AVOID #10** (post-quantum
  hashing) + **AVOID #17** (generic cross-domain extraction). *(The WASM court simulator is now executed — see
  the panel-batch ledger entry below.)*
- [x] **Wave 7 (post-deferral, after the user installed the Lean toolchain + asked for wasm32).** The `no_std`
  core also builds for **`wasm32-unknown-unknown`** (the WASM-simulator substrate; same crate as the thumbv7m
  embedded build) [`bead455`]. **Lean 4 formal verification — DONE:** `formal/lean/DsfbGrammar.lean` (pure Lean
  4 core, `lake build` verifies) machine-proves the grammar obligations over unbounded `Int`, discharging the
  two previously-open **quorum soundness** + **episode-compression monotonicity** obligations and re-proving the
  three Kani ones unbounded; `ProofObligationLedgerV1` updated (`Lean4Verified`; 5 machine-checked / 1 open —
  only replay determinism remains empirical) [`4887308`]. **Coq/Rocq port — DONE + verified:**
  `formal/coq/DsfbGrammar.v` (Rocq 9.1.1; `classifyAxis` over `Z`) cross-checks the same obligations in a
  second prover kernel (`coqc` verifies) → the grammar/fusion obligations are now checked by **three**
  independent tools (Kani · Lean 4 · Coq).
- [x] **Phase-C executable — DONE (two governed re-freezes; the catalogue widened by adding executions,
  never trimming).** **Detectors 14→18** [`198cc35`]: `mewma` (Lowry 1992), `dpca` (Ku 1995), `mosum`
  (Bauer&Hackl 1978), `mmd` (Gretton 2012, landmark) implemented in `edge/detectors.rs` + appended to
  `build_bank` (append-only → replay order preserved) + matching atlas Executed records; each warms-up-
  suppressed so a cold-start transient is never a spurious episode; corpus TOML 18 exec / 39 cat / 57 total.
  **F2 pump cavitation 6→7 signatures** [`9e1b706`]: a faithful synthetic `gen_cavitation` demonstrator the
  pipeline genuinely catches (broadband-vibration SPE motif → `spectral_entropy_spe`/`knn_spe`/`ewma_spe`,
  exactly one onset episode, gated in `tests/fault_demonstrators.rs`). Governed: `atlas_hash_v1` re-minted
  `8f921f58…`→`fcadf486…`→**`936ac67a…`** (detector then signature flip; both pinned + noted); golden replay
  6/6, the 20 bundle roots + court pin unchanged (no sealed root embeds `atlas_hash`); counts reconciled +
  60 figures regenerated + paper 51pp. Follow-up [`37261b5`] cleared the 9 pre-existing clippy 1.94 warnings
  (behavior-preserving — golden replay byte-exact, no hash moved) + a dated "since-actioned" note on the
  panel review.
- [x] **Panel-9.5 sharpening batch — DONE.** Build-all, sequenced, additive (responding to the 9.5/10 panel
  verdict: archive-proof discipline + consistency hardening + execution-coverage expansion; breadth never
  narrowed). Build hygiene: redirected the standalone `py`/`qemu-smoke`/`wasm` crate targets into the shared
  root `target/` [`589dd8f`]. Consistency fixes (#3 228→229, #4 Coq "uses stdlib List/ZArith/Lia", #5
  crate-count) [`ad30af0`]. **WASM court simulator** (standalone `dsfb-chemical-engineering-wasm`, raw
  `extern "C"`, no wasm-bindgen, + static HTML/JS shell) [`705b0ab`]. **P81** archive-mode release-scrub
  (`--archive-dir`; fails on a smuggled `SESSION_*`) [`67f7201`]. **P82** `EvidenceKind` taxonomy bridged to
  ClaimStrength + PhysicalWitnessStrength [`46f2e0b`]. **P83** universal operator-report legend (GOVERNED:
  20 bundle roots + court pin re-minted; evidence roots invariant) [`ea55967`]. **P86** proof-wording
  reconciliation [`a1367b4`]. **P84** `confidential-demo` CLI (one-command redacted partner-evaluation bundle;
  the 17th subcommand) [`6592628`]. **P85** embedded memory-budget doc + QEMU re-verify [`71b7c33`].
  **Navigation layer LAST:** top-level `breadth_surface.toml` (every claim → artifact → reproduction → tier)
  + `breadth_surface_court` self-check (counts vs `atlas::validate()` + manifests; canonical hash pinned),
  so it indexes the new objects. AVOIDs #10/#17 + strictly-chemical held throughout.

> **The ~100-item mechanized-breadth program is COMPLETE for the in-sandbox-tractable scope** (Waves 1–7 +
> Phase-C catalogue **and the Phase-C executable re-freeze**). Remaining are deliberately bounded: the
> Wave-7 external-tool handoffs, the routine paper-rebuild / panel-v3, and the deferred P80 molecular corpus.

> **Current artifact state (post-narration-context batch, 2026-05-28):** default
> `cargo test --workspace` = **295 tests, 0 failed** = 291 unit/integration + 4 doc-tests. unit/integration: edge
> 245 (213 lib + 32 integration); atlas 7; corpus 7; cuda CPU-reference path 14; **core `no_std` 8**;
> **`dsfb-densor-runtime` 8**. The +4 over the prior 289 is `NarrationContextV1` (incl. the hallucination-gate
> round-trip + the committed-sample byte-check). **`narration-context <dataset>`** emits the citable-anchor
> narration context (+ `casefile` auto-emits it); the per-episode anchor is the shared `report::episode_evidence_anchor`
> (the refactor was byte-identical → no bundle re-mint). doc-tests (4): `hashing::sha256_hex` · `hashing::CanonicalHasher` ·
> `pipeline::analyze` · `court_record::write_court_record`. (The P101 `densor-runtime-demo` test is feature-gated
> and NOT in this default count.) GPU-gated `--features cuda` tests run on the host only. The +2 over the prior 287
> is the P98 `ArtifactIndexCourtV1`'s 2 tests. **`verify-index` court: INDEX-VERIFIED 9/0** (the committed
> `reports/index.{html,json}` re-derives + matches the live artifacts). `verify-replay` **6/6**; bundles **20/20**
> — re-minted this batch: P102 added per-episode ClaimStrength + EvidenceAnchor columns to operator_report.html
> (display-only → all 20 `bundle_root`s + court golden shifted, all 20 `evidence_root`s byte-UNCHANGED), and a
> **governed A1 correction** re-froze cstr_reactor's `evidence_root` (the linalg degeneracy guard genuinely changed
> its constant-channel evidence — A1 was wrongly reported hash-neutral; bundle check had read a stale demo dir) +
> cstr/three_tank `bundle_root`s. `completeness-court` **COMPLETE 7/0** (9/0 with
> `--features soft-sensor-corpus`); `release-scrub` **RELEASE-CLEAN 5/0** (P82 controlled-data + P87 roles-metadata
> gates added since the 9.5 batch); `unit-consistency` **6/6 balances**; `data-readiness` runs on a real CSV;
> **core builds for thumbv7m + runs on emulated Cortex-M3 (QEMU)**; **CPU-vs-GPU end-to-end 7.4×/15.9×/33.2×**
> (host-measured, unchanged this batch); Kani **6/6**; **Coq + Lean grammar proofs verify**; paper **66 pp, 0
> overfull / 0 undefined**; `cargo clippy --workspace --all-targets` **0 warnings**. Executed coverage **18/57
> detectors** and **7/12 fault signatures**; frozen authority `atlas_hash_v1 = 936ac67a…`,
> `corpus_hash_v1 = 7ce33a2e…` (intact). **Verification-tools campaign (all run in-sandbox):** cargo-fuzz
> 225.2M execs/0 crashes · valgrind CLEAN/0 errors · hax 716-line F\* model · Creusot Coma IR · Flux core-clean ·
> crux-mir **Valid 4/4** · loom N/A by design; **Miri** also clean on the new `dsfb-densor-runtime` (8/8, no UB).
> **P92–P97 additions:** `generate-index` emits a deterministic `reports/index.{html,json}` + sealed `index_root`
> (8 crates · 20 bundles · 60 figures · 20 datasets · 2 courts · docs); `dsfb-densor-runtime` is the sixth
> workspace member (`#![forbid(unsafe_code)]`, no chemical/cross-domain claims); dsfb-gray now reads `edge` **65.6 %**
> (well-commented growth) and `dsfb-densor-runtime` **70.2 %**; cargo-audit **42 deps, clean**; geiger **5/7 forbid-unsafe**.
> **Pending (bounded):** Wave-7 external-tool handoffs (`docs/wave7_research_roadmap.md`); Creusot SMT discharge
> (hermetic why3 fork); deferred P80 molecular corpus.
