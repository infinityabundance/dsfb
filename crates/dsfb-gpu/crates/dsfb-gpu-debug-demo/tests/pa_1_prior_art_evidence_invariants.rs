//! PA.1 Prior-Art Evidence Package — panel-required invariant tests.
//!
//! These six tests pin the repository-level prior-art evidence
//! package against drift. Each test walks one of the eight artifact
//! files at the repo root and verifies the panel-locked properties
//! the package's external readability depends on.
//!
//! Path discipline: this integration test lives at
//! `crates/dsfb-gpu-debug-demo/tests/`, so repo-root files are
//! accessed via `../../<filename>` relative to the crate's
//! cargo-test cwd. The existing `s_real_3_bundle_integrity`
//! suite uses the same pattern.
//!
//! Hash discipline: the SHA-256 over file bytes is computed via the
//! workspace's zero-dep `dsfb_gpu_debug_core::hash::sha256` (FIPS
//! 180-4 compliant, audited as part of the v0 prior-art proof).
//! Re-using that helper means the manifest-validation hash and the
//! per-stage chain hashes share one source of truth.

use dsfb_gpu_debug_core::hash::sha256;
use std::fmt::Write as _;
use std::fs;

// ----------------------------------------------------------------------
// Panel-locked element list (17 disclosed architecture elements).
// Element 18 (commercial-clean subset partition) is panel-deferred to
// S-REAL.4 (post-RELEASE.1) and is documented as PENDING in
// PRIOR_ART_MAP.md and CLAIM_BOUNDARY_MATRIX.md.
// ----------------------------------------------------------------------

/// Substrings that MUST appear (case-sensitive) somewhere in
/// `PRIOR_ART_MAP.md`. Each substring is the panel-locked element
/// name; missing any one fails the build before commit.
const PANEL_LOCKED_ELEMENT_NAMES: &[&str] = &[
    "Endoductive evidence court",
    "Densor / tekmeric evidence model",
    "CUDA evidence factory / CPU court split",
    "Semantic Non-Bypass Axiom",
    "BankAdmissionToken private-constructor enforcement",
    "Q16.16 fixed-point deterministic numeric contract",
    "Locked CUDA kernel sequence",
    "Stage hash chain / verdict case file",
    "Device Traffic Receipt / measurement law",
    "Layer-A resident densor pipeline",
    "Family compaction / detector-count-not-kernel-count",
    "Digest preservation contract",
    "A6.1 structural fusion optimisation",
    "S-REAL 20-dataset replay audit",
    "30-fixture saturation-regime classifier",
    "Colab public replay gate",
    "DPU architectural implication boundary",
];

/// Panel-locked CITATION.cff required field keys. Each must appear
/// (as a top-level key line `key:`) in the CFF file; the test does
/// substring matching on the canonical key form to remain robust
/// against whitespace variations YAML allows.
const CITATION_REQUIRED_FIELD_KEYS: &[&str] = &[
    "cff-version:",
    "message:",
    "title:",
    "type:",
    "authors:",
    "license:",
    "repository-code:",
    "abstract:",
    "keywords:",
];

/// Panel-locked SBOM license / creation-info fields. The first three
/// are top-level document-level fields; the last two are package-level
/// fields on the primary `dsfb-gpu-debug` package.
const SBOM_REQUIRED_FIELDS: &[&str] = &[
    "\"dataLicense\":",
    "\"creationInfo\":",
    "\"licenseConcluded\":",
    "\"licenseDeclared\":",
];

/// Forbidden placeholder URLs / strings inside `TIMESTAMP_RECEIPT.md`.
///
/// Explicit "Pending RELEASE.1" markers are NOT forbidden (they are
/// honest deferrals); only the patterns below indicate sloppy
/// drafting that would undermine the receipt's evidentiary value.
///
/// Bare stale-marker tokens (the workspace-wide convention covering
/// the four canonical to-be-resolved shorthands) are caught
/// independently by `scripts/docs_freshness.sh` — this list is
/// scoped to the placeholder-URL patterns docs_freshness does NOT
/// catch.
const FORBIDDEN_PLACEHOLDER_TOKENS: &[&str] = &[
    "https://example.com",
    "<URL TBD>",
    "<DOI TBD>",
    "<SWHID TBD>",
    "<PENDING>",
    "lorem ipsum",
];

// ----------------------------------------------------------------------
// Repo-root file helpers
// ----------------------------------------------------------------------

/// Open a repo-root file at `path` and return its bytes. Panics with
/// a clear message if the file is missing — that is the panel-locked
/// contract: every PA.1 artifact MUST exist on disk pre-commit.
fn read_repo_root(path: &str) -> Vec<u8> {
    let full = format!("../../{path}");
    fs::read(&full).unwrap_or_else(|e| panic!("PA.1 artifact `{path}` missing at `{full}`: {e}"))
}

