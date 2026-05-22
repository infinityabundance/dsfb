//! S-REAL — Real Dataset Audit Gauntlet CLI driver
//! (canonical subcommand: `s-real-audit`; historical alias:
//! `s-real-1-audit`).
//!
//! WHY: This is the integration module that ties the S-REAL sealed-gauntlet
//! pipeline end-to-end:
//!
//! ```text
//!   real bytes
//!     -> SHA-pinned TSV fixture (cli::ingest)
//!     -> deterministic TraceEvent lowering (cli::ingest)
//!     -> Contract::scaled(observed_signals, observed_windows)
//!     -> build_gpu(events, contract) twice (replay-verification)
//!     -> emit(&case) → casefile.json
//!     -> serialize_episodes_jsonl → episodes.jsonl
//!     -> render_audit_report_html → audit_report.html
//!     -> 9 panel-locked artifacts per dataset in reports/<tier_dir>/<id>/
//! ```
//!
//! Panel-locked S-REAL scope (current state, post-S-REAL.3.1 seal at
//! `fde8a99`): **20 sealed audits across S-REAL.1/2/3** (3 + 10 + 7
//! datasets vendored under panel-cleared licenses — TADBench /
//! Illinois SocialNetwork / AIOps Challenge / LO2 / DeepTraLog /
//! MultiDim / Defects4J / BugsInPy / PROMISE / NASA C-MAPSS subsets),
//! **plus 10 large saturation-class fixtures** (1 M-cell RF I/Q /
//! mmWave / database-derived residual surfaces — RadioML 2018.01A,
//! DeepBeam, RadioML-Gold, POWDER, Oracle, DeepSense6G, IMDb tarball,
//! IMDb DuckDB, Snowset, SQLShare). The 30-fixture sweep is captured
//! at `reports/s_real_saturation_sweep.txt`; the 20-dataset audit
//! bundle and its 60-row hash chain live under `reports/s_real_<tier>/`.
//!
//! **This is a sealed gauntlet, not a general ingestion product.**
//! Arbitrary external datasets are intentionally not supported: the
//! value of the S-REAL audits is that the gauntlet commits to a fixed,
//! byte-pinned, license-cleared dataset set so reviewers can reproduce
//! the result exactly.
//!
//! Non-claims (preserved verbatim into every dataset's `limitations.md`,
//! mirroring `cli::audit_report::NON_CLAIMS`):
//! - Does NOT claim DSFB has identified the "real" anomaly in the dataset.
//! - Does NOT claim DSFB outperforms any other anomaly detector.
//! - Does NOT claim DSFB has discovered causality.
//! - Does NOT claim fitness-for-purpose on regulated / safety-critical use.
//! - Does NOT claim the dataset is "correctly labeled" or "ground truth".
//! - Does NOT claim the corpus or registry is exhaustive.
//! - Does NOT claim replay determinism across different driver / CUDA /
//!   hardware versions; the replay receipt records the toolchain
//!   explicitly.
//!
//! License: Apache-2.0. Background IP: Invariant Forge LLC.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use dsfb_gpu_debug_core::bank::{bank_hash, Episode};
use dsfb_gpu_debug_core::casefile::{emit, CaseFile};
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::hash::sha256;
use dsfb_gpu_debug_core::motif::registry_hash;

use super::audit_report::{
    render_audit_report_html, DatasetManifest, ReplayVerification, SchemaMap,
};
use super::ingest::{
    build_ingest_report, load_residual_projection_tsv, lower_to_trace_events, sha256_to_hex_lower,
    LoweringConfig,
};
use super::{parse_flags, usage_error};

// =====================================================================
// Panel-locked S-REAL dataset identity surfaces (split into two tables).
// =====================================================================
//
// WHY THIS SPLIT EXISTS (S-REAL.3.1.2, panel-locked 2026-05-20):
//
// Before S-REAL.3.1.2 the driver had a single `DATASETS` constant
// carrying 30 entries (20 audit datasets + 10 large saturation-class
// fixtures). `s-real-audit --dataset all` enumerated all 30, which
// (a) made the public Colab notebook's central command silently
// dispatch the 10 saturation witnesses whose TSVs the slim tarball
// deliberately excludes (~90 MB), and (b) misrepresented what "all"
// meant in the notebook's narration ("all 20 datasets") vs the code
// (all 30 entries). The bug is scope-boundary confusion, not a
// pipeline defect.
//
// The fix splits the dataset identity surface into two const tables
// with distinct authority semantics:
//
//   * `AUDIT_DATASETS` (20 entries): the sealed S-REAL.3 audit
//     gauntlet bundle. Every entry has a `tier_dir` of `s_real_1`,
//     `s_real_2`, or `s_real_3` matching the on-disk sealed bundle
//     layout that `reports/s_real_3/bundle_manifest.toml` pins.
//     `s-real-audit --dataset all` enumerates ONLY this table.
//
//   * `SATURATION_FIXTURES` (10 entries): the large saturation-class
//     real-data fixtures (RF I/Q + mmWave + database-derived
//     residual surfaces). Every entry has `tier_dir =
//     "s_real_saturation"`. These fixtures are dispatched ONLY by
//     `scripts/s_real_saturation_sweep.sh` and by explicit single-id
//     audit calls (`s-real-audit --dataset radioml_2018_snr30_large`).
//     They are NEVER selected by `--dataset all`.
//
// Both tables share the same `DatasetSpec` struct. The `tier_dir`
// field is load-bearing: it determines the output directory layout
// so a freshly-emitted bundle matches the sealed `reports/s_real_<tier>/`
// directories byte-for-byte without further path massaging.
//
// Hard-coding the 20-audit + 10-saturation tables is deliberate. The
// audit gauntlet's reproducibility story depends on every reviewer
// running against the same vendored, SHA-256-pinned, license-cleared
// fixtures; changes to entries here are panel-acknowledged schema-
// upgrade events, not runtime configuration.
struct DatasetSpec {
    dataset_id: &'static str,
    display_name: &'static str,
    upstream_doi_or_url: &'static str,
    license: &'static str,
    source_class: &'static str,
    default_path: &'static str,
    fixture_sha256_hex: &'static str,
    /// Output tier directory under `reports/`. For the 20 audit
    /// datasets this is `s_real_1` / `s_real_2` / `s_real_3` matching
    /// the sealed bundle's `reports/s_real_3/bundle_manifest.toml`
    /// `tier_dir` field. For the 10 saturation fixtures this is
    /// `s_real_saturation` (they have no sealed bundle membership; they
    /// emit only when an operator explicitly requests them).
    ///
    /// WHY a field, not a hard-coded default: before S-REAL.3.1.2 the
    /// driver wrote every dataset to `reports/s_real_1/<id>/` regardless
    /// of the dataset's true tier, which silently diverged from the
    /// sealed bundle's on-disk layout. Threading the tier through the
    /// const table makes the emit path match the manifest by
    /// construction; the
    /// `audit_dataset_tier_dir_matches_bundle_manifest` acceptance
    /// test cross-validates the mirror.
    tier_dir: &'static str,
}

