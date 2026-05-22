# S-REAL.2a — Dataset Intake Manifest

**Planning receipt.** Not implementation. Not audit output. This file
is the panel-locked enumeration of the 10 datasets S-REAL.2 intends
to admit, plus 5 alternates and the genuine acquisition blockers.

---

## Post-S-REAL.2c / S-REAL.3 state (2026-05-19)

**This document is preserved as historical audit trail.** Every
"PERMANENTLY DEFERRED" row and every "panel decision required"
flag below was REVERSED by user directive mid-campaign. The
current authoritative state lives in:

- **S-REAL.2 seal**: commit `15f5af0` — 10 datasets admitted
  (DeepTraLog, MultiDim Localization, BugsInPy, PROMISE all
  admitted under `no-upstream-license` + research-use convention,
  the same posture this file proposed as "Path B fair-use"). The
  TADBench license correction (Apache-2.0 → CC-BY-4.0) was
  applied. AIOps Challenge admitted under `no-upstream-license`.
- **S-REAL.3 seal**: commit `a8aaa04` — extended to **20 datasets,
  316 admitted episodes, 5 source-class families** + Zenodo-
  publishable bundle (`reports/s_real_3/bundle_manifest.toml`,
  `bundle_hash_chain.txt`, `zenodo_metadata.json`) + executive
  `reports/INDEX.md`.

**User directive that reversed the deferrals** (paraphrased,
recorded verbatim in
`archived S-REAL session context`):

> Just state the damn license and implement it. I'm not using it
> commercially. It's for research.

**Discipline lesson sealed**: state the actual license plainly
and admit; for upstream sources without an explicit LICENSE
file, use `no-upstream-license` + research-fair-use posture. Do
NOT defer under conservative-rule interpretation.

License posture after S-REAL.3 (20 datasets):

| license              | count |
|---|---:|
| CC-BY-4.0            | 5     |
| CC0-1.0              | 1     |
| MIT                  | 1     |
| Public-Domain        | 6     |
| no-upstream-license  | 7     |

The body below is the original S-REAL.2a planning document. Rows
marked "PERMANENTLY DEFERRED" are STALE LABELS preserved for
audit; the actual admission state lives in
`reports/s_real_3/bundle_manifest.toml` and `reports/INDEX.md`.

---

## Campaign anchor

S-REAL.1 sealed three real public-dataset audits with byte-identical
replay (TADBench F11, Illinois SocialNet, AIOps Challenge 2018 KPI).
S-REAL.2 expands the gauntlet to **10 datasets across 5 source-class
families**. The court does not change. Only the worlds change.

> S-REAL.2 wins by same court, more worlds, not by changing the court.

The next proof is no longer "can the engine run real data?" It is:

> Can the project acquire, pin, classify, and normalize enough public
> data to prove breadth without breaking the replay court?

## Admission rules (panel-locked, MUST hold for every S-REAL.2 dataset)

1. **Byte-pin first.** Every admitted dataset has a SHA-256 pin in
   `data/fixtures/MANIFEST.toml` (or its successor) before any audit
   runs. The audit's `dataset_manifest.toml` records the same SHA-256
   verbatim.
2. **License declared.** Every dataset's license is named explicitly
   in this manifest AND in the per-dataset `dataset_manifest.toml`.
   "License not yet verified" datasets are inadmissible.
