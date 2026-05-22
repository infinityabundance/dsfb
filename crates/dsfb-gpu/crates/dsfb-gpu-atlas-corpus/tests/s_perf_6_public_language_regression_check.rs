//! S-PERF.6 public-language regression check.
//!
//! Eight panel-required negatives that walk the live README,
//! the checked-in S-PERF.6 summary receipt, and the live
//! `lib.rs` module docstring, then reject any drift away from
//! the unified S-PERF.6 measured CUDA pipeline baseline framing.
//!
//! The collapse from the prior dual-section (scaffold +
//! "repair") surface into a single S-PERF.6 measured baseline
//! is panel-locked. These tests fail the build if a future
//! commit re-introduces the apology branding, restores the
//! all-zero `NoClaim` posture as the current S-PERF.6
//! result, claims 13.33 GB/s is saturation, or removes the
//! host-segment disclosure that anchors the honest "not pure
//! Layer-A" caveat.
//!
//! Eight negatives (verbatim from the panel directive):
//!
//!  1. `s_perf_6_rejects_public_repair_language`
//!  2. `s_perf_6_rejects_public_s_perf_6r_current_state`
//!  3. `s_perf_6_rejects_current_no_claim_performance_result`
//!  4. `s_perf_6_rejects_missing_13_33_gbps_result`
//!  5. `s_perf_6_rejects_missing_186_bp_result`
//!  6. `s_perf_6_rejects_missing_saturation_false`
//!  7. `s_perf_6_rejects_claim_that_13_33_gbps_is_saturation`
//!  8. `s_perf_6_rejects_removal_of_host_segment_disclosure`

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

fn readme() -> String {
    read("README.md")
}

fn s_perf_6_summary() -> String {
    read("reports/s_perf_6_measured_cuda_pipeline_summary.txt")
}

fn lib_rs() -> String {
    read("crates/dsfb-gpu-atlas-corpus/src/lib.rs")
}

fn required_public_measurement_surfaces() -> Vec<(&'static str, String)> {
    vec![
        ("README.md", readme()),
        (
            "reports/s_perf_6_measured_cuda_pipeline_summary.txt",
            s_perf_6_summary(),
        ),
    ]
}

fn public_and_source_surfaces() -> Vec<(&'static str, String)> {
    let mut surfaces = required_public_measurement_surfaces();
    surfaces.push(("crates/dsfb-gpu-atlas-corpus/src/lib.rs", lib_rs()));
    surfaces
}

