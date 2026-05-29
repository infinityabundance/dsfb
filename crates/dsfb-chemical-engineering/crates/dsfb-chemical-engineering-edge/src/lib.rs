//! # dsfb-chemical-engineering-edge
//!
//! Deterministic, read-only residual-semiotics layer for chemical-engineering and chemometric
//! process monitoring — CPU / edge build, no GPU required, `#![forbid(unsafe_code)]`.
//!
//! The crate ingests the residuals, scores, distances, reconstruction errors, and detector
//! disagreements that established chemometrics methods already emit, then applies Drift–Slew Fusion
//! Bootstrap (drift, slew, admissibility envelopes, deterministic quorum fusion, a chemical
//! heuristics bank) to produce auditable structural episodes with deterministic replay.
//!
//! It is **not** a controller, plant model, or replacement estimator. It writes to no upstream
//! register; removing it restores the pre-deployment baseline exactly.
//!
//! ## Pipeline
//! `baseline → standardise → PCA(T²/SPE) → detector atlas → per-detector DSFB grammar → quorum
//! fusion → heuristics bank → metrics → deterministic replay hash`.
//!
//! See [`pipeline::analyze`] for the entry point and [`atlas::Corpus`] for the catalogued detector
//! atlas (executed + prior-art surface).
//!
//! ## Module layout
//!
//! | Module | Role |
//! |---|---|
//! | [`atlas`] | Corpus of chemometric detectors (prior-art catalogue + validation) |
//! | [`balance`] | Mass/energy-balance witnesses (three-tank, CSTR, CSTH, tank-volume) for instrumented / real datasets |
//! | [`cli`] | CLI commands: `demo` / `figures` / `analyze` / `atlas` / `corpus` / `verify-replay` / `completeness-court` / `release-scrub` / `unit-consistency` / `data-readiness` / `authority-diff` / `regime-eval` / `historian` / `balance-witness` / `control-action` / `casefile` |
//! | [`completeness`] | `ArtifactCompletenessCourtV1` — machine-checked artifact-graph completeness + consistency gate |
//! | [`release_scrub`] | `PublicReleaseScrubCourtV1` — public-release hygiene gate (placeholder DOI / private-backup leakage / controlled-access rows / required artifacts) |
//! | [`court_record`] | Chemical Court Record v1 bundle writer (claim-boundary badges, rejected candidates, non-claims) |
//! | [`data`] | `DataMatrix` and `Baseline` (mean/std fit over a baseline window) |
//! | [`datasets`] | CSV slice loader + deterministic synthetic-fault generator |
//! | [`detectors`] | Detector bank: runs prior-art estimators and emits `DetectorOutput` |
//! | [`dsfb_core`] | DSFB grammar state machine, admissibility envelope, episode calculus |
//! | [`fft`] | FFT helper (frequency-domain features fed to detectors) |
//! | [`figures`] | Byte-stable trace-CSV export for downstream figure rendering |
//! | [`figure_export`] | Byte-stable JSON/DOT export of the not-demo-exercised evidence objects, for the 60-figure gallery |
//! | [`fusion`] | Per-detector DSFB timelines → quorum fusion → `FusedEpisode` |
//! | [`hashing`] | SHA-256 `CanonicalHasher` for replay verification and provenance gates |
//! | [`heuristics`] | Chemical heuristics bank (emits "unknown" when no rule matches) |
//! | [`linalg`] | PCA model: fit (power iteration / SVD), T², SPE-Q |
//! | [`pipeline`] | End-to-end orchestrator; public entry point is [`pipeline::analyze`] |
//! | [`report`] | CSV/JSON/Markdown/ZIP artifact writers |
//!
//! ### Named hash-sealed evidence objects (P57–P69 + the post-P71 Waves 2–7 — each additive + off the replay path)
//!
//! Every object below is deterministic, sealed by a `CanonicalHasher` digest, self-verifying (`verify()`),
//! and bounded (no root cause, no causality, candidate-only, advisory). Grouped by role:
//!
//! | Module | Evidence object(s) |
//! |---|---|
//! | [`regime_envelope`] | `RegimeEnvelopeV1` — per-regime/phase admissibility envelope with `provenance_hash` |
//! | [`authority_law`] | `ChemicalAuthoritySeparationLawV1` — executable authority/execution separation doctrine |
//! | [`passport`] | `ChemometricPassportV1` — per-detector provenance + threshold/normalization/missingness policy |
//! | [`provenance_graph`] | `ResidualProvenanceGraphV1` — raw→residual→detector→episode→label→court_root DAG (DOT/JSON) |
//! | [`disagreement`] | `DetectorDisagreementForensicsV1` + `NegativeWitnessV1` (silence as first-class evidence) |
//! | [`confuser`] | `ConfuserDocketV1` — per-episode "what was ruled out, and why" (cites atlas confusers) |
//! | [`balance_witness`] | `BalanceWitnessV1` — mass/energy/stoichiometric/yield/selectivity + P75 pack (tank-inventory/splitter-mixer/HX-energy/element/separator-component/utility-loop) closure witness |
//! | [`softsensor`] | `SoftSensorWitnessV1` — measured/prediction/residual/interval + training-scope hash |
//! | [`controller_masking`] | `ControllerMaskingHeuristicV2` + `ValveStictionWitnessV1` |
//! | [`operator_reports`] | `AlarmFloodCompressionReportV1` (ISA-18.2/IEC 62682) + `OperatorIncidentHTMLV1` |
//! | [`sweep`] | `SensitivitySweepReceiptV1` — deterministic threshold-grid robustness receipt |
//! | [`ablation`] | `AblationCourtV1` — per-component ablation deltas vs the full pipeline |
//! | [`topology`] | `ProcessTopologyGraphV1` + `ResidenceTimeAlignmentV1` (unit-ops + residence times) |
//! | [`propagation`] | `FaultPropagationWitnessV1` + `CausalNonClaimGraphV1` (non-causal disclaimer sealed) |
//! | [`precedent`] | `EpisodeShapeHashV1` + motif-NN + recurrence + case-law precedent + fleet comparison (advisory) |
//! | [`narrative`] | `ProcessNarrativeCompilerV1` (non-LLM, anchored) + `NoNarrativeHallucinationGateV1` |
//! | [`migration`] | `SemanticDiffReportV1` + `DetectorRegistryCompatibilityV1` + `HeuristicMigrationReceiptV1` |
//! | [`annotation`] | recipe-transition / maintenance overlays + append-only annotation ledger + amendment chain |
//! | [`transition_pack`] | `SBIRTransitionPackV1` — machine + human Phase-I-style readiness pack (generic) |
//! | [`claim_strength`] | `ClaimStrengthV1` — executable Tier-1/2/3 hierarchy (SealedFact/EvidenceInterpretation/SpeculativeImplication/NonClaim) |
//! | [`witness_strength`] | `PhysicalWitnessStrengthV1` — ladder ranking physical evidence (balance closure) above advisory structure |
//! | [`unit_consistency`] | `UnitConsistencyCourtV1` — deterministic unit/dimension checker (°C↔K, bar↔Pa, mass↔mole-fraction) so a unit bug never reads as an anomaly |
//! | [`spec_limit`] | `SpecLimitWitnessV1` — separates a hard engineering spec/operating-limit breach (relief setpoint, design pressure) from a statistical anomaly (no SIS authority) |
//! | [`permit_boundary`] | `PermitBoundaryWitnessV1` — permit/consent variables (NOx, effluent pH, discharge T) + tightest-approach headroom (non-claim: NOT compliance certification / not a CEMS) |
//! | [`firstprinciples`] | `FirstPrinciplesWitnessAdapterV1` + `EquationResidualPassportV1` — model–plant residual from a governing equation (Arrhenius/Antoine/Raoult/Henry/heat-transfer/pump-curve/valve-Cv) + sealed provenance/validity passport |
//! | [`energy_budget`] | `ResidualEnergyBudgetV1` — decompose an episode's total residual energy into an interpretable budget (balance-closure/spectral/controller-effort/variable-group shares) — magnitude split, not causal attribution |
//! | [`data_readiness`] | `IndustrialDataReadinessCourtV1` — grade a historian export before analysis (Ready / ReadyWithCaveats / NotReadyMissingCriticalWitnesses) — advisory, not a data-validation certificate |
//! | [`multirate`] | `MultiRateAlignmentCourtV1` + `ManualSampleBridgeV1` — auditable ragged multi-rate → grid resampling receipts (ZOH/linear/nearest; never extrapolates) + sparse lab-sample bridge (alignment uncertainty + custody hash) |
//! | [`control_loop`] | `SetpointResidualSeparationV1` + `ControllerModeGuardV1` + `ControlLoopInteractionMapV1` — PV/MV/SP context that separates setpoint-driven motion from process deviation, guards mode transitions, maps candidate loop coupling |
//! | [`startup_shutdown`] | `StartupShutdownEnvelopeV1` — per-transient-phase (cold/hot-start, shutdown-ramp, purge, inerting) envelopes (relaxed, never tighter than steady) + transient-alarms-reclassified metric |
//! | [`material_lot`] | `MaterialLotWitnessV1` + `CertificateOfAnalysisDensorV1` + `CleanInPlaceWitnessV1` — material/QA context (lot window, supplier CoA vs spec, CIP cycle); non-claims: context not causation / not lab truth / not cleaning validation |
//! | [`sensor_trust`] | `SensorTrustDegradationLedgerV1` + `CalibrationEventWitnessV1` — per-sensor trust (1−worst indicator) + recalibration step context; non-claims: advisory health, not a calibration verdict / metrology certificate |
//! | [`batch_genealogy`] | `BatchGenealogyGraphV1` — DAG of batch lineage (lot/equipment/parent links) + shared-factor finder + DOT; non-claim: structural lineage, not causal attribution |
//! | [`plant_twin`] | `PlantTwinReplayV1` — divergence of a live run from an operator-supplied twin/reference (onset, run length, peak/RMS); non-claim: replay-consistency witness, not a controller/simulator/state estimate |
//! | [`plant_data_contract`] | `PlantDataContractV1` + `HistorianImportReceiptV1` — documented input contract (required/optional inputs → verdict) + per-file import receipt (hash, counts, range, warnings, verdict) |
//! | [`data_quality`] | `DataQualityEpisodeV1` + `FrozenTagDetectorV1` + `ClockSkewWitnessV1` — typed data-quality defects (frozen tag / missingness / timestamp skew) distinct from process episodes |
//! | [`observability`] | `InstrumentationCoverageMapV1` + `ObservabilityNonClaimReceiptV1` + `ResidualWitnessCoverageScoreV1` — can the data even see this fault? per-class observability + honest refusal-to-label + coverage score |
//! | [`event_evidence`] | `ChemicalEventOntologyV1` + `EpisodeEvidenceLedgerV1` + `EvidenceMinimumsMatrixV1` + `WitnessBurdenOfProofV1` — typed event class + per-episode witness ledger + burden-of-proof (fail → unknown, never a fabricated label) |
//! | [`adversarial`] | `AdversarialConfuserSuiteV1` + `FalseNarrativeRegressionTestV1` — sealed proof the bank distinguishes confusers (or emits unknown) and never emits a tempting-but-unsupported narrative |
//! | [`redaction`] | `CaseFileRedactionMapV1` + `TamperEvidenceSealV1` + `AuditTrailExportV1` — share evidence without data: alias map (names hashed, values stripped) + append-only hash-chain seal + compliance-format export manifest |
//! | [`confidential_eval`] | `ConfidentialEvaluationBundleV1` + `PartnerDataEscrowProtocolV1` + `ChemicalProcessPlaybookV1` — the commercial unlock: redacted shareable bundle (no raw time-series) + escrow protocol (raw never egresses) + advisory next-inspection playbook |
//! | [`interval_physics`] | `Interval` + `PhysicsInformedEnvelopeV1` — interval-arithmetic first-principles bounds fused with the statistical envelope; a value outside the physics interval is a PhysicsViolation (model–plant mismatch) |
//! | [`multiphysics`] | `MultiPhysicsCrossWitnessV1` — joint grammar over chemical/mechanical/thermal/acoustic domains; cross-domain motifs (candidate co-occurrence, not a proven causal chain) |
//! | [`multiscale`] | `HierarchicalMultiScaleFusionV1` — sensor→unit→plant episode rollup with a cross-scale consistency invariant |
//! | [`spectral_grammar`] | `SpectralGrammarTokenV1` — frequency-domain tokens (limit-cycle resonance / broadband / none) over the FFT power spectrum |
//! | [`amendment_dag`] | `MerkleDagAmendmentChainV1` — Git-like append-only amendment DAG over an immutable record; Merkle-linked lineage proofs + selective disclosure |
//! | [`uncertainty`] | `OperatorUncertaintyDashboardV1` — effort-to-resolve score per unknown (persistence + disagreement entropy + closest-heuristic distance), ranked |
//! | [`discovery`] | `SignatureDiscoveryAssistantV1` — clusters recurring StructuralUnmapped episodes into candidate signatures for HUMAN review (no auto-generated heuristics) |
//! | [`bench_suite`] | `DsfbBenchV1` — provenance-locked challenge-suite definition (adversarial/multi-fault/partial-instrumentation/confuser/data-quality) with honest metrics; makes no win claim |
//! | [`safety_dossier`] | `SafetyCertificationDossierV1` — IEC 61511 / ISA-84 / 21 CFR Part 11 clause→artifact traceability matrix (honest gaps kept); NOT a certification |
//! | [`proof_obligations`] | `ProofObligationLedgerV1` — the grammar/fusion proof obligations: 3 Kani + 2 Lean4-verified, all cross-checked in Coq (`formal/lean/` + `formal/coq/`); 1 open (replay determinism, empirical) |
//!
//! ## Determinism contract
//!
//! Every public function that participates in the pipeline is deterministic given the same inputs:
//! no thread-local state, no RNG outside [`datasets`] (which uses a seeded SplitMix64), no
//! floating-point ordering, and no HashMap iteration (all maps are replaced by sorted `Vec` or
//! `BTreeMap`). The replay hash in [`pipeline::canonical_replay_hash`] is the machine-checkable
//! certificate of this contract.