/// The 20 sealed S-REAL audit datasets that `--dataset all` enumerates.
///
/// WHY: this is the audit gauntlet's load-bearing identity manifest.
/// Every entry's `dataset_id` and `tier_dir` mirror the
/// `reports/s_real_3/bundle_manifest.toml` entry of the same name; the
/// 60-row bundle hash chain that CI-gates the sealed artifacts depends
/// on this mirror holding. The cross-validation lives in
/// `tests/s_real_audit_dataset_table_invariants.rs`.
const AUDIT_DATASETS: &[DatasetSpec] = &[
    DatasetSpec {
        dataset_id: "tadbench_f11",
        display_name: "TADBench TrainTicket F11",
        upstream_doi_or_url: "10.5281/zenodo.6979726",
        license: "CC-BY-4.0",
        source_class: "DebuggingSoftwareTelemetry",
        default_path: "data/fixtures/tadbench_trainticket_F11.tsv",
        fixture_sha256_hex: "cdebdab01e310d4d02241b694b3771acfa9beca3060b0cee0e19fb237f65dc97",
        tier_dir: "s_real_1",
    },
    DatasetSpec {
        dataset_id: "tadbench_f04",
        display_name: "TADBench TrainTicket F04",
        upstream_doi_or_url: "10.5281/zenodo.6979726",
        license: "CC-BY-4.0",
        source_class: "DebuggingSoftwareTelemetry",
        default_path: "data/fixtures/tadbench_trainticket_F04.tsv",
        fixture_sha256_hex: "b3e7ba260617b4496aa5b09f7d399fecae2cc60dbb2002f6405e65a5f37cb1d7",
        tier_dir: "s_real_2",
    },
    DatasetSpec {
        dataset_id: "tadbench_f11b",
        display_name: "TADBench TrainTicket F11b",
        upstream_doi_or_url: "10.5281/zenodo.6979726",
        license: "CC-BY-4.0",
        source_class: "DebuggingSoftwareTelemetry",
        default_path: "data/fixtures/tadbench_trainticket_F11b.tsv",
        fixture_sha256_hex: "2f45979af791964873a48e6f53a769e0610c02a6452c4a96b7b8bb208d5c492e",
        tier_dir: "s_real_2",
    },
    DatasetSpec {
        dataset_id: "tadbench_f19",
        display_name: "TADBench TrainTicket F19",
        upstream_doi_or_url: "10.5281/zenodo.6979726",
        license: "CC-BY-4.0",
        source_class: "DebuggingSoftwareTelemetry",
        default_path: "data/fixtures/tadbench_trainticket_F19.tsv",
        fixture_sha256_hex: "d412b6e44a83bb583d7f4ea2e5c3405115e686282fd2840db3f640c1dc3dd480",
        tier_dir: "s_real_2",
    },
    DatasetSpec {
        dataset_id: "illinois_socialnet",
        display_name: "Illinois SocialNetwork (DeathStarBench)",
        upstream_doi_or_url: "10.13012/B2IDB-6738796_V1",
        license: "CC0-1.0",
        source_class: "ObservabilityTraces",
        default_path: "data/fixtures/illinois_socialnetwork.tsv",
        fixture_sha256_hex: "c86b5abd1b412f69cccaab9b3e838da742c5ea8a1ba1b6dce634ff3407cc082c",
        tier_dir: "s_real_1",
    },
    DatasetSpec {
        dataset_id: "lo2",
        display_name: "LO2 (Bakhtin et al. 2025)",
        upstream_doi_or_url: "10.5281/zenodo.14257989",
        license: "CC-BY-4.0",
        source_class: "ObservabilityTraces",
        default_path: "data/fixtures/lo2.tsv",
        fixture_sha256_hex: "24d5d7c06d755366b2148a3887151478adfd46062bbec54e9ce20dc358b4647c",
        tier_dir: "s_real_2",
    },
    DatasetSpec {
        dataset_id: "deeptralog",
        display_name: "DeepTraLog F01-02 (Zhang et al. ICSE 2022)",
        upstream_doi_or_url: "github.com/FudanSELab/DeepTraLog",
        license: "no-upstream-license",
        source_class: "ObservabilityTraces",
        default_path: "data/fixtures/deeptralog.tsv",
        fixture_sha256_hex: "90f2625258b4f823e317a331a8b800dbc0c13c4ee3182b81be6c06a9aaef1853",
        tier_dir: "s_real_2",
    },
    DatasetSpec {
        dataset_id: "aiops_kpi",
        display_name: "AIOps Challenge 2018 KPI (Bagel sample)",
        upstream_doi_or_url: "Li, Chen, Pei. IPCCC 2018; github.com/NetManAIOps/Bagel",
        license: "no-upstream-license",
        source_class: "TimeSeriesAnomaly",
        default_path: "data/fixtures/aiops_challenge.tsv",
        fixture_sha256_hex: "be17110ebe6647d00fad79dc1ca69b1b01b22788773202bad6e3322e97b0602e",
        tier_dir: "s_real_1",
    },
    DatasetSpec {
        dataset_id: "multidim_localization",
        display_name: "MultiDim Localization (2019 AIOps Challenge match 2)",
        upstream_doi_or_url: "github.com/NetManAIOps/MultiDimension-Localization",
        license: "no-upstream-license",
        source_class: "TimeSeriesAnomaly",
        default_path: "data/fixtures/multidim_localization.tsv",
        fixture_sha256_hex: "b1237ea1f26e380f6323c5a42f158f5c5ce8901b28d94c7d3aae281747d20926",
        tier_dir: "s_real_2",
    },
    DatasetSpec {
        dataset_id: "defects4j",
        display_name: "Defects4J (Just, Jalali, Ernst)",
        upstream_doi_or_url: "github.com/rjust/defects4j",
        license: "MIT",
        source_class: "SoftwareDefects",
        default_path: "data/fixtures/defects4j.tsv",
        fixture_sha256_hex: "a6fa8c9d8e1fa78e2bb9c9f33e40d02aa6f26a04c0af2f760539793d33875a63",
        tier_dir: "s_real_2",
    },
    DatasetSpec {
        dataset_id: "bugsinpy",
        display_name: "BugsInPy patch-line complexity (Widyasari et al.)",
        upstream_doi_or_url: "github.com/soarsmu/BugsInPy",
        license: "no-upstream-license",
        source_class: "SoftwareDefects",
        default_path: "data/fixtures/bugsinpy.tsv",
        fixture_sha256_hex: "1e9349f3a0e3c76f20681d4623ff0c0ee768943c07a814ef3c7b481021a57f94",
        tier_dir: "s_real_2",
    },
    DatasetSpec {
        dataset_id: "promise_defect_prediction",
        display_name: "PROMISE Ant 1.3 CK metrics (Sayyad Shirabad, Menzies)",
        upstream_doi_or_url: "github.com/ssea-lab/PROMISE",
        license: "no-upstream-license",
        source_class: "SoftwareDefects",
        default_path: "data/fixtures/promise_defect_prediction.tsv",
        fixture_sha256_hex: "14856ef507c9ef8ff6b7120a8c50177d76b492bbff0653b1e635312eadba4461",
        tier_dir: "s_real_2",
    },
    // S-REAL.3 admissions — 5 C-MAPSS sisters + 1 PROMISE Ant 1.4 + 1 DeepTraLog F02 = 20 datasets total.
    DatasetSpec {
        dataset_id: "cmapss_fd001_unit50",
        display_name: "NASA C-MAPSS FD001 unit 50",
        upstream_doi_or_url: "NASA PCoE PHM08",
        license: "Public-Domain",
        source_class: "ReliabilityIndustrial",
        default_path: "data/fixtures/cmapss_fd001_unit50.tsv",
        fixture_sha256_hex: "d4920f9801bf1e27104387702f17e2413194201578ed9434e7a4f1a5b84fe5da",
        tier_dir: "s_real_3",
    },
    DatasetSpec {
        dataset_id: "cmapss_fd002_unit1",
        display_name: "NASA C-MAPSS FD002 unit 1 (multi-condition)",
        upstream_doi_or_url: "NASA PCoE PHM08",
        license: "Public-Domain",
        source_class: "ReliabilityIndustrial",
        default_path: "data/fixtures/cmapss_fd002_unit1.tsv",
        fixture_sha256_hex: "07a94ffc69c54e9838cbf78c2f087e1a247f6971a7e7e75bc47fe6947d6c6c6a",
        tier_dir: "s_real_3",
    },
    DatasetSpec {
        dataset_id: "cmapss_fd002_unit100",
        display_name: "NASA C-MAPSS FD002 unit 100",
        upstream_doi_or_url: "NASA PCoE PHM08",
        license: "Public-Domain",
        source_class: "ReliabilityIndustrial",
        default_path: "data/fixtures/cmapss_fd002_unit100.tsv",
        fixture_sha256_hex: "34310904bd557b0e264d0e8d874695ed007026d05e62ca02745f295efbbbe2b0",
        tier_dir: "s_real_3",
    },
    DatasetSpec {
        dataset_id: "cmapss_fd003_unit1",
        display_name: "NASA C-MAPSS FD003 unit 1 (multi-fault)",
        upstream_doi_or_url: "NASA PCoE PHM08",
        license: "Public-Domain",
        source_class: "ReliabilityIndustrial",
        default_path: "data/fixtures/cmapss_fd003_unit1.tsv",
        fixture_sha256_hex: "8eeb96ec624782e06f5abc4f77dc62f11e714634c6b6e520221432a2be4a87b1",
        tier_dir: "s_real_3",
    },
    DatasetSpec {
        dataset_id: "cmapss_fd004_unit1",
        display_name: "NASA C-MAPSS FD004 unit 1 (multi-condition + multi-fault)",
        upstream_doi_or_url: "NASA PCoE PHM08",
        license: "Public-Domain",
        source_class: "ReliabilityIndustrial",
        default_path: "data/fixtures/cmapss_fd004_unit1.tsv",
        fixture_sha256_hex: "abc69ca301edea69b7d60fe0bc8eac912e991eb80d4ec0fb2f2725167266bdae",
        tier_dir: "s_real_3",
    },
    DatasetSpec {
        dataset_id: "promise_ant_1_4",
        display_name: "PROMISE Apache Ant 1.4 CK metrics",
        upstream_doi_or_url: "github.com/ssea-lab/PROMISE",
        license: "no-upstream-license",
        source_class: "SoftwareDefects",
        default_path: "data/fixtures/promise_ant_1_4.tsv",
        fixture_sha256_hex: "f965a049b98d69ade3391c5f7a777c4ea37089c386cd886b05012cc86748024c",
        tier_dir: "s_real_3",
    },
    DatasetSpec {
        dataset_id: "deeptralog_f02",
        display_name: "DeepTraLog F02-04 ERROR fault period",
        upstream_doi_or_url: "github.com/FudanSELab/DeepTraLog",
        license: "no-upstream-license",
        source_class: "ObservabilityTraces",
        default_path: "data/fixtures/deeptralog_f02.tsv",
        fixture_sha256_hex: "2ce480415b20dc49d09119b79933f2dd5d0ba62c186a8ecc0f2e2bf8b1e1a95f",
        tier_dir: "s_real_3",
    },
    DatasetSpec {
        dataset_id: "cmapss_fd001_unit1",
        display_name: "NASA C-MAPSS FD001 unit 1 (run-to-failure, z-residual)",
        upstream_doi_or_url:
            "Saxena, Goebel, Simon, Eklund (PHM08); NASA PCoE Prognostics Data Repository",
        license: "Public-Domain",
        source_class: "ReliabilityIndustrial",
        default_path: "data/fixtures/cmapss_fd001_unit1.tsv",
        fixture_sha256_hex: "633442bb93f128bb44e82f4b09d0dd0f175933107bc9c0c3e1fc6bd6b040c93e",
        tier_dir: "s_real_2",
    },
];

