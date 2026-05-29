//! `ArtifactCompletenessCourtV1` — a deterministic gate that the committed artifact graph is complete
//! and mutually consistent (P54).
//!
//! The repository makes many cross-referencing claims: the dataset MANIFEST, per-slice SHA-256 digests,
//! the frozen Court Record `bundle_root`/`evidence_root` table, and the authority hashes
//! (`atlas_hash_v1`, `corpus_hash_v1`). Each is checked in isolation elsewhere; this court checks they
//! **agree with each other** — that every dataset the manifest declares also has a digest and a frozen
//! bundle/evidence root, that the two tables name the *same* datasets with consistent provenance kinds,
//! that the authority hashes are well-formed and their crates validate, and that the corpus
//! classification census partitions every record. It emits a structured, hash-sealed pass/fail report.
//!
//! Scope, stated honestly: this gate asserts the **machine-checkable artifact graph** from committed
//! files (TOML + the authority crates). It does **not** parse the paper PDF or re-verify prose counts —
//! that reconciliation is the stale-number sweep done by hand (and at the P71 QA pass). What it *can*
//! and does enforce is that no dataset is declared without a digest + a sealed root, that the manifest
//! and bundle-root tables are name-for-name consistent, and that the authority hashes are present and
//! their validators pass. A failing court is a hard release blocker.
//!
//! Determinism: every finding is a pure function of the committed bytes + the authority crates, in a
//! fixed order, so the `report_hash` is stable on the pinned toolchain.

use std::collections::BTreeMap;
use std::path::Path;

use crate::hashing::CanonicalHasher;

/// One check outcome. `detail` carries the human-readable evidence (counts, the offending name, …).
#[derive(Debug, Clone)]
pub struct Finding {
    pub check: String,
    pub passed: bool,
    pub detail: String,
}

/// The court's verdict: the ordered findings, pass/fail tallies, and a deterministic digest over them.
#[derive(Debug, Clone)]
pub struct CompletenessReport {
    pub findings: Vec<Finding>,
    pub n_pass: usize,
    pub n_fail: usize,
    /// SHA-256 (via [`CanonicalHasher`]) over `(check, passed)` for every finding, in order — a stable
    /// fingerprint of the verdict that the verification report can cite.
    pub report_hash: String,
}

impl CompletenessReport {
    /// The artifact graph is complete + consistent iff no finding failed.
    pub fn all_passed(&self) -> bool {
        self.n_fail == 0
    }

    /// Render the report as plain text (one line per finding + a summary), for the CLI + the report file.
    pub fn render(&self) -> String {
        let mut s = String::new();
        for f in &self.findings {
            s.push_str(if f.passed { "  PASS " } else { "  FAIL " });
            s.push_str(&f.check);
            s.push_str(" — ");
            s.push_str(&f.detail);
            s.push('\n');
        }
        s.push_str(&format!(
            "verdict: {} ({} pass, {} fail)\nreport_hash: {}\n",
            if self.all_passed() {
                "COMPLETE"
            } else {
                "INCOMPLETE"
            },
            self.n_pass,
            self.n_fail,
            self.report_hash
        ));
        s
    }
}

/// True iff `s` is exactly 64 lowercase/uppercase hex characters (a SHA-256 digest).
fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

// ── TOML shapes (subset; `#[serde(default)]` so unrelated fields are ignored) ──────────────────────

#[derive(serde::Deserialize)]
struct ManifestEntry {
    name: String,
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default)]
    kind: Option<String>,
}

#[derive(serde::Deserialize)]
struct Manifest {
    #[serde(default)]
    dataset_count: usize,
    #[serde(default)]
    dataset: Vec<ManifestEntry>,
}

#[derive(serde::Deserialize)]
struct BundleEntry {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    bundle_root: String,
    #[serde(default)]
    evidence_root: String,
}