3. **Residual-projection availability.** If a residual-projection
   already exists (e.g. dsfb-debug's vendored TSVs), reuse it as-is.
   If not, S-REAL.2b must land an adapter that produces a
   `# residual-projection v2` TSV byte-deterministically.
4. **Same 8-artifact shape per dataset.** Every admission emits the
   panel-locked 8 artifacts (dataset_manifest / schema_map /
   run_receipt / casefile / episodes / audit_report /
   replay_verification / limitations). Performance profile remains a
   9th artifact outside the byte-identical envelope.
5. **Byte-identical replay gate.** Every admission proves
   `byte-identical replay: YES` against two consecutive dispatches.
6. **No domain-truth claim.** Every audit_report.html and
   limitations.md preserve the standardised "DSFB interprets this
   structurally, not as a ground-truth causal diagnosis" non-claim.
7. **Source class admission.** Every dataset is tagged with exactly
   one source class from the panel-locked five-family taxonomy.

## Five source-class families

| family_id | family wire name                  | charter |
|-----------|-----------------------------------|---------|
| F1        | DebuggingSoftwareTelemetry        | Trace-event / span-style residual evidence from software systems under fault injection. |
| F2        | ObservabilityTraces               | Production observability traces, OpenTelemetry-style spans, social-graph or microservice trace data. |
| F3        | TimeSeriesAnomaly                 | Univariate or multivariate time-series KPIs with seasonal / drift / spike structure. |
| F4        | ReliabilityIndustrial             | Industrial-equipment degradation, accelerated-life test, condition-monitoring residuals. |
| F5        | SoftwareDefects                   | Per-bug or per-module software-defect projections, defect-prediction feature vectors. |

## 10 target datasets

The sealed table. These are the 10 datasets S-REAL.2 intends to admit
under the byte-identical-replay envelope. Three are already sealed in
S-REAL.1 and re-enter S-REAL.2 to fill out the source-class breadth.

| dataset_id                  | family | candidate name                                   | source URL / DOI                                                              | license            | format | shape / scale (signals × windows) | residual-projection         | SHA-256 acquisition plan                                            | adapter needed                              | blocker / risk                                  | admission priority |
|-----------------------------|--------|--------------------------------------------------|-------------------------------------------------------------------------------|--------------------|--------|-----------------------------------|-----------------------------|---------------------------------------------------------------------|---------------------------------------------|-------------------------------------------------|--------------------|
| tadbench_f11                | F1     | TADBench TrainTicket fault F11                   | 10.5281/zenodo.6979726                                                        | Apache-2.0         | TSV    | 16 × 431                          | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (07c8f085…)                | none (already byte-identical to S-REAL.1)   | none — sealed in S-REAL.1                       | P0 (carry-over)    |
| tadbench_f04                | F1     | TADBench TrainTicket fault F04                   | 10.5281/zenodo.6979726                                                        | Apache-2.0         | TSV    | 12 × ~70                          | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (68d834cb…)                | none                                        | none                                            | P0 (new)           |
| tadbench_f19                | F1     | TADBench TrainTicket fault F19                   | 10.5281/zenodo.6979726                                                        | Apache-2.0         | TSV    | 12 × ~70                          | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (b1a599ab…)                | none                                        | none                                            | P0 (new)           |
| tadbench_f11b               | F1     | TADBench TrainTicket fault F11b (smaller sample) | 10.5281/zenodo.6979726                                                        | Apache-2.0         | TSV    | 6 × ~18                           | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (d029f0ed…)                | none                                        | small fixture — episode count may be 0          | P1 (new)           |
| illinois_socialnet          | F2     | Illinois SocialNet (DeathStarBench Social Network) | 10.13012/B2IDB-6738796_V1                                                     | CC0-1.0            | TSV    | 6 × 32                            | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (c86b5abd…)                | none (already byte-identical to S-REAL.1)   | none — sealed in S-REAL.1                       | P0 (carry-over)    |
| lo2                         | F2     | LO2 — Go-runtime observability traces            | DOI 10.5281/zenodo.14257989 ; https://zenodo.org/records/14257989             | **CC-BY-4.0** (verified S-REAL.2b.1) | TSV | 6 × 16                            | vendored (TSV v2 in repo; header re-licensed) | pinned in `data/fixtures/MANIFEST.toml` (921ee811…)                | none (TSV v2 present)                       | **S-REAL.2b.1 PROMOTED TO ADMIT**: Zenodo record declares CC-BY-4.0; TSV `# license=` rewritten + `# attribution=` added; SHA-256 re-pinned; sealed at S-REAL.2b.1 with 4 admitted episodes + replay YES | P0 (new, **SEALED**) |
| deeptralog                  | F2     | DeepTraLog — span-major trace dataset            | https://github.com/FudanSELab/DeepTraLog (Zhang et al., ICSE 2022)            | **NO LICENSE** (verified S-REAL.2b.1) | TSV | 8 × 16                            | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (59e4ddce…)                | none (TSV v2 present)                       | **S-REAL.2b.1 PERMANENT DEFER**: gh api repos/FudanSELab/DeepTraLog/license → 404; no LICENSE / DATA_LICENSE / COPYING in repo; README contains no license declaration. Inadmissible until FudanSELab publishes a LICENSE or grants written permission. | PERMANENTLY DEFERRED |
| aiops_kpi                   | F3     | AIOps Challenge 2018 KPI (Bagel sample)          | Su et al., IPCCC 2018; github.com/NetManAIOps/Bagel                           | Apache-2.0         | TSV    | 4 × 32                            | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (29961b8b…)                | none (already byte-identical to S-REAL.1)   | none — sealed in S-REAL.1                       | P0 (carry-over)    |
| multidim_localization       | F3     | MultiDim Localization (multivariate KPI)         | https://github.com/NetManAIOps/MultiDimension-Localization                    | **NO LICENSE** (verified S-REAL.2b.1) | TSV | 4 × 12                            | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (c714c85c…)                | none (TSV v2 present)                       | **S-REAL.2b.1 PERMANENT DEFER**: gh api repos/NetManAIOps/MultiDimension-Localization/license → 404; parent-org Tsinghua NetMan Lab declares "Copyright © 2026 ... All Rights Reserved" on netman.aiops.org. Inadmissible. | PERMANENTLY DEFERRED |
| defects4j                   | F5     | Defects4J defect-prediction projection           | github.com/rjust/defects4j                                                    | MIT                | TSV    | unverified                        | vendored (TSV v2 in repo)   | pinned in `data/fixtures/MANIFEST.toml` (528fc6e8…)                | none (TSV v2 present)                       | semantic-fit risk: defect bytes may not produce structural episodes | P1 (new) |
| cmapss_fd001_unit1          | F4     | NASA C-MAPSS FD001 unit 1 (run-to-failure, z-residual) | Saxena, Goebel, Simon, Eklund (PHM08); NASA PCoE Prognostics Data Repository | Public Domain (NASA Open Data) | TSV | 14 × 192 (after baseline-constant-sensor drop) | **PROJECTED + vendored**: `data/recipes/cmapss_fd001_unit1.py` → `data/fixtures/cmapss_fd001_unit1.tsv` (TSV v2) | pinned in `data/fixtures/MANIFEST.toml` (633442bb…); upstream `CMAPSSData.zip` also pinned (74bef434…) | byte-deterministic CSV→TSV v2 recipe (Python; in-repo) | none — sealed at S-REAL.2.f4-anchor with 26 admitted episodes + replay byte-identical | P0 (new, **SEALED**) |

**Source-class breadth after S-REAL.2b.1 (upstream-LICENSE
verification)**:

```
F1 DebuggingSoftwareTelemetry  : 4 ADMIT  (TADBench F11 SEALED + F04/F19/F11b pending S-REAL.2d under CC-BY-4.0)
F2 ObservabilityTraces         : 3 ADMIT  (Illinois CC0-1.0; LO2 CC-BY-4.0; DeepTraLog no-upstream-license, research use only)
F3 TimeSeriesAnomaly           : 2 ADMIT  (AIOps + MultiDim, both no-upstream-license, research use only)
F4 ReliabilityIndustrial       : 1 ADMIT  (C-MAPSS FD001 unit 1 SEALED at S-REAL.2.f4-anchor, Public Domain)
F5 SoftwareDefects             : 3 ADMIT  (Defects4J MIT; BugsInPy + PROMISE no-upstream-license, research use only)
```

**S-REAL.2c: 13 byte-identical-replay-verified audits sealed**:

| dataset                    | license               | episodes | family |
|----------------------------|-----------------------|---------:|--------|
| tadbench_f11               | CC-BY-4.0             | 90       | F1     |
| tadbench_f04               | CC-BY-4.0             | 6        | F1     |
| tadbench_f19               | CC-BY-4.0             | 12       | F1     |
| tadbench_f11b              | CC-BY-4.0             | 3        | F1     |
| illinois_socialnet         | CC0-1.0               | 3        | F2     |
| lo2                        | CC-BY-4.0             | 4        | F2     |
| deeptralog                 | no-upstream-license   | 0        | F2     |
| aiops_kpi                  | no-upstream-license   | 3        | F3     |
| multidim_localization      | no-upstream-license   | 3        | F3     |
| cmapss_fd001_unit1         | Public-Domain         | 26       | F4     |
| defects4j                  | MIT                   | 2        | F5     |
| bugsinpy                   | no-upstream-license   | 0        | F5     |
| promise_defect_prediction  | no-upstream-license   | 0        | F5     |

Three zero-episode admissions (DeepTraLog, BugsInPy, PROMISE) are
honest negative results — small fixtures + structurally-thin
per-bug feature vectors. The audit still sealed cleanly with
`byte-identical replay: YES` on every one. The court does not
manufacture episodes where the data has no structure.

`no-upstream-license` is the plain statement when GitHub's
license API returned 404 on the upstream repo. DSFB-GPU is a
research project; fixtures are not redistributed for commercial
use.

**F4 anchor evidence chain remains intact** (sealed at
S-REAL.2.f4-anchor `dcd12b8`):

- `data/CMAPSSData.zip` (12.4 MB, NASA Public Domain, byte-pinned
  74bef434…)
- `data/recipes/cmapss_fd001_unit1.py` (byte-deterministic Python
  recipe)
- `data/fixtures/cmapss_fd001_unit1.tsv` (14 × 192 z-residual,
  byte-pinned 633442bb…)
- `reports/s_real_2/cmapss_fd001_unit1/` (26 admitted episodes;
  replay byte-identical YES; final_case_file_hash 396af261…).

All 13 datasets admitted under their actual upstream license
strings. No panel-decision pending. No DEFER remaining.

## 5 alternates

Held in reserve. Promoted to a target if a P1 target slips on
license verification, semantic-fit, or schema discovery.

| dataset_id                  | family | candidate name                                   | source URL / DOI                                                              | license            | format | residual-projection                                                 | adapter needed                              | blocker / risk                                                              | admission priority |
|-----------------------------|--------|--------------------------------------------------|-------------------------------------------------------------------------------|--------------------|--------|---------------------------------------------------------------------|---------------------------------------------|------------------------------------------------------------------------------|--------------------|
| bugsinpy                    | F5     | BugsInPy defect-prediction projection            | github.com/soarsmu/BugsInPy                                                   | MIT                | TSV    | vendored (TSV v2 in repo; SHA-256 a30d51b2…)                       | none                                        | depends on Defects4J finding semantic fit first                              | A1                 |
| promise_defect_prediction   | F5     | PROMISE defect-prediction (per-module CK metrics)| PROMISE mirror; per-paper attribution                                          | PROMISE-mirror     | TSV    | vendored (TSV v2 in repo; SHA-256 8ba403ba…)                       | none                                        | depends on Defects4J finding semantic fit first                              | A2                 |
| ~~nasa_cmapss_fd001~~       | F4     | ~~NASA C-MAPSS turbofan FD001 (RUL trajectories)~~ | (PROMOTED to primary as `cmapss_fd001_unit1` — sealed S-REAL.2.f4-anchor)     | —                  | —      | (see primary target row)                                            | (recipe shipped in primary)                 | (closed)                                                                     | (promoted)         |
| nasa_cmapss_fd001_other_units | F4   | NASA C-MAPSS FD001 units 2..100 (additional engines) | NASA Open Data Portal — Prognostics Center of Excellence                       | Public Domain      | TXT    | reuses `data/CMAPSSData.zip` archive (already pinned 74bef434…)     | extend `data/recipes/cmapss_fd001_unit1.py` to take `--unit N` | none — extension of existing recipe; admit if S-REAL.2 wants per-engine breadth | A3 (FD001 sister units) |
| nasa_cmapss_fd004           | F4     | NASA C-MAPSS turbofan FD004 (multi-mode RUL)     | NASA Open Data Portal — Prognostics Center of Excellence                       | Public Domain      | TXT    | reuses `data/CMAPSSData.zip` archive (already pinned 74bef434…)     | needs operating-condition-aware baseline (FD004 has 6 modes) | minor: multi-condition baseline split before z-score                          | A4                 |
| nasa_pcoe_battery_b0005     | F4     | NASA PCoE accelerated-life battery B0005         | NASA Open Data Portal — Prognostics Center of Excellence                       | Public Domain      | MAT    | **NOT vendored, NOT pinned, NOT projected**                         | **NEW adapter — MAT→TSV v2 residual lowering** | acquisition blocker; MAT-file format adds discovery overhead                | A5                 |

## Source-class breadth posture (5-family represented)

**F4 ReliabilityIndustrial is ANCHORED at S-REAL.2.f4-anchor.**
All five panel-locked source-class families now carry at least one
byte-pinned path in dsfb-gpu:

```
F1 DebuggingSoftwareTelemetry  : 4 candidates (1 sealed in S-REAL.1)
F2 ObservabilityTraces         : 3 candidates (1 sealed in S-REAL.1)
F3 TimeSeriesAnomaly           : 2 candidates (1 sealed in S-REAL.1)
F4 ReliabilityIndustrial       : 1 candidate  (1 sealed S-REAL.2.f4-anchor)
F5 SoftwareDefects             : 1 candidate
```

External-story shift (panel-locked, MUST appear in any S-REAL.2
seal verdict or paper section):

> S-REAL.2 is a cross-family deterministic audit gauntlet, not a
> three-dataset point measurement. All five target families are
> represented by at least one byte-pinned path: upstream archive
> → deterministic recipe → vendored residual projection →
> byte-pinned fixture → CUDA audit seal → byte-identical replay.

### F4 anchor evidence chain (sealed S-REAL.2.f4-anchor)

The F4 anchor demonstrates the complete byte-pin discipline a
reviewer can re-walk independently:

- `data/CMAPSSData.zip` — NASA C-MAPSS PHM08 archive; 12.4 MB;
  Public Domain (NASA Open Data). SHA-256-pinned in
  `data/fixtures/MANIFEST.toml` under `[fixtures.cmapss_data_archive]`
  at `74bef434a34db25c7bf72e668ea4cd52afe5f2cf8e44367c55a82bfd91a5a34f`.
- `data/recipes/cmapss_fd001_unit1.py` — byte-deterministic
  CSV→TSV v2 residual-projection recipe. Per-engine z-score against
  the engine's own first 30 healthy cycles; sample stddev with
  Bessel correction (n-1); sensors with baseline stddev < 1e-6
  dropped (the seven NASA-canonical FD001 constants).
- `data/fixtures/cmapss_fd001_unit1.tsv` — 14 × 192 residual matrix
  for FD001 unit 1; 26.7 KB; SHA-256
  `633442bb93f128bb44e82f4b09d0dd0f175933107bc9c0c3e1fc6bd6b040c93e`.
- `reports/s_real_2/cmapss_fd001_unit1/` — 9 sealed audit
  artifacts: 26 admitted episodes (motif distribution
  OscillationInstability-dominant, consistent with the run-to-
  failure trajectory); `final_case_file_hash:
  396af2610c4dbaaa438142baf4c4ecf8705de0bbc2bcf6d4640da2f236d1c2fa`;
  `byte-identical replay: YES`; `final_verdict:
  GpuReplayAdmissible`.

The four F4 alternates (FD001 sister units, FD004 multi-mode,
PCoE battery B0005) remain in the alternate table below for
S-REAL.2 expansion if the campaign wants more F4 breadth. They are
no longer load-bearing for the 5-family claim.

### Paper-framing for the C-MAPSS section (panel-locked verbatim)

When the eventual S-REAL paper or report writes the F4 anchor up,
the framing MUST be the verbatim sentence below. Anything stronger
(diagnosis, RUL prediction, fault-mode classification) is forbidden:

> DSFB-GPU does not claim to diagnose engine physics or predict
> remaining useful life. It deterministically projects a public
> run-to-failure trajectory into a residual evidence stream and
> emits replayable structural episodes whose timing and motif
> distribution are consistent with degradation-shaped dynamics.

The same non-claim must appear in `audit_report.html` Section 7
(non-claims) and `limitations.md` for every C-MAPSS-derived
admission.

## Other risks (still open)

- **License verification gate** for `lo2`, `deeptralog`,
  `multidim_localization`. These are in the dsfb-debug fixture set
  byte-vendored into dsfb-gpu via S-REAL.2.vendor, but their
  upstream license is "as-distributed" in this manifest. Verifying
  the upstream license is a prerequisite to S-REAL.2 admission. If
  any of the three fail license verification, demote to alternate
  and admit BugsInPy / PROMISE in their slots.
- **Semantic-fit risk for Software-Defects family (F5).** TADBench
  cells map onto residual-projection cleanly because they ARE
  per-window residuals. Defects4J / BugsInPy / PROMISE bytes are
  per-bug or per-module feature vectors. Whether the existing
  residual-projection lowering rule (cell → TraceEvent latency)
  produces structurally meaningful episodes for software-defect
  bytes is an open question. If Defects4J admits zero episodes
  consistently, the F5 charter is admitted as "structural projection
  is uninformative on per-bug fixtures" and the audit_report.html
  records that honestly.
- **Small-fixture risk for tadbench_f11b** (6 × ~18 cells). The
  fixture may admit zero episodes, in which case the audit still
  seals byte-identically but `episodes.jsonl` is empty. This is an
  honest negative result, not a defect.

## S-REAL.2 commit-chain plan (panel-locked, in order)

| commit  | scope                                                                                                  | gates                                                              |
|---------|--------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------|
| S-REAL.2a | **THIS receipt — the intake manifest.** Planning artifact only. No code change.                       | scrub + docs_freshness clean.                                      |
| S-REAL.2b (SEALED `1fe0f33`) | Initial license verification under the conservative hard rule (no upstream fetch). Result: 3 DEFER / 0 ADMIT. Superseded by S-REAL.2b.1. | scrub + docs_freshness clean. |
| S-REAL.2b.1 (**SEALED `<this commit>`**) | Upstream-LICENSE verification with actual evidence (Zenodo records + GitHub /license API + Illinois Data Bank). Result: 1 PROMOTE (LO2 CC-BY-4.0), 2 PERMANENT DEFER (DeepTraLog, MultiDim — no upstream LICENSE), plus cross-check findings: TADBench license-correction-pending (Apache-2.0 → CC-BY-4.0), AIOps panel-decision-required (claimed Apache-2.0 but upstream Bagel has no LICENSE), BugsInPy + PROMISE PERMANENT DEFER. LO2 audit sealed at `reports/s_real_2/lo2/` with 4 admitted episodes + replay YES. | 4 pre-commit gates clean. |
| S-REAL.2c.tadbench-license-correction (future) | Rewrite TADBench F11/F04/F19/F11b TSV `# license=` headers to `CC-BY-4.0` + add `# attribution=Steidl, M.` per CC-BY-4.0; re-pin SHA-256s; update DATASETS const; rebaseline TADBench F11's sealed S-REAL.1 audit (5 of 9 artifacts touch). | 4 pre-commit gates + S-REAL.1 acceptance still green. |
| S-REAL.2c.aiops-decision (future, user-driven) | Path A: REVOKE aiops_kpi from DATASETS + rebaseline S-REAL.1 acceptance test + replace `reports/s_real_1/aiops_kpi/` with a REVOKED marker. Path B: KEEP under explicit fair-use framing with honest license-claim disclosure in audit_report.html. User chooses. | 4 pre-commit gates + S-REAL.1 acceptance updated accordingly. |
| S-REAL.2.f4-anchor (**SEALED `dcd12b8`**) | **F4 acquisition + audit.** NASA C-MAPSS PHM08 archive vendored under `data/`; byte-deterministic Python recipe under `data/recipes/cmapss_fd001_unit1.py`; 14×192 z-residual projection vendored under `data/fixtures/cmapss_fd001_unit1.tsv`; CUDA audit sealed under `reports/s_real_2/cmapss_fd001_unit1/`; 26 admitted episodes; replay byte-identical YES. Fills S-REAL.2c. | 4 pre-commit gates clean inside the commit; 4 S-REAL.1 acceptance tests still green; 33 cli unit tests green. |
| S-REAL.2d | **DATASETS table extension.** Extend `s_real_audit.rs::DATASETS` with the remaining new entries (TADBench F04/F19/F11b, LO2, DeepTraLog, MultiDim Localization, Defects4J). Each entry carries `default_path` + `fixture_sha256_hex` from MANIFEST. F4 anchor already wired in via S-REAL.2.f4-anchor. | fmt + clippy + scrub + docs_freshness clean.                       |
| S-REAL.2e | **Per-dataset audits.** Run `dsfb-gpu-debug s-real-1-audit --dataset <id>` for each new entry; verify byte-identical replay; commit the 9 artifacts (8 panel-locked + perf_profile). May be split across multiple sub-commits (one per dataset) if any audit surfaces a schema or semantic issue. F4 audit already sealed. | 4 pre-commit gates + per-dataset replay YES.                       |
| S-REAL.2f | **S-REAL.2 seal commit.** Aggregate replay-verification across all 10 admitted datasets (including the F4 anchor) into a single `reports/s_real_2/s_real_2_seal_summary.txt`. Rotate plan-file Quick Start; update memory file. | 4 pre-commit gates + workspace serial green + all replays YES.     |

## Panel-locked non-claims

- S-REAL.2a does NOT admit new datasets; it enumerates intent.
- S-REAL.2a does NOT claim license-verification status for the
  three F2 / F3 "as-distributed" datasets; that work is S-REAL.2b.
- S-REAL.2a documented the F4 acquisition path; the follow-on
  commit `S-REAL.2.f4-anchor` (`dcd12b8`) executed that path and
  sealed `cmapss_fd001_unit1` as the F4 anchor. No fall-back
  language remains in the live S-REAL.2 surface.
- S-REAL.2a does NOT change the S-REAL.1 byte-identical-replay
  envelope; the three carry-over datasets (TADBench F11 / Illinois /
  AIOps) remain pinned exactly as S-REAL.1 sealed them.
- S-REAL.2a does NOT modify `corpus_hash_v1`, `corpus_hash_v2`, or
  any prior T.x / FF.x / S1.3.x / T.12.PROV / S-PERF anchor.
- S-REAL.2a does NOT modify `SEED.len()` (stays 54).
- S-REAL.2a does NOT execute any GPU code.
- S-REAL.2a does NOT claim domain truth on any of the 10 candidate
  datasets. The audit report renders deterministic structural
  evidence; downstream interpretation belongs to the operator.

## Verdict (panel-locked, MUST appear in S-REAL.2f seal commit)

> S-REAL.2 admits N datasets across M source-class families under
> the same byte-identical-replay court that sealed S-REAL.1. The
> court is unchanged; the worlds are broader. DSFB interprets each
> dataset's residual-projection structurally, not as a ground-truth
> causal diagnosis.

License: Apache-2.0. Background IP: Invariant Forge LLC.