/// The 10 saturation-class real-data fixtures used by
/// `scripts/s_real_saturation_sweep.sh` (and by explicit single-id
/// audit calls like `s-real-audit --dataset radioml_2018_snr30_large`).
///
/// WHY a separate table from `AUDIT_DATASETS`: these fixtures are 1 M-
/// cell TSV projections (~9 MB each) that `pack_for_colab.sh`
/// deliberately excludes from the slim Colab tarball (~90 MB of
/// saturation TSV data that the audit-gauntlet's 20-dataset replay
/// does NOT need; the saturation sweep is a dev-machine hardware-
/// anchored measurement on RTX 4080 SUPER / CUDA 13.2). Before
/// S-REAL.3.1.2 these lived inside `DATASETS` alongside the 20 audit
/// datasets, so `s-real-audit --dataset all` would attempt to
/// dispatch them on Colab — where the TSVs are intentionally absent
/// — and fail. The split makes the authority surface explicit:
/// `--dataset all` enumerates only `AUDIT_DATASETS`; saturation
/// fixtures dispatch only via explicit single-id or via the
/// saturation-sweep script.
///
/// All entries have `tier_dir = "s_real_saturation"`: they have no
/// sealed bundle membership and emit ad-hoc receipts at
/// `reports/s_real_saturation/<id>/` on the rare occasion an
/// operator dispatches one for diagnostic re-run.
const SATURATION_FIXTURES: &[DatasetSpec] = &[
    // S-PERF.16.a-magnitude large fixture from the RadioML 2018.01
    // SNR=30 dB HDF5 corpus. 1024 entities × 1024 windows =
    // 1,048,576 cells per dispatch — same event magnitude as the
    // S-PERF.16.a saturation bench, but built from REAL RF I/Q
    // bytes via the z-score projection in
    // `data/recipes/radioml_2018_snr30_large.py`.
    //
    // WHY this entry lives in SATURATION_FIXTURES: the throughput bench
    // (scripts/s_real_throughput_bench.sh) measures
    // fixture_bytes / dispatch_median_us. At 1M events / dispatch
    // the dispatcher escapes the launch-overhead regime and
    // surfaces actual sustained throughput, which is impossible
    // to see from the 128-656-event public fixtures sealed at
    // S-REAL.1 / S-REAL.2 / S-REAL.3.
    //
    // Upstream HDF5 is NOT vendored (97 MB; lives on external
    // drive). The projected TSV at default_path IS pinned via the
    // SHA-256 below; the recipe is byte-deterministic so two runs
    // of `radioml_2018_snr30_large.py` against the same HDF5
    // produce the same TSV bytes.
    //
    // License: DeepSig RadioML 2018.01 is distributed under
    // CC-BY-NC-SA-4.0 (research-use; non-commercial). DSFB-GPU
    // does NOT redistribute the upstream HDF5; only the
    // deterministic residual projection of a fixed slice. The
    // audit reports STRUCTURAL residual evidence, NOT a
    // modulation-classification verdict.
    DatasetSpec {
        dataset_id: "radioml_2018_snr30_large",
        display_name: "RadioML 2018.01 SNR=30 dB (1024×1024 z-residual; large-fixture throughput)",
        upstream_doi_or_url:
            "DeepSig RadioML 2018.01 (O'Shea, Corgan); https://www.deepsig.ai/datasets",
        license: "CC-BY-NC-SA-4.0",
        source_class: "RfCommunications",
        default_path: "data/fixtures/radioml_2018_snr30_1024x1024.tsv",
        fixture_sha256_hex: "0a626804be42f113bc62afd2245f86f9ac7c7204472e29fa09faf976ee7f6e86",
        tier_dir: "s_real_saturation",
    },
    // Second S-PERF.16.a-magnitude large-fixture entry. Same shape
    // as the radioml entry (1024×1024 = 1,048,576 cells) but built
    // from REAL DeepBeam I/Q magnitudes via the recipe at
    // `data/recipes/deepbeam_large.py`. Lives here so the saturation
    // sweep (scripts/s_real_saturation_sweep.sh) can compare two
    // independent real-data fixtures against the synthetic
    // S-PERF.16.a saturation median (22.74 GB/s).
    //
    // Same non-claims as the radioml entry: upstream HDF5 is NOT
    // vendored (56 GB; lives on external drive). The projected TSV
    // at default_path IS pinned via the SHA-256 below. DSFB-GPU
    // does NOT classify beam-pattern; it reports STRUCTURAL residual
    // evidence under the deterministic IQ-magnitude + z-score
    // projection. DeepBeam is admitted as a throughput witness, not
    // an RF-domain-truth claim.
    //
    // License: DeepBeam is distributed by the NEU GeneSys Lab for
    // research use. The dataset README does not include an SPDX-
    // style LICENSE token; we record `no-upstream-license` per the
    // research-fair-use convention used for the 7 other S-REAL
    // datasets without explicit LICENSE files (mirror of the
    // S-REAL.2c license-discipline reversal).
    DatasetSpec {
        dataset_id: "deepbeam_large",
        display_name: "DeepBeam (1024×1024 IQ-magnitude z-residual; large-fixture throughput)",
        upstream_doi_or_url:
            "NEU GeneSys Lab DeepBeam (neu_ww72bk394.h5); https://genesys-lab.org/oracle",
        license: "no-upstream-license",
        source_class: "RfCommunications",
        default_path: "data/fixtures/deepbeam_1024x1024.tsv",
        fixture_sha256_hex: "242aae914fc3a88c0a5027536923a72f598062ea0647078f7b0e0024a8aa7929",
        tier_dir: "s_real_saturation",
    },
    // ---- Post-S-REAL.3.1 large-fixture throughput witnesses ----
    //
    // The 7 entries below extend the saturation-class real-data
    // surface from 2 fixtures (radioml + deepbeam) to 9. Each is
    // a 1024×1024 cell (~1M event) residual-projection v2 TSV
    // built from real upstream data via a deterministic recipe
    // under data/recipes/. Used by the saturation sweep
    // (scripts/s_real_saturation_sweep.sh) to surface the
    // saturation / launch-bound boundary across an
    // intentionally-diverse fixture set: 4 RF I/Q sources +
    // 1 mmWave-power source + 3 database-telemetry sources +
    // 1 byte-frequency projection of a DuckDB binary.
    //
    // All are throughput witnesses, NOT domain-truth claims.
    // Same non-claim discipline as radioml_2018_snr30_large /
    // deepbeam_large.
    DatasetSpec {
        dataset_id: "radioml_gold_large",
        display_name: "RadioML 2018.01 GOLD full corpus (1024×1024 IQ-magnitude z-residual)",
        upstream_doi_or_url:
            "DeepSig RadioML 2018.01 GOLD_XYZ_OSC.0001_1024.hdf5; https://www.deepsig.ai/datasets",
        license: "CC-BY-NC-SA-4.0",
        source_class: "RfCommunications",
        default_path: "data/fixtures/radioml_gold_1024x1024.tsv",
        fixture_sha256_hex: "06f156dc662a2ce26c9867b49cbf87e534be92b93bfed7402bfe56971906aeaf",
        tier_dir: "s_real_saturation",
    },
    DatasetSpec {
        dataset_id: "powder_large",
        display_name: "POWDER 4G LTE Band-7 I/Q (Globecom 2020; 1024×1024 z-residual)",
        upstream_doi_or_url: "University of Utah POWDER (Globecom 2020); neu_m046tb444.zip",
        license: "no-upstream-license",
        source_class: "RfCommunications",
        default_path: "data/fixtures/powder_1024x1024.tsv",
        fixture_sha256_hex: "8438f02bc033142797411f5789d8ef6a3f9c214a120ff1aeaabfa3528edb08f2",
        tier_dir: "s_real_saturation",
    },
    DatasetSpec {
        dataset_id: "oracle_large",
        display_name: "ORACLE WiFi 802.11a fingerprinting (Sankhe et al. INFOCOM 2019; 1024×1024)",
        upstream_doi_or_url:
            "NEU/KRI 16-Device ORACLE dataset (Sankhe et al. INFOCOM 2019); neu_m044q5210.zip",
        license: "no-upstream-license",
        source_class: "RfCommunications",
        default_path: "data/fixtures/oracle_1024x1024.tsv",
        fixture_sha256_hex: "6ada4586de505fdc67f2be053b24f92732acc32d3bc8154d17a27b52db570133",
        tier_dir: "s_real_saturation",
    },
    DatasetSpec {
        dataset_id: "deepsense6g_large",
        display_name: "Deepsense6G Scenario 23 mmWave-power (512×1024 z-residual; sub-saturation)",
        upstream_doi_or_url:
            "Deepsense6G Scenario 23 (Alkhateeb et al. 2022); https://deepsense6g.net/scenario-23/",
        license: "no-upstream-license",
        source_class: "RfCommunications",
        default_path: "data/fixtures/deepsense6g_512x1024.tsv",
        fixture_sha256_hex: "4d71dc2c087697f1c1455d78f926d11870149828786bbfca865b277afab6052e",
        tier_dir: "s_real_saturation",
    },
    DatasetSpec {
        dataset_id: "imdb_tgz_large",
        display_name: "IMDB Join-Order-Benchmark cast_info.csv (1020×1024 numeric-ID z-residual)",
        upstream_doi_or_url:
            "IMDB Join-Order-Benchmark (Leis et al. VLDB 2015); imdb.tgz cast_info.csv",
        license: "no-upstream-license",
        source_class: "DatabaseWorkload",
        default_path: "data/fixtures/imdb_tgz_1020x1024.tsv",
        fixture_sha256_hex: "9e0d356f9a706f6132461873a5903df123e046326e67181497413ff122234aa6",
        tier_dir: "s_real_saturation",
    },
    DatasetSpec {
        dataset_id: "imdb_duckdb_large",
        display_name: "IMDB DuckDB binary byte-frequency residual (1024×1024; byte-projection)",
        upstream_doi_or_url: "IMDB Join-Order-Benchmark DuckDB binary dump (sister of imdb.tgz)",
        license: "no-upstream-license",
        source_class: "DatabaseWorkload",
        default_path: "data/fixtures/imdb_duckdb_1024x1024.tsv",
        fixture_sha256_hex: "2970fbd9d0d6bdb7b98b461d549dff9a77ae4b0a94a4fc2967acbb7eec3e12e7",
        tier_dir: "s_real_saturation",
    },
    DatasetSpec {
        dataset_id: "snowset_large",
        display_name: "Snowset Snowflake-telemetry CSV (Vuppalapati et al. NSDI 2020; 1024×1024)",
        upstream_doi_or_url:
            "Snowset (Vuppalapati et al. NSDI 2020); github.com/resource-disaggregation/snowset",
        license: "no-upstream-license",
        source_class: "DatabaseWorkload",
        default_path: "data/fixtures/snowset_1024x1024.tsv",
        fixture_sha256_hex: "0ec2b78c1fc4db066208968ab6fc452a4ac6dad31dc740b0870a77a3e73fc3c8",
        tier_dir: "s_real_saturation",
    },
    DatasetSpec {
        dataset_id: "sqlshare_large",
        display_name: "SQLShare 2015 oceanographic CSV residual (Jain et al. UW 2015; 1024×1024)",
        upstream_doi_or_url:
            "SQLShare 2015 (Jain et al., U.Washington); uwescience.github.io/sqlshare",
        license: "no-upstream-license",
        source_class: "DatabaseWorkload",
        default_path: "data/fixtures/sqlshare_1024x1024.tsv",
        fixture_sha256_hex: "e663b53ccdcc1c2bad808a7268af10c9b638e05f108a371707c3e1ac924c37c8",
        tier_dir: "s_real_saturation",
    },
];

/// Convenience iterator over every dataset entry in both tables, in
/// the canonical (audit-first, saturation-second, table-order-within)
/// sequence. Useful for cross-table invariants (SHA-pin sanity,
/// duplicate-id check) without re-deriving the union on every caller.
fn all_dataset_specs() -> impl Iterator<Item = &'static DatasetSpec> {
    AUDIT_DATASETS.iter().chain(SATURATION_FIXTURES.iter())
}

/// Resolve a dataset_id against BOTH the audit-gauntlet and
/// saturation-fixture tables.
///
/// WHY both: single-id audit (e.g. `s-real-audit --dataset
/// radioml_2018_snr30_large`) is still admissible as a diagnostic
/// re-run path. Only `--dataset all` enforces the audit-only
/// AUDIT_DATASETS set; an operator who explicitly names a saturation
/// fixture knows what they are asking for.
fn lookup(id: &str) -> Option<&'static DatasetSpec> {
    all_dataset_specs().find(|d| d.dataset_id == id)
}

// =====================================================================
// CLI entry point.
// =====================================================================

