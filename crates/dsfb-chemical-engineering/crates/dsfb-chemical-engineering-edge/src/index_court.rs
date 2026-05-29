//! `ArtifactIndexCourtV1` (P98) — makes the generated artifact index (`reports/index.{html,json}`)
//! **self-verifying**.
//!
//! `generate-index` ([`crate::artifact_index`]) emits a deterministic inventory sealed by an `index_root`.
//! On its own that proves the JSON is internally consistent, but not that the *committed* index still
//! reflects reality — a hand-edit, a stale commit, or a moved artifact would go unnoticed. This court
//! closes that gap: it reads the committed `reports/index.json` and checks every claim against the **live**
//! artifacts —
//! - the `index_root` re-derives byte-for-byte from a fresh [`artifact_index::build_index`] (catches a
//!   stale / tampered / hand-edited index);
//! - the paper PDF exists and its SHA-256 matches the recorded `pdf_sha256`, or the index explicitly records that
//!   no paper PDF is bundled in this crate snapshot;
//! - every evidence bundle's `bundle_root`+`evidence_root` matches `data/EXPECTED_BUNDLE_ROOTS.toml`;
//! - every figure's `png_sha256` matches `paper/figures/figure_manifest.json`;
//! - every `docs/` link resolves on disk and every listed crate directory exists;
//! - the controlled-access policy equals [`ControlledAccessDatasetPolicy::STANDING`];
//! - the two embedded court `report_hash`es re-derive from the live completeness + release-scrub courts.
//!
//! It is **standalone** — deliberately NOT added to the index's own `courts` list, because it verifies the
//! index and so must not be part of the inventory it checks (that would be circular and would change
//! `index_root`). Like the sibling courts it emits a structured, hash-sealed pass/fail report
//! (`artifact_index_court_v1`); a failing court is a hard release blocker. Determinism: every finding is a
//! pure function of the committed bytes + the live files, in a fixed order, so `report_hash` is stable.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use crate::artifact_index;
use crate::hashing::{sha256_hex, CanonicalHasher};
use crate::release_scrub::ControlledAccessDatasetPolicy;

const PAPER_NOT_BUNDLED: &str = "not bundled in this crate snapshot";

/// One check outcome; `detail` carries the human-readable evidence.
#[derive(Debug, Clone)]
pub struct Finding {
    pub check: String,
    pub passed: bool,
    pub detail: String,
}

/// The court verdict: ordered findings, tallies, and a deterministic digest over `(check, passed)`.
#[derive(Debug, Clone)]
pub struct IndexCourtReport {
    pub findings: Vec<Finding>,
    pub n_pass: usize,
    pub n_fail: usize,
    pub report_hash: String,
}

impl IndexCourtReport {
    /// The committed index is verified iff no finding failed.
    pub fn all_passed(&self) -> bool {
        self.n_fail == 0
    }

    /// Render the report as plain text (one line per finding + a summary), for the CLI + report files.
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
                "INDEX-VERIFIED"
            } else {
                "INDEX-STALE"
            },
            self.n_pass,
            self.n_fail,
            self.report_hash
        ));
        s
    }
}

