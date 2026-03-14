pub mod add;
pub mod core;
pub mod dscd;
pub mod dsfb;
pub mod hret;
pub mod srd;
pub mod tmtr;

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use crate::cli::{BankSelection, RunSelection};
use crate::csv_writer::write_csv_rows;
use crate::output::OutputLayout;
use crate::registry::{Component, RealizationSpec, TheoremRegistry, TheoremSpec};

#[derive(Debug, Clone)]
pub struct TheoremExecutionResult {
    pub theorem_id: String,
    pub component: Component,
    pub csv_relative_path: String,
    pub case_count: usize,
    pub pass_count: usize,
    pub fail_count: usize,
}

#[derive(Debug, Clone)]
pub struct RealizationExecutionResult {
    pub component: Component,
    pub csv_relative_path: String,
    pub row_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct RunExecution {
    pub theorem_results: Vec<TheoremExecutionResult>,
    pub realization_results: Vec<RealizationExecutionResult>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RunnerContext<'a> {
    pub layout: &'a OutputLayout,
    pub seed: u64,
}

#[derive(Debug, Serialize)]
struct RealizationRow<'a> {
    component: &'a str,
    realization_name: &'a str,
    category: &'a str,
    operator_domain: &'a str,
    operator_codomain: &'a str,
    notes: &'a str,
    empirical_status: &'a str,
}

pub fn run_selection(
    registry: &TheoremRegistry,
    selection: &RunSelection,
    layout: &OutputLayout,
    seed: u64,
) -> Result<RunExecution> {
    let ctx = RunnerContext { layout, seed };
    let mut execution = RunExecution::default();

    let components = theorem_components(selection);
    execution.logs.push(format!("selection={selection:?}"));
    execution.logs.push(format!("seed={seed}"));

    for component in components {
        for spec in registry.theorems_for(component) {
            let result = run_theorem(spec, &ctx)?;
            execution.logs.push(format!(
                "wrote {} for {} ({} rows)",
                result.csv_relative_path, result.theorem_id, result.case_count
            ));
            execution.theorem_results.push(result);
        }
    }

    let realization_components = realization_components(selection);
    if !realization_components.is_empty() {
        let results = write_realization_exports(registry, layout, &realization_components)?;
        for result in results {
            execution.logs.push(format!(
                "wrote {} ({} rows)",
                result.csv_relative_path, result.row_count
            ));
            execution.realization_results.push(result);
        }
    }

    Ok(execution)
}

pub fn theorem_output_path(layout: &OutputLayout, spec: &TheoremSpec) -> (PathBuf, String) {
    let file_name = if spec.component == Component::Core {
        format!(
            "{}.csv",
            slug_title(&spec.title).trim_end_matches("_theorem")
        )
    } else {
        format!("{:02}_{}.csv", spec.ordinal, slug_title(&spec.title))
    };
    let dir = layout.component_dir(spec.component);
    let path = dir.join(&file_name);
    let relative = path
        .strip_prefix(&layout.root)
        .expect("path lives under run root")
        .to_string_lossy()
        .replace('\\', "/");
    (path, relative)
}

fn run_theorem(spec: &TheoremSpec, ctx: &RunnerContext<'_>) -> Result<TheoremExecutionResult> {
    match spec.component {
        Component::Core => core::run(spec, ctx),
        Component::Dsfb => dsfb::run(spec, ctx),
        Component::Dscd => dscd::run(spec, ctx),
        Component::Tmtr => tmtr::run(spec, ctx),
        Component::Add => add::run(spec, ctx),
        Component::Srd => srd::run(spec, ctx),
        Component::Hret => hret::run(spec, ctx),
    }
}

pub fn write_component_rows<T>(
    spec: &TheoremSpec,
    ctx: &RunnerContext<'_>,
    rows: &[T],
    pass_count: usize,
    fail_count: usize,
) -> Result<TheoremExecutionResult>
where
    T: Serialize,
{
    let (path, relative) = theorem_output_path(ctx.layout, spec);
    write_csv_rows(&path, rows)?;
    Ok(TheoremExecutionResult {
        theorem_id: spec.id.clone(),
        component: spec.component,
        csv_relative_path: relative,
        case_count: rows.len(),
        pass_count,
        fail_count,
    })
}

fn theorem_components(selection: &RunSelection) -> Vec<Component> {
    match selection {
        RunSelection::All => Component::ALL.to_vec(),
        RunSelection::Core => vec![Component::Core],
        RunSelection::Bank(bank) => vec![component_from_bank(*bank)],
    }
}

fn realization_components(selection: &RunSelection) -> Vec<Component> {
    match selection {
        RunSelection::All => Component::BANKS.to_vec(),
        RunSelection::Core => Vec::new(),
        RunSelection::Bank(bank) => vec![component_from_bank(*bank)],
    }
}

fn component_from_bank(bank: BankSelection) -> Component {
    match bank {
        BankSelection::Dsfb => Component::Dsfb,
        BankSelection::Dscd => Component::Dscd,
        BankSelection::Tmtr => Component::Tmtr,
        BankSelection::Add => Component::Add,
        BankSelection::Srd => Component::Srd,
        BankSelection::Hret => Component::Hret,
    }
}

fn write_realization_exports(
    registry: &TheoremRegistry,
    layout: &OutputLayout,
    components: &[Component],
) -> Result<Vec<RealizationExecutionResult>> {
    let mut results = Vec::new();
    let mut combined = Vec::new();
    for component in components {
        let rows = registry.realizations_for(*component);
        if rows.is_empty() {
            continue;
        }
        combined.extend(rows.clone());
        let file_name = format!("{}_realizations.csv", component.as_str());
        let path = layout.realizations_dir.join(&file_name);
        write_realization_csv(&path, &rows)?;
        results.push(RealizationExecutionResult {
            component: *component,
            csv_relative_path: path
                .strip_prefix(&layout.root)
                .expect("realization path lives under run root")
                .to_string_lossy()
                .replace('\\', "/"),
            row_count: rows.len(),
        });
    }
    if !combined.is_empty() {
        let path = layout.realizations_dir.join("all_realizations.csv");
        write_realization_csv(&path, &combined)?;
        results.push(RealizationExecutionResult {
            component: Component::Core,
            csv_relative_path: path
                .strip_prefix(&layout.root)
                .expect("realization path lives under run root")
                .to_string_lossy()
                .replace('\\', "/"),
            row_count: combined.len(),
        });
    }
    Ok(results)
}

fn write_realization_csv(path: &Path, realizations: &[RealizationSpec]) -> Result<()> {
    let rows = realizations
        .iter()
        .map(|spec| RealizationRow {
            component: spec.component.as_str(),
            realization_name: &spec.realization_name,
            category: &spec.category,
            operator_domain: &spec.operator_domain,
            operator_codomain: &spec.operator_codomain,
            notes: &spec.notes,
            empirical_status: &spec.empirical_status,
        })
        .collect::<Vec<_>>();
    write_csv_rows(path, &rows)
}

pub fn slug_title(title: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            slug.push('_');
            previous_was_separator = true;
        }
    }
    slug.trim_matches('_').to_string()
}
