//! Command-line driver for the edge crate. Manual arg parsing (no external CLI dependency).
//!
//! Arg parsing is intentionally minimal — a single match on the first argument — so the binary
//! has no dependency on `clap` or similar crates. This keeps the edge-crate binary small and
//! avoids transitive version-conflicts on constrained targets.
//!
//! ## Commands
//!
//! | Command | Function | Description |
//! |---|---|---|
//! | `demo` | [`run_demo`] | Full demo over all datasets → timestamped artifact bundle |
//! | `analyze <name>` | [`run_analyze`] | Run one named dataset, print metrics to stdout |
//! | `atlas` | [`run_atlas`] | Print detector-atlas + atlas authority-crate summary (SHA-256, counts, `atlas_hash_v1`) |
//! | `corpus` | [`run_corpus`] | Print soft-sensor data-corpus authority summary (`corpus_hash_v1`; needs feature `soft-sensor-corpus`) |
//! | `verify-replay` | [`run_verify_replay`] | Run synthetic suite twice; confirm identical hashes |
//! | `completeness-court` | [`run_completeness_court`] | Machine-checked artifact-graph completeness + consistency gate |
//! | `release-scrub` | [`run_release_scrub`] | `PublicReleaseScrubCourtV1` — public-release hygiene gate (placeholder DOI / private-file leakage / controlled-access rows / required artifacts) |
//! | `authority-diff <old.json> <new.json>` | [`run_authority_diff`] | Governed authority-drift receipts (`SemanticDiffReportV1` + `DetectorRegistryCompatibilityV1`) over two id→hash snapshots |
//! | `figures` | [`run_figures`] | Render the 60-figure gallery (figures + verbose log + deterministic ZIP); does NOT build the paper |
//!
//! ## Dataset discovery order
//!
//! 1. Look for committed CSV slices in `<crate_dir>/data/slices/*.csv` (real public datasets).
//! 2. If none found, fall back to the deterministic synthetic suite (see [`synthetic_suite`]).
//!
//! This means the demo is always runnable (even without committed data slices), and results are
//! clearly tagged with `kind = "synthetic"` so they are never confused with measured data.

use crate::atlas::Corpus;
use crate::datasets::{self, SyntheticFault};
use crate::figures;
use crate::fusion::DetectorTimeline;
use crate::pipeline::{self, AnalysisResult, PipelineConfig};
use crate::report::{self, DatasetEntry, RunManifest};
use crate::DataMatrix;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A fully loaded dataset ready to pass to the pipeline.
///
/// `kind` is a free-form provenance tag: "measured/slice" for committed real CSV slices,
/// "synthetic" for generated data. It propagates into the output artifacts so reports clearly
/// distinguish the two.
pub struct LoadedDataset {
    /// Short human-readable identifier (used as CSV header and artifact directory name).
    pub name: String,
    /// Provenance tag; written to every output artifact for honesty.
    pub kind: String,
    pub matrix: DataMatrix,
    /// Number of leading rows to treat as the nominal baseline window.
    pub n_base: usize,
}

/// Discover datasets to analyse.
///
/// Attempts to load committed CSV slices from `<crate_dir>/data/slices/*.csv` first, sorting
/// paths lexicographically so the order is deterministic across filesystems. Each successfully
/// parsed slice is tagged `"measured/slice"`.
///
/// If the `data/slices` directory does not exist or yields no valid CSVs (all failed parsing are
/// only warned, not fatal), falls back to [`synthetic_suite`] so the demo binary always produces
/// output even without committed data.
///
/// The 40 % baseline fraction passed to `load_csv_slice` is a default; the loader overrides it
/// with the fault-onset annotation when present in the CSV.
pub fn discover_datasets(crate_dir: &Path) -> Vec<LoadedDataset> {
    let slices = crate_dir.join("data/slices");
    // Honest per-dataset provenance: stamp each slice's `kind` from MANIFEST.toml
    // (measured/simulation/agreement-gated) instead of a blanket "measured/slice" — so `metrics.csv`,
    // every casefile.json `data_kind`, the operator report, and manifest.json match the manifest and the
    // paper's dataset table. This is the sim≠real discipline applied to the artifacts an evaluator runs.
    let kinds = datasets::manifest_kinds(crate_dir);
    let mut found: Vec<LoadedDataset> = Vec::new();
    if slices.is_dir() {
        // Sort paths for deterministic order: filesystem readdir order is not guaranteed.
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&slices)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().map(|x| x == "csv").unwrap_or(false))
                    .collect()
            })
            .unwrap_or_default();
        paths.sort();
        for p in paths {
            match datasets::load_csv_slice(&p, 0.4) {
                Ok((m, nb)) => {
                    let name = p.file_stem().unwrap().to_string_lossy().to_string();
                    // Fall back to "measured/slice" only if a slice is absent from the manifest (warn-worthy
                    // but non-fatal); a manifest entry's kind always wins.
                    let kind = kinds
                        .get(&name)
                        .map(|k| datasets::data_kind_tag(k))
                        .unwrap_or_else(|| "measured/slice".to_string());
                    found.push(LoadedDataset {
                        name,
                        kind,
                        matrix: m,
                        n_base: nb,
                    });
                }
                Err(e) => eprintln!("warn: skipping {}: {e}", p.display()),
            }
        }
    }
    if found.is_empty() {
        found = synthetic_suite();
    }
    found
}

/// Build the deterministic synthetic test suite spanning all four fault archetypes.
///
/// The suite is used as a sanity baseline when real dataset slices are unavailable, and as the
/// corpus for `verify-replay`. Outputs are always tagged `kind = "synthetic"` so they are never
/// presented as real process data.
///
/// Each entry is `(name, n_vars, fault_kind, seed)`. All datasets use 600 samples with fault
/// onset at sample 400, giving 300 normal samples for baseline training and 200 fault samples.
/// The per-dataset seeds are different prime-adjacent constants so the six datasets exercise
/// different noise realisations.
pub fn synthetic_suite() -> Vec<LoadedDataset> {
    let specs = [
        ("synth_step_drift", 12, SyntheticFault::StepDrift, 1),
        ("synth_ramp_drift", 12, SyntheticFault::RampDrift, 2),
        ("synth_sensor_bias", 8, SyntheticFault::SensorBias, 3),
        ("synth_oscillation", 8, SyntheticFault::Oscillation, 4),
        ("synth_wide_step", 30, SyntheticFault::StepDrift, 5),
        ("synth_wide_ramp", 30, SyntheticFault::RampDrift, 6),
    ];
    specs
        .iter()
        .map(|(name, nv, fault, seed)| {
            let n = 600usize;
            let onset = 400usize;
            let m = datasets::synthetic_process(name, n, *nv, onset, *fault, *seed);
            // n_base = 300: exactly the normal-operation window before fault onset.
            LoadedDataset {
                name: (*name).to_string(),
                kind: "synthetic".into(),
                matrix: m,
                n_base: 300,
            }
        })
        .collect()
}

/// Compute the current UTC wall-clock time as two formatted strings.
///
/// Returns `(compact_stamp, iso_stamp)` where:
/// - `compact_stamp` is `"YYYYMMDD-HHmmss"` — used as the output directory name so runs are
///   lexicographically sortable on disk.
/// - `iso_stamp` is `"YYYY-MM-DDTHH:MM:SSZ"` — written into `manifest.json` for human readers.
///
/// No time-zone crate dependency: the civil date is derived from the Unix epoch using
/// Howard Hinnant's public-domain algorithm (integer-only, no lookup tables).
fn timestamp_utc() -> (String, String) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    // Howard Hinnant's algorithm: convert Unix days to (year, month, day).
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month prime
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    let stamp = format!("{:04}{:02}{:02}-{:02}{:02}{:02}", year, m, d, h, mi, s);
    let iso = format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, m, d, h, mi, s);
    (stamp, iso)
}

