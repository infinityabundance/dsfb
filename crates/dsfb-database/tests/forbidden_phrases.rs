//! Tripwire: scan the crate for the eleven forbidden marketing /
//! over-claim phrases that the paper's epistemic-honesty discipline
//! forbids.
//!
//! The Pass-1 plan (`/home/one/.claude/plans/only-focus-on-dsfb-database-curious-scroll.md`)
//! enumerates the list:
//!
//! ```text
//!   real-time, real time, lead-time advantage, flight recorder,
//!   mission-critical, data diode, privacy-preserving,
//!   non-invasive biopsy, 1000th percentile, legendary, masterstroke
//! ```
//!
//! Several of these phrases occur *legitimately* in the current crate
//! and paper — always inside a non-claim, a documented metaphor, or a
//! framing-device citation that explicitly disclaims the marketing read.
//! Those occurrences are line-anchored in [`ALLOWED`] below; any other
//! occurrence in the scanned tree fails this test.
//!
//! ### Self-exclusion
//!
//! This file (`tests/forbidden_phrases.rs`) is itself excluded from the
//! scan via [`is_self_or_target`] because it necessarily contains every
//! forbidden phrase as a string literal (the test data).
//!
//! ### Frozen scope (Pass-2 guardrail)
//!
//! Adding a *new* legitimate occurrence to [`ALLOWED`] is a non-routine
//! edit and requires explicit reasoning in the commit message. The
//! Pass-2 plan freezes the eight currently-allowed sites; any growth of
//! that list signals scope creep and should be reviewed manually.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: &[&str] = &[
    "real-time",
    "real time",
    "lead-time advantage",
    "flight recorder",
    "mission-critical",
    "data diode",
    "privacy-preserving",
    "non-invasive biopsy",
    "1000th percentile",
    "legendary",
    "masterstroke",
];

/// Line-anchored exceptions: (relative_path_from_crate_root, 1-based line, phrase).
///
/// Each entry is a verified-legitimate occurrence as of Pass-2, with
/// rationale recorded in the same row. Any *new* occurrence of a
/// forbidden phrase that is not in this set must be removed (or, if
/// load-bearing, added to this list with a written justification in the
/// PR description — the test will then accept it).
const ALLOWED: &[(&str, usize, &str)] = &[
    // Non-claim #7 in source-of-truth crate const, mirrored in three
    // other locations (paper, README, lock test). All four occurrences
    // are byte-equal and locked by `tests/non_claim_lock.rs`.
    ("src/non_claims.rs", 16, "real-time"),
    ("tests/non_claim_lock.rs", 23, "real-time"),
    ("paper/dsfb-database.tex", 2002, "real-time"),
    ("README.md", 64, "real-time"),
    // README §Live-adapter prose paragraph that explicitly cites the
    // 7th non-claim's "hard real-time" disclaimer.
    ("README.md", 424, "real-time"),
    // Paper §15 ¶7 — "lead-time advantage relative to the same
    // orchestration" is in the *limitation* discussion of why the
    // grammar's structural guarantees do not translate into a wall-clock
    // speedup. The phrase is used to *deny* the marketing read, not
    // assert it.
    ("paper/dsfb-database.tex", 3194, "lead-time advantage"),
    // Paper §16 ¶3 — "mission-critical operators" is a downstream
    // consumer descriptor inside the prior-art-positioning paragraph,
    // not an audience claim. Kept verbatim because the phrase has a
    // specific meaning in the SRE / ops literature being cited.
    ("paper/dsfb-database.tex", 3390, "mission-critical"),
    // src/live/mod.rs L27 — "software data diode" is inside scare-
    // quotes as a metaphor for the three-layer code-audit contract; the
    // surrounding paragraph immediately disclaims any cryptographic
    // diode property. The phrase is cited as a *framing reference* to
    // the term-of-art (NCSC / NSA hardware data-diode concept), not a
    // claim of equivalence.
    ("src/live/mod.rs", 27, "data diode"),
];

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Skip this test file itself (it contains every forbidden phrase as
/// data) and skip output / build directories that are not part of the
/// reviewed surface.
fn is_self_or_target(rel: &Path) -> bool {
    let s = rel.to_string_lossy();
    s == "tests/forbidden_phrases.rs"
        || s.starts_with("target/")
        || s.starts_with("out/")
        || s.starts_with("data/")
        || s.starts_with("colab/")
        || s.starts_with("fuzz/target/")
        || s.starts_with("fuzz/corpus/")
        || s.starts_with("fuzz/artifacts/")
        || s.starts_with("paper/build/")
        || s.starts_with("paper/figs/")
        || s.starts_with("paper/fixtures/")
        || s.starts_with("paper/tables/")
        || s.starts_with("paper/cache/")
        || s.starts_with(".git/")
}

/// Files we actually scan: source, tests, scripts, docs, the paper
/// .tex, README / CHANGELOG, experiment shell + Python drivers, deny /
/// CI configs, benches.
fn is_scanned(rel: &Path) -> bool {
    let Some(name) = rel.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    if name.starts_with('.') {
        return false;
    }
    if let Some(ext) = rel.extension().and_then(|s| s.to_str()) {
        return matches!(
            ext,
            "rs" | "tex" | "md" | "sh" | "py" | "toml" | "yml" | "yaml"
        );
    }
    false
}

fn walk(dir: &Path, root: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        if is_self_or_target(&rel) {
            continue;
        }
        if path.is_dir() {
            walk(&path, root, files);
        } else if is_scanned(&rel) {
            files.push(path);
        }
    }
}

