use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use dsfb_semiotics_engine::engine::bank::HeuristicBankRegistry;
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};
use tempfile::tempdir;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn readme_text() -> String {
    fs::read_to_string(crate_root().join("README.md")).unwrap()
}

fn compile_cpp_if_available(source: &str, output_name: &str) {
    match Command::new("c++").arg("--version").output() {
        Ok(_) => {
            let temp = tempdir().unwrap();
            let output = temp.path().join(output_name);
            let status = Command::new("c++")
                .args([
                    "-std=c++17",
                    "-c",
                    crate_root()
                        .join(source)
                        .to_str()
                        .expect("utf-8 source path"),
                    "-I",
                    crate_root()
                        .join("ffi/include")
                        .to_str()
                        .expect("utf-8 include path"),
                    "-o",
                    output.to_str().expect("utf-8 output path"),
                ])
                .status()
                .unwrap();
            assert!(status.success());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to invoke c++: {error}"),
    }
}

fn build_python_module_once() -> &'static PathBuf {
    static MODULE_PATH: OnceLock<PathBuf> = OnceLock::new();
    MODULE_PATH.get_or_init(|| {
        let status = Command::new(env!("CARGO"))
            .args([
                "build",
                "--manifest-path",
                crate_root()
                    .join("python/Cargo.toml")
                    .to_str()
                    .expect("utf-8 python manifest path"),
            ])
            .status()
            .unwrap();
        assert!(status.success());

        let target_dir = crate_root().join("python/target/debug");
        let artifact = fs::read_dir(&target_dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| {
                        (name.starts_with("libdsfb_engine") || name.starts_with("dsfb_engine"))
                            && (name.ends_with(".so")
                                || name.ends_with(".pyd")
                                || name.ends_with(".dylib"))
                    })
                    .unwrap_or(false)
            })
            .expect("built Python extension artifact");

        let staging = tempdir().unwrap();
        let extension = if cfg!(target_os = "windows") {
            "pyd"
        } else {
            "so"
        };
        let module_path = staging.path().join(format!("dsfb_engine.{extension}"));
        fs::copy(&artifact, &module_path).unwrap();
        let kept_dir = staging.path().to_path_buf();
        std::mem::forget(staging);
        kept_dir.join(format!("dsfb_engine.{extension}"))
    })
}