/// Print a human-readable summary of the chemometric detector-atlas corpus to stdout.
///
/// Loads the embedded corpus (YAML/TOML bytes compiled into the binary), validates it, and
/// prints counts and the corpus SHA-256 digest. Returns exit code 0 on success, 1 on error.
/// This is the `atlas` CLI sub-command.
pub fn run_atlas() -> i32 {
    match Corpus::load().and_then(|c| c.validate().map(|v| (c, v))) {
        Ok((c, v)) => {
            println!("chemometric detector atlas: {}", c.corpus_name);
            println!("  detectors      : {}", v.n_total);
            println!("  executed       : {}", v.n_executed);
            println!("  catalogued     : {}", v.n_catalogued);
            println!("  primitive fams : {}", v.n_primitive_families);
            println!("  corpus sha256  : {}", v.corpus_sha256);
            // Authority crate: the curated detector/heuristic records the edge pipeline is bound to.
            match dsfb_chemical_engineering_atlas::validate() {
                Ok(a) => {
                    println!("authority crate : dsfb-chemical-engineering-atlas");
                    println!("  detectors      : {} (executed records)", a.n_detectors);
                    println!(
                        "  heuristics     : {} (H1-H6 process records)",
                        a.n_heuristics
                    );
                    println!(
                        "  fault sigs     : {} ({} executed, {} catalogued)",
                        a.n_fault_signatures,
                        a.n_fault_signatures_executed,
                        a.n_fault_signatures - a.n_fault_signatures_executed
                    );
                    println!(
                        "  atlas_hash_v1  : {}",
                        dsfb_chemical_engineering_atlas::hashes::atlas_hash_v1()
                    );
                    0
                }
                Err(e) => {
                    eprintln!("atlas authority validation FAILED: {e}");
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("atlas validation FAILED: {e}");
            1
        }
    }
}

/// Print the soft-sensor data-corpus authority summary (dataset counts + frozen `corpus_hash_v1`).
///
/// The corpus authority crate is an *optional* dependency (Cargo feature `soft-sensor-corpus`, off
/// by default so the process-monitoring build stays lean — the corpus is dataset-provenance
/// authority metadata, never on the residual/replay path). Built WITHOUT the feature this prints
/// how to enable it; built WITH it, it validates the catalogue and prints the frozen hash, so the
/// edge binary is demonstrably bound to the same dataset authority the paper and tests cite.
#[cfg(feature = "soft-sensor-corpus")]
pub fn run_corpus() -> i32 {
    match dsfb_chemical_engineering_corpus::validate() {
        Ok(r) => {
            let c = dsfb_chemical_engineering_corpus::census();
            println!(
                "authority crate : dsfb-chemical-engineering-corpus (soft-sensor data catalogue)"
            );
            println!("  datasets       : {}", r.n_datasets);
            println!("  cheap-sensor   : {}", r.n_cheap_sensor);
            println!("  process domains: {}", r.n_domains);
            println!("  w/ fault labels: {}", r.n_with_fault_labels);
            // P53 provenance-classification census (honest disclosure; each axis partitions all records).
            println!(
                "  licence tiers  : explicit-open {} · copyleft {} · stated-verify {} · research-use {} · agreement {}",
                c.lic_explicit_open, c.lic_explicit_copyleft, c.lic_stated_needs_verification,
                c.lic_research_use_customary, c.lic_agreement_governed
            );
            println!(
                "  access tiers   : open-confirmed {} · open-mirror {} · account {} · code-gen {} · agreement {}",
                c.acc_open_confirmed, c.acc_open_mirror_unverified, c.acc_account_required,
                c.acc_generated_by_code, c.acc_agreement_required
            );
            println!(
                "  redistribution : permits-attr {} · copyleft-SA {} · verify-first {} · prohibited {}",
                c.redist_permits_attribution, c.redist_copyleft_share_alike,
                c.redist_verify_before, c.redist_prohibited_by_agreement
            );
            println!(
                "  authority kinds: DOI {} · curated-ML {} · package {} · simulator {} · testbed {} · community {} · vendor-host {}",
                c.auth_doi_archive, c.auth_curated_ml_repository, c.auth_package_distribution,
                c.auth_simulator_codebase, c.auth_governed_testbed, c.auth_community_upload,
                c.auth_author_or_vendor_host
            );
            println!(
                "  corpus_hash_v1 : {}",
                dsfb_chemical_engineering_corpus::corpus_hash_v1()
            );
            0
        }
        Err(e) => {
            eprintln!("corpus authority validation FAILED: {e}");
            1
        }
    }
}

/// Stub for when the optional `soft-sensor-corpus` feature is not compiled in (see the feature-gated
/// twin above). Prints the exact command that enables the corpus authority and exits success, so
/// `corpus` is always a recognised command rather than an "unknown command" error.
#[cfg(not(feature = "soft-sensor-corpus"))]
pub fn run_corpus() -> i32 {
    println!("soft-sensor data-corpus authority is an optional feature (off by default).");
    println!("enable it with:");
    println!(
        "  cargo run -p dsfb-chemical-engineering-edge --features soft-sensor-corpus -- corpus"
    );
    0
}

/// Run the `ArtifactCompletenessCourtV1` and print its report; exit non-zero if any check fails.
///
/// Gates that the committed artifact graph is complete + mutually consistent (manifest ↔ digests ↔
/// bundle/evidence roots ↔ authority hashes ↔ metrics protocol). A release blocker: a failing court
/// means a dataset, digest, sealed root, or authority hash is missing or inconsistent.
pub fn run_completeness_court(crate_dir: &std::path::Path) -> i32 {
    let report = crate::completeness::run_completeness_court(crate_dir);
    println!("ArtifactCompletenessCourtV1");
    print!("{}", report.render());
    if report.all_passed() {
        0
    } else {
        eprintln!("ARTIFACT COMPLETENESS FAILED — see findings above");
        1
    }
}

/// `verify-index` (P98): run the [`crate::index_court`] over the committed `reports/index.json`, proving
/// the generated artifact index still reflects the live artifacts (root re-derives, paper/figure/bundle
/// digests match, doc + crate links resolve, policy + court hashes agree). A failing court is a release
/// blocker, so this exits non-zero on any failed check.
pub fn run_verify_index(crate_dir: &std::path::Path) -> i32 {
    // repo root = <repo>/crates/dsfb-chemical-engineering-edge → parent().parent(); same resolution as
    // run_generate_index, with a sane fallback so the command never panics on an unexpected layout.
    let repo_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| crate_dir.to_path_buf());
    let report = crate::index_court::run_index_court(crate_dir, &repo_root);
    println!("ArtifactIndexCourtV1");
    print!("{}", report.render());
    if report.all_passed() {
        0
    } else {
        eprintln!("INDEX VERIFICATION FAILED — the committed reports/index.json is stale; run `generate-index`");
        1
    }
}

/// Run the full demo pipeline and write a timestamped artifact bundle.
///
/// ## Artifact layout
/// ```text
/// output-dsfb-chemical-engineering/<YYYYMMDD-HHmmss>/
///   manifest.json                  — run metadata + per-dataset replay hashes
///   metrics.csv                    — one row per dataset, all Metrics fields
///   replay_hashes.json             — dataset → hash (machine-verifiable)
///   report.md                      — human-readable summary table + non-claims
///   artifact_bundle.zip            — store-only ZIP of all the above
///   figures/trace_data/
///     trace_<dataset>.csv          — SPE, T², episode mask, onset marker
///     grammar_<dataset>_pca_spe_q.csv — per-step grammar token for SPE detector
///   datasets/<name>/
///     detector_outputs.csv         — raw score + breach flag per (detector, sample)
///     residual_streams.csv         — DSFB triple (r, delta, sigma) + grammar state per step
///     dsfb_episodes.csv            — fused structural episodes
///     heuristic_labels.csv         — per-episode heuristic label or "unknown"
///     result.json                  — full AnalysisResult as JSON
/// ```
///
/// Returns exit code 0 (all replay hashes verified), 1 (atlas load error), or 2 (any replay FAIL).
pub fn run_demo(crate_dir: &Path) -> i32 {
    // Validate the corpus before running: a malformed atlas is a hard error, not a warning.
    let corpus = match Corpus::load().and_then(|c| c.validate().map(|v| (c, v))) {
        Ok(cv) => cv,
        Err(e) => {
            eprintln!("atlas validation FAILED: {e}");
            return 1;
        }
    };
    let (_, cval) = &corpus;

    // Dataset-provenance gate (Rust-side, not just the notebook): verify each committed slice against
    // its SHA-256 in MANIFEST.toml before analysing anything. A mismatch means a slice's bytes changed
    // vs. the recorded provenance — abort rather than silently produce results over altered data.
    match datasets::verify_manifest_sha256(crate_dir) {
        Ok(g) => {
            println!(
                "manifest SHA-256 gate: {} verified, {} missing, {} mismatched",
                g.verified, g.missing, g.mismatched
            );
            if g.mismatched > 0 {
                eprintln!(
                    "PROVENANCE GATE FAILED — slices changed vs MANIFEST.toml: {}",
                    g.mismatched_names.join(", ")
                );
                return 1;
            }
        }
        Err(e) => eprintln!("manifest SHA-256 gate skipped: {e}"),
    }

    let datasets = discover_datasets(crate_dir);
    println!("== DSFB-Chemical-Engineering edge demo ==");
    println!(
        "atlas: {} detectors ({} executed); {} datasets to analyse",
        cval.n_total,
        cval.n_executed,
        datasets.len()
    );

    let (stamp, iso) = timestamp_utc();
    // Place output under the workspace root (two levels above the crate dir) so it is accessible
    // regardless of which `target/` profile is active.
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let out_root = workspace_root
        .join("output-dsfb-chemical-engineering")
        .join(&stamp);
    let fig_dir = out_root.join("figures/trace_data");
    std::fs::create_dir_all(&fig_dir).ok();

    let cfg = PipelineConfig::default();
    let mut results: Vec<AnalysisResult> = Vec::new();
    let mut manifest_entries: Vec<DatasetEntry> = Vec::new();

    // Artifact-write failures are tracked (not silently swallowed): a single bad write must not abort the
    // whole run, but the operator must learn the bundle is incomplete. `track!(expr, "what")` records any
    // Err with its context; the count is reported at the end and folded into the exit code. (Previously
    // these were `.ok()` — a full disk / permission error produced a malformed bundle with no warning.)
    let mut write_failures: Vec<String> = Vec::new();
    macro_rules! track {
        ($e:expr, $what:expr) => {
            if let Err(e) = $e {
                write_failures.push(format!("{}: {}", $what, e));
            }
        };
    }

    for d in &datasets {
        // `analyze` and `timelines_for` both run the pipeline; the duplication is accepted so
        // that the serialisable AnalysisResult stays clean (no raw timelines field) while still
        // allowing detailed per-step CSV export.
        let res = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        let timelines: Vec<DetectorTimeline> =
            pipeline::timelines_for(&d.name, &d.matrix, d.n_base, cfg);

        let ds_dir = out_root.join("datasets").join(&d.name);
        track!(
            std::fs::create_dir_all(&ds_dir),
            format!("mkdir {}", ds_dir.display())
        );

        // Per-dataset CSVs — errors are non-fatal (a single bad write does not abort the whole demo run)
        // but are *tracked* (see `track!`) so a malformed bundle is reported, never silently produced.
        track!(
            report::write_detector_outputs(
                &ds_dir.join("detector_outputs.csv"),
                &d.name,
                &timelines
            ),
            format!("{}/detector_outputs.csv", d.name)
        );
        track!(
            report::write_residual_streams(
                &ds_dir.join("residual_streams.csv"),
                &d.name,
                &timelines
            ),
            format!("{}/residual_streams.csv", d.name)
        );
        track!(
            report::write_episodes(&ds_dir.join("dsfb_episodes.csv"), &d.name, &res),
            format!("{}/dsfb_episodes.csv", d.name)
        );
        track!(
            report::write_heuristic_labels(&ds_dir.join("heuristic_labels.csv"), &d.name, &res),
            format!("{}/heuristic_labels.csv", d.name)
        );
        track!(
            report::write_episode_evidence(
                &ds_dir.join("episode_evidence.csv"),
                &d.name,
                &res,
                &timelines
            ),
            format!("{}/episode_evidence.csv", d.name)
        );
        track!(
            report::write_ne107(&ds_dir, &d.name, &res, &timelines),
            format!("{}/ne107", d.name)
        );
        track!(
            report::write_alarm_rationalization(
                &ds_dir.join("alarm_rationalization.csv"),
                &d.name,
                &res
            ),
            format!("{}/alarm_rationalization.csv", d.name)
        );
        track!(
            report::write_operator_report_html(&ds_dir.join("operator_report.html"), &d.name, &res),
            format!("{}/operator_report.html", d.name)
        );
        track!(
            report::write_residual_provenance(
                &ds_dir.join("residual_provenance_ledger.csv"),
                &d.name,
                &timelines,
                d.n_base
            ),
            format!("{}/residual_provenance_ledger.csv", d.name)
        );
        track!(
            report::write_non_admission(
                &ds_dir.join("non_admission.csv"),
                &d.name,
                &crate::fusion::non_admitted_runs(&timelines, cfg.fusion)
            ),
            format!("{}/non_admission.csv", d.name)
        );
        track!(
            report::write_contribution_traces(
                &ds_dir.join("contribution_traces.csv"),
                &d.name,
                &res,
                &d.matrix,
                d.n_base,
                5
            ),
            format!("{}/contribution_traces.csv", d.name)
        );
        track!(
            report::write_result_json(&ds_dir.join("result.json"), &res),
            format!("{}/result.json", d.name)
        );
        // Canonical Chemical Court Record bundle (dsfb_chemical_engineering_casefile_v1/). Additive:
        // its fields are not part of canonical_replay_hash, so emitting it leaves replay byte-identical.
        track!(
            crate::court_record::write_court_record(
                &ds_dir, &res, &timelines, d.n_base, cfg.fusion
            ),
            format!("{}/court_record", d.name)
        );

        // Figure traces: recover the SPE/T² series from the detector outputs by ID rather than
        // recomputing PCA. The `pca_spe_q` and `pca_t2` detectors emit the values as `raw_score`.
        let mask = figures::episode_mask(d.matrix.n_samples, &res.fused_episodes);
        let spe = recover_series(&timelines, "pca_spe_q");
        let t2 = recover_series(&timelines, "pca_t2");
        track!(
            figures::export_trace(&fig_dir, &d.name, &spe, &t2, &mask, d.matrix.fault_onset),
            format!("{}/trace", d.name)
        );
        // Grammar timeline export: only for the SPE detector (representative; others available
        // via residual_streams.csv if needed).
        if let Some(tl) = timelines.iter().find(|t| t.detector_id == "pca_spe_q") {
            track!(
                figures::export_grammar_timeline(&fig_dir, &d.name, tl),
                format!("{}/grammar_trace", d.name)
            );
        }

        println!(
            "  [{}] {:<22} samples={:<6} vars={:<4} PCs={:<2} episodes={:<3} compression={:.2}x unknown={:.0}% replay={}",
            d.kind,
            d.name,
            res.metrics.n_samples,
            res.metrics.n_vars,
            res.pca_components,
            res.metrics.fused_episodes,
            res.metrics.episode_compression_ratio,
            res.metrics.unknown_rate * 100.0,
            if res.metrics.replay_deterministic { "OK" } else { "FAIL" }
        );

        manifest_entries.push(DatasetEntry {
            name: d.name.clone(),
            kind: d.kind.clone(),
            replay_hash: res.replay_hash.clone(),
            fused_episodes: res.metrics.fused_episodes,
            unknown_rate: res.metrics.unknown_rate,
            episode_compression_ratio: res.metrics.episode_compression_ratio,
        });
        results.push(res);
    }

    // Run-level artifacts written after all datasets complete.
    let manifest = RunManifest {
        tool: "dsfb-chemical-engineering-edge".into(),
        crate_version: crate::VERSION.into(),
        timestamp_utc: iso,
        corpus_sha256: cval.corpus_sha256.clone(),
        corpus_detectors: cval.n_total,
        corpus_executed: cval.n_executed,
        datasets: manifest_entries,
    };
    track!(
        std::fs::write(
            out_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap()
        ),
        "manifest.json"
    );
    track!(
        report::write_replay_hashes(&out_root.join("replay_hashes.json"), &results),
        "replay_hashes.json"
    );
    track!(
        report::write_summary_md(&out_root.join("report.md"), &results),
        "report.md"
    );
    track!(
        write_metrics_csv(&out_root.join("metrics.csv"), &results),
        "metrics.csv"
    );

    // Bundle: collect_files sorts entries, ensuring the ZIP is byte-for-byte identical on
    // re-runs over the same artifact directory (store-only ZIP, no compression timestamps).
    let files = report::collect_files(&out_root, &out_root);
    let zip_path = out_root.join("artifact_bundle.zip");
    track!(report::zip_store(&zip_path, &files), "artifact_bundle.zip");

    println!("\nartifacts: {}", out_root.display());
    println!("bundle   : {}", zip_path.display());
    // Surface any artifact-write failures (a malformed/incomplete bundle must never be silent).
    if !write_failures.is_empty() {
        eprintln!(
            "\nWARNING: {} artifact write(s) FAILED — the bundle is incomplete:",
            write_failures.len()
        );
        for f in &write_failures {
            eprintln!("  - {f}");
        }
    }
    // Exit code 2 signals replay failure to CI/testing harnesses; 3 signals incomplete artifacts;
    // normal success is 0.
    let all_ok = results.iter().all(|r| r.metrics.replay_deterministic);
    if !all_ok {
        2
    } else if !write_failures.is_empty() {
        3
    } else {
        0
    }
}

/// Run the `PublicReleaseScrubCourtV1` over the repository and print its report; exit non-zero if dirty.
///
/// Public-release hygiene gate (distinct from the completeness court): scans the tracked text surface for
/// placeholder DOIs, private-backup leakage, controlled-access dataset rows, and missing trust artifacts.
/// `crate_dir` is the edge crate; the repo root is two levels up.
pub fn run_release_scrub(crate_dir: &Path) -> i32 {
    let repo_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let report = crate::release_scrub::run_release_scrub(repo_root);
    println!("PublicReleaseScrubCourtV1");
    print!("{}", report.render());
    if report.all_passed() {
        0
    } else {
        eprintln!("RELEASE SCRUB FAILED — the tracked release surface is not fit to publish (see findings)");
        1
    }
}

/// P81 — archive-mode scrub: run the public-release hygiene gate over a *materialised release directory*
/// (`--archive-dir <path>`), the artifact a user actually uploads. Fails hard if a private session backup
/// smuggled in or the hygiene config is missing — the raw-zip leak the panel flagged. Run before publishing.
pub fn run_release_scrub_archive(archive_dir: &str) -> i32 {
    let dir = Path::new(archive_dir);
    if !dir.is_dir() {
        eprintln!("release-scrub --archive-dir: '{archive_dir}' is not a directory");
        return 1;
    }
    let report = crate::release_scrub::run_release_scrub_archive(dir);
    println!("PublicReleaseScrubCourtV1 (archive mode) — {archive_dir}");
    print!("{}", report.render());
    if report.all_passed() {
        0
    } else {
        eprintln!(
            "ARCHIVE SCRUB FAILED — this staged archive is not fit to publish (see findings)"
        );
        1
    }
}

/// `authority-diff old.json new.json` (Wave-2 P79) — diff two authority snapshots and emit governed-drift
/// receipts. Each input is a JSON object mapping a stable record id → its content hash (the snapshot shape
/// `migration` diffs over — e.g. detector-registry or heuristic-bank id→hash). Prints a `SemanticDiffReportV1`
/// (added / removed / changed / unchanged) and a `DetectorRegistryCompatibilityV1` verdict (backward-compatible
/// iff nothing was removed), each hash-sealed. Returns 0 on success, 1 on a read/parse error.
pub fn run_authority_diff(old_path: &str, new_path: &str) -> i32 {
    use crate::migration::{DetectorRegistryCompatibilityV1, RecordSnapshot, SemanticDiffReportV1};
    let load = |p: &str| -> Result<RecordSnapshot, String> {
        let txt = std::fs::read_to_string(p).map_err(|e| format!("read {p}: {e}"))?;
        serde_json::from_str::<RecordSnapshot>(&txt)
            .map_err(|e| format!("parse {p} (expected a JSON object of id→hash): {e}"))
    };
    let (from, to) = match (load(old_path), load(new_path)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("authority-diff: {e}");
            return 1;
        }
    };
    let diff = SemanticDiffReportV1::diff(old_path, new_path, &from, &to);
    let compat = DetectorRegistryCompatibilityV1::check(old_path, new_path, &from, &to);
    println!("SemanticDiffReportV1 ({} → {})", old_path, new_path);
    println!(
        "  added   ({}): {}",
        diff.added.len(),
        diff.added.join(", ")
    );
    println!(
        "  removed ({}): {}",
        diff.removed.len(),
        diff.removed.join(", ")
    );
    println!(
        "  changed ({}): {}",
        diff.changed.len(),
        diff.changed.join(", ")
    );
    println!("  unchanged: {}", diff.unchanged);
    println!("  diff_hash: {}", diff.diff_hash);
    println!(
        "DetectorRegistryCompatibilityV1: backward_compatible = {} ({})",
        compat.backward_compatible,
        if compat.backward_compatible {
            "nothing relied-upon was removed"
        } else {
            "a record was REMOVED — downstream callers may break"
        }
    );
    println!("  report_hash: {}", compat.report_hash);
    0
}

