//! Machine-readable theorem-to-code traceability helpers.
//!
//! This module scans implementation source for structured `TRACE:` tags and renders the
//! human-auditable theorem-to-code matrix committed under `docs/`.

use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

pub const TRACE_TAG_PREFIX: &str = "TRACE:";
pub const TRACEABILITY_MATRIX_RELATIVE_PATH: &str = "docs/THEOREM_TO_CODE_TRACEABILITY.md";
pub const TRACEABILITY_GUIDE_RELATIVE_PATH: &str = "docs/traceability.md";

const SCAN_ROOTS: &[&str] = &[
    "src",
    "tests",
    "examples",
    "ffi/src",
    "ffi/include",
    "ffi/examples",
    "python/src",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaperItemType {
    Theorem,
    Proposition,
    Lemma,
    Corollary,
    Definition,
    Assumption,
    Algorithm,
    Claim,
    Interface,
}

impl PaperItemType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Theorem => "THEOREM",
            Self::Proposition => "PROPOSITION",
            Self::Lemma => "LEMMA",
            Self::Corollary => "COROLLARY",
            Self::Definition => "DEFINITION",
            Self::Assumption => "ASSUMPTION",
            Self::Algorithm => "ALGORITHM",
            Self::Claim => "CLAIM",
            Self::Interface => "INTERFACE",
        }
    }

    #[must_use]
    pub const fn sort_key(self) -> u8 {
        match self {
            Self::Theorem => 0,
            Self::Proposition => 1,
            Self::Lemma => 2,
            Self::Corollary => 3,
            Self::Definition => 4,
            Self::Assumption => 5,
            Self::Algorithm => 6,
            Self::Claim => 7,
            Self::Interface => 8,
        }
    }
}

impl Display for PaperItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PaperItemType {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "THEOREM" => Ok(Self::Theorem),
            "PROPOSITION" => Ok(Self::Proposition),
            "LEMMA" => Ok(Self::Lemma),
            "COROLLARY" => Ok(Self::Corollary),
            "DEFINITION" => Ok(Self::Definition),
            "ASSUMPTION" => Ok(Self::Assumption),
            "ALGORITHM" => Ok(Self::Algorithm),
            "CLAIM" => Ok(Self::Claim),
            "INTERFACE" => Ok(Self::Interface),
            _ => Err(format!("unsupported paper item type `{value}`")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceTag {
    pub item_type: PaperItemType,
    pub item_id: String,
    pub short_title: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceEntry {
    pub item_type: PaperItemType,
    pub item_id: String,
    pub short_title: String,
    pub file: String,
    pub line: usize,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceDiagnostic {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub source_line: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceabilityScan {
    pub entries: Vec<TraceEntry>,
    pub diagnostics: Vec<TraceDiagnostic>,
}

#[must_use]
pub fn default_matrix_path(crate_root: &Path) -> PathBuf {
    crate_root.join(TRACEABILITY_MATRIX_RELATIVE_PATH)
}

pub fn parse_trace_tag(raw: &str) -> std::result::Result<TraceTag, String> {
    let trimmed = raw.trim();
    let Some(start) = trimmed.find(TRACE_TAG_PREFIX) else {
        return Err("missing TRACE: prefix".to_string());
    };
    let content = &trimmed[start..];
    let mut parts = content.splitn(5, ':');
    let prefix = parts.next().unwrap_or_default();
    if prefix != "TRACE" {
        return Err("tag must start with TRACE".to_string());
    }

    let item_type = parts
        .next()
        .ok_or_else(|| "missing paper item type".to_string())?
        .parse::<PaperItemType>()?;
    let item_id = parts
        .next()
        .ok_or_else(|| "missing paper item identifier".to_string())?
        .trim()
        .to_string();
    if !valid_trace_id(&item_id) {
        return Err(format!(
            "paper item identifier `{item_id}` must use uppercase hyphenated tokens"
        ));
    }

    let short_title = parts
        .next()
        .ok_or_else(|| "missing short title".to_string())?
        .trim()
        .to_string();
    if short_title.is_empty() {
        return Err("short title must be non-empty".to_string());
    }
    if short_title.contains('|') {
        return Err("short title must not contain `|`".to_string());
    }

    let note = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    if note.as_deref().is_some_and(|value| value.contains('|')) {
        return Err("note must not contain `|`".to_string());
    }

    Ok(TraceTag {
        item_type,
        item_id,
        short_title,
        note,
    })
}

pub fn collect_traceability(crate_root: &Path) -> Result<TraceabilityScan> {
    let mut files = Vec::new();
    for relative_root in SCAN_ROOTS {
        let candidate = crate_root.join(relative_root);
        collect_scan_files(crate_root, &candidate, &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut entries = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for relative_file in files {
        let absolute = crate_root.join(&relative_file);
        let content = fs::read_to_string(&absolute)
            .with_context(|| format!("failed to read `{}`", absolute.display()))?;
        for (line_index, line) in content.lines().enumerate() {
            let Some(candidate) = extract_trace_candidate(line) else {
                continue;
            };
            match parse_trace_tag(candidate) {
                Ok(tag) => {
                    entries.insert(TraceEntry {
                        item_type: tag.item_type,
                        item_id: tag.item_id,
                        short_title: tag.short_title,
                        file: relative_file.clone(),
                        line: line_index + 1,
                        note: tag.note,
                    });
                }
                Err(message) => diagnostics.push(TraceDiagnostic {
                    file: relative_file.clone(),
                    line: line_index + 1,
                    message,
                    source_line: line.trim().to_string(),
                }),
            }
        }
    }

    let mut entries = entries.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.item_type
            .sort_key()
            .cmp(&right.item_type.sort_key())
            .then_with(|| left.item_id.cmp(&right.item_id))
            .then_with(|| left.file.cmp(&right.file))
            .then_with(|| left.line.cmp(&right.line))
    });

    Ok(TraceabilityScan {
        entries,
        diagnostics,
    })
}

pub fn generate_traceability_matrix(crate_root: &Path) -> Result<String> {
    let scan = collect_traceability(crate_root)?;
    if !scan.diagnostics.is_empty() {
        return Err(anyhow!(format_diagnostics(&scan.diagnostics)));
    }
    if scan.entries.is_empty() {
        return Err(anyhow!(
            "no trace tags were found under the configured scan roots"
        ));
    }

    let mut output = String::new();
    output.push_str("# Theorem-to-Code Traceability Matrix\n\n");
    output.push_str("This document is machine-generated from structured `TRACE:TYPE:ID:SHORT_TITLE[:NOTE]` tags embedded in the crate implementation source.\n\n");
    output.push_str("It is a traceability aid for auditors, reviewers, and systems engineers. It is not a proof of correctness by itself.\n\n");
    output.push_str("Regenerate it from the crate root with:\n\n");
    output.push_str("```bash\n");
    output.push_str("cargo run --manifest-path Cargo.toml --bin dsfb-traceability\n");
    output.push_str("```\n\n");
    output.push_str("Check freshness without rewriting files:\n\n");
    output.push_str("```bash\n");
    output.push_str("cargo run --manifest-path Cargo.toml --bin dsfb-traceability -- --check\n");
    output.push_str("```\n\n");
    output.push_str("The generator scans `src/`, `tests/`, `examples/`, `ffi/`, and `python/src/` for implementation-linked trace tags.\n\n");
    output.push_str("| Paper Item Type | Paper Item ID | Short Title | File | Line | Notes / Implementation Role |\n");
    output.push_str("| --- | --- | --- | --- | ---: | --- |\n");
    for entry in &scan.entries {
        let note = entry.note.as_deref().unwrap_or("");
        output.push_str(&format!(
            "| {} | {} | {} | `{}` | {} | {} |\n",
            escape_markdown(entry.item_type.as_str()),
            escape_markdown(&entry.item_id),
            escape_markdown(&entry.short_title),
            escape_markdown(&entry.file),
            entry.line,
            escape_markdown(note),
        ));
    }
    Ok(output)
}

pub fn write_traceability_matrix(crate_root: &Path, output_path: &Path) -> Result<()> {
    let matrix = generate_traceability_matrix(crate_root)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create parent directories for `{}`",
                output_path.display()
            )
        })?;
    }
    fs::write(output_path, matrix)
        .with_context(|| format!("failed to write `{}`", output_path.display()))?;
    Ok(())
}

pub fn check_traceability_matrix_fresh(crate_root: &Path) -> Result<()> {
    let generated = generate_traceability_matrix(crate_root)?;
    let matrix_path = default_matrix_path(crate_root);
    let committed = fs::read_to_string(&matrix_path).with_context(|| {
        format!(
            "missing committed traceability matrix `{}`",
            matrix_path.display()
        )
    })?;
    check_traceability_matrix_contents(&generated, &committed)
}

pub fn check_traceability_matrix_contents(generated: &str, committed: &str) -> Result<()> {
    if normalize_newlines(committed) != normalize_newlines(generated) {
        return Err(anyhow!(
            "traceability matrix is stale; regenerate `{}`",
            TRACEABILITY_MATRIX_RELATIVE_PATH
        ));
    }
    Ok(())
}

#[must_use]
pub fn valid_trace_id(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-'
        })
        && value.contains('-')
}