/// Run the S-REAL audit gauntlet on one dataset, on every audit
/// dataset, or on an explicitly-named saturation fixture.
///
/// Flags:
/// - `--dataset <id|all>` — a canonical dataset id (e.g. `tadbench_f11`,
///   `illinois_socialnet`, `aiops_kpi`, `cmapss_fd001_unit50`,
///   `deeptralog`, `multidim_localization`, ...) or `all`. Required.
///   `all` enumerates the 20 sealed audit datasets in `AUDIT_DATASETS`;
///   the 10 saturation fixtures in `SATURATION_FIXTURES` are NEVER
///   selected by `all` (they dispatch only via explicit single-id audit
///   calls and via `scripts/s_real_saturation_sweep.sh`).
/// - `--out-dir <path>` — root output directory. Defaults to `reports`.
///   Each dataset writes to `<out-dir>/<tier_dir>/<dataset_id>/` where
///   `tier_dir` is `s_real_1` / `s_real_2` / `s_real_3` per the sealed
///   bundle's `reports/s_real_3/bundle_manifest.toml` layout (or
///   `s_real_saturation` for an explicit single-id saturation run).
///   Each dataset produces 9 panel-locked artifacts there
///   (audit_report.html / casefile.json / dataset_manifest.toml /
///   episodes.jsonl / limitations.md / perf_profile.txt /
///   replay_verification.txt / run_receipt.txt / schema_map.toml).
///
/// Exit codes (mirrors the existing `run-gpu` convention so downstream
/// automation handles the result uniformly):
/// - 0 — success on every selected dataset.
/// - 1 — CLI usage error.
/// - 2 — GPU unavailable (`--features cuda` not built) or kernel failed.
/// - 5 — I/O failure (read fixture, write artifact).
/// - 6 — fixture SHA-256 mismatch (pinned hash divergence).
/// - 7 — replay-verification failure (run 1 != run 2 bytes).
pub fn parse_and_run(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args) {
        Ok(f) => f,
        Err(msg) => return usage_error(&msg),
    };
    let dataset_arg = if let Some(s) = flags.get("dataset") {
        s.clone()
    } else {
        // Build the valid-id enumeration deterministically from both
        // tables so adding a new dataset never leaves the error
        // message stale. AUDIT_DATASETS first, then SATURATION_FIXTURES,
        // then `all`.
        let ids: Vec<&str> = all_dataset_specs().map(|d| d.dataset_id).collect();
        let menu = format!("{}|all", ids.join("|"));
        return usage_error(&format!("missing required flag --dataset ({menu})"));
    };
    let default_out = "reports".to_string();
    let out_dir = flags.get("out-dir").cloned().unwrap_or(default_out);

    // S-REAL.PERF flags. --iters >= 2 (replay verification needs at
    // least run 1 + run 2). --catalogs >= 1.
    let iters: u32 = flags
        .get("iters")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2)
        .max(2);
    let catalogs: u32 = flags
        .get("catalogs")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1);

    let selected: Vec<&'static DatasetSpec> = if dataset_arg == "all" {
        // `--dataset all` enumerates ONLY the 20 sealed audit datasets.
        // Saturation fixtures dispatch only via explicit single-id or
        // via the saturation-sweep script; this protects the Colab
        // public-replay path from accidentally trying to read a
        // 1M-cell TSV that the slim tarball deliberately excludes.
        AUDIT_DATASETS.iter().collect()
    } else if let Some(d) = lookup(&dataset_arg) {
        vec![d]
    } else {
        let ids: Vec<&str> = all_dataset_specs().map(|d| d.dataset_id).collect();
        let menu = format!("{}, all", ids.join(", "));
        return usage_error(&format!(
            "unknown dataset id {dataset_arg:?}; valid values: {menu}"
        ));
    };

    for spec in selected {
        // Output path layout matches the sealed bundle's `tier_dir`
        // by construction: `<out_dir>/<tier_dir>/<dataset_id>/`. The
        // 20 audit datasets land in `reports/s_real_1/...`,
        // `reports/s_real_2/...`, `reports/s_real_3/...`; the 10
        // saturation fixtures land in `reports/s_real_saturation/...`
        // when invoked explicitly.
        let dataset_dir = PathBuf::from(&out_dir)
            .join(spec.tier_dir)
            .join(spec.dataset_id);
        match run_one_dataset(spec, &dataset_dir, iters, catalogs) {
            Ok(()) => eprintln!(
                "dsfb-gpu-debug s-real-audit: {} sealed at {} (iters={iters}, catalogs={catalogs})",
                spec.dataset_id,
                dataset_dir.display(),
            ),
            Err(code) => return code,
        }
    }
    ExitCode::SUCCESS
}

/// S-REAL.PERF per-dataset performance profile.
///
/// WHY: S-REAL.1 / .1.1 / .1.1.1 proved DSFB-GPU processes real
/// datasets deterministically and emits human-readable artifacts.
/// S-REAL.PERF answers the next honest question: *how long does that
/// take, and what dominates the wall?* Every timing is host-Instant
/// wall-clock microseconds; cudaEvent kernel-level timing remains
/// S-PERF territory. The profile is **runtime-dependent by design**
/// — the admission disclosure makes that explicit so an operator does
/// not conflate timing-replay with byte-replay.
#[derive(Clone, Debug, Default)]
pub struct PerformanceProfile {
    pub iters: u32,
    pub catalogs: u32,
    pub ingest_us: u64,
    pub lowering_us: u64,
    pub contract_setup_us: u64,
    pub cuda_dispatch_run1_us: u64,
    pub cuda_dispatch_run2_us: u64,
    pub cuda_dispatch_extra_us: Vec<u64>,
    pub casefile_emit_us: u64,
    pub episodes_jsonl_emit_us: u64,
    pub audit_report_emit_us: u64,
    pub total_us: u64,
    pub events_emitted: u32,
    pub finite_cells: u32,
    pub fixture_byte_size: u64,
    /// Sequential single-catalog total wall when --catalogs > 1.
    /// HONEST LABEL: NOT a batched dispatch; this is K sequential
    /// build_gpu calls on the same (events, contract). Reported so
    /// the operator can see launch-overhead amortization without
    /// being misled into thinking the dispatcher supports true
    /// K-batched mode.
    pub catalogs_total_us: u64,
}

impl PerformanceProfile {
    fn dispatch_samples_sorted(&self) -> Vec<u64> {
        let mut v = Vec::with_capacity(2 + self.cuda_dispatch_extra_us.len());
        v.push(self.cuda_dispatch_run1_us);
        v.push(self.cuda_dispatch_run2_us);
        v.extend(self.cuda_dispatch_extra_us.iter().copied());
        v.sort_unstable();
        v
    }

    fn dispatch_median_us(&self) -> u64 {
        let s = self.dispatch_samples_sorted();
        if s.is_empty() {
            0
        } else {
            s[s.len() / 2]
        }
    }

    fn percentile_us(&self, p: u32) -> u64 {
        let s = self.dispatch_samples_sorted();
        if s.is_empty() {
            return 0;
        }
        let idx = (((s.len() as u64).saturating_sub(1)) * u64::from(p) / 100) as usize;
        s[idx]
    }

    fn events_per_second(&self) -> u64 {
        if self.total_us == 0 {
            0
        } else {
            (u64::from(self.events_emitted) * 1_000_000) / self.total_us
        }
    }

    fn finite_cells_per_second(&self) -> u64 {
        if self.total_us == 0 {
            0
        } else {
            (u64::from(self.finite_cells) * 1_000_000) / self.total_us
        }
    }

    fn logical_bytes_per_second(&self) -> u64 {
        if self.total_us == 0 {
            0
        } else {
            (self.fixture_byte_size * 1_000_000) / self.total_us
        }
    }
}

// =====================================================================
// Single-dataset run.
// =====================================================================