/// `unit-consistency` (Wave-3 physics) — run the [`crate::unit_consistency::UnitConsistencyCourtV1`] over
/// every instrumented demonstrator's documented balance, proving the channels each balance adds/subtracts
/// carry identical units (so a unit bug — °C vs K, bar vs Pa, a level in cm vs m — never reads out as a
/// process anomaly). Loads each `data/instrumented/*.roles.json`, derives the same-unit assertions implied
/// by its balance type ([`crate::unit_consistency::assertions_for_balance`]), and prints a sealed verdict
/// per dataset. Read-only; touches no frozen artifact. Exit 0 if every balance is unit-consistent, else 2.
pub fn run_unit_consistency(crate_dir: &Path) -> i32 {
    use crate::unit_consistency::{assertions_for_balance, UnitConsistencyCourtV1};
    let dir = crate_dir.join("data").join("instrumented");
    let mut roles_files: Vec<PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.to_string_lossy().ends_with(".roles.json"))
            .collect(),
        Err(e) => {
            eprintln!("unit-consistency: read {}: {e}", dir.display());
            return 1;
        }
    };
    roles_files.sort(); // deterministic order regardless of filesystem enumeration
    println!("UnitConsistencyCourtV1 — units of every documented balance");
    let mut any_hazard = false;
    let mut checked = 0usize;
    for rf in &roles_files {
        let roles = match crate::balance::RolesDoc::load(rf) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("unit-consistency: {e}");
                return 1;
            }
        };
        let asserts = assertions_for_balance(&roles);
        if asserts.is_empty() {
            continue; // no additively-combined channel group to check for this balance type
        }
        let court = UnitConsistencyCourtV1::build(&asserts);
        debug_assert!(court.verify());
        checked += 1;
        any_hazard |= !court.all_consistent();
        let btype = roles
            .balance
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        println!("\n[{}]  balance={}", roles.dataset, btype);
        print!("{}", court.render());
    }
    println!(
        "\noverall: {} ({} balances checked)",
        if any_hazard {
            "UNIT-HAZARD FOUND"
        } else {
            "ALL BALANCES UNIT-CONSISTENT"
        },
        checked
    );
    if any_hazard {
        2
    } else {
        0
    }
}

