# DSFB-Debug — Real-world dataset folder

This folder is the empirical floor of the DSFB-Debug crate. Every byte
under `data/fixtures/` is either an extracted slice of a named upstream
public dataset (with DOI, license, and SHA-256 in `MANIFEST.toml`) or an
explicit `# UPSTREAM_FIXTURE_NOT_VENDORED` sentinel.

**Crate authoring policy — non-negotiable:**

> Real-world data only. No synthetic signal generators. No simulators
> (MuJoCo, Isaac Sim, Gazebo, etc.). `paper-lock` evaluation hard-errors
> with `MissingRealData` whenever a fixture is the sentinel form, and
> with `HashMismatch` whenever an in-tree slice has drifted from the
> SHA-256 recorded in `MANIFEST.toml`. Synthetic fall-back is forbidden.

## Folder layout

```
data/
├── MANIFEST.toml             — one entry per vendored fixture
├── README.md                 — this file
├── dataset.txt               — panel transcript (Phase I dataset selection)
├── fixtures/                 — extracted slices, one TSV per slice
│   ├── tadbench_trainticket_F04.tsv
│   ├── tadbench_trainticket_F11.tsv
│   ├── tadbench_trainticket_F19.tsv
│   └── illinois_socialnetwork.tsv
└── fault_labels/             — per-dataset fault-window mappings
    ├── tadbench.json
    └── illinois.json
```

## Phase I dataset selection

Per the panel transcript at [`dataset.txt`](dataset.txt) and confirmed
during planning:

| Priority | Dataset | Why                                                       | Source DOI |
|---------:|---------|-----------------------------------------------------------|------------|
| 1 | **TADBench / TrainTicket** | Canonical microservice trace anomaly benchmark; 22 industrial fault cases; 41 microservices; expected for IEEE TSC review. | `10.5281/zenodo.6979726` (TrainTicket-Anomaly subset) + `https://github.com/FudanSELab/train-ticket` |
| 2 | **Illinois unsampled traces** | 100 % trace capture eliminates sampling-aliasing artefacts that would otherwise be misread as slew events. | Illinois Data Bank `IDB-6738796` |

Phase II datasets (deferred, not vendored): AIOps Challenge 2020/2021,
LO2 (PROMISE 2025), MultiDimension-Localization, DeepTraLog. See
[`../docs/dataset_provenance.md`](../docs/dataset_provenance.md) for
full provenance and Phase II citation chain.

## Fixture format — residual-projection v1

Each fixture is a UTF-8 TSV file with a comment-prefixed header and one
data row per evaluation window. Header keys (in any order, before the
data rows):

```
# residual-projection v1
# num_windows=<u32>
# num_signals=<u16>
# healthy_window_end=<u32>     ← boundary index between baseline and evaluation regions
# fault_labels=<csv of u32>    ← window indices flagged as faults; may be empty
# upstream_doi=<string>
# upstream_url=<string>
# upstream_archive_sha256=<64-hex>
# extracted_at=<ISO-8601>
# extraction_recipe=<string>
# license=<SPDX id>
```

After the header, exactly `num_windows` rows of `num_signals` TAB-separated
f64 values. Values are residual projections — typically per-service
`(latency_p50_ms, error_rate)` pairs binned at fixed window width.

A fixture with the comment line `# UPSTREAM_FIXTURE_NOT_VENDORED` and no
data rows is a sentinel: the test harness recognises it and emits
`MissingRealData` rather than synthesising a fall-back.

## Extraction recipe — TADBench / TrainTicket

The crate does not run network fetches inside `cargo test`. To populate a
TADBench fixture:

1. Obtain the upstream archive from
   `https://zenodo.org/records/6979726` (canonical `dsfb-* paper` cite)
   or `https://github.com/FudanSELab/train-ticket` (current upstream).
2. Compute the SHA-256 of the archive and record it in
   `MANIFEST.toml` under the relevant block's `upstream_archive_sha256`.
3. For the chosen fault case (e.g. F-04), select a contiguous window of
   spans covering the healthy baseline + the injected fault. Per-service
   bin into 1-second windows and compute:
   - `latency_p50_ms` — 50th-percentile span duration (start → end)
   - `error_rate` — fraction of spans tagged `error=true` in the bin
   These two metrics constitute one signal pair per service. Aggregate
   across the active services to produce the `num_signals` columns.
4. Mark the fault-injection windows in the `fault_labels=` header as a
   comma-separated list of window indices.
