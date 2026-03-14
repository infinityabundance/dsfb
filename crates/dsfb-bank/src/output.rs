use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::cli::{Cli, RunSelection};
use crate::registry::{workspace_root, Component, TheoremRegistry};
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

pub fn write_manifest(
    path: &Path,
    run_dir: &RunDirectory,
    cli: &Cli,
    selection: &RunSelection,
    registry: &TheoremRegistry,
    execution: &RunExecution,
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
