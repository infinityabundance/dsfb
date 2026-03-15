use std::path::Path;

use anyhow::Result;

use crate::cli::RunSelection;
use crate::output::write_run_notes;
use crate::output::TheoremCsvSummary;
use crate::registry::Component;
use crate::runners::RunExecution;
use crate::timestamp::RunDirectory;

pub fn write_run_summary(
    path: &Path,
    run_dir: &RunDirectory,
    selection: &RunSelection,
    execution: &RunExecution,
    theorem_csv_summary: &TheoremCsvSummary,
) -> Result<()> {
    let body = render_run_summary(run_dir, selection, execution, theorem_csv_summary);
    write_run_notes(path, &body)
}

fn render_run_summary(
    run_dir: &RunDirectory,
    selection: &RunSelection,
    execution: &RunExecution,
    theorem_csv_summary: &TheoremCsvSummary,
) -> String {
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
    for row in &theorem_csv_summary.component_rows {
        lines.push(format!(
            "- `{}`: `{}` theorems, `{}` rows",
            row.component, row.theorem_count, row.cases
        ));
    }
    lines.push(String::new());
    lines.push(String::from("## Case-class interpretation"));
    lines.push(String::new());
    lines.push(String::from(
        "Boundary and violating rows in this artifact are deliberate assumption-sensitive witnesses. They mark admissible boundary behavior or non-admissible / assumption-violating cases where the theorem is not expected to apply; they are not presented as theorem falsifications.",
    ));
    lines.push(String::new());
    lines.push(String::from(
        "Core violating rows are intentional theorem-non-applicable witnesses for the cross-layer statements, while DSFB violating rows highlight non-injective or non-image observations where exact recovery is not admissible.",
    ));
    lines.push(String::new());
    lines.push(String::from("## Global case-class counts"));
    lines.push(String::new());
    lines.push(format!(
        "- `passing`: `{}`",
        theorem_csv_summary.case_class_counts_global.passing
    ));
    lines.push(format!(
        "- `boundary`: `{}`",
        theorem_csv_summary.case_class_counts_global.boundary
    ));
    lines.push(format!(
        "- `violating`: `{}`",
        theorem_csv_summary.case_class_counts_global.violating
    ));
    lines.push(String::new());
    lines.push(String::from("## By-component case-class counts"));
    lines.push(String::new());
    for component in Component::ALL {
        let counts = theorem_csv_summary
            .case_class_counts_by_component
            .get(component.as_str())
            .cloned()
            .unwrap_or_default();
        lines.push(format!(
            "- `{}`: `passing={}`, `boundary={}`, `violating={}`",
            component.as_str(),
            counts.passing,
            counts.boundary,
            counts.violating
        ));
    }
    lines.push(String::new());
    lines.push(String::from("## Pass/fail and assumption summary"));
    lines.push(String::new());
    for row in &theorem_csv_summary.component_rows {
        lines.push(format!(
            "- `{}`: `pass={}`, `fail={}`, `assumption_satisfied={}`, `assumption_violated={}`",
            row.component,
            row.pass,
            row.fail,
            row.assumption_satisfied_count,
            row.assumption_violated_count
        ));
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
