# DSFB-GPU S-REAL Audit Gauntlet — Master Index

**Executive reader surface.** One page. Every audit, every replay
status, every episode count, every source class, every license,
every final hash. No prose, no decoration. Replay the gauntlet
by running `cargo run --release --features cuda -p dsfb-gpu-debug-demo --bin dsfb-gpu-debug -- s-real-audit --dataset all` plus the
S-REAL.3 datasets via `--dataset <id>`. (`s-real-1-audit` is
accepted as a historical alias from the original 3-dataset
S-REAL.1 seal; the handler now covers all 20 sealed fixtures.)

## Headline

**20 byte-identical-replay-verified deterministic audits across 5
source-class families. 316 admitted episodes. Zero ground-truth
claims.**

```
F1 DebuggingSoftwareTelemetry  : 4 datasets, 111 episodes
F2 ObservabilityTraces         : 4 datasets,  19 episodes
F3 TimeSeriesAnomaly           : 2 datasets,   6 episodes
F4 ReliabilityIndustrial       : 6 datasets, 161 episodes
F5 SoftwareDefects             : 4 datasets,  19 episodes
                                 ──────────────────────────
                                  20 datasets, 316 episodes
```

## Saturation snapshot (S-REAL.3.1)

The same audit chain, re-run through the synthetic S-PERF.16.a
saturation bench harness on 10 large 1M-cell fixtures
(real public RF I/Q, mmWave power, and database-derived
residual surfaces — JOB/IMDB byte-frequency, IMDB tarball
cast-info projection, Snowset query-event CSV, SQLShare
seaflow CSV),
reaches **16.16 .. 26.66 GB/s** logical throughput on the
264-byte `DetectorCellWide` arena — up to **117 %** of the
sealed synthetic S-PERF.16.a median (22.74 GB/s on RTX 4080
SUPER / CUDA 13.2). The 20 small audit fixtures remain
launch-bound (0.05 .. 0.82 GB/s) and are reported honestly.

| classification     | count | wide GB/s range          |
|--------------------|------:|--------------------------|
| saturation-class   |    10 | 16.16 .. 26.66           |
| launch-bound       |    20 |  0.05 ..  0.82           |
| transition         |     0 | —                        |

Top saturation-class results (sealed receipt
`reports/s_real_saturation_sweep.txt`, 15 iters per fixture):

| dataset_id                 | n_cells   | wide GB/s |
|----------------------------|----------:|----------:|
| imdb_tgz_large             | 1 044 480 |    26.66  |
| snowset_large              | 1 048 576 |    26.09  |
| sqlshare_large             | 1 048 576 |    26.13  |
| imdb_duckdb_large          | 1 048 576 |    25.94  |
| powder_large               | 1 048 576 |    25.92  |
| radioml_gold_large         | 1 048 576 |    25.92  |
| oracle_large               | 1 048 576 |    25.88  |
| deepbeam_large             | 1 048 576 |    25.89  |
| deepsense6g_large          |   524 288 |    20.11  |
| radioml_2018_snr30_large   | 1 048 576 |    16.16  |

**Plan-locked non-claims** (must be preserved verbatim
downstream):

- Numbers above are LOGICAL throughput on the 264-byte
  wide-cell arena, NOT physical DRAM bandwidth. Physical
  DRAM% lives under
  `scripts/s_real_perf_per_dataset.sh --ncu <id>`.
- Saturation-class / launch-bound classification is a property
  of cell-count and dispatcher-shape — NOT a detector-quality
  claim and NOT an RF / observability / industrial /
  database domain-truth claim.
- Cross-driver / cross-CUDA / cross-hardware throughput
  identity is NOT claimed.
- The 22.74 GB/s anchor is the sealed S-PERF.16.a synthetic
  median; results "above 100 % of anchor" simply reflect
  fixture-specific cache/launch behaviour, not a tighter
  bound on physical bandwidth.

## Master audit table

