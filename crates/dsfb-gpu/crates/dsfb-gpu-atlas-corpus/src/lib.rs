//! Literature detector corpus and Canonicalisation Court for DSFB-GPU-Atlas.
//!
//! **Front-door identity (panel-locked)**: DSFB-GPU-Atlas implements
//! **densorial / tekmeric inference** --- deterministic inference over
//! evidence densors plus evidence-based deterministic adjudication ---
//! not neural inference over learned-weight tensors. This crate is the
//! court half of that posture: a provenance-bound, deduplicated,
//! deterministic jurisprudence system over every detector witness the
//! literature has named. The GPU side (the
//! [`dsfb-gpu-debug-cuda`](../../dsfb_gpu_debug_cuda/index.html) crate)
//! is the byte-exact evidence factory; this corpus crate is the court
//! that decides what the evidence is allowed to mean.
//!
//! ```text
//! neural inference:
//!   tensor → learned weights → probabilistic output
//!
//! densorial / tekmeric inference:
//!   densor → deterministic witness court → replayable case file
//! ```
//!
//! This crate is the formal bridge from DSFB-GPU-Debug (the R.9-R.13
//! sealed prior-art proof) to DSFB-GPU-Atlas. It is not a detector
//! library: it is a provenance-bound, deduplicated, deterministic
//! jurisprudence system for every detector witness the literature has
//! named. The panel verdict explicitly named this as a higher-leverage
//! campaign than further detector-count scaling (D205+), because the
//! moat being built is not "more detectors" but "a defensible court
//! around the detector functions."
//!
//! **Current state through S-PERF.11.1** (each T-section /
//! S-section landed as its own atomic commit under the
//! 10-step per-section discipline ritual):
//!
//! * **T.1a / T.1b** ([`types`], [`seed`], [`verify`]): the full
//!   structural schema and a 54-record literature seed across the
//!   panel-named source classes (statistical process control,
//!   sequential change detection, drift, robust statistics,
//!   distribution distance, information theory, signal / spectral,
//!   time-series structure, fault detection, graph anomaly,
//!   debug / observability, ...). The `verify` walker rejects any
//!   record missing required fields.
//! * **T.2** ([`toml_parser`], [`loader`], [`dump`]): TOML
//!   source-ingestion format with a hand-rolled subset parser
//!   plus a dumper. The single-file `corpus/corpus.toml` mirrors
//!   the static seed byte-for-byte; equivalence tests pin
//!   round-trip identity.
//! * **T.3** ([`identity`]): five-hash detector identity (source,
//!   formula, parameter, implementation, semantic-role) plus the
//!   composite `detector_identity_hash = SHA256(domain || formula
//!   || parameter || semantic_role)`. Source and implementation
//!   are deliberately omitted from the composite so corpus citation
//!   fixes and L-band upgrades do not break equivalence classes.
//! * **T.4** ([`court`], [`claims`]): deterministic dedup court
//!   over the 54 seed records plus 12 alias claims plus 2
//!   composition canonicals. The court emits one of nine explicit
//!   `CanonicalisationDecision` variants per subject with a reason
//!   code; no fuzzy similarity scoring.
//! * **T.5** ([`genealogy`]): deterministic detector genealogy
//!   graph over already-admitted court records. Seven edge kinds
//!   (DerivedFrom / Generalizes / SpecialCaseOf / ParameterVariantOf
//!   / DomainTransferOf / Composes / AliasCollapsedInto), DAG
//!   verifier on strict-ancestry edges, DOT and JSON exports under
//!   the `DSFB-GPU-ATLAS:GENEALOGY:v1` schema.
//! * **T.6** ([`fusion`]): witness-role and fusion-axis semantics.
//!   Eight `FusionPlane` variants, a deterministic
//!   `axes_to_planes` mapping from the v1 9-axis fusion onto the
//!   8 panel-locked planes, and a declarative `COMPATIBILITY_RULES`
//!   table covering Confuser/Primary/CleanWindow/Corroborating
//!   pair semantics (Confuser suppresses Primary; CleanWindow
//!   incompatible with Primary; etc.).
//! * **T.7** ([`lband`]): implementation-status (L-band) ladder
//!   verifier. `GPU_IMPLEMENTED_CANONICAL_IDS` whitelists the
//!   five dsfb-gpu-debug-core bank surface IDs (14, 15, 41, 42,
//!   43). The verifier rejects L5/L6 records not in the whitelist,
//!   L7 records (no benchmark artifact yet), and L8 records
//!   (still gated until measured usefulness-ledger evidence
//!   exists — see [`lband`] for the precise post-T.8 wording).
//!   **L-band is an honesty marker, not a quality score.**
//! * **T.8** ([`usefulness`]): deterministic detector usefulness
//!   ledger shell. The embedded
//!   [`types::UsefulnessLedgerSnapshot`] (zero-init on every
//!   record) is the per-detector prior summary; the new
//!   [`usefulness::UsefulnessLedgerRow`] is the richer
//!   per-(detector × task × dataset) ledger keyed by
//!   `(canonical_id, task_id, domain, dataset_id)` with a
//!   [`usefulness::UsefulnessEvidenceLevel`] honesty ladder
//!   (`Unmeasured` / `LiteraturePrior` / `RoleSeeded` / measured
//!   variants). The conservative T.8 seed marks every row
//!   `NotScored`; nothing in the corpus claims empirical
//!   usefulness until a row is backed by a named benchmark
//!   artifact. **The Atlas records usefulness; it never learns
//!   it.**
//! * **T.9** ([`audit_report`]): internal corpus audit bundle.
//!   Renders `corpus_t9_audit_report.{txt,json}` plus refreshed
//!   genealogy DOT/JSON for the operator-facing audit; intended
//!   as an internal review artifact, not a publication.
//! * **T.10** ([`corpus_hash`]): `corpus_hash_v1` — canonical
//!   SHA-256 over the sorted T.1–T.8 record bytes under domain
//!   separator `DSFB-GPU-ATLAS:LITERATURE-CORPUS:v1\0`. Two
//!   builds against the same SEED produce byte-identical hashes;
//!   this anchor binds future Atlas surfaces (S1.2 registry,
//!   T.11 court layer, etc.) to the corpus.
//! * **S1.2 (registry, separate crate)**: literature-bound
//!   `registry_hash_v2` over 54 corpus primitives × 3-point
//!   parameter grid = 162 `DetectorSpec` records. Every spec
//!   carries `source_corpus_hash = compute_corpus_hash_v1()`,
//!   `CorpusBindingStatus::HashFrozenT10`,
//!   `ImplementationKind::ScalarCpu` (honesty rule). The 2,000-
//!   detector grid is a deliberate follow-on (S1.2.x+).
//! * **T.11a** ([`passport`]): `DetectorPassport` — per-detector
//!   legal-identity packet. One passport per canonical SEED
//!   record bundling identity hashes (T.3), dedup decision (T.4),
//!   genealogy edges (T.5), witness role (T.6), L-band (T.7),
//!   usefulness evidence level (T.8), lifecycle state, and
//!   constitution flags into a hashable record under domain
//!   separator `DSFB-GPU-ATLAS:DETECTOR-PASSPORT:v1\0`.
//! * **T.11b** ([`precedent`]): `CourtPrecedent` ledger —
//!   cumulative jurisprudence over T.4 dedup decisions
//!   (`ROBUST_Z_MAD_ALIAS_COLLAPSE`, `PCA_SPE_Q_EQUIVALENCE`,
//!   `WESTERN_ELECTRIC_COMPOSITION_OF_SHEWHART`, ...). Hash
//!   `precedent_hash_v1` under domain
//!   `DSFB-GPU-ATLAS:COURT-PRECEDENT:v1\0`.
//! * **T.11c** ([`admissibility`]):
//!   `AdmissibilityGrammarSnapshot` — versioned grammar of
//!   admissible episode forms. 9 `EpisodeAdmissibilityRule`
//!   records + 9 `ConfuserSuppressionRule` records, each citing
//!   at least one T.11b precedent. Plus a passport ↔ grammar
//!   crosswalk artifact (panel-locked: NOT a passport field, so
//!   passport hashes do not churn).
//! * **T.11d** ([`trial_transcript`]): `TrialTranscriptV1` —
//!   minimal real CaseFileV2 trial-transcript body for the
//!   panel-locked synthetic LatencyRamp fixture. The court
//!   stops issuing abstract receipts and starts showing real
//!   trial records.
//! * **T.11e** ([`execution_attestation`]):
//!   `ExecutionAttestationReceiptV1` — unsigned, local,
//!   DSFB-native execution attestation. Records repo commit,
//!   rustc/cargo/nvcc versions, gate cleanliness, workspace
//!   test totals, all hash-chain anchors, and 7 panel-locked
//!   non-claims (NOT SLSA / in-toto / signed / third-party
//!   verified / reproducible-builds.org compliant).
//! * **T.11f** ([`challenge_docket`]): `ChallengeDocketV1` —
//!   the court's adversarial self-audit overlay. 10-entry
//!   conservative seed (0 Open, 0 Sustained, 6 Overruled, 4
//!   Deferred); 17 verifier reject kinds. Records objections
//!   against detector identities, precedent judgments, grammar
//!   rules, trial transcripts, execution receipts, and corpus
//!   / registry globals. **Does NOT mutate any upstream
//!   surface**; sustaining a challenge requires a SEPARATE
//!   later commit.
//! * **T.11g** ([`contraindication`]):
//!   `DetectorContraindicationReceiptV1` — the court's
//!   datasheet / model-card / safety-label layer. Nine
//!   panel-locked questions per detector (works-best-when,
//!   fails-when, known-confusers, required-sampling-law,
//!   required-units, minimum-support, do-not-use-for,
//!   closest-aliases, closest-non-aliases) plus an
//!   adversarial-twin layer. 11 verifier reject kinds. Passport
//!   ↔ contraindication crosswalk in a separate namespace so
//!   passport hashes do not churn.
//! * **T.11h** ([`coverage_holes`]):
//!   `CoverageHoleReportV1` — audit-only honesty report over the
//!   sealed T.1–T.11g surfaces. Seven panel-locked buckets
//!   (DetectorCoverage / WitnessLawCoverage /
//!   ImplementationCoverage / SemanticsCoverage /
//!   JurisprudenceCoverage / SourceProvenanceCoverage /
//!   ReasonCodeCoverage) under domain
//!   `DSFB-GPU-ATLAS:COVERAGE-HOLES:v1\0`. Headline metric is
//!   per-surface Reason-Code Coverage (100% by construction
//!   across every T.11 surface). **Does NOT mutate any upstream
//!   hash or repair any hole**; it only surfaces them.
//! * **S1.3a** ([`activation`]): `ActivationPlanV1` — the first
//!   deterministic court decision over the sealed T.11 stack.
//!   Per-detector reason-coded `Enabled` / `Disabled` /
//!   `WarnOnly` / `Deferred` decisions; consumes
//!   `DetectorPassport` (T.11a), `CourtPrecedent` (T.11b),
//!   `AdmissibilityGrammar` (T.11c), `ChallengeDocket` (T.11f),
//!   `ContraindicationReceipt` (T.11g), `CoverageHoleReport`
//!   (T.11h), `corpus_hash_v1` (T.10), and `registry_hash_v2`
//!   (S1.2). Emits `activation_plan_hash_v1` under domain
//!   `DSFB-GPU-ATLAS:ACTIVATION-PLAN:v1\0` plus enabled /
//!   disabled reason histograms and per-decision citation lists
//!   (blocking + warning receipt hashes, cited challenge /
//!   contraindication / coverage-hole IDs).
//! * **S1.3b** ([`activation_audit`]): the explanation + diff
//!   court built on top of S1.3a. `ActivationDecisionTranscript`
//!   surfaces, per detector, the full reason route (passport →
//!   L-band → coverage hole → contraindication → challenge →
//!   final decision) with categorical contributing facts,
//!   ordered blocking chain, and a counterfactual path to
//!   Enabled. `ActivationDiffV1` is the structural two-plan diff
//!   (court-level, not byte-level) with five categorical
//!   change kinds. Two own-namespace hashes:
//!   `activation_decision_transcript_hash_v1` under
//!   `DSFB-GPU-ATLAS:ACTIVATION-TRANSCRIPT:v1\0` and
//!   `activation_diff_hash_v1` under
//!   `DSFB-GPU-ATLAS:ACTIVATION-DIFF:v1\0`.
//! * **S1.3c** ([`activation_context`]): TaskManifestV1 +
//!   DatasetManifestV1 + ActivationContextV1 — bind activation
//!   decisions to a declared task, domain, schema, units,
//!   sampling law, and artifact fixedness contract. The
//!   verifier enforces 11 panel-locked rules including fixed-
//!   artifact source-hash requirements, time-series timestamp-
//!   law requirements, and per-detector spectral / unit-
//!   sensitive activation crosschecks. Three own-namespace
//!   hashes: `task_manifest_hash_v1` under
//!   `DSFB-GPU-ATLAS:TASK-MANIFEST:v1\0`,
//!   `dataset_manifest_hash_v1` under
//!   `DSFB-GPU-ATLAS:DATASET-MANIFEST:v1\0`, and
//!   `activation_context_hash_v1` under
//!   `DSFB-GPU-ATLAS:ACTIVATION-CONTEXT:v1\0`. Conservative
//!   seed task/dataset manifests provided for the
//!   DSFB-GPU-Debug fixture family. **Schema + verifier +
//!   seed only**; budget pruning (S1.3d), kernel-plan emission
//!   (S1.3e), and CaseFileV2 integration (S1.3f) are deferred.
//! * **T.12.0** ([`amendment`]): CorpusAmendmentProposalV1 +
//!   CorpusExpansionBatch + DedupCourtDelta — the legal
//!   intake system for the T.12 Literature Corpus Scale-Out.
//!   New literature primitives will enter as reviewable
//!   amendment proposals (not silent mutations of
//!   `corpus_hash_v1`). Three new own-namespace hashes:
//!   `literature_expansion_batch_hash_v1` under
//!   `DSFB-GPU-ATLAS:LITERATURE-EXPANSION-BATCH:v1\0`,
//!   `corpus_amendment_proposal_hash_v1` under
//!   `DSFB-GPU-ATLAS:CORPUS-AMENDMENT-PROPOSAL:v1\0`, and
//!   `dedup_court_delta_hash_v1` under
//!   `DSFB-GPU-ATLAS:DEDUP-COURT-DELTA:v1\0`. T.12.0 ships
//!   the schema, the 23-variant `SourceClass` enum, a verifier
//!   covering 7 panel-locked rules + 2 structural integrity
//!   checks, and an empty proof-of-life proposal.
//! * **T.12.a** ([`t12_a_spc`]): the first real corpus
//!   expansion proposal — Statistical Process Control.
//!   Proposes 2 new canonical primitives (MEWMA, MCUSUM;
//!   reserved canonical ids 5001/5002 above SEED's 54-record
//!   range), 3 alias collapses (Q statistic / Squared
//!   Prediction Error → PCA_SPE_Q_RESIDUAL; Hotelling
//!   T-square → HOTELLING_T2), and 2 court-level composition
//!   reclassifications (Western Electric / Nelson rules
//!   → `CompositionOf`). Page-Hinkley is left to T.12.b's
//!   authority (sequential-change-detection-adjacent;
//!   already canonical in SEED at id 4, not touched here).
//!   `status = Open` pending review. **Does NOT mutate
//!   SEED** (`SEED.len()` stays at 54); a future formal
//!   freeze campaign produces `corpus_hash_v2` once the
//!   court has ratified.
//! * **T.12.b** ([`t12_b_scd`]): the second real corpus
//!   expansion proposal — Sequential Change Detection. Proves
//!   **cross-class dedup authority** (not detector quantity).
//!   Admits FOUR genuinely new canonical primitives
//!   (Shiryaev-Roberts, GLR, Binary segmentation, PELT-style
//!   deterministic) at reserved canonical ids 5201, 5202, 5207,
//!   5208. Records SEVEN `ExistingCanonicalAuthorityResolution`
//!   decisions keeping CUSUM (SEED id 3), Page-Hinkley (id 4),
//!   Mann-Kendall (id 11), Pettitt (id 34), SNHT (id 35), MOSUM
//!   (id 36), Buishand range (id 37) canonical without
//!   duplication. Records ONE `DomainTransferOf` decision
//!   naming CUSUM as the shared SCD ancestor. Rejects BOCPD
//!   (Adams & MacKay 2007) as `RejectedNotDeterministic` at
//!   reserved id 5209 — BOCPD appears in `proposed_primitives`
//!   but NOT in `new_canonical_records`. Stream-drift detectors
//!   (ADWIN, DDM, EDDM, HDDM, KSWIN) are explicitly deferred to
//!   T.12.c. `status = Open` pending review. **Does NOT mutate
//!   SEED** (stays at 54); the T.12.b SCD
//!   `corpus_amendment_proposal_hash_v1` is distinct from
//!   T.12.0's proof-of-life hash and T.12.a's SPC hash.
//! * **T.12.c** ([`t12_c_drift`]): the third real corpus
//!   expansion proposal — Drift Detection and Distribution-
//!   Distance Authority. Proves **cross-class dedup authority
//!   at scale**: a walk of SEED found ELEVEN distribution-
//!   distance primitives already canonical (KS id 8, KL 9, MMD
//!   10, Anderson-Darling 26, Cramer-von Mises 27, Wasserstein
//!   28, Energy distance 29, Hellinger 30, PSI 31, Jensen-
//!   Shannon 32, Total variation 33). All eleven are recorded
//!   as `ExistingCanonicalAuthorityResolution` records. FOUR
//!   genuinely new canonicals at reserved ids 5301..=5304
//!   (Kuiper, ADWIN, DDM, HDDM) are admitted as
//!   `CanonicalAddition` with declared deterministic contracts
//!   (reference-distribution / window-pair / Hoeffding-delta
//!   / cut-rule / numeric mode). ONE `DomainTransferOf`
//!   names KS (SEED id 8) as the shared two-sample
//!   distribution-distance ancestor. TWO `ParameterizationOf`
//!   records (EDDM of DDM, KSWIN of KS) document streaming /
//!   family variants that appear in `proposed_primitives` but
//!   NOT in `new_canonical_records`. The new wire-name
//!   category `ParameterizationOf` lands for the first time.
//!   `status = Open` pending review. **Does NOT mutate SEED**
//!   (stays at 54); the T.12.c drift
//!   `corpus_amendment_proposal_hash_v1` is distinct from
//!   every prior T.12.x proposal hash.
//! * **T.12.d** ([`t12_d_robust`]): the fourth real corpus
//!   expansion proposal — Robust Statistics. **First proposal
//!   to exercise ALL FIVE panel-locked court-delta categories**.
//!   SEED walk found THREE robust-statistics primitives already
//!   canonical (Robust z-score id 6, Hampel filter id 7, Tukey
//!   fences id 18); all three become
//!   `ExistingCanonicalAuthorityResolution` records. FOUR
//!   genuinely new canonicals at reserved ids 5401..=5404
//!   (Theil-Sen slope, biweight midvariance, trimmed mean shift,
//!   winsorized mean shift) are admitted as `CanonicalAddition`
//!   with declared estimator-law contracts (pair-selection law /
//!   tuning constant + convergence / trim fraction / winsor
//!   limit). ONE `DomainTransferOf` names robust-z (SEED id 6)
//!   as the shared robust-location-estimator ancestor. THREE
//!   `ParameterizationOf` records — modified z-score
//!   (ParameterizationOf robust-z), rolling Hampel
//!   (ParameterizationOf Hampel), k×IQR fence
//!   (ParameterizationOf Tukey fences) — at reserved ids
//!   5405..=5407. ONE `RejectedNotDeterministic` for RANSAC
//!   residual proxy (Fischler & Bolles 1981) at reserved id
//!   5408 — randomized in origin; admitted neither to SEED nor
//!   to `new_canonical_records` unless a future T.12.x
//!   proposal admits a `Deterministic_RANSAC_Proxy` canonical
//!   with sample seed, iteration budget, fixed sample schedule,
//!   tie-break law, and numeric mode declared. The wire-name
//!   set across CanonicalAddition,
//!   ExistingCanonicalAuthorityResolution, DomainTransferOf,
//!   ParameterizationOf, and RejectedNotDeterministic is now
//!   closed at five panel-locked categories. `status = Open`
//!   pending review. **Does NOT mutate SEED** (stays at 54);
//!   the T.12.d robust `corpus_amendment_proposal_hash_v1` is
//!   distinct from every prior T.12.x proposal hash.
//! * **T.12.e** ([`t12_e_spectral`]): the fifth real corpus
//!   expansion proposal — Signal Processing / Spectral /
//!   Wavelet. Panel-locked warning: *"In spectral detectors,
//!   the transform law is the detector. No transform law, no
//!   canonical admission."* SEED walk found FIVE
//!   signal / spectral primitives already canonical: FFT band-
//!   energy anomaly (id 12), residual envelope exit (id 22),
//!   spectral entropy (id 38), wavelet coefficient energy
//!   (id 39), autocorrelation-coefficient break (id 40). All
//!   five become `ExistingCanonicalAuthorityResolution`. SIX
//!   genuinely new canonicals at reserved ids 5501..=5506
//!   (spectral centroid shift, wavelet packet energy, STFT
//!   ridge shift, cepstral anomaly, matched filter residual,
//!   Hilbert amplitude anomaly) are admitted as
//!   `CanonicalAddition` with declared transform-law contracts
//!   (window function + normalization + band / packet-tree
//!   depth + ridge selection law + FFT convention + template
//!   provenance + analytic-signal extraction). ONE
//!   `DomainTransferOf` names FFT band-energy as the shared
//!   spectral-transform ancestor. THREE `ParameterizationOf`
//!   records — FFT bandpower variant (5507) of SEED 12;
//!   wavelet family variant (5508) of SEED 39; STFT window/hop
//!   variant (5509) of 5503. ONE `RejectedNotDeterministic`
//!   for randomized spectral projection (Rahimi & Recht 2007
//!   random Fourier features and family) at reserved id 5510 —
//!   admission requires seed + projection matrix definition +
//!   dimension + numeric mode declared. `status = Open`
//!   pending review. **Does NOT mutate SEED** (stays at 54);
//!   the T.12.e spectral `corpus_amendment_proposal_hash_v1`
//!   is distinct from every prior T.12.x proposal hash.
//! * **T.12.f** ([`t12_f_timeseries`]): the sixth real corpus
//!   expansion proposal — Time-Series Structure / Control
//!   Residuals. Panel-locked warning: *"A model is not a
//!   detector until the residual and decision law are
//!   declared."* SEED walk found FOUR primitives already
//!   canonical (sensor bias 23, actuator stiction 24, valve
//!   hunting 25, Error burst 41), plus two recognised in
//!   T.12.e (residual envelope exit 22, autocorrelation break
//!   40) recognised again here under TimeSeriesStructure.
//!   EIGHT genuinely new canonicals at reserved ids
//!   5601..=5608 (AR residual / ARIMA residual / STL residual /
//!   lag-correlation break / variance-ratio shift / run-length
//!   anomaly / observer residual / parity-space residual)
//!   admitted as `CanonicalAddition` with declared model-and-
//!   decision-law contracts (model order + fit law + residual
//!   definition + threshold + envelope law + sampling /
//!   window-pair / lag-range / parity equations + state model +
//!   observer gain). ONE `DomainTransferOf` names residual
//!   envelope exit (SEED 22) as the shared residual-witness
//!   ancestor for TimeSeriesStructure. THREE `ParameterizationOf`
//!   records: innovation sequence (5609) of observer residual
//!   (5607); periodicity break (5610) of lag-correlation break
//!   (5604); burstiness index (5611) of Error burst (SEED 41).
//!   ONE `RejectedNotDeterministic` for unidentified-model
//!   anomaly at reserved id 5612 — any "ARIMA with auto-
//!   determined order via random search", "Kalman without
//!   declared Q / R covariances", or "STL with adaptive
//!   seasonality" requires the model-order-search seed +
//!   identification algorithm + fit-data anchor + tie-break
//!   law + numeric mode declared. `status = Open` pending
//!   review. **Does NOT mutate SEED** (stays at 54); the T.12.f
//!   time-series `corpus_amendment_proposal_hash_v1` is
//!   distinct from every prior T.12.x proposal hash.
//! * **T.12.g** ([`t12_g_graph`]): the seventh real corpus
//!   expansion proposal — Graph / Topology Anomaly. Panel-
//!   locked warning: *"A graph metric is not a detector until
//!   the baseline, update law, metric law, and decision law
//!   are declared."* SEED walk found ONE graph-adjacent
//!   primitive already canonical (Fanout cascade 43); the
//!   corpus is graph-anomaly-sparse. EIGHT genuinely new
//!   canonicals at reserved ids 5701..=5708 (degree spike,
//!   betweenness shift, clustering-coefficient shift, PageRank
//!   residual, edge-cut anomaly, bridge-node emergence,
//!   cascade precursor, motif-count anomaly) admitted as
//!   `CanonicalAddition` with declared graph-model, baseline,
//!   update-law, metric-law, and decision-law contracts. One
//!   `DomainTransferOf` names Fanout cascade (SEED 43) as the
//!   shared cascade ancestor. Three `ParameterizationOf`
//!   records: weighted-degree spike (5709) of degree spike
//!   (5701); k-hop fanout (5710) of Fanout cascade (SEED 43);
//!   directed motif-count (5711) of motif-count anomaly (5708).
//!   Two `RejectedNotDeterministic` records — the **first
//!   T.12.x proposal with two rejections in one commit**:
//!   community boundary shift (5712, Louvain / Leiden /
//!   label propagation / Infomap are randomized; admission
//!   requires algorithm, seed, tie-break, modularity rule,
//!   resolution parameter, and convergence law declared);
//!   random-walk embedding anomaly (5713, DeepWalk / node2vec
//!   are randomized; admission requires walk seed, walk
//!   length, walk count, tie-break, embedding-projection
//!   matrix anchor, and numeric mode declared). `status = Open`
//!   pending review. **Does NOT mutate SEED** (stays at 54);
//!   the T.12.g graph `corpus_amendment_proposal_hash_v1` is
//!   distinct from every prior T.12.x proposal hash.
//! * **T.12.h** ([`t12_h_dataquality`]): the eighth real
//!   corpus expansion proposal — Data Quality / Tabular /
//!   Database Integrity Constraints. Panel-locked warning:
//!   *"A validation rule is not a detector until scope,
//!   baseline, null / type / key semantics, and decision law
//!   are declared."* SEED walk found FIVE data-quality
//!   primitives already canonical (Missingness spike 13,
//!   Missingness coupling 44, Schema drift 45, Cardinality
//!   drift 46, Uniqueness violation 47); all five become
//!   `ExistingCanonicalAuthorityResolution` records. EIGHT
//!   genuinely new canonicals at reserved ids 5801..=5808
//!   (functional dependency violation, type instability,
//!   target-leakage candidate, correlation break, covariance
//!   shift, null-run anomaly, tabular range envelope exit,
//!   category emergence) admitted as `CanonicalAddition` with
//!   declared scope, baseline, null-semantics, key-scope,
//!   type-system, range / unit, association-law, and
//!   decision-law contracts. ONE `DomainTransferOf` names
//!   Missingness spike (SEED 13) as the shared data-quality
//!   ancestor. THREE `ParameterizationOf` records:
//!   per-column missingness (5809), composite-key uniqueness
//!   (5810), category collapse (5811). TWO
//!   `RejectedNotDeterministic` records (second T.12.x with
//!   two rejections, following T.12.g): learned data-quality
//!   anomaly score (5812; autoencoder / Mahalanobis-with-
//!   learned-cov / Isolation Forest / LOF) and auto-schema
//!   inference anomaly (5813; TFDV-style / Great Expectations
//!   profiler with random sampling). Target-leakage candidate
//!   (5803) is admitted with the panel-locked non-claim:
//!   CANDIDATE witness, not proof of leakage. `status = Open`
//!   pending review. **Does NOT mutate SEED** (stays at 54);
//!   the T.12.h data-quality
//!   `corpus_amendment_proposal_hash_v1` is distinct from
//!   every prior T.12.x proposal hash.
//! * **T.12.i** ([`t12_i_observability`]): the ninth real
//!   corpus expansion proposal — Observability / Debugging.
//!   Panel-locked warning: *"An observability symptom is not
//!   a detector until the telemetry field, aggregation law,
//!   baseline, topology scope, and confuser semantics are
//!   declared."* SEED walk found FIVE observability primitives
//!   already canonical and protected from re-canonicalisation:
//!   the dsfb-gpu-debug-core L6 bank surface (Latency ramp 14,
//!   Single-window spike confuser 15, Error burst 41, Slew
//!   shock 42, Fanout cascade 43); all five become
//!   `ExistingCanonicalAuthorityResolution` records with their
//!   telemetry-field + aggregation-law + window-law +
//!   baseline + topology-scope + confuser-profile contracts
//!   declared. EIGHT genuinely new canonicals at reserved ids
//!   5901..=5908 (retry storm, queue-depth pressure, saturation
//!   precursor, cold-start transient, timeout burst, GC pause
//!   spike, thread-pool exhaustion, backpressure propagation)
//!   admitted as `CanonicalAddition`. TWO `DomainTransferOf`
//!   records name Fanout cascade and Error burst as shared
//!   ancestors for `ObservabilityDebugging`. FOUR
//!   `ParameterizationOf` records: HTTP 5xx burst (5909),
//!   p95 / p99 latency ramp (5910), k-hop dependency fanout
//!   (5911), retry-rate burst (5912). TWO
//!   `RejectedNotDeterministic` records (third T.12.x with
//!   two rejections, following T.12.g and T.12.h): vendor APM
//!   black-box anomaly score (5913; Datadog / New Relic /
//!   Dynatrace / Splunk MLTK / AWS DevOps Guru) and learned
//!   incident classifier (5914; PagerDuty / Splunk On-Call /
//!   ServiceNow AIOps). `status = Open` pending review. **Does
//!   NOT mutate SEED** (stays at 54); the T.12.i observability
//!   `corpus_amendment_proposal_hash_v1` is distinct from
//!   every prior T.12.x proposal hash.
//! * **T.12.j** ([`t12_j_biosignal`]): the tenth real corpus
//!   expansion proposal — Medical / Biosignal. Panel-locked
//!   warning: *"Count signal witnesses, not diagnoses. No
//!   sampling / filtering / morphology law, no canonical
//!   admission."* SEED walk found FOUR biosignal primitives
//!   already canonical (R-peak interval anomaly 49, HRV
//!   time-domain shift 50, QRS width anomaly 51, ST-segment
//!   deviation proxy 52); all four become
//!   `ExistingCanonicalAuthorityResolution` records with
//!   declared signal-source + sampling-rate + filtering-law +
//!   morphology-or-interval-measurement-law + baseline +
//!   artifact-confuser contracts. EIGHT genuinely new
//!   canonicals at reserved ids 6001..=6008 (P-wave morphology
//!   anomaly, T-wave morphology anomaly, QT interval anomaly,
//!   PR interval anomaly, spectral HRV band shift, baseline
//!   wander detector, motion artifact detector, saturation /
//!   clipping detector) admitted as `CanonicalAddition`. TWO
//!   `DomainTransferOf` records name FFT band-energy (SEED 12)
//!   as the shared spectral ancestor and Residual envelope
//!   exit (SEED 22) as the shared envelope-boundary ancestor
//!   for `MedicalBiosignal`. FOUR `ParameterizationOf` records:
//!   RR-interval irregularity (6009), HRV time-domain SDNN /
//!   RMSSD / pNN50 (6010), HRV LF / HF band-specific (6011),
//!   lead-specific ST deviation (6012). TWO
//!   `RejectedNotDeterministic` records (fourth T.12.x with
//!   two rejections, following T.12.g / T.12.h / T.12.i):
//!   learned arrhythmia classifier (6013; Hannun et al.\ 2019
//!   deep-learning ECG classifier) and clinician-label-only
//!   diagnostic rule (6014). Panel-locked non-claim: T.12.j
//!   does NOT admit medical diagnoses; it admits deterministic
//!   biosignal witnesses (morphology / interval / artifact /
//!   spectral signal structures) under declared sampling,
//!   filtering, and measurement laws — clinical
//!   interpretation remains out of scope, pinned by
//!   `t12_j_rejects_diagnostic_claim_language` which scans
//!   every CanonicalAddition / ExistingCanonicalAuthority
//!   Resolution reason text for forbidden diagnostic terms.
//!   `status = Open` pending review. **Does NOT mutate SEED**
//!   (stays at 54); the T.12.j biosignal
//!   `corpus_amendment_proposal_hash_v1` is distinct from
//!   every prior T.12.x proposal hash.
//! * **T.12.k** ([`t12_k_industrial`]): the eleventh real
//!   corpus expansion proposal — Industrial / Fault Detection
//!   and Diagnostics / Condition Monitoring. Panel-locked
//!   warning: *"An industrial fault witness is not a diagnosis
//!   of machine cause unless the plant model, residual law,
//!   sensor law, operating regime, and confuser profile are
//!   declared."* SEED walk found EIGHT industrial / FDD
//!   primitives already canonical (largest SEED-collision set
//!   of any T.12.x to date): FFT band-energy (12), PCA T² (19),
//!   PCA SPE / Q (20), PLS residual (21), Residual envelope
//!   exit (22), Sensor bias (23), Actuator stiction (24),
//!   Valve hunting (25); all eight become
//!   `ExistingCanonicalAuthorityResolution`. The campaign's
//!   strength comes from cross-class dedup discipline rather
//!   than detector count: only SIX genuinely new canonicals at
//!   6101..=6106 (Kalman innovation whiteness — Mehra &
//!   Peschon 1971; operating-regime transition; condition-
//!   indicator drift; fault signature angle; contribution-plot
//!   spike; spectral kurtosis — Antoni 2006). TWO
//!   `DomainTransferOf` records (SEED 12 + SEED 22 as shared
//!   ancestors for `FaultDetectionDiagnostics`). FOUR
//!   `ParameterizationOf` records collapsing panel-candidate
//!   primitives that don't survive SEED-walk (bearing
//!   vibration band-energy 6107 → SEED 12; motor current
//!   signature 6108 → SEED 12; temperature envelope excursion
//!   6109 → SEED 22; pressure transient 6110 → SEED 42 Slew
//!   shock). TWO `RejectedNotDeterministic` records (fifth
//!   T.12.x with two rejections, following T.12.g / h / i / j):
//!   proprietary PdM black-box score (6111; GE Predix /
//!   Siemens MindSphere / IBM Maximo Predict / Honeywell Forge
//!   / Aspen Mtell) and learned fault classifier (6112; Wen
//!   et al.\ 2017, Khan & Yairi 2018). Panel-locked non-claim:
//!   T.12.k admits deterministic condition-monitoring / FDD
//!   witnesses, NOT root-cause certainty and NOT maintenance
//!   recommendations — pinned by
//!   `t12_k_rejects_root_cause_claim_language` parametric
//!   scanner. The MOST IMPORTANT load-bearing negative is
//!   `t12_k_rejects_fault_detector_without_plant_or_residual_contract`
//!   which enforces every CanonicalAddition reason text
//!   declare plant / observer / residual / model / state-
//!   machine / latent-space contract AND a decision law /
//!   functional / predicate. `status = Open` pending review.
//!   **Does NOT mutate SEED** (stays at 54); the T.12.k
//!   industrial `corpus_amendment_proposal_hash_v1` is
//!   distinct from every prior T.12.x proposal hash.
//! * **T.12.l** ([`t12_l_chemometrics`]): the twelfth real
//!   corpus expansion proposal — Chemometrics. Panel-locked
//!   warning: *"A chemometric witness is admissible only when
//!   the sample matrix, preprocessing law, scaling law,
//!   latent-space model, calibration / residual law,
//!   validation split, unit semantics, and decision functional
//!   are declared."* SEED walk found FOUR chemometric
//!   primitives already canonical — the same latent-space +
//!   envelope set T.12.k authority-resolved under
//!   FaultDetectionDiagnostics, now re-resolved under
//!   Chemometrics: PCA T² (19), PCA SPE / Q (20), PLS residual
//!   (21), Residual envelope exit (22). Panel-locked
//!   success-shape applied (same as T.12.k): only FIVE new
//!   canonicals at 6201..=6205 (calibration residual witness,
//!   leverage outlier, concentration drift witness, SIMCA
//!   class-distance witness per Wold & Sjöström 1977, VIP
//!   shift witness per Wold 1995) with declared sample-matrix,
//!   preprocessing, scaling, latent-space-model,
//!   calibration / residual law, validation split, unit
//!   semantics, and decision-functional contracts. TWO
//!   `DomainTransferOf` records (SEED 19 latent-space ancestor
//!   and SEED 22 envelope ancestor for `Chemometrics`). FOUR
//!   `ParameterizationOf` records collapsing panel-candidate
//!   primitives that don't survive SEED-walk (PCA score
//!   outlier 6206 → SEED 19; Mahalanobis-on-scores 6207 → SEED
//!   19; latent-variable control chart 6208 → SEED 20;
//!   spectral preprocessing artifact 6209 → SEED 22). TWO
//!   `RejectedNotDeterministic` records (sixth T.12.x with
//!   two rejections, following T.12.g / h / i / j / k):
//!   black-box spectroscopy classifier (6210; Bruker AI-IDENT,
//!   Mettler-Toledo Spectraline, Thermo Scientific OMNIC ML,
//!   Agilent MicroLab AI) and adaptive-AutoML / stochastic-CV
//!   chemometric model (6211; auto-sklearn, H2O AutoML, TPOT).
//!   Panel-locked non-claim: T.12.l does NOT admit chemical
//!   causation, material identification certainty, regulatory
//!   compliance, lab diagnosis, or process-control authority —
//!   pinned by both
//!   `t12_l_rejects_material_identification_claim_language`
//!   and `t12_l_rejects_regulatory_compliance_claim_language`
//!   parametric scanners. The MOST IMPORTANT load-bearing
//!   negative is
//!   `t12_l_rejects_chemometric_detector_without_preprocessing_or_latent_model_contract`
//!   which enforces every CanonicalAddition reason declare
//!   preprocessing law / scaling law / latent-space model /
//!   calibration model AND decision functional. `status =
//!   Open` pending review. **Does NOT mutate SEED** (stays at
//!   54); the T.12.l chemometrics
//!   `corpus_amendment_proposal_hash_v1` is distinct from
//!   every prior T.12.x proposal hash.
//! * **T.12.m** ([`t12_m_rf`]): the thirteenth real corpus
//!   expansion proposal — RF / Communications. Panel-locked
//!   warning: *"An RF witness is admissible only when the
//!   signal representation, sampling law, unit law, carrier /
//!   channel assumption, synchronization assumption, window /
//!   transform law, decision functional, confuser profile, and
//!   numeric mode are declared."* SEED walk found SIX RF-
//!   relevant primitives already canonical: FFT band-energy
//!   (12), Residual envelope exit (22), Spectral entropy (38),
//!   Autocorrelation break (40), Carrier-frequency-offset
//!   residual (53; Morelli & Mengali 1999), and Error Vector
//!   Magnitude anomaly (54; Shafik / Rahman / Islam 2006).
//!   Panel-locked success-shape applied: SIX new canonicals at
//!   6303..=6308 (constellation spread, channel impulse
//!   response drift, IQ imbalance, phase-noise per Razavi 1996,
//!   symbol-timing offset residual, cyclostationary feature
//!   shift per Gardner 1987) with declared signal
//!   representation, sampling, unit, carrier / channel,
//!   synchronization, window / transform, decision-functional,
//!   confuser, and numeric-mode contracts. Reserved ids 6301
//!   and 6302 are deliberately unused — the CFO and EVM ideas
//!   they once shadowed collapsed onto SEED 53 and SEED 54
//!   respectively under the SEED-walk-first discipline. TWO
//!   `DomainTransferOf` records
//!   (SEED 12 spectral ancestor and SEED 22 envelope ancestor
//!   for `RfCommunications`). FOUR `ParameterizationOf`
//!   records collapsing panel-candidate primitives that don't
//!   survive SEED-walk (spectral mask violation 6309 → SEED
//!   12; SNR drop 6310 → SEED 12; burst preamble miss 6311 →
//!   SEED 40; frame-error burst 6312 → SEED 41). TWO
//!   `RejectedNotDeterministic` records (seventh T.12.x with
//!   two rejections, following T.12.g / h / i / j / k / l):
//!   learned RF fingerprinting classifier (6313; Restuccia
//!   2019 DeepRadioID, Sankhe 2019 ORACLE, Wang 2022 RF
//!   device identification) and black-box modulation
//!   classifier / proprietary spectrum anomaly score (6314;
//!   Keysight signal-analysis ML, Rohde & Schwarz spectrum
//!   monitoring AI, NI RFIC analyser ML, Ettus USRP-based
//!   learned pipelines). Panel-locked non-claim: T.12.m does
//!   NOT admit emitter attribution, transmitter
//!   identification, geolocation, spectrum-enforcement
//!   authority, military classification, or communications-
//!   intelligence conclusions — pinned by
//!   `t12_m_rejects_emitter_identification_claim_language`,
//!   `t12_m_rejects_geolocation_or_attribution_claim_language`,
//!   and `t12_m_rejects_spectrum_enforcement_claim_language`
//!   parametric scanners. The MOST IMPORTANT load-bearing
//!   negative is
//!   `t12_m_rejects_rf_detector_without_signal_or_sampling_contract`
//!   which enforces every CanonicalAddition reason declare
//!   signal representation / sampling law / carrier or
//!   channel assumption / window-or-transform law AND decision
//!   functional. `status = Open` pending review. **Does NOT
//!   mutate SEED** (stays at 54); the T.12.m RF /
//!   communications `corpus_amendment_proposal_hash_v1` is
//!   distinct from every prior T.12.x proposal hash.
//! * **T.12.n** ([`t12_n_econometrics_reliability`]): the
//!   fourteenth real corpus expansion proposal, Econometrics
//!   with Reliability and Survival combined into one proposal
//!   because the two domains share structural-break, CUSUM,
//!   and envelope-residual ancestry. Panel-locked warning:
//!   *"An econometric or reliability / survival witness is
//!   admissible only when the stationarity contract, window
//!   contract, regression / hazard model, censoring law,
//!   time-origin law, residual definition, decision
//!   functional, confuser profile, and numeric mode are
//!   declared."* SEED walk found FOUR T.12.n-relevant
//!   primitives already canonical: CUSUM (3, shared
//!   structural-change ancestor), Page-Hinkley (4, structural-
//!   break F-test parameterization target), Mann-Kendall
//!   (11, trend ancestor), Residual envelope exit (22,
//!   envelope-boundary ancestor). Panel-locked success-shape
//!   applied: EIGHT new canonicals at 6401..=6408, four
//!   econometric (GARCH residual per Bollerslev 1986 against
//!   a conditional-variance model, cointegration-break per
//!   Hansen 1992 with CUSUM-of-squared-residuals, Hausman per
//!   Hausman 1978 chi-squared on a parameter-difference
//!   vector, Bai-Perron per Bai-Perron 1998 / 2003 admitting
//!   multiple breaks under an information-criterion-selected
//!   count) plus four reliability / survival (Kaplan-Meier
//!   survival residual per Kaplan-Meier 1958 with declared
//!   censoring and time-origin, Cox-Schoenfeld per Cox 1972
//!   and Schoenfeld 1982 with Grambsch-Therneau 1994 test
//!   law, Weibull failure-rate envelope exit per Weibull
//!   1951 with declared shape, scale, and MLE, Paris-Erdogan
//!   crack-growth per Paris-Erdogan 1963 with a declared
//!   stress-intensity-range model and C and m parameters),
//!   each with declared stationarity, window, regression or
//!   hazard model, censoring law where applicable, time-
//!   origin law where applicable, residual definition, and
//!   decision-functional contracts. TWO `DomainTransferOf`
//!   records name SEED 3 as the shared structural-change
//!   ancestor and SEED 22 as the shared envelope-boundary
//!   ancestor for `Econometrics` and `ReliabilitySurvival`.
//!   FOUR `ParameterizationOf` records collapse panel-
//!   candidate primitives that don't survive SEED-walk:
//!   CUSUM-of-recursive-residuals (6409, Brown / Durbin /
//!   Evans 1975) parameterises CUSUM (SEED 3); Quandt-Andrews
//!   / Chow structural-break F-test (6410, Quandt 1960 / Chow
//!   1960 / Andrews 1993) parameterises Page-Hinkley (SEED
//!   4); hazard-rate change (6411) parameterises Residual
//!   envelope exit (SEED 22); cumulative damage residual
//!   (6412, Palmgren 1924 / Miner 1945) parameterises CUSUM
//!   (SEED 3). TWO `RejectedNotDeterministic` records
//!   (eighth T.12.x with two rejections, following T.12.g,
//!   h, i, j, k, l, m): learned market predictor / black-box
//!   financial forecaster (6413; Bloomberg AIM, AlphaSense,
//!   Kavout, Goldman SecDB ML, JP Morgan COIN / LOXM) and
//!   learned RUL classifier / black-box predictive-
//!   maintenance score (6414; Uptake AI, C3.ai, Senseye,
//!   IBM Maximo RUL, Siemens MindSphere). Panel-locked
//!   non-claim: T.12.n does NOT admit market prediction,
//!   investment advice, credit-decision authority, actuarial
//!   pricing authority, causal economic certainty, RUL
//!   certainty, maintenance recommendations, or failure-time
//!   prediction; pinned by
//!   `t12_n_rejects_market_prediction_claim_language`,
//!   `t12_n_rejects_investment_or_credit_decision_claim_language`,
//!   and
//!   `t12_n_rejects_rul_or_failure_time_certainty_claim_language`
//!   parametric scanners PLUS
//!   `t12_n_rejects_econometric_witness_without_stationarity_or_window_contract`
//!   AND
//!   `t12_n_rejects_survival_witness_without_censoring_or_time_origin_contract`
//!   contract-discipline scanners. `status = Open` pending
//!   review. **Does NOT mutate SEED** (stays at 54); the
//!   T.12.n econometrics with reliability / survival
//!   `corpus_amendment_proposal_hash_v1` is distinct from
//!   every prior T.12.x proposal hash.
//! * **T.12.o** ([`t12_o_streaming_sketches`]): the fifteenth
//!   real corpus expansion proposal, Streaming Sketches.
//!   Panel-locked warning: *"A streaming-sketch witness is
//!   admissible only when the hash family, width, depth,
//!   seed, bucket count, merge law, update order, error-
//!   bound semantics, residual definition, decision
//!   functional, confuser profile, and numeric mode are
//!   declared."* SEED walk found FOUR T.12.o-relevant
//!   primitives already canonical: KS two-sample test (8,
//!   shared distribution-distance ancestor), Missingness
//!   spike (13, Bloom-inversion ancestor), Error burst (41,
//!   sliding-window heavy-hitter ancestor), Cardinality
//!   drift (46, pre-HLL cardinality-estimator ancestor).
//!   Panel-locked success-shape applied: EIGHT new canonicals
//!   at 6501..=6508, eight structurally distinct streaming-
//!   sketch primitives: CMS residual per Cormode-Muthukrishnan
//!   2005 (hash family, width, depth, per-row seed array,
//!   min-over-d collision rule); HyperLogLog cardinality
//!   shift per Flajolet-Fusy-Gandouet-Meunier 2007 (single
//!   hash family, bucket count m = 2^precision, harmonic-
//!   mean estimator with bias correction; no per-row seed);
//!   Bloom-filter membership anomaly per Bloom 1970 (hash
//!   family, bit-array size, hash count, seed array,
//!   probabilistic false-positive-rate envelope); Misra-Gries
//!   heavy-hitter shift per Misra-Gries 1982 (k counter
//!   slots, decrement-on-miss law; deterministic, no hash);
//!   Space-Saving heavy-hitter shift per Metwally-Agrawal-El
//!   Abbadi 2005 (k counter slots, replace-smallest-on-miss
//!   law; structurally distinct from Misra-Gries via the
//!   different bookkeeping rule); Greenwald-Khanna quantile
//!   summary drift per Greenwald-Khanna 2001 (epsilon error
//!   bound, tuple-insertion rule, deterministic epsilon-
//!   approximate quantile guarantee); t-digest summary
//!   residual per Dunning 2019 (compression delta, centroid
//!   scale function, DETERMINISTIC centroid-merge law); AMS
//!   moment sketch per Alon-Matias-Szegedy 1999 (4-wise-
//!   independent hash family, per-sketch seed, sketch width,
//!   moment order p). TWO `DomainTransferOf` records
//!   (SEED 46 Cardinality drift as shared cardinality
//!   ancestor; SEED 8 KS as shared distribution-distance
//!   ancestor for `StreamingSketches`). FOUR
//!   `ParameterizationOf` records collapsing panel-candidate
//!   primitives that don't survive SEED-walk: Flajolet-Martin
//!   pre-HLL probabilistic counting (6509) parameterises
//!   SEED 46 Cardinality drift; streaming approximate KS via
//!   quantile sketch (6510) parameterises SEED 8 KS; sliding-
//!   window error-burst sketch (6511) parameterises SEED 41
//!   Error burst; sketch-approximate missingness via Bloom
//!   inversion (6512) parameterises SEED 13 Missingness
//!   spike. TWO `RejectedNotDeterministic` records (ninth
//!   T.12.x with two rejections, following T.12.g, h, i, j,
//!   k, l, m, n): learned streaming-anomaly score (6513;
//!   Datadog Watchdog AI, DataRobot Streaming AutoML, Splunk
//!   Stream ML, AWS Lookout for Metrics, Azure Anomaly
//!   Detector) and black-box approximate-streaming
//!   proprietary sketch without declared hash / width /
//!   depth / seed / merge contract (6514; Snowflake APPROX_*,
//!   BigQuery APPROX_*, Druid approximate aggregators,
//!   ClickHouse uniqHLL12 / quantileTDigest / topK, AWS
//!   Athena APPROX_*). Panel-locked non-claim: T.12.o does
//!   NOT admit probabilistic accuracy as certainty,
//!   randomized sketch behavior without seed / width / depth
//!   / hash-family declaration, privacy claims, database
//!   correctness authority, or approximate-query truth;
//!   pinned by
//!   `t12_o_rejects_sketch_without_hash_family_width_depth_or_seed_contract`,
//!   `t12_o_rejects_probabilistic_error_bound_as_deterministic_certainty`,
//!   `t12_o_rejects_approximate_query_truth_claim_language`,
//!   `t12_o_rejects_privacy_or_anonymization_claim_language`,
//!   `t12_o_rejects_mergeable_sketch_without_merge_law`, AND
//!   `t12_o_rejects_black_box_streaming_anomaly_score_without_formula`
//!   parametric scanners. `status = Open` pending review.
//!   **Does NOT mutate SEED** (stays at 54); the T.12.o
//!   streaming-sketches `corpus_amendment_proposal_hash_v1`
//!   is distinct from every prior T.12.x proposal hash.
//! * **T.12.p** ([`t12_p_information_theory`]): the sixteenth
//!   real corpus expansion proposal, Information Theory catch-
//!   up. Panel-locked warning: *"An information-theoretic
//!   witness is admissible only when the estimator, binning or
//!   kernel, smoothing, sample-support, joint-distribution
//!   contract (where applicable), log base, empty-bin law, and
//!   numeric mode are declared."* SEED walk found THREE
//!   T.12.p-relevant primitives already canonical: KL
//!   divergence (9, foundational information-theoretic
//!   divergence ancestor), Jensen-Shannon divergence (32,
//!   symmetric bounded JS variant), Spectral entropy (38,
//!   Shannon entropy on the normalised power spectrum). Panel-
//!   locked success-shape applied: FIVE new canonicals at
//!   6601..=6605, five structurally distinct information-
//!   theoretic primitives: Shannon entropy shift per Shannon
//!   1948 (declared log base, binning or partition law, empty-
//!   bin law, smoothing, sample-support bound, estimator);
//!   Conditional entropy shift per Cover-Thomas 2006 chapter 2
//!   (declared joint-distribution contract, joint binning,
//!   empty-bin law, smoothing, sample-support bound, log base);
//!   Mutual information break per Cover-Thomas 2006 chapter 2
//!   (declared joint-distribution contract, binning OR kernel-
//!   density-estimator, bias-correction rule Miller-Madow or
//!   James-Stein or none, log base; structurally distinct from
//!   SEED 9 KL because MI is a functional on the JOINT vs
//!   PRODUCT-OF-MARGINALS whereas KL is a divergence between
//!   two declared distributions); Cross-entropy / negative-log-
//!   likelihood residual per Shannon 1948 / Cover-Thomas 2006
//!   (declared FIXED MODEL distribution parameter-pinned and
//!   frozen across the comparison window, empirical sample
//!   distribution, log base, smoothing for log(0)); Minimum
//!   description length / coding-length residual per Rissanen
//!   1978 / 1986 (declared model class fixed prefix code or
//!   fixed universal code or two-part code, L(D | M) +
//!   L(M) decomposition with declared parameter-cost law).
//!   TWO `DomainTransferOf` records (SEED 9 KL divergence as
//!   shared information-theoretic divergence ancestor; SEED 38
//!   Spectral entropy as shared Shannon-entropy-on-distribution
//!   ancestor for `InformationTheory`). FOUR
//!   `ParameterizationOf` records collapsing panel-candidate
//!   primitives that don't survive SEED-walk: Normalized mutual
//!   information (6606) parameterises Mutual information (6603)
//!   with declared normalisation function; Transfer entropy
//!   proxy per Schreiber 2000 (6607) parameterises Mutual
//!   information (6603) with declared lagged-joint contract AND
//!   admitted ONLY AS A DETERMINISTIC NON-CAUSAL WITNESS; Rényi
//!   entropy per Rényi 1961 and Tsallis entropy per Tsallis
//!   1988 (6608) parameterise Shannon entropy (6601) with
//!   declared order-alpha parameter law AND declared limit-
//!   recovery; Compression-ratio anomaly per Ziv-Lempel 1977 /
//!   1978 / Welch 1984 LZW (6609) parameterises MDL (6605) with
//!   declared compression algorithm. TWO `RejectedNotDeterministic`
//!   records (tenth T.12.x with two rejections, following
//!   T.12.g, h, i, j, k, l, m, n, o): learned mutual-
//!   information estimator (6610; MINE Belghazi et al. 2018
//!   Mutual Information Neural Estimation, InfoMax /
//!   variational MI bounds, neural KL estimators, InfoVAE, CPC
//!   contrastive predictive coding MI lower bounds) and black-
//!   box information-theoretic anomaly score (6611; AWS Macie
//!   information-leakage scoring, IBM Guardium DAM information-
//!   theoretic anomaly heuristics, Microsoft Purview
//!   information-leakage classifier, Symantec / Broadcom DLP
//!   entropy-based anomaly score, Cisco Talos information-
//!   theoretic threat scoring). Panel-locked non-claim: T.12.p
//!   does NOT admit semantic meaning, causal information flow
//!   certainty, privacy leakage certainty, cryptographic
//!   security claims, or learned representation claims; pinned
//!   by `t12_p_rejects_information_witness_without_estimator_or_binning_contract`,
//!   `t12_p_rejects_entropy_detector_without_base_smoothing_and_empty_bin_law`,
//!   `t12_p_rejects_mutual_information_without_joint_distribution_contract`,
//!   `t12_p_rejects_causal_information_flow_claim_language`,
//!   `t12_p_rejects_privacy_or_security_claim_language`, AND
//!   `t12_p_rejects_learned_embedding_information_score_without_formula`
//!   parametric scanners. `status = Open` pending review.
//!   **Does NOT mutate SEED** (stays at 54); the T.12.p
//!   information-theory `corpus_amendment_proposal_hash_v1`
//!   is distinct from every prior T.12.x proposal hash.
//! * **T.12.consolidate** ([`consolidate`]): the amendment-
//!   review and `corpus_hash_v2` freeze layer. Loads every
//!   T.12.0..T.12.p proposal (17 total: T.12.0 proof-of-life
//!   plus 16 real T.12.a..T.12.p), verifies every proposal
//!   hash, batch hash, and dedup-delta hash by recomputation,
//!   walks every dedup record across all proposals, enforces
//!   ten panel-required negatives (missing proposal, duplicate
//!   reserved id, unused-reserved-id pin, SEED collision,
//!   parameterization-without-parent, authority-without-target,
//!   rejection-without-contract, hash-mismatch,
//!   SEED-or-corpus_hash_v1-mutation, uncredited-literature-
//!   record), builds the sorted T.12 expansion index (one row
//!   per CanonicalAddition across all proposals; 98 entries
//!   spanning 5001..=6699), and emits THREE new own-namespace
//!   hashes: `consolidation_report_hash_v1` under
//!   `DSFB-GPU-ATLAS:T12-CONSOLIDATION-REPORT:v1\0`,
//!   `t12_expansion_index_hash_v1` under
//!   `DSFB-GPU-ATLAS:T12-EXPANSION-INDEX:v1\0`, and
//!   `corpus_hash_v2` under
//!   `DSFB-GPU-ATLAS:LITERATURE-CORPUS:v2\0`. `corpus_hash_v2`
//!   META-hashes `corpus_hash_v1`, the consolidation report,
//!   the expansion index, sorted admitted canonical ids, and
//!   SEED length. Aggregate court delta across T.12.a..T.12.p:
//!   98 CanonicalAddition (includes 2 T.12.a-era "Canonical"
//!   historical-wire-name records), 76
//!   ExistingCanonicalAuthorityResolution, 23 DomainTransferOf,
//!   49 ParameterizationOf, 24 RejectedNotDeterministic,
//!   1 T.12.a AliasOf, 2 T.12.a CompositionOf — total 273
//!   dedup-court records. Panel-locked non-claims: does NOT
//!   add new literature primitives; does NOT mutate `SEED`
//!   (stays at 54); does NOT mutate `corpus_hash_v1` (stays
//!   historical); does NOT mutate any prior T.11 / S1.3 /
//!   T.12.x hash; does NOT promote individual proposals to
//!   `Accepted`. `corpus_hash_v2` is a META-hash over the
//!   ratified-expansion set; it is NOT a full re-hash of a
//!   new SEED table. Per-proposal migration into a new SEED
//!   table is a separate future commit gated on individual
//!   `ProposalStatus::Accepted` ratifications.
//! * **FF.1** ([`ff1_passport_materialisation`]): the first
//!   ratification campaign above `corpus_hash_v2`. Materialises
//!   one DetectorPassport per ratified CanonicalAddition entry
//!   (98 passports spanning canonical ids 5001..=6699) by
//!   pulling the T.12 expansion index from
//!   [`consolidate`] read-only and deriving operational fields
//!   (display name, source class, origin proposal, GPU-family
//!   wire name from the panel-locked SourceClass mapping,
//!   activation-applicability tags from the panel-locked
//!   SourceClass-to-tag mapping, contraindication-linkage
//!   stub, challenge-surface stub). Emits THREE new own-
//!   namespace hash layers: per-passport `passport_hash_v1`
//!   under `DSFB-GPU-ATLAS:FF1-T12-RATIFIED-PASSPORT:v1\0`
//!   (one value per passport),
//!   `ff1_passport_index_hash_v1` under
//!   `DSFB-GPU-ATLAS:FF1-PASSPORT-INDEX:v1\0` (over the sorted
//!   passport list plus pinned upstream anchors), and
//!   `ff1_materialisation_report_hash_v1` under
//!   `DSFB-GPU-ATLAS:FF1-MATERIALISATION-REPORT:v1\0` (over
//!   the index plus per-source-class counts plus total).
//!   Verifier enforces TEN panel-required negatives: passport-
//!   for-non-ratified-canonical-id, passport-if-corpus_hash_v2-
//!   mismatch, passport-materialisation-mutated-T.12-proposal-
//!   hash, passport-materialisation-mutated-corpus_hash_v2,
//!   duplicate-passport-for-same-canonical-id, missing-source-
//!   lineage-for-literature-passport, missing-GPU-family-
//!   mapping, missing-activation-applicability-tags, missing-
//!   contraindication-linkage-stub, missing-challenge-surface-
//!   stub. Panel-locked non-claims: does NOT reopen T.12
//!   dedup decisions; does NOT add new literature primitives;
//!   does NOT alter `corpus_hash_v1`, `corpus_hash_v2`, any
//!   T.12.x proposal hash, or any T.12.consolidate hash; does
//!   NOT mutate `SEED.len()` (stays at 54); does NOT activate
//!   any detector; does NOT decide contraindications or
//!   challenges (stub fields reserve the slot for later
//!   commits); does NOT generate CUDA kernels.
//! * **FF.2** ([`ff2_activation_ratification_gate`]): the
//!   ratification gate that teaches the activation court to
//!   refuse any detector proposal lacking `corpus_hash_v2`
//!   ratification + FF.1 passport authority. Adds a new
//!   [`activation::DisabledReason::DisabledUnratifiedProposal`]
//!   variant so the disable failure mode is operator-visible
//!   and reason-coded; the panel warning is explicit that
//!   unratified proposals must not silently collapse into the
//!   generic `DisabledByWeakLBand` fallback. Classifies every
//!   candidate canonical id into one of four mutually-exclusive
//!   buckets (`SeedHistorical` / `T12RatifiedAndPassported` /
//!   `MissingPassport` / `UnratifiedProposal`), emits one gate
//!   decision per id sorted by `canonical_id`, and aggregates
//!   into a top-level
//!   [`ff2_activation_ratification_gate::Ff2ActivationRatificationGate`]
//!   that pins the four upstream anchor hashes
//!   (`corpus_hash_v1`, `corpus_hash_v2`,
//!   `consolidation_report_hash_v1`,
//!   `ff1_passport_index_hash_v1`). Emits TWO new own-namespace
//!   hash layers: `ff2_activation_ratification_gate_hash_v1`
//!   under `DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1\0`
//!   (over the sorted decision set plus pinned anchors plus
//!   per-status counts), and
//!   `ff2_activation_ratification_gate_summary_hash_v1` under
//!   `DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE-SUMMARY:v1\0`
//!   (over the gate plus panel-locked non-claim block).
//!   Verifier enforces the SIX panel-required negatives:
//!   activation-for-unratified-proposal, activation-for-
//!   missing-FF.1-passport, passport-index-hash-mismatch,
//!   unratified-proposal-without-reason-code, silent-fallback-
//!   to-DisabledByWeakLBand, activation-reason-without-
//!   corpus_hash_v2-binding. Panel-locked non-claims: does NOT
//!   add new detectors; does NOT alter `corpus_hash_v1`,
//!   `corpus_hash_v2`, any T.12.x proposal hash, any
//!   T.12.consolidate hash, or any FF.1 passport / index /
//!   report hash; does NOT mutate `SEED.len()` (stays at 54);
//!   does NOT promote any open proposal to Accepted; does NOT
//!   change S1.3a SEED activation decisions; does NOT generate
//!   CUDA kernels; does NOT decide contraindications or
//!   challenges.
//! * **FF.3** ([`ff3_registry_generation_gate`]): the second
//!   META-discipline layer above S1.3a + FF.1 + FF.2; teaches
//!   the S1.2 registry generator to refuse any `DetectorSpec`
//!   whose source authority is not (a) a SEED canonical record
//!   under `corpus_hash_v1` OR (b) a `corpus_hash_v2`-ratified
//!   entry materialised through FF.1 passport authority.
//!   Classifies every candidate
//!   ([`ff3_registry_generation_gate::Ff3RegistryGenerationCandidate`])
//!   into one of seven mutually-exclusive
//!   [`ff3_registry_generation_gate::Ff3RegistryGenerationEligibility`]
//!   buckets (`Eligible` / `RejectedUnratifiedProposal` /
//!   `RejectedMissingFf1Passport` /
//!   `RejectedCorpusHashV2Mismatch` /
//!   `RejectedPassportIndexHashMismatch` / `RejectedAdHocRecord`
//!   / `RejectedUnknownSourceAuthority`); emits one decision
//!   per id sorted ascending; aggregates into a top-level gate
//!   that pins the five upstream anchor hashes
//!   (`corpus_hash_v1`, `corpus_hash_v2`,
//!   `consolidation_report_hash_v1`,
//!   `ff1_passport_index_hash_v1`,
//!   `ff2_activation_ratification_gate_hash_v1`). Emits TWO
//!   new own-namespace hash layers:
//!   `ff3_registry_generation_gate_hash_v1` under
//!   `DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1\0` (over
//!   the sorted decision set plus pinned anchors plus per-
//!   status counts), and
//!   `ff3_registry_generation_gate_summary_hash_v1` under
//!   `DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE-SUMMARY:v1\0`
//!   (over the gate plus panel-locked non-claim block).
//!   Verifier enforces EIGHT panel-required negatives:
//!   detector-spec-for-unratified-proposal, detector-spec-for-
//!   missing-FF.1-passport, detector-spec-when-
//!   corpus_hash_v2-mismatch, detector-spec-when-passport-
//!   index-hash-mismatch, detector-spec-from-ad-hoc-record,
//!   detector-spec-with-unknown-source-authority, registry-
//!   generation-that-skips-FF.2-ratification-gate (FF.3 MUST
//!   consult the live FF.2 gate), registry-generation-that-
//!   mutates-existing-registry-hash (FF.3 cannot admit more
//!   candidates for registry generation than FF.2 admits for
//!   activation). Panel-locked non-claims: does NOT add new
//!   detectors; does NOT alter `corpus_hash_v1`,
//!   `corpus_hash_v2`, any T.12.x proposal hash, any
//!   T.12.consolidate hash, any FF.1 passport / index / report
//!   hash, or any FF.2 hash; does NOT mutate `SEED.len()`
//!   (stays at 54); does NOT promote any open proposal to
//!   Accepted; does NOT change S1.3a SEED activation decisions
//!   or FF.2 ratification decisions; does NOT itself emit
//!   `DetectorSpec` records (it is a pure-decision module that
//!   the S1.2 registry generator consults); does NOT modify
//!   `dsfb-gpu-atlas-registry`'s existing 162-spec
//!   `registry_hash_v2`; does NOT generate CUDA kernels; does
//!   NOT decide contraindications or challenges.
//! * **FF.4** ([`ff4_readme_authority_boundary`]): the
//!   communication-hygiene seal that makes the post-
//!   T.12.consolidate / post-FF.1 / post-FF.2 / post-FF.3
//!   authority-boundary state unmissable at the README front
//!   door. Pins a canonical authority-boundary block (19
//!   lines), a required-substring set (6 entries the README
//!   MUST contain), and a forbidden-substring set (7 stale
//!   pre-ratification phrasings the README MUST NOT contain).
//!   The verifier walks any README text against the policy and
//!   emits SEVEN panel-required negatives: stale-future-
//!   ratification-language, missing-corpus_hash_v1-historical-
//!   anchor, missing-corpus_hash_v2-ratified-authority,
//!   missing-FF.1-passport-materialisation, missing-FF.2/FF.3-
//!   unratified-rejection, claim-T.12-mutated-SEED, claim-
//!   FF.1-mutated-corpus_hash_v2. Emits ONE new own-namespace
//!   hash layer:
//!   `ff4_readme_authority_boundary_policy_hash_v1` under
//!   `DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0`.
//!   Panel-locked non-claims: does NOT add new detectors; does
//!   NOT alter any upstream hash anchor; does NOT mutate
//!   `SEED.len()` (stays at 54); does NOT change S1.3a / FF.2
//!   / FF.3 court decisions; does NOT generate CUDA kernels;
//!   does NOT decide contraindications or challenges; does NOT
//!   mutate the registry crate. FF.4 changes the README text;
//!   it does not change court state. Panel-locked one-line
//!   verdict: *"FF.4 makes the authority boundary unmissable
//!   at the front door; it does not move any boundary."*
//! * **FF.5** ([`proposal_schema_policy`]): the
//!   forward-looking governance policy defining how proposal
//!   schema upgrades may re-render historical proposal
//!   artifacts without erasing the old artifact hashes or
//!   confusing the court lineage. Core rule: schema upgrade ≠
//!   silent artifact rewrite. Required doctrine: any schema
//!   change that re-renders old proposals MUST preserve the
//!   old artifact hash, emit the new schema hash, explain why
//!   the rendered bytes changed, and provide an explicit
//!   `old_hash → new_hash` migration table. Pins the 10-line
//!   policy doctrine + an empty migration table (no schema
//!   upgrades have happened yet) + the six upstream anchor
//!   hashes (`corpus_hash_v1`, `corpus_hash_v2`,
//!   `ff1_passport_index_hash_v1`,
//!   `ff2_activation_ratification_gate_hash_v1`,
//!   `ff3_registry_generation_gate_hash_v1`,
//!   `ff4_readme_authority_boundary_policy_hash_v1`). Emits
//!   THREE new own-namespace hash layers:
//!   `proposal_schema_upgrade_policy_hash_v1` under
//!   `DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1\0`,
//!   `proposal_schema_migration_table_hash_v1` under
//!   `DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1\0`,
//!   and `schema_upgrade_receipt_hash_v1` (per-receipt) under
//!   `DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1\0`. Verifier
//!   enforces NINE panel-required negatives: schema-rerender-
//!   without-old-hash, schema-rerender-without-new-schema-hash,
//!   schema-rerender-without-migration-table, schema-rerender-
//!   without-reason, migration-table-with-duplicate-old-hash,
//!   migration-table-with-duplicate-new-hash, claim-that-old-
//!   artifact-hash-was-invalid, schema-upgrade-that-mutates-
//!   corpus_hash_v1, schema-upgrade-that-mutates-corpus_hash_v2-
//!   without-freeze-campaign. Panel-locked non-claims: does
//!   NOT add new detectors; does NOT alter any upstream hash
//!   anchor; does NOT mutate `SEED.len()` (stays at 54); does
//!   NOT itself perform any schema upgrade (it is a
//!   forward-looking governance artifact pinning the contract
//!   future upgrades MUST satisfy); does NOT change S1.3a /
//!   FF.2 / FF.3 / FF.4 court decisions; does NOT generate
//!   CUDA kernels; does NOT decide contraindications or
//!   challenges. Panel-locked one-line verdict: *"Schema
//!   upgrade != silent artifact rewrite."*
//! * **S1.3d** ([`s1_3d_budget_pruning`]): the budget pruning +
//!   redundancy suppression layer above S1.3a + FF.2 + FF.3.
//!   Consumes the FF.3-eligible detector surface (152 ratified
//!   candidates by default = 54 SEED + 98 T12-passported) plus
//!   a declared `TaskBudget` envelope plus an explicit set of
//!   `RedundancyCluster` declarations; emits per-candidate
//!   `S13dBudgetDecision` records each carrying either an
//!   `Active` outcome (with a `RetainedAsBudgetSurvivor` or
//!   `RetainedAsRepresentativeWitness` retain reason) or a
//!   `Disabled` outcome with one of eight reason-coded budget
//!   disable variants (`DisabledByBudget`,
//!   `DisabledByRedundancy`, `DisabledByGpuFamilyQuota`,
//!   `DisabledByTaskBudget`, `DisabledByRuntimeBudget`,
//!   `DisabledByMemoryBudget`,
//!   `DisabledByContraindicationBudget`,
//!   `DisabledByCoverageHoleBudget`). The default production
//!   task budget is panel-permissive (`max_active_detectors =
//!   10_000`, `u64::MAX` runtime + memory ceilings, empty
//!   per-GPU-family quota set, empty redundancy cluster set,
//!   `reject_open_contraindications = false`,
//!   `reject_open_coverage_holes = false`); under those
//!   conditions every FF.3-eligible candidate flows through to
//!   `Active` with `RetainedAsBudgetSurvivor`. Tests inject
//!   pressure-bearing budgets and cluster sets to exercise
//!   each of the eight disable reason codes deterministically.
//!   Emits THREE new own-namespace hash layers:
//!   `budget_pruning_plan_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13D-BUDGET-PRUNING-PLAN:v1\0` (the
//!   per-decision plan + tie-break transcript + per-reason
//!   counts + pinned upstream anchors),
//!   `redundancy_suppression_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13D-REDUNDANCY-SUPPRESSION:v1\0` (the
//!   cluster declarations + retained representatives +
//!   suppression count, hashed under a distinct domain so
//!   future commits can grow cluster sets without churning the
//!   plan hash), and `budgeted_activation_summary_hash_v1`
//!   under `DSFB-GPU-ATLAS:S13D-BUDGETED-ACTIVATION-SUMMARY:v1\0`
//!   (the top-level summary META-hash binding the plan +
//!   redundancy report). Verifier enforces EIGHT panel-required
//!   load-bearing negatives: rejects-active-decision-without-
//!   budget-witness, rejects-disabled-decision-without-reason-
//!   code, rejects-redundancy-disable-without-cluster-reference,
//!   rejects-budget-disable-when-budget-is-permissive, rejects-
//!   tie-break-without-deterministic-transcript, rejects-plan-
//!   that-mutates-ff2-or-ff3-decisions, rejects-plan-that-
//!   admits-unratified-candidate, rejects-plan-that-disables-
//!   eligible-without-budget-or-redundancy-reason. Panel-locked
//!   non-claims: does NOT add new detectors; does NOT alter any
//!   upstream hash anchor (`corpus_hash_v1`, `corpus_hash_v2`,
//!   every FF.1 / FF.2 / FF.3 / FF.4 / FF.5 hash unchanged);
//!   does NOT rewrite any prior T.11 / S1.3 / T.12.x / FF.x
//!   hash; does NOT mutate `SEED.len()` (stays at 54); does
//!   NOT change S1.3a SEED activation decisions or FF.2
//!   ratification decisions or FF.3 registry-generation
//!   eligibility; does NOT generate CUDA kernels; does NOT
//!   itself emit `KernelPlan` records (that is S1.3e); does
//!   NOT decide contraindications or challenges; does NOT
//!   modify the registry crate. Panel-locked one-line verdict:
//!   *"Eligibility is not activation; activation is not budget
//!   admission."*
//! * **S1.3e** ([`s1_3e_kernel_plan`]): the deterministic
//!   GPU-family execution-plan layer above S1.3d. Consumes the
//!   S1.3d-Active candidate set (152 retained witnesses at
//!   baseline = 54 SEED + 98 T12-passported); resolves each
//!   id's GPU family from either the SEED record's
//!   `gpu_family` field (id ≤ 54) or the FF.1 passport's
//!   `gpu_family_wire_name` field (id > 54); groups the set
//!   by family; emits one `FamilyLane` per family (sorted
//!   ascending by family wire name) carrying the lane's
//!   active canonical ids, declared cost model (panel-locked
//!   non-empty wire string from a lookup table), expected
//!   kernel name, and aggregate cost estimate. Default
//!   production emits 14 lanes / 152 active detectors. Three
//!   new own-namespace hash layers: `kernel_plan_hash_v1`
//!   under `DSFB-GPU-ATLAS:S13E-KERNEL-PLAN:v1\0`,
//!   `kernel_family_schedule_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13E-KERNEL-FAMILY-SCHEDULE:v1\0`,
//!   `kernel_parameter_table_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13E-KERNEL-PARAMETER-TABLE:v1\0`.
//!   Verifier enforces EIGHT panel-required load-bearing
//!   negatives: kernel-plan-using-budget-disabled-detector;
//!   kernel-plan-using-FF.3-rejected-record; kernel-plan-
//!   without-GPU-family-mapping; parameter-table-without-
//!   stable-order; family-schedule-without-declared-cost-
//!   model; kernel-plan-that-mutates-activation-or-budget-
//!   hash; cuda-execution-claim-inside-kernel-plan
//!   (case-insensitive substring scanner); nondeterministic-
//!   tie-break-in-family-order. Panel-locked non-claims: does
//!   NOT execute kernels; does NOT emit CUDA source, PTX,
//!   SASS, or cubin bytes; does NOT alter any upstream hash
//!   anchor; does NOT mutate `SEED.len()` (stays at 54);
//!   does NOT change S1.3a / FF.2 / FF.3 / S1.3d court
//!   decisions; does NOT itself emit a `CaseFileV2Header`
//!   (that integration is S1.3f); does NOT decide
//!   contraindications or challenges; does NOT modify the
//!   registry crate. Panel-locked one-line verdict: *"S1.3d
//!   says who survives budgeted deployment; S1.3e says how
//!   the survivors are packed into deterministic GPU-family
//!   execution lanes."*
//! * **S1.3f** ([`s1_3f_casefile_v2_activation`]): the
//!   CaseFileV2 activation-integration layer binding S1.3a /
//!   S1.3b / S1.3c / S1.3d / S1.3e / FF.2 / FF.3 / T.11g /
//!   T.11f / T.11h / `corpus_hash_v1` / `corpus_hash_v2`
//!   into a single replayable authority chain every emitted
//!   case file MUST cite. Three META-hashes:
//!   `casefile_v2_activation_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:CASEFILE-V2-ACTIVATION-BINDING:v1\0`,
//!   `casefile_v2_kernel_plan_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:CASEFILE-V2-KERNEL-PLAN-BINDING:v1\0`,
//!   `casefile_v2_authority_chain_hash_v1` under
//!   `DSFB-GPU-ATLAS:CASEFILE-V2-AUTHORITY-CHAIN:v1\0`. The
//!   kernel-plan binding carries a 152-row per-detector lane
//!   membership index. Verifier enforces TEN panel-required
//!   load-bearing negatives plus 4 structural defect rules.
//!   Panel-locked non-claims: does NOT emit detector outputs
//!   / witness records / fusion tensors / candidate
//!   intervals / episodes; does NOT execute kernels; does
//!   NOT alter any upstream hash anchor; does NOT mutate
//!   `SEED.len()`; does NOT change S1.3a / FF.2 / FF.3 /
//!   S1.3d / S1.3e court decisions; does NOT decide
//!   contraindications or challenges (it only links them);
//!   does NOT modify the registry crate. Panel-locked one-
//!   line verdict: *"S1.3f makes CaseFileV2 carry the whole
//!   activation-to-kernel authority chain, so evidence
//!   output cannot be detached from the court decisions
//!   that allowed it to exist."*
//! * **S1.3g** ([`s1_3g_otel_binding`]): the deterministic
//!   receipt-only mapping schema from OpenTelemetry spans /
//!   metrics / logs / resources into `EvidenceDensor` fields.
//!   Receipt-only: it does NOT ingest live OTLP streams, run
//!   collectors, open sockets, depend on an OTel SDK, or
//!   claim runtime interoperability. Four per-signal binding
//!   records ([`s1_3g_otel_binding::SpanToEvidenceDensorBindingV1`],
//!   [`s1_3g_otel_binding::MetricToEvidenceDensorBindingV1`],
//!   [`s1_3g_otel_binding::LogToEvidenceDensorBindingV1`],
//!   [`s1_3g_otel_binding::ResourceToEvidenceDensorBindingV1`])
//!   each declare laws for timestamp, identity, and
//!   attribute ordering; the top-level
//!   [`s1_3g_otel_binding::OTelBindingReceiptTypesV1`] wraps
//!   them plus `corpus_hash_v1` and `SEED.len()`. Five new
//!   own-namespace hashes: `otel_span_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-SPAN-BINDING:v1\0`,
//!   `otel_metric_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-METRIC-BINDING:v1\0`,
//!   `otel_log_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-LOG-BINDING:v1\0`,
//!   `otel_resource_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-RESOURCE-BINDING:v1\0`, and
//!   `otel_binding_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-BINDING-RECEIPT:v1\0`. Verifier
//!   enforces TEN panel-required load-bearing negatives
//!   (timestamp law absent on any binding; unit / temporality
//!   law absent on metric; trace / span identity law absent
//!   on span; body-hash / severity law absent on log;
//!   resource identity law absent on resource; live-
//!   ingestion claim via flag or substring; OTel-SDK-runtime
//!   dependency via flag or substring; stale `"S1.3a OTel
//!   binding"` references via substring scanner; non-
//!   deterministic attribute ordering; empty
//!   evidence_densor_fields). Panel-locked non-claims: does
//!   NOT ingest live OTLP streams, run collectors, open
//!   sockets, depend on an OTel SDK, or claim runtime
//!   interoperability; does NOT emit detector outputs /
//!   witness records / fusion tensors / candidate intervals
//!   / episodes; does NOT mutate any upstream hash anchor;
//!   does NOT alter `SEED.len()`; does NOT change S1.3a /
//!   FF.2 / FF.3 / S1.3d / S1.3e / S1.3f court decisions;
//!   does NOT decide contraindications or challenges; does
//!   NOT modify the registry crate. Panel-locked one-line
//!   verdict: *"S1.3f binds court authority into CaseFileV2;
//!   S1.3g defines how external OTel telemetry can be mapped
//!   into EvidenceDensor fields without yet ingesting it."*
//! * **T.12.PROV** ([`t12_prov_scientific_provenance`]): the
//!   Scientific Provenance Credit Pass. Derivation-only walk
//!   of every T.12.a..T.12.p `CanonicalAddition` that emits
//!   one [`t12_prov_scientific_provenance::ScientistCredit`]
//!   row per canonical (98 total, sorted ascending by id),
//!   one [`t12_prov_scientific_provenance::SourceBibliographyEntry`]
//!   per unique `(citation_key, source_class)` pair (133
//!   total), and a top-level
//!   [`t12_prov_scientific_provenance::ProvenanceCreditReport`]
//!   binding both indexes + `corpus_hash_v1` + `SEED.len()` +
//!   per-class record counts. Three new own-namespace hashes:
//!   `scientist_credit_index_hash_v1` under
//!   `DSFB-GPU-ATLAS:SCIENTIST-CREDIT-INDEX:v1\0`,
//!   `source_bibliography_index_hash_v1` under
//!   `DSFB-GPU-ATLAS:SOURCE-BIBLIOGRAPHY-INDEX:v1\0`, and
//!   `provenance_credit_report_hash_v1` under
//!   `DSFB-GPU-ATLAS:PROVENANCE-CREDIT-REPORT:v1\0`. Verifier
//!   enforces EIGHT panel-required load-bearing negatives
//!   (`CanonicalAdditionWithoutScientistCredit`,
//!   `CanonicalAdditionWithoutSourceRef`,
//!   `ScientistCreditWithoutContributionNote`,
//!   `SourceRefKeyNotInProposalSources`,
//!   `DsfbInventionClaimForPriorDetector` — case-insensitive
//!   forbidden-substring scanner over 8 phrases including
//!   `"dsfb invented"`, `"we invented"`, `"originally
//!   introduced by dsfb"`,
//!   `EngineeringPracticeRecordWithoutProvenanceNote`,
//!   `RejectedRecordWithoutMethodFamilyCredit`,
//!   `ParameterizationWithoutParentLineageNote`). Every credit
//!   row carries the panel-locked
//!   [`t12_prov_scientific_provenance::T12_PROV_DSFB_CREDIT_NOTE`]
//!   verbatim ("DSFB-GPU-Atlas canonizes, deduplicates,
//!   normalizes, contracts, and activates this detector
//!   primitive into a deterministic, replayable witness
//!   record. DSFB-GPU-Atlas does not claim invention of this
//!   primitive; named scientists and source papers above
//!   carry the original credit."). Panel-locked non-claims:
//!   does NOT claim DSFB invention of any detector primitive;
//!   does NOT mutate any upstream hash anchor; does NOT alter
//!   `SEED.len()`; does NOT change S1.3a / FF.2 / FF.3 /
//!   S1.3d / S1.3e / S1.3f / S1.3g court decisions; does NOT
//!   emit detector outputs / episodes; does NOT generate CUDA
//!   kernels; does NOT decide contraindications or challenges;
//!   does NOT modify the registry crate. Panel-locked
//!   one-line verdict: *"The identity commit says what
//!   DSFB-GPU is; T.12.PROV makes sure the scientists whose
//!   methods became court witnesses are visibly credited."*
//! * **S-PERF.1**
//!   ([`s_perf_1_device_traffic_receipt`]): the
//!   DeviceTrafficReceiptV1 measurement law. Defines the
//!   byte-accounting envelope every future memory-bandwidth
//!   or saturation claim MUST cite.
//!   [`s_perf_1_device_traffic_receipt::DeviceTrafficReceiptV1`]
//!   carries 22 hashable fields: device identity (5 fields:
//!   `device_name`, `device_uuid_or_identity_hash`,
//!   `sm_arch`, `driver_version`, `cuda_version`); bandwidth
//!   posture (4: `theoretical_memory_bandwidth_gbps`,
//!   `measured_kernel_time_us`, `timing_method`, `layer`);
//!   workload (2: `detector_count`, `catalog_count`); byte
//!   accounting (8: `input_bytes`, `evidence_bytes_read`,
//!   `evidence_bytes_written`, `witness_bytes_written`,
//!   `fusion_bytes_read_written`, `digest_bytes_read`,
//!   `candidate_summary_bytes`, `total_accounted_device_bytes`);
//!   effective claim (3:
//!   `effective_bandwidth_gbps`,
//!   `percent_of_peak_basis_points`,
//!   `accounting_overflow_acknowledged`); anchors (2:
//!   `artifact_hashes`, `contract_hashes`); receipt hash (1).
//!   [`s_perf_1_device_traffic_receipt::TimingMethod`]
//!   enumerates `CudaEvent` / `CudaStreamSync` /
//!   `HostInstantOnly` / `HostJsonInclusiveTime` / `Unknown`;
//!   [`s_perf_1_device_traffic_receipt::DeviceBandwidthLayer`]
//!   enumerates `LayerA` / `LayerB` / `LayerC`.
//!   [`s_perf_1_device_traffic_receipt::DeviceBandwidthClaimPolicyV1`]
//!   carries the eight panel-locked policy lines plus
//!   `device_bandwidth_claim_policy_hash_v1`. Two new
//!   own-namespace hashes:
//!   `device_traffic_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:DEVICE-TRAFFIC-RECEIPT:v1\0` and
//!   `device_bandwidth_claim_policy_hash_v1` under
//!   `DSFB-GPU-ATLAS:DEVICE-BANDWIDTH-CLAIM-POLICY:v1\0`.
//!   The verifier enforces EIGHT panel-required load-bearing
//!   negatives: bandwidth-claim-without-byte-accounting;
//!   peak-percentage-without-device-bandwidth-declared;
//!   Layer-A-claim-when-host-JSON-time-included;
//!   saturation-claim-without-CUDA-event-timing (saturation
//!   = `percent_of_peak_basis_points >= 8000` = 80.00 %);
//!   cross-device-comparison-without-device-identity;
//!   effective-bandwidth-when-total-bytes-zero;
//!   percent-of-peak-above-100-without-explicit-error-flag;
//!   receipt-missing-contract-hashes. The baseline receipt
//!   pins the RTX 4080 SUPER reference host with every
//!   measurement field zero; later S-PERF.* commits replace
//!   these zeros with measured values. Panel-locked
//!   non-claims: does NOT claim bandwidth saturation on any
//!   GPU; does NOT claim production CUDA performance; does
//!   NOT benchmark B300 / GB300 hardware (that is the
//!   S-PERF.7 / S-MG.6 victory-lap commit); does NOT change
//!   any CUDA kernel; does NOT change any court decision;
//!   does NOT mutate any upstream hash anchor; does NOT
//!   alter `SEED.len()` (stays at 54); does NOT emit
//!   detector outputs / episodes; does NOT generate CUDA
//!   kernels; does NOT decide contraindications or
//!   challenges; does NOT modify the registry crate.
//!   Panel-locked one-line verdict: *"T.12.PROV made the
//!   science creditable; S-PERF.1 makes future CUDA
//!   performance claims accountable."*
//! * **S-PERF.2**
//!   ([`s_perf_2_layer_a_resident_pipeline`]): the Layer-A
//!   resident densor pipeline. Three composable receipt
//!   types:
//!   [`s_perf_2_layer_a_resident_pipeline::LayerAResidentPipelineV1`]
//!   declares the stage sequence (panel-locked five
//!   canonical stages: `EvidenceDensorProjection` /
//!   `WitnessDensorEvaluation` / `FusionDensorReduction` /
//!   `CandidateDensorCollapse` / `StageDigestEmission`),
//!   the per-densor residency policy (Evidence / Witness /
//!   Fusion are `DeviceResidentOnly`; Candidate /
//!   StageDigest are `DeviceResidentWithCompactD2H` with
//!   caps 2 048 / 160 bytes per catalog), and five
//!   forbidden-host-activity flags (all `false` for Layer-A
//!   admission);
//!   [`s_perf_2_layer_a_resident_pipeline::LayerADeviceResidencyReceiptV1`]
//!   carries per-densor H2D / D2H byte accounting;
//!   [`s_perf_2_layer_a_resident_pipeline::LayerATrafficReceiptV1`]
//!   META-hashes pipeline + residency receipt + a
//!   referenced S-PERF.1 `device_traffic_receipt_hash_v1`
//!   (with the inner `timing_method` wire name carried
//!   alongside for verification) + the court-authority
//!   anchor list the pipeline promises not to mutate.
//!   Three new own-namespace hashes:
//!   `layer_a_resident_pipeline_hash_v1` under
//!   `DSFB-GPU-ATLAS:LAYER-A-RESIDENT-PIPELINE:v1\0`,
//!   `layer_a_device_residency_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:LAYER-A-DEVICE-RESIDENCY-RECEIPT:v1\0`,
//!   and `layer_a_traffic_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:LAYER-A-TRAFFIC-RECEIPT:v1\0`.
//!   Companion enums:
//!   [`s_perf_2_layer_a_resident_pipeline::LayerADensorKind`]
//!   (Evidence / Witness / Fusion / Candidate /
//!   StageDigest) and
//!   [`s_perf_2_layer_a_resident_pipeline::DeviceResidencyClass`]
//!   (DeviceResidentOnly /
//!   DeviceResidentWithCompactD2H / HostMaterialized; the
//!   last is forbidden for Layer-A). The verifier enforces
//!   EIGHT panel-required load-bearing negatives: Layer-A
//!   receipt with host JSON time; Layer-A receipt with
//!   CaseFileV2 materialisation time; pipeline without
//!   device residency declaration; full witness D2H dump
//!   when summary-only declared; missing H2D / D2H byte
//!   accounting; CUDA timing method not allowed by
//!   S-PERF.1; Layer-A claim without device-traffic-receipt
//!   reference; pipeline that mutates court-authority
//!   hashes. The S-PERF.2 baseline pins the panel-locked
//!   five canonical stages with all measurement bytes zero
//!   (per-densor lists populated so accounting is present)
//!   and references the S-PERF.1 baseline
//!   `DeviceTrafficReceiptV1`. Panel-locked non-claims:
//!   does NOT claim bandwidth saturation on any GPU; does
//!   NOT benchmark B300 / GB300 hardware; does NOT change
//!   any CUDA kernel; does NOT change any court decision;
//!   does NOT mutate any upstream hash anchor; does NOT
//!   alter `SEED.len()` (stays at 54); does NOT emit
//!   detector outputs / episodes; does NOT generate CUDA
//!   kernels; does NOT decide contraindications or
//!   challenges; does NOT modify the registry crate.
//!   Panel-locked one-line verdict: *"S-PERF.1 gave the
//!   ruler; S-PERF.2 isolates the GPU evidence-factory
//!   path the ruler will measure."*
//! * **S-PERF.3**
//!   ([`s_perf_3_public_data_saturation_bundle`]): the
//!   public-data saturation bundle. Three composable
//!   receipt types:
//!   [`s_perf_3_public_data_saturation_bundle::PublicArtifactManifestV1`]
//!   declares per-dataset identity (`dataset_id`,
//!   `display_name`), classification (`dataset_class`,
//!   `layer_a_role_mapping`), access posture
//!   (`access_note`, `license_or_access_status`,
//!   `usage_mode`), hash policy (`hash_policy_kind`,
//!   `per_artifact_sha256_count`,
//!   `source_archive_sha256`), synthetic flag, and
//!   materialization recipe (source URL / DOI, local path
//!   template, ordered steps, expected bytes, deterministic
//!   postprocess flag, live-remote-fetch flag);
//!   [`s_perf_3_public_data_saturation_bundle::DatasetMaterializationPolicyV1`]
//!   carries the 8-line panel-locked policy doctrine
//!   pinned by its own
//!   `dataset_materialization_policy_hash_v1`;
//!   [`s_perf_3_public_data_saturation_bundle::PublicDataSaturationBundleV1`]
//!   META-hashes every manifest (sorted ascending by
//!   `dataset_id`) plus the policy plus the bundle
//!   identity. Three new own-namespace hashes:
//!   `public_artifact_manifest_hash_v1` (one per dataset)
//!   under
//!   `DSFB-GPU-ATLAS:PUBLIC-ARTIFACT-MANIFEST:v1\0`,
//!   `dataset_materialization_policy_hash_v1` under
//!   `DSFB-GPU-ATLAS:DATASET-MATERIALIZATION-POLICY:v1\0`,
//!   and `public_data_saturation_bundle_hash_v1` under
//!   `DSFB-GPU-ATLAS:PUBLIC-DATA-SATURATION-BUNDLE:v1\0`.
//!   Five companion enums (stable wire names):
//!   `DatasetClass` (DebugObservabilityTrace /
//!   SoftwareDefectTable / DataScienceTabular /
//!   TimeSeriesAnomaly / IndustrialPublicFixture);
//!   `HashPolicyKind` (Sha256OfSourceArchive /
//!   Sha256PerFileManifest / UpstreamProvidedChecksum /
//!   Unknown — forbidden); `LicenseOrAccessStatus`
//!   (PublicDomain / Bsd2Clause / Bsd3Clause / MitLicense /
//!   Apache2 / CcBy / CcBySa / CcZero /
//!   AcademicResearchOnly / RegisteredAccess /
//!   UnknownLicense — forbidden); `DatasetUsageMode`
//!   (CitationOnly / MeasuredFixture); `LayerARoleMapping`
//!   (EvidenceDensorSource / WitnessDensorReference /
//!   FusionDensorReference / CandidateDensorReference /
//!   StageDigestReference / Unmapped — forbidden). The
//!   verifier enforces EIGHT panel-required load-bearing
//!   negatives: dataset without source or access note;
//!   artifact without hash policy; bundle with synthetic-
//!   only data; dataset without materialization recipe;
//!   license or access status missing; unpinned download
//!   or live-remote dependency; dataset role without
//!   Layer-A mapping; benchmark claim inside bundle
//!   definition (case-insensitive scanner over 12
//!   forbidden substrings: "achieves saturation",
//!   "saturates the bandwidth", "% of peak", "outperforms",
//!   "world record", "fastest gpu", "petaflops", etc.).
//!   The panel-locked baseline bundle covers all five
//!   dataset classes with five citation-only manifests:
//!   TADBench (Apache-2), Defects4J v2 (MIT), ADBench
//!   subset (BSD-2), TSB-UAD (Apache-2), NASA C-MAPSS
//!   (public domain). All `is_synthetic=false`; all
//!   `usage_mode=CitationOnly`. Later S-PERF.* commits
//!   flip the mode to `MeasuredFixture`. Panel-locked
//!   non-claims: does NOT claim memory-bandwidth
//!   saturation; does NOT benchmark throughput; does NOT
//!   emit any timing receipt; does NOT change any CUDA
//!   kernel; does NOT change any court decision; does NOT
//!   mutate any upstream hash anchor; does NOT alter
//!   `SEED.len()` (stays at 54); does NOT emit detector
//!   outputs / episodes; does NOT generate CUDA kernels;
//!   does NOT decide contraindications or challenges; does
//!   NOT modify the registry crate; does NOT download any
//!   dataset bytes. Panel-locked one-line verdict:
//!   *"S-PERF.2 isolated the evidence-factory path;
//!   S-PERF.3 gives that path a reproducible public
//!   workload to run on."*
//! * **S-PERF.4**
//!   ([`s_perf_4_active_family_compaction`]): the active-
//!   detector family compaction benchmark schema. Three
//!   composable receipt types:
//!   [`s_perf_4_active_family_compaction::ActiveFamilyCompactionPlanV1`]
//!   declares the per-family lane entries (sorted ascending
//!   by GPU family wire name) with active canonical-id
//!   lists, per-lane detector count, parameter-table
//!   offset, expected kernel name, aggregate cost estimate,
//!   plus four upstream anchor hashes
//!   (`source_budget_summary_hash`,
//!   `source_kernel_plan_hash`,
//!   `source_passport_index_hash`, `corpus_hash_v1`);
//!   [`s_perf_4_active_family_compaction::CompactedParameterTableReceiptV1`]
//!   pins per-family byte size + total byte size + panel-
//!   locked `sort_order_wire_name =
//!   "CanonicalIdAscendingWithinFamily"`;
//!   [`s_perf_4_active_family_compaction::FamilyCompactionBenchmarkSchemaV1`]
//!   META-hashes the plan + parameter-table receipt +
//!   S-PERF.2 Layer-A pipeline + traffic receipt hashes +
//!   S-PERF.3 public-data bundle hash. Three new
//!   own-namespace hashes:
//!   `active_family_compaction_plan_hash_v1` under
//!   `DSFB-GPU-ATLAS:ACTIVE-FAMILY-COMPACTION-PLAN:v1\0`,
//!   `compacted_parameter_table_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:COMPACTED-PARAMETER-TABLE-RECEIPT:v1\0`,
//!   and `family_compaction_benchmark_schema_hash_v1` under
//!   `DSFB-GPU-ATLAS:FAMILY-COMPACTION-BENCHMARK-SCHEMA:v1\0`.
//!   The verifier enforces EIGHT panel-required load-
//!   bearing negatives: benchmark schema without kernel
//!   plan hash; detector not active in budget summary;
//!   family lane without GPU family mapping; parameter
//!   table without stable sort order; compaction that
//!   counts detector variants as new canonicals (same
//!   canonical id in more than one family lane);
//!   benchmark claim inside schema (case-insensitive
//!   scanner over 12 forbidden substrings mirroring
//!   S-PERF.3); dataset bundle hash mismatch; Layer-A
//!   pipeline hash mismatch. The S-PERF.4 baseline is
//!   derived deterministically from the live S1.3d / S1.3e
//!   / FF.1 / S-PERF.2 / S-PERF.3 modules: 152 active
//!   detectors compacted into 14 GPU-family lanes
//!   (DistributionDistance 28 / SequentialRecurrence 26 /
//!   Spectral 23 / WindowStatistic 20 / ResidualObserver
//!   13 / TabularConstraint 11 / GraphLocal 9 /
//!   ProjectionResidual 9 / ScalarThreshold 6 /
//!   Missingness 2 / RankStatistic 2 / CategoricalHistogram
//!   1 / NegativeWitness 1 / Wavelet 1). Panel-locked
//!   non-claims: does NOT run any benchmark; does NOT
//!   claim memory-bandwidth saturation; does NOT emit any
//!   timing receipt; does NOT change any CUDA kernel;
//!   does NOT change any court decision; does NOT alter
//!   activation outcomes; does NOT mutate any upstream
//!   hash anchor; does NOT alter `SEED.len()` (stays at
//!   54); does NOT emit detector outputs / episodes; does
//!   NOT generate CUDA kernels; does NOT decide
//!   contraindications or challenges; does NOT modify the
//!   registry crate; does NOT download any dataset bytes.
//!   Panel-locked one-line verdict: *"S-PERF.3 gives the
//!   evidence factory public data; S-PERF.4 packs the
//!   active court witnesses into benchmarkable GPU-family
//!   lanes."*
//! * **S-PERF.5**
//!   ([`s_perf_5_effective_bandwidth_report`]): the verdict
//!   layer of the performance-discipline arc. Three
//!   composable receipt types:
//!   [`s_perf_5_effective_bandwidth_report::LayerABandwidthMeasurementV1`]
//!   carries the raw measurement (cites the S-PERF.1
//!   receipt hash; mirrors device identity, theoretical peak
//!   GB/s, measured kernel time, timing method, total
//!   accounted device bytes, computed effective bandwidth in
//!   GB/s, computed percent-of-peak in basis points, and the
//!   LayerA forbidden-flag mirror);
//!   [`s_perf_5_effective_bandwidth_report::BandwidthClaimAdmissionV1`]
//!   carries the verdict (claim kind, panel-locked
//!   admissibility reason wire name, admitted boolean);
//!   [`s_perf_5_effective_bandwidth_report::EffectiveBandwidthReportV1`]
//!   META-hashes the measurement, admission, and four
//!   upstream anchor hashes (S-PERF.1 DeviceTrafficReceipt,
//!   S-PERF.2 LayerATrafficReceipt, S-PERF.3
//!   PublicDataSaturationBundle, S-PERF.4
//!   FamilyCompactionBenchmarkSchema).
//!   [`s_perf_5_effective_bandwidth_report::BandwidthClaimKind`]
//!   enumerates the four claim variants
//!   (`NoClaim`, `EffectiveBandwidth`, `PercentOfPeak`,
//!   `Saturation`). Three new own-namespace hashes:
//!   `layer_a_bandwidth_measurement_hash_v1` under
//!   `DSFB-GPU-ATLAS:LAYER-A-BANDWIDTH-MEASUREMENT:v1\0`,
//!   `bandwidth_claim_admission_hash_v1` under
//!   `DSFB-GPU-ATLAS:BANDWIDTH-CLAIM-ADMISSION:v1\0`, and
//!   `effective_bandwidth_report_hash_v1` under
//!   `DSFB-GPU-ATLAS:EFFECTIVE-BANDWIDTH-REPORT:v1\0`. The
//!   verifier enforces TEN panel-required load-bearing
//!   negatives: report without S-PERF.1 receipt; report
//!   without S-PERF.2 LayerA receipt; report without
//!   S-PERF.3 bundle hash; report without S-PERF.4 compaction
//!   hash; saturation claim below 8000 bp; saturation claim
//!   with host timing; effective-bandwidth mismatch from
//!   bytes and time; report that includes host JSON /
//!   casefile / transcript time; cross-device claim without
//!   device identity; benchmark claim without public artifact
//!   manifest. Plus 5 structural defect rules
//!   (`ReportIdEmpty`, `AdmissibilityReasonEmpty`,
//!   `BenchmarkClaimInsideReport`,
//!   `ClaimKindIncoherentWithMeasurement`,
//!   `InadmissibleClaimWithoutVerifierReason`). The
//!   S-PERF.5 baseline is `BandwidthClaimKind::NoClaim`
//!   (uninstrumented; mirrors the S-PERF.1 baseline);
//!   future S-PERF.* commits replace zeros with measured
//!   values. Panel-locked non-claims: does NOT claim
//!   memory-bandwidth saturation at baseline; does NOT run
//!   any benchmark; does NOT change any CUDA kernel; does
//!   NOT change any court decision; does NOT mutate any
//!   upstream hash anchor; does NOT alter `SEED.len()`
//!   (stays at 54); does NOT emit detector outputs or
//!   episodes; does NOT decide contraindications or
//!   challenges; does NOT modify the registry crate; does
//!   NOT download any dataset bytes. Panel-locked one-line
//!   verdict: *"S-PERF.4 packs the active witnesses into
//!   benchmarkable lanes; S-PERF.5 turns measured Layer-A
//!   bytes and time into an admissible bandwidth report."*
//! * **S-PERF.6**
//!   ([`s_perf_6_rtx4080_super_measured_cuda_pipeline`]):
//!   the RTX 4080 SUPER measured CUDA pipeline baseline.
//!   Records the measured bandwidth result captured by
//!   `dsfb-gpu-debug-cuda`'s existing bench harness, sourced
//!   verbatim from
//!   `reports/d64_stage_timing_256x4096_K1.txt`. Three
//!   composable receipt types:
//!   [`s_perf_6_rtx4080_super_measured_cuda_pipeline::Rtx4080SuperMeasuredCudaPipelineV1`]
//!   carries the raw measured CUDA pipeline record (panel-
//!   locked RTX 4080 SUPER device identity, every stage
//!   timing, host segments, measured wide bandwidth, and
//!   source-report provenance);
//!   [`s_perf_6_rtx4080_super_measured_cuda_pipeline::Rtx4080SuperMeasuredBandwidthClaimV1`]
//!   carries the verdict
//!   (`MeasuredCudaPipelineBandwidth` claim kind, admitted
//!   = true, saturation_admitted = false, threshold 8000
//!   bp, observed 107 bp);
//!   [`s_perf_6_rtx4080_super_measured_cuda_pipeline::Rtx4080SuperMeasuredBaselineReportV1`]
//!   META-hashes the measurement, claim, four upstream
//!   anchor hashes (S-PERF.2 / S-PERF.3 / S-PERF.4 /
//!   S-PERF.5), and three R.12b episode-count integrity
//!   pins (13 / 89 / 1917). The module also pins the RTX
//!   4080 SUPER device-identity constants
//!   (`RTX_4080_SUPER_DEVICE_NAME`, `sm_89`,
//!   theoretical peak 716 GB/s) and a local
//!   [`s_perf_6_rtx4080_super_measured_cuda_pipeline::MeasuredCudaPipelineClaimKind`]
//!   enum (so the variant does not touch the S-PERF.5
//!   `BandwidthClaimKind` enum, keeping prior S-PERF.5
//!   hashes byte-identical). Three own-namespace hashes:
//!   `rtx4080_super_measured_cuda_pipeline_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-CUDA-PIPELINE:v1\0`,
//!   `rtx4080_super_measured_bandwidth_claim_hash_v1`
//!   under
//!   `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BANDWIDTH-CLAIM:v1\0`,
//!   and `rtx4080_super_measured_baseline_report_hash_v1`
//!   under
//!   `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BASELINE-REPORT:v1\0`.
//!   Measured values sourced verbatim from the on-disk
//!   bench report: RTX 4080 SUPER, CUDA 13.2, theoretical
//!   peak 716 GB/s, host wall median 44 880 us, device
//!   total 35 959 us, `consensus_grid_kernel_wide` 812 us,
//!   `tree_digest consensus` 7 630 us (21.20%), host
//!   `compute_features` 6 656 us, host bank admit + case
//!   finalize 2 253 us, measured wide bandwidth 7.70 GB/s
//!   (770 centi-GB/s), percent of peak 107 bp = 1.07%
//!   (FLOOR rounding), saturation threshold 8000 bp =
//!   80.00%, saturation_admitted = false. Rounding law
//!   (FLOOR): `measured_centi_gbps * 10000 /
//!   (theoretical_gbps * 100) = 770 * 10000 / 71600 =
//!   107`. The verifier enforces FOURTEEN panel-required
//!   load-bearing negatives covering zero bandwidth, zero
//!   device time, missing source report path, missing RTX
//!   4080 SUPER identity (4 field variants), arithmetic
//!   mismatch, saturation claim below 8000 bp, claim that
//!   7.70 GB/s is saturation (dual gate: behavioural +
//!   substring), B300 / GB300 claim (case-insensitive),
//!   production performance claim (case-insensitive),
//!   rebaseline of R.12b episode counts (3 pin variants),
//!   missing tree-digest stage timing, missing host
//!   segment disclosure (honest Layer-A-impurity
//!   disclosure), empty claim kind, and NoClaim baseline
//!   for measured result. Plus 4 structural defect rules.
//!   Panel-locked report sentence (verbatim): *"The RTX
//!   4080 SUPER measured CUDA pipeline baseline reports
//!   7.70 GB/s, approximately 1.07% of the declared 716
//!   GB/s theoretical memory-bandwidth anchor. This is an
//!   admissible measured CUDA pipeline bandwidth result,
//!   not a saturation claim."* Panel-locked non-claims:
//!   does NOT claim saturation, B300 / GB300, production
//!   performance, or Layer-A purity (host segments
//!   honestly disclosed); does NOT generate new detector
//!   results; does NOT change any CUDA kernel; does NOT
//!   change any court decision; does NOT mutate any
//!   upstream hash anchor; does NOT alter `SEED.len()`
//!   (stays at 54); does NOT emit detector outputs or
//!   episodes; does NOT decide contraindications or
//!   challenges; does NOT modify the registry crate; does
//!   NOT download any dataset bytes; does NOT rebaseline
//!   the R.12b D64 saturation pinned baseline; does NOT
//!   run the benchmark from inside the corpus crate (the
//!   corpus stays panel-locked host-only with zero CUDA
//!   dependency; the measurement was captured by
//!   `dsfb-gpu-debug-cuda`). One-line verdict: *"S-PERF.6
//!   measures 13.33 GB/s on the RTX 4080 SUPER. That is
//!   1.86% of the 716 GB/s vendor-datasheet peak, not
//!   saturation."*
//! * **S-PERF.7**
//!   ([`s_perf_7_source_report_import_verifier`]): the
//!   source-report import verifier. Parses
//!   `reports/d64_stage_timing_256x4096_K1.txt` and
//!   `reports/r12_d64_saturation.txt` on disk and asserts
//!   the panel-pinned S-PERF.6 receipt matches the parsed
//!   values field-for-field. Two parsers
//!   ([`s_perf_7_source_report_import_verifier::parse_d64_stage_timing`]
//!   and
//!   [`s_perf_7_source_report_import_verifier::parse_r12b_d64_saturation`]),
//!   one verifier
//!   ([`s_perf_7_source_report_import_verifier::verify_source_reports_match_s_perf_6_baseline`]),
//!   and a hashable envelope
//!   ([`s_perf_7_source_report_import_verifier::SourceReportImportVerifierReportV1`]).
//!   One own-namespace hash
//!   `source_report_import_verifier_hash_v1 = 99cc8a71…`
//!   under
//!   `DSFB-GPU-ATLAS:S-PERF-7-SOURCE-REPORT-IMPORT-VERIFIER:v1\0`;
//!   binds the parsed values, verifier provenance, and the
//!   upstream S-PERF.6 baseline report hash so two builds
//!   against the same source-report bytes and the same
//!   S-PERF.6 baseline produce byte-identical hashes. FOUR
//!   panel-required load-bearing negatives (verbatim from
//!   directive): bandwidth-differs, device-total-differs,
//!   host-segment-differs (fires on either of the two host
//!   segments), R.12b-episode-pins-differ (fires on any
//!   of the three pins disagreeing with 13/89/1917). Plus
//!   structural rules covering non-required stage timings
//!   and cross-report episode-count consistency. 26-test
//!   acceptance suite. Bandwidth-line grammar is strict:
//!   `X.YY GB/s` with exactly two decimal digits;
//!   `7.7 GB/s` is rejected as malformed so silent
//!   rounding cannot disguise a precision drop. R.12b
//!   parser intentionally ignores wall-time columns
//!   (which drift run-to-run with thermal load) and reads
//!   only the K=1 episode-count rows --- pinning wall-time
//!   would force a rebaseline on every bench rerun, which
//!   is exactly the panel-forbidden anti-pattern. S-PERF.7
//!   does NOT run the bench, rewrite source reports,
//!   mutate the S-PERF.6 receipt, mutate any prior hash
//!   anchor (every S-PERF.1--S-PERF.6 hash byte-identical),
//!   alter `SEED.len()`, change court decisions, emit
//!   detector outputs, modify the registry crate, download
//!   any dataset bytes, or rebaseline the R.12b D64
//!   saturation pinned baseline. One-line verdict:
//!   *"S-PERF.7 does not change measured bandwidth; it
//!   strengthens the measurement chain so subsequent Track
//!   B legs ratchet the live measurement upward with the
//!   receipt automatically tracking the bench output
//!   rather than drifting silently."*
//! * **S-PERF.8 (S-PERF.8.1 hardening pass)**
//!   ([`s_perf_8_batched_k_saturation_receipt`]): the
//!   batched-K saturation receipt with hardening pass.
//!   Track B leg 2. Converts the existing R.12b D64
//!   saturation sweep (which already processes K as a
//!   host loop of K serial single-catalog dispatches on
//!   a hot `GpuWorkspace`) into a mechanically-auditable
//!   corpus receipt recording the per-scale K-amortisation
//!   pattern with panel-pinned execution-mode labels,
//!   device identity, R.12b episode pins, per-scale
//!   pre/post bandwidth + delta + interpretation label,
//!   and a campaign-identity negative that mechanically
//!   rejects the overclaim "canonical 16x128 got 1.76x
//!   speedup, therefore K batching solved full-scale."
//!   The corpus crate stays panel-locked host-only with
//!   zero CUDA dependency; the parser walks
//!   `reports/r12_d64_saturation.txt`'s K matrix and the
//!   verifier asserts coherence against S-PERF.6 + the
//!   S-PERF.7 source-report-import verifier. One own-
//!   namespace hash
//!   `batched_k_saturation_receipt_hash_v1 = 37212c42…`
//!   under
//!   `DSFB-GPU-ATLAS:S-PERF-8-BATCHED-K-SATURATION-RECEIPT:v1\0`;
//!   binds the parsed 3 by 6 K matrix (scales canonical /
//!   mid / full crossed with K in `{1, 4, 16, 32, 64, 128}`),
//!   per-scale summaries with interpretation labels, four
//!   panel-pinned execution-mode labels (dispatch mode
//!   `=` host-loop K serial dispatches; catalog order
//!   `=` canonical scale-major then K-ascending; merge
//!   policy `=` no inter-catalog merge; CUDA Graph status
//!   `=` NOT engaged), device identity (RTX 4080 SUPER
//!   with sm_89 and 716 GB/s peak), three R.12b episode
//!   pins (13 / 89 / 1917), upstream S-PERF.6 baseline
//!   hash, and S-PERF.7 verifier hash. FOURTEEN
//!   panel-required load-bearing negatives cover all the
//!   campaign-identity guards: KMatrixIncomplete,
//!   K1FullScalePerCatInconsistentWithReceipt,
//!   K1CatPerSecArithmeticMismatch,
//!   KAmortisationGainExceedsCeiling (5x per-scale
//!   ceiling), HostLoopKClaimedAsBatched,
//!   MissingBatchedKSourceReport,
//!   MissingPrePostBandwidthDelta,
//!   FullScaleClaimAboveMeasuredDelta,
//!   ClaimFullScaleReached25GbpsWithoutMeasurement,
//!   SaturationClaimBelow8000Bp, the CAMPAIGN IDENTITY
//!   negative `CanonicalLaunchBoundGainGeneralizedToFullScale`
//!   (the full 256x4096 summary's interpretation may NOT
//!   be `LaunchBoundGainAtSmallFixture` --- the canonical
//!   small-fixture gain does not generalize to full
//!   scale), R12bEpisodePinsDrift, CatalogOrderDrift,
//!   and CompletionOrderMergeRejected; plus 4 structural
//!   rules and 6 additional integrity rules. 43-test
//!   acceptance suite (was 18 at S-PERF.8 baseline;
//!   gained 25 tests from the S-PERF.8.1 hardening pass)
//!   with pinned-hash back-stop at
//!   `37212c42b4fdf06069c019e34474cf0d6660843bad7cd4a9cbc92020a7b9f201`.
//!   Local enum `BatchedKResultInterpretation`
//!   classifies per-scale gains into one of four
//!   panel-locked variants: `LaunchBoundGainAtSmallFixture`
//!   (gain at or above 1.5x), `ModestFullScaleGain`
//!   (gain in 1.01x..1.5x), `NoFullScaleImprovement`
//!   (gain in 1.00x..1.01x), `Regressed` (gain below
//!   1.00x). Honest measurement (live R.12b sweep, RTX
//!   4080 SUPER, CUDA 13.2): canonical 16x128 batches
//!   from 326.7 to 575.2 cat/sec (1.76x gain,
//!   `LaunchBoundGainAtSmallFixture`); mid 64x512 stays
//!   flat at 411.9 cat/sec (1.00x,
//!   `NoFullScaleImprovement`); full 256x4096 batches
//!   from 34.9 to 36.1 cat/sec (1.03x,
//!   `ModestFullScaleGain`; pre 13.33 to post 13.78
//!   GB/s, gain of 3.43%). The panel-pinned Track B
//!   25 to 50 GB/s target for S-PERF.8 presumed a
//!   launch-bound workload; the live data shows the
//!   workload is device-bound at the headline scale and
//!   batched K delivers only 3.43% gain. Real bandwidth
//!   gains require kernel-shape changes targeting the
//!   four `tree_digest` stages (about 57% of device
//!   time), which is S-PERF.10 territory. S-PERF.8.1
//!   does NOT run the bench, claim true batched-K
//!   execution, claim CUDA Graph engagement, rewrite
//!   source reports, mutate the S-PERF.6 receipt or any
//!   prior hash anchor, alter `SEED.len()`, change
//!   court decisions, emit detector outputs, modify the
//!   registry crate, download any dataset bytes, or
//!   rebaseline the R.12b D64 saturation pinned
//!   baseline. One-line verdict:
//!   *"S-PERF.8 records what host-loop K batching
//!   actually delivers at every scale: launch-bound 1.76x
//!   gain at canonical, device-bound 1.03x at full. The
//!   S-PERF.8.1 hardening pass makes the per-scale
//!   interpretation auditable AND inadmissible to
//!   overclaim. The next Track B leg (S-PERF.10 digest
//!   lane compaction) attacks the ~57% of device time
//!   consumed by the 4 tree_digest stages, not a
//!   launch-bound lever that demonstrably doesn't apply
//!   at the headline scale."*
//! * **S-PERF.10** ([`s_perf_10_digest_lane_plan`]):
//!   DigestLanePlanV1 / digest-cost audit. Track B leg 3,
//!   receipt-only. Parses the four `tree_digest` rows
//!   from `reports/d64_stage_timing_256x4096_K1.txt`
//!   (residual / sign / detector / consensus) and writes
//!   the byte-identical digest-root preservation contract
//!   that any future digest compaction kernel rewrite
//!   MUST satisfy. Three new own-namespace hashes:
//!   `digest_stage_cost_audit_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-STAGE-COST-AUDIT:v1\0`
//!   pins the four parsed stage timings + total;
//!   `digest_compaction_contract_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-COMPACTION-CONTRACT:v1\0`
//!   pins the four preservation laws (digest root,
//!   fragment merge order, digest mode identity, casefile
//!   chain); top-level `digest_lane_plan_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-LANE-PLAN:v1\0`
//!   META-hashes the audit + contract + upstream S-PERF.6
//!   measured baseline + S-PERF.7 verifier + S-PERF.8.1
//!   receipt + full-scale R.12b episode pin (1917).
//!   EIGHT panel-required load-bearing negatives, with
//!   the CAMPAIGN IDENTITY
//!   `DigestOptimisationClaimWithoutByteIdenticalDigestRoots`
//!   mechanically forbidding any digest "optimisation"
//!   claim that does not preserve byte-identical digest
//!   roots; plus
//!   `DigestPlanWithoutFourTreeDigestStageTimings`,
//!   `DigestPlanWithoutTotalDigestShare` (band 50%..65%),
//!   `DigestPlanWithoutSPerf81Anchor`,
//!   `DigestPlanWithoutSPerf6MeasuredBaselineAnchor`,
//!   `DigestPlanThatClaimsBandwidthImprovement` (forbidden
//!   substring scanner for "bandwidth improvement",
//!   "speedup", "saturation reached", etc.; S-PERF.10 is
//!   an audit, not a measurement),
//!   `DigestPlanWithoutFutureRewriteContract`,
//!   `DigestPlanWithEpisodeCountDrift`. Plus structural
//!   defect rules. Measured digest-lane shape pinned
//!   from the live R.12b source report (RTX 4080 SUPER,
//!   CUDA 13.2): `tree_digest residual` 2364 us,
//!   `tree_digest sign` 2684 us,
//!   `tree_digest detector (wide cells)` 2509 us,
//!   `tree_digest consensus` 4338 us,
//!   `digest_total_us` 11895 us, `digest_total_pct`
//!   ~57.3% of device_total. S-PERF.10 does NOT change
//!   kernels, does NOT claim bandwidth improvement, does
//!   NOT benchmark anything, does NOT run any CUDA code,
//!   does NOT compact digests (that is S-PERF.11 /
//!   S-PERF.10b), does NOT mutate the S-PERF.6 /
//!   S-PERF.7 / S-PERF.8 receipts or any prior hash
//!   anchor, does NOT alter `SEED.len()` (stays 54),
//!   does NOT rebaseline R.12b. One-line verdict:
//!   *"S-PERF.10 audits the measured digest-lane
//!   bottleneck and emits DigestLanePlanV1. It does not
//!   claim bandwidth improvement. It defines the
//!   preservation contract that any future digest
//!   compaction kernel rewrite must satisfy."*
//! * **S-PERF.11**
//!   ([`s_perf_11_measured_digest_compaction`]):
//!   measured digest-lane compaction (panel-locked
//!   2026-05-18). Panel-locked core sentence: *"S-PERF.11
//!   performs the first measured digest-lane rewrite
//!   above the S-PERF.10 preservation contract. It
//!   reduces digest_total_us from 11,895 to 8,556 while
//!   preserving byte-identical TreeSha256V1 roots and
//!   R.12b episode counts."* Swaps the D64 tree-digest
//!   leaf launch from `tree_digest_leaf_kernel` (one
//!   chunk per block) to `tree_digest_leaf_kernel_v2`
//!   (32 chunks per block, one chunk per thread within a
//!   warp). Per-chunk SHA-256 input bytes unchanged →
//!   per-stage `TreeSha256V1` root digests byte-identical
//!   → S-PERF.10's `same_mode_digest_root_law` satisfied
//!   by construction (pinned by the
//!   `s_perf_11_pre_rewrite_root_capture` CUDA-gated
//!   acceptance test AND the receipt-level
//!   `s_perf_11_digest_root_equivalence_hash_v1`).
//!   Measured pre/post on canonical 256×4096 K=1 D64
//!   (RTX 4080 SUPER, CUDA 13.2): digest_total_us
//!   11895 → 8556 (-3339, 1.39× speedup); bandwidth
//!   13.33 → 16.38 GB/s (+22.9%, +2288 bp);
//!   saturation_admitted = false (228 bp ≪ 8000 bp
//!   threshold). Three panel-locked own-namespace
//!   hashes:
//!   `s_perf_11_digest_compaction_measurement_hash_v1`
//!   under
//!   `DSFB-GPU-ATLAS:S-PERF-11-DIGEST-COMPACTION-MEASUREMENT:v1\0`
//!   (pins pre/post stage timings + totals + speedup +
//!   source paths + folded kernel-rewrite metadata);
//!   `s_perf_11_digest_root_equivalence_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-11-DIGEST-ROOT-EQUIVALENCE:v1\0`
//!   (pins four pre + four post TreeSha256V1 root
//!   digests + per-stage equivalence flags); top-level
//!   `s_perf_11_bandwidth_delta_report_hash_v1` =
//!   `1a27154e…` under
//!   `DSFB-GPU-ATLAS:S-PERF-11-BANDWIDTH-DELTA-REPORT:v1\0`.
//!   EIGHT panel-required campaign-identity negatives
//!   (verbatim names: speedup_without_digest_root_equivalence
//!   / speedup_without_r12b_episode_stability /
//!   digest_total_not_reduced / bandwidth_not_improved /
//!   missing_s_perf_10_digest_lane_plan_hash /
//!   tree_sha256v1_root_drift /
//!   saturation_claim_below_8000_bp /
//!   claim_that_16_38_gbps_is_memory_saturation), plus
//!   5 structural defects + 4 defense-in-depth
//!   structural extras. 61-test acceptance suite.
//!   Bundled into the S-PERF.11 atomic commit: stale
//!   B300 / S-PERF.7 victory-lap text fix; S-PERF.10
//!   contract wording clarification (4 law renames +
//!   2 text rewrites) that rebaselines
//!   `digest_compaction_contract_hash_v1` +
//!   `digest_lane_plan_hash_v1` (= `e9cf5c34…`); pre-
//!   rewrite root-capture safety harness. R.12b episodes
//!   13/89/1917 byte-stable; pinned R.12b baseline NOT
//!   rebaselined. S-PERF.11 does NOT claim memory-
//!   bandwidth saturation, does NOT change digest mode,
//!   does NOT mutate S-PERF.6 / S-PERF.7 / S-PERF.8 audit
//!   content, does NOT alter `SEED.len()`, does NOT run
//!   CUDA from inside the corpus crate. Panel-locked
//!   one-line verdict (verbatim): *"S-PERF.10 locked the
//!   digest preservation law; S-PERF.11 performs the
//!   first measured digest-lane rewrite and moves the
//!   scoreboard from 13.33 to 16.38 GB/s without digest-
//!   root or episode-count drift."*
//! * **S-PERF.11.1**
//!   ([`s_perf_11_1_post_rewrite_bottleneck_triage`]):
//!   post-S-PERF.11 bottleneck triage (panel-locked
//!   2026-05-18, verbatim panel directive; receipt-only
//!   re-profile between S-PERF.11 kernel rewrite and
//!   S-PERF.12 architectural conversion). Parses the live
//!   triage source-report at
//!   `reports/d64_stage_timing_256x4096_K1_post_s_perf_11_triage.txt`,
//!   buckets per-stage timings into the 7 panel-locked
//!   categories (digest aggregate / detector_motif / host
//!   compute_features / consensus / candidate collapse /
//!   host bank admit / other), classifies the dominant
//!   bucket, applies the panel-locked decision tree
//!   (digest → S-PERF.12 / host compute_features →
//!   S-PERF.13 / detector / consensus / candidate → re-
//!   rank). One new own-namespace hash
//!   `s_perf_11_1_bottleneck_triage_hash_v1 = 70dd967b…`
//!   under
//!   `DSFB-GPU-ATLAS:S-PERF-11-1-BOTTLENECK-TRIAGE:v1\0`.
//!   Six panel-required campaign-identity negatives +
//!   4 structural defect rules; 24-test suite.
//!   Live triage decision (2026-05-18): digest aggregate
//!   = 15334 us / 50.63% of device_total_us →
//!   `BottleneckCategory::DigestStillDominant` →
//!   `NextStrikeRecommendation::SPerf12CompactDensorDigestV1`.
//!   S-PERF.11.1 does NOT change kernels, does NOT claim
//!   bandwidth improvement, does NOT mutate the pinned
//!   post-S-PERF.11 source-report file, does NOT mutate
//!   any prior anchor, does NOT alter `SEED.len()`, does
//!   NOT rebaseline R.12b, does NOT execute the next
//!   strike. Panel-locked one-line verdict (verbatim):
//!   *"S-PERF.11 proves the saturation campaign can move
//!   the scoreboard while preserving deterministic
//!   evidence roots; S-PERF.11.1 re-profiles the device
//!   wall and records the panel-locked next strike under
//!   one hashable triage receipt."*
//! * **CLI** (`dsfb-corpus` binary): `verify`, `report`,
//!   `genealogy`, `genealogy-dot`, `genealogy-json`, `dump`,
//!   `load-check`, `report-bundle`, `passport`,
//!   `passports-emit`, `precedents`, `precedents-emit`,
//!   `admissibility`, `admissibility-emit`, `trial-transcript`,
//!   `trial-transcript-emit`, `execution-attestation`,
//!   `execution-attestation-emit`, `challenges`,
//!   `challenges-emit`, `contraindication`,
//!   `contraindications-emit`, `coverage-holes`,
//!   `coverage-holes-emit`, `activation-plan`,
//!   `activation-plan-emit`, `activation-plan-explain`,
//!   `activation-plan-audit-emit`, `activation-plan-diff`,
//!   `task-manifest`, `dataset-manifest`, `activation-context`,
//!   `activation-context-emit`, `amendment-proposal`,
//!   `amendment-proposal-emit`, `t12-a-spc-proposal`,
//!   `t12-a-spc-proposal-emit`, `t12-b-scd-proposal`,
//!   `t12-b-scd-proposal-emit`, `t12-c-drift-proposal`,
//!   `t12-c-drift-proposal-emit`, `t12-d-robust-proposal`,
//!   `t12-d-robust-proposal-emit`, `t12-e-spectral-proposal`,
//!   `t12-e-spectral-proposal-emit`, `t12-f-timeseries-proposal`,
//!   `t12-f-timeseries-proposal-emit`, `t12-g-graph-proposal`,
//!   `t12-g-graph-proposal-emit`, `t12-h-dataquality-proposal`,
//!   `t12-h-dataquality-proposal-emit`,
//!   `t12-i-observability-proposal`,
//!   `t12-i-observability-proposal-emit`,
//!   `t12-j-biosignal-proposal`,
//!   `t12-j-biosignal-proposal-emit`,
//!   `t12-k-industrial-proposal`,
//!   `t12-k-industrial-proposal-emit`,
//!   `t12-l-chemometrics-proposal`,
//!   `t12-l-chemometrics-proposal-emit`,
//!   `t12-m-rf-proposal`,
//!   `t12-m-rf-proposal-emit`,
//!   `t12-n-econometrics-reliability-proposal`,
//!   `t12-n-econometrics-reliability-proposal-emit`,
//!   `t12-o-streaming-sketches-proposal`,
//!   `t12-o-streaming-sketches-proposal-emit`,
//!   `t12-p-information-theory-proposal`,
//!   `t12-p-information-theory-proposal-emit`,
//!   `t12-consolidate-report`,
//!   `t12-consolidate-report-emit`,
//!   `t12-corpus-v2-freeze`,
//!   `t12-corpus-v2-freeze-emit`,
//!   `t12-expansion-index`,
//!   `t12-expansion-index-emit`, `ff1-passport-index`,
//!   `ff1-passport-index-emit`, `ff1-materialisation-report`,
//!   `ff1-materialisation-report-emit`,
//!   `ff2-gate`,
//!   `ff2-gate-emit`,
//!   `ff2-gate-summary`,
//!   `ff2-gate-summary-emit`,
//!   `ff3-gate`,
//!   `ff3-gate-emit`,
//!   `ff3-gate-summary`,
//!   `ff3-gate-summary-emit`,
//!   `ff4-policy`,
//!   `ff4-policy-emit`,
//!   `ff4-authority-boundary-block`,
//!   `ff5-policy`,
//!   `ff5-policy-emit`,
//!   `ff5-migration-table`,
//!   `s1-3d-plan`,
//!   `s1-3d-plan-emit`,
//!   `s1-3d-redundancy`,
//!   `s1-3d-summary`,
//!   `s1-3e-plan`,
//!   `s1-3e-plan-emit`,
//!   `s1-3e-schedule`,
//!   `s1-3e-parameter-table`,
//!   `s1-3f-authority-chain`,
//!   `s1-3f-authority-chain-emit`,
//!   `s1-3f-activation-binding`,
//!   `s1-3f-kernel-plan-binding`,
//!   `s1-3g-binding`,
//!   `s1-3g-binding-emit`,
//!   `s1-3g-span-binding`,
//!   `s1-3g-metric-binding`,
//!   `s1-3g-log-binding`,
//!   `s1-3g-resource-binding`,
//!   `t12-prov-report`,
//!   `t12-prov-report-emit`,
//!   `t12-prov-scientist-credit-index`,
//!   `t12-prov-source-bibliography-index`,
//!   `s-perf-1-receipt`,
//!   `s-perf-1-receipt-emit`,
//!   `s-perf-1-policy`,
//!   `s-perf-1-policy-emit`,
//!   `s-perf-2-pipeline`,
//!   `s-perf-2-residency-receipt`,
//!   `s-perf-2-traffic-receipt`,
//!   `s-perf-2-receipts-emit`,
//!   `s-perf-3-bundle`,
//!   `s-perf-3-policy`,
//!   `s-perf-3-bundle-emit`,
//!   `s-perf-4-plan`,
//!   `s-perf-4-parameter-table-receipt`,
//!   `s-perf-4-schema`,
//!   `s-perf-4-receipts-emit`,
//!   `s-perf-5-measurement`,
//!   `s-perf-5-admission`,
//!   `s-perf-5-report`,
//!   `s-perf-5-receipts-emit`,
//!   `s-perf-6-measurement`,
//!   `s-perf-6-claim`,
//!   `s-perf-6-baseline`,
//!   `s-perf-6-receipts-emit`,
//!   `s-perf-7-verifier`,
//!   `s-perf-7-verifier-emit`,
//!   `s-perf-8-batched-k`,
//!   `s-perf-8-batched-k-emit`,
//!   `s-perf-10-digest-lane`,
//!   `s-perf-10-digest-lane-emit`,
//!   `s-perf-11-digest-compaction`,
//!   `s-perf-11-digest-compaction-emit`,
//!   `s-perf-11-1-bottleneck-triage`,
//!   `s-perf-11-1-bottleneck-triage-emit`. Each subcommand is
//!   corpus-only; the corpus crate stays free of any GPU /
//!   registry dependency.
//!
//! Doctrine constraints carried verbatim from Section T of the
//! plan:
//!
//! * **Host-only.** No GPU dependency. No CUDA feature flag. The
//!   crate must build on a host without `nvcc`.
//! * **Hash-chain posture (post-T.10)**: `corpus_hash_v1` is
//!   sealed. Every subsequent surface (passport, precedent,
//!   grammar, transcript, execution attestation, challenge
//!   docket, contraindication receipt) lives in its OWN domain-
//!   separated hash namespace and never mutates an upstream
//!   anchor. Cross-linkage between surfaces lives in separate
//!   crosswalk artifacts, not field-level dependencies, so
//!   passport hashes (and every other upstream hash) stay
//!   stable across the T.11 campaign.
//! * **Zero dependencies.** Mirrors the workspace's zero-dep posture.
//!   No serde, no clap, no anyhow. The seed table is a `const`
//!   array, the verifier is hand-rolled, the CLI is argv-parsed
//!   inline.
//! * **Audits usefulness, does not learn it.** The T.8 ledger is
//!   an audit surface, not a ranking model. T.8 shipped the
//!   schema, verifier, and conservative `NotScored` rows; future
//!   commits may populate measured rows backed by real benchmark
//!   artifacts.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Tests assert with `.unwrap()` / `.expect()` for short, readable
// invariants; the workspace's pedantic lints would otherwise flag
// each occurrence.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod activation;
pub mod activation_audit;
pub mod activation_context;
pub mod admissibility;
pub mod amendment;
pub mod audit_report;
pub mod challenge_docket;
pub mod claims;
pub mod consolidate;
pub mod contraindication;
pub mod corpus_hash;
pub mod court;
pub mod coverage_holes;
pub mod dump;
pub mod execution_attestation;
pub mod ff1_passport_materialisation;
pub mod ff2_activation_ratification_gate;
pub mod ff3_registry_generation_gate;
pub mod ff4_readme_authority_boundary;
pub mod fusion;
pub mod genealogy;
pub mod identity;
pub mod lband;
pub mod loader;
pub mod passport;
pub mod precedent;
pub mod proposal_schema_policy;
pub mod report;
pub mod s1_3d_budget_pruning;

