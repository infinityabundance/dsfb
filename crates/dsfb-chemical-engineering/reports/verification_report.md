# DSFB-Chemical-Engineering — verification report (top-level pointer)

A one-click, root-level summary of what is verified. The **authoritative, verbatim, re-runnable** report
(capture host, exact commands, per-binary test counts, `.log` audit, frozen authority hashes) lives at:

→ [`crates/dsfb-chemical-engineering-edge/reports/verification_report.md`](../crates/dsfb-chemical-engineering-edge/reports/verification_report.md)

## Verified state (current)

| Check | Result |
|---|---|
| `cargo test --workspace` (CPU-only, no GPU needed) | **295 tests pass, 0 failed** = 291 unit/integration + 4 doc-tests (incl. the `no_std` core's 8, the `dsfb-densor-runtime` substrate's 8, +2 ArtifactIndexCourtV1, +4 NarrationContextV1 incl. the hallucination-gate round-trip, +3 H7–H12 narration-failure heuristic bank/detector). A further 3 run under `--features narration-heuristic-demo` (the adversarial narration demonstrator audit). |
| `dsfb-chemical-engineering-core` embedded | builds for **`thumbv7m-none-eabi`** + **`wasm32-unknown-unknown`**; **runs on emulated Cortex-M3** (QEMU `lm3s6965evb`) via the `qemu-smoke` harness (token-checksum `15794140667410667713`); memory budget ≈ **168 B/channel** (`N=8`), no heap — see [`docs/embedded_memory_budget.md`](../docs/embedded_memory_budget.md) |
| `dsfb-chemical-engineering-wasm` (browser what-if simulator) | builds the `core` grammar to **`wasm32-unknown-unknown`** (raw `extern "C"`, no wasm-bindgen) + a static HTML/JS shell; pure `simulate_into` logic host-tested (`cargo test`, **5/5**) — replays a residual stream under an operator-amended envelope over immutable evidence (advisory/training; not a controller) |
| CPU-vs-GPU end-to-end (evidence factory) | GPU **7.4× / 15.9× / 33.2×** over the CPU reference at 32 / 128 / 512 MB (still ~66× below the 637 GB/s roofline — value is auditability, R4) |
| `cargo test -p …-cuda --features cuda` (GPU) | **13 tests pass** — GPU↔CPU `evidence_root` byte-exact |
| `verify-replay` (deterministic synthetic suite) | **6/6 byte-identical** to frozen golden |
| Court Record bundles (`verify_reproducibility.py --bundles`) | **20/20** `bundle_root` + `evidence_root` match `EXPECTED_BUNDLE_ROOTS.toml` |
| `ArtifactCompletenessCourtV1` (`completeness-court`) | **COMPLETE (7/0)** — + an independent re-parse oracle |
| `breadth_surface.toml` self-check (`cargo test … breadth_surface_court`) | **7/0** — the top-level navigation index (every claim → artifact → reproduction → tier) is validated against reality: tiers resolve to `ClaimStrength`, CLI reproductions are real subcommands, artifacts exist, counts equal `atlas::validate()` + the manifests (18/7/12/6 · 20 · 6 · 6 · 60), Tier-3 claims state a boundary; canonical digest pinned |
| `PublicReleaseScrubCourtV1` (`release-scrub`) | **RELEASE-CLEAN (5/0)** — no placeholder DOI / private-backup leak / **controlled-access dataset rows outside `research/`** / **controlled roles sidecars declare the four no-rows provenance flags** (P87); required artifacts present. **Archive mode** (`--archive-dir <dir>`, P81/P82/P87) scrubs a materialised release tree and **fails hard** if a `SESSION_*` backup or a **controlled-access dataset row** (`swat/batadal/wadi *_instrumented.csv` or `*.witness.csv`) actually shipped, the `.gitignore`/`.gitattributes` hygiene config is missing, or a controlled `*_instrumented.roles.json` ships without its no-rows flags — run on the staged `git archive` before upload (PASS 6/0 on a clean archive; FAIL on a raw working-dir zip that includes `research/`) |
| `UnitConsistencyCourtV1` (`unit-consistency`) | **all 6 documented balances UNIT-CONSISTENT** — every additively-combined channel group shares units |
| Behavioural tests | every synthetic fault detected within its window; baseline not FP-dominated; compression ≥ 1× |
| Kani formal proofs (`cargo kani`) | grammar soundness harnesses verify **SUCCESSFUL** (interior-finite ≠ SensorFault; beyond-bound ≠ interior; `classify` total) |
| Lean 4 proofs (`cd formal/lean && lake build`) | **builds OK** — 7 theorems machine-verified over unbounded `Int`: the 3 Kani obligations re-proven unbounded **+ quorum soundness + episode-compression monotonicity** (previously open). Replay determinism stays empirical. |
| Coq / Rocq proofs (`cd formal/coq && coqc DsfbGrammar.v`) | **verifies** — the same obligations cross-checked in a second prover kernel (Rocq 9.1.1), using only the standard-library modules `List` / `ZArith` / `Lia` (no third-party Coq dependencies). |
| Paper (`bash paper/build_paper.sh`) | **66 pages, 0 overfull hbox, 0 undefined** (`.log`-audited; 22 benign underfull, not claimed) |
| Figure gallery (`dsfb-chem-edge figures`) | **60 figures**, SHA-256 manifest, re-render byte-identical |
| Frozen authority | `atlas_hash_v1 = 936ac67a…` · `corpus_hash_v1 = 7ce33a2e…` |

