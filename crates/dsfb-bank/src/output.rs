use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use csv::StringRecord;
use serde::Serialize;

use crate::cli::{Cli, RunSelection};
use crate::csv_writer::write_csv_rows;
use crate::registry::{workspace_root, Component, TheoremRegistry};
use crate::runners::common::CaseClass;
use crate::runners::RunExecution;
use crate::timestamp::RunDirectory;

#[derive(Debug, Clone)]
pub struct OutputLayout {
    pub root: PathBuf,
    component_dirs: BTreeMap<Component, PathBuf>,
    pub realizations_dir: PathBuf,
}

impl OutputLayout {
    pub fn component_dir(&self, component: Component) -> &Path {
        self.component_dirs
            .get(&component)
            .map(PathBuf::as_path)
            .expect("component directory exists")
    }
}

#[derive(Debug, Serialize)]
struct ManifestComponentCounts {
    theorem_count: usize,
    case_count: usize,
    csv_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct CaseClassCounts {
    pub passing: usize,
    pub boundary: usize,
    pub violating: usize,
}

impl CaseClassCounts {
    fn record(&mut self, case_class: CaseClass) {
        match case_class {
            CaseClass::Passing => self.passing += 1,
            CaseClass::Boundary => self.boundary += 1,
            CaseClass::Violating => self.violating += 1,
        }
    }