#[allow(
    clippy::too_many_lines,
    reason = "End-to-end S-REAL driver per panel-locked design: ingest \
              + lower + two-run dispatch + replay-verify + 9-artifact \
              panel-locked emit must live in one function so the audit's \
              load-bearing steps are visible top-to-bottom."
)]
fn run_one_dataset(
    spec: &'static DatasetSpec,
    out_dir: &Path,
    iters: u32,
    catalogs: u32,
) -> Result<(), ExitCode> {
    let t_total_start = Instant::now();

    // 1. Read fixture bytes + 2. Parse + verify SHA-256 byte-pin.
    let t_ingest_start = Instant::now();
    let bytes = fs::read(spec.default_path).map_err(|e| {
        eprintln!(
            "dsfb-gpu-debug s-real-1-audit: failed to read {}: {e}",
            spec.default_path
        );
        ExitCode::from(5)
    })?;
    let fixture_byte_size = bytes.len() as u64;
    let fixture = load_residual_projection_tsv(&bytes, spec.fixture_sha256_hex).map_err(|e| {
        eprintln!("dsfb-gpu-debug s-real-1-audit: ingest error: {e}");
        ExitCode::from(6)
    })?;
    let ingest_us = t_ingest_start.elapsed().as_micros() as u64;

    // 3. Deterministic lowering: cells → TraceEvents.
    let t_lowering_start = Instant::now();
    let lowering = LoweringConfig::default();
    let events = lower_to_trace_events(&fixture, &lowering);
    let ingest_report = build_ingest_report(&fixture, &events, fixture_byte_size);
    let lowering_us = t_lowering_start.elapsed().as_micros() as u64;

    // 4. Build scaled contract with bank + registry pins.
    let t_contract_start = Instant::now();
    let n_entities = ingest_report.observed_num_signals.max(1);
    let n_windows = ingest_report.observed_num_windows.max(1);
    let mut contract = Contract::scaled(n_entities, n_windows);
    contract.pin_bank_hash(bank_hash());
    contract.pin_detector_registry_hash(registry_hash());
    let contract_setup_us = t_contract_start.elapsed().as_micros() as u64;

    // 5. Run dispatcher TWICE for replay verification (iter-1 +
    //    iter-2), plus optional extra iters for variance recording.
    let t_run1 = Instant::now();
    let case_run1 = run_gpu_or_emit(&events, &contract)?;
    let cuda_dispatch_run1_us = t_run1.elapsed().as_micros() as u64;

    let t_run2 = Instant::now();
    let case_run2 = run_gpu_or_emit(&events, &contract)?;
    let cuda_dispatch_run2_us = t_run2.elapsed().as_micros() as u64;

    let mut cuda_dispatch_extra_us: Vec<u64> = Vec::new();
    for _ in 2..iters {
        let t = Instant::now();
        let _ = run_gpu_or_emit(&events, &contract)?;
        cuda_dispatch_extra_us.push(t.elapsed().as_micros() as u64);
    }

    // 5b. Sequential single-catalog amortization mode (--catalogs K).
    //     HONEST LABEL: K sequential build_gpu calls; not a true
    //     batched dispatch. Reports aggregate wall so an operator can
    //     see launch-overhead amortization vs the per-call median.
    let catalogs_total_us = if catalogs > 1 {
        let t = Instant::now();
        for _ in 0..catalogs {
            let _ = run_gpu_or_emit(&events, &contract)?;
        }
        t.elapsed().as_micros() as u64
    } else {
        0
    };

    // 6. Serialize canonical bytes for both runs and compute per-artifact
    //    SHA-256 receipts.
    let t_casefile_emit_start = Instant::now();
    let casefile_run1 = emit(&case_run1);
    let casefile_run2 = emit(&case_run2);
    let casefile_emit_us = t_casefile_emit_start.elapsed().as_micros() as u64;

    let t_episodes_emit_start = Instant::now();
    let episodes_run1 = serialize_episodes_jsonl(&case_run1.episodes);
    let episodes_run2 = serialize_episodes_jsonl(&case_run2.episodes);
    let episodes_jsonl_emit_us = t_episodes_emit_start.elapsed().as_micros() as u64;

    let casefile_run1_hex = sha256_to_hex_lower(&sha256(&casefile_run1));
    let casefile_run2_hex = sha256_to_hex_lower(&sha256(&casefile_run2));
    let episodes_run1_hex = sha256_to_hex_lower(&sha256(&episodes_run1));
    let episodes_run2_hex = sha256_to_hex_lower(&sha256(&episodes_run2));

    // 7. Build the audit-report inputs and render the HTML twice (the
    //    replay law applies to audit_report.html too).
    let manifest = DatasetManifest {
        dataset_id: spec.dataset_id.to_string(),
        display_name: spec.display_name.to_string(),
        upstream_doi_or_url: spec.upstream_doi_or_url.to_string(),
        license: spec.license.to_string(),
        source_class: spec.source_class.to_string(),
        vendored_path: spec.default_path.to_string(),
        fixture_sha256_hex: spec.fixture_sha256_hex.to_string(),
        fixture_byte_size,
    };
    let mut schema = SchemaMap::from(&ingest_report);
    schema.declared_healthy_window_end = fixture.declared_healthy_window_end;
    schema.lowering_config = lowering;

    // Toolchain identity is recorded deterministically: rustc version
    // pulled from CARGO_PKG_RUST_VERSION via env! at compile time (not
    // available; we record the cargo version instead) and CUDA / GPU
    // identity from canonical S-PERF.16.a panel-locked values. A real
    // operator can override these at audit-emit time in a future S-REAL.1.1
    // commit; for now they are pinned constants so two runs in the same
    // process produce byte-identical toolchain blocks by construction.
    let mut toolchain = BTreeMap::new();
    toolchain.insert(
        "dsfb_gpu_debug_demo_version".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    toolchain.insert("cuda_version".to_string(), "13.2".to_string());
    toolchain.insert("gpu_name".to_string(), "RTX 4080 SUPER".to_string());
    toolchain.insert("backend".to_string(), case_run1.backend.to_string());

    let replay_pre = ReplayVerification {
        run_count: 2,
        casefile_json_sha256_run1: casefile_run1_hex.clone(),
        casefile_json_sha256_run2: casefile_run2_hex.clone(),
        episodes_jsonl_sha256_run1: episodes_run1_hex.clone(),
        episodes_jsonl_sha256_run2: episodes_run2_hex.clone(),
        final_case_file_hash_run1_hex: sha256_to_hex_lower(&case_run1.final_case_file_hash),
        final_case_file_hash_run2_hex: sha256_to_hex_lower(&case_run2.final_case_file_hash),
        episode_count_run1: case_run1.episodes.len() as u32,
        episode_count_run2: case_run2.episodes.len() as u32,
        toolchain,
    };

    // Render the HTML against case_run1; verify the second render is
    // byte-identical (this is what the acceptance test will also check
    // independently).
    let t_audit_emit_start = Instant::now();
    let html_run1 = render_audit_report_html(&manifest, &schema, &case_run1, &replay_pre);
    let html_run2 = render_audit_report_html(&manifest, &schema, &case_run2, &replay_pre);
    let audit_report_emit_us = t_audit_emit_start.elapsed().as_micros() as u64;
    let html_run1_hex = sha256_to_hex_lower(&sha256(html_run1.as_bytes()));
    let html_run2_hex = sha256_to_hex_lower(&sha256(html_run2.as_bytes()));

    // Assemble the performance profile from the timing samples gathered above.
    let total_us = t_total_start.elapsed().as_micros() as u64;
    let perf = PerformanceProfile {
        iters,
        catalogs,
        ingest_us,
        lowering_us,
        contract_setup_us,
        cuda_dispatch_run1_us,
        cuda_dispatch_run2_us,
        cuda_dispatch_extra_us,
        casefile_emit_us,
        episodes_jsonl_emit_us,
        audit_report_emit_us,
        total_us,
        events_emitted: ingest_report.emitted_event_count,
        finite_cells: ingest_report.finite_cell_count,
        fixture_byte_size,
        catalogs_total_us,
    };

    // The audit's load-bearing replay-admission gate: every artifact must
    // be byte-identical across the two runs. Exit code 7 surfaces a
    // replay failure so downstream CI can react distinctly from a CUDA
    // failure.
    let admits = casefile_run1_hex == casefile_run2_hex
        && episodes_run1_hex == episodes_run2_hex
        && html_run1_hex == html_run2_hex;

    // 8. Emit 9 panel-locked artifacts in canonical order
    //    (audit_report.html, casefile.json, dataset_manifest.toml,
    //    episodes.jsonl, limitations.md, perf_profile.txt,
    //    replay_verification.txt, run_receipt.txt, schema_map.toml).
    //    Three of these — casefile.json, dataset_manifest.toml,
    //    episodes.jsonl — are chain-pinned under
    //    reports/s_real_3/bundle_hash_chain.txt; the other six are
    //    receipt-only (regenerated by every audit run).
    //
    //    HONEST NAMING: the emit is sequential `fs::create_dir_all` +
    //    `fs::write`, not filesystem-atomic. A partial write on
    //    disk-full would leave a half-populated directory. "Panel-
    //    locked" describes the per-artifact byte content (deterministic
    //    per the SHA-pinned input + the static driver source), not
    //    filesystem-atomic emit. A future S-REAL.4 hardening pass may
    //    add staging→verify→rename atomic-swap discipline; for
    //    research-court purposes the byte-equivalence test
    //    (s_real_1_replay_byte_identity) is the load-bearing gate.
    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!(
            "dsfb-gpu-debug s-real-1-audit: could not create {}: {e}",
            out_dir.display()
        );
        return Err(ExitCode::from(5));
    }

    let write = |name: &str, content: &[u8]| -> Result<(), ExitCode> {
        let path = out_dir.join(name);
        fs::write(&path, content).map_err(|e| {
            eprintln!(
                "dsfb-gpu-debug s-real-1-audit: failed to write {}: {e}",
                path.display()
            );
            ExitCode::from(5)
        })
    };

    write(
        "dataset_manifest.toml",
        emit_dataset_manifest_toml(&manifest).as_bytes(),
    )?;
    write("schema_map.toml", emit_schema_map_toml(&schema).as_bytes())?;
    write(
        "run_receipt.txt",
        emit_run_receipt_txt(spec, &manifest, &schema, &case_run1).as_bytes(),
    )?;
    write("casefile.json", &casefile_run1)?;
    write("episodes.jsonl", &episodes_run1)?;
    write("audit_report.html", html_run1.as_bytes())?;
    write(
        "replay_verification.txt",
        emit_replay_verification_txt(spec, &replay_pre, &html_run1_hex, &html_run2_hex, admits)
            .as_bytes(),
    )?;
    write("limitations.md", emit_limitations_md(spec).as_bytes())?;
    // S-REAL.PERF: ninth artifact. Runtime-dependent timing block; not
    // checked for byte-identity across re-invocations (timing varies).
    write(
        "perf_profile.txt",
        emit_perf_profile_txt(spec, &perf).as_bytes(),
    )?;

    if !admits {
        eprintln!(
            "dsfb-gpu-debug s-real-1-audit: replay verification FAILED for {} (artifacts emitted; see replay_verification.txt)",
            spec.dataset_id
        );
        return Err(ExitCode::from(7));
    }
    Ok(())
}

// =====================================================================
// Dispatch wrapper that surfaces honest exit codes.
// =====================================================================

fn run_gpu_or_emit(
    events: &[dsfb_gpu_debug_core::event::TraceEvent],
    contract: &Contract,
) -> Result<CaseFile, ExitCode> {
    use dsfb_gpu_debug_cuda::{build_gpu, GpuError};
    match build_gpu(events, contract) {
        Ok(case) => Ok(case),
        Err(GpuError::CudaUnavailable) => {
            eprintln!(
                "dsfb-gpu-debug s-real-1-audit: GPU pipeline unavailable \
                 (built without --features cuda)"
            );
            Err(ExitCode::from(2))
        }
        Err(GpuError::KernelFailed(code)) => {
            eprintln!("dsfb-gpu-debug s-real-1-audit: GPU kernel failed with cuda status {code}");
            Err(ExitCode::from(2))
        }
        Err(GpuError::InvalidInput(msg)) => {
            eprintln!("dsfb-gpu-debug s-real-1-audit: GPU dispatcher rejected input: {msg}");
            Err(ExitCode::from(2))
        }
    }
}

// =====================================================================
// Artifact serializers.
// =====================================================================

/// JSONL serialization of admitted episodes.
///
/// WHY: One line per admitted episode. The episodes are pre-sorted by
/// `(entity_id, start_window, end_window, reason_code as u8)` so two
/// consecutive calls on the same `case.episodes` produce byte-identical
/// output regardless of insertion order. Q16 fields render as the raw
/// signed integer from `.0` (no decimal scaling) so the receipt carries
/// the exact bytes the bank stage saw, not a lossy display form.
///
/// Key order is fixed and matches the audit_report table column order.
/// No whitespace beyond the trailing newline per line.
#[must_use]
pub fn serialize_episodes_jsonl(episodes: &[Episode]) -> Vec<u8> {
    let mut sorted: Vec<&Episode> = episodes.iter().collect();
    sorted.sort_by_key(|e| (e.entity_id, e.start_window, e.end_window, e.reason as u8));
    let mut buf: Vec<u8> = Vec::new();
    for (idx, e) in sorted.iter().enumerate() {
        // Hand-roll JSON to avoid serde and to keep the key order pinned.
        let _ = writeln!(
            &mut buf as &mut dyn std::io::Write,
            "{{\"idx\":{},\"entity_id\":{},\"start_window\":{},\"end_window\":{},\
             \"motif\":\"{}\",\"reason\":\"{}\",\"peak_state\":\"{}\",\
             \"peak_residual_q\":{},\"peak_drift_q\":{},\"peak_slew_q\":{},\
             \"detector_bit_count\":{}}}",
            idx,
            e.entity_id,
            e.start_window,
            e.end_window,
            motif_name(e.motif),
            reason_name(e.reason),
            grammar_name(e.peak_state),
            e.peak_residual_q.0,
            e.peak_drift_q.0,
            e.peak_slew_q.0,
            e.detector_bit_count,
        );
    }
    buf
}

fn motif_name(m: dsfb_gpu_debug_core::bank::BankMotif) -> &'static str {
    use dsfb_gpu_debug_core::bank::BankMotif;
    match m {
        BankMotif::LatencyRamp => "LatencyRamp",
        BankMotif::ErrorBurst => "ErrorBurst",
        BankMotif::SlewShockRecovery => "SlewShockRecovery",
        BankMotif::SustainedDegradation => "SustainedDegradation",
        BankMotif::OscillationInstability => "OscillationInstability",
        BankMotif::LocalizedRouteFault => "LocalizedRouteFault",
        BankMotif::FanoutCascadeCandidate => "FanoutCascadeCandidate",
        BankMotif::ConfuserTransient => "ConfuserTransient",
    }
}

