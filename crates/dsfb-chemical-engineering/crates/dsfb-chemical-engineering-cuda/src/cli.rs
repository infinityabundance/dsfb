//! Command-line driver for the CUDA crate.
//!
//! Each `run_*` function implements one CLI command and returns an exit code (0 = success, non-zero
//! = failure). The caller (`main.rs`) converts the code to `std::process::exit`.
//!
//! Commands:
//!   demo            run the forensic court over all datasets -> hash-linked case files + replay + zip
//!   bench           run GB/s throughput benchmarks (multiple runs, multiple sizes) -> reports/
//!   verify-replay   assemble each case file and confirm byte-exact evidence-root replay
//!   device          print the active backend + device name
//!   profile         single kernel launch at configurable size for clean Nsight capture
//!   atlas           print the atlas authority summary + frozen atlas_hash_v1 (shared with edge)
//!   corpus          print the soft-sensor data-corpus authority (feature `soft-sensor-corpus`)
//!
//! # Output layout (demo command)
//!
//! ```text
//! output-dsfb-chemical-engineering-cuda/<stamp>/
//!   case_files/<dataset>.casefile.json   — one per dataset, JSON-serialised CaseFile
//!   manifest.json                        — run-level summary (all datasets, roots, backends)
//!   artifact_bundle.zip                  — zip of all of the above (store, no compression)
//! ```

use crate::bench;
use crate::court::CaseFile;
use crate::dispatch::{self, evidence_cpu};
use dsfb_chemical_engineering_edge::cli as edge_cli;
use dsfb_chemical_engineering_edge::report;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Compute the current UTC timestamp without any external crate dependency.
///
/// Returns two strings: a compact filesystem-safe stamp (`YYYYMMDD-HHMMSS`) and an ISO 8601
/// string (`YYYY-MM-DDTHH:MM:SSZ`). Uses a pure-integer civil calendar algorithm
/// (Euclidean affine functions; see Howard Hinnant's date algorithms).
///
/// NOTE: this is a best-effort UTC timestamp from `SystemTime::now()`. Accuracy depends on the
/// system clock; no leap-second correction is applied.
fn timestamp_utc() -> (String, String) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    // Split seconds since epoch into whole days and the remaining time-of-day.
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Civil calendar conversion (Hinnant's algorithm):
    // shift epoch from 1970-01-01 to 0000-03-01 for simpler leap-year arithmetic.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097; // 400-year era
    let doe = z - era * 146_097; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year (March-based)
    let mp = (5 * doy + 2) / 153; // month index (March = 0)
    let d = doy - (153 * mp + 2) / 5 + 1; // day of month [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // calendar month [1, 12]
    let year = if m <= 2 { y + 1 } else { y }; // adjust year for Jan/Feb
    (
        format!("{:04}{:02}{:02}-{:02}{:02}{:02}", year, m, d, h, mi, s),
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, m, d, h, mi, s),
    )
}

/// Resolve the edge crate directory (for dataset slices) relative to this crate's directory.
///
/// Datasets are stored under `dsfb-chemical-engineering-edge/` which sits beside this crate in
/// the workspace. If `parent()` returns `None` (the crate dir has no parent), fall back to the
/// crate dir itself to avoid a panic.
fn edge_dir(crate_dir: &Path) -> PathBuf {
    crate_dir
        .parent()
        .map(|p| p.join("dsfb-chemical-engineering-edge"))
        .unwrap_or_else(|| crate_dir.to_path_buf())
}

/// Print the active backend and CUDA device name (if available) to stdout. Always returns 0.
pub fn run_device() -> i32 {
    println!("cuda backend compiled: {}", crate::CUDA_ENABLED);
    #[cfg(feature = "cuda")]
    {
        println!("device: {}", crate::ffi::device_name());
    }
    #[cfg(not(feature = "cuda"))]
    {
        println!("device: cpu-reference (build with --features cuda for the GPU path)");
    }
    0
}