fn collect_scan_files(crate_root: &Path, directory: &Path, files: &mut Vec<String>) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    if directory.is_file() {
        if should_scan_file(directory) {
            files.push(relative_path(crate_root, directory)?);
        }
        return Ok(());
    }

    let mut children = fs::read_dir(directory)
        .with_context(|| format!("failed to read `{}`", directory.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to enumerate `{}`", directory.display()))?;
    children.sort_by_key(|entry| entry.path());

    for child in children {
        let path = child.path();
        if path.is_dir() {
            collect_scan_files(crate_root, &path, files)?;
        } else if should_scan_file(&path) {
            files.push(relative_path(crate_root, &path)?);
        }
    }
    Ok(())
}

fn should_scan_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "c" | "cpp" | "h" | "hpp" | "py")
    )
}

fn extract_trace_candidate(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let comment_body = ["///", "//!", "//", "#", "/*", "*"]
        .iter()
        .find_map(|prefix| trimmed.strip_prefix(prefix).map(str::trim_start))?;
    if comment_body.starts_with(TRACE_TAG_PREFIX) {
        Some(comment_body)
    } else {
        None
    }
}

fn relative_path(crate_root: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(crate_root).with_context(|| {
        format!(
            "`{}` is not inside `{}`",
            path.display(),
            crate_root.display()
        )
    })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").trim_end().to_string()
}

fn escape_markdown(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn format_diagnostics(diagnostics: &[TraceDiagnostic]) -> String {
    let mut message = String::from("malformed trace tags detected:\n");
    for diagnostic in diagnostics {
        message.push_str(&format!(
            "- {}:{}: {} [{}]\n",
            diagnostic.file, diagnostic.line, diagnostic.message, diagnostic.source_line
        ));
    }
    message
}
