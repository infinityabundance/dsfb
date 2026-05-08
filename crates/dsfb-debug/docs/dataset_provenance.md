# DSFB-Debug — Dataset Provenance

This document is the authoritative provenance ledger for every real-world
dataset cited in the paper or referenced by the test harness. Each entry
records: canonical source, DOI, licence, version/commit hash where
applicable, and the precise extraction recipe used to produce the
in-tree slice. The control-IDs from
[`standards_alignment.md`](standards_alignment.md) cite these entries by
name; the manifest at [`../data/MANIFEST.toml`](../data/MANIFEST.toml)
cross-references this document.

## Authoring policy

- **Real-world data only.** No synthetic generators, no simulators, no
  hand-fabricated bytes.
- **Provenance-first.** Every fixture file traces to a DOI; every DOI
  traces to a published archive; every archive has a SHA-256.
- **Sentinel-safe.** A fixture that has not yet been extracted from
  upstream is the `# UPSTREAM_FIXTURE_NOT_VENDORED` sentinel, never a
  fall-back.
- **Post-Phase-G state (current).** Twelve real-bytes vendored fixtures
  across nine distinct upstream public datasets. All DOI-pinned, all
  SHA-256-gated, all deterministic-replay-verified.

## Vendored fixtures (12 total, all real bytes)

