# Changelog — DSFB-Chemical-Engineering

All notable changes. The authoritative per-phase development ledger is in `PROJECT_PLAN.md`; this file is the
human-readable summary. Versioning is workspace-wide (`0.1.0`); this is a prior-art artifact, not a released
package (publishing is maintainer-only).

## Unreleased
- **Narrator-generator extension — H7–H12 narration-failure heuristic bank + feature-gated demonstrator:** the
  residual-semiotics / evidence-court discipline applied to the project's own narration generator.
  `crate::narration_heuristics` catalogues H7 claim-tier breach · H8 unanchored sentence (the existing gate) · H9
  forbidden-claim phrasing · H10 unknown-relabeling · H11 cross-episode inference · H12 anchor-coverage drift, each
  with honest false-positive/false-negative modes, sealed by its own `narration_heuristics_hash_v1`, with a
  deterministic `detect` that makes the constrained-narration contract *mechanically enforceable*, not merely stated.
  A feature-gated synthetic demonstrator (`--features narration-heuristic-demo`, subcommand `narration-heuristic-demo`)
  trips each failure exactly while a faithful template narrative trips none; the bank + detector are always compiled
  and tested (+3), the adversarial constructors live behind the feature (+3 audit tests there). Catalogued in
  `docs/narration_heuristics_h7_h12.md`; breadth claim CASE-03 + the new subcommand re-froze the breadth-surface hash
  (SUBCOMMANDS 21→22). Off the replay path; the new doc joined the index inventory.
- **release-scrub simplification:** removed a redundant tracked-text hygiene scan that duplicated a commit-time +
  never-push discipline (not a file-level concern); `RELEASE-CLEAN` 6/0 → 5/0, the REL-01 breadth claim narrowed to
  the checks it performs (placeholder DOI + private-backup leak). The court `report_hash` + the artifact index were
  re-minted accordingly. Workspace **295** tests (0 failed).
- **Constrained-narration context emitter (auto-emit, purpose-unnamed):** `dsfb-chem-edge narration-context
  <dataset>` (and `casefile` auto-emit) writes a deterministic `narration_context.{md,json}` — the complete
  vocabulary of citable evidence **anchors** for a Court Record (one per fused episode, each with its claim tier /
  evidence kind / witness rung), the binding contract, and the non-claims, sealed by a `context_root`. So a
  downstream constrained narrator can produce only anchored, tier-bounded sentences and `NoNarrativeHallucination
  GateV1` rejects the rest (proven by a gate round-trip test). The per-episode anchor was factored into a shared
  `report::episode_evidence_anchor` (byte-identical refactor → no bundle re-mint). The emitted document names **no
  consumer** — zero "LLM"/"AI" (test-asserted) — it states only the rules + the sealed evidence. A committed,
  byte-checked reference is at `reports/narration_context_sample.md`, linked from the index reviewer routes.
  Workspace 289→**293** tests.
- **Self-verification + navigation + substrate-demo batch (P98–P102):** `ArtifactIndexCourtV1` + a `verify-index`
  CLI that cross-checks the committed `reports/index.json` against the live artifacts (index_root re-derives,
  paper/figure/bundle digests match, doc + crate links resolve, policy + court hashes agree) → INDEX-VERIFIED 9/0;
  `build_public_archive.sh` now emits a generated `reports/public_archive_proof.md` (commit + manifest hash + scrub
  verdict + no-session/no-controlled-row proof; git-ignored deposit-time artifact); reviewer-route §0
  sections in the artifact index (chemical / Rust / CUDA / embedded / SBIR-operator / release-auditor, each linking
  5–8 real artifacts) + copyable reproduce commands; an optional `densor-runtime-demo` feature carrying edge
  episodes through the `dsfb-densor-runtime` substrate (two authority-gated stages → sealed RuntimeReceiptV1) with
  no chemical logic moved in; and **P102** per-episode ClaimStrength + EvidenceAnchor columns in the operator report
  (governed display-only re-mint: all 20 bundle_roots + court golden shifted, all 20 evidence_roots byte-UNCHANGED).
  Also a **governed correction**: the earlier A1 degeneracy-guard floor was wrongly reported hash-neutral — it
  genuinely changed cstr_reactor's evidence_root (constant-channel) + cstr/three_tank bundle_roots; re-frozen with a
  note (the bundle check had read a stale demo dir). Workspace 287→**289** tests.
- **Panel-review hardening batch (all hash-neutral — no frozen evidence/bundle/authority hash moved):** an
  SPE/T² **degeneracy guard** in `linalg` (z-scoring + score-variance floors `1e-12`→`1e-8`, capping the
  `~1e35` blow-up on near-constant baseline columns; verified hash-neutral on all 6 synthetic + 20 datasets);
  surfaced previously-**swallowed write failures** in the historian + figure-bundle paths (no more silent
  malformed forensic bundles on a full disk); pinned the **`CanonicalHasher` f64q** float-boundary contract with
  tests; proved the executed detector set is an **edge↔atlas bijection** (every atlas executed-authority record is
  built by `build_bank`); added **runnable doc-tests** on the load-bearing public APIs; a **CI workflow**
  (`.github/workflows/ci.yml`) mirroring the hand-run gates + an optional GPU evidence-root-parity job; and three
  **disclosure docs** (regime-conditioning applicability framework, a balance-band commissioning-estimation
  procedure, an atlas no-pre-screen selection statement) + a GPU realistic-lane timing handoff. Workspace
  277→**287** tests (283 unit/integration + 4 doc-tests).