#[test]
fn no_new_forbidden_phrase_occurrences() {
    let root = crate_root();
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "scanner found no files under {} — directory layout drifted?",
        root.display()
    );

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        // Normalise Windows-style separators so allow-list comparisons
        // are platform-independent. (CI runs on Linux but this keeps
        // the test honest under cross-platform contributors.)
        let rel_norm = rel.replace('\\', "/");
        let body = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue, // binary or unreadable — skip silently
        };
        for (lineno, line) in body.lines().enumerate() {
            let lineno = lineno + 1;
            for phrase in FORBIDDEN {
                if !line.contains(phrase) {
                    continue;
                }
                let allowed = ALLOWED
                    .iter()
                    .any(|(f, l, p)| *f == rel_norm && *l == lineno && *p == *phrase);
                if !allowed {
                    violations.push(format!(
                        "{}:{}: forbidden phrase {:?} (not in ALLOWED line-anchored exceptions)",
                        rel_norm, lineno, phrase
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "forbidden-phrase tripwire fired:\n  {}\n\nIf the occurrence is load-bearing (e.g. inside a non-claim or a \
         framing-quote that explicitly disclaims the marketing read), add a (file, line, phrase) tuple to ALLOWED \
         in tests/forbidden_phrases.rs with a comment explaining the rationale.",
        violations.join("\n  ")
    );
}

#[test]
fn allowed_exceptions_are_actually_present() {
    // Defends against `ALLOWED` rotting — if a paper edit shifts a line
    // number, this test fails immediately so the exception entry can be
    // updated rather than silently masking a new violation elsewhere.
    //
    // The `paper/` directory is a local-only working artefact (never
    // published to the public GitHub repo or the crates.io `include`
    // list), so ALLOWED entries that point into `paper/` are validated
    // only when the file is actually present. A missing file is
    // skipped silently; a present-but-drifted file still fires.
    let root = crate_root();
    for (rel, lineno, phrase) in ALLOWED {
        let path = root.join(rel);
        let body = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => panic!("ALLOWED references unreadable file {}: {}", path.display(), e),
        };
        let line = body.lines().nth(*lineno - 1).unwrap_or_else(|| {
            panic!(
                "ALLOWED references {}:{} but file only has {} lines",
                rel,
                lineno,
                body.lines().count()
            )
        });
        assert!(
            line.contains(*phrase),
            "ALLOWED entry {}:{} {:?} is stale: line is {:?}",
            rel,
            lineno,
            phrase,
            line
        );
    }
}
