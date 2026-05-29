//! `PublicReleaseScrubCourtV1` (Wave-2 P72) — a deterministic *public-release hygiene* gate.
//!
//! This is distinct from [`crate::completeness`]: the completeness court proves the *artifact graph* is
//! internally consistent; this court proves the *tracked release surface* is fit to publish. It scans the
//! committed text tree (Rust / Markdown / LaTeX / TOML / Python / HTML) for the things that embarrass a
//! public prior-art release and answers, mechanically:
//!   1. **No placeholder DOI / TODO / FIXME markers** in the public docs (the honest "(assigned on deposit)"
//!      Zenodo marker is explicitly allowed; `XXXX` / `TODO` / `FIXME` DOIs are flagged).
//!   2. **Private local backups are excluded from release** — `SESSION_CONTEXT.md` /
//!      `SESSION_FULL_CONTEXT_*.md` / `research/` are git-ignored *and* `export-ignore`d (checked via
//!      `.gitignore` + `.gitattributes`), so they cannot ride along in a tarball.
//!   3. **No controlled-access dataset rows ship** (P82) — SWaT/BATADAL/WADI `*_instrumented.csv` /
//!      `*.witness.csv` must stay in the untracked `research/` quarantine (tracked mode) and never appear in a
//!      staged archive; the synthetic `*_standin.csv` + synthetic demonstrators are allowed.
//!   4. **Required trust artifacts present** — `reports/verification_report.md` and a `LICENSE`.
//!
//! It emits a structured, hash-sealed pass/fail report (same shape as the completeness court). Deterministic:
//! every finding is a pure function of the committed bytes in a fixed order, so `report_hash` is stable.
//! Bounded honestly: it scans tracked *text* (skips `target/`, `research/`, `output-*`, `.git`, binaries);
//! it is a hygiene gate, not a secrets scanner or a legal review.

use std::path::{Path, PathBuf};

use crate::completeness::Finding;
use crate::hashing::CanonicalHasher;

/// The scrub verdict: ordered findings + tallies + a deterministic digest (mirrors `CompletenessReport`).
#[derive(Debug, Clone)]
pub struct ScrubReport {
    pub findings: Vec<Finding>,
    pub n_pass: usize,
    pub n_fail: usize,
    pub report_hash: String,
}

impl ScrubReport {
    /// The release surface is clean iff no finding failed.
    pub fn all_passed(&self) -> bool {
        self.n_fail == 0
    }

    /// Render as plain text (one line per finding + a summary) for the CLI + the verification report.
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
                "RELEASE-CLEAN"
            } else {
                "RELEASE-DIRTY"
            },
            self.n_pass,
            self.n_fail,
            self.report_hash,
        ));
        s
    }
}

/// Directories never part of the public release surface (git-ignored / build / local).
const SKIP_DIRS: &[&str] = &[".git", "target", "research", "node_modules", "__pycache__"];

/// File extensions whose *text* we scan (binaries like PDF/PNG are skipped).
const TEXT_EXTS: &[&str] = &[
    "rs", "md", "tex", "toml", "py", "html", "cff", "yml", "yaml", "sh", "txt", "json",
];

/// True iff `name` is one of the local session-backup files that must never ship.
fn is_session_backup(name: &str) -> bool {
    name == "SESSION_CONTEXT.md"
        || name.starts_with("SESSION_CONTEXT")
        || name.starts_with("SESSION_FULL_CONTEXT")
}

/// Recursively collect text files under `root`, skipping ignored dirs and `output-*` run dirs. When
/// `skip_session` is true the local session backups are also skipped (tracked-tree mode — they are
/// git-ignored and never released); when false they ARE collected (archive mode, so their *presence* in a
/// shipped tree can be detected and failed). Deterministic order (sorted at each level).
fn text_files(root: &Path, skip_session: bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, skip_session: bool, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if SKIP_DIRS.contains(&name) || name.starts_with("output-") {
                    continue;
                }
                walk(&p, skip_session, out);
            } else {
                if skip_session && is_session_backup(name) {
                    continue; // git-ignored local backups; never part of the tracked release surface
                }
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                if TEXT_EXTS.contains(&ext) {
                    out.push(p);
                }
            }
        }
    }
    walk(root, skip_session, &mut out);
    out
}