/// `data-readiness <csv>` (Wave-4 historian layer) — grade a real historian-export CSV with the
/// [`crate::data_readiness::IndustrialDataReadinessCourtV1`] *before* analysis. This is the real-data
/// drop-in path: any role-labelled or plain wide CSV runs through unchanged. The profile is derived honestly
/// from what a bare values file can show (tag/row counts, missingness) and from a sibling `<stem>.roles.json`
/// when present (unit coverage, controller context); fields a values file cannot show (timestamp span,
/// duplicate timestamps) are not assessed and surface as caveats — never optimistically assumed. Licence
/// clearance is taken as true because the operator is running it locally on their own data (the whole point
/// of the local-evaluation design); that assumption is printed. Exit 0 if evaluable, else 2.
pub fn run_data_readiness(crate_dir: &Path, csv_arg: &str) -> i32 {
    use crate::data_readiness::{HistorianExportProfile, IndustrialDataReadinessCourtV1};
    let _ = crate_dir;
    let path = PathBuf::from(csv_arg);
    // The generic wide-CSV loader (same one `analyze`/`demo` use) — a plain role-labelled or wide historian
    // export loads unchanged. A `timestamp`-keyed historian CSV is also accepted via the historian loader.
    let (m, n_base) = match datasets::load_csv_slice(&path, 0.4)
        .or_else(|_| datasets::load_historian_csv(&path, 0.4))
    {
        Ok(x) => x,
        Err(e) => {
            eprintln!("data-readiness: {e}");
            return 1;
        }
    };
    // Derive what a values matrix can show.
    let total_cells = (m.n_samples * m.n_vars).max(1);
    let n_nonfinite = m.data.iter().filter(|x| !x.is_finite()).count();
    let missingness_fraction = n_nonfinite as f64 / total_cells as f64;

    // A sibling `<stem>.roles.json`, if present, unlocks unit coverage + controller context.
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let roles_path = path.with_file_name(format!("{stem}.roles.json"));
    let (unit_coverage_fraction, has_controllers, roles_note) =
        match crate::balance::RolesDoc::load(&roles_path) {
            Ok(roles) if !roles.variables.is_empty() => {
                let with_unit = roles
                    .variables
                    .iter()
                    .filter(|v| !v.unit.trim().is_empty())
                    .count();
                (
                    with_unit as f64 / roles.variables.len() as f64,
                    !roles.manipulated().is_empty(),
                    "roles sidecar found (units + controller context derived)",
                )
            }
            _ => (
                0.0,
                false,
                "no roles sidecar (units + controller context unknown ⇒ caveats)",
            ),
        };

    let profile = HistorianExportProfile {
        n_tags: m.n_vars,
        n_rows: m.n_samples,
        baseline_window_present: n_base >= 2,
        license_cleared: true, // local run on the operator's own data (printed assumption)
        missingness_fraction,
        unit_coverage_fraction,
        duplicate_timestamp_fraction: 0.0, // not assessable from a values matrix (no timestamps)
        time_span_hours: f64::NAN, // not assessable from a values matrix ⇒ caveat, not assumed
        sampling_regular: true, // assumed; the historian loader handles ragged sampling separately
        has_controllers,
        has_maintenance: false,
        has_batches: m.phase.is_some(), // batch/phase labels in the CSV unlock batch-phase envelopes
        has_lab_samples: false,
    };
    let court = IndustrialDataReadinessCourtV1::grade(&profile);
    println!("IndustrialDataReadinessCourtV1 — {}", path.display());
    println!(
        "derived: {} tags × {} rows, {:.1}% missing; {}",
        profile.n_tags,
        profile.n_rows,
        100.0 * missingness_fraction,
        roles_note
    );
    println!("assumed: licence_cleared=true (local run on your own data); time-span + duplicate-timestamps not assessed from a bare values file");
    print!("{}", court.render());
    debug_assert!(court.verify());
    if court.is_evaluable() {
        0
    } else {
        eprintln!("data-readiness: NOT READY — supply the missing critical witnesses above before evaluating");
        2
    }
}

/// Newest immediate subdirectory of `dir` by name (the run timestamps sort lexicographically = chronologically).
fn newest_subdir(dir: &Path) -> Option<PathBuf> {
    let mut subs: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    subs.sort();
    subs.pop()
}

/// Run the full figure-generation campaign (the `figures` command) — one of the two figure-producing
/// methods (the other is the Colab notebook). It does NOT build the paper.
///
/// ## Pipeline
/// 1. Refresh the standard demo data (per-dataset CSVs, trace data, metrics) by reusing [`run_demo`].
/// 2. Export the evidence-object figure data the demo does not produce ([`crate::figure_export`]).
/// 3. Render the full figure gallery via the bundled Python renderer (`python3 -m figures`), which writes
///    PNG+PDF to `paper/figures/` plus a `figure_manifest.json` (id → source → caption → sha256).
/// 4. Capture a verbose `figure_build_log.txt` (every step + the renderer's own output) and pack a
///    deterministic ZIP (`figures_bundle.zip`) of the figures + manifest + log.
///
/// Degrades gracefully (logged) if `python3`/matplotlib is unavailable: the figure DATA + DOT + log are
/// still written, only the raster/vector rendering is skipped. Off the replay path; additive.
///
/// Returns 0 on success, 1 if the demo data could not be produced, 3 if the renderer failed/was absent.
pub fn run_figures(crate_dir: &Path) -> i32 {
    use std::io::Write as _;
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let mut log: Vec<String> = Vec::new();
    macro_rules! note { ($($a:tt)*) => {{ let s = format!($($a)*); println!("{s}"); log.push(s); }} }

    note!("== DSFB-Chemical-Engineering figure campaign ==");
    note!("step 1/4 — refreshing demo data (per-dataset CSVs, trace data, metrics)");
    if run_demo(crate_dir) == 1 {
        eprintln!("figure campaign aborted: demo data could not be produced");
        return 1;
    }
    // Regime comparison (global vs regime-conditioned envelopes) → reports/regime_comparison.csv, which the
    // group-H figures read. Non-fatal: a regime-eval hiccup must not abort the whole figure campaign.
    note!(
        "  + regime-eval (global vs regime-conditioned envelopes → reports/regime_comparison.csv)"
    );
    let _ = run_regime_eval(crate_dir);

    let runs_root = workspace_root.join("output-dsfb-chemical-engineering");
    let run = match newest_subdir(&runs_root) {
        Some(r) => r,
        None => {
            eprintln!("no demo run directory found under {}", runs_root.display());
            return 1;
        }
    };
    note!("using run: {}", run.display());

    note!("step 2/4 — exporting evidence-object figure data (representative instances)");
    for line in crate::figure_export::export_all(&run) {
        println!("{line}");
        log.push(line);
    }

    note!("step 3/4 — rendering the figure gallery (Python/matplotlib + graphviz)");
    // Flush the log so far; the Python renderer appends its per-figure lines to the same file.
    let log_path = run.join("figure_build_log.txt");
    std::fs::write(&log_path, log.join("\n") + "\n").ok();
    let scripts_dir = crate_dir.join("scripts");
    let render = std::process::Command::new("python3")
        .args(["-m", "figures", "--run"])
        .arg(&run)
        .arg("--log")
        .arg(&log_path)
        .current_dir(&scripts_dir)
        .output();
    match render {
        Ok(out) => {
            print!("{}", String::from_utf8_lossy(&out.stdout));
            if !out.status.success() {
                eprintln!(
                    "renderer exited non-zero — figures may be incomplete; see {}",
                    log_path.display()
                );
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
            }
        }
        Err(e) => {
            let msg = format!("python3 renderer unavailable ({e}); skipped rendering. Figure DATA + DOT + build log were still written.");
            eprintln!("{msg}");
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&log_path) {
                writeln!(f, "{msg}").ok();
            }
            return 3;
        }
    }

    note!("step 4/4 — packing the deterministic figure ZIP");
    let figdir = workspace_root.join("paper").join("figures");
    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&figdir) {
        let mut paths: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "png" || ext == "pdf" {
                if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                    entries.push((format!("figures/{n}"), p.clone()));
                }
            }
        }
    }
    let manifest = figdir.join("figure_manifest.json");
    if manifest.exists() {
        entries.push(("figure_manifest.json".into(), manifest));
    }
    if log_path.exists() {
        entries.push(("figure_build_log.txt".into(), log_path.clone()));
    }
    entries.sort();
    let zip_path = run.join("figures_bundle.zip");
    // Surface a zip failure rather than swallowing it: the success lines below would otherwise claim a
    // "bundle" that was never written (e.g. on a full disk).
    if let Err(e) = report::zip_store(&zip_path, &entries) {
        eprintln!(
            "figures: FAILED to write the deterministic ZIP {}: {e}",
            zip_path.display()
        );
        return 1;
    }

    println!("\nfigures : {}", figdir.display());
    println!("log     : {}", log_path.display());
    println!(
        "bundle  : {} ({} entries)",
        zip_path.display(),
        entries.len()
    );
    0
}

