# dsfb-debug

**DSFB-Debug — Structural Semiotics Engine for Software Debugging**

A deterministic, read-only, observer-only augmentation layer that turns
the residuals every observability stack already discards into typed,
human-readable debugging episodes.

*Invariant Forge LLC — Riaan de Beer — ORCID: 0009-0006-1155-027X*

## What It Does

- Reads residuals from existing observability tools (OpenTelemetry,
  Jaeger, Datadog, ELK, Prometheus, Sentry) and from code-defect
  catalogs (Defects4J, BugsInPy, PROMISE).
- Computes typed structural interpretation: drift, slew, grammar state,
  motif, episode.
- Performs **Trace Event Collapse**: raw alerts → few typed episodes
  (RSCR ranges from 3.67× on the F-11 production-trace fixture to 62×
  on the Defects4J bug-catalog fixture; verbatim test stdout, bounded
  to those fixtures).
- Fuses **205 deterministic detectors** organised across **27 mathematical
  axes** (Tiers A–U + EXTRA + V/X/Y/Z/AA) into a consensus-validated
  typed output via the heuristics bank.
- Operates the **9-axis bank-aware fusion** with a 4-rung
  *Anti-Hallucination Ladder* — Phase 0 (raw) / Phase 5.6 (confuser-
  boundary) / Phase 7 (tier-witness) / Phase 8 (per-detector witness)
  — each rung a single config flag, each rung tested deterministically.
- Produces deterministic, reproducible, auditable outputs (Theorem 9 —
  empirically verified on every real-bytes fixture).

## What It Does NOT Do

- Does NOT replace any existing observability tool.
- Does NOT claim faster detection or higher accuracy than existing
  methods. The framing is augmentation, not competition.
- Does NOT modify upstream data (compile-time enforced via
  `&[T]`-only public surface).
- Does NOT use heap allocation, unsafe code, unwrap, or any runtime
  Cargo dependency in the no_std core.

## Crate Properties

| Property | Enforcement |
|---|---|
| `#![no_std]` | No standard library dependency |
| `#![forbid(unsafe_code)]` | Zero unsafe blocks |
| `#![deny(clippy::unwrap_used)]` | No panic paths |
| Zero runtime deps | Hand-rolled SHA-256, DFT, matrix algebra |
| Observer-only | All APIs accept `&[T]` only |
| Deterministic | Theorem 9: identical inputs → identical outputs |

## Mathematical Summary

The engine is a deterministic function from a residual matrix to a
list of typed episodes. The core construction at each `(window w,
signal s)` cell:

| Concept | Definition | Source |
|---|---|---|
| Residual signature | `σ(k) = (‖r(k)‖, ṙ(k), r̈(k))` | per-(w,s) magnitude, slew, curvature |
| Admissibility envelope | `‖r(k)‖ ≤ ρ` | violation triggers grammar state Boundary or Violation |
| Drift persistence | `Σ_{w' in [w-D, w]} I(‖r(k')‖ > μ)` over rolling D windows | drift dwell length |
| Slew density | mean `|ṙ(k)|` over rolling W | abrupt-change rate |
| Grammar state | `{Admissible, Boundary, Violation}` from σ + thresholds | per-(w,s) symbol |
| Motif | typed pattern from `HeuristicsBank::lookup(reason_code, drift, slew)` | per-(w,s) typing |
| Episode | contiguous run of non-Admissible windows aggregated | per-signal collapse |

**Fusion consensus** at each `(w, s)` cell:

```
consensus(w, s) = Σ_{cell-level detectors d}     I(d fires at (w, s))
                + Σ_{window-level detectors d}   I(d fires at w)
fire(w, s)      = consensus(w, s) ≥ min_consensus
```

A DSFB structural episode is **consensus-confirmed** when at least one
of its windows fires. The bank's `match_episode_with_consensus` then
scores the typed motif with consensus boost:

```
score(motif, ep) = w_drift   · drift_score(ep)
                 + w_slew    · slew_score(ep)
                 + w_bound   · boundary_score(ep)
                 + w_corr    · correlation_score(ep)
                 + w_dur     · duration_score(ep)
                 + α · (consensus_max(ep) / max_detectors)
```

The bank's per-motif weights `w_*` and consensus-factor α are documented
inline at [`src/heuristics_bank.rs`](src/heuristics_bank.rs).

**Theorem 9 (Deterministic Replay).** For any byte-stable residual matrix
input, two consecutive `engine.run_evaluation(...)` calls produce
byte-identical episode output. Mechanically proven by composition of
deterministic stages; empirically verified on F-11 (35 604 Jaeger spans).

## Code Tour

```rust
use dsfb_debug::DsfbDebugEngine;

// Engine creation under paper-lock parameters.
let engine = DsfbDebugEngine::<32, 64>::paper_lock().unwrap();

// Per-signal evaluation (single signal).
let eval = engine.evaluate_signal(&residual_slice, num_windows);

// Full residual-matrix evaluation.
let mut evals    = vec![Default::default(); num_signals * num_windows];
let mut episodes = vec![Default::default(); 256];
let (count, metrics) = engine.run_evaluation(
    &data, num_signals, num_windows, &fault_labels,
    healthy_window_end, &mut evals, &mut episodes, "fixture_name",
).unwrap();

// Causality (graph attribution): augments episodes with root_cause_signal_index.
let (count, _) = engine.run_evaluation_with_graph(
    &data, num_signals, num_windows, &fault_labels,
    healthy_window_end, &service_graph, &mut evals, &mut episodes, "fixture",
).unwrap();
```

### Multi-detector fusion

```rust
use dsfb_debug::fusion::{run_fusion_evaluation, FusionConfig};

let cfg = FusionConfig {
    min_consensus: 3,
    ..FusionConfig::ALL_DEFAULT  // enables all 205 detectors + 9-axis bank-aware fusion
};
let metrics = run_fusion_evaluation(
    &engine, &data, num_signals, num_windows, healthy_window_end,
    &fault_labels, &cfg, "fixture",
).unwrap();
```

### Operator-side helpers

```rust
use dsfb_debug::{render_episode_summary, EpisodeCatalog};
use dsfb_debug::calibration::recommend_config_from_healthy;

// Render an episode for an operator dashboard.
let summary = render_episode_summary(&episode, &signal_names);

// Episode catalog: similarity matching against past closed episodes.
let mut catalog = EpisodeCatalog::<256>::new();
catalog.record(episode);
let nearest = catalog.find_similar(&query);

// Site calibration on a healthy slice.
let report = recommend_config_from_healthy(
    &healthy_data, num_signals, healthy_windows, /* percentile = */ 0.90);
```

## Heuristics Bank Coverage

The bank ships **32 hand-curated motifs** across six tiers, each with
provenance (`FrameworkDesign` / `DatasetObserved` / `FieldValidated`),
the upstream dataset name + DOI it was observed in, a dashboard hint
for production debug engineers, and an IEEE 24765 / Avizienis-Laprie-
Randell taxonomy anchor — no ad-hoc names.

| Tier | Source | Motifs | Provenance |
|------|--------|-------:|------------|
| 1 | Original framework | 10 | `FrameworkDesign` |
| 2 | TADBench fault cases | 5 | `DatasetObserved` |
| 3 | AIOps Challenge categories | 6 | `DatasetObserved` |
| 4 | MultiDim-Localization patterns | 3 | `DatasetObserved` |
| 5 | DeepTraLog log+trace fusion | 3 | `DatasetObserved` |
| 6 | Cross-cutting structural | 5 | `FrameworkDesign` |
| **Total** | | **32** | |