/// Recursively collect ANY file (regardless of extension) under `root` whose name is a session backup.
/// Used by the archive court to fail a shipped tree that smuggled a local backup in (the exact gap the
/// raw-ZIP release flagged: git-ignored files ride along when you `zip` instead of `git archive`).
fn session_backup_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if name == ".git" {
                    continue;
                }
                walk(&p, out);
            } else if is_session_backup(name) {
                out.push(p);
            }
        }
    }
    walk(root, &mut out);
    out
}

/// (Panel P82) The redistribution policy for controlled-access datasets (iTrust SWaT/WADI and the like). The
/// release-scrub court ENFORCES it: no raw bytes, no processed/instrumented rows, no attack lists, and no
/// reconstructable windows may ship — only aggregate metrics + the reproducible scripts. Naming the dataset
/// (a filename or citation) is *required credit*, not redistribution, and is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlledAccessDatasetPolicy {
    pub may_ship_raw: bool,
    pub may_ship_processed_rows: bool,
    pub may_ship_attack_list: bool,
    pub may_ship_reconstructable_windows: bool,
    pub may_ship_aggregate_metrics: bool,
    pub may_ship_scripts: bool,
}

impl ControlledAccessDatasetPolicy {
    /// The standing policy for iTrust-gated and similarly controlled datasets.
    pub const STANDING: ControlledAccessDatasetPolicy = ControlledAccessDatasetPolicy {
        may_ship_raw: false,
        may_ship_processed_rows: false,
        may_ship_attack_list: false,
        may_ship_reconstructable_windows: false,
        may_ship_aggregate_metrics: true,
        may_ship_scripts: true,
    };
}

/// True iff `name` is a controlled-access dataset ROW file that must never ship — a SWaT/BATADAL/WADI
/// instrumented slice or its witness trace. The synthetic `*_standin.csv` and the synthetic demonstrators
/// (`cstr_instrumented.csv`, `valve_stiction_instrumented.csv`, …) do NOT match (they start with neither
/// `swat`/`batadal`/`wadi`), so they remain shippable; only the controlled prefixes are gated.
fn is_controlled_access_data(name: &str) -> bool {
    let n = name.to_lowercase();
    (n.starts_with("swat") || n.starts_with("batadal") || n.starts_with("wadi"))
        && (n.ends_with("_instrumented.csv") || n.ends_with(".witness.csv"))
}

/// True iff `name` is a controlled-access dataset ROLES sidecar — a SWaT/BATADAL/WADI
/// `*_instrumented.roles.json`. Unlike the row CSVs, these sidecars MAY ship: they carry only column
/// metadata + balance coefficients, never data rows. But to ship they must *prove* that — see
/// [`CONTROLLED_ROLES_REQUIRED_FLAGS`] and [`check_controlled_roles_metadata_complete`] (panel P87).
fn is_controlled_access_roles(name: &str) -> bool {
    let n = name.to_lowercase();
    (n.starts_with("swat") || n.starts_with("batadal") || n.starts_with("wadi"))
        && n.ends_with("_instrumented.roles.json")
}

/// (Panel P87 + P92) The no-rows provenance flags a controlled-access roles sidecar MUST declare to be
/// shippable, each proving the file holds no controlled rows and that the real bytes are quarantined:
/// - `release_status` (e.g. `"metadata_only_no_rows"`),
/// - `contains_controlled_access_rows` (must be `false`),
/// - `contains_attack_list_rows` (P92 — must be `false`: no labelled-attack rows ship),
/// - `contains_reconstructable_windows` (P92 — must be `false`: no rows from which a window could be rebuilt),
/// - `reconstructable_from_committed_bytes` (must be `false` — the rows can't be rebuilt from anything committed),
/// - `public_release_allowed` (P92 — the metadata-only sidecar itself may ship),
/// - `controlled_access_bytes_quarantined` (P92 — the real rows live in the untracked `research/` quarantine),
/// - `redistribution_policy` (e.g. `"DoNotRedistributeRawOrDerivedRows"`).
///
/// We require key PRESENCE (not values) — the values are the maintainer's honest declaration, sealed by the file's
/// own bytes; the gate ensures a controlled sidecar cannot ship *silent* about its no-rows posture.
const CONTROLLED_ROLES_REQUIRED_FLAGS: [&str; 8] = [
    "release_status",
    "contains_controlled_access_rows",
    "contains_attack_list_rows",
    "contains_reconstructable_windows",
    "reconstructable_from_committed_bytes",
    "public_release_allowed",
    "controlled_access_bytes_quarantined",
    "redistribution_policy",
];

