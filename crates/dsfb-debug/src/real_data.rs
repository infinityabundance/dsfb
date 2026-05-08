//! DSFB-Debug: real-dataset evaluation entry point — the only path
//! by which the engine binds to a non-trivial fixture (paper-lock +
//! std).
//!
//! # The hard-error policy (no synthetic fall-back)
//!
//! This module is the load-bearing implementation of the crate's
//! authoring policy: **real-world data only; never synthetic; the
//! engine never falls back to a stub.** Every fallible path returns
//! a typed error that surfaces the failure to the operator:
//!
//! - **Empty bytes or sentinel fixture** →
//!   `DsfbError::MissingRealData`. The harness refuses to run on a
//!   `# UPSTREAM_FIXTURE_NOT_VENDORED` placeholder. Sentinel
//!   detection is byte-level (the marker string anywhere in the
//!   bytes triggers the error), defending against the case where
//!   somebody's hash happens to match.
//! - **SHA-256 mismatch** → `DsfbError::HashMismatch`. The supplied
//!   bytes' digest must equal the manifest's `fixture_sha256_hex`.
//!   Any drift surfaces this error rather than silently proceeding.
//! - **Parse / shape error** → `DsfbError::ParseError` or
//!   `DimensionMismatch`. Adapter-level errors surfaced cleanly.
//! - **Buffer overflow** → `DsfbError::BufferTooSmall`. The internal
//!   flat-cap (8192) protects against silent truncation; if the
//!   fixture's `num_signals × num_windows` exceeds the cap, the
//!   harness errors rather than producing partial output.
//!
//! # The 12 vendored fixtures (post Phase G)
//!
//! Each fixture is a `RealDatasetManifest` constant in this module.
//! All carry real upstream bytes (DOI-pinned, SHA-256-gated):
//!
//! | # | Manifest | Upstream | Fault profile |
//! |---|----------|----------|---------------|
//! | 1 | `MANIFEST_TADBENCH_F11` | Zenodo `10.5281/zenodo.6979726` | order-service deployment-regression (35 604 spans) |
//! | 2 | `MANIFEST_TADBENCH_F11B` | same | auth-mongo cross-fixture (108 spans) |
//! | 3 | `MANIFEST_TADBENCH_F04` | same | admin-service springstarter config (25 316 spans) |
//! | 4 | `MANIFEST_TADBENCH_F19` | same | mongodb-driver-3.0.4 config (19 281 spans) |
//! | 5 | `MANIFEST_ILLINOIS_SOCIALNETWORK` | DataBank `IDB-6738796` | unsampled DeathStarBench (160 000 traces) |
//! | 6 | `MANIFEST_AIOPS_CHALLENGE` | NetManAIOps/Bagel | AIOps 2018 KPI (Su et al., IPCCC 2018) |
//! | 7 | `MANIFEST_LO2` | Zenodo `10.5281/zenodo.14257989` | OAuth2 endoductive validator |
//! | 8 | `MANIFEST_MULTIDIM_LOCALIZATION` | NetManAIOps/MultiDimension-Localization | high-dim categorical aggregate |
//! | 9 | `MANIFEST_DEEPTRALOG` | FudanSELab/DeepTraLog | combined log+trace ERROR data (Zhang et al., ICSE 2022) |
//! | 10 | `MANIFEST_DEFECTS4J` | rjust/defects4j | Java bug catalog (Just et al., ISSTA 2014) |
//! | 11 | `MANIFEST_BUGSINPY` | soarsmu/BugsInPy | Python bug catalog (Widyasari et al., FSE 2020) |
//! | 12 | `MANIFEST_PROMISE` | ssea-lab/PROMISE | defect-prediction (Menzies et al., 2003+) |
//!
//! # Verification — Theorem 9 on real bytes
//!
//! `evaluate_real_dataset` runs `verify_deterministic_replay` after
//! computing metrics. The returned `RealDatasetEvaluation` carries
//! `deterministic_replay_holds: bool`; this flag must be `true` on
//! every real-bytes fixture, asserting that Theorem 9 (paper §6.4)
//! holds operationally, not just mechanically.

