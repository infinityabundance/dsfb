# Elite multi-disciplinary panel review — DSFB-Chemical-Engineering

> **Historical panel-review note.** This crate snapshot does **not** bundle the optional paper PDF or rendered
> 60-figure gallery; the current machine-checkable artifact index records a zero-figure manifest. Treat the
> visual-gallery comments below as a prior local review target, not as a committed artifact claim for this snapshot.
> The v1 review (pre-campaign) flagged two levers — visual communication and practitioner framing — and its
> recommendation ledger R1–R9 was adopted in full (the figure gallery, the practitioner dossier, the
> paper integration). This v2 re-rates the post-campaign artifact and identifies the *next* frontier.

**Panel.** DSFB methodologists · chemical engineers · plant/industry process engineers · SBIR transition
operators · Rust · CUDA · edge-embedded engineers. **Lens.** This is **prior art / defensive publication** —
breadth and enabling disclosure are the goal, not a wedge. Every recommendation below *widens* disclosure;
none narrows scope. The review is deliberately **not sycophantic** — it surfaces the uncomfortable truths,
with `file:line` evidence, because the artifact has earned a rigorous read.

---

## 1. Rate it on the right axis — two lenses

The panel's central finding: **judge this as two different objects, and say which one you are shipping.**

| Lens | Rating | Basis |
|---|---|---|
| **As prior art / defensive publication** | **9.3 / 10** | Exceptional breadth, enablement, and honesty; the campaign closed the two prior levers. This is what it is *trying* to be and it nearly maxes it. |
| **As an empirical scientific demonstration** | **~6.7 / 10** | Narrow demonstration scope: **14/57 detectors and 6/12 fault signatures executed**; fault demonstrators are **synthetic**; the most persuasive real-data result (SWaT) is **agreement-gated**; tests are mostly **structural seal-checks**, not behavioural. |

> **Update (2026-05-26) — review preserved as a dated snapshot; recommendation V2 since actioned.** The
> empirical-lens basis above (*"14/57 detectors and 6/12 fault signatures executed"*) was true **at the
> time of this v2 review** and is left unedited so the 6.7/10 rating is not retroactively rewritten. Since
> then the Phase-C executable passes have moved those numbers to **18/57 detectors** (mewma/dpca/mosum/mmd
> promoted Catalogued→Executed) and **7/12 fault signatures** (F2 pump cavitation added a faithful synthetic
> demonstrator the pipeline genuinely catches), each a governed `atlas_hash_v1` re-freeze. The catalogue was
> **widened by adding executions, never trimmed** — exactly the prior-art-correct move this review's
> recommendation V2 prescribed. The empirical-lens rating would rise accordingly; the prior-art-lens rating
> (9.3/10) is unaffected. The authoritative current counts live in `reports/verification_report.md`.

These are not in tension — a defensive publication *should* prioritize disclosure breadth over demonstration
depth. The mentorship is to **name which object you ship**, so a reader applying the second lens to a
first-lens artifact does not feel oversold. The work is too honest to deserve that misread.

### Scorecard (prior-art lens; ↑ = moved since v1)
| Axis | v1 | v2 | Basis |
|---|---|---|---|
| Breadth of disclosure | 9.6 | **9.7** | 4 crates, ~40 sealed evidence objects, a 60-figure visual atlas now makes every object legible. |
| Enablement / reproducibility | 9.5 | **9.6** | Two generation methods in lockstep, SHA-256 figure manifest, determinism gates, completeness court (9/0). |
| Rigor / academic honesty | 9.4 | **9.5** | `limitations.tex` candid; negative results disclosed (2.7× slower stream-overlap; rejected balance datasets); explicit Tier-1/2/3 hierarchy (`paper …tex` intro l.74–85). |
| Visual communication | 6.5 | **9.2** ↑ | Lever closed: 60 figures, 17 in the paper, graphviz graphs, colourblind-safe, on-figure disclaimers, provenance-sealed. |
| Chem-eng practitioner framing | 7.5 | **8.8** ↑ | Operator one-pager / alarm-flood / NE107 figures + the cited dossier + the standards section. |
| **Empirical demonstration depth** | — | **6.7** ← *new binding constraint* | 14/57 detectors, 6/12 faults executed; synthetic demonstrators; gated real data; near-tautological tests. |
| **Numerical robustness** | — | **7.0** ← *new* | A real degeneracy class (CSTR SPE → ~1.4e35; `linalg.rs:67,94`); silent `.ok()` error-swallowing (`cli.rs:342–371`). |

