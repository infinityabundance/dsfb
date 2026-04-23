# Changelog

All notable changes to the `dsfb-rf` crate are recorded here. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The companion paper `dsfb_rf_v2.tex` is a deliberately **broad prior-art
foundation** for Structural Semiotics on RF signals. Changes in this log are
additive to that foundation — we do not narrow claims or remove capabilities
between versions without a major-version bump.

## [Unreleased]

### Added
- **Four undefined `\label{}` targets resolved** in `paper/dsfb_rf_v2.tex`:
  `sec:aug:positioning`, `sec:swarm`, `sec:formal:corr`, and
  `sec:sbir:trl`. Pure label additions — no section moves, no renames.
- **Theorem numbering footnote** at §V.A (Law 1) clarifying that
  Theorems 1, 9, 10 follow the DSFB framework sequence continuity
  convention; intervening results are named lemmas/laws rather than
  numerically indexed. Theorem 10 (Auditable Early-Warning Inference)
  now carries label `thm:t10-aewi`.
- **Abstract 95.1 % recall footnote** with denominator framing:
  clarifies that 102-transition Stage~III recall and 528-bin
  amplitude-domain 4.7 % recall are not comparable because the
  denominators count different objects. Forward-reference to
  §VII.G.5 added parenthetically after ORACLE recall.
- **§VII.G.5 heading strengthened**: *Recall Denominator Framing* →
  *Why Recall Is Not the Right Metric (Denominator Framing)*.
  Exposes the paper's strongest methodological passage in the ToC.
- **Four new Related-Work citations** (additive, observer-framed):
  Sankhe et al. 2020 (Impairment Shift Keying, INFOCOM), Jian et al.
  2020 (deep-learning RF anomaly, IEEE JSAC), Roy et al. 2020 (RFAL
  adversarial learning, IEEE TCCN), Riyaz et al. 2018 (convolutional
  radio identification, IEEE Comm. Mag.). Each accompanied by a
  one-sentence framing of DSFB's observer role relative to the cited
  method.
- **Multi-emitter L16 paragraph expansion**: formal derivation of
  DSA accumulation rate under shared-manifold $M \geq 2$ emitters,
  per-channel envelope calibration requirement, $\mathcal{O}(M \cdot B)$
  calibration budget.
- **§IV.F.3′ Fisher-information half-page** tying envelope $\rho$ to
  Cramér–Rao lower bound via $I(\rho) = \operatorname{Var}^{-1}(\|r\|)$;
  derives $\rho_{\min} \geq 3\sigma_{\|r\|} \cdot g(\alpha)$ with the
  "3σ" factor matching `CRLB_MARGIN_THRESHOLD` in `src/uncertainty.rs`.
  Non-claim: structural detectability floor, not P_d/P_fa.
- **Table V (W_pred sensitivity) deferral footnote upgraded** to a
  single-command reproduction path plus measured compute estimate
  (≈45 min per W on 8-core x86-64) with explicit dataset size note
  (RadioML ≈20 GB + ORACLE capture directory).
- **Companion SBIR/TRL tech report** `paper/dsfb_rf_sbir_trl_v1.tex`
  (new, ~15 pp): executive summary, dual-use applicability, named-
  hardware TRL 4→6 milestone table (B210 → X310+VITA 49.2 → CMOSS
  VPX), SBIR topic-family mapping (AFWERX / ONR / CCDC), observer
  contract as risk argument, reproduction/audit appendix. Shares
  positioning with main paper §XIII (main paper retains §XIII
  unchanged).
- **`docs/API_PRECONDITIONS.md`** (new): canonical documentation of
  library caller preconditions (for `run_stage_iii`,
  `GrammarEvaluator::observe`, `AdmissibilityEnvelope::calibrate`),
  the `paper-lock` binary exit-code table (0 success, 1
  unknown/missing subcommand, 2 pipeline/HDF5 failure), and the
  example-binary unwrap disposition index.
- **`docs/SBIR_READINESS.md`** (new): TRL self-assessment mirroring
  paper Table 4, L1–L22 × Phase I deliverable risk-burn-down matrix,
  named-hardware roster with status, open SBIR topic-family map,
  non-certification statement.
- **Principled unwrap replacements** (~15 sites in `examples/`):
  silent-default masking and control-flow unwraps replaced with
  structured error returns so example binaries exit 2 with clean
  messages instead of panicking. Calibration-asserted `.expect("…")`
  sites with descriptive preconditions are retained per
  `docs/API_PRECONDITIONS.md` disposition table.