`MAX_MOTIFS = 64` leaves 32 free slots for v0.3 / v0.4 expansion. The
bank exposes two lookup paths: per-signal `lookup` and per-episode
`match_episode_with_consensus` (multi-feature weighted scoring with
topology / duration / consensus gates). LO2 endoductive anomalies are
deliberately NOT in the bank — they validate the
`SemanticDisposition::Unknown` branch.

See [`docs/heuristics_bank.md`](docs/heuristics_bank.md) for the
per-motif reference.

## Detector Orthogonality

The fusion harness wires **205 deterministic detectors** organised across
**27 mathematical axes** (Tiers A–U + EXTRA + V/X/Y/Z/AA). Each axis
contributes a distinct mathematical foundation; orthogonal combinations
form the consensus arithmetic and route through per-motif tier-affinity
masks per the **Routed Evidence Principle** (paper §6.5).

| Tier | Family | Count | Examples | Mathematical Axis |
|------|--------|------:|----------|-------------------|
| A | Tier-A parametric trio | 3 | scalar, CUSUM, EWMA | fixed-baseline / cumulative / smoothed |
| B | Robust statistics | 3 | robust-Z, Page-Hinkley, Tukey-IQR | MAD-based / min-CUSUM / IQR fence |
| C | Model / non-parametric | 5 | spectral residual, matrix profile, BOCPD, IsoForest, LOF | spectral / sub-sequence / Bayesian / tree / density |
| D | Additional non-dep | 5 | Mann-Kendall, rolling-Z, AR(1), Mahalanobis, KS-rolling | trend / adaptive / autoregressive / multivariate / distribution |
| E | Debugging-specific stats | 3 | Poisson burst, saturation chain, chi-sq proportion | Poisson tail / N-consecutive / proportion shift |
| F | Neuroscience burst | 4 | MaxInterval, LogISI, Rank-Surprise, MISI | inter-event-interval translated detectors |
| Extra | Session 8.5 | 5 | GLR, ADWIN, MEWMA, retry-storm, correlation-break | LR-test / Hoeffding / multivariate EWMA / domain |
| G | Concept drift streaming | 9 | Shiryaev-Roberts, DDM, EDDM, HDDM-A/W, STEPD, ECDD, KSWIN, FHDDM | error-rate sequential change-point |
| H | Distribution shift | 10 | Wasserstein, JS, KL, PSI, AD, CvM, Energy, MMD, Bhattacharyya, Hellinger | distributional distance |
| I | Robust / nonparametric | 10 | Median-abs-slope, Theil-Sen, Sen-CP, Mood, Brown-Forsythe, Levene, Sign, Runs, WW, Sequential-rank | rank-based / robust |
| J | Forecast residual | 10 | SES, Holt, Holt-Winters, AR(2), ARIMA, Kalman, SavGol, STL, Prophet, naive seasonal | forecast-residual envelope |
| K | Frequency / oscillation | 10 | FFT-band, Welch, Wavelet, autocorr, Lomb-Scargle, ZCR, dominant-freq, spectral-entropy, cepstral, phase-locking | spectral / temporal |
| L | Multivariate relationship | 9 | Hotelling T², MCUSUM, PCA-recon, Robust-PCA, corr-mat-dist, partial-corr, Laplacian, CCA, MI | covariance / subspace |
| M | Debugging-native | 18 | flap, sawtooth, deadband, quantization, plateau, counter-wrap, monotone-leak, hysteresis, limit-cycle, ping-pong, backpressure, causal-lag, fan-out, fan-in, phase-slip, jitter-bloom, tail-thickening, burst-after-silence | named bug-shapes |
| N | Offline CPD | 8 | PELT, BinSeg, BottomUp, Window, DP, Kernel, Piecewise-linear, Bayesian-offline | dynamic programming / segmentation |
| O | Rare changepoint | 10 | MOSUM, NOT, WBS2, Seeded-BS, SMUCE, FDRSeg, FPOP, TGUH, Inspect, Double-CUSUM-BS | multiscale / interval-randomized |
| P | Streaming / sequential | 9 | E-detector, conformal-martingale, exch-martingale, power-martingale, mixture-martingale, mixture-SPRT, scan-stat, higher-criticism, Berk-Jones | e-process / multi-test |
| Q | Concept drift rarer | 10 | MDDM-A/E/G, LFR, FPDD, OPTWIN, SeqDrift2, D3, QuantTree, NN-DVI | weighted Hoeffding / Fisher / discriminative |
| R | Robust depth | 8 | halfspace, projection, Stahel-Donoho, MCD, spatial-sign, S-est, depth-rank, median-polish | depth-based / direction-based |
| S | Count / event-process | 3 | Bayesian-blocks, Index-of-dispersion, Allan-variance | count-process / variance-time |
| T | Info-theoretic | 6 | MDL, NCD, Lempel-Ziv, transfer-entropy, Fisher-info, Renyi-entropy | description-length / surprise |
| U | Dynamical systems | 8 | permutation-entropy, sample-entropy, RQA, Lyapunov, correlation-dim, BDS, 0-1-chaos, delay-embedding-NN | nonlinear dynamics |
| V | Industrial fault-diagnosis (FDD) | ~8 | parity-space residual, observer-based, dependability-engineering | parity / observer |
| X | Climate homogeneity | ~8 | Buishand-range, SNHT, Pettitt-step, cumulative-deviation | cumulative-deviation tests |
| Y | Robust dispersion / rank | ~8 | median-of-means, U-statistic, rank-step | dispersion / rank-step |
| Z | Circular / directional | ~8 | R-bar, Rayleigh, circular-mean shift, phase-jump | phase / directional |
| AA | Higher-order nonlinear | ~11 | ARCH, kurtosis, second-moment volatility | non-Gaussian / heavy-tail |
| **Total** | | **~205** | + DSFB structural | **27 orthogonal axes** |