/// Read a repo-root text file as UTF-8.
fn read_repo_root_text(path: &str) -> String {
    let full = format!("../../{path}");
    fs::read_to_string(&full)
        .unwrap_or_else(|e| panic!("PA.1 artifact `{path}` missing at `{full}`: {e}"))
}

fn sha256_hex_lower(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        // `write!` instead of `push_str(&format!(...))` keeps clippy
        // happy on the `format_push_string` lint without an extra
        // allocation per byte.
        let _ = write!(out, "{b:02x}");
    }
    out
}

// ----------------------------------------------------------------------
// Test 1 — PRIOR_ART_MAP.md mentions all 17 elements
// ----------------------------------------------------------------------

#[test]
fn prior_art_map_mentions_all_core_elements() {
    let body = read_repo_root_text("PRIOR_ART_MAP.md");
    let mut missing: Vec<&str> = Vec::new();
    for name in PANEL_LOCKED_ELEMENT_NAMES {
        if !body.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "PRIOR_ART_MAP.md is missing panel-locked element names: {missing:?}",
    );
}

// ----------------------------------------------------------------------
// Test 2 — ARTIFACT_MANIFEST.v1.toml paths exist + SHA-256 matches
// ----------------------------------------------------------------------

/// One parsed manifest entry — minimal fields the test needs.
#[derive(Debug, Default)]
struct ManifestEntry {
    path: Option<String>,
    sha256: Option<String>,
    availability: Option<String>,
}

/// Tiny TOML-subset parser for `[[artifact]]` array-of-tables in
/// `ARTIFACT_MANIFEST.v1.toml`. The format is regular enough that a
/// dedicated extractor is cleaner than pulling in a TOML crate.
fn parse_artifact_manifest(text: &str) -> Vec<ManifestEntry> {
    let mut entries: Vec<ManifestEntry> = Vec::new();
    let mut current: Option<ManifestEntry> = None;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[artifact]]" {
            if let Some(e) = current.take() {
                entries.push(e);
            }
            current = Some(ManifestEntry::default());
            continue;
        }
        // A non-artifact section header closes the current entry.
        if line.starts_with('[') && line.ends_with(']') {
            if let Some(e) = current.take() {
                entries.push(e);
            }
            continue;
        }
        let Some(entry) = current.as_mut() else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        let value_unquoted = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        };
        match key {
            "path" => entry.path = Some(value_unquoted),
            "sha256" => entry.sha256 = Some(value_unquoted),
            "availability" => entry.availability = Some(value_unquoted),
            _ => {}
        }
    }
    if let Some(e) = current {
        entries.push(e);
    }
    entries
}

#[test]
fn artifact_manifest_paths_exist() {
    let body = read_repo_root_text("ARTIFACT_MANIFEST.v1.toml");
    let entries = parse_artifact_manifest(&body);
    assert!(
        !entries.is_empty(),
        "ARTIFACT_MANIFEST.v1.toml parsed zero entries; manifest is empty?",
    );
    for entry in &entries {
        let path = entry
            .path
            .as_ref()
            .unwrap_or_else(|| panic!("manifest entry missing `path`"));
        let recorded = entry
            .sha256
            .as_ref()
            .unwrap_or_else(|| panic!("manifest entry for `{path}` missing `sha256`"));
        assert!(
            recorded.len() == 64
                && recorded
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "manifest SHA-256 for `{path}` must be 64 lowercase hex chars; got `{recorded}`",
        );
        if entry.availability.as_deref() == Some("external-archive") {
            continue;
        }
        let bytes = read_repo_root(path);
        let actual = sha256_hex_lower(&bytes);
        assert_eq!(
            recorded.to_lowercase(),
            actual,
            "manifest SHA-256 drift for `{path}`: recorded `{recorded}` vs actual `{actual}`",
        );
    }
}

// ----------------------------------------------------------------------
// Test 3 — TIMESTAMP_RECEIPT.md has no placeholder URLs
// ----------------------------------------------------------------------

#[test]
fn timestamp_receipt_has_no_placeholder_urls() {
    let body = read_repo_root_text("TIMESTAMP_RECEIPT.md");
    let mut hits: Vec<&str> = Vec::new();
    for forbidden in FORBIDDEN_PLACEHOLDER_TOKENS {
        if body.contains(forbidden) {
            hits.push(forbidden);
        }
    }
    assert!(
        hits.is_empty(),
        "TIMESTAMP_RECEIPT.md contains forbidden placeholder tokens: {hits:?}. \
         Explicit `Pending RELEASE.1` markers are allowed; bare TODOs / \
         placeholder URLs are not.",
    );
}

// ----------------------------------------------------------------------
// Test 4 — CITATION.cff has the panel-required fields
// ----------------------------------------------------------------------