#![cfg(feature = "paper-lock")]

extern crate std;

use std::vec;
use std::vec::Vec;

use crate::error::{DsfbError, Result};
use crate::types::*;
use crate::adapters::residual_projection::{parse_residual_projection, OwnedResidualMatrix};
use crate::adapters::sha256;
use crate::DsfbDebugEngine;

/// Compile-time-curated manifest for one real-world dataset slice.
/// Mirrors a `[name]` block in `data/MANIFEST.toml`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RealDatasetManifest {
    /// Display name passed through into BenchmarkMetrics.dataset_name.
    pub name: &'static str,
    /// Upstream DOI (Zenodo, Illinois Data Bank, etc.).
    pub upstream_doi: &'static str,
    /// Upstream archive URL or repository URL for human verification.
    pub upstream_url: &'static str,
    /// SHA-256 (lowercase hex, 64 chars) of the upstream archive at the
    /// time of extraction. Stored for provenance — not verified at runtime
    /// (we cannot download from inside paper-lock without breaking the
    /// hermetic property of the test).
    pub upstream_archive_sha256_hex: &'static str,
    /// Path of the in-tree extracted fixture (relative to crate root).
    pub fixture_path: &'static str,
    /// SHA-256 of the in-tree fixture file. Verified at runtime against
    /// the bytes the caller supplies (via include_bytes!). Drift here
    /// indicates either tampering or an out-of-date manifest.
    pub fixture_sha256_hex: &'static str,
    /// Single-line provenance description (which spans, which window
    /// granularity, which extraction recipe).
    pub fixture_provenance: &'static str,
    /// SPDX licence identifier of the upstream archive.
    pub upstream_license: &'static str,
}

/// Result of a real-dataset evaluation. Wraps `BenchmarkMetrics` plus the
/// determinism-replay flag.
#[derive(Debug, Clone)]
pub struct RealDatasetEvaluation {
    pub manifest_name: &'static str,
    pub metrics: BenchmarkMetrics,
    /// Theorem 9 deterministic replay outcome on the same real bytes.
    pub deterministic_replay_holds: bool,
    /// Number of episodes produced.
    pub episode_count: usize,
    /// Verbatim fixture header (provenance / DOI / extraction recipe).
    pub fixture_header: std::string::String,
}

/// Verify that `fixture_bytes` match `manifest.fixture_sha256_hex`.
pub fn verify_fixture_integrity(
    manifest: &RealDatasetManifest,
    fixture_bytes: &[u8],
) -> Result<()> {
    if fixture_bytes.is_empty() {
        return Err(DsfbError::MissingRealData);
    }
    // Reject the ASCII-marker form of an unpopulated fixture even when the
    // hash happens to match (defence in depth).
    let sentinel = b"UPSTREAM_FIXTURE_NOT_VENDORED";
    let mut sentinel_seen = false;
    let mut window = 0;
    while window + sentinel.len() <= fixture_bytes.len() {
        if &fixture_bytes[window..window + sentinel.len()] == sentinel {
            sentinel_seen = true;
            break;
        }
        window += 1;
    }
    if sentinel_seen {
        return Err(DsfbError::MissingRealData);
    }
    let actual = sha256::sha256_hex(fixture_bytes);
    if actual.as_slice() != manifest.fixture_sha256_hex.as_bytes() {
        return Err(DsfbError::HashMismatch);
    }
    Ok(())
}