fn s_perf_6_section(text: &str) -> Option<String> {
    // Anchor at the first S-PERF.6 section header per markup
    // dialect: README uses `## S-PERF.6`; LaTeX uses
    // `\section{RTX 4080 SUPER Measured CUDA Pipeline`; lib.rs
    // docstrings use a bare "S-PERF.6" bullet. Section ends
    // at the next sibling section break.
    let candidates = [
        "## S-PERF.6",
        "\\section{RTX 4080 SUPER Measured CUDA Pipeline",
        "**S-PERF.6**\n",
        "S-PERF.6",
    ];
    let start = candidates.iter().find_map(|c| text.find(c))?;
    let tail = &text[start..];
    let breaks = ["\n## ", "\n\\section{", "\n//! * **", "\n//! \n"];
    let end = breaks
        .iter()
        // skip the initial anchor: search after the first 8
        // chars so the bracket isn't the section's own header
        .filter_map(|b| tail[8..].find(b).map(|i| i + 8))
        .min()
        .unwrap_or(tail.len());
    Some(tail[..end].to_string())
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

// 1. Public measurement surfaces + lib.rs must NOT contain the word "repair"
//    inside any S-PERF.6 section. The collapse removed the
//    "repair" framing as panel-locked apology branding; any
//    re-introduction is a regression.
#[test]
fn s_perf_6_rejects_public_repair_language() {
    for (name, text) in public_and_source_surfaces() {
        if let Some(section) = s_perf_6_section(&text) {
            assert!(
                !contains_ci(&section, "repair"),
                "{name}: S-PERF.6 section must not contain the word 'repair' \
                 (apology branding removed by the collapse)"
            );
        }
    }
}

// 2. Public measurement surfaces + lib.rs must NOT carry a current-state
//    section header for "S-PERF.6R". The current state is
//    just S-PERF.6; any "S-PERF.6R" mention as a public
//    section is a regression (historical incidental prose
//    elsewhere is allowed).
#[test]
fn s_perf_6_rejects_public_s_perf_6r_current_state() {
    for (name, text) in public_and_source_surfaces() {
        let forbidden_section_headers = [
            "## S-PERF.6R",
            "### S-PERF.6R",
            "\\section{S-PERF.6R",
            "\\subsection{S-PERF.6R",
            "**S-PERF.6R**\n",
            "**S-PERF.6R**\r\n",
        ];
        for f in forbidden_section_headers {
            assert!(
                !text.contains(f),
                "{name}: must not carry a current-state public section header for {f:?}"
            );
        }
    }
}

// 3. Public measurement surfaces must NOT describe the current S-PERF.6
//    performance result as `BandwidthClaimKind::NoClaim` or
//    "NoClaim baseline" or "uninstrumented baseline". The
//    current result IS measured; calling it NoClaim is a
//    regression to the deleted scaffold.
#[test]
fn s_perf_6_rejects_current_no_claim_performance_result() {
    for (name, text) in required_public_measurement_surfaces() {
        if let Some(section) = s_perf_6_section(&text) {
            for forbidden in [
                "BandwidthClaimKind::NoClaim",
                "NoClaim baseline",
                "uninstrumented baseline",
            ] {
                assert!(
                    !contains_ci(&section, forbidden),
                    "{name}: S-PERF.6 section must not describe the current \
                     performance result as {forbidden:?}"
                );
            }
        }
    }
}

// 4. Public measurement surfaces must contain the literal "13.33 GB/s" in
//    the S-PERF.6 section (the measured bandwidth must be
//    visible at the public surface).
#[test]
fn s_perf_6_rejects_missing_13_33_gbps_result() {
    for (name, text) in required_public_measurement_surfaces() {
        let section =
            s_perf_6_section(&text).unwrap_or_else(|| panic!("{name}: no S-PERF.6 section found"));
        assert!(
            section.contains("13.33 GB/s"),
            "{name}: S-PERF.6 section must contain '13.33 GB/s' (the measured bandwidth)"
        );
    }
}

// 5. Public measurement surfaces must contain either "186 bp" or "1.86%"
//    in the S-PERF.6 section (the percent-of-peak must be
//    visible at the public surface).
#[test]
fn s_perf_6_rejects_missing_186_bp_result() {
    for (name, text) in required_public_measurement_surfaces() {
        let section =
            s_perf_6_section(&text).unwrap_or_else(|| panic!("{name}: no S-PERF.6 section found"));
        assert!(
            section.contains("186 bp") || section.contains("1.86%") || section.contains("1.86 %"),
            "{name}: S-PERF.6 section must contain '186 bp' or '1.86%' (percent of peak)"
        );
    }
}

// 6. Public measurement surfaces must contain a public statement that
//    saturation is not admitted (either the explicit
//    "saturation_admitted" with `false`, or the phrase
//    "not a saturation claim" / "not saturation" inside the
//    S-PERF.6 section).
#[test]
fn s_perf_6_rejects_missing_saturation_false() {
    for (name, text) in required_public_measurement_surfaces() {
        let section =
            s_perf_6_section(&text).unwrap_or_else(|| panic!("{name}: no S-PERF.6 section found"));
        let lower = section.to_ascii_lowercase();
        let ok = section.contains("saturation_admitted")
            || lower.contains("not a saturation claim")
            || lower.contains("not saturation");
        assert!(
            ok,
            "{name}: S-PERF.6 section must publicly state saturation is not admitted \
             ('saturation_admitted' / 'not a saturation claim' / 'not saturation')"
        );
    }
}

// 7. Public measurement surfaces must NOT contain any prose that asserts
//    13.33 GB/s reaches saturation. Case-insensitive scan
//    over a closed set of forbidden positive-saturation
//    phrases inside the S-PERF.6 section. The phrases are
//    chosen to fire ONLY on positive-claim prose; the
//    phrase "memory-bandwidth saturation" is intentionally
//    excluded because it commonly appears inside
//    disclaimer prose ("does NOT claim memory-bandwidth
//    saturation").
#[test]
fn s_perf_6_rejects_claim_that_13_33_gbps_is_saturation() {
    let forbidden_phrases = [
        "13.33 GB/s saturates",
        "13.33 saturates",
        "achieves saturation",
        "saturation reached",
        "saturates the bandwidth",
        "saturates peak",
    ];
    for (name, text) in required_public_measurement_surfaces() {
        if let Some(section) = s_perf_6_section(&text) {
            for f in forbidden_phrases {
                assert!(
                    !contains_ci(&section, f),
                    "{name}: S-PERF.6 section must not claim 13.33 GB/s reaches \
                     saturation (forbidden phrase: {f:?})"
                );
            }
        }
    }
}

// 8. Public measurement surfaces must continue to disclose the host
//    segments (`host_compute_features` AND
//    `host_bank_admit_case_finalize`, or their prose form).
//    Removing the host-segment disclosure would hide the
//    panel-locked honest "not pure Layer-A" caveat.
#[test]
fn s_perf_6_rejects_removal_of_host_segment_disclosure() {
    for (name, text) in required_public_measurement_surfaces() {
        let section =
            s_perf_6_section(&text).unwrap_or_else(|| panic!("{name}: no S-PERF.6 section found"));
        // LaTeX escapes underscore as `\_`; normalise so the
        // check works against both markup dialects.
        let normalised = section.replace("\\_", "_");
        let has_features =
            normalised.contains("host_compute_features") || normalised.contains("compute_features");
        let has_bank = normalised.contains("host_bank_admit_case_finalize")
            || normalised.contains("bank admit")
            || normalised.contains("case finalize");
        assert!(
            has_features && has_bank,
            "{name}: S-PERF.6 section must continue to disclose both host segments \
             (compute_features AND bank admit + case finalize)"
        );
    }
}

// 9. `rejects_stale_b300_victory_lap_named_as_s_perf_7` --- per
//    the panel verdict (2026-05-18). S-PERF.7 is the
//    source-report import verifier (Track B leg 1); it is NOT
//    the B300 / GB300 cloud victory-lap. The stale
//    "S-PERF.7 / S-MG.6 victory-lap" wording previously lived
//    in the S-PERF.1 areas of public docs and is forbidden;
//    the corrected form is "B300 / GB300 hardware benchmarking
//    is deferred to a later post-S-PERF / S-MG victory-lap
//    campaign."
#[test]
fn rejects_stale_b300_victory_lap_named_as_s_perf_7() {
    for (name, text) in required_public_measurement_surfaces() {
        // Match the forbidden pattern in any spacing variant.
        let normalised = text
            .replace("\\_", "_")
            .replace("S-PERF.7 / S-MG.6", "S-PERF.7/S-MG.6");
        let lower = normalised.to_ascii_lowercase();
        let forbidden_substrings = [
            "s-perf.7/s-mg.6 victory-lap",
            "s-perf.7 / s-mg.6 victory-lap",
            "s-perf.7/s-mg.6 victory lap",
        ];
        for needle in &forbidden_substrings {
            assert!(
                !lower.contains(needle),
                "{name}: stale victory-lap wording `{needle}` is forbidden; \
                 S-PERF.7 is the source-report import verifier (Track B leg 1), \
                 not the B300/GB300 victory-lap. Replace with \"deferred to a \
                 later post-S-PERF / S-MG victory-lap campaign.\""
            );
        }
    }
}