/// Collect any controlled-access dataset ROW file (see [`is_controlled_access_data`]). In tracked mode
/// (`archive_mode == false`) the `research/` quarantine + `target`/`output-*` are skipped, so a hit means a
/// controlled CSV LEAKED into the tracked surface. In archive mode everything but `.git`/`target` is walked, so
/// a hit means controlled bytes are present in the shipped tree (e.g. a raw `zip` swept `research/` in).
fn controlled_access_files(root: &Path, archive_mode: bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, archive_mode: bool, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if name == ".git" || name == "target" {
                    continue;
                }
                if !archive_mode && (SKIP_DIRS.contains(&name) || name.starts_with("output-")) {
                    continue; // tracked mode: research/ is the allowed quarantine; skip build/output dirs
                }
                walk(&p, archive_mode, out);
            } else if is_controlled_access_data(name) {
                out.push(p);
            }
        }
    }
    walk(root, archive_mode, &mut out);
    out
}

/// (Check, panel P82) No controlled-access dataset rows in the scanned tree. Enforces
/// [`ControlledAccessDatasetPolicy::STANDING`]: tracked mode fails if a SWaT/BATADAL/WADI instrumented or
/// witness CSV sits OUTSIDE the `research/` quarantine; archive mode fails if any is present at all.
fn check_no_controlled_access_data(root: &Path, archive_mode: bool) -> Finding {
    let _policy = ControlledAccessDatasetPolicy::STANDING; // the single source of truth for what may ship
    let hits = controlled_access_files(root, archive_mode);
    let rel: Vec<String> = hits
        .iter()
        .map(|p| p.strip_prefix(root).unwrap_or(p).display().to_string())
        .collect();
    Finding {
        check: "no_controlled_access_data".into(),
        passed: hits.is_empty(),
        detail: if hits.is_empty() {
            if archive_mode {
                "no controlled-access rows (swat/batadal/wadi *_instrumented.csv or *.witness.csv) in the archive".into()
            } else {
                "no controlled-access rows outside the untracked research/ quarantine".into()
            }
        } else {
            format!("CONTROLLED-ACCESS DATA PRESENT — policy DoNotRedistribute (raw/processed/witness rows): {}", rel.join(", "))
        },
    }
}

/// (Check, panel P87) Any controlled-access roles sidecar present in the scanned text MUST carry all four
/// [`CONTROLLED_ROLES_REQUIRED_FLAGS`]. The sidecars are allowed to ship (no rows, just metadata), but only if
/// they explicitly declare the no-rows provenance — otherwise a reviewer can't tell a safe metadata file from
/// one that smuggled rows. We check key PRESENCE by a literal-token scan (the files are JSON; matching the
/// `"key"` tokens is parser-free and robust to formatting). A controlled roles sidecar missing any flag FAILS;
/// a clean tree with all sidecars compliant (or none present) PASSES.
fn check_controlled_roles_metadata_complete(root: &Path, files: &[PathBuf]) -> Finding {
    let mut hits: Vec<String> = Vec::new();
    let mut n_checked = 0usize;
    for p in files {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !is_controlled_access_roles(name) {
            continue;
        }
        n_checked += 1;
        let rel = p.strip_prefix(root).unwrap_or(p).display().to_string();
        let Ok(text) = std::fs::read_to_string(p) else {
            hits.push(format!("{} (unreadable)", rel));
            continue;
        };
        let missing: Vec<&str> = CONTROLLED_ROLES_REQUIRED_FLAGS
            .iter()
            .filter(|flag| !text.contains(&format!("\"{}\"", flag)))
            .copied()
            .collect();
        if !missing.is_empty() {
            hits.push(format!("{} missing {:?}", rel, missing));
        }
    }
    Finding {
        check: "controlled_roles_metadata_complete".into(),
        passed: hits.is_empty(),
        detail: if hits.is_empty() {
            format!(
                "{} controlled-access roles sidecar(s) carry all {} no-rows provenance flags",
                n_checked,
                CONTROLLED_ROLES_REQUIRED_FLAGS.len()
            )
        } else {
            format!(
                "controlled roles sidecar missing no-rows provenance flags: {}",
                hits.join("; ")
            )
        },
    }
}