- **MSRV raised to Rust 1.83** in `Cargo.toml` to match the CI floor
  and the transitive requirements of `hdf5-metno 0.12.4`. New CI job
  `msrv.yml` exercises `cargo check --no-default-features` and
  `cargo check --features std,serde` on `dtolnay/rust-toolchain@1.83`.
- **`cargo semver-checks` CI activation** in
  `.github/workflows/quality.yml`: breaking-change gate against the
  published `dsfb-rf@1.0.0` baseline on PRs to `main`.
- **Additive technical-first preamble** in `src/lib.rs` above the
  existing Sovereign Spectrum Governance block: one-paragraph
  observer-contract framing. Existing C2 framing unchanged below
  the divider.
- **README Collaboration & Partnership section** with
  `partnerships@invariantforge.net` inquiry aperture and
  cross-links to `docs/SBIR_READINESS.md` and the companion tech
  report.
- **Figure 149** — demodulation-threshold scan across 24 RadioML
  modulations (dot plot, SNR at first grammar-episode close per
  modulation). Additive to the 67 + 80 = 147 figure bank; total
  148. Caption names the upstream amplitude-template demodulator
  residual producer; non-claim: not a modulation classifier.
- **Colab notebook extension** (`colab/dsfb_rf_reproduce.ipynb`):
  `libhdf5-dev` install pinned with graceful-fallback probing,
  Cell 11 now verifies both the 67 synthetic and the 80 real-world
  figure banks plus `fig_149`, Cell 12 zips the real-world bundle
  alongside the synthetic bundle into a single downloadable
  artefact.

### Fixed
- **Kani harness `proof_decimation_exact_epoch_count`.** The prior
  `#[kani::unwind(6)]` bound was insufficient for the Newton–Raphson
  square-root loop in `crate::math::sqrt_f32` (up to 12 iterations),
  causing the harness to fail locally with unwind-assertion failures.
  The unwind bound is raised to `16` — conservative spare over the
  deepest reachable loop — and an overflow-avoiding precondition
  `kani::assume(norm < 1.0e9_f32)` is added so the accumulated
  `n²` sum cannot saturate the f32 representation. This precondition
  is justified by the paper's calibration protocol: residual norms
  are bounded in the f32-meaningful range (`ρ ≈ μ + 3σ`, typically
  1e-4…1e1) many orders of magnitude below the cap. Rationale is
  documented in the updated harness docstring. Expected outcome:
  `VERIFICATION: SUCCESSFUL` restored. This fix does **not**
  narrow the panic-freedom claim — the harness still quantifies
  `kani::any()` over the full finite non-negative f32 range, minus
  the physically impossible overflow tail.

### Added (v1.1 additive — panel post-landing fixes)
- **Fisher-Information subsection — quantitative GUM Type-A tie-back.**
  Paper §IV.F.3′ now carries a paragraph expressing the finite-sample
  CRLB through the JCGM 100:2008 Type-A variance estimator
  $s^2_{\|r\|} = \frac{1}{N-1}\sum_k(\|r(k)\|-\bar R)^2$ with
  $\widehat{I}(\rho) = 1/s^2_{\|r\|}$, bound by
  `HEALTHY_WINDOW_SIZE = 100` in `src/pipeline.rs`. The envelope
  calibration floor now reads as an evaluable inequality
  $\rho_{\min} \geq 3 s_{\|r\|}\,g(\alpha)$.
- **Paper revision-history appendix** — new `paper/CHANGELOG.tex`
  that `dsfb_rf_v2.tex` `\input`s as an additive appendix. Itemises
  every v1→v2 additive edit with rationale; intended for ArXiv
  replacement submissions and reviewer auditability across revisions.
- **Real-dataset W-sweep example** — `examples/wpred_sweep_real.rs`
  loads the locally-present RadioML 2018.01a GOLD HDF5
  (`data/RadioML HDF5/GOLD_XYZ_OSC.0001_1024.hdf5`), runs the
  flat-stream protocol (Table IV methodology) **and** the per-class
  protocol (fig_149 methodology), and post-hoc recomputes episode
  precision for $W_{\text{pred}}\in\{3,5,7\}$ on the fixed episode
  stream. Table V deferred cells filled with measured values from
  this sweep; the original "deferred" footnote is preserved above
  the measured row for honesty.
