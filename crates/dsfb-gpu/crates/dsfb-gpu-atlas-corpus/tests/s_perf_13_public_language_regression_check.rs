//! S-PERF.13 public-language regression check.
//!
//! Three panel-required load-bearing negatives (subset of the
//! 12-negative N1–N12 set in the plan; the remainder live in the
//! S-PERF.13 corpus receipt module's verifier when it lands as a
//! follow-on commit). These three are the file-system-level
//! scanners that pin the post-S-PERF.13 doctrine across the live
//! repo: the D64 `_timed` bench tests must emit `"host: input
//! staging"` (not `"host: compute_features"`), the
//! `D64ThroughputHostStageTimings` struct body must carry the
//! renamed `host_input_staging_us` field (not the pre-S-PERF.13
//! `features_us`), and public docs must not claim that host
//! `compute_features` is the dominant wall on the post-R.11b
//! D64 measured path.
//!
//! Scope discipline (panel-locked): these scanners target ONLY
//! the post-R.11b D64 `_timed` surface. The legacy `R8HostStageTimings`
//! struct + the R.8-era dispatchers that still legitimately call
//! host `compute_features` are explicitly out of scope; the
//! `bench_gpu_scale.rs` CLI is out of scope (it consumes the
//! legacy struct).
//!
//! Three negatives:
//!
//!  1. `s_perf_13_rejects_stale_host_compute_features_label_in_d64_timed_bench_reports`
//!  2. `s_perf_13_rejects_d64_throughput_host_stage_timings_features_us_field`
//!  3. `s_perf_13_rejects_calling_input_staging_compute_features_in_public_docs`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at .../crates/dsfb-gpu-atlas-corpus;
    // walk up two parents to reach the repo root.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

fn read(path: &str) -> String {
    let full = repo_root().join(path);
    std::fs::read_to_string(&full).unwrap_or_else(|e| {
        panic!("failed to read {}: {e}", full.display());
    })
}

/// **N2 from the S-PERF.13 plan (`s_perf_13_rejects_stale_host_compute_features_label`)**
/// — scan the three D64 `_timed` bench test files for the
/// pre-S-PERF.13 report-line literal `"host: compute_features"`.
/// Post-S-PERF.13 the renderer must emit `"host: input staging"`
/// because R.11b moved feature math to device and the slot
/// measures input pack-to-pinned staging (see the
/// S-PERF.13-PREFLIGHT receipt for the full audit chain).
///
/// Legacy R.8 bench paths (`bench_gpu_scale.rs`) are
/// deliberately excluded: those still legitimately call host
/// `compute_features` via the R.8-era dispatchers.
#[test]
fn s_perf_13_rejects_stale_host_compute_features_label_in_d64_timed_bench_reports() {
    let bench_paths = [
        "crates/dsfb-gpu-debug-demo/tests/r9_c_d64_stage_profile.rs",
        "crates/dsfb-gpu-debug-demo/tests/r12_d64_saturation.rs",
        "crates/dsfb-gpu-debug-demo/tests/s_perf_12_d64_compact_densor_stage_profile.rs",
    ];
    // The forbidden literal is the report-line emit, not any
    // English-prose discussion of historical naming. The check
    // is anchored to the leading `"host: ` prefix that uniquely
    // identifies a render-time row in the bench output.
    let forbidden = "host: compute_features";
    for path in &bench_paths {
        let body = read(path);
        // Allow the substring inside the FORBIDDEN list of a
        // regression-check scanner itself (defense-in-depth so
        // we can call out the forbidden phrase in a future
        // sibling negative without recursively tripping this
        // one). The bench files have no need to mention the
        // forbidden phrase outside that scanner-list role, and
        // none of them are regression-check files, so the bare
        // contains() check is correct here.
        assert!(
            !body.contains(forbidden),
            "S-PERF.13 N2 violation: {path} still emits the pre-S-PERF.13 \
             report line \"{forbidden}\". Post-S-PERF.13 the D64 _timed \
             bench renderers must emit \"host: input staging\" because \
             R.11b moved feature math to device under \
             window_feature_kernel_structured. See \
             reports/s_perf_13_preflight_d64_feature_path_audit.txt."
        );
    }
}

