use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use dsfb_computer_graphics::config::DemoConfig;
use dsfb_computer_graphics::dsfb::run_gated_taa;
use dsfb_computer_graphics::pipeline::run_all;
use dsfb_computer_graphics::report::{COMPATIBILITY_SENTENCE, COST_SENTENCE, EXPERIMENT_SENTENCE};
use dsfb_computer_graphics::scene::{generate_sequence_for_definition, scenario_suite};
use serde_json::Value;

fn unique_output_dir(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("generated")
        .join("test_runs")
        .join(format!("{name}_{stamp}"));
    fs::create_dir_all(&dir).expect("test output directory should be creatable");
    dir
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("text file should be readable")
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_str(&read(path)).expect("json should parse")
}

fn full_suite_dir() -> &'static PathBuf {
    static RUN_DIR: OnceLock<PathBuf> = OnceLock::new();
    RUN_DIR.get_or_init(|| {
        let output_dir = unique_output_dir("full_suite");
        run_all(&DemoConfig::default(), &output_dir).expect("run-all should succeed");
        output_dir
    })
}

fn demo_a_metrics() -> Value {
    read_json(full_suite_dir().join("metrics.json"))
}

fn demo_b_metrics() -> Value {
    read_json(full_suite_dir().join("demo_b").join("metrics.json"))
}

#[test]
fn scene_generation_is_deterministic_across_the_suite() {
    let config = DemoConfig::default();
    for definition in scenario_suite(&config.scene) {
        let sequence_a = generate_sequence_for_definition(&definition);
        let sequence_b = generate_sequence_for_definition(&definition);

        assert_eq!(sequence_a.target_mask, sequence_b.target_mask);
        assert_eq!(sequence_a.frames.len(), sequence_b.frames.len());
        for (frame_a, frame_b) in sequence_a.frames.iter().zip(sequence_b.frames.iter()) {
            assert_eq!(frame_a.layers, frame_b.layers);
            assert_eq!(frame_a.motion, frame_b.motion);
            assert_eq!(frame_a.disocclusion_mask, frame_b.disocclusion_mask);
            assert_eq!(frame_a.depth, frame_b.depth);
            assert_eq!(frame_a.normals, frame_b.normals);
            assert_eq!(
                frame_a
                    .ground_truth
                    .encode_png()
                    .expect("frame_a should encode"),
                frame_b
                    .ground_truth
                    .encode_png()
                    .expect("frame_b should encode")
            );
        }
    }
}

#[test]
fn host_realistic_trust_values_remain_in_unit_interval() {
    let config = DemoConfig::default();
    let definition = scenario_suite(&config.scene)
        .into_iter()
        .find(|definition| definition.id.as_str() == "thin_reveal")
        .expect("canonical scenario should exist");
    let sequence = generate_sequence_for_definition(&definition);
    let dsfb = run_gated_taa(
        &sequence,
        config.dsfb_alpha_range.min,
        config.dsfb_alpha_range.max,
    );

    for supervision in &dsfb.supervision_frames {
        for trust in supervision.trust.values() {
            assert!(
                (0.0..=1.0).contains(trust),
                "trust value {trust} should stay in [0, 1]"
            );
        }
    }
}

#[test]
fn run_all_generates_required_artifacts() {
    let output_dir = full_suite_dir();

    for relative in [
        "artifact_manifest.json",
        "metrics.json",
        "report.md",
        "reviewer_summary.md",
        "ablation_report.md",
        "cost_report.md",
        "completion_note.md",
        "five_mentor_audit.md",
        "check_signing_blockers.md",
        "trust_mode_report.md",
        "external_replay_report.md",
        "realism_bridge_report.md",
        "product_positioning_report.md",
        "operating_band_report.md",
        "demo_b_competitive_baselines_report.md",
        "check_signing_readiness.md",
        "demo_b_decision_report.md",
        "figures/fig_system_diagram.svg",
        "figures/fig_trust_map.svg",
        "figures/fig_before_after.svg",
        "figures/fig_trust_vs_error.svg",
        "figures/fig_intervention_alpha.svg",
        "figures/fig_ablation.svg",
        "figures/fig_roi_nonroi_error.svg",
        "figures/fig_leaderboard.svg",
        "figures/fig_scenario_mosaic.svg",
        "demo_b/metrics.json",
        "demo_b/report.md",
        "demo_b/figures/fig_demo_b_sampling.svg",
        "demo_b/figures/fig_demo_b_budget_efficiency.svg",
        "docs_placeholder",
    ] {
        if relative == "docs_placeholder" {
            continue;
        }
        let path = output_dir.join(relative);
        let metadata = fs::metadata(&path).expect("artifact metadata should exist");
        assert!(
            metadata.len() > 0,
            "artifact {} should be non-empty",
            path.display()
        );
    }
}