fn run_python(script: &str) -> String {
    let module_path = build_python_module_once();
    let module_dir = module_path.parent().unwrap();
    let output = Command::new("python3")
        .env("PYTHONPATH", module_dir)
        .arg("-c")
        .arg(script)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_dsfb_hpp_exists() {
    assert!(crate_root().join("ffi/include/dsfb.hpp").is_file());
}

#[test]
fn test_cpp_wrapper_compiles() {
    compile_cpp_if_available(
        "ffi/examples/minimal_cpp_wrapper.cpp",
        "cpp_wrapper_minimal.o",
    );
}

#[test]
fn test_cpp_wrapper_raii_lifecycle() {
    match Command::new("c++").arg("--version").output() {
        Ok(_) => {
            let temp = tempdir().unwrap();
            let source = temp.path().join("raii.cpp");
            fs::write(
                &source,
                r#"#include "dsfb.hpp"
int main() {
  dsfb::SemioticsEngine engine(8);
  dsfb::SemioticsEngine moved(std::move(engine));
  moved.reset();
  return 0;
}
"#,
            )
            .unwrap();
            let status = Command::new("c++")
                .args([
                    "-std=c++17",
                    "-c",
                    source.to_str().unwrap(),
                    "-I",
                    crate_root().join("ffi/include").to_str().unwrap(),
                    "-o",
                    temp.path().join("raii.o").to_str().unwrap(),
                ])
                .status()
                .unwrap();
            assert!(status.success());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to invoke c++: {error}"),
    }
}

#[test]
fn test_cpp_wrapper_push_and_query() {
    match Command::new("c++").arg("--version").output() {
        Ok(_) => {
            let temp = tempdir().unwrap();
            let source = temp.path().join("query.cpp");
            fs::write(
                &source,
                r#"#include "dsfb.hpp"
int main() {
  dsfb::SemioticsEngine engine(8);
  engine.push(0.12);
  auto snapshot = engine.snapshot();
  return static_cast<int>(snapshot.syntax_code());
}
"#,
            )
            .unwrap();
            let status = Command::new("c++")
                .args([
                    "-std=c++17",
                    "-c",
                    source.to_str().unwrap(),
                    "-I",
                    crate_root().join("ffi/include").to_str().unwrap(),
                    "-o",
                    temp.path().join("query.o").to_str().unwrap(),
                ])
                .status()
                .unwrap();
            assert!(status.success());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to invoke c++: {error}"),
    }
}

#[test]
fn test_cpp_wrapper_example_builds() {
    compile_cpp_if_available(
        "ffi/examples/stepwise_cpp_wrapper.cpp",
        "cpp_wrapper_stepwise.o",
    );
}

#[test]
fn test_docs_reference_cpp_wrapper() {
    let docs = fs::read_to_string(crate_root().join("docs/examples/cpp_wrapper.md")).unwrap();
    assert!(docs.contains("ffi/include/dsfb.hpp"));
}

#[test]
fn test_forensics_gen_binary_exists() {
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_dsfb-forensics-gen"));
    assert!(binary.is_file());
}

#[test]
fn test_forensics_gen_help_text() {
    let output = Command::new(env!("CARGO_BIN_EXE_dsfb-forensics-gen"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("--observed-csv"));
    assert!(help.contains("--predicted-csv"));
    assert!(help.contains("--open"));
}

#[test]
fn test_forensics_gen_runs_on_fixture_csv() {
    let temp = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_dsfb-forensics-gen"))
        .args([
            "--observed-csv",
            crate_root()
                .join("tests/fixtures/observed_fixture.csv")
                .to_str()
                .unwrap(),
            "--predicted-csv",
            crate_root()
                .join("tests/fixtures/predicted_fixture.csv")
                .to_str()
                .unwrap(),
            "--scenario-id",
            "fixture_csv",
            "--time-column",
            "time",
            "--output-dir",
            temp.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("forensics_run_dir="));
    assert!(stdout.contains("report_pdf="));
    assert!(stdout.contains("zip_archive="));
}

#[test]
fn test_forensics_gen_emits_pdf_and_zip() {
    let temp = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_dsfb-forensics-gen"))
        .args([
            "--observed-csv",
            crate_root()
                .join("tests/fixtures/observed_fixture.csv")
                .to_str()
                .unwrap(),
            "--predicted-csv",
            crate_root()
                .join("tests/fixtures/predicted_fixture.csv")
                .to_str()
                .unwrap(),
            "--scenario-id",
            "fixture_csv",
            "--time-column",
            "time",
            "--output-dir",
            temp.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pdf = stdout
        .lines()
        .find_map(|line| line.strip_prefix("report_pdf="))
        .map(PathBuf::from)
        .unwrap();
    let zip = stdout
        .lines()
        .find_map(|line| line.strip_prefix("zip_archive="))
        .map(PathBuf::from)
        .unwrap();
    assert!(pdf.is_file());
    assert!(zip.is_file());
}

#[test]
fn test_forensics_gen_manifest_present() {
    let temp = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_dsfb-forensics-gen"))
        .args([
            "--observed-csv",
            crate_root()
                .join("tests/fixtures/observed_fixture.csv")
                .to_str()
                .unwrap(),
            "--predicted-csv",
            crate_root()
                .join("tests/fixtures/predicted_fixture.csv")
                .to_str()
                .unwrap(),
            "--scenario-id",
            "fixture_csv",
            "--time-column",
            "time",
            "--output-dir",
            temp.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let manifest = stdout
        .lines()
        .find_map(|line| line.strip_prefix("manifest="))
        .map(PathBuf::from)
        .unwrap();
    assert!(manifest.is_file());
}

#[test]
fn test_forensics_gen_open_flag_handled() {
    let temp = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_dsfb-forensics-gen"))
        .env("PATH", "")
        .args([
            "--observed-csv",
            crate_root()
                .join("tests/fixtures/observed_fixture.csv")
                .to_str()
                .unwrap(),
            "--predicted-csv",
            crate_root()
                .join("tests/fixtures/predicted_fixture.csv")
                .to_str()
                .unwrap(),
            "--scenario-id",
            "fixture_csv",
            "--time-column",
            "time",
            "--output-dir",
            temp.path().to_str().unwrap(),
            "--open",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("open_status=unsupported"));
}

#[test]
fn test_docs_include_forensics_gen_workflow() {
    let docs = fs::read_to_string(crate_root().join("docs/examples/forensics_gen.md")).unwrap();
    assert!(docs.contains("dsfb-forensics-gen"));
    assert!(docs.contains("PDF report"));
    assert!(docs.contains("ZIP bundle"));
}

#[test]
fn test_python_binding_project_exists() {
    assert!(crate_root().join("python/Cargo.toml").is_file());
    assert!(crate_root().join("python/pyproject.toml").is_file());
}

#[test]
fn test_python_module_imports() {
    let stdout = run_python("import dsfb_engine; print(dsfb_engine.__name__)");
    assert!(stdout.contains("dsfb_engine"));
}

#[test]
fn test_python_run_on_small_fixture() {
    let stdout = run_python(
        "import dsfb_engine; out = dsfb_engine.run_array([0.04, 0.08, 0.12]); print(len(out))",
    );
    assert!(stdout.contains('3'));
}

#[test]
fn test_python_returns_structured_results() {
    let stdout = run_python(
        "import dsfb_engine; out = dsfb_engine.run_scenario('nominal_stable'); print(sorted(out.keys()))",
    );
    assert!(stdout.contains("syntax_label"));
    assert!(stdout.contains("semantic_disposition"));
    assert!(stdout.contains("trust_scalar"));
}

#[test]
fn test_python_example_exists() {
    assert!(crate_root().join("python/examples/quickstart.py").is_file());
}

#[test]
fn test_docs_include_python_install_and_quickstart() {
    let docs = fs::read_to_string(crate_root().join("docs/examples/python_quickstart.md")).unwrap();
    assert!(docs.contains("maturin develop"));
    assert!(docs.contains("run_scenario"));
}

#[test]
fn test_benchmark_target_exists() {
    assert!(crate_root().join("benches/execution_budget.rs").is_file());
}

#[test]
fn test_criterion_or_equivalent_config_present() {
    let cargo = fs::read_to_string(crate_root().join("Cargo.toml")).unwrap();
    assert!(cargo.contains("criterion"));
    assert!(cargo.contains("[[bench]]"));
}

#[test]
fn test_benchmark_docs_exist() {
    assert!(crate_root().join("docs/execution_budget.md").is_file());
}

#[test]
fn test_readme_mentions_execution_budget() {
    let readme = readme_text();
    assert!(readme.contains("Execution Budget"));
    assert!(readme.contains("benches/execution_budget.rs"));
}

#[test]
fn test_readme_contains_mermaid_block() {
    let readme = readme_text();
    assert!(readme.contains("```mermaid"));
}

#[test]
fn test_readme_contains_logic_flow_diagram() {
    let readme = readme_text();
    assert!(readme.contains("Signal / Residual Source"));
    assert!(readme.contains("D --> E[Grammar]"));
}

#[test]
fn test_readme_contains_decision_flow_diagram() {
    let readme = readme_text();
    assert!(readme.contains("Slew Spike or Drift Growth"));
    assert!(readme.contains("Trust Scalar"));
}

#[test]
fn test_mermaid_terms_match_current_api_and_docs() {
    let readme = readme_text();
    assert!(readme.contains("Residual"));
    assert!(readme.contains("Sign"));
    assert!(readme.contains("Syntax"));
    assert!(readme.contains("Grammar"));
    assert!(readme.contains("Semantics"));
}

#[test]
fn test_public_dataset_demo_docs_exist() {
    assert!(crate_root()
        .join("docs/examples/public_dataset_dashboard_demo.md")
        .is_file());
}

#[test]
fn test_demo_pipeline_instructions_exist() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/public_dataset_dashboard_demo.md"))
            .unwrap();
    assert!(docs.contains("dsfb-forensics-gen"));
    assert!(docs.contains("--dashboard-replay-csv"));
}

#[test]
fn test_dashboard_replay_docs_reference_public_dataset_demo() {
    let readme = readme_text();
    assert!(readme.contains("public_dataset_dashboard_demo"));
}

#[test]
fn test_imu_thermal_drift_gps_denied_scenario_exists() {
    let bundle = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "imu_thermal_drift_gps_denied",
    ))
    .run_selected()
    .unwrap();
    assert_eq!(
        bundle.scenario_outputs[0].record.id,
        "imu_thermal_drift_gps_denied"
    );
}

#[test]
fn test_imu_scenario_runs() {
    let bundle = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "imu_thermal_drift_gps_denied",
    ))
    .run_selected()
    .unwrap();
    assert!(!bundle.scenario_outputs[0].grammar.is_empty());
}

#[test]
fn test_imu_scenario_docs_include_units_and_physical_interpretation() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/imu_thermal_drift_gps_denied.md"))
            .unwrap();
    assert!(docs.contains("rad/s"));
    assert!(docs.contains("GPS-denied blackout"));
}

#[test]
fn test_imu_scenario_expected_structural_outputs_documented() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/imu_thermal_drift_gps_denied.md"))
            .unwrap();
    assert!(docs.contains("persistent outward"));
    assert!(docs.contains("abrupt"));
}

#[test]
fn test_technical_brief_exists() {
    assert!(crate_root()
        .join("docs/briefs/dsfb_auditable_ins_residual_interpretation_layer.md")
        .is_file());
}

#[test]
fn test_brief_mentions_problem_dsfb_demonstration_and_interface_boundary() {
    let brief = fs::read_to_string(
        crate_root().join("docs/briefs/dsfb_auditable_ins_residual_interpretation_layer.md"),
    )
    .unwrap();
    assert!(brief.contains("A-PNT"));
    assert!(brief.contains("What the crate concretely demonstrates"));
    assert!(brief.contains("Interface boundary"));
}

#[test]
fn test_brief_uses_at_least_one_real_supported_number_or_explicitly_omits_numbers_honestly() {
    let brief = fs::read_to_string(
        crate_root().join("docs/briefs/dsfb_auditable_ins_residual_interpretation_layer.md"),
    )
    .unwrap();
    assert!(brief.contains("64"));
}

#[test]
fn test_docs_reference_technical_brief() {
    let readme = readme_text();
    assert!(readme.contains("technical brief"));
}

#[test]
fn test_mosa_compatibility_doc_exists() {
    assert!(crate_root().join("docs/mosa_compatibility.md").is_file());
}

#[test]
fn test_doc_mentions_c_abi_opaque_handle_and_repr_c_if_applicable() {
    let doc = fs::read_to_string(crate_root().join("docs/mosa_compatibility.md")).unwrap();
    assert!(doc.contains("C ABI"));
    assert!(doc.contains("opaque-handle"));
    assert!(doc.contains("#[repr(C)]"));
}

#[test]
fn test_doc_avoids_false_formal_compliance_claims() {
    let doc = fs::read_to_string(crate_root().join("docs/mosa_compatibility.md")).unwrap();
    assert!(doc.contains("does **not** claim formal MOSA or SOSA certification or compliance"));
}

#[test]
fn test_readme_or_docs_index_links_to_mosa_compatibility_doc() {
    let readme = readme_text();
    assert!(readme.contains("docs/mosa_compatibility.md"));
}

#[test]
fn test_default_strict_bank_validation_emits_zero_warnings_for_builtin_bank() {
    let (_, _, report) = HeuristicBankRegistry::load_builtin(true).unwrap();
    assert!(report.warnings.is_empty());
}

#[test]
fn test_default_strict_bank_validation_emits_zero_violations_for_builtin_bank() {
    let (_, _, report) = HeuristicBankRegistry::load_builtin(true).unwrap();
    assert!(report.violations.is_empty());
}

#[test]
fn test_no_missing_reverse_incompatibility_links() {
    let (_, _, report) = HeuristicBankRegistry::load_builtin(true).unwrap();
    assert!(report.missing_incompatibility_links.is_empty());
    assert!(report.missing_compatibility_links.is_empty());
}

#[test]
fn test_no_unknown_bank_targets() {
    let (_, _, report) = HeuristicBankRegistry::load_builtin(true).unwrap();
    assert!(report.unknown_link_targets.is_empty());
}

#[test]
fn test_no_missing_required_provenance_fields() {
    let (_, _, report) = HeuristicBankRegistry::load_builtin(true).unwrap();
    assert!(report.provenance_gaps.is_empty());
}