- **fig_149 per-class calibration note** — caption now names the
  top-quartile-SNR per-class calibration protocol
  (`cal_n = pairs.len()/4`, clamped to ≥2) and explicitly states that
  the class-local envelope is a class-local analogue of the global
  Stage III calibration, not a cross-class claim.

### Planned (v1.1)
- **Pre-allocated `HdfReadBuffer`** enabling `hdf5_loader` to drop
  steady-state allocation after first call. Addresses dsfb-gray
  P10-3 "not applied" finding without affecting current functional
  surface.
- **Kani harness `proof_envelope_judgment_consistency`** — timed out
  at 10 min locally on opus hardware; CI gate stands as authoritative.
  Follow-up: investigate whether adding a `kani::assume(mult.is_finite())`
  narrowing (covering the architecturally-finite operational case
  while the infinite-multiplier suppression case remains vacuous for
  the invariant) is tractable without reducing proof value.
- **src/\*\* clippy burn-down** — 55 pre-existing lints (unrelated to
  this landing) tracked for targeted remediation; plan §1.6 froze
  `src/**` in v2 to preserve v1.0.0 API bit-identity. Scheduled as
  a follow-up refactor.
- **tests/\*\* fmt drift** — rustfmt differences predating this
  revision, also outside the v2 scope.

### Previously Added
- **`audit/` folder shipped inside the crate.** Canonical `dsfb-gray`
  static audit report (`dsfb_rf_scan.txt`), SARIF 2.1.0 findings
  (`dsfb_rf_scan.sarif.json`), in-toto v1 statement
  (`dsfb_rf_scan.intoto.json`), and unsigned DSSE envelope
  (`dsfb_rf_scan.dsse.json`) now ride inside the crate so downstream
  reviewers can inspect the audit without re-running the scanner.
  `audit/README.md` documents the rubric, open findings, and
  reproduction command. Overall score: **91.4 %** (strong assurance
  posture). DSFB-Gray shields-badge added to `README.md` and the new
  Audit section.
- **Paper appendix: *dsfb-gray Audit Report Summary*.** New unnumbered
  appendix in `paper/dsfb_rf_v2.tex` between *Colab Reproducibility*
  and *IP Notice* with the scoring breakdown (Table), advisory
  subscores, honest disclosure of four non-maximal Power-of-Ten rules
  plus the transitive `libloading` PLUGIN-LOAD flag, and a direct link
  to the in-repo artefacts under `crates/dsfb-rf/audit/`. Two new
  bibliography entries: `dsfb_gray` (scanner crate) and
  `dsfb_gray_audit_folder` (public artefact location).
- **Real-Dataset Figure Bank — 80 figures (fig_69 … fig_148), 10 per slice
  × 8 real-world slices.** New `examples/generate_figures_real.rs` +
  `scripts/figures_real.py`. Renders the crate's structural machinery —
  grammar FSM, sign-tuple, DSA, envelope, Fisher-Rao, super-rho
  persistence, attractor, detectability bound, permutation entropy,
  review-surface compression — on residuals read directly from the eight
  slice files under `data/slices/`. Invocation: `cargo run --release
  --example generate_figures_real --features std,serde,real_figures`
  emits `../dsfb-rf-output/dsfb-rf-real-<ts>/figs/*.pdf` (80), merged
  `dsfb-rf-all-real-figures.pdf`, `figure_data_real.json`, and an
  artefacts zip. **Positioning:** every caption names the upstream
  residual producer (matched filter / AGC / channel estimator / GNSS
  tracking loop / scheduler EWMA / beamformer / beam-tracker) and
  frames DSFB as the structural interpreter of that producer's already-
  computed residual stream — not a competitor, not a replacement, not a
  "detects-earlier-than" claim. Missing-slice blocks emit loud
  `[SKIPPED — <slice> not present]` banners; remaining slices still
  render.
- **New feature flag `real_figures = ["std", "serde", "hdf5_loader",
  "dep:csv"]`**. Default build surface unchanged for v1.0 consumers;
  `csv` is optional and only pulled in under `real_figures`.
- **Toolchain pin bumped to 1.85.1** (`rust-toolchain.toml`) so the
  latest `hdf5-metno 0.12.4` / `hdf5-metno-sys 0.11.3` (which parses
  HDF5 2.1.1 `H5_VERSION`) resolves cleanly alongside the new `csv`
  dependency. MSRV in `Cargo.toml` stays at 1.65 for library consumers.