/// One `[dataset]` row of `EXPECTED_BUNDLE_ROOTS.toml` (the frozen truth bundle roots are checked against).
#[derive(serde::Deserialize)]
struct ExpectedBundle {
    #[serde(default)]
    bundle_root: String,
    #[serde(default)]
    evidence_root: String,
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn short(s: &str) -> &str {
    if s.len() >= 12 {
        &s[..12]
    } else {
        s
    }
}

/// Run the index court: verify the committed `reports/index.json` against the live artifacts.
/// Pure + deterministic; never panics (I/O and parse failures become failing findings).
pub fn run_index_court(crate_dir: &Path, repo_root: &Path) -> IndexCourtReport {
    let mut f: Vec<Finding> = Vec::new();

    // ── The committed index files must be present + parseable. ────────────────────────────────────
    let index_json_path = repo_root.join("reports").join("index.json");
    let html_present = repo_root.join("reports").join("index.html").exists();
    let committed: Option<Value> = std::fs::read_to_string(&index_json_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    f.push(Finding {
        check: "index:files_present".into(),
        passed: committed.is_some() && html_present,
        detail: format!(
            "index.json {} · index.html {}",
            if committed.is_some() {
                "parsed"
            } else {
                "MISSING/unparseable"
            },
            if html_present { "present" } else { "MISSING" }
        ),
    });
    let committed = match committed {
        Some(c) => c,
        // Without the committed index there is nothing to verify; seal the single failing finding.
        None => return seal(f),
    };

    // ── index_root re-derives from a fresh build (the stale/tamper gate). ─────────────────────────
    let model = artifact_index::build_index(crate_dir, repo_root);
    let rebuilt = serde_json::to_value(&model).unwrap_or(Value::Null);
    let committed_root = committed
        .get("index_root")
        .and_then(Value::as_str)
        .unwrap_or("");
    let rebuilt_root = rebuilt
        .get("index_root")
        .and_then(Value::as_str)
        .unwrap_or("");
    f.push(Finding {
        check: "index:root_is_current".into(),
        passed: is_hex64(rebuilt_root) && rebuilt_root == committed_root,
        detail: format!(
            "committed {}… vs rebuilt {}…",
            short(committed_root),
            short(rebuilt_root)
        ),
    });

    // ── Paper PDF exists and its SHA-256 matches the recorded digest. ─────────────────────────────
    let pdf_path = committed
        .pointer("/paper/pdf_path")
        .and_then(Value::as_str)
        .unwrap_or("");
    let pdf_sha = committed
        .pointer("/paper/pdf_sha256")
        .and_then(Value::as_str)
        .unwrap_or("");
    let live_pdf_sha = std::fs::read(repo_root.join(pdf_path))
        .ok()
        .map(|b| sha256_hex(&b));
    let paper_ok = if pdf_sha == PAPER_NOT_BUNDLED {
        live_pdf_sha.is_none()
    } else {
        is_hex64(pdf_sha) && live_pdf_sha.as_deref() == Some(pdf_sha)
    };
    f.push(Finding {
        check: "index:paper_pdf_matches".into(),
        passed: paper_ok,
        detail: match &live_pdf_sha {
            Some(live) if live == pdf_sha => format!("{} sha256 {}…", pdf_path, short(pdf_sha)),
            Some(live) => format!(
                "{}: recorded {}… but live {}…",
                pdf_path,
                short(pdf_sha),
                short(live)
            ),
            None if pdf_sha == PAPER_NOT_BUNDLED => {
                "paper PDF is explicitly marked not bundled".into()
            }
            None => format!("{}: not readable on disk", pdf_path),
        },
    });

    // ── Every bundle root matches EXPECTED_BUNDLE_ROOTS.toml. ──────────────────────────────────────
    let expected: BTreeMap<String, ExpectedBundle> =
        std::fs::read_to_string(crate_dir.join("data").join("EXPECTED_BUNDLE_ROOTS.toml"))
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();
    let bundles = committed
        .get("bundles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut bundle_mismatch: Option<String> = None;
    for b in &bundles {
        let ds = b.get("dataset").and_then(Value::as_str).unwrap_or("");
        let br = b.get("bundle_root").and_then(Value::as_str).unwrap_or("");
        let er = b.get("evidence_root").and_then(Value::as_str).unwrap_or("");
        match expected.get(ds) {
            None => bundle_mismatch = Some(format!("{ds}: not in EXPECTED_BUNDLE_ROOTS.toml")),
            Some(e) if e.bundle_root != br => {
                bundle_mismatch = Some(format!("{ds}: bundle_root drift"))
            }
            Some(e) if e.evidence_root != er => {
                bundle_mismatch = Some(format!("{ds}: evidence_root drift"))
            }
            Some(_) => {}
        }
        if bundle_mismatch.is_some() {
            break;
        }
    }
    f.push(Finding {
        check: "index:bundle_roots_match_expected".into(),
        passed: bundle_mismatch.is_none() && bundles.len() >= 20,
        detail: match &bundle_mismatch {
            Some(m) => m.clone(),
            None => format!(
                "{} bundles all match EXPECTED_BUNDLE_ROOTS.toml",
                bundles.len()
            ),
        },
    });

    // ── Every figure's PNG SHA matches the figure manifest. ───────────────────────────────────────
    let manifest: Value = std::fs::read_to_string(
        repo_root
            .join("paper")
            .join("figures")
            .join("figure_manifest.json"),
    )
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or(Value::Null);
    let man_figs: BTreeMap<String, String> = manifest
        .get("figures")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|g| {
                    let id = g.get("id").and_then(Value::as_str)?;
                    let sha = g.get("png_sha256").and_then(Value::as_str).unwrap_or("");
                    Some((id.to_string(), sha.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    let figs = committed
        .get("figures")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut fig_mismatch: Option<String> = None;
    for g in &figs {
        let id = g.get("id").and_then(Value::as_str).unwrap_or("");
        let sha = g.get("png_sha256").and_then(Value::as_str).unwrap_or("");
        match man_figs.get(id) {
            Some(m) if m == sha => {}
            Some(_) => fig_mismatch = Some(format!("{id}: png_sha256 drift vs manifest")),
            None => fig_mismatch = Some(format!("{id}: not in figure_manifest.json")),
        }
        if fig_mismatch.is_some() {
            break;
        }
    }
    f.push(Finding {
        check: "index:figure_shas_match_manifest".into(),
        passed: fig_mismatch.is_none(),
        detail: fig_mismatch
            .clone()
            .unwrap_or_else(|| format!("{} figures match the manifest", figs.len())),
    });

    // ── Every docs/ link resolves on disk. ────────────────────────────────────────────────────────
    let docs = committed
        .get("docs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let missing_doc = docs
        .iter()
        .filter_map(|d| d.get("path").and_then(Value::as_str))
        .find(|p| !repo_root.join(p).exists());
    f.push(Finding {
        check: "index:docs_paths_resolve".into(),
        passed: missing_doc.is_none(),
        detail: match missing_doc {
            Some(p) => format!("missing doc on disk: {p}"),
            None => format!("all {} doc links resolve", docs.len()),
        },
    });

    // ── Every listed crate directory exists. ──────────────────────────────────────────────────────
    let crates = committed
        .get("crates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let missing_crate = crates
        .iter()
        .filter_map(|c| c.get("name").and_then(Value::as_str))
        .find(|name| {
            !repo_root
                .join("crates")
                .join(name)
                .join("Cargo.toml")
                .exists()
        });
    f.push(Finding {
        check: "index:crates_exist".into(),
        passed: missing_crate.is_none() && !crates.is_empty(),
        detail: match missing_crate {
            Some(c) => format!("crate dir missing: crates/{c}"),
            None => format!("all {} crate directories exist", crates.len()),
        },
    });

    // ── Controlled-access policy equals the single source of truth. ───────────────────────────────
    let p = ControlledAccessDatasetPolicy::STANDING;
    let pol = |k: &str| {
        committed
            .pointer(&format!("/controlled_access_policy/{k}"))
            .and_then(Value::as_bool)
    };
    let policy_ok = pol("may_ship_raw") == Some(p.may_ship_raw)
        && pol("may_ship_processed_rows") == Some(p.may_ship_processed_rows)
        && pol("may_ship_attack_list") == Some(p.may_ship_attack_list)
        && pol("may_ship_reconstructable_windows") == Some(p.may_ship_reconstructable_windows)
        && pol("may_ship_aggregate_metrics") == Some(p.may_ship_aggregate_metrics)
        && pol("may_ship_scripts") == Some(p.may_ship_scripts);
    f.push(Finding {
        check: "index:policy_matches_standing".into(),
        passed: policy_ok,
        detail: if policy_ok {
            "controlled_access_policy == ControlledAccessDatasetPolicy::STANDING".into()
        } else {
            "controlled_access_policy drifted from STANDING".into()
        },
    });

    // ── The two embedded court report_hashes re-derive from the live courts. ──────────────────────
    let comp = crate::completeness::run_completeness_court(crate_dir);
    let scrub = crate::release_scrub::run_release_scrub(repo_root);
    let court_hashes: Vec<String> = committed
        .get("courts")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|c| c.get("report_hash").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let courts_ok =
        court_hashes.contains(&comp.report_hash) && court_hashes.contains(&scrub.report_hash);
    f.push(Finding {
        check: "index:court_hashes_rederive".into(),
        passed: courts_ok,
        detail: if courts_ok {
            format!(
                "completeness {}… + release-scrub {}… re-derive",
                short(&comp.report_hash),
                short(&scrub.report_hash)
            )
        } else {
            "an embedded court report_hash did not re-derive from the live court".into()
        },
    });

    seal(f)
}

/// Tally + the deterministic `artifact_index_court_v1` report hash (seals only `(check, passed)`).
fn seal(f: Vec<Finding>) -> IndexCourtReport {
    let n_pass = f.iter().filter(|x| x.passed).count();
    let n_fail = f.len() - n_pass;
    let mut h = CanonicalHasher::new();
    h.field("court", b"artifact_index_court_v1");
    for x in &f {
        h.field("check", x.check.as_bytes());
        h.u64("passed", x.passed as u64);
    }
    IndexCourtReport {
        findings: f,
        n_pass,
        n_fail,
        report_hash: h.finalize_hex(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> (std::path::PathBuf, std::path::PathBuf) {
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = crate_dir.parent().unwrap().parent().unwrap().to_path_buf();
        (crate_dir, repo_root)
    }

    #[test]
    fn committed_index_is_verified() {
        let (crate_dir, repo_root) = paths();
        let r = run_index_court(&crate_dir, &repo_root);
        assert!(
            r.all_passed(),
            "index court must pass on the committed tree:\n{}",
            r.render()
        );
        assert_eq!(r.report_hash.len(), 64);
        // The currency gate must be one of the checks.
        assert!(r
            .findings
            .iter()
            .any(|x| x.check == "index:root_is_current" && x.passed));
    }

    #[test]
    fn render_states_a_verdict() {
        let (crate_dir, repo_root) = paths();
        let r = run_index_court(&crate_dir, &repo_root);
        assert!(r.render().contains("verdict: INDEX-VERIFIED"));
    }
}
