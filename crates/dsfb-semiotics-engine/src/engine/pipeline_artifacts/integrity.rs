//! Figure-integrity helpers for deterministic artifact validation.
//!
//! This module is intentionally limited to source-table/render consistency checks and the
//! machine-readable rows exported for those checks.

use std::path::Path;

use serde::Serialize;

use crate::engine::types::{FigureArtifact, RunMetadata};
use crate::evaluation::types::FigureIntegrityCheck;
use crate::figures::source::FigureSourceTable;
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct FigureIntegrityCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    figure_id: String,
    expected_panel_count: usize,
    observed_panel_count: usize,
    expected_panels: String,
    observed_panels: String,
    panel_labels: String,
    series_lengths: String,
    source_row_count: usize,
    source_table_present: bool,
    nonempty_series: bool,
    nonzero_values_present: bool,
    count_like_panels_integerlike: bool,
    consistent_with_source: bool,
    integrity_passed: bool,
    failures: String,
    source_csv: String,
    source_json: String,
    png_path: String,
    svg_path: String,
    png_present: bool,
    svg_present: bool,
    note: String,
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-integrity helpers keep the export contract explicit without hiding artifact fields behind ad hoc tuples."
)]
pub(crate) fn build_figure_integrity_check(
    run_metadata: &RunMetadata,
    table: &FigureSourceTable,
    artifact: Option<&FigureArtifact>,
    source_csv: &str,
    source_json: &str,
    count_like_integer_tolerance: f64,
    source_csv_present: bool,
    source_json_present: bool,
) -> FigureIntegrityCheck {
    let observed_panels = ordered_panel_ids(table);
    let panel_labels = observed_panels
        .iter()
        .map(|panel_id| panel_title(table, panel_id))
        .collect::<Vec<_>>();
    let series_lengths = observed_panels
        .iter()
        .map(|panel_id| {
            table
                .rows
                .iter()
                .filter(|row| row.panel_id == *panel_id)
                .count()
        })
        .collect::<Vec<_>>();
    let source_row_count = table.rows.len();
    let nonempty_series = !table.rows.is_empty() && series_lengths.iter().all(|length| *length > 0);
    let nonzero_values_present = table.rows.iter().any(|row| {
        row.y_value.abs() > 1.0e-12
            || row
                .secondary_y_value
                .map(|value| value.abs() > 1.0e-12)
                .unwrap_or(false)
    });
    let count_like_panels_integerlike = table.count_like_panel_ids.iter().all(|panel_id| {
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == *panel_id)
            .all(|row| (row.y_value - row.y_value.round()).abs() <= count_like_integer_tolerance)
    });
    let png_path = artifact
        .map(|artifact| artifact.png_path.clone())
        .unwrap_or_default();
    let svg_path = artifact
        .map(|artifact| artifact.svg_path.clone())
        .unwrap_or_default();
    let png_present = !png_path.is_empty() && Path::new(&png_path).is_file();
    let svg_present = !svg_path.is_empty() && Path::new(&svg_path).is_file();
    let expected_panels = if table.expected_panel_ids.is_empty() {
        (0..table.expected_panel_count)
            .map(|index| format!("panel_{}", index + 1))
            .collect::<Vec<_>>()
    } else {
        table.expected_panel_ids.clone()
    };
    let source_table_present = source_csv_present && source_json_present;
    let failures = figure_integrity_failures(
        table,
        &expected_panels,
        &observed_panels,
        source_table_present,
        nonempty_series,
        count_like_panels_integerlike,
        png_present,
        svg_present,
    );
    let integrity_passed = failures.is_empty();

    FigureIntegrityCheck {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: run_metadata.crate_version.clone(),
        bank_version: run_metadata.bank.bank_version.clone(),
        figure_id: table.figure_id.clone(),
        expected_panel_count: table.expected_panel_count,
        observed_panel_count: observed_panels.len(),
        expected_panels,
        observed_panels,
        panel_labels,
        series_lengths,
        source_row_count,
        source_table_present,
        nonempty_series,
        nonzero_values_present,
        count_like_panels_integerlike,
        consistent_with_source: integrity_passed,
        integrity_passed,
        failures,
        source_csv: source_csv.to_string(),
        source_json: source_json.to_string(),
        png_path,
        svg_path,
        png_present,
        svg_present,
        note: "Figure rendered from the exported figure-source table; integrity check covers panel identity, source rows, count-like panels, and emitted PNG/SVG presence."
            .to_string(),
    }
}