## Honest scope
This summary asserts the **machine-checkable artifact graph** (tests / replay / bundle roots / authority
hashes / paper audit) on the maintainer's host. It does not parse the PDF prose. Bounded throughout: advisory,
read-only, no root cause, no causality, no control or safety-instrumented-function authority. Governed re-freezes
(e.g. the Wave-1 degeneracy guard + the operator-report claim banner) are recorded in `EXPECTED_BUNDLE_ROOTS.toml`
and `PROJECT_PLAN.md`.

The Wave-3 physics objects (`UnitConsistencyCourtV1`, `SpecLimitWitnessV1`, `PermitBoundaryWitnessV1`,
`FirstPrinciplesWitnessAdapterV1` + `EquationResidualPassportV1`, the P75 balance-witness pack, and
`ResidualEnergyBudgetV1`) and the Wave-4 industrial-historian-layer objects (`IndustrialDataReadinessCourtV1`
+ `data-readiness` CLI, `MultiRateAlignmentCourtV1` + `ManualSampleBridgeV1`, `SetpointResidualSeparationV1` +
`ControllerModeGuardV1` + `ControlLoopInteractionMapV1`, `StartupShutdownEnvelopeV1`, and the context/QA
witnesses `MaterialLotWitnessV1` / `CertificateOfAnalysisDensorV1` / `CleanInPlaceWitnessV1` /
`SensorTrustDegradationLedgerV1` / `CalibrationEventWitnessV1` / `BatchGenealogyGraphV1` / `PlantTwinReplayV1`)
are each **additive, off the replay path, hash-sealed, self-verifying, and carry explicit non-claims** — they
widen the evidence surface without moving any frozen hash. None asserts a root cause, a calibrated state
estimate, regulatory compliance, or any control/safety authority. The real-ungated-CSV drop-in path is
documented in `docs/real_data_dropin.md` (full real-plant TRL-4→5 depends on a user-supplied export).

