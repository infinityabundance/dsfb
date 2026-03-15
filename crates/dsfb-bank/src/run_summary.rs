use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use crate::cli::RunSelection;
use crate::output::write_run_notes;
use crate::registry::Component;
use crate::runners::RunExecution;
use crate::timestamp::RunDirectory;

pub fn write_run_summary(
    path: &Path,
    run_dir: &RunDirectory,
    selection: &RunSelection,
    execution: &RunExecution,
) -> Result<()> {
    let body = render_run_summary(run_dir, selection, execution);
    write_run_notes(path, &body)
}

fn render_run_summary(
    run_dir: &RunDirectory,
    selection: &RunSelection,
    execution: &RunExecution,
) -> String {
    let theorem_counts = component_counts(execution);
    let mut lines = Vec::new();
    lines.push(format!("# dsfb-bank run summary"));
    lines.push(String::new());
    lines.push(format!("- Timestamp: `{}`", run_dir.timestamp));
    lines.push(format!("- Selection: `{:?}`", selection));
    lines.push(format!(
        "- Theorem demos executed: `{}`",
        execution.theorem_results.len()
    ));
    lines.push(format!(
        "- Realization CSVs written: `{}`",
        execution.realization_results.len()
    ));
    lines.push(String::new());
    lines.push(String::from("## Theorem count by component"));
    lines.push(String::new());
    for component in Component::ALL {
        let count = theorem_counts
            .get(component.as_str())
            .copied()
            .unwrap_or_default();
        lines.push(format!("- `{}`: `{}`", component.as_str(), count));
    }
    lines.push(String::new());
    lines.push(String::from("## Pass/fail summary"));
    lines.push(String::new());
    lines.push(String::from(
        "Failing rows in this artifact are deliberate boundary or assumption-violating witnesses, not claimed theorem counterexamples.",
    ));
    lines.push(String::new());
    for component in Component::ALL {
        let results = execution
            .theorem_results
            .iter()
            .filter(|result| result.component == component)
            .collect::<Vec<_>>();
        let passes: usize = results.iter().map(|result| result.pass_count).sum();
        let fails: usize = results.iter().map(|result| result.fail_count).sum();
        if !results.is_empty() {
            lines.push(format!(
                "- `{}`: `{}` passing rows, `{}` failing rows",
                component.as_str(),
                passes,
                fails
            ));
        }
    }
    lines.push(String::new());
    lines.push(String::from("## Generated CSVs"));
    lines.push(String::new());
    for result in &execution.theorem_results {
        lines.push(format!(
            "- `{}` (`{}` rows)",
            result.csv_relative_path, result.case_count
        ));
    }
    for result in &execution.realization_results {
        lines.push(format!(
            "- `{}` (`{}` rows)",
            result.csv_relative_path, result.row_count
        ));
    }
    lines.push(String::new());
    lines.push(String::from("## Realization-space outputs"));
    lines.push(String::new());
    if execution.realization_results.is_empty() {
        lines.push(String::from("- none for this selection"));
    } else {
        for result in &execution.realization_results {
            lines.push(format!("- `{}`", result.csv_relative_path));
        }
    }
    lines.join("\n")
}

fn component_counts(execution: &RunExecution) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for result in &execution.theorem_results {
        *counts
            .entry(result.component.as_str().to_string())
            .or_default() += 1;
    }
    counts
}