/// Write the Chemical Court Record v1 bundle for a single named dataset (the `casefile` command).
///
/// Discovers the dataset (committed slice or synthetic fallback), runs the pipeline, and emits
/// `dsfb_chemical_engineering_casefile_v1/` under
/// `output-dsfb-chemical-engineering-casefile/<name>/`. Prints the evidence root, the bundle root,
/// and a one-line summary. This is the canonical "court record" entry point: the framework does not
/// emit an alarm — it emits a replayable record of why an alarm-like structure was or was not admitted.
pub fn run_casefile(crate_dir: &Path, name: &str) -> i32 {
    let datasets = discover_datasets(crate_dir);
    let d = match datasets.iter().find(|d| d.name == name) {
        Some(d) => d,
        None => {
            eprintln!(
                "casefile: dataset '{name}' not found. Available: {}",
                datasets
                    .iter()
                    .map(|d| d.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return 1;
        }
    };
    let cfg = PipelineConfig::default();
    let res = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
    let timelines = pipeline::timelines_for(&d.name, &d.matrix, d.n_base, cfg);

    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let out_dir = workspace_root
        .join("output-dsfb-chemical-engineering-casefile")
        .join(&d.name);
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("casefile: cannot create {}: {e}", out_dir.display());
        return 1;
    }
    match crate::court_record::write_court_record(&out_dir, &res, &timelines, d.n_base, cfg.fusion)
    {
        Ok(bundle_root) => {
            let bundle = out_dir.join(crate::court_record::CASEFILE_FORMAT);
            println!(
                "Chemical Court Record v1 — {}",
                crate::court_record::KEY_STATEMENT
            );
            println!("dataset       : {} [{}]", d.name, d.kind);
            println!("admitted      : {} episodes", res.fused_episodes.len());
            println!(
                "unknown       : {} ({:.0}%)",
                res.metrics.unknown_episodes,
                res.metrics.unknown_rate * 100.0
            );
            println!("evidence_root : {}", res.replay_hash);
            println!("bundle_root   : {bundle_root}");
            println!("bundle        : {}", bundle.display());
            // Auto-emit the narration context (the citable-anchor vocabulary + binding contract) next to the
            // bundle. A SEPARATE standalone artifact, NOT an 11th CONTENT_FILE — so bundle_root is unaffected.
            let ctx = crate::narration_context::build_narration_context(&d.name, &res);
            let nmd = out_dir.join("narration_context.md");
            let njson = out_dir.join("narration_context.json");
            if let Err(e) = std::fs::write(&nmd, crate::narration_context::render_markdown(&ctx))
                .and_then(|_| std::fs::write(&njson, crate::narration_context::render_json(&ctx)))
            {
                eprintln!("casefile: WARNING — narration context write failed: {e}");
            } else {
                println!(
                    "narration     : {} (context_root {}…)",
                    nmd.display(),
                    &ctx.context_root[..12]
                );
            }
            0
        }
        Err(e) => {
            eprintln!("casefile: write failed: {e}");
            1
        }
    }
}

/// `narration-context <dataset>` (P-narration): emit the constrained-narration CONTEXT document for one dataset —
/// the citable evidence-anchor vocabulary (one per fused episode, with its claim tier) + the binding contract + the
/// non-claims, sealed by a deterministic `context_root`. The emitted document names no consumer and states no use;
/// it is a navigation/contract layer over the sealed Court Record (creates no evidence, off the replay path).
pub fn run_narration_context(crate_dir: &Path, name: &str) -> i32 {
    // Search the real datasets AND the synthetic suite (discover_datasets only falls back to synthetics when no
    // slices exist, so a synthetic name like `synth_wide_step` must be looked up explicitly).
    let mut datasets = discover_datasets(crate_dir);
    for s in synthetic_suite() {
        if !datasets.iter().any(|d| d.name == s.name) {
            datasets.push(s);
        }
    }
    let d = match datasets.iter().find(|d| d.name == name) {
        Some(d) => d,
        None => {
            eprintln!(
                "narration-context: dataset '{name}' not found. Available: {}",
                datasets
                    .iter()
                    .map(|d| d.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return 1;
        }
    };
    let cfg = PipelineConfig::default();
    let res = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
    let ctx = crate::narration_context::build_narration_context(&d.name, &res);
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let out_dir = workspace_root
        .join("output-dsfb-chemical-engineering-narration")
        .join(&d.name);
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!(
            "narration-context: cannot create {}: {e}",
            out_dir.display()
        );
        return 1;
    }
    let nmd = out_dir.join("narration_context.md");
    let njson = out_dir.join("narration_context.json");
    if let Err(e) = std::fs::write(&nmd, crate::narration_context::render_markdown(&ctx))
        .and_then(|_| std::fs::write(&njson, crate::narration_context::render_json(&ctx)))
    {
        eprintln!("narration-context: write failed: {e}");
        return 1;
    }
    println!("narration context — {} [{}]", d.name, d.kind);
    println!("  evidence_root : {}", res.replay_hash);
    println!("  context_root  : {}", ctx.context_root);
    println!(
        "  anchors       : {} citable evidence anchors",
        ctx.anchors.len()
    );
    println!(
        "  documents     : {}  +  {}",
        nmd.display(),
        njson.display()
    );
    0
}

/// List the synthetic fixtures `confidential-demo` accepts — every `.csv` under `data/instrumented/` and
/// `data/historian/` (names without `.csv`). Both are synthetic; this project never ships real plant data.
fn list_confidential_fixtures(crate_dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = ["instrumented", "historian"]
        .iter()
        .flat_map(|d| {
            std::fs::read_dir(crate_dir.join("data").join(d))
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .filter(|n| n.ends_with(".csv"))
                        .map(|n| n.trim_end_matches(".csv").to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Resolve a `confidential-demo` fixture name to its CSV, searching `data/instrumented/` then
/// `data/historian/` (the gated fault demonstrators live under instrumented; the historian fixtures under
/// historian). `None` if the name matches neither.
fn resolve_confidential_fixture(crate_dir: &Path, fixture: &str) -> Option<PathBuf> {
    ["instrumented", "historian"]
        .iter()
        .map(|d| {
            crate_dir
                .join("data")
                .join(d)
                .join(format!("{fixture}.csv"))
        })
        .find(|p| p.exists())
}

/// P84 — `confidential-demo [--fixture <name>]`: run the read-only court locally on a SYNTHETIC historian
/// fixture and emit ONLY a redacted, hash-linked evidence bundle — the Wave-6 confidential-evaluation chain
/// made concrete in one command (*the operator runs the court locally and shares only redacted, hash-linked
/// evidence — raw plant data never leaves their control*). The shareable dir holds sealed JSONs + a SUMMARY,
/// never a CSV / raw time-series. Bounded: a synthetic-fixture illustration of the protocol, not a certified
/// data-handling guarantee.
pub fn run_confidential_demo(crate_dir: &Path, fixture: &str) -> i32 {
    use crate::confidential_demo::{assemble, unit_class, EpisodeBrief};

    let csv = match resolve_confidential_fixture(crate_dir, fixture) {
        Some(p) => p,
        None => {
            eprintln!("confidential-demo: fixture '{fixture}' not found under data/instrumented/ or data/historian/");
            eprintln!(
                "  available fixtures: {}",
                list_confidential_fixtures(crate_dir).join(", ")
            );
            return 1;
        }
    };
    let (m, n_base) = match datasets::load_csv_slice(&csv, 0.4) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("confidential-demo: load {fixture}: {e}");
            return 1;
        }
    };
    let cfg = PipelineConfig::default();
    let res = pipeline::analyze(fixture, "synthetic/incident", &m, n_base, cfg);
    let timelines = pipeline::timelines_for(fixture, &m, n_base, cfg);

    let ws = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let out = ws
        .join("output-dsfb-chemical-engineering-confidential")
        .join(fixture);
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("confidential-demo: mkdir {}: {e}", out.display());
        return 1;
    }
    // (1) The operator's LOCAL full Court Record — kept local; only the redacted bundle below is shared.
    let bundle_root =
        match crate::court_record::write_court_record(&out, &res, &timelines, n_base, cfg.fusion) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("confidential-demo: court record: {e}");
                return 1;
            }
        };
    let casefile_dir = out.join(crate::court_record::CASEFILE_FORMAT);
    // (2) The casefile manifest's per-file hashes (proof links) → the tamper seal + audit trail.
    let cf: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(casefile_dir.join("casefile.json")).unwrap_or_default(),
    )
    .unwrap_or(serde_json::Value::Null);
    let file_hashes: Vec<(String, String)> = cf["files"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|f| {
                    Some((
                        f["name"].as_str()?.to_string(),
                        f["sha256"].as_str()?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    // (3) Tag roles → redaction rows. The real name is hashed away; only alias + coarse structure is shared.
    let roles_path = csv.with_file_name(format!("{fixture}.roles.json"));
    let roles_rows: Vec<(String, String, String)> =
        match crate::balance::RolesDoc::load(&roles_path) {
            Ok(r) if !r.variables.is_empty() => r
                .variables
                .iter()
                .map(|v| (v.name.clone(), unit_class(&v.unit), v.role.clone()))
                .collect(),
            _ => m
                .var_names
                .iter()
                .map(|n| (n.clone(), "other".to_string(), "measured".to_string()))
                .collect(),
        };
    // (4) Episode briefs (label → playbook class) + a counts/rates metrics summary (no raw signal).
    let episodes: Vec<EpisodeBrief> = res
        .fused_episodes
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let label = res
                .heuristic_labels
                .get(i)
                .map(|l| {
                    if l.matched {
                        l.episode_label.clone()
                    } else {
                        "unknown structural episode".to_string()
                    }
                })
                .unwrap_or_else(|| "unknown structural episode".to_string());
            EpisodeBrief {
                label,
                start: e.start_index,
                end: e.end_index,
            }
        })
        .collect();
    let metrics_summary = vec![
        (
            "fused_episodes".to_string(),
            res.fused_episodes.len() as f64,
        ),
        ("unknown_rate".to_string(), res.metrics.unknown_rate),
        (
            "baseline_fp_rate".to_string(),
            res.metrics.baseline_false_positive_rate,
        ),
        ("n_samples".to_string(), res.metrics.n_samples as f64),
    ];
    let vr_hash = std::fs::read(ws.join("reports/verification_report.md"))
        .map(|b| crate::hashing::sha256_hex(&b))
        .unwrap_or_default();

    let bundle = assemble(
        fixture,
        &episodes,
        &roles_rows,
        &file_hashes,
        &res.replay_hash,
        &bundle_root,
        &vr_hash,
        metrics_summary,
    );

    // (5) Write the SHAREABLE dir — sealed JSONs + a human SUMMARY. No CSV, no raw time-series.
    let share = out.join("confidential_evaluation_bundle");
    if let Err(e) = std::fs::create_dir_all(&share) {
        eprintln!("confidential-demo: mkdir share: {e}");
        return 1;
    }
    let wj = |name: &str, json: String| {
        let _ = std::fs::write(share.join(name), json);
    };
    wj(
        "redaction_map.json",
        serde_json::to_string_pretty(&bundle.redaction_map).unwrap_or_default(),
    );
    wj(
        "tamper_seal.json",
        serde_json::to_string_pretty(&bundle.tamper_seal).unwrap_or_default(),
    );
    wj(
        "audit_trail.json",
        serde_json::to_string_pretty(&bundle.audit_trail).unwrap_or_default(),
    );
    wj(
        "confidential_bundle.json",
        serde_json::to_string_pretty(&bundle.confidential_bundle).unwrap_or_default(),
    );
    wj(
        "escrow_protocol.json",
        serde_json::to_string_pretty(&bundle.escrow_protocol).unwrap_or_default(),
    );
    wj(
        "playbooks.json",
        serde_json::to_string_pretty(&bundle.playbooks).unwrap_or_default(),
    );

    let shareable = bundle.is_shareable();
    let summary = format!(
        "# Confidential evaluation bundle — {fixture}\n\n\
         The operator ran the read-only DSFB court LOCALLY on a synthetic historian fixture and shares ONLY\n\
         this redacted, hash-linked evidence — **no raw time-series leaves**.\n\n\
         - episodes: {}\n- evidence_root: `{}`\n- bundle_root: `{}`\n- redacted tags: {} (aliases only)\n\
         - contains_raw_timeseries: **{}**\n- shareable: **{}**\n- escrow valid (raw never egressed): **{}**\n\n\
         Files (all sealed, all alias/hash only): redaction_map.json · tamper_seal.json · audit_trail.json ·\n\
         confidential_bundle.json · escrow_protocol.json · playbooks.json. Bounded: a synthetic-fixture\n\
         illustration of the partner-evaluation protocol, not a certified data-handling guarantee.\n",
        bundle.confidential_bundle.n_episodes,
        res.replay_hash,
        bundle_root,
        bundle.redaction_map.entries.len(),
        bundle.confidential_bundle.contains_raw_timeseries,
        shareable,
        bundle.escrow_protocol.valid,
    );
    wj("SUMMARY.md", summary);

    println!(
        "ConfidentialEvaluationBundleV1 — operator shares redacted evidence; raw data never leaves"
    );
    println!("fixture        : {fixture} (synthetic; no real plant data)");
    println!("episodes       : {}", bundle.confidential_bundle.n_episodes);
    println!("evidence_root  : {}", res.replay_hash);
    println!("bundle_root    : {bundle_root}");
    println!(
        "redacted tags  : {} (aliases only; real names hashed away)",
        bundle.redaction_map.entries.len()
    );
    println!(
        "raw time-series shared: {}",
        bundle.confidential_bundle.contains_raw_timeseries
    );
    println!(
        "shareable      : {shareable}  ·  escrow valid: {}",
        bundle.escrow_protocol.valid
    );
    println!("bundle dir     : {}", share.display());
    if shareable {
        0
    } else {
        eprintln!(
            "confidential-demo: FAILED the shareability gate (a sealed object did not verify)"
        );
        1
    }
}

/// Extract a raw-score time series for the named detector from the timeline slice.
///
/// Used to recover SPE (`"pca_spe_q"`) and T² (`"pca_t2"`) series for figure export without
/// re-running the PCA computation. Returns an empty Vec if the detector id is not found —
/// downstream figure exporters handle missing data gracefully.
fn recover_series(timelines: &[DetectorTimeline], id: &str) -> Vec<f64> {
    timelines
        .iter()
        .find(|t| t.detector_id == id)
        .map(|t| t.outputs.iter().map(|o| o.raw_score).collect())
        .unwrap_or_default()
}

/// Write the per-run summary metrics table as `metrics.csv` (one row per dataset).
///
/// `detection_delay` is written as "na" when no fault onset is known (i.e., for datasets that
/// carry no `fault_onset` annotation). All float fields use 4 decimal places for readability;
/// the underlying `AnalysisResult` fields carry full precision.
fn write_metrics_csv(path: &Path, results: &[AnalysisResult]) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "kind",
        "n_samples",
        "n_vars",
        "n_baseline",
        "pca_components",
        "n_detectors",
        "raw_breach_steps",
        "fused_episodes",
        "episode_compression_ratio",
        "labeled_episodes",
        "unknown_episodes",
        "explanation_coverage",
        "unknown_rate",
        "detection_delay",
        "baseline_false_positive_rate",
        "replay_deterministic",
    ])?;
    for r in results {
        let m = &r.metrics;
        w.write_record([
            r.dataset.clone(),
            r.data_kind.clone(),
            m.n_samples.to_string(),
            m.n_vars.to_string(),
            m.n_baseline.to_string(),
            r.pca_components.to_string(),
            m.n_executed_detectors.to_string(),
            m.raw_breach_steps.to_string(),
            m.fused_episodes.to_string(),
            format!("{:.4}", m.episode_compression_ratio),
            m.labeled_episodes.to_string(),
            m.unknown_episodes.to_string(),
            format!("{:.4}", m.explanation_coverage),
            format!("{:.4}", m.unknown_rate),
            m.detection_delay
                .map(|d| d.to_string())
                .unwrap_or_else(|| "na".into()),
            format!("{:.4}", m.baseline_false_positive_rate),
            m.replay_deterministic.to_string(),
        ])?;
    }
    w.flush()
}