#[must_use]
pub(crate) fn figure_integrity_csv_row(check: &FigureIntegrityCheck) -> FigureIntegrityCsvRow {
    FigureIntegrityCsvRow {
        schema_version: check.schema_version.clone(),
        engine_version: check.engine_version.clone(),
        bank_version: check.bank_version.clone(),
        figure_id: check.figure_id.clone(),
        expected_panel_count: check.expected_panel_count,
        observed_panel_count: check.observed_panel_count,
        expected_panels: check.expected_panels.join(" | "),
        observed_panels: check.observed_panels.join(" | "),
        panel_labels: check.panel_labels.join(" | "),
        series_lengths: check
            .series_lengths
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(" | "),
        source_row_count: check.source_row_count,
        source_table_present: check.source_table_present,
        nonempty_series: check.nonempty_series,
        nonzero_values_present: check.nonzero_values_present,
        count_like_panels_integerlike: check.count_like_panels_integerlike,
        consistent_with_source: check.consistent_with_source,
        integrity_passed: check.integrity_passed,
        failures: check.failures.join(" | "),
        source_csv: check.source_csv.clone(),
        source_json: check.source_json.clone(),
        png_path: check.png_path.clone(),
        svg_path: check.svg_path.clone(),
        png_present: check.png_present,
        svg_present: check.svg_present,
        note: check.note.clone(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-integrity failures are assembled from explicit typed checks so each exported failure surface stays auditable."
)]
pub(crate) fn figure_integrity_failures(
    table: &FigureSourceTable,
    expected_panels: &[String],
    observed_panels: &[String],
    source_table_present: bool,
    nonempty_series: bool,
    count_like_panels_integerlike: bool,
    png_present: bool,
    svg_present: bool,
) -> Vec<String> {
    let mut failures = Vec::new();
    if !source_table_present {
        failures.push("source table missing".to_string());
    }
    if observed_panels.len() != table.expected_panel_count {
        failures.push(format!(
            "expected {} panels but observed {}",
            table.expected_panel_count,
            observed_panels.len()
        ));
    }
    if expected_panels != observed_panels {
        failures.push(format!(
            "expected panels [{}] but observed [{}]",
            expected_panels.join(", "),
            observed_panels.join(", ")
        ));
    }
    if !nonempty_series {
        failures.push("one or more observed panels has an empty series".to_string());
    }
    if !count_like_panels_integerlike {
        failures.push("count-like panel used non-integer-like values".to_string());
    }
    if !png_present {
        failures.push("png render missing".to_string());
    }
    if !svg_present {
        failures.push("svg render missing".to_string());
    }
    failures
}

fn ordered_panel_ids(table: &FigureSourceTable) -> Vec<String> {
    let mut panel_ids = Vec::new();
    for row in &table.rows {
        if !panel_ids.iter().any(|panel_id| panel_id == &row.panel_id) {
            panel_ids.push(row.panel_id.clone());
        }
    }
    panel_ids
}

fn panel_title(table: &FigureSourceTable, panel_id: &str) -> String {
    table
        .rows
        .iter()
        .find(|row| row.panel_id == panel_id)
        .map(|row| row.panel_title.clone())
        .unwrap_or_else(|| panel_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_figure_integrity_check, figure_integrity_failures};
    use crate::engine::bank::{BankSourceKind, LoadedBankDescriptor};
    use crate::engine::settings::EngineSettings;
    use crate::engine::types::{FigureArtifact, RunMetadata};
    use crate::figures::source::{FigureSourceRow, FigureSourceTable};
    use crate::io::schema::ARTIFACT_SCHEMA_VERSION;

    fn run_metadata() -> RunMetadata {
        RunMetadata {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            bank_version: "heuristic-bank/test".to_string(),
            run_configuration_hash: "config-hash".to_string(),
            crate_name: "dsfb-semiotics-engine".to_string(),
            crate_version: env!("CARGO_PKG_VERSION").to_string(),
            rust_version: None,
            git_commit: None,
            timestamp: "2026-03-20T10:00:00Z".to_string(),
            input_mode: "synthetic".to_string(),
            seed: 123,
            steps: 16,
            dt: 1.0,
            engine_settings: EngineSettings::default(),
            bank: LoadedBankDescriptor {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                bank_schema_version: "dsfb-semiotics-engine-bank/v1".to_string(),
                bank_version: "heuristic-bank/test".to_string(),
                source_kind: BankSourceKind::Builtin,
                source_path: None,
                content_hash: "bank-hash".to_string(),
                strict_validation: true,
                validation_mode: "strict".to_string(),
                note: "test".to_string(),
            },
            online_history_buffer_capacity: 64,
            numeric_mode: "f64".to_string(),
            cli_args: vec!["--scenario".to_string(), "nominal_stable".to_string()],
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        }
    }

    fn row(panel_id: &str, series_id: &str, y_value: f64) -> FigureSourceRow {
        FigureSourceRow {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            bank_version: "heuristic-bank/test".to_string(),
            figure_id: "figure_test".to_string(),
            plot_title: "Figure test".to_string(),
            panel_id: panel_id.to_string(),
            panel_title: panel_id.to_string(),
            x_label: "x".to_string(),
            y_label: "y".to_string(),
            series_id: series_id.to_string(),
            series_label: series_id.to_string(),
            series_kind: "bar".to_string(),
            color_key: "default".to_string(),
            point_order: 0,
            x_value: 0.0,
            y_value,
            secondary_x_value: None,
            secondary_y_value: None,
            x_tick_label: "scenario".to_string(),
            annotation_text: String::new(),
            scenario_id: "scenario".to_string(),
            note: "test".to_string(),
        }
    }

    #[test]
    fn missing_panel_fails_integrity_check() {
        let failures = figure_integrity_failures(
            &FigureSourceTable {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                bank_version: "heuristic-bank/test".to_string(),
                figure_id: "figure_test".to_string(),
                plot_title: "Figure test".to_string(),
                generation_timestamp: "2026-03-20T10:00:00Z".to_string(),
                expected_panel_count: 2,
                expected_panel_ids: vec!["panel_a".to_string(), "panel_b".to_string()],
                count_like_panel_ids: vec!["panel_a".to_string()],
                panel_ids: vec!["panel_a".to_string()],
                series_ids: vec!["series".to_string()],
                rows: vec![row("panel_a", "series", 1.0)],
            },
            &["panel_a".to_string(), "panel_b".to_string()],
            &["panel_a".to_string()],
            true,
            true,
            true,
            true,
            true,
        );
        assert!(!failures.is_empty());
    }

    #[test]
    fn wrong_series_mapping_fails_integrity_check() {
        let check = build_figure_integrity_check(
            &run_metadata(),
            &FigureSourceTable {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                bank_version: "heuristic-bank/test".to_string(),
                figure_id: "figure_test".to_string(),
                plot_title: "Figure test".to_string(),
                generation_timestamp: "2026-03-20T10:00:00Z".to_string(),
                expected_panel_count: 1,
                expected_panel_ids: vec!["panel_a".to_string()],
                count_like_panel_ids: vec!["panel_a".to_string()],
                panel_ids: vec!["panel_b".to_string()],
                series_ids: vec!["series".to_string()],
                rows: vec![row("panel_b", "series", 1.0)],
            },
            Some(&FigureArtifact {
                figure_id: "figure_test".to_string(),
                caption: "test".to_string(),
                png_path: "/tmp/does-not-exist.png".to_string(),
                svg_path: "/tmp/does-not-exist.svg".to_string(),
            }),
            "/tmp/source.csv",
            "/tmp/source.json",
            1.0e-9,
            true,
            true,
        );
        assert!(!check.integrity_passed);
    }
}
