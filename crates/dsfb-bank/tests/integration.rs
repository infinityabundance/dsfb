use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use csv::StringRecord;
use dsfb_bank::cli::{BankSelection, RunSelection};
use dsfb_bank::csv_writer::write_csv_rows;
use dsfb_bank::execute;
use dsfb_bank::output::prepare_output_layout;
use dsfb_bank::registry::{Component, TheoremRegistry};
use dsfb_bank::runners::run_selection;
use dsfb_bank::timestamp::{create_timestamped_run_dir, RunDirectory};
use serde::Serialize;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn loads_all_theorem_specs() {
    let registry = TheoremRegistry::load().expect("registry loads");
    assert_eq!(registry.theorems_for(Component::Core).len(), 11);
    assert_eq!(registry.theorems_for(Component::Dsfb).len(), 20);
    assert_eq!(registry.theorems_for(Component::Dscd).len(), 20);
    assert_eq!(registry.theorems_for(Component::Tmtr).len(), 20);
    assert_eq!(registry.theorems_for(Component::Add).len(), 20);
    assert_eq!(registry.theorems_for(Component::Srd).len(), 20);
    assert_eq!(registry.theorems_for(Component::Hret).len(), 20);
    assert_eq!(registry.all_theorems().len(), 131);
}

#[test]
fn registry_exposes_realizations() {
    let registry = TheoremRegistry::load().expect("registry loads");
    assert_eq!(registry.realizations_for(Component::Dsfb).len(), 3);
    assert_eq!(registry.realizations_for(Component::Dscd).len(), 3);
    assert_eq!(registry.realizations_for(Component::Tmtr).len(), 3);
    assert_eq!(registry.realizations_for(Component::Add).len(), 3);
    assert_eq!(registry.realizations_for(Component::Srd).len(), 3);
    assert_eq!(registry.realizations_for(Component::Hret).len(), 3);
    assert_eq!(registry.all_realizations().len(), 18);
}

#[test]
fn creates_timestamped_output_path() {
    let temp = tempdir().expect("tempdir");
    let run_dir = create_timestamped_run_dir(temp.path()).expect("timestamped dir");
    assert!(run_dir.run_dir.exists());
    assert_eq!(run_dir.run_dir.parent(), Some(temp.path()));
    assert_eq!(run_dir.timestamp.len(), "2026-03-14_12-34-56".len());
}

#[test]
fn selected_runner_is_deterministic_for_same_seed() {
    let registry = TheoremRegistry::load().expect("registry loads");
    let temp = tempdir().expect("tempdir");

    let first = manual_run_dir(temp.path(), "run_one");
    let second = manual_run_dir(temp.path(), "run_two");

    let first_layout = prepare_output_layout(&first).expect("first layout");
    let second_layout = prepare_output_layout(&second).expect("second layout");

    let first_run = run_selection(
        &registry,
        &RunSelection::Bank(BankSelection::Dsfb),
        &first_layout,
        7,
    )
    .expect("first run");
    let second_run = run_selection(
        &registry,
        &RunSelection::Bank(BankSelection::Dsfb),
        &second_layout,
        7,
    )
    .expect("second run");

    assert_eq!(first_run.theorem_results.len(), 20);
    assert_eq!(second_run.theorem_results.len(), 20);

    let first_csv = temp
        .path()
        .join("run_one")
        .join("dsfb")
        .join("02_injective_forward_map_implies_unique_structural_observability.csv");
    let second_csv = temp
        .path()
        .join("run_two")
        .join("dsfb")
        .join("02_injective_forward_map_implies_unique_structural_observability.csv");

    let first_body = fs::read_to_string(first_csv).expect("first csv");
    let second_body = fs::read_to_string(second_csv).expect("second csv");
    assert_eq!(first_body, second_body);
}