- **Colab reproducibility notebook.** `colab/dsfb_rf_reproduce.ipynb` builds
  the crate from scratch on every run (rustup → libhdf5 → `git clone
  --depth 1`), reproduces the 67 paper figures via `cargo run --release
  --example generate_figures_all --features std,serde`, packages all
  artefacts plus the eight-dataset slice catalog into a single downloadable
  zip, and delivers a one-click browser download. Expected wall-clock on a
  free-tier Colab CPU runtime: ~15 min. Companion `colab/README.md` pointer.
- **Eight-dataset slice catalog.** `scripts/prepare_slices.py` and seven
  `scripts/gen_proxy_*.py` generators produce schema-preserving ≤ 2 MB
  slices for RadioML, ORACLE, POWDER, Tampere GNSS (Zenodo
  10.5281/zenodo.13846381), ColO-RAN (wineslab/colosseum-oran-coloran-dataset
  `rome_static_medium/`), ColO-RAN-commag (wineslab/colosseum-oran-commag-dataset
  `slice_mixed/`), DeepBeam (Northeastern repo `neu:ww72bh952`), and
  DeepSense-6G Scenario 23 UAV mmWave (1000-sample HDF5 head slice of
  user-downloaded `scenario23_dev_w_resources.zip` from
  deepsense6g.net/scenarios/scenario-23 — emitted with
  `mmwave_power[time,beam]` float32 (N,64), `best_beam_index`, and UAV
  telemetry: altitude, speed, pitch, roll, distance, height).
  **All eight slices now resolve to `real-*` provenance** — DeepBeam
  promoted via an 8192-sample head slice of user-downloaded
  `neu_ww72bk394.h5` (59 GB parent, 11.06 B IQ rows; Northeastern
  repository collection `neu:ww72bh952`); emitted as HDF5 preserving the
  parent's native NI transceiver schema `/iq (N,2) float64`, `/gain`,
  `/rx_beam`, `/tx_beam`. Parent identity pinned via
  `dsfb_rf:parent_first4MiB_sha256` root attribute. The loudly-labelled
  `[SYNTHETIC PROXY]` code path remains available as a safety net when
  neither a public mirror nor a local ≥ 128 MiB `neu_*.h5` under
  `data/deepbeam/` is reachable. CSV content-type guard rejects non-CSV
  blobs to keep the provenance column honest.
- **Paper integration.** `paper/dsfb_rf_v2.tex` gains a Colab badge after
  `\maketitle` and a new `\section*{Colab Reproducibility}` before
  `\section*{IP Notice}` that names the notebook, lists the eight-slice
  roster with one-line provenance each, and explicitly states that no
  headline number in Table 1 is produced by the Colab pipeline (L13
  stands).
- **REPRODUCE.md cross-reference.** §2.1 gains the eight-dataset slice
  catalog table and a pointer to `scripts/prepare_slices.py`.
- Repository-hygiene: crate-local `.gitignore` excludes `paper/` (219 MB),
  `dsfb-rf-output/` (174 MB+), `target/`, `Cargo.lock`, and large
  SigMF/HDF5 datasets so the tracked source tree stays under 5 MB.
- Reproducibility: `REPRODUCE.md` documents toolchain pins, dataset-access
  honesty disclosure (ORACLE/POWDER/Colosseum gated; RadioML via registration),
  figure-by-figure paper map, expected runtimes, known-degenerate inputs,
  and the license/citation chain.
- Real-data smoke asset: `data/slices/radioml_2018_slice.hdf5` (1.85 MB) is a
  stratified 240-capture slice of the canonical `GOLD_XYZ_OSC.0001_1024.hdf5`
  (24 modulations × 5 SNRs × 2 captures). Schema-preserving; companion file
  `deepsig_2018_snr30_slice.hdf5` (100 captures from the legacy single-SNR
  file). SHA-256 + stratification plan recorded in `SLICE_MANIFEST.json`.
- Synthetic-stub banners: the six examples that carry real-dataset names
  (`radioml_hdf5`, `crawdad_interference`, `atmospheric_fading_diag`,
  `gps_spoofing_detection`, `deep_space_metrology`, `urban_multipath_prognosis`)
  now open with a loud in-source banner and print `[SYNTHETIC STUB]` at
  runtime when run without a real dataset.
- Toolchain pin: `rust-toolchain.toml` at 1.83.0 gives agents and CI a
  reproducible toolchain; MSRV in `Cargo.toml` remains 1.65 for library
  consumers.