Each detector is implemented as a deterministic `pub fn` in
[`src/incumbent_baselines.rs`](src/incumbent_baselines.rs), citation in
the doc-comment. Where literature requires FFT/DFT, MCMC, or non-
deterministic randomization, the implementation uses a documented
deterministic-seed reduction or time-domain proxy — the simplification
is named honestly in each detector's doc-comment. See
[`docs/fusion_design.md`](docs/fusion_design.md) for the per-tier
notes and consensus contribution rules.

## Real-World Dataset Coverage — 12 Real-Bytes Fixtures Across 9 Datasets

All Phase F + Phase G fixtures are vendored as **real upstream byte
slices** (no synthetic data, ever; `paper-lock` hard-errors on missing
real data with `MissingRealData`). Every fixture is DOI-pinned, SHA-256-
gated, and replayable under Theorem 9.

| # | Dataset | Source | Distinguishing regime | Test file |
|---|---------|--------|-----------------------|-----------|
| 1 | **TADBench / TrainTicket F-11** | Zenodo `10.5281/zenodo.6979726` | order-service deployment-regression (35 604 spans) | [`tests/eval_tadbench.rs`](tests/eval_tadbench.rs) |
| 2 | **TADBench / TrainTicket F-11b** | same | auth-mongo fault — cross-fixture validation (108 spans) | same |
| 3 | **TADBench / TrainTicket F-04** | same | admin-service springstarter version-config (25 316 spans) | same |
| 4 | **TADBench / TrainTicket F-19** | same | mongodb-driver-3.0.4 version-config (19 281 spans) | same |
| 5 | **Illinois SocialNetwork** | DataBank `10.13012/B2IDB-6738796_V1` | unsampled DeathStarBench trace (160 000 traces) | [`tests/eval_illinois.rs`](tests/eval_illinois.rs) |
| 6 | **AIOps Challenge 2018 KPI** | NetManAIOps/Bagel `sample_data.csv` | KPI seasonal anomaly (Su et al., IPCCC 2018) | [`tests/eval_aiops_challenge.rs`](tests/eval_aiops_challenge.rs) |
| 7 | **LO2 (PROMISE 2025)** | Zenodo `10.5281/zenodo.14257989` | OAuth2 Go-runtime metrics; **endoductive validator** | [`tests/eval_lo2.rs`](tests/eval_lo2.rs) |
| 8 | **MultiDim-Localization** | NetManAIOps/MultiDimension-Localization | 5-dim categorical aggregate (high-dim service graph) | [`tests/eval_multidim.rs`](tests/eval_multidim.rs) |
| 9 | **DeepTraLog F-01** | FudanSELab/DeepTraLog | combined log+trace ERROR span data (Zhang et al., ICSE 2022) | [`tests/eval_deeptralog.rs`](tests/eval_deeptralog.rs) |
| 10 | **Defects4J** *(Phase G)* | rjust/defects4j | Java bug catalog — 6 projects × 30 bugs (Just et al., ISSTA 2014) | [`tests/eval_defects4j.rs`](tests/eval_defects4j.rs) |
| 11 | **BugsInPy** *(Phase G)* | soarsmu/BugsInPy | Python bug catalog — 6 projects × 30 bugs (Widyasari et al., FSE 2020) | [`tests/eval_bugsinpy.rs`](tests/eval_bugsinpy.rs) |
| 12 | **PROMISE defect-prediction** *(Phase G)* | ssea-lab/PROMISE | Per-module Chidamber-Kemerer OO metrics + bug counts (Menzies et al., 2003+) | [`tests/eval_promise.rs`](tests/eval_promise.rs) |