| # | dataset_id                | family | license              | episodes | replay | sealed-tier | final_case_file_hash                                           |
|--:|---------------------------|--------|----------------------|---------:|--------|-------------|----------------------------------------------------------------|
|  1 | tadbench_f11             | F1     | CC-BY-4.0            |       90 | YES    | S-REAL.1    | `23a8975841995421e840b7d368f951206d99477c2c1146667c4fcaf88d5f70cc` |
|  2 | tadbench_f04             | F1     | CC-BY-4.0            |        6 | YES    | S-REAL.2    | `e8ab40737df0b5aafeefb03f59a694a231d5335b8115a90cf484505e384cb6b8` |
|  3 | tadbench_f19             | F1     | CC-BY-4.0            |       12 | YES    | S-REAL.2    | `0727e92beb8d2fcf8492acd08b1aea168b8281abd5ad2de227bf11fd6ce3ed5b` |
|  4 | tadbench_f11b            | F1     | CC-BY-4.0            |        3 | YES    | S-REAL.2    | `78d28a1540515980402c28b27f65fa31d80e7a60688d1a070e76bec4ae98bca8` |
|  5 | illinois_socialnet       | F2     | CC0-1.0              |        3 | YES    | S-REAL.1    | `219ff3d06d79130029b37c00e7a7508d33a788504fa0d4c06f18f3824dead98e` |
|  6 | lo2                      | F2     | CC-BY-4.0            |        4 | YES    | S-REAL.2    | `cc1995c903104c3847bc9e9ce3aa84d2a99139418a8c6811355c472b2cee597f` |
|  7 | deeptralog               | F2     | no-upstream-license  |        3 | YES    | S-REAL.2    | `99430c33871d13dd53ad6580d9f886372311869e6e5cbcc89504433a555c9b01` |
|  8 | deeptralog_f02           | F2     | no-upstream-license  |        9 | YES    | S-REAL.3    | `b22acc48d40224adc73afcff6a818af6c1c2baf20742ac230f385aadb753e8c0` |
|  9 | aiops_kpi                | F3     | no-upstream-license  |        3 | YES    | S-REAL.1    | `06a26a61c4dde60e44274eb6c521079edd00e2113c0c4e8e151a4b552b4e5082` |
| 10 | multidim_localization    | F3     | no-upstream-license  |        3 | YES    | S-REAL.2    | `7c730edae4b1c3204d15a020c9251a90e9f8c78427603fad16a894ead66eba2b` |
| 11 | cmapss_fd001_unit1       | F4     | Public-Domain        |       26 | YES    | S-REAL.2    | `396af2610c4dbaaa438142baf4c4ecf8705de0bbc2bcf6d4640da2f236d1c2fa` |
| 12 | cmapss_fd001_unit50      | F4     | Public-Domain        |       33 | YES    | S-REAL.3    | `aac75f765e5c87361ea83310991378590f8fcd12f73b85647fd12af6b5f15b83` |
| 13 | cmapss_fd002_unit1       | F4     | Public-Domain        |       25 | YES    | S-REAL.3    | `8803d9867aa50e4d0ad6c0cbf04d7d7ddeba9824377908551563771af6d9f071` |
| 14 | cmapss_fd002_unit100     | F4     | Public-Domain        |       29 | YES    | S-REAL.3    | `8169b972a1c256c7d90984eec28f6686b94aaa29e087196d5eee54b444e9714c` |
| 15 | cmapss_fd003_unit1       | F4     | Public-Domain        |       19 | YES    | S-REAL.3    | `316a876bd430864fec265e65def561ee548873b570d75dc9ef87b6afc2b27b9c` |
| 16 | cmapss_fd004_unit1       | F4     | Public-Domain        |       29 | YES    | S-REAL.3    | `ff47164d3a755c8ee4ec7dad41e1176d284896d01e23b20e974b3844dbae25a3` |
| 17 | defects4j                | F5     | MIT                  |        2 | YES    | S-REAL.2    | `0410667a6d67784dd8b178308d7278635fcbfa8ea0b0a0b1c42cc89bbea79095` |
| 18 | bugsinpy                 | F5     | no-upstream-license  |        1 | YES    | S-REAL.2    | `8412fae131b8af27c7804aa46894ff53afabaa4d8a65b524e8c68687077a1564` |
| 19 | promise_defect_prediction| F5     | no-upstream-license  |        5 | YES    | S-REAL.2    | `2cd73eceef0f89f2586acec2c76c78f86dd819212a5046f13567de495b52a35e` |
| 20 | promise_ant_1_4          | F5     | no-upstream-license  |       11 | YES    | S-REAL.3    | `379f34d0768c0e4f82b039f00fe0615b64325f72e7a3ae01c602b6665c13721f` |

## License posture

| license               | datasets | non-commercial-redistribution research-use posture |
|-----------------------|---------:|-----------------------------------------------------|
| CC-BY-4.0             |        5 | open with attribution                               |
| CC0-1.0               |        1 | public-domain dedication                            |
| MIT                   |        1 | open                                                |
| Public-Domain         |        6 | NASA Open Data, U.S. government work                |
| no-upstream-license   |        7 | redistributed under academic research-use convention; no commercial redistribution right asserted |

## Tier directories

```
reports/s_real_1/     3 audits sealed at S-REAL.1 (proof-of-life)
reports/s_real_2/     9 audits sealed at S-REAL.2 (cross-family breadth)
                      + cmapss_fd001_unit1 at S-REAL.2.f4-anchor
reports/s_real_3/     7 audits sealed at S-REAL.3 (Zenodo-publishable bundle)
                      + bundle_manifest.toml + bundle_hash_chain.txt + zenodo_metadata.json
```

## Replay protocol

Per-dataset replay verification:

1. Read the fixture SHA-256 pin from `data/fixtures/MANIFEST.toml`.
2. Re-hash the fixture file in-place. Match required.
3. Run `cargo run --release --features cuda -p dsfb-gpu-debug-demo --bin dsfb-gpu-debug -- s-real-audit --dataset <id> --out-dir <out>`.
4. Compare the emitted `final_case_file_hash` to the value in the
   "final_case_file_hash" column above.
5. Run again. Compare run-2 hash to run-1 hash. Byte-identical
   → `replay = YES`.

Full-gauntlet replay:

```
cargo run --release --features cuda -p dsfb-gpu-debug-demo \
    --bin dsfb-gpu-debug -- s-real-audit --dataset all \
    --out-dir reports/s_real_2
# Then per-dataset for the 7 S-REAL.3 admissions (each --dataset <id> --out-dir reports/s_real_3).
```

## Honest framing (must be quoted verbatim in any downstream paper or report)

> DSFB-GPU processed 20 public/research fixtures across 5
> source-class families and emitted byte-identical
> replay-verified deterministic audit artifacts. The result is
> a breadth-and-replayability proof, not a claim of ground-truth
> anomaly correctness, detector superiority, causality, or
> commercial redistribution rights for datasets with no upstream
> license.

For NASA C-MAPSS audits specifically:

> DSFB-GPU does not claim to diagnose engine physics or predict
> remaining useful life. It deterministically projects a public
> run-to-failure trajectory into a residual evidence stream and
> emits replayable structural episodes whose timing and motif
> distribution are consistent with degradation-shaped dynamics.

## Audit artifact shape (every dataset emits)

```
reports/<tier>/<dataset_id>/
  dataset_manifest.toml      upstream identity + license + SHA-256 pin
  schema_map.toml            cell → EvidenceDensor field mapping
  run_receipt.txt            dispatcher invocation + hash chain
  casefile.json              full CaseFileV2 body
  episodes.jsonl             admitted episodes (one per line, deterministic order)
  audit_report.html          human-readable 7-section deterministic HTML
  replay_verification.txt    run-1 vs run-2 SHA-256s + admission line
  limitations.md             plan-locked non-claims
  perf_profile.txt           timing fields + multi-iter variance (S-REAL.PERF)
```

Four artifacts are byte-stable under any header-only change:
`casefile.json` + `episodes.jsonl` + `schema_map.toml` +
`limitations.md`. Five record the recorded SHA / license / path
and rebaseline when those change.

## What this gauntlet does NOT prove

- Detector accuracy on labeled anomaly tasks.
- Ground-truth correctness on the audited datasets.
- Commercial redistribution rights for `no-upstream-license`
  fixtures.
- Cross-driver / cross-CUDA / cross-hardware byte-identity.
- Domain causality on any dataset.

## What this gauntlet does prove

- The same deterministic court runs across 20 public fixtures
  from 5 different source-class families.
- Two consecutive dispatches produce byte-identical final
  case-file hashes on every dataset.
- The recipe-to-fixture-to-audit chain is fully byte-pinned at
  every step (upstream archive SHA → recipe Python → projected
  TSV SHA → audit artifact SHAs).
- Honest zero-episode outcomes are reported when the data
  structure doesn't admit DSFB motifs; the court never
  manufactures episodes.

## Files

| file                                              | purpose                                            |
|---------------------------------------------------|----------------------------------------------------|
| `reports/INDEX.md` (this file)                    | Executive-reader top-level summary                 |
| `reports/s_real_1/`                               | 3 sealed S-REAL.1 proof-of-life audits             |
| `reports/s_real_2/`                               | 10 sealed S-REAL.2 cross-family audits             |
| `reports/s_real_2/intake_manifest.md`             | S-REAL.2 candidate-intake planning record          |
| `reports/s_real_2/license_verification_report.md` | Upstream-LICENSE verification evidence             |
| `reports/s_real_3/`                               | 7 sealed S-REAL.3 audits + Zenodo-publishable bundle |
| `reports/s_real_3/bundle_manifest.toml`           | Master manifest of all 20 audits                   |
| `reports/s_real_3/bundle_hash_chain.txt`          | SHA-256 of every byte-stable artifact              |
| `reports/s_real_3/zenodo_metadata.json`           | Zenodo deposit template                            |
| `data/fixtures/MANIFEST.toml`                     | Per-fixture SHA-256 + license pins                 |
| `data/fixtures/*.tsv`                             | 20 byte-pinned residual-projection TSVs            |
| `data/recipes/*.py`                               | Byte-deterministic projection recipes              |
| `data/upstream/`                                  | Vendored upstream archives                         |
| `data/CMAPSSData.zip`                             | NASA C-MAPSS PHM08 archive (Public-Domain)         |

License: Apache-2.0. Background IP: Invariant Forge LLC.