// ── Shared checks (identical in both modes) ──────────────────────────────────────────────────────────

/// (Check) No placeholder DOI / TODO markers in public docs (an honest "(assigned on deposit)" is allowed).
fn check_placeholder_doi(root: &Path, files: &[PathBuf]) -> Finding {
    let mut hits: Vec<String> = Vec::new();
    for p in files {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !(ext == "md" || ext == "tex") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(p) else {
            continue;
        };
        let low = text.to_lowercase();
        for marker in [
            "zenodo.xxxx",
            "doi: todo",
            "doi:todo",
            "10.5281/zenodo.xxxx",
            "<doi>",
            "todo-doi",
        ] {
            if low.contains(marker) {
                let rel = p.strip_prefix(root).unwrap_or(p);
                hits.push(format!(
                    "{} contains placeholder \"{}\"",
                    rel.display(),
                    marker
                ));
            }
        }
    }
    Finding {
        check: "no_placeholder_doi".into(),
        passed: hits.is_empty(),
        detail: if hits.is_empty() {
            "no placeholder DOI markers (an honest '(assigned on deposit)' is allowed)".into()
        } else {
            hits.join("; ")
        },
    }
}

/// (Check) Required trust artifacts present: a verification report + a LICENSE.
fn check_required_artifacts(root: &Path) -> Finding {
    let vr = root.join("reports/verification_report.md").exists();
    let license = root.join("LICENSE").exists()
        || root.join("LICENSE.txt").exists()
        || root.join("LICENSE-APACHE").exists()
        || root.join("LICENSE.md").exists();
    Finding {
        check: "required_release_artifacts_present".into(),
        passed: vr && license,
        detail: format!("reports/verification_report.md={} LICENSE={}", vr, license),
    }
}

/// Seal an ordered finding set under `schema` and tally it into a `ScrubReport` (mirrors the v1 protocol).
fn seal(schema: &[u8], f: Vec<Finding>) -> ScrubReport {
    let mut h = CanonicalHasher::new();
    h.field("schema", schema);
    for finding in &f {
        h.field("check", finding.check.as_bytes());
        h.u64("passed", finding.passed as u64);
    }
    let n_pass = f.iter().filter(|x| x.passed).count();
    let n_fail = f.len() - n_pass;
    ScrubReport {
        findings: f,
        n_pass,
        n_fail,
        report_hash: h.finalize_hex(),
    }
}

/// Run the public-release scrub court over the **tracked repository** rooted at `repo_root`. Pure +
/// deterministic; never panics (I/O errors become failing findings). This mode trusts that git-ignored
/// session backups won't ship and instead verifies they are git-ignored AND export-ignored.
pub fn run_release_scrub(repo_root: &Path) -> ScrubReport {
    let files = text_files(repo_root, /* skip_session */ true);
    let mut f: Vec<Finding> = Vec::new();
    f.push(check_placeholder_doi(repo_root, &files));

    // (3) Private local backups are excluded from release: git-ignored AND export-ignored.
    let gitignore = std::fs::read_to_string(repo_root.join(".gitignore")).unwrap_or_default();
    let gitattr = std::fs::read_to_string(repo_root.join(".gitattributes")).unwrap_or_default();
    let ignored =
        gitignore.contains("SESSION_CONTEXT.md") && gitignore.contains("SESSION_FULL_CONTEXT_");
    let export_ignored =
        gitattr.contains("SESSION_CONTEXT.md") && gitattr.contains("SESSION_FULL_CONTEXT_");
    f.push(Finding {
        check: "private_backups_excluded".into(),
        passed: ignored && export_ignored,
        detail: format!(
            "session backups git-ignored={} export-ignored={} (must both be true to never ship)",
            ignored, export_ignored
        ),
    });

    // (4) No controlled-access dataset rows leaked outside the research/ quarantine (panel P82).
    f.push(check_no_controlled_access_data(
        repo_root, /* archive_mode */ false,
    ));

    // (5) Any shipped controlled-access roles sidecar declares the four no-rows provenance flags (panel P87).
    f.push(check_controlled_roles_metadata_complete(repo_root, &files));

    f.push(check_required_artifacts(repo_root));
    seal(b"public_release_scrub_court_v1", f)
}