fn reason_name(r: dsfb_gpu_debug_core::grammar::ReasonCode) -> &'static str {
    use dsfb_gpu_debug_core::grammar::ReasonCode;
    match r {
        ReasonCode::Admissible => "Admissible",
        ReasonCode::BoundaryApproach => "BoundaryApproach",
        ReasonCode::SustainedOutwardDrift => "SustainedOutwardDrift",
        ReasonCode::AbruptSlewViolation => "AbruptSlewViolation",
        ReasonCode::RecurrentBoundaryGrazing => "RecurrentBoundaryGrazing",
        ReasonCode::EnvelopeViolation => "EnvelopeViolation",
        ReasonCode::DriftWithRecovery => "DriftWithRecovery",
        ReasonCode::SingleCrossing => "SingleCrossing",
    }
}

fn grammar_name(g: dsfb_gpu_debug_core::grammar::GrammarState) -> &'static str {
    use dsfb_gpu_debug_core::grammar::GrammarState;
    match g {
        GrammarState::Admissible => "Admissible",
        GrammarState::Boundary => "Boundary",
        GrammarState::Violation => "Violation",
        GrammarState::Recovery => "Recovery",
    }
}

fn emit_dataset_manifest_toml(m: &DatasetManifest) -> String {
    let mut s = String::new();
    s.push_str("# S-REAL.1 dataset manifest. Provenance record co-pinning\n");
    s.push_str("# upstream identity, license, vendored bytes path, and\n");
    s.push_str("# SHA-256 byte-pin of the file the audit actually read.\n\n");
    s.push_str("[dataset]\n");
    let _ = writeln!(&mut s, "dataset_id          = \"{}\"", m.dataset_id);
    let _ = writeln!(&mut s, "display_name        = \"{}\"", m.display_name);
    let _ = writeln!(
        &mut s,
        "upstream_doi_or_url = \"{}\"",
        m.upstream_doi_or_url
    );
    let _ = writeln!(&mut s, "license             = \"{}\"", m.license);
    let _ = writeln!(&mut s, "source_class        = \"{}\"", m.source_class);
    let _ = writeln!(&mut s, "vendored_path       = \"{}\"", m.vendored_path);
    s.push('\n');
    s.push_str("[fixture]\n");
    let _ = writeln!(&mut s, "sha256_hex          = \"{}\"", m.fixture_sha256_hex);
    let _ = writeln!(&mut s, "byte_size           = {}", m.fixture_byte_size);
    s
}

fn emit_schema_map_toml(schema: &SchemaMap) -> String {
    let mut s = String::new();
    s.push_str("# S-REAL.1 schema map. Records the upstream-declared shape,\n");
    s.push_str("# the observed shape after parsing, and the deterministic\n");
    s.push_str("# event-lowering rule used to project cells into TraceEvents.\n\n");
    s.push_str("[upstream_declared]\n");
    let _ = writeln!(
        &mut s,
        "num_windows         = {}",
        schema.declared_num_windows
    );
    let _ = writeln!(
        &mut s,
        "num_signals         = {}",
        schema.declared_num_signals
    );
    let _ = writeln!(
        &mut s,
        "healthy_window_end  = {}",
        schema.declared_healthy_window_end
    );
    s.push('\n');
    s.push_str("[observed]\n");
    let _ = writeln!(
        &mut s,
        "num_windows         = {}",
        schema.observed_num_windows
    );
    let _ = writeln!(
        &mut s,
        "num_signals         = {}",
        schema.observed_num_signals
    );
    let _ = writeln!(&mut s, "nan_cell_count      = {}", schema.nan_cell_count);
    let _ = writeln!(&mut s, "finite_cell_count   = {}", schema.finite_cell_count);
    s.push('\n');
    s.push_str("[event_lowering]\n");
    let _ = writeln!(
        &mut s,
        "value_to_microsecond_scale = {}",
        schema.lowering_config.value_to_microsecond_scale
    );
    let _ = writeln!(
        &mut s,
        "latency_clamp_us           = {}",
        schema.lowering_config.latency_clamp_us
    );
    let _ = writeln!(
        &mut s,
        "window_size_ns             = {}",
        schema.lowering_config.window_size_ns
    );
    s.push('\n');
    s.push_str("[output]\n");
    let _ = writeln!(
        &mut s,
        "emitted_event_count = {}",
        schema.emitted_event_count
    );
    s
}

#[allow(
    clippy::too_many_lines,
    reason = "Receipt emitter is a single byte-stable text block; \
              splitting risks ordering divergence between two builds."
)]
fn emit_run_receipt_txt(
    _spec: &DatasetSpec,
    manifest: &DatasetManifest,
    schema: &SchemaMap,
    case: &CaseFile,
) -> String {
    let mut s = String::new();
    s.push_str("=== S-REAL.1 run receipt ===\n");
    let _ = writeln!(&mut s, "dataset:                 {}", manifest.dataset_id);
    let _ = writeln!(&mut s, "display_name:            {}", manifest.display_name);
    let _ = writeln!(&mut s, "license:                 {}", manifest.license);
    let _ = writeln!(
        &mut s,
        "upstream_doi_or_url:     {}",
        manifest.upstream_doi_or_url
    );
    s.push('\n');
    s.push_str("Input\n");
    let _ = writeln!(
        &mut s,
        "  vendored_path:         {}",
        manifest.vendored_path
    );
    let _ = writeln!(
        &mut s,
        "  fixture_sha256:        {}",
        manifest.fixture_sha256_hex
    );
    let _ = writeln!(
        &mut s,
        "  fixture_byte_size:     {}",
        manifest.fixture_byte_size
    );
    s.push('\n');
    s.push_str("Lowering\n");
    let _ = writeln!(
        &mut s,
        "  value_to_microsecond_scale: {}",
        schema.lowering_config.value_to_microsecond_scale
    );
    let _ = writeln!(
        &mut s,
        "  latency_clamp_us:           {}",
        schema.lowering_config.latency_clamp_us
    );
    let _ = writeln!(
        &mut s,
        "  window_size_ns:             {}",
        schema.lowering_config.window_size_ns
    );
    let _ = writeln!(
        &mut s,
        "  finite_cells:               {}",
        schema.finite_cell_count
    );
    let _ = writeln!(
        &mut s,
        "  nan_cells_skipped:          {}",
        schema.nan_cell_count
    );
    let _ = writeln!(
        &mut s,
        "  events_emitted:             {}",
        schema.emitted_event_count
    );
    s.push('\n');
    s.push_str("Run\n");
    let _ = writeln!(&mut s, "  backend:               {}", case.backend);
    let _ = writeln!(
        &mut s,
        "  n_entities (= observed_num_signals): {}",
        schema.observed_num_signals
    );
    let _ = writeln!(
        &mut s,
        "  n_windows  (= observed_num_windows): {}",
        schema.observed_num_windows
    );
    let _ = writeln!(
        &mut s,
        "  contract_hash:         sha256:{}",
        sha256_to_hex_lower(&case.hashes.contract)
    );
    let _ = writeln!(
        &mut s,
        "  bank_hash:             sha256:{}",
        sha256_to_hex_lower(&case.hashes.bank)
    );
    let _ = writeln!(
        &mut s,
        "  detector_registry_hash: sha256:{}",
        sha256_to_hex_lower(&case.hashes.detector_registry)
    );
    s.push('\n');
    s.push_str("Result\n");
    let _ = writeln!(&mut s, "  episodes_admitted:     {}", case.episodes.len());
    let _ = writeln!(
        &mut s,
        "  final_verdict:         {}",
        case.final_verdict.name()
    );
    let _ = writeln!(
        &mut s,
        "  final_case_file_hash:  sha256:{}",
        sha256_to_hex_lower(&case.final_case_file_hash)
    );
    s
}

fn emit_replay_verification_txt(
    spec: &DatasetSpec,
    r: &ReplayVerification,
    html_run1_hex: &str,
    html_run2_hex: &str,
    admits: bool,
) -> String {
    let mut s = String::new();
    s.push_str("=== S-REAL.1 replay verification ===\n");
    let _ = writeln!(&mut s, "dataset: {}", spec.dataset_id);
    let _ = writeln!(&mut s, "runs:    {}", r.run_count);
    s.push('\n');
    let cf_ok = r.casefile_json_sha256_run1 == r.casefile_json_sha256_run2;
    let ep_ok = r.episodes_jsonl_sha256_run1 == r.episodes_jsonl_sha256_run2;
    let hr_ok = html_run1_hex == html_run2_hex;
    let _ = writeln!(
        &mut s,
        "byte-identical replay: {}",
        if admits { "YES" } else { "NO" }
    );
    let _ = writeln!(
        &mut s,
        "  casefile.json:        {}",
        if cf_ok { "YES" } else { "NO" }
    );
    let _ = writeln!(
        &mut s,
        "  episodes.jsonl:       {}",
        if ep_ok { "YES" } else { "NO" }
    );
    let _ = writeln!(
        &mut s,
        "  audit_report.html:    {}",
        if hr_ok { "YES" } else { "NO" }
    );
    s.push('\n');
    s.push_str("Run 1 SHA-256\n");
    let _ = writeln!(
        &mut s,
        "  casefile.json:        {}",
        r.casefile_json_sha256_run1
    );
    let _ = writeln!(
        &mut s,
        "  episodes.jsonl:       {}",
        r.episodes_jsonl_sha256_run1
    );
    let _ = writeln!(&mut s, "  audit_report.html:    {html_run1_hex}");
    s.push('\n');
    s.push_str("Run 2 SHA-256\n");
    let _ = writeln!(
        &mut s,
        "  casefile.json:        {}",
        r.casefile_json_sha256_run2
    );
    let _ = writeln!(
        &mut s,
        "  episodes.jsonl:       {}",
        r.episodes_jsonl_sha256_run2
    );
    let _ = writeln!(&mut s, "  audit_report.html:    {html_run2_hex}");
    s.push('\n');
    let _ = writeln!(
        &mut s,
        "final_case_file_hash (run 1): {}",
        r.final_case_file_hash_run1_hex
    );
    let _ = writeln!(
        &mut s,
        "final_case_file_hash (run 2): {}",
        r.final_case_file_hash_run2_hex
    );
    let _ = writeln!(
        &mut s,
        "episode_count        (run 1): {}",
        r.episode_count_run1
    );
    let _ = writeln!(
        &mut s,
        "episode_count        (run 2): {}",
        r.episode_count_run2
    );
    s.push('\n');
    s.push_str("Toolchain\n");
    for (k, v) in &r.toolchain {
        let _ = writeln!(&mut s, "  {k}: {v}");
    }
    s.push('\n');
    s.push_str("Note: replay determinism is asserted only for the toolchain\n");
    s.push_str("recorded above. Different driver / CUDA / hardware versions\n");
    s.push_str("may produce different bytes; the audit does NOT claim\n");
    s.push_str("cross-toolchain replay byte-identity.\n");
    s
}