| # | Fixture file | Upstream | Real bytes? |
|---|--------------|----------|:------:|
| 1 | `tadbench_trainticket_F11.tsv` | Zenodo `10.5281/zenodo.6979726` (order-service mongodb 4.2.2) | ✓ |
| 2 | `tadbench_trainticket_F11b.tsv` | same archive (auth-mongo 5.0.9) | ✓ |
| 3 | `tadbench_trainticket_F04.tsv` | same archive (admin-service springstarter 1.5.22) | ✓ |
| 4 | `tadbench_trainticket_F19.tsv` | same archive (mongodb-driver-3.0.4) | ✓ |
| 5 | `illinois_socialnetwork.tsv` | DataBank `10.13012/B2IDB-6738796_V1` | ✓ |
| 6 | `aiops_challenge.tsv` | NetManAIOps/Bagel `sample_data.csv` (AIOps Challenge 2018 KPI) | ✓ |
| 7 | `lo2.tsv` | Zenodo `10.5281/zenodo.14257989` (LO2 PROMISE 2025) | ✓ |
| 8 | `multidim_localization.tsv` | NetManAIOps/MultiDimension-Localization `part1.zip` | ✓ |
| 9 | `deeptralog.tsv` | FudanSELab/DeepTraLog `F01.zip` | ✓ |
| 10 | `defects4j.tsv` | rjust/defects4j `framework/projects` (6 projects' commit-db) | ✓ |
| 11 | `bugsinpy.tsv` | soarsmu/BugsInPy (6 projects' bug.info files) | ✓ |
| 12 | `promise_defect_prediction.tsv` | ssea-lab/PROMISE (6 PROMISE CSVs) | ✓ |

Each projection script lives at `data/upstream/project_<dataset>.py` and
is deterministic: re-running it on the same upstream archive produces
the same TSV (modulo the timestamp header line). Per-fixture SHA-256
digests live in [`../data/MANIFEST.toml`](../data/MANIFEST.toml).

---

## Per-dataset provenance

### TADBench / TrainTicket fault-injection traces

| Field | Value |
|-------|-------|
| Citation key | `tadbench2025` / `defects4j` (associated TrainTicket) |
| Canonical paper | Sun et al., "TADBench: A Comprehensive Benchmark for Trace Anomaly Detection." IEEE Transactions on Services Computing, 2025. |
| Upstream archive | `https://github.com/FudanSELab/train-ticket` |
| Zenodo mirror | `10.5281/zenodo.6979726` (TrainTicket-Anomaly subset) |
| Licence | Apache-2.0 |
| Format | Jaeger spans (JSON) |
| Faults represented | 22 industrial fault cases (cascading-timeout, deployment-regression, connection-pool-exhaustion, resource-saturation, queue-backpressure, …) |
| Vendored slices | 4 (F-04 admin-service config, F-11 order-service mongodb 4.2.2 deployment regression, F-11b auth-mongo, F-19 mongodb-driver-3.0.4 config) |
| Crate manifest blocks | `tadbench_trainticket_F04`, `tadbench_trainticket_F11`, `tadbench_trainticket_F11b`, `tadbench_trainticket_F19` |
| Fault-label mapping | `data/fault_labels/tadbench.json` |
| Slice format | Residual-projection v2 TSV (per-service `latency_p50_ms`, `error_rate` at 15-second windows) |

### Illinois Microservice Tracing Dataset

| Field | Value |
|-------|-------|
| Citation key | `illinois_traces` |
| Canonical source | University of Illinois Data Bank, dataset `IDB-6738796` |
| Upstream URL | `https://databank.illinois.edu/datasets/IDB-6738796` |
| DOI | `10.13012/B2IDB-6738796_V1` |
| Licence | CC0-1.0 |
| Format | Pre-processed trace CSVs |
| Benchmark applications | SocialNetwork, MediaMicroservices, HotelReservation, TrainTicket |
| Cluster | 15-node heterogeneous Kubernetes |
| Sampling | **100% trace capture (unsampled)** — the critical property for DSFB-Debug drift / slew confound elimination |
| Vendored slices | 1 (SocialNetwork compose-review unsampled, 160 000 traces) |
| Crate manifest block | `illinois_socialnetwork` |
| Fault-label mapping | `data/fault_labels/illinois.json` |

### AIOps Challenge 2018 KPI (NetManAIOps / Bagel sample)

| Field | Value |
|-------|-------|
| Citation key | (Su et al., IPCCC 2018 — Bagel) |
| Canonical source | Tsinghua NetMan group, AIOps Challenge 2018 KPI dataset, distributed via the Bagel sample |
| Upstream | `https://github.com/NetManAIOps/Bagel` (`sample_data.csv`) |
| Licence | Apache-2.0 (Bagel repository) |
| Modalities | 1D KPI time-series (5-min cadence, 17 569 samples) with binary anomaly labels |
| Vendored slices | 1 (32 windows × 4 sub-segments reshape) |
| Crate manifest block | `aiops_challenge` |
| Fault-label mapping | `data/fault_labels/aiops_challenge.json` |

### LO2 (PROMISE 2025)

| Field | Value |
|-------|-------|
| Citation key | `lo2` |
| Canonical paper | Bakhtin et al., "LO2: Microservice API Anomaly Dataset of Logs and Metrics," PROMISE 2025 |
| Upstream | Zenodo `10.5281/zenodo.14257989` |
| Licence | per-archive (verify before vendoring) |
| Novel property | API-semantic anomalies (OAuth2.0 flow failures, architectural degradation) |
| Endoductive role | Faults map to API-level semantics NOT in the heuristics bank; engine produces `SemanticDisposition::Unknown` with a valid `(r,d,s)` tuple chain — exercising the §5.6 endoductive branch that flat threshold detectors cannot exercise at all. |
| Vendored slices | 1 (light-oauth2 first-CSV first-16-rows Go-runtime metrics) |
| Crate manifest block | `lo2` |
| Fault-label mapping | `data/fault_labels/lo2.json` |

### MultiDimension-Localization

| Field | Value |
|-------|-------|
| Citation key | (NetManAIOps / MultiDimension-Localization) |
| Canonical source | NetManAIOps GitHub (`MultiDimension-Localization`) |
| Properties | 21 microservices × 19 metrics per service = 399 time series; 58 anomaly cases with ground-truth root causes |
| Licence | per-archive |
| Vendored slices | 1 (part1, 12 windows × 4 categorical levels mean) |
| Crate manifest block | `multidim_localization` |
| Fault-label mapping | `data/fault_labels/multidim_localization.json` |

### DeepTraLog

| Field | Value |
|-------|-------|
| Citation key | `deeptralog2022` |
| Canonical paper | Zhang et al., "DeepTraLog," ICSE 2022 |
| Upstream | `https://github.com/FudanSELab/DeepTraLog` (`TraceLogData/F01.zip`) |
| Licence | as-distributed-by-FudanSELab/DeepTraLog |
| Properties | Combined log + trace data for normal and abnormal microservice executions |
| Vendored slices | 1 (F01-01 ERROR span data, 600 spans, 16 windows × 8 signals) |
| Crate manifest block | `deeptralog` |
| Fault-label mapping | `data/fault_labels/deeptralog.json` |

### Defects4J (Phase G — code-debugging)

| Field | Value |
|-------|-------|
| Citation key | `defects4j` |
| Canonical paper | Just, Jalali, and Ernst, "Defects4J: A Database of Existing Faults to Enable Controlled Testing Studies for Java Programs," ISSTA 2014 |
| Upstream | `https://github.com/rjust/defects4j` (`framework/projects/`) |
| Licence | MIT |
| Properties | Java bug benchmark; 17 OSS projects, 835 bugs total; per-project `commit-db` files documenting buggy/fixed commits + JIRA report ids |
| Vendored slices | 1 (six projects: Lang / Math / Closure / Mockito / JacksonDatabind / Jsoup × first-30-bug residual matrix) |
| Crate manifest block | `defects4j` |
| Fault-label mapping | (none — projection signal is JIRA report id sequence) |

### BugsInPy (Phase G — code-debugging)

| Field | Value |
|-------|-------|
| Citation key | `bugsinpy_fse2020` |
| Canonical paper | Widyasari et al., "BugsInPy: A Database of Existing Bugs in Python Programs to Enable Controlled Testing and Debugging Studies," ESEC/FSE 2020 |
| Upstream | `https://github.com/soarsmu/BugsInPy` |
| Licence | MIT |
| Properties | Python bug benchmark; 17 projects, 493 bugs; per-bug `bug.info` files with buggy/fixed commit ids |
| Vendored slices | 1 (six projects: ansible / pandas / keras / fastapi / scrapy / tornado × first-30-bug residual matrix; signal = SHA-1-prefix-as-int) |
| Crate manifest block | `bugsinpy` |
| Fault-label mapping | (none — projection signal is SHA-1 prefix sequence) |

### PROMISE defect-prediction (Phase G — code-debugging)

| Field | Value |
|-------|-------|
| Citation key | `promise2003` |
| Canonical source | Menzies et al., PROMISE Repository (2003+) |
| Upstream mirror | `https://github.com/ssea-lab/PROMISE` |
| Licence | as-distributed-by-ssea-lab/PROMISE-mirror |
| Properties | Per-module Chidamber-Kemerer OO metrics + bug counts across 34 Java OSS project versions |
| Vendored slices | 1 (six PROMISE CSVs: 1.csv, 5.csv, 10.csv, 15.csv, 20.csv, 25.csv × first-30-module bug-count) |
| Crate manifest block | `promise_defect_prediction` |
| Fault-label mapping | (none — projection signal is per-module bug count) |

---

## Future scope (no current vendoring claim)

The data folder structure already supports any defence-mission,
production-cloud, or industry-partner dataset that becomes available —
none are in scope today, and none are silently absent: adding any new
dataset triggers a new MANIFEST.toml block, a new `RealDatasetManifest`
constant, a new fixture (sentinel until vendored), and a new eval
test, following the recipe documented in
[`../data/README.md`](../data/README.md).

---

## Reproducibility checklist

A reviewer reproducing the published numbers should:

- [ ] Verify each upstream archive's SHA-256 against the manifest's
      `upstream_archive_sha256` field.
- [ ] Re-run the corresponding `data/upstream/project_<dataset>.py`
      script on the upstream archive.
- [ ] Confirm the recomputed slice's SHA-256 matches
      `MANIFEST.toml`'s `fixture_sha256` field.
- [ ] Run `cargo test --features "std paper-lock" -- --nocapture` and
      capture the JSON metric blocks.
- [ ] Cross-check the captured JSON against the paper's §13 results
      table; numbers must match within the precision printed by the
      harness.
- [ ] Confirm `deterministic_replay_holds = true` for every dataset.

If any step fails, the dataset is not reproducible. Underclaim,
overdeliver: do not publish numbers that have not been replayed at least
once on a fresh checkout.