5. Write the resulting TSV to the path declared in the manifest.
6. Recompute the file's SHA-256 (any standard tool — the crate also
   exposes `dsfb_debug::adapters::sha256::sha256_hex` if you wish to
   compute it from a host program) and update the manifest's
   `fixture_sha256` field with the lowercase hex digest.

## Extraction recipe — Illinois unsampled

1. Download the SocialNetwork benchmark archive from
   `https://databank.illinois.edu/datasets/IDB-6738796`.
2. Confirm the archive's SHA-256 against the upstream listing and update
   the manifest.
3. Bin spans into 500-ms windows (the dataset's nominal granularity);
   compute the same `(latency_p50_ms, error_rate)` projection per service.
4. Healthy baseline = first 100 windows of nominal load
   (precedes the fault-injection script's start). Mark fault windows.
5. Write the TSV; recompute and record `fixture_sha256`.

## How the crate consumes these fixtures

```rust
// in tests/eval_tadbench.rs (gated #[cfg(all(feature = "std", feature = "paper-lock"))])
use dsfb_debug::real_data::{evaluate_real_dataset, MANIFEST_TADBENCH_F04};
use dsfb_debug::DsfbDebugEngine;

const F04_BYTES: &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F04.tsv");

let engine = DsfbDebugEngine::<32, 64>::paper_lock().unwrap();
match evaluate_real_dataset(&engine, &MANIFEST_TADBENCH_F04, F04_BYTES) {
    Ok(eval) => println!("{eval:#?}"),
    Err(e)   => println!("{e} — populate fixture per data/README.md §Extraction"),
}
```

## Acquisition log (Session 4, 2026-05-05)

| Dataset | Status | Local path | Size | SHA-256 / commit |
|---------|--------|-----------|------|------------------|
| TrainTicket-Anomaly | **Vendored end-to-end** | `data/fixtures/tadbench_trainticket_F11.tsv` | 32 KB (projected) / 147 MB (upstream) | fixture `07c8f085...12c3` ; archive `18456279...4403` |
| LO2 (sample) | Acquired; projection deferred | `data/upstream/lo2_sample.zip` | 1021 MB | `2d9516fe...a882` |
| MultiDim-Localization | Acquired; projection deferred | `data/upstream/multidim_localization/` (git) | 1.3 GB | git commit `d3159c50` |
| DeepTraLog | Acquired; projection deferred | `data/upstream/deeptralog/` (git) | 1.8 GB | git commit `04376dcb` |
| AIOps Challenge 2020 | Manual fetch (Tsinghua Cloud / Google Drive) | — | ~? GB | per-archive `fac7fe1b4e048c81ef88874334b73534` (md5, upstream README) |
| Illinois IDB-6738796 | Manual fetch (presigned S3, large) | — | ~? GB | needs HEAD with auth |

Archives under `data/upstream/` are gitignored. They are intentionally
local-only: SHA-256 (or git commit) per archive is recorded in
`MANIFEST.toml` for reviewer verification, and the projected slice
under `data/fixtures/` is the bytes the engine actually consumes.

### Manual-fetch recipes

**AIOps Challenge 2020/2021** — gated by Tsinghua Cloud / Google Drive
links in `https://github.com/NetManAIOps/AIOps-Challenge-2020-Data` and
the corresponding 2021 repository's README. Download the per-stage
archives, verify against the README's `md5sum`, and place under
`data/upstream/aiops_challenge_2020/` and `data/upstream/aiops_challenge_2021/`.

**Illinois Microservice Tracing Dataset (IDB-6738796)** — visit
`https://databank.illinois.edu/datasets/IDB-6738796` in a browser
session and download `tracing-data.tar.gz` (the redirect target is a
short-lived presigned S3 URL). Place under
`data/upstream/illinois_socialnetwork.tar.gz` and run
`sha256sum` to verify against the manifest.

## Reproducibility checklist

A reviewer reproducing the published metrics should verify, in order:

- [ ] `MANIFEST.toml` parses cleanly under TOML 1.0.
- [ ] Every `fixture_path` exists and is non-empty.
- [ ] No fixture begins with `# UPSTREAM_FIXTURE_NOT_VENDORED`.
- [ ] The SHA-256 of each `fixture_path` matches its `fixture_sha256`.
- [ ] The SHA-256 of each upstream archive matches `upstream_archive_sha256`.
- [ ] `cargo test --features "std paper-lock" -- --nocapture` reports
      `deterministic_replay_holds = true` for every dataset block.
