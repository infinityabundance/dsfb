//! `dsfb-atlas`: generator for the DSFB-ATLAS 10,000-theorem atlas.
//!
//! Loads YAML Part specs from `--spec-dir`, validates structure, generates
//! per-Part LaTeX includes (10,000 theorems total), emits the augmenting
//! `dsfb.bib`, the longtable theorem index, and a coverage report; runs
//! SHA-256 deduplication on every proof body and fails the build on any
//! collision or theorem-count mismatch.
//!
//! Function-level decomposition follows NASA/JPL Power-of-Ten 4: every
//! function is small enough to review on one page (≤ 60 LOC) and each
//! has a documented invariant.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unused_qualifications,
    clippy::all,
    clippy::pedantic
)]
#![allow(clippy::needless_pass_by_value, clippy::module_name_repetitions)]

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod bib_emit;
mod dedup;
mod generator;
mod index_emit;
mod schema;

use schema::Part;

/// Hard caps used both for invariant assertions and for bounding the
/// `walkdir` traversals in [`load_parts`] / [`validate_bank_ids`]. Any
/// well-formed input is far below these caps; exceeding them indicates a
/// configuration mistake and the build should fail closed.
const MAX_PART_FILES: usize = 64;
const MAX_BANK_FILES: usize = 256;
/// The atlas is contractually exactly 10,000 theorems. The build fails if
/// the count drifts.
const EXPECTED_THEOREM_COUNT: usize = 10_000;

/// CLI surface for the `dsfb-atlas` binary.
#[derive(Parser, Debug)]
#[command(name = "dsfb-atlas", version, about = "DSFB-ATLAS 10,000-theorem atlas generator")]
struct Cli {
    /// Directory containing `PNN_*.yaml` part specs and `_schema.json`.
    #[arg(long)]
    spec_dir: PathBuf,
    /// Output directory for generated `.tex` and `.bib` files.
    #[arg(long)]
    out: PathBuf,
    /// Optional bank-spec directory for cross-validating cited bank IDs.
    #[arg(long)]
    bank_spec_dir: Option<PathBuf>,
    /// Inject git hash into generated artefacts (passed in by build script).
    #[arg(long, default_value = "dev")]
    git_hash: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    fs::create_dir_all(&cli.out).context("create output dir")?;

    let parts = load_parts(&cli.spec_dir)?;
    println!("loaded {} parts", parts.len());

    if let Some(bsd) = &cli.bank_spec_dir {
        validate_bank_ids(&parts, bsd)?;
    }

    let mut dedup = dedup::Dedup::new();
    let total_theorems = emit_all_parts(&parts, &cli.out, &mut dedup)?;
    emit_aux_artifacts(&parts, &cli.out)?;

    let report = dedup.finalize();
    write_dedup_report(&cli.out, &report, &cli.git_hash)?;
    print_summary_and_check(total_theorems, &report)
}

// ---------------------------------------------------------------------------
// Per-Part LaTeX emission (delegates to generator::generate_part)
// ---------------------------------------------------------------------------

fn emit_all_parts(parts: &[Part], out_dir: &Path, dedup: &mut dedup::Dedup) -> Result<usize> {
    let mut total_theorems = 0usize;
    for part in parts {
        let part_num: usize = part
            .part_id
            .get(1..3)
            .and_then(|s| s.parse().ok())
            .with_context(|| format!("malformed part_id `{}`", part.part_id))?;
        let (latex, count) = generator::generate_part(part, dedup)?;
        let path = out_dir.join(format!("part_{part_num:02}.tex"));
        fs::write(&path, latex).with_context(|| format!("write {path:?}"))?;
        println!("part {} -> {path:?} ({count} theorems)", part.part_id);
        total_theorems = total_theorems.saturating_add(count);
    }
    Ok(total_theorems)
}

fn emit_aux_artifacts(parts: &[Part], out_dir: &Path) -> Result<()> {
    let bib = bib_emit::emit_bib(parts)?;
    let bib_path = out_dir.join("dsfb.bib");
    fs::write(&bib_path, bib).context("write generated/dsfb.bib")?;
    println!("emitted {bib_path:?}");

    let idx = index_emit::emit_index(parts)?;
    let idx_path = out_dir.join("index_longtable.tex");
    fs::write(&idx_path, idx).context("write index_longtable.tex")?;
    println!("emitted {idx_path:?}");

    let cov = generate_coverage_report(parts);
    fs::write(out_dir.join("coverage_report.tex"), cov).context("write coverage_report.tex")?;
    Ok(())
}

fn write_dedup_report(out_dir: &Path, report: &dedup::DedupReport, git_hash: &str) -> Result<()> {
    let dedup_json = serde_json::json!({
        "total": report.total,
        "unique": report.unique,
        "collisions": report.collisions,
        "git_hash": git_hash,
    });
    fs::write(
        out_dir.join("dedup_report.json"),
        serde_json::to_string_pretty(&dedup_json)?,
    )
    .context("write dedup_report.json")?;
    Ok(())
}