/// Run one named dataset through the full pipeline and print the metrics JSON to stdout.
///
/// Discovers datasets the same way `run_demo` does, then finds the first entry whose `name`
/// matches the argument. Prints the `Metrics` struct as pretty-printed JSON and the replay hash
/// on a separate line, then returns 0. Returns 1 and prints the available names if not found.
pub fn run_analyze(crate_dir: &Path, name: &str) -> i32 {
    let datasets = discover_datasets(crate_dir);
    match datasets.iter().find(|d| d.name == name) {
        Some(d) => {
            let res = pipeline::analyze(
                &d.name,
                &d.kind,
                &d.matrix,
                d.n_base,
                PipelineConfig::default(),
            );
            println!("{}", serde_json::to_string_pretty(&res.metrics).unwrap());
            println!("replay_hash: {}", res.replay_hash);
            0
        }
        None => {
            eprintln!("dataset '{name}' not found. available:");
            for d in &datasets {
                eprintln!("  {}", d.name);
            }
            1
        }
    }
}

/// Verify deterministic replay over the synthetic suite.
///
/// Runs `pipeline::analyze` twice on each synthetic dataset (same inputs, same config) and
/// compares the resulting `replay_hash` strings. Both calls must produce byte-identical hex
/// digests; any mismatch indicates a non-determinism bug in the pipeline.
///
/// Prints one line per dataset: name, "OK" or "FAIL", and the hash from the first run.
/// Returns exit code 0 (all pass) or 2 (any fail).
pub fn run_verify_replay() -> i32 {
    let suite = synthetic_suite();
    let cfg = PipelineConfig::default();
    let mut ok = true;
    for d in &suite {
        let a = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        let b = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        let pass = a.replay_hash == b.replay_hash;
        ok &= pass;
        println!(
            "{:<22} replay {}  {}",
            d.name,
            if pass { "OK  " } else { "FAIL" },
            a.replay_hash
        );
    }
    if ok {
        0
    } else {
        2
    }
}

/// `regime-eval` — A/B comparison of the global vs regime-conditioned admissibility envelope on
/// every discovered dataset that carries per-sample regime/phase labels (`DataMatrix::phase`).
///
/// For each such dataset `analyze` is run with `regime_conditioned = false` then `= true`; the
/// baseline false-positive rate, fused-episode count, unknown rate, and replay hash are reported for
/// both. The `true` config is run twice to confirm regime-conditioned replay is itself
/// deterministic. Datasets without a phase column are skipped (the envelope is identical either way,
/// so there is nothing to compare). Writes `<crate>/reports/regime_comparison.csv` — the
/// reproducible source for the paper's regime-conditioned-envelope result — and returns 0, or 2 if
/// any regime-on replay is non-deterministic.
pub fn run_regime_eval(crate_dir: &Path) -> i32 {
    use std::collections::BTreeSet;
    let datasets = discover_datasets(crate_dir);
    let phase_bearing: Vec<&LoadedDataset> = datasets
        .iter()
        .filter(|d| d.matrix.phase.is_some())
        .collect();
    if phase_bearing.is_empty() {
        eprintln!("regime-eval: no dataset carries a 'phase' column; nothing to compare.");
        return 0;
    }
    let off = PipelineConfig::default();
    let on = PipelineConfig {
        regime_conditioned: true,
        ..PipelineConfig::default()
    };
    let mut rows = vec![
        "dataset,n_regimes,baseline_fp_global,baseline_fp_regime,episodes_global,episodes_regime,\
         unknown_global,unknown_regime,replay_global,replay_regime,regime_deterministic"
            .to_string(),
    ];
    let mut all_det = true;
    for d in &phase_bearing {
        let r_off = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, off);
        let r_on = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, on);
        let r_on2 = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, on);
        // Regime-conditioned replay must itself be deterministic (two identical runs agree).
        let det = r_on.replay_hash == r_on2.replay_hash && r_on.metrics.replay_deterministic;
        all_det &= det;
        let n_reg = d
            .matrix
            .phase
            .as_ref()
            .map(|p| p.iter().collect::<BTreeSet<_>>().len())
            .unwrap_or(0);
        println!(
            "{:<26} baseline FP {:.3} -> {:.3} | episodes {} -> {} | unknown {:.3} -> {:.3} | {} regimes | det={}",
            d.name,
            r_off.metrics.baseline_false_positive_rate, r_on.metrics.baseline_false_positive_rate,
            r_off.metrics.fused_episodes, r_on.metrics.fused_episodes,
            r_off.metrics.unknown_rate, r_on.metrics.unknown_rate, n_reg, det,
        );
        rows.push(format!(
            "{},{},{:.6},{:.6},{},{},{:.6},{:.6},{},{},{}",
            d.name,
            n_reg,
            r_off.metrics.baseline_false_positive_rate,
            r_on.metrics.baseline_false_positive_rate,
            r_off.metrics.fused_episodes,
            r_on.metrics.fused_episodes,
            r_off.metrics.unknown_rate,
            r_on.metrics.unknown_rate,
            r_off.replay_hash,
            r_on.replay_hash,
            det,
        ));
    }
    let out_dir = crate_dir.join("reports");
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("regime-eval: cannot create {}: {e}", out_dir.display());
        return 1;
    }
    let path = out_dir.join("regime_comparison.csv");
    if let Err(e) = std::fs::write(&path, rows.join("\n") + "\n") {
        eprintln!("regime-eval: write {}: {e}", path.display());
        return 1;
    }
    println!("wrote {}", path.display());
    if all_det {
        0
    } else {
        2
    }
}

