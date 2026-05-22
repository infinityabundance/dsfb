# DSFB-GPU-Debug — Timestamp Receipt

This receipt records public-accessibility evidence for the
DSFB-GPU-Debug prior-art evidence package. It enumerates the
sealed artifacts of the project, their SHA-256 hashes, the
sealed commit hash that produced them, the human-readable
public-accessibility URLs (where assigned), and the placeholder
slots that **RELEASE.1** will fill (DOI, Software Heritage SWHID,
Zenodo timestamp).

**Disclaimer (verbatim, plan-locked).** This receipt records
public-accessibility evidence for the DSFB-GPU prior-art package.
It is not legal advice and does not substitute for counsel.

---

## Repository identity

- **Project name:** DSFB-GPU-Debug
- **Organization:** Invariant Forge LLC (Delaware LLC No. 10529072)
- **Author:** Riaan de Beer (ORCID
  `https://orcid.org/0009-0006-1155-027X`)
- **Email:** [riaan@invariantforge.net](mailto:riaan@invariantforge.net)
- **Licensing email:**
  [licensing@invariantforge.net](mailto:licensing@invariantforge.net)
- **Repository URL:**
  [https://github.com/infinityabundance/dsfb](https://github.com/infinityabundance/dsfb)
- **License:** Apache-2.0 (see [`LICENSE`](LICENSE)); Background IP
  belongs to Invariant Forge LLC (see [`NOTICE`](NOTICE)).
- **Sealed commit at PA.1 seal:** `2281cc8` (PA.1).
- **Note on archive consumers:** [`.zenodo.json`](.zenodo.json)
  is a dotfile (plan-locked, because Zenodo's deposit-metadata
  convention expects exactly that filename). Some zip-extraction
  tools omit dotfiles by default; consumers building a clean
  archive of this package MUST include hidden files (the standard
  `git archive` command preserves them).

## Archived prior-art manuscript artifact

- **Archive locator:** DOI `10.5281/zenodo.20338027`, artifact
  `dsfb_gpu_debug.pdf`
- **Page count:** 140
- **File size:** 1,272,887 bytes
- **SHA-256:** `9b352c74a0cac631f2d53474e211757ca71d0e2abf490b8b5ef3b4bdf3dac7fa`
- **Title:** *DSFB-GPU-Debug: Clear-Box Deterministic Inference
  CUDA Acceleration for Replayable Trace-Event Verdicts*
- **Title subtitle:** *A Prior-Art Architecture for GPU-Accelerated
  Residual Signs, Detector Motifs, Bank-Governed Fusion, and
  Byte-Exact Case Files Without Probabilistic Models*
- **Built from:** archived PA.1 manuscript source (commit `117c237`).
- **Sealed paper sections at PA.1 seal:**
  - §1 Introduction
  - §2 Densorial and Tekmeric Inference (`sec:densorial`)
  - §2.1 Endoduction as the Fourth Mode of Inference
    (`sec:endoduction`)
  - §3 CUDA Evidence Factory + Fig 1
  - §4 Scientific Provenance Credit Pass
  - §5 Device Traffic Receipt (Measurement Law)
  - §6 Layer-A Resident Densor Pipeline
  - §7 Public-Data Saturation Bundle
  - §8 S-REAL Audit Gauntlet (20-Dataset Sealed Bundle) + Fig 2
  - §9 S-REAL Saturation Sweep (30-Fixture Classification) + Fig 3
  - §10 S-REAL Non-Claims Matrix
  - §11 Case Studies (TADBench F11 / C-MAPSS F4 / A6.1 + Fig 4 /
    RadioML / DB)
  - §12 Hardware Implications: Densorial / DPU Accelerator
    Architectural Read + Fig 5
  - §13 Active-Detector Family Compaction
  - §14 Effective-Bandwidth Report
  - §15 RTX 4080 SUPER Measured CUDA Pipeline Baseline
  - §16 Source-Report Import Verifier
  - §17 Batched-K Saturation Receipt (S-PERF.8.1 Hardening Pass)
  - §18 DigestLanePlanV1 / Digest-Cost Audit (S-PERF.10)
  - §19 Measured Digest-Lane Compaction (S-PERF.11)
  - §20 Post-S-PERF.11 Bottleneck Triage (S-PERF.11.1)
  - Appendix A.1–A.17 (Problem Statement through Conclusion)
  - Propositions (Deterministic Replay, Acceleration Preservation,
    Semantic Non-Bypass)

## Sealed bundle artifacts (S-REAL.3 audit gauntlet)

- **Bundle manifest:** `reports/s_real_3/bundle_manifest.toml`
  - SHA-256: `9d605f8584c37a88368719f4d74ca827e0a4ead638b692c65f2fa596f7ae5636`
  - Sealed at S-REAL.3 (`a8aaa04`).
- **60-row bundle hash chain:**
  `reports/s_real_3/bundle_hash_chain.txt`
  - SHA-256: `c809eb42fcc9758b85fcb18a0875384afdb272904b0d21febefa88f22115cebd`
  - Sealed at S-REAL.3 (`a8aaa04`); CI-guarded by
    `crates/dsfb-gpu-debug-demo/tests/s_real_3_bundle_integrity.rs`.
- **20 sealed datasets, 9 plan-locked audit artifacts each:**
  full per-dataset tier directories under
  `reports/s_real_<tier>/<dataset_id>/`.

## Sealed sweep + performance receipts

- **30-fixture saturation sweep:**
  `reports/s_real_saturation_sweep.txt`
  - SHA-256: `1598b70f2840b363247b63f462eb70863b5924bb047689c156c80b402b3530ef`
  - Sealed at S-REAL.3.1 (`fde8a99`); surface split sealed at
    S-REAL.3.1.2 (`6843a40`).
- **A6.1 post-Nsight receipt:** `reports/s_perf_16_a6_1_post.txt`
  - SHA-256: `caa81dcc2ac037ff48b8efcba2484bdebfbb63b7830d308cdd6428f0dce39394`
  - Sealed at S-PERF.16.a / A6.1 (`3e84e05`).

## Public replay surface (COLAB.S-REAL.1)

- **Path:** `notebooks/dsfb_gpu_debug_colab.ipynb`
  - SHA-256: `def8440b742a972bd37f556d39514316c3fa2e7bf8116e33904e0d8591731746`
  - Sealed at COLAB.S-REAL.1 (`3548366`); PENDING-guard sealed
    at S-REAL.3.1.2 (`6843a40`).
- **Operator README:** `notebooks/README.md`
- **Pack-for-Colab script:** `scripts/pack_for_colab.sh`
- **ZIP packager:** `scripts/package_s_real_colab_outputs.sh`

## Source-code identity

- **Workspace root:** `Cargo.toml`
- **Top-level crates:**
  - `dsfb-gpu-debug-core` (host-only deterministic court)
  - `dsfb-gpu-debug-cuda` (CUDA FFI + kernel dispatch)
  - `dsfb-gpu-debug-demo` (CLI binary + tests)
  - `dsfb-gpu-atlas-corpus` (corpus / S-PERF / S-REAL / FF / S1.3
    sealed surfaces)
  - `dsfb-gpu-atlas-registry` (registry generator scaffold)
- **CUDA kernel source:** `cuda/kernels.cu`, `cuda/sha256.cuh`,
  `cuda/common.cuh`, `cuda/layout.cuh`.

## Citation metadata

- **CITATION.cff:** [`CITATION.cff`](CITATION.cff) — see for the
  canonical citation entry, including author, ORCID,
  repository-code, keywords, and abstract.
- **CodeMeta JSON-LD:** [`codemeta.json`](codemeta.json).
- **Zenodo deposit metadata:** [`.zenodo.json`](.zenodo.json) (DOI
  pending RELEASE.1).
- **SPDX SBOM:** [`sbom.spdx.json`](sbom.spdx.json).

## RELEASE.1 — Public Archive Seal (SEALED 2026-05-22)

The v1.0 deposit was sealed to Zenodo on 2026-05-22:

- **Citation:** de Beer, R. (2026). *DSFB-GPU — Clear-Box Pure
  Deterministic Inference CUDA Acceleration for Replayable
  Trace-Event Verdicts — A Prior-Art Architecture for
  non-probabilistic, non-stochastic, non-weighted, GPU-Accelerated
  Residual Signs, Detector Motifs, Bank-Governed Fusion, and
  Byte-Exact Case Files Without Probabilistic Models* (v1.0).
  Zenodo. https://doi.org/10.5281/zenodo.20338027
- **Zenodo DOI:** [`10.5281/zenodo.20338027`](https://doi.org/10.5281/zenodo.20338027).
- **Zenodo concept DOI:** `10.5281/zenodo.20338027` (resolves to
  the current v1.0 deposit; subsequent versions get their own
  version-specific DOIs while the concept DOI tracks the latest).
- **Zenodo publication timestamp:** 2026-05-22.
- **Zenodo deposit URL:** https://zenodo.org/records/20338027.
- **Published version:** v1.0.
- **License (as deposited):** Apache-2.0 (reference implementation)
  + Invariant Forge LLC Background IP notice per the [`NOTICE`](NOTICE)
  file in this repository.


The Zenodo DOI is the primary §102 prior-art anchor and is now
publicly resolvable. GitHub release + Software Heritage SWHID are
additional public-accessibility surfaces that may be added later;
they do not gate the prior-art claim, which the Zenodo deposit
establishes on its own.

## Sealed campaign chain (most recent first)

- `0e5f8e3` PRIOR-ART-HARDENING.2 — panel-driven
  follow-on extending PRIOR-ART-HARDENING.1 with two reinforcements
  the post-HARDENING.1 panel surfaced as remaining concerns: (1)
  new subsection F.7.1 'Formal emission-type signature for the DPU
  primitives' added immediately after the F.7 architectural-contract
  enforcement paragraph; records the same Semantic Non-Bypass
  property as F.7 but at the type-signature level rather than in
  prose, with a Rust-style typed-emission-operation signature in a
  `lstlisting` block (WFU/MML/FOR/DRL emission types declared; the
  Bank::emit_admission_token signature requires a Bank state value
  as its first argument; no DPU primitive is declared with a Bank
  value in its emission-context type; the exclusion is structural
  at the type-signature level, not enforced by convention or
  runtime check). (2) F.9 cudaprog citation-reuse fixed: the
  RISC-V precedent claim now cites the precise authority
  `asanovic2014riscv` (Asanović & Patterson UCB/EECS-2014-146);
  the NVIDIA Hopper/Blackwell claim retains `cudaprog` but with
  precise scoping ('the CUDA Programming Guide is the published
  reference for the GPU-programming surfaces'); the Cerebras /
  SambaNova / Graphcore / Tenstorrent dataflow-accelerator claim
  drops the misplaced `cudaprog` citation entirely (CUDA
  Programming Guide is NOT the source for those vendors'
  architectures) and names them by project identifier only.
  Paper rebuilt: 140 pages (was 139; +1 page from F.7.1
  type-signature subsection), 1,272,887 bytes, SHA-256
  `9b352c74a0cac631f2d53474e211757ca71d0e2abf490b8b5ef3b4bdf3dac7fa`;
  overfull-\hbox = 5 (ceiling held). Paper-text only;
  corpus_hash_v1 / corpus_hash_v2 / every prior court anchor
  byte-identical; no kernel changes; no measurement changes;
  no R.12b rebaselining.
- `cbf392e` PRIOR-ART-HARDENING.1 — panel-driven
  hardening pass extending the PAPER-TRASHING-PURGE arc with three
  architectural prior-art reinforcements: (1) bibliography
  fact-check on the F.1.5 / F.1.6 load-bearing citations — Khan
  2015 corrected to the real authors Osama U. Khan + David D.
  Wentzloff (TVLSI 24(3) 837-845, 2016, DOI
  10.1109/TVLSI.2015.2420663); Abts 2020 Groq TSP author list
  corrected (Gagarin spelling fix + six fabricated trailing names
  replaced with the seven real additional authors per the Groq
  engineering team listing); Reuther 2020 HPEC ML Accelerator
  Survey + Lamb 2022 IEEE Software Reproducible Builds tightened
  with pages + DOIs; the other 7 load-bearing citations verified
  against published-archive metadata. (2) Tier-frame clarification
  paragraph added to F.0 (the architectural-prior-art posture is
  Tier 1 confidence in the *specification artifact* over an
  architectural-concept *substrate* that is Tier 3 — the apparent
  tier-mismatch the panel flagged is the correct prior-art
  posture). (3) Architectural-contract enforcement paragraph added
  to F.7: no architectural-output-type of any DPU primitive
  includes BankAdmissionToken; the token is the capability-typed
  output of a distinct architectural module (the CPU-side bank);
  the contract-level statement of Semantic Non-Bypass is the
  architectural-specification statement, not a gate-level RTL
  statement (broad prior-art posture, not narrow academic-wedge).
  Paper rebuilt: 139 pages (was 138; +1 page), 1,266,265 bytes,
  SHA-256
  `1c3b7472f6b38806e8824d12d68a97156134e59d0af11dc7c9406ff51266ae39`;
  overfull-\hbox = 5 (ceiling held). Paper-text + bibliography
  cleanups only; corpus_hash_v1 / corpus_hash_v2 / every prior
  court anchor byte-identical; no kernel changes; no measurement
  changes; no R.12b rebaselining.
- `6ba09f7` PAPER-TRASHING-PURGE.2 — panel-driven
  second sweep extending PAPER-TRASHING-PURGE.1 with ten
  additional paper-trashing cleanups: §S-PERF.10 CUDA path
  defensive sentence rewritten; §C-MAPSS case-study defensive
  tail rewritten; §Related Work non-claims itemize replaced
  with positive axis-difference framing; §LLM Cost-Compression
  Analysis non-claims itemize replaced with positive
  scope-of-measurement framing; §S-PERF.11 verifier description
  tightened removing internal-campaign-ID leakage
  ("plan directive" / "CAMPAIGN IDENTITY" / "bundles X inside
  the same atomic commit"); §Related Work Streaming-vs-replayable
  bullet "plan-deferred ... at PAPER.1g" wording removed;
  Appendix B preamble "pre-PAPER.1f" release-ID leakage removed;
  Appendix C Top-level-artifacts TIMESTAMP_RECEIPT bullet
  "RELEASE.1 slots" rephrased as "public-archive identifier
  slots"; Appendix C Element-18 description "plan-deferred to
  S-REAL.4 (post-RELEASE.1)" rephrased as "lands in a
  follow-on bundle-partition campaign"; §Conclusion T.13.GAP
  paragraph "plan-locked survey taxonomies" /
  "CAMPAIGN IDENTITY load-bearing negative" / explicit "does NOT"
  bullets rephrased as positive scope descriptions; Appendix G
  G.5 / G.6 "CAMPAIGN IDENTITY" / "plan-required" internal
  jargon replaced with "campaign-defining" / "load-bearing";
  Appendix D sm_100+ row "at PAPER.1f" release-ID leakage
  rephrased as "at this release"; Appendix E.2 commit-hash
  placeholder rewritten to reference TIMESTAMP_RECEIPT.md
  sealed-campaign chain. Paper now reads as a self-contained
  prior-art instrument with no exposed release-management
  vocabulary, no defensive non-claim itemizes that pre-emptively
  volunteer absences, no internal-jargon labels leaking
  development-process scaffolding to readers. Paper-text-only;
  corpus_hash_v1 / corpus_hash_v2 / every prior court anchor
  byte-identical; no kernel changes; no measurement changes;
  no R.12b rebaselining. Paper rebuilt: 138 pages, 1,260,134
  bytes, SHA-256
  `45e6396b704645bea80df925c8354402aa68c250672ebdad7ee56af1bdd70032`;
  overfull-\hbox count = 5 (ceiling held).
- `8948e78` PAPER-TRASHING-PURGE.1 — comprehensive
  removal of paper-trashing absence-disclosures and self-disclaiming
  legal-scope language scattered across the paper body. Every
  instance of patentability self-disclaim, every "not legal advice"
  TIMESTAMP_RECEIPT-disclaimer reference, every "no silicon / no
  fabrication / no tape-out / no FPGA bitstream / no synthesis
  report exists / silicon does not exist / no thermal envelope /
  NOT a measurement / NOT a deployment claim / no DPU implementation
  / no silicon claim / no silicon-level commit / does not prove a
  DPU exists / does not claim such a prototype exists / Tier 3
  architectural-read non-claim band / deliberately not claimed /
  not claimed as a philosophical novelty / not claimed as
  exhaustive" phrase removed from the paper body. Locations:
  F.0 thesis defensive sentence; F.0 "What this appendix is, and
  is not" rewritten as positive scope; F.1.5 "What the class
  declaration does NOT claim" itemize replaced with positive
  scope paragraph; F.1.5 architecture-first declaration paragraph
  tightened; F.1.6 "What the class declaration does NOT claim"
  itemize replaced with positive scope paragraph; F.3/F.4/F.5
  throughput-target defensive bullets removed; F.8 cross-architecture
  posture defensive bullets removed; F.9 "What is not claimed"
  paragraph removed; F.10 future-work preamble defensive framing
  removed; F.11 retitled from "Plan-locked non-claims" to "Scope"
  with paper-trashing items removed; Appendix F section heading
  "(Tier 3 Architectural Read)" suffix removed; §Hardware
  Implications Figure 5 caption non-claim tail removed; §Hardware
  Implications closing "Plan-locked non-claims" paragraph removed;
  §Endoduction "not claimed as a philosophical novelty / not
  claimed as exhaustive" tail removed; §Limitations "No legal /
  patent advice" bullet removed; Q14 SBIR-operator "Patentability
  is not claimed / not legal advice disclaimer" sentence removed;
  E.8 reproducibility-checklist "does NOT certify novelty /
  patentability / prior-art ruling" bullet removed; Appendix C
  "does not assert new prior-art rulings" editorial-scope paragraph
  tightened. Architectural specification stands on its own merits
  as prior art. Paper-text-only; no kernel changes; no measurement
  changes; corpus_hash_v1 / corpus_hash_v2 / every prior court
  anchor byte-identical.
- `8a964f9` DPU.1.B.3 — F.10 Future Work paper-text-only
  cleanup. Removes three paper-trashing defensive sentences from
  the future-work enumeration: the DPU.3 FPGA-prototype bullet's
  trailing "this appendix does NOT claim such a prototype exists"
  clause; the DPU.4 ASIC-feasibility-study bullet's trailing
  "this appendix does NOT claim such estimates exist" clause;
  the DPU.5 cross-vendor portability-abstraction bullet's trailing
  "its design is not declared in this appendix" clause. Defensive
  non-claim language belongs in F.11's dedicated plan-locked
  non-claims band where it is properly scoped; scattering "does
  NOT claim X exists" through future work pre-emptively announces
  gaps and undermines the architectural prior-art posture. Future
  work in academic style describes the next step; it does not
  volunteer absences. F.11 retains all twelve plan-locked non-claims
  and continues to bound the scope honestly. Paper-text-only
  cleanup; corpus_hash_v1 / corpus_hash_v2 / every prior court
  anchor byte-identical.
- `1022f1d` DPU.1.B.2 — F.1.5 prior-art-floor tightening.
  Removes the "subsumed as instances" giveaway that had
  incorrectly granted regex / DFA / Bloom-filter / SAT / FFT
  silicon instance status of the deterministic-inference
  accelerator class. Those systems perform pattern matching /
  probabilistic membership queries (with non-zero false-positive
  rate by construction) / boolean decision procedures / pure
  signal transforms — not inference over evidence. They share at
  most one property (architectural determinism on the declared
  operation) and lack the structural-output, evidence-emission-
  with-separated-admission, hash-chained-output, and Semantic-
  Non-Bypass-Axiom properties that constitute the full class
  definition. Reframed as adjacent fixed-function computational
  primitives that could be composed into a deterministic-inference
  accelerator as sub-primitives (a regex ASIC could serve as a
  sequential-pattern detector lane inside an MML; an FFT DSP could
  serve as a spectral-witness lane) but are not themselves
  instances of the class. Corresponding non-claim line that
  granted them instance status removed. Class-level summary
  paragraph rewritten to name the DPU specification as the
  load-bearing prior-art instrument for the class definition
  itself. Paper-text-only correction; corpus_hash_v1 / corpus_hash_v2
  / every prior court anchor byte-identical.
- `9a4053c` DPU.1.B — TWO new Appendix F subsections
  lifting the DPU specification from a single-architecture proposal
  into TWO class-level prior-art claims for the deterministic-
  inference accelerator class. F.1.5 *The deterministic-inference
  accelerator class (general prior-art claim)* defines the class by
  four first-class published-contract properties (inference output
  is structural-not-probabilistic; evidence-emission with separated
  admission; architectural determinism contract enforced at the
  architecture level rather than as an opt-in runtime flag;
  hash-chained output contract); distinguishes the class from
  "accelerator with deterministic compute substrate" at the
  architectural-output layer; honestly bounds the prior-art floor
  by naming five reference points the class is NOT (Groq TSP/LPU
  as the strongest substrate-determinism instance, EigenAI-style
  byte-replay layers, adjacent fixed-function computational
  primitives (regex/DFA/Bloom/SAT/FFT silicon) which share at most
  one property (architectural determinism on the declared
  operation) and are NOT instances of the class, the symmetric
  probabilistic-inference
  accelerator class, and reference accelerator-architecture
  surveys); declares the class under the RISC-V architecture-first
  precedent (Asanović & Patterson UCB/EECS-2014-146). F.1.6
  *Full-stack clear-box replayability (silicon to case file)*
  declares a second first-class class-level property — every byte
  the accelerator emits at every layer from silicon up is replayable
  by re-execution under the same inputs and same declared contract;
  introduces the clear-box vocabulary as a deliberate term-of-art
  for execution replayability orthogonal to the established
  glass-box vocabulary for model-class interpretability; lays out
  the eight-layer audit-chain ladder (silicon determinism contract
  / toolchain pinning / kernel byte-stable output / dispatcher
  replay anchors / evidence-emission chain / host-side authority
  boundary / case-file tamper-evident log / external provenance
  receipts) each with its established-literature anchor; articulates
  the substantive prior-art distinction as "opportunistic hashing
  vs structural hashing" per Lamb & Zacchiroli 2022 IEEE Software;
  honestly bounds the prior-art floor by naming seven reference
  points the class is NOT (TEEs and confidential computing,
  verifiable-inference layers over existing accelerators, zero-
  knowledge inference, supply-chain attestation, MLPerf
  reproducibility + PyTorch / TensorFlow deterministic flags,
  post-hoc explainability, verified-software stacks). Eleven new
  bib entries added across both subsections: `asanovic2014riscv`,
  `abts2020tsp`, `goodfellow2016deep`, `khan2015probabilistic`,
  `reuther2020survey`, `lamb2022reproducible`, `haber1991timestamp`,
  `klein2009sel4`, `costan2016sgx`, `crosby2009tamper`,
  `torres2019intoto`. Twelve plan-locked non-claims total close the
  two subsections (six per subsection). DPU.1.B is paper-text-only;
  no kernel changes; no measurement changes; corpus_hash_v1 /
  corpus_hash_v2 / every prior court anchor byte-identical.
- `2fbaf03` T.13.GAP.PAPER.A — Appendix G scientific
  provenance expansion (36 surveyed methods × named scientist +
  year + disposition across the seven survey panels, mirroring
  the T.12.PROV "DSFB-GPU-Atlas does not erase prior detector
  science" doctrine into Appendix G); plus removal of three
  internal-operational lines from the previous paper-side
  non-claims band (DOI / SWHID slot reference + commit-locality
  line removed) and one internal-methodology line from the
  S-REAL Non-Claims Matrix; reviewer-facing audit hygiene
  pass.
- `f7e6d9e` T.13.GAP.PAPER — Appendix G: Deterministic
  Witness Family Gap Audit (T.13.GAP); paper-side surface of the
  T.13.GAP corpus campaign sealed at `7d7729f`. Documents the
  seven plan-locked survey panels (Classic outlier / Time-series
  anomaly / SPC / Streaming sketch / Graph topology / Robust
  statistics / Deterministic ML-adjacent), twelve disposition
  buckets, four own-namespace hashes, ten plan-required load-
  bearing negatives (CAMPAIGN IDENTITY: completeness-claim
  scanner), and four deterministic gap candidates RECORDED but
  NOT promoted (Matrix Profile / SAX / persistent-homology
  summary / Minimum Covariance Determinant). Audit-not-claim
  framing carried verbatim from the corpus module.
- `7d7729f` T.13.GAP — Deterministic Witness Family Gap Audit
  (corpus module + 33 acceptance tests + 4 own-namespace
  hashes)
- `7c36312` DPU.1.1 — PA.1.1-style placeholder rotation
- `4046668` DPU.1 — Appendix F: Densorial Processing
  Unit Conceptual Architecture Specification (Tier 3 architectural
  read; 4 primitives WFU / MML / FOR / DRL; determinism contract;
  hash-chain topology; related work; future work DPU.2–DPU.5;
  plan-locked non-claims band). Architectural prior-art at silicon
  concept level; no silicon claim.
- `ea1caa3` PLAN.REBRAND.1 — panel→plan documentation rebrand
  (no hash mutation; code-side enum-wire rename deferred to
  PLAN.REBRAND.2 + corpus\_hash\_v3 freeze)
- `920a66d` PAPER.1l — Legendary Limitations expansion + 20-
  question critical-reviewer anticipation set (CUDA / GPU /
  statistics / LLM-AI / SBIR / DSFB / academic perspectives)
- `dd93d95` PAPER.1k — Hostile-reviewer triple pass
  (CUDA / statistics / SBIR-patent perspectives; 2 minimal
  tightening edits)
- `17003b9` PAPER.1j — LLM Cost-Compression Analysis
  (byte-honest from sealed bundle bytes only; 1.82× / 4.45×
  ratios; 200K / 1M context fit; no LLM run)
- `cd54c93` PAPER.1i — Appendix E NeurIPS-style reproducibility
  checklist + `sec:bank-governed-episode-collapse` label fix
- `a398978` PAPER.1h — Related Work and State-of-the-Industry
  Placement section
- `35717fd` PAPER.1g — Cross-Architecture Determinism and
  Reproducibility Scope appendix (Appendix D)
- `fb2742e` PAPER.1f — Reader's Route + Appendix B Atlas
  Continuation Ledger + Appendix C Prior-Art Enablement +
  Nsight Compute measurement-discipline subsection +
  author-year bibliography + uniform `panel-locked`→`plan-locked`
  terminology rebrand across 220 mentions
- `117c237` PAPER.1e — §2.1 *Endoduction* + light threading +
  hostile-reviewer audit
- `72f5b31` PAPER.1d — five claim-bounded TikZ figures +
  robust `\dsfbid`
- `6843a40` S-REAL.3.1.2 — public-runner hygiene (audit /
  saturation surface split + tier-aware output paths +
  plan-locked artifact wording + notebook PENDING guard +
  bundled `PIN_AIOPS` ingest pin refresh)
- `eed6fe2` PAPER.1c — Case Studies (4 subsections) +
  Hardware Implications / DPU section
- `5f890c0` PAPER.1b — S-REAL Audit Gauntlet section +
  Saturation Sweep section + Non-Claims Matrix table
- `090b080` PAPER.1a — claim-hierarchy tcolorbox + key
  protective sentence + COLAB.S-REAL.1 reproducibility binding
- `3548366` COLAB.S-REAL.1 — public Colab replay notebook
- `3fdf42f` S-REAL.3.1.1 — hygiene close-out
- `fde8a99` S-REAL.3.1 — bundle integrity gate + saturation sweep
- `a8aaa04` S-REAL.3 — 20-dataset sealed bundle
- `3e84e05` S-PERF.16.a / A6.1 — structural fusion optimisation

Full historical chain available via `git log --oneline`.

## Verification protocol (operator-side)

To verify the public-accessibility properties of this package:

```bash
# 1. Clone the repository at the sealed commit.
git clone https://github.com/infinityabundance/dsfb dsfb-gpu
cd dsfb-gpu
git checkout 2281cc8  # PA.1 seal (prior-art evidence package)

# 2. Verify every artifact's SHA-256 against ARTIFACT_MANIFEST.v1.toml.
#    (The test `artifact_manifest_paths_exist` does this automatically.)
cargo test -p dsfb-gpu-debug-demo --test pa_1_prior_art_evidence_invariants

# 3. Re-run the S-REAL audit gauntlet end-to-end on local CUDA.
cargo build --release --features cuda
./target/release/dsfb-gpu-debug s-real-audit --dataset all --out-dir reports

# 4. (Operator option) Re-run the public Colab replay surface.
#    See notebooks/README.md for the COLAB.S-REAL.1 protocol.
```

## Cross-references

- [`PRIOR_ART_MAP.md`](PRIOR_ART_MAP.md) — 17 disclosed
  architecture elements mapped to code / tests / receipts /
  hashes.
- [`CLAIM_BOUNDARY_MATRIX.md`](CLAIM_BOUNDARY_MATRIX.md) — what is
  disclosed vs what is NOT claimed.
- [`ARTIFACT_MANIFEST.v1.toml`](ARTIFACT_MANIFEST.v1.toml) —
  machine-readable artifact index.
- [`CITATION.cff`](CITATION.cff) — citation metadata.
- [`codemeta.json`](codemeta.json) — CodeMeta JSON-LD.
- [`.zenodo.json`](.zenodo.json) — Zenodo deposit metadata.
- [`sbom.spdx.json`](sbom.spdx.json) — SPDX SBOM.
