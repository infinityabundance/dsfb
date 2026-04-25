//! Mechanical JSON Schema validation of `PaperLockReport` JSON output.
//!
//! For a representative subset of datasets, this test:
//! 1. Runs the production paper-lock binary (via `run_real_data_with_csv_path`
//!    for hermeticity).
//! 2. Serialises the resulting `PaperLockReport` to JSON.
//! 3. Validates the JSON against `paper/paper_lock_schema.json` using
//!    Python's `jsonschema` package via a subprocess call.
//!
//! Why a subprocess to Python rather than a Rust JSON Schema crate:
//! the dsfb-robotics crate intentionally keeps its dev-dependency
//! footprint small (proptest + criterion + serde_json + approx). A
//! native Rust schema validator would add 10+ transitive deps for
//! one test. Python's jsonschema is the canonical reference
//! implementation of the draft 2020-12 spec and is universally
//! available on developer machines and CI runners; the test gracefully
//! skips when Python or jsonschema are absent.
//!
//! The test catches drift between `paper/paper_lock_schema.json` and
//! the engine's emitted JSON: a future engine refactor that adds a
//! new field, renames a field, or changes a constraint will fail this
//! test before reaching main.
#![cfg(all(feature = "std", feature = "paper_lock"))]

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use dsfb_robotics::datasets::DatasetId;
use dsfb_robotics::paper_lock::{run_real_data_with_csv_path, serialize_report};

fn locate_crate_root() -> PathBuf {
    let candidates = [
        PathBuf::from("crates/dsfb-robotics"),
        PathBuf::from("."),
    ];
    for c in candidates.iter() {
        if c.join("paper").join("paper_lock_schema.json").is_file() {
            return c.clone();
        }
    }
    PathBuf::from(".")
}

fn schema_path() -> PathBuf {
    locate_crate_root().join("paper").join("paper_lock_schema.json")
}

fn python_jsonschema_available() -> bool {
    Command::new("python3")
        .args(["-c", "import jsonschema"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn locate_csv(slug: &str) -> Option<PathBuf> {
    let root = locate_crate_root();
    for filename in [format!("{slug}_published.csv"), format!("{slug}.csv")] {
        let p = root.join("data").join("processed").join(&filename);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn validate_json_against_schema(json: &str, schema: &PathBuf) -> Result<(), String> {
    // python3 -c '<jsonschema script>' < <json>
    let mut child = Command::new("python3")
        .args([
            "-c",
            "import sys, json, jsonschema\n\
             schema_path = sys.argv[1]\n\
             schema = json.load(open(schema_path))\n\
             doc = json.load(sys.stdin)\n\
             jsonschema.validate(doc, schema)\n\
             print('OK')",
            schema.to_str().expect("schema path must be utf8"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn python3: {e}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "missing stdin".to_string())?
        .write_all(json.as_bytes())
        .map_err(|e| format!("write json to python: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("python3 exit: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "schema validation failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    Ok(())
}

const VALIDATION_DATASETS: &[(DatasetId, &str)] = &[
    (DatasetId::Cwru, "cwru"),
    (DatasetId::PandaGaz, "panda_gaz"),
    (DatasetId::IcubPushRecovery, "icub_pushrecovery"),
    (DatasetId::Droid, "droid"),
    (DatasetId::Openx, "openx"),
];

#[test]
#[cfg_attr(miri, ignore = "Miri cannot model filesystem syscalls")]
fn paper_lock_json_validates_against_schema_default_mode() {
    if !schema_path().is_file() {
        eprintln!("skipping: schema not found at {:?}", schema_path());
        return;
    }
    if !python_jsonschema_available() {
        eprintln!("skipping: python3 jsonschema package not available");
        return;
    }
    for (id, slug) in VALIDATION_DATASETS.iter() {
        let Some(csv) = locate_csv(slug) else {
            eprintln!("skipping {slug}: CSV missing");
            continue;
        };
        let report = run_real_data_with_csv_path(*id, false, &csv).expect("paper-lock");
        let json = serialize_report(&report).expect("serialize");
        validate_json_against_schema(&json, &schema_path())
            .unwrap_or_else(|e| panic!("{slug}: {e}"));
    }
}

#[test]
#[cfg_attr(miri, ignore = "Miri cannot model filesystem syscalls")]
fn paper_lock_json_validates_against_schema_with_trace() {
    if !schema_path().is_file() {
        eprintln!("skipping: schema not found");
        return;
    }
    if !python_jsonschema_available() {
        eprintln!("skipping: python3 jsonschema package not available");
        return;
    }
    let Some(csv) = locate_csv("cwru") else {
        eprintln!("skipping: cwru CSV missing");
        return;
    };
    // include_trace=true exercises the Episode array branch of the schema.
    let report = run_real_data_with_csv_path(DatasetId::Cwru, true, &csv).expect("paper-lock");
    let json = serialize_report(&report).expect("serialize");
    validate_json_against_schema(&json, &schema_path())
        .unwrap_or_else(|e| panic!("with-trace schema validation: {e}"));
}