/// S-REAL.PERF: emit the per-dataset performance profile.
///
/// WHY: A ninth artifact next to the original eight. Carries the
/// per-stage host-Instant wall-clock microseconds, multi-iteration
/// dispatch variance (median + p50/p95/p99), throughput metrics
/// (events/s, finite-cells/s, logical-bytes/s), and the optional
/// sequential-catalog amortization total. Explicitly RUNTIME-
/// DEPENDENT — the byte-identical-replay claim covers the inference
/// chain (`casefile.json` + `episodes.jsonl`), NOT this timing block.
/// The disclosure paragraph at the bottom records that distinction.
#[allow(
    clippy::too_many_lines,
    reason = "Receipt emitter is a single byte-stable text block; \
              splitting risks ordering divergence between two builds."
)]
fn emit_perf_profile_txt(spec: &DatasetSpec, p: &PerformanceProfile) -> String {
    let mut s = String::new();
    s.push_str("=== S-REAL.PERF performance profile ===\n");
    let _ = writeln!(&mut s, "dataset: {}", spec.dataset_id);
    let _ = writeln!(&mut s, "iters:    {}", p.iters);
    let _ = writeln!(&mut s, "catalogs: {}", p.catalogs);
    s.push('\n');
    s.push_str("Per-stage wall (microseconds, host Instant):\n");
    let _ = writeln!(&mut s, "  ingest_us              : {}", p.ingest_us);
    let _ = writeln!(&mut s, "  lowering_us            : {}", p.lowering_us);
    let _ = writeln!(&mut s, "  contract_setup_us      : {}", p.contract_setup_us);
    let _ = writeln!(
        &mut s,
        "  cuda_dispatch_run1_us  : {}",
        p.cuda_dispatch_run1_us
    );
    let _ = writeln!(
        &mut s,
        "  cuda_dispatch_run2_us  : {}",
        p.cuda_dispatch_run2_us
    );
    if !p.cuda_dispatch_extra_us.is_empty() {
        let _ = writeln!(
            &mut s,
            "  cuda_dispatch_extra_us : {:?}",
            p.cuda_dispatch_extra_us
        );
    }
    let _ = writeln!(&mut s, "  casefile_emit_us       : {}", p.casefile_emit_us);
    let _ = writeln!(
        &mut s,
        "  episodes_jsonl_emit_us : {}",
        p.episodes_jsonl_emit_us
    );
    let _ = writeln!(
        &mut s,
        "  audit_report_emit_us   : {}",
        p.audit_report_emit_us
    );
    let _ = writeln!(&mut s, "  total_us               : {}", p.total_us);
    s.push('\n');
    s.push_str("Dispatch variance (across all recorded iters):\n");
    let _ = writeln!(
        &mut s,
        "  dispatch_median_us     : {}",
        p.dispatch_median_us()
    );
    let _ = writeln!(&mut s, "  dispatch_p50_us        : {}", p.percentile_us(50));
    let _ = writeln!(&mut s, "  dispatch_p95_us        : {}", p.percentile_us(95));
    let _ = writeln!(&mut s, "  dispatch_p99_us        : {}", p.percentile_us(99));
    s.push('\n');
    s.push_str("Throughput (end-to-end wall):\n");
    let _ = writeln!(
        &mut s,
        "  events_emitted             : {}",
        p.events_emitted
    );
    let _ = writeln!(&mut s, "  finite_cells               : {}", p.finite_cells);
    let _ = writeln!(
        &mut s,
        "  fixture_byte_size          : {}",
        p.fixture_byte_size
    );
    let _ = writeln!(
        &mut s,
        "  events_per_second          : {}",
        p.events_per_second()
    );
    let _ = writeln!(
        &mut s,
        "  finite_cells_per_second    : {}",
        p.finite_cells_per_second()
    );
    let _ = writeln!(
        &mut s,
        "  logical_bytes_per_second   : {}",
        p.logical_bytes_per_second()
    );
    if p.catalogs > 1 {
        s.push('\n');
        s.push_str("Sequential-catalog amortization (--catalogs > 1):\n");
        let _ = writeln!(
            &mut s,
            "  catalogs_total_us          : {}",
            p.catalogs_total_us
        );
        let _ = writeln!(
            &mut s,
            "  per_catalog_us             : {}",
            p.catalogs_total_us / u64::from(p.catalogs)
        );
        s.push_str("  note                       : K sequential build_gpu calls; NOT a batched dispatch.\n");
    }
    s.push('\n');
    s.push_str("Honest framing (panel-locked, MUST appear):\n");
    s.push_str("  - Timing values are runtime-dependent. The byte-identical-replay\n");
    s.push_str("    claim covers casefile.json + episodes.jsonl (the inference\n");
    s.push_str("    chain), NOT this perf_profile.txt or the timing values inside\n");
    s.push_str("    audit_report.html. Re-invoking s-real-1-audit will produce a\n");
    s.push_str("    new perf_profile.txt with new timing values; the casefile +\n");
    s.push_str("    episodes bytes will remain byte-identical to the sealed S-REAL.1.1.1\n");
    s.push_str("    artifacts.\n");
    s.push_str("  - At these small fixture sizes (128 / 192 / 656 events) the wall\n");
    s.push_str("    is overhead-dominated. Real-data throughput numbers below are\n");
    s.push_str("    honest measurements on this hardware at this scale; they are NOT\n");
    s.push_str("    saturation claims, NOT production-deployment throughput, and NOT\n");
    s.push_str("    detector-superiority benchmarks. CUDA timing is host-Instant\n");
    s.push_str("    wall (not cudaEvent kernel time; that lives in S-PERF).\n");
    s.push_str("  - Cross-driver / cross-CUDA / cross-hardware replay byte-identity\n");
    s.push_str("    or throughput-identity is NOT claimed.\n");
    s
}

fn emit_limitations_md(spec: &DatasetSpec) -> String {
    let mut s = String::new();
    let _ = writeln!(
        &mut s,
        "# S-REAL.1 audit — limitations and non-claims ({})\n",
        spec.dataset_id
    );
    s.push_str("This file accompanies the `audit_report.html` for this dataset. The\n");
    s.push_str("audit's deliverable is **deterministic, replayable structural\n");
    s.push_str("evidence on real public dataset bytes** — not domain-truth\n");
    s.push_str("claims.\n\n");
    s.push_str("## Non-claims\n\n");
    for nc in NON_CLAIMS_LINES {
        let _ = writeln!(&mut s, "- {nc}");
    }
    s.push_str("\n## Lowering disclosure\n\n");
    s.push_str("The upstream fixture is in `residual-projection v2` form\n");
    s.push_str("(window-major × signal-minor TSV). DSFB-GPU normally takes a\n");
    s.push_str("`Vec<TraceEvent>` and projects events into residuals via its\n");
    s.push_str("window-feature kernel; the upstream is already past that\n");
    s.push_str("projection. To run the deterministic engine on this form\n");
    s.push_str("without modifying the dispatcher, the audit lowers each\n");
    s.push_str("finite cell into one synthetic `TraceEvent` via a documented\n");
    s.push_str("rule (see `schema_map.toml` and section 2 of\n");
    s.push_str("`audit_report.html`). The audit does NOT claim to recover the\n");
    s.push_str("upstream's original trace events; it claims DSFB-GPU saw\n");
    s.push_str("exactly the events that rule produces from these bytes.\n");
    s
}

const NON_CLAIMS_LINES: &[&str] = &[
    "Does NOT claim DSFB has identified the \"real\" anomaly in the dataset.",
    "Does NOT claim DSFB outperforms any other anomaly detector.",
    "Does NOT claim DSFB has discovered causality.",
    "Does NOT claim DSFB has measured remediation effectiveness.",
    "Does NOT claim fitness-for-purpose on regulated or safety-critical use.",
    "Does NOT claim the dataset is \"correctly labeled\" or \"ground truth\".",
    "Does NOT claim the corpus or registry is exhaustive.",
    "Does NOT claim replay determinism across different driver / CUDA / hardware versions.",
];

#[cfg(test)]
mod tests {
    use super::*;
    use dsfb_gpu_debug_core::bank::{BankMotif, Episode};
    use dsfb_gpu_debug_core::fixed::Q16;
    use dsfb_gpu_debug_core::grammar::{GrammarState, ReasonCode};

    fn mk_episode(
        entity: u32,
        start: u32,
        end: u32,
        motif: BankMotif,
        reason: ReasonCode,
    ) -> Episode {
        Episode {
            entity_id: entity,
            start_window: start,
            end_window: end,
            motif,
            reason,
            peak_state: GrammarState::Boundary,
            peak_residual_q: Q16(123),
            peak_drift_q: Q16(456),
            peak_slew_q: Q16(789),
            detector_bit_count: 3,
            admission: None,
        }
    }

    #[test]
    fn lookup_admits_known_datasets() {
        // All 13 S-REAL admitted datasets across 5 source classes.
        for id in [
            // S-REAL.1 + S-REAL.2c (13 datasets, 5 source-class families).
            "tadbench_f11",
            "tadbench_f04",
            "tadbench_f11b",
            "tadbench_f19",
            "illinois_socialnet",
            "lo2",
            "deeptralog",
            "aiops_kpi",
            "multidim_localization",
            "defects4j",
            "bugsinpy",
            "promise_defect_prediction",
            "cmapss_fd001_unit1",
            // S-REAL.3 admissions (7 more, reaching 20).
            "cmapss_fd001_unit50",
            "cmapss_fd002_unit1",
            "cmapss_fd002_unit100",
            "cmapss_fd003_unit1",
            "cmapss_fd004_unit1",
            "promise_ant_1_4",
            "deeptralog_f02",
            // Large-fixture throughput entry (post-S-REAL.3; 21st
            // dataset, 1024×1024 cells from RadioML 2018 SNR=30 dB
            // HDF5; used by scripts/s_real_throughput_bench.sh to
            // measure dispatcher throughput at S-PERF.16.a magnitude
            // on real RF I/Q bytes).
            "radioml_2018_snr30_large",
            // Second large-fixture entry — DeepBeam IQ-magnitude
            // residual projection at 1024×1024 cells. Same role
            // as radioml_2018_snr30_large: throughput-witness for
            // the saturation sweep.
            "deepbeam_large",
            // S-REAL.3.1 saturation-sweep extension: 7 more
            // throughput-witness fixtures (4 RF + 3 database + 1
            // sub-saturation mmWave). All built from real
            // upstream data via deterministic recipes; all
            // throughput witnesses, not domain-truth claims.
            "radioml_gold_large",
            "powder_large",
            "oracle_large",
            "deepsense6g_large",
            "imdb_tgz_large",
            "imdb_duckdb_large",
            "snowset_large",
            "sqlshare_large",
        ] {
            assert!(lookup(id).is_some(), "lookup must admit {id}");
        }
        assert!(lookup("unknown_dataset").is_none());
    }

    #[test]
    fn dataset_sha256_pins_are_lower_hex_64() {
        for spec in all_dataset_specs() {
            assert_eq!(
                spec.fixture_sha256_hex.len(),
                64,
                "{} pin must be 64 hex chars",
                spec.dataset_id
            );
            assert!(
                spec.fixture_sha256_hex
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "{} pin must be lowercase hex",
                spec.dataset_id
            );
        }
    }