**Verbatim metrics** from the latest `cargo test --features "std paper-lock" -- --nocapture` run:

| Fixture | Episodes | Raw alerts | RSCR | Clean FP | Replay |
|---|---:|---:|---:|---:|:---:|
| TADBench F-11 (deployment-regression) | 3 | 11 | 3.67× | 0.0070 | ok |
| TADBench F-11b (auth-mongo) | 1 | 7 | 7.0× | 0.2500 | ok |
| TADBench F-04 (config regression) | 0 | 0 | 0 | 0 | ok |
| TADBench F-19 (mongodb-driver) | 0 | 0 | 0 | 0 | ok |
| Illinois SocialNetwork | 1 | 24 | 24.0× | 0.0313 | ok |
| AIOps Challenge KPI | 1 | 52 | 52.0× | 0.0323 | ok |
| LO2 OAuth2 (endoductive) | 0 | 0 | 0 | 0 | ok |
| MultiDim part1 | 1 | 19 | 19.0× | 0.0833 | ok |
| DeepTraLog F-01 | 0 | 0 | 0 | 0 | ok |
| Defects4J | 1 | 62 | 62.0× | 0.0333 | ok |
| BugsInPy | 0 | 0 | 0 | 0 | ok |
| PROMISE | 1 | 4 | 4.0× | 0.0333 | ok |

See [`data/MANIFEST.toml`](data/MANIFEST.toml) for per-dataset DOI,
license, and SHA-256 gates; [`docs/dataset_provenance.md`](docs/dataset_provenance.md)
for the full provenance ledger;
[`data/upstream/project_*.py`](data/upstream/) for each fixture's
deterministic projection script.

## 9-Axis Bank-Aware Fusion + Anti-Hallucination Ladder

The fusion harness is governed by a **nine-axis configuration** that
progressively tightens the typed-disposition decision. Each axis is a
single `FusionConfig` flag; each axis was introduced in a specific phase
to close a specific failure mode (paper §11.x).