/// Run the completeness court over the committed artifacts under `crate_dir/data/` + the authority
/// crates. Pure + deterministic; never panics (I/O and parse errors become failing findings).
pub fn run_completeness_court(crate_dir: &Path) -> CompletenessReport {
    let mut f: Vec<Finding> = Vec::new();
    let data = crate_dir.join("data");

    // ── MANIFEST.toml ───────────────────────────────────────────────────────────────────────────
    let manifest_path = data.join("MANIFEST.toml");
    let manifest: Option<Manifest> = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|t| toml::from_str(&t).ok());
    let mut manifest_names: Vec<String> = Vec::new();
    match &manifest {
        None => f.push(Finding {
            check: "manifest:parse".into(),
            passed: false,
            detail: format!("could not read/parse {}", manifest_path.display()),
        }),
        Some(m) => {
            manifest_names = m.dataset.iter().map(|d| d.name.clone()).collect();
            // dataset_count field matches the actual number of [[dataset]] blocks.
            f.push(Finding {
                check: "manifest:declared_count_matches_blocks".into(),
                passed: m.dataset_count == m.dataset.len(),
                detail: format!(
                    "dataset_count={} blocks={}",
                    m.dataset_count,
                    m.dataset.len()
                ),
            });
            // Every dataset carries a well-formed SHA-256 digest.
            let bad: Vec<&str> = m
                .dataset
                .iter()
                .filter(|d| !d.sha256.as_deref().map(is_hex64).unwrap_or(false))
                .map(|d| d.name.as_str())
                .collect();
            f.push(Finding {
                check: "manifest:every_dataset_has_sha256".into(),
                passed: bad.is_empty(),
                detail: if bad.is_empty() {
                    format!("{} datasets, all with 64-hex sha256", m.dataset.len())
                } else {
                    format!("missing/!hex64 sha256: {bad:?}")
                },
            });
        }
    }

    // ── EXPECTED_BUNDLE_ROOTS.toml ──────────────────────────────────────────────────────────────
    let bundles_path = data.join("EXPECTED_BUNDLE_ROOTS.toml");
    let bundles: Option<BTreeMap<String, BundleEntry>> = std::fs::read_to_string(&bundles_path)
        .ok()
        .and_then(|t| toml::from_str(&t).ok());
    let mut bundle_names: Vec<String> = Vec::new();
    match &bundles {
        None => f.push(Finding {
            check: "bundle_roots:parse".into(),
            passed: false,
            detail: format!("could not read/parse {}", bundles_path.display()),
        }),
        Some(b) => {
            bundle_names = b.keys().cloned().collect();
            let bad: Vec<&str> = b
                .iter()
                .filter(|(_, e)| !(is_hex64(&e.bundle_root) && is_hex64(&e.evidence_root)))
                .map(|(k, _)| k.as_str())
                .collect();
            f.push(Finding {
                check: "bundle_roots:every_entry_has_64hex_roots".into(),
                passed: bad.is_empty(),
                detail: if bad.is_empty() {
                    format!(
                        "{} entries, all with 64-hex bundle_root + evidence_root",
                        b.len()
                    )
                } else {
                    format!("missing/!hex64 roots: {bad:?}")
                },
            });
        }
    }

    // ── Cross-table consistency: MANIFEST names == BUNDLE_ROOTS names ────────────────────────────
    if manifest.is_some() && bundles.is_some() {
        let man: std::collections::BTreeSet<&String> = manifest_names.iter().collect();
        let bun: std::collections::BTreeSet<&String> = bundle_names.iter().collect();
        let only_manifest: Vec<&String> = man.difference(&bun).cloned().collect();
        let only_bundle: Vec<&String> = bun.difference(&man).cloned().collect();
        f.push(Finding {
            check: "cross:manifest_and_bundle_roots_name_for_name".into(),
            passed: only_manifest.is_empty() && only_bundle.is_empty(),
            detail: if only_manifest.is_empty() && only_bundle.is_empty() {
                format!(
                    "both tables name the same {} datasets",
                    manifest_names.len()
                )
            } else {
                format!("manifest-only={only_manifest:?} bundle-only={only_bundle:?}")
            },
        });
        // Provenance-kind consistency: each manifest kind maps to the bundle's data_kind tag.
        if let (Some(m), Some(b)) = (&manifest, &bundles) {
            let mut mismatched: Vec<String> = Vec::new();
            for d in &m.dataset {
                if let (Some(k), Some(be)) = (d.kind.as_deref(), b.get(&d.name)) {
                    let want = crate::datasets::data_kind_tag(k);
                    if want != be.kind {
                        mismatched.push(format!("{}({k}->{want}!={})", d.name, be.kind));
                    }
                }
            }
            f.push(Finding {
                check: "cross:provenance_kind_tags_agree".into(),
                passed: mismatched.is_empty(),
                detail: if mismatched.is_empty() {
                    "every manifest kind matches its bundle data_kind tag".into()
                } else {
                    format!("kind mismatches: {mismatched:?}")
                },
            });
        }
    }

    // ── Authority hashes: atlas ─────────────────────────────────────────────────────────────────
    match dsfb_chemical_engineering_atlas::validate() {
        Ok(_) => {
            let h = dsfb_chemical_engineering_atlas::hashes::atlas_hash_v1();
            f.push(Finding {
                check: "authority:atlas_validates_and_hash_wellformed".into(),
                passed: is_hex64(&h),
                detail: format!("atlas_hash_v1={}…", &h[..16.min(h.len())]),
            });
        }
        Err(e) => f.push(Finding {
            check: "authority:atlas_validates_and_hash_wellformed".into(),
            passed: false,
            detail: format!("atlas validation failed: {e}"),
        }),
    }

    // ── Authority hashes + census: corpus (only when the optional feature is compiled in) ─────────
    #[cfg(feature = "soft-sensor-corpus")]
    {
        match dsfb_chemical_engineering_corpus::validate() {
            Ok(_) => {
                let h = dsfb_chemical_engineering_corpus::corpus_hash_v1();
                f.push(Finding {
                    check: "authority:corpus_validates_and_hash_wellformed".into(),
                    passed: is_hex64(&h),
                    detail: format!("corpus_hash_v1={}…", &h[..16.min(h.len())]),
                });
                // The P53 classification census must partition every record on all four axes.
                let c = dsfb_chemical_engineering_corpus::census();
                let ok = c.license_total() == c.n_datasets
                    && c.access_total() == c.n_datasets
                    && c.redistribution_total() == c.n_datasets
                    && c.authority_total() == c.n_datasets;
                f.push(Finding {
                    check: "authority:corpus_census_partitions_all_records".into(),
                    passed: ok,
                    detail: format!(
                        "n={} licence={} access={} redist={} authority={}",
                        c.n_datasets,
                        c.license_total(),
                        c.access_total(),
                        c.redistribution_total(),
                        c.authority_total()
                    ),
                });
            }
            Err(e) => f.push(Finding {
                check: "authority:corpus_validates_and_hash_wellformed".into(),
                passed: false,
                detail: format!("corpus validation failed: {e}"),
            }),
        }
    }

    // ── Metrics protocol present (denominators for the headline rates) ───────────────────────────
    let metrics_path = data.join("METRICS_DEFINITIONS.toml");
    let metrics: Option<toml::Table> = std::fs::read_to_string(&metrics_path)
        .ok()
        .and_then(|t| t.parse::<toml::Table>().ok());
    match &metrics {
        None => f.push(Finding {
            check: "metrics:definitions_present".into(),
            passed: false,
            detail: format!("could not read/parse {}", metrics_path.display()),
        }),
        Some(t) => {
            // The headline rates all live under these protocol sections.
            let want = ["swat_t101", "batadal_t1", "tep_idv01"];
            let missing: Vec<&str> = want
                .iter()
                .copied()
                .filter(|k| !t.contains_key(*k))
                .collect();
            f.push(Finding {
                check: "metrics:headline_rate_sections_present".into(),
                passed: missing.is_empty(),
                detail: if missing.is_empty() {
                    format!("protocol sections present: {want:?}")
                } else {
                    format!("missing metric sections: {missing:?}")
                },
            });
        }
    }

    // ── Tally + deterministic report hash ────────────────────────────────────────────────────────
    let n_pass = f.iter().filter(|x| x.passed).count();
    let n_fail = f.len() - n_pass;
    let mut h = CanonicalHasher::new();
    h.field("court", b"artifact_completeness_court_v1");
    for x in &f {
        h.field("check", x.check.as_bytes());
        h.u64("passed", x.passed as u64);
    }
    CompletenessReport {
        findings: f,
        n_pass,
        n_fail,
        report_hash: h.finalize_hex(),
    }
}