    // =================================================================
    // S-REAL.3.1.2 — panel-required dataset-table-split invariants.
    //
    // The 5 tests below pin the audit/saturation surface boundary.
    // Before S-REAL.3.1.2 the driver had a single `DATASETS` constant
    // carrying 30 entries and `--dataset all` enumerated all 30,
    // which silently dispatched the 10 large saturation fixtures on
    // Colab where their TSVs are deliberately excluded from the
    // tarball. These tests make the boundary load-bearing: any future
    // commit that re-merges the tables, drops a saturation id back
    // into the audit set, or breaks the tier_dir → bundle_manifest
    // mirror fails the suite before reaching the bundle integrity
    // gate.
    // =================================================================

    /// Panel-required negative: `--dataset all` must enumerate exactly
    /// the 20 sealed audit datasets. The bundle hash chain has 60
    /// rows = 20 datasets × 3 chain-pinned artifacts; the count is a
    /// load-bearing identity for the entire S-REAL.3 bundle.
    #[test]
    fn audit_all_dataset_count_is_20() {
        assert_eq!(
            AUDIT_DATASETS.len(),
            20,
            "AUDIT_DATASETS must hold exactly 20 sealed audit datasets; \
             changing this is a panel-acknowledged schema-upgrade event"
        );
    }

    /// Panel-required negative: `SATURATION_FIXTURES` must hold
    /// exactly the 10 saturation-class real-data fixtures the
    /// 30-fixture sweep depends on (the other 20 sweep fixtures
    /// are the audit datasets at their native cell counts, dispatched
    /// from AUDIT_DATASETS).
    #[test]
    fn saturation_fixture_count_is_10() {
        assert_eq!(
            SATURATION_FIXTURES.len(),
            10,
            "SATURATION_FIXTURES must hold exactly 10 saturation-class \
             real-data fixtures (RadioML/DeepBeam/RadioML-Gold/POWDER/\
             ORACLE/Deepsense6G/IMDb/Snowset/SQLShare)"
        );
    }

    /// Panel-required CAMPAIGN IDENTITY negative: no saturation
    /// fixture id appears in `AUDIT_DATASETS`, AND no AUDIT_DATASETS
    /// entry's default_path matches the saturation-TSV exclusion
    /// pattern that `pack_for_colab.sh` uses (`*x1024.tsv`). Together
    /// these guarantee `--dataset all` can never accidentally
    /// dispatch a TSV that the slim Colab tarball excludes.
    #[test]
    fn audit_all_excludes_large_saturation_fixtures() {
        const SATURATION_IDS: &[&str] = &[
            "radioml_2018_snr30_large",
            "deepbeam_large",
            "radioml_gold_large",
            "powder_large",
            "oracle_large",
            "deepsense6g_large",
            "imdb_tgz_large",
            "imdb_duckdb_large",
            "snowset_large",
            "sqlshare_large",
        ];
        let audit_ids: Vec<&str> = AUDIT_DATASETS.iter().map(|d| d.dataset_id).collect();
        for sat_id in SATURATION_IDS {
            assert!(
                !audit_ids.contains(sat_id),
                "AUDIT_DATASETS must NOT contain saturation fixture {sat_id}; \
                 the slim Colab tarball excludes its TSV"
            );
        }
        // Defense-in-depth: no AUDIT_DATASETS entry's TSV path can
        // match the saturation-TSV exclusion pattern. The
        // `pack_for_colab.sh` exclusion uses the glob `*x1024.tsv`
        // (which covers 1024x1024 / 1020x1024 / 512x1024 variants).
        for spec in AUDIT_DATASETS {
            let lower_path = spec.default_path.to_lowercase();
            assert!(
                !lower_path.ends_with("x1024.tsv"),
                "AUDIT_DATASETS entry {} has default_path {:?} matching \
                 the saturation-TSV exclusion pattern *x1024.tsv; this \
                 would break the Colab public-replay path",
                spec.dataset_id,
                spec.default_path
            );
        }
    }

    /// Panel-required negative: no dataset_id appears in BOTH tables.
    /// The two tables must be a partition, not just a covering set.
    #[test]
    fn audit_dataset_table_no_duplicate_ids() {
        let audit_ids: std::collections::HashSet<&str> =
            AUDIT_DATASETS.iter().map(|d| d.dataset_id).collect();
        let sat_ids: std::collections::HashSet<&str> =
            SATURATION_FIXTURES.iter().map(|d| d.dataset_id).collect();
        let overlap: Vec<&&str> = audit_ids.intersection(&sat_ids).collect();
        assert!(
            overlap.is_empty(),
            "AUDIT_DATASETS and SATURATION_FIXTURES must be disjoint; \
             found overlap: {overlap:?}"
        );
        // Also: no id is repeated within a single table.
        assert_eq!(
            audit_ids.len(),
            AUDIT_DATASETS.len(),
            "AUDIT_DATASETS contains a duplicate dataset_id"
        );
        assert_eq!(
            sat_ids.len(),
            SATURATION_FIXTURES.len(),
            "SATURATION_FIXTURES contains a duplicate dataset_id"
        );
    }

    /// Panel-required cross-validation negative: every AUDIT_DATASETS
    /// entry's `tier_dir` matches the `tier_dir` field of the
    /// corresponding entry in `reports/s_real_3/bundle_manifest.toml`.
    /// This is the load-bearing mirror — if the driver's tier table
    /// drifts from the sealed bundle's tier layout, the freshly-
    /// emitted artifacts land in the wrong directory and the bundle
    /// integrity test catches it downstream; this test catches the
    /// drift at the table-identity level first.
    ///
    /// The bundle manifest stores `tier_dir = "reports/s_real_X/<id>"`
    /// (full path); the driver stores `tier_dir = "s_real_X"` (root-
    /// relative segment). The cross-check normalises both.
    #[test]
    fn audit_dataset_tier_dir_matches_bundle_manifest() {
        let manifest_path = std::path::Path::new("../../reports/s_real_3/bundle_manifest.toml");
        let manifest_path = if manifest_path.exists() {
            manifest_path.to_path_buf()
        } else {
            // Test runner CWD can be either the workspace root or the
            // crate root depending on how tests are launched; resolve
            // both layouts.
            std::path::PathBuf::from("reports/s_real_3/bundle_manifest.toml")
        };
        let body = match std::fs::read_to_string(&manifest_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "audit_dataset_tier_dir_matches_bundle_manifest: skipping \
                     (bundle_manifest.toml not readable at {}: {e})",
                    manifest_path.display()
                );
                return;
            }
        };
        // Parse the manifest as a flat key-walker. Each `[datasets.<id>]`
        // section is followed by a `tier_dir = "reports/s_real_X/<id>"`
        // line. Build the map id → manifest_tier_dir.
        let mut current_id: Option<String> = None;
        let mut manifest_tier_by_id: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for line in body.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("[datasets.") {
                if let Some(end) = rest.find(']') {
                    current_id = Some(rest[..end].to_string());
                }
            } else if let Some(rest) = trimmed.strip_prefix("tier_dir = \"") {
                if let Some(end) = rest.find('"') {
                    if let Some(id) = current_id.take() {
                        manifest_tier_by_id.insert(id, rest[..end].to_string());
                    }
                }
            }
        }
        if manifest_tier_by_id.is_empty() {
            eprintln!(
                "audit_dataset_tier_dir_matches_bundle_manifest: skipping \
                 (manifest at {} contains no parseable entries)",
                manifest_path.display()
            );
            return;
        }
        for spec in AUDIT_DATASETS {
            let expected_full = format!("reports/{}/{}", spec.tier_dir, spec.dataset_id);
            let manifest_full = manifest_tier_by_id.get(spec.dataset_id).unwrap_or_else(|| {
                panic!(
                    "AUDIT_DATASETS entry {} is missing from bundle_manifest.toml",
                    spec.dataset_id
                )
            });
            assert_eq!(
                manifest_full, &expected_full,
                "AUDIT_DATASETS[{}].tier_dir mirror divergence: driver = {:?}, \
                 manifest = {:?}",
                spec.dataset_id, expected_full, manifest_full
            );
        }
    }

    #[test]
    fn serialize_episodes_jsonl_is_deterministic_and_sorted() {
        // Same multiset but different insertion order; output must be identical.
        let a = vec![
            mk_episode(
                5,
                10,
                12,
                BankMotif::LatencyRamp,
                ReasonCode::BoundaryApproach,
            ),
            mk_episode(
                2,
                1,
                4,
                BankMotif::ErrorBurst,
                ReasonCode::EnvelopeViolation,
            ),
            mk_episode(2, 1, 4, BankMotif::ErrorBurst, ReasonCode::BoundaryApproach),
        ];
        let b = vec![
            mk_episode(2, 1, 4, BankMotif::ErrorBurst, ReasonCode::BoundaryApproach),
            mk_episode(
                5,
                10,
                12,
                BankMotif::LatencyRamp,
                ReasonCode::BoundaryApproach,
            ),
            mk_episode(
                2,
                1,
                4,
                BankMotif::ErrorBurst,
                ReasonCode::EnvelopeViolation,
            ),
        ];
        let sa = serialize_episodes_jsonl(&a);
        let sb = serialize_episodes_jsonl(&b);
        assert_eq!(sa, sb);
        let text = std::str::from_utf8(&sa).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        // Verify sort order: idx 0 has entity_id 2 + reason BoundaryApproach (u8 1)
        assert!(lines[0].contains("\"entity_id\":2"));
        assert!(lines[0].contains("\"reason\":\"BoundaryApproach\""));
        // idx 1: entity 2, reason EnvelopeViolation (u8 5)
        assert!(lines[1].contains("\"entity_id\":2"));
        assert!(lines[1].contains("\"reason\":\"EnvelopeViolation\""));
        // idx 2: entity 5
        assert!(lines[2].contains("\"entity_id\":5"));
    }

    #[test]
    fn serialize_episodes_jsonl_handles_empty() {
        let s = serialize_episodes_jsonl(&[]);
        assert!(s.is_empty());
    }

    #[test]
    fn dataset_manifest_toml_carries_required_keys() {
        let m = DatasetManifest {
            dataset_id: "x".to_string(),
            display_name: "X".to_string(),
            upstream_doi_or_url: "doi:test".to_string(),
            license: "Apache-2.0".to_string(),
            source_class: "TestClass".to_string(),
            vendored_path: "/tmp/x".to_string(),
            fixture_sha256_hex: "0".repeat(64),
            fixture_byte_size: 42,
        };
        let toml = emit_dataset_manifest_toml(&m);
        for key in [
            "dataset_id",
            "display_name",
            "upstream_doi_or_url",
            "license",
            "source_class",
            "vendored_path",
            "sha256_hex",
            "byte_size",
        ] {
            assert!(toml.contains(key), "missing {key}");
        }
    }

    #[test]
    fn limitations_md_carries_every_non_claim() {
        let s = emit_limitations_md(&AUDIT_DATASETS[0]);
        for nc in NON_CLAIMS_LINES {
            assert!(s.contains(nc), "missing non-claim: {nc}");
        }
    }
}