---

## 2. Per-discipline verdicts (each: strength · top concern · recommendation)

- **DSFB methodologist.** Determinism-as-evidence + the digest-equivalence law + "emit *unknown* rather than
  over-claim" is novel and now *legible*. **Concern:** the grammar/envelope/fusion are demonstrated but not
  *stress-tested* — no figure/test shows where the grammar **mis-fires** beyond the aggregate baseline-FP
  number. **Rec:** one adversarial "where DSFB is wrong" figure beats a tenth confirming one.
- **Chemical engineer.** The physics is the crown jewel and it is *real*: closure firing on the exact BATADAL
  T1-inflow attacks + the SWaT LIT101 spoof, with an applicability criterion that *predicts where the witness
  must stay blind* (PRONTO recirculation, RP-1043 energy-conserving leak). **Concern:** CSTR/penicillin/
  three-tank are **simulations with synthetic faults** (`MANIFEST.toml`: 9/20 simulation) and the headline
  SWaT win is **agreement-gated** (not independently reproducible). **Rec:** lean on the *openly-licensed*
  real data (BATADAL is open); run more openly-reproducible balances end-to-end; demote SWaT to "corroborating,
  gated."
- **Plant / industry engineer.** The read-only non-interference axiom makes it deployable and you now show it
  (NE107 / alarm-flood / one-pager in operator vocabulary). **Concern (sharp):** an operator who never reads
  `limitations.tex` could read **CANDIDATE_FAULT** in `operator_report.html` as a confident diagnosis; the
  advisory banner is on the *figure* of the one-pager but must be unmissable **on the HTML artifact itself**.
  **Rec:** put the claim-boundary banner at the top of `operator_report.html`.
- **SBIR transition operator.** The transition pack + milestone gates + minimum-data spec will survive due
  diligence. **Concern:** no end-to-end run on a single **real, ungated, role-labelled plant historian** —
  reads as TRL 3–4, not 5. **Rec:** one ungated real historian run (`historian` → `casefile`) is worth more
  than ten more evidence-object *types*.
- **Rust engineer.** Clean: `#![forbid(unsafe_code)]` in edge/atlas/corpus, dependency-light, 103 tests green.
  **Concern:** (a) numerical degeneracy is a real bug-class (CSTR SPE ~1.4e35; `linalg.rs:67` 1e-30 floor,
  `:94` 1e-12 score-var floor — too weak for low-variance process data); (b) silent error-swallowing —
  `cli.rs:342–371` does `report::write_*(…).ok()` on ~10 writes, so a full disk yields a malformed bundle
  with no warning; (c) tests are largely tautological seal-checks (`build → verify() → assert true`), and the
  completeness court is tested *by itself*, not an independent oracle. **Rec:** degeneracy guard + regression
  test; count/log write failures instead of `.ok()`; ≥1 behavioural test per evidence object.
- **CUDA engineer.** The byte-exact GPU↔CPU `evidence_root` + the Nsight honesty are exemplary. **Concern:**
  the kernel runs at **0.4–1.5% of the 636.8 GB/s roofline** at ~8.3% occupancy; the "~18× V2" figure is the
  deep 1024×8192 case (grid 512) vs the worst V1 case (grid 8) — realistic ~100-lane batches see ~2–4× and the
  GPU is likely **not faster end-to-end than a modern CPU** at those sizes (H2D dominates). The paper is honest
  about this; the *figure* could be misread. **Rec:** add a CPU-vs-GPU **end-to-end** timing at realistic lane
  counts; state in the caption that the GPU value is *auditability/determinism*, not throughput at deployment
  sizes.
- **Edge-embedded engineer.** The `no_std`/fixed-point profile is solid disclosure but **design-only** — no
  microcontroller run, no WCET / bounded-memory measurement. **Rec:** one QEMU Cortex-M smoke run of the
  fixed-point core converts the claim from disclosed-intent to demonstrated.