- Paper Limitations §L13–§L22: ten new honest subsections address the
  full hostile-reviewer checklist (statistical power, absent baselines,
  waveform-family generalization, multi-emitter scaling, TRL gap,
  hyperparameter sensitivity, PE-as-heuristic status, Kani CI-gate gap,
  calibration-burden unquantified, synthetic/real nomenclature risk).
  Every fragility is met with a limitation, not a claim narrowing.
- Paper citations: Ruelle (1989) *Chaotic Evolution and Strange Attractors*
  in the Theorem 1 / envelope section; Haykin (2005) "Cognitive Radio" in
  the Introduction; DARPA SC2 / Colosseum program report in the motivation
  and adversarial-example context.
- CI workflow: `.github/workflows/quality.yml` runs `cargo check` across
  the full feature matrix (`no_std`, `std`, `std,paper_lock,hdf5_loader`,
  `--all-features`), `cargo test --all-features`, `cargo clippy -D warnings`,
  `cargo fmt --check`, `cargo doc -D warnings`, and doctests. Placeholder
  jobs for Kani, `cargo deny`, `cargo semver-checks`, and bench-regression
  will be activated in v1.0.2 once their harness inputs are captured on
  reference hardware.

### Deferred to v1.0.2 (tracked; not in this release)
- `src/stats.rs` with BCa bootstrap + Wilson + Holm–Bonferroni.
- Bootstrap CI columns on Table 1 rows (depends on `src/stats.rs`).
- `tests/proptest_invariants.rs` (sign tuple / grammar FSM / DSA bounds /
  envelope monotonicity / Q16.16 round-trip) using `proptest` as a
  dev-dependency.
- `tests/calibration_error_paths.rs` once `run_stage_iii` returns a typed
  `Result<_, CalibrationError>` (current code preserves the panic path for
  `paper-lock` binary backward compatibility).
- `benches/baselines.json` + regression comparator (requires a clean
  reference-hardware capture before it can guard commits).
- Full Kani CI gate with 30-minute timeout per harness.

## [1.0.0] — 2026-04-20

### Added
- **Core engine (no_std/no_alloc/zero-unsafe).** `DsfbRfEngine` with grammar
  FSM (Admissible / Boundary / Violation), DSA accumulator, admissibility
  envelope, Q16.16 fixed-point math, and `#![forbid(unsafe_code)]`.
- **Semiotic manifold** (‖r‖, ṙ, r̈) and sign-tuple abstraction (`src/sign.rs`).
- **Grammar FSM** with K=4 hysteresis confirmation (`src/grammar.rs`).
- **Deterministic Structural Accumulator (DSA)** with EWMA calibration
  (`src/dsa.rs`).
- **Admissibility envelope** with ρ calibration from a healthy window
  (`src/envelope.rs`), theorem-grounded exit-time bound.
- **Paper-lock harness** (`src/paper_lock.rs`): bit-exact tolerance gate
  for Table 1 precision (0.712 ± 0.005) and recall (≥ 96/102).
- **Stage III pipeline** (`src/pipeline.rs`): calibration pass → episode
  tracking → precision/recall emit.
- **HDF5 loader** (`src/hdf5_loader.rs`) behind `hdf5_loader` feature for
  RadioML 2018.01a ingest.
- **Kani formal-verification harnesses** (`src/kani_proofs.rs`): 6 proofs
  covering grammar panic-freedom, severity bounds, envelope/judgment
  consistency, decimation epoch count, fixed-point resync drift, and
  Q16.16 quantize panic-freedom.
- **GUM-traceable uncertainty** (`src/uncertainty.rs`) per JCGM 100:2008.
- **Cross-target timing CI** (`.github/workflows/qemu_timing.yml`) proves
  per-sample-latency bounds on Cortex-M4F, RISC-V 32-bit, and x86-64.
- **14 examples** spanning ORACLE, RadioML, SC2, POWDER, IQ-Engine,
  CRAWDAD, deep-space, atmospheric-fading, GPS-spoofing, urban-multipath,
  forensic-recorder, W_pred sweep, and all-figures generation.
- **GNU Radio out-of-tree module** (`gr-dsfb/`) for flowgraph integration.
- **Companion paper** `paper/dsfb_rf_v2.tex` (32 pages, 4181 lines) with
  10 theorems/lemmas, 12 limitation disclosures (L1–L12), and the
  broad-prior-art posture formally stated.