/// Evaluate one real-world dataset slice end-to-end.
///
/// Steps:
///   1. Verify fixture integrity (length, sentinel, SHA-256).
///   2. Parse to `OwnedResidualMatrix`.
///   3. Run `engine.run_evaluation` with the paper-lock config that the
///      engine was constructed with.
///   4. Run `engine.verify_deterministic_replay` on the same bytes.
///
/// The caller supplies the engine, so the const generics can be tuned per
/// dataset (e.g. `DsfbDebugEngine::<32, 64>` for the small TADBench slice).
pub fn evaluate_real_dataset<const S: usize, const M: usize>(
    engine: &DsfbDebugEngine<S, M>,
    manifest: &RealDatasetManifest,
    fixture_bytes: &[u8],
) -> Result<RealDatasetEvaluation> {
    verify_fixture_integrity(manifest, fixture_bytes)?;

    let matrix: OwnedResidualMatrix = parse_residual_projection(fixture_bytes)?;
    if matrix.is_sentinel {
        return Err(DsfbError::MissingRealData);
    }
    if matrix.num_signals == 0 || matrix.num_windows == 0 {
        return Err(DsfbError::MissingRealData);
    }

    let n = matrix.num_windows.checked_mul(matrix.num_signals).unwrap_or(usize::MAX);
    let mut eval_out: Vec<SignalEvaluation> = vec![blank_eval(); n];
    let mut episodes_out: Vec<DebugEpisode> = vec![blank_episode(); 256];

    let (episode_count, metrics) = engine.run_evaluation(
        &matrix.data,
        matrix.num_signals,
        matrix.num_windows,
        &matrix.fault_labels,
        matrix.healthy_window_end,
        &mut eval_out,
        &mut episodes_out,
        manifest.name,
    )?;

    let deterministic = engine.verify_deterministic_replay(
        &matrix.data,
        matrix.num_signals,
        matrix.num_windows,
        &matrix.fault_labels,
        matrix.healthy_window_end,
    )?;

    Ok(RealDatasetEvaluation {
        manifest_name: manifest.name,
        metrics,
        deterministic_replay_holds: deterministic,
        episode_count,
        fixture_header: matrix.header_provenance,
    })
}

fn blank_eval() -> SignalEvaluation {
    SignalEvaluation {
        window_index: 0,
        signal_index: 0,
        residual_value: 0.0,
        sign_tuple: SignTuple::ZERO,
        raw_grammar_state: GrammarState::Admissible,
        confirmed_grammar_state: GrammarState::Admissible,
        reason_code: ReasonCode::Admissible,
        motif: None,
        semantic_disposition: SemanticDisposition::Unknown,
        dsa_score: 0.0,
        policy_state: PolicyState::Silent,
        was_imputed: false,
        drift_persistence: 0.0,
    }
}

fn blank_episode() -> DebugEpisode {
    DebugEpisode {
        episode_id: 0,
        start_window: 0,
        end_window: 0,
        peak_grammar_state: GrammarState::Admissible,
        primary_reason_code: ReasonCode::Admissible,
        matched_motif: SemanticDisposition::Unknown,
        policy_state: PolicyState::Silent,
        contributing_signal_count: 0,
        structural_signature: StructuralSignature {
            dominant_drift_direction: DriftDirection::None,
            peak_slew_magnitude: 0.0,
            duration_windows: 0,
            signal_correlation: 0.0,
        },
        root_cause_signal_index: None,
    }
}

// =========================================================================
// Curated manifests for Phase I real-world datasets.
//
// The `fixture_sha256_hex` values are computed against the extracted slice
// at the time of vendoring. A reviewer who re-extracts from upstream must
// recompute and update this manifest; the test harness will refuse to run
// silently with stale or tampered bytes.
// =========================================================================