#[test]
fn citation_cff_has_required_fields() {
    let body = read_repo_root_text("CITATION.cff");
    let mut missing: Vec<&str> = Vec::new();
    for key in CITATION_REQUIRED_FIELD_KEYS {
        if !body.contains(key) {
            missing.push(key);
        }
    }
    assert!(
        missing.is_empty(),
        "CITATION.cff is missing panel-required fields: {missing:?}",
    );

    // Extra rigour: the `type:` field MUST be `software` (Zenodo /
    // GitHub citation tooling routes the entry as software metadata,
    // not as a dataset or article).
    assert!(
        body.contains("type: software"),
        "CITATION.cff `type:` must be `software` to be routed as \
         software metadata; got something else",
    );
}

// ----------------------------------------------------------------------
// Test 5 — sbom.spdx.json has the repository license fields
// ----------------------------------------------------------------------

#[test]
fn spdx_sbom_has_repository_license_fields() {
    let body = read_repo_root_text("sbom.spdx.json");
    let mut missing: Vec<&str> = Vec::new();
    for field in SBOM_REQUIRED_FIELDS {
        if !body.contains(field) {
            missing.push(field);
        }
    }
    assert!(
        missing.is_empty(),
        "sbom.spdx.json is missing panel-required SPDX fields: {missing:?}",
    );

    // The document-level `dataLicense` MUST be `CC0-1.0` per SPDX 2.3
    // recommendation for SBOMs (allows third-party reuse without
    // re-licensing the metadata).
    assert!(
        body.contains("\"dataLicense\": \"CC0-1.0\""),
        "sbom.spdx.json `dataLicense` must be `CC0-1.0` per SPDX 2.3 \
         recommendation; got something else",
    );
    // The primary package's licenseConcluded / licenseDeclared MUST be
    // Apache-2.0 (matches LICENSE + NOTICE + Cargo.toml license field).
    assert!(
        body.contains("\"licenseConcluded\": \"Apache-2.0\""),
        "sbom.spdx.json packages[0].licenseConcluded must be Apache-2.0",
    );
    assert!(
        body.contains("\"licenseDeclared\": \"Apache-2.0\""),
        "sbom.spdx.json packages[0].licenseDeclared must be Apache-2.0",
    );
}

// ----------------------------------------------------------------------
// Test 6 — CLAIM_BOUNDARY_MATRIX.md has a `Not claimed` cell per element
// ----------------------------------------------------------------------

#[test]
fn claim_boundary_matrix_has_nonclaim_for_each_claim() {
    let body = read_repo_root_text("CLAIM_BOUNDARY_MATRIX.md");
    // Every element row uses the `### Element N — <name>` header
    // exactly once, then the body of that subsection MUST contain a
    // `- **Not claimed:**` line. We count how many element headers
    // appear and how many `Not claimed:` markers appear; both counts
    // MUST equal the panel-locked element count (17 active + 1
    // pending element 18). Element 18 ("PENDING") carries an explicit
    // "panel-deferred to S-REAL.4" line instead of a Not-claimed line,
    // so the test accepts either a `Not claimed:` marker OR an
    // explicit `(PENDING)` marker in the element section to remain
    // robust to that one row.
    let element_header_count = body.matches("### Element ").count();
    assert!(
        element_header_count >= 18,
        "CLAIM_BOUNDARY_MATRIX.md must declare at least 18 elements \
         (17 active + 1 pending element 18 / S-REAL.4 + future growth); \
         found {element_header_count}",
    );
    let not_claimed_count = body.matches("- **Not claimed:**").count();
    let pending_count = body.matches("(PENDING)").count();
    assert!(
        not_claimed_count + pending_count >= element_header_count,
        "CLAIM_BOUNDARY_MATRIX.md must carry a `- **Not claimed:**` \
         marker (or `(PENDING)` for the deferred element 18) for every \
         element row; found {not_claimed_count} not-claimed + \
         {pending_count} pending = {} total (need >= {element_header_count})",
        not_claimed_count + pending_count,
    );

    // Tightest invariant: the 17 active element headers MUST each be
    // followed (within the section) by a `Not claimed:` line. Walk
    // the file once and verify the per-section presence.
    let mut sections: Vec<&str> = body.split("### Element ").collect();
    // First chunk is the preface above the first element header;
    // discard it.
    if !sections.is_empty() {
        sections.remove(0);
    }
    assert!(
        sections.len() >= 18,
        "split shape unexpected; found {} sections",
        sections.len()
    );
    for (i, section) in sections.iter().enumerate() {
        let element_num = i + 1;
        if element_num == 18 {
            // Element 18 is the explicit PENDING row.
            assert!(
                section.contains("PENDING") || section.contains("panel-deferred"),
                "Element 18 must be explicitly marked PENDING / panel-deferred",
            );
        } else {
            assert!(
                section.contains("- **Not claimed:**"),
                "Element {element_num} section must contain a `- **Not claimed:**` line; \
                 first 120 chars: {}",
                &section[..section.len().min(120)],
            );
        }
    }
}