#[test]
fn demo_a_metrics_include_required_suite_baselines_ablations_and_behavioral_gates() {
    let metrics = demo_a_metrics();
    let summary = &metrics["summary"];
    let scenario_ids = summary["scenario_ids"]
        .as_array()
        .expect("scenario ids should be an array");
    let baseline_ids = summary["baseline_ids"]
        .as_array()
        .expect("baseline ids should be an array");
    let ablation_ids = summary["ablation_ids"]
        .as_array()
        .expect("ablation ids should be an array");

    for expected in [
        "thin_reveal",
        "fast_pan",
        "diagonal_reveal",
        "reveal_band",
        "motion_bias_band",
        "layered_slats",
        "noisy_reprojection",
        "heuristic_friendly_pan",
        "contrast_pulse",
        "stability_holdout",
    ] {
        assert!(
            scenario_ids
                .iter()
                .any(|value| value.as_str() == Some(expected)),
            "missing scenario {expected}"
        );
    }

    for expected in [
        "fixed_alpha",
        "residual_threshold",
        "neighborhood_clamp",
        "depth_normal_reject",
        "reactive_mask",
        "strong_heuristic",
    ] {
        assert!(
            baseline_ids
                .iter()
                .any(|value| value.as_str() == Some(expected)),
            "missing baseline {expected}"
        );
    }

    for expected in [
        "dsfb_synthetic_visibility",
        "dsfb_host_realistic",
        "dsfb_no_visibility",
        "dsfb_no_thin",
        "dsfb_no_motion_edge",
        "dsfb_no_grammar",
        "dsfb_residual_only",
        "dsfb_trust_no_alpha",
    ] {
        assert!(
            ablation_ids
                .iter()
                .any(|value| value.as_str() == Some(expected)),
            "missing ablation {expected}"
        );
    }

    assert!(
        summary["primary_behavioral_result"]
            .as_str()
            .expect("primary result should exist")
            .contains("host-realistic DSFB"),
        "primary result should explicitly mention host-realistic DSFB"
    );
    assert!(
        !summary["mixed_or_neutral_scenarios"]
            .as_array()
            .expect("mixed / neutral scenarios should be an array")
            .is_empty(),
        "at least one neutral or mixed scenario must be surfaced"
    );
    assert!(
        summary["scenario_ids"]
            .as_array()
            .expect("scenario ids should be an array")
            .len()
            >= 8,
        "the realism-expanded suite should include the broader scenario taxonomy"
    );

    let canonical = metrics["scenarios"]
        .as_array()
        .expect("scenarios should be an array")
        .iter()
        .find(|scenario| scenario["scenario_id"].as_str() == Some("thin_reveal"))
        .expect("canonical scenario should exist");
    let runs = canonical["runs"]
        .as_array()
        .expect("runs should be an array");
    let fixed = runs
        .iter()
        .find(|run| run["summary"]["run_id"].as_str() == Some("fixed_alpha"))
        .expect("fixed alpha should exist");
    let host = runs
        .iter()
        .find(|run| run["summary"]["run_id"].as_str() == Some("dsfb_host_realistic"))
        .expect("host-realistic should exist");
    let strong = runs
        .iter()
        .find(|run| run["summary"]["run_id"].as_str() == Some("strong_heuristic"))
        .expect("strong heuristic should exist");

    assert!(
        host["summary"]["cumulative_roi_mae"]
            .as_f64()
            .expect("host cumulative ROI MAE should exist")
            < fixed["summary"]["cumulative_roi_mae"]
                .as_f64()
                .expect("fixed cumulative ROI MAE should exist")
    );
    assert!(
        host["summary"]["cumulative_roi_mae"]
            .as_f64()
            .expect("host cumulative ROI MAE should exist")
            < strong["summary"]["cumulative_roi_mae"]
                .as_f64()
                .expect("strong cumulative ROI MAE should exist"),
        "host-realistic should remain competitive against the strongest heuristic on the canonical case"
    );
}