/// `historian <csv>` — replay a plant-historian export through the full pipeline.
///
/// Loads a long-format historian CSV (`timestamp,tag,value[,unit,quality,source,phase]`) via
/// [`datasets::load_historian_csv`], runs the deterministic pipeline (regime-conditioned automatically
/// when the export carried `phase`/`phase_id` batch labels), and writes the full per-dataset operator
/// bundle (episodes, heuristic labels + unknown subtypes, episode evidence, NE 107 views, alarm
/// rationalization, operator HTML, residual provenance, non-admission records, contribution traces,
/// result JSON), the canonical **Chemical Court Record v1** bundle, and --- when a `<name>.roles.json`
/// sidecar sits next to the CSV declaring a mass/energy balance --- a **balance witness**
/// (`balance_witness.csv` + baseline-vs-tail closure), under
/// `output-dsfb-chemical-engineering-historian/<name>/`. Prints metrics and the replay hash. One
/// command from a historian dump to a replayable, operator-readable case file. The long-format schema
/// (`timestamp,tag,value` + optional `quality,phase_id,unit,controller_mode,setpoint,
/// manipulated_variable`) is parsed by [`crate::datasets::load_historian_csv`].
pub fn run_historian(crate_dir: &Path, csv_arg: &str) -> i32 {
    let path = PathBuf::from(csv_arg);
    let (m, n_base) = match datasets::load_historian_csv(&path, 0.4) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("historian: {e}");
            return 1;
        }
    };
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "historian".into());
    // Regime-condition automatically when the historian carried batch/phase labels (batch-record mode).
    let cfg = PipelineConfig {
        regime_conditioned: m.phase.is_some(),
        ..PipelineConfig::default()
    };
    let res = pipeline::analyze(&name, "historian/replay", &m, n_base, cfg);
    let timelines = pipeline::timelines_for(&name, &m, n_base, cfg);
    let out = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|w| {
            w.join("output-dsfb-chemical-engineering-historian")
                .join(&name)
        })
        .unwrap_or_else(|| PathBuf::from("historian_out").join(&name));
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("historian: cannot create {}: {e}", out.display());
        return 1;
    }
    // Track every artifact write instead of swallowing its `io::Error` with `.ok()`: on a full disk (or a
    // read-only mount) a silent failure here would emit a *malformed* historian bundle — wrong byte content
    // under a citable name — which is exactly the kind of forensic artifact that must never fail quietly.
    // We mirror the demo path's `track!` discipline: collect failures, then report them and exit non-zero.
    let mut write_failures: Vec<String> = Vec::new();
    macro_rules! track {
        ($e:expr, $what:expr) => {
            if let Err(e) = $e {
                write_failures.push(format!("{}: {}", $what, e));
            }
        };
    }
    track!(
        report::write_episodes(&out.join("dsfb_episodes.csv"), &name, &res),
        "dsfb_episodes.csv"
    );
    track!(
        report::write_heuristic_labels(&out.join("heuristic_labels.csv"), &name, &res),
        "heuristic_labels.csv"
    );
    track!(
        report::write_episode_evidence(&out.join("episode_evidence.csv"), &name, &res, &timelines),
        "episode_evidence.csv"
    );
    track!(report::write_ne107(&out, &name, &res, &timelines), "ne107");
    track!(
        report::write_alarm_rationalization(&out.join("alarm_rationalization.csv"), &name, &res),
        "alarm_rationalization.csv"
    );
    track!(
        report::write_operator_report_html(&out.join("operator_report.html"), &name, &res),
        "operator_report.html"
    );
    track!(
        report::write_residual_provenance(
            &out.join("residual_provenance_ledger.csv"),
            &name,
            &timelines,
            n_base
        ),
        "residual_provenance_ledger.csv"
    );
    track!(
        report::write_non_admission(
            &out.join("non_admission.csv"),
            &name,
            &crate::fusion::non_admitted_runs(&timelines, cfg.fusion)
        ),
        "non_admission.csv"
    );
    track!(
        report::write_contribution_traces(
            &out.join("contribution_traces.csv"),
            &name,
            &res,
            &m,
            n_base,
            5
        ),
        "contribution_traces.csv"
    );
    track!(
        report::write_result_json(&out.join("result.json"), &res),
        "result.json"
    );
    // Canonical Chemical Court Record bundle for the historian run (same artifact a `casefile`/`demo`
    // run produces), so a plant historian export becomes one citable, replayable case file.
    track!(
        crate::court_record::write_court_record(&out, &res, &timelines, n_base, cfg.fusion),
        "court_record"
    );
    // Balance witness, when a roles sidecar sits next to the historian CSV (`<name>.roles.json`):
    // recompute the declared mass/energy-balance closure from the raw columns and write it as
    // historian evidence (this is the "balance witness if roles exist" path of the operator protocol).
    let roles_path = path.with_file_name(format!("{name}.roles.json"));
    if roles_path.exists() {
        match crate::balance::RolesDoc::load(&roles_path).and_then(|roles| {
            crate::balance::balance_residual(&m, &roles).map(|bres| (roles, bres))
        }) {
            Ok((roles, bres)) => {
                if let Ok(mut w) = csv::Writer::from_path(out.join("balance_witness.csv")) {
                    let _ = w.write_record(["dataset", "time_index", "balance_residual"]);
                    for (k, v) in bres.iter().enumerate() {
                        let _ = w.write_record([name.as_str(), &k.to_string(), &format!("{v:.6}")]);
                    }
                    let _ = w.flush();
                }
                // Baseline vs tail closure magnitude — a sustained shift is the witnessable break.
                let mean = |lo: usize, hi: usize| {
                    let v: Vec<f64> = (lo..hi.min(bres.len()))
                        .map(|k| bres[k].abs())
                        .filter(|x| x.is_finite())
                        .collect();
                    if v.is_empty() {
                        0.0
                    } else {
                        v.iter().sum::<f64>() / v.len() as f64
                    }
                };
                let (bm, fm) = (mean(1, n_base), mean(n_base, bres.len()));
                println!(
                    "balance witness ({}): closure |residual| baseline {:.3} -> tail {:.3} ({:.1}x)",
                    roles.balance.get("type").and_then(|v| v.as_str()).unwrap_or("?"),
                    bm, fm, fm / bm.max(1e-9)
                );
            }
            Err(e) => eprintln!("historian balance witness: {e}"),
        }
    }
    println!("{}", serde_json::to_string_pretty(&res.metrics).unwrap());
    println!("replay_hash: {}", res.replay_hash);
    println!("regime_conditioned: {}", cfg.regime_conditioned);
    println!("artifacts: {}", out.display());
    // A historian bundle with any failed write is malformed; surface it loudly and exit non-zero rather
    // than reporting success over an incomplete forensic artifact.
    if !write_failures.is_empty() {
        eprintln!(
            "historian: {} artifact write(s) FAILED — the bundle is incomplete:",
            write_failures.len()
        );
        for f in &write_failures {
            eprintln!("  - {f}");
        }
        return 1;
    }
    0
}

/// Resolve an instrumented-demonstrator CSV by name.
///
/// Controlled-access datasets (the real SWaT / BATADAL bytes, governed by the iTrust agreement / batadal.net
/// terms) are quarantined in the **untracked** `research/controlled/` tree — user-supplied, never committed or
/// shipped — so we look there first. The tracked *synthetic* demonstrators (cavitation, valve-stiction, CSTR,
/// three-tank, …) live under `data/instrumented/` and are found by the fallback. `DSFB_CONTROLLED_DIR` overrides
/// the quarantine location (e.g. when the operator keeps their licensed copy elsewhere). The `.roles.json`
/// sidecar is always read from the tracked `data/instrumented/` dir regardless — it carries no data rows.
fn resolve_instrumented_csv(crate_dir: &Path, name: &str) -> std::path::PathBuf {
    let controlled = std::env::var_os("DSFB_CONTROLLED_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            // crate_dir = <repo>/crates/dsfb-chemical-engineering-edge → <repo>/research/controlled
            crate_dir
                .parent()
                .and_then(|p| p.parent())
                .map(|root| root.join("research").join("controlled"))
                .unwrap_or_else(|| crate_dir.join("research").join("controlled"))
        });
    let quarantined = controlled.join(format!("{name}.csv"));
    if quarantined.exists() {
        quarantined
    } else {
        crate_dir
            .join("data")
            .join("instrumented")
            .join(format!("{name}.csv"))
    }
}