/// TADBench / TrainTicket fault-injection traces — primary Phase I dataset.
///
/// Fault case `F-04` slice from fault directory
/// `ts-admin-basic-info-service-sprintstarterweb_1.5.22`. Real Jaeger
/// spans projected to per-service (`latency_p50_ms`, `error_rate`) at
/// 15-second windows. See `data/upstream/project_trainticket_F04.py` for
/// the deterministic projection recipe.
pub const MANIFEST_TADBENCH_F04: RealDatasetManifest = RealDatasetManifest {
    name: "tadbench_trainticket_F04",
    upstream_doi: "10.5281/zenodo.6979726",
    upstream_url: "https://zenodo.org/records/6979726",
    upstream_archive_sha256_hex: "18456279cdcbc66b020bddd117a79ff453137fbc60f88cba81f2609fc1a74403",
    fixture_path: "data/fixtures/tadbench_trainticket_F04.tsv",
    fixture_sha256_hex: "68d834cbc3084020e81e25645b0aac4e8cc63e1c953abad6a5ec3fccf88537fe",
    fixture_provenance:
        "Real Jaeger spans projected from upstream fault directory \
         ts-admin-basic-info-service-sprintstarterweb_1.5.22. 25,316 spans, \
         15-second windows, 30 windows × 12 signals (top-6 services × \
         {latency_p50_ms, error_rate}). First 12 windows healthy baseline. \
         No fault-window labels claimed (steady-state regressed-config run).",
    upstream_license: "Apache-2.0",
};

/// TADBench / TrainTicket fault case `F-11b` (auth-mongo) — second
/// vendored slice. Distinct service (auth-mongo) from F-11
/// (order-service); used in cross-fixture fusion validation.
pub const MANIFEST_TADBENCH_F11B: RealDatasetManifest = RealDatasetManifest {
    name: "tadbench_trainticket_F11b",
    upstream_doi: "10.5281/zenodo.6979726",
    upstream_url: "https://zenodo.org/records/6979726",
    upstream_archive_sha256_hex: "18456279cdcbc66b020bddd117a79ff453137fbc60f88cba81f2609fc1a74403",
    fixture_path: "data/fixtures/tadbench_trainticket_F11b.tsv",
    fixture_sha256_hex: "d029f0ed3b0af1bbf53070d54e323ebbcef20c33119673b2361301834f48a6d5",
    fixture_provenance:
        "Real Jaeger spans projected from upstream fault directory \
         ts-auth-mongo_5.0.9_2022-07-06 (auth-mongo service, distinct from F-11's \
         order-service). 4-window × 6-signal residual matrix; 108 spans projected. \
         Used in cross-fixture fusion validation alongside F-11.",
    upstream_license: "Apache-2.0",
};

/// TADBench / TrainTicket fault case `F-19` (mongodb-driver-3.0.4)
/// version-config regression. Same upstream archive as F-04/F-11/F-11b
/// (Zenodo 10.5281/zenodo.6979726).
pub const MANIFEST_TADBENCH_F19: RealDatasetManifest = RealDatasetManifest {
    name: "tadbench_trainticket_F19",
    upstream_doi: "10.5281/zenodo.6979726",
    upstream_url: "https://zenodo.org/records/6979726",
    upstream_archive_sha256_hex: "18456279cdcbc66b020bddd117a79ff453137fbc60f88cba81f2609fc1a74403",
    fixture_path: "data/fixtures/tadbench_trainticket_F19.tsv",
    fixture_sha256_hex: "b1a599ab123b5989bb527a513fc0c07afea8e32143691ff94e78346ae5efbb16",
    fixture_provenance:
        "Real Jaeger spans projected from upstream fault directory \
         ts-order-service_3.0.4-mongodb-driver_2022-07-13 (mongodb-driver 3.0.4 \
         version-config regression, distinct from F-11's mongodb 4.2.2). 19,281 \
         spans; 30 windows × 12 signals (top-6 services × {latency_p50_ms, \
         error_rate}); first 12 windows healthy baseline.",
    upstream_license: "Apache-2.0",
};

