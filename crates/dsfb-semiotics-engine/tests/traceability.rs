use std::fs;
use std::path::PathBuf;
use std::process::Command;

use dsfb_semiotics_engine::traceability::{
    check_traceability_matrix_contents, check_traceability_matrix_fresh, collect_traceability,
    default_matrix_path, generate_traceability_matrix, parse_trace_tag, valid_trace_id,
    PaperItemType,
};
use tempfile::tempdir;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MatrixRow {
    item_type: PaperItemType,
    item_id: String,
    short_title: String,
    file: String,
    line: usize,
    note: String,
}

fn parse_matrix_rows(markdown: &str) -> Vec<MatrixRow> {
    markdown
        .lines()
        .filter(|line| line.starts_with('|'))
        .skip(2)
        .filter_map(|line| {
            let columns = line
                .split('|')
                .map(str::trim)
                .filter(|column| !column.is_empty())
                .collect::<Vec<_>>();
            if columns.len() != 6 {
                return None;
            }
            Some(MatrixRow {
                item_type: columns[0].parse().expect("valid paper item type"),
                item_id: columns[1].to_string(),
                short_title: columns[2].to_string(),
                file: columns[3].trim_matches('`').to_string(),
                line: columns[4].parse().expect("valid source line number"),
                note: columns[5].to_string(),
            })
        })
        .collect()
}

#[test]
fn test_trace_tags_exist_in_source() {
    let scan = collect_traceability(crate_root().as_path()).unwrap();
    assert!(scan.diagnostics.is_empty());
    assert!(!scan.entries.is_empty());
    assert!(scan
        .entries
        .iter()
        .any(|entry| entry.file.starts_with("src/")));
}

#[test]
fn test_trace_tag_format_is_machine_parsable() {
    let tag = parse_trace_tag(
        "// TRACE:DEFINITION:DEF-RESIDUAL:Residual construction:Implements observed minus predicted residual formation.",
    )
    .unwrap();
    assert_eq!(tag.item_type, PaperItemType::Definition);
    assert_eq!(tag.item_id, "DEF-RESIDUAL");
    assert_eq!(tag.short_title, "Residual construction");
    assert_eq!(
        tag.note.as_deref(),
        Some("Implements observed minus predicted residual formation.")
    );
}

#[test]
fn test_traceability_generator_runs() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("TRACEABILITY.md");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .current_dir(crate_root())
        .arg("run")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .arg("--bin")
        .arg("dsfb-traceability")
        .arg("--")
        .arg("--output")
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(output.is_file());
}

#[test]
fn test_traceability_matrix_generated() {
    let matrix_path = default_matrix_path(crate_root().as_path());
    assert!(matrix_path.is_file());
    let matrix = fs::read_to_string(matrix_path).unwrap();
    assert!(matrix.starts_with("# Theorem-to-Code Traceability Matrix"));
}

#[test]
fn test_traceability_matrix_contains_required_columns() {
    let matrix = fs::read_to_string(default_matrix_path(crate_root().as_path())).unwrap();
    assert!(matrix.contains("Paper Item Type"));
    assert!(matrix.contains("Paper Item ID"));
    assert!(matrix.contains("Short Title"));
    assert!(matrix.contains("File"));
    assert!(matrix.contains("Line"));
    assert!(matrix.contains("Notes / Implementation Role"));
}

#[test]
fn test_traceability_matrix_contains_real_source_locations() {
    let matrix = fs::read_to_string(default_matrix_path(crate_root().as_path())).unwrap();
    let rows = parse_matrix_rows(&matrix);
    assert!(!rows.is_empty());
    for row in rows {
        let file = crate_root().join(&row.file);
        assert!(
            file.is_file(),
            "missing traceability path {}",
            file.display()
        );
        let line_count = fs::read_to_string(&file).unwrap().lines().count();
        assert!(
            row.line >= 1 && row.line <= line_count,
            "traceability line {} out of range for {}",
            row.line,
            file.display()
        );
    }
}

#[test]
fn test_traceability_matrix_sorted_stably() {
    let matrix = fs::read_to_string(default_matrix_path(crate_root().as_path())).unwrap();
    let rows = parse_matrix_rows(&matrix);
    let mut sorted = rows.clone();
    sorted.sort_by(|left, right| {
        left.item_type
            .sort_key()
            .cmp(&right.item_type.sort_key())
            .then_with(|| left.item_id.cmp(&right.item_id))
            .then_with(|| left.file.cmp(&right.file))
            .then_with(|| left.line.cmp(&right.line))
    });
    assert_eq!(rows, sorted);
}

#[test]
fn test_malformed_trace_tags_are_detected() {
    let temp = tempdir().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        "// TRACE:DEFINITION::Missing identifier\npub fn demo() {}\n",
    )
    .unwrap();
    let error = generate_traceability_matrix(temp.path()).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("malformed trace tags detected"));
    assert!(message.contains("Missing identifier"));
}

#[test]
fn test_traceability_doc_freshness_check() {
    check_traceability_matrix_fresh(crate_root().as_path()).unwrap();
    let generated = generate_traceability_matrix(crate_root().as_path()).unwrap();
    let stale = format!("{generated}\n<!-- stale -->\n");
    assert!(check_traceability_matrix_contents(&generated, &stale).is_err());
}

#[test]
fn test_docs_reference_traceability_matrix() {
    let readme = fs::read_to_string(crate_root().join("README.md")).unwrap();
    let architecture = fs::read_to_string(crate_root().join("docs/architecture.md")).unwrap();
    assert!(readme.contains("docs/THEOREM_TO_CODE_TRACEABILITY.md"));
    assert!(readme.contains("docs/traceability.md"));
    assert!(architecture.contains("THEOREM_TO_CODE_TRACEABILITY.md"));
}

#[test]
fn test_traceability_ids_match_documented_conventions() {
    let docs = fs::read_to_string(crate_root().join("docs/traceability.md")).unwrap();
    assert!(docs.contains("uppercase hyphenated"));
    let scan = collect_traceability(crate_root().as_path()).unwrap();
    assert!(scan
        .entries
        .iter()
        .all(|entry| valid_trace_id(&entry.item_id)));
}

#[test]
fn test_traceability_quality_gate_wired_into_ci_and_qa() {
    let justfile = fs::read_to_string(crate_root().join("justfile")).unwrap();
    let workflow =
        fs::read_to_string(crate_root().join(".github/workflows/crate-quality-gate.yml")).unwrap();
    assert!(justfile.contains("traceability-check"));
    assert!(workflow.contains("Traceability freshness check"));
    assert!(workflow.contains("cargo run --bin dsfb-traceability -- --check"));
}