| Axis # | Name | Phase | Role |
|:------:|------|:----:|------|
| 1 | Provenance gate | 0 | Filter motifs by `Provenance` rank |
| 2 | Margin gate | 2 | Demote to ambiguous when top-vs-runner-up margin below θ |
| 3 | Tier-affinity scoring | 2.5 | Per-motif `affinity_tiers` bitmask routes detector evidence |
| 4 | Zero-tier-firing filter | 5.5 | Reject motifs whose entire affinity mask saw no detector firings |
| 5 | Adaptive margin gate | 5.5 | Halve θ when `tier_consensus_factor > 0.5` |
| 6 | Confuser-boundary adjudication | 5.6 | Compute `margin_vs_confuser`; emit `ConfuserAmbiguous` below θ_c |
| 7 | Structural disambiguator boost | 6 | Bonus on per-motif `disambiguator_tier` |
| 8 | Tier-level primary witness | 7 | At least one detector in `primary_witness_tiers` must fire |
| 9 | Per-detector named witness | 8 | At least one of `primary_witness_detectors` must fire (vacuous-pass when zero captured) |

The **Anti-Hallucination Ladder** uses axes 6/8/9 as four progressive
strictness rungs (Phase 0 / 5.6 / 7 / 8). Operators choose the rung
appropriate for their fault profile; recall trades against typing
precision.

## Fusion Sweep Results — F-11

Captured from the `fusion_sweep_on_f11_fixture` test (35 604 real Jaeger
spans, 16 services, 431 windows). Per-window FP rate computed against
the healthy slice. Run `cargo test --features "std paper-lock" --test
fusion_compare -- --nocapture` for the verbatim matrix.

| Min consensus | Layer-2 episodes | Layer-2 FP rate | Layer-3 typed-confirmed | Determinism |
|--------------:|-----------------:|----------------:|------------------------:|:-----------:|
| N≥1 | 12 | 0.1230 | 3 | ok |
| N≥3 | 7 | 0.0302 | 3 | ok |
| N≥5 | 1 | 0.0093 | 1 | ok |
| **N≥7** | **1** | **0.0023** | **1** | **ok** |
| N≥9 | 0 | 0.0000 | 0 | ok |

Single-detector minima for reference (`tests/incumbent_compare.rs`):
scalar-3σ 0.0139, CUSUM 0.0116, EWMA 0.0812, DSFB-structural 0.0070.
At N≥7, fusion FP rate (0.0023) is ~3× lower than DSFB-structural alone,
~5× lower than CUSUM, ~6× lower than scalar-3σ, ~35× lower than EWMA.

**Honest claim:** F-11 + F-11b numbers are bounded to those fixtures.
Generalisation requires Phase II partner-data engagement.

## Standards Alignment

NIST SP 800-53 (AU-2/3/6/12), NIST SP 800-92, NIST SP 800-171,
DO-178C §6.3 (pathway-eligible foresight, not certification),
IEEE 1012-2016, ISO/IEC 25010, OpenTelemetry, W3C Trace Context,
SOC 2 Type II.

| Control | Mapped Component | Audit Path |
|---------|------------------|------------|
| AU-2 (Audit Events) | structured `BenchmarkMetrics` JSON output | `tests/fusion_compare.rs` stdout |
| AU-3 (Record Content) | per-motif `HeuristicEntry` (provenance, DOI, taxonomy) | `src/heuristics_bank.rs` |
| AU-6 (Review/Analysis) | episode catalog + similarity matching | `src/episode_catalog.rs` |
| AU-12 (Audit Generation) | `verify_deterministic_replay` mechanism | `src/lib.rs` |
| CMMC AU.L2-3.3.1/2 | content-of-records discipline | `docs/standards_alignment.md` |
| DO-178C §6.3 | pathway-eligible architectural alignment | observer-only contract type-enforced |

See [`docs/standards_alignment.md`](docs/standards_alignment.md) for
control-level mapping.

## Quick Verification

```
cargo build --no-default-features
cargo test  --no-default-features
cargo test  --features "std paper-lock" -- --nocapture
cargo clippy --features "std paper-lock" --all-targets -- -D warnings
```

## License

Apache 2.0 (reference implementation). Background IP: Invariant Forge LLC.
Commercial deployment requires separate written license.
Contact: licensing@invariantforge.net