/// Print the atlas authority-crate summary and the frozen `atlas_hash_v1` this binary is bound to.
///
/// The GPU evidence court and the CPU edge pipeline are bound to *one* authority — the same curated
/// detector, H1–H6 heuristic, and F1–F12 fault-signature records. Printing the identical
/// `atlas_hash_v1` from both binaries is a cheap, scriptable proof that "what evidence is allowed to
/// mean" is identical across execution backends, independent of whether the GPU path is compiled in.
pub fn run_atlas() -> i32 {
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

/// Print the soft-sensor data-corpus authority summary (counts + frozen `corpus_hash_v1`).
///
/// Optional, mirroring the edge crate: the corpus authority is opt-in via the `soft-sensor-corpus`
/// Cargo feature (off by default). Built WITH it, this validates the catalogue and prints the frozen
/// hash, so the GPU court binary is demonstrably bound to the same dataset authority as edge.
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
/// twin above). Prints the exact command that enables the corpus authority and exits success.
#[cfg(not(feature = "soft-sensor-corpus"))]
pub fn run_corpus() -> i32 {
    println!("soft-sensor data-corpus authority is an optional feature (off by default).");
    println!("enable it with:");
    println!(
        "  cargo run -p dsfb-chemical-engineering-cuda --features soft-sensor-corpus -- corpus"
    );
    0
}

/// Run the forensic court over all discovered datasets.
///
/// For each dataset:
/// 1. Build residual lanes and compute the input SHA-256.
/// 2. Run the evidence factory (GPU if available, else CPU reference).
/// 3. Assemble a `CaseFile` with a hash-linked precedent chain (the previous dataset's root).
/// 4. Attach an execution attestation (backend, device, timing, cross-verification result).
/// 5. Perform a byte-exact replay via the CPU reference and record whether it matches.
/// 6. Write the case file as JSON and collect all files into a zip bundle.
///
/// Returns 0 if every dataset's replay matched and cross-backend verification passed; 2 otherwise.
pub fn run_demo(crate_dir: &Path) -> i32 {
    let datasets = edge_cli::discover_datasets(&edge_dir(crate_dir));
    if datasets.is_empty() {
        eprintln!("no datasets found");
        return 1;
    }
    let (stamp, iso) = timestamp_utc();
    // Output directory: two levels up from the crate dir places it at the workspace root.
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let out_root = workspace_root
        .join("output-dsfb-chemical-engineering-cuda")
        .join(&stamp);
    std::fs::create_dir_all(&out_root).ok();

    println!("== DSFB-Chemical-Engineering CUDA forensic court ==");
    println!(
        "cuda backend compiled: {} ; datasets: {}",
        crate::CUDA_ENABLED,
        datasets.len()
    );

    /// Per-dataset summary entry written into `manifest.json`.
    #[derive(Serialize)]
    struct Entry {
        dataset: String,
        backend: String,
        evidence_root: String,
        replay_matched: bool,
        /// Self-consistent: CPU reference ran, or a GPU run matched it. True on the CPU-only path.
        cross_backend_verified: bool,
        /// Genuine two-backend agreement: true only when a GPU actually ran and matched the CPU
        /// reference. False on the CPU-only path. (Schema note: added P38 — distinguishes a
        /// CPU-reference manifest entry from real GPU cross-verification.)
        gpu_cross_verified: bool,
        throughput_gbps: f64,
        n_lanes: u32,
        n_samples: u32,
    }
    let mut entries: Vec<Entry> = Vec::new();
    // `precedent` carries the evidence root of the previous dataset to form a hash chain.
    let mut precedent: Option<String> = None;
    let mut all_ok = true;

    for d in &datasets {
        let lanes = dispatch::build_lanes(&d.matrix, d.n_base);
        let run = dispatch::run(&lanes);
        let total = (lanes.n_lanes as u64) * (lanes.n_samples as u64);
        // Assemble the case file, link it to the previous dataset's root, and attach attestation.
        let mut cf = CaseFile::assemble(
            &d.name,
            lanes.input_sha256.clone(),
            total,
            run.lanes,
            precedent.clone(),
        );
        cf.attest(
            run.backend.label(),
            &run.device,
            &run.kernel,
            run.elapsed_ns,
            run.bytes_processed,
            run.cross_backend_verified,
            run.gpu_cross_verified,
            &iso,
        );
        // Byte-exact replay: re-derive the evidence root via the CPU reference and compare.
        let recomputed = evidence_cpu(&lanes);
        let rr = cf.verify_replay(&recomputed);
        all_ok &= rr.matched && run.cross_backend_verified;

        let ds_dir = out_root.join("case_files");
        std::fs::create_dir_all(&ds_dir).ok();
        std::fs::write(
            ds_dir.join(format!("{}.casefile.json", d.name)),
            serde_json::to_string_pretty(&cf).unwrap(),
        )
        .ok();

        println!(
            "  {:<28} backend={:<13} lanes={:<4} samples={:<5} root={}… replay={} xverify={}",
            d.name,
            run.backend.label(),
            lanes.n_lanes,
            lanes.n_samples,
            &cf.evidence_root[..12], // show first 12 hex chars for a quick visual check
            if rr.matched { "OK" } else { "FAIL" },
            if run.cross_backend_verified {
                "OK"
            } else {
                "MISMATCH"
            },
        );
        entries.push(Entry {
            dataset: d.name.clone(),
            backend: run.backend.label().into(),
            evidence_root: cf.evidence_root.clone(),
            replay_matched: rr.matched,
            cross_backend_verified: run.cross_backend_verified,
            gpu_cross_verified: run.gpu_cross_verified,
            throughput_gbps: cf
                .attestation
                .as_ref()
                .map(|a| a.throughput_gbps)
                .unwrap_or(0.0),
            n_lanes: lanes.n_lanes,
            n_samples: lanes.n_samples,
        });
        // Pass the evidence root forward to the next dataset as the precedent hash.
        precedent = Some(cf.evidence_root);
    }

    /// Run-level manifest written to `manifest.json`.
    #[derive(Serialize)]
    struct Manifest {
        tool: String,
        crate_version: String,
        timestamp_utc: String,
        cuda_enabled: bool,
        contract_id: String,
        datasets: Vec<Entry>,
    }
    let manifest = Manifest {
        tool: "dsfb-chemical-engineering-cuda".into(),
        crate_version: crate::VERSION.into(),
        timestamp_utc: iso,
        cuda_enabled: crate::CUDA_ENABLED,
        contract_id: crate::evidence::CONTRACT_ID.into(),
        datasets: entries,
    };
    std::fs::write(
        out_root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .ok();

    // Bundle all output files into a zip archive (store mode, no compression).
    let files = report::collect_files(&out_root, &out_root);
    let zip = out_root.join("artifact_bundle.zip");
    report::zip_store(&zip, &files).ok();

    println!("\nartifacts: {}", out_root.display());
    println!("bundle   : {}", zip.display());
    if all_ok {
        0
    } else {
        // Exit code 2: evidence produced but at least one replay or cross-verification failed.
        2
    }
}

/// Assemble case files for all datasets and verify that every evidence root replays byte-for-byte.
///
/// Lighter than `run_demo`: does not write output files or produce a zip. The exit code is 0 only
/// if every dataset's replay matched and (if CUDA is enabled) cross-backend verification passed.
/// Useful as a quick smoke test after modifying the evidence contract or build flags.
pub fn run_verify_replay(crate_dir: &Path) -> i32 {
    let datasets = edge_cli::discover_datasets(&edge_dir(crate_dir));
    let mut ok = true;
    for d in &datasets {
        let lanes = dispatch::build_lanes(&d.matrix, d.n_base);
        let run = dispatch::run(&lanes);
        let total = (lanes.n_lanes as u64) * (lanes.n_samples as u64);
        // No precedent chain needed for a standalone replay check.
        let cf = CaseFile::assemble(&d.name, lanes.input_sha256.clone(), total, run.lanes, None);
        // Re-derive the evidence using the CPU reference and compare roots.
        let rr = cf.verify_replay(&evidence_cpu(&lanes));
        ok &= rr.matched && run.cross_backend_verified;
        println!(
            "  {:<28} replay {}  xverify {}  root={}…",
            d.name,
            if rr.matched { "OK" } else { "FAIL" },
            if run.cross_backend_verified {
                "OK"
            } else {
                "MISMATCH"
            },
            &cf.evidence_root[..12]
        );
    }
    if ok {
        0
    } else {
        2
    }
}

/// Single, isolated evidence-kernel launch at a fixed size — for clean Nsight (`ncu`) capture.
///
/// Issues exactly one kernel call so that Nsight Systems/Compute can capture the kernel in
/// isolation without warmup or multi-run noise. Size defaults to 4096 lanes x 16384 samples
/// (~512 MB) but can be overridden via environment variables `DSFB_BENCH_LANES` and
/// `DSFB_BENCH_SAMPLES`. Requires `--features cuda`; exits with code 1 in CPU-only builds.
pub fn run_profile() -> i32 {
    let nl: usize = std::env::var("DSFB_BENCH_LANES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4096);
    let ns: usize = std::env::var("DSFB_BENCH_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(16384);
    #[cfg(feature = "cuda")]
    {
        let x = bench::synthetic(nl, ns);
        match crate::ffi::run_evidence(&x, nl as u32, ns as u32) {
            Ok(ev) => {
                let bytes = (x.len() * 8) as f64; // input bytes (f64 = 8 bytes each)
                let gbps = if ev.elapsed_ms > 0.0 {
                    bytes / (ev.elapsed_ms as f64 * 1.0e6)
                } else {
                    0.0
                };
                println!(
                    "profile: evidence_kernel {nl}x{ns} ({} MB) {:.3} ms {:.1} GB/s",
                    (x.len() * 8) / (1 << 20),
                    ev.elapsed_ms,
                    gbps
                );
                0
            }
            Err(rc) => {
                eprintln!("cuda evidence launch failed rc={rc}");
                1
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        let _ = (nl, ns); // suppress unused-variable warnings in CPU-only builds
        eprintln!("profile requires --features cuda");
        1
    }
}

/// Single, isolated **V2-A batched** evidence-kernel launch — for clean Nsight capture of
/// `evidence_kernel_batched`.
///
/// Constructs a jagged layout of `DSFB_BENCH_LANES` lanes, each of `DSFB_BENCH_SAMPLES` samples,
/// stored lane-contiguous, and runs them all in ONE launch. This is how V2-A clears the occupancy
/// wall: where V1's grid is `lanes/128` for a *single* dataset, here the grid is `total_lanes/128`
/// across the whole batch — so to watch occupancy climb, raise `DSFB_BENCH_LANES` (e.g. 16384 lanes
/// x 512 samples = 128 blocks > 80 SMs). Defaults: 16384 lanes x 512 samples (~64 MB, the same total
/// work as the V1 `profile` variants, but spread across many more lanes). Requires `--features cuda`.
pub fn run_profile_batched() -> i32 {
    let nl: usize = std::env::var("DSFB_BENCH_LANES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(16384);
    let ns: usize = std::env::var("DSFB_BENCH_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(512);
    #[cfg(feature = "cuda")]
    {
        // Flat lane-contiguous buffer; per-lane (offset, nsamples) descriptors. Values are arbitrary
        // (profiling measures the compute shape, not the data) but deterministic for reproducibility.
        let total = nl * ns;
        let mut xcat = vec![0.0f64; total];
        for (i, v) in xcat.iter_mut().enumerate() {
            *v = ((i % 1000) as f64) / 100.0 - 5.0;
        }
        let offsets: Vec<u64> = (0..nl).map(|l| (l * ns) as u64).collect();
        let nsamples: Vec<u32> = vec![ns as u32; nl];
        match crate::ffi::run_evidence_batched(&xcat, &offsets, &nsamples) {
            Ok(ev) => {
                let bytes = (total * 8) as f64;
                let gbps = if ev.elapsed_ms > 0.0 {
                    bytes / (ev.elapsed_ms as f64 * 1.0e6)
                } else {
                    0.0
                };
                let blocks = nl.div_ceil(128);
                println!(
                    "profile-batched: evidence_kernel_batched {nl} lanes x {ns} samples ({} MB, grid {blocks} blocks) {:.3} ms {:.1} GB/s",
                    (total * 8) / (1 << 20),
                    ev.elapsed_ms,
                    gbps
                );
                0
            }
            Err(rc) => {
                eprintln!("cuda batched evidence launch failed rc={rc}");
                1
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        let _ = (nl, ns);
        eprintln!("profile-batched requires --features cuda");
        1
    }
}

/// Single isolated **V2-B** segment-parallel launch — for clean Nsight capture of
/// `evidence_kernel_v2_segmented`.
///
/// Builds `DSFB_BENCH_LANES` lanes x `DSFB_BENCH_SAMPLES` samples and splits each lane into
/// `DSFB_BENCH_SEG`-sample segments (default `evidence::SEGMENT_SIZE`), launching one thread per
/// segment. This is the direct comparison to V1 on the *deep* shape: at 1024 lanes x 8192 samples,
/// seg 512 -> 16 segments/lane x 1024 = 16384 threads = 128 blocks, vs V1's 8 blocks — and each
/// thread's SHA chain is `seg` long, not 8192. Requires `--features cuda`.
pub fn run_profile_v2_segmented() -> i32 {
    let nl: usize = std::env::var("DSFB_BENCH_LANES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024);
    let ns: usize = std::env::var("DSFB_BENCH_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8192);
    let seg: usize = std::env::var("DSFB_BENCH_SEG")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(crate::evidence::SEGMENT_SIZE);
    #[cfg(feature = "cuda")]
    {
        if seg == 0 || nl == 0 || ns == 0 {
            eprintln!("profile-v2-segmented: lanes, samples, and seg must all be > 0");
            return 1;
        }
        let w = crate::evidence::DRIFT_WINDOW;
        let total = nl * ns;
        let mut xcat = vec![0.0f64; total];
        for (i, v) in xcat.iter_mut().enumerate() {
            *v = ((i % 1000) as f64) / 100.0 - 5.0;
        }
        let (mut seg_base, mut seg_start, mut seg_end, mut seg_warmup) = (
            Vec::<u64>::new(),
            Vec::<u32>::new(),
            Vec::<u32>::new(),
            Vec::<u32>::new(),
        );
        for l in 0..nl {
            let base = (l * ns) as u64;
            let nseg = ns.div_ceil(seg);
            for k in 0..nseg {
                let s = k * seg;
                seg_base.push(base);
                seg_start.push(s as u32);
                seg_end.push(((k + 1) * seg).min(ns) as u32);
                seg_warmup.push(s.min(w) as u32);
            }
        }
        let n_seg = seg_base.len();
        match crate::ffi::run_evidence_v2_segmented(
            &xcat,
            &seg_base,
            &seg_start,
            &seg_end,
            &seg_warmup,
        ) {
            Ok(ev) => {
                let bytes = (total * 8) as f64;
                let gbps = if ev.elapsed_ms > 0.0 {
                    bytes / (ev.elapsed_ms as f64 * 1.0e6)
                } else {
                    0.0
                };
                let blocks = n_seg.div_ceil(128);
                println!(
                    "profile-v2-segmented: evidence_kernel_v2_segmented {nl}x{ns} seg={seg} ({} MB, {n_seg} segments, grid {blocks} blocks) {:.3} ms {:.1} GB/s",
                    (total * 8) / (1 << 20),
                    ev.elapsed_ms,
                    gbps
                );
                0
            }
            Err(rc) => {
                eprintln!("cuda v2-segmented launch failed rc={rc}");
                1
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        let _ = (nl, ns, seg);
        eprintln!("profile-v2-segmented requires --features cuda");
        1
    }
}

/// Run GB/s throughput benchmarks over a sweep of sizes and write a JSON report to `reports/`.
///
/// Sweeps three `(n_lanes, n_samples)` sizes for the evidence factory: 1024x4096, 2048x8192,
/// 4096x16384. Each size runs 5 timed repetitions (plus a warm-up). Prints median/min/max/stddev
/// to stdout and writes a timestamped `bench_<iso>.json` file to `<crate_dir>/reports/`.
///
/// The memory roofline (CUDA only) is also run at 64 Mi elements x 20 streaming passes to give
/// an upper-bound DRAM bandwidth reference alongside the evidence-kernel numbers.
pub fn run_bench(crate_dir: &Path) -> i32 {
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let reports = crate_dir.join("reports");
    std::fs::create_dir_all(&reports).ok();
    let (_, iso) = timestamp_utc();

    // Three sizes; five timed runs each (one warm-up is discarded inside bench_evidence).
    let sizes = [(1024usize, 4096usize), (2048, 8192), (4096, 16384)];
    let runs = 5;
    let mut summaries = Vec::new();
    println!(
        "== DSFB-Chemical-Engineering CUDA throughput bench (cuda={}) ==",
        crate::CUDA_ENABLED
    );
    for (nl, ns) in sizes {
        let s = bench::bench_evidence(nl, ns, runs);
        println!(
            "  evidence  {:>5}x{:<6} {:>5} MB  {:>8.1} GB/s (median; min {:.1}, max {:.1}, sd {:.2}) [{}]",
            nl, ns, s.bytes / (1 << 20), s.gbps_median, s.gbps_min, s.gbps_max, s.gbps_stddev, s.backend
        );
        summaries.push(s);
    }
    // Memory roofline: 64 Mi doubles (512 MB), 20 streaming passes per timed call, 5 runs.
    if let Some(r) = bench::bench_roofline(64 * 1024 * 1024, 20, runs) {
        println!(
            "  roofline  {} Melem x20  {:>8.1} GB/s (median; min {:.1}, max {:.1}) [{}]",
            r.n_samples / (1 << 20),
            r.gbps_median,
            r.gbps_min,
            r.gbps_max,
            r.backend
        );
        summaries.push(r);
    }

    /// JSON report written to `reports/bench_<timestamp>.json`.
    #[derive(Serialize)]
    struct Report {
        timestamp_utc: String,
        cuda_enabled: bool,
        contract_id: String,
        summaries: Vec<bench::BenchSummary>,
    }
    let rep = Report {
        timestamp_utc: iso.clone(),
        cuda_enabled: crate::CUDA_ENABLED,
        contract_id: crate::evidence::CONTRACT_ID.into(),
        summaries,
    };
    // Replace colons and hyphens in the ISO timestamp so the filename is filesystem-safe.
    let out = reports.join(format!("bench_{}.json", iso.replace([':', '-'], "")));
    std::fs::write(&out, serde_json::to_string_pretty(&rep).unwrap()).ok();
    let _ = workspace_root; // used only for context; no files written there from bench
    println!("report: {}", out.display());
    0
}