/// `balance-witness <name>` — run a mass/energy-balance witness on an instrumented demonstrator.
///
/// Loads the `<name>.csv` (controlled bytes from `research/controlled/` if present, else the synthetic
/// demonstrator under `data/instrumented/`) + the tracked `data/instrumented/<name>.roles.json`, recomputes the documented
/// balance-closure residual from the raw columns ([`crate::balance::balance_residual`]), admits it as
/// a `ProcessStructure` witness detector through the DSFB grammar, and reports whether the witness
/// fires at the labelled fault onset (detection delay), the baseline-vs-fault closure magnitudes, and
/// --- when the roles declare a control loop --- the control-action context (manipulated saturation).
/// Writes a per-sample witness trace next to the slice. Demonstrates mass/energy imbalance as
/// court-admissible residual evidence on data that actually encodes the balance. Exit 0 if the witness
/// fires after onset, else 2.
pub fn run_balance_witness(crate_dir: &Path, name: &str) -> i32 {
    let dir = crate_dir.join("data").join("instrumented");
    // Controlled SWaT/BATADAL bytes live in the untracked research/controlled/ quarantine; the synthetic
    // demonstrators live in data/instrumented/. The roles sidecar (no data rows) stays tracked under `dir`.
    let csv = resolve_instrumented_csv(crate_dir, name);
    let roles_path = dir.join(format!("{name}.roles.json"));
    let (m, n_base) = match datasets::load_csv_slice(&csv, 0.4) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("balance-witness: {e}");
            return 1;
        }
    };
    let roles = match crate::balance::RolesDoc::load(&roles_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("balance-witness: {e}");
            return 1;
        }
    };
    let res = match crate::balance::balance_residual(&m, &roles) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("balance-witness: {e}");
            return 1;
        }
    };
    let onset = m.fault_onset.unwrap_or(n_base);

    // Calibrate the witness threshold from the baseline window: a robust high quantile of |closure|,
    // so the stable baseline offset is admissible and only a sustained shift breaches.
    let mut base_abs: Vec<f64> = (1..n_base.min(res.len()))
        .map(|k| res[k].abs())
        .filter(|x| x.is_finite())
        .collect();
    base_abs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q = if base_abs.is_empty() {
        1.0
    } else {
        let rank = ((0.99 * base_abs.len() as f64).ceil() as usize).clamp(1, base_abs.len());
        base_abs[rank - 1].max(1e-9)
    };

    // Build the witness detector stream (normalised closure) and run it through the DSFB grammar.
    let outputs: Vec<crate::detectors::DetectorOutput> = (0..m.n_samples)
        .map(|k| {
            let norm = res[k].abs() / q;
            crate::detectors::DetectorOutput {
                detector_id: "mass_energy_balance_witness".into(),
                family: crate::detectors::DetectorFamily::ProcessStructure,
                time_index: k,
                variable_scope: "balance-closure".into(),
                raw_score: res[k].abs(),
                normalized_score: norm,
                threshold: q,
                signed_margin: norm - 1.0,
                breach: norm > 1.0,
            }
        })
        .collect();
    let timeline = crate::fusion::run_detector_dsfb(
        outputs,
        crate::detectors::DetectorFamily::ProcessStructure,
        "mass_energy_balance_witness",
        15,
    );

    // Detection: first non-nominal (non-SensorFault) grammar step at or after the fault onset.
    let detect = (onset..timeline.steps.len()).find(|&k| {
        let s = timeline.steps[k].state;
        s.is_non_nominal() && s != crate::dsfb_core::GrammarState::SensorFault
    });

    let mean = |lo: usize, hi: usize| -> f64 {
        let v: Vec<f64> = (lo..hi.min(res.len()))
            .map(|k| res[k].abs())
            .filter(|x| x.is_finite())
            .collect();
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    let (base_mean, fault_mean) = (mean(1, n_base), mean(onset, m.n_samples));

    println!("dataset: {}", roles.dataset);
    println!(
        "balance: {}",
        roles
            .balance
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
    );
    println!(
        "closure |residual|: baseline {:.3} -> fault {:.3} ({:.1}x)",
        base_mean,
        fault_mean,
        fault_mean / base_mean.max(1e-9)
    );
    // Contiguous label==1 windows. Real multi-attack datasets (e.g. BATADAL) carry several disjoint
    // ones; only some target the balanced unit.
    let windows: Vec<(usize, usize)> = m
        .labels
        .as_ref()
        .map(|labels| {
            let mut w = Vec::new();
            let mut i = 0;
            while i < labels.len() {
                if labels[i] == 1 {
                    let s = i;
                    while i < labels.len() && labels[i] == 1 {
                        i += 1;
                    }
                    w.push((s, i - 1));
                } else {
                    i += 1;
                }
            }
            w
        })
        .unwrap_or_default();

    println!("fault onset (label): {onset}");
    match detect {
        // For a single labelled window the onset-relative delay is meaningful; for several disjoint
        // windows it is not (onset is the first window, which may target a different unit), so defer
        // to the per-window response below.
        Some(k) if windows.len() <= 1 => println!(
            "witness fired: sample {k} (detection delay {} vs onset; motif {})",
            k as i64 - onset as i64,
            timeline.steps[k].state.token()
        ),
        Some(k) => println!(
            "witness fired: sample {k} (motif {}); multi-attack data -- see per-window response",
            timeline.steps[k].state.token()
        ),
        None => println!("witness did NOT fire after onset"),
    }

    // Per-window response: a tank's balance should fire on its own attacks and stay quiet on others.
    // Each window's peak normalised closure and whether the witness produced a non-nominal
    // (non-SensorFault) grammar state inside it.
    if windows.len() > 1 {
        println!(
            "labelled windows: {} (per-window witness response)",
            windows.len()
        );
        for (s, e) in &windows {
            let peak = (*s..=*e)
                .filter_map(|k| res.get(k))
                .map(|x| x.abs() / q)
                .fold(0.0f64, f64::max);
            let fired = (*s..=*e).any(|k| {
                timeline.steps.get(k).is_some_and(|st| {
                    st.state.is_non_nominal()
                        && st.state != crate::dsfb_core::GrammarState::SensorFault
                })
            });
            let nsamp = e - s + 1;
            let verdict = if fired { "FIRED" } else { "quiet" };
            println!(
                "  window [{s}..{e}] ({nsamp} samples): peak |closure|/thr {peak:.2}x -> {verdict}"
            );
        }
    }

    if let Some(ca) = &roles.control_action {
        if let Some(mv) = ca.get("manipulated").and_then(|v| v.as_str()) {
            if let Some(mc) = m.var_names.iter().position(|h| h == mv) {
                let base_mv = (0..n_base).map(|k| m.row(k)[mc]).sum::<f64>() / n_base.max(1) as f64;
                let post: Vec<f64> = (onset..m.n_samples).map(|k| m.row(k)[mc]).collect();
                let post_mv = post.iter().sum::<f64>() / post.len().max(1) as f64;
                let sat = ca
                    .get("saturation_high")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(f64::INFINITY);
                let satfrac = post.iter().filter(|&&x| x >= 0.95 * sat).count() as f64
                    / post.len().max(1) as f64;
                println!("control-action: manipulated '{mv}' baseline mean {base_mv:.1} -> post-onset mean {post_mv:.1} (saturated {:.0}% of post-onset).", satfrac * 100.0);
                if satfrac >= 0.2 {
                    println!("  -> controller pins the manipulated variable to its range extreme while the balance witness flags an inconsistency: the loop has exhausted its authority and cannot absorb the change (control-action mismatch).");
                } else {
                    println!("  -> controller responded with headroom (manipulated variable not saturated): the balance witness and the control loop disagree -- the witness flags an inconsistency the loop did not need to saturate to hold. That disagreement is ambiguous between a sensor artifact the loop is chasing and a real disturbance the loop is absorbing; the disagreement is the admitted evidence, the disposition is the operator's.");
                }
            }
        }
    }
    // Write the witness trace next to the resolved CSV, so a controlled-data run stays inside research/controlled/.
    let trace = csv.with_extension("witness.csv");
    let mut lines = vec!["time_index,balance_residual,normalized,grammar_state,label".to_string()];
    for k in 0..m.n_samples {
        let lbl = m.labels.as_ref().map(|l| l[k]).unwrap_or(0);
        lines.push(format!(
            "{},{:.6},{:.4},{},{}",
            k,
            res[k],
            res[k].abs() / q,
            timeline.steps[k].state.token(),
            lbl
        ));
    }
    let _ = std::fs::write(&trace, lines.join("\n") + "\n");
    println!("witness trace: {}", trace.display());
    if detect.is_some() {
        0
    } else {
        2
    }
}

/// `control-action <name>` — control-action context fingerprint on a dataset that carries manipulated
/// variables. For the Tennessee Eastman benchmark the manipulated variables are the documented
/// `xmv1..xmv11` (feed / purge / recycle / coolant valves), detected by the `xmv` name prefix; other
/// datasets may declare manipulated roles in `data/instrumented/<name>.roles.json`. The command ranks
/// the manipulated variables by how far their post-onset mean deviates from baseline (z-score) --- i.e.
/// which valves the loop drove in response to the fault --- and flags any that pin to an extreme of
/// their observed range (lost actuation authority). It surfaces the control response as inspectable
/// evidence (context) on real public-benchmark data; it asserts no diagnosis.
pub fn run_control_action(crate_dir: &Path, name: &str) -> i32 {
    let mut csv = crate_dir.join("data/slices").join(format!("{name}.csv"));
    if !csv.exists() {
        csv = resolve_instrumented_csv(crate_dir, name);
    }
    let (m, n_base) = match datasets::load_csv_slice(&csv, 0.4) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("control-action: {e}");
            return 1;
        }
    };
    // Manipulated columns: a roles sidecar if present, else the TEP `xmv*` naming convention.
    let roles_path = crate_dir
        .join("data/instrumented")
        .join(format!("{name}.roles.json"));
    let manip: Vec<usize> = if roles_path.exists() {
        match crate::balance::RolesDoc::load(&roles_path) {
            Ok(r) => {
                let names: Vec<String> = r.manipulated().iter().map(|s| s.to_string()).collect();
                m.var_names
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| names.iter().any(|n| n == *h))
                    .map(|(i, _)| i)
                    .collect()
            }
            Err(e) => {
                eprintln!("control-action: {e}");
                return 1;
            }
        }
    } else {
        m.var_names
            .iter()
            .enumerate()
            .filter(|(_, h)| h.to_ascii_lowercase().starts_with("xmv"))
            .map(|(i, _)| i)
            .collect()
    };
    if manip.is_empty() {
        eprintln!("control-action: no manipulated variables (xmv*/roles) found in '{name}'");
        return 1;
    }
    let onset = m.fault_onset.unwrap_or(n_base);
    let mut rows: Vec<(String, f64, f64, f64, f64)> = Vec::new(); // (name, z, base_mean, post_mean, sat_frac)
    for &c in &manip {
        let base: Vec<f64> = (0..n_base)
            .map(|k| m.row(k)[c])
            .filter(|x| x.is_finite())
            .collect();
        let post: Vec<f64> = (onset..m.n_samples)
            .map(|k| m.row(k)[c])
            .filter(|x| x.is_finite())
            .collect();
        if base.is_empty() || post.is_empty() {
            continue;
        }
        let bm = base.iter().sum::<f64>() / base.len() as f64;
        let bsd = (base.iter().map(|x| (x - bm) * (x - bm)).sum::<f64>() / base.len() as f64)
            .sqrt()
            .max(1e-9);
        let pm = post.iter().sum::<f64>() / post.len() as f64;
        // Saturation: fraction of post-onset samples within 1% of the observed full range extremes.
        let allv: Vec<f64> = (0..m.n_samples)
            .map(|k| m.row(k)[c])
            .filter(|x| x.is_finite())
            .collect();
        let lo = allv.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = allv.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let span = (hi - lo).max(1e-9);
        let sat = post
            .iter()
            .filter(|&&x| (x - lo) < 0.01 * span || (hi - x) < 0.01 * span)
            .count() as f64
            / post.len() as f64;
        rows.push((m.var_names[c].clone(), (pm - bm) / bsd, bm, pm, sat));
    }
    rows.sort_by(|a, b| {
        b.1.abs()
            .partial_cmp(&a.1.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    println!(
        "control-action context: {name} (fault onset {onset}; {} manipulated variables)",
        manip.len()
    );
    println!("  rank  variable      z-dev   baseline -> post-onset   pinned");
    for (i, (nm, z, bm, pm, sat)) in rows.iter().take(8).enumerate() {
        println!(
            "  {:>2}    {:<11}  {:>+6.2}  {:>9.3} -> {:<9.3}  {:>3.0}%{}",
            i + 1,
            nm,
            z,
            bm,
            pm,
            sat * 100.0,
            if *sat >= 0.2 {
                "  <- pinned to range extreme (lost authority)"
            } else {
                ""
            }
        );
    }
    println!("(control response surfaced as evidence; no diagnosis asserted)");
    0
}