#![forbid(unsafe_code)]

pub mod ablation;
pub mod adversarial;
pub mod amendment_dag;
pub mod annotation;
pub mod artifact_index;
pub mod atlas;
pub mod authority_law;
pub mod balance;
pub mod balance_witness;
pub mod basis_descriptor;
pub mod batch_genealogy;
pub mod bench_suite;
pub mod breadth_surface;
pub mod calibration_passport;
pub mod claim_strength;
pub mod cli;
pub mod completeness;
pub mod confidential_demo;
pub mod confidential_eval;
pub mod confuser;
pub mod control_loop;
pub mod controller_masking;
pub mod court_record;
pub mod data;
pub mod data_quality;
pub mod data_readiness;
pub mod datasets;
/// Optional integration demo (P101): carry an edge episode summary through the `dsfb-densor-runtime`
/// substrate. Behind the `densor-runtime-demo` feature; off the default build + replay path.
#[cfg(feature = "densor-runtime-demo")]
pub mod densor_demo;
pub mod detectors;
pub mod disagreement;
pub mod discovery;
pub mod dsfb_core;
pub mod energy_budget;
pub mod equipment_signatures;
pub mod event_evidence;
pub mod evidence_kind;
pub mod fft;
pub mod figure_export;
pub mod figures;
pub mod firstprinciples;
pub mod fusion;
pub mod hashing;
pub mod hazop_guidewords;
pub mod heuristics;
pub mod index_court;
pub mod interval_physics;
pub mod linalg;
pub mod material_lot;
pub mod migration;
pub mod multiphysics;
pub mod multirate;
pub mod multiscale;
pub mod narration_context;
/// Optional synthetic demonstrator (panel narrator-generator extension): runs the H7–H12 narration detector over
/// deliberately-malformed narratives to prove each contract rule is mechanically enforceable. Behind the
/// `narration-heuristic-demo` feature; off the default build + replay path; names no consumer.
#[cfg(feature = "narration-heuristic-demo")]
pub mod narration_heuristic_demo;
/// H7–H12 narration-failure heuristic bank + deterministic detector — the residual-semiotics discipline applied to
/// the project's own narration generator. Always compiled (pure, off the replay path; its own catalogue hash).
pub mod narration_heuristics;
pub mod narrative;
pub mod ne107_adapter;
pub mod observability;
pub mod operator_reports;
pub mod passport;
pub mod permit_boundary;
pub mod pipeline;
pub mod plant_data_contract;
pub mod plant_twin;
pub mod precedent;
pub mod proof_obligations;
pub mod propagation;
pub mod provenance_graph;
pub mod redaction;
pub mod regime_envelope;
pub mod release_scrub;
pub mod report;
pub mod safety_dossier;
pub mod sensor_trust;
pub mod softsensor;
pub mod spec_limit;
pub mod spectral_grammar;
pub mod startup_shutdown;
pub mod sweep;
pub mod topology;
pub mod transition_pack;
pub mod uncertainty;
pub mod unit_consistency;
pub mod witness_strength;

// Kani reachability / proof harnesses are compiled only under the `kani` cfg flag
// (set by the Kani model-checker toolchain). They are never part of a regular build.
#[cfg(kani)]
mod kani_proofs;

pub use atlas::Corpus;
pub use data::{Baseline, DataMatrix};
pub use dsfb_core::{
    aggregate_episodes, deterministic_replay, AdmissibilityEnvelope, DeterministicDsfb, Episode,
    GrammarClassifier, GrammarState, NonIntrusiveGuarantee, ReasonCode, ResidualSample,
    ResidualTriple,
};
pub use pipeline::{analyze, AnalysisResult, Metrics, PipelineConfig};

/// Crate version string, surfaced in artifact manifests and the run `manifest.json`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