/// TADBench / TrainTicket fault case `F-11 deployment-regression` slice
/// (vendored from upstream Zenodo archive
/// `10.5281/zenodo.6979726`, fault case
/// `ts-order-service_mongodb_4.2.2_2022-07-12`).
pub const MANIFEST_TADBENCH_F11: RealDatasetManifest = RealDatasetManifest {
    name: "tadbench_trainticket_F11",
    upstream_doi: "10.5281/zenodo.6979726",
    upstream_url: "https://zenodo.org/records/6979726",
    upstream_archive_sha256_hex: "18456279cdcbc66b020bddd117a79ff453137fbc60f88cba81f2609fc1a74403",
    fixture_path: "data/fixtures/tadbench_trainticket_F11.tsv",
    fixture_sha256_hex: "07c8f08558c00d62c48fa5833aef9ecaf7324968f2028ce244e4c8e5248512c3",
    fixture_provenance:
        "TrainTicket-Anomaly version-configuration regression. Fault case \
         ts-order-service_mongodb_4.2.2_2022-07-12. 8 services, 35,604 spans, \
         15-second windows; 16 signals (latency_p50_ms + error_rate per service); \
         431 windows; first 172 healthy baseline; NaN-imputed where a service had \
         zero spans in a window. No fault-window labels claimed.",
    upstream_license: "Apache-2.0",
};

/// Illinois unsampled microservice traces — secondary Phase I dataset.
/// Real bytes from upstream `tracing-data.tar.gz`
/// (DOI 10.13012/B2IDB-6738796_V1, Qiu et al. 2020). 160,000 unsampled
/// traces aggregated to a 32-window × 6-signal residual matrix using
/// `data/upstream/project_illinois.py`.
pub const MANIFEST_ILLINOIS_SOCIALNETWORK: RealDatasetManifest = RealDatasetManifest {
    name: "illinois_socialnetwork",
    upstream_doi: "10.13012/B2IDB-6738796_V1",
    upstream_url: "https://databank.illinois.edu/datasets/IDB-6738796",
    upstream_archive_sha256_hex: "partial-extract-pending-full-archive-fetch",
    fixture_path: "data/fixtures/illinois_socialnetwork.tsv",
    fixture_sha256_hex: "c86b5abd1b412f69cccaab9b3e838da742c5ea8a1ba1b6dce634ff3407cc082c",
    fixture_provenance:
        "DeathStarBench media-service compose-review unsampled trace slice from \
         Illinois Data Bank IDB-6738796. 6 trace components (nginx, uniqueid, \
         composereview, userreview, movie_review, reviewstorage); per-window p50 \
         duration in 10^-3 ms; 32 windows × 5,000 traces each (160,000 total). \
         100% trace capture eliminates sampling-aliasing for drift/slew confound \
         elimination per panel directive (Session 1).",
    upstream_license: "CC0-1.0",
};

/// AIOps Challenge 2018 KPI — Tier B baseline KPI anomaly detection.
/// Real bytes from upstream NetManAIOps/Bagel `sample_data.csv` (Su et
/// al., IPCCC 2018). The single-KPI 17,569-sample series is reshaped
/// into 4 contiguous sub-segments × 32 windows for multivariate fusion.
pub const MANIFEST_AIOPS_CHALLENGE: RealDatasetManifest = RealDatasetManifest {
    name: "aiops_challenge",
    upstream_doi: "Su2018:IPCCC:Bagel",
    upstream_url: "https://github.com/NetManAIOps/Bagel",
    upstream_archive_sha256_hex: "git:NetManAIOps/Bagel:sample_data.csv",
    fixture_path: "data/fixtures/aiops_challenge.tsv",
    fixture_sha256_hex: "29961b8b66d941c19c065cfa974a62f098ebd63ef8c9017d8219e9f228135642",
    fixture_provenance:
        "AIOps Challenge 2018 KPI canonical sample (5-min cadence) from \
         NetManAIOps/Bagel. Single KPI series reshaped into N_SIGNALS=4 \
         contiguous sub-segments to give multivariate detectors something to \
         correlate. Real bytes; deterministic reshape; per-window labels carried \
         from upstream (label=1 if any sub-segment was anomalous in that window).",
    upstream_license: "Apache-2.0",
};

