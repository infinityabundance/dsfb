# S-REAL.2b — License Verification Report (with upstream evidence)

---

## Post-S-REAL.2c reversal block (2026-05-19)

**This document is historical audit trail.** Every PERMANENT DEFER
decision below was REVERSED by user directive mid-campaign. The
Path A / Path B fork on `aiops_kpi` was resolved (no decision needed —
admitted under `no-upstream-license`). The current authoritative
admission state lives in `reports/s_real_3/bundle_manifest.toml` +
`reports/INDEX.md`.

### What was reversed

| dataset                    | this-doc verdict   | actual state (post-S-REAL.2c) |
|---|---|---|
| `deeptralog`               | PERMANENT DEFER    | **ADMITTED** under `no-upstream-license` + research-fair-use; 3 episodes sealed at S-REAL.2 |
| `multidim_localization`    | PERMANENT DEFER    | **ADMITTED** under `no-upstream-license` + research-fair-use; 3 episodes sealed at S-REAL.2 |
| `bugsinpy`                 | PERMANENT DEFER    | **ADMITTED** under `no-upstream-license`; 1 episode sealed at S-REAL.2 (rich re-projection from upstream bugs catalog) |
| `promise_defect_prediction`| PERMANENT DEFER    | **ADMITTED** under `no-upstream-license`; 5 episodes sealed at S-REAL.2; sister `promise_ant_1_4` 11 episodes sealed at S-REAL.3 |
| `aiops_kpi`                | Path A / Path B fork (panel decision required) | **ADMITTED** under `no-upstream-license`; 3 episodes sealed at S-REAL.1 (no path-fork resolution needed) |
| `tadbench_*`               | license-correction-pending (Apache-2.0 → CC-BY-4.0) | **APPLIED** — all 4 TADBench fixtures now record `CC-BY-4.0`; consistent with Zenodo record `10.5281/zenodo.6979726` |

### User directive that drove the reversal (verbatim)

> Just state the damn license and implement it. You're getting
> caught up on having all the licenses the same. Of course they
> will be different. Just say what they are. I'm not using it
> commercially. It's for research.

### Discipline lesson sealed

State the actual license plainly and admit. For upstream sources
without an explicit LICENSE file, use `no-upstream-license` +
research-fair-use posture. Do NOT defer under conservative-rule
interpretation. The hard rule "ambiguous license = DEFER" stated
below was REVOKED for research artifacts; commercial redistribution
remains a separate posture (S-REAL.4, deferred).

### Where the current authoritative state lives

- **`reports/s_real_3/bundle_manifest.toml`** — per-dataset license
  declaration (5 CC-BY-4.0 + 1 CC0-1.0 + 1 MIT + 6 Public-Domain +
  7 no-upstream-license) across the 20-dataset audit gauntlet.
- **`reports/INDEX.md`** — executive-reader summary of the same
  20-dataset license posture.
- **`data/fixtures/MANIFEST.toml`** — per-fixture `license = "..."`
  string + SHA-256 byte-pin.

The body below is the original S-REAL.2b verification analysis,
preserved as historical audit trail. Every "PERMANENT DEFER" row
is a STALE LABEL. Read this file as a record of why we INITIALLY
chose to defer, not as the current state.

---

## Original report (preserved as audit trail)

**Verification receipt.** Not implementation. Not audit output.
This file records the per-dataset license-verification decision
for every S-REAL fixture currently vendored under `data/fixtures/`
and every alternate enumerated in the S-REAL.2a intake manifest.

The S-REAL.2 hard rule was panel-locked at the time of this report:

> **ambiguous license = DEFER**

**This hard rule was reversed by the user directive shown above.**
The current admission posture is "state the actual license plainly
and admit; no-upstream-license = research-fair-use, NOT
inadmissible." The original verification analysis below remains
useful as evidence-of-due-diligence (we DID check Zenodo, we DID
check GitHub /license API, we DID check parent-org policy); the
ADMIT/DEFER decisions it produced no longer apply.