- **Navigation + substrate + release-confidence batch (P92–P97):** a `generate-index` CLI emitting a deterministic,
  self-contained `reports/index.{html,json}` + a sealed `index_root` over the committed inventory; a new sixth
  workspace member **`dsfb-densor-runtime`** — a thin `#![forbid(unsafe_code)]` deterministic execution-substrate
  skeleton carrying **no chemical / no cross-domain claims** (Miri-clean, 8/8; dsfb-gray 70.2 % on first audit);
  `scripts/build_public_archive.sh` (git-archive → release-scrub → per-file SHA-256 manifest); a constrained
  narrator-over-sealed-evidence contract (`docs/constrained_narration_extension.md`); typed `evidence_kind` on the
  confidential bundle (self-sealed) and on the court-record `EpisodeBadge` (**hash-neutral** — `casefile.json` is the
  manifest, not a hashed content file); four more controlled-sidecar role flags + an extended scrub gate; a CUDA
  evidence-format appendix table (paper 61→**62** pp); a less-concessive Q94 answer; and a README front-door that
  leads with `reports/index.html`. The `edge` dsfb-gray score rose 62.6 %→65.6 % on the well-commented new code.
- **Audit suite** (`audit/`): dsfb-gray assurance scores + Rust security/UB suite (cargo-audit, cargo-geiger,
  cargo-auditable, cargo-vet, cargo-crev, cargo-scan, Miri, static panic-surface analysis) with an HTML
  dashboard and README badges.
- **Verification-tools campaign — all run in-sandbox** (`audit/{cargo-fuzz,cargo-valgrind,creusot,flux,hax,crux-mir,loom}/`):
  **cargo-fuzz** (libFuzzer+ASan, 225.2M executions, 0 crashes over 3 pure targets); **valgrind** Memcheck **CLEAN
  (0 errors)** on the static-musl pipeline (the glibc route SIGILLs on this CachyOS x86-64-v4 host's compiled-in
  AVX-512 — proven: even `/usr/bin/true` SIGILLs under valgrind); **hax** extracted the `no_std` core to a 716-line
  **F\*** model; **Creusot** built `creusot-rustc` and translated `classify_axis` to **Coma** verification IR (SMT
  discharge pending its hermetic forked why3); **Flux** checks the `core` crate clean; **crux-mir** (Galois Crucible,
  a second symbolic engine) proved the core invariants **`Valid` (4/4)** — corroborating Kani; **loom** is N/A by
  design (no shared-state concurrency).
- **WASM what-if Chemical Court simulator** (`dsfb-chemical-engineering-wasm`, standalone).
- **Phase-C executable:** detector bank 14→18, F2 pump cavitation 6→7 fault signatures (governed re-freezes).
- **Plant-reality / evidence objects (Wave-8):** `HazopGuidewordMappingV1`, `BasisDescriptorV1`,
  `CalibrationModelPassportV1`, `NamurNe107AdapterV1`, `EquipmentSignatureRecordV1`/`EquipmentSignatureBankV1`
  (each self-sealed, off the replay path); per-episode `EvidenceKind` in the operator report and (P88) on
  `ChemicalProcessPlaybookV1`; `CudaOptimizationStatus`.
- **Release hardening:** `ControlledAccessDatasetPolicy` + controlled-data scrub gate (P82) and controlled-roles
  metadata gate (P87, `RELEASE-CLEAN 5/0`); `docs/public_archive_proof.md` (P89);
  the CUDA evidence-format table (P91); the SWaT/BATADAL real-data CSVs quarantined to untracked `research/`.
- **Panel-9.5 batch:** archive-mode `release-scrub`, `EvidenceKind` taxonomy, universal operator-report legend,
  proof-wording reconciliation, `confidential-demo` CLI, embedded memory-budget doc, `breadth_surface.toml`
  navigation layer + self-check court, SBIR 30/60/90 workplan.
- Three machine provers (Kani bounded · Lean 4 unbounded over Int · Coq/Rocq cross-check).
- **Paper — legendary limitations + 100-question hostile-reviewer surface:** structural limitations 12→22 and
  the critical-reviewer attack surface 20→100 questions across 17 personas; plus two anti-overclaim tables (P90:
  evidence-strength hierarchy, surface-status ledger); the paper grew 51→**61 pages** (0 overfull / 0 undefined,
  `.log`-audited).
- **Kani folded into the audit stack** (`audit/kani/`): 6/6 grammar-soundness harnesses verified (1047 checks,
  0 failures); **Miri** expanded to the three `no_std` crates (core 8/8 + atlas 7/7 + corpus 7/7, no UB).
- Workspace test suite grew 242 → **293** (0 failed) across these batches.

## 1.0 — 2026-05-23
- Initial prior-art deposit: 5 workspace crates (edge/cuda/atlas/corpus/core) + 2 standalone (py/wasm),
  20 datasets, 18/57 executed detectors, 7/12 executed fault signatures, 60-figure gallery, 51-page paper.