    fn extend(&mut self, other: &Self) {
        self.passing += other.passing;
        self.boundary += other.boundary;
        self.violating += other.violating;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentSummaryRow {
    pub component: String,
    pub theorem_count: usize,
    pub cases: usize,
    pub pass: usize,
    pub fail: usize,
    pub boundary: usize,
    pub violating: usize,
    pub passing: usize,
    pub assumption_satisfied_count: usize,
    pub assumption_violated_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ManifestCaseClassCounts {
    global: CaseClassCounts,
    by_component: BTreeMap<String, CaseClassCounts>,
}

#[derive(Debug, Clone)]
pub struct TheoremCsvSummary {
    pub component_rows: Vec<ComponentSummaryRow>,
    pub case_class_counts_global: CaseClassCounts,
    pub case_class_counts_by_component: BTreeMap<String, CaseClassCounts>,
}

#[derive(Debug, Serialize)]
struct Manifest {
    timestamp: String,
    crate_version: String,
    git_commit_hash: Option<String>,
    command_invoked: Vec<String>,
    theorem_specs_loaded: Vec<String>,
    theorem_demos_run: Vec<String>,
    output_file_inventory: Vec<String>,
    counts_by_component: BTreeMap<String, ManifestComponentCounts>,
    case_class_counts: ManifestCaseClassCounts,
    selection: String,
}

pub fn default_output_root() -> PathBuf {
    workspace_root().join("output-dsfb-bank")
}

pub fn prepare_output_layout(run_dir: &RunDirectory) -> Result<OutputLayout> {
    let mut component_dirs = BTreeMap::new();
    for component in Component::ALL {
        let dir = run_dir.run_dir.join(component.as_str());
        fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
        component_dirs.insert(component, dir);
    }
    let realizations_dir = run_dir.run_dir.join("realizations");
    fs::create_dir_all(&realizations_dir)
        .with_context(|| format!("failed to create {}", realizations_dir.display()))?;

    Ok(OutputLayout {
        root: run_dir.run_dir.clone(),
        component_dirs,
        realizations_dir,
    })
}

pub fn write_run_notes(path: &Path, body: &str) -> Result<()> {
    fs::write(path, body).with_context(|| format!("failed to write {}", path.display()))
}

pub fn write_logs(path: &Path, lines: &[String]) -> Result<()> {
    let mut file =
        File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    for line in lines {
        writeln!(file, "{line}")?;
    }
    Ok(())
}

pub fn summarize_theorem_csv_outputs(layout: &OutputLayout) -> Result<TheoremCsvSummary> {
    let mut component_rows = Vec::new();
    let mut case_class_counts_global = CaseClassCounts::default();
    let mut case_class_counts_by_component = BTreeMap::new();

    for component in Component::ALL {
        let mut theorem_ids = BTreeSet::new();
        let mut cases = 0usize;
        let mut pass = 0usize;
        let mut fail = 0usize;
        let mut boundary = 0usize;
        let mut violating = 0usize;
        let mut passing = 0usize;
        let mut assumption_satisfied_count = 0usize;
        let mut assumption_violated_count = 0usize;
        let mut component_case_counts = CaseClassCounts::default();

        let mut csv_paths = fs::read_dir(layout.component_dir(component))
            .with_context(|| {
                format!(
                    "failed to read theorem output directory {}",
                    layout.component_dir(component).display()
                )
            })?
            .map(|entry| entry.map(|item| item.path()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| {
                format!(
                    "failed to enumerate theorem output directory {}",
                    layout.component_dir(component).display()
                )
            })?;
        csv_paths.retain(|path| path.extension().and_then(|value| value.to_str()) == Some("csv"));
        csv_paths.sort();

        for csv_path in csv_paths {
            let mut reader = csv::Reader::from_path(&csv_path)
                .with_context(|| format!("failed to open {}", csv_path.display()))?;
            let headers = reader
                .headers()
                .with_context(|| format!("failed to read headers from {}", csv_path.display()))?
                .clone();
            let theorem_id_index = header_index(&headers, "theorem_id", &csv_path)?;
            let pass_index = header_index(&headers, "pass", &csv_path)?;
            let case_class_index = header_index(&headers, "case_class", &csv_path)?;
            let assumption_index = header_index(&headers, "assumption_satisfied", &csv_path)?;

            for record in reader.records() {
                let record = record
                    .with_context(|| format!("failed to read row from {}", csv_path.display()))?;
                theorem_ids
                    .insert(cell(&record, theorem_id_index, &csv_path, "theorem_id")?.to_string());
                cases += 1;

                if parse_bool(
                    cell(&record, pass_index, &csv_path, "pass")?,
                    &csv_path,
                    "pass",
                )? {
                    pass += 1;
                } else {
                    fail += 1;
                }

                match parse_case_class(
                    cell(&record, case_class_index, &csv_path, "case_class")?,
                    &csv_path,
                )? {
                    CaseClass::Passing => passing += 1,
                    CaseClass::Boundary => boundary += 1,
                    CaseClass::Violating => violating += 1,
                }
                component_case_counts.record(parse_case_class(
                    cell(&record, case_class_index, &csv_path, "case_class")?,
                    &csv_path,
                )?);

                if parse_bool(
                    cell(&record, assumption_index, &csv_path, "assumption_satisfied")?,
                    &csv_path,
                    "assumption_satisfied",
                )? {
                    assumption_satisfied_count += 1;
                } else {
                    assumption_violated_count += 1;
                }
            }
        }

        case_class_counts_global.extend(&component_case_counts);
        case_class_counts_by_component.insert(
            component.as_str().to_string(),
            component_case_counts.clone(),
        );
        component_rows.push(ComponentSummaryRow {
            component: component.as_str().to_string(),
            theorem_count: theorem_ids.len(),
            cases,
            pass,
            fail,
            boundary,
            violating,
            passing,
            assumption_satisfied_count,
            assumption_violated_count,
        });
    }

    Ok(TheoremCsvSummary {
        component_rows,
        case_class_counts_global,
        case_class_counts_by_component,
    })
}

pub fn write_component_summary(path: &Path, summary: &TheoremCsvSummary) -> Result<()> {
    write_csv_rows(path, &summary.component_rows)
}

pub fn write_manifest(
    path: &Path,
    run_dir: &RunDirectory,
    cli: &Cli,
    selection: &RunSelection,
    registry: &TheoremRegistry,
    execution: &RunExecution,
    theorem_csv_summary: &TheoremCsvSummary,
    output_file_inventory: Vec<String>,
) -> Result<()> {
    let mut counts_by_component = BTreeMap::new();
    for component in Component::ALL {
        let theorem_results = execution
            .theorem_results
            .iter()
            .filter(|result| result.component == component)
            .collect::<Vec<_>>();
        let csv_count = theorem_results.len();
        let case_count = theorem_results.iter().map(|result| result.case_count).sum();
        counts_by_component.insert(
            component.as_str().to_string(),
            ManifestComponentCounts {
                theorem_count: theorem_results.len(),
                case_count,
                csv_count,
            },
        );
    }
    if !execution.realization_results.is_empty() {
        let case_count = execution
            .realization_results
            .iter()
            .map(|result| result.row_count)
            .sum();
        counts_by_component.insert(
            String::from("realizations"),
            ManifestComponentCounts {
                theorem_count: execution.realization_results.len(),
                case_count,
                csv_count: execution.realization_results.len(),
            },
        );
    }

    let manifest = Manifest {
        timestamp: run_dir.timestamp.clone(),
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit_hash: git_commit_hash(),
        command_invoked: std::env::args().collect(),
        theorem_specs_loaded: registry
            .all_theorems()
            .iter()
            .map(|spec| spec.id.clone())
            .collect(),
        theorem_demos_run: execution
            .theorem_results
            .iter()
            .map(|result| result.theorem_id.clone())
            .collect(),
        output_file_inventory,
        counts_by_component,
        case_class_counts: ManifestCaseClassCounts {
            global: theorem_csv_summary.case_class_counts_global.clone(),
            by_component: theorem_csv_summary.case_class_counts_by_component.clone(),
        },
        selection: format!("{selection:?}"),
    };

    let file =
        File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(file, &manifest)
        .with_context(|| format!("failed to write {}", path.display()))?;

    if cli.output.is_some() {
        let _ = cli;
    }

    Ok(())
}

pub fn collect_inventory(root: &Path) -> Result<Vec<String>> {
    fn visit(root: &Path, dir: &Path, items: &mut Vec<String>) -> Result<()> {
        for entry in
            fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, items)?;
            } else if path.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .expect("file lives under root")
                    .to_string_lossy()
                    .replace('\\', "/");
                items.push(relative);
            }
        }
        Ok(())
    }

    let mut items = Vec::new();
    visit(root, root, &mut items)?;
    items.sort();
    Ok(items)
}

fn git_commit_hash() -> Option<String> {
    Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .current_dir(workspace_root())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn header_index(headers: &StringRecord, name: &str, path: &Path) -> Result<usize> {
    headers
        .iter()
        .position(|header| header == name)
        .with_context(|| format!("{} is missing required column {}", path.display(), name))
}

fn cell<'a>(
    record: &'a StringRecord,
    index: usize,
    path: &Path,
    column_name: &str,
) -> Result<&'a str> {
    record.get(index).with_context(|| {
        format!(
            "{} row is missing value for column {}",
            path.display(),
            column_name
        )
    })
}

fn parse_bool(value: &str, path: &Path, column_name: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!(
            "{} has invalid boolean {}={} ",
            path.display(),
            column_name,
            value
        ),
    }
}

fn parse_case_class(value: &str, path: &Path) -> Result<CaseClass> {
    match value.trim().to_ascii_lowercase().as_str() {
        "passing" => Ok(CaseClass::Passing),
        "boundary" => Ok(CaseClass::Boundary),
        "violating" => Ok(CaseClass::Violating),
        _ => anyhow::bail!("{} has invalid case_class={}", path.display(), value),
    }
}