The first version of this report (commit `1fe0f33`) applied the
hard rule conservatively — three DEFERs without inspecting the
upstream sources. The user verdict on that report:

> stop taking the easy route. do the work.

This rewrite **does the work**: every fixture's claimed license
is verified against the actual upstream source (Zenodo record
license, GitHub LICENSE-file presence, parent-organization
licensing policy). The findings change the admission status of
six datasets and surface two cross-check issues that were not
caught at S-REAL.1 / S-REAL.2.f4-anchor seal time.

## Verification method

1. **Zenodo records**: read the canonical license/rights field via
   `https://zenodo.org/records/<id>`.
2. **GitHub repositories**: query `gh api repos/<owner>/<repo>/license`
   (GitHub's license detector reads LICENSE / LICENSE.md / COPYING).
   404 = no recognized LICENSE file → fall back to inspecting README +
   parent-org licensing policy.
3. **Illinois Data Bank**: read the rights field from the dataset
   record page.
4. **NASA Open Data**: well-established Public Domain release
   policy for Prognostics Center of Excellence datasets.

## Per-dataset findings (10 evaluated)

### S-REAL.1 sealed datasets

#### tadbench_f11 — **license-correction required** (admissible)

- **Upstream**: Zenodo DOI 10.5281/zenodo.6979726, "Anomalies in
  Microservice Architecture (train-ticket) based on version
  configurations", Steidl, M. (University of Innsbruck), 2022.
- **Verified license**: **CC-BY-4.0** (Creative Commons
  Attribution 4.0 International).
- **TSV header claim**: `as-distributed-by-zenodo-6979726`
  (placeholder).
- **DATASETS const claim (s_real_audit.rs)**: `Apache-2.0`.
- **Status**: MISCLAIM. Apache-2.0 is the wrong canonical
  short-form; the correct short-form is CC-BY-4.0. Both are
  open-source-friendly licenses — admissibility is not in
  question, but the public claim must be corrected.
- **Decision**: **ADMIT** (data is admissible). Follow-on
  commit S-REAL.2c.tadbench-license-correction rewrites
  TADBench F11/F04/F19/F11b TSV headers and the DATASETS const
  to declare CC-BY-4.0 with attribution per the CC-BY-4.0
  requirements. The rebaseline of TADBench F11's sealed
  S-REAL.1 audit (changing the TSV bytes changes its SHA-256 →
  changes 5 of 9 sealed artifacts) is a known consequence
  documented in that follow-on commit.

#### illinois_socialnet — **verified correct**

- **Upstream**: Illinois Data Bank DOI 10.13012/B2IDB-6738796_V1.
- **Verified license**: **CC0-1.0** (Public Domain Dedication).
- **TSV header claim**: matches.
- **DATASETS const claim**: matches.
- **Status**: correct. No action required.
- **Decision**: **ADMIT** (no change).

#### aiops_kpi — **PANEL DECISION REQUIRED** (license claim is incorrect)

- **Upstream**: GitHub repo `NetManAIOps/Bagel`
  (Li, Chen, Pei, IPCCC 2018). The fixture is extracted from
  `Bagel/sample_data.csv`.
- **Verified license**: **NO LICENSE FILE in upstream**. GitHub's
  license detector returns 404 on `repos/NetManAIOps/Bagel/license`.
  The Bagel README contains a citation request but no license
  declaration. The parent Tsinghua NetMan Lab website
  (`netman.aiops.org`) carries the footer "Copyright © 2026
  Tsinghua NetMan Lab. All Rights Reserved" with no separate
  data-license policy.
- **TSV header claim**: `Apache-2.0 (Bagel repo)`.
- **DATASETS const claim**: `Apache-2.0`.
- **Status**: MISCLAIM. The claim "Apache-2.0 (Bagel repo)"
  is incorrect — Bagel has no LICENSE file. Apache-2.0 is the
  license of OTHER NetManAIOps repos (some of which carry it),
  not this one.
- **Decision**: **PANEL DECISION REQUIRED**. Two paths:
  - **Path A (strict hard-rule application)**: REVOKE the
    `aiops_kpi` admission. Remove from DATASETS. Update the
    sealed S-REAL.1 audit at `reports/s_real_1/aiops_kpi/`
    with a revocation marker; rebaseline the S-REAL.1
    acceptance test from 3 × 8 = 24 artifacts to 2 × 8 = 16
    artifacts.
  - **Path B (fair-use academic redistribution)**: keep the
    admission with an explicit "fair-use academic
    redistribution; original dataset terms unclear; cite Li et
    al. IPCCC 2018" framing. The AIOps Challenge 2018 data
    has been in de-facto public academic circulation for 7
    years; multiple research papers reuse it. But "de-facto
    public" is not the same as "canonical open license", which
    is what the S-REAL.2 hard rule requires.
  - This commit does NOT execute either path. The user
    chooses; a follow-on commit S-REAL.2c.aiops-decision
    enacts the chosen path.

### S-REAL.2.f4-anchor sealed dataset

#### cmapss_fd001_unit1 — **verified correct**

- **Upstream**: NASA PCoE Prognostics Data Repository (Saxena,
  Goebel, Simon, Eklund, PHM08). Archive `data/CMAPSSData.zip`
  byte-pinned in `MANIFEST.toml`.
- **Verified license**: **Public Domain** (NASA Open Data —
  well-established U.S. government work policy).
- **DATASETS const claim**: matches.
- **Status**: correct. No action required.
- **Decision**: **ADMIT** (no change).

### S-REAL.2a candidate datasets (the three originally DEFERRED)

#### lo2 — **PROMOTE FROM DEFER**

- **Upstream**: Zenodo DOI 10.5281/zenodo.14257989, "LO2:
  Microservice Dataset of Logs and Metrics", Bakhtin et al.,
  February 28, 2025.
- **Verified license**: **CC-BY-4.0** (Creative Commons
  Attribution 4.0 International).
- **TSV header claim**: `as-distributed-by-LO2-PROMISE-2025`
  (placeholder).
- **Status**: license clearly declared on the Zenodo record;
  the previous DEFER (conservative-rule, no upstream
  inspection) is now resolved.
- **Decision**: **ADMIT under CC-BY-4.0** with attribution to
  Bakhtin, Nyyssölä, Wang, Ahmad, Ping, Esposito, Mäntylä,
  Taibi. This commit rewrites the LO2 TSV header to
  `# license=CC-BY-4.0` plus a `# attribution=` line, re-pins
  SHA-256 in `MANIFEST.toml`, and adds `lo2` to
  `s_real_audit.rs::DATASETS`.

#### deeptralog — **PERMANENT DEFER** (evidence-based)

- **Upstream**: GitHub repo `FudanSELab/DeepTraLog` (Zhang et al.,
  ICSE 2022).
- **Verified license**: **NO LICENSE**. GitHub's license detector
  returns 404. The repo root contains README (one line),
  `.gitattributes`, `GraphData/`, `TraceLogData/`, `model/`,
  `results/` — no LICENSE / LICENSE.md / COPYING / DATA_LICENSE.
  The README contains no license declaration.
- **Decision**: **PERMANENT DEFER**. Under default copyright
  law, no LICENSE file = all-rights-reserved. The DeepTraLog
  fixture vendored in `data/fixtures/deeptralog.tsv` is
  inadmissible until FudanSELab grants explicit written
  permission, publishes a LICENSE file, or releases the
  dataset under a canonical short-form. The fixture stays
  vendored (the bytes are publicly readable via GitHub) but
  is permanently excluded from DATASETS and from any S-REAL
  audit.

#### multidim_localization — **PERMANENT DEFER** (evidence-based)

- **Upstream**: GitHub repo
  `NetManAIOps/MultiDimension-Localization` (2019 AIOps
  Challenge match 2).
- **Verified license**: **NO LICENSE**. GitHub's license detector
  returns 404. The repo contains the README (which links to
  competition.aiops-challenge.com — unreachable at verification
  time), `ground_truth.csv`, and 10 ZIP files. No LICENSE.
- **Parent-org policy**: Tsinghua NetMan Lab website declares
  "Copyright © 2026 Tsinghua NetMan Lab. All Rights Reserved"
  with no separate data-licensing carve-out.
- **Decision**: **PERMANENT DEFER**. Same posture as
  deeptralog.

### S-REAL.2a primary admission candidates (F1 + F5)

#### tadbench_f04, tadbench_f19, tadbench_f11b — **license-correction required** (admissible)

Same Zenodo source as tadbench_f11 (DOI 10.5281/zenodo.6979726).

- **Verified license**: **CC-BY-4.0**.
- **MANIFEST claim**: `as-distributed` placeholder.
- **Intake manifest claim**: `Apache-2.0`.
- **Status**: same MISCLAIM as tadbench_f11. Apache-2.0 is the
  wrong canonical short-form.
- **Decision**: **ADMIT** under CC-BY-4.0. Follow-on commit
  S-REAL.2c.tadbench-license-correction handles header rewrites
  + SHA re-pin. S-REAL.2d adds these three to DATASETS with
  CC-BY-4.0.

#### defects4j — **verified correct**

- **Upstream**: GitHub repo `rjust/defects4j`.
- **Verified license**: **MIT** (via `gh api repos/rjust/defects4j`,
  `license.spdx_id = "MIT"`).
- **Intake manifest claim**: MIT.
- **Status**: correct.
- **Decision**: **ADMIT** under MIT. S-REAL.2d adds to DATASETS.

### S-REAL.2a alternates

#### bugsinpy — **PERMANENT DEFER** (evidence-based)

- **Upstream**: GitHub repo `soarsmu/BugsInPy`.
- **Verified license**: **NO LICENSE**. GitHub license detector
  returns 404. The repo contains README only — no LICENSE
  declaration, no copyright statement in README.
- **Intake manifest claim**: MIT (incorrect — that claim
  appears to have been inferred from "MIT-friendly academic
  release" rather than actual LICENSE verification).
- **Status**: MISCLAIM. The previous intake manifest's
  "license = MIT" entry is wrong.
- **Decision**: **PERMANENT DEFER**. Same posture as
  deeptralog / multidim_localization. The BugsInPy fixture
  stays vendored in `data/fixtures/` but is permanently
  inadmissible until soarsmu grants explicit written
  permission, publishes a LICENSE file, or releases under a
  canonical short-form.

#### promise_defect_prediction — **PERMANENT DEFER** (evidence-based; no canonical upstream)

- **Upstream**: PROMISE software-engineering data repository
  (originally hosted at promisedata.org; mirrored across
  multiple GitHub repos). The dsfb-debug projection used an
  unspecified PROMISE mirror.
- **Verified license**: **NO CANONICAL LICENSE**. The PROMISE
  corpus does not publish a unified canonical license; each
  contributing paper retains its own publisher's licensing,
  and the mirror operators distribute "for research use"
  without a canonical short-form. This is the most explicit
  example of the dsfb-debug pipeline's `as-distributed`
  pattern producing a label that does not map to any
  canonical short-form.
- **Decision**: **PERMANENT DEFER**.

## Aggregate result

| family | dataset                       | claimed                     | verified                                 | decision                              |
|--------|-------------------------------|-----------------------------|------------------------------------------|---------------------------------------|
| F1     | tadbench_f11 (SEALED S-REAL.1) | Apache-2.0                  | **CC-BY-4.0**                            | ADMIT — license-correction follow-on  |
| F1     | tadbench_f04                  | Apache-2.0                  | **CC-BY-4.0**                            | ADMIT — DATASETS extension in S-REAL.2d |
| F1     | tadbench_f19                  | Apache-2.0                  | **CC-BY-4.0**                            | ADMIT — DATASETS extension in S-REAL.2d |
| F1     | tadbench_f11b                 | Apache-2.0                  | **CC-BY-4.0**                            | ADMIT — DATASETS extension in S-REAL.2d |
| F2     | illinois_socialnet (SEALED)   | CC0-1.0                     | CC0-1.0                                  | ADMIT — no change                     |
| F2     | lo2                           | as-distributed (DEFER)      | **CC-BY-4.0**                            | **PROMOTE TO ADMIT** in this commit   |
| F2     | deeptralog                    | as-distributed (DEFER)      | **NO LICENSE**                           | **PERMANENT DEFER**                   |
| F3     | aiops_kpi (SEALED S-REAL.1)   | Apache-2.0                  | **NO LICENSE**                           | **PANEL DECISION REQUIRED**           |
| F3     | multidim_localization         | as-distributed (DEFER)      | **NO LICENSE** + org all-rights-reserved | **PERMANENT DEFER**                   |
| F4     | cmapss_fd001_unit1 (SEALED)   | Public Domain (NASA)        | Public Domain (NASA)                     | ADMIT — no change                     |
| F5     | defects4j                     | MIT                         | MIT                                      | ADMIT — DATASETS extension in S-REAL.2d |
| F5     | bugsinpy (alternate)          | MIT                         | **NO LICENSE**                           | **PERMANENT DEFER**                   |
| F5     | promise_defect_prediction     | PROMISE-mirror              | **NO CANONICAL LICENSE**                 | **PERMANENT DEFER**                   |

Summary by decision:
- **9 ADMIT** total: 4 TADBench (CC-BY-4.0), 1 Illinois (CC0-1.0),
  1 LO2 (CC-BY-4.0 — promoted in this commit), 1 C-MAPSS
  (Public Domain), 1 Defects4J (MIT), plus 1 pending the
  AIOps panel decision.
- **4 PERMANENT DEFER**: DeepTraLog, MultiDim Localization,
  BugsInPy, PROMISE — all four have no upstream LICENSE.
- **1 PANEL DECISION REQUIRED**: aiops_kpi (sealed S-REAL.1
  audit + DATASETS entry claim Apache-2.0; upstream has no
  LICENSE).

## Source-class breadth posture after these decisions

Strict-hard-rule application (Path A on AIOps):

```
F1 DebuggingSoftwareTelemetry  : 4 ADMIT  (TADBench F11/F04/F19/F11b)
F2 ObservabilityTraces         : 2 ADMIT  (Illinois + LO2)
F3 TimeSeriesAnomaly           : 0 ADMIT  (AIOps revoked; no other F3 candidate)
F4 ReliabilityIndustrial       : 1 ADMIT  (C-MAPSS FD001 unit 1)
F5 SoftwareDefects             : 1 ADMIT  (Defects4J)
```

The F3 family loses representation under strict hard-rule
application. This is a significant cost; the user may choose
Path B to preserve F3 representation via the AIOps fair-use
admission.

Fair-use admission (Path B on AIOps):

```
F1 DebuggingSoftwareTelemetry  : 4 ADMIT
F2 ObservabilityTraces         : 2 ADMIT
F3 TimeSeriesAnomaly           : 1 ADMIT  (AIOps under explicit fair-use framing)
F4 ReliabilityIndustrial       : 1 ADMIT
F5 SoftwareDefects             : 1 ADMIT
```

Path B preserves 5-family breadth but accepts the licensing
ambiguity with an honest disclosure in the audit reports.

## What this commit (S-REAL.2b.1) executes

1. **LO2 promotion**: rewrites `data/fixtures/lo2.tsv` header
   `# license=` line to `CC-BY-4.0` + an `# attribution=`
   line crediting the LO2 authors (CC-BY-4.0 requirement);
   re-pins the SHA-256 in `data/fixtures/MANIFEST.toml`; adds
   the `lo2` entry to `s_real_audit.rs::DATASETS`; updates the
   `lookup_admits_known_datasets` test; runs the S-REAL audit
   on `lo2` and commits the 9 sealed artifacts under
   `reports/s_real_2/lo2/`.

2. **DeepTraLog + MultiDim Localization + BugsInPy + PROMISE
   permanent-DEFER markers**: each `MANIFEST.toml` entry gains
   a `license_verification_status =
   "permanently_deferred_no_upstream_license"` field. The
   fixtures stay vendored (the bytes are publicly readable
   via GitHub) but are clearly marked inadmissible to any
   S-REAL audit.

3. **Verification report**: this file is the verification
   receipt; no further verification report needed.

4. **Intake manifest update**: rewrite the relevant target +
   alternate rows with the verified license findings and the
   updated decision codes (DEFER → ADMIT for LO2;
   DEFER → PERMANENT DEFER for DeepTraLog / MultiDim;
   MIT → PERMANENT DEFER for BugsInPy; etc.).

## What this commit does NOT execute

- **TADBench license-claim correction** (Apache-2.0 →
  CC-BY-4.0). Surfaces the misclaim but defers the action to
  a follow-on `S-REAL.2c.tadbench-license-correction` commit.
  Reason: rewriting the TADBench TSV headers + re-pinning
  SHA-256s + rebaselining the sealed S-REAL.1 TADBench F11
  audit is a separate cohesive change that deserves its own
  commit and message.
- **AIOps revocation or fair-use admission**. Surfaces the
  misclaim and the panel decision required. Defers action to
  a follow-on `S-REAL.2c.aiops-decision` commit driven by the
  user's chosen path.
- **DATASETS extension with the other admitted candidates**
  (TADBench F04/F19/F11b + Defects4J). Defers to S-REAL.2d
  per the original commit-chain plan. The TADBench three
  admit cleanly only AFTER the license-claim correction lands.
- **Per-dataset audits** for the new admissions. Defers to
  S-REAL.2e.

## Panel-locked non-claims

- S-REAL.2b.1 does NOT claim retroactive validation of the
  S-REAL.1 sealed audits. The AIOps misclaim is real and
  requires the panel decision described above. The TADBench
  misclaim is a documentation accuracy issue, not an
  admissibility issue — both Apache-2.0 and CC-BY-4.0 are
  open-source-friendly licenses, so the sealed S-REAL.1
  TADBench F11 audit's binary admissibility is unaffected,
  only its license-string accuracy.
- S-REAL.2b.1 does NOT browse private or authenticated
  sources. Verification is bounded to public Zenodo / GitHub
  / Illinois Data Bank / NASA Open Data Portal metadata.
- S-REAL.2b.1 does NOT modify the C-MAPSS FD001 unit 1 sealed
  S-REAL.2.f4-anchor audit. Its license claim is verified
  correct.
- S-REAL.2b.1 does NOT modify the Illinois SocialNet sealed
  S-REAL.1 audit. Its license claim is verified correct.

## Verification methodology log (for reproducibility)

The findings above are reproducible by running the following
queries:

```
# LO2 — Zenodo record license
curl -s https://zenodo.org/api/records/14257989 | jq '.metadata.license, .metadata.rights'

# TADBench TrainTicket — Zenodo record license
curl -s https://zenodo.org/api/records/6979726 | jq '.metadata.license, .metadata.rights'

# Illinois SocialNet — Illinois Data Bank rights field
# (web page only; no public API)
curl -s 'https://databank.illinois.edu/datasets/IDB-6738796' \
    | grep -iE 'license|rights|cc0|cc-by|public domain'

# Defects4J — GitHub license API
gh api repos/rjust/defects4j/license --jq '{name: .license.name, spdx_id: .license.spdx_id}'

# Bagel (AIOps fixture source) — GitHub license API
gh api repos/NetManAIOps/Bagel/license
# → 404 Not Found (no LICENSE file)

# DeepTraLog — GitHub license API
gh api repos/FudanSELab/DeepTraLog/license
# → 404 Not Found

# MultiDim Localization — GitHub license API
gh api repos/NetManAIOps/MultiDimension-Localization/license
# → 404 Not Found

# BugsInPy — GitHub license API
gh api repos/soarsmu/BugsInPy/license
# → 404 Not Found

# Tsinghua NetMan Lab parent-org licensing policy
# https://netman.aiops.org/ footer: "Copyright © 2026 Tsinghua NetMan Lab. All Rights Reserved"
```

License: Apache-2.0. Background IP: Invariant Forge LLC.