fn print_summary_and_check(total_theorems: usize, report: &dedup::DedupReport) -> Result<()> {
    println!(
        "TOTAL: {} theorems emitted, {} unique proof hashes, {} collisions",
        total_theorems,
        report.unique,
        report.collisions.len()
    );
    if !report.collisions.is_empty() {
        bail!(
            "SHA-256 dedup collisions detected ({}). See dedup_report.json. Build fails.",
            report.collisions.len()
        );
    }
    if total_theorems != EXPECTED_THEOREM_COUNT {
        bail!(
            "Expected exactly {EXPECTED_THEOREM_COUNT} theorems but emitted {total_theorems}. Check YAML stems/modifiers (10x10 per chapter)."
        );
    }
    println!("OK: 10,000 atlas theorems generated with structurally unique proofs.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Spec loaders / validators
// ---------------------------------------------------------------------------

fn load_parts(spec_dir: &Path) -> Result<Vec<Part>> {
    let mut parts = Vec::with_capacity(MAX_PART_FILES);
    for entry in WalkDir::new(spec_dir).max_depth(1).into_iter().take(MAX_PART_FILES) {
        let entry = entry?;
        if !is_part_yaml(&entry) {
            continue;
        }
        let path = entry.path();
        let raw = fs::read_to_string(path).with_context(|| format!("read {path:?}"))?;
        let part: Part =
            serde_yaml::from_str(&raw).with_context(|| format!("parse {path:?}"))?;
        validate_part_shape(&part)?;
        parts.push(part);
    }
    parts.sort_by(|a, b| a.part_id.cmp(&b.part_id));
    if parts.len() != 10 {
        bail!("expected 10 parts, found {}", parts.len());
    }
    Ok(parts)
}

fn is_part_yaml(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_file() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    name.starts_with('P') && name.ends_with(".yaml")
}

fn validate_part_shape(part: &Part) -> Result<()> {
    if part.chapters.len() != 10 {
        bail!(
            "{} has {} chapters; expected 10",
            part.part_id,
            part.chapters.len()
        );
    }
    for (i, c) in part.chapters.iter().enumerate() {
        if c.stems.len() != 10 || c.modifiers.len() != 10 {
            bail!(
                "{} chapter {i}: stems={}, modifiers={}; expected 10x10",
                part.part_id,
                c.stems.len(),
                c.modifiers.len()
            );
        }
    }
    Ok(())
}

fn validate_bank_ids(parts: &[Part], bank_dir: &Path) -> Result<()> {
    let known_ids = collect_bank_ids(bank_dir)?;
    let missing = find_unknown_anchor_ids(parts, &known_ids);
    if missing.is_empty() {
        println!("bank-id validation: all {} ids resolve.", count_anchor_ids(parts));
    } else {
        for m in &missing {
            eprintln!("WARN: {m}");
        }
        eprintln!("(soft warning; not failing build)");
    }
    Ok(())
}

fn collect_bank_ids(bank_dir: &Path) -> Result<std::collections::HashSet<String>> {
    let mut known_ids = std::collections::HashSet::with_capacity(256);
    for entry in WalkDir::new(bank_dir).max_depth(1).into_iter().take(MAX_BANK_FILES) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.ends_with(".yaml") {
            continue;
        }
        let raw = fs::read_to_string(path).unwrap_or_default();
        for line in raw.lines() {
            let l = line.trim();
            if let Some(rest) = l.strip_prefix("- id:") {
                let id = rest
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                known_ids.insert(id);
            }
        }
    }
    Ok(known_ids)
}

fn find_unknown_anchor_ids(
    parts: &[Part],
    known_ids: &std::collections::HashSet<String>,
) -> Vec<String> {
    let mut missing = Vec::new();
    for p in parts {
        for c in &p.chapters {
            for id in &c.anchor_bank_ids {
                if !known_ids.contains(id) {
                    missing.push(format!(
                        "{} chapter {} cites unknown bank id '{id}'",
                        p.part_id, c.chapter_id
                    ));
                }
            }
        }
    }
    missing
}

fn count_anchor_ids(parts: &[Part]) -> usize {
    parts
        .iter()
        .flat_map(|p| p.chapters.iter())
        .map(|c| c.anchor_bank_ids.len())
        .sum()
}

// ---------------------------------------------------------------------------
// Coverage report
// ---------------------------------------------------------------------------

fn generate_coverage_report(parts: &[Part]) -> String {
    let mut tier_counts = std::collections::BTreeMap::<String, usize>::new();
    let mut total = 0usize;
    for p in parts {
        for c in &p.chapters {
            let t = c
                .anchor_tier
                .clone()
                .unwrap_or_else(|| p.default_anchor_tier.clone());
            let n = c.stems.len() * c.modifiers.len();
            *tier_counts.entry(t).or_insert(0) += n;
            total += n;
        }
    }
    let mut s = String::with_capacity(512);
    s.push_str("% generated/coverage_report.tex\n\n");
    s.push_str("\\begin{tabular}{lr}\n\\toprule\nTier & Theorem count \\\\\n\\midrule\n");
    for (t, c) in &tier_counts {
        s.push_str(&format!("{t} & {c} \\\\\n"));
    }
    s.push_str(&format!(
        "\\midrule\n\\textbf{{Total}} & \\textbf{{{total}}} \\\\\n"
    ));
    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s
}
