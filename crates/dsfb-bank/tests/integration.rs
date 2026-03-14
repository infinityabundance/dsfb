use std::fs;

use dsfb_bank::cli::{BankSelection, RunSelection};
use dsfb_bank::csv_writer::write_csv_rows;
use dsfb_bank::output::prepare_output_layout;
use dsfb_bank::registry::{Component, TheoremRegistry};
use dsfb_bank::runners::run_selection;
use dsfb_bank::timestamp::{create_timestamped_run_dir, RunDirectory};
use serde::Serialize;
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

fn manual_run_dir(root: &std::path::Path, name: &str) -> RunDirectory {
    let run_dir = root.join(name);
    fs::create_dir_all(&run_dir).expect("create run dir");
    RunDirectory {
        output_root: root.to_path_buf(),
        timestamp: name.to_string(),
        run_dir,
    }
}
