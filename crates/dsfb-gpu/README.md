[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-gpu/notebooks/dsfb_gpu_debug_colab.ipynb)

# dsfb-gpu


**Try it now (no install required) Click the Open in Colab badge above**
- Open the Colab notebook
- Click ► Run all

---



**Clear-box pure deterministic inference CUDA GPU Acceleration: residual evidence goes in,
a replayable verdict case file comes out. No neural network. No learned weights.
No probabilistic black box.**

DSFB-GPU is a RUST and CUDA based prior-art implementation of **densorial / tekmeric
inference**: deterministic evidence adjudication over residuals, signs,
detector motifs, consensus fields, candidate intervals, and bank-governed
episodes. The CUDA layer accelerates evidence production; the CPU court
keeps semantic authority. The output is not a prediction. It is a
hash-linked case file whose intermediate evidence can be replayed.

To our knowledge, DSFB-GPU is the first public working instance of GPU-accelerated pure deterministic inference: non-stochastic, non-probabilistic, non-neural, and replayable from residual evidence bytes to hash-linked verdict case files.

```text
trace / residual bytes
  → deterministic CUDA evidence factory
  → residual signs + detector motifs + consensus/candidates
  → CPU bank admission
  → replayable verdict case file
  → optional LLM narration over admitted evidence only
```
The core doctrine is simple:

> The GPU produces evidence. The court decides what that evidence is allowed
> to mean.

## What this repository proves

- **Deterministic GPU inference is possible without neural weights.**
  DSFB-GPU-Debug maps residual extraction, drift/slew signs, detector motif
  scoring, consensus formation, candidate collapse, bank-governed episode
  emission, and case-file assembly onto fixed-point CUDA + Rust court logic.

- **The reasoning chain is replayable.**
  Case files carry byte-stable hash chains over the evidence path instead of
  post-hoc explanations or probabilistic confidence scores.

- **The real-data audit surface is public and reproducible.**
  The repository includes vendored fixture paths, audit receipts, a Colab
  replay notebook, and metadata files so external evaluators can rebuild the
  CUDA path and inspect the replay artifacts.

- **The Atlas layer preserves the prior-art surface.**
  DSFB-GPU-Atlas is the deterministic jurisprudence court over detector
  literature, provenance, passports, precedents, admissibility grammar,
  activation decisions, execution receipts, challenge dockets, and
  coverage-hole reports.

- **The paper and repository are intentionally exhaustive.**
  This is a prior-art disclosure artifact. Detail is preserved on purpose:
  campaign ledgers, hashes, non-claims, receipts, and artifact manifests are
  part of the disclosure surface.

## What this is not

DSFB-GPU is not a neural network, not Bayesian inference, not stochastic
sampling, not an LLM, not an APM replacement, and not a black-box anomaly
score. The optional LLM role is downstream narration over admitted case-file
evidence; it never creates court evidence or changes admission status.

## Quick start

```bash
# CPU-only build
cargo build --workspace

# CUDA build
cargo build --workspace --features cuda

# Run the public Colab replay surface instead of configuring CUDA locally:
# use the "Open in Colab" badge above.
```

For the full command surface, see **Building**, **Running**, and
**Reproducibility** below.

## Evidence map

| surface | purpose |
|---|---|
| `paper/dsfb_gpu_debug.pdf` | prior-art paper and full disclosure narrative |
| `PRIOR_ART_MAP.md` | disclosed architecture elements mapped to code / tests / receipts |
| `CLAIM_BOUNDARY_MATRIX.md` | what is claimed, what is disclosed, and what is not claimed |
| `ARTIFACT_MANIFEST.v1.toml` | SHA-256-pinned artifact index |
| `TIMESTAMP_RECEIPT.md` | public-accessibility and archive receipt |
| `CITATION.cff`, `codemeta.json`, `.zenodo.json` | machine-readable citation / release metadata |
| `notebooks/dsfb_gpu_debug_colab.ipynb` | public replay path |
| `reports/` | sealed receipts, timing reports, replay verification, and audit artifacts |


## Citation

de Beer, R. (2026). *DSFB-GPU — Clear-Box Pure Deterministic
Inference CUDA Acceleration for Replayable Trace-Event Verdicts —
A Prior-Art Architecture for non-probabilistic, non-stochastic,
non-weighted, GPU-Accelerated Residual Signs, Detector Motifs,
Bank-Governed Fusion, and Byte-Exact Case Files Without
Probabilistic Models* (v1.0). Zenodo.
[https://doi.org/10.5281/zenodo.20346478](https://doi.org/10.5281/zenodo.20346478).

Machine-readable citation metadata:
[`CITATION.cff`](CITATION.cff), [`codemeta.json`](codemeta.json),
[`.zenodo.json`](.zenodo.json). Timestamp receipt:
[`TIMESTAMP_RECEIPT.md`](TIMESTAMP_RECEIPT.md).

This repository carries two complementary layers:

- **DSFB-GPU-Debug (sealed at R.13)** — the bounded prior-art proof
  that artificial inference over structured trace evidence can be
  GPU-accelerated *without* becoming a probabilistic black box. The
  reasoning chain (residual fields, drift/slew signs, detector
  motifs, consensus, candidate intervals, bank-governed episodes,
  verdict case files) is deterministic, hash-chained, and
  operator-auditable end to end. Measured ~55× full-pipeline
  campaign reduction at D64 on RTX 4080 SUPER / CUDA 13.2; see
  *Performance — current headline* below.
- **DSFB-GPU-Atlas (T.1–S1.3g, S1.3a–S1.3g)** — the *emerging*
  frontier built on top of the sealed acceleration proof: a
  host-only, zero-dependency, deterministic jurisprudence court
  over a 54-record literature detector corpus with provenance-
  bound dedup, genealogy, witness-role + L-band ladder,
  usefulness-ledger schema, court precedents, admissibility
  grammar, trial transcripts, execution-attestation receipts,
  challenge docket, contraindication receipts, coverage-hole
  report, reason-coded activation plan (S1.3a), per-detector
  activation-decision transcripts + ActivationDiffV1 (S1.3b),
  TaskManifestV1 + DatasetManifestV1 + ActivationContextV1
  binding decisions to declared task / dataset / units /
  sampling-law contracts (S1.3c), the CorpusAmendmentProposal
  intake scaffold (T.12.0), the first real expansion proposal
  — Statistical Process Control (T.12.a: MEWMA + MCUSUM
  canonicals, Q-stat/SPE/Hotelling-T-sq aliases, Western
  Electric + Nelson composition reclassifications, four
  genealogy edges, four source refs), AND the second real
  expansion proposal — Sequential Change Detection (T.12.b:
  Shiryaev-Roberts + GLR + Binary segmentation + PELT-style
  deterministic canonicals at reserved ids 5201/5202/5207/5208;
  seven `ExistingCanonicalAuthorityResolution` records keeping
  CUSUM / Page-Hinkley / Mann-Kendall / Pettitt / SNHT / MOSUM /
  Buishand range canonical without duplication; one
  `DomainTransferOf` for CUSUM as shared SCD ancestor; BOCPD
  acknowledged but rejected as `RejectedNotDeterministic`),
  AND the third real expansion proposal — Drift Detection and
  Distribution-Distance Authority (T.12.c: Kuiper + ADWIN + DDM
  + HDDM canonicals at reserved ids 5301..=5304; eleven
  `ExistingCanonicalAuthorityResolution` records for every
  existing SEED distribution-distance primitive — KS / KL / MMD
  / Anderson-Darling / Cramer-von Mises / Wasserstein / Energy
  distance / Hellinger / PSI / Jensen-Shannon / Total variation;
  one `DomainTransferOf` for KS as shared two-sample distance
  ancestor; two `ParameterizationOf` records — EDDM of DDM,
  KSWIN of KS — the new wire-name category lands for the first
  time), AND the fourth real expansion proposal — Robust
  Statistics (T.12.d: Theil-Sen + biweight midvariance +
  trimmed mean shift + winsorized mean shift canonicals at
  reserved ids 5401..=5404 with declared estimator laws; three
  `ExistingCanonicalAuthorityResolution` records for robust-z /
  Hampel filter / Tukey fences with declared windowed-local-
  median / quartile / IQR-multiplier laws; one
  `DomainTransferOf`; three `ParameterizationOf` records —
  modified z-score / rolling Hampel / k×IQR fence — and one
  `RejectedNotDeterministic` for the randomized RANSAC
  residual proxy; the first proposal to exercise all five
  plan-locked court-delta categories), AND the fifth real
  expansion proposal — Signal Processing / Spectral / Wavelet
  (T.12.e: spectral centroid shift + wavelet packet energy +
  STFT ridge shift + cepstral anomaly + matched filter
  residual + Hilbert amplitude anomaly canonicals at reserved
  ids 5501..=5506 with declared transform-law contracts; five
  `ExistingCanonicalAuthorityResolution` records for FFT band-
  energy / residual envelope exit / spectral entropy / wavelet
  coefficient energy / autocorrelation break; one
  `DomainTransferOf`; three `ParameterizationOf` records — FFT
  bandpower / wavelet family / STFT window-hop — and one
  `RejectedNotDeterministic` for randomized spectral
  projection (Rahimi & Recht 2007 random Fourier features)),
  AND the sixth real expansion proposal — Time-Series
  Structure / Control Residuals (T.12.f: AR / ARIMA / STL
  residual + lag-correlation break + variance-ratio shift +
  run-length anomaly + observer residual + parity-space
  residual canonicals at reserved ids 5601..=5608 with
  declared model-and-decision-law contracts; six
  `ExistingCanonicalAuthorityResolution` records for residual
  envelope exit / sensor bias / actuator stiction / valve
  hunting / autocorrelation break / Error burst; one
  `DomainTransferOf`; three `ParameterizationOf` records —
  innovation sequence / periodicity break / burstiness index —
  and one `RejectedNotDeterministic` for unidentified-model
  anomaly), AND the seventh real expansion proposal — Graph /
  Topology Anomaly (T.12.g: degree spike + betweenness shift
  + clustering-coefficient shift + PageRank residual +
  edge-cut anomaly + bridge-node emergence + cascade precursor
  + motif-count anomaly canonicals at reserved ids 5701..=5708
  with declared graph-model / baseline / update-law / metric-
  law / decision-law contracts; one
  `ExistingCanonicalAuthorityResolution` for Fanout cascade
  (SEED 43); one `DomainTransferOf`; three
  `ParameterizationOf` records — weighted-degree spike /
  k-hop fanout / directed motif-count; AND TWO
  `RejectedNotDeterministic` records — community boundary
  shift and random-walk embedding anomaly — the FIRST T.12.x
  proposal with two rejection records in one commit), AND the
  eighth real expansion proposal — Data Quality / Tabular /
  Database Integrity Constraints (T.12.h: functional-dependency
  violation + type instability + target-leakage candidate
  (plan-locked "candidate, not proof") + correlation break +
  covariance shift + null-run anomaly + range envelope exit
  tabular + category emergence canonicals at reserved ids
  5801..=5808 with declared scope / baseline / null / type /
  key / range / association / decision-law contracts; five
  `ExistingCanonicalAuthorityResolution` records for SEED 13 /
  44 / 45 / 46 / 47; one `DomainTransferOf` for Missingness
  spike as the shared data-quality ancestor; three
  `ParameterizationOf` records — per-column missingness /
  composite-key uniqueness / category collapse; AND TWO
  `RejectedNotDeterministic` records — learned data-quality
  anomaly score and auto-schema inference anomaly — the
  SECOND T.12.x proposal with two rejection records in one
  commit), AND the ninth real expansion proposal —
  Observability / Debugging (T.12.i: retry storm + queue-depth
  pressure + saturation precursor + cold-start transient +
  timeout burst + GC pause spike + thread-pool exhaustion +
  backpressure propagation canonicals at reserved ids
  5901..=5908 with declared telemetry-field / aggregation-law /
  window / baseline / topology-scope / threshold / confuser-
  profile contracts; FIVE `ExistingCanonicalAuthorityResolution`
  records for the dsfb-gpu-debug bank surface — SEED 14 / 15 /
  41 / 42 / 43, the L6 GPU-implemented set — protecting the L6
  honesty marker from re-canonicalisation; TWO
  `DomainTransferOf` records for Fanout cascade and Error burst
  as shared ancestors for `ObservabilityDebugging`; FOUR
  `ParameterizationOf` records — HTTP 5xx burst / p95/p99
  latency ramp / k-hop dependency fanout / retry-rate burst;
  AND TWO `RejectedNotDeterministic` records — vendor APM
  black-box anomaly score (Datadog / New Relic / Dynatrace /
  Splunk MLTK / AWS DevOps Guru) and learned incident
  classifier (PagerDuty / Splunk On-Call / ServiceNow AIOps) —
  the THIRD T.12.x proposal with two rejection records in one
  commit), AND the tenth real expansion proposal — Medical /
  Biosignal (T.12.j: P-wave morphology + T-wave morphology +
  QT interval + PR interval + spectral HRV band shift +
  baseline wander + motion artifact + saturation/clipping
  canonicals at reserved ids 6001..=6008 with declared signal-
  source / sampling-rate / filtering-law / morphology-or-
  interval-measurement-law / baseline / artifact-confuser-
  profile / decision-functional contracts; FOUR
  `ExistingCanonicalAuthorityResolution` records for the SEED
  biosignal set — R-peak interval anomaly 49 / HRV time-domain
  shift 50 / QRS width anomaly 51 / ST-segment deviation proxy
  52; TWO `DomainTransferOf` records for FFT band-energy
  (SEED 12) as shared spectral ancestor and Residual envelope
  exit (SEED 22) as shared envelope-boundary ancestor for
  `MedicalBiosignal`; FOUR `ParameterizationOf` records —
  RR-interval irregularity / HRV SDNN-RMSSD-pNN50 / HRV LF-HF
  / lead-specific ST; AND TWO `RejectedNotDeterministic`
  records — learned arrhythmia classifier (Hannun et al. 2019
  deep-learning ECG) and clinician-label-only diagnostic rule
  — the FOURTH T.12.x proposal with two rejection records in
  one commit; plan-locked non-claim: **T.12.j does not admit
  medical diagnoses** — it admits deterministic biosignal
  witnesses under declared sampling, filtering, and measurement
  laws; pinned by the
  `t12_j_rejects_diagnostic_claim_language` parametric scanner
  — all without mutating SEED or `corpus_hash_v1`), AND the
  eleventh real expansion proposal — Industrial / Fault
  Detection and Diagnostics / Condition Monitoring (T.12.k:
  EIGHT existing-canonical authority resolutions for the
  industrial SEED set — FFT band-energy 12 / PCA T² 19 / PCA
  SPE/Q 20 / PLS residual 21 / Residual envelope exit 22 /
  Sensor bias 23 / Actuator stiction 24 / Valve hunting 25,
  the LARGEST SEED-collision ratification of any T.12.x to
  date; only SIX new canonicals at 6101..=6106 — Kalman
  innovation whiteness witness per Mehra & Peschon 1971,
  operating-regime transition witness, condition-indicator
  drift, fault signature angle, contribution-plot spike,
  spectral kurtosis per Antoni 2006 — with declared plant /
  observer / residual / model / state-machine / latent-space /
  estimator / envelope / computation contracts AND decision-
  law contracts; TWO `DomainTransferOf` records for SEED 12
  and SEED 22 as shared spectral / envelope-boundary ancestors
  for `FaultDetectionDiagnostics`; FOUR `ParameterizationOf`
  records collapsing plan-candidate primitives that don't
  survive the SEED-walk (bearing vibration band-energy 6107 →
  SEED 12; motor current signature 6108 → SEED 12; temperature
  envelope excursion 6109 → SEED 22; pressure transient 6110 →
  SEED 42 Slew shock); AND TWO `RejectedNotDeterministic`
  records — proprietary PdM black-box score (GE Predix /
  Siemens MindSphere / IBM Maximo Predict / Honeywell Forge /
  Aspen Mtell) and learned fault classifier (Wen 2017 / Khan &
  Yairi 2018) — the FIFTH T.12.x proposal with two rejection
  records in one commit; plan-locked non-claim: **T.12.k
  admits deterministic condition-monitoring / FDD witnesses,
  NOT root-cause certainty and NOT maintenance
  recommendations** — pinned by both
  `t12_k_rejects_root_cause_claim_language` (parametric
  scanner of forbidden root-cause / RUL / failure-mode-
  classification terms) AND
  `t12_k_rejects_fault_detector_without_plant_or_residual_contract`
  (every CanonicalAddition must declare math-structure +
  decision-functional contract) — all without mutating SEED
  or `corpus_hash_v1`), AND the twelfth real expansion proposal
  — Chemometrics (T.12.l: FOUR existing-canonical authority
  resolutions for the chemometric SEED set — PCA T² 19 / PCA
  SPE/Q 20 / PLS residual 21 / Residual envelope exit 22; FIVE
  new canonicals at 6201..=6205 — calibration residual
  witness, leverage outlier (hat-matrix sample-influence),
  concentration drift, SIMCA class-distance per Wold & Sjostrom
  1977, VIP shift per Wold 1995 — with declared preprocessing /
  latent / calibration / residual / score / hat-matrix /
  per-class / VIP / model contracts AND decision-functional
  contracts; TWO `DomainTransferOf` records for SEED 19 and
  SEED 22 as shared latent-space / envelope-boundary ancestors
  for `Chemometrics`; FOUR `ParameterizationOf` records
  collapsing plan-candidate primitives that don't survive the
  SEED-walk (PCA score outlier 6206 → SEED 19; Mahalanobis-on-
  scores 6207 → SEED 19; LV control chart 6208 → SEED 20;
  spectral preprocessing artifact 6209 → SEED 22); AND TWO
  `RejectedNotDeterministic` records — black-box spectroscopy
  classifier (Bruker AI-IDENT / Mettler-Toledo Spectraline /
  Thermo Scientific OMNIC ML / Agilent MicroLab AI) and
  adaptive-AutoML / stochastic-CV chemometric pipeline (auto-
  sklearn / H2O AutoML / TPOT) — the SIXTH T.12.x proposal
  with two rejection records in one commit; plan-locked
  non-claim: **T.12.l admits deterministic chemometric signal
  witnesses, NOT material identification certainty and NOT
  regulatory compliance verdicts** — pinned by both
  `t12_l_rejects_material_identification_claim_language`
  (parametric scanner of forbidden material-identification /
  chemical-causation / lab-diagnosis terms) AND
  `t12_l_rejects_regulatory_compliance_claim_language`
  (parametric scanner of FDA / ISO / regulatory-verdict terms)
  AND
  `t12_l_rejects_chemometric_detector_without_preprocessing_or_latent_model_contract`
  (every CanonicalAddition must declare math-structure +
  decision-functional contract) — all without mutating SEED
  or `corpus_hash_v1`), AND the thirteenth real expansion
  proposal — RF / Communications (T.12.m: SIX existing-
  canonical authority resolutions for the spectral + envelope
  + entropy + correlation + carrier-offset + modulation-quality
  SEED family that RF heavily reuses — FFT band-energy 12 /
  Residual envelope exit 22 / Spectral entropy 38 /
  Autocorrelation break 40 / Carrier-frequency-offset residual
  53 (Morelli & Mengali 1999 OFDM CFO estimator) / Error
  Vector Magnitude anomaly 54 (Shafik / Rahman / Islam 2006);
  SIX new canonicals at 6303..=6308 — constellation spread
  (second-moment per cluster, not first-moment per-symbol —
  distinct from SEED 54 EVM), channel impulse response (CIR)
  drift (system response to declared impulse, not signal
  autocorrelation), IQ imbalance, phase-noise per Razavi 1996,
  symbol-timing offset residual per Gardner / early-late, and
  cyclostationary feature shift per Gardner 1987 (with
  DECLARED cycle frequencies, not implicit autocorrelation) —
  with declared signal representation / sampling law / unit
  law / carrier or channel assumption / synchronization
  assumption / window-or-transform law / decision functional /
  confuser profile / numeric mode contracts; reserved ids
  6301 and 6302 deliberately unused because the CFO and EVM
  ideas they once shadowed collapsed onto SEED 53 and SEED 54
  respectively under the SEED-walk-first discipline; TWO
  `DomainTransferOf` records for SEED 12 and SEED 22 as shared
  spectral / envelope-boundary ancestors for `RfCommunications`;
  FOUR `ParameterizationOf` records collapsing plan-candidate
  primitives that don't survive the SEED-walk (spectral mask
  violation 6309 → SEED 12 with ITU-R SM / ETSI EN / FCC
  Part 15 emission-mask law; SNR drop 6310 → SEED 12 with
  signal-vs-noise band partition; burst preamble miss 6311 →
  SEED 40 with cross-correlation template against known
  preamble; frame-error burst 6312 → SEED 41 Error burst with
  IEEE 802.11 / IEEE 802.15.4 / 3GPP LTE / 5G NR frame format
  + CRC / FEC decode law); AND TWO `RejectedNotDeterministic`
  records — learned RF fingerprinting classifier (Restuccia
  2019 DeepRadioID / Sankhe 2019 ORACLE / Wang 2022 RF-based
  device identification) and black-box modulation classifier /
  proprietary spectrum-anomaly score (Keysight signal-analysis
  ML / Rohde & Schwarz spectrum monitoring AI / NI RFIC
  analyser ML / Ettus USRP-based learned pipelines) — the
  SEVENTH T.12.x proposal with two rejection records in one
  commit; plan-locked non-claim: **T.12.m admits deterministic
  RF signal witnesses, NOT emitter attribution, transmitter
  identification, geolocation, spectrum-enforcement authority,
  military classification, or communications-intelligence
  conclusions** — pinned by THREE parametric scanners
  (`t12_m_rejects_emitter_identification_claim_language`,
  `t12_m_rejects_geolocation_or_attribution_claim_language`,
  `t12_m_rejects_spectrum_enforcement_claim_language`) AND
  `t12_m_rejects_rf_detector_without_signal_or_sampling_contract`
  (every CanonicalAddition must declare signal representation
  + sampling + carrier-or-channel + window/transform +
  decision functional) — all without mutating SEED or
  `corpus_hash_v1`), AND the fourteenth real expansion
  proposal — Econometrics + Reliability / Survival (T.12.n:
  combined campaign because the two domains share structural-
  break / CUSUM / envelope-residual ancestry; FOUR existing-
  canonical authority resolutions over the structural-change +
  envelope SEED family — CUSUM 3 / Page-Hinkley 4 / Mann-
  Kendall 11 / Residual envelope exit 22; EIGHT new canonicals
  at 6401..=6408 — four econometric (GARCH volatility residual
  per Bollerslev 1986 against conditional-variance model;
  cointegration-break per Hansen 1992 / Quintos-Phillips 1993
  with CUSUM-of-squared-residuals on cointegration residual;
  Hausman-test residual per Hausman 1978 chi-squared on
  parameter-difference vector; Bai-Perron multiple-break
  detector per Bai-Perron 1998 / 2003 with information-
  criterion + Quandt-Andrews supremum-F) plus four reliability
  / survival (Kaplan-Meier survival residual per Kaplan-Meier
  1958 with declared censoring + time-origin; Cox proportional-
  hazards / Schoenfeld residual per Cox 1972 / Schoenfeld
  1982 / Grambsch-Therneau 1994; Weibull failure-rate
  envelope exit per Weibull 1951 with declared shape + scale
  + MLE; Crack-growth law residual per Paris-Erdogan 1963
  with stress-intensity-range model and C / m parameters) —
  with declared stationarity + window + regression / hazard
  model + censoring law (where applicable) + time-origin law
  (where applicable) + residual definition + decision-
  functional contracts; TWO `DomainTransferOf` records for
  SEED 3 (structural-change ancestor) and SEED 22 (envelope-
  boundary ancestor) across both `Econometrics` and
  `ReliabilitySurvival`; FOUR `ParameterizationOf` records
  collapsing plan-candidate primitives that don't survive
  the SEED-walk (CUSUM-of-recursive-residuals 6409 per Brown-
  Durbin-Evans 1975 → SEED 3; Quandt-Andrews / Chow
  structural-break F-test 6410 per Quandt 1960 / Chow 1960 /
  Andrews 1993 → SEED 4; hazard-rate change 6411 → SEED 22;
  cumulative damage residual 6412 per Palmgren 1924 / Miner
  1945 linear cumulative damage rule → SEED 3); AND TWO
  `RejectedNotDeterministic` records — learned market
  predictor / black-box financial forecaster (Bloomberg AIM /
  AlphaSense / Kavout / Goldman SecDB ML / JP Morgan COIN)
  and learned RUL classifier / black-box predictive-
  maintenance score (Uptake AI / C3.ai / Senseye / IBM Maximo
  / Siemens MindSphere) — the EIGHTH T.12.x proposal with two
  rejection records in one commit; plan-locked non-claim:
  **T.12.n admits deterministic econometric, reliability,
  survival, and degradation witnesses, NOT market prediction,
  investment advice, credit-decision authority, actuarial
  pricing authority, causal economic certainty, RUL certainty,
  maintenance recommendations, or failure-time prediction** —
  pinned by THREE parametric claim-language scanners
  (`t12_n_rejects_market_prediction_claim_language`,
  `t12_n_rejects_investment_or_credit_decision_claim_language`,
  `t12_n_rejects_rul_or_failure_time_certainty_claim_language`)
  AND TWO contract scanners
  (`t12_n_rejects_econometric_witness_without_stationarity_or_window_contract`,
  `t12_n_rejects_survival_witness_without_censoring_or_time_origin_contract`)
  AND a black-box-forecaster-without-formula scanner — all
  without mutating SEED or `corpus_hash_v1`), AND the
  fifteenth real expansion proposal — Streaming Sketches
  (T.12.o: FOUR existing-canonical authority resolutions for
  the KS + missingness + error-burst + cardinality SEED
  family — KS 8 / Missingness spike 13 / Error burst 41 /
  Cardinality drift 46; EIGHT new canonicals at 6501..=6508
  (Count-Min sketch residual per Cormode-Muthukrishnan 2005,
  HyperLogLog cardinality shift per Flajolet-Fusy-Gandouet-
  Meunier 2007, Bloom-filter membership anomaly per Bloom
  1970, Misra-Gries heavy-hitter shift per Misra-Gries 1982,
  Space-Saving heavy-hitter shift per Metwally-Agrawal-El
  Abbadi 2005 distinct from Misra-Gries via replace-smallest-
  on-miss, Greenwald-Khanna quantile summary drift per
  Greenwald-Khanna 2001 with deterministic epsilon-approximate
  quantile guarantee, t-digest summary residual per Dunning
  2019 with declared DETERMINISTIC centroid-merge law, AMS
  moment sketch per Alon-Matias-Szegedy 1999 with 4-wise-
  independent hash family) with declared hash family / width
  / bucket count / depth / per-row or per-sketch seed / merge
  law / update order / error-bound semantics / residual
  definition / decision-functional contracts; TWO
  `DomainTransferOf` records for SEED 46 Cardinality drift
  and SEED 8 KS as shared cardinality and distribution-
  distance ancestors; FOUR `ParameterizationOf` records
  collapsing Flajolet-Martin / probabilistic-counting / LogLog
  cardinality estimator (6509) → SEED 46; streaming
  approximate KS via quantile sketch (6510) → SEED 8;
  sliding-window error-burst sketch (6511) → SEED 41; sketch-
  approximate missingness via Bloom inversion (6512) → SEED
  13; AND TWO `RejectedNotDeterministic` records — learned
  streaming-anomaly score (Datadog Watchdog AI / DataRobot
  Streaming AutoML / Splunk Stream ML / AWS Lookout for
  Metrics / Azure Anomaly Detector) and black-box approximate-
  streaming proprietary sketch without declared hash / width
  / depth / seed / merge contract (Snowflake APPROX_* /
  BigQuery APPROX_* / Druid approximate aggregators /
  ClickHouse uniqHLL12 / quantileTDigest / topK / AWS Athena
  APPROX_*) — the NINTH T.12.x proposal with two rejection
  records in one commit; plan-locked non-claim: **T.12.o
  admits deterministic streaming-sketch witnesses, NOT
  probabilistic accuracy as certainty, randomized sketch
  behavior without seed / width / depth / hash-family
  declaration, privacy claims, database correctness
  authority, or approximate-query truth** — pinned by SIX
  scanners
  (`t12_o_rejects_sketch_without_hash_family_width_depth_or_seed_contract`,
  `t12_o_rejects_probabilistic_error_bound_as_deterministic_certainty`,
  `t12_o_rejects_approximate_query_truth_claim_language`,
  `t12_o_rejects_privacy_or_anonymization_claim_language`,
  `t12_o_rejects_mergeable_sketch_without_merge_law`,
  `t12_o_rejects_black_box_streaming_anomaly_score_without_formula`)
  — all without mutating SEED or `corpus_hash_v1`), AND the
  sixteenth real expansion proposal — Information Theory catch-
  up (T.12.p: THREE existing-canonical authority resolutions
  for the KL + JS + Spectral-entropy SEED family — KL 9 /
  JS 32 / Spectral entropy 38; FIVE new canonicals at
  6601..=6605 (Shannon entropy shift per Shannon 1948,
  Conditional entropy shift per Cover-Thomas 2006, Mutual
  information break per Cover-Thomas 2006 structurally distinct
  from SEED 9 KL because MI is a functional on the JOINT vs
  PRODUCT-OF-MARGINALS whereas KL is a divergence between two
  declared distributions, Cross-entropy / negative-log-
  likelihood residual per Shannon 1948 with FIXED MODEL
  distribution parameter-pinned and frozen across the
  comparison window, Minimum description length / coding-
  length residual per Rissanen 1978 / 1986 with declared two-
  part code and L(D | M) + L(M) decomposition) with declared
  estimator / binning or partition law / empty-bin law /
  smoothing / sample-support bound / log base / joint-
  distribution contract / bias-correction rule contracts; TWO
  `DomainTransferOf` records for SEED 9 KL as shared
  information-theoretic divergence ancestor and SEED 38
  Spectral entropy as shared Shannon-entropy-on-distribution
  ancestor; FOUR `ParameterizationOf` records collapsing
  Normalized MI (6606) → MI 6603; Transfer entropy proxy per
  Schreiber 2000 (6607) → MI 6603 admitted ONLY AS A
  DETERMINISTIC NON-CAUSAL WITNESS; Rényi-Tsallis entropy per
  Rényi 1961 / Tsallis 1988 (6608) → Shannon entropy 6601 with
  declared order-alpha parameter law AND limit-recovery;
  Compression-ratio anomaly per Ziv-Lempel 1977 / 1978 / Welch
  1984 LZW (6609) → MDL 6605; AND TWO `RejectedNotDeterministic`
  records — learned mutual-information estimator (MINE Belghazi
  et al. 2018 / InfoMax / variational MI bounds / neural KL
  estimators / InfoVAE / CPC contrastive predictive coding MI
  lower bounds) and black-box information-theoretic anomaly
  score (AWS Macie information-leakage scoring / IBM Guardium
  DAM information-theoretic anomaly heuristics / Microsoft
  Purview information-leakage classifier / Symantec / Broadcom
  DLP entropy-based anomaly score / Cisco Talos information-
  theoretic threat scoring) — the TENTH T.12.x proposal with
  two rejection records in one commit; plan-locked non-claim:
  **T.12.p admits deterministic information-theoretic
  witnesses, NOT semantic meaning, causal information flow
  certainty, privacy leakage certainty, cryptographic security
  claims, or learned representation claims** — pinned by SIX
  scanners
  (`t12_p_rejects_information_witness_without_estimator_or_binning_contract`,
  `t12_p_rejects_entropy_detector_without_base_smoothing_and_empty_bin_law`,
  `t12_p_rejects_mutual_information_without_joint_distribution_contract`,
  `t12_p_rejects_causal_information_flow_claim_language`,
  `t12_p_rejects_privacy_or_security_claim_language`,
  `t12_p_rejects_learned_embedding_information_score_without_formula`)
  — all without mutating SEED or `corpus_hash_v1`), AND the
  T.12.consolidate META-hash freeze layer (ratification not
  expansion: loads every T.12.0..T.12.p proposal, verifies
  every proposal hash / batch hash / dedup-delta hash by
  recomputation, walks every dedup record across all 17
  proposals, enforces TEN plan-required negatives — missing
  proposal / duplicate reserved id / unused-reserved-id pin /
  SEED collision / parameterization-without-parent /
  authority-without-target / rejection-without-contract /
  hash-mismatch / SEED-or-corpus_hash_v1-mutation /
  uncredited-literature-record — builds the sorted T.12
  expansion index (98 entries spanning 5001..=6699 with
  T.12.m's 6301 + 6302 deliberately-unused slots verified
  absent), emits THREE new own-namespace hashes
  (`consolidation_report_hash_v1`,
  `t12_expansion_index_hash_v1`, `corpus_hash_v2`).
  Aggregate court delta across T.12.a..T.12.p: 98
  CanonicalAddition + 76 ExistingCanonicalAuthorityResolution
  + 23 DomainTransferOf + 49 ParameterizationOf + 24
  RejectedNotDeterministic + 1 T.12.a-era AliasOf + 2 T.12.a-
  era CompositionOf = 273 total dedup-court records.
  `corpus_hash_v2` is the **ratified-corpus AUTHORITY** anchor;
  it META-hashes `corpus_hash_v1` + the consolidation report
  + the expansion index + sorted admitted canonical ids +
  SEED length. Plan-locked non-claim: **T.12.consolidate
  does NOT add new literature primitives, does NOT mutate
  SEED, does NOT mutate `corpus_hash_v1`, does NOT promote
  individual proposals to Accepted**. The transition is from
  "proposal court" to "ratified corpus authority"; per-
  proposal migration into a new SEED table is a separate
  future commit gated on individual ProposalStatus::Accepted
  ratifications), AND the FF.1 passport-materialisation layer
  (the first ratification campaign above corpus_hash_v2;
  materialises one DetectorPassport per ratified
  CanonicalAddition entry — 98 passports spanning canonical
  ids 5001..=6699 — by pulling the T.12 expansion index
  read-only and deriving operational fields: display name,
  source class, origin proposal, GPU-family wire name from
  the plan-locked SourceClass-to-GpuFamilyKernel mapping,
  activation-applicability tags from the plan-locked
  SourceClass-to-tag mapping, contraindication-linkage stub,
  challenge-surface stub; emits THREE new own-namespace hash
  layers — per-passport `passport_hash_v1` under
  `DSFB-GPU-ATLAS:FF1-T12-RATIFIED-PASSPORT:v1\0`,
  `ff1_passport_index_hash_v1` under
  `DSFB-GPU-ATLAS:FF1-PASSPORT-INDEX:v1\0`, and
  `ff1_materialisation_report_hash_v1` under
  `DSFB-GPU-ATLAS:FF1-MATERIALISATION-REPORT:v1\0`. Plan-
  locked non-claim: **FF.1 does NOT reopen T.12 dedup
  decisions, does NOT add new literature primitives, does
  NOT alter corpus_hash_v1 or corpus_hash_v2, does NOT
  rewrite any historical T.12 proposal hash or any
  T.12.consolidate hash, does NOT mutate SEED, does NOT
  activate any detector, does NOT decide contraindications
  or challenges (stubs reserve the slot for later commits),
  does NOT generate CUDA kernels** — pinned by TEN scanners
  (`ff1_rejects_passport_for_non_ratified_canonical_id`,
  `ff1_rejects_passport_if_corpus_hash_v2_mismatch`,
  `ff1_rejects_passport_materialisation_that_mutates_t12_proposal_hash`,
  `ff1_rejects_passport_materialisation_that_mutates_corpus_hash_v2`,
  `ff1_rejects_duplicate_passport_for_same_canonical_id`,
  `ff1_rejects_missing_source_lineage_for_literature_passport`,
  `ff1_rejects_missing_gpu_family_mapping`,
  `ff1_rejects_missing_activation_applicability_tags`,
  `ff1_rejects_missing_contraindication_linkage_stub`,
  `ff1_rejects_missing_challenge_surface_stub`)), AND the FF.2
  activation ratification gate (the META-discipline layer above
  S1.3a + FF.1; adds `DisabledReason::DisabledUnratifiedProposal`
  so the activation court can explicitly disable detector
  proposals lacking corpus_hash_v2 ratification + FF.1 passport
  authority — replacing the pre-FF.2 silent `DisabledByWeakLBand`
  fallback the plan warning forbids; classifies every candidate
  canonical id into four mutually-exclusive buckets
  (`SeedHistorical` / `T12RatifiedAndPassported` /
  `MissingPassport` / `UnratifiedProposal`); emits TWO new own-
  namespace hash layers — `ff2_activation_ratification_gate_hash_v1`
  under `DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1\0`
  and `ff2_activation_ratification_gate_summary_hash_v1` under
  `DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE-SUMMARY:v1\0`;
  pinned by SIX plan-required negatives
  (`ff2_rejects_activation_for_unratified_proposal`,
  `ff2_rejects_activation_for_missing_ff1_passport`,
  `ff2_rejects_activation_when_passport_index_hash_mismatch`,
  `ff2_rejects_unratified_proposal_without_reason_code`,
  `ff2_rejects_silent_fallback_to_disabled_by_weak_lband`,
  `ff2_rejects_activation_reason_without_corpus_hash_v2_binding`)),
  AND the FF.3 registry-generation gate (the second META-
  discipline layer above S1.3a + FF.1 + FF.2; teaches the S1.2
  registry generator to refuse any `DetectorSpec` whose source
  authority is not a SEED canonical record under
  `corpus_hash_v1` OR a `corpus_hash_v2`-ratified entry
  materialised through FF.1 passport authority; classifies
  every candidate into one of seven mutually-exclusive
  `Ff3RegistryGenerationEligibility` buckets (`Eligible` /
  `RejectedUnratifiedProposal` / `RejectedMissingFf1Passport`
  / `RejectedCorpusHashV2Mismatch` /
  `RejectedPassportIndexHashMismatch` / `RejectedAdHocRecord`
  / `RejectedUnknownSourceAuthority`); emits TWO new own-
  namespace hash layers —
  `ff3_registry_generation_gate_hash_v1` under
  `DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1\0` and
  `ff3_registry_generation_gate_summary_hash_v1` under
  `DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE-SUMMARY:v1\0`;
  pinned by EIGHT plan-required negatives
  (`ff3_rejects_detector_spec_for_unratified_proposal`,
  `ff3_rejects_detector_spec_for_missing_ff1_passport`,
  `ff3_rejects_detector_spec_when_corpus_hash_v2_mismatch`,
  `ff3_rejects_detector_spec_when_passport_index_hash_mismatch`,
  `ff3_rejects_detector_spec_from_ad_hoc_record`,
  `ff3_rejects_detector_spec_with_unknown_source_authority`,
  `ff3_rejects_registry_generation_that_skips_ff2_ratification_gate`,
  `ff3_rejects_registry_generation_that_mutates_existing_registry_hash`)),
  AND the FF.4 README authority-boundary policy (a
  communication-hygiene seal pinning a canonical 19-line
  authority-boundary block + a 6-entry required-substring set
  + a 7-entry forbidden-substring set; the live README sweep
  test reads `README.md` from disk and verifies every required
  substring is present and every forbidden substring is absent;
  emits ONE new own-namespace hash layer —
  `ff4_readme_authority_boundary_policy_hash_v1` under
  `DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0`;
  pinned by SEVEN plan-required negatives
  (`ff4_readme_rejects_stale_future_ratification_language`,
  `ff4_readme_requires_corpus_hash_v1_historical_anchor_language`,
  `ff4_readme_requires_corpus_hash_v2_ratified_authority_language`,
  `ff4_readme_requires_ff1_passport_materialisation_language`,
  `ff4_readme_requires_ff2_ff3_unratified_rejection_language`,
  `ff4_readme_rejects_claim_that_t12_proposals_mutated_seed`,
  `ff4_readme_rejects_claim_that_ff1_mutated_corpus_hash_v2`);
  plan-locked one-line verdict: *"FF.4 makes the authority
  boundary unmissable at the front door; it does not move any
  boundary."*),
  AND the FF.5 ProposalSchemaUpgradePolicy (the forward-looking
  governance policy defining how proposal schema upgrades may
  re-render historical proposal artifacts without erasing the
  old artifact hashes or confusing the court lineage; core rule
  is *"schema upgrade != silent artifact rewrite"*; pins a
  10-line doctrine + an empty migration table (no schema
  upgrades have happened yet) + the six upstream anchor hashes;
  emits THREE new own-namespace hash layers —
  `proposal_schema_upgrade_policy_hash_v1` under
  `DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1\0`,
  `proposal_schema_migration_table_hash_v1` under
  `DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1\0`, and
  per-receipt `schema_upgrade_receipt_hash_v1` under
  `DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1\0`; pinned by NINE
  plan-required negatives
  (`ff5_rejects_schema_rerender_without_old_hash`,
  `ff5_rejects_schema_rerender_without_new_schema_hash`,
  `ff5_rejects_schema_rerender_without_migration_table`,
  `ff5_rejects_schema_rerender_without_reason`,
  `ff5_rejects_migration_table_with_duplicate_old_hash`,
  `ff5_rejects_migration_table_with_duplicate_new_hash`,
  `ff5_rejects_claim_that_old_artifact_hash_was_invalid`,
  `ff5_rejects_schema_upgrade_that_mutates_corpus_hash_v1`,
  `ff5_rejects_schema_upgrade_that_mutates_corpus_hash_v2_without_freeze_campaign`);
  plan-locked non-claim: does NOT add new detectors, does NOT
  alter any upstream hash anchor, does NOT itself perform any
  schema upgrade — it is a forward-looking governance artifact
  pinning the contract future upgrades MUST satisfy).
  The Atlas is the prior-art lane:
  **idempotent forensic pre-inference for fixed artifacts**,
  with replayable case files the LLM consumes after the
  deterministic court has spoken.

Plan-locked anchor:

> **DSFB-GPU-Atlas makes LLM inference cheaper by moving first-pass
> evidence discovery out of the probabilistic model and into an
> idempotent deterministic evidence court.** The LLM, when used,
> consumes admitted evidence — it never generates it.

## Authority boundary (post-T.12.consolidate + FF.1 + FF.2 + FF.3)

Important authority-state note. T.12.a..T.12.p were amendment proposals.
They did not mutate SEED, corpus_hash_v1, registry_hash_v2, historical
DetectorPassports, or activation outputs while they were filed.

T.12.consolidate ratified the accepted T.12 expansion set and froze
corpus_hash_v2 as the first post-amendment corpus authority.

FF.1 then materialized 98 ratified T.12 CanonicalAddition entries into
T12RatifiedPassport records under ff1_passport_index_hash_v1.

FF.2 and FF.3 now enforce that activation and registry generation consume
only SeedHistorical records or T12RatifiedAndPassported records. Unratified,
non-passported, ad-hoc, or unknown-source records are rejected by explicit
reason code (DisabledUnratifiedProposal at activation; RejectedUnratifiedProposal,
RejectedMissingFf1Passport, RejectedCorpusHashV2Mismatch, RejectedPassportIndexHashMismatch,
RejectedAdHocRecord, RejectedUnknownSourceAuthority at registry generation).

- SEED and corpus_hash_v1 remain the historical seed-corpus anchor.
- T.12 proposals did not mutate seed authority while filed.
- T.12.consolidate froze corpus_hash_v2 as ratified post-amendment authority.
- FF.1 materialized ratified T.12 additions into passports.
- FF.2 / FF.3 prevent unratified records from entering activation or registry generation.

## Densorial / Tekmeric Inference (front-door identity)

DSFB-GPU is not a neural inference stack.

Neural inference, as used in machine learning, applies learned
weights to input tensors and emits probabilistic predictions,
embeddings, logits, or generated outputs.

DSFB-GPU implements **densorial / tekmeric inference**:

- **Densorial inference** is deterministic inference over
  **evidence densors**: fixed, typed, hashable data objects
  produced from residuals, telemetry, traces, signals, tables,
  or other observed fields. A densorial pipeline applies declared
  deterministic witness functions to those densors and emits
  canonical witness bytes, reason codes, transcripts, and case
  files.
- **Tekmeric inference** is evidence-based deterministic
  adjudication: the process of deriving admissible, replayable,
  challengeable conclusions from declared witnesses, provenance,
  contraindications, activation decisions, coverage holes, and
  hash-linked court records. Tekmeric inference is not learned
  prediction; it is structured evidence adjudication.

In short:

```
neural inference:
  tensor → learned weights → probabilistic output

densorial / tekmeric inference:
  densor → deterministic witness court → replayable case file
```

DSFB-GPU does **not** replace neural inference. It can run
before neural or LLM systems as an idempotent forensic pre-
inference layer: first-pass evidence discovery is moved into a
deterministic court, and the probabilistic model receives a
compact, cited, replayable case record instead of raw
unstructured evidence.

### Vocabulary contrast

| term | meaning in DSFB-GPU | not this |
|---|---|---|
| Tensor | general numeric array used by ML/GPU systems | evidence object by itself |
| Densor | deterministic evidence object with typed meaning, contract, and hash role | learned embedding |
| Neural inference | learned-weight inference over tensors | replayable evidence court |
| Densorial inference | deterministic witness execution over densors | neural prediction |
| Tekmeric inference | evidence-based deterministic adjudication | classifier confidence |
| Witness | declared deterministic evidence function output | model opinion |
| Case file | hash-linked replayable court record | post-hoc explanation |

## CUDA Evidence Factory (front-door identity)

DSFB-GPU includes a CUDA acceleration path for deterministic
evidence production.

The CUDA layer does not perform neural inference, black-box
anomaly detection, or semantic admission. It acts as a byte-
exact evidence factory: residual densors are processed by
deterministic detector-family kernels into canonical witness
bytes, breach flags, candidate summaries, and stage digests.

```
residual densors
  → CUDA deterministic witness families
  → witness densors / candidate summaries / stage digests
  → CPU court admission
  → replayable case file
```

The design target is a GPU workload shaped like evidence
rendering:

- device-resident residual / evidence densors
- compact detector-family kernels
- fixed numeric contracts
- canonical output byte layouts
- deterministic reduction order
- no semantic authority on the GPU
- hash-linked shard and case-file receipts

**The GPU produces evidence. The court decides what that
evidence is allowed to mean.**

Identity sentence (verbatim, plan-locked):

> DSFB-GPU is a CUDA-accelerated deterministic evidence court:
> byte-exact witness-family kernels shade residual densors into
> canonical evidence bytes, then a CPU-side jurisprudence layer
> admits, challenges, contraindicates, and records them into
> replayable case files.

### Non-claims (plan-locked)

DSFB-GPU does NOT:

- replace neural inference;
- claim peak memory-bandwidth saturation on any GPU
  (saturation is an explicit later performance campaign);
- claim production CUDA performance (R.13 sealed the
  full-pipeline campaign reduction; deployment-scale
  saturation remains a future S-PERF milestone);
- grant semantic authority to the GPU (the Semantic Non-
  Bypass Axiom keeps admission on the CPU bank);
- perform live OpenTelemetry ingestion (S1.3g declares the
  binding receipt schema only — no collector, no socket, no
  OTel SDK runtime dependency);
- turn OTel binding receipts into runtime adapters.

## Scientific Provenance Credit Pass (T.12.PROV)

DSFB-GPU-Atlas does not erase prior detector science. It
preserves named scientific lineage while converting detector
primitives into deterministic, replayable witness records.

T.12.PROV walks every T.12.a..T.12.p `CanonicalAddition` (98
total) through the live `consolidate::load_all_t12_proposals`
loader and emits three artifacts:

- a **scientist credit index** — one row per
  `CanonicalAddition` carrying the detector name, source class,
  origin proposal id, source-ref citation keys, dedup-court
  contribution text, proposed-primitive motivation text, and
  the plan-locked DSFB credit note;
- a **source bibliography index** — one entry per unique
  `(citation_key, source_class)` pair (133 total) carrying
  title / year / venue / source class / origin proposal id;
- a **provenance credit report** — META-hashing both indexes
  plus `corpus_hash_v1`, `SEED.len()`, and per-class record
  counts (98 canonical / 133 bibliography / 24 rejection / 49
  parameterization).

Every credit row carries the same plan-locked credit note
verbatim:

> DSFB-GPU-Atlas canonizes, deduplicates, normalizes,
> contracts, and activates this detector primitive into a
> deterministic, replayable witness record. DSFB-GPU-Atlas
> does not claim invention of this primitive; named scientists
> and source papers above carry the original credit.

The verifier enforces eight plan-required load-bearing
negatives (every `CanonicalAddition` must have a matching
`ProposedPrimitive`; every batch must carry at least one
`ProposedSourceRef`; every reason text must be non-empty for
canonical / rejection / parameterization records; every
source_ref_key on a credit row must appear in the originating
proposal's source-ref list; engineering-practice records with
year=0 must carry a non-empty venue) plus a case-insensitive
forbidden-substring scanner that refuses any text claiming
"dsfb invented", "we invented", "originally introduced by
dsfb", or similar.

Three new own-namespace hashes:
`scientist_credit_index_hash_v1` under
`DSFB-GPU-ATLAS:SCIENTIST-CREDIT-INDEX:v1\0`,
`source_bibliography_index_hash_v1` under
`DSFB-GPU-ATLAS:SOURCE-BIBLIOGRAPHY-INDEX:v1\0`, and
`provenance_credit_report_hash_v1` under
`DSFB-GPU-ATLAS:PROVENANCE-CREDIT-REPORT:v1\0`.

Plan-locked one-line verdict (verbatim):

> The identity commit says what DSFB-GPU is; T.12.PROV makes
> sure the scientists whose methods became court witnesses are
> visibly credited.

T.12.PROV does NOT mutate any prior corpus / T.11 / T.12.x /
T.12.consolidate / FF.x / S1.3.x hash anchor; does NOT alter
`SEED.len()` (stays at 54); does NOT change court decisions;
does NOT emit detector outputs / episodes; does NOT generate
CUDA kernels; does NOT modify the registry crate. T.12.PROV
records scientific provenance; it does not change court state.

## S-PERF.1 — DeviceTrafficReceiptV1 (measurement law)

S-PERF.1 defines the byte-accounting receipt every future
DSFB-GPU memory-bandwidth or saturation claim MUST cite. It
does not claim bandwidth saturation; it creates the
measurement law.

Plan-locked core rule (eight lines):

1. No bandwidth claim without byte accounting.
2. No peak-percentage claim without declared device bandwidth.
3. No CUDA timing claim without CUDA event timing.
4. No Layer-A claim if host JSON / report time is included.
5. No cross-device comparison without device identity.
6. No effective bandwidth when total accounted bytes equals zero.
7. No percent-of-peak above 100 without explicit error flag.
8. Every receipt MUST declare contract hashes.

`DeviceTrafficReceiptV1` carries 22 hashable fields: device
identity (`device_name`, `device_uuid_or_identity_hash`,
`sm_arch`, `driver_version`, `cuda_version`); bandwidth
posture (`theoretical_memory_bandwidth_gbps`,
`measured_kernel_time_us`, `timing_method`, `layer`);
workload (`detector_count`, `catalog_count`); byte
accounting (`input_bytes`, `evidence_bytes_read`,
`evidence_bytes_written`, `witness_bytes_written`,
`fusion_bytes_read_written`, `digest_bytes_read`,
`candidate_summary_bytes`, `total_accounted_device_bytes`);
effective claim (`effective_bandwidth_gbps`,
`percent_of_peak_basis_points`,
`accounting_overflow_acknowledged`); and anchors
(`artifact_hashes`, `contract_hashes`).

Companion enums encode the timing posture
(`TimingMethod::{CudaEvent, CudaStreamSync, HostInstantOnly,
HostJsonInclusiveTime, Unknown}`) and the bandwidth-accounting
layer (`DeviceBandwidthLayer::{LayerA, LayerB, LayerC}`).
The verifier treats any receipt with
`percent_of_peak_basis_points >= 8000` (80.00 % of peak) as
a saturation claim and requires CUDA-event or CUDA-stream-sync
timing.

Two new own-namespace hashes:
`device_traffic_receipt_hash_v1` under
`DSFB-GPU-ATLAS:DEVICE-TRAFFIC-RECEIPT:v1\0`, and
`device_bandwidth_claim_policy_hash_v1` under
`DSFB-GPU-ATLAS:DEVICE-BANDWIDTH-CLAIM-POLICY:v1\0`. A third
domain separator
(`DSFB-GPU-ATLAS:DEVICE-IDENTITY:v1\0`) governs the
deterministic device-identity hash used as a stand-in for
`cudaDeviceGetUuid`.

The S-PERF.1 baseline ships the RTX 4080 SUPER reference
host with every measurement field zero — a known-good
receipt that exercises the schema without claiming any
measured bandwidth. Later S-PERF.* commits replace these
zeros with measured values.

Plan-locked one-line verdict (verbatim):

> T.12.PROV made the science creditable; S-PERF.1 makes
> future CUDA performance claims accountable.

S-PERF.1 does NOT claim bandwidth saturation on any GPU;
does NOT claim production CUDA performance; does NOT
benchmark B300 / GB300 hardware (B300 / GB300 hardware
benchmarking is deferred to a later post-S-PERF / S-MG
victory-lap campaign, gated on the remaining S-PERF +
S-MG legs); does NOT change any CUDA kernel; does
NOT change any court decision; does NOT mutate any upstream
hash anchor; does NOT alter `SEED.len()` (stays at 54);
does NOT emit detector outputs or episodes; does NOT
generate CUDA kernels; does NOT modify the registry crate.

## S-PERF.2 — Layer-A resident densor pipeline

S-PERF.2 builds the first Layer-A device-resident evidence
pipeline so traffic receipts can measure GPU evidence
production without host JSON, report rendering, transcript
construction, or court explanation time mixed in. It does
not claim bandwidth saturation; it isolates the GPU
evidence-factory path the S-PERF.1 ruler will measure.

Core rule (plan-locked):

> Layer-A measures evidence production only:
> EvidenceDensor → WitnessDensor → FusionDensor →
> CandidateDensor + digests. No host-side transcript. No
> JSON/report timing. No CaseFileV2 materialization. No
> semantic admission timing.

S-PERF.2 ships three composable receipt types:

- `LayerAResidentPipelineV1` declares the stage sequence
  (five canonical stages: EvidenceDensorProjection /
  WitnessDensorEvaluation / FusionDensorReduction /
  CandidateDensorCollapse / StageDigestEmission), the
  per-densor residency policy (Evidence / Witness / Fusion
  `DeviceResidentOnly`; Candidate / StageDigest
  `DeviceResidentWithCompactD2H` with caps 2 048 / 160 bytes
  per catalog), and five forbidden-host-activity flags
  (`casefile_materialization_present`, `host_transcript_present`,
  `host_json_emission_present`, `semantic_admission_present`,
  `mutates_court_authority_hashes` --- all must be `false`).
- `LayerADeviceResidencyReceiptV1` declares per-densor H2D /
  D2H byte accounting for one pipeline run. The verifier
  rejects any `DeviceResidentOnly` densor with non-zero D2H
  bytes (plan-locked negative #4: full-witness-D2H-dump).
- `LayerATrafficReceiptV1` META-hashes pipeline + residency
  receipt + a referenced S-PERF.1 `DeviceTrafficReceiptV1`
  (by hash, with timing-method wire name carried alongside
  for verification) + the court-authority hash anchors the
  pipeline promises to keep stable.

Three new own-namespace hashes:
`layer_a_resident_pipeline_hash_v1` under
`DSFB-GPU-ATLAS:LAYER-A-RESIDENT-PIPELINE:v1\0`;
`layer_a_device_residency_receipt_hash_v1` under
`DSFB-GPU-ATLAS:LAYER-A-DEVICE-RESIDENCY-RECEIPT:v1\0`;
and `layer_a_traffic_receipt_hash_v1` under
`DSFB-GPU-ATLAS:LAYER-A-TRAFFIC-RECEIPT:v1\0`. All
deterministic across two builds; pairwise distinct.

The verifier enforces eight plan-required load-bearing
negatives plus structural defect rules (empty pipeline_id,
empty stage_names, duplicate densor kind, HostMaterialized
class in Layer-A, residency-receipt pipeline-hash mismatch,
D2H exceeds declared cap, missing `corpus_hash_v1` anchor in
the court-authority list).

The S-PERF.2 baseline composes the plan-locked Layer-A
pipeline + an uninstrumented residency receipt (every
per-densor H2D / D2H byte zero, but per-densor lists fully
populated for all five kinds so accounting is present) + the
S-PERF.1 baseline `DeviceTrafficReceiptV1` reference + the
`corpus_hash_v1` anchor. Later S-PERF.* commits replace the
zeros with measured values from real device runs.

Plan-locked one-line verdict (verbatim):

> S-PERF.1 gave the ruler; S-PERF.2 isolates the GPU
> evidence-factory path the ruler will measure.

S-PERF.2 does NOT claim bandwidth saturation; does NOT
benchmark B300 / GB300 hardware; does NOT change any CUDA
kernel; does NOT change any court decision; does NOT mutate
any upstream hash anchor; does NOT alter `SEED.len()` (stays
at 54); does NOT emit detector outputs or episodes; does
NOT generate CUDA kernels; does NOT modify the registry
crate.

## S-PERF.3 — Public-data saturation bundle

S-PERF.3 defines the byte-pinned public artifact bundle
every future Layer-A saturation measurement is taken
against. It does not claim saturation, does not benchmark
throughput, and does not change kernels. It creates the
reproducible public-data workload surface that
S-PERF.4 / S-PERF.5 will measure.

Core rule (plan-locked):

> No saturation benchmark without a byte-pinned public-data
> bundle. No dataset claim without source, license / access
> status, hash policy, and fixed materialization recipe.

S-PERF.3 ships three composable receipt types:

- `PublicArtifactManifestV1` declares per-dataset identity
  (`dataset_id`, `display_name`), classification
  (`dataset_class`, `layer_a_role_mapping`), access posture
  (`access_note`, `license_or_access_status`, `usage_mode`),
  hash policy (`hash_policy_kind`,
  `per_artifact_sha256_count`, `source_archive_sha256`),
  synthetic flag (`is_synthetic`), and materialization
  recipe (`source_url_or_doi`, `local_path_template`,
  `materialization_steps`,
  `expected_bytes_after_materialization`,
  `deterministic_postprocess`,
  `requires_live_remote_fetch`).
- `DatasetMaterializationPolicyV1` carries the 8-line
  plan-locked policy doctrine pinned by its own hash.
- `PublicDataSaturationBundleV1` META-hashes every manifest
  (sorted ascending by dataset_id) + the policy + the
  bundle identity.

Three new own-namespace hashes:
`public_artifact_manifest_hash_v1` (one per dataset) under
`DSFB-GPU-ATLAS:PUBLIC-ARTIFACT-MANIFEST:v1\0`;
`dataset_materialization_policy_hash_v1` under
`DSFB-GPU-ATLAS:DATASET-MATERIALIZATION-POLICY:v1\0`; and
`public_data_saturation_bundle_hash_v1` under
`DSFB-GPU-ATLAS:PUBLIC-DATA-SATURATION-BUNDLE:v1\0`. All
deterministic across two builds; pairwise distinct.

The plan-locked baseline bundle covers all five
plan-named dataset classes with five citation-only
manifests: TADBench (debug observability trace, Apache-2),
Defects4J v2 (software defect table, MIT), ADBench subset
(data-science tabular, BSD-2), TSB-UAD (time-series
anomaly, Apache-2), and NASA PCoE C-MAPSS (industrial
public fixture, public domain). Each declares
hash policy, materialization recipe, Layer-A role mapping
(`EvidenceDensorSource` for all five), and `is_synthetic =
false`. Later S-PERF.* commits flip `usage_mode` to
`MeasuredFixture` and populate per-file SHA-256 hashes.

The verifier enforces eight plan-required load-bearing
negatives. The case-insensitive benchmark-claim scanner
runs over 12 forbidden substrings ("achieves saturation",
"saturates the bandwidth", "% of peak", "outperforms",
"world record", "fastest gpu", "petaflops", etc.) on every
free-text field of every manifest and on the bundle
identifier; any match fires negative #8 (the bundle is for
defining workloads, not making benchmark claims).

Plan-locked one-line verdict (verbatim):

> S-PERF.2 isolated the evidence-factory path; S-PERF.3
> gives that path a reproducible public workload to run on.

S-PERF.3 does NOT claim memory-bandwidth saturation; does
NOT benchmark throughput; does NOT emit any timing
receipt; does NOT change any CUDA kernel; does NOT change
any court decision; does NOT mutate any upstream hash
anchor; does NOT alter `SEED.len()` (stays at 54); does
NOT emit detector outputs or episodes; does NOT generate
CUDA kernels; does NOT modify the registry crate; does NOT
download any dataset bytes (the baseline is citation-only;
live-remote fetches are forbidden by plan-required
negative #6).

## S-PERF.4 — Active-detector family compaction benchmark schema

S-PERF.4 defines how the 152 S1.3d-Active detectors are
compacted into the 14 GPU-family lanes from the S1.3e
KernelPlan for Layer-A measurement. It does not run the
benchmark, claim saturation, change CUDA kernels, or alter
activation / corpus authority. It defines the benchmark
schema, family grouping, parameter-table shape, and
compaction accounting that S-PERF.5 will measure.

Core rule (plan-locked):

> Detector count is not kernel count. Active witnesses
> must be family-compacted before performance claims are
> made.

S-PERF.4 ships three composable receipt types:

- `ActiveFamilyCompactionPlanV1` — per-family lane entries
  (sorted ascending by GPU family wire name) with active
  canonical-id lists, per-lane detector count,
  parameter-table offset, expected kernel name, and
  aggregate cost estimate. Pins four upstream anchor hashes:
  `source_budget_summary_hash` (S1.3d),
  `source_kernel_plan_hash` (S1.3e),
  `source_passport_index_hash` (FF.1), and `corpus_hash_v1`.
- `CompactedParameterTableReceiptV1` — per-family
  parameter-table byte size + total byte size + plan-locked
  `sort_order_wire_name = "CanonicalIdAscendingWithinFamily"`.
- `FamilyCompactionBenchmarkSchemaV1` — top-level META-hash
  envelope binding the plan + the parameter-table receipt
  + the S-PERF.2 Layer-A pipeline + traffic receipt hashes
  + the S-PERF.3 public-data bundle hash.

The plan-locked baseline is derived deterministically from
the live S1.3d `BudgetedActivationSummary` + S1.3e
`KernelPlanV1` + FF.1 passport index + S-PERF.2 Layer-A
traffic receipt + S-PERF.3 bundle, so the baseline cannot
drift from the production court state. The 14 family lanes
hold 152 active detectors: DistributionDistanceFamily (28),
SequentialRecurrenceFamily (26), SpectralFamily (23),
WindowStatisticFamily (20), ResidualObserverFamily (13),
TabularConstraintFamily (11), GraphLocalFamily (9),
ProjectionResidualFamily (9), ScalarThresholdFamily (6),
MissingnessFamily (2), RankStatisticFamily (2),
CategoricalHistogramFamily (1), NegativeWitnessFamily (1),
WaveletFamily (1).

Three new own-namespace hashes:
`active_family_compaction_plan_hash_v1` under
`DSFB-GPU-ATLAS:ACTIVE-FAMILY-COMPACTION-PLAN:v1\0`;
`compacted_parameter_table_receipt_hash_v1` under
`DSFB-GPU-ATLAS:COMPACTED-PARAMETER-TABLE-RECEIPT:v1\0`;
and `family_compaction_benchmark_schema_hash_v1` under
`DSFB-GPU-ATLAS:FAMILY-COMPACTION-BENCHMARK-SCHEMA:v1\0`.
All deterministic across two builds; pairwise distinct.

The verifier enforces eight plan-required load-bearing
negatives plus a case-insensitive 12-substring forbidden-
benchmark-claim scanner (mirrors S-PERF.3's set) on every
free-text field (`schema_id`, `plan_id`, `family_wire_name`,
`expected_kernel_name`). Negative #5 specifically rejects
any plan that counts detector variants as new canonicals
(the same canonical id appearing in more than one family
lane), enforcing the plan-locked rule "detector count is
not kernel count".

Plan-locked one-line verdict (verbatim):

> S-PERF.3 gives the evidence factory public data;
> S-PERF.4 packs the active court witnesses into
> benchmarkable GPU-family lanes.

S-PERF.4 does NOT run any benchmark; does NOT claim
memory-bandwidth saturation; does NOT emit any timing
receipt; does NOT change any CUDA kernel; does NOT change
any court decision; does NOT alter activation outcomes;
does NOT mutate any upstream hash anchor; does NOT alter
`SEED.len()` (stays at 54); does NOT emit detector outputs
or episodes; does NOT generate CUDA kernels; does NOT
modify the registry crate; does NOT download any dataset
bytes.

## S-PERF.5 — Effective-bandwidth report

> S-PERF.4 packs the active witnesses into benchmarkable
> lanes; S-PERF.5 turns measured Layer-A bytes and time
> into an admissible bandwidth report.

S-PERF.5 is the **verdict layer** of the post-T.12.consolidate
performance-discipline arc. It defines how a measured
`DeviceTrafficReceiptV1` (S-PERF.1) is combined with the
plan-locked Layer-A pipeline (S-PERF.2), public-data workload
bundle (S-PERF.3), and compacted active-family plan (S-PERF.4)
to produce an admissible effective-bandwidth report. It does
not run a benchmark; it judges whether a measurement is
allowed to make a claim.

**Core rule** (plan-locked):

> Effective bandwidth report ≠ saturation claim. Saturation
> requires accounted device bytes, admissible CUDA timing,
> declared device peak bandwidth, percent-of-peak ≥ 8000 bp,
> and a Layer-A-only timing boundary.

**Three composable receipt types**:

- `LayerABandwidthMeasurementV1` — raw measurement: cites
  the S-PERF.1 receipt hash, mirrors the device identity,
  carries theoretical peak GB/s, measured kernel time in
  microseconds, timing method wire name, total accounted
  device bytes, computed effective bandwidth, computed
  percent-of-peak in basis points, plus the LayerA
  forbidden-flag mirror (host JSON / casefile materialisation
  / host transcript flags, all `false`).
- `BandwidthClaimAdmissionV1` — verdict: claim kind
  (`NoClaim` / `EffectiveBandwidth` / `PercentOfPeak` /
  `Saturation`), plan-locked admissibility reason wire
  name, admitted boolean.
- `EffectiveBandwidthReportV1` — top-level META-hash binding
  the measurement + admission + four upstream anchor hashes.

**Three new own-namespace hashes** (none folded upstream):

- `layer_a_bandwidth_measurement_hash_v1 = 0554ec29…` under
  `DSFB-GPU-ATLAS:LAYER-A-BANDWIDTH-MEASUREMENT:v1\0`
- `bandwidth_claim_admission_hash_v1 = c45f8b88…` under
  `DSFB-GPU-ATLAS:BANDWIDTH-CLAIM-ADMISSION:v1\0`
- `effective_bandwidth_report_hash_v1 = a129d7e0…` under
  `DSFB-GPU-ATLAS:EFFECTIVE-BANDWIDTH-REPORT:v1\0`

All three deterministic across two builds; pairwise distinct;
distinct from every prior anchor.

**TEN plan-required load-bearing negatives**:

1. `s_perf_5_rejects_report_without_s_perf_1_receipt`
2. `s_perf_5_rejects_report_without_s_perf_2_layer_a_receipt`
3. `s_perf_5_rejects_report_without_s_perf_3_bundle_hash`
4. `s_perf_5_rejects_report_without_s_perf_4_compaction_hash`
5. `s_perf_5_rejects_saturation_claim_below_8000_bp`
6. `s_perf_5_rejects_saturation_claim_with_host_timing`
7. `s_perf_5_rejects_effective_bandwidth_mismatch_from_bytes_and_time`
8. `s_perf_5_rejects_report_that_includes_host_json_or_casefile_time`
9. `s_perf_5_rejects_cross_device_claim_without_device_identity`
10. `s_perf_5_rejects_benchmark_claim_without_public_artifact_manifest`

Plus 5 structural defect rules: `ReportIdEmpty`,
`AdmissibilityReasonEmpty`, `BenchmarkClaimInsideReport`
(case-insensitive 12-substring scanner mirroring S-PERF.3 /
S-PERF.4), `ClaimKindIncoherentWithMeasurement`, and
`InadmissibleClaimWithoutVerifierReason`. 66-test acceptance
suite.

**Saturation threshold** (plan-locked):
`S_PERF_1_SATURATION_BP = 8000` (80.00 % of theoretical
peak). Inherited from S-PERF.1; any saturation claim MUST be
backed by CUDA-event or CUDA-stream-sync timing AND the full
S-PERF.1/2/3/4 receipt chain AND a non-zero device identity
hash.

**Baseline posture**: the S-PERF.5 baseline report is
`BandwidthClaimKind::NoClaim` — every measurement field is
zero, mirroring the S-PERF.1 uninstrumented baseline. The
report exists to pin the receipt chain; later S-PERF.*
commits replace the zeros with measured values on real
hardware.

**Plan-locked non-claims**:

S-PERF.5 does NOT claim memory-bandwidth saturation at
baseline; does NOT run any benchmark; does NOT change any
CUDA kernel; does NOT change any court decision; does NOT
mutate any upstream hash anchor (every prior corpus / T.11.x
/ T.12.x / FF.x / S1.3.x / T.12.PROV / S-PERF.1 / S-PERF.2 /
S-PERF.3 / S-PERF.4 hash byte-identical); does NOT alter
`SEED.len()` (stays at 54); does NOT emit detector outputs
or episodes; does NOT generate CUDA kernels; does NOT decide
contraindications or challenges; does NOT modify the
registry crate; does NOT download any dataset bytes.

## S-PERF.6 — RTX 4080 SUPER measured CUDA pipeline baseline

> S-PERF.6 measures 13.33 GB/s on the RTX 4080 SUPER. That is
> 1.86 % of the 716 GB/s vendor-datasheet peak, not saturation.

S-PERF.6 records the measured RTX 4080 SUPER CUDA pipeline
result as a real bandwidth receipt. The corpus crate is
plan-locked host-only with zero CUDA dependency; the
measurement is captured by `dsfb-gpu-debug-cuda`'s existing
D64 throughput-pinned-async stage profiler
(`tests/r9_c_d64_stage_profile.rs::r9_c_d64_stage_profile_256x4096_k1`)
and written to `reports/d64_stage_timing_256x4096_K1.txt`. The
S-PERF.6 receipt mirrors those values into the corpus court.

**Core rule** (plan-locked):

> Measure first. Claim second. Claim exactly what was
> measured, and no more.

**Plan-locked report sentence** (verbatim):

> The RTX 4080 SUPER measured CUDA pipeline baseline reports
> 13.33 GB/s, approximately 1.86 % of the declared 716 GB/s
> theoretical memory-bandwidth anchor. This is an admissible
> measured CUDA pipeline bandwidth result, not a saturation
> claim.

**Plan-locked bottleneck sentence** (verbatim):

> The profile does not indicate memory-bandwidth saturation.
> The measured path is dominated by pipeline structure
> including tree_digest consensus and host-side
> feature/admission/finalization segments.

**Measured values** (sourced verbatim from
`reports/d64_stage_timing_256x4096_K1.txt`, RTX 4080 SUPER +
CUDA 13.2, D64 256×4096 K=1, median of 3 iters post-warmup):

| field | value |
|---|---|
| Hardware | RTX 4080 SUPER |
| CUDA version | 13.2 |
| Theoretical peak | 716 GB/s |
| Measured CUDA pipeline bandwidth | 13.33 GB/s |
| Percent of peak | 186 bp / 1.86% |
| Saturation threshold | 8000 bp / 80.00% |
| saturation_admitted | false |
| Dominant device stage | tree_digest consensus, 4 338 µs / 20.88 % |
| Host compute_features | 7 525 µs |
| Host bank admit + case finalize | 2 237 µs |

**Rounding law** (plan-locked: FLOOR):

```
percent_of_peak_basis_points
  = measured_wide_bandwidth_centi_gbps * 10000
    / (theoretical_memory_bandwidth_gbps * 100)
  = 1333 * 10000 / (716 * 100)
  = 13 330 000 / 71 600
  = 186.17...
  -> floor 186
```

**Three composable receipt types**:

- `Rtx4080SuperMeasuredCudaPipelineV1` — raw measured CUDA
  pipeline record pinned to the plan-locked RTX 4080 SUPER
  device identity (`"RTX 4080 SUPER"`, `sm_89`, 716 GB/s
  vendor-datasheet peak). Carries every measured stage
  timing, host segments, measured wide bandwidth, and
  source-report provenance path.
- `Rtx4080SuperMeasuredBandwidthClaimV1` — verdict
  (`claim_kind = MeasuredCudaPipelineBandwidth`,
  plan-locked admissibility reason wire name,
  `admitted = true`, `saturation_admitted = false`,
  threshold 8000 bp, observed 186 bp).
- `Rtx4080SuperMeasuredBaselineReportV1` — top-level
  META-hash binding the measurement + claim + four upstream
  anchor hashes (S-PERF.2 / S-PERF.3 / S-PERF.4 / S-PERF.5)
  + three R.12b episode-count integrity pins (13 / 89 /
  1917).

The module also pins the RTX 4080 SUPER device-identity
constants (`RTX_4080_SUPER_DEVICE_NAME`, `RTX_4080_SUPER_SM_ARCH`,
`RTX_4080_SUPER_THEORETICAL_PEAK_GBPS`) and the three R.12b
episode-count integrity constants, plus a local
`MeasuredCudaPipelineClaimKind` enum (so the variant does not
touch the S-PERF.5 `BandwidthClaimKind` enum, preserving all
prior S-PERF.5 hashes byte-identical).

**Three own-namespace hashes** (none folded upstream;
S-PERF.5 hashes byte-identical):

- `rtx4080_super_measured_cuda_pipeline_hash_v1 = a5b58bc8…`
  under
  `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-CUDA-PIPELINE:v1\0`
- `rtx4080_super_measured_bandwidth_claim_hash_v1 = 4fdf8699…`
  under
  `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BANDWIDTH-CLAIM:v1\0`
- `rtx4080_super_measured_baseline_report_hash_v1 = d44c9ec5…`
  under
  `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BASELINE-REPORT:v1\0`

All three deterministic across two builds; pairwise distinct;
distinct from every prior anchor.

**FOURTEEN plan-required load-bearing negatives** (plus 4
structural defect rules; 49-test acceptance suite):

1. `s_perf_6_rejects_zero_measured_bandwidth`
2. `s_perf_6_rejects_zero_device_total_time`
3. `s_perf_6_rejects_missing_source_report_path`
4. `s_perf_6_rejects_missing_rtx4080_super_identity`
5. `s_perf_6_rejects_percent_of_peak_arithmetic_mismatch`
6. `s_perf_6_rejects_saturation_claim_below_8000_bp`
7. `s_perf_6_rejects_claim_that_13_33_gbps_is_saturation`
8. `s_perf_6_rejects_claim_that_result_is_b300_or_gb300`
9. `s_perf_6_rejects_claim_that_result_is_production_performance`
10. `s_perf_6_rejects_rebaseline_of_r12b_episode_counts`
11. `s_perf_6_rejects_missing_tree_digest_stage_timing`
12. `s_perf_6_rejects_missing_host_segment_disclosure`
13. `s_perf_6_rejects_empty_claim_kind`
14. `s_perf_6_rejects_no_claim_baseline_for_measured_result`

A second test file
(`tests/s_perf_6_public_language_regression_check.rs`)
adds 8 plan-required public-language negatives that walk
this README + the paper + the `lib.rs` module docstring to
reject any drift toward apology branding, the deleted
`NoClaim` scaffold posture, removal of the
`saturation_admitted = false` disclosure, or removal of the
host-segment disclosure.

**Plan-locked non-claims**:

S-PERF.6 does NOT claim memory-bandwidth saturation (186 bp ≪
8 000 bp threshold); does NOT claim B300 / GB300 performance;
does NOT claim production CUDA performance; does NOT claim
Layer-A purity (the measured pipeline includes host-side
`compute_features` and bank-admit + finalize segments outside
what S-PERF.2 defines as Layer-A — both segments are honestly
disclosed in the `host_compute_features_us` and
`host_bank_admit_case_finalize_us` fields); does NOT generate
new detector results; does NOT change any CUDA kernel; does
NOT change any court decision; does NOT mutate any upstream
hash anchor (every prior corpus / T.11.x / T.12.x / FF.x /
S1.3.x / T.12.PROV / S-PERF.1 / S-PERF.2 / S-PERF.3 /
S-PERF.4 / S-PERF.5 hash byte-identical); does NOT alter
`SEED.len()` (stays at 54); does NOT emit detector outputs or
episodes; does NOT decide contraindications or challenges;
does NOT modify the registry crate; does NOT download any
dataset bytes; does NOT rebaseline the R.12b D64 saturation
pinned baseline; does NOT run the benchmark from inside the
corpus crate (the corpus stays plan-locked host-only with
zero CUDA dependency).

## S-PERF.7 — source-report import verifier

> S-PERF.7 makes the S-PERF.6 measurement chain
> mechanically empirical: the corpus crate now parses the
> on-disk bench source reports and rejects any drift
> between the parsed values and the plan-pinned S-PERF.6
> receipt constants.

Before S-PERF.7 a hand-edit of the `S_PERF_6_*` const
prefix in `s_perf_6_rtx4080_super_measured_cuda_pipeline.rs`
could silently drift away from disk and still pass every
existing test. S-PERF.7 closes that loop. The parser walks
the actual bytes of
`reports/d64_stage_timing_256x4096_K1.txt` and
`reports/r12_d64_saturation.txt`, extracts the measured
values + R.12b episode pins, and the verifier asserts they
match the S-PERF.6 receipt field-for-field. Any divergence
fires a plan-required negative at receipt-build time.

**What this DOES**:

- Parses `reports/d64_stage_timing_256x4096_K1.txt` into a
  typed `ParsedD64StageTimingV1` struct (`host_wall_median`,
  `device_total`, `consensus_grid_kernel_wide`,
  `tree_digest consensus`, both host segments, measured
  wide bandwidth in centi-GB/s, and the
  `episode_count` line at 256×4096 K=1).
- Parses `reports/r12_d64_saturation.txt` into a typed
  `ParsedR12bSaturationV1` struct carrying the three K=1
  episode pins (canonical 13 / mid 89 / full 1917).
- Builds a hashable
  `SourceReportImportVerifierReportV1` envelope binding
  both parsed reports + the S-PERF.6 baseline report hash
  + verifier provenance.
- `verify_source_reports_match_s_perf_6_baseline(...)`
  rejects any drift via four plan-required load-bearing
  negatives plus structural rules covering the non-required
  stage timings and cross-report episode-count consistency.

**What this DOES NOT do**:

- Does NOT run the bench (corpus crate is plan-locked
  host-only with zero CUDA dependency).
- Does NOT rewrite source reports.
- Does NOT mutate the S-PERF.6 receipt or any prior hash
  anchor.
- Does NOT alter `SEED.len()`.
- Does NOT rebaseline R.12b.

**One own-namespace hash** (none folded upstream):

- `source_report_import_verifier_hash_v1 = 99cc8a71…`
  under
  `DSFB-GPU-ATLAS:S-PERF-7-SOURCE-REPORT-IMPORT-VERIFIER:v1\0`

Deterministic across two builds; pairwise distinct from
every prior anchor.

**FOUR plan-required load-bearing negatives** (verbatim
from the directive):

1. `s_perf_7_rejects_receipt_if_source_report_bandwidth_differs`
2. `s_perf_7_rejects_receipt_if_source_report_device_total_differs`
3. `s_perf_7_rejects_receipt_if_source_report_host_segment_differs`
   (fires on either `host_compute_features_us` or
   `host_bank_admit_case_finalize_us`)
4. `s_perf_7_rejects_receipt_if_r12b_episode_pins_differ`
   (fires on any of the three pins disagreeing with
   13 / 89 / 1917)

Plus structural rules
(`SourceReportTreeDigestConsensusDiffers`,
`SourceReportConsensusGridDiffers`,
`SourceReportHostWallMedianDiffers`,
`SourceReportEpisodeCountDiffersFromFullPin`,
`CrossReportEpisodeCountInconsistent`, `VerifierIdEmpty`,
`D64SourceReportPathEmpty`, `R12bSourceReportPathEmpty`).
26-test acceptance suite covering all plan-required
negatives + parser tests + hash determinism + sensitivity
+ renderer byte-stability + a pinned-hash back-stop.

**CLI** (new):

```
dsfb-corpus s-perf-7-verifier         [--json] [--out PATH]
dsfb-corpus s-perf-7-verifier-emit    [--out-dir DIR]
```

`s-perf-7-verifier` exits non-zero if any drift is found,
which is the mechanically-empirical gate the S-PERF.6
constants are now anchored to.

**Track B linkage**: S-PERF.7 does NOT change measured
bandwidth (no kernel changes). It strengthens the
measurement chain so subsequent Track B legs can ratchet
the live measurement upward with the receipt automatically
tracking the bench output rather than drifting silently.

## S-PERF.8 — batched-K saturation receipt (S-PERF.8.1 hardening pass)

> S-PERF.8 replaces K-as-host-loop with batched-K execution
> and measures the effect. The result is mixed and
> informative: canonical 16×128 improves by 1.76×,
> confirming launch-amortization benefit on small fixtures,
> while full 256×4096 improves only +3.4% / 1.03×, showing
> that the full-scale path is not primarily launch-bound.
> This is a measured optimization result, not a saturation
> claim.

S-PERF.8 is the second Track B leg. It converts the
existing R.12b D64 saturation sweep — which already
processes K as a host loop of K serial single-catalog
dispatches on a hot `GpuWorkspace` — into a mechanically-
auditable corpus receipt. The corpus crate stays plan-
locked host-only with zero CUDA dependency; the parser
walks `reports/r12_d64_saturation.txt`'s K matrix and the
verifier asserts coherence against S-PERF.6.

The S-PERF.8.1 hardening pass extends the receipt with
plan-pinned execution-mode labels (dispatch mode, catalog
order, merge policy, CUDA Graph status), device identity,
the three R.12b episode pins (13 / 89 / 1917), per-scale
pre/post bandwidth + delta + interpretation label, and a
campaign-identity negative that mechanically rejects the
overclaim "canonical 16×128 got 1.76× speedup, therefore K
batching solved the full-scale workload."

**Honest measurement with bandwidth delta + interpretation
label** (live R.12b sweep, RTX 4080 SUPER, CUDA 13.2):

| scale | K=1 cat/sec | best K | best cat/sec | gain | delta | pre GB/s | post GB/s | interpretation |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| canonical 16×128 | 326.7 | 64 | 575.2 | 1.76× | +76.06% | 0.00 | 0.00 | LaunchBoundGainAtSmallFixture |
| mid 64×512 | 411.9 | 1 | 411.9 | 1.00× | +0.00% | 0.00 | 0.00 | NoFullScaleImprovement |
| full 256×4096 | 34.9 | 16 | 36.1 | 1.03× | +3.43% | 13.33 | 13.78 | ModestFullScaleGain |

The plan-pinned Track B target of 25–50 GB/s for S-PERF.8
presumed a launch-bound workload. The live data shows the
workload is device-bound at full-scale: per-cat time is
28 644 µs at K=1 vs 27 770 µs at K=128 — only 3.4 % gain
from amortising launch overhead because per-cat work
massively dominates per-launch overhead. Batched K at the
headline scale is NOT the lever to ratchet bandwidth from
13.33 GB/s toward saturation. The real lever lives in
S-PERF.10 (digest lane compaction; the 4 tree_digest
stages currently consume ~57 % of device time across the
plan-pinned profile and are the immediate next attack
target).

S-PERF.8's value is the auditability: every future Track B
leg can compare against the S-PERF.8 K-saturation pattern
to verify the new optimisation actually changes the
K-amortisation shape, not just the K=1 number.

**`BatchedKResultInterpretation` enum (plan-locked)**:

- `LaunchBoundGainAtSmallFixture` (≥ 1.5× gain)
- `ModestFullScaleGain` (1.01×..1.5× gain)
- `NoFullScaleImprovement` (1.00×..1.01× gain)
- `Regressed` (< 1.00× gain)

The classifier `from_gain_basis_points()` pins the four
thresholds (15000 / 10100 / 10000 bp); every per-scale
summary carries the label flowing from its measured gain.
The CAMPAIGN IDENTITY negative
`s_perf_8_rejects_canonical_launch_bound_gain_generalized_to_full_scale`
mechanically forbids the full 256×4096 result from
carrying `LaunchBoundGainAtSmallFixture` — the canonical
small-fixture gain does not generalize to full-scale.

**What this DOES**:

- Parses the K-saturation matrix from
  `reports/r12_d64_saturation.txt` (3 × 6 = 18 cells
  covering scales × K ∈ {1, 4, 16, 32, 64, 128}).
- Records each cell's `per_cat_us`, `cat/sec`,
  `features_pct`, `dev_total_pct`, `finalize_pct`, and
  top-stage label + percentage.
- Computes per-scale summaries (K=1 baseline, best K,
  best cat/sec, K-amortisation gain ratio, delta basis
  points, pre/post bandwidth, plan-locked interpretation
  label).
- Carries the four plan-pinned execution-mode labels
  (dispatch mode = host-loop K; catalog order = canonical
  scale-major then K-ascending; merge policy = no inter-
  catalog merge; CUDA Graph status = NOT engaged).
- Carries device identity (RTX 4080 SUPER + sm_89 +
  716 GB/s vendor peak) and the three R.12b episode pins.
- Binds the parsed table + summaries + execution-mode
  labels + device identity + episode pins + upstream
  S-PERF.6 baseline hash + S-PERF.7 verifier hash into a
  single hashable envelope.

**What this DOES NOT do**:

- Does NOT run the bench (corpus crate stays plan-locked
  host-only).
- Does NOT rewrite source reports.
- Does NOT claim the workload is true batched-K execution
  (R.12b is host-loop K; verifier rejects any contrary
  claim in `dispatch_mode_label`).
- Does NOT claim 25–50 GB/s as the S-PERF.8 result. The
  honest full-scale measurement is +3.4 % (13.33 → 13.78
  GB/s).
- Does NOT claim CUDA Graph or single-launch K is engaged.
- Does NOT mutate the S-PERF.6 receipt or any prior hash
  anchor.
- Does NOT alter `SEED.len()`.
- Does NOT rebaseline R.12b.

**One own-namespace hash** (rebaselined under S-PERF.8.1
schema upgrade; plan-acknowledged):

- `batched_k_saturation_receipt_hash_v1 = 37212c42…`
  under
  `DSFB-GPU-ATLAS:S-PERF-8-BATCHED-K-SATURATION-RECEIPT:v1\0`

Deterministic across two builds; pairwise distinct from
every prior S-PERF anchor; binds the live S-PERF.6
baseline-report hash AND the live S-PERF.7 source-report
verifier hash AND the plan-pinned execution-mode labels
AND device identity AND the three R.12b episode pins.

**FOURTEEN plan-required load-bearing negatives**:

1. `s_perf_8_rejects_receipt_with_incomplete_k_matrix`
2. `s_perf_8_rejects_receipt_if_k1_full_scale_per_cat_inconsistent_with_s_perf_6`
3. `s_perf_8_rejects_receipt_if_k1_cat_per_sec_arithmetic_mismatch`
4. `s_perf_8_rejects_receipt_if_k_amortisation_gain_exceeds_ceiling`
   (5× per-scale gain ceiling)
5. `s_perf_8_rejects_host_loop_k_claimed_as_batched`
6. `s_perf_8_rejects_missing_batched_k_source_report`
7. `s_perf_8_rejects_missing_pre_post_bandwidth_delta`
8. `s_perf_8_rejects_full_scale_claim_above_measured_delta`
9. `s_perf_8_rejects_claim_that_full_scale_reached_25gbps_if_it_did_not`
10. `s_perf_8_rejects_saturation_claim_below_8000bp`
11. **`s_perf_8_rejects_canonical_launch_bound_gain_generalized_to_full_scale`** (CAMPAIGN IDENTITY)
12. `s_perf_8_rejects_r12b_episode_pins_drift`
13. `s_perf_8_rejects_catalog_order_drift`
14. `s_perf_8_rejects_completion_order_merge`

Plus 4 structural defect rules + 6 additional integrity
rules. 43-test acceptance suite.

**CLI**:

```
dsfb-corpus s-perf-8-batched-k        [--json] [--out PATH]
dsfb-corpus s-perf-8-batched-k-emit   [--out-dir DIR]
```

`s-perf-8-batched-k` exits 3 on drift, 0 on admit.

## S-PERF.10 — DigestLanePlanV1 / digest-cost audit

> S-PERF.10 audits the measured digest-lane bottleneck
> and emits DigestLanePlanV1. It does not claim bandwidth
> improvement. It defines the preservation contract that
> any future digest compaction kernel rewrite must
> satisfy.

S-PERF.10 is Track B leg 3 (receipt-only). The S-PERF.8.1
hardening pass proved K batching is NOT the full-scale
lever; the S-PERF.6 receipt + S-PERF.7 source-report
parser already record, but do not codify, that the four
`tree_digest` stages collectively dominate device time.
S-PERF.10 codifies that finding as a hashable receipt AND
writes the byte-identical digest-root preservation
contract that any future digest rewrite (S-PERF.11 /
S-PERF.10b) MUST satisfy. The corpus crate stays
plan-locked host-only with zero CUDA dependency.

**Measured digest-lane share** (live R.12b source report,
RTX 4080 SUPER, CUDA 13.2; parsed from
`reports/d64_stage_timing_256x4096_K1.txt`):

| stage                              |   us  |   % of device_total |
|------------------------------------|------:|--------------------:|
| `tree_digest residual`             | 2 364 | 11.40% |
| `tree_digest sign`                 | 2 684 | 12.90% |
| `tree_digest detector (wide cells)`| 2 509 | 12.10% |
| `tree_digest consensus`            | 4 338 | 20.90% |
| **`digest_total`**                 | **11 895** | **57.30%** |

**Preservation contract** (plan-locked; folded into
`digest_compaction_contract_hash_v1`; any future digest
rewrite MUST satisfy every law verbatim):

- **digest_root_law** — Future digest compaction MUST
  preserve byte-identical digest roots; `SerialSha256` /
  `TreeSha256V1` mode identity is invariant under any
  rewrite.
- **fragment_merge_order_law** — Per-block digest
  fragments MUST be merged in canonical order;
  completion-order merging is plan-forbidden.
- **digest_mode_identity_law** — Throughput-mode
  `TreeSha256V1` and Audit-mode `SerialSha256` MUST
  produce identical digest roots when the digest mode is
  held constant.
- **casefile_chain_law** — `CaseFile` per-stage hash chain
  MUST stay byte-identical; no digest rewrite may insert,
  remove, or reorder the 12 chain links.

**What this DOES**:

- Parses the four `tree_digest` rows from
  `reports/d64_stage_timing_256x4096_K1.txt` (us +
  percent-of-device-total) and computes `digest_total_us`
  + `digest_total_pct`.
- Builds the plan-locked digest-root preservation
  contract above.
- Binds the audit + contract + upstream S-PERF.6 measured
  baseline hash + S-PERF.7 verifier hash + S-PERF.8.1
  receipt hash + R.12b full-scale episode pin (1917) into
  a single hashable `DigestLanePlanV1` envelope.

**What this DOES NOT do**:

- Does NOT change kernels.
- Does NOT claim bandwidth improvement.
- Does NOT benchmark anything.
- Does NOT run any CUDA code.
- Does NOT compact digests (that is the S-PERF.11 /
  S-PERF.10b commit).
- Does NOT mutate the S-PERF.6 / S-PERF.7 / S-PERF.8
  receipts or any prior hash anchor.
- Does NOT alter `SEED.len()`.
- Does NOT rebaseline R.12b.

**Three own-namespace hashes** (none folded upstream):

- `digest_stage_cost_audit_hash_v1` under
  `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-STAGE-COST-AUDIT:v1\0`
- `digest_compaction_contract_hash_v1` under
  `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-COMPACTION-CONTRACT:v1\0`
- `digest_lane_plan_hash_v1 = 558c1a0a…` under
  `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-LANE-PLAN:v1\0`

All deterministic across two builds; pairwise distinct;
distinct from every prior S-PERF anchor; binds the live
S-PERF.6 baseline + S-PERF.7 verifier + S-PERF.8.1
receipt.

**EIGHT plan-required load-bearing negatives**:

1. **`s_perf_10_rejects_digest_optimisation_claim_without_byte_identical_digest_roots`** (CAMPAIGN IDENTITY)
2. `s_perf_10_rejects_digest_plan_without_four_tree_digest_stage_timings`
3. `s_perf_10_rejects_digest_plan_without_total_digest_share` (band [50%, 65%])
4. `s_perf_10_rejects_digest_plan_without_s_perf_8_1_anchor`
5. `s_perf_10_rejects_digest_plan_without_s_perf_6_measured_baseline_anchor`
6. `s_perf_10_rejects_digest_plan_that_claims_bandwidth_improvement`
7. `s_perf_10_rejects_digest_plan_without_future_rewrite_contract`
8. `s_perf_10_rejects_digest_plan_with_episode_count_drift`

Plus 6 structural defect rules. 42-test acceptance suite.

**CLI**:

```
dsfb-corpus s-perf-10-digest-lane          [--json] [--out PATH]
dsfb-corpus s-perf-10-digest-lane-emit     [--out-dir DIR]
```

`s-perf-10-digest-lane` exits 3 on drift, 0 on admit.

## S-PERF.11 — measured digest-lane compaction

Plan-locked core sentence (verbatim):

> S-PERF.11 performs the first measured digest-lane
> rewrite above the S-PERF.10 preservation contract. It
> reduces digest_total_us from 11,895 to 8,556 while
> preserving byte-identical TreeSha256V1 roots and R.12b
> episode counts.

Plan-locked report wording (verbatim):

> S-PERF.11 records a measured digest-lane compaction
> improvement on RTX 4080 SUPER / CUDA 13.2. The four
> TreeSha256V1 digest roots remain byte-identical, and
> R.12b episode counts remain 13 / 89 / 1917. The
> measured digest total falls from 11,895 us to 8,556 us,
> a 1.39× digest-speedup, while measured CUDA pipeline
> bandwidth rises from 13.33 GB/s to 16.38 GB/s, a +22.9%
> improvement. This is an admissible measured
> improvement, not a memory-bandwidth saturation claim.

S-PERF.11 is the safe first strike of the saturation
campaign. The D64 tree-digest leaf launch is swapped from
`tree_digest_leaf_kernel` (one chunk per block) to
`tree_digest_leaf_kernel_v2` (32 chunks per block, one
chunk per thread within a warp). Per-chunk SHA-256 input
bytes are unchanged; per-stage `TreeSha256V1` root digests
remain byte-identical; S-PERF.10's
`same_mode_digest_root_law` is satisfied by construction
(pinned by the `s_perf_11_pre_rewrite_root_capture`
CUDA-gated acceptance test).

**Honest framing.** The current measured CUDA pipeline
reaches **16.38 GB/s post-rewrite**, **~2.28% of the RTX
4080 SUPER 716 GB/s memory-bandwidth anchor**. This is NOT
the saturation target. It is a safe first strike. The
architectural conversion toward 700 GB/s begins at
S-PERF.12 (CompactDensorDigestV1 throughput mode) and
continues through S-PERF.16 (saturation microkernel +
roofline receipt).

Measured pre/post on canonical 256×4096 K=1 D64 (RTX 4080
SUPER, CUDA 13.2):

| stage                              |   pre |  post |  delta |
|------------------------------------|------:|------:|-------:|
| `tree_digest residual`             |  2364 |  1685 |   -679 |
| `tree_digest sign`                 |  2684 |  1929 |   -755 |
| `tree_digest detector (wide cells)`|  2509 |  2052 |   -457 |
| `tree_digest consensus`            |  4338 |  2890 |  -1448 |
| **digest_total_us**                |**11895**|**8556**|**-3339** |

```
digest_speedup_x        : 1.39x  (pre / post)
pre_bandwidth_gbps      : 13.33 GB/s   (186 bp /  1.86% of peak)
post_bandwidth_gbps     : 16.38 GB/s   (228 bp /  2.28% of peak)
bandwidth_delta         : +3.05 GB/s   (+2288 bp relative to pre)
saturation_admitted     : false  (gate = 8000 bp; observed = 228 bp)
R.12b episodes          : 13 / 89 / 1917  (BYTE-STABLE)
```

Three plan-locked own-namespace hashes (none folded
upstream):

- `s_perf_11_digest_compaction_measurement_hash_v1` under
  `DSFB-GPU-ATLAS:S-PERF-11-DIGEST-COMPACTION-MEASUREMENT:v1\0`.
  Pins the pre/post `tree_digest` stage timings + totals +
  speedup + source paths + folded kernel-rewrite metadata.
- `s_perf_11_digest_root_equivalence_hash_v1` under
  `DSFB-GPU-ATLAS:S-PERF-11-DIGEST-ROOT-EQUIVALENCE:v1\0`.
  Pins the four pre-rewrite + four post-rewrite
  `TreeSha256V1` root digests + per-stage equivalence
  flags. Receipt-level mirror of the kernel-side
  `s_perf_11_pre_rewrite_root_capture` safety harness.
- `s_perf_11_bandwidth_delta_report_hash_v1` under
  `DSFB-GPU-ATLAS:S-PERF-11-BANDWIDTH-DELTA-REPORT:v1\0`
  = `1a27154e335c27df6db939d4c8ff0f36f8baf75871be06c750f0853f2268adc8`.
  Top-level META-hash binding measurement +
  root-equivalence + bandwidth pre/post + delta +
  saturation flag + four upstream anchors + three R.12b
  episode pins.

EIGHT plan-required campaign-identity negatives (verbatim):
`s_perf_11_rejects_speedup_without_digest_root_equivalence`,
`s_perf_11_rejects_speedup_without_r12b_episode_stability`,
`s_perf_11_rejects_digest_total_not_reduced`,
`s_perf_11_rejects_bandwidth_not_improved`,
`s_perf_11_rejects_missing_s_perf_10_digest_lane_plan_hash`,
`s_perf_11_rejects_tree_sha256v1_root_drift`,
`s_perf_11_rejects_saturation_claim_below_8000_bp`,
`s_perf_11_rejects_claim_that_16_38_gbps_is_memory_saturation`.

TWELVE plan-required positive tests (verbatim):
`admits_measured_digest_compaction_result`,
`computes_digest_speedup_1_39x`,
`computes_bandwidth_delta_3_05_gbps`,
`computes_bandwidth_gain_22_9_percent`,
`preserves_four_tree_sha256v1_roots`,
`preserves_r12b_episode_counts_13_89_1917`,
`binds_s_perf_10_digest_lane_plan_hash`,
`renderers_are_byte_stable`,
`hashes_are_deterministic`,
`changing_post_digest_total_changes_hash`,
`changing_post_bandwidth_changes_hash`,
`changing_any_digest_root_changes_equivalence_hash`.

Plus 5 structural defect rules + 4 plan-acknowledged
defense-in-depth structural extras (completion-order
fragment merge / casefile chain drift / missing pre-post
digest table / K-amortisation overclaim). 61-test
acceptance suite inside the corpus crate + a separate
`s_perf_11_pre_rewrite_root_capture` CUDA-gated test that
pins the four pre-rewrite `TreeSha256V1` root digests as
`[u8; 32]` constants and asserts byte-equality after the
kernel swap.

This S-PERF.11 atomic commit also bundles three pre-rewrite
hygiene items: (a) stale victory-lap wording in README +
paper is replaced with "deferred to a later post-S-PERF /
S-MG victory-lap campaign" + a new public-language
scanner negative, (b) the S-PERF.10
preservation-contract laws are renamed
(`same_mode_digest_root_law`,
`canonical_fragment_merge_order_law`,
`digest_mode_non_aliasing_law`,
`casefile_chain_preservation_law`) and 2 ambiguous
cross-mode wordings are rewritten, rebaselining the
S-PERF.10 `digest_compaction_contract_hash_v1` +
`digest_lane_plan_hash_v1` (= `e9cf5c34…`), and (c) the
pre-rewrite root capture safety harness pins
byte-identical roots across the kernel swap.

Plan-locked non-claims: S-PERF.11 does NOT claim
memory-bandwidth saturation (228 bp << 8000 bp threshold);
does NOT change digest mode (TreeSha256V1 preserved); does
NOT mutate S-PERF.6 / S-PERF.7 / S-PERF.8 / S-PERF.10's
existing receipts (the rebaselined S-PERF.10 contract and
plan hashes are a plan-acknowledged wording-clarification
schema upgrade, not an audit-content change); does NOT
rebaseline R.12b episodes (13 / 89 / 1917 byte-stable);
does NOT alter `SEED.len()` (stays 54); does NOT run CUDA
from inside the corpus crate (the measurement is captured
by `dsfb-gpu-debug-cuda`'s bench harness and pinned to
`reports/d64_stage_timing_256x4096_K1_post_s_perf_11.txt`).

CLI:

```
dsfb-corpus s-perf-11-digest-compaction       [--json] [--out PATH]
dsfb-corpus s-perf-11-digest-compaction-emit  [--out-dir DIR]
```

`s-perf-11-digest-compaction` exits 3 on drift, 0 on admit.

Plan-locked one-line verdict (verbatim):

> S-PERF.10 locked the digest preservation law;
> S-PERF.11 performs the first measured digest-lane
> rewrite and moves the scoreboard from 13.33 to 16.38
> GB/s without digest-root or episode-count drift.

**Next move after S-PERF.11**: bottleneck triage. Before
S-PERF.12 commits, re-run
`r9_c_d64_stage_profile_256x4096_k1` and inspect whether
the four `tree_digest` stages are still dominant
(justifying **S-PERF.12 — CompactDensorDigestV1 throughput
mode**, gate 1→2, target ~50 GB/s) or whether the
bottleneck moved to host `compute_features` / consensus /
candidate-collapse (which would re-rank the saturation-
campaign ladder). Cross-mode digest rewrites
(CompactDensorDigestV1 etc.) live under their own
digest-mode identifier per S-PERF.10's
`digest_mode_non_aliasing_law` (each declared mode owns
its own root-byte projection).

## S-PERF.11.1 — post-S-PERF.11 bottleneck triage

Plan-locked thesis (verbatim): *"S-PERF.11.1 re-profiles the
device wall on the committed post-S-PERF.11 kernel and records
the dominant stage classification + chosen `next_strike`
recommendation under one own-namespace hash. It does not
change kernels, does not claim bandwidth improvement, and does
not execute the next strike."*

Plan-locked one-line verdict (verbatim): *"S-PERF.11 proves
the saturation campaign can move the scoreboard while
preserving deterministic evidence roots; S-PERF.11.1 re-
profiles the device wall and records the plan-locked next
strike under one hashable triage receipt."*

S-PERF.11.1 re-runs `r9_c_d64_stage_profile_256x4096_k1` on
the committed post-S-PERF.11 kernel, parses the per-stage
timings, classifies the dominant device-side bucket against
the seven plan-locked categories (digest aggregate /
detector_motif / host compute_features / consensus / candidate
collapse / host bank admit / other), and emits the
plan-locked decision tree's `next_strike` recommendation
under one own-namespace hash
`s_perf_11_1_bottleneck_triage_hash_v1 = 70dd967b…` under
`DSFB-GPU-ATLAS:S-PERF-11-1-BOTTLENECK-TRIAGE:v1\0`. It binds
the rebaselined S-PERF.11
`s_perf_11_bandwidth_delta_report_hash_v1 = 1a27154e…` so the
triage receipt cannot be reused against a stale S-PERF.11.

**Triage result (live, RTX 4080 SUPER / CUDA 13.2, 2026-05-18)**:

- `dominant_stage_label`              = `tree_digest (4-stage aggregate)`
- `dominant_stage_us`                 = 15334
- `dominant_stage_pct_basis_points`   = 5063 (50.63% of `device_total_us`)
- `bottleneck_category`               = `DigestStillDominant`
- `next_strike_recommendation`        = `SPerf12CompactDensorDigestV1`

Six plan-required campaign-identity negatives + 4 structural
defect rules. 24-test acceptance suite. CLI: `dsfb-corpus
s-perf-11-1-bottleneck-triage [--json] [--out PATH]` (exits 3
on drift, 0 on admit); `dsfb-corpus
s-perf-11-1-bottleneck-triage-emit [--out-dir DIR]` (writes 2
byte-stable artifacts).

**Plan-locked non-claims**: S-PERF.11.1 does NOT change
kernels; does NOT claim bandwidth improvement; does NOT mutate
the pinned post-S-PERF.11 source-report file; does NOT mutate
any prior S-PERF / T.11 / T.12.x / FF.x / S1.3.x / T.12.PROV
hash anchor; does NOT alter `SEED.len()`; does NOT rebaseline
R.12b; does NOT execute the next strike — that is the next
commit (S-PERF.12 per the receipt's `next_strike_recommendation`).

## S-REAL — Real Dataset Audit Chain (four-layer foundation)

DSFB-GPU is not only a deterministic CUDA performance experiment.
S-REAL proves it is a **replayable data-science evidence engine**
on real public datasets, with operator-legible reports and honest
runtime-dependent timing measurements — all without perturbing the
inference byte-chain.

The chain sealed in four atomic commits across 2026-05-19:

| commit  | section       | layer            | claim |
|---|---|---|---|
| `0ce7ee7` | S-REAL.1      | **usable**         | real public datasets run end-to-end with byte-identical replay |
| `6969913` | S-REAL.1.1    | **understandable** | operator-legible aggregations (motif histogram / entity summary / episode timeline / top spans / motif glossary) |
| `099ee35` | S-REAL.1.1.1  | **understandable** | 30-second summary card + Replay Proof card at top of report |
| `42ef68e` | S-REAL.PERF   | **measured**       | per-stage timing + multi-run variance + sequential-K amortization, runtime-dependent |

**Three sealed datasets, all SHA-256-byte-pinned and license-
cleared**:

| dataset | upstream | license | source class | shape | events | episodes | replay |
|---|---|---|---|---|---:|---:|---|
| TADBench TrainTicket F11 | DOI 10.5281/zenodo.6979726 | Apache-2.0 | DebuggingSoftwareTelemetry | 16×431, 6240 NaN skipped | 656 | **90** | byte-identical YES |
| Illinois SocialNet (DeathStarBench) | DOI 10.13012/B2IDB-6738796_V1 | CC0-1.0 | ObservabilityTraces | 6×32 | 192 | 3 | byte-identical YES |
| AIOps Challenge 2018 KPI | Su et al., IPCCC 2018 (Bagel) | Apache-2.0 | TimeSeriesAnomaly | 4×32 | 128 | 3 | byte-identical YES |

**Per-dataset artifact shape (9 receipts, atomic emit)** in
`reports/s_real_1/<dataset_id>/`:

1. `dataset_manifest.toml` — upstream identity + SHA-256 byte-pin + license + source class.
2. `schema_map.toml` — event-lowering rule + observed shape.
3. `run_receipt.txt` — dispatcher receipt + hash chain.
4. `casefile.json` — canonical `CaseFile` bytes via `emit(&case)`.
5. `episodes.jsonl` — admitted episodes sorted by `(entity_id, start_window, end_window, reason as u8)`.
6. `audit_report.html` — deterministic no-JavaScript HTML report (summary card + Replay Proof card + 7 base sections + 6 aggregation sub-sections in section 4 + motif glossary).
7. `replay_verification.txt` — per-artifact run-1 vs run-2 SHA-256s + admission line.
8. `limitations.md` — plan-locked non-claims verbatim.
9. `perf_profile.txt` — runtime-dependent timing block (S-REAL.PERF; explicitly outside the byte-identical-replay envelope).

**Replay law**: two consecutive dispatches on the same SHA-
pinned fixture bytes produce byte-identical
`casefile.json` + `episodes.jsonl` + `audit_report.html`. Every
dataset's `replay_verification.txt` records `byte-identical
replay: YES` plus the matching SHA-256 from both runs.
`perf_profile.txt` is explicitly excluded — timing values are
runtime-dependent and will differ across re-invocations, but
the inference chain (casefile + episodes) remains byte-identical.

**Lowering rule** (recorded in every audit's `schema_map.toml`
+ `audit_report.html` section 2): each finite `# residual-
projection v2` cell becomes one synthetic `TraceEvent` via
`ts_ns = window_idx * 1_000_000_000`, `entity_id = signal_idx`,
`span_id = window_idx * 65536 + signal_idx`,
`latency_us = clamp(value * 1000, 0, 32_767_000)`. NaN cells
produce no event. The rule is deterministic, fully documented,
and reversibly inspectable from `episodes.jsonl`.

**Measured S-REAL.PERF profile** (TADBench, RTX 4080 SUPER /
CUDA 13.2 / S-PERF.16.a A6.1 build, `--iters 5`):

```
cold-start CUDA context  200,588 µs   (iter 1)
steady-state dispatch      2,950 – 3,193 µs   (iters 2-5)
dispatch_median_us         3,005 µs
casefile_emit_us              28 µs
episodes_jsonl_emit_us        45 µs
audit_report_emit_us         142 µs
```

Cold-start vs steady-state spread is honestly reported. Real-data
throughput at small fixture sizes is overhead-dominated; the
honest framing is preserved in every `perf_profile.txt`.

**CLI surface**:

The canonical subcommand is `s-real-audit`; the historical
alias `s-real-1-audit` (from the original 3-dataset S-REAL.1
seal) remains accepted verbatim so every committed bundle
artifact's "Re-invoking s-real-1-audit ..." prose continues
to invoke a real handler without artifact-byte regeneration.

```fish
./target/release/dsfb-gpu-debug s-real-audit --dataset aiops_kpi
./target/release/dsfb-gpu-debug s-real-audit --dataset illinois_socialnet
./target/release/dsfb-gpu-debug s-real-audit --dataset tadbench_f11
./target/release/dsfb-gpu-debug s-real-audit --dataset all
./target/release/dsfb-gpu-debug s-real-audit --dataset all --iters 5
./target/release/dsfb-gpu-debug s-real-audit --dataset all --iters 5 --catalogs 16
```

Default `--out-dir reports/s_real_1`, `--iters 2`, `--catalogs 1`.
Exit code 7: replay-verification failure (run 1 != run 2). Exit
code 6: SHA-256 pin divergence. Exit code 2: CUDA-unavailable.

**Tier ladder (plan-locked, refreshed 2026-05-19 post-chain-seal)**:

| tier         | status       | scope |
|---|---|---|
| **S-REAL.1**     | **SEALED `0ce7ee7`** | 3 datasets, 24 artifacts, byte-identical replay |
| **S-REAL.1.1**   | **SEALED `6969913`** | operator richness (aggregations + glossary + scanner) |
| **S-REAL.1.1.1** | **SEALED `099ee35`** | summary card + Replay Proof card polish |
| **S-REAL.PERF**  | **SEALED `42ef68e`** | per-dataset timing + multi-run variance + sequential-K amortization |
| **S-REAL.2**     | **SEALED `15f5af0`** | 10 datasets admitted across 5 source-class families (F1–F5); F4 anchored via NASA C-MAPSS PHM08 (FD001 unit 1 → 26 episodes) |
| **S-REAL.3**     | **SEALED `a8aaa04`** | 20 datasets, 316 admitted episodes, Zenodo-publishable bundle (`bundle_manifest.toml` + `bundle_hash_chain.txt` + `zenodo_metadata.json`) + executive `reports/INDEX.md` |
| **S-REAL.3.1**   | **SEALED `fde8a99`** | hardening + 10 saturation-class 1 M-cell fixtures (RF I/Q / mmWave / database-derived; 16.16 .. 26.66 GB/s wide arena, up to 117 % of S-PERF.16.a synthetic anchor); bundle integrity test (60-row chain) + tier-scope clarification + `s-real-audit` canonical alias |
| **S-REAL.3.1.1** | **active**           | hygiene close-out: stale-language cleanup (`21→30 fixtures`, observability→database-derived data-class wording) + full code commentary on bundle integrity + saturation bench + sweep script; untracked session-context archive; per-section routine reaffirmed |
| S-REAL.4         | deferred             | licensing / commercial-evaluation package (deferred per user directive; commercial-clean subset = 13 datasets carrying CC-BY-4.0 / CC0-1.0 / MIT / Public-Domain) |

**Plan-locked non-claims** (preserved verbatim into every
dataset's `limitations.md` + every `audit_report.html` section 7):

- Does NOT claim DSFB has identified the "real" anomaly in
  the dataset.
- Does NOT claim DSFB outperforms any other anomaly detector.
- Does NOT claim DSFB has discovered causality.
- Does NOT claim fitness-for-purpose on regulated or safety-
  critical use.
- Does NOT claim the dataset is "correctly labeled" or
  "ground truth".
- Does NOT claim the corpus or registry is exhaustive.
- Does NOT claim replay determinism across different driver
  / CUDA / hardware versions; the replay receipt records the
  toolchain explicitly.
- Does NOT claim saturation, production throughput, or
  detector superiority; S-REAL.PERF timing values are honest
  runtime measurements at small fixture sizes, NOT benchmark
  superiority claims.

**Saturation snapshot (S-REAL.3.1, sealed
`reports/s_real_saturation_sweep.txt`)**: the same byte-pinned
audit chain re-run through the synthetic S-PERF.16.a saturation
bench harness on 10 large 1 M-cell fixtures (real public RF I/Q,
mmWave power, and database-derived residual surfaces — JOB/IMDB
byte-frequency + cast-info projection, Snowset query-event CSV,
SQLShare seaflow CSV) reaches **16.16 .. 26.66 GB/s** logical
throughput on the 264-byte `DetectorCellWide` arena — up to
**117 %** of the sealed synthetic S-PERF.16.a median (22.74
GB/s on RTX 4080 SUPER / CUDA 13.2). The 20 small audit fixtures remain
launch-bound (0.05 .. 0.82 GB/s) and are reported honestly;
sharp bimodal — 10 saturation-class, 20 launch-bound, 0
transition. Numbers are LOGICAL throughput on the wide-cell
arena, NOT physical DRAM bandwidth; saturation classification
is a property of cell-count and dispatcher-shape, NOT a
detector-quality or domain-truth claim. Bundle integrity is
gated by `tests/s_real_3_bundle_integrity.rs` (5 acceptance
tests over the 20-dataset × 9-artifact = 60-row hash chain).

**Causal-diagnosis-language regression scanner** (S-REAL.1.1):
every motif and reason-code prose line ends with the
standardised tail *"DSFB interprets this structurally, not as
a ground-truth causal diagnosis."* A regression test scans
the new operator-facing sub-sections of `audit_report.html`
for forbidden phrases and rejects any drift toward
causal-correctness claims.

## What this is

- A small Rust workspace (six crates) + CUDA kernels + Colab notebook
  + Atlas jurisprudence court.
- **GPU layer**: deterministic, replayable, byte-exact inference
  from input trace catalog to final verdict case file. Fixed-point
  Q16.16 arithmetic on both CPU and GPU so CPU↔GPU byte-equality is
  achievable across stages.
- **Atlas layer**: 54 deduplicated literature detector primitives
  across 22 source classes (SPC, change detection, drift, robust
  stats, distribution distance, info theory, signal/spectral,
  time-series, FDD, graph anomaly, debug/observability, ...) with
  full provenance, hashed under domain
  `DSFB-GPU-ATLAS:LITERATURE-CORPUS:v1\0`; T.11a–T.11h court
  surfaces stacked on top under their own domain-separated hash
  namespaces; S1.3a reason-coded activation plan tying them all
  together.
- A self-contained mirror of the relevant subset of `dsfb-debug`.
  It does not depend on `dsfb-debug` as a crate.

## What this is **not**

- Not a machine-learning system. No neural networks. No learned
  weights. No embeddings. No calibration. No probability anywhere.
- Not Bayesian inference, MCMC, stochastic sampling, logits, or
  confidence scores. The Atlas court is **categorical reason codes
  only**.
- Not an LLM and not a benchmark of one. The LLM (when used)
  consumes admitted case-file evidence after the deterministic
  court has spoken; it never generates the court.
- Not a performance benchmark first. The first success criterion
  is byte-exact replay, not speed (the R.13 ~55× full-pipeline
  campaign reduction is sealed but secondary to the doctrine).
- Not a replacement for APM / ELK / OpenTelemetry. The verdict is
  a replayable court record, not an alerting dashboard.
- Not SLSA / in-toto / W3C PROV / OpenLineage / NIST AI RMF /
  RO-Crate / SPDX / CycloneDX compatible by claim. Every DSFB
  hash is domain-separated and DSFB-native; interoperability
  mappings are separate future commitments.

## Layout

```
crates/
  dsfb-gpu-debug-core/   no_std, zero deps, semantic authority (CPU bank)
  dsfb-gpu-debug-cuda/   FFI + CUDA kernels behind `cuda` feature
  dsfb-gpu-debug-demo/   CLI binary for the GPU acceleration path
  dsfb-gpu-atlas-corpus/ host-only literature corpus + T.11 jurisprudence
                         court + S1.3a activation planner + S1.3b activation
                         explanation/diff court + S1.3c task/dataset/context
                         manifests + T.12.0 amendment-proposal intake +
                         T.12.a first real expansion proposal (SPC) +
                         T.12.b cross-class dedup authority (SCD) +
                         T.12.c drift / distribution-distance authority +
                         first ParameterizationOf category + T.12.d
                         robust statistics (first proposal exercising
                         all five court-delta categories) + T.12.e
                         signal / spectral / wavelet (transform-law
                         discipline) + T.12.f time-series structure /
                         control residuals (residual-and-decision-law
                         discipline) + T.12.g graph / topology anomaly
                         (first proposal with two
                         RejectedNotDeterministic records) +
                         T.12.h data quality / tabular / database
                         integrity (validation-rule discipline; second
                         proposal with two RejectedNotDeterministic
                         records; target-leakage admitted as
                         "candidate, not proof") + T.12.i
                         observability / debugging (third proposal
                         with two RejectedNotDeterministic records;
                         protects the dsfb-gpu-debug L6 bank surface
                         from re-canonicalisation; vendor APM scores
                         and learned incident classifiers rejected)
                         + T.12.j medical / biosignal (signal-
                         witness-not-diagnosis discipline; fourth
                         proposal with two RejectedNotDeterministic
                         records; learned arrhythmia classifier and
                         clinician-label-only diagnostic rule
                         rejected; parametric
                         diagnostic-claim-language scanner enforces
                         the "signal witness, not a medical
                         diagnosis" non-claim across every canonical)
                         + T.12.k industrial / FDD / condition
                         monitoring (8 existing-canonical authority
                         resolutions — the LARGEST SEED-collision
                         ratification of any T.12.x; only 6 new
                         canonicals via plan "success shape" — lean
                         on cross-class dedup discipline rather than
                         detector count; fifth proposal with two
                         RejectedNotDeterministic records; proprietary
                         PdM black-box scores and learned fault
                         classifiers rejected; root-cause-claim-
                         language scanner enforces "condition-
                         monitoring witness, not a maintenance
                         recommendation" non-claim; plant-or-
                         residual contract scanner enforces math-
                         structure + decision-functional declaration)
                         + T.12.l chemometrics (4 existing-canonical
                         authority resolutions for the latent-space +
                         envelope SEED family; only 5 new canonicals
                         via plan "success shape"; sixth proposal
                         with two RejectedNotDeterministic records;
                         black-box spectroscopy classifiers and
                         adaptive-AutoML chemometric pipelines
                         rejected; material-identification-claim-
                         language and regulatory-compliance-claim-
                         language scanners enforce "chemometric
                         signal witness, not a material identification
                         or a regulatory compliance verdict" non-
                         claim; preprocessing-or-latent-model
                         contract scanner enforces math-structure +
                         decision-functional declaration)
                         + T.12.m RF / communications (6 existing-
                         canonical authority resolutions for the
                         spectral + envelope + entropy + correlation
                         + carrier-offset + modulation-quality SEED
                         family RF heavily reuses including SEED 53
                         Carrier-frequency-offset residual and SEED 54
                         EVM anomaly; only 6 new canonicals at
                         6303..=6308 via plan "success shape" —
                         constellation spread / CIR drift / IQ
                         imbalance / phase-noise / symbol-timing
                         offset / cyclostationary feature shift;
                         reserved ids 6301 and 6302 deliberately
                         unused after SEED-walk-first caught SEED 53 /
                         54 collisions; seventh proposal with two
                         RejectedNotDeterministic records; learned RF
                         fingerprinting classifiers and black-box
                         modulation classifiers / proprietary spectrum-
                         anomaly scores rejected; three parametric
                         scanners enforce "RF signal witness, not
                         emitter attribution / geolocation / spectrum-
                         enforcement" non-claim; signal-or-sampling
                         contract scanner enforces signal representation
                         + sampling + carrier-or-channel + window/
                         transform + decision-functional declaration)
                         + T.12.n econometrics + reliability /
                         survival (combined campaign because the two
                         domains share structural-break / CUSUM /
                         envelope-residual ancestry; 4 existing-
                         canonical authority resolutions for the
                         structural-change + envelope SEED family —
                         CUSUM 3 / Page-Hinkley 4 / Mann-Kendall 11
                         / Residual envelope exit 22; 8 new
                         canonicals at 6401..=6408 (4 econometric:
                         GARCH residual per Bollerslev 1986 /
                         cointegration-break per Hansen 1992 /
                         Hausman per Hausman 1978 / Bai-Perron per
                         Bai-Perron 1998-2003; 4 reliability /
                         survival: Kaplan-Meier survival residual
                         per Kaplan-Meier 1958 / Cox-Schoenfeld per
                         Cox 1972 / Schoenfeld 1982 / Weibull
                         failure-rate per Weibull 1951 / Paris-
                         Erdogan crack-growth per Paris-Erdogan
                         1963); 4 ParameterizationOf collapsing
                         CUSUM-of-recursive-residuals / Quandt-
                         Andrews-Chow F-test / hazard-rate-change /
                         cumulative-damage residual into existing
                         SEED canonicals; eighth proposal with two
                         RejectedNotDeterministic records; learned
                         market predictors / black-box financial
                         forecasters AND learned RUL classifiers /
                         black-box predictive-maintenance scores
                         rejected; three parametric claim-language
                         scanners enforce "not market prediction /
                         not investment-credit decision / not RUL-
                         or-failure-time certainty" non-claim; two
                         contract scanners enforce stationarity-and-
                         window declaration for econometric records
                         and censoring-and-time-origin declaration
                         for survival records)
                         + T.12.o streaming sketches (4 existing-
                         canonical authority resolutions for the KS
                         + missingness + error-burst + cardinality
                         SEED family streaming-sketch summaries
                         heavily reuse — KS 8 / Missingness spike
                         13 / Error burst 41 / Cardinality drift 46;
                         8 new canonicals at 6501..=6508 (CMS
                         residual per Cormode-Muthukrishnan 2005 /
                         HLL cardinality shift per Flajolet-Fusy-
                         Gandouet-Meunier 2007 / Bloom membership
                         per Bloom 1970 / Misra-Gries heavy-hitter
                         per Misra-Gries 1982 / Space-Saving heavy-
                         hitter per Metwally-Agrawal-El Abbadi 2005
                         distinct from Misra-Gries via replace-
                         smallest-on-miss / Greenwald-Khanna quantile
                         per Greenwald-Khanna 2001 / t-digest per
                         Dunning 2019 with DETERMINISTIC centroid-
                         merge law / AMS moment sketch per Alon-
                         Matias-Szegedy 1999); 4 ParameterizationOf
                         collapsing Flajolet-Martin pre-HLL /
                         streaming-approximate KS / sliding-window
                         error-burst sketch / sketch-approximate
                         missingness via Bloom inversion into
                         existing SEED canonicals; ninth proposal
                         with two RejectedNotDeterministic records;
                         learned streaming-anomaly scores (Datadog
                         Watchdog AI / DataRobot Streaming AutoML /
                         Splunk Stream ML / AWS Lookout for Metrics
                         / Azure Anomaly Detector) AND black-box
                         vendor approximate-streaming aggregators
                         (Snowflake APPROX_* / BigQuery APPROX_* /
                         Druid / ClickHouse uniqHLL12 / topK / AWS
                         Athena APPROX_*) rejected; SIX scanners
                         enforce contract discipline — hash family /
                         width / depth / seed required for hash-
                         based sketches AND update rule / merge law
                         required for deterministic sketches;
                         probabilistic-bound-as-deterministic-
                         certainty / approximate-query-truth /
                         privacy-or-anonymization / mergeable-
                         without-merge-law claim-language scanners)
                         + T.12.p information theory catch-up (3
                         existing-canonical authority resolutions
                         for the KL + JS + Spectral-entropy SEED
                         family information-theoretic witnesses
                         heavily reuse — KL 9 / JS 32 / Spectral
                         entropy 38; 5 new canonicals at
                         6601..=6605 (Shannon entropy per Shannon
                         1948 / Conditional entropy per Cover-
                         Thomas 2006 / Mutual information per
                         Cover-Thomas 2006 structurally distinct
                         from SEED 9 KL via JOINT-vs-PRODUCT-OF-
                         MARGINALS contract / Cross-entropy per
                         Shannon 1948 with FIXED MODEL distribution
                         parameter-pinned / MDL per Rissanen 1978
                         / 1986 with two-part code); 4
                         ParameterizationOf collapsing Normalized
                         MI / Transfer entropy proxy per Schreiber
                         2000 admitted ONLY AS A DETERMINISTIC
                         NON-CAUSAL WITNESS / Rényi-Tsallis
                         entropy per Rényi 1961 / Tsallis 1988 /
                         Compression-ratio anomaly per Ziv-Lempel
                         1977 / 1978 / Welch 1984 LZW into 6601
                         / 6603 / 6605; tenth proposal with two
                         RejectedNotDeterministic records;
                         learned mutual-information estimator
                         (MINE Belghazi et al. 2018 / InfoMax /
                         variational MI bounds / neural KL
                         estimators / InfoVAE / CPC) AND black-
                         box vendor IT score (AWS Macie / IBM
                         Guardium / Microsoft Purview / Symantec
                         / Broadcom DLP / Cisco Talos) rejected;
                         SIX scanners enforce contract discipline
                         — estimator / binning OR partition law
                         required + log base / smoothing / empty-
                         bin law required for entropy-style
                         canonicals + joint-distribution contract
                         over (X, Y) required for MI / conditional
                         entropy + causal-information-flow /
                         privacy-or-security / learned-embedding
                         claim-language scanners)
                         + T.12.consolidate META-hash freeze
                         layer (ratification not expansion;
                         loads every T.12.0..T.12.p proposal +
                         verifies every hash by recomputation +
                         enforces 10 plan-required negatives +
                         builds sorted T.12 expansion index of
                         98 entries spanning 5001..=6699 +
                         emits 3 new own-namespace hashes
                         consolidation_report_hash_v1 /
                         t12_expansion_index_hash_v1 /
                         corpus_hash_v2 as the ratified-corpus
                         AUTHORITY anchor; aggregate court
                         delta 98 CanonicalAddition + 76
                         ExistingCanonicalAuthorityResolution
                         + 23 DomainTransferOf + 49
                         ParameterizationOf + 24
                         RejectedNotDeterministic + 1 T.12.a-
                         era AliasOf + 2 T.12.a-era
                         CompositionOf = 273 total dedup-court
                         records; does NOT mutate SEED, does
                         NOT mutate corpus_hash_v1, does NOT
                         promote individual proposals to
                         Accepted)
                         + FF.1 passport materialisation (98
                         T12RatifiedPassport records, one per
                         ratified CanonicalAddition; per-
                         passport passport_hash_v1 under
                         DSFB-GPU-ATLAS:FF1-T12-RATIFIED-
                         PASSPORT:v1\0; aggregate
                         ff1_passport_index_hash_v1 +
                         ff1_materialisation_report_hash_v1;
                         plan-locked SourceClass->GpuFamilyKernel
                         mapping; activation-applicability tags
                         per source class; contraindication +
                         challenge stubs reserved for later;
                         TEN scanners enforce non-claim
                         discipline; does NOT activate any
                         detector; does NOT decide
                         contraindications or challenges; does
                         NOT generate CUDA kernels; does NOT
                         mutate any upstream hash)
                         + FF.2 activation ratification gate
                         (DisabledReason::DisabledUnratifiedProposal
                         enum variant + 4-bucket classifier
                         (SeedHistorical / T12RatifiedAndPassported
                         / MissingPassport / UnratifiedProposal);
                         ff2_activation_ratification_gate_hash_v1
                         under
                         DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1\0
                         and gate-summary hash under
                         DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE-SUMMARY:v1\0;
                         SIX plan-required negatives forbid the
                         silent DisabledByWeakLBand fallback;
                         54 SEED + 98 ratified = 152 decisions in
                         the default production gate; does NOT
                         add detectors; does NOT mutate any
                         upstream hash; does NOT change S1.3a
                         SEED activation decisions)
                         + FF.3 registry generation gate
                         (7-bucket Ff3RegistryGenerationEligibility
                         classifier (Eligible /
                         RejectedUnratifiedProposal /
                         RejectedMissingFf1Passport /
                         RejectedCorpusHashV2Mismatch /
                         RejectedPassportIndexHashMismatch /
                         RejectedAdHocRecord /
                         RejectedUnknownSourceAuthority);
                         ff3_registry_generation_gate_hash_v1
                         under
                         DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1\0
                         and gate-summary hash under
                         DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE-SUMMARY:v1\0;
                         EIGHT plan-required negatives gate
                         every registry-generation source claim;
                         152 eligible + 0 rejected in the default
                         production gate; does NOT add detectors;
                         does NOT itself emit DetectorSpec
                         records; does NOT modify
                         dsfb-gpu-atlas-registry's existing 162-
                         spec registry_hash_v2; does NOT mutate
                         any upstream hash; does NOT change S1.3a
                         SEED activation decisions or FF.2
                         ratification decisions)
                         + FF.4 README authority-boundary policy
                         (communication-hygiene seal pinning a
                         canonical 19-line authority-boundary
                         block + 6 required substrings + 7
                         forbidden substrings; live README sweep
                         test verifies the on-disk README against
                         the policy on every build;
                         ff4_readme_authority_boundary_policy_hash_v1
                         under
                         DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0;
                         SEVEN plan-required negatives forbid
                         stale "future ratification" prose and
                         require corpus_hash_v1 / corpus_hash_v2 /
                         FF.1 / FF.2 / FF.3 anchor phrasings;
                         does NOT add detectors; does NOT mutate
                         any upstream hash; does NOT change court
                         state; the seal changes README text only)
                         + FF.5 ProposalSchemaUpgradePolicy
                         (forward-looking governance policy +
                         empty migration table + receipt type +
                         verifier defining how proposal schema
                         upgrades may re-render historical
                         artifacts without erasing the old hashes;
                         3 new own-namespace hashes
                         (proposal_schema_upgrade_policy_hash_v1
                         under
                         DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1\0,
                         proposal_schema_migration_table_hash_v1
                         under
                         DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1\0,
                         schema_upgrade_receipt_hash_v1 per-receipt
                         under
                         DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1\0);
                         NINE plan-required negatives gate every
                         future schema upgrade; core rule "schema
                         upgrade != silent artifact rewrite"; does
                         NOT add detectors; does NOT mutate any
                         upstream hash; does NOT itself perform any
                         schema upgrade — forward-looking governance
                         only)
                         + S1.3d budget pruning + redundancy
                         suppression (deterministic budget-aware
                         deployment court above FF.2 + FF.3 with
                         eight reason-coded disable variants
                         (DisabledByBudget / DisabledByRedundancy /
                         DisabledByGpuFamilyQuota /
                         DisabledByTaskBudget /
                         DisabledByRuntimeBudget /
                         DisabledByMemoryBudget /
                         DisabledByContraindicationBudget /
                         DisabledByCoverageHoleBudget) plus a
                         deterministic tie-break transcript; 3
                         new own-namespace hashes
                         (budget_pruning_plan_hash_v1 under
                         DSFB-GPU-ATLAS:S13D-BUDGET-PRUNING-PLAN:v1\0,
                         redundancy_suppression_hash_v1 under
                         DSFB-GPU-ATLAS:S13D-REDUNDANCY-SUPPRESSION:v1\0,
                         budgeted_activation_summary_hash_v1 under
                         DSFB-GPU-ATLAS:S13D-BUDGETED-ACTIVATION-SUMMARY:v1\0);
                         EIGHT plan-required negatives; under the
                         plan-permissive default budget every
                         FF.3-eligible candidate flows through to
                         Active as RetainedAsBudgetSurvivor
                         (152/152 active, 0 disabled); does NOT add
                         detectors; does NOT mutate any upstream
                         hash; does NOT generate CUDA kernels; does
                         NOT itself emit KernelPlan records (S1.3e
                         next))
                         + S1.3e KernelPlanV1 (deterministic
                         GPU-family execution-plan layer above
                         S1.3d emitting per-family lanes,
                         parameter-table ranges, and
                         execution-plan receipts; 14 lanes /
                         152 active detectors at baseline;
                         3 new own-namespace hashes
                         (kernel_plan_hash_v1 under
                         DSFB-GPU-ATLAS:S13E-KERNEL-PLAN:v1\0,
                         kernel_family_schedule_hash_v1 under
                         DSFB-GPU-ATLAS:S13E-KERNEL-FAMILY-SCHEDULE:v1\0,
                         kernel_parameter_table_hash_v1 under
                         DSFB-GPU-ATLAS:S13E-KERNEL-PARAMETER-TABLE:v1\0);
                         EIGHT plan-required negatives;
                         core rule "budget admission is not
                         execution"; does NOT execute kernels;
                         does NOT emit CUDA / PTX / SASS / cubin
                         bytes; does NOT alter any upstream hash;
                         does NOT change S1.3a / FF.2 / FF.3 /
                         S1.3d court decisions; does NOT itself
                         emit a CaseFileV2Header (S1.3f next))
                         + S1.3f CaseFileV2 activation
                         integration (binds S1.3a / S1.3b /
                         S1.3c / S1.3d / S1.3e / FF.2 / FF.3 /
                         T.11g / T.11f / T.11h / corpus_hash_v1
                         / corpus_hash_v2 into a single
                         replayable authority chain that every
                         emitted case file MUST carry; 3 new
                         own-namespace hashes
                         (casefile_v2_activation_binding_hash_v1
                         under DSFB-GPU-ATLAS:CASEFILE-V2-ACTIVATION-BINDING:v1\0,
                         casefile_v2_kernel_plan_binding_hash_v1
                         under DSFB-GPU-ATLAS:CASEFILE-V2-KERNEL-PLAN-BINDING:v1\0,
                         casefile_v2_authority_chain_hash_v1
                         under DSFB-GPU-ATLAS:CASEFILE-V2-AUTHORITY-CHAIN:v1\0);
                         TEN plan-required negatives; core
                         rule "a case file must not contain
                         witness/candidate results without
                         the activation and kernel-plan
                         authority chain that made those
                         witnesses admissible"; does NOT emit
                         detector outputs / witness records /
                         fusion tensors / candidate intervals
                         / episodes; does NOT execute kernels;
                         does NOT alter any upstream hash;
                         does NOT change S1.3a / FF.2 / FF.3 /
                         S1.3d / S1.3e court decisions; does
                         NOT decide contraindications or
                         challenges (it only links them);
                         does NOT modify the registry crate)
                         + S1.3g OTelBindingReceiptTypes
                         (deterministic receipt-only schema
                         mapping OpenTelemetry spans / metrics
                         / logs / resources into
                         EvidenceDensor fields; 4 per-signal
                         binding records + 1 top-level
                         wrapper; 5 new own-namespace hashes
                         (otel_span_binding_hash_v1 under
                         DSFB-GPU-ATLAS:OTEL-SPAN-BINDING:v1\0,
                         otel_metric_binding_hash_v1 under
                         DSFB-GPU-ATLAS:OTEL-METRIC-BINDING:v1\0,
                         otel_log_binding_hash_v1 under
                         DSFB-GPU-ATLAS:OTEL-LOG-BINDING:v1\0,
                         otel_resource_binding_hash_v1 under
                         DSFB-GPU-ATLAS:OTEL-RESOURCE-BINDING:v1\0,
                         otel_binding_receipt_hash_v1 under
                         DSFB-GPU-ATLAS:OTEL-BINDING-RECEIPT:v1\0);
                         TEN plan-required negatives + 4
                         structural defect rules + a stale-
                         S1.3a-reference rename-discipline
                         scanner; core rule "mapping is not
                         ingestion; receipt type is not
                         adapter; binding schema is not
                         telemetry collection"; does NOT
                         ingest live OTLP streams / run
                         collectors / open sockets / depend
                         on an OTel SDK / claim runtime
                         interoperability; does NOT emit
                         detector outputs / witness records;
                         does NOT alter any upstream hash;
                         does NOT change S1.3a / FF.2 / FF.3 /
                         S1.3d / S1.3e / S1.3f court
                         decisions);
                         the Atlas frontier
  dsfb-gpu-atlas-registry/ literature-bound DetectorRegistryV2 generator
                         (S1.1 + S1.1.1 + S1.2 — 54×3-grid = 162 specs;
                          source_corpus_hash binds every spec to T.10)
cuda/                    .cu kernels + common.cuh (Q16.16 ops)
fixtures/                canonical synthetic trace + contract + bank
notebooks/               Colab end-to-end runner
scripts/                 scrub, docs-freshness, packaging
reports/                 per-section receipts (R.7/.8/.10/.11/.12 + T/S)
```

## Doctrine

**Foundational** (cross-cutting, applies to every layer):

> When the source artifact is fixed and reproducible, the primary
> inference layer should itself be fixed and reproducible;
> probabilistic models should enter only after deterministic
> evidence has been projected, witnessed, fused, and admitted.

Operationally:

- **GPU = deterministic evidence accelerator.** Residuals, signs,
  detector motifs, consensus, candidate intervals. No semantic
  authority on the GPU.
- **Rust/CPU = semantic authority.** Heuristics bank, 9-axis
  fusion, confuser suppression, episode collapse, court
  jurisprudence layer.
- **Case file = replayable court record.** A JSON document
  containing the full hash chain from input catalog through every
  intermediate stage to the final verdict — and now, post-T.11,
  passport / precedent / grammar / transcript / attestation /
  challenge / contraindication / coverage-hole / activation-plan
  citations.
- **No silent court logic** (plan-locked from T.11h forward).
  Every `pub` item AND every private helper in the corpus crate
  carries a doc comment whose first sentence states the WHY for a
  future engineer. The corpus crate is an internal legal system;
  silent helpers are audit-surface failures.

## Bounded v0 / Atlas scope

The DSFB-GPU-Debug v0 layer prefers correctness and inspectability
to performance and scale. These are deliberate scope cuts, not
omissions to disclaim:

- Windowing originally ran on the CPU; R.11b moved it onto a GPU
  `window_feature_kernel_structured` while preserving byte equivalence.
  The CPU window path is still the reference for Audit mode.
- Residual norms are L1 (`|x| + |y|`), not L2. No sqrt in Q16.16.
- Single-GPU only. The throughput path now uses pinned host memory
  (R.6a), async CUDA streams (R.6b), CUDA Graph capture (R.6c), and
  constant-memory thresholds (R.6d); multi-GPU sharding is deferred
  future work.
- Two fixture profiles ship: a canonical 10 000-event 16×128 fixture
  for Audit-mode parity, and a courthouse-factory scale-large
  generator (256 entities × 4096 windows) for the R.13 throughput
  headline. Both are deterministic from the same LCG seed.

## Building

```
cargo build --workspace                 # CPU-only build, works without nvcc
cargo build --workspace --features cuda # requires nvcc on PATH
cargo test  --workspace                 # all non-CUDA tests
cargo test  --workspace --features cuda # all tests
```

## Running

GPU layer (DSFB-GPU-Debug acceleration path):

```
cargo run -p dsfb-gpu-debug-demo -- generate-fixture --out fixtures/synthetic_trace.json
cargo run -p dsfb-gpu-debug-demo -- run-cpu  --fixture fixtures/synthetic_trace.json \
                                              --contract contract.toml \
                                              --out reports/cpu_case.json
cargo run -p dsfb-gpu-debug-demo --features cuda -- run-gpu \
                                              --fixture fixtures/synthetic_trace.json \
                                              --contract contract.toml \
                                              --out reports/gpu_case.json
cargo run -p dsfb-gpu-debug-demo -- compare  --cpu reports/cpu_case.json \
                                              --gpu reports/gpu_case.json \
                                              --out reports/replay_comparison.json
```

Atlas layer (DSFB-GPU-Atlas jurisprudence court — host-only):

```
# Sealed corpus surfaces (T.1-T.10):
cargo run -p dsfb-gpu-atlas-corpus -- verify
cargo run -p dsfb-gpu-atlas-corpus -- report
cargo run -p dsfb-gpu-atlas-corpus -- genealogy-{dot,json}

# T.11 court (jurisprudence layer):
cargo run -p dsfb-gpu-atlas-corpus -- passport <canonical_id>
cargo run -p dsfb-gpu-atlas-corpus -- precedents
cargo run -p dsfb-gpu-atlas-corpus -- admissibility
cargo run -p dsfb-gpu-atlas-corpus -- trial-transcript
cargo run -p dsfb-gpu-atlas-corpus -- execution-attestation
cargo run -p dsfb-gpu-atlas-corpus -- challenges
cargo run -p dsfb-gpu-atlas-corpus -- contraindication <canonical_id>
cargo run -p dsfb-gpu-atlas-corpus -- coverage-holes

# S1.3a activation plan:
cargo run -p dsfb-gpu-atlas-corpus -- activation-plan

# Bulk-emit any surface to its `out/` artifact:
cargo run -p dsfb-gpu-atlas-corpus -- <surface>-emit --out-dir crates/dsfb-gpu-atlas-corpus/out
```

S1.2 detector registry:

```
cargo run -p dsfb-gpu-atlas-registry --bin dsfb-registry-emit
```

## Reproducibility

**GPU layer**: two consecutive runs with identical fixture,
contract, bank, and detector registry produce byte-identical case
files. This is the load-bearing property. See
`crates/dsfb-gpu-debug-core/tests/replay.rs` and
`tests/cross_stage_chain.rs`.

**Atlas layer**: two builds of any court surface against the same
sealed inputs produce byte-identical bytes. Every T.11 surface
plus S1.3a carries its own canonical-byte hash under a domain-
separated namespace; sensitivity tests in each surface's
`tests/<surface>_invariants.rs` pin that a one-byte mutation
anywhere in the inputs changes the output hash. The corpus crate
ships ≥ 1969 acceptance tests in total across 52 test groups.

## Prior-Art Archive

The prior-art manuscript and source are archived outside this crate
checkout. The archived PDF hash, page count, byte count, and public
deposit locator are pinned in `ARTIFACT_MANIFEST.v1.toml` and
`TIMESTAMP_RECEIPT.md`. The crate itself carries the Rust, CUDA,
notebook, metadata, and receipt surface needed for local verification
and crates.io packaging.

## Performance — current headline (R.13 sealed, D64 post-R.11c)

The headline of DSFB-GPU-Debug is the **D64 throughput path's
full-pipeline campaign reduction** across the R.9–R.11 optimization
campaign, measured on the courthouse-factory scale-large fixture
(256 entities × 4096 windows). Hardware: RTX 4080 SUPER, CUDA 13.2.
Head commit: `99a0f3b` (R.9.d.1 D128 committed separately as a
scaling-ladder proof; D128 is NOT in this headline by design).

| commit    | section              | per-cat µs | cat/sec | compound reduction |
|-----------|----------------------|-----------:|--------:|-------------------:|
| `122139e` | R.9.b.3 baseline     |  1,820,000 |    0.55 |              1.0×  |
| `80303a2` | R.10a axis-5 hoist   |    128,893 |    7.76 |             14.1×  |
| `fd71dc3` | R.10b compact digest |     84,946 |   11.77 |             21.4×  |
| `e084075` | R.10c parallel cand. |     75,329 |   13.27 |             24.2×  |
| `31e6b49` | R.11b GPU features   |     42,513 |   23.52 |             42.8×  |
| `086e209` | R.11c compact events |     33,144 |   30.18 |          **54.9×** |

Plan-safe sentence (verbatim, locked):

> On the RTX 4080 SUPER / CUDA 13.2 scale-large courthouse-factory
> workload, the D64 throughput path improved from approximately
> 1.82 s per catalog to 33.1 ms per catalog across the R.9–R.11
> optimization campaign, a measured ~55× full-pipeline reduction
> while preserving deterministic replay, audit-mode golden hashes,
> and the Semantic Non-Bypass Axiom.

**Critical overclaim guardrail**: the ~55× number is a **full-pipeline
campaign reduction** (commit-to-commit on the same fixture), NOT a
"GPU is 55× faster than CPU" claim. `reports/r12_d64_saturation.txt`
records that the CPU Layer B D64 wide-path comparator is **deferred
to R.12b.1** — there is no CPU `consensus_grid_wide` /
`candidate_collapse_wide` driver in core today, so the
`spd_vs_cpub` column stays `—` until R.12b.1 lands. The full sweep
across K ∈ {1, 4, 16, 32, 64, 128} × scale ∈ {canonical, mid, full}
is in [reports/r12_d64_saturation.txt](reports/r12_d64_saturation.txt);
the curated R.13 headline is in [reports/money_table.txt](reports/money_table.txt).

At full 256×4096 K=64 (the R.13 headline cell, formally measured by
R.12b post-R.11c): **27 781 µs/catalog, 36.0 catalogs/sec,
3.77 × 10⁷ cells/sec, 2.42 × 10⁹ detector-evaluations/sec,
1917 episodes/catalog**. Episode counts are byte-identical across
the entire trajectory (canonical K=1 → 13, mid K=1 → 89, full K=1 →
1917; episodes do not depend on K).

## Performance — historical optimization trail (R.7 / Tier 3A / Tier 3B / R.6)

The numbers below are preserved for the architectural-trajectory
record — they are the *pre-R.11c* state, NOT the current headline.
The R.7 D16 money table is archived at
[reports/money_table_r7_baseline.txt](reports/money_table_r7_baseline.txt).

R.7 D16 sweep (pre-R.11c, historical), representative run on the
courthouse-factory fixture at the same hardware:

| fixture            | K   | layer     | per-catalog µs | spd vs CPU B |
|--------------------|----:|-----------|---------------:|-------------:|
| canonical 16×128   |  32 | CPU B     |          1 716 |         1.0× |
| canonical 16×128   |  32 | GPU A     |          1 475 |         1.1× |
| canonical 16×128   |  32 | GPU B     |          1 811 |         0.9× |
| scaled 256×4096    |   1 | CPU B     |        888 300 |         1.0× |
| scaled 256×4096    |  16 | GPU B     |        327 100 |     **2.7×** |
| scaled 256×4096    |  64 | GPU B     |        296 873 |     **2.9×** |
| scaled 256×4096    |  64 | GPU A     |        627 059 |         1.4× |

The pre-R.11c Audit / Throughput / Tier 3A / Tier 3B ladder on the
canonical 16×128 fixture (D16 detectors):

| mode | median wall | throughput |
|------|-------------|------------|
| CPU Audit                              | 5 486 µs        | 182 cases/s |
| CPU Throughput (O.15 compact input)    | 1 694 µs        | 590 cases/s |
| GPU Audit (workspace + cell-parallel)  | 5 914 µs        | 169 cases/s |
| GPU Throughput (O.15 + workspace)      | 2 705 µs        | 370 cases/s |
| GPU Batched K=4 (O.16 Tier 2)          | 1 701 µs/cat    | 587 cases/s |
| GPU Batched K=32 (O.16 Tier 2)         | 1 682 µs/cat    | 594 cases/s |
| GPU Tier 3B K=32 (O.17, device SHA-256)| 1 891 µs/cat    | 528 cases/s |

Tier 3B (O.17) is the on-device per-stage SHA-256 path (four
`__device__` SHA-256 kernels hash the residual / sign / detector /
consensus cell buffers in place; only the 4 × 32-byte digests cross
PCIe). At the canonical fixture this was a measured regression
(small per-stage buffers); at scaled 256×1024 with K=16 it delivered
a 1.36× speedup over Tier 3A (60 748 µs/catalog vs 82 432 µs/catalog).
Both numbers are pre-R.11c and pre-R.10a, included here as the
historical pre-campaign baseline. The R.13 D64 campaign that followed
moved well past these numbers; see the headline above.

R.8.5 introduced a deterministic domain-separated tree digest
(`DSFB_STAGE_TREE_V1`) for the Throughput path; Audit mode retains
canonical serial SHA-256 and Audit golden hashes are unchanged. The
digest mode is recorded in the case file so replay cannot silently
mix Audit and Throughput receipts.

R.6a/b/c/d landed pinned host memory, async CUDA streams, opt-in
CUDA Graph capture (`graph_plan_hash` recorded in the case file
when captured), and opt-in `__constant__`-memory detector
thresholds. All are byte-equivalence-pinned to the pageable / sync
reference path.

## Detector corpus (T.1a + T.1b + T.2 + T.3 + T.4 + T.5 + T.6 + T.7 + T.8 + T.9 + T.10 — sealed)

After the R.13 publication lock sealed, the campaign opened
**Section T — Literature Detector Corpus and Detector Canonicalisation
Court** in a new host-only crate at
[crates/dsfb-gpu-atlas-corpus/](crates/dsfb-gpu-atlas-corpus/).
The crate is intentionally decoupled from the GPU acceleration story:
no CUDA dependency, no hash-chain coupling yet, zero external deps.

- **T.1a** landed the structural schema and a 15-primitive seed.
- **T.1b** expanded the seed to 54 high-confidence literature
  primitives across 16 primitive families (SPC, robust statistics,
  distribution distance, sequential change-point, spectral, wavelet,
  tabular constraint, categorical histogram, missingness, residual
  observer, projection residual, multivariate hypothesis,
  operability diagnostic, debug/observability, rank statistic,
  negative witness).
- **T.2** added a TOML source-ingestion format. The 54-record corpus
  is mirrored at
  [crates/dsfb-gpu-atlas-corpus/corpus/corpus.toml](crates/dsfb-gpu-atlas-corpus/corpus/corpus.toml)
  and a hand-rolled zero-dep TOML-subset parser + loader proves
  byte-equivalence against the static seed via 14 acceptance tests.
  The static seed stays as the canonical fixture; the TOML is the
  parallel data path. Splitting into 11 source-class files is
  deferred to T.2.1 if needed.
- **T.3** introduces the five-hash detector identity:
  `source_hash`, `formula_hash`, `parameter_hash`,
  `implementation_hash`, `semantic_role_hash`, plus the composite
  `detector_identity_hash` over (formula, parameter, semantic
  role). The composite **deliberately omits** `source_hash` and
  `implementation_hash` — the load-bearing philosophical claim,
  pinned by `source_hash_does_not_define_detector_identity` and
  `implementation_hash_does_not_define_detector_identity`, is that
  the corpus can fix citations and upgrade the L-band ladder
  without breaking canonical equivalence classes. 18 acceptance
  tests pin the change-localisation, dump/load parity, domain-
  separator versioning, and equivalence-class properties. Used by
  T.4's dedup court.
- **T.4** introduces the deterministic dedup-court skeleton with
  the first explicit batch of judgments. A separate `CLAIMS` seed
  declares 12 alias-name claims (Robust z's 3 aliases, PCA SPE /
  Q residual's 3, Hotelling T-squared's 2, Page-Hinkley's 2,
  Jensen-Shannon's 2). The court emits one `DedupRecord` per
  subject (54 canonical seed records + 12 alias claims = 66
  records total), each carrying a `CanonicalisationDecision` and
  a `DedupReason`. Western Electric (canonical 16) and Nelson
  rules (canonical 17) are classified as `CompositionOf`
  judgments over Shewhart — keeping their canonical records but
  recording their compositional structure. 17 acceptance tests
  pin determinism, the philosophical invariants (semantic-role
  difference prevents alias collapse), and the verify pass.
  No fuzzy similarity scoring, no probabilistic dedup — every
  judgment is a deterministic policy decision with an explicit
  reason code.
- **T.5** introduces the deterministic detector genealogy graph
  over the court records. 66 nodes (54 canonical + 12 alias
  claims) and 42 edges, all derived from either (a) the seed's
  existing `GenealogyEdges` field or (b) T.4 court decisions
  (`AliasOf` → `AliasCollapsedInto`, `CompositionOf` →
  `Composes`). DAG-verified over the strict-ancestry edges; DOT
  + JSON exports (schema `DSFB-GPU-ATLAS:GENEALOGY:v1`) ship as
  [reports/corpus_t5_genealogy.dot](reports/corpus_t5_genealogy.dot)
  and [reports/corpus_t5_genealogy.json](reports/corpus_t5_genealogy.json).
  18 acceptance tests pin DAG-ness, byte-deterministic exports,
  the "T.4 court decisions imply T.5 graph edges" audit
  invariant, cycle rejection on synthesised fixtures, and the
  alias-claim-has-exactly-one-incoming-edge property. Fixed a
  pre-existing latent cycle in Nelson rules' seed
  (`derived_from=[1, 16]` + `generalizes=[16]` was
  self-contradictory; `generalizes=[]` per plan discipline
  "missing edge beats wrong edge").

  ```
  cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- \
      genealogy-dot --out genealogy.dot
  dot -Tsvg genealogy.dot -o genealogy.svg
  ```

- **T.6** adds the witness-law layer over the corpus court. Eight
  plan-locked fusion planes (`ProvenanceAdmissibility`,
  `NumericStrength`, `TemporalStructure`, `CrossSignalStructure`,
  `DistributionStructure`, `SemanticBankStructure`,
  `ReliabilityConfuserControl`, `TaskUtility`); a deterministic
  `axes_to_planes` mapping from the v1 9-axis fusion onto the 8
  planes; and a declarative `COMPATIBILITY_RULES` table covering
  Confuser/Primary/CleanWindow/Corroborating/Distribution/Topology/
  Timing pair semantics. The public report grows by six sections
  ((7) negative-witness histogram, (8) fusion-plane histogram,
  (9) role × plane coverage matrix, (10) witness-law coverage
  invariants, (11) primary-witness list, (12) confuser list). 24
  acceptance tests pin every-detector-has-role,
  every-axis-maps-to-at-least-one-plane,
  confuser-roles-have-negative-witness, primary-witnesses-cannot-be-
  negative-only, clean-window-cannot-admit-alone, and the
  declarative rule-table integrity (no self-loops, Confuser
  suppresses Primary, CleanWindow incompatible with Primary). The
  full T.6 corpus report is committed at
  [reports/corpus_t6_report.txt](reports/corpus_t6_report.txt) and
  the per-section discipline receipt at
  [reports/corpus_t6_regression_check.txt](reports/corpus_t6_regression_check.txt).

- **T.7** lands the implementation-status (L-band) ladder as an
  auditable honesty claim. **The L-band is an honesty marker, not
  a quality score.** A detector at L1 is cited and canonicalised;
  it is not "worse" than one at L6. The T.7 verifier rejects any
  record whose L-band exceeds the workspace's actual evidence:
  L5/L6 require canonical-id membership in
  `GPU_IMPLEMENTED_CANONICAL_IDS` (the dsfb-gpu-debug-core bank
  surface — IDs 14, 15, 41, 42, 43 mapping to LatencyRamp,
  ConfuserTransient, ErrorBurst, SlewShockRecovery, and
  FanoutCascadeCandidate); L7 (`BenchmarkCharacterised`) is
  forbidden at T.7 until a per-detector benchmark artifact exists;
  L8 (`LedgerCharacterised`) is forbidden until T.8 lands the
  usefulness ledger. The public report grows by one section
  ((13) L-band honesty invariants — histogram + GPU whitelist +
  verifier result). 17 acceptance tests pin the
  every-detector-has-exactly-one-L-band invariant, the
  histogram-sums-to-seed-count invariant, the
  whitelist-is-sorted-and-deduplicated invariant, and the four
  rejection rules (forged L5/L6 for non-whitelisted ids, any L7,
  any L8). The full T.7 corpus report is committed at
  [reports/corpus_t7_report.txt](reports/corpus_t7_report.txt) and
  the per-section discipline receipt at
  [reports/corpus_t7_regression_check.txt](reports/corpus_t7_regression_check.txt).

- **T.8** lands the deterministic detector usefulness ledger
  shell. **The usefulness ledger is an audit surface, not a
  learned ranking model.** T.8 records declared evidence levels
  and conservative contribution fields; empirical usefulness
  remains unclaimed until a row is backed by a named benchmark
  artifact. The existing 7-field embedded `UsefulnessLedgerRow`
  was renamed to `UsefulnessLedgerSnapshot` (zero-init per-detector
  prior summary; byte-identical layout preserved). A new
  [`src/usefulness.rs`](crates/dsfb-gpu-atlas-corpus/src/usefulness.rs)
  module introduces the richer `UsefulnessLedgerRow` keyed by
  `(canonical_id, task_id, domain, dataset_id)` plus the
  `UsefulnessEvidenceLevel` honesty ladder
  (`Unmeasured` / `LiteraturePrior` / `RoleSeeded` /
  `SyntheticFixtureMeasured` / `RealDatasetMeasured` /
  `CrossDomainReplicated` / `RetiredByEvidence`), the
  `UsefulnessScoreKind` enum (`NotScored` / `PriorScore` /
  `MeasuredScore`), the deterministic `usefulness_score(...)`
  policy, the 11-rule `verify_usefulness_ledger(...)` checker,
  and the `USEFULNESS_LEDGER` static (54 rows — one per canonical
  detector). The conservative T.8 seed marks the 5 GPU-implemented
  IDs (14, 15, 41, 42, 43) as `RoleSeeded` / `Active` /
  `GpuSurfaceSeededFromDsfbGpuDebugCore`, and the remaining 49 as
  `LiteraturePrior` / `Dormant` / `LiteraturePriorOnly`. Every row
  carries `score_kind = NotScored` and zero empirical fields. The
  verifier rejects (a) unmeasured rows claiming empirical gain,
  (b) L8 records without measured ledger evidence, (c) retired
  states without measured negative evidence, (d) GPU-active claims
  outside the L5/L6 whitelist, (e) duplicate triples, (f) unknown
  detector IDs, (g) reason/evidence inconsistency, (h) nonzero
  scores on `NotScored` rows, (i) Active+Retired on the same
  triple, (j) missing task / dataset / domain, and (k) coverage
  gaps. The public report grows by one section ((14) Usefulness
  ledger honesty invariants — evidence-level histogram +
  lifecycle cross-check + no-fabricated-claims invariant +
  verifier result). 37 acceptance tests pin every plan-required
  rule. The full T.8 corpus report is committed at
  [reports/corpus_t8_report.txt](reports/corpus_t8_report.txt) and
  the per-section discipline receipt at
  [reports/corpus_t8_regression_check.txt](reports/corpus_t8_regression_check.txt).

- **T.9** lands the **internal corpus audit report bundle**. T.9
  is a cold-reader internal audit artifact, NOT an external
  release artifact: no Zenodo deposit, no DOI, no publication
  metadata, no `corpus_hash_v1`, no `CaseFileV2` integration.
  The bundle is a deterministic four-file output emitted by a
  new `dsfb-corpus report-bundle` subcommand:
  [reports/corpus_t9_audit_report.txt](reports/corpus_t9_audit_report.txt)
  (human-readable, 10 stable top-level sections),
  [reports/corpus_t9_audit_report.json](reports/corpus_t9_audit_report.json)
  (machine-readable, schema `DSFB-GPU-ATLAS:CORPUS-AUDIT-REPORT:v1`),
  and refreshed
  [reports/corpus_t9_genealogy.dot](reports/corpus_t9_genealogy.dot)
  /
  [reports/corpus_t9_genealogy.json](reports/corpus_t9_genealogy.json)
  (byte-identical to the T.5 exports). Counts in the bundle are
  cross-checked against the T.1-T.8 source modules at test time
  (court counts against `court::classify_all`, L-band against
  `lband::compute_histogram`, usefulness against the ledger
  verifier, genealogy against the graph builder). The TXT
  carries a hard non-claim section listing what the report does
  NOT claim (implementation, usefulness, GPU readiness for L0-L4,
  `corpus_hash_v1`, activation planning) plus the deferred-gates
  list (T.10 / T.11 / R.9.d.2 / Section S Phase 1). 31 acceptance
  tests pin determinism, cross-source consistency, honesty
  invariants (aliases not counted as unique primitives; L0-L4 not
  GPU-ready; L7/L8 forbidden), and publication-language
  exclusion (no Zenodo / DOI / deposit markers anywhere in the
  rendered TXT or JSON). The per-section discipline receipt is at
  [reports/corpus_t9_regression_check.txt](reports/corpus_t9_regression_check.txt).

```
cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- verify
cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- report
cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- genealogy
cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- dump --out corpus.toml
cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- load-check
cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- report-bundle
```

The architecture's deterministic evidence object is a **densor**
(deterministic, contract-bound, hash-addressed multidimensional
evidence structure), not a tensor. Tensors are ML latent carriers;
densors are auditable evidence objects whose cells are produced,
ordered, reduced, and admitted by replayable rules. T.2 keeps the
schema neutral on this distinction; T.2.1+ introduces the
`EvidenceDensor` / `WitnessDensor` / `FusionDensor` /
`CandidateDensor` / `BankDensor` / `ReplayDensor` type aliases as
the canonical Atlas vocabulary.

What is **NOT** yet in the corpus crate (these land in T.10+):
the `corpus_hash_v1` that binds the corpus into a future
`CaseFileV2` (T.10–T.11), the GPU family-kernel mapping that
unblocks Section S1.2 (T.11), and any external Zenodo / deposit
artifact (deliberately deferred — T.9 is internal-only).
Five-hash detector identity (T.3), the dedup court (T.4), the
genealogy DOT/JSON export (T.5), the witness-role fusion
semantics (T.6), the L-band ladder (T.7), the usefulness ledger
schema + verifier + conservative seed (T.8), and the internal
audit report bundle (T.9) are all landed. **Measured** empirical
usefulness remains unclaimed across the corpus; the T.8 verifier
rejects any row that fabricates a benchmark number, and the T.9
audit report's honesty invariants pin that posture.

## Atlas algebra status (S1.1 type surface + S1.1.1 post-T.10 cross-field rule)

S1.1 defined the detector algebra type surface; S1.1.1 added the
post-T.10 cross-field rule binding every spec to either an
all-zero `source_corpus_hash` (pre-hash fixture) or the canonical
`corpus_hash_v1` (post-T.10 registry-bound). T.10 has landed (see
the "Corpus identity freeze" section below); the S1.2 registry
generator is a separate follow-on commit. The S1.1/S1.1.1 crate
itself does NOT generate the parameterized DetectorSpec registry,
compute `registry_hash_v2`, emit `CaseFileV2` bodies, or execute
Atlas family kernels. Its purpose is to pin the deterministic
grammar + cross-field binding rule that S1.2 generation must obey.

The crate lives at
[crates/dsfb-gpu-atlas-registry/](crates/dsfb-gpu-atlas-registry/)
and exposes:

- **`DetectorFamily`** — 43 plan-recommended family variants
  (Shewhart, EWMA, CUSUM, RobustZMad, KolmogorovSmirnov, …,
  LatencyRamp, EvmAnomaly). The order is plan-locked; changing
  it would invalidate every derived `FamilyId`.
- **`Transform` / `Statistic` / `Comparator` / `Gate` /
  `WindowSpec`** — the algebra's parameter coordinates.
- **`DomainTag` / `DomainTagSet` / `AxisBinding`** — bit
  positions byte-identical to the corpus crate's
  [`dsfb_gpu_atlas_corpus::types::DomainTagSet`].
- **`CostClass` / `NumericMode` / `ImplementationKind`** — coarse
  cost / arithmetic / kernel-surface tags. Defaults pinned:
  `NumericMode::Q16_16` for audit mode; `ImplementationKind::ScalarCpu`
  is the spec-construction default (explicitly NOT a GPU claim).
- **`DetectorTemplate`** — the algebra-grid expansion point.
  Requires a `primitive_id: DetectorCanonicalId` linking the
  template back to a corpus literature primitive.
- **`DetectorSpec`** — one fully-resolved detector identity with
  canonical name + parameter hash + axis binding + corpus binding.
- **`CorpusBindingStatus`** — honest enum
  (`PreHashT9InternalAudit` / `HashFrozenT10`) paired with a
  `source_corpus_hash: [u8; 32]` field on every spec. **Post-T.10**
  the verifier enforces the cross-field rule
  `HashFrozenT10 ⇔ source_corpus_hash != [0; 32]`. See the
  later "T.10 — corpus_hash_v1" section for the canonical
  `corpus_hash_v1` bytes that S1.2-generated specs bind to.
- **`CanonicalDetectorName`** — plan-locked naming grammar:
  `{FAMILY}__{TRANSFORM}__W{WINDOW}__{STATISTIC}__{COMPARATOR}__P{PERSISTENCE}`.
  Example: `ROBUST_Z_MAD__RESIDUAL__W64__MAD__TWO_SIDED__P3`.

The crate has 28 acceptance tests in five files covering the
plan-required invariants: canonical-name stability, family
ordering, parameter-hash gating, the post-T.10 cross-field
corpus-binding rule
(`detector_spec_rejects_hash_frozen_without_source_corpus_hash`
+ `detector_spec_admits_hash_frozen_t10_with_non_zero_corpus_hash`
+ `detector_spec_rejects_pre_hash_with_non_zero_source_corpus_hash`),
the registry-level verifier
(`verify_registry_spec_admits_hash_frozen_t10_with_live_corpus_hash`
plus three rejection paths covering stale corpus hash, unknown
primitive_id, and pre-hash status),
implementation-kind-is-not-gpu-claim-by-default, and verifier
determinism.

**S1.1 does NOT do**: 2,000-detector generation, `registry_hash_v2`,
CUDA family kernels, NVRTC / JIT Mode B, Detector Delta Ledger
writes, D512 / D1024 / D2000 ladder, or any publication claim
about Atlas detector count. The R.13 D64 headline + D128 / D205
scaling-ladder proofs are unchanged.

## Corpus identity freeze (T.10 corpus_hash_v1 + CaseFileV2 header)

T.10 freezes the T.1–T.9 corpus material into `corpus_hash_v1`
(a 32-byte SHA-256 commitment over a canonical-byte projection
of the live corpus) and introduces a minimal `CaseFileV2Header`
receipt that can carry that hash through future case-file
chains. The hash is computed in [`dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1`](crates/dsfb-gpu-atlas-corpus/src/corpus_hash.rs)
under the plan-locked domain separator `DSFB-GPU-ATLAS:LITERATURE-CORPUS:v1\0`.

The current `corpus_hash_v1`:

```
35c276c73a52d916daafda2598b215d73e7fd694d4a0673e34ac1ef948f5a4b7
```

The hash is NOT computed over rendered TXT/JSON reports. It is
over the canonical bytes of all 54 detector records, 66
dedup-court decisions, 12 alias claims, 54 usefulness-ledger
rows, and the schema/version strings — every byte that future
T-section work might mutate is covered. Re-rendering the public
reports does NOT change the hash; mutating any T.1–T.9 payload
byte WILL.

[`CaseFileV2Header`](crates/dsfb-gpu-debug-core/src/casefile_v2.rs)
is a 7-field receipt struct in `dsfb-gpu-debug-core` carrying
`corpus_hash_v1`, `corpus_stage = FrozenT10`, the active
`DetectorProfile`, its registry hash, `atlas_algebra_status =
S1_1TypeSurfaceOnly`, and `semantic_non_bypass = true`. The
[`casefile_v2_header_hash`](crates/dsfb-gpu-debug-core/src/casefile_v2.rs)
function produces a 32-byte commitment over the header under the
domain separator `DSFB-GPU-ATLAS:CASEFILE-V2-HEADER:v1\0`. The
verifier rejects three load-bearing failures: all-zero
`corpus_hash_v1`, `semantic_non_bypass = false` (the Semantic
Non-Bypass Axiom is non-negotiable), and
`corpus_stage = InternalAuditPreFreeze` at T.10.

T.10 has 33 acceptance tests (17 in
[tests/corpus_hash_v1.rs](crates/dsfb-gpu-atlas-corpus/tests/corpus_hash_v1.rs)
and 16 in
[tests/casefile_v2_header.rs](crates/dsfb-gpu-debug-core/tests/casefile_v2_header.rs))
covering determinism, domain-separator load-bearingness,
section-coverage in the hash material, one-bit mutation
sensitivity, the no-report-rendering invariant, header field
shape, hash sensitivity to corpus / profile / non-bypass
changes, verifier rejection rules, and the T.10 non-claims
(no `registry_hash_v2`, no detector registry generation,
D16/D64/D128/D205 audit-path hashes unchanged).

**T.10 freezes corpus_hash_v1 and introduces a CaseFileV2 header
receipt. It does NOT generate `registry_hash_v2`, does NOT emit
the 2,000-detector Atlas registry, does NOT run Atlas family
kernels, does NOT publish a corpus artifact externally, and
does NOT change the R.13 D64 headline.** The header receipt is
the audit anchor that lets future Atlas case files chain back
to the T.10-frozen corpus identity without requiring those
future commits to re-implement the freeze. Receipts and the
discipline checklist are committed at
[reports/t10_corpus_hash_receipt.txt](reports/t10_corpus_hash_receipt.txt),
[reports/t10_casefile_v2_header_receipt.txt](reports/t10_casefile_v2_header_receipt.txt),
and
[reports/t10_regression_check.txt](reports/t10_regression_check.txt).

## Atlas registry generation (S1.2 — 162 specs, plan-locked first pass)

S1.2 is the literature-bound registry generator. It walks the 54
T.10-frozen corpus primitives across a plan-locked 3-point
parameter grid and emits 162 `DetectorSpec` records, all bound
to `corpus_hash_v1`. The grid:

```
point 0 : (W32,  P=2, comparator=HIGH,       gate=PERSISTENCE)
point 1 : (W64,  P=3, comparator=TWO_SIDED,  gate=PERSISTENCE)
point 2 : (W128, P=5, comparator=TWO_SIDED,  gate=PERSISTENCE)
```

Each generated spec carries `corpus_binding_status =
HashFrozenT10`, `source_corpus_hash = compute_corpus_hash_v1()`,
`primitive_id = Some(record.canonical_id)`, and
`ImplementationKind::ScalarCpu` (honesty rule — the registry
generator has no GPU dispatch layer yet, so even the five L6
corpus records do not claim a GPU surface here).

Counts and pinned hashes:

```
literature_primitives : 54
parameterized_specs   : 162
active_detectors      : 0    (no activation planner at S1.2)
admitted_episodes     : 0    (no GPU execution at S1.2)

corpus_hash_v1     : 35c276c73a52d916daafda2598b215d73e7fd694d4a0673e34ac1ef948f5a4b7
registry_hash_v2   : d3cf63000cee922818e8dbc79ffecbc27d288063efbaed589e1eb1812bc37a08
```

The canonical registry artifacts are committed at
[crates/dsfb-gpu-atlas-registry/out/detector_registry_v2.bin](crates/dsfb-gpu-atlas-registry/out/detector_registry_v2.bin)
(42 286 bytes, canonical-byte material) and
[crates/dsfb-gpu-atlas-registry/out/detector_registry_v2.json](crates/dsfb-gpu-atlas-registry/out/detector_registry_v2.json)
(127 010 bytes, human-readable mirror). They are regenerated by:

```
cargo run --bin dsfb-registry-emit -p dsfb-gpu-atlas-registry
```

Two invocations produce byte-identical output (pinned by 12
acceptance tests in `tests/s1_2_generator.rs`).

**S1.2 does NOT do**: 2,000-detector generation (the wider grid
is S1.2.1+), activation planning, GPU execution, `CaseFileV2`
body emission, NVRTC / JIT Mode B, Detector Delta Ledger writes,
D512 / D1024 / D2000 ladder, or any publication claim about
Atlas detector count. R.13 D64 headline + D128 / D205
scaling-ladder proofs are unchanged.

Receipts:
[reports/s1_2_registry_summary.txt](reports/s1_2_registry_summary.txt),
[reports/s1_2_registry_verification.txt](reports/s1_2_registry_verification.txt),
[reports/s1_2_regression_check.txt](reports/s1_2_regression_check.txt).

## Detector passport (T.11a — per-detector legal-identity packet)

T.11a turns the corpus from "internally correct" into
"inspectable." Every SEED canonical record now resolves to a
single hashable `DetectorPassport` that gathers every T.1–T.10
fact about that detector: canonical id, aliases, source refs,
the five T.3 identity hashes + composite `detector_identity_hash`,
T.4 dedup decision + reason, T.5 genealogy edges, T.6 witness
role + fusion planes, T.7 L-band, T.8 lifecycle + usefulness
evidence level, and the eight constitution flags. The 32-byte
`passport_hash` covers every byte of every field.

CLI:

```
dsfb-corpus passport <canonical_id> [--json]
dsfb-corpus passports-emit [--out-dir DIR]
```

Sample (canonical_id = 1, Shewhart control chart; T.11b passport
extended with `linked_precedent_ids`):

```
DetectorPassport (canonical_id = 1)
  display_name              : Shewhart control chart
  aliases                   :
                                - 3-sigma rule
                                - x-bar chart
  source_refs               :
                                - shewhart1924 (1931) [Van Nostrand (book); Bell Labs memo 1924]
  primitive_family          : ScalarThreshold
  mathematical_form         : StandardisedDeviation
  decision_functional       : TwoSided
  detector_identity_hash    : b8102a9e1c4dfdbd6224acc09b8c1d1d730e1cf3a12fd7b6ecb04d58eac74569
  dedup_decision            : Canonical
  dedup_reason              : OriginRecord
  witness_role              : Primary
  fusion_planes             :
                                - NumericStrength
  implementation_level      : L1_Canonicalised
  lifecycle_state           : Active
  usefulness_evidence_level : LiteraturePrior
  linked_precedent_ids      : 13, 14, 15, 16, 17, 18, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83
  passport_hash             : 7bbca7282908d41e0a1bcb0a87bd16052b23c7beb5567f4db64c1a825d14c465
```

Bulk artifacts (regenerable; two builds byte-identical):
[crates/dsfb-gpu-atlas-corpus/out/passports.txt](crates/dsfb-gpu-atlas-corpus/out/passports.txt)
(114 782 bytes; one passport per SEED record) and
[crates/dsfb-gpu-atlas-corpus/out/passports.json](crates/dsfb-gpu-atlas-corpus/out/passports.json)
(91 168 bytes; deterministic JSON array). The passport count
equals `SEED.len()` (54); alias-side claims live in
`claims::CLAIMS` and never inflate the passport count.

**T.11a does NOT do**: emit `CaseFileV2` episode-transcript
bodies (T.11d), implement court precedents (T.11b),
admissibility grammar snapshots (T.11c), unit-semantics /
sampling-law receipts (T.11e), S1.3 activation planning,
external provenance export (DSFB-PROV / OpenLineage / NIST AI
RMF / RO-Crate), or claim learned detector usefulness. T.11a
also does NOT include S1.2 registry-spec linkage on the
passport (deferred to keep the corpus crate registry-free).
The passport hash is DSFB-native; no in-toto / SLSA / SPDX /
CycloneDX compatibility claim.

15 acceptance tests in
[tests/passport_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/passport_invariants.rs)
pin: passport hash determinism + sensitivity to every T.3 /
T.6 / T.7 / T.8 / constitution-flag axis, the alias-not-counted
rule, the L6-claims-require-L-band-verifier-admit invariant,
text / JSON rendering determinism, and pairwise-distinct hashes
across all 54 SEED records. Receipts:
[reports/t11a_passport_summary.txt](reports/t11a_passport_summary.txt),
[reports/t11a_passport_verification.txt](reports/t11a_passport_verification.txt),
[reports/t11a_regression_check.txt](reports/t11a_regression_check.txt).

## Court precedents (T.11b — deterministic jurisprudence over T.4 / T.6 / T.7 / T.8 / T.10 / S1.2 / T.11a)

T.11b turns the corpus's existing rules into a **cumulative
precedent ledger**. Every alias collapse, composition judgment,
witness-role law, L-band honesty rule, usefulness-honesty rule,
corpus-hash freeze rule, registry-binding rule, constitution-
coverage rule, and plan-locked deferred gate is projected into
a single canonical-sorted set of `CourtPrecedent` records. The
plan framing:

> The Atlas court is not merely a set of current
> classifications; it carries deterministic precedents
> explaining why each alias, composition, witness role,
> implementation claim, and usefulness claim is admitted,
> rejected, or deferred.

CLI:

```
dsfb-corpus precedents      [--json] [--out PATH]
dsfb-corpus precedents-emit [--out-dir DIR]
```

Precedent counts at T.11b (plan-locked at 83 total):

```
DedupCanonical             : 52
AliasCollapse              : 12
CompositionJudgment        :  2   (Western Electric, Nelson)
ParameterizationJudgment   :  0
SemanticRoleSeparation     :  0   (reserved for future
                                   stochastic-reduction admissions)
WitnessLaw                 :  2
NegativeWitnessLaw         :  1
LBandHonestyLaw            :  3
UsefulnessHonestyLaw       :  2
CorpusHashLaw              :  1
RegistryBindingLaw         :  2
ConstitutionLaw            :  2
DeferredGateLaw            :  4
```

Pinned hashes (the court-layer receipt):

```
precedent_hash_v1 : 6721f511f1eb951ba7eff4fa36832f233331507f6e4208d4f97866afd984dd14

corpus_hash_v1    : 35c276c73a52d916daafda2598b215d73e7fd694d4a0673e34ac1ef948f5a4b7  (unchanged)
registry_hash_v2  : d3cf63000cee922818e8dbc79ffecbc27d288063efbaed589e1eb1812bc37a08  (unchanged)
```

`corpus_hash_v1` stays frozen — `precedent_hash_v1` is a
**separate cumulative receipt**, not a re-freeze of the corpus.

Passport extension (T.11a × T.11b): every `DetectorPassport`
gains a `linked_precedent_ids: Vec<PrecedentId>` field carrying
every global law plus every per-record precedent that references
the canonical id. The passport hash includes the linkage bytes;
the bulk artifacts
[crates/dsfb-gpu-atlas-corpus/out/passports.txt](crates/dsfb-gpu-atlas-corpus/out/passports.txt)
and
[crates/dsfb-gpu-atlas-corpus/out/passports.json](crates/dsfb-gpu-atlas-corpus/out/passports.json)
are regenerated. The Shewhart sample above shows the new shape.

Bulk artifacts (regenerable; two builds byte-identical):
[crates/dsfb-gpu-atlas-corpus/out/court_precedents.txt](crates/dsfb-gpu-atlas-corpus/out/court_precedents.txt)
(14 691 bytes; one block per precedent) and
[crates/dsfb-gpu-atlas-corpus/out/court_precedents.json](crates/dsfb-gpu-atlas-corpus/out/court_precedents.json)
(21 532 bytes; deterministic JSON array).

The verifier `verify_precedent_set` rejects: missing canonical
subjects, missing alias subjects, kind/reason incompatibilities
(load-bearing — AliasCollapse with a role-drift reason is
rejected, preserving the T.3 semantic_role_hash collapse
invariant), kind/binding incompatibilities, and duplicate ids.

**T.11b does NOT do**: emit CaseFileV2 episode-transcript
bodies (T.11d), implement UnitSemantics / SamplingLaw receipts
(T.11e), implement external provenance export (DSFB-PROV /
OpenLineage / NIST AI RMF / RO-Crate), change `corpus_hash_v1`,
change `registry_hash_v2`, or change D16 / D64 / D128 / D205
GPU behavior. `precedent_hash_v1` is DSFB-native; no in-toto /
SLSA / SPDX / CycloneDX compatibility claim.

27 acceptance tests in
[tests/precedent_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/precedent_invariants.rs)
pin determinism + hash sensitivity, T.4 / T.5 derivations,
T.6 / T.7 / T.8 / T.10 / S1.2 law coverage, passport linkage,
renderer determinism, two negative-direction tests (missing
subject + alias-with-role-drift), severity / id-density /
plan-locked count. Receipts:
[reports/t11b_precedent_summary.txt](reports/t11b_precedent_summary.txt),
[reports/t11b_precedent_verification.txt](reports/t11b_precedent_verification.txt),
[reports/t11b_regression_check.txt](reports/t11b_regression_check.txt).

## Admissibility grammar (T.11c — versioned grammar of admissible episode forms)

T.11c is the **derived law layer** on top of T.11b precedents.
Nine `EpisodeAdmissibilityRule` records + nine
`ConfuserSuppressionRule` records (one per `NegativeWitnessKind`
variant) declare the only witness configurations the court
admits as episodes. Every rule cites at least one T.11b
precedent; the collector invents no new judgments. Plan
framing:

> Detector firings alone are never episodes until the
> bank-governed grammar admits them.

CLI:

```
dsfb-corpus admissibility    [--json] [--out PATH]
dsfb-corpus admissibility-emit [--out-dir DIR]
```

Pinned hashes (T.11c is a SEPARATE court-layer receipt):

```
admissibility_grammar_hash_v1 : ff66706a726d0cddc5f343e21f2ffbd8f81392a1504ff1b2002f8609d14a5ba7

corpus_hash_v1     : 35c276c7…  (unchanged)
registry_hash_v2   : d3cf6300…  (unchanged)
precedent_hash_v1  : 6721f511…  (unchanged)
every passport_hash:            (unchanged — linkage carried by crosswalk, not by extending the passport struct)
```

Future T.11d body chain (plan-locked):

```
corpus_hash_v1
  → registry_hash_v2
  → precedent_hash_v1
  → admissibility_grammar_hash_v1
  → casefile_v2_body_hash
```

Episode-admissibility rules (9): `PrimaryWitnessRequiresPositiveSupport`,
`CleanWindowWitnessCannotAdmitAlone`, `BoundaryWitnessCannotClassifyAlone`,
`RecoveryWitnessCannotOriginateAlone`,
`NegativeWitnessBlocksAdmissionUnlessBankOverride`,
`MinimumPrimaryWitnessEvidence`,
`BankAdmissionTokenIsTheOnlyAdmissionRoute` (SemanticNonBypass),
`GpuOutputIsEvidenceOnly` (SemanticNonBypass),
`UnknownOrDeferredOutcomeIsExplicit` (DeferredUnknown).

Confuser-suppression rules (9): one per
`SmallSampleConfuser` / `SingleWindowSpikeConfuser` /
`PeriodicBoundaryConfuser` / `MissingnessArtifactConfuser` /
`SchemaChangeConfuser` / `UnitScaleChangeConfuser` /
`DeploymentMarkerConfuser` / `ClockSkewConfuser` /
`BatchBoundaryConfuser`, all with `BlockAdmission` effect.

Bulk artifacts (regenerable; two builds byte-identical):
[crates/dsfb-gpu-atlas-corpus/out/admissibility_grammar.txt](crates/dsfb-gpu-atlas-corpus/out/admissibility_grammar.txt)
(6 614 bytes),
[crates/dsfb-gpu-atlas-corpus/out/admissibility_grammar.json](crates/dsfb-gpu-atlas-corpus/out/admissibility_grammar.json)
(10 629 bytes),
[crates/dsfb-gpu-atlas-corpus/out/passport_grammar_crosswalk.txt](crates/dsfb-gpu-atlas-corpus/out/passport_grammar_crosswalk.txt)
(12 760 bytes; 54 rows, one per SEED canonical), and
[crates/dsfb-gpu-atlas-corpus/out/passport_grammar_crosswalk.json](crates/dsfb-gpu-atlas-corpus/out/passport_grammar_crosswalk.json)
(15 035 bytes).

The verifier `verify_grammar_snapshot` rejects: a rule without
precedent links, an `EpisodeAdmission` rule that allows
confuser-only admission (load-bearing Semantic Non-Bypass
invariant), a cited precedent id outside the live T.11b set,
and duplicate rule ids.

**T.11c does NOT do**: emit CaseFileV2 episode-transcript
bodies (T.11d), implement UnitSemantics / SamplingLaw receipts,
implement external provenance export, change any prior hash or
passport hash, change D16 / D64 / D128 / D205 GPU behavior, or
include S1.2 registry-spec linkage. The grammar hash is
DSFB-native; no in-toto / SLSA / SPDX / CycloneDX compatibility
claim.

38 acceptance tests in
[tests/admissibility_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/admissibility_invariants.rs)
pin: determinism + hash sensitivity (4); precedent-linkage
coverage (3); all 10 plan-required rule-presence checks;
renderer / crosswalk determinism (4); 4-direction verifier
including the two plan-required negatives
(`grammar_verifier_rejects_rule_without_precedent_link` and
`grammar_verifier_rejects_episode_admission_rule_that_allows_confuser_only_admission`);
hash-stability cross-checks (`corpus_hash_v1`,
`precedent_hash_v1`, every passport hash including Shewhart pinned
byte-for-byte); and structural / coverage invariants. Receipts:
[reports/t11c_admissibility_summary.txt](reports/t11c_admissibility_summary.txt),
[reports/t11c_admissibility_verification.txt](reports/t11c_admissibility_verification.txt),
[reports/t11c_regression_check.txt](reports/t11c_regression_check.txt).

## Trial transcript body (T.11d — minimal real CaseFileV2 court record)

T.11d is the **boundary-crossing commit**: the court stops being
a stack of frozen identities + abstract receipts and starts
emitting real trial records. One synthetic
`TrialTranscriptV1` fixture (a LatencyRamp episode) carries every
hash-chain anchor + every witness role + the rejected confuser +
the disabled-but-relevant detector + reason-code coverage, and
ships with a brutal 16-direction verifier. Plan framing:

> A transcript that says **why** an episode was admitted,
> **which witnesses** spoke, **which confusers** were rejected,
> **which law** admitted it, **which precedent** supports it,
> and **which hash-bound corpus / registry / grammar** produced
> it.

CLI:

```
dsfb-corpus trial-transcript      [--json] [--out PATH]
dsfb-corpus trial-transcript-emit [--out-dir DIR]
```

Pinned hashes — full chain bound by the transcript:

```
trial_transcript_hash_v1      : 37618a45c1e60da3bb66ddae4161d94ed762287483caf88c21a5db3cff64bbee  (NEW)

corpus_hash_v1                : 35c276c7…   (unchanged)
registry_hash_v2              : d3cf6300…   (unchanged)
precedent_hash_v1             : 6721f511…   (unchanged)
admissibility_grammar_hash_v1 : ff66706a…   (unchanged)
```

Synthetic fixture (LatencyRamp episode):
- motif `LatencyRamp` on `entity_id=7`, windows `[100..131]`.
- Admitted by rule 4 (`PrimaryWitnessRequiresPositiveSupport`).
- Primary: LatencyRamp (id 14). Corroborating: EWMA (2), CUSUM (3).
  Boundary: Shewhart (1), Page-Hinkley (4). Clean-window:
  Robust-Z (6).
- Rejected confuser: `SingleWindowSpikeConfuser` (reason
  `NotFired`).
- Disabled-but-relevant: FFT band-energy (id 12), reason
  `MissingSpectralProjection`.
- Reason-code coverage: 100.00%.

Bulk artifacts (regenerable; two builds byte-identical):
[crates/dsfb-gpu-atlas-corpus/out/trial_transcript_v1.txt](crates/dsfb-gpu-atlas-corpus/out/trial_transcript_v1.txt)
(1 484 bytes) and
[crates/dsfb-gpu-atlas-corpus/out/trial_transcript_v1.json](crates/dsfb-gpu-atlas-corpus/out/trial_transcript_v1.json)
(1 336 bytes). **Rendered text is NOT in the hash** — canonical
bytes only.

The verifier `verify_trial_transcript` rejects 16 plan-locked
failure modes — including the two load-bearing negatives:
`ConfuserOnlyAdmissionAttempted` (Semantic Non-Bypass) and
`MissingAdmissibilityGrammarLink` (no court without grammar
citation).

**T.11d does NOT do**: derive transcripts from GPU-produced
CaseFileV1 episodes (the binding lands after CaseFileV2 body
integration); implement ActivationPlanner / OTel / ChallengeDocket /
Contraindications / PROV export / attestation / Arrow layout
(T.11e+); change any prior hash or passport hash; change
D16 / D64 / D128 / D205 GPU behavior; or claim in-toto / SLSA /
SPDX / CycloneDX compatibility on `trial_transcript_hash_v1`.

46 acceptance tests in
[tests/trial_transcript_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/trial_transcript_invariants.rs)
pin: determinism + clean fixture (2); hash inclusion + sensitivity
(8); cross-citation pinning to live corpus / registry / precedent
/ grammar (4); fixture structural shape (7); renderer determinism
+ no-publication-language (3); the two load-bearing negatives
plus 12 other reject paths (14 total); schema + enum coverage
(5); hash-stability cross-checks (4). Receipts:
[reports/t11d_trial_transcript_summary.txt](reports/t11d_trial_transcript_summary.txt),
[reports/t11d_trial_transcript_verification.txt](reports/t11d_trial_transcript_verification.txt),
[reports/t11d_regression_check.txt](reports/t11d_regression_check.txt).

## Execution Attestation Receipt V1 (T.11e — unsigned, local, DSFB-native)

T.11e ships an **unsigned, local, DSFB-native execution
attestation receipt** that records what the operator just ran:
the build + verification commands, the toolchain (rustc / cargo
/ nvcc), the repository commit + dirty flag, the gate summary,
the workspace-test summary, the materials + subjects each
addressed by their v1 hash, and a plan-locked set of non-claims
that the verifier enforces. **T.11e is NOT a SLSA / in-toto /
SPDX / CycloneDX compliance claim**; the receipt is a court
record for the operator's own reproducibility trail.

Schema: `DSFB-GPU-ATLAS:EXECUTION-ATTESTATION:v1` (domain
separator `DSFB-GPU-ATLAS:EXECUTION-ATTESTATION:v1\0`). New
artifact lives in its own namespace:

```text
receipt_hash_v1 = SHA256(
    "DSFB-GPU-ATLAS:EXECUTION-ATTESTATION:v1\0"
    || schema id || receipt id
    || repo_commit || repo_dirty || dirty_override || branch
    || rustc + cargo + cuda + nvcc + gpu metadata
    || sorted build_commands || sorted verification_commands
    || gate summary || workspace-test summary
    || R12bEpisodeCounts(13, 89, 1917)
    || sorted MaterialDigest[]   // corpus / registry / precedent / grammar
    || sorted SubjectDigest[]    // transcript / grammar / precedent_ledger
    || sorted nvcc_flags
    || sorted rust_flags
    || claimed_slsa_level (always None)
    || claimed_signed_attestation (always false)
    || sorted AttestationNonClaim[]
)
```

CLI subcommands (corpus-only; no registry / GPU dependency
introduced):

```text
dsfb-corpus execution-attestation [--json]
dsfb-corpus execution-attestation-emit --out-dir <DIR>
```

The verifier
[verify_execution_attestation](crates/dsfb-gpu-atlas-corpus/src/execution_attestation.rs)
rejects **20 plan-locked failure modes**:

  1. `ZeroCorpusHash` — corpus_hash_v1 must be set
  2. `ZeroRegistryHash` — registry_hash_v2 must be set
  3. `ZeroPrecedentHash` — precedent_hash_v1 must be set
  4. `ZeroGrammarHash` — admissibility_grammar_hash_v1 must be set
  5. `ZeroTranscriptHash` — trial_transcript_hash_v1 must be set
  6. `EmptyRepoCommit` — repo_commit must be present
  7. `InvalidRepoCommitFormat` — 40-char lowercase hex required
  8. `DirtyRepoWithoutOverride` — dirty trees only with explicit
     acknowledgement (one of two plan-required load-bearing negatives)
  9. `EmptyBuildCommands` — at least one Build command required
 10. `MissingRequiredGateCommand` — Format / Clippy / Scrub /
     DocsFreshness / WorkspaceTest must all be declared
 11. `GateNotClean` — every GateSummary boolean must be true
 12. `WorkspaceTestFailed` — workspace_failed must be zero AND
     workspace_groups must be > 0
 13. `R12bEpisodeCountsDrift` — pinned R.12b counts must equal
     13 / 89 / 1917
 14. `ClaimedSlsaLevelPresent` — the receipt MUST NOT claim a
     SLSA level (plan-required load-bearing negative)
 15. `ClaimedSignedAttestation` — the receipt MUST NOT claim a
     detached signature
 16. `SubjectDigestMissing` — every subject must carry a real digest
 17. `MaterialDigestMissing` — every material must carry a real digest
 18. `ReceiptHashMismatch` — pinned receipt_hash_v1 must equal
     compute_execution_attestation_hash_v1(&receipt)
 19. `NonClaimsIncomplete` — all 7 `AttestationNonClaim` variants
     must be present (`UnsignedLocalReceipt`, `NotSlsaComplianceClaim`,
     `NotInTotoSignedStatement`, `NotReleaseArtifact`,
     `NotThirdPartyVerified`, `NotReproducibleBuildProof`,
     `RecordsObservedEnvironmentOnly`)
 20. `HashChainCrossCheckFailed` — declared subject hashes must
     match the corresponding hash-chain anchors

53 acceptance tests in
[tests/execution_attestation_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/execution_attestation_invariants.rs)
pin: two-build determinism, sensitivity to every hash-chain
anchor + every metadata field, rendering independence, byte-stable
JSON pin, one reject test per verifier kind, positive admission
for a clean fixture + a dirty repo with override, and four pin-
stability cross-checks that the receipt's declared anchors agree
with the live computations of `corpus_hash_v1`,
`court_precedents_hash()`,
`compute_admissibility_grammar_hash_v1`, and
`compute_trial_transcript_hash_v1`.

The bulk-emit CLI writes
[crates/dsfb-gpu-atlas-corpus/out/execution_attestation_v1.txt](crates/dsfb-gpu-atlas-corpus/out/execution_attestation_v1.txt)
and
[crates/dsfb-gpu-atlas-corpus/out/execution_attestation_v1.json](crates/dsfb-gpu-atlas-corpus/out/execution_attestation_v1.json),
querying git / rustc / cargo / nvcc for live environment metadata.
Two invocations against the same environment produce byte-stable
artifacts.

**T.11e does NOT do**: sign the receipt; emit a SLSA Provenance
v1.0 / in-toto v1 / DSSE envelope / CycloneDX BOM / SPDX SBOM;
claim third-party verification; claim
reproducible-builds.org bit-for-bit binary reproducibility; hash
repo source-bytes (the corpus / registry / court materials are
already addressed via their v1 anchors); implement
ChallengeDocket (T.11f), DetectorContraindicationReceipt
(T.11g), or the activation planner (S1.3); change `corpus_hash_v1`,
`registry_hash_v2`, `precedent_hash_v1`,
`admissibility_grammar_hash_v1`, `trial_transcript_hash_v1`, any
DetectorPassport hash, or D16 / D64 / D128 / D205 GPU behavior.

Receipts:
[reports/t11e_execution_attestation_summary.txt](reports/t11e_execution_attestation_summary.txt),
[reports/t11e_execution_attestation_verification.txt](reports/t11e_execution_attestation_verification.txt),
[reports/t11e_regression_check.txt](reports/t11e_regression_check.txt).

## Challenge Docket V1 (T.11f — the court's adversarial self-audit layer)

T.11f ships the court's **appeals layer**. The docket records
objections against detector identities (T.11a), precedent
judgments (T.11b), grammar rules (T.11c), trial transcripts
(T.11d), execution receipts (T.11e), and corpus / registry
globals. It is an **adversarial overlay** — sustaining a
challenge requires a **separate later commit** that mutates the
canonical artifact. This separation is what makes the docket a
court instead of an issue tracker.

Schema: `DSFB-GPU-ATLAS:CHALLENGE-DOCKET:v1` (domain separator
`DSFB-GPU-ATLAS:CHALLENGE-DOCKET:v1\0`). The new
`challenge_docket_hash_v1 =
dde4ecb4de491f93fe671b77d6ffd8567199f6d1a3f5cca51ea4facb21c10415`
lives in its own namespace and is **not folded into**
`corpus_hash_v1`; future CaseFileV2 body receipts may cross-cite
it.

CLI subcommands (corpus-only; no registry / GPU dependency):

```text
dsfb-corpus challenges [--json] [--out PATH]
dsfb-corpus challenges-emit --out-dir <DIR>
```

The 11 plan-locked `ChallengeKind` variants (three additions
beyond the user-listed eight): `OverbroadAlias`,
`MissingConfuser`, `WrongWitnessRole`, `BadSource`,
`FormulaMismatch`, `DomainMisapplied`, `UnimplementedButClaimed`,
`RuntimeTooHigh`, `HashBindingMismatch`,
`MissingNegativeWitness`, `EvidenceLevelOverclaimed`.

The 5 plan-locked `ChallengeStatus` variants (plan-required
addition: `Superseded`): `Open`, `Sustained`, `Overruled`,
`Deferred`, `Superseded`. The Superseded state matters — it lets
a later corpus / precedent / grammar change replace an old
challenge without deleting the audit history.

The verifier
[verify_challenge_docket](crates/dsfb-gpu-atlas-corpus/src/challenge_docket.rs)
rejects **17 plan-locked failure modes**:

  1. `ChallengeAgainstMissingDetector`
  2. `ChallengeAgainstMissingPrecedent`
  3. `ChallengeAgainstMissingGrammarRule`
  4. `EmptyClaim`
  5. `EmptyChallenger`
  6. `DuplicateChallengeId`
  7. `SustainedWithoutResolution` (plan-required
     load-bearing negative #1)
  8. `OverruledWithoutCourtResponse`
  9. `DeferredWithoutDeferralReason`
 10. `SupersededWithoutCommitReference`
 11. `RuntimeTooHighWithoutRuntimeEvidence`
 12. `FormulaMismatchWithoutFormulaHashReference`
 13. `BadSourceWithoutSourceEvidence` (plan-required
     load-bearing negative #2)
 14. `WrongWitnessRoleWithoutSemanticRoleHashReference`
 15. `UnimplementedButClaimedAgainstHonestLBand`
 16. `OpenCriticalWithoutDeferredGate`
 17. `StatusResponseInconsistent`

62 acceptance tests in
[tests/challenge_docket_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/challenge_docket_invariants.rs)
pin: schema constants (5); hash determinism + sensitivity to
every field (12); positive admission of clean seed + open-critical
with deferred gate (3); one reject test per verifier kind plus
two plan-required load-bearing negatives (17); enum wire-name
stability (4); rendering determinism + content (8); and
conservative-seed shape invariants (13: no Open / no Sustained;
every Overruled carries `OverruledReason` with non-empty text;
every Deferred carries `DeferredToGate` with non-empty text;
every entry created in `T.11f`; `RuntimeTooHigh` seed entries
carry `RuntimeCostUs` evidence; `BadSource` seed entries carry
`SourceRef`/`SourceHash` evidence; `AffectedHashSet::default()`
is all-false; struct-update produces non-empty set).

**Conservative seed** (10 entries, plan-locked: 0 Open, 0
Sustained, 6 Overruled, 4 Deferred, 0 Superseded). Examples:
`RuntimeTooHigh` against the D128/D205 wide-digest baseline
(Deferred to R.10b post-R.13); `UnimplementedButClaimed` guard
against L7/L8 (Overruled — T.7 verifier already rejects);
`MissingConfuser` against spectral coverage in the LatencyRamp
transcript (Deferred to S1.3); `HashBindingMismatch` against
`corpus_hash_v1` excluding rendered report text (Overruled — by
design); `BadSource` self-audit against the 5 L6 GPU IDs citing
dsfb-gpu-debug-core (Overruled — that surface IS the
authoritative L6 provenance).

The bulk-emit CLI writes
[crates/dsfb-gpu-atlas-corpus/out/challenge_docket_v1.txt](crates/dsfb-gpu-atlas-corpus/out/challenge_docket_v1.txt)
and
[crates/dsfb-gpu-atlas-corpus/out/challenge_docket_v1.json](crates/dsfb-gpu-atlas-corpus/out/challenge_docket_v1.json).
Two invocations produce byte-identical files.

**T.11f does NOT do**: mutate `corpus_hash_v1`,
`registry_hash_v2`, `precedent_hash_v1`,
`admissibility_grammar_hash_v1`, `trial_transcript_hash_v1`, any
DetectorPassport hash, or any T.11e attestation receipt; resolve
a Sustained challenge (separate later commit required); implement
DetectorContraindicationReceipt (T.11g), CoverageHoleReport
(T.11h), NullVerdict / CorpusAmendment / RetirementCase (T.11i),
the ActivationPlanner (S1.3), or OTel binding receipts (S1.3a);
change D16 / D64 / D128 / D205 GPU behavior; or claim W3C PROV /
SLSA / in-toto / SPDX / CycloneDX compatibility on
`challenge_docket_hash_v1`.

Receipts:
[reports/t11f_challenge_docket_summary.txt](reports/t11f_challenge_docket_summary.txt),
[reports/t11f_challenge_docket_verification.txt](reports/t11f_challenge_docket_verification.txt),
[reports/t11f_regression_check.txt](reports/t11f_regression_check.txt).

## Detector Contraindication Receipts V1 (T.11g — the court's safety-label layer)

T.11g ships the court's **datasheet / model-card / safety-label
layer**. A detector is not fully admissible until the court knows
**when it should not be trusted**. Each canonical SEED record
gets one `DetectorContraindicationReceiptV1` answering nine
plan-locked questions:

  1. `works_best_when` — categorical conditions where the
     detector is most accurate.
  2. `fails_when` — categorical failure modes.
  3. `known_confusers` — cross-links to T.6
     `NegativeWitnessKind` variants.
  4. `required_sampling_law` — `RegularFixedRate` /
     `OrderedNonRegular` / `UnorderedRowSet` / `GraphAdjacency`,
     with min observations + regularity tolerance.
  5. `required_units` — `PhysicalUnitsRequired` /
     `DimensionlessRatio` / `CountOrCardinality` /
     `CategoricalLabels` / `BooleanState` / `None`.
  6. `minimum_support` — baseline / active observations /
     distinct entities.
  7. `do_not_use_for` — categorical disqualifiers
     (`StreamingWithoutReplay`, `ProbabilisticDecisionMaking`,
     `BlackBoxRetrievalAugmentation`, etc.).
  8. `closest_aliases` — legitimate aliases per T.4 dedup court.
  9. `closest_non_aliases` — surface-similar but semantically
     distinct detectors.

Plus an **adversarial-twin layer** (`DetectorTwinRelation`):
`SameFormulaDifferentRole`, `SameRoleDifferentFormula`,
`SameFamilyDifferentSamplingLaw`,
`AliasLikeButSemanticallyDistinct`, `ConfuserOfPrimary`.

Schema: `DSFB-GPU-ATLAS:DETECTOR-CONTRAINDICATION:v1` (domain
separator `DSFB-GPU-ATLAS:DETECTOR-CONTRAINDICATION:v1\0`). The
new `detector_contraindication_hash_v1 =
1b899f5d5d6bcdd68d75cfbc19d6ce41ae256743f0a89126afb556d9dd001458`
lives in its own namespace and is **not folded into**
`corpus_hash_v1`. The passport ↔ contraindication binding lives
in a separate **crosswalk artifact**, NOT a passport field —
the same pattern T.11c used for grammar linkage — so every
`DetectorPassport` hash stays byte-stable.

CLI subcommands (corpus-only; no registry / GPU dependency):

```text
dsfb-corpus contraindication [--json] [--out PATH]
dsfb-corpus contraindications-emit --out-dir <DIR>
```

The receipt is built **deterministically from corpus SEED
metadata** (PrimitiveFamily / WitnessRole / InputRequirementSet
/ ImplementationLevel) rather than hand-curated, so two builds
against the same SEED produce byte-identical hashes and the
verifier rules are satisfied by construction. Hand-curated
overrides land in T.11g.1+ if and when needed.

The verifier
[verify_contraindications](crates/dsfb-gpu-atlas-corpus/src/contraindication.rs)
rejects **11 plan-locked failure modes**:

  1. `PrimaryWithoutKnownConfuser` (plan-required negative #1)
  2. `LBandL4PlusWithoutContraindications`
  3. `LBandL5OrL6WithoutRequiredSamplingLaw`
  4. `UnitSensitiveWithoutUnitSemantics` (plan-required
     negative #3; covers `InputRequirementSet::UNITS` AND
     `PrimitiveFamily::{Spectral, Wavelet}`)
  5. `SpectralWithoutSamplingLaw` (plan-required negative #2)
  6. `TimeSeriesWithoutOrderedTimeDeclaration`
  7. `DistributionWithoutReferenceBaseline`
  8. `ClosestAliasMissing` / `ClosestNonAliasMissing`
  9. `ContraindicationWithoutCrossReference`
 10. `ActiveWithoutDoNotUseFor`
 11. `AdversarialTwinMissing` / `AdversarialTwinSelfReference`
     / `UnknownDetector` / `DuplicateReceipt`

57 acceptance tests in
[tests/contraindication_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/contraindication_invariants.rs)
pin: schema constants (5); hash determinism + sensitivity to
every field (13); positive admission of clean seed (1); one
reject test per verifier kind plus three plan-required
load-bearing negatives (14); seed-shape invariants (5); enum
wire-name stability (7); rendering determinism + content (6);
passport ↔ contraindication crosswalk (5).

**Scope-discipline lock** (plan iteration 6, audit item 8):
T.11g absorbs UnitSemantics and SamplingLaw at the
**receipt-level enum + verifier-rule layer ONLY**. It does NOT
ship a unit-conversion engine, a resampler, a numeric coercion
engine, or ingestion-format changes. Operational use (e.g.
disabling a spectral detector because `sample_rate_hz` is
missing) is the S1.3 ActivationPlanner's job; T.11g declares
the requirement and the verifier enforces the declaration.

**T.11g does NOT do**: mutate `corpus_hash_v1`,
`registry_hash_v2`, `precedent_hash_v1`,
`admissibility_grammar_hash_v1`, `trial_transcript_hash_v1`,
`challenge_docket_hash_v1`, any DetectorPassport hash, or any
T.11e attestation receipt; resolve any T.11f ChallengeDocket
entry; implement S1.3 ActivationPlanner / S1.3a OTel binding /
T.11h CoverageHoleReport / T.11i NullVerdict /
CorpusAmendment / RetirementCase; land a unit-conversion engine
or resampler; change D16 / D64 / D128 / D205 GPU behavior; or
claim datasheet-for-datasets / model-card / NIST AI RMF / SLSA
/ in-toto / SPDX / CycloneDX compatibility on
`detector_contraindication_hash_v1`.

Receipts:
[reports/t11g_contraindication_summary.txt](reports/t11g_contraindication_summary.txt),
[reports/t11g_contraindication_verification.txt](reports/t11g_contraindication_verification.txt),
[reports/t11g_regression_check.txt](reports/t11g_regression_check.txt).

## Coverage Hole Report V1 (T.11h — the court's audit-only honesty layer)

T.11h ships the court's **audit-only honesty layer**. The
`CoverageHoleReportV1` walks every sealed T.1–T.11g surface and
surfaces structural gaps deterministically. It does not repair
anything; it does not mutate any upstream hash; it does not
issue verdicts. It tells the operator (and a future reviewer)
where the court is intentionally thin and where the next round
of corpus / activation / OTel / contraindication work should land.

Seven plan-locked buckets:

  1. **DetectorCoverage** — per detector. Missing contraindication
     receipt claim; missing precedent support; missing genealogy
     edge; missing GPU family-kernel mapping where the L-band claim
     would expect one.
  2. **WitnessLawCoverage** — per family. Families with a Primary
     witness but no Confuser; families that admit Boundary episodes
     but have no Boundary witness declared.
  3. **ImplementationCoverage** — per family / per detector.
     Families whose entire roster sits at L0/L1/L2 and would
     benefit from GPU mapping; L5+ detectors without
     GpuFamilyKernel mapping.
  4. **SemanticsCoverage** — per detector receipt. Ordered-time
     detectors without `required_sampling_law`; unit-sensitive
     detectors without `required_units` at L < 5. Bucket is
     **honest-empty in the current SEED** because T.11g's
     deterministic derivation populates these fields for every
     receipt by construction; the bucket exists in the schema so
     future hand-curated overrides have a place to land.
  5. **JurisprudenceCoverage** — per detector. No precedent binding
     citing the detector; no genealogy edge (and not `is_origin`).
  6. **SourceProvenanceCoverage** — per detector. All references
     pre-2000 with no modern engineering validation; at least one
     post-2000 reference without DOI where the venue would normally
     carry one.
  7. **ReasonCodeCoverage** — per surface. The headline metric.
     Counts records with categorical reason codes on the
     corpus dedup court, T.11b precedents, T.11c grammar rules,
     T.11d transcript entries, T.11e attestation receipts, T.11f
     challenge docket entries, and T.11g contraindication receipts.
     **100% by construction on every surface in the current
     court.**

Schema: `DSFB-GPU-ATLAS:COVERAGE-HOLES:v1` (domain separator
`DSFB-GPU-ATLAS:COVERAGE-HOLES:v1\0`). New
`coverage_hole_hash_v1 =
671e2164001a6cb16024f7ece8f6217dc0fad7d251521f8dbd2f3da023f96d32`
binds the canonical-byte form. The receipt does NOT mutate
`corpus_hash_v1`, `registry_hash_v2`, `precedent_hash_v1`,
`admissibility_grammar_hash_v1`, `trial_transcript_hash_v1`,
`execution_attestation_receipt_hash_v1`, `challenge_docket_hash_v1`,
`detector_contraindication_hash_v1`, or any DetectorPassport
hash. Two builds against the same sealed surfaces produce
byte-identical artifacts.

CLI:

```bash
dsfb-corpus coverage-holes        [--json] [--out PATH]
dsfb-corpus coverage-holes-emit   --out-dir <DIR>
```

The receipt is **derived deterministically** from the SEED + every
sealed T.11 surface — no hand-curated entries — so two builds
produce byte-identical artifacts and the report evolves with the
court rather than drifting.

Verifier
([verify_coverage_hole_report](crates/dsfb-gpu-atlas-corpus/src/coverage_holes.rs))
emits 13 reject kinds:

  1. `CriticalHoleWithoutResolutionGate` (load-bearing negative #1)
  2. `MissingContraindicationClaimWhenDetectorLacksCrosswalk`
     (load-bearing negative #2)
  3. `ReasonCoverageRowWithImpossibleDenominator`
     (load-bearing negative #3)
  4. `DuplicateHoleId`
  5. `HoleSubjectDetectorMissing`
  6. `SubjectFamilyMissing`
  7. `EvidenceSeedRecordMissing`
  8. `EvidencePrecedentMissing`
  9. `EvidenceChallengeMissing`
 10. `ReasonCoverageRowCoveredExceedsRequired`
 11. `MissingReasonCoverageSurface`
 12. `UnknownReasonCoverageSurface`
 13. `EmptyHoleRosterButReasonCoverageNonZero`

The acceptance suite (52 plan-required tests) lives at
[tests/coverage_hole_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/coverage_hole_invariants.rs):
schema constants (3), hash determinism (8), sensitivity (8),
positive admission (2), verifier reject coverage (10),
wire-name stability (7), rendering (8), seed-shape (6).

**T.11h scope-discipline lock** (plan iteration 6, audit item
8 — guards against scope creep): T.11h is audit-only and DOES
NOT:

- repair any hole (resolution is downstream;
  `CoverageHoleResolutionGate::FutureCorpusExpansion` / etc.
  is a marker, not an action);
- mutate `corpus_hash_v1`, `registry_hash_v2`, or any
  T.11a-T.11g hash anchor;
- implement S1.3 ActivationPlanner, S1.3a OTel binding, or
  T.11i NullVerdict / CorpusAmendment / RetirementCase;
- change D16 / D64 / D128 / D205 GPU behaviour;
- claim datasheet-for-datasets / model-card / NIST AI RMF /
  SLSA / in-toto / SPDX / CycloneDX compatibility on
  `coverage_hole_hash_v1`.

R.12b D64 baseline NOT rebaselined; pinned R.12b episode
counts (13/89/1917 for canonical/mid/full at K=1) byte-stable.

Receipts:
[reports/t11h_coverage_holes_summary.txt](reports/t11h_coverage_holes_summary.txt),
[reports/t11h_coverage_holes_verification.txt](reports/t11h_coverage_holes_verification.txt),
[reports/t11h_regression_check.txt](reports/t11h_regression_check.txt).

## Activation Plan V1 (S1.3a — the first deterministic court decision)

S1.3a is the **first real legal bridge** from the sealed T.11
court surfaces to detector activation. It is plan-locked as:

> **S1.3 makes detector activation a deterministic court
> decision, not a heuristic filter.**

A normal activation planner says "this detector is applicable."
S1.3a says "this detector is admissible under the current
evidence contract, and here is the full reason route."

S1.3a is the surface that turns T.11h coverage holes, T.11g
contraindications, and T.11f challenges from honest
documentation into real activation consequences:

> A coverage hole that never affects activation, admission, or
> audit is just documentation. S1.3a is what makes T.11h
> operational.

**Inputs (the planner consumes the sealed court stack)**:

```text
DetectorPassport             T.11a
CourtPrecedent               T.11b   (consulted)
AdmissibilityGrammar         T.11c   (referenced)
TrialTranscript              T.11d   (reference-only)
ExecutionAttestation         T.11e   (anchor)
ChallengeDocket              T.11f
ContraindicationReceipt      T.11g
CoverageHoleReport           T.11h
corpus_hash_v1               T.10
registry_hash_v2             S1.2    (KNOWN_S12_REGISTRY_HASH_V2 const)
```

**Per-detector decision shape**:

```rust
DetectorActivationDecision {
    canonical_id,
    display_name,
    activation_status: Enabled | Disabled | WarnOnly | Deferred,
    enabled_reason  | disabled_reason,    // categorical enums
    blocking_receipt_hashes,              // 32-byte court hashes
    warning_receipt_hashes,
    cited_challenge_ids,
    cited_contraindication_ids,
    cited_coverage_hole_ids,
    cited_passport_hash,
}
```

**Plan wrapper** records the five upstream hash anchors
(`corpus_hash_v1`, `registry_hash_v2`,
`challenge_docket_hash_v1`,
`detector_contraindication_hash_v1`, `coverage_hole_hash_v1`),
the status counts, a reason histogram, and
`activation_plan_hash_v1` under domain
`DSFB-GPU-ATLAS:ACTIVATION-PLAN:v1\0`.

**Current seed plan** (54 SEED records × pinned
`KNOWN_S12_REGISTRY_HASH_V2`):

- Enabled    : 0
- Disabled   : 49 (DisabledByWeakLBand — passport L-band is L0/L1/L2)
- WarnOnly   : 5  (the GPU-implemented surface; EnabledByRoleSeededGpuSurface
                   with contraindication warnings attached)
- Deferred   : 0

The all-Disabled-by-weak-L-band outcome is **honest, not
defective**. The corpus currently sits at L0–L2 for 49 of 54
records (literature primitives without implementations) and L6
for 5 (the dsfb-gpu-debug-core bank surface). S1.3a admits the
five L6 detectors via `EnabledByRoleSeededGpuSurface` and
correctly blocks the 49 literature-only records until host
implementations land. Future work (S1.3 budget planner, T.6.x
L-band promotions backed by measured benchmark receipts) will
shift detectors from Disabled-by-weak-L-band into Enabled as
implementations and evidence accumulate.

**Disable-reason enum (11 variants, plan-locked)**:

- `DisabledByCoverageHole`
- `DisabledByContraindication` (reserved for S1.3 once evidence
  contracts wire in; not emitted at S1.3a)
- `DisabledByUnresolvedChallenge`
- `DisabledByWeakLBand`
- `DisabledByMissingSamplingLaw`
- `DisabledByMissingUnitSemantics`
- `DisabledByMissingConfuser`
- `DisabledByThinPrecedentSupport`
- `DisabledByDomainMismatch`
- `DisabledByBudgetDeferred`
- `DisabledByUnimplementedSurface`

**Enable-reason enum (8 variants, priority order)**:

`EnabledByRoleSeededGpuSurface` > `EnabledAsPrimaryWitness` >
`EnabledAsBoundaryWitness` > `EnabledAsConfuserWitness` >
`EnabledByNoBlockingCoverageHole` >
`EnabledByPassportComplete` >
`EnabledByContraindicationSatisfied` >
`EnabledByChallengeClear`.

**Verifier — 14 reject kinds** (each pinned by an acceptance
test; four plan-required load-bearing negatives):

  1. `activation_plan_rejects_enabled_detector_with_blocking_coverage_hole`
     (load-bearing #1)
  2. `activation_plan_rejects_enabled_detector_with_blocking_contraindication`
     (load-bearing #2)
  3. `activation_plan_rejects_disabled_detector_without_reason`
     (load-bearing #3)
  4. `activation_plan_hash_changes_when_one_decision_changes`
     (load-bearing #4)
  5. `EnabledWithoutEnabledReason`
  6. `EnabledDetectorWithBlockingChallenge`
  7. `DisabledDecisionWithoutBlockingHash`
  8. `DuplicateDecisionForCanonicalId`
  9. `DecisionForUnknownDetector`
 10. `DecisionCitesUnknownChallenge`
 11. `DecisionCitesUnknownCoverageHole`
 12. `DecisionCitesUnknownContraindication`
 13. `PlanMissingCorpusHash`
 14. `PlanMissingRegistryHash`
 15. `DecisionPassportHashMismatch`

CLI:

```bash
dsfb-corpus activation-plan        [--json] [--out PATH] [--registry-hash HEX]
dsfb-corpus activation-plan-emit   --out-dir <DIR>       [--registry-hash HEX]
```

`--registry-hash HEX` accepts a 64-character hex string; default
is the `KNOWN_S12_REGISTRY_HASH_V2` constant pinned to the live
S1.2 seal at commit `8ccd522`. Future S1.2.x re-emits refresh the
constant and bump the test that asserts well-formed bytes.

The acceptance suite (37 tests) lives at
[tests/activation_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/activation_invariants.rs):
schema constants (3), roster shape (7), hash determinism +
sensitivity (4), rendering (5), verifier positive (1), verifier
reject coverage (10), wire-name stability (3, covering 23 enum
variants explicitly).

**S1.3a scope-discipline lock (plan-locked)**: this commit
ships **schema + reason-coded planner**. It does NOT:

- run GPU kernels;
- execute 2,000 detectors;
- change `corpus_hash_v1`, `registry_hash_v2`, or any
  T.11a–T.11h hash;
- mutate any DetectorPassport hash;
- fix any coverage hole / resolve any challenge / retire any
  detector;
- claim empirical usefulness;
- implement budget optimisation, redundancy suppression, or
  T.8 ledger consumption (S1.3 proper);
- bind OpenTelemetry semantics (S1.3a OTel binding receipt is
  a separate later commit);
- claim datasheet / model-card / NIST AI RMF / SLSA / in-toto /
  SPDX / CycloneDX compatibility on
  `activation_plan_hash_v1`.

R.12b D64 baseline NOT rebaselined; pinned R.12b episode counts
(13/89/1917 for canonical/mid/full at K=1) byte-stable.

Receipts:
[reports/s1_3a_activation_plan_summary.txt](reports/s1_3a_activation_plan_summary.txt),
[reports/s1_3a_activation_plan_verification.txt](reports/s1_3a_activation_plan_verification.txt),
[reports/s1_3a_regression_check.txt](reports/s1_3a_regression_check.txt).

## Activation Plan Audit V1 (S1.3b — the explanation + diff court)

S1.3a says **what** each decision is. S1.3b says **why**, with
full citation back to the court artifact(s) that drove it, and
can diff two activation plans deterministically.

Plan-locked first thesis:

> **DSFB-GPU-Atlas does not merely choose detectors; it issues
> replayable activation decisions under a hash-bound court
> record.**

S1.3b is the explanation + diff court built on top of S1.3a.
Two own-namespace hashes:

- `activation_decision_transcript_hash_v1` under
  `DSFB-GPU-ATLAS:ACTIVATION-TRANSCRIPT:v1\0`
- `activation_diff_hash_v1` under
  `DSFB-GPU-ATLAS:ACTIVATION-DIFF:v1\0`

The five operator questions S1.3b answers deterministically:

  1. **Why was detector N WarnOnly?**
  2. **Why was detector N Disabled?**
  3. **What would need to change for detector N to become
     Enabled?**
  4. **What changed between two activation plans?**
  5. **Which court artifact caused the block?**

**Transcript schema** (per detector):

```rust
ActivationDecisionTranscript {
    canonical_id,
    display_name,
    activation_status,
    final_reason,                              // wire-named EnabledReason or DisabledReason
    contributing_facts: Vec<ContributingFact>, // sorted, deterministic
    blocking_chain: Vec<BlockingLink>,         // root cause first
    counterfactual_path_to_enabled: Vec<CounterfactualStep>,
    transcript_hash_v1,
}

ContributingFact {
    artifact_kind: Passport | CoverageHole | Contraindication
                   | Challenge | LBand | RegistryHash | CorpusHash,
    artifact_id,
    artifact_hash,                              // 32-byte citation
    role: Blocking | Warning | Supporting | Informational,
    reason_code,                                // wire name
    operator_message,                           // one short sentence
}
```

Sample explain output for detector 14 (Latency ramp, the GPU
bank-surface L6 record):

```
canonical_id              : 14
display_name              : Latency ramp
activation_status         : WarnOnly
final_reason              : EnabledByRoleSeededGpuSurface
transcript_hash_v1        : 68240f6d...

Contributing facts (sorted by kind / id / role)
  [Passport      ] id=14   role=Supporting    reason=PassportPresent
  [Passport      ] id=14   role=Supporting    reason=GpuSurfaceRoleSeeded
  [CoverageHole  ] id=163  role=Warning       reason=FamilyMissingConfuserCoverage
  [Contraindication] id=14 role=Warning       reason=ContraindicationReceiptPresent
  [Challenge     ] id=10   role=Informational reason=Overruled
  [LBand         ] id=6    role=Supporting    reason=L6_CpuGpuByteEquivalent
  [RegistryHash  ] id=0    role=Informational reason=AnchorBound
  [CorpusHash    ] id=0    role=Informational reason=AnchorBound
```

**Diff schema** (two-plan, structural, categorical):

```rust
ActivationDiffV1 {
    old_activation_plan_hash_v1,
    new_activation_plan_hash_v1,
    corpus_hash_v1,                       // plan-locked: same on both plans
    rows: Vec<ActivationDiffRow>,
    decisions_added, decisions_removed,
    decisions_status_changed,
    decisions_reason_changed,
    decisions_citation_changed,
    activation_diff_hash_v1,
}
```

Five categorical `DiffChangeKind` variants:
`DecisionAdded`, `DecisionRemoved`, `StatusChanged`,
`ReasonChanged`, `CitationChanged`.

Plan-locked rule: **two plans with different `corpus_hash_v1`
cannot be diffed**. Diffing across corpus generations is
meaningless because the evidence base changed under both feet.
The verifier rejects such a diff via
`DiffRejectsMismatchedCorpusHash`.

**Verifier reject kinds**:

Transcript (4):

  1. `DisabledTranscriptWithoutBlockingFact` (load-bearing #3)
  2. `UnknownDetector`
  3. `FinalReasonMissing`
  4. `TranscriptHashMissing`

Diff (2):

  1. `DiffRejectsMismatchedCorpusHash` (load-bearing #2)
  2. `DiffHashMissing`

Plus a build-time invariant pinned by acceptance test:

  - `transcript_hash_changes_when_one_contributing_fact_changes`
    (load-bearing #4)
  - `explain_rejects_unknown_canonical_id` (load-bearing #1 —
    `build_transcript_for` returns `None`)

CLI:

```bash
dsfb-corpus activation-plan-explain <canonical_id> [--json] [--out PATH]
dsfb-corpus activation-plan-audit-emit              [--out-dir DIR]
dsfb-corpus activation-plan-diff --old OLD.json --new NEW.json [--json] [--out PATH]
```

The `activation-plan-diff` CLI in this commit demonstrates the
schema by diffing the live plan against itself (empty diff
with zero counts). External-file plan ingestion is honestly
deferred to S1.3c+ alongside TaskManifest parsing; the schema +
in-memory diff machinery + verifier are sealed here.

Bulk artifacts:

```
crates/dsfb-gpu-atlas-corpus/out/activation_plan_audit_v1.{txt,json}
```

The acceptance suite (34 tests) lives at
[tests/activation_audit_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/activation_audit_invariants.rs):
schema constants (2), transcript shape (7), hash determinism +
sensitivity (4), transcript verifier (3 incl. load-bearing
negatives), rendering (4), audit wrapper (3), diff (6 incl.
load-bearing negatives), upstream hash preservation (1), wire-
name stability (2).

**S1.3b scope-discipline lock (plan-locked)**: this commit
ships **explanation + diff machinery only**. It does NOT:

- consume TaskManifest / DatasetManifest (S1.3c);
- prune by budget or suppress redundancy (S1.3d);
- emit KernelPlanV1 (S1.3e);
- integrate into CaseFileV2 (S1.3f);
- consume T.8 usefulness ledger;
- mutate `corpus_hash_v1`, `registry_hash_v2`, any T.11
  hash anchor, or `activation_plan_hash_v1`;
- execute detectors or change GPU behaviour;
- claim datasheet / model-card / NIST AI RMF / SLSA /
  in-toto / SPDX / CycloneDX compatibility on the new hashes.

R.12b D64 baseline NOT rebaselined; pinned R.12b episode counts
(13/89/1917 for canonical/mid/full at K=1) byte-stable.

Receipts:
[reports/s1_3b_activation_audit_summary.txt](reports/s1_3b_activation_audit_summary.txt),
[reports/s1_3b_activation_audit_verification.txt](reports/s1_3b_activation_audit_verification.txt),
[reports/s1_3b_regression_check.txt](reports/s1_3b_regression_check.txt).

## Activation Context V1 (S1.3c — context-bound activation)

S1.3a issues decisions. S1.3b explains them. S1.3c binds them
to a declared task, domain, schema, units, sampling law, and
artifact fixedness contract.

Plan-locked first thesis:

> **S1.3c makes activation context-bound: detector decisions
> are issued against a declared task, domain, schema, units,
> sampling law, and artifact fixedness contract.**

Before S1.3c the court said "given the present corpus / legal
surfaces, here is the activation state." After S1.3c it can
say "given THIS specific task and dataset, here is the
activation state."

**Three new schema types**:

```rust
TaskManifestV1 {
    task_id,
    task_kind,                  // DebugTraceResidualCourt, TimeSeriesAnomalyCourt, ...
    domain_tags,                // bitset
    target_episode_kinds,       // Primary | Boundary | Recovery | Drift | Spike | ...
    required_witness_roles,     // bitset
    forbidden_witness_roles,    // bitset
    strictness_level,           // Phase0 / Phase5_6 / Phase7 / Phase8
    task_manifest_hash_v1,
}

DatasetManifestV1 {
    dataset_id,
    artifact_fixedness,         // FixedBytes / FixedEventCatalog / StreamingAppendOnly / ...
    schema_hash,
    column_kinds,               // bitset
    unit_semantics,             // LatencyMillisecondsAndErrorIndicator / DimensionlessRatios / ...
    sampling_law,               // OrderedRegularWindows / OrderedNonRegular / UnorderedRowSet / ...
    missingness_profile,
    timestamp_law,              // MonotonicStrict / MonotonicNonDecreasing / Unordered / NoneDeclared
    source_artifact_hash,
    dataset_manifest_hash_v1,
}

ActivationContextV1 {
    corpus_hash_v1,
    registry_hash_v2,
    task_manifest_hash_v1,
    dataset_manifest_hash_v1,
    coverage_hole_hash_v1,
    detector_contraindication_hash_v1,
    activation_context_hash_v1,
}
```

**Three new own-namespace hashes** (plan-locked: NOT folded
upstream):

- `task_manifest_hash_v1` under
  `DSFB-GPU-ATLAS:TASK-MANIFEST:v1\0`
- `dataset_manifest_hash_v1` under
  `DSFB-GPU-ATLAS:DATASET-MANIFEST:v1\0`
- `activation_context_hash_v1` under
  `DSFB-GPU-ATLAS:ACTIVATION-CONTEXT:v1\0`

The seed context binds the DSFB-GPU-Debug fixture:

```
task_kind          = DebugTraceResidualCourt
domain_tags        = Debug | Telemetry | TimeSeries
artifact_fixedness = FixedEventCatalog
unit_semantics     = LatencyMillisecondsAndErrorIndicator
sampling_law       = OrderedRegularWindows
timestamp_law      = MonotonicStrict
```

Sample emitted activation-context bytes:

```
corpus_hash_v1                        : 35c276c7...
registry_hash_v2                      : d3cf6300...
task_manifest_hash_v1                 : 88a33338...
dataset_manifest_hash_v1              : 3864db34...
coverage_hole_hash_v1                 : 671e2164...
detector_contraindication_hash_v1     : 1b899f5d...
activation_context_hash_v1            : 4948bf45...
```

**Verifier — 12 reject kinds (11 plan-locked rules + 1
structural)**:

  1. *FixedArtifactMissingSourceHash (load-bearing #1)
  2. *TimeSeriesTaskWithoutTimestampLaw (load-bearing #2)
  3. *UnitSensitiveDetectorWithoutUnits (load-bearing #3)
  4. *activation_context_hash sensitivity to sampling-law
     change (load-bearing #4)
  5.  ContextMissingCorpusHash
  6.  ContextMissingRegistryHash
  7.  TaskManifestMissingTaskId
  8.  DatasetManifestMissingDatasetId
  9.  TaskManifestDomainTagsEmpty
 10.  DatasetManifestSchemaHashZero
 11.  SpectralDetectorWithoutSamplingLaw
 12.  DecisionCitesContextFactNotPresent (placeholder for
      S1.3d's per-decision crosscheck)
 13.  ManifestHashMismatch (structural integrity)

CLI:

```bash
dsfb-corpus task-manifest           [--json] [--out PATH]
dsfb-corpus dataset-manifest        [--json] [--out PATH]
dsfb-corpus activation-context      [--json] [--out PATH]
dsfb-corpus activation-context-emit [--out-dir DIR]
```

The acceptance suite (35 tests) lives at
[tests/activation_context_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/activation_context_invariants.rs):
schema constants (2), seed shape (4), hash determinism +
sensitivity (7), verifier reject coverage (10), rendering (6),
wire-name stability (4), upstream-anchor preservation (1),
spectral-activation crosscheck (1).

Bulk-emit artifacts (six files):

```
crates/dsfb-gpu-atlas-corpus/out/task_manifest_v1.{txt,json}
crates/dsfb-gpu-atlas-corpus/out/dataset_manifest_v1.{txt,json}
crates/dsfb-gpu-atlas-corpus/out/activation_context_v1.{txt,json}
```

**S1.3c scope-discipline lock (plan-locked)**: schema +
verifier + conservative seed only. S1.3c does NOT:

- prune by budget or suppress redundancy (S1.3d);
- emit KernelPlanV1 (S1.3e);
- integrate into CaseFileV2 (S1.3f);
- consume T.8 usefulness ledger;
- mutate `corpus_hash_v1`, `registry_hash_v2`, or any
  T.11/S1.3a/S1.3b hash anchor;
- ingest external OTel / EBOM / PROV / RO-Crate exports;
- claim datasheet / model-card / NIST AI RMF / SLSA /
  in-toto / SPDX / CycloneDX compatibility on the new
  manifest hashes.

R.12b D64 baseline NOT rebaselined; pinned R.12b episode counts
(13/89/1917 for canonical/mid/full at K=1) byte-stable.

Receipts:
[reports/s1_3c_activation_context_summary.txt](reports/s1_3c_activation_context_summary.txt),
[reports/s1_3c_activation_context_verification.txt](reports/s1_3c_activation_context_verification.txt),
[reports/s1_3c_regression_check.txt](reports/s1_3c_regression_check.txt).

## Corpus Amendment Proposal V1 (T.12.0 — legal intake system for corpus scale-out)

T.12.0 ships the **legal intake system** for the T.12
Literature Corpus Scale-Out arc. No new literature primitives
land in T.12.0; they enter via T.12.a..m sub-campaigns
(starting with T.12.a = statistical process control).

Plan-locked first thesis:

> **T.12.0 introduces the amendment court for corpus scale-
> out: new literature primitives enter as reviewable
> amendment proposals, not silent mutations of
> `corpus_hash_v1`.**

This protects T.10's sealed `corpus_hash_v1`. New primitives
flow through a `CorpusAmendmentProposal` that the court
reviews (Open / Accepted / Rejected / Deferred); a future
formal freeze campaign produces `corpus_hash_v2` once enough
proposals have ratified.

**Three new own-namespace hashes** (plan-locked, NOT folded
upstream):

- `literature_expansion_batch_hash_v1` under
  `DSFB-GPU-ATLAS:LITERATURE-EXPANSION-BATCH:v1\0`
- `corpus_amendment_proposal_hash_v1` under
  `DSFB-GPU-ATLAS:CORPUS-AMENDMENT-PROPOSAL:v1\0`
- `dedup_court_delta_hash_v1` under
  `DSFB-GPU-ATLAS:DEDUP-COURT-DELTA:v1\0`

**Schema**:

```rust
CorpusExpansionBatch {
    batch_id,
    source_class,                          // one of 23 plan-locked variants
    proposed_primitives,                   // ProposedPrimitive[]
    proposed_aliases,                      // ProposedAliasClaim[]
    proposed_dedup_records,                // ProposedDedupRecord[]
    proposed_genealogy_edges,              // ProposedGenealogyEdge[]
    proposed_source_refs,                  // ProposedSourceRef[]
    literature_expansion_batch_hash_v1,
}

CorpusAmendmentProposal {
    proposal_id,
    motivation,
    target_source_class,
    body: CorpusExpansionBatch,
    dedup_court_delta: DedupCourtDelta,
    status,                                // Open / Accepted / Rejected / Deferred
    proposer_role,                         // PlanMember / ExternalReviewer / RobotIngestion
    created_at_commit,
    corpus_amendment_proposal_hash_v1,
}

DedupCourtDelta {
    delta_id,
    new_canonical_records,                 // DetectorCanonicalId[]
    new_alias_records,                     // DetectorAliasId[]
    new_composition_records,               // DetectorCanonicalId[]
    rejection_records,                     // RejectionRecord[]
    deferred_records,                      // DetectorAliasId[]
    dedup_court_delta_hash_v1,
}
```

**SourceClass enum** (23 plan-locked variants):
`StatisticalProcessControl`, `SequentialChangeDetection`,
`DriftDetection`, `RobustStatistics`, `DistributionDistance`,
`InformationTheory`, `SignalProcessing`, `SpectralAndWavelet`,
`TimeSeriesStructure`, `ControlResiduals`,
`FaultDetectionDiagnostics`, `ConditionMonitoring`,
`IndustrialProcessMonitoring`, `GraphAnomalyDetection`,
`StreamingSketches`, `DataQualityRules`,
`DatabaseIntegrityConstraints`, `ObservabilityDebugging`,
`MedicalBiosignal`, `RfCommunications`, `Chemometrics`,
`Econometrics`, `ReliabilitySurvival`.

**Conservative seed (proof-of-life)**: an empty proposal
under `StatisticalProcessControl` with status `Open` —
exercises the schema + hash + verifier + CLI surface without
filing any actual expansion. T.12.a (statistical process
control) replaces it with the first real proposal.

Sample emitted hashes from the proof-of-life proposal:

```
corpus_amendment_proposal_hash_v1    : 325bbf3deff3595b429a3cda1d55a2fc9e31d689aaeb7a88e3bf8f691fc80092
literature_expansion_batch_hash_v1   : a57190d895c661b0c0f83ba64917ede3b97339f2b90c365e3b555dca432973ed
dedup_court_delta_hash_v1            : 6cf1c20e9a028d7c1fe98676c4604051cfc21303ec2943ce07218fb34e279c87
```

**Verifier — 9 reject kinds (7 plan-locked rules + 2
structural)**:

  1. *`ProposalIdEmpty` (load-bearing #1)
  2. *`UnknownSourceClass` (load-bearing #2 — enforced at the
     enum level; verifier rule reserved for future TOML
     loader)
  3. *`DedupDeltaCollidesWithExistingSeedCanonicalId`
     (load-bearing #3 — would silently mutate the corpus)
  4. *`amendment_proposal_hash_changes_when_batch_changes`
     (load-bearing #4 — hash sensitivity)
  5.  `BatchIdEmpty`
  6.  `AcceptedProposalWithoutBodyOrDelta` (a no-op
      acceptance would silently commit nothing to a future
      freeze)
  7.  `AcceptedProposalWithoutFutureFreezeGate` (Accepted
      status requires a non-empty `created_at_commit`)
  8.  `AmendmentProposalHashMismatch` (structural integrity)
  9.  `BatchHashMismatch` / `DedupDeltaHashMismatch`
      (structural integrity)

CLI:

```bash
dsfb-corpus amendment-proposal      [--json] [--out PATH]
dsfb-corpus amendment-proposal-emit [--out-dir DIR]
```

The acceptance suite (31 tests) lives at
[tests/amendment_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/amendment_invariants.rs):
schema constants (4), seed shape (4), hash determinism +
sensitivity (7), verifier reject coverage (10),
rendering (4), upstream-anchor preservation (2).

Bulk artifacts:

```
crates/dsfb-gpu-atlas-corpus/out/corpus_amendment_proposal_v1.{txt,json}
```

**T.12.0 scope-discipline lock (plan-locked)**: this is
the legal intake system. T.12.0 does NOT:

- add any new literature primitive (those land in T.12.a..m);
- mutate `corpus_hash_v1`;
- create `corpus_hash_v2` (a future formal freeze does that
  once enough T.12.x sub-campaigns have landed);
- change `registry_hash_v2`, any T.11/S1.3a/b/c hash, or any
  DetectorPassport hash;
- activate new detectors;
- execute on GPU;
- regenerate the S1.2 registry;
- claim empirical usefulness;
- claim datasheet / model-card / NIST AI RMF / SLSA /
  in-toto / SPDX / CycloneDX compatibility on the three
  new hash namespaces.

R.12b D64 baseline NOT rebaselined; pinned R.12b episode counts
(13/89/1917 for canonical/mid/full at K=1) byte-stable.

**Plan-locked sub-campaign sequence**: T.12.a (SPC) →
T.12.b (sequential change detection) → T.12.c (drift /
distribution distance) → T.12.d (robust statistics) →
T.12.e (signal/spectral/wavelet) → T.12.f (time-series) →
T.12.g (graph/topology) → T.12.h (data quality / tabular /
DB constraints) → T.12.i (observability/debugging) →
T.12.j (medical / biosignal) → T.12.k (industrial / FDD /
condition monitoring) → T.12.l (chemometrics) → T.12.m
(RF / communications) → T.12.n (econometrics) → T.12.o
(reliability / survival) → T.12.p (streaming sketches) →
T.12.q (information theory) → T.12.consolidate (amendment
review + corpus_hash_v2 freeze).

Receipts:
[reports/t12_0_amendment_scaffold_summary.txt](reports/t12_0_amendment_scaffold_summary.txt),
[reports/t12_0_amendment_scaffold_verification.txt](reports/t12_0_amendment_scaffold_verification.txt),
[reports/t12_0_regression_check.txt](reports/t12_0_regression_check.txt).

## T.12.a — Statistical Process Control (first real expansion proposal)

T.12.a files the **first real corpus expansion proposal**
through the T.12.0 amendment court: Statistical Process
Control. The proposal does NOT mutate SEED; it is a docketed
legal act that the court can review.

Plan-locked commit identity:

> **T.12.a files the first real corpus expansion proposal:
> Statistical Process Control. It proposes canonical SPC
> primitives, collapses known aliases and rule-set
> compositions, emits a dedup-court delta, and proves the
> literature corpus can grow without mutating the frozen T.10
> corpus hash.**

**Scope (plan-locked)**:

- **2 new canonical primitives** (reserved canonical ids
  5001/5002, well above SEED's 54-record range):
  - `MEWMA` (Multivariate EWMA chart; Lowry et al. 1992)
  - `MCUSUM` (Multivariate CUSUM chart; Crosier 1988)
- **3 alias collapses** (targeting existing SEED canonicals):
  - "Q statistic" → PCA SPE / Q residual (SEED id 20)
  - "Squared Prediction Error" → PCA SPE / Q residual
  - "Hotelling T-square chart" → Hotelling T-squared (SEED id 5)
- **2 court-level composition reclassifications** of existing
  SEED canonicals (recorded; do not mutate SEED):
  - Western Electric SPC rules (SEED id 16) →
    `CompositionOf(Shewhart)`
  - Nelson SPC rules (SEED id 17) →
    `CompositionOf(Shewhart, Western Electric)`
- **4 genealogy edges** (MEWMA `DerivedFrom` EWMA; MCUSUM
  `DerivedFrom` CUSUM; Western Electric `Composes` Shewhart;
  Nelson `Composes` Western Electric).
- **4 source-ref records** for the new primitives and the
  composition decisions.

**Page-Hinkley note**: already canonical in SEED at id 4 and
is SPC-adjacent / sequential-change-detection-adjacent. Per
the plan verdict, T.12.a takes the "leave final authority
to T.12.b" path — Page-Hinkley is NOT touched by this
proposal. Pinned by the `t12_a_does_not_touch_page_hinkley`
acceptance test.

**Hash posture (plan-locked, MUST hold)**:

- `corpus_hash_v1` byte-identical (no SEED mutation).
- `SEED.len()` stays at 54 (pinned by
  `t12_a_does_not_add_records_to_seed`).
- `corpus_hash_v2` NOT created (a future formal freeze
  campaign does that).
- `registry_hash_v2`, every T.11/S1.3 hash, every
  DetectorPassport hash byte-identical.
- R.12b episodes 13/89/1917 byte-stable.
- **NEW**: `corpus_amendment_proposal_hash_v1` for the SPC
  proposal = `ae493a850c77ef681ef15ba5a5e11d77bc09446618ed6b1f6e6155feb1a8e92a`
  (distinct from T.12.0's proof-of-life hash).

**4 plan-required load-bearing negatives**:

1. `spc_rejects_duplicate_canonical_name_without_alias_decision`
2. `spc_rejects_western_electric_as_canonical_when_marked_composition`
3. `spc_rejects_q_statistic_alias_without_pca_spe_target`
4. `spc_amendment_hash_changes_when_one_source_ref_changes`

Plus 5 shape / non-mutation invariants:
`spc_proposal_does_not_mutate_seed_len`,
`spc_delta_lists_every_alias_with_reason_code`,
`spc_delta_lists_every_composition_with_components`,
`spc_source_refs_are_nonempty_for_every_claim`,
`spc_render_is_byte_stable_across_two_builds`.

CLI:

```bash
dsfb-corpus t12-a-spc-proposal      [--json] [--out PATH]
dsfb-corpus t12-a-spc-proposal-emit [--out-dir DIR]
```

Bulk artifacts (two builds byte-identical):

```
crates/dsfb-gpu-atlas-corpus/out/t12_a_spc_proposal_v1.{txt,json}
```

The acceptance suite (26 tests) lives at
[tests/t12_a_spc_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/t12_a_spc_invariants.rs):
seed shape + admissibility (10), hash determinism +
sensitivity (4), load-bearing negatives (3 explicit + 1
hash sensitivity), rendering (2), upstream-anchor
preservation (5), Page-Hinkley deferral (1), reserved-id
guard (1).

**T.12.a scope-discipline lock**: this is a proposal, not
a corpus mutation. T.12.a does NOT:

- mutate `SEED` (the 54-record T.10 baseline);
- mutate `corpus_hash_v1`;
- create `corpus_hash_v2`;
- change `registry_hash_v2` or any T.11/S1.3 hash;
- activate the new detectors (they remain reserved
  canonical ids pending the formal freeze);
- touch Page-Hinkley (deferred to T.12.b);
- claim empirical usefulness;
- execute on GPU.

Receipts:
[reports/t12_a_spc_proposal_summary.txt](reports/t12_a_spc_proposal_summary.txt),
[reports/t12_a_spc_proposal_verification.txt](reports/t12_a_spc_proposal_verification.txt),
[reports/t12_a_regression_check.txt](reports/t12_a_regression_check.txt).

## T.12.b — Sequential Change Detection (cross-class dedup authority)

The second real corpus expansion proposal through the T.12.0
amendment court. **Plan-locked thesis**:

> **T.12.b files the Sequential Change Detection amendment
> proposal. It adds only defensible new canonical primitives,
> resolves cross-class authority for existing Page-Hinkley /
> CUSUM / Mann-Kendall (plus the already-canonical Pettitt /
> SNHT / MOSUM / Buishand range records), rejects
> non-deterministic BOCPD as a canonical detector, and emits a
> dedup-court delta without mutating the frozen T.10 corpus.**

Main plan instruction: *"Do not chase quantity yet. Prove
cross-class dedup authority."* The headline is therefore
**cross-class dedup authority**, not detector quantity.

**Body counts**:

| Category | Count |
|---|---|
| Proposed primitives (4 canonical + 1 BOCPD shell) | 5 |
| Proposed aliases | 0 |
| Proposed dedup-court records | 13 |
| Proposed genealogy edges | 6 |
| Proposed source refs | 6 |
| `new_canonical_records` in the delta | 4 |

**Four court-delta categories** (plan-locked wire names in
`ProposedDedupRecord::decision_wire_name`):

- `CanonicalAddition` (×4): Shiryaev-Roberts (5201), GLR
  (5202), Binary segmentation (5207), PELT-style deterministic
  (5208).
- `ExistingCanonicalAuthorityResolution` (×7): CUSUM (SEED 3),
  Page-Hinkley (4), Mann-Kendall (11), Pettitt (34), SNHT (35),
  MOSUM (36), Buishand range (37). Each kept canonical; no
  duplicate admitted under reserved 5xxx ids.
- `DomainTransferOf` (×1): CUSUM (SEED 3) recorded as the
  shared SCD ancestor for the four new canonicals without
  re-canonicalising CUSUM.
- `RejectedNotDeterministic` (×1): BOCPD (reserved id 5209) —
  the literature record is acknowledged in
  `proposed_primitives` but explicitly NOT admitted to
  `new_canonical_records`. A future T.12.x proposal may admit a
  `Deterministic_BOCPD_Proxy` canonical with the hazard, prior,
  update law, truncation, and numeric mode declared.

`DeferredToDriftDetection` (prose-only, NOT a court-delta
record): ADWIN, DDM, EDDM, HDDM, KSWIN and the stream-drift
family are NOT folded into T.12.b — they belong in T.12.c
(drift / distribution distance).

**Why 4 new canonicals, not 8** — an earlier draft proposed
adding Pettitt / SNHT / MOSUM / Buishand range as new canonicals,
but a walk of [src/seed.rs](crates/dsfb-gpu-atlas-corpus/src/seed.rs)
found all four already canonical (ids 34, 35, 36, 37). Promoting
them again would have produced the very duplication T.12.b
exists to forbid. They become
`ExistingCanonicalAuthorityResolution` records instead, and four
plan-required load-bearing negatives pin the rule.

**14 plan-required load-bearing negatives** (in
[tests/t12_b_scd_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/t12_b_scd_invariants.rs)):
SEED non-mutation; seven SEED-collision tests (one per CUSUM /
Page-Hinkley / Mann-Kendall / Pettitt / SNHT / MOSUM / Buishand);
BOCPD not in `new_canonical_records`; BOCPD present in
`proposed_primitives`; alias-target-must-exist (vacuous, encodes
invariant); domain-transfer-target-must-exist; every new
canonical has a source ref; every rejection has a reason code;
plus the hash-sensitivity test. Plus 20 additional invariants
(shape, hash determinism, hash sensitivity, rendering byte-
stability, court-delta category coverage, BOCPD rejection-record
completeness, canonical-id reserved-range guard). Total: 34
acceptance tests.

**Sample emitted hashes** (byte-identical across two builds):

```
literature_expansion_batch_hash_v1 : 8d043a5c4694015216e57c61f5240fe04477b3fe0b535f4188a461076adc6d13
dedup_court_delta_hash_v1          : 6ba7a7810d2d801472c16efec9f0aae958e0a262eeb2bddb15a1bb50e713ce0f
corpus_amendment_proposal_hash_v1  : ac10e27cdc9286b44e18ac8ae29a46143571ecf89ca55e0283cf97ea2c19525f
```

Distinct from T.12.0's proof-of-life hash (`325bbf3d...`) and
T.12.a's SPC hash (`ae493a85...`).

**CLI**:

```
dsfb-corpus t12-b-scd-proposal      [--json] [--out PATH]
dsfb-corpus t12-b-scd-proposal-emit [--out-dir DIR]
```

Bulk artifacts:
[out/t12_b_scd_proposal_v1.txt](crates/dsfb-gpu-atlas-corpus/out/t12_b_scd_proposal_v1.txt),
[out/t12_b_scd_proposal_v1.json](crates/dsfb-gpu-atlas-corpus/out/t12_b_scd_proposal_v1.json).

**Plan-locked non-claims** — T.12.b does NOT:

- mutate `corpus_hash_v1` (`SEED.len()` stays at 54);
- mutate `registry_hash_v2`, any T.11/S1.3/T.12.0/T.12.a hash,
  or any DetectorPassport hash;
- create `corpus_hash_v2`;
- promote BOCPD to `new_canonical_records` (rejected as
  `RejectedNotDeterministic`);
- relabel Mann-Kendall as a generic change-point detector (it
  stays a TREND witness);
- fold the stream-drift family into the SCD source class
  (deferred to T.12.c);
- activate new detectors (S1.3a planner not re-emitted);
- execute on GPU (corpus crate stays host-only and zero-dep);
- claim empirical usefulness for the new primitives (T.8
  ledger stays at `NotScored` for the 4 reserved canonicals).

Receipts:
[reports/t12_b_scd_proposal_summary.txt](reports/t12_b_scd_proposal_summary.txt),
[reports/t12_b_scd_proposal_verification.txt](reports/t12_b_scd_proposal_verification.txt),
[reports/t12_b_regression_check.txt](reports/t12_b_regression_check.txt).

Next campaign: **T.12.c — drift / distribution distance**
(ADWIN, DDM, EDDM, HDDM, KSWIN under `DriftDetection`).

## T.12.c — Drift Detection and Distribution-Distance Authority

The third real corpus expansion proposal through the T.12.0
amendment court. **Plan-locked thesis**:

> **T.12.c files the Drift Detection / Distribution Distance
> amendment proposal. It adds only deterministic drift-distance
> primitives whose reference-distribution, windowing, binning,
> and sampling contracts are declared; resolves collisions with
> existing SEED records; classifies streaming drift algorithms
> as canonical, parameterized, domain-transfer, or deferred
> without mutating the frozen T.10 corpus.**

Main plan instruction: *"Do not count method names. Count
distinct deterministic decision functionals with declared
reference / window / sampling contracts."*

T.12.c's design began with the plan-required
`t12_c_detects_existing_seed_collisions_before_new_canonical_assignment`
walk. A grep of [seed.rs](crates/dsfb-gpu-atlas-corpus/src/seed.rs)
for every candidate name in the plan's draft list found
**eleven** distribution-distance primitives already canonical
(KS id 8, KL 9, MMD 10, Anderson-Darling 26, Cramer-von Mises
27, Wasserstein 28, Energy distance 29, Hellinger 30, PSI 31,
Jensen-Shannon 32, Total variation 33). Of the six remaining
candidates, the court ruled four are canonical additions and
two are parameterizations.

**Body counts**:

| Category | Count |
|---|---|
| Proposed primitives (4 canonical + 2 parameterizations) | 6 |
| Proposed aliases | 0 |
| Proposed dedup-court records | 18 |
| Proposed genealogy edges | 6 |
| Proposed source refs | 6 |
| `new_canonical_records` in the delta | 4 |

**Four court-delta categories** — the new `ParameterizationOf`
category lands for the first time at T.12.c:

- `CanonicalAddition` (×4): Kuiper (5301), ADWIN (5302), DDM
  (5303), HDDM (5304). Each carries a declared deterministic
  contract (reference window pair / Hoeffding-delta / cut rule
  / ordered binary error sequence + running-minimum reference
  window / numeric mode).
- `ExistingCanonicalAuthorityResolution` (×11): SEED 8, 9, 10,
  26..=33 — every existing SEED distribution-distance canonical
  kept canonical under the `DriftDetection` source class without
  duplication. PSI's record additionally declares the binning
  law contract (bin edges, bin count, empty-bin treatment).
- `DomainTransferOf` (×1): KS (SEED 8) named as the shared
  two-sample distribution-distance ancestor recognised by the
  drift-detection source class.
- `ParameterizationOf` (×2, NEW category): EDDM (5305) as
  `ParameterizationOf(DDM)`; KSWIN (5306) as
  `ParameterizationOf(KS)`. Both appear in `proposed_primitives`
  but NOT in `new_canonical_records`. A future T.12.x proposal
  may promote either to canonical if the family-distinct
  decision functional warrants it.

`DeferredToDriftDetection` from T.12.b is now actively
exercised — the SCD primitives admitted in T.12.b
(Shiryaev-Roberts, GLR, Binary segmentation, PELT) are NOT
duplicated under the `DriftDetection` source class.

**Sample emitted hashes** (byte-identical across two builds):

```
literature_expansion_batch_hash_v1 : 5723f79a9288a824fd05fccdc8bd56e101d3842c3aae7f99071c4432f0b8c935
dedup_court_delta_hash_v1          : 2a9c0e1a32b1bbb4990f33b008056d0934d40f950a35d643be489c4d7946d10c
corpus_amendment_proposal_hash_v1  : 0ffb0639256c318c0d3d1ffa3a24cc5b567a931ece1a0617251ad23253a4a92d
```

Distinct from T.12.0 (`325bbf3d...`), T.12.a (`ae493a85...`),
and T.12.b (`ac10e27c...`).

**9 plan-required load-bearing negatives** (in
[tests/t12_c_drift_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/t12_c_drift_invariants.rs)):
SEED non-mutation; parametric SEED-collision loop (11 ids ×
collision-rule firing); KSWIN-without-KS-relationship;
ADWIN-without-adaptive-window-law; DDM-family-variant-without-
family-relationship; distribution-distance-without-reference-
distribution-requirement; PSI-without-binning-law;
probabilistic-or-randomized-distance-without-deterministic-
reduction; hash-changes-when-distance-formula-or-source-ref-
changes. Plus 5 representative per-SEED-id collision tests + 22
additional invariants. **Total: 36 acceptance tests.**

**CLI**:

```
dsfb-corpus t12-c-drift-proposal      [--json] [--out PATH]
dsfb-corpus t12-c-drift-proposal-emit [--out-dir DIR]
```

Bulk artifacts:
[out/t12_c_drift_proposal_v1.txt](crates/dsfb-gpu-atlas-corpus/out/t12_c_drift_proposal_v1.txt),
[out/t12_c_drift_proposal_v1.json](crates/dsfb-gpu-atlas-corpus/out/t12_c_drift_proposal_v1.json).

**Plan-locked non-claims** — T.12.c does NOT:

- mutate `corpus_hash_v1` (`SEED.len()` stays at 54);
- mutate any T.11/S1.3/T.12.0/T.12.a/T.12.b hash or any
  DetectorPassport hash;
- create `corpus_hash_v2`;
- promote EDDM or KSWIN to `new_canonical_records`;
- duplicate any already-canonical SEED distribution-distance
  primitive (the eleven authority-resolution records + the
  parametric collision-loop test prevent it);
- duplicate the SCD primitives T.12.b admitted (Shiryaev-
  Roberts / GLR / Binary segmentation / PELT remain under SCD
  only);
- claim a reference-distribution-free distance / divergence
  (every record declares its reference contract);
- claim probabilistic / randomized distances as canonical
  without an explicit deterministic-reduction status;
- activate new detectors (S1.3a planner not re-emitted);
- execute on GPU (corpus crate stays host-only and zero-dep);
- claim empirical usefulness for the new primitives (T.8
  ledger stays `NotScored` for the 4 new canonicals).

Receipts:
[reports/t12_c_drift_proposal_summary.txt](reports/t12_c_drift_proposal_summary.txt),
[reports/t12_c_drift_proposal_verification.txt](reports/t12_c_drift_proposal_verification.txt),
[reports/t12_c_regression_check.txt](reports/t12_c_regression_check.txt).

Next campaign: **T.12.d — robust statistics** (robust-z, MAD,
Hampel filter, Tukey fences, trimmed mean shift, biweight
midvariance, Theil-Sen slope, etc.). Same SEED-walk-first
discipline.

## T.12.d — Robust Statistics (first proposal exercising all five court-delta categories)

The fourth real corpus expansion proposal through the T.12.0
amendment court. **Plan-locked thesis**:

> **T.12.d files the Robust Statistics amendment proposal.
> It resolves robust z / MAD aliases against the existing SEED
> canonical, admits only robust primitives with explicit
> estimator, window, trimming, quartile, or pair-selection
> laws, rejects stochastic RANSAC-style claims unless
> deterministically reduced, and preserves the frozen T.10
> corpus hash.**

Main plan instruction: *"Robust-statistics names are
alias-heavy. Make estimator law explicit, or collapse / defer."*

**Architectural milestone**: T.12.d is the **first proposal to
exercise ALL FIVE plan-locked court-delta categories**
(`CanonicalAddition`, `ExistingCanonicalAuthorityResolution`,
`DomainTransferOf`, `ParameterizationOf`,
`RejectedNotDeterministic`). The wire-name set is now closed
at five.

T.12.d's design began with the SEED-walk-first discipline. A
grep of [seed.rs](crates/dsfb-gpu-atlas-corpus/src/seed.rs)
found **three** robust-statistics primitives already canonical
(robust-z id 6, Hampel filter id 7, Tukey fences id 18). All
three become `ExistingCanonicalAuthorityResolution` records;
modified z-score, MAD outlier detector, rolling Hampel, k×IQR
variants — every alias name in the literature — collapse into
one of these three records OR into a `ParameterizationOf`
shell.

**Body counts**:

| Category | Count |
|---|---|
| Proposed primitives (4 canonical + 3 parameterizations + 1 RANSAC shell) | 8 |
| Proposed aliases | 0 |
| Proposed dedup-court records | 12 |
| Proposed genealogy edges | 7 |
| Proposed source refs | 8 |
| `new_canonical_records` in the delta | 4 |

**Five court-delta categories** (the wire-name set is now
closed):

- `CanonicalAddition` (×4): Theil-Sen slope estimator (5401;
  declared pair-selection + slope-median + tie-break + window
  laws), biweight midvariance (5402; tuning constant +
  convergence threshold + iteration count), trimmed mean shift
  (5403; trim fraction + symmetric/one-sided + percentile/count
  semantics), winsorized mean shift (5404; winsor limit +
  replacement rule).
- `ExistingCanonicalAuthorityResolution` (×3): robust z-score
  (SEED 6; declared median + MAD + threshold + window), Hampel
  filter (SEED 7; declared windowed local-median + MAD +
  threshold + replacement/rejection rule), Tukey fences (SEED
  18; declared quartile estimator + IQR multiplier +
  inclusive/exclusive fence + tie-handling).
- `DomainTransferOf` (×1): robust-z (SEED 6) as the shared
  robust-location-estimator ancestor for the
  `RobustStatistics` source class.
- `ParameterizationOf` (×3): modified z-score (5405) as
  `ParameterizationOf(robust-z, SEED 6)`; rolling Hampel
  (5406) as `ParameterizationOf(Hampel, SEED 7)`; k×IQR fence
  (5407) as `ParameterizationOf(Tukey fences, SEED 18)`. All
  three appear in `proposed_primitives` but NOT in
  `new_canonical_records`.
- `RejectedNotDeterministic` (×1): RANSAC residual proxy
  (5408; randomized in origin — admitted neither to SEED nor
  to `new_canonical_records` unless a future T.12.x proposal
  admits a `Deterministic_RANSAC_Proxy` canonical with sample
  seed + iteration budget + fixed sample schedule + tie-break
  law + numeric mode all brutally explicit).

**Sample emitted hashes** (byte-identical across two builds):

```
literature_expansion_batch_hash_v1 : 7fffd93ab387b101626aad84c950915b4c43c3f514b37e5c14ad671f8c2509e9
dedup_court_delta_hash_v1          : 182000af0592fd6ff992b0bb8b22abf602562db417859089be2dfc788aac759b
corpus_amendment_proposal_hash_v1  : 7b51703fae33c53ee438bdcc8b4fef796d7da736025416bc81e6318f4b426eff
```

Distinct from T.12.0 (`325bbf3d…`), T.12.a (`ae493a85…`),
T.12.b (`ac10e27c…`), and T.12.c (`0ffb0639…`).

**9 plan-required load-bearing negatives** (in
[tests/t12_d_robust_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/t12_d_robust_invariants.rs)):
SEED non-mutation; robust-z duplicate rejection; Hampel
without windowed-local-median law; Tukey fence without
quartile law; Theil-Sen without pair-selection law; RANSAC
without deterministic seed + schedule + iteration budget +
tie-break; trimmed mean without trim-fraction law; winsorized
mean without winsor-limit law; hash-changes-when-robust-
statistic-law-changes. Plus parametric SEED collision loop +
2 per-named-record collision tests + 4 ParameterizationOf
family-relationship tests + biweight estimator-law test + 19
shape / determinism / rendering invariants. **Total: 38
acceptance tests.**

**CLI**:

```
dsfb-corpus t12-d-robust-proposal      [--json] [--out PATH]
dsfb-corpus t12-d-robust-proposal-emit [--out-dir DIR]
```

Bulk artifacts:
[out/t12_d_robust_proposal_v1.txt](crates/dsfb-gpu-atlas-corpus/out/t12_d_robust_proposal_v1.txt),
[out/t12_d_robust_proposal_v1.json](crates/dsfb-gpu-atlas-corpus/out/t12_d_robust_proposal_v1.json).

**Plan-locked non-claims** — T.12.d does NOT:

- mutate `corpus_hash_v1` (`SEED.len()` stays at 54);
- mutate any prior T.11/S1.3/T.12.x hash or any
  DetectorPassport hash;
- create `corpus_hash_v2`;
- duplicate robust-z, Hampel, or Tukey fences (the three
  authority resolutions + the parametric collision-loop test
  prevent it);
- promote modified z-score, rolling Hampel, or k×IQR fence
  to `new_canonical_records` (each is `ParameterizationOf`);
- promote RANSAC to `new_canonical_records` (rejected
  `RejectedNotDeterministic`);
- claim a robust-statistics primitive without declared
  estimator law (every record declares its specific
  operational contract);
- activate new detectors (S1.3a planner not re-emitted);
- execute on GPU (corpus crate stays host-only and zero-dep);
- claim empirical usefulness for the new primitives (T.8
  ledger stays `NotScored` for the 4 new canonicals).

Receipts:
[reports/t12_d_robust_proposal_summary.txt](reports/t12_d_robust_proposal_summary.txt),
[reports/t12_d_robust_proposal_verification.txt](reports/t12_d_robust_proposal_verification.txt),
[reports/t12_d_regression_check.txt](reports/t12_d_regression_check.txt).

Next campaign: **T.12.e — signal processing / spectral /
wavelet**. Same SEED-walk-first discipline; FFT band energy,
spectral entropy, wavelet coefficient burst, matched filter
residual, envelope detector are existing SEED records that
will become `ExistingCanonicalAuthorityResolution`.

## T.12.e — Signal Processing / Spectral / Wavelet (transform-law discipline)

The fifth real corpus expansion proposal through the T.12.0
amendment court. **Plan-locked thesis**:

> **T.12.e files the Signal Processing / Spectral / Wavelet
> amendment proposal. It admits only deterministic transform-
> based primitives whose sampling, windowing, normalization,
> band, boundary, and template laws are declared; resolves
> collisions with existing SEED records; classifies transform
> variants as parameterizations; rejects randomized or learned
> spectral claims unless deterministically reduced; and
> preserves the frozen T.10 corpus hash.**

Main plan warning: *"In spectral detectors, the transform law
is the detector. No transform law, no canonical admission."*

T.12.e's design began with the SEED-walk-first discipline.
A grep of [seed.rs](crates/dsfb-gpu-atlas-corpus/src/seed.rs)
found **five** signal/spectral primitives already canonical:
FFT band-energy anomaly (id 12), residual envelope exit (id
22), spectral entropy (id 38), wavelet coefficient energy (id
39), autocorrelation-coefficient break (id 40). All five
become `ExistingCanonicalAuthorityResolution` records with
declared transform-law contracts.

**Body counts**:

| Category | Count |
|---|---|
| Proposed primitives (6 canonical + 3 parameterizations + 1 RANSAC-style shell) | 10 |
| Proposed aliases | 0 |
| Proposed dedup-court records | 16 |
| Proposed genealogy edges | 9 |
| Proposed source refs | 7 |
| `new_canonical_records` in the delta | 6 |

**Five court-delta categories** (all exercised):

- `CanonicalAddition` (×6): Spectral centroid shift (5501;
  power-spectrum convention + frequency-bin mapping + first-
  moment formula + sampling law), wavelet packet energy
  (5502; wavelet family + packet-tree depth + energy
  convention + boundary handling + sampling law), STFT ridge
  shift (5503; window function + window length + hop / overlap
  + ridge selection law + extrapolation + sampling law),
  cepstral anomaly (5504; FFT convention + log base +
  real/complex cepstrum + sampling rate), matched filter
  residual (5505; template provenance + sampling-rate match +
  normalization), Hilbert amplitude anomaly (5506;
  analytic-signal extraction method + sampling law).
- `ExistingCanonicalAuthorityResolution` (×5): SEED 12, 22,
  38, 39, 40 — each with declared transform-law contract.
- `DomainTransferOf` (×1): FFT band-energy (SEED 12) as the
  shared spectral-transform ancestor for `SignalProcessing`.
- `ParameterizationOf` (×3): FFT bandpower variant (5507) of
  SEED 12; wavelet family variant (5508) of SEED 39; STFT
  window/hop variant (5509) of 5503.
- `RejectedNotDeterministic` (×1): randomized spectral
  projection (5510; Rahimi & Recht 2007 random Fourier
  features) — admission requires seed + projection matrix
  definition + dimension + numeric mode declared.

**Sample emitted hashes** (byte-identical across two builds):

```
literature_expansion_batch_hash_v1 : fbb3a3b80a71cf8940bce7e2fd2cc3733a78f7af27602c316644874bdb31c526
dedup_court_delta_hash_v1          : dc2b6d828fd478c81a899c0b4e8bf5ebb539371ded6d9e99e1ce85c7baae4896
corpus_amendment_proposal_hash_v1  : 0036435fb1b000557c75d4a7ae418560a0117761fe7ea874b00ecc30b12ac2f8
```

Distinct from T.12.0/T.12.a/T.12.b/T.12.c/T.12.d.

**10 plan-required load-bearing negatives** (in
[tests/t12_e_spectral_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/t12_e_spectral_invariants.rs)):
SEED non-mutation; FFT-variant-without-window-and-normalization-
law (most important for spectral); spectral-entropy-without-bin-
or-power-normalization-law; wavelet-detector-without-wavelet-
family-and-boundary-law; STFT-ridge-without-window-hop-and-
ridge-selection-law; matched-filter-without-template-provenance;
Hilbert-amplitude-without-sampling-law; randomized-spectral-
projection-without-deterministic-reduction; hash-changes-when-
transform-law-changes; parametric SEED collision loop. Plus 3
per-named-record collision tests + 4 ParameterizationOf family
tests + 2 per-CanonicalAddition transform-law declarations +
18 shape / determinism / rendering invariants. **Total: 39
acceptance tests.**

**CLI**:

```
dsfb-corpus t12-e-spectral-proposal      [--json] [--out PATH]
dsfb-corpus t12-e-spectral-proposal-emit [--out-dir DIR]
```

Bulk artifacts:
[out/t12_e_spectral_proposal_v1.txt](crates/dsfb-gpu-atlas-corpus/out/t12_e_spectral_proposal_v1.txt),
[out/t12_e_spectral_proposal_v1.json](crates/dsfb-gpu-atlas-corpus/out/t12_e_spectral_proposal_v1.json).

**Plan-locked non-claims** — T.12.e does NOT: mutate any
upstream hash; create `corpus_hash_v2`; duplicate FFT band-
energy / residual envelope exit / spectral entropy / wavelet
coefficient energy / autocorrelation break; promote FFT
bandpower variant / wavelet family variant / STFT window/hop
variant to `new_canonical_records`; promote randomized
spectral projection to `new_canonical_records`; claim a
spectral primitive without declared transform law; activate
new detectors; execute on GPU; claim empirical usefulness
(T.8 ledger stays `NotScored`).

Receipts:
[reports/t12_e_spectral_proposal_summary.txt](reports/t12_e_spectral_proposal_summary.txt),
[reports/t12_e_spectral_proposal_verification.txt](reports/t12_e_spectral_proposal_verification.txt),
[reports/t12_e_regression_check.txt](reports/t12_e_regression_check.txt).

Next campaign: **T.12.f — time-series structure / control
residuals** (AR / ARIMA / STL residual, seasonal decomposition,
lag correlation break, variance ratio, burstiness, run-length,
periodicity break, recurrence interval, observer residual,
innovation sequence, parity-space residual).

## T.12.f — Time-Series Structure / Control Residuals (residual-and-decision-law discipline)

The sixth real corpus expansion proposal through the T.12.0
amendment court. **Plan-locked thesis**:

> **T.12.f files the Time-Series Structure / Control Residuals
> amendment proposal. It admits only deterministic time-
> structure and residual-observer witnesses whose model, lag,
> residual, envelope, innovation, sampling, and decision laws
> are declared; resolves SEED collisions; classifies model
> variants as parameterizations; rejects stochastic or
> unidentified model-fitting claims unless deterministically
> reduced; and preserves the frozen T.10 corpus hash.**

Main plan warning: *"A model is not a detector until the
residual and decision law are declared."*

T.12.f's design began with the SEED-walk-first discipline.
A grep of [seed.rs](crates/dsfb-gpu-atlas-corpus/src/seed.rs)
found **four** T.12.f-relevant primitives already canonical
(sensor bias 23, actuator stiction 24, valve hunting 25, Error
burst 41), plus **two** records already recognised in T.12.e
under `SignalProcessing` (residual envelope exit 22,
autocorrelation break 40) recognised again here under
`TimeSeriesStructure`. All six become
`ExistingCanonicalAuthorityResolution` records with declared
contract laws.

**Body counts**:

| Category | Count |
|---|---|
| Proposed primitives (8 canonical + 3 parameterizations + 1 unidentified-model shell) | 12 |
| Proposed aliases | 0 |
| Proposed dedup-court records | 19 |
| Proposed genealogy edges | 10 |
| Proposed source refs | 8 |
| `new_canonical_records` in the delta | 8 |

**Five court-delta categories** (all exercised):

- `CanonicalAddition` (×8) with declared model-and-decision
  laws: AR residual (5601), ARIMA residual (5602), STL
  residual (5603), lag-correlation break (5604), variance-
  ratio shift (5605), run-length anomaly (5606), observer
  residual (5607; plant-or-observer contract + state model +
  measurement model + observer gain + residual + envelope +
  threshold), parity-space residual (5608; plant-or-observer
  contract + parity equations + residual + envelope +
  threshold).
- `ExistingCanonicalAuthorityResolution` (×6): SEED 22, 23,
  24, 25, 40, 41 — each with declared contract law.
- `DomainTransferOf` (×1): residual envelope exit (SEED 22)
  as shared residual-witness ancestor for `TimeSeriesStructure`
  (recognised by both time-series-residual AND control-
  residual sub-families).
- `ParameterizationOf` (×3): innovation sequence (5609) as
  `ParameterizationOf(Observer residual, 5607)` with Kalman-
  specific Q/R covariance declaration; periodicity break
  (5610) as `ParameterizationOf(Lag-correlation break, 5604)`
  with peak-selection law; burstiness index (5611) as
  `ParameterizationOf(Error burst, SEED 41)`.
- `RejectedNotDeterministic` (×1): unidentified-model anomaly
  (5612) — "ARIMA with auto-determined order", "Kalman
  without declared Q/R", "STL with adaptive seasonality",
  "observer with fit-during-deployment" are randomized /
  unidentified in origin; admission requires model-order-
  search seed + identification algorithm + fit-data anchor +
  tie-break law + numeric mode declared.

**Sample emitted hashes** (byte-identical across two builds):

```
literature_expansion_batch_hash_v1 : 4b54d67b8cfbad5b698db85c070ea6a576568cd6ab5952bdfa0256e6d77f0d77
dedup_court_delta_hash_v1          : e807688693b1740d0c435ea4efc3bbd63e5705182bbdd7545a2e9edb016d66dc
corpus_amendment_proposal_hash_v1  : 76717e0232019595922fa8258599084c22d8c5d566d0734f4b476507fa204802
```

Distinct from every prior T.12.x proposal hash.

**10 plan-required load-bearing negatives** (in
[tests/t12_f_timeseries_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/t12_f_timeseries_invariants.rs)):
SEED non-mutation; ARIMA-residual-without-model-order-and-fit-
law; STL-residual-without-seasonality-and-decomposition-law;
innovation-sequence-without-observer-model; periodicity-break-
without-lag-and-peak-selection-law; variance-ratio-without-
window-pair-law; run-length-without-event-definition; control-
residual-without-plant-or-observer-contract (MOST IMPORTANT;
covers both 5607/5608 canonicals AND SEED 23/24/25 authority
resolutions); hash-changes-when-residual-definition-changes;
parametric SEED collision loop over 6 ids. Plus 3 per-named
collision tests + AR-residual contract test + lag-correlation
declaration test + burstiness ParameterizationOf test +
unidentified-model rejection contract test + 19 shape /
determinism / rendering invariants. **Total: 37 acceptance
tests.**

**CLI**:

```
dsfb-corpus t12-f-timeseries-proposal      [--json] [--out PATH]
dsfb-corpus t12-f-timeseries-proposal-emit [--out-dir DIR]
```

Bulk artifacts:
[out/t12_f_timeseries_proposal_v1.txt](crates/dsfb-gpu-atlas-corpus/out/t12_f_timeseries_proposal_v1.txt),
[out/t12_f_timeseries_proposal_v1.json](crates/dsfb-gpu-atlas-corpus/out/t12_f_timeseries_proposal_v1.json).

**Plan-locked non-claims** — T.12.f does NOT: mutate any
upstream hash; create `corpus_hash_v2`; duplicate sensor bias /
actuator stiction / valve hunting / residual envelope exit /
autocorrelation break / Error burst; promote innovation
sequence / periodicity break / burstiness index to
`new_canonical_records`; promote unidentified-model anomaly to
`new_canonical_records`; claim a model-derived detector
without declared residual + decision law; activate new
detectors; execute on GPU; claim empirical usefulness (T.8
ledger stays `NotScored`).

Receipts:
[reports/t12_f_timeseries_proposal_summary.txt](reports/t12_f_timeseries_proposal_summary.txt),
[reports/t12_f_timeseries_proposal_verification.txt](reports/t12_f_timeseries_proposal_verification.txt),
[reports/t12_f_regression_check.txt](reports/t12_f_regression_check.txt).

Next campaign: **T.12.g — graph / topology anomaly** (degree
spike, betweenness shift, PageRank residual, community
boundary shift, cascade precursor, motif-count anomaly).

## T.12.g — Graph / Topology Anomaly (first proposal with two RejectedNotDeterministic records)

The seventh real corpus expansion proposal through the T.12.0
amendment court. **Plan-locked thesis**:

> **T.12.g files the Graph / Topology Anomaly amendment
> proposal. It admits only deterministic topology witnesses
> whose graph model, baseline, update law, metric definition,
> normalization, node / edge identity law, and decision
> functional are declared; resolves SEED collisions;
> classifies metric variants as parameterizations; rejects
> community / embedding / random-walk claims unless
> deterministically reduced; and preserves the frozen T.10
> corpus hash.**

Main plan warning: *"A graph metric is not a detector until
the baseline, update law, metric law, and decision law are
declared."*

**Architectural milestone**: T.12.g is the **first T.12.x
proposal to carry TWO `RejectedNotDeterministic` records in
one commit** — community boundary shift (Louvain / Leiden /
label propagation / Infomap) AND random-walk embedding
anomaly (DeepWalk / node2vec). Both rejected as randomized /
implementation-sensitive in origin.

The SEED walk found only ONE graph-adjacent primitive already
canonical (Fanout cascade 43); the corpus is graph-anomaly-
sparse. T.12.g is therefore CanonicalAddition-heavy.

**Body counts**:

| Category | Count |
|---|---|
| Proposed primitives (8 canonical + 3 parameterizations + 2 rejection shells) | 13 |
| Proposed aliases | 0 |
| Proposed dedup-court records | 15 |
| Proposed genealogy edges | 4 |
| Proposed source refs | 10 |
| `new_canonical_records` in the delta | 8 |

**Five court-delta categories** (all exercised; with TWO
RejectedNotDeterministic records for the first time):

- `CanonicalAddition` (×8) with declared graph + decision
  laws: degree spike (5701), betweenness shift (5702),
  clustering-coefficient shift (5703), PageRank residual
  (5704), edge-cut anomaly (5705), bridge-node emergence
  (5706), cascade precursor (5707; temporal predictor;
  distinct from SEED 43 active-cascade detector), motif-count
  anomaly (5708).
- `ExistingCanonicalAuthorityResolution` (×1): Fanout cascade
  (SEED 43).
- `DomainTransferOf` (×1): Fanout cascade as shared cascade
  ancestor for `GraphAnomalyDetection`.
- `ParameterizationOf` (×3): weighted-degree spike (5709) of
  degree spike; k-hop fanout (5710) of Fanout cascade;
  directed motif-count (5711) of motif-count anomaly.
- `RejectedNotDeterministic` (×2): community boundary shift
  (5712 — admission requires algorithm + seed + tie-break +
  modularity rule + resolution parameter + convergence law);
  random-walk embedding anomaly (5713 — admission requires
  walk seed + walk length + walk count + tie-break +
  embedding-projection matrix anchor + numeric mode).

**Sample emitted hashes** (byte-identical across two builds):

```
literature_expansion_batch_hash_v1 : 0358c53be687c24c5eed965dfde8b8ef28f013a48bed098917f3d6b063348528
dedup_court_delta_hash_v1          : 37a2d159a60fa2d655acccef97307496f3cd9c76e01f86393e2ad3e80afadbe1
corpus_amendment_proposal_hash_v1  : 1bc142666cdf680f352ee8327ec53a4699f63601b5371db284600f75bde81ddd
```

Distinct from every prior T.12.x proposal hash.

**10 plan-required load-bearing negatives** (in
[tests/t12_g_graph_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/t12_g_graph_invariants.rs)):
SEED non-mutation; degree-spike-without-baseline-and-graph-
update-law; betweenness-shift-without-shortest-path-and-
normalization-law; PageRank-residual-without-damping-and-
dangling-node-law; community-shift-without-deterministic-
partition-law (MOST IMPORTANT); motif-count-without-motif-
enumeration-law; cascade-precursor-without-temporal-edge-
order-law; bridge-node-without-connectivity-definition; hash-
changes-when-graph-metric-law-changes; SEED-collision-
requires-authority-resolution. Plus random-walk-embedding
rejection contract test + edge-cut/clustering contract tests +
3 ParameterizationOf family tests + 2-rejection-records
witness + shape/determinism/rendering invariants. **Total: 36
acceptance tests.**

**CLI**:

```
dsfb-corpus t12-g-graph-proposal      [--json] [--out PATH]
dsfb-corpus t12-g-graph-proposal-emit [--out-dir DIR]
```

Bulk artifacts:
[out/t12_g_graph_proposal_v1.txt](crates/dsfb-gpu-atlas-corpus/out/t12_g_graph_proposal_v1.txt),
[out/t12_g_graph_proposal_v1.json](crates/dsfb-gpu-atlas-corpus/out/t12_g_graph_proposal_v1.json).

**Plan-locked non-claims** — T.12.g does NOT: mutate any
upstream hash; create `corpus_hash_v2`; duplicate Fanout
cascade; promote weighted-degree spike / k-hop fanout /
directed motif-count to `new_canonical_records`; promote
community boundary shift or random-walk embedding anomaly to
`new_canonical_records`; claim a graph metric as a detector
without declared baseline + update law + metric law +
decision law; activate new detectors; execute on GPU; claim
empirical usefulness (T.8 ledger stays `NotScored`).

Receipts:
[reports/t12_g_graph_proposal_summary.txt](reports/t12_g_graph_proposal_summary.txt),
[reports/t12_g_graph_proposal_verification.txt](reports/t12_g_graph_proposal_verification.txt),
[reports/t12_g_regression_check.txt](reports/t12_g_regression_check.txt).

Next campaign: **T.12.h — data quality / tabular / database
integrity constraints** (missingness spike, missingness
coupling, null-run anomaly, cardinality drift, category
emergence / collapse, uniqueness violation, functional-
dependency violation, range envelope exit, schema drift, type
instability).

## T.12.h — Data Quality / Tabular / Database Integrity (validation-rule discipline)

The eighth real corpus expansion proposal through the T.12.0
amendment court. **Plan-locked thesis**:

> **T.12.h files the Data Quality / Tabular / Database
> Integrity amendment proposal. It admits only deterministic
> table, schema, integrity, and column-structure witnesses
> whose scope, baseline, null semantics, cardinality law,
> dependency law, type law, range law, and decision functional
> are declared; resolves SEED collisions; classifies variants
> as parameterizations or domain transfers; rejects
> underspecified leakage or learned data-quality claims; and
> preserves the frozen T.10 corpus hash.**

Main plan warning: *"A validation rule is not a detector
until scope, baseline, null / type / key semantics, and
decision law are declared."*

SEED walk found **five** T.12.h-relevant primitives already
canonical (Missingness spike 13, Missingness coupling 44,
Schema drift 45, Cardinality drift 46, Uniqueness violation
47); all five become `ExistingCanonicalAuthorityResolution`
records with declared scope + baseline + null-semantics +
key-scope contracts.

**Body counts**:

| Category | Count |
|---|---|
| Proposed primitives (8 canonical + 3 parameterizations + 2 rejection shells) | 13 |
| Proposed aliases | 0 |
| Proposed dedup-court records | 19 |
| Proposed genealogy edges | 10 |
| Proposed source refs | 9 |
| `new_canonical_records` in the delta | 8 |

**Five court-delta categories** (all exercised; second T.12.x
proposal with two `RejectedNotDeterministic` records,
following T.12.g):

- `CanonicalAddition` (×8) with declared scope + decision
  laws: FD violation (5801), type instability (5802), target-
  leakage candidate (5803; plan-locked non-claim: candidate,
  not proof), correlation break (5804), covariance shift
  (5805; generalizes 5804), null-run anomaly (5806), tabular
  range envelope exit (5807; distinct from SEED 22 residual-
  magnitude envelope), category emergence (5808; distinct from
  SEED 46 cardinality drift — tracks IDENTITY of new
  categories).
- `ExistingCanonicalAuthorityResolution` (×5): SEED 13, 44,
  45, 46, 47 with declared contracts.
- `DomainTransferOf` (×1): Missingness spike (SEED 13) as
  shared data-quality ancestor.
- `ParameterizationOf` (×3): per-column missingness (5809),
  composite-key uniqueness (5810), category collapse (5811).
- `RejectedNotDeterministic` (×2): learned data-quality
  anomaly score (5812; autoencoder / Mahalanobis-with-learned-
  cov / Isolation Forest / LOF — admission requires model-
  identification seed + training-data anchor + feature schema
  + tie-break + numeric mode); auto-schema inference anomaly
  (5813; TFDV / Great Expectations profiler with random
  sampling — admission requires inference algorithm + sample
  seed + sampling schedule + schema-version anchor + tie-break).

**Plan-locked target-leakage non-claim** (load-bearing):
target-leakage candidate (5803)'s reason text carries the
exact phrase "candidate, not proof" so a future activation
planner / case-file emitter does NOT promote candidate
signals into ratified leakage verdicts. Pinned by
[`t12_h_rejects_target_leakage_without_target_and_time_availability_law`](crates/dsfb-gpu-atlas-corpus/tests/t12_h_dataquality_invariants.rs).

**Sample emitted hashes**:

```
literature_expansion_batch_hash_v1 : 6e77d173bc12f3e6147297e5981800cb7c6883ada039d3ea40df23373fcde0ae
dedup_court_delta_hash_v1          : 568719a54af6a7e93d9bc381328eb929a43ab90324d70f1e20cad5c3f85f9e70
corpus_amendment_proposal_hash_v1  : d4566a74b0816d4e1bd4612fb7ee677ff5ae9a679af44b1bcdacab670f9779da
```

Distinct from every prior T.12.x proposal hash.

**11 plan-required load-bearing negatives** (in
[tests/t12_h_dataquality_invariants.rs](crates/dsfb-gpu-atlas-corpus/tests/t12_h_dataquality_invariants.rs)):
SEED non-mutation; missingness-without-null-semantics;
cardinality-drift-without-category-identity-law; uniqueness-
without-key-scope; FD-without-determinant-and-dependent;
schema-drift-without-schema-version-or-column-identity; range-
envelope-without-unit-and-boundary; type-instability-without-
type-system; target-leakage-without-target-and-time-
availability (MOST IMPORTANT — includes the "candidate, not
proof" plan-locked non-claim check); hash-changes-when-null-
semantics-or-fd-law-changes; SEED-collision-requires-authority-
resolution. Plus 2 rejection contract tests + 4 per-canonical
contract tests + 3 ParameterizationOf family tests + shape /
determinism / rendering invariants. **Total: 41 acceptance
tests.**

**CLI**:

```
dsfb-corpus t12-h-dataquality-proposal      [--json] [--out PATH]
dsfb-corpus t12-h-dataquality-proposal-emit [--out-dir DIR]
```

Bulk artifacts:
[out/t12_h_dataquality_proposal_v1.txt](crates/dsfb-gpu-atlas-corpus/out/t12_h_dataquality_proposal_v1.txt),
[out/t12_h_dataquality_proposal_v1.json](crates/dsfb-gpu-atlas-corpus/out/t12_h_dataquality_proposal_v1.json).

**Plan-locked non-claims** — T.12.h does NOT: mutate any
upstream hash; create `corpus_hash_v2`; duplicate Missingness
spike / coupling / Schema drift / Cardinality drift /
Uniqueness violation; promote per-column missingness /
composite-key uniqueness / category collapse / learned-DQ-
score / auto-schema-inference to `new_canonical_records`;
promote target-leakage candidate to a ratified leakage verdict
(candidate, not proof); claim a validation rule as a detector
without declared scope + baseline + null/type/key semantics +
decision law; activate new detectors; execute on GPU; claim
empirical usefulness (T.8 ledger stays `NotScored`).

Receipts:
[reports/t12_h_dataquality_proposal_summary.txt](reports/t12_h_dataquality_proposal_summary.txt),
[reports/t12_h_dataquality_proposal_verification.txt](reports/t12_h_dataquality_proposal_verification.txt),
[reports/t12_h_regression_check.txt](reports/t12_h_regression_check.txt).

## T.12.i — Observability / Debugging (telemetry-and-confuser-law discipline)

The ninth real corpus expansion proposal through the T.12.0
amendment court AND the third T.12.x with two
`RejectedNotDeterministic` records in one commit (following
T.12.g and T.12.h). **Plan-locked thesis**:

> **T.12.i files the Observability / Debugging amendment
> proposal. It admits only deterministic software-observability
> witnesses whose trace / span / log / metric field, aggregation
> window, topology scope, baseline, decision law, and confuser
> semantics are declared; resolves collisions with the existing
> DSFB-GPU-Debug bank surface; classifies deployment / runtime
> variants as parameterizations or domain transfers; rejects
> learned APM anomaly scores or underspecified vendor
> heuristics; and preserves the frozen T.10 corpus hash.**

Main plan warning: *"An observability symptom is not a
detector until the telemetry field, aggregation law, baseline,
topology scope, and confuser semantics are declared."*

**SEED collision walk** found FIVE T.12.i-relevant primitives
already canonical — exactly the dsfb-gpu-debug-core L6 bank
surface that motivated DSFB-GPU-Debug in the first place:

- Latency ramp (SEED 14)
- Single-window spike confuser (SEED 15)
- Error burst (SEED 41)
- Slew shock (SEED 42)
- Fanout cascade (SEED 43)

All five become `ExistingCanonicalAuthorityResolution` records
under the `ObservabilityDebugging` source class. **Re-adding any
of these as new canonicals would inflate the corpus AND erase
the L6 honesty marker; the court refuses** (pinned by the
parametric collision-rule test).

**Eight new canonicals** at reserved ids 5901..=5908:

- 5901 Retry storm (retry-event field + counting law + window
  + scope + threshold + confuser profile)
- 5902 Queue-depth pressure (queue-depth metric source +
  capacity contract + aggregation law)
- 5903 Saturation precursor (USE method: resource capacity +
  utilisation aggregation + slope/threshold law + must
  distinguish utilisation from saturation from error)
- 5904 Cold-start transient (deployment/warm-up marker +
  warmup window + suppression law)
- 5905 Timeout burst (timeout-event field — distinct from
  Error burst's general error-event class)
- 5906 GC pause spike (language runtime + GC pause-duration
  metric + quantile/max aggregation)
- 5907 Thread-pool exhaustion (pool source + pool-capacity
  contract; `DerivedFrom(Saturation precursor)`)
- 5908 Backpressure propagation (producer-consumer scope +
  flow-control signal + propagation law; `DerivedFrom(Fanout
  cascade)`)

**Two `DomainTransferOf`** records: Fanout cascade (SEED 43)
and Error burst (SEED 41) become shared ancestors for
`ObservabilityDebugging` (Fanout cascade is the same
primitive recognised under T.12.g GraphAnomalyDetection;
Error burst is the shared rate-burst ancestor for service
telemetry).

**Four `ParameterizationOf`** records:

- 5909 HTTP 5xx burst → `ParameterizationOf(Error burst, SEED 41)`
- 5910 p95/p99 latency ramp → `ParameterizationOf(Latency ramp,
  SEED 14)`
- 5911 k-hop dependency fanout → `ParameterizationOf(Fanout
  cascade, SEED 43)`
- 5912 Retry-rate burst → `ParameterizationOf(Retry storm, 5901)`

**Two `RejectedNotDeterministic`** records (third T.12.x with
two rejections in one commit, following T.12.g and T.12.h):

- 5913 Vendor APM black-box anomaly score (Datadog anomaly
  detection, New Relic AI-applied intelligence, Dynatrace
  Davis, Splunk MLTK, AWS DevOps Guru) — admission requires a
  deterministic formula + model-identification anchor +
  training-data anchor + feature schema + tie-break + numeric
  mode all brutally explicit.
- 5914 Learned incident classifier (PagerDuty intelligent
  triage, Splunk On-Call ML classifiers, ServiceNow AIOps) —
  admission requires model-identification seed + training-data
  anchor + label schema + tie-break + numeric mode declared.

**Vendor-APM non-claim (most-important load-bearing
negative)**: vendor APM products expose "anomaly scores"
without stable public decision functionals. The court does NOT
launder those as deterministic witnesses. The reason text must
require a "deterministic formula" AND name at least one vendor
explicitly — both pinned by
`t12_i_rejects_vendor_apm_score_without_deterministic_formula`.

**Eleven plan-required load-bearing negatives** plus rejection
contract tests, per-canonical contract assertions,
ParameterizationOf family tests, DomainTransferOf assertion
tests, bank-surface coverage witness, and reserved-id-range
tests for all three buckets total **45 acceptance tests**.

Sample hashes:

- `corpus_amendment_proposal_hash_v1 = 9776c1b9...`
- `literature_expansion_batch_hash_v1 = 115aa30f...`
- `dedup_court_delta_hash_v1 = b3c80c74...`

All three are distinct from every prior T.12.x proposal hash.
SEED stays at 54. `corpus_hash_v1` and every prior T.11 /
S1.3 / T.12.x hash byte-identical.

Receipts:
[reports/t12_i_observability_proposal_summary.txt](reports/t12_i_observability_proposal_summary.txt),
[reports/t12_i_observability_proposal_verification.txt](reports/t12_i_observability_proposal_verification.txt),
[reports/t12_i_regression_check.txt](reports/t12_i_regression_check.txt).

## T.12.j — Medical / Biosignal (signal-witness-not-diagnosis discipline)

The tenth real corpus expansion proposal through the T.12.0
amendment court AND the fourth T.12.x with two
`RejectedNotDeterministic` records in one commit (following
T.12.g, T.12.h, T.12.i). **Plan-locked thesis**:

> **T.12.j files the Medical / Biosignal amendment proposal.
> It admits only deterministic biosignal witnesses whose
> signal source, sampling law, filtering law, morphology
> measurement law, baseline / noise handling, artifact
> confuser profile, and decision functional are declared;
> resolves SEED collisions; classifies measurement variants
> as parameterizations or domain transfers; rejects
> diagnostic classifiers and learned arrhythmia scores
> unless deterministically reduced; and preserves the frozen
> T.10 corpus hash.**

Main plan warning: *"Count signal witnesses, not diagnoses.
No sampling / filtering / morphology law, no canonical
admission."*

**Plan-locked non-claim (MUST appear in every artifact)**:

> T.12.j does not admit medical diagnoses. It admits
> deterministic biosignal witnesses: morphology, interval,
> artifact, and spectral signal structures under declared
> sampling, filtering, and measurement laws. Clinical
> interpretation remains out of scope.

**SEED collision walk** found FOUR T.12.j-relevant primitives
already canonical:

- R-peak interval anomaly (RR-interval) (SEED 49)
- HRV time-domain shift (SEED 50)
- QRS width anomaly (SEED 51)
- ST-segment deviation proxy (SEED 52)

All four become `ExistingCanonicalAuthorityResolution` records
with declared signal-source + sampling-rate + filtering-law +
morphology-or-interval-measurement-law + baseline + artifact-
confuser-profile contracts.

**Eight new canonicals** at reserved ids 6001..=6008:

- 6001 P-wave morphology anomaly (lead/channel + sampling +
  filtering + P-wave fiducial-detection + amplitude/duration/
  polarity)
- 6002 T-wave morphology anomaly (T-wave fiducial-detection +
  amplitude/duration/polarity/inversion)
- 6003 QT interval anomaly (Q-onset to T-offset; optional
  Bazett / Fridericia / Framingham / Hodges rate correction)
- 6004 PR interval anomaly (P-onset to R-onset)
- 6005 Spectral HRV band shift (RR-interval extraction +
  resampling cubic-spline 4 Hz / Welch / Lomb-Scargle +
  spectral-estimation + VLF / LF / HF band definitions)
- 6006 Baseline wander detector (high-pass filter cutoff +
  below-0.5-Hz wander-band law; ECG / PPG / EMG)
- 6007 Motion artifact detector (accelerometer-corroborated /
  amplitude-saturation / baseline-jump definitions + sensor
  source + confuser handling)
- 6008 Saturation / clipping detector (ADC bit-depth or
  saturation boundary + consecutive-samples-at-boundary
  threshold)

**Two `DomainTransferOf`** records: FFT band-energy anomaly
(SEED 12) as the shared spectral ancestor for
`MedicalBiosignal` (spectral HRV band shift 6005 is the
biosignal descendant); Residual envelope exit (SEED 22) as the
shared envelope-boundary ancestor (saturation/clipping 6008
and motion artifact 6007 inherit the exit-the-envelope
semantic).

**Four `ParameterizationOf`** records:

- 6009 RR-interval irregularity → `ParameterizationOf(R-peak
  interval anomaly, SEED 49)`
- 6010 HRV SDNN / RMSSD / pNN50 → `ParameterizationOf(HRV
  time-domain shift, SEED 50)` (Task Force 1996 statistics)
- 6011 HRV LF / HF band-specific → `ParameterizationOf
  (Spectral HRV band shift, 6005)`
- 6012 Lead-specific ST deviation → `ParameterizationOf
  (ST-segment deviation proxy, SEED 52)` (anterior / inferior
  / lateral lead groups)

**Two `RejectedNotDeterministic`** records (fourth T.12.x with
two rejections in one commit):

- 6013 Learned arrhythmia classifier (Hannun et al.\ 2019
  deep-learning ECG classifier; commercial deep-learning
  rhythm detectors) — admission requires model-identification
  seed + training-data anchor (pinned PhysioNet record-hash) +
  label schema + tie-break + numeric mode declared. The court
  does NOT issue diagnostic verdicts.
- 6014 Clinician-label-only diagnostic rule — depends on
  clinical-labeller-specific judgement; admission requires
  deterministic signal-based reduction (morphology + interval
  + rhythm law declared).

**Diagnostic-claim discipline (MOST IMPORTANT)**:
`t12_j_rejects_diagnostic_claim_language` is a parametric
scanner over every CanonicalAddition AND
ExistingCanonicalAuthorityResolution reason text. For each
qualifying record it asserts the reason text contains none of
[arrhythmia, fibrillation, infarction, ischemia, ischaemia,
tachycardia, bradycardia, "diagnoses ", "diagnostic verdict"]
AND ends with "signal witness, not a medical diagnosis".
Diagnostic terms may appear ONLY inside
RejectedNotDeterministic reason text (where they describe what
is NOT admitted) or in the rejection-shell display name.

**Thirteen plan-required load-bearing negatives** plus 1
clinician-label-only rejection contract test, per-canonical
contract assertions, 4 ParameterizationOf family tests, 2
DomainTransferOf assertion tests, bank-surface coverage
witness, reserved-id-range tests for all three buckets, shape
/ determinism / rendering invariants total **48 acceptance
tests**.

**Source-ref honesty (interim)**: medical references should
carry DOIs / PMIDs / URLs as first-class fields. T.12.j embeds
DOIs in the `venue` string (Pan-Tompkins 1985 carries
`doi:10.1109/TBME.1985.325532`) as an interim solution; the
formal `doi_or_url` field lands in the plan-flagged
`T.12.schema-v2-source-ref` schema upgrade (see plan section
B.4 of "Plan-audit deferred items").

Sample hashes:

- `corpus_amendment_proposal_hash_v1 = e64b603b...`
- `literature_expansion_batch_hash_v1 = 4f7fb0b4...`
- `dedup_court_delta_hash_v1 = 12069c59...`

All three are distinct from every prior T.12.x proposal hash.
SEED stays at 54. `corpus_hash_v1` and every prior T.11 /
S1.3 / T.12.x hash byte-identical.

Receipts:
[reports/t12_j_biosignal_proposal_summary.txt](reports/t12_j_biosignal_proposal_summary.txt),
[reports/t12_j_biosignal_proposal_verification.txt](reports/t12_j_biosignal_proposal_verification.txt),
[reports/t12_j_regression_check.txt](reports/t12_j_regression_check.txt).

## T.12.k — Industrial / FDD / Condition Monitoring (plant-or-residual contract + root-cause non-claim discipline)

The eleventh real corpus expansion proposal through the T.12.0
amendment court AND the fifth T.12.x with two
`RejectedNotDeterministic` records in one commit (following
T.12.g / T.12.h / T.12.i / T.12.j). **Plan-locked thesis**:

> **T.12.k files the Industrial / FDD / Condition Monitoring
> amendment proposal. It admits only deterministic condition-
> monitoring / FDD witnesses whose plant or sensor model,
> physical quantity, unit law, sampling law, operating regime,
> baseline / nominal envelope, residual definition, fault-
> signature decision law, and confuser / nuisance-process
> profile are declared; resolves SEED collisions; classifies
> variants as parameterizations or domain transfers; rejects
> proprietary PdM black-box scores and learned fault
> classifiers without training-artifact anchors; and preserves
> the frozen T.10 corpus hash.**

Main plan warning: *"An industrial fault witness is not a
diagnosis of machine cause unless the plant model, residual
law, sensor law, operating regime, and confuser profile are
declared."*

**Plan-locked non-claim (MUST appear in every artifact)**:

> T.12.k admits deterministic condition-monitoring / FDD
> witnesses, not root-cause certainty and not maintenance
> recommendations.

**SEED collision walk** found EIGHT T.12.k-relevant primitives
already canonical — the **largest SEED-collision set of any
T.12.x to date**:

- FFT band-energy anomaly (SEED 12)
- PCA T² on score vector (SEED 19)
- PCA SPE / Q residual (SEED 20)
- PLS residual / Q on PLS (SEED 21)
- Residual envelope exit (SEED 22)
- Sensor bias detector (SEED 23)
- Actuator stiction detector (SEED 24)
- Valve hunting (control-loop oscillation) (SEED 25)

All eight become `ExistingCanonicalAuthorityResolution` records.
**Plan-locked success-shape**: the campaign's strength comes
from cross-class dedup discipline (8 authority resolutions),
not detector count. Plan-suggested 8 new canonicals collapsed
to **6** via SEED-walk-first discipline.

**Six new canonicals** at reserved ids 6101..=6106
(structurally distinct decision functionals that survived
SEED-walk):

- 6101 Kalman innovation whiteness witness (Mehra & Peschon
  1971; autocorrelation-of-innovations whiteness — NOT
  magnitude, distinct from T.12.f 5609)
- 6102 Operating-regime transition witness (process state-
  machine baseline switch — no existing ancestor)
- 6103 Condition-indicator drift (derived CI rate-of-change —
  distinct from SEED 23 raw-sensor bias)
- 6104 Fault signature angle (angular direction in PCA score
  space — distinct from SEED 19 T² and SEED 20 SPE magnitudes)
- 6105 Contribution-plot spike (per-variable contribution
  series — distinct from aggregate T² / SPE scalars)
- 6106 Spectral kurtosis (Antoni 2006; fourth-moment shape —
  distinct from SEED 12 FFT band-energy magnitude)

**Two `DomainTransferOf`** records: SEED 12 FFT band-energy as
shared spectral ancestor for `FaultDetectionDiagnostics`
(bearing vibration 6107, motor current signature 6108, and
spectral kurtosis 6106 are descendants); SEED 22 Residual
envelope exit as shared envelope-boundary ancestor for
`FaultDetectionDiagnostics` (temperature envelope excursion
6109 is the descendant).

**Four `ParameterizationOf`** records (plan-candidate
canonicals that collapsed on closer inspection — the strength
of T.12.k):

- 6107 Bearing vibration band-energy → `ParameterizationOf
  (FFT band-energy, SEED 12)` with BPFO / BPFI / BSF / FTF
  defect-frequency parameterization per McFadden & Smith 1984
- 6108 Motor current signature anomaly (MCSA) →
  `ParameterizationOf(FFT band-energy, SEED 12)` with motor-
  current spectral parameterization per Thomson 2001
- 6109 Temperature envelope excursion → `ParameterizationOf
  (Residual envelope exit, SEED 22)` with thermal-time-
  constant parameterization
- 6110 Pressure transient witness → `ParameterizationOf(Slew
  shock, SEED 42)` with pressure-physics parameterization

**Two `RejectedNotDeterministic`** records (fifth T.12.x with
two rejections in one commit):

- 6111 Proprietary PdM black-box score (GE Predix, Siemens
  MindSphere, IBM Maximo Predict, Honeywell Forge, Aspen
  Mtell) — admission requires deterministic formula + model-
  identification anchor + training-data anchor + feature
  schema + tie-break + numeric mode declared.
- 6112 Learned fault classifier (Wen et al. 2017 CNN bearing
  classifier; Khan & Yairi 2018 deep-learning fault classifier
  review) — admission requires model-identification seed +
  training-data anchor (pinned dataset record-hash, e.g. CWRU
  bearing) + label schema + tie-break + numeric mode declared.

**Plant-or-residual contract discipline (MOST IMPORTANT load-
bearing negative)**: `t12_k_rejects_fault_detector_without_plant_or_residual_contract`
asserts every CanonicalAddition reason declares at least one
math-structure term (plant / observer / residual / model /
state-machine / latent-space / estimator / envelope /
computation) AND at least one decision-functional term
(decision law / decision functional / decision predicate). The
campaign's identity: refuse fault witnesses without declared
mathematical structure and decision rule.

**Root-cause-claim discipline (MOST IMPORTANT)**:
`t12_k_rejects_root_cause_claim_language` is a parametric
scanner over every CanonicalAddition AND
ExistingCanonicalAuthorityResolution reason text. It blacklists
[root cause, diagnosis of machine cause, remaining useful life,
predicted rul, failure mode classification] AND requires every
qualifying reason to end with "condition-monitoring witness,
not a maintenance recommendation". Forbidden terms appear ONLY
in `RejectedNotDeterministic` reason text. "Maintenance
recommendation" is NOT blacklisted because the non-claim
phrase itself uses it as a disclaimer.

**Sixteen plan-required load-bearing negatives** plus per-SEED
duplicate-rejection tests (one per SEED 12 / 19 / 20 / 21 /
22 / 23 / 24 / 25), per-canonical contract assertions, 4
ParameterizationOf family tests, 2 DomainTransferOf tests,
authority-coverage-of-all-SEED-industrial-ids witness,
reserved-id-range tests for all three buckets (6101..=6106,
6107..=6110, 6111..=6112), shape / determinism / rendering
invariants total **49 acceptance tests**.

Sample hashes:

- `corpus_amendment_proposal_hash_v1 = 22f06c76...`
- `literature_expansion_batch_hash_v1 = 5c952a60...`
- `dedup_court_delta_hash_v1 = 3c32085d...`

All three are distinct from every prior T.12.x proposal hash.
SEED stays at 54. `corpus_hash_v1` and every prior T.11 /
S1.3 / T.12.x hash byte-identical.

Receipts:
[reports/t12_k_industrial_proposal_summary.txt](reports/t12_k_industrial_proposal_summary.txt),
[reports/t12_k_industrial_proposal_verification.txt](reports/t12_k_industrial_proposal_verification.txt),
[reports/t12_k_regression_check.txt](reports/t12_k_regression_check.txt).

## T.12.l — Chemometrics (preprocessing + latent-space + calibration + material-identification non-claim)

T.12.l files the **twelfth real corpus expansion proposal**
through the T.12.0 amendment court: Chemometrics. The
proposal does NOT mutate SEED; it is a docketed amendment
proposing 5 new canonical chemometric primitives plus 4
existing-canonical authority resolutions for the latent-space
+ envelope SEED family + 2 domain transfers + 4
parameterizations + 2 rejections.

Plan-locked thesis:

> **T.12.l files the Chemometrics amendment proposal. It
> admits only deterministic chemometric witnesses whose
> chemometric model class, preprocessing law, latent-space
> contract, calibration contract, residual definition,
> decision functional, and confuser / nuisance profile are
> declared.**

Plan-locked non-claim (verbatim in receipt + paper):

> T.12.l admits deterministic chemometric residual / latent-
> space / calibration / concentration-structure witnesses. It
> does not admit chemical causation, material identification
> certainty, regulatory compliance, lab diagnosis, or
> process-control authority.

**SEED collision walk** found four T.12.l-relevant primitives
already canonical:

- 19 PCA T² on score vector (latent-space ancestor)
- 20 PCA SPE / Q residual
- 21 PLS residual / Q on PLS
- 22 Residual envelope exit (envelope-boundary ancestor)

All four become `ExistingCanonicalAuthorityResolution`
records.

**Court-delta** (all five plan-locked categories exercised;
sixth T.12.x with two RejectedNotDeterministic records):

- 5 `CanonicalAddition` at 6201..=6205 (calibration residual,
  leverage outlier, concentration drift, SIMCA class-distance
  per Wold & Sjostrom 1977, VIP shift per Wold 1995) with
  declared preprocessing / latent / calibration / residual /
  score / hat-matrix / per-class / VIP / model contracts AND
  decision-functional contracts;
- 4 `ExistingCanonicalAuthorityResolution` for SEED 19 / 20 /
  21 / 22;
- 2 `DomainTransferOf`: SEED 19 PCA T² as shared latent-space
  ancestor for `Chemometrics`; SEED 22 Residual envelope exit
  as shared envelope-boundary ancestor for `Chemometrics`;
- 4 `ParameterizationOf` collapsing plan-candidate primitives
  that did NOT survive SEED-walk (PCA score outlier 6206 →
  SEED 19; Mahalanobis-on-scores 6207 → SEED 19; LV control
  chart 6208 → SEED 20; spectral preprocessing artifact 6209
  → SEED 22);
- 2 `RejectedNotDeterministic` at 6210/6211: black-box
  spectroscopy classifier (Bruker AI-IDENT / Mettler-Toledo
  Spectraline / Thermo Scientific OMNIC ML / Agilent MicroLab
  AI) and adaptive-AutoML / stochastic-CV chemometric
  pipeline (auto-sklearn / H2O AutoML / TPOT).

**Material-identification-claim discipline** is enforced by
`t12_l_rejects_material_identification_claim_language`: every
CanonicalAddition AND ExistingCanonicalAuthorityResolution
reason text is scanned for forbidden terms (material
identification certainty, chemical causation, lab diagnosis,
process-control authority, identifies the material) AND must
end with the plan-locked non-claim "chemometric signal
witness, not a material identification or a regulatory
compliance verdict". **Regulatory-compliance-claim
discipline** is enforced by
`t12_l_rejects_regulatory_compliance_claim_language`:
forbidden regulatory terms (FDA approval, ISO certification,
regulatory compliance verdict) appear ONLY in the 6210/6211
rejection records. **Preprocessing-or-latent-model contract
discipline** is enforced by
`t12_l_rejects_chemometric_detector_without_preprocessing_or_latent_model_contract`:
every CanonicalAddition reason declares math-structure +
decision-functional contract.

**Sixteen plan-required load-bearing negatives** plus per-
SEED duplicate-rejection tests (one per SEED 19 / 20 / 21 /
22), per-canonical contract assertions, 4 ParameterizationOf
family tests, 2 DomainTransferOf tests, authority-coverage-
of-all-SEED-chemometric-ids witness, reserved-id-range tests
for all three buckets (6201..=6205, 6206..=6209, 6210..=6211),
shape / determinism / rendering invariants total **44
acceptance tests**.

Sample hashes:

- `corpus_amendment_proposal_hash_v1 = 8b3e9511...`
- `literature_expansion_batch_hash_v1 = c9f41165...`
- `dedup_court_delta_hash_v1 = e4226efe...`

All three are distinct from every prior T.12.x proposal hash.
SEED stays at 54. `corpus_hash_v1` and every prior T.11 /
S1.3 / T.12.x hash byte-identical.

Receipts:
[reports/t12_l_chemometrics_proposal_summary.txt](reports/t12_l_chemometrics_proposal_summary.txt),
[reports/t12_l_chemometrics_proposal_verification.txt](reports/t12_l_chemometrics_proposal_verification.txt),
[reports/t12_l_regression_check.txt](reports/t12_l_regression_check.txt).

## T.12.m — RF / Communications (signal / channel / modulation / synchronization witness with emitter-attribution + geolocation + spectrum-enforcement non-claim discipline)

T.12.m files the **thirteenth real corpus expansion proposal**
through the T.12.0 amendment court: RF / Communications. The
proposal does NOT mutate SEED; it is a docketed amendment
proposing 6 new canonical RF primitives plus 6 existing-
canonical authority resolutions for the spectral + envelope +
entropy + correlation + carrier-offset + modulation-quality
SEED family RF heavily reuses, plus 2 domain transfers, 4
parameterizations, and 2 rejections.

Plan-locked thesis:

> **T.12.m files the RF / Communications amendment proposal.
> It admits only deterministic RF / communications signal
> witnesses whose signal representation, sampling law, unit
> law, carrier / channel assumption, synchronization
> assumption, window / transform law, decision functional,
> confuser profile, and numeric mode are declared.**

Plan-locked non-claim (verbatim in receipt + paper):

> T.12.m admits deterministic RF / communications signal
> witnesses, not emitter attribution, transmitter
> identification, geolocation, spectrum-enforcement authority,
> military classification, or communications-intelligence
> conclusions.

**SEED collision walk** found six T.12.m-relevant primitives
already canonical:

- 12 FFT band-energy anomaly (shared spectral RF ancestor)
- 22 Residual envelope exit (shared envelope-boundary RF
  ancestor)
- 38 Spectral entropy
- 40 Autocorrelation break
- 53 Carrier-frequency-offset residual (Morelli & Mengali
  1999 OFDM CFO estimator)
- 54 Error Vector Magnitude (EVM) anomaly (Shafik / Rahman /
  Islam 2006 EVM-BER-SNR relations)

All six become `ExistingCanonicalAuthorityResolution`
records. Reserved canonical ids 6301 and 6302 are
deliberately UNUSED in this proposal: the CFO and EVM ideas
that once shadowed them collapsed onto SEED 53 and SEED 54
respectively under the SEED-walk-first discipline.

**Court-delta** (all five plan-locked categories exercised;
seventh T.12.x with two RejectedNotDeterministic records):

- 6 `CanonicalAddition` at 6303..=6308 (constellation
  spread (second-moment per cluster — distinct from SEED 54
  EVM), channel impulse response (CIR) drift (system response
  to declared impulse — distinct from SEED 40 autocorrelation
  break), IQ imbalance, phase-noise per Razavi 1996,
  symbol-timing offset residual per Gardner / early-late,
  cyclostationary feature shift per Gardner 1987 (with
  DECLARED cycle frequencies — distinct from SEED 40 implicit
  autocorrelation)) with declared signal representation +
  sampling + unit + carrier / channel + synchronization +
  window/transform + decision functional + confuser + numeric
  mode contracts;
- 6 `ExistingCanonicalAuthorityResolution` for SEED 12 / 22 /
  38 / 40 / 53 / 54;
- 2 `DomainTransferOf`: SEED 12 FFT band-energy as shared
  spectral ancestor for `RfCommunications`; SEED 22 Residual
  envelope exit as shared envelope-boundary ancestor;
- 4 `ParameterizationOf` collapsing plan-candidate
  primitives that did NOT survive SEED-walk (spectral mask
  violation 6309 → SEED 12 with ITU-R SM / ETSI EN / FCC
  Part 15 emission-mask law; SNR drop 6310 → SEED 12 with
  signal/noise band partition; burst preamble miss 6311 →
  SEED 40 with cross-correlation template against known
  preamble; frame-error burst 6312 → SEED 41 Error burst
  with IEEE 802.11 / IEEE 802.15.4 / 3GPP LTE / 5G NR frame
  format + CRC / FEC decode law);
- 2 `RejectedNotDeterministic` at 6313/6314: learned RF
  fingerprinting classifier (Restuccia 2019 DeepRadioID /
  Sankhe 2019 ORACLE / Wang 2022 RF-based device
  identification) and black-box modulation classifier /
  proprietary spectrum-anomaly score (Keysight signal-
  analysis ML / Rohde & Schwarz spectrum monitoring AI / NI
  RFIC analyser ML / Ettus USRP-based learned pipelines).

**Emitter-identification + geolocation + spectrum-enforcement
discipline** is enforced by THREE parametric scanners
(`t12_m_rejects_emitter_identification_claim_language`,
`t12_m_rejects_geolocation_or_attribution_claim_language`,
`t12_m_rejects_spectrum_enforcement_claim_language`): every
CanonicalAddition AND ExistingCanonicalAuthorityResolution
reason text is scanned for forbidden emitter / transmitter-
identification / geolocation / regulatory-enforcement /
SIGINT / COMINT terms AND must contain a plan-locked RF
signal-witness self-description AND a "not emitter
attribution ..." disclaimer. **Signal-or-sampling contract
discipline** is enforced by
`t12_m_rejects_rf_detector_without_signal_or_sampling_contract`:
every CanonicalAddition reason declares signal representation
+ sampling law + carrier-or-channel assumption + window-or-
transform law + decision functional.

**Eight plan-required load-bearing negatives** plus per-
SEED duplicate-rejection tests (one per SEED 12 / 22 / 38 /
40 / 53 / 54), per-canonical contract assertions, 4
ParameterizationOf family tests, 2 DomainTransferOf tests,
SEED-walk-first authority-resolution guards for CFO (asserts
reserved id 6301 stays unused; SEED 53 carries the canonical
authority with the Morelli reference) and EVM (asserts
reserved id 6302 stays unused; SEED 54 carries the canonical
authority with the Shafik reference), authority-coverage-of-
all-SEED-RF-ids witness, reserved-id-range tests for all
three buckets (6303..=6308 for CanonicalAddition with 6301
/ 6302 deliberately unused; 6309..=6312; 6313..=6314), shape
/ determinism / rendering invariants total **50 acceptance
tests**.

Sample hashes:

- `corpus_amendment_proposal_hash_v1 = 3263e87b...`
- `literature_expansion_batch_hash_v1 = 437e6f91...`
- `dedup_court_delta_hash_v1 = 669e0b6a...`

All three are distinct from every prior T.12.x proposal hash.
SEED stays at 54. `corpus_hash_v1` and every prior T.11 /
S1.3 / T.12.x hash byte-identical.

Receipts:
[reports/t12_m_rf_proposal_summary.txt](reports/t12_m_rf_proposal_summary.txt),
[reports/t12_m_rf_proposal_verification.txt](reports/t12_m_rf_proposal_verification.txt),
[reports/t12_m_regression_check.txt](reports/t12_m_regression_check.txt).

## T.12.n — Econometrics + Reliability / Survival (combined campaign; structural-break + envelope ancestry shared across both domains; market-prediction / credit-decision / RUL-certainty non-claim discipline)

T.12.n files the **fourteenth real corpus expansion proposal**
through the T.12.0 amendment court: Econometrics + Reliability
/ Survival, combined into one proposal because the two
domains share structural-break / CUSUM / envelope-residual
ancestry. The proposal does NOT mutate SEED; it is a docketed
amendment proposing 8 new canonical primitives (4 econometric
+ 4 reliability / survival) plus 4 existing-canonical
authority resolutions for the structural-change + envelope
SEED family + 2 domain transfers + 4 parameterizations + 2
rejections.

Plan-locked thesis:

> **T.12.n files the Econometrics + Reliability / Survival
> amendment proposal. It admits only deterministic
> econometric, reliability, survival, and degradation
> witnesses whose stationarity contract, window contract,
> regression / hazard model, censoring law, time-origin law,
> residual definition, decision functional, confuser profile,
> and numeric mode are declared.**

Plan-locked non-claim (verbatim in receipt + paper):

> T.12.n admits deterministic econometric, reliability,
> survival, and degradation witnesses. It does not admit
> market prediction, investment advice, credit-decision
> authority, actuarial pricing authority, causal economic
> certainty, RUL certainty, maintenance recommendations, or
> failure-time prediction.

**SEED collision walk** found four T.12.n-relevant primitives
already canonical:

- 3 CUSUM (cumulative sum) chart (shared structural-change
  ancestor for Econometrics + ReliabilitySurvival)
- 4 Page-Hinkley test (structural-break F-test
  parameterization target)
- 11 Mann-Kendall trend test (econometric / reliability trend
  ancestor)
- 22 Residual envelope exit (shared envelope-boundary
  ancestor for reliability failure-rate + hazard-rate
  envelope)

All four become `ExistingCanonicalAuthorityResolution`
records.

**Court-delta** (all five plan-locked categories exercised;
eighth T.12.x with two RejectedNotDeterministic records):

- 8 `CanonicalAddition` at 6401..=6408:
  - Econometric (4): GARCH volatility residual anomaly
    (6401; Bollerslev 1986; against conditional-variance
    model), cointegration-break detector (6402; Hansen 1992
    / Quintos-Phillips 1993; CUSUM-of-squared-residuals on
    cointegration regression), Hausman-test residual (6403;
    Hausman 1978; chi-squared on parameter-difference vector),
    Bai-Perron multiple-break detector (6404; Bai-Perron 1998
    / 2003; information-criterion + Quandt-Andrews
    supremum-F);
  - Reliability / Survival (4): Kaplan-Meier survival-
    residual (6405; Kaplan-Meier 1958 with declared
    censoring + time-origin), Cox proportional-hazards /
    Schoenfeld residual (6406; Cox 1972 / Schoenfeld 1982 /
    Grambsch-Therneau 1994), Weibull failure-rate envelope
    exit (6407; Weibull 1951 with declared shape + scale +
    MLE), Crack-growth law residual (6408; Paris-Erdogan
    1963 with stress-intensity-range model + C / m
    parameters);
- 4 `ExistingCanonicalAuthorityResolution`: SEED 3 / 4 / 11
  / 22;
- 2 `DomainTransferOf`: SEED 3 CUSUM as shared structural-
  change ancestor for Econometrics + ReliabilitySurvival;
  SEED 22 Residual envelope exit as shared envelope-boundary
  ancestor for ReliabilitySurvival;
- 4 `ParameterizationOf` collapsing plan-candidate
  primitives that did NOT survive SEED-walk (CUSUM-of-
  recursive-residuals 6409 per Brown-Durbin-Evans 1975 →
  SEED 3; Quandt-Andrews / Chow structural-break F-test 6410
  per Quandt 1960 / Chow 1960 / Andrews 1993 → SEED 4;
  hazard-rate change 6411 → SEED 22; cumulative damage
  residual 6412 per Palmgren 1924 / Miner 1945 → SEED 3);
- 2 `RejectedNotDeterministic` at 6413/6414: learned market
  predictor / black-box financial forecaster (Bloomberg AIM
  / AlphaSense / Kavout / Goldman SecDB ML / JP Morgan COIN
  / LOXM) and learned RUL classifier / black-box predictive-
  maintenance score (Uptake AI / C3.ai / Senseye / IBM
  Maximo RUL / Siemens MindSphere Asset Analytics).

**Market-prediction / investment-or-credit-decision / RUL-or-
failure-time-certainty discipline** is enforced by THREE
parametric scanners
(`t12_n_rejects_market_prediction_claim_language`,
`t12_n_rejects_investment_or_credit_decision_claim_language`,
`t12_n_rejects_rul_or_failure_time_certainty_claim_language`):
every CanonicalAddition AND
ExistingCanonicalAuthorityResolution reason text is scanned
for forbidden market-prediction / credit-decision / RUL-
certainty / failure-time-prediction / maintenance-recommendation
terms; forbidden terms appear ONLY in
`RejectedNotDeterministic` reason text. **Contract discipline**
is enforced by two scanners
(`t12_n_rejects_econometric_witness_without_stationarity_or_window_contract`
and
`t12_n_rejects_survival_witness_without_censoring_or_time_origin_contract`):
every econometric CanonicalAddition declares stationarity +
window contract; every survival / reliability CanonicalAddition
declares censoring law + time-origin law.

**Six plan-required load-bearing negatives** plus per-SEED
duplicate-rejection tests (one per SEED 3 / 4 / 11 / 22),
per-canonical structural-distinctness assertions (GARCH-
distinct-from-level-model, cointegration-distinct-from-raw-
CUSUM, Hausman-distinct-from-residual-sequence, Bai-Perron-
multiple-breaks, KM declares Kaplan-Meier, Cox/Schoenfeld
declares proportional-hazards, Weibull declares shape +
scale, Paris-Erdogan declares stress-intensity), 4
ParameterizationOf family tests, 2 DomainTransferOf tests,
authority-coverage-of-all-T.12.n-SEED-ids witness, reserved-
id-range tests for all three buckets (6401..=6408;
6409..=6412; 6413..=6414), shape / determinism / rendering
invariants total **47 acceptance tests**.

Sample hashes:

- `corpus_amendment_proposal_hash_v1 = b1c5ea45...`
- `literature_expansion_batch_hash_v1 = ea8d7fb5...`
- `dedup_court_delta_hash_v1 = eef41dca...`

All three are distinct from every prior T.12.x proposal hash.
SEED stays at 54. `corpus_hash_v1` and every prior T.11 /
S1.3 / T.12.x hash byte-identical.

Receipts:
[reports/t12_n_econometrics_reliability_proposal_summary.txt](reports/t12_n_econometrics_reliability_proposal_summary.txt),
[reports/t12_n_econometrics_reliability_proposal_verification.txt](reports/t12_n_econometrics_reliability_proposal_verification.txt),
[reports/t12_n_regression_check.txt](reports/t12_n_regression_check.txt).

## T.12.o — Streaming Sketches (bounded-memory mergeable summaries with hash + width + depth + seed + merge-law contract discipline; probabilistic-bound-as-deterministic-certainty + approximate-query-truth + privacy + anonymization non-claim discipline)

T.12.o files the **fifteenth real corpus expansion proposal**
through the T.12.0 amendment court: Streaming Sketches. The
proposal does NOT mutate SEED; it is a docketed amendment
proposing 8 new canonical streaming-sketch primitives plus 4
existing-canonical authority resolutions for the KS +
missingness + error-burst + cardinality SEED family that
streaming-sketch summaries heavily reuse, plus 2 domain
transfers, 4 parameterizations, and 2 rejections.

Plan-locked thesis:

> **T.12.o files the Streaming Sketches amendment proposal.
> It admits only deterministic streaming-sketch witnesses:
> bounded-memory, mergeable or update-order-declared
> summaries for frequency, cardinality, quantile, heavy-
> hitter, membership, and moment / variance evidence whose
> hash family, width, depth, seed, bucket count, merge law,
> update order, error-bound semantics, residual definition,
> decision functional, confuser profile, and numeric mode
> are declared.**

Plan-locked non-claim (verbatim in receipt + paper):

> T.12.o admits deterministic streaming-sketch witnesses:
> bounded-memory, mergeable or update-order-declared
> summaries for frequency, cardinality, quantile, heavy-
> hitter, membership, and moment / variance evidence. It does
> not admit probabilistic accuracy as certainty, randomized
> sketch behavior without seed / width / depth / hash-family
> declaration, privacy claims, database correctness authority,
> or approximate-query truth.

**SEED collision walk** found four T.12.o-relevant primitives
already canonical:

- 8 Kolmogorov-Smirnov two-sample test (shared distribution-
  distance ancestor)
- 13 Missingness spike (Bloom-filter-based missingness
  inversion ancestor)
- 41 Error burst (sliding-window heavy-hitter sketches over
  error-event streams ancestor)
- 46 Cardinality drift (pre-HLL cardinality estimators
  ancestor)

All four become `ExistingCanonicalAuthorityResolution`
records.

**Court-delta** (all five plan-locked categories exercised;
ninth T.12.x with two RejectedNotDeterministic records):

- 8 `CanonicalAddition` at 6501..=6508: CMS residual per
  Cormode-Muthukrishnan 2005 (hash family + width + depth +
  seed array + min-over-d collision rule); HyperLogLog
  cardinality shift per Flajolet-Fusy-Gandouet-Meunier 2007
  (hash family + bucket count m = 2^precision + harmonic-mean
  estimator + bias correction); Bloom-filter membership
  anomaly per Bloom 1970 (hash family + bit-array size + hash
  count + seed array + false-positive-rate envelope); Misra-
  Gries heavy-hitter shift per Misra-Gries 1982 (k counter
  slots + decrement-on-miss law; deterministic, no hash);
  Space-Saving heavy-hitter shift per Metwally-Agrawal-El
  Abbadi 2005 (k counter slots + replace-smallest-on-miss
  law; structurally distinct from Misra-Gries via the
  different bookkeeping rule); Greenwald-Khanna quantile
  summary drift per Greenwald-Khanna 2001 (epsilon error
  bound + tuple-insertion + deterministic epsilon-approximate
  quantile guarantee); t-digest summary residual per Dunning
  2019 (compression delta + centroid scale function +
  DETERMINISTIC centroid-merge law); AMS moment sketch per
  Alon-Matias-Szegedy 1999 (4-wise-independent hash family +
  per-sketch seed + sketch width + moment order p);
- 4 `ExistingCanonicalAuthorityResolution` for SEED 8 / 13 /
  41 / 46;
- 2 `DomainTransferOf`: SEED 46 Cardinality drift as shared
  cardinality ancestor for StreamingSketches; SEED 8 KS as
  shared distribution-distance ancestor;
- 4 `ParameterizationOf` collapsing plan-candidate
  primitives that did NOT survive SEED-walk (Flajolet-Martin
  / pre-HLL 6509 → SEED 46; streaming-approximate KS via
  quantile sketch 6510 → SEED 8; sliding-window error-burst
  sketch 6511 → SEED 41; sketch-approximate missingness via
  Bloom inversion 6512 → SEED 13);
- 2 `RejectedNotDeterministic` at 6513/6514: learned
  streaming-anomaly score (Datadog Watchdog AI / DataRobot
  Streaming AutoML / Splunk Stream ML / AWS Lookout for
  Metrics / Azure Anomaly Detector) and black-box
  approximate-streaming proprietary sketch (Snowflake APPROX_*
  / BigQuery APPROX_* / Druid / ClickHouse uniqHLL12 /
  quantileTDigest / topK / AWS Athena APPROX_*).

**Probabilistic-bound / approximate-query-truth / privacy /
anonymization / mergeable-without-merge-law discipline** is
enforced by FOUR claim-language scanners
(`t12_o_rejects_probabilistic_error_bound_as_deterministic_certainty`,
`t12_o_rejects_approximate_query_truth_claim_language`,
`t12_o_rejects_privacy_or_anonymization_claim_language`,
`t12_o_rejects_mergeable_sketch_without_merge_law`) PLUS the
hash-family / width / depth / seed contract scanner
(`t12_o_rejects_sketch_without_hash_family_width_depth_or_seed_contract`)
PLUS the black-box-streaming-anomaly-and-vendor-sketch
contract scanner
(`t12_o_rejects_black_box_streaming_anomaly_score_without_formula`).

**Six plan-required load-bearing negatives** plus per-SEED
duplicate-rejection tests (one per SEED 8 / 13 / 41 / 46),
per-canonical structural-distinctness assertions (Space-
Saving distinct-from-Misra-Gries; HLL distinct-from-pre-HLL-
Flajolet-Martin via harmonic-mean estimator; t-digest
declares deterministic centroid-merge law; Greenwald-Khanna
declares deterministic epsilon-approximate quantile
guarantee; AMS declares 4-wise-independent hash family),
4 ParameterizationOf family tests, 2 DomainTransferOf tests,
authority-coverage-of-all-T.12.o-SEED-ids witness, reserved-
id-range tests for all three buckets (6501..=6508;
6509..=6512; 6513..=6514), shape / determinism / rendering
invariants total **44 acceptance tests**.

Sample hashes:

- `corpus_amendment_proposal_hash_v1 = 1164f567...`
- `literature_expansion_batch_hash_v1 = fa14bd13...`
- `dedup_court_delta_hash_v1 = 434a174c...`

All three are distinct from every prior T.12.x proposal hash.
SEED stays at 54. `corpus_hash_v1` and every prior T.11 /
S1.3 / T.12.x hash byte-identical.

Receipts:
[reports/t12_o_streaming_sketches_proposal_summary.txt](reports/t12_o_streaming_sketches_proposal_summary.txt),
[reports/t12_o_streaming_sketches_proposal_verification.txt](reports/t12_o_streaming_sketches_proposal_verification.txt),
[reports/t12_o_regression_check.txt](reports/t12_o_regression_check.txt).

Next campaign: **T.12.p — Information Theory catch-up**
(reserved id band 6601..=6699; landed at `3e79f7a`+1 — see the
T.12.p section below).

## T.12.p — Information Theory catch-up (entropy / divergence / mutual information / coding-length with estimator + binning + smoothing + joint-distribution + log-base + empty-bin contract discipline; causal-information-flow + privacy / security + learned-representation non-claim discipline)

T.12.p files the **sixteenth real corpus expansion proposal**
through the T.12.0 amendment court. Plan-locked thesis:

> **T.12.p files the Information Theory catch-up amendment
> proposal. It admits only deterministic information-theoretic
> witnesses: entropy, conditional entropy, mutual information,
> cross-entropy / negative-log-likelihood, and minimum
> description length / coding-length residuals whose estimator,
> binning, smoothing, sample-support, joint-distribution
> contract, log base, empty-bin law, and numeric mode are
> declared; resolves SEED collisions with KL divergence,
> Jensen-Shannon divergence, and Spectral entropy; classifies
> variants as parameterizations or domain transfers; rejects
> learned mutual-information estimators and black-box
> information-theoretic anomaly scores without declared
> deterministic-binning / kernel / partition / formula contract;
> and preserves the frozen T.10 corpus hash.**

Plan-locked non-claim (MUST appear verbatim):

> T.12.p admits deterministic information-theoretic witnesses:
> entropy, divergence, mutual-information, coding-length,
> compression, surprise, and dependence-structure evidence with
> declared estimator, binning, smoothing, sample-support, and
> numeric laws. It does not admit semantic meaning, causal
> information flow certainty, privacy leakage certainty,
> cryptographic security claims, or learned representation
> claims.

**SEED collision walk** found three T.12.p-relevant primitives
already canonical: Kullback-Leibler divergence (SEED 9,
foundational information-theoretic divergence ancestor), Jensen-
Shannon divergence (SEED 32, symmetric bounded JS variant per
Lin 1991), and Spectral entropy (SEED 38, Shannon entropy on
the normalised power spectrum per Inouye 1991). All three
become `ExistingCanonicalAuthorityResolution` records under the
`InformationTheory` source class. Plan-locked success-shape
applied: cross-class dedup discipline (3 authority resolutions
over the KL + JS + Spectral-entropy SEED family that
information-theoretic witnesses heavily reuse).

**Five new canonicals at 6601..=6605** survived the SEED-walk
as structurally distinct information-theoretic decision
functionals:
- **6601 Shannon entropy shift witness** (Shannon 1948 A
  Mathematical Theory of Communication) — declared log base,
  binning or partition law (equal-width / equal-frequency /
  Freedman-Diaconis / declared partition function), empty-bin
  law (skip / Laplace smoothing alpha / Krichevsky-Trofimov
  1/2), smoothing rule, sample-support bound, estimator
  (plug-in / Miller-Madow / James-Stein / declared).
- **6602 Conditional entropy shift witness** (Cover-Thomas
  2006 chapter 2) — declared joint-distribution contract over
  (X, Y), joint binning, binning law for both marginals AND
  the joint, empty-bin law, smoothing, sample-support bound,
  log base. Per-window H(Y|X) = H(X,Y) - H(X) residual.
- **6603 Mutual information break witness** (Cover-Thomas
  2006 chapter 2) — declared joint-distribution contract,
  binning OR kernel-density-estimator law, bias-correction
  rule (Miller-Madow / James-Stein / none), log base. Per-
  window I(X; Y) = H(X) + H(Y) - H(X, Y) break vs baseline.
  Structurally distinct from SEED 9 KL because MI is a
  functional on the JOINT vs PRODUCT-OF-MARGINALS, whereas
  KL is a divergence between two declared distributions; MI
  is symmetric and non-directional by construction.
- **6604 Cross-entropy / negative-log-likelihood residual
  witness** (Shannon 1948 / Cover-Thomas 2006) — declared
  FIXED MODEL distribution q (parameter-pinned; frozen across
  the comparison window; no learned parameters at decision
  time), empirical sample distribution p (declared estimator),
  log base, smoothing (epsilon for log(0)), empty-bin law,
  sample-support bound.
- **6605 Minimum description length / coding-length residual
  witness** (Rissanen 1978 / Rissanen 1986) — declared model
  class (fixed prefix code / fixed universal code / two-part
  code with declared parameter-cost law), code-length
  functional L(D | M), L(M) parameter-encoding cost (two-part
  code; model-cost is not silently dropped), sample-support
  bound, numeric mode.

Plus **two `DomainTransferOf` records** (SEED 9 KL as shared
information-theoretic divergence ancestor for `InformationTheory`;
SEED 38 Spectral entropy as shared Shannon-entropy-on-
distribution ancestor), **four `ParameterizationOf` records at
6606..=6609** (Normalized MI → MI 6603; Transfer entropy proxy
per Schreiber 2000 → MI 6603 — ADMITTED ONLY AS A DETERMINISTIC
NON-CAUSAL WITNESS; Rényi-Tsallis entropy per Rényi 1961 /
Tsallis 1988 → Shannon entropy 6601 with declared order-alpha
parameter law and limit-recovery; Compression-ratio anomaly per
Ziv-Lempel 1977 / Ziv-Lempel 1978 / Welch 1984 LZW → MDL 6605,
with the court explicitly NOT admitting compression as a
surrogate for true description length), and **two
`RejectedNotDeterministic` records at 6610..=6611** (learned
mutual-information estimator: MINE Belghazi et al. 2018, InfoMax
/ variational MI bounds, neural KL estimators, InfoVAE, CPC
contrastive predictive coding MI lower bounds; black-box
information-theoretic anomaly score: AWS Macie information-
leakage scoring, IBM Guardium DAM information-theoretic anomaly
heuristics, Microsoft Purview information-leakage classifier,
Symantec / Broadcom DLP entropy-based anomaly score, Cisco
Talos information-theoretic threat scoring). T.12.p is the
**tenth T.12.x with two RejectedNotDeterministic records** in
one commit, following T.12.g / h / i / j / k / l / m / n / o.

45-test suite with 6 plan-required load-bearing negatives:
- `t12_p_rejects_information_witness_without_estimator_or_binning_contract`,
- `t12_p_rejects_entropy_detector_without_base_smoothing_and_empty_bin_law`,
- `t12_p_rejects_mutual_information_without_joint_distribution_contract`,
- `t12_p_rejects_causal_information_flow_claim_language`,
- `t12_p_rejects_privacy_or_security_claim_language`,
- `t12_p_rejects_learned_embedding_information_score_without_formula`.

Plus per-canonical structural-distinctness assertions (MI
distinct from KL, cross-entropy pins fixed model distribution,
MDL declares two-part code, Shannon entropy declares partition-
law options, conditional entropy declares joint-minus-marginal
form), ParameterizationOf family tests, DomainTransferOf tests,
authority-coverage-of-all-T.12.p-SEED-ids witness, reserved-id-
range tests (6601..=6605 / 6606..=6609 / 6610..=6611), and
rendering / determinism / hash-distinctness invariants.

Causal-information-flow / privacy / security forbidden-term
scanners follow the same nuanced discipline as T.12.o's
anonymization-authority scanner: bare phrases ("causal
information flow", "intervention truth", "cryptographic
security") are admitted INSIDE legitimate "does NOT admit ..."
disclaimers; only positive-claim variants ("claims X" / "issues
X verdicts" / "admits X" / "guarantees X") are forbidden.

**Status**: `Open` pending future review. **Does NOT mutate**
`SEED` (still 54 records), `corpus_hash_v1` (still
`35c276c7...`), `registry_hash_v2` (still `d3cf6300...`), any
T.11/S1.3/T.12.0..T.12.o hash, any `DetectorPassport` hash, or
R.12b D64 episodes (still 13/89/1917). T.12.p
`corpus_amendment_proposal_hash_v1 =
338198ebc09cc2a867f2a38aa949b9134edbde9f1778002209f157ca7bb335ca`
distinct from every prior T.12.x.

Receipts:
[reports/t12_p_information_theory_proposal_summary.txt](reports/t12_p_information_theory_proposal_summary.txt),
[reports/t12_p_information_theory_proposal_verification.txt](reports/t12_p_information_theory_proposal_verification.txt),
[reports/t12_p_regression_check.txt](reports/t12_p_regression_check.txt).

Next campaign: **T.12.consolidate — amendment review +
`corpus_hash_v2` freeze** (landed; see the T.12.consolidate
section below).

## T.12.consolidate — amendment review + corpus_hash_v2 freeze (transition from proposal court to ratified corpus authority; META-hash freeze layer above the frozen T.10 corpus surface)

T.12.consolidate is the **transition from proposal court to
ratified corpus authority**. Plan-locked thesis:

> **T.12.consolidate reviews every T.12 amendment proposal,
> verifies that all dedup-court deltas are internally
> consistent, freezes the admitted expansion set, and emits
> `corpus_hash_v2`. It does not add new literature primitives
> except through explicitly rejected late-amendment handling.
> Its purpose is ratification, not expansion.**

Paper / README framing (plan-locked verbatim):

> T.12.consolidate closes the literature expansion arc. The
> Atlas no longer treats the T.12.x proposals as isolated
> source-class filings; it ratifies them as one deduplicated
> expansion set and freezes `corpus_hash_v2`. `corpus_hash_v1`
> remains the historical seed-corpus anchor, while
> `corpus_hash_v2` becomes the first post-amendment corpus
> authority.

**Three new own-namespace hashes**:

- `consolidation_report_hash_v1` under
  `DSFB-GPU-ATLAS:T12-CONSOLIDATION-REPORT:v1\0` = `2842f6ae...`
- `t12_expansion_index_hash_v1` under
  `DSFB-GPU-ATLAS:T12-EXPANSION-INDEX:v1\0` = `11fe6543...`
- `corpus_hash_v2` under
  `DSFB-GPU-ATLAS:LITERATURE-CORPUS:v2\0` = `f1d132eb...`

`corpus_hash_v2` is the **ratified-corpus AUTHORITY anchor**.
META-hashes `corpus_hash_v1` + the consolidation report + the
expansion index + sorted admitted canonical ids + SEED length.
Does NOT mutate SEED; does NOT mutate `corpus_hash_v1`.

**Loaded proposal set**: 17 proposals (T.12.0 proof-of-life +
T.12.a..T.12.p real proposals). Every proposal hash, batch
hash, and dedup-delta hash verified by recomputation.

**Aggregate court delta across T.12.a..T.12.p**:

- 98 CanonicalAddition (includes 2 T.12.a-era `Canonical`
  historical wire-name records; post-T.12.b plan-locked era
  uses `CanonicalAddition`)
- 76 `ExistingCanonicalAuthorityResolution`
- 23 `DomainTransferOf`
- 49 `ParameterizationOf`
- 24 `RejectedNotDeterministic`
- 1 T.12.a-era `AliasOf`
- 2 T.12.a-era `CompositionOf`

**Total**: 273 dedup-court records.

**Expansion index**: 98 entries, sorted ascending by
canonical_id, spanning 5001..=6699. T.12.m's reserved ids
6301 + 6302 deliberately unused (SEED-walk-first caught the
SEED 53 + 54 collisions); verified absent from the expansion
index.

**Ten plan-required load-bearing negatives** (all PASS):

- `consolidate_rejects_missing_t12_proposal`
- `consolidate_rejects_duplicate_reserved_id`
- `consolidate_rejects_unused_reserved_id_without_pin_or_explanation`
- `consolidate_rejects_canonical_addition_colliding_with_seed`
- `consolidate_rejects_parameterization_without_parent`
- `consolidate_rejects_authority_resolution_without_existing_target`
- `consolidate_rejects_rejected_record_without_rejection_contract`
- `consolidate_rejects_hash_mismatch_against_emitted_artifact`
- `consolidate_rejects_corpus_hash_v2_if_corpus_hash_v1_mutated`
- `consolidate_rejects_uncredited_literature_record`

Plus aggregate-count regression sentinels, hash determinism /
sensitivity invariants, expansion-index structural invariants,
rendering byte-stability checks, domain-separator pins, and
SEED + `corpus_hash_v1` invariance witnesses. 53 acceptance
tests total.

**Plan-locked non-claims**:

- T.12.consolidate does NOT add new literature primitives.
- T.12.consolidate does NOT mutate `SEED` (stays at 54).
- T.12.consolidate does NOT mutate `corpus_hash_v1`.
- T.12.consolidate does NOT mutate any prior T.11 / S1.3 /
  T.12.x hash.
- T.12.consolidate does NOT promote individual proposals to
  `Accepted`. Every proposal stays at `Open` status; future
  per-proposal ratification commits change status.
- `corpus_hash_v2` is a META-hash over the ratified-expansion
  set; it is NOT a full re-hash of a new SEED table. The
  migration into a new SEED table is a separate future commit
  gated on per-proposal `Accepted` status.

Receipts:
[reports/t12_consolidate_summary.txt](reports/t12_consolidate_summary.txt),
[reports/t12_consolidate_verification.txt](reports/t12_consolidate_verification.txt),
[reports/t12_consolidate_regression_check.txt](reports/t12_consolidate_regression_check.txt).

Next campaign: **per-proposal `Accepted` migration arc**
starting with **FF.1 passport materialisation** (landed; see
the FF.1 section below).

## FF.1 — DetectorPassport materialisation for corpus_hash_v2-ratified Accepted entries (the META-derivation layer above T.12.consolidate; makes accepted T.12 corpus expansion entries operational court citizens)

FF.1 is the **first ratification campaign above
`corpus_hash_v2`**. Plan-locked opening guard:

> **FF.1 materializes DetectorPassport records for Accepted
> T.12 expansion entries ratified by `corpus_hash_v2`. It
> does not reopen T.12 dedup decisions, add new literature
> primitives, alter `corpus_hash_v1`, alter `corpus_hash_v2`,
> or rewrite historical proposal hashes.**

Plan-locked thesis:

> T.12.consolidate ratified the expansion.
> FF.1 gives the accepted records passports.

**Method**: pull the T.12 expansion index (98 ratified
CanonicalAddition records) from the consolidate module read-
only; for each entry, derive the operational passport fields
(canonical_id, display_name, source_class, origin_proposal_id
directly from the expansion-index entry; GPU-family wire
name from the plan-locked SourceClass → GpuFamilyKernel
mapping; activation-applicability tags from the plan-locked
SourceClass → tag-set mapping; contraindication-linkage stub
+ challenge-surface stub at empty-but-declared); emit a per-
passport `passport_hash_v1`; aggregate into a sorted
`Ff1PassportIndex` with `ff1_passport_index_hash_v1`; emit a
top-level `Ff1MaterialisationReport` with
`ff1_materialisation_report_hash_v1`.

**Three new own-namespace hash layers**:

- Per-passport `passport_hash_v1` (98 values) under
  `DSFB-GPU-ATLAS:FF1-T12-RATIFIED-PASSPORT:v1\0`.
- `ff1_passport_index_hash_v1` = `1ad2dc2d...` under
  `DSFB-GPU-ATLAS:FF1-PASSPORT-INDEX:v1\0`.
- `ff1_materialisation_report_hash_v1` = `5edacbc4...` under
  `DSFB-GPU-ATLAS:FF1-MATERIALISATION-REPORT:v1\0`.

**Pinned upstream anchors** (FF.1 does NOT mutate any of
these):

- `corpus_hash_v1` = `35c276c7...` (historical seed-corpus
  anchor; unchanged).
- `corpus_hash_v2` = `f1d132eb...` (ratified-corpus authority
  anchor from T.12.consolidate; unchanged).
- `consolidation_report_hash_v1` = `2842f6ae...` (unchanged).
- `t12_expansion_index_hash_v1` = `11fe6543...` (unchanged).
- `SEED.len()` = 54 (unchanged).

**SourceClass → GpuFamilyKernel mapping** (plan-locked one-
to-one mapping; fixed at FF.1 time; re-routing requires a
future FF.1.x schema-upgrade commit). T.12.m's reserved ids
6301 + 6302 (deliberately-unused after the SEED-walk-first
restructure caught the SEED 53 + 54 collisions) verified
absent from the expansion index.

**Ten plan-required load-bearing negatives** (all PASS):

- `ff1_rejects_passport_for_non_ratified_canonical_id`
- `ff1_rejects_passport_if_corpus_hash_v2_mismatch`
- `ff1_rejects_passport_materialisation_that_mutates_t12_proposal_hash`
- `ff1_rejects_passport_materialisation_that_mutates_corpus_hash_v2`
- `ff1_rejects_duplicate_passport_for_same_canonical_id`
- `ff1_rejects_missing_source_lineage_for_literature_passport`
- `ff1_rejects_missing_gpu_family_mapping`
- `ff1_rejects_missing_activation_applicability_tags`
- `ff1_rejects_missing_contraindication_linkage_stub`
- `ff1_rejects_missing_challenge_surface_stub`

Plus shape + materialisation discipline witnesses, hash
invariance invariants, FF.1 new-own-namespace-hash
determinism + sensitivity invariants, per-passport field
invariants, SourceClass-mapping discipline tests, and
rendering byte-stability checks. 46 acceptance tests total.

**Plan-locked non-claims**:

- FF.1 does NOT reopen T.12 dedup decisions.
- FF.1 does NOT add new literature primitives.
- FF.1 does NOT alter `corpus_hash_v1`.
- FF.1 does NOT alter `corpus_hash_v2`.
- FF.1 does NOT rewrite historical T.12 proposal hashes.
- FF.1 does NOT rewrite any T.12.consolidate hash.
- FF.1 does NOT mutate `SEED.len()` (stays at 54).
- FF.1 does NOT activate any detector. Activation is S1.3a's
  job; FF.1 only materialises the passport so the activation
  planner has a target to read.
- FF.1 does NOT decide contraindications or challenges. The
  stub fields reserve the space; population is a later
  commit (post-FF.3).
- FF.1 does NOT generate CUDA kernels. The GPU family
  mapping is a declaration of which family the passport
  would route to; kernel generation is a much later commit.

Receipts:
[reports/ff1_passport_summary.txt](reports/ff1_passport_summary.txt),
[reports/ff1_passport_verification.txt](reports/ff1_passport_verification.txt),
[reports/ff1_passport_regression_check.txt](reports/ff1_passport_regression_check.txt).

Next campaign: **FF.2** (landed; see the FF.2 section below).

## FF.2 — ActivationReason::DisabledUnratifiedProposal (the activation ratification gate teaching the court to reject any detector proposal lacking corpus_hash_v2 ratification + FF.1 passport authority)

FF.2 is the **first META-discipline layer above S1.3a + FF.1**.
Plan-locked opening guard:

> **FF.2 makes activation refuse any detector proposal that is
> not ratified by `corpus_hash_v2` and materialized through
> FF.1 passport authority. Core rule: no ratification + no
> passport = no activation. It does not add new detectors,
> alter `corpus_hash_v2`, rewrite FF.1 passports, or change
> prior activation decisions except by making the unratified-
> proposal failure mode explicit and reason-coded.**

Plan-locked one-line verdict:

> FF.1 gave ratified witnesses passports;
> FF.2 teaches the activation court to reject anyone without one.

**Method**: pull the live consolidation report + live FF.1
passport index read-only; walk a candidate canonical-id set
(production default = SEED 1..=54 ∪ FF.1 passport index
5001..=6699) and classify each id into one of four mutually-
exclusive `Ff2RatificationStatus` buckets:

- `SeedHistorical` — id ∈ SEED. Passes the gate; downstream
  S1.3a activation continues to issue per-detector decisions.
- `T12RatifiedAndPassported` — id ∈ ratified expansion index
  AND ∈ FF.1 passport index. Passes the gate; downstream
  activation continues with a passport binding.
- `MissingPassport` — id ∈ ratified expansion index but NOT ∈
  FF.1 passport index. Structural defect (should never occur
  in production); reserved so the verifier's
  `ActivationForMissingFf1Passport` rule has an explicit
  status to surface.
- `UnratifiedProposal` — id outside both SEED and the ratified
  expansion index. The new failure mode FF.2 surfaces
  explicitly: an operator-facing reason code replacing the
  pre-FF.2 silent `DisabledByWeakLBand` fallback.

Emit one `Ff2GateDecision` per id, sorted ascending; aggregate
into the top-level `Ff2ActivationRatificationGate` with per-
status counts and the four pinned anchor hashes
(`corpus_hash_v1`, `corpus_hash_v2`,
`consolidation_report_hash_v1`,
`ff1_passport_index_hash_v1`). Wrap in
`Ff2ActivationRatificationGateSummary` with a plan-locked
non-claim block hashed under a distinct domain.

**Two new own-namespace hash layers**:

- `ff2_activation_ratification_gate_hash_v1` = `05c1b552...`
  under `DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1\0`.
- `ff2_activation_ratification_gate_summary_hash_v1` = `e671cfc0...`
  under `DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE-SUMMARY:v1\0`.

**Pinned upstream anchors** (FF.2 does NOT mutate any of these):

- `corpus_hash_v1` = `35c276c7...` (unchanged).
- `corpus_hash_v2` = `f1d132eb...` (unchanged).
- `consolidation_report_hash_v1` = `2842f6ae...` (unchanged).
- `ff1_passport_index_hash_v1` = `1ad2dc2d...` (unchanged).
- `SEED.len()` = 54 (unchanged).

**New activation enum variant**:
`DisabledReason::DisabledUnratifiedProposal` (wire name
`"DisabledUnratifiedProposal"`). Emitted only by the FF.2
gate; the verifier's `SilentFallbackToDisabledByWeakLBand`
rule rejects any non-ratified decision carrying any other
wire name.

**Per-status decision counts in the default production gate**:
54 `SeedHistorical` + 98 `T12RatifiedAndPassported` + 0
`MissingPassport` + 0 `UnratifiedProposal` = 152 total
decisions.

**Six plan-required load-bearing negatives** (all PASS):

- `ff2_rejects_activation_for_unratified_proposal`
- `ff2_rejects_activation_for_missing_ff1_passport`
- `ff2_rejects_activation_when_passport_index_hash_mismatch`
- `ff2_rejects_unratified_proposal_without_reason_code`
- `ff2_rejects_silent_fallback_to_disabled_by_weak_lband`
- `ff2_rejects_activation_reason_without_corpus_hash_v2_binding`

Plus structural defect rules (duplicate decisions, sort-order,
anchor cross-checks), determinism + sensitivity invariants,
upstream-anchor invariance witnesses, field-level + wire-name
+ domain-separator pins, count-shape regression sentinels,
hash-namespace distinctness assertions, and renderer-coverage
checks. 60 acceptance tests total.

**Plan-locked non-claims**:

- FF.2 does NOT add new detectors.
- FF.2 does NOT alter `corpus_hash_v1`, `corpus_hash_v2`,
  `consolidation_report_hash_v1`,
  `t12_expansion_index_hash_v1`,
  `ff1_passport_index_hash_v1`, or
  `ff1_materialisation_report_hash_v1`.
- FF.2 does NOT rewrite any prior T.11 / S1.3 / T.12.x / FF.1
  hash.
- FF.2 does NOT mutate `SEED.len()` (stays at 54).
- FF.2 does NOT promote any open proposal to Accepted.
- FF.2 does NOT change S1.3a SEED activation decisions; SEED
  ids continue to flow through the existing S1.3a planner.
  FF.2 layers above S1.3a as a ratification-discipline gate.
- FF.2 does NOT generate CUDA kernels.
- FF.2 does NOT decide contraindications or challenges.

**Plan warning enforced verbatim**:

> Do not let unratified proposals collapse into generic
> `DisabledByWeakLBand`. That would erase the court
> distinction. FF.2 exists so the operator can see: this
> detector is disabled because it is not ratified / not
> passported.

Receipts:
[reports/ff2_activation_ratification_gate_summary.txt](reports/ff2_activation_ratification_gate_summary.txt),
[reports/ff2_activation_ratification_gate_verification.txt](reports/ff2_activation_ratification_gate_verification.txt),
[reports/ff2_activation_ratification_gate_regression_check.txt](reports/ff2_activation_ratification_gate_regression_check.txt).

Next campaign: **FF.3** (landed; see the FF.3 section below).

## FF.3 — RegistryGenerationGate (the second META-discipline layer above S1.3a + FF.1 + FF.2; teaches the S1.2 registry generator to refuse any DetectorSpec whose source authority is not a SEED record under corpus_hash_v1 OR a corpus_hash_v2-ratified entry materialised through FF.1 passport authority)

FF.3 is the **registry-generation boundary gate**. Plan-
locked opening guard:

> **FF.3 adds a registry-generation gate for S1.2
> `DetectorSpec` generation. The generator must accept only
> `SeedHistorical` records from `corpus_hash_v1` and
> `T12RatifiedAndPassported` records from `corpus_hash_v2` +
> FF.1 passport authority. It does not add detectors, mutate
> `corpus_hash_v1`, mutate `corpus_hash_v2`, rewrite FF.1
> passports, or change activation decisions. It only prevents
> unratified / non-passported / stale / ad-hoc records from
> entering generated registry output.**

Plan-locked one-line verdict:

> FF.2 blocks unratified activation;
> FF.3 blocks unratified registry generation.

The reason-code separation is the load-bearing win:

```
DisabledByWeakLBand               !=
DisabledUnratifiedProposal        !=    (FF.2 activation reason)
RejectedUnratifiedProposal              (FF.3 registry rejection)
```

A weak-but-ratified detector, an unratified-at-activation
detector, and an unratified-at-registry-generation detector are
three different court failures the operator must be able to
distinguish.

**Method**: pull the live consolidation report + live FF.1
passport index + live FF.2 ratification gate read-only; walk a
candidate registry-generation source-record set (production
default = SEED 1..=54 claiming `SeedHistorical` ∪ FF.1 passport
ids 5001..=6699 claiming `T12RatifiedAndPassported`); classify
each candidate into one of seven mutually-exclusive
`Ff3RegistryGenerationEligibility` buckets:

- `Eligible` — claim verifies against live state; a
  `DetectorSpec` may be generated.
- `RejectedUnratifiedProposal` — `T12RatifiedAndPassported`
  claim for an id NOT in the ratified expansion index. Mirror
  of FF.2's `UnratifiedProposal` bucket.
- `RejectedMissingFf1Passport` — ratified id NOT in FF.1
  passport index. Structural defect (should never occur in
  production); reserved so the verifier rule has an explicit
  status to surface.
- `RejectedCorpusHashV2Mismatch` — pinned `corpus_hash_v2`
  drifted; every ratified claim rejected.
- `RejectedPassportIndexHashMismatch` — pinned
  `ff1_passport_index_hash_v1` drifted; every ratified claim
  rejected.
- `RejectedAdHocRecord` — candidate declared
  `AdHocUnsanctioned` source authority. Forbidden by
  construction.
- `RejectedUnknownSourceAuthority` — candidate declared
  `UnknownExternal` source authority. Forbidden by
  construction.

Emit one `Ff3RegistryGenerationEligibilityDecision` per
candidate sorted ascending; aggregate into the top-level
`Ff3RegistryGenerationGate` with per-status counts + five
pinned upstream anchor hashes (`corpus_hash_v1`,
`corpus_hash_v2`, `consolidation_report_hash_v1`,
`ff1_passport_index_hash_v1`,
`ff2_activation_ratification_gate_hash_v1`). Wrap in
`Ff3RegistryGenerationGateSummary` with a plan-locked
non-claim block hashed under a distinct domain.

**Two new own-namespace hash layers**:

- `ff3_registry_generation_gate_hash_v1` = `2ffd0222...`
  under
  `DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1\0`.
- `ff3_registry_generation_gate_summary_hash_v1` = `c66f8174...`
  under
  `DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE-SUMMARY:v1\0`.

**Pinned upstream anchors** (FF.3 does NOT mutate any of these):

- `corpus_hash_v1` = `35c276c7...` (unchanged).
- `corpus_hash_v2` = `f1d132eb...` (unchanged).
- `consolidation_report_hash_v1` = `2842f6ae...` (unchanged).
- `ff1_passport_index_hash_v1` = `1ad2dc2d...` (unchanged).
- `ff2_activation_ratification_gate_hash_v1` = `05c1b552...`
  (unchanged).
- `SEED.len()` = 54 (unchanged).

**Per-status decision counts in the default production gate**:
152 `Eligible` (54 SEED + 98 ratified) + 0 of every rejection
bucket = 152 total decisions.

**Eight plan-required load-bearing negatives** (all PASS):

- `ff3_rejects_detector_spec_for_unratified_proposal`
- `ff3_rejects_detector_spec_for_missing_ff1_passport`
- `ff3_rejects_detector_spec_when_corpus_hash_v2_mismatch`
- `ff3_rejects_detector_spec_when_passport_index_hash_mismatch`
- `ff3_rejects_detector_spec_from_ad_hoc_record`
- `ff3_rejects_detector_spec_with_unknown_source_authority`
- `ff3_rejects_registry_generation_that_skips_ff2_ratification_gate`
  (FF.3 MUST consult the live FF.2 gate hash)
- `ff3_rejects_registry_generation_that_mutates_existing_registry_hash`
  (FF.3 cannot admit MORE candidates for registry generation
  than FF.2 admits for activation)

Plus seven structural defect rules (duplicate decisions, sort-
order, anchor cross-checks, eligible-with-non-empty-reason,
rejection-with-empty-reason, claim/classification consistency,
SEED invariance), determinism + sensitivity invariants,
upstream-anchor invariance witnesses, field-level + wire-name +
domain-separator pins, count-shape regression sentinels,
hash-namespace distinctness assertions, and renderer-coverage
checks. 67 acceptance tests total.

**Plan-locked non-claims**:

- FF.3 does NOT add new detectors.
- FF.3 does NOT alter `corpus_hash_v1`, `corpus_hash_v2`,
  `consolidation_report_hash_v1`,
  `t12_expansion_index_hash_v1`,
  `ff1_passport_index_hash_v1`,
  `ff1_materialisation_report_hash_v1`, or
  `ff2_activation_ratification_gate_hash_v1`.
- FF.3 does NOT rewrite any prior T.11 / S1.3 / T.12.x /
  FF.1 / FF.2 hash.
- FF.3 does NOT mutate `SEED.len()` (stays at 54).
- FF.3 does NOT promote any open proposal to Accepted.
- FF.3 does NOT change S1.3a SEED activation decisions or
  FF.2 ratification decisions; it layers ABOVE FF.2 as a
  registry-generation-boundary gate.
- FF.3 does NOT itself emit `DetectorSpec` records. It is a
  pure-decision module that the S1.2 registry generator
  consults; integration with `dsfb-gpu-atlas-registry` lands
  in a follow-on commit.
- FF.3 does NOT modify `dsfb-gpu-atlas-registry`'s existing
  162-spec `registry_hash_v2`; that hash stays unchanged
  until the integration commit lands.
- FF.3 does NOT generate CUDA kernels.
- FF.3 does NOT decide contraindications or challenges.

Receipts:
[reports/ff3_registry_generation_gate_summary.txt](reports/ff3_registry_generation_gate_summary.txt),
[reports/ff3_registry_generation_gate_verification.txt](reports/ff3_registry_generation_gate_verification.txt),
[reports/ff3_registry_generation_gate_regression_check.txt](reports/ff3_registry_generation_gate_regression_check.txt).

Next campaign: **FF.4** (landed; see the FF.4 section below).

## FF.4 — README authority-boundary policy (communication-hygiene seal making the post-T.12.consolidate + FF.1 + FF.2 + FF.3 authority-boundary state unmissable at the README front door)

FF.4 is the **first communication-hygiene seal** in the
post-ratification arc. Plan-locked opening guard:

> **FF.4 makes the post-T.12.consolidate / post-FF.1 / post-FF.2
> / post-FF.3 authority-boundary state unmissable at the README
> front door. It does not add detectors, mutate any upstream
> hash anchor, modify SEED, modify any court artifact, or
> change activation / registry-generation behaviour. It is a
> communication-hygiene seal: the operator-facing README MUST
> carry the canonical authority-boundary block stating that
> T.12.a..T.12.p were filed as amendment proposals (and did
> not mutate seed authority while filed), that T.12.consolidate
> froze `corpus_hash_v2` as the ratified post-amendment
> authority, that FF.1 materialized 98 ratified
> CanonicalAddition entries into T12RatifiedPassport records,
> and that FF.2 + FF.3 reject unratified / non-passported /
> ad-hoc / unknown-source records by explicit reason code.
> Stale pre-ratification phrasings (the FF.4 forbidden-
> substring set; see `FF4_FORBIDDEN_SUBSTRINGS`) are rejected
> in the front-door area now that the ratification + passport
> materialization already happened.**

Plan-locked one-line verdict:

> FF.4 makes the authority boundary unmissable at the front
> door; it does not move any boundary.

**Why**: before T.12.consolidate the README warning correctly
identified the T.12.a..j filings as amendment proposals that
did not mutate SEED, `corpus_hash_v1`, `registry_hash_v2`,
`DetectorPassports`, or activation outputs while filed —
deferring any seed-authority change to a then-future ratification.
After T.12.consolidate + FF.1 + FF.2 + FF.3 that deferred
ratification already happened, the deferred freeze already
produced `corpus_hash_v2`, and the 98 ratified entries already
have FF.1 passports + FF.2 activation status + FF.3 registry-
generation eligibility. The pre-ratification phrasings are
therefore retired by the FF.4 forbidden-substring set.

**Method**: pin a canonical 19-line authority-boundary block
(`FF4_AUTHORITY_BOUNDARY_BLOCK_LINES`); pin a 6-entry
required-substring set (`FF4_REQUIRED_SUBSTRINGS`); pin a
7-entry forbidden-substring set (`FF4_FORBIDDEN_SUBSTRINGS`);
emit a top-level `Ff4ReadmeAuthorityBoundaryPolicy` artifact
pinning the five upstream anchor hashes (`corpus_hash_v1`,
`corpus_hash_v2`, `ff1_passport_index_hash_v1`,
`ff2_activation_ratification_gate_hash_v1`,
`ff3_registry_generation_gate_hash_v1`) plus the canonical
block + substring sets; verify any README text against the
policy. A live README sweep test (`ff4_live_readme_satisfies_policy`)
reads `README.md` from disk and runs the verifier on every
build, so future commits cannot regress the front-door
authority-state story.

**One new own-namespace hash**:

- `ff4_readme_authority_boundary_policy_hash_v1` =
  `22b9dcb5...` under
  `DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0`.

**Pinned upstream anchors** (FF.4 does NOT mutate any of these):

- `corpus_hash_v1` = `35c276c7...` (unchanged).
- `corpus_hash_v2` = `f1d132eb...` (unchanged).
- `ff1_passport_index_hash_v1` = `1ad2dc2d...` (unchanged).
- `ff2_activation_ratification_gate_hash_v1` = `05c1b552...`
  (unchanged).
- `ff3_registry_generation_gate_hash_v1` = `2ffd0222...`
  (unchanged).
- `SEED.len()` = 54 (unchanged).

**Seven plan-required load-bearing negatives** (all PASS):

- `ff4_readme_rejects_stale_future_ratification_language`
- `ff4_readme_requires_corpus_hash_v1_historical_anchor_language`
- `ff4_readme_requires_corpus_hash_v2_ratified_authority_language`
- `ff4_readme_requires_ff1_passport_materialisation_language`
- `ff4_readme_requires_ff2_ff3_unratified_rejection_language`
- `ff4_readme_rejects_claim_that_t12_proposals_mutated_seed`
- `ff4_readme_rejects_claim_that_ff1_mutated_corpus_hash_v2`

Plus a live README sweep, determinism + sensitivity invariants,
upstream-anchor invariance witnesses, field-level + wire-name
+ domain-separator pins, disjoint-set discipline (required
substrings ∩ forbidden substrings = ∅), block-coverage
invariants, hash-namespace distinctness assertions, and
renderer-coverage checks. 42 acceptance tests total.

**The actual README change**: the canonical 19-line
authority-boundary block is now embedded near the top of this
README (between the Plan-locked-anchor block and the `## What
this is` section), at the front-door area an operator first
encounters.

**Plan-locked non-claims**:

- FF.4 does NOT add new detectors.
- FF.4 does NOT alter any upstream hash anchor (`corpus_hash_v1`,
  `corpus_hash_v2`, any T.12.x proposal hash, any
  T.12.consolidate hash, any FF.1 / FF.2 / FF.3 hash).
- FF.4 does NOT rewrite any prior T.11 / S1.3 / T.12.x / FF.1
  / FF.2 / FF.3 hash.
- FF.4 does NOT mutate `SEED.len()` (stays at 54).
- FF.4 does NOT change S1.3a / FF.2 / FF.3 court decisions.
- FF.4 does NOT itself emit `DetectorSpec` records, mutate the
  registry crate, decide contraindications or challenges, or
  generate CUDA kernels.
- FF.4 changes the README text; it does not change court
  state.

Receipts:
[reports/ff4_readme_authority_boundary_summary.txt](reports/ff4_readme_authority_boundary_summary.txt),
[reports/ff4_readme_authority_boundary_verification.txt](reports/ff4_readme_authority_boundary_verification.txt),
[reports/ff4_readme_authority_boundary_regression_check.txt](reports/ff4_readme_authority_boundary_regression_check.txt).

Next campaign: **FF.5** (landed; see the FF.5 section below).

## FF.5 — ProposalSchemaUpgradePolicy (forward-looking governance policy + migration-table type + receipt type + verifier defining how proposal schema upgrades may re-render historical proposal artifacts without erasing the old artifact hashes or confusing the court lineage)

FF.5 is the **second communication / governance seal** in the
post-ratification arc, sibling of FF.4 but operating on
forward-looking schema-upgrade contracts rather than current
README text. Plan-locked opening guard:

> **FF.5 defines how proposal schema upgrades are allowed to
> re-render historical proposal artifacts without erasing the
> old artifact hashes or confusing the court lineage. Core
> rule: schema upgrade ≠ silent artifact rewrite. Required
> doctrine: if a schema change re-renders old proposals, the
> migration must preserve the old artifact hash, emit the new
> schema hash, explain why the rendered bytes changed, and
> provide an explicit `old_hash → new_hash` migration table.
> The old artifact remains part of the evidence trail; the
> new artifact becomes the active schema rendering only
> through an explicit migration receipt.**

Plan-locked one-line verdict:

> Schema upgrade != silent artifact rewrite.

**Why**: the post-T.12.consolidate / post-FF.1 / post-FF.2 /
post-FF.3 / post-FF.4 arc surfaced several future-work items
(richer `ProposedSourceRef` with `authors` / `doi_or_url`,
structured contract flags on `ProposedPrimitive`,
`ratification_commit` split) that will eventually require
re-rendering historical T.12.x proposal artifacts under a v2
schema. Without an upfront policy, the most likely failure
mode is a silent re-render — new bytes overwrite the
historical hash anchors without provenance, and the court
lineage becomes ambiguous. FF.5 lands the policy BEFORE any
schema upgrade so every future upgrade has a known contract
to satisfy.

**Method**: pin a 10-line plan-locked doctrine; declare four
record types — `MigrationRow` (one row per re-rendered
artifact, carrying old hash + new hash + old/new schema
versions + reason), `ProposalSchemaMigrationTable` (the
rolling list of every migration row, sorted by
`old_artifact_hash`), `ProposalSchemaUpgradeReceipt` (the
receipt shape every future schema-upgrade commit must emit;
carries `upgrade_id`, schema versions, `semantic_reason`,
migration rows, `preserves_corpus_hash_v1`,
`preserves_corpus_hash_v2`, `freeze_campaign_id`,
`declares_old_artifact_hash_valid`), and
`ProposalSchemaUpgradePolicy` (the top-level artifact pinning
the doctrine + table + six upstream anchor hashes); expose
two verifiers (`verify_schema_upgrade_receipt` for individual
receipts, `verify_migration_table` for whole-table
invariants). At FF.5 baseline the migration table is empty;
the type + verifier are the deliverable.

**Three new own-namespace hash layers**:

- `proposal_schema_upgrade_policy_hash_v1` = `94e00ab1...`
  under `DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1\0`.
- `proposal_schema_migration_table_hash_v1` = `625824d0...`
  under
  `DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1\0`
  (empty table at FF.5 baseline).
- per-receipt `schema_upgrade_receipt_hash_v1` under
  `DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1\0` (type +
  verifier shipped; no receipt at FF.5 baseline).

**Pinned upstream anchors** (FF.5 does NOT mutate any of these):

- `corpus_hash_v1` = `35c276c7...` (unchanged).
- `corpus_hash_v2` = `f1d132eb...` (unchanged).
- `ff1_passport_index_hash_v1` = `1ad2dc2d...` (unchanged).
- `ff2_activation_ratification_gate_hash_v1` = `05c1b552...`
  (unchanged).
- `ff3_registry_generation_gate_hash_v1` = `2ffd0222...`
  (unchanged).
- `ff4_readme_authority_boundary_policy_hash_v1` =
  `22b9dcb5...` (unchanged).
- `SEED.len()` = 54 (unchanged).

**Nine plan-required load-bearing negatives** (all PASS):

- `ff5_rejects_schema_rerender_without_old_hash`
- `ff5_rejects_schema_rerender_without_new_schema_hash`
- `ff5_rejects_schema_rerender_without_migration_table`
- `ff5_rejects_schema_rerender_without_reason`
- `ff5_rejects_migration_table_with_duplicate_old_hash`
- `ff5_rejects_migration_table_with_duplicate_new_hash`
- `ff5_rejects_claim_that_old_artifact_hash_was_invalid`
- `ff5_rejects_schema_upgrade_that_mutates_corpus_hash_v1`
- `ff5_rejects_schema_upgrade_that_mutates_corpus_hash_v2_without_freeze_campaign`

Plus five structural defect rules (identical-schema-versions,
empty upgrade-id, empty freeze-campaign-id, empty artifact-id,
empty schema-version), determinism + sensitivity invariants,
upstream-anchor invariance witnesses (FF.5 mutates nothing),
pinned anchor cross-checks, field-level + wire-name +
domain-separator pins, hash-namespace distinctness assertions,
and renderer-coverage checks. 58 acceptance tests total.

**Plan-locked non-claims**:

- FF.5 does NOT add new detectors.
- FF.5 does NOT alter any upstream hash anchor.
- FF.5 does NOT rewrite any prior T.11 / S1.3 / T.12.x /
  FF.1 / FF.2 / FF.3 / FF.4 hash.
- FF.5 does NOT mutate `SEED.len()` (stays at 54).
- FF.5 does NOT itself perform any schema upgrade. It is a
  forward-looking governance artifact pinning the contract
  future upgrades MUST satisfy.
- FF.5 does NOT change S1.3a / FF.2 / FF.3 / FF.4 court
  decisions.
- FF.5 does NOT generate CUDA kernels.
- FF.5 does NOT decide contraindications or challenges.

Receipts:
[reports/ff5_proposal_schema_policy_summary.txt](reports/ff5_proposal_schema_policy_summary.txt),
[reports/ff5_proposal_schema_policy_verification.txt](reports/ff5_proposal_schema_policy_verification.txt),
[reports/ff5_proposal_schema_policy_regression_check.txt](reports/ff5_proposal_schema_policy_regression_check.txt).

Next campaign: **S1.3d — Budget pruning + redundancy
suppression**. Consumes `ActivationContextV1`, T.8 usefulness
ledger, contraindications, coverage holes, and task budget;
emits reason-coded `DisabledByBudget` / `DisabledByRedundancy`
decisions. With FF.2 + FF.3 + FF.4 + FF.5 in place, the
post-S1.3a activation arc continues into the full S1.3d/e/f
maturity sequence.

## S1.3d — Budget pruning + redundancy suppression (deterministic budget-aware deployment court above FF.2 + FF.3 with eight reason-coded disable variants and a replayable tie-break transcript)

**Status**: SEALED. Post-FF.5 plan directive (2026-05-16).

S1.3d is the deterministic decision layer that turns the
FF.3-eligible detector surface (152 ratified candidates at
baseline = 54 SEED + 98 T12-passported) into a budget-aware
deployment plan. It consumes a declared `TaskBudget` envelope
plus an explicit set of `RedundancyCluster` declarations and
emits one reason-coded `S13dBudgetDecision` per candidate.

Each decision carries either an `Active` outcome (with a
`RetainedAsBudgetSurvivor` or `RetainedAsRepresentativeWitness`
retain reason) or a `Disabled` outcome with one of eight
budget disable variants: `DisabledByBudget`,
`DisabledByRedundancy`, `DisabledByGpuFamilyQuota`,
`DisabledByTaskBudget`, `DisabledByRuntimeBudget`,
`DisabledByMemoryBudget`, `DisabledByContraindicationBudget`,
`DisabledByCoverageHoleBudget`.

The reason-code separation is the load-bearing win:
`DisabledByWeakLBand` (S1.3a) ≠ `DisabledUnratifiedProposal`
(FF.2) ≠ `RejectedUnratifiedProposal` (FF.3) ≠
`DisabledByBudget` (S1.3d) ≠ `DisabledByRedundancy` (S1.3d) —
five different court failures the operator must distinguish.

The plan-locked default task budget is intentionally
permissive (`max_active_detectors = 10_000`, `u64::MAX`
runtime + memory ceilings, empty per-GPU-family quota set,
empty redundancy cluster set, no contraindication / coverage-
hole gates). Under those conditions every FF.3-eligible
candidate flows through to `Active` with
`RetainedAsBudgetSurvivor`; the synthetic budget-pressure
scenarios in the acceptance suite inject tighter budgets and
cluster sets to exercise each of the eight disable reason
codes.

Three new own-namespace hashes:

- `budget_pruning_plan_hash_v1 = 82be2289...` under
  `DSFB-GPU-ATLAS:S13D-BUDGET-PRUNING-PLAN:v1\0` (META-hashes
  the seven pinned upstream anchors + the task-budget envelope
  + the 152 sorted decisions + per-reason counts + the
  tie-break transcript).
- `redundancy_suppression_hash_v1 = 875b5b60...` under
  `DSFB-GPU-ATLAS:S13D-REDUNDANCY-SUPPRESSION:v1\0` (META-
  hashes the cluster declarations + retained representatives
  + suppression count; empty at S1.3d baseline because no
  production clusters have been declared yet).
- `budgeted_activation_summary_hash_v1 = 5feab238...` under
  `DSFB-GPU-ATLAS:S13D-BUDGETED-ACTIVATION-SUMMARY:v1\0`
  (top-level META-hash wrapping the plan + redundancy report
  so operators can pin one hash that fixes the entire S1.3d
  state).

EIGHT plan-required load-bearing negatives pin the contract:
`s13d_rejects_budget_plan_that_uses_ff3_rejected_record`,
`s13d_rejects_silent_detector_drop_without_suppression_reason`,
`s13d_rejects_redundancy_suppression_without_surviving_representative`,
`s13d_rejects_budget_overrun_without_reason_coded_pruning`,
`s13d_rejects_nondeterministic_tie_break_between_equal_priority_detectors`,
`s13d_rejects_gpu_family_budget_without_declared_cost_model`,
`s13d_rejects_pruning_that_mutates_corpus_hash_v1_or_v2`,
`s13d_rejects_schema_upgrade_side_effect_inside_budget_pruning`.

Plan-locked verdict (verbatim): *"Eligibility is not
activation; activation is not budget admission."*

**Plan-locked non-claims**: S1.3d does NOT add new
detectors; does NOT alter `corpus_hash_v1` (stays at
`35c276c7…`); does NOT alter `corpus_hash_v2` (stays at
`f1d132eb…`); does NOT alter any FF.1 / FF.2 / FF.3 / FF.4 /
FF.5 hash; does NOT rewrite any prior T.11 / S1.3 / T.12.x /
FF.x hash; does NOT mutate `SEED.len()` (stays at 54); does
NOT change S1.3a SEED activation decisions or FF.2
ratification decisions or FF.3 registry-generation
eligibility; does NOT generate CUDA kernels; does NOT itself
emit `KernelPlan` records (that is S1.3e); does NOT decide
contraindications or challenges; does NOT modify the registry
crate.

CLI: `dsfb-corpus s1-3d-plan [--json] [--out PATH]`,
`s1-3d-plan-emit [--out-dir DIR]`,
`s1-3d-redundancy [--json] [--out PATH]`,
`s1-3d-summary [--json] [--out PATH]`. Bulk artifacts:
`crates/dsfb-gpu-atlas-corpus/out/s1_3d_budget_pruning_plan_v1.{txt,json}`,
`.../s1_3d_redundancy_suppression_v1.{txt,json}`,
`.../s1_3d_budgeted_activation_summary_v1.{txt,json}`.

Next campaign: **S1.3e — KernelPlanV1** (landed; see the
S1.3e section below).

## S1.3e — KernelPlanV1 (deterministic GPU-family execution-plan layer above S1.3d emitting per-family lanes, parameter-table ranges, and execution-plan receipts without running a single kernel)

**Status**: SEALED. Post-S1.3d plan directive (2026-05-17).

S1.3e converts the budgeted activation surface into a
deterministic GPU-family execution plan. It does NOT execute
kernels, generate CUDA code, mutate corpus authority, or
change activation / budget decisions. It maps retained
witnesses into family-compacted kernel lanes, parameter-table
ranges, and execution-plan receipts.

Core rule (plan-locked): **budget admission is not
execution; `KernelPlanV1` is a deterministic plan, not a GPU
run**.

S1.3e consumes the S1.3d-Active candidate set (152 retained
witnesses at baseline = 54 SEED + 98 T12-passported),
resolves each id's GPU family from either the SEED record's
`gpu_family` field (id ≤ 54) or the FF.1 passport's
`gpu_family_wire_name` field (id > 54), groups the set by
family, and emits:

- One `FamilyLane` per GPU family (sorted ascending by
  `gpu_family_wire_name`), each carrying `active_canonical_ids`
  (sorted ascending), `active_detector_count`,
  `declared_cost_model` (non-empty wire string from a
  plan-locked lookup table, e.g.
  `"O(window) per cell sliding statistic"`),
  `expected_kernel_name`, and `aggregate_cost_us` (count ×
  per_detector_runtime_us from S1.3d's inherited task budget).
- One `ParameterTableRow` per active detector, sorted by
  `(gpu_family_wire_name, canonical_id)` ascending.
- A top-level `KernelPlanV1` META-hash binding the schedule +
  parameter table + nine pinned upstream anchor hashes.

The reason-code separation continues: S1.3e never collapses
execution into activation. The plan-required negatives
explicitly reject any kernel-plan that names a
budget-disabled or FF.3-rejected canonical id.

Under the plan-permissive default budget the schedule emits
**14 lanes / 152 active detectors / 152 000 µs aggregate cost**.
Per-lane distribution: DistributionDistanceFamily (28),
SequentialRecurrenceFamily (26), SpectralFamily (23),
WindowStatisticFamily (20), ResidualObserverFamily (13),
TabularConstraintFamily (11), GraphLocalFamily (9),
ProjectionResidualFamily (9), ScalarThresholdFamily (6),
MissingnessFamily (2), RankStatisticFamily (2),
CategoricalHistogramFamily (1), NegativeWitnessFamily (1),
WaveletFamily (1).

Three new own-namespace hashes:

- `kernel_plan_hash_v1 = e48c89b9...` under
  `DSFB-GPU-ATLAS:S13E-KERNEL-PLAN:v1\0` (META-hashes the
  nine pinned upstream anchors plus lane count + total active
  count + total aggregate cost + the family-schedule hash +
  the parameter-table hash).
- `kernel_family_schedule_hash_v1 = 8a58d3bc...` under
  `DSFB-GPU-ATLAS:S13E-KERNEL-FAMILY-SCHEDULE:v1\0` (sorted
  list of 14 lanes).
- `kernel_parameter_table_hash_v1 = 6b27dcbe...` under
  `DSFB-GPU-ATLAS:S13E-KERNEL-PARAMETER-TABLE:v1\0` (sorted
  list of 152 parameter-table rows).

EIGHT plan-required load-bearing negatives pin the contract:
`s13e_rejects_kernel_plan_using_budget_disabled_detector`,
`s13e_rejects_kernel_plan_using_ff3_rejected_record`,
`s13e_rejects_kernel_plan_without_gpu_family_mapping`,
`s13e_rejects_parameter_table_without_stable_order`,
`s13e_rejects_family_schedule_without_declared_cost_model`,
`s13e_rejects_kernel_plan_that_mutates_activation_or_budget_hash`,
`s13e_rejects_cuda_execution_claim_inside_kernel_plan`
(case-insensitive substring scanner forbidding "kernel launch",
"cuda execution", "ptx emission", etc. inside any lane's
cost-model or expected-kernel string),
`s13e_rejects_nondeterministic_tie_break_in_family_order`.

Plan-locked verdict (verbatim): *"S1.3d says who survives
budgeted deployment; S1.3e says how the survivors are packed
into deterministic GPU-family execution lanes."*

**Plan-locked non-claims**: S1.3e does NOT execute kernels;
does NOT emit CUDA source, PTX, SASS, or cubin bytes; does
NOT alter `corpus_hash_v1` (stays at `35c276c7…`); does NOT
alter `corpus_hash_v2` (stays at `f1d132eb…`); does NOT alter
any FF.1 / FF.2 / FF.3 / FF.4 / FF.5 / S1.3d hash; does NOT
rewrite any prior T.11 / S1.3 / T.12.x / FF.x hash; does NOT
mutate `SEED.len()` (stays at 54); does NOT change S1.3a /
FF.2 / FF.3 / S1.3d court decisions; does NOT itself emit a
`CaseFileV2Header` (that integration is S1.3f); does NOT
decide contraindications or challenges; does NOT modify the
registry crate.

CLI: `dsfb-corpus s1-3e-plan [--json] [--out PATH]`,
`s1-3e-plan-emit [--out-dir DIR]`,
`s1-3e-schedule [--json] [--out PATH]`,
`s1-3e-parameter-table [--json] [--out PATH]`. Bulk artifacts:
`crates/dsfb-gpu-atlas-corpus/out/s1_3e_kernel_plan_v1.{txt,json}`,
`.../s1_3e_kernel_family_schedule_v1.{txt,json}`,
`.../s1_3e_kernel_parameter_table_v1.{txt,json}`.

Next campaign: **S1.3f — CaseFileV2 activation integration**
(landed; see the S1.3f section below).

## S1.3f — CaseFileV2 activation integration (binds activation, context, budget pruning, redundancy suppression, and KernelPlanV1 into a single replayable authority chain so case-file body evidence cannot detach from the court decisions that authorised it)

**Status**: SEALED. Post-S1.3e plan directive (2026-05-17).

S1.3f binds activation (S1.3a), transcript root (S1.3b),
context (S1.3c), budget pruning (S1.3d), redundancy
suppression (S1.3d), kernel plan (S1.3e), FF.2 + FF.3 gate
hashes, T.11g contraindication snapshot, T.11f challenge
docket snapshot, T.11h coverage-hole snapshot, and the
corpus authority anchors (`corpus_hash_v1` / `corpus_hash_v2`)
into a single replayable authority chain that every emitted
case file MUST carry.

Core rule (plan-locked): **a case file must not contain
witness / candidate results without the activation and
kernel-plan authority chain that made those witnesses
admissible to run**.

S1.3f produces three case-file sections + three META-hashes:

- **Activation binding** — META-hashes the six activation-
  side anchors (S1.3a `activation_plan_hash_v1` + S1.3b
  `activation_decision_transcript_root_hash_v1` + S1.3c
  `activation_context_hash_v1` + S1.3d
  `budget_pruning_plan_hash_v1` +
  `redundancy_suppression_hash_v1` +
  `budgeted_activation_summary_hash_v1`) so one hash pins
  the entire "who is activated and why" decision tree.
- **Kernel-plan binding** — META-hashes the three S1.3e
  anchors (plan / schedule / parameter table) plus a per-
  detector lane membership index. The index is 152 rows at
  baseline, each mapping an Active canonical id to a
  `(gpu_family_wire_name, lane_offset)` pair, sorted
  ascending by canonical id. This is the linkage S1.3e
  could not declare on its own.
- **Authority chain** (top-level) — META-hashes both
  bindings above plus FF.2 + FF.3 gate hashes + the
  contraindication / challenge / coverage-hole snapshot
  hashes + `corpus_hash_v1` + `corpus_hash_v2`. One hash
  pins the entire court chain a replayer can verify against
  the live upstream state.

Under the plan-permissive default budget the chain pins
54 activation decisions / 152 budgeted-Active candidates /
0 budgeted-Disabled / 14 kernel-plan lanes / 152 lane
membership rows / `activation_decision_transcript_root_hash_v1
= 022f2471…`.

Three new own-namespace hashes:

- `casefile_v2_activation_binding_hash_v1 = fac45fea...` under
  `DSFB-GPU-ATLAS:CASEFILE-V2-ACTIVATION-BINDING:v1\0`.
- `casefile_v2_kernel_plan_binding_hash_v1 = 5df9541c...` under
  `DSFB-GPU-ATLAS:CASEFILE-V2-KERNEL-PLAN-BINDING:v1\0`.
- `casefile_v2_authority_chain_hash_v1 = 52398079...` under
  `DSFB-GPU-ATLAS:CASEFILE-V2-AUTHORITY-CHAIN:v1\0`.

TEN plan-required load-bearing negatives pin the contract:
`s13f_rejects_casefile_without_activation_plan_hash`,
`s13f_rejects_casefile_without_activation_context_hash`,
`s13f_rejects_casefile_without_budget_summary_hash`,
`s13f_rejects_casefile_without_kernel_plan_hash`,
`s13f_rejects_casefile_with_kernel_plan_not_matching_budgeted_activation`,
`s13f_rejects_casefile_with_detector_result_not_in_kernel_plan`
(a body claim is rejected if its canonical id is not in the
lane membership index),
`s13f_rejects_casefile_with_suppressed_detector_result_as_active`
(a body claim is rejected if it labels a budgeted-Disabled
canonical id as `Active`),
`s13f_rejects_casefile_without_ff2_or_ff3_gate_hash`,
`s13f_rejects_casefile_without_challenge_or_contraindication_linkage`
(rejects when any of contraindication / challenge / coverage-
hole hashes are zero),
`s13f_rejects_casefile_authority_chain_mutating_upstream_hashes`
(rejects when any binding's pinned anchor does not equal the
live upstream).

Plan-locked verdict (verbatim): *"S1.3f makes CaseFileV2
carry the whole activation-to-kernel authority chain, so
evidence output cannot be detached from the court decisions
that allowed it to exist."*

**Plan-locked non-claims**: S1.3f does NOT emit detector
outputs, witness records, fusion tensors, candidate
intervals, episodes, or any other body-of-evidence field;
does NOT execute kernels; does NOT alter `corpus_hash_v1`
(stays at `35c276c7…`); does NOT alter `corpus_hash_v2`
(stays at `f1d132eb…`); does NOT alter any FF.1 / FF.2 /
FF.3 / FF.4 / FF.5 / S1.3d / S1.3e hash; does NOT mutate
`SEED.len()` (stays at 54); does NOT change S1.3a / FF.2 /
FF.3 / S1.3d / S1.3e court decisions; does NOT generate
CUDA kernels; does NOT decide contraindications or
challenges (it only links them); does NOT modify the
registry crate.

CLI: `dsfb-corpus s1-3f-authority-chain [--json] [--out PATH]`,
`s1-3f-authority-chain-emit [--out-dir DIR]`,
`s1-3f-activation-binding [--json] [--out PATH]`,
`s1-3f-kernel-plan-binding [--json] [--out PATH]`. Bulk
artifacts:
`crates/dsfb-gpu-atlas-corpus/out/casefile_v2_authority_chain_v1.{txt,json}`,
`.../casefile_v2_activation_binding_v1.{txt,json}`,
`.../casefile_v2_kernel_plan_binding_v1.{txt,json}`.

Next campaign: **S1.3g — OTelBindingReceiptTypes** (landed;
see the S1.3g section below).

## S1.3g — OTelBindingReceiptTypes (deterministic receipt-only schema for mapping OpenTelemetry spans, metrics, logs, and resources into EvidenceDensor fields without yet ingesting them)

**Status**: SEALED. Post-S1.3f plan directive (2026-05-17).

S1.3g defines deterministic receipt types for mapping
OpenTelemetry spans, metrics, logs, and resources into
`EvidenceDensor` fields. It is **receipt-only**: it does NOT
ingest live OTLP streams, run collectors, open sockets,
depend on an OTel SDK, or claim runtime interoperability.

Core rule (plan-locked):

- **Mapping is not ingestion.**
- **Receipt type is not adapter.**
- **Binding schema is not telemetry collection.**

S1.3g closes the S1.3 series with the OTel mapping contract
every future ingest commit MUST satisfy. It produces four
per-signal binding records plus a top-level wrapper:

- `SpanToEvidenceDensorBindingV1` — declares laws for
  trace_id / span_id identity, timestamp, duration,
  service.name, operation / span.name, status_code, error
  flag, and attribute ordering. Maps onto 8 `EvidenceDensor`
  fields.
- `MetricToEvidenceDensorBindingV1` — declares laws for
  metric_name (canonical lowercase NFC UTF-8), unit (UCUM
  unit codes), temporality (Cumulative / Delta / Gauge wire
  enum), timestamp, and attribute ordering. Maps onto 5
  `EvidenceDensor` fields.
- `LogToEvidenceDensorBindingV1` — declares laws for
  timestamp, severity (OTel SeverityNumber 1-24 +
  SeverityText override), body hash (SHA-256 over canonical
  body bytes; the receipt carries the hash, never the
  body), and attribute ordering. Maps onto 3 `EvidenceDensor`
  fields.
- `ResourceToEvidenceDensorBindingV1` — declares laws for
  resource identity (service.name + service.instance.id +
  service.version + host.id, canonical lowercase NFC),
  timestamp (resource snapshot capture), and attribute
  ordering. Maps onto 5 `EvidenceDensor` fields.
- `OTelBindingReceiptTypesV1` (top-level) — wraps the four
  per-signal bindings + `corpus_hash_v1` + `SEED.len()` so
  one hash pins the entire S1.3g mapping contract.

Every binding's `admits_live_ingestion` and
`depends_on_otel_sdk_runtime` flags are `false` by
construction; the verifier rejects any binding that flips
either to `true` or that names a forbidden live-ingestion /
SDK-runtime / stale-S1.3a substring in any law string.

Five new own-namespace hashes:

- `otel_span_binding_hash_v1 = 09b83355...` under
  `DSFB-GPU-ATLAS:OTEL-SPAN-BINDING:v1\0`.
- `otel_metric_binding_hash_v1 = 52d46e86...` under
  `DSFB-GPU-ATLAS:OTEL-METRIC-BINDING:v1\0`.
- `otel_log_binding_hash_v1 = 2466a899...` under
  `DSFB-GPU-ATLAS:OTEL-LOG-BINDING:v1\0`.
- `otel_resource_binding_hash_v1 = 82a7f2c9...` under
  `DSFB-GPU-ATLAS:OTEL-RESOURCE-BINDING:v1\0`.
- `otel_binding_receipt_hash_v1 = 0cebab9f...` under
  `DSFB-GPU-ATLAS:OTEL-BINDING-RECEIPT:v1\0`.

TEN plan-required load-bearing negatives pin the contract:
`s13g_rejects_binding_without_timestamp_law`,
`s13g_rejects_metric_binding_without_unit_or_temporality_law`,
`s13g_rejects_span_binding_without_trace_or_span_identity_law`,
`s13g_rejects_log_binding_without_body_hash_or_severity_law`,
`s13g_rejects_resource_binding_without_resource_identity_law`,
`s13g_rejects_binding_that_claims_live_ingestion` (rejects
via the boolean flag AND via a case-insensitive substring
scanner that looks for forbidden phrases in any law string),
`s13g_rejects_binding_that_depends_on_otel_sdk_runtime`
(same dual-vector rejection),
`s13g_rejects_stale_s13a_otel_binding_references` (rename-
discipline enforcer: the post-T.11h next-arc sequence
renamed the slot `"S1.3a OTel binding"` to `"S1.3g
OTelBindingReceiptTypes"`),
`s13g_rejects_nondeterministic_attribute_ordering` (every
attribute-ordering law string must contain the canonical
substring `"sorted ascending by attribute key"`),
`s13g_rejects_binding_without_evidence_densor_field_mapping`.

Plan-locked verdict (verbatim): *"S1.3f binds court
authority into CaseFileV2; S1.3g defines how external OTel
telemetry can be mapped into EvidenceDensor fields without
yet ingesting it."*

**Plan-locked non-claims**: S1.3g does NOT ingest live
OTLP streams; does NOT run collectors, agents, or sidecars;
does NOT open sockets; does NOT depend on an OTel SDK; does
NOT claim runtime interoperability with the OTel reference
implementation; does NOT emit detector outputs / witness
records / fusion tensors / candidate intervals / episodes;
does NOT alter `corpus_hash_v1` (stays at `35c276c7…`); does
NOT alter any FF.x / S1.3d / S1.3e / S1.3f hash; does NOT
mutate `SEED.len()` (stays at 54); does NOT change S1.3a /
FF.2 / FF.3 / S1.3d / S1.3e / S1.3f court decisions; does
NOT decide contraindications or challenges; does NOT modify
the registry crate.

CLI: `dsfb-corpus s1-3g-binding [--json] [--out PATH]`,
`s1-3g-binding-emit [--out-dir DIR]`,
`s1-3g-span-binding [--json] [--out PATH]`,
`s1-3g-metric-binding [--json] [--out PATH]`,
`s1-3g-log-binding [--json] [--out PATH]`,
`s1-3g-resource-binding [--json] [--out PATH]`. Bulk artifacts
(10 files):
`crates/dsfb-gpu-atlas-corpus/out/otel_binding_receipt_v1.{txt,json}`,
`.../otel_span_binding_v1.{txt,json}`,
`.../otel_metric_binding_v1.{txt,json}`,
`.../otel_log_binding_v1.{txt,json}`,
`.../otel_resource_binding_v1.{txt,json}`.

The S1.3 series (S1.3a–g) is now complete. Next campaign:
the densorial / tekmeric inference + CUDA Evidence Factory
front-door identity commit per the post-T.12.consolidate
scaling + framing roadmap.

## Performance — scaling ladder (D128 + D205 separately committed; D512+ deferred)

The detector ladder extends beyond the D64 headline as a separate
scaling proof. **D128** (16 motifs × 8 variants = 128 detectors,
[u64; 32] wide mask using words 0..2) landed at commit
`99a0f3b` as **R.9.d.1**, a scaling-ladder proof of the registry /
kernel / case-file path handling wider profiles. The D128 throughput
path deliberately omits R.10b compact-pack and digests the full
264-byte wide stride; this is the honest wide-digest baseline, NOT
the R.13 headline performance number. Bridge invariants verified
by 10 acceptance tests in
[tests/r9_d_d128_acceptance.rs](crates/dsfb-gpu-debug-demo/tests/r9_d_d128_acceptance.rs):
D128.V0 mask equals canonical D16, D128 OR ⊇ D64 OR ⊇ canonical
D16, D128 episodes are bank-admitted, D128 registry hash is
distinct from D16 and D64.

**D205** (16 motifs × 13 variants = 208 candidate slots; 205
active, the top 3 reserved-not-fired) landed as **R.9.d.2** (CPU)
and **R.9.d.2.1** (GPU byte-equivalence completion). The "205"
canonical name mirrors the dsfb-debug mature 205-detector
taxonomy count — the bridge to DSFB-Debug's mature detector
identity. **D205 is a scaling-ladder proof, NOT a new R.13
headline.** R.9.d.2 ships the CPU wide-mask evaluator + 15
bridge-invariant tests; R.9.d.2.1 adds the GPU kernels
(`detector_motif_kernel_wide_d205`,
`consensus_grid_kernel_wide_d205`,
`candidate_pack_kernel_wide_d205`) plus a Rust dispatch wrapper
`build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact`
and 12 GPU acceptance tests in
[tests/r9_d2_1_d205_gpu_acceptance.rs](crates/dsfb-gpu-debug-demo/tests/r9_d2_1_d205_gpu_acceptance.rs).
Bridge invariants verified at CPU level in
[tests/r9_d2_d205_acceptance.rs](crates/dsfb-gpu-debug-demo/tests/r9_d2_d205_acceptance.rs):
D205 V0..V7 firings byte-identical to D128 V0..V7; D205 V0-only
projection equals canonical D16; D205 OR ⊇ D128 OR ⊇ D64 OR ⊇
canonical D16; high bits ≥ 205 deterministically zero; D205
registry hash distinct from D16 / D64 / D128. GPU-side
invariants pinned: D205 replay deterministic across two GPU
runs; D205 case file binds to `DetectorProfile::D205.registry_hash()`;
D205 episodes bank-admitted (Semantic Non-Bypass); D205 dispatch
does not perturb D64 or D128 paths; D205 episodes carry
`detector_bit_count <= 16` (the canonical 16-motif basis). The
GPU detector tree-digest hashes the full 264-byte wide stride;
R.10b compact-pack for D205 is deferred to a future commit. The
ladder now reaches the dsfb-debug mature 205-detector count with
CPU + GPU byte-equivalent execution paths at D16 / D64 / D128 /
D205.

**Deferred to paper §16** (do NOT mistake these for current state):

- D512 / D1024 / D2000 detector-ladder scaling
- D128 / D205 compact-pack (R.10b-style for the wide profile)
- HIP / ROCm port
- Multi-GPU deterministic sharding
- OpenTelemetry span ingestion
- Cross-architecture replay matrix (sm_75 / sm_80 / sm_86 /
  sm_89 / sm_90 byte-equivalence verification)
- CPU D64 wide-path comparator (R.12b.1) — needed to convert the
  campaign-reduction story into a clean GPU-vs-CPU speedup column
- DSFB-GPU-Atlas continuation (S Phase 1+)

Reproducing the headline:

```
cargo test --release --features cuda -p dsfb-gpu-debug-demo \
  --test r12_d64_saturation -- --nocapture
```

This refreshes both `reports/r12_d64_saturation.txt` (full sweep)
and the underlying numbers feeding `reports/money_table.txt`.
The session writes `graph_status` and `graph_plan_hash` to the
report when CUDA Graph capture succeeds.

## Reproducibility receipt

The canonical fixture (LCG seed `0xD5FBD5FBD5FBD5FB`) under
`Contract::canonical()` with the canonical 8-motif bank and 16-detector
registry produces these stage hashes:

| stage              | SHA-256 |
|--------------------|---------|
| input_catalog      | `1eeefffa2a1029672a9e6a55e575c928fca926e8077e6784a392597ccd640487` |
| contract           | `4c2b473e660d5034b39eb68be28353ec04e31b4b002d354541d12546ca3566b5` |
| bank               | `497a40c495b7c70e65086770a3575569c31c28249e400f6cd3787e8d97216506` |
| detector_registry  | `72c0cde2732087a89d7369825780a90bfb3278a8f055122ac5a500752a05535d` |
| kernel_sequence    | `77117e5d74f1b45168be90159a517d9629fa324c0c97bc902865bfd903041960` |
| window_feature     | `4104ba351e5e23aef861b4cc1e32b4208feee378111682a69caf0044213e67f9` |
| residual_field     | `52578099ee75e1213372d9128402401433a063e5576a533ee605abe128247ef7` |
| sign_field         | `e2609c77de5f1b16378a9c6953735111ae8469877932fd5ebb370f6339b234df` |
| detector_cell      | `c4fbabbd807574bd22ae9bcfef6e776a1cb8071194c2f6f3983df79dda4b7d26` |
| consensus_grid     | `4b5f3ba7229bcb9c8c677c3228799ddb48b170f3032d6129fb8200f71662a9c8` |
| candidate_interval | `ae6036982dd46a90b210fee29aed121bf6b05ce5bd90fe3480cc5e7f025a7296` |
| episode            | `5498bb237244c161867a2883eb6b02001536727b41b8a4396c05a79a04c3958a` |
| final case file    | `e895db09b77189e0b362c7647638cf79b1ebc53c82dca238aed2755a6c45272c` |

These values are pinned in `tests/golden_hashes.rs` as a literal test
fixture. Any contract change that would alter one of them must update
the corresponding constant in that file in the same commit; otherwise
the test fails.

The `candidate_interval`, `episode`, and `final case file` hashes
above were refreshed in Section R's R.5 commit when `CandidateInterval`
gained the `entity_avg_q` and `grid_avg_q` axis-5 locality fields. The
first nine chain links (`input_catalog` through `consensus_grid`) are
unchanged from the prior receipt — only the candidate bytes and
everything downstream of them re-pin.

Verified on RTX 4080 SUPER (Ada, sm_89) / CUDA 13.2: the GPU pipeline
produces byte-identical hashes at every stage and an identical
admitted-episode list (15 episodes: `LatencyRamp` on entity 3,
`ErrorBurst` on entity 7, `SlewShockRecovery` on entity 11, plus
several `OscillationInstability` motifs from secondary-cell ringing).

## License

Apache 2.0 (reference implementation). Background IP: Invariant Forge LLC.
Commercial deployment requires separate written license.
Contact: licensing@invariantforge.net