pub mod s1_3e_kernel_plan;

pub mod s1_3f_casefile_v2_activation;

pub mod s1_3g_otel_binding;

pub mod s_perf_10_digest_lane_plan;
pub mod s_perf_11_1_post_rewrite_bottleneck_triage;
pub mod s_perf_11_measured_digest_compaction;
pub mod s_perf_12_compact_densor_digest_v1_promotion;
pub mod s_perf_1_device_traffic_receipt;
pub mod s_perf_2_layer_a_resident_pipeline;
pub mod s_perf_3_public_data_saturation_bundle;
pub mod s_perf_4_active_family_compaction;
pub mod s_perf_5_effective_bandwidth_report;
pub mod s_perf_6_rtx4080_super_measured_cuda_pipeline;
pub mod s_perf_7_source_report_import_verifier;
pub mod s_perf_8_batched_k_saturation_receipt;
pub mod seed;
pub mod t12_a_spc;
pub mod t12_b_scd;
pub mod t12_c_drift;
pub mod t12_d_robust;
pub mod t12_e_spectral;
pub mod t12_f_timeseries;
pub mod t12_g_graph;
pub mod t12_h_dataquality;
pub mod t12_i_observability;
pub mod t12_j_biosignal;
pub mod t12_k_industrial;
pub mod t12_l_chemometrics;
pub mod t12_m_rf;
pub mod t12_n_econometrics_reliability;
pub mod t12_o_streaming_sketches;
pub mod t12_p_information_theory;
pub mod t12_prov_scientific_provenance;
pub mod t13_gap_witness_family_audit;
pub mod toml_parser;
pub mod trial_transcript;
pub mod types;
pub mod usefulness;
pub mod verify;

pub use seed::SEED;
pub use types::{
    AxisBindingSet, CanonicalisationDecision, ConfuserProfile, ConstitutionFlags,
    DecisionFunctional, DedupReason, DedupRecord, DetectorAliasId, DetectorCanonicalId,
    DeterministicStatus, DomainTagSet, DuplicateGroupId, GenealogyEdges, GpuFamilyKernel,
    ImplementationLevel, InputRequirementSet, LifecycleState, LiteratureDetector, MathFormId,
    NegativeWitnessKind, ParameterBounds, PrimitiveFamily, SourceRef, UsefulnessLedgerSnapshot,
    WitnessKind, WitnessRole,
};
pub use verify::{verify_record, VerifyError, VerifyReport};