#[test]
fn demo_b_metrics_preserve_budget_and_surface_nontrivial_competition() {
    let metrics = demo_b_metrics();
    let summary = &metrics["summary"];

    assert!(
        summary["imported_trust_beats_uniform_scenarios"]
            .as_u64()
            .expect("imported-trust win count should exist")
            >= 1
    );
    assert!(!summary["neutral_or_mixed_scenarios"]
        .as_array()
        .expect("neutral or mixed scenarios should be an array")
        .is_empty());

    for scenario in metrics["scenarios"]
        .as_array()
        .expect("Demo B scenarios should be an array")
    {
        let policies = scenario["policies"]
            .as_array()
            .expect("Demo B policies should be an array");
        let expected_total = policies[0]["total_samples"]
            .as_u64()
            .expect("policy total sample count should exist");
        for policy in policies {
            assert_eq!(
                policy["total_samples"]
                    .as_u64()
                    .expect("policy total sample count should exist"),
                expected_total,
                "all policies must preserve the same total budget"
            );
        }
    }

    let canonical = metrics["scenarios"]
        .as_array()
        .expect("Demo B scenarios should be an array")
        .iter()
        .find(|scenario| scenario["scenario_id"].as_str() == Some("thin_reveal"))
        .expect("canonical Demo B scenario should exist");
    let policies = canonical["policies"]
        .as_array()
        .expect("canonical policies should be an array");
    let uniform = policies
        .iter()
        .find(|policy| policy["policy_id"].as_str() == Some("uniform"))
        .expect("uniform policy should exist");
    let imported = policies
        .iter()
        .find(|policy| policy["policy_id"].as_str() == Some("imported_trust"))
        .expect("imported trust policy should exist");
    let combined = policies
        .iter()
        .find(|policy| policy["policy_id"].as_str() == Some("combined_heuristic"))
        .expect("combined heuristic should exist");

    assert!(
        imported["roi_mae"]
            .as_f64()
            .expect("imported trust ROI MAE should exist")
            < uniform["roi_mae"]
                .as_f64()
                .expect("uniform ROI MAE should exist")
    );
    assert!(
        imported["roi_mae"]
            .as_f64()
            .expect("imported trust ROI MAE should exist")
            < combined["roi_mae"]
                .as_f64()
                .expect("combined heuristic ROI MAE should exist"),
        "imported trust should remain competitive with the stronger heuristic allocator on the canonical case"
    );
}

#[test]
fn reports_and_docs_contain_required_honesty_and_blocker_language() {
    let output_dir = full_suite_dir();

    let report = read(output_dir.join("report.md"));
    let reviewer_summary = read(output_dir.join("reviewer_summary.md"));
    let blocker_report = read(output_dir.join("check_signing_blockers.md"));
    let demo_b_report = read(output_dir.join("demo_b_decision_report.md"));
    let gpu_report = read(output_dir.join("gpu_execution_report.md"));
    let trust_mode_report = read(output_dir.join("trust_mode_report.md"));
    let external_replay_report = read(output_dir.join("external_replay_report.md"));
    let external_report = read(output_dir.join("external_handoff_report.md"));
    let realism_report = read(output_dir.join("realism_suite_report.md"));
    let realism_bridge_report = read(output_dir.join("realism_bridge_report.md"));
    let competitive_report = read(output_dir.join("competitive_baseline_analysis.md"));
    let non_roi_report = read(output_dir.join("non_roi_penalty_report.md"));
    let product_positioning_report = read(output_dir.join("product_positioning_report.md"));
    let operating_band_report = read(output_dir.join("operating_band_report.md"));
    let demo_b_competitive_report =
        read(output_dir.join("demo_b_competitive_baselines_report.md"));
    let readiness_report = read(output_dir.join("check_signing_readiness.md"));
    let readme = read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"));
    let integration_doc = read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("integration_surface.md"),
    );
    let cost_doc = read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("cost_model.md"),
    );

    for text in [
        &report,
        &demo_b_report,
        &gpu_report,
        &trust_mode_report,
        &external_replay_report,
        &external_report,
        &realism_report,
        &realism_bridge_report,
        &competitive_report,
        &non_roi_report,
        &product_positioning_report,
        &operating_band_report,
        &demo_b_competitive_report,
        &readiness_report,
        &readme,
        &integration_doc,
        &cost_doc,
    ] {
        assert!(text.contains(EXPERIMENT_SENTENCE));
    }
    assert!(report.contains("## Remaining Blockers"));
    assert!(report.contains("## What Is Not Proven"));
    assert!(demo_b_report.contains("## What is not proven"));
    assert!(gpu_report.contains("Actual GPU timing measured"));
    assert!(gpu_report.contains("## Remaining Blockers"));
    assert!(trust_mode_report.contains("near-binary") || trust_mode_report.contains("WeaklyGraded"));
    assert!(external_replay_report.contains("external-capable"));
    assert!(external_report.contains("external-capable"));
    assert!(realism_report.contains("realism-stress"));
    assert!(realism_bridge_report.contains("Region-ROI evidence"));
    assert!(competitive_report.contains("targeted supervisory overlay"));
    assert!(non_roi_report.contains("non-ROI penalty"));
    assert!(product_positioning_report.contains("instability-focused specialist"));
    assert!(operating_band_report.contains("moderately sensitive"));
    assert!(demo_b_competitive_report.contains("variance-guided"));
    assert!(readiness_report.contains("blocked pending external evidence"));
    assert!(blocker_report.contains("## Remaining"));
    assert!(reviewer_summary.contains("What is still blocked"));
    assert!(readme.contains("## Strongest Current Evidence"));
    assert!(readme.contains("## Biggest Remaining Blockers"));
    assert!(readme.contains("run-external-replay"));
    assert!(readme.contains("run-realism-bridge"));
    assert!(readme.contains("validate-final"));
    assert!(cost_doc.contains(COST_SENTENCE));
    assert!(cost_doc.contains(COMPATIBILITY_SENTENCE));
}