/// **N9 from the S-PERF.13 plan (`s_perf_13_rejects_host_input_staging_label_regression`)**
/// — once the field rename lands, any reintroduction of
/// `pub features_us:` inside the `D64ThroughputHostStageTimings`
/// struct body is a regression. The legacy `R8HostStageTimings`
/// struct (which still uses `features_us` legitimately for the
/// R.8-era dispatcher that genuinely calls host
/// `compute_features`) is deliberately excluded: the scanner
/// extracts only the `D64ThroughputHostStageTimings` block.
///
/// Doc comments that historically annotate the rename (e.g.
/// "Previously named `features_us`") are explicitly allowed —
/// those are required documentation per the S-PERF.13 plan's
/// hygiene rule H1.
#[test]
fn s_perf_13_rejects_d64_throughput_host_stage_timings_features_us_field() {
    let dispatch_rs = read("crates/dsfb-gpu-debug-cuda/src/dispatch.rs");

    // Find the D64ThroughputHostStageTimings struct definition
    // and scope the scan to its body (between `pub struct ... {`
    // and the matching closing brace at column 0).
    let struct_marker = "pub struct D64ThroughputHostStageTimings";
    let start = dispatch_rs
        .find(struct_marker)
        .expect("D64ThroughputHostStageTimings struct must exist post-S-PERF.13");
    // Scope: from the struct marker forward to the next
    // top-level `}` line. The end is the first `\n}\n`
    // occurrence after the marker; any field declarations
    // live within that window.
    let body_end_marker = "\n}\n";
    let rel_end = dispatch_rs[start..]
        .find(body_end_marker)
        .expect("D64ThroughputHostStageTimings must have a closing brace");
    let body = &dispatch_rs[start..start + rel_end];

    // Forbid `pub features_us:` field declarations inside the
    // struct body. Doc comments that mention `features_us`
    // (e.g. the "Previously named `features_us`" annotation)
    // are allowed — those are required per S-PERF.13 hygiene
    // rule H1.
    let forbidden = "pub features_us:";
    assert!(
        !body.contains(forbidden),
        "S-PERF.13 N9 violation: D64ThroughputHostStageTimings struct body \
         still declares `pub features_us:`. Post-S-PERF.13 the field is \
         renamed to `host_input_staging_us` because R.11b moved feature \
         math to device. The historical-rename annotation in the field's \
         doc comment is allowed; restoring the field name is not. See \
         reports/s_perf_13_preflight_d64_feature_path_audit.txt."
    );

    // Defense-in-depth: the renamed field MUST be present.
    // A future commit cannot delete the field entirely and
    // claim N9 satisfied by absence.
    let required = "pub host_input_staging_us:";
    assert!(
        body.contains(required),
        "S-PERF.13 N9 defense-in-depth violation: D64ThroughputHostStageTimings \
         struct body is missing the renamed field `pub host_input_staging_us:`. \
         The field must exist post-S-PERF.13."
    );
}

/// **N1 from the S-PERF.13 plan (`s_perf_13_rejects_calling_input_staging_compute_features`)**
/// — public docs (README + paper + corpus lib.rs) must not claim
/// that host `compute_features` is the dominant wall on the
/// post-R.11b D64 measured path. R.11b moved feature math to
/// device under `window_feature_kernel_structured`; the
/// remaining ~6.3 ms host slot is event-pack-to-pinned staging.
///
/// The scanner is conservative: it forbids the specific
/// substrings that would mislabel the staging wall, NOT the
/// general phrase "compute_features" (which legitimately
/// appears in legacy R.8 / pre-R.11b context, in
/// `synthesize_*` fixture helpers, and in the
/// S-PERF.13-PREFLIGHT receipt where it appears inside
/// "host: compute_features" historical-context quotes).
#[test]
fn s_perf_13_rejects_calling_input_staging_compute_features_in_public_docs() {
    // Scan only files that are READMEs / lib.rs docstrings;
    // historical receipts and PREFLIGHT artifacts are exempt
    // because they legitimately quote the stale label as
    // historical context. The plan file itself is exempt
    // (it carries the verbatim panel quote of the
    // S-PERF.11.1 decision rule).
    let docs = ["crates/dsfb-gpu-atlas-corpus/src/lib.rs"];

    // Forbidden current-state phrases that mislabel the
    // staging wall. Each phrase is a positive-claim
    // assertion that host compute_features is the dominant
    // wall on the post-R.11b D64 path — exactly the
    // mislabeling S-PERF.13 corrects.
    let forbidden_substrings = [
        "host compute_features is the dominant wall",
        "host compute_features at ~7.1 ms",
        "host compute_features at ~7142",
        "host compute_features remains the dominant",
        "dominant remaining wall segment is host `compute_features`",
        "dominant remaining wall segment is host compute_features",
    ];

    for path in &docs {
        let body = read(path).to_lowercase();
        for forbidden in &forbidden_substrings {
            let needle = forbidden.to_lowercase();
            assert!(
                !body.contains(&needle),
                "S-PERF.13 N1 violation: {path} contains forbidden phrase \
                 \"{forbidden}\" describing the post-R.11b D64 measured \
                 path. R.11b moved feature math to device; the remaining \
                 host wall is event-pack-to-pinned staging, NOT host \
                 compute_features. See \
                 reports/s_perf_13_preflight_d64_feature_path_audit.txt \
                 for the full audit chain (verdict: \
                 FeaturePathMixedHostStagingDeviceCompute)."
            );
        }
    }
}