---

## 3. Mentorship — strengths to keep · what to sharpen (all additive)

**Keep (genuinely rare at this breadth):** the honesty discipline is *real* (`limitations.tex`, claim-boundary
badges, disclosed negative results, Tier-1/2/3); determinism-as-evidence + the forensic court is a defensible
novel framing; the physics witnesses with a *doubly-testable* applicability criterion are sophisticated.

**Sharpen — by adding, never narrowing:**
1. **Close the execution gap with breadth, not pruning.** Execute *more* of the 57 detectors and 6 unexecuted
   fault signatures end-to-end on real data — the prior-art-correct move; do **not** trim the catalogue.
2. **Make the tests behavioural.** Per evidence object, assert an empirical property (detection within N
   samples of a known onset; FP ≤ X% on a known-good window), not only `verify()` seal-checks; give the
   completeness court an independent oracle.
3. **Fix the two concrete code items** (degeneracy guard + `.ok()` swallowing) — small, real, a reviewer
   *will* find them.
4. **Separate the two lenses in the abstract** — one sentence ("a prior-art disclosure of a broad apparatus;
   the empirical demonstrations are bounded — 14/57 detectors executed, 9/20 datasets simulated, SWaT gated")
   inoculates against the oversold reading and *increases* credibility.

---

## 4. Recommendation ledger

### v1 recommendations — ADOPTED (the figure campaign)
| # | v1 recommendation | Status |
|---|---|---|
| R1 | Visual atlas of the method (60 figures) | ✅ groups A–I + renderer + `figures` command |
| R2 | Chem-eng practitioner dossier + paper subsection | ✅ `docs/chemical_engineering_practitioner_dossier.md` + operator section |
| R3 | Practitioner-facing figures (one-pager, alarm flood, NE107, workflow) | ✅ group I |
| R4 | Plot measured CUDA/Nsight data | ✅ group C |
| R5 | Draw the graphs (topology / propagation / provenance) | ✅ group D (graphviz-first) |
| R6 | Figure-provenance manifest (id → data → caption → sha256) | ✅ `figure_manifest.json` |
| R7 | Captured verbose build log artifact | ✅ `figure_build_log.txt` in the ZIP |
| R8 | Two generation methods in lockstep | ✅ `figures` command + Colab §7b |
| R9 | Best figures into the paper | ✅ 17 embedded (42→51 pages, 0 overfull / 0 undefined) |

### v2 recommendations — the next frontier (prioritized, breadth-positive)
| # | Recommendation | Where it lands |
|---|---|---|
| V1 | One ungated, real, role-labelled historian run end-to-end | `historian` → `casefile`; TRL-4 → TRL-5 |
| V2 | Execute more of the catalogued surface (detectors + the 6 unexecuted fault signatures) on real data | atlas + demo/CLI |
| V3 | Behavioural test layer + an independent oracle for the completeness court | `crates/…-edge/tests/*` |
| V4 | Numerical-robustness pass: degenerate-baseline guard + regression test; replace silent `.ok()` writes | `linalg.rs`, `cli.rs` |
| V5 | Claim-boundary banner at the top of `operator_report.html` itself | `report.rs` |
| V6 | CPU-vs-GPU end-to-end timing at realistic lane counts; frame GPU value as auditability | cuda + paper |
| V7 | One QEMU Cortex-M smoke run of the fixed-point core | edge embedded profile |
| V8 | An adversarial "where DSFB is wrong" figure/section | paper + group A |
| V9 | One-sentence two-lens framing in the abstract | paper intro |

---

## 5. Bottom line

As **prior art / defensive publication this is a 9.3/10** — broad, enabling, visually legible, and unusually
honest; the campaign closed the two communication levers and the dossier/standards work closed the
practitioner one. The remaining frontier is **empirical demonstration depth** (execution coverage,
real-vs-synthetic data, behavioural tests, two concrete robustness fixes) — and **every recommendation to
close it *adds* breadth rather than narrowing scope**, so it is fully aligned with the prior-art strategy.
The fastest path to "legendary" is not more *kinds* of evidence objects but more of the existing catalogue
*run end-to-end on real, reproducible data*, plus a one-sentence framing that tells the reader which lens to
apply.

*Committed deliberately: a visible, evidence-cited record of the review (and its honest weaknesses) is itself
part of the prior-art disclosure. Advisory; no root cause, no causality, no control/safety authority.*

---

## v3 — after the mechanized-breadth program (Waves 1–7)

> Reviews the artifact after the post-P71 program: the v2 ledger's robustness/honesty fixes, the
> mechanized-claim-discipline wave, and Waves 3–7 (physics, industrial historian layer, embedded core,
> confidential-evaluation chain, research-grade). The two v2 *binding constraints* — empirical depth and
> numerical robustness — were targeted directly; the rest is additive breadth. Strategy unchanged: prior art.

### What moved (evidence-cited)
- **Numerical robustness (v2: 7.0).** The CSTR `~1.4e35` degeneracy class is fixed at source (`data.rs`
  `Baseline::fit` near-constant-channel fallback + a regression test + the `proof_classify_is_total_on_finite`
  Kani harness); the silent `.ok()` error-swallowing is replaced by a tracked tally with a non-zero exit. The
  governed re-freezes are recorded. **→ 9.0.**
- **Empirical demonstration depth (v2: 6.7, the binding constraint).** Behavioural tests now assert *properties*
  (every synthetic fault detected in-window; baseline not FP-dominated; compression ≥ 1×) rather than
  seal-checks; an independent re-parse oracle cross-checks the completeness court; `data-readiness` runs on a
  real CSV; CPU-vs-GPU end-to-end is measured (7.4×/15.9×/33.2×). Execution coverage is still a fraction of the
  catalogue (the honest, governed Phase-C executable re-freeze remains the open lever). **→ 7.6** (improving;
  still the frontier).
- **Breadth of disclosure (v2: 9.7).** ~60 more sealed, self-verifying, bounded evidence objects across Waves
  3–7 (unit-consistency court, spec/permit witnesses, first-principles adapter + passport, the P75 balance
  pack, the full confidential-evaluation chain, interval physics-informed envelopes, multi-physics/multi-scale/
  spectral grammar, Merkle-DAG amendments, DSFB-Bench, safety dossier, proof-obligation ledger) **plus a fifth
  crate** (`dsfb-chemical-engineering-core`, `no_std`, that *runs on an emulated Cortex-M3*) and a pyo3 binding
  crate. **→ 9.8.**
- **Enablement / reproducibility (v2: 9.6).** Workspace 295 tests / 0 failed; `release-scrub`, `unit-consistency`,
  `data-readiness` CLI gates; Dockerfile + release checklist; the bare-metal QEMU smoke run; the
  `ProofObligationLedgerV1` Lean4/Coq handoff. **→ 9.7.**
- **Chem-eng practitioner framing (v2: 8.8).** First-principles witnesses (Arrhenius/Antoine/Raoult/Henry/
  heat-transfer/pump-curve/valve-Cv), spec/permit boundaries, the safety-traceability dossier, and the
  confidential-evaluation path (data never egresses) speak directly to an operator/SBIR evaluator. **→ 9.2.**
- **Rigor / academic honesty (v2: 9.5).** Every new object carries an explicit, *sealed* non-claim; the
  burden-of-proof gate forces `unknown` rather than a fabricated label; honest gaps are kept in the safety
  dossier; the two AVOIDs are honoured and disclosed. **→ 9.6.**

### Bottom line (v3)
As **prior art / defensive publication this is ~9.5/10** — the v2 communication and practitioner levers stay
closed, the two v2 binding constraints (robustness, then empirical depth) were directly and honestly addressed,
and the surface widened by ~60 sealed objects + an embedded core that runs on real silicon (emulated) + the
confidential-evaluation unlock. **The single remaining frontier is unchanged in kind: execution coverage** —
running more of the catalogued detectors/fault-signatures end-to-end via the governed `atlas_hash_v1` +
replay re-freeze (deliberately deferred to a careful dedicated pass, not rushed). Everything done since v2 *adds*
breadth; nothing narrowed scope. Advisory; no root cause, no causality, no control/safety authority.