#[test]
fn csv_writer_emits_headers_and_rows() {
    #[derive(Serialize)]
    struct Row<'a> {
        theorem_id: &'a str,
        pass: bool,
    }

    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("sample.csv");
    write_csv_rows(
        &path,
        &[Row {
            theorem_id: "TEST-01",
            pass: true,
        }],
    )
    .expect("csv write");

    let body = fs::read_to_string(path).expect("csv body");
    assert!(body.contains("theorem_id"));
    assert!(body.contains("TEST-01"));
}

#[test]
fn full_run_emits_hardened_schema_and_explicit_violations() {
    let temp = tempdir().expect("tempdir");
    let cli = dsfb_bank::cli::Cli {
        all: true,
        core: false,
        bank: None,
        list: false,
        output: Some(temp.path().join("artifact")),
        seed: Some(0),
    };

    let run_dir = execute(&cli)
        .expect("full run succeeds")
        .expect("run dir returned");

    assert!(
        run_dir.join("manifest.json").exists(),
        "manifest.json missing"
    );
    assert!(
        run_dir.join("component_summary.csv").exists(),
        "component_summary.csv missing"
    );
    assert!(
        run_dir.join("run_summary.md").exists(),
        "run_summary.md missing"
    );
    assert!(run_dir.join("logs.txt").exists(), "logs.txt missing");

    let common_columns = [
        "theorem_id",
        "theorem_name",
        "component",
        "case_id",
        "case_class",
        "assumption_satisfied",
        "expected_outcome",
        "observed_outcome",
        "pass",
        "notes",
    ];

    let mut violating_components = BTreeSet::new();
    let mut tmtr_violating_rows = 0usize;
    for (component, fields) in [
        (
            "core",
            vec![
                "time_step",
                "signal_value",
                "observation_value",
                "reconstructed_state",
                "residual_value",
                "trust_value",
                "regime_label",
                "anomaly_flag",
            ],
        ),
        (
            "dsfb",
            vec![
                "injective_flag",
                "observation_id",
                "structural_state_id",
                "reconstructed_state_id",
                "residual_value",
                "exact_recovery_flag",
                "time_step",
                "signal_value",
                "observation_value",
                "reconstructed_state",
            ],
        ),
        (
            "dscd",
            vec![
                "graph_id",
                "node_count",
                "edge_count",
                "longest_path",
                "reachability_count",
                "acyclic_flag",
                "attempted_edge_addition_flag",
                "cycle_created_flag",
                "reduction_edge_count",
                "repaired_edge_count",
            ],
        ),
        (
            "tmtr",
            vec![
                "orbit_id",
                "iteration",
                "trust_value",
                "residual_value",
                "fixed_point_flag",
                "stabilization_iteration",
                "trust_gap",
                "trust_increase_attempt_flag",
                "trust_gap_satisfied_flag",
                "monotonicity_satisfied_flag",
            ],
        ),
        (
            "add",
            vec![
                "signal_id",
                "time_step",
                "signal_value",
                "residual_value",
                "first_difference",
                "second_difference",
                "threshold",
                "detector_output",
                "anomaly_magnitude",
            ],
        ),
        (
            "srd",
            vec![
                "trajectory_id",
                "time_step",
                "state_id",
                "fine_regime",
                "coarse_regime",
                "transition_flag",
                "coarse_transition_flag",
                "regime_valid_flag",
            ],
        ),
        (
            "hret",
            vec![
                "trace_id",
                "event_index",
                "trace_length",
                "prefix_length",
                "suffix_length",
                "observation_code",
                "reconstruction_success",
                "replayability_flag",
                "injective_observation_flag",
            ],
        ),
    ] {
        for csv_path in sorted_csvs(&run_dir.join(component)) {
            let mut reader = csv::Reader::from_path(&csv_path).expect("open csv");
            let headers = reader.headers().expect("csv headers").clone();
            assert_has_columns(&headers, &common_columns, &csv_path);
            assert_has_columns(&headers, &fields, &csv_path);

            for row in reader.records() {
                let row = row.expect("csv row");
                if component != "core"
                    && row_value(&headers, &row, "case_class") == "violating"
                    && row_value(&headers, &row, "assumption_satisfied") == "false"
                    && row_value(&headers, &row, "pass") == "false"
                {
                    violating_components.insert(component.to_string());
                    if component == "tmtr" {
                        tmtr_violating_rows += 1;
                    }
                }
            }
        }
    }

    assert_eq!(
        violating_components,
        BTreeSet::from([
            String::from("add"),
            String::from("dsfb"),
            String::from("dscd"),
            String::from("hret"),
            String::from("srd"),
            String::from("tmtr"),
        ])
    );
    assert!(
        tmtr_violating_rows > 1,
        "expected multiple TMTR violating rows, found {tmtr_violating_rows}"
    );

    let component_summary_path = run_dir.join("component_summary.csv");
    let mut component_summary_reader =
        csv::Reader::from_path(&component_summary_path).expect("open component_summary.csv");
    let component_summary_headers = component_summary_reader
        .headers()
        .expect("component_summary headers")
        .clone();
    assert_has_columns(
        &component_summary_headers,
        &[
            "component",
            "theorem_count",
            "cases",
            "pass",
            "fail",
            "boundary",
            "violating",
            "passing",
            "assumption_satisfied_count",
            "assumption_violated_count",
        ],
        &component_summary_path,
    );
    let component_summary_rows = component_summary_reader
        .records()
        .map(|row| row.expect("component_summary row"))
        .collect::<Vec<_>>();
    assert_eq!(
        component_summary_rows.len(),
        7,
        "expected one component_summary row per theorem component"
    );
    let component_summary_components = component_summary_rows
        .iter()
        .map(|row| row_value(&component_summary_headers, row, "component").to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        component_summary_components,
        BTreeSet::from([
            String::from("core"),
            String::from("dsfb"),
            String::from("dscd"),
            String::from("tmtr"),
            String::from("add"),
            String::from("srd"),
            String::from("hret"),
        ])
    );

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("manifest.json")).expect("manifest body"),
    )
    .expect("manifest json");
    let case_class_counts = manifest
        .get("case_class_counts")
        .and_then(Value::as_object)
        .expect("manifest.case_class_counts object");
    let global_counts = case_class_counts
        .get("global")
        .and_then(Value::as_object)
        .expect("manifest.case_class_counts.global object");
    for key in ["passing", "boundary", "violating"] {
        assert!(
            global_counts.contains_key(key),
            "manifest.case_class_counts.global missing {key}"
        );
    }
    let by_component = case_class_counts
        .get("by_component")
        .and_then(Value::as_object)
        .expect("manifest.case_class_counts.by_component object");
    for component in ["core", "dsfb", "dscd", "tmtr", "add", "srd", "hret"] {
        let component_counts = by_component
            .get(component)
            .and_then(Value::as_object)
            .unwrap_or_else(|| {
                panic!("manifest.case_class_counts.by_component missing {component}")
            });
        for key in ["passing", "boundary", "violating"] {
            assert!(
                component_counts.contains_key(key),
                "manifest.case_class_counts.by_component.{component} missing {key}"
            );
        }
    }
}

fn manual_run_dir(root: &std::path::Path, name: &str) -> RunDirectory {
    let run_dir = root.join(name);
    fs::create_dir_all(&run_dir).expect("create run dir");
    RunDirectory {
        output_root: root.to_path_buf(),
        timestamp: name.to_string(),
        run_dir,
    }
}

fn sorted_csvs(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = fs::read_dir(dir)
        .expect("read component dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("csv"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn assert_has_columns(headers: &StringRecord, required: &[&str], path: &Path) {
    let header_set = headers.iter().collect::<BTreeSet<_>>();
    for column in required {
        assert!(
            header_set.contains(column),
            "{} missing required column {}",
            path.display(),
            column
        );
    }
}

fn row_value<'a>(headers: &'a StringRecord, row: &'a StringRecord, name: &str) -> &'a str {
    let index = headers
        .iter()
        .position(|header| header == name)
        .unwrap_or_else(|| panic!("missing header {name}"));
    row.get(index).unwrap_or("")
}