The Wave-6 **confidential-evaluation chain** (`PlantDataContractV1` + `HistorianImportReceiptV1`; the
data-quality objects `DataQualityEpisodeV1` / `FrozenTagDetectorV1` / `ClockSkewWitnessV1`; the observability
objects `InstrumentationCoverageMapV1` / `ObservabilityNonClaimReceiptV1` / `ResidualWitnessCoverageScoreV1`;
the evidence objects `ChemicalEventOntologyV1` / `EpisodeEvidenceLedgerV1` / `EvidenceMinimumsMatrixV1` /
`WitnessBurdenOfProofV1`; the anti-hallucination objects `AdversarialConfuserSuiteV1` /
`FalseNarrativeRegressionTestV1`; and the export objects `CaseFileRedactionMapV1` / `TamperEvidenceSealV1` /
`AuditTrailExportV1` / `ConfidentialEvaluationBundleV1` / `PartnerDataEscrowProtocolV1` /
`ChemicalProcessPlaybookV1`) makes the adoption headline concrete: **the operator runs the read-only court
locally and shares only a redacted, hash-linked evidence bundle — raw plant data never leaves their control.**
The `confidential-demo [--fixture <name>]` CLI (P84) wires that whole chain into **one command**: it runs the
local court on a synthetic fixture (default `cavitation_instrumented` — a gated incident that forms one
episode + an advisory playbook) and emits a shareable bundle (redaction map · tamper seal · audit
trail · confidential bundle · escrow protocol · per-episode playbooks) of **sealed JSON only — no CSV / raw
time-series**, asserting `is_shareable()` (every object self-verifies, `contains_raw_timeseries=false`, escrow
raw-never-egressed) before declaring success. A weak heuristic that fails its evidence burden is forced to emit
`unknown`, never a fabricated label.

The Wave-7 research objects (`Interval` + `PhysicsInformedEnvelopeV1` — interval-arithmetic model–plant
mismatch as first-class evidence; `MultiPhysicsCrossWitnessV1`; `HierarchicalMultiScaleFusionV1`;
`SpectralGrammarTokenV1`; `MerkleDagAmendmentChainV1`; `OperatorUncertaintyDashboardV1`;
`SignatureDiscoveryAssistantV1`; `DsfbBenchV1`; `SafetyCertificationDossierV1`; `ProofObligationLedgerV1`) are
likewise additive, sealed, self-verifying, and bounded. The grammar/fusion obligations are now machine-checked by **three independent tools** — Kani (bounded),
**Lean 4** (`formal/lean/`, unbounded over `Int`), and **Coq/Rocq** (`formal/coq/`, with `classifyAxis` over
`Z`); quorum soundness and episode-compression monotonicity are proven (not just stated), and only replay
determinism remains empirical. The `no_std` core compiles to **`wasm32-unknown-unknown`** (the WASM-simulator
substrate). The remaining research-grade items needing external infra (the WASM simulator shell, OPC-UA
streaming, heterogeneous/WGSL backend, open registry) are catalogued as handoff in
`docs/wave7_research_roadmap.md`, honouring the two curated AVOIDs (post-quantum hashing; a generic
cross-domain extraction).

The Wave-8 plant-reality + release-hardening objects make the artifact read like senior process engineering and
mechanise the release discipline. The five plant-reality objects — `HazopGuidewordMappingV1` (HAZOP No/More/Less/
Reverse guidewords → residual analogues), `BasisDescriptorV1` (wet/dry/mass/mole quantity basis with
`comparable_with`), `CalibrationModelPassportV1` (PAT/NIR calibration provenance: RMSEP/bias/leverage/Q-residual/
transfer, with an in/out-of-validation-range gate), `NamurNe107AdapterV1` (an *executable* DSFB-state → NAMUR NE 107
status adapter, test-pinned equal to the report's string mapping, every `Failure` only a witness-qualified
*candidate*), and the `EquipmentSignatureRecordV1` / `EquipmentSignatureBankV1` equipment-class bank (pump / heat
exchanger / reactor / column, with required/forbidden/supporting witnesses and an A–D burden tier) — are each
additive, self-sealed (their own per-object hash, **not** folded into `atlas_hash_v1`), self-verifying, and carry
explicit non-claims. Alongside them the release-hardening objects — `ControlledAccessDatasetPolicy` + the
controlled-data and controlled-roles-metadata scrub gates (P82/P87), `CudaOptimizationStatus` (the GPU
digest-equivalence verdict, not lane evidence), and the per-episode `EvidenceKind` column in the operator report
(via `EvidenceKind::from_witness_strength`, display-only — off the sealed `evidence_root`) — harden the discipline
without moving any frozen authority hash or the court goldens. Empirically, `cargo-fuzz` adds 225.2M executions
(0 crashes) over the pure grammar/lexer entry points as the unbounded-domain companion to the bounded Kani proofs.