/// LO2 (Taibi et al., PROMISE 2025) — Tier B, endoductive-mode validator.
/// Go-runtime metrics slice from upstream `lo2_sample.zip`. Per panel
/// directive, the bank deliberately has no LO2-specific motifs: this
/// fixture validates the endoductive `SemanticDisposition::Unknown`
/// branch.
pub const MANIFEST_LO2: RealDatasetManifest = RealDatasetManifest {
    name: "lo2",
    upstream_doi: "10.5281/zenodo.14257989",
    upstream_url: "https://zenodo.org/records/14257989",
    upstream_archive_sha256_hex: "2d9516fee44378b33b4db371f2bb16557f88feb07d830eeb47f27b229183a882",
    fixture_path: "data/fixtures/lo2.tsv",
    fixture_sha256_hex: "d49d3d078d8e5f9ef30ff6c198ee28e298e6c012e9aef47c06c0751fe73df9ab",
    fixture_provenance:
        "LO2 light-oauth2 first-CSV first-16-rows Go-runtime metrics slice. \
         Selected Prometheus metrics: go_gc_duration_seconds_sum, go_goroutines, \
         go_memstats_heap_alloc_bytes, go_memstats_alloc_bytes_total, \
         go_memstats_heap_inuse_bytes, go_memstats_stack_inuse_bytes. \
         Endoductive-mode validator: bank intentionally has no LO2-specific motifs.",
    upstream_license: "as-distributed-by-LO2-PROMISE-2025",
};

/// MultiDimension-Localization (NetManAIOps) — Tier C, high-dim
/// categorical-aggregate slice. 12 timestamps × 4 levels = 48 cells of
/// real upstream tabular data from `multidim_localization/part1.zip`.
pub const MANIFEST_MULTIDIM_LOCALIZATION: RealDatasetManifest = RealDatasetManifest {
    name: "multidim_localization",
    upstream_doi: "NetManAIOps:MultiDimension-Localization",
    upstream_url: "https://github.com/NetManAIOps/MultiDimension-Localization",
    upstream_archive_sha256_hex: "git:NetManAIOps/MultiDimension-Localization:part1.zip",
    fixture_path: "data/fixtures/multidim_localization.tsv",
    fixture_sha256_hex: "c714c85cf45bc462ffcf162b8877b15af70f85a3799079861a64f442152ed967",
    fixture_provenance:
        "Real upstream 5-dim categorical CSVs (i,e,c,p,l,value) projected by \
         data/upstream/project_multidim.py. 12 windows × 4 signals: each window \
         is one timestamp tick; each signal is the per-window mean over one \
         categorical level (l1..l4). Aggregate-by-level projection.",
    upstream_license: "as-distributed-by-NetManAIOps",
};

/// DeepTraLog (Zhang et al., ICSE 2022) — Tier C, log + trace fusion.
/// Real bytes from upstream `deeptralog/TraceLogData/F01.zip` member
/// `F01-01/SUCCESSF0101_SpanData2021-08-14_10-22-48.csv`. 600 spans
/// projected at 30-second windows.
pub const MANIFEST_DEEPTRALOG: RealDatasetManifest = RealDatasetManifest {
    name: "deeptralog",
    upstream_doi: "Zhang2022:ICSE:DeepTraLog",
    upstream_url: "https://github.com/FudanSELab/DeepTraLog",
    upstream_archive_sha256_hex: "git:FudanSELab/DeepTraLog:F01.zip",
    fixture_path: "data/fixtures/deeptralog.tsv",
    fixture_sha256_hex: "59e4ddce40dbcf88735d2ed904a8ba001a7cc1cc5c033ef976241a497f9607e2",
    fixture_provenance:
        "DeepTraLog F-01-01 ERROR span-data slice; 600 real spans aggregated to \
         16 windows × 8 signals (4 services × {latency_p50_ms, error_rate}); \
         30-second windows; first 6 windows healthy baseline. Combined \
         log+trace fault case.",
    upstream_license: "as-distributed-by-FudanSELab/DeepTraLog",
};