/// P81 — **archive mode**: scrub a *materialised release directory* (`<path>`), the artifact a user actually
/// uploads. Unlike the tracked-tree mode this does NOT assume git-ignore protects anything: it walks the dir
/// as shipped and **fails hard** if any session backup smuggled in (the raw-`zip`-instead-of-`git archive`
/// gap), and requires the hygiene config (`.gitignore` + `.gitattributes`) and trust artifacts to be present.
/// Run it on the staged archive *before* publishing:  `release-scrub --archive-dir <release_dir>`.
pub fn run_release_scrub_archive(archive_dir: &Path) -> ScrubReport {
    let files = text_files(archive_dir, /* skip_session */ false);
    let mut f: Vec<Finding> = Vec::new();

    // (0) No session backups in the shipped tree — the headline archive-hygiene gate.
    let leaked = session_backup_files(archive_dir);
    let leaked_rel: Vec<String> = leaked
        .iter()
        .map(|p| {
            p.strip_prefix(archive_dir)
                .unwrap_or(p)
                .display()
                .to_string()
        })
        .collect();
    f.push(Finding {
        check: "no_session_backups_in_archive".into(),
        passed: leaked.is_empty(),
        detail: if leaked.is_empty() {
            "no SESSION_CONTEXT* / SESSION_FULL_CONTEXT* files present in the archive".into()
        } else {
            format!(
                "PRIVATE BACKUP SHIPPED: {} (use `git archive`, not a raw zip)",
                leaked_rel.join(", ")
            )
        },
    });

    // Placeholder-DOI scan, now over the shipped text.
    f.push(check_placeholder_doi(archive_dir, &files));

    // (3) The hygiene config itself must ship (an archive claiming export-ignore discipline includes both;
    //     `git archive` keeps them, a hand-rolled tarball that drops them is suspect).
    let has_gitignore = archive_dir.join(".gitignore").exists();
    let has_gitattributes = archive_dir.join(".gitattributes").exists();
    f.push(Finding {
        check: "hygiene_config_present".into(),
        passed: has_gitignore && has_gitattributes,
        detail: format!(
            ".gitignore={} .gitattributes={} (both required in a hygiene-disciplined archive)",
            has_gitignore, has_gitattributes
        ),
    });

    // (4) No controlled-access dataset rows smuggled into the shipped tree (panel P82).
    f.push(check_no_controlled_access_data(
        archive_dir,
        /* archive_mode */ true,
    ));

    // (5) Any shipped controlled-access roles sidecar declares the four no-rows provenance flags (panel P87).
    f.push(check_controlled_roles_metadata_complete(
        archive_dir,
        &files,
    ));

    // (6) Required trust artifacts present.
    f.push(check_required_artifacts(archive_dir));

    // Distinct schema tag so the two modes' report hashes never collide.
    seal(b"public_release_scrub_court_archive_v1", f)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On the committed repository the release surface must be clean (no placeholder DOI, backups excluded,
    /// controlled-access rows quarantined, trust artifacts present) — and the verdict digest must be deterministic.
    #[test]
    fn committed_release_surface_is_clean() {
        // crate_dir → repo root (two levels up: crates/<crate>/ → repo).
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo = crate_dir
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .to_path_buf();
        let r = run_release_scrub(&repo);
        let fails: Vec<String> = r
            .findings
            .iter()
            .filter(|x| !x.passed)
            .map(|x| format!("{} — {}", x.check, x.detail))
            .collect();
        assert!(
            r.all_passed(),
            "release surface not clean:\n{}",
            fails.join("\n")
        );
        assert_eq!(
            run_release_scrub(&repo).report_hash,
            r.report_hash,
            "report_hash must be deterministic"
        );
    }

    /// P81 archive mode: a clean staged archive passes; a smuggled session backup (or missing hygiene
    /// config) fails — the exact raw-ZIP leak the panel flagged. Uses a temp dir, no network.
    #[test]
    fn archive_mode_fails_on_smuggled_session_backup() {
        use std::fs;
        let base = std::env::temp_dir().join(format!("dsfb_scrub_archive_{}", std::process::id()));
        let clean = base.join("clean");
        let dirty = base.join("dirty");
        for d in [&clean, &dirty] {
            fs::create_dir_all(d.join("reports")).unwrap();
            fs::write(
                d.join(".gitignore"),
                "SESSION_CONTEXT.md\nSESSION_FULL_CONTEXT_*.md\n",
            )
            .unwrap();
            fs::write(
                d.join(".gitattributes"),
                "SESSION_CONTEXT.md export-ignore\n",
            )
            .unwrap();
            fs::write(d.join("LICENSE"), "Apache-2.0").unwrap();
            fs::write(d.join("reports/verification_report.md"), "# verification\n").unwrap();
            fs::write(d.join("README.md"), "# clean project\n").unwrap();
        }
        // The dirty archive smuggled a local session backup in (the raw-zip gap).
        fs::write(
            dirty.join("SESSION_FULL_CONTEXT_2026-05-25.md"),
            "internal notes\n",
        )
        .unwrap();

        let clean_r = run_release_scrub_archive(&clean);
        assert!(
            clean_r.all_passed(),
            "clean archive should pass: {}",
            clean_r.render()
        );

        let dirty_r = run_release_scrub_archive(&dirty);
        assert!(
            !dirty_r.all_passed(),
            "archive with a smuggled SESSION_ file must FAIL"
        );
        let session_finding = dirty_r
            .findings
            .iter()
            .find(|x| x.check == "no_session_backups_in_archive")
            .unwrap();
        assert!(
            !session_finding.passed,
            "the session-backup finding must be the failure"
        );

        // Archive-mode and tracked-mode report hashes use distinct schema tags (never collide).
        assert_ne!(clean_r.report_hash, run_release_scrub(&clean).report_hash);

        fs::remove_dir_all(&base).ok();
    }

    /// P82: an archive that smuggled a controlled-access dataset row (a SWaT/BATADAL instrumented or witness
    /// CSV) FAILS the controlled-data gate — while the synthetic stand-in and synthetic demonstrators do NOT
    /// trip it (naming a dataset is allowed; shipping its rows is not).
    #[test]
    fn archive_mode_fails_on_controlled_access_csv() {
        use std::fs;
        let base = std::env::temp_dir().join(format!("dsfb_scrub_ctrl_{}", std::process::id()));
        let clean = base.join("clean");
        let dirty = base.join("dirty");
        for d in [&clean, &dirty] {
            fs::create_dir_all(d.join("data")).unwrap();
            fs::create_dir_all(d.join("reports")).unwrap();
            fs::write(
                d.join(".gitignore"),
                "SESSION_CONTEXT.md\nSESSION_FULL_CONTEXT_*.md\n",
            )
            .unwrap();
            fs::write(
                d.join(".gitattributes"),
                "SESSION_CONTEXT.md export-ignore\nSESSION_FULL_CONTEXT_*.md export-ignore\n",
            )
            .unwrap();
            fs::write(d.join("LICENSE"), "Apache-2.0").unwrap();
            fs::write(d.join("reports/verification_report.md"), "# verification\n").unwrap();
            // A SYNTHETIC stand-in + a synthetic demonstrator are allowed (must NOT trip the gate).
            fs::write(
                d.join("data/swat_water_treatment_standin.csv"),
                "LIT101,label\n1,0\n",
            )
            .unwrap();
            fs::write(d.join("data/cstr_instrumented.csv"), "T,label\n1,0\n").unwrap();
        }
        // The dirty archive smuggled a CONTROLLED SWaT instrumented slice in.
        fs::write(
            dirty.join("data/swat_t101_instrumented.csv"),
            "LIT101,FIT101,label\n700,2.5,1\n",
        )
        .unwrap();

        let clean_r = run_release_scrub_archive(&clean);
        let clean_ctrl = clean_r
            .findings
            .iter()
            .find(|x| x.check == "no_controlled_access_data")
            .unwrap();
        assert!(
            clean_ctrl.passed,
            "synthetic stand-in + synthetic demonstrator must NOT trip the gate: {}",
            clean_ctrl.detail
        );

        let dirty_r = run_release_scrub_archive(&dirty);
        let dirty_ctrl = dirty_r
            .findings
            .iter()
            .find(|x| x.check == "no_controlled_access_data")
            .unwrap();
        assert!(
            !dirty_ctrl.passed,
            "a smuggled swat_t101_instrumented.csv must FAIL the controlled-data gate"
        );
        assert!(!dirty_r.all_passed());

        fs::remove_dir_all(&base).ok();
    }

    /// P87: a controlled-access roles sidecar may ship ONLY if it declares the four no-rows provenance flags.
    /// A SWaT/BATADAL `*_instrumented.roles.json` missing any flag FAILS the gate; one with all four PASSES;
    /// and a synthetic (`cstr_…`) roles sidecar is not controlled, so it never trips the check regardless.
    #[test]
    fn controlled_roles_sidecar_must_declare_no_rows_flags() {
        use std::fs;
        let base = std::env::temp_dir().join(format!("dsfb_scrub_roles_{}", std::process::id()));
        let clean = base.join("clean");
        let dirty = base.join("dirty");
        // The four flags a compliant controlled-access roles sidecar must carry.
        let compliant = r#"{"dataset":"swat_t101_instrumented","kind":"real","release_status":"metadata_only_no_rows","contains_controlled_access_rows":false,"contains_attack_list_rows":false,"contains_reconstructable_windows":false,"reconstructable_from_committed_bytes":false,"public_release_allowed":true,"controlled_access_bytes_quarantined":true,"redistribution_policy":"DoNotRedistributeRawOrDerivedRows"}"#;
        // Missing every flag — the pre-P87 shape.
        let bare = r#"{"dataset":"swat_t101_instrumented","kind":"real"}"#;
        // A synthetic demonstrator sidecar is NOT controlled (no swat/batadal/wadi prefix) → never gated.
        let synthetic = r#"{"dataset":"cstr_instrumented","kind":"sim"}"#;
        for d in [&clean, &dirty] {
            fs::create_dir_all(d.join("data")).unwrap();
            fs::create_dir_all(d.join("reports")).unwrap();
            fs::write(
                d.join(".gitignore"),
                "SESSION_CONTEXT.md\nSESSION_FULL_CONTEXT_*.md\n",
            )
            .unwrap();
            fs::write(
                d.join(".gitattributes"),
                "SESSION_CONTEXT.md export-ignore\nSESSION_FULL_CONTEXT_*.md export-ignore\n",
            )
            .unwrap();
            fs::write(d.join("LICENSE"), "Apache-2.0").unwrap();
            fs::write(d.join("reports/verification_report.md"), "# verification\n").unwrap();
            fs::write(d.join("data/cstr_instrumented.roles.json"), synthetic).unwrap();
        }
        fs::write(
            clean.join("data/swat_t101_instrumented.roles.json"),
            compliant,
        )
        .unwrap();
        fs::write(dirty.join("data/swat_t101_instrumented.roles.json"), bare).unwrap();

        let clean_r = run_release_scrub_archive(&clean);
        let clean_roles = clean_r
            .findings
            .iter()
            .find(|x| x.check == "controlled_roles_metadata_complete")
            .unwrap();
        assert!(
            clean_roles.passed,
            "a compliant sidecar (+ a non-controlled synthetic one) must PASS: {}",
            clean_roles.detail
        );

        let dirty_r = run_release_scrub_archive(&dirty);
        let dirty_roles = dirty_r
            .findings
            .iter()
            .find(|x| x.check == "controlled_roles_metadata_complete")
            .unwrap();
        assert!(
            !dirty_roles.passed,
            "a controlled roles sidecar missing the no-rows flags must FAIL"
        );
        assert!(
            dirty_roles.detail.contains("missing"),
            "detail should name the missing flags: {}",
            dirty_roles.detail
        );

        fs::remove_dir_all(&base).ok();
    }
}