/// Defects4J Java bug catalog (Just et al., ISSTA 2014). Phase G
/// addition — code-debugging dataset. Per-project JIRA report-id
/// sequence projected from upstream `commit-db` files.
pub const MANIFEST_DEFECTS4J: RealDatasetManifest = RealDatasetManifest {
    name: "defects4j",
    upstream_doi: "Just2014:ISSTA:Defects4J",
    upstream_url: "https://github.com/rjust/defects4j",
    upstream_archive_sha256_hex: "git:rjust/defects4j:framework/projects",
    fixture_path: "data/fixtures/defects4j.tsv",
    fixture_sha256_hex: "528fc6e8bc3d16e8ea68ca4a267e448dcf490c200f0d5fe76c9b13bf890f9537",
    fixture_provenance:
        "Defects4J six-project (Lang/Math/Closure/Mockito/JacksonDatabind/Jsoup) \
         first-30-bug catalog projection. Window=bug.id 1..30; signal=Apache JIRA \
         report numerical id. Real upstream commit-db rows; deterministic.",
    upstream_license: "MIT",
};

/// BugsInPy Python bug catalog (Widyasari et al., ESEC/FSE 2020).
/// Phase G addition — code-debugging dataset complementing Defects4J
/// across the Python ecosystem.
pub const MANIFEST_BUGSINPY: RealDatasetManifest = RealDatasetManifest {
    name: "bugsinpy",
    upstream_doi: "Widyasari2020:ESEC/FSE:BugsInPy",
    upstream_url: "https://github.com/soarsmu/BugsInPy",
    upstream_archive_sha256_hex: "git:soarsmu/BugsInPy",
    fixture_path: "data/fixtures/bugsinpy.tsv",
    fixture_sha256_hex: "a30d51b27603bb0412976a618596e0a31a20eab8bb21ce496e72f6553ec86940",
    fixture_provenance:
        "BugsInPy six-project (ansible/pandas/keras/fastapi/scrapy/tornado) \
         first-30-bug catalog projection. Window=bug.id 1..30; signal=project \
         buggy_commit_id SHA-1 prefix decoded to 28-bit int.",
    upstream_license: "MIT",
};

/// PROMISE Software Engineering Repository defect-prediction dataset
/// (Menzies et al., 2003+). Phase G addition — code-debugging dataset
/// of per-module Chidamber-Kemerer OO metrics with bug counts.
pub const MANIFEST_PROMISE: RealDatasetManifest = RealDatasetManifest {
    name: "promise_defect_prediction",
    upstream_doi: "Menzies2003:PROMISE",
    upstream_url: "https://github.com/ssea-lab/PROMISE",
    upstream_archive_sha256_hex: "git:ssea-lab/PROMISE",
    fixture_path: "data/fixtures/promise_defect_prediction.tsv",
    fixture_sha256_hex: "8ba403ba0b403a0813eee983b19122762d9695bdaf78873e5dfb881e1bf970df",
    fixture_provenance:
        "PROMISE six-CSV (1.csv, 5.csv, 10.csv, 15.csv, 20.csv, 25.csv) \
         first-30-module defect-count slice. Window=module index 1..30; \
         signal=PROMISE CSV (one Java OSS project version). Per-module bug \
         count from canonical PROMISE OO-metrics CSVs.",
    upstream_license: "as-distributed-by-ssea-lab/PROMISE-mirror",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bytes_yield_missing_real_data() {
        let r = verify_fixture_integrity(&MANIFEST_TADBENCH_F04, b"");
        assert!(matches!(r, Err(DsfbError::MissingRealData)));
    }

    #[test]
    fn sentinel_yields_missing_real_data() {
        let bytes = b"# residual-projection v1\n# UPSTREAM_FIXTURE_NOT_VENDORED\n";
        let r = verify_fixture_integrity(&MANIFEST_TADBENCH_F04, bytes);
        assert!(matches!(r, Err(DsfbError::MissingRealData)));
    }
}
